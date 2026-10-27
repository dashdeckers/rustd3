//! d3's curveBundle: a basis spline through points pulled toward the
//! straight line from the run's first point to its last.

use super::basis::Basis;
use crate::path::PathContext;

/// d3's Bundle: buffers the run and draws it through a [`Basis`] at
/// `line_end`. It has no area flag: an area draws `Bundle` as `Basis`.
pub(crate) struct Bundle {
    basis: Basis,
    beta: f64,
    /// The run's points (d3's `_x`, `_y`).
    points: Vec<(f64, f64)>,
}

impl Bundle {
    pub(crate) fn new(beta: f64) -> Bundle {
        Bundle {
            basis: Basis::new(),
            beta,
            points: Vec::new(),
        }
    }

    pub(crate) fn line_start(&mut self) {
        self.points.clear();
        self.basis.line_start();
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if let [(x0, y0), .., (xj, yj)] = self.points[..] {
            let j = (self.points.len() - 1) as f64;
            let (dx, dy) = (xj - x0, yj - y0);
            for (i, &(x, y)) in self.points.iter().enumerate() {
                let t = i as f64 / j;
                self.basis.point(
                    self.beta * x + (1.0 - self.beta) * (x0 + t * dx),
                    self.beta * y + (1.0 - self.beta) * (y0 + t * dy),
                    ctx,
                );
            }
        }
        self.points.clear();
        self.basis.line_end(ctx);
    }

    /// Buffers the point; the run is drawn at `line_end`.
    pub(crate) fn point(&mut self, x: f64, y: f64, _ctx: &mut impl PathContext) {
        self.points.push((x, y));
    }
}
