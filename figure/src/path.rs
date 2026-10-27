//! [`Bez`]: d3's canvas path interface recorded into a kurbo [`BezPath`],
//! so every generator of the core draws into the drawing list.
//!
//! ```
//! use rustd3_core::path::PathContext;
//! use rustd3_figure::path;
//!
//! let p = path::bez(|p| {
//!     p.move_to(0.0, 0.0);
//!     p.line_to(10.0, 0.0);
//! });
//! assert_eq!(p.elements().len(), 2);
//! ```

use std::f64::consts::TAU;

use kurbo::{BezPath, PathEl, Point};
use rustd3_core::path::PathContext;

/// d3's tolerance for coincident points and empty arcs.
const EPSILON: f64 = 1e-6;
/// The accuracy of an arc's cubic approximation, in path units.
const ARC_TOLERANCE: f64 = 0.1;

/// A [`PathContext`] that records into a [`BezPath`]. Arcs follow d3 and
/// [`SvgPath`](rustd3_core::path::SvgPath) exactly (as cubics); a segment
/// with no subpath to continue starts one, as a canvas does, where d3 would
/// write path data no renderer reads.
#[derive(Debug, Clone, Default)]
pub struct Bez {
    path: BezPath,
    /// The end of the current subpath, as d3 tracks it.
    current: Option<Point>,
    /// The start of the current subpath, where `close_path` returns.
    start: Option<Point>,
}

impl Bez {
    /// An empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// The recorded path.
    pub fn finish(self) -> BezPath {
        self.path
    }

    /// Starts a subpath at `at` when the path has none to continue.
    fn ensure(&mut self, at: Point) {
        if self.path.elements().is_empty() {
            self.path.move_to(at);
        }
    }
}

impl PathContext for Bez {
    fn move_to(&mut self, x: f64, y: f64) {
        let p = Point::new(x, y);
        self.path.move_to(p);
        self.start = Some(p);
        self.current = Some(p);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        let p = Point::new(x, y);
        if self.path.elements().is_empty() {
            self.move_to(x, y);
            return;
        }
        self.path.line_to(p);
        self.current = Some(p);
    }

    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        let p = Point::new(x, y);
        self.ensure(Point::new(x1, y1));
        self.path.quad_to(Point::new(x1, y1), p);
        self.current = Some(p);
    }

    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        let p = Point::new(x, y);
        self.ensure(Point::new(x1, y1));
        self.path
            .curve_to(Point::new(x1, y1), Point::new(x2, y2), p);
        self.current = Some(p);
    }

    /// d3's arc: a line to its start unless that is within 1e-6 of the
    /// current point (a move when there is none), then cubics; nothing for
    /// a negative radius and nothing past the start for a zero one.
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) {
        if r < 0.0 {
            return;
        }
        let from = Point::new(x + r * start.cos(), y + r * start.sin());
        match self.current {
            None => self.path.move_to(from),
            Some(c) if (c.x - from.x).abs() > EPSILON || (c.y - from.y).abs() > EPSILON => {
                self.path.line_to(from);
            }
            Some(_) => {}
        }
        if r == 0.0 || r.is_nan() {
            return;
        }
        let mut da = if anticlockwise {
            start - end
        } else {
            end - start
        };
        if da < 0.0 {
            da = da % TAU + TAU;
        }
        let sign = if anticlockwise { -1.0 } else { 1.0 };
        let (sweep, to) = if da > TAU - EPSILON {
            (TAU, from)
        } else if da > EPSILON {
            (da, Point::new(x + r * end.cos(), y + r * end.sin()))
        } else {
            return;
        };
        let first = self.path.elements().len();
        kurbo::Arc::new((x, y), (r, r), start, sign * sweep, 0.0)
            .to_cubic_beziers(ARC_TOLERANCE, |a, b, c| self.path.curve_to(a, b, c));
        // The last cubic ends where d3's arc does, exactly.
        if self.path.elements().len() > first
            && let Some(PathEl::CurveTo(_, _, last)) = self.path.elements_mut().last_mut()
        {
            *last = to;
        }
        self.current = Some(to);
    }

    fn close_path(&mut self) {
        if self.current.is_some() {
            self.path.close_path();
            self.current = self.start;
        }
    }
}

/// The path `draw` records.
///
/// ```
/// use std::f64::consts::TAU;
///
/// use rustd3_core::path::PathContext;
/// use rustd3_figure::kurbo::PathEl;
/// use rustd3_figure::path;
///
/// let circle = path::bez(|p| p.arc(0.0, 0.0, 10.0, 0.0, TAU, false));
/// let els = circle.elements();
/// assert_eq!(els[0], PathEl::MoveTo((10.0, 0.0).into()));
/// assert!(matches!(els[els.len() - 1], PathEl::CurveTo(_, _, p) if p == (10.0, 0.0).into()));
/// ```
pub fn bez(draw: impl FnOnce(&mut Bez)) -> BezPath {
    let mut bez = Bez::new();
    draw(&mut bez);
    bez.finish()
}
