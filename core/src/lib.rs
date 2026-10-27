//! The d3 port: scales, ticks, axes, shapes and `PathContext`, number and
//! time formats, locales, color and schemes, interpolation and easing,
//! array statistics, recordings of sampled data, and zoom, brush, drag and
//! animation as plain values and state machines, renderer-agnostic and
//! graded by d3's own tests.

pub mod axis;
pub mod color;
pub mod ease;
pub mod format;
pub mod interpolate;
pub mod locale;
pub mod path;
pub mod scale;
pub mod ticks;

mod js;
