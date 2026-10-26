//! The list replayed into any anyrender scene: vello
//! (GPU), vello_cpu, vello_hybrid or skia, one adapter.

use anyrender::{Glyph, PaintScene};
use figmodel::plot::draw::{Command, List};
use kurbo::Affine;
use peniko::{Brush, Color, Fill};

fn solid(b: &Brush) -> Color {
    match b {
        Brush::Solid(c) => *c,
        _ => Color::from_rgb8(128, 128, 128),
    }
}

pub fn replay(l: &List, s: &mut impl PaintScene, base: Affine) {
    let mut t = vec![base];
    let mut clips = vec![false];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        match c {
            Command::Fill { path, paint } => {
                s.fill(Fill::NonZero, cur, solid(paint), None, path)
            }
            Command::Stroke { path, style, paint } => {
                s.stroke(style, cur, solid(paint), None, path)
            }
            Command::Markers { shape, at, paint } => {
                for q in at {
                    let m = cur * Affine::translate(q.to_vec2());
                    s.fill(Fill::NonZero, m, solid(paint), None, shape);
                }
            }
            Command::Glyphs(r) => s.draw_glyphs(
                &l.fonts[r.font],
                r.size,
                false,
                &[],
                kurbo::Vec2::ZERO,
                Fill::NonZero,
                r.fill,
                1.0,
                cur * r.transform,
                None,
                r.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                }),
            ),
            Command::Label(_) => panic!("shape labels first"),
            Command::Push { transform, clip } => {
                let next = cur * *transform;
                t.push(next);
                if let Some(p) = clip {
                    s.push_clip_layer(Fill::NonZero, next, p);
                }
                clips.push(clip.is_some());
            }
            Command::Pop => {
                t.pop();
                if clips.pop() == Some(true) {
                    s.pop_layer();
                }
            }
        }
    }
}
