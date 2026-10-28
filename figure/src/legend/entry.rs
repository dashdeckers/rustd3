//! [`Entry`]: one entry of a legend of keys, as its guide gives it.

use rustd3_core::color::Rgb;

use crate::hatch::Hatch;

/// One entry of a legend of keys: its swatch's color (`None` when the
/// palette has none, drawn in the theme's gray), its label, the hatch of
/// the marks filled with it, and, for a dot, its diameter in px (`None`:
/// the theme's marker size).
pub(crate) struct Entry {
    pub(crate) color: Option<Rgb>,
    pub(crate) label: String,
    pub(crate) hatch: Option<Hatch>,
    pub(crate) size: Option<f64>,
}
