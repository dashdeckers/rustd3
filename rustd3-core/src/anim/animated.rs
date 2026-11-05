//! [`Animated`]: one value in transition.

use std::time::Duration;

use crate::anim::{Instant, Timing};
use crate::ease::Ease;
use crate::interpolate::Interpolate;

/// One value in transition, sampled at caller-supplied `Instant`s: where it
/// came from, where it is going, when and how. Retargeting starts from the
/// value it has then, as an interrupted d3 transition does.
///
/// ```
/// use std::time::Duration;
///
/// use rustd3_core::anim::{Animated, Instant, Timing};
/// use rustd3_core::ease::Ease;
///
/// // Any base instant: values depend only on the offsets from it.
/// let t0 = Instant::now();
/// let ms = |n| t0 + Duration::from_millis(n);
/// let linear = Timing {
///     duration: Duration::from_millis(100),
///     ease: Ease::Linear,
///     ..Timing::default()
/// };
/// let mut width = Animated::new(0.0, ms(0));
/// width.go(100.0, ms(0), linear, Duration::ZERO);
/// assert_eq!(width.value(ms(25)), 25.0);
/// // Interrupted at 50 ms, it heads back from where it is.
/// width.go(0.0, ms(50), linear, Duration::ZERO);
/// assert_eq!(width.value(ms(50)), 50.0);
/// assert_eq!(width.value(ms(100)), 25.0);
/// assert!(width.is_animating(ms(100)));
/// assert!(!width.is_animating(ms(150)));
/// assert_eq!(*width.get_target(), 0.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Animated<T> {
    from: T,
    to: T,
    start: Instant,
    duration: Duration,
    ease: Ease,
}

impl<T: Interpolate> Animated<T> {
    /// `value`, at rest from `now`.
    pub fn new(value: T, now: Instant) -> Self {
        Animated {
            from: value.clone(),
            to: value,
            start: now,
            duration: Duration::ZERO,
            ease: Ease::default(),
        }
    }

    /// Heads for `target` from the value at `now`, starting after
    /// `timing.delay` plus `extra_delay` (a stagger) and taking
    /// `timing.duration` along `timing.ease`. If the delayed start
    /// overflows `Instant`, it starts at `now`.
    pub fn go(&mut self, target: T, now: Instant, timing: Timing, extra_delay: Duration) {
        self.from = self.value(now);
        self.to = target;
        let delay = timing.delay.saturating_add(extra_delay);
        self.start = now.checked_add(delay).unwrap_or(now);
        self.duration = timing.duration;
        self.ease = timing.ease;
    }

    /// The value at `now`: where it came from until the start, the eased
    /// blend during the duration, the target after.
    pub fn value(&self, now: Instant) -> T {
        let Some(elapsed) = now.checked_duration_since(self.start) else {
            return self.from.clone();
        };
        if elapsed >= self.duration {
            return self.to.clone();
        }
        let t = elapsed.div_duration_f64(self.duration);
        self.from.lerp(&self.to, self.ease.apply(t))
    }

    /// Where it is going.
    pub fn get_target(&self) -> &T {
        &self.to
    }

    /// Whether it has not reached its target by `now`, either waiting for
    /// its start or moving.
    pub fn is_animating(&self, now: Instant) -> bool {
        now.checked_duration_since(self.start)
            .is_none_or(|elapsed| elapsed < self.duration)
    }
}
