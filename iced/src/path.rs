//! [`IcedPath`]: d3's canvas path interface over iced's path builder.

use std::f64::consts::{FRAC_PI_2, TAU};

use iced_core::Point;
use iced_widget::canvas;
use rustd3_core::path::PathContext;

/// d3's tolerance for coincident points and empty arcs, as in `SvgPath`.
const EPSILON: f64 = 1e-6;

/// A d3 path context that draws into an iced path builder.
///
/// iced's own `Builder::arc` starts a new subpath, which would split a d3
/// pie slice in two, and the builder exposes no current point. So
/// `IcedPath` tracks the current point and the subpath's start, and cuts
/// arcs into cubics of at most a quarter turn itself, with d3's arc
/// semantics: a negative radius draws nothing.
///
/// ```
/// # mod rustd3 { pub use rustd3_core::path; }
/// use rustd3::path::PathContext;
///
/// let wedge = rustd3_iced::path(|p| {
///     p.move_to(10.0, 0.0);
///     p.arc(0.0, 0.0, 10.0, 0.0, 1.0, false);
///     p.line_to(0.0, 0.0);
///     p.close_path();
/// });
/// // One subpath: begin, the arc's cubic, the line, end.
/// assert_eq!(wedge.raw().iter().count(), 4);
/// ```
pub struct IcedPath<'a> {
    builder: &'a mut canvas::path::Builder,
    /// The end of the last segment drawn, as d3's path tracks it.
    current: Option<(f64, f64)>,
    /// The start of the current subpath, where `close_path` returns.
    start: Option<(f64, f64)>,
}

impl<'a> std::fmt::Debug for IcedPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcedPath").finish_non_exhaustive()
    }
}

impl PathContext for IcedPath<'_> {
    fn move_to(&mut self, x: f64, y: f64) {
        self.builder.move_to(point(x, y));
        self.start = Some((x, y));
        self.current = Some((x, y));
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.builder.line_to(point(x, y));
        self.reach((x, y));
    }

    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.builder.quadratic_curve_to(point(x1, y1), point(x, y));
        self.reach((x, y));
    }

    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.builder
            .bezier_curve_to(point(x1, y1), point(x2, y2), point(x, y));
        self.reach((x, y));
    }

    /// Draws d3's arc, cut into cubics of at most a quarter turn. A
    /// negative radius draws nothing.
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) {
        if r < 0.0 {
            return;
        }
        let from = (x + r * start.cos(), y + r * start.sin());
        match self.current {
            None => self.move_to(from.0, from.1),
            Some((cx, cy)) => {
                if (cx - from.0).abs() > EPSILON || (cy - from.1).abs() > EPSILON {
                    self.line_to(from.0, from.1);
                }
            }
        }
        if r == 0.0 || r.is_nan() {
            return;
        }
        let mut sweep = if anticlockwise {
            start - end
        } else {
            end - start
        };
        if sweep < 0.0 {
            sweep = sweep % TAU + TAU;
        }
        let (sweep, to) = if sweep > TAU - EPSILON {
            (TAU, from)
        } else if sweep > EPSILON {
            (sweep, (x + r * end.cos(), y + r * end.sin()))
        } else {
            return;
        };
        let direction = if anticlockwise { -1.0 } else { 1.0 };
        // `sweep` lies in (EPSILON, TAU] here: one to four pieces.
        let pieces = (sweep / FRAC_PI_2).ceil() as u32;
        let step = direction * sweep / f64::from(pieces);
        // 4/3 · tan(θ/4) · r, signed by the direction of travel.
        let k = 4.0 / 3.0 * (step / 4.0).tan() * r;
        let mut angle = start;
        let mut at = from;
        for piece in 1..=pieces {
            let next = start + step * f64::from(piece);
            let reach = if piece == pieces {
                to
            } else {
                (x + r * next.cos(), y + r * next.sin())
            };
            let c1 = (at.0 - k * angle.sin(), at.1 + k * angle.cos());
            let c2 = (reach.0 + k * next.sin(), reach.1 - k * next.cos());
            self.builder.bezier_curve_to(
                point(c1.0, c1.1),
                point(c2.0, c2.1),
                point(reach.0, reach.1),
            );
            angle = next;
            at = reach;
        }
        self.current = Some(to);
    }

    fn close_path(&mut self) {
        if self.current.is_some() {
            self.builder.close();
            self.current = self.start;
        }
    }
}

impl<'a> IcedPath<'a> {
    /// A context drawing into `builder`, which must start empty, since the
    /// context's current point starts unset.
    pub fn new(builder: &'a mut canvas::path::Builder) -> Self {
        IcedPath {
            builder,
            current: None,
            start: None,
        }
    }

    /// Records the end of a segment. A segment drawn with no current point
    /// starts its subpath there, as iced's builder (lyon's) does.
    fn reach(&mut self, to: (f64, f64)) {
        if self.current.is_none() {
            self.start = Some(to);
        }
        self.current = Some(to);
    }
}

/// Builds an iced path from anything that draws into a
/// [`PathContext`].
///
/// ```
/// # mod rustd3 { pub use rustd3_core::shape; }
/// use rustd3::shape;
///
/// let trace = rustd3_iced::path(|p| shape::line().draw([(0.0, 120.0), (320.0, 120.0)], p));
/// assert_eq!(trace.raw().iter().count(), 3); // begin, line, end
/// ```
pub fn path(draw: impl FnOnce(&mut IcedPath<'_>)) -> canvas::Path {
    canvas::Path::new(|builder| draw(&mut IcedPath::new(builder)))
}

/// An iced point. Geometry converts to `f32` only here, at the backend
/// boundary.
fn point(x: f64, y: f64) -> Point {
    Point::new(x as f32, y as f32)
}
