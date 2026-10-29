//! Examples of rustd3 at both levels: a multi-channel trace, a scatter,
//! the four-wheel figure, the two core-level use cases and the chart
//! presets, shared with the benchmark. The use cases draw in iced and
//! need the `widget` feature (on by default); the figures do not.

#[cfg(feature = "widget")]
pub mod focus_context;
pub mod presets;
pub mod scatter;
pub mod trace;
#[cfg(feature = "widget")]
pub mod update_pattern;
pub mod wheels;
