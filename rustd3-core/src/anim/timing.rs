//! [`Timing`]: how a change animates.

use std::time::Duration;

use crate::ease::Ease;

/// How a change animates: d3's `transition.duration(…).delay(…).ease(…)`,
/// plus a per-item `stagger` for d3's `delay((d, i) => i * stagger)`.
///
/// Plain data. The default is d3's: 250 ms, no delay, no stagger, cubic
/// in-out.
///
/// ```
/// use std::time::Duration;
///
/// use rustd3_core::anim::Timing;
/// use rustd3_core::ease::{Ease, Mode};
///
/// let timing = Timing {
///     duration: Duration::from_millis(750),
///     stagger: Duration::from_millis(20),
///     // no delay, cubic in-out: d3's defaults
///     ..Timing::default()
/// };
/// assert_eq!(timing.delay, Duration::ZERO);
/// assert_eq!(timing.ease, Ease::Cubic(Mode::InOut));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    /// How long a change takes once it starts.
    pub duration: Duration,
    /// How long a change waits before it starts.
    pub delay: Duration,
    /// The extra delay per item, by its position among the targets of
    /// [`Marks::update`](crate::anim::Marks::update).
    pub stagger: Duration,
    /// The easing curve over the duration.
    pub ease: Ease,
}

impl Default for Timing {
    /// d3's transition defaults: 250 ms, no delay, cubic in-out; no
    /// stagger.
    fn default() -> Self {
        Timing {
            duration: Duration::from_millis(250),
            delay: Duration::ZERO,
            stagger: Duration::ZERO,
            ease: Ease::default(),
        }
    }
}
