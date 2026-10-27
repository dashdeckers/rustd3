//! [`Outlines`]: glyph outlines read from a list's fonts with ttf-parser.

use std::collections::HashMap;

use kurbo::{Affine, BezPath, Rect, Shape};
use peniko::FontData;

use crate::list::{Glyph, GlyphRun};

/// Glyph outlines from a list's fonts, cached per font file, face and
/// glyph, in em units with y down (1 em = 1.0). Keys are the font's
/// [`Blob`](peniko::Blob) id, not its index in a list, so one cache serves
/// any number of lists.
#[derive(Debug, Default)]
pub struct Outlines {
    cache: HashMap<(u64, u32, u32), Option<Outline>>,
}

/// One glyph's outline and its bounds, em units, y down.
#[derive(Debug)]
struct Outline {
    path: BezPath,
    bounds: Rect,
}

/// Records ttf-parser's outline in font units.
struct Pen(BezPath);

impl ttf_parser::OutlineBuilder for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((f64::from(x), f64::from(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((f64::from(x), f64::from(y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0
            .quad_to((f64::from(x1), f64::from(y1)), (f64::from(x), f64::from(y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.curve_to(
            (f64::from(x1), f64::from(y1)),
            (f64::from(x2), f64::from(y2)),
            (f64::from(x), f64::from(y)),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

impl Outlines {
    /// Glyph `id` of `font`, 1 em = 1.0, y down; `None` for a glyph with
    /// no outline (a space), an id past 65,535 or a font ttf-parser cannot
    /// read.
    pub fn get(&mut self, font: &FontData, id: u32) -> Option<&BezPath> {
        self.outline(font, id).map(|o| &o.path)
    }

    /// The bounds of [`get`](Outlines::get)'s outline.
    pub(crate) fn bounds(&mut self, font: &FontData, id: u32) -> Option<Rect> {
        self.outline(font, id).map(|o| o.bounds)
    }

    fn outline(&mut self, font: &FontData, id: u32) -> Option<&Outline> {
        self.cache
            .entry((font.data.id(), font.index, id))
            .or_insert_with(|| read(font, id))
            .as_ref()
    }
}

/// Where a glyph's em-unit outline lands in the run's group:
/// `transform × translate(x, y) × scale(size)`.
pub(crate) fn place(run: &GlyphRun, glyph: &Glyph) -> Affine {
    run.transform
        * Affine::translate((f64::from(glyph.x), f64::from(glyph.y)))
        * Affine::scale(f64::from(run.size))
}

/// The outline of glyph `id`, read from the font file.
fn read(font: &FontData, id: u32) -> Option<Outline> {
    let face = ttf_parser::Face::parse(font.data.data(), font.index).ok()?;
    let upem = f64::from(face.units_per_em());
    let mut pen = Pen(BezPath::new());
    face.outline_glyph(ttf_parser::GlyphId(u16::try_from(id).ok()?), &mut pen)?;
    let path = Affine::scale_non_uniform(1.0 / upem, -1.0 / upem) * pen.0;
    let bounds = path.bounding_box();
    Some(Outline { path, bounds })
}
