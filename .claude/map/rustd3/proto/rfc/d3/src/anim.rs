//! Transitions as values sampled at an `Instant`, driven by the host's frame clock.

use crate::interpolate::Interpolate;
use crate::join::join;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// An easing curve; any of d3-ease's functions fits.
pub type Easing = fn(f64) -> f64;

/// A few of d3-ease's curves as plain functions.
pub mod easing {
    use d3_core::ease::{CubicInOut, Ease, Linear};
    pub fn linear(t: f64) -> f64 {
        Linear.ease(t)
    }
    pub fn cubic_in_out(t: f64) -> f64 {
        CubicInOut.ease(t)
    }
}

/// How a change animates: d3's `transition.duration(...).delay(...).ease(...)`.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    pub duration: Duration,
    pub delay: Duration,
    /// Extra delay per item in display order, for staggered transitions.
    pub stagger: Duration,
    pub ease: Easing,
}

impl Default for Timing {
    /// d3's defaults: 250 ms, no delay, cubic in-out.
    fn default() -> Self {
        Self { duration: Duration::from_millis(250), delay: Duration::ZERO, stagger: Duration::ZERO, ease: easing::cubic_in_out }
    }
}

/// One animated value. Retargeting starts from wherever it is now, as an interrupted d3 transition does.
#[derive(Debug, Clone)]
pub struct Animated<T> {
    from: T,
    to: T,
    start: Instant,
    duration: Duration,
    ease: Easing,
}

impl<T: Interpolate> Animated<T> {
    pub fn new(value: T, now: Instant) -> Self {
        Self { from: value.clone(), to: value, start: now, duration: Duration::ZERO, ease: easing::linear }
    }

    pub fn go(&mut self, target: T, now: Instant, timing: Timing, delay: Duration) {
        self.from = self.value(now);
        self.to = target;
        self.start = now + timing.delay + delay;
        self.duration = timing.duration;
        self.ease = timing.ease;
    }

    pub fn value(&self, now: Instant) -> T {
        if self.duration.is_zero() || now >= self.start + self.duration {
            return self.to.clone();
        }
        let t = now.saturating_duration_since(self.start).as_secs_f64() / self.duration.as_secs_f64();
        self.from.lerp(&self.to, (self.ease)(t))
    }

    pub fn target(&self) -> &T {
        &self.to
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        now < self.start + self.duration
    }
}

/// Where a mark is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Present,
    Exiting,
}

/// Keyed marks with enter, update and exit transitions: the general update pattern as state.
#[derive(Debug, Clone)]
pub struct Marks<K, T> {
    items: Vec<(K, Animated<T>, Phase)>,
}

impl<K, T> Default for Marks<K, T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<K: Eq + Hash + Clone, T: Interpolate> Marks<K, T> {
    /// Reconciles with the next data. `enter` gives a new mark's starting state from its target;
    /// `exit` gives a leaving mark's final state from where it is now.
    pub fn update(
        &mut self,
        targets: impl IntoIterator<Item = (K, T)>,
        now: Instant,
        timing: Timing,
        enter: impl Fn(&T) -> T,
        exit: impl Fn(&T) -> T,
    ) {
        let targets: Vec<(K, T)> = targets.into_iter().collect();
        let plan = join(
            self.items.iter().filter(|m| m.2 == Phase::Present).map(|m| m.0.clone()),
            targets.iter().map(|t| t.0.clone()),
        );
        for (key, mark, phase) in &mut self.items {
            if plan.exit.contains(key) {
                let end = exit(&mark.value(now));
                mark.go(end, now, timing, Duration::ZERO);
                *phase = Phase::Exiting;
            }
        }
        for (i, (key, target)) in targets.into_iter().enumerate() {
            let delay = timing.stagger * i as u32;
            match self.items.iter_mut().find(|m| m.0 == key) {
                Some((_, mark, phase)) => {
                    mark.go(target, now, timing, delay);
                    *phase = Phase::Present;
                }
                None => {
                    let mut mark = Animated::new(enter(&target), now);
                    mark.go(target, now, timing, delay);
                    self.items.push((key, mark, Phase::Present));
                }
            }
        }
    }

    /// Every mark's value at `now`, exiting ones included until they finish.
    pub fn values(&self, now: Instant) -> impl Iterator<Item = (&K, T, Phase)> + '_ {
        self.items.iter().map(move |(k, m, p)| (k, m.value(now), *p))
    }

    /// Drops exiting marks whose transition has finished.
    pub fn prune(&mut self, now: Instant) {
        self.items.retain(|(_, m, p)| *p == Phase::Present || m.is_animating(now));
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.items.iter().any(|(_, m, _)| m.is_animating(now))
    }
}
