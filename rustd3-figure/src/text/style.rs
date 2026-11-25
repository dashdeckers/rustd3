//! [`Style`]: how a label is set.

use rustd3_core::color::Rgb;

use crate::text::anchor;

/// How a label is set: font family, size, weight, color, and how its lines
/// align.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The font family, as its name table gives it (`"DejaVu Sans Mono"`);
    /// `None` for the fonts' fallback order, which starts with DejaVu Sans
    /// when the fonts are the bundled ones.
    pub font: Option<&'static str>,
    /// The family of `raw` markup (typst's monospaced text); `None` for
    /// typst's own, DejaVu Sans Mono, bundled with the `fonts` feature.
    pub mono: Option<&'static str>,
    /// Font size in px (typst's `pt` used as a plain unit).
    pub size: f64,
    /// Whether the whole label is bold.
    pub bold: bool,
    /// The color of text no span colors.
    pub fill: Rgb,
    /// How the lines of a multi-line label align against each other.
    pub align: anchor::Horizontal,
}
