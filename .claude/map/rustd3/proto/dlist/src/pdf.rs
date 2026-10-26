//! The list written as PDF with krilla. Glyph runs keep
//! their source text, so the PDF has real, selectable
//! text with subset fonts embedded.

use figmodel::plot::draw::{Command, List};
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Transform};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{
    Fill, FillRule, LineCap, LineJoin, Stroke, StrokeDash,
};
use krilla::text::{Font, GlyphId, KrillaGlyph};
use krilla::{Data, Document};
use kurbo::{Affine, BezPath, PathEl};
use peniko::Brush;
use std::sync::Arc;

fn path(p: &BezPath) -> Option<krilla::geom::Path> {
    let mut b = PathBuilder::new();
    for e in p.elements() {
        match *e {
            PathEl::MoveTo(a) => b.move_to(a.x as f32, a.y as f32),
            PathEl::LineTo(a) => b.line_to(a.x as f32, a.y as f32),
            PathEl::QuadTo(a, c) => b.quad_to(
                a.x as f32, a.y as f32, c.x as f32, c.y as f32,
            ),
            PathEl::CurveTo(a, c, d) => b.cubic_to(
                a.x as f32, a.y as f32, c.x as f32, c.y as f32,
                d.x as f32, d.y as f32,
            ),
            PathEl::ClosePath => b.close(),
        }
    }
    b.finish()
}

fn xf(t: Affine) -> Transform {
    let [a, b, c, d, e, f] = t.as_coeffs();
    Transform::from_row(
        a as f32, b as f32, c as f32, d as f32, e as f32, f as f32,
    )
}

fn color(b: &Brush) -> (rgb::Color, NormalizedF32) {
    let Brush::Solid(c) = b else {
        return (rgb::Color::new(128, 128, 128), NormalizedF32::ONE);
    };
    let [r, g, bb, _] = c.to_rgba8().to_u8_array();
    let a = NormalizedF32::new(c.components[3]).unwrap();
    (rgb::Color::new(r, g, bb), a)
}

fn fill(b: &Brush) -> Fill {
    let (c, opacity) = color(b);
    Fill { paint: c.into(), opacity, rule: FillRule::NonZero }
}

/// `outlined` draws glyphs as paths, as a backend that
/// cannot embed fonts would.
pub fn write(l: &List, outlined: bool) -> Vec<u8> {
    let [w, h] = l.size;
    let fonts: Vec<Font> = l
        .fonts
        .iter()
        .map(|f| {
            let bytes: Arc<dyn AsRef<[u8]> + Send + Sync> =
                Arc::new(f.data.data().to_vec());
            Font::new(Data::from(bytes), f.index).unwrap()
        })
        .collect();
    let mut doc = Document::new();
    let mut page = doc.start_page_with(
        PageSettings::from_wh(w as f32, h as f32).unwrap(),
    );
    let mut s = page.surface();
    let mut open = Vec::new();
    for c in &l.commands {
        match c {
            Command::Fill { path: p, paint } => {
                let Some(p) = path(p) else { continue };
                s.set_stroke(None);
                s.set_fill(Some(fill(paint)));
                s.draw_path(&p);
            }
            Command::Stroke { path: p, style, paint } => {
                let Some(p) = path(p) else { continue };
                let (c, opacity) = color(paint);
                let dash = (!style.dash_pattern.is_empty()).then(|| {
                    StrokeDash {
                        array: style
                            .dash_pattern
                            .iter()
                            .map(|v| *v as f32)
                            .collect(),
                        offset: style.dash_offset as f32,
                    }
                });
                s.set_fill(None);
                s.set_stroke(Some(Stroke {
                    paint: c.into(),
                    width: style.width as f32,
                    miter_limit: style.miter_limit as f32,
                    line_cap: match style.start_cap {
                        kurbo::Cap::Butt => LineCap::Butt,
                        kurbo::Cap::Round => LineCap::Round,
                        kurbo::Cap::Square => LineCap::Square,
                    },
                    line_join: match style.join {
                        kurbo::Join::Miter => LineJoin::Miter,
                        kurbo::Join::Round => LineJoin::Round,
                        kurbo::Join::Bevel => LineJoin::Bevel,
                    },
                    opacity,
                    dash,
                    ..Default::default()
                }));
                s.draw_path(&p);
            }
            Command::Markers { shape, at, paint } => {
                let Some(p) = path(shape) else { continue };
                s.set_stroke(None);
                s.set_fill(Some(fill(paint)));
                for q in at {
                    s.push_transform(&xf(Affine::translate(
                        q.to_vec2(),
                    )));
                    s.draw_path(&p);
                    s.pop();
                }
            }
            Command::Glyphs(r) => {
                let mut pen = 0.0;
                let size = r.size;
                let glyphs: Vec<KrillaGlyph> = r
                    .glyphs
                    .iter()
                    .map(|g| {
                        let q = KrillaGlyph::new(
                            GlyphId::new(g.id),
                            g.advance / size,
                            (g.x - pen) / size,
                            -g.y / size,
                            0.0,
                            g.text.0 as usize..g.text.1 as usize,
                            None,
                        );
                        pen += g.advance;
                        q
                    })
                    .collect();
                s.push_transform(&xf(r.transform));
                s.set_stroke(None);
                s.set_fill(Some(fill(&Brush::Solid(r.fill))));
                s.draw_glyphs(
                    Point::from_xy(0.0, 0.0),
                    &glyphs,
                    fonts[r.font].clone(),
                    &r.text,
                    size,
                    outlined,
                );
                s.pop();
            }
            Command::Label(_) => panic!("shape labels first"),
            Command::Push { transform, clip } => {
                s.push_transform(&xf(*transform));
                let mut n = 1;
                if let Some(p) = clip.as_ref().and_then(path) {
                    s.push_clip_path(&p, &FillRule::NonZero);
                    n = 2;
                }
                open.push(n);
            }
            Command::Pop => {
                for _ in 0..open.pop().unwrap_or(0) {
                    s.pop();
                }
            }
        }
    }
    s.finish();
    page.finish();
    doc.finish().unwrap()
}
