#![doc = include_str!("../README.md")]

pub use rustd3_core::{
    anim, array, axis, color, ease, format, interact, interpolate, join, locale, path, scale,
    scheme, series, shape, ticks, time,
};

pub use rustd3_core::jiff;

pub use rustd3_core::fmt;

#[cfg(feature = "figure")]
pub use rustd3_figure as figure;
