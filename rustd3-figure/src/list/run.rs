//! [`GlyphRun`] and [`Glyph`]: shaped text.

use std::ops::Range;
use std::sync::Arc;

use kurbo::Affine;
use peniko::Brush;

/// Shaped text in one font, size and paint: what layout measured, placed.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    /// Index into [`List::fonts`](crate::list::List::fonts).
    pub font: usize,
    /// Font size in px of the run's space.
    pub size: f32,
    /// Origin, rotation and anchor: the run's space in its group's.
    pub transform: Affine,
    /// The glyphs' paint, positioned in the group's space.
    pub fill: Brush,
    /// The glyphs, in order.
    pub glyphs: Vec<Glyph>,
    /// Source text, for PDF text and SVG `<text>`.
    pub text: Arc<str>,
}

/// One glyph of a run, at its pen position in the run's space (y down).
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    /// The glyph's id in its font.
    pub id: u32,
    /// Pen position, px.
    pub x: f32,
    /// Baseline position, px.
    pub y: f32,
    /// Advance, px.
    pub advance: f32,
    /// Bytes of the run's text this glyph shows.
    pub text: Range<u32>,
}
