//! rustd3 speaks d3's vocabulary in Rust: scales, ticks, shapes, number
//! and time formats, color, zoom and brush from `rustd3-core`, and, behind
//! the `figure` feature, figures with typst text, legends and layout that
//! render to SVG, PNG, PDF and GIF from `rustd3-figure`; this facade
//! re-exports both, so an application adds one dependency.

pub use rustd3_core::{
    anim, array, axis, color, ease, format, interact, interpolate, join, locale, path, scale,
    scheme, series, shape, ticks, time,
};

pub use rustd3_core::jiff;

pub use rustd3_core::fmt;

#[cfg(feature = "figure")]
pub use rustd3_figure as figure;
