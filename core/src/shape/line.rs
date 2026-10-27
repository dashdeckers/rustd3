//! d3's line generator ([`Line`](crate::shape::Line)): one curve through a
//! run of points, a new run after every gap; and its radial form
//! ([`Radial`]).

mod radial;

pub use radial::{Radial, radial};

use crate::path::PathContext;
use crate::shape::curve::State;
use crate::shape::{Curve, Line, Vertex};

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
