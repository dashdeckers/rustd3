//! [`Snap`]: how a target lays plot-area edges and hairlines on its pixel
//! grid.

use kurbo::{Point, Rect};

use crate::raster::snap;
use crate::target::Target;

/// How a target lays plot-area edges and hairlines (axis lines, ticks,
/// borders, rules) on its pixel grid: raster targets put edges on device
/// pixel boundaries and hairlines on whole device pixels
/// ([`raster::snap`](crate::raster::snap)); vector output keeps both as
/// they are. A mark reads the target's from [`Ctx::snap`](crate::mark::Ctx::snap).
///
/// ```
/// use rustd3_figure::kurbo::{Rect, Shape};
/// use rustd3_figure::list::{self, Command, List};
/// use rustd3_figure::mark::Ctx;
///
/// /// A custom mark's outlined box, as crisp as a rect mark's on raster
/// /// targets: its fill on device pixel boundaries, its edge on whole
/// /// device pixels.
/// fn boxed(ctx: &Ctx<'_>, r: Rect, list: &mut List) {
///     let (snap, theme) = (ctx.snap(), ctx.get_theme());
///     let w = theme.get_patch_width();
///     list.commands.push(Command::Fill {
///         path: snap.rect(r).to_path(0.1),
///         paint: list::brush(theme.get_color()),
///     });
///     let at = |v: f64| snap.line(v, w);
///     let edge = Rect::new(at(r.x0), at(r.y0), at(r.x1), at(r.y1));
///     list.commands.push(Command::Stroke {
///         path: edge.to_path(0.1),
///         style: list::stroke(snap.width(w)),
///         paint: list::brush(theme.get_ink()),
///     });
/// }
/// # let _ = boxed;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snap {
    /// Device pixels per list px; `None` for vector output.
    scale: Option<f64>,
}

impl Snap {
    /// The snapping of `target`.
    pub(crate) fn of(target: Target) -> Snap {
        let scale = match target {
            Target::Vector => None,
            Target::File { scale } | Target::Screen { scale } => Some(scale),
        };
        Snap { scale }
    }

    /// Device pixels per list px; `None` for vector output.
    pub(crate) fn scale(self) -> Option<f64> {
        self.scale
    }

    /// `r` with its edges on device pixel boundaries: a plot area, or a
    /// filled box a mark draws without antialiased edges.
    #[must_use]
    pub fn rect(self, r: Rect) -> Rect {
        let Some(s) = self.scale else { return r };
        Rect::new(
            snap::edge(r.x0, s),
            snap::edge(r.y0, s),
            snap::edge(r.x1, s),
            snap::edge(r.y1, s),
        )
    }

    /// The width a hairline `w` px wide is stroked at: whole device pixels,
    /// at least one.
    #[must_use]
    pub fn width(self, w: f64) -> f64 {
        self.scale.map_or(w, |s| snap::width(w, s))
    }

    /// The centre `x` of a hairline `w` px wide (its unsnapped width), such
    /// that it covers whole device pixels: the x of a vertical line or the
    /// y of a horizontal one.
    #[must_use]
    pub fn line(self, x: f64, w: f64) -> f64 {
        self.scale.map_or(x, |s| snap::line(x, w, s))
    }

    /// `p` with both coordinates as [`line`](Snap::line) centres: a
    /// hairline's end, or a corner of a stroked rectangle.
    pub(crate) fn point(self, p: Point, w: f64) -> Point {
        Point::new(self.line(p.x, w), self.line(p.y, w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_output_snaps_nothing() {
        let snap = Snap::of(Target::Vector);
        let r = Rect::new(10.3, 2.2, 99.9, 50.05);
        assert_eq!(snap.rect(r), r);
        assert_eq!((snap.width(1.667), snap.line(10.3, 1.667)), (1.667, 10.3));
    }

    #[test]
    fn raster_targets_snap_edges_and_hairlines_to_device_pixels() {
        let snap = Snap::of(Target::File { scale: 1.25 });
        let r = snap.rect(Rect::new(10.3, 2.2, 99.9, 50.05));
        for v in [r.x0, r.y0, r.x1, r.y1] {
            assert!((v * 1.25 - (v * 1.25).round()).abs() < 1e-9, "{v}");
        }
        // 1.25 pt = 1.667 px is 2.083 device px: 2.
        assert!((snap.width(5.0 / 3.0) * 1.25 - 2.0).abs() < 1e-12);
        // An even width centres on a pixel boundary, an odd one inside one.
        assert_eq!(snap.line(10.3, 5.0 / 3.0) * 1.25, 13.0);
        let screen = Snap::of(Target::Screen { scale: 1.0 });
        assert_eq!(screen.line(10.3, 5.0 / 3.0), 10.0);
        assert_eq!(screen.line(10.3, 1.0), 10.5);
    }
}
