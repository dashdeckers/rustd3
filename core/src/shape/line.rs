//! d3's line generator: one curve through a run of points, a new run
//! after every gap.

use crate::path::PathContext;
use crate::shape::Curve;
use crate::shape::curve::State;

/// An item a line draws: a point, or `None` for a gap.
pub trait Vertex {
    /// The point, or `None` to break the line (d3's `defined`).
    fn vertex(self) -> Option<(f64, f64)>;
}

/// d3.line() over already-scaled points, built by consuming builders.
///
/// ```
/// use rustd3_core::path;
/// use rustd3_core::shape;
///
/// let readings = [Some(75.0), Some(25.0), None, Some(50.0), Some(0.0)];
/// let points = readings
///     .iter()
///     .enumerate()
///     .map(|(i, v)| v.map(|v| (i as f64 * 100.0, v)));
/// let d = path::svg(|p| shape::line().draw(points, p));
/// assert_eq!(d, "M0,75L100,25M300,50L400,0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Line {
    curve: Curve,
}

impl Vertex for (f64, f64) {
    fn vertex(self) -> Option<(f64, f64)> {
        Some(self)
    }
}

impl Vertex for Option<(f64, f64)> {
    fn vertex(self) -> Option<(f64, f64)> {
        self
    }
}

impl Line {
    /// Joins the points with `curve`: d3's `line.curve`.
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve }
    }

    /// Draws `points` into `ctx` as d3's `line(data)` does: each run of
    /// `Some` items is one curve, and a `None` ends the run before it. NaN
    /// coordinates are drawn as given; only `None` breaks the line.
    pub fn draw<V: Vertex>(&self, points: impl IntoIterator<Item = V>, ctx: &mut impl PathContext) {
        let mut state = State::new(self.curve);
        let mut defined = false;
        for point in points {
            match point.vertex() {
                Some((x, y)) => {
                    if !defined {
                        state.line_start();
                        defined = true;
                    }
                    state.point(x, y, ctx);
                }
                None if defined => {
                    state.line_end(ctx);
                    defined = false;
                }
                None => {}
            }
        }
        if defined {
            state.line_end(ctx);
        }
    }
}
