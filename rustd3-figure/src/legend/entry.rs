//! [`Entry`]: one entry of a legend of keys, as its guide gives it.

use rustd3_core::color::Rgb;

use crate::hatch::Hatch;
use crate::legend::Swatch;

/// One entry of a legend of keys.
pub(crate) struct Entry {
    /// Its swatch's color; `None` when the palette has none, which draws in
    /// the theme's gray.
    pub(crate) color: Option<Rgb>,
    /// Its label.
    pub(crate) label: String,
    /// Its swatch.
    pub(crate) glyph: Swatch,
    /// The hatch of the marks filled with its color.
    pub(crate) hatch: Option<Hatch>,
    /// For a dot, its diameter in px; `None` for the theme's marker size.
    pub(crate) size: Option<f64>,
}
