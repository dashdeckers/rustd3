//! Transitions as values sampled at caller-supplied `Instant`s:
//! [`Timing`] (d3's duration, delay and ease, plus a stagger),
//! [`Animated`] (one value) and [`Marks`] (keyed marks through enter,
//! update and exit, d3's general update pattern). The host drives the
//! clock and passes its instants in; nothing here reads one.
//!
//! [`Instant`] is `web_time`'s: `std::time::Instant` itself on native
//! targets, and a clock that works in the browser on wasm, where std's
//! panics. iced's `time::Instant` is the same type. Durations are
//! `std::time::Duration`.

mod animated;
mod marks;
mod timing;

pub use animated::Animated;
pub use marks::{Marks, Phase};
pub use timing::Timing;
pub use web_time::Instant;
