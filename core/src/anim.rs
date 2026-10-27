//! Transitions as values sampled at caller-supplied `Instant`s:
//! [`Timing`] (d3's duration, delay and ease, plus a stagger),
//! [`Animated`] (one value) and [`Marks`] (keyed marks through enter,
//! update and exit, d3's general update pattern). The host drives the
//! clock and passes its instants in; nothing here reads one.

mod animated;
mod marks;
mod timing;

pub use animated::Animated;
pub use marks::{Marks, Phase};
pub use timing::Timing;
