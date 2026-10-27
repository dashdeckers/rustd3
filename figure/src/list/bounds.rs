//! [`Bounds`]: every command's bounds in list px, an explicit value.

use std::f64::consts::SQRT_2;

use kurbo::{Affine, BezPath, Cap, Join, Point, Rect, Shape};

use crate::list::{Clip, Command, List, Outlines, outline};

/// Per-command bounds in list px, computed once per list.
///
/// A command's bounds hold everything it may draw: a fill's path bbox, a
/// stroke's inflated for its joins and caps, a markers command's points one
/// by one, a run's glyph outlines; each through the transforms of its
/// groups and intersected with their clips. `Push`, `Pop` and commands that
/// draw nothing (an empty path, no points, everything clipped away) have
/// `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds(Vec<Option<Rect>>);

impl Bounds {
    /// The bounds of every command of `list`, glyph outlines read through
    /// `outlines`. Any list is accepted: a run whose font is outside the
    /// table has no outlines.
    pub fn of(list: &List, outlines: &mut Outlines) -> Bounds {
        let mut groups = vec![(Affine::IDENTITY, None::<Rect>)];
        let mut bounds = Vec::with_capacity(list.commands.len());
        for command in &list.commands {
            let (transform, clip) = *groups
                .last()
                .expect("invariant: the root group is never popped");
            let local = match command {
                Command::Fill { path, .. } => Some(path.bounding_box()),
                Command::Stroke { path, style, .. } => {
                    let reach = reach(style);
                    Some(path.bounding_box().inflate(reach, reach))
                }
                Command::Markers {
                    shape, at, sizes, ..
                } => markers(shape, at, sizes.as_deref()),
                Command::Glyphs(run) => list.fonts.get(run.font).and_then(|font| {
                    run.glyphs
                        .iter()
                        .filter_map(|glyph| {
                            let em = outlines.bounds(font, glyph.id)?;
                            Some(outline::place(run, glyph).transform_rect_bbox(em))
                        })
                        .reduce(|a, b| a.union(b))
                }),
                Command::Image { rect, .. } => Some(rect.abs()),
                Command::Push {
                    transform: inner,
                    clip: inner_clip,
                } => {
                    let next = transform * *inner;
                    let own = inner_clip.as_ref().map(|c| {
                        next.transform_rect_bbox(match c {
                            Clip::Rect(r) => r.abs(),
                            Clip::Path(p) => p.bounding_box(),
                        })
                    });
                    let clip = match (clip, own) {
                        (Some(a), Some(b)) => Some(a.intersect(b)),
                        (a, b) => a.or(b),
                    };
                    groups.push((next, clip));
                    None
                }
                Command::Pop => {
                    if groups.len() > 1 {
                        groups.pop();
                    }
                    None
                }
            };
            let global = local
                .map(|r| transform.transform_rect_bbox(r))
                .map(|r| clip.map_or(r, |c| r.intersect(c)))
                .filter(|r| r.area() > 0.0);
            bounds.push(global);
        }
        Bounds(bounds)
    }

    /// The bounds of command `command`; `None` for a group's ends, a
    /// command that draws nothing, or an index past the list.
    pub fn get(&self, command: usize) -> Option<Rect> {
        self.0.get(command).copied().flatten()
    }

    /// The number of commands.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the list had no commands.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// How far a stroke reaches past its path: half its width, times the miter
/// limit for miter joins and √2 for square caps (a cap's corner on a
/// diagonal).
fn reach(style: &kurbo::Stroke) -> f64 {
    let join = match style.join {
        Join::Miter => style.miter_limit.max(1.0),
        Join::Round | Join::Bevel => 1.0,
    };
    let cap = |cap: Cap| match cap {
        Cap::Square => SQRT_2,
        Cap::Butt | Cap::Round => 1.0,
    };
    style.width.abs() / 2.0 * join.max(cap(style.start_cap)).max(cap(style.end_cap))
}

/// The union of the shape's bbox scaled by each point's size and centred on
/// it; `None` without points.
fn markers(shape: &BezPath, at: &[Point], sizes: Option<&[f64]>) -> Option<Rect> {
    let unit = shape.bounding_box();
    at.iter()
        .enumerate()
        .map(|(i, p)| {
            let size = sizes.and_then(|s| s.get(i)).copied().unwrap_or(1.0);
            (Affine::translate(p.to_vec2()) * Affine::scale(size)).transform_rect_bbox(unit)
        })
        .reduce(|a, b| a.union(b))
}
