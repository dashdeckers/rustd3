//! Examples of rustd3 at both levels: a multi-channel trace, a scatter,
//! the four-wheel figure, two core-level apps (focus and context, the
//! update pattern) and the chart presets, shared with the benchmark. The
//! apps draw in iced and
//! need the `widget` feature (on by default); the figures do not.

#[cfg(feature = "widget")]
pub mod focus_context;
pub use benchkit::gallery::{presets, trace, wheels};
pub mod scatter;
#[cfg(feature = "widget")]
pub mod update_pattern;
