//! d3's curveNatural: the natural cubic spline through the run, its
//! second derivative zero at both ends.

use super::Line;
use crate::path::PathContext;

/// d3's Natural: buffers the run and solves for its control points at
/// `line_end`.
pub(crate) struct Natural {
    line: Line,
    /// The run's coordinates (d3's `_x`, `_y`).
    xs: Vec<f64>,
    ys: Vec<f64>,
}

impl Natural {
    pub(crate) fn new() -> Natural {
        Natural {
            line: Line::Unset,
            xs: Vec::new(),
            ys: Vec::new(),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.xs.clear();
        self.ys.clear();
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        let (xs, ys) = (&self.xs, &self.ys);
        let n = xs.len();
        if n > 0 {
            self.line.start(ctx, xs[0], ys[0]);
        }
        if n == 2 {
            ctx.line_to(xs[1], ys[1]);
        } else if n > 2 {
            let (px, py) = (control_points(xs), control_points(ys));
            for i in 0..n - 1 {
                ctx.bezier_curve_to(px.0[i], py.0[i], px.1[i], py.1[i], xs[i + 1], ys[i + 1]);
            }
        }
        if self.line.closes(n == 1) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
        self.line_start();
    }

    /// Buffers the point; the run is drawn at `line_end`.
    pub(crate) fn point(&mut self, x: f64, y: f64, _ctx: &mut impl PathContext) {
        self.xs.push(x);
        self.ys.push(y);
    }
}

/// d3's `controlPoints(x)` for at least three knots: the first and second
/// control point of each of the n − 1 segments, from the tridiagonal
/// system of the natural spline (Thomas algorithm).
fn control_points(x: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = x.len() - 1;
    let (mut a, mut b, mut r) = (vec![1.0; n], vec![4.0; n], vec![0.0; n]);
    (a[0], b[0], r[0]) = (0.0, 2.0, x[0] + 2.0 * x[1]);
    for i in 1..n - 1 {
        r[i] = 4.0 * x[i] + 2.0 * x[i + 1];
    }
    (a[n - 1], b[n - 1], r[n - 1]) = (2.0, 7.0, 8.0 * x[n - 1] + x[n]);
    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m;
        r[i] -= m * r[i - 1];
    }
    a[n - 1] = r[n - 1] / b[n - 1];
    for i in (0..n - 1).rev() {
        a[i] = (r[i] - a[i + 1]) / b[i];
    }
    b[n - 1] = (x[n] + a[n - 1]) / 2.0;
    for i in 0..n - 1 {
        b[i] = 2.0 * x[i + 1] - a[i + 1];
    }
    (a, b)
}
