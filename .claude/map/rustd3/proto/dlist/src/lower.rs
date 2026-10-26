//! Capabilities and lowering. A backend says what it
//! draws natively; `lower` rewrites the rest into
//! commands it does draw, so backends stay thin.

use crate::outline::Outlines;
use figmodel::plot::draw::{Command, List};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use kurbo::{Affine, BezPath, PathEl, Rect, Shape, StrokeOpts};
use peniko::Brush;

#[derive(Clone, Copy, Debug)]
pub struct Caps {
    /// Draws glyph runs from font data.
    pub glyphs: bool,
    /// Stamps one shape at many points.
    pub markers: bool,
    /// Clips to any path, not only rectangles.
    pub clip_paths: bool,
}

pub const ALL: Caps =
    Caps { glyphs: true, markers: true, clip_paths: true };

/// iced's canvas: rectangle clips; its own text, so
/// glyph runs arrive as outlines.
/// Whether `lower` rejects whole shapes by bounding
/// box before cutting; off reproduces the first sketch.
pub static PRUNE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

fn caps_prune(_: Caps) -> bool {
    PRUNE.load(std::sync::atomic::Ordering::Relaxed)
}

pub const ICED: Caps =
    Caps { glyphs: false, markers: false, clip_paths: false };

#[derive(Default, Debug)]
pub struct Stats {
    pub glyphs: usize,
    pub markers: usize,
    pub clipped: usize,
    /// Of the clipped shapes: kept whole, dropped,
    /// and cut by i_overlay.
    pub inside: usize,
    pub outside: usize,
    pub cut: usize,
}

/// A clip polygon in figure coordinates, with its
/// bounds for quick rejection.
pub struct Poly {
    pub rings: Vec<Vec<[f64; 2]>>,
    pub bb: Rect,
}

#[derive(Debug, PartialEq)]
enum Where {
    Inside,
    Outside,
    Partial,
}

fn winding(rings: &[Vec<[f64; 2]>], x: f64, y: f64) -> i32 {
    let mut w = 0;
    for r in rings {
        for i in 0..r.len() {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            if a[1] <= y {
                if b[1] > y && cross(a, b, x, y) > 0.0 {
                    w += 1;
                }
            } else if b[1] <= y && cross(a, b, x, y) < 0.0 {
                w -= 1;
            }
        }
    }
    w
}

fn cross(a: [f64; 2], b: [f64; 2], x: f64, y: f64) -> f64 {
    (b[0] - a[0]) * (y - a[1]) - (x - a[0]) * (b[1] - a[1])
}

/// Does segment ab touch the rectangle?
fn hits(a: [f64; 2], b: [f64; 2], r: Rect) -> bool {
    // Liang–Barsky on the segment's parameter.
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [
        (-dx, a[0] - r.x0),
        (dx, r.x1 - a[0]),
        (-dy, a[1] - r.y0),
        (dy, r.y1 - a[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Exact for the shape's bounding box: no clip edge
/// crosses it, so it lies wholly inside or outside.
fn locate(poly: &Poly, bb: Rect) -> Where {
    if bb.x1 < poly.bb.x0 || bb.x0 > poly.bb.x1 || bb.y1 < poly.bb.y0 || bb.y0 > poly.bb.y1 {
        return Where::Outside;
    }
    for r in &poly.rings {
        for i in 0..r.len() {
            if hits(r[i], r[(i + 1) % r.len()], bb) {
                return Where::Partial;
            }
        }
    }
    let c = bb.center();
    if winding(&poly.rings, c.x, c.y) != 0 {
        Where::Inside
    } else {
        Where::Outside
    }
}

/// Axis-aligned rectangle paths, as `push_clip(rect)`
/// emits them.
fn as_rect(p: &BezPath) -> Option<Rect> {
    let els = p.elements();
    let pts: Vec<_> = els
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(a) | PathEl::LineTo(a) => Some(*a),
            PathEl::ClosePath => None,
            _ => Some(kurbo::Point::new(f64::NAN, f64::NAN)),
        })
        .collect();
    if pts.len() != 4 || pts.iter().any(|q| q.x.is_nan()) {
        return None;
    }
    let r = p.bounding_box();
    let on = |q: &kurbo::Point| {
        (q.x == r.x0 || q.x == r.x1) && (q.y == r.y0 || q.y == r.y1)
    };
    pts.iter().all(on).then_some(r)
}

fn contours(p: &BezPath) -> Vec<Vec<[f64; 2]>> {
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    kurbo::flatten(p.iter(), 0.05, |e| match e {
        PathEl::MoveTo(a) => out.push(vec![[a.x, a.y]]),
        PathEl::LineTo(a) => {
            if let Some(c) = out.last_mut() {
                c.push([a.x, a.y]);
            }
        }
        _ => {}
    });
    out.retain(|c| c.len() > 2);
    out
}

fn intersect(p: &BezPath, clip: &[Vec<[f64; 2]>]) -> BezPath {
    let subj = contours(p);
    let shapes = subj.overlay(
        &clip.to_vec(),
        OverlayRule::Intersect,
        FillRule::NonZero,
    );
    let mut out = BezPath::new();
    for shape in shapes {
        for c in shape {
            for (i, q) in c.iter().enumerate() {
                if i == 0 {
                    out.move_to((q[0], q[1]));
                } else {
                    out.line_to((q[0], q[1]));
                }
            }
            out.close_path();
        }
    }
    out
}

pub fn lower(
    l: &List,
    caps: Caps,
    o: &mut Outlines,
    st: &mut Stats,
) -> List {
    let mut out = List::new(l.size);
    out.fonts = l.fonts.clone();
    let mut t = vec![Affine::IDENTITY];
    // Clip polygons in figure coordinates; None for
    // groups whose clip the backend draws itself.
    let mut clips: Vec<Option<std::rc::Rc<Poly>>> = vec![None];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        let clip = clips.iter().rev().flatten().next().cloned();
        let prune = caps_prune(caps);
        // A filled shape, clipped in geometry if needed.
        let mut emit = |out: &mut List, p: BezPath, paint: Brush| {
            let p = match &clip {
                Some(cl) => {
                    st.clipped += 1;
                    if prune {
                        match locate(cl, (cur * p.clone()).bounding_box()) {
                            Where::Inside => {
                                st.inside += 1;
                                out.commands.push(Command::Fill { path: p, paint });
                                return;
                            }
                            Where::Outside => {
                                st.outside += 1;
                                return;
                            }
                            Where::Partial => st.cut += 1,
                        }
                    }
                    let inv = cur.inverse();
                    let local: Vec<Vec<[f64; 2]>> = cl
                        .rings
                        .iter()
                        .map(|c| {
                            c.iter()
                                .map(|q| {
                                    let v = inv * kurbo::Point::new(q[0], q[1]);
                                    [v.x, v.y]
                                })
                                .collect()
                        })
                        .collect();
                    intersect(&p, &local)
                }
                None => p,
            };
            out.commands.push(Command::Fill { path: p, paint });
        };
        match c {
            Command::Fill { path, paint } => {
                emit(&mut out, path.clone(), paint.clone())
            }
            Command::Stroke { path, style, paint } => {
                if clip.is_some() {
                    let p = kurbo::stroke(
                        path.iter(),
                        style,
                        &StrokeOpts::default(),
                        0.05,
                    );
                    emit(&mut out, p, paint.clone());
                } else {
                    out.commands.push(c.clone());
                }
            }
            Command::Markers { shape, at, paint } => {
                if caps.markers && clip.is_none() {
                    out.commands.push(c.clone());
                    continue;
                }
                st.markers += 1;
                for q in at {
                    let p = Affine::translate(q.to_vec2())
                        * shape.clone();
                    emit(&mut out, p, paint.clone());
                }
            }
            Command::Glyphs(r) => {
                if caps.glyphs && clip.is_none() {
                    out.commands.push(c.clone());
                    continue;
                }
                st.glyphs += 1;
                let p = o.run_path(&l.fonts, r);
                emit(&mut out, p, Brush::Solid(r.fill));
            }
            Command::Label(_) => panic!("shape labels first"),
            Command::Push { transform, clip: cp } => {
                let next = cur * *transform;
                t.push(next);
                let geometric = cp.as_ref().filter(|p| {
                    !caps.clip_paths && as_rect(p).is_none()
                });
                match geometric {
                    Some(p) => {
                        let rings = contours(&(next * p.clone()));
                        let bb = (next * p.clone()).bounding_box();
                        clips.push(Some(std::rc::Rc::new(Poly { rings, bb })));
                        out.commands.push(Command::Push {
                            transform: *transform,
                            clip: None,
                        });
                    }
                    None => {
                        clips.push(None);
                        out.commands.push(c.clone());
                    }
                }
            }
            Command::Pop => {
                t.pop();
                clips.pop();
                out.commands.push(Command::Pop);
            }
        }
    }
    out
}
