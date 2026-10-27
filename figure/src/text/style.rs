//! [`Style`]: how a label is set.

use rustd3_core::color::Rgb;

use crate::text::anchor;

/// How a label is set: size, weight, color, and how its lines align.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Font size in px (typst's `pt` used as a plain unit).
    pub size: f64,
    /// Bold weight for the whole label.
    pub bold: bool,
    /// The color of text no span colors.
    pub fill: Rgb,
    /// How the lines of a multi-line label align against each other.
    pub align: anchor::Horizontal,
}
