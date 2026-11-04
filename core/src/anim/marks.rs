//! [`Marks`]: keyed marks through enter, update and exit, and the
//! [`Phase`] each is in.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::time::Duration;

use crate::anim::{Animated, Instant, Timing};
use crate::interpolate::Interpolate;
use crate::join;

/// Where a mark is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// In the data: entering, updating or at rest.
    Present,
    /// Out of the data: animating to its exit state until
    /// [`prune`](Marks::prune) drops it.
    Exiting,
}

/// Keyed marks with enter, update and exit transitions: d3's general
/// update pattern as state the application owns, one animated value per
/// key. [`update`](Marks::update) plans with [`join::join`];
/// [`values`](Marks::values) samples every mark at an `Instant`, exiting
/// ones included until they finish and are pruned.
///
/// The update pattern (100 ms transitions, 20 ms stagger, cubic in-out;
/// marks enter from 0 and exit to 0): new data at 60 ms, mid-flight, sends
/// `a` out from where it is, retargets `b` without a jump and enters `c`
/// one stagger step later.
///
/// ```
/// use std::time::Duration;
///
/// use rustd3_core::anim::{Instant, Marks, Phase, Timing};
///
/// // Any base instant: values depend only on the offsets from it.
/// let t0 = Instant::now();
/// let ms = |n| t0 + Duration::from_millis(n);
/// let timing = Timing {
///     duration: Duration::from_millis(100),
///     stagger: Duration::from_millis(20),
///     ..Timing::default()
/// };
/// let at = |marks: &Marks<&'static str, f64>, n| {
///     marks
///         .values(ms(n))
///         .map(|(key, value, phase)| (*key, (value * 10.0).round() / 10.0, phase))
///         .collect::<Vec<_>>()
/// };
/// let mut marks = Marks::default();
/// marks.update([("a", 10.0), ("b", 20.0)], ms(0), timing, |_| 0.0, |_| 0.0);
/// assert_eq!(at(&marks, 50), [("a", 5.0, Phase::Present), ("b", 2.2, Phase::Present)]);
/// marks.update([("b", 40.0), ("c", 30.0)], ms(60), timing, |_| 0.0, |_| 0.0);
/// assert_eq!(
///     at(&marks, 110),
///     [("a", 3.7, Phase::Exiting), ("b", 22.6, Phase::Present), ("c", 3.2, Phase::Present)]
/// );
/// marks.prune(ms(180));
/// assert_eq!(at(&marks, 180), [("b", 40.0, Phase::Present), ("c", 30.0, Phase::Present)]);
/// assert!(!marks.is_animating(ms(180)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Marks<K, T> {
    items: Vec<Mark<K, T>>,
}

/// One keyed mark.
#[derive(Debug, Clone, PartialEq)]
struct Mark<K, T> {
    key: K,
    value: Animated<T>,
    phase: Phase,
}

impl<K, T> Default for Marks<K, T> {
    /// No marks.
    fn default() -> Self {
        Marks { items: Vec::new() }
    }
}

impl<K: Eq + Hash + Clone, T: Interpolate> Marks<K, T> {
    /// Reconciles the marks with the next data, `targets` in display
    /// order, at `now`.
    ///
    /// A target whose key has no mark enters from `enter(&target)`; a mark
    /// whose key is a target heads for it, an exiting one becoming
    /// [`Phase::Present`] again from where it is; both wait `timing`'s
    /// delay plus its stagger times the target's position. A present mark
    /// whose key is not a target exits towards `exit(&current)` without
    /// stagger. A key given twice keeps one mark, which heads for the later
    /// target.
    pub fn update(
        &mut self,
        targets: impl IntoIterator<Item = (K, T)>,
        now: Instant,
        timing: Timing,
        enter: impl Fn(&T) -> T,
        exit: impl Fn(&T) -> T,
    ) {
        let targets: Vec<(K, T)> = targets.into_iter().collect();
        let present = self
            .items
            .iter()
            .filter(|mark| mark.phase == Phase::Present)
            .map(|mark| mark.key.clone());
        let plan = join::join(present, targets.iter().map(|(key, _)| key.clone()));
        let leaving: HashSet<K> = plan.exit.into_iter().collect();
        let mut index = HashMap::with_capacity(self.items.len() + targets.len());
        for (i, mark) in self.items.iter_mut().enumerate() {
            if leaving.contains(&mark.key) {
                let end = exit(&mark.value.value(now));
                mark.value.go(end, now, timing, Duration::ZERO);
                mark.phase = Phase::Exiting;
            }
            index.insert(mark.key.clone(), i);
        }
        for (position, (key, target)) in targets.into_iter().enumerate() {
            let steps = u32::try_from(position).unwrap_or(u32::MAX);
            let delay = timing.stagger.saturating_mul(steps);
            match index.get(&key) {
                Some(&i) => {
                    let mark = &mut self.items[i];
                    mark.value.go(target, now, timing, delay);
                    mark.phase = Phase::Present;
                }
                None => {
                    let mut value = Animated::new(enter(&target), now);
                    value.go(target, now, timing, delay);
                    index.insert(key.clone(), self.items.len());
                    self.items.push(Mark {
                        key,
                        value,
                        phase: Phase::Present,
                    });
                }
            }
        }
    }
}

impl<K, T: Interpolate> Marks<K, T> {
    /// Every mark's key, value at `now` and phase, in the order the keys
    /// first entered.
    pub fn values(&self, now: Instant) -> impl Iterator<Item = (&K, T, Phase)> {
        self.items
            .iter()
            .map(move |mark| (&mark.key, mark.value.value(now), mark.phase))
    }

    /// Drops the exiting marks that have finished by `now`.
    pub fn prune(&mut self, now: Instant) {
        self.items
            .retain(|mark| mark.phase == Phase::Present || mark.value.is_animating(now));
    }

    /// Some mark has not reached its target by `now`: while this holds, the
    /// application keeps its frame clock running.
    #[must_use]
    pub fn is_animating(&self, now: Instant) -> bool {
        self.items.iter().any(|mark| mark.value.is_animating(now))
    }
}
