//! The list rasterized directly with tiny-skia: the
//! conversions a non-vello backend needs.

use crate::outline::Outlines;
use figmodel::plot::draw::{Command, List};
use kurbo::{Affine, BezPath, PathEl};
use peniko::Brush;
use tiny_skia as ts;

fn path(p: &BezPath) -> Option<ts::Path> {
    let mut b = ts::PathBuilder::new();
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

fn xf(t: Affine) -> ts::Transform {
    let [a, b, c, d, e, f] = t.as_coeffs();
    ts::Transform::from_row(
        a as f32, b as f32, c as f32, d as f32, e as f32, f as f32,
    )
}

fn paint(b: &Brush) -> ts::Paint<'static> {
    let mut p = ts::Paint::default();
    p.anti_alias = true;
    if let Brush::Solid(c) = b {
        let [r, g, bb, a] = c.components;
        p.set_color(ts::Color::from_rgba(r, g, bb, a).unwrap());
    }
    p
}

pub fn render(
    l: &List,
    scale: f64,
    o: &mut Outlines,
) -> ts::Pixmap {
    let [w, h] = l.size;
    let (pw, ph) = ((w * scale) as u32, (h * scale) as u32);
    let mut px = ts::Pixmap::new(pw, ph).unwrap();
    let mut t = vec![Affine::scale(scale)];
    let mut masks: Vec<Option<ts::Mask>> = vec![None];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        let mask = masks.last().unwrap().as_ref();
        match c {
            Command::Fill { path: p, paint: b } => {
                if let Some(p) = path(p) {
                    px.fill_path(
                        &p,
                        &paint(b),
                        ts::FillRule::Winding,
                        xf(cur),
                        mask,
                    );
                }
            }
            Command::Stroke { path: p, style, paint: b } => {
                let Some(p) = path(p) else { continue };
                let mut s = ts::Stroke::default();
                s.width = style.width as f32;
                s.miter_limit = style.miter_limit as f32;
                s.line_cap = match style.start_cap {
                    kurbo::Cap::Butt => ts::LineCap::Butt,
                    kurbo::Cap::Round => ts::LineCap::Round,
                    kurbo::Cap::Square => ts::LineCap::Square,
                };
                s.line_join = match style.join {
                    kurbo::Join::Miter => ts::LineJoin::Miter,
                    kurbo::Join::Round => ts::LineJoin::Round,
                    kurbo::Join::Bevel => ts::LineJoin::Bevel,
                };
                if !style.dash_pattern.is_empty() {
                    let d = style
                        .dash_pattern
                        .iter()
                        .map(|v| *v as f32)
                        .collect();
                    s.dash = ts::StrokeDash::new(
                        d,
                        style.dash_offset as f32,
                    );
                }
                px.stroke_path(&p, &paint(b), &s, xf(cur), mask);
            }
            Command::Markers { shape, at, paint: b } => {
                let Some(p) = path(shape) else { continue };
                let pt = paint(b);
                for q in at {
                    let m = cur * Affine::translate(q.to_vec2());
                    px.fill_path(
                        &p,
                        &pt,
                        ts::FillRule::Winding,
                        xf(m),
                        mask,
                    );
                }
            }
            Command::Glyphs(r) => {
                let pt = paint(&Brush::Solid(r.fill));
                for (i, g) in r.glyphs.iter().enumerate() {
                    let Some(gp) = o.get(&l.fonts, r.font, g.id)
                    else {
                        continue;
                    };
                    let Some(gp) = path(gp) else { continue };
                    let m = cur * Outlines::place(r, i);
                    px.fill_path(
                        &gp,
                        &pt,
                        ts::FillRule::Winding,
                        xf(m),
                        mask,
                    );
                }
            }
            Command::Label(_) => panic!("shape labels first"),
            Command::Push { transform, clip } => {
                let next = cur * *transform;
                t.push(next);
                let m = match clip.as_ref().and_then(path) {
                    Some(p) => {
                        let mut m = match mask {
                            Some(m) => m.clone(),
                            None => {
                                let mut m =
                                    ts::Mask::new(pw, ph).unwrap();
                                m.fill_path(
                                    &ts::PathBuilder::from_rect(
                                        ts::Rect::from_xywh(
                                            0.0, 0.0, pw as f32,
                                            ph as f32,
                                        )
                                        .unwrap(),
                                    ),
                                    ts::FillRule::Winding,
                                    false,
                                    ts::Transform::identity(),
                                );
                                m
                            }
                        };
                        m.intersect_path(
                            &p,
                            ts::FillRule::Winding,
                            true,
                            xf(next),
                        );
                        Some(m)
                    }
                    None => mask.cloned(),
                };
                masks.push(m);
            }
            Command::Pop => {
                t.pop();
                masks.pop();
            }
        }
    }
    px
}
