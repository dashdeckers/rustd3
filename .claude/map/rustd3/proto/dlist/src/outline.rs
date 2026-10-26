//! Glyph outlines from the list's fonts, cached per
//! (font, glyph) in em units with y pointing down.

use figmodel::plot::draw::GlyphRun;
use kurbo::{Affine, BezPath};
use peniko::FontData;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::collections::HashMap;

#[derive(Default)]
pub struct Outlines {
    cache: HashMap<(usize, u32), Option<BezPath>>,
    pub misses: usize,
}

struct Pen(BezPath);

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to((x as f64, y as f64));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to((x as f64, y as f64));
    }
    fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
        self.0.quad_to((a as f64, b as f64), (x as f64, y as f64));
    }
    fn curve_to(
        &mut self,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        x: f32,
        y: f32,
    ) {
        self.0.curve_to(
            (a as f64, b as f64),
            (c as f64, d as f64),
            (x as f64, y as f64),
        );
    }
    fn close(&mut self) {
        self.0.close_path();
    }
}

impl Outlines {
    /// The outline of one glyph, 1 em = 1.0, y down.
    pub fn get(
        &mut self,
        fonts: &[FontData],
        font: usize,
        id: u32,
    ) -> Option<&BezPath> {
        let misses = &mut self.misses;
        self.cache
            .entry((font, id))
            .or_insert_with(|| {
                *misses += 1;
                let f = &fonts[font];
                let r = FontRef::from_index(f.data.data(), f.index)
                    .ok()?;
                let upem = r
                    .metrics(Size::unscaled(), LocationRef::default())
                    .units_per_em as f64;
                let g = r.outline_glyphs().get(GlyphId::new(id))?;
                let mut pen = Pen(BezPath::new());
                let s = DrawSettings::unhinted(
                    Size::unscaled(),
                    LocationRef::default(),
                );
                g.draw(s, &mut pen).ok()?;
                let flip = Affine::scale_non_uniform(
                    1.0 / upem,
                    -1.0 / upem,
                );
                Some(flip * pen.0)
            })
            .as_ref()
    }

    /// Where glyph `i` of a run sits, in the run's
    /// parent coordinates, scaled from em to px.
    pub fn place(run: &GlyphRun, i: usize) -> Affine {
        let g = &run.glyphs[i];
        run.transform
            * Affine::translate((g.x as f64, g.y as f64))
            * Affine::scale(run.size as f64)
    }

    /// The whole run as one path, in parent coordinates.
    pub fn run_path(
        &mut self,
        fonts: &[FontData],
        run: &GlyphRun,
    ) -> BezPath {
        let mut out = BezPath::new();
        for (i, g) in run.glyphs.iter().enumerate() {
            if let Some(p) = self.get(fonts, run.font, g.id) {
                let t = Self::place(run, i);
                out.extend((t * p.clone()).elements().iter().cloned());
            }
        }
        out
    }
}
