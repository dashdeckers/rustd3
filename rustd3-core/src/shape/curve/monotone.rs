//! d3's curveMonotoneX and curveMonotoneY: cubic Hermite splines whose
//! tangents (Steffen, 1990) keep the curve monotone in y between points,
//! or in x when reflected.

use std::f64::consts::FRAC_PI_2;

use super::Line;
use crate::js;
use crate::path::PathContext;

/// d3's MonotoneX, or MonotoneY when `reflect`. As in d3, MonotoneY draws
/// MonotoneX of the swapped points through a [`Reflect`] context.
pub(crate) struct Monotone {
    reflect: bool,
    line: Line,
    /// Points of the run so far, counted up to 3 (d3's `_point`).
    point: u8,
    /// The two previous points (d3's `_x0, _y0` and `_x1, _y1`).
    p0: (f64, f64),
    p1: (f64, f64),
    /// The tangent at `p1` (d3's `_t0`).
    t0: f64,
}

/// d3's ReflectContext: draws into the context it wraps with x and y
/// swapped in every call.
struct Reflect<'a, C>(&'a mut C);

impl<C: PathContext> PathContext for Reflect<'_, C> {
    fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to(y, x);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(y, x);
    }

    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.0.quadratic_curve_to(y1, x1, y, x);
    }

    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.0.bezier_curve_to(y1, x1, y2, x2, y, x);
    }

    /// The mirror image of the arc: angle θ becomes π/2 − θ, and the sweep
    /// turns the other way.
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) {
        self.0
            .arc(y, x, r, FRAC_PI_2 - start, FRAC_PI_2 - end, !anticlockwise);
    }

    fn close_path(&mut self) {
        self.0.close_path();
    }
}

impl Monotone {
    pub(crate) fn new(reflect: bool) -> Monotone {
        Monotone {
            reflect,
            line: Line::Unset,
            point: 0,
            p0: (f64::NAN, f64::NAN),
            p1: (f64::NAN, f64::NAN),
            t0: f64::NAN,
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        (self.p0, self.p1, self.t0) = ((f64::NAN, f64::NAN), (f64::NAN, f64::NAN), f64::NAN);
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.reflect {
            self.finish(&mut Reflect(ctx));
        } else {
            self.finish(ctx);
        }
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if self.reflect {
            self.next(y, x, &mut Reflect(ctx));
        } else {
            self.next(x, y, ctx);
        }
    }

    /// MonotoneX's `lineEnd`.
    fn finish(&mut self, ctx: &mut impl PathContext) {
        match self.point {
            2 => ctx.line_to(self.p1.0, self.p1.1),
            3 => self.bezier(ctx, self.t0, self.slope2(self.t0)),
            _ => {}
        }
        if self.line.closes(self.point == 1) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    /// MonotoneX's `point(x, y)`; a point equal to the last is ignored.
    fn next(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if (x, y) == self.p1 {
            return;
        }
        let mut t1 = f64::NAN;
        match self.point {
            0 => {
                self.point = 1;
                self.line.start(ctx, x, y);
            }
            1 => self.point = 2,
            2 => {
                self.point = 3;
                t1 = self.slope3(x, y);
                self.bezier(ctx, self.slope2(t1), t1);
            }
            _ => {
                t1 = self.slope3(x, y);
                self.bezier(ctx, self.t0, t1);
            }
        }
        (self.p0, self.p1, self.t0) = (self.p1, (x, y), t1);
    }

    /// d3's `slope3`: the tangent at `p1` with the next point (`x2`, `y2`),
    /// limited so the curve stays monotone. It is 0 where the limit is NaN.
    fn slope3(&self, x2: f64, y2: f64) -> f64 {
        let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
        let (h0, h1) = (x1 - x0, x2 - x1);
        let s0 = (y1 - y0) / or_zero(h0, h1);
        let s1 = (y2 - y1) / or_zero(h1, h0);
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        let t = (sign(s0) + sign(s1)) * js::min(js::min(s0.abs(), s1.abs()), 0.5 * p.abs());
        if js::truthy(t) { t } else { 0.0 }
    }

    /// d3's `slope2`: the one-sided tangent at an end of the run.
    fn slope2(&self, t: f64) -> f64 {
        let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
        let h = x1 - x0;
        if js::truthy(h) {
            (3.0 * (y1 - y0) / h - t) / 2.0
        } else {
            t
        }
    }

    /// d3's monotone `point(that, t0, t1)`: the Hermite segment from `p0`
    /// to `p1` with tangents `t0` and `t1`, as a cubic Bézier.
    fn bezier(&self, ctx: &mut impl PathContext, t0: f64, t1: f64) {
        let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
        let dx = (x1 - x0) / 3.0;
        ctx.bezier_curve_to(x0 + dx, y0 + dx * t0, x1 - dx, y1 - dx * t1, x1, y1);
    }
}

/// d3's `h || other < 0 && -0`: a zero (or NaN) step becomes a zero signed
/// like the step beside it, so that a vertical segment's slope is ±∞ with
/// the right sign.
fn or_zero(h: f64, other: f64) -> f64 {
    if js::truthy(h) {
        h
    } else if other < 0.0 {
        -0.0
    } else {
        0.0
    }
}

/// d3's `sign`: −1 below zero, else 1 (NaN included).
fn sign(x: f64) -> f64 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::SvgPath;

    #[test]
    fn reflect_swaps_every_coordinate_and_mirrors_arcs() {
        let mut reflected = SvgPath::new();
        let mut r = Reflect(&mut reflected);
        r.move_to(1.0, 2.0);
        r.line_to(3.0, 4.0);
        r.quadratic_curve_to(5.0, 6.0, 7.0, 8.0);
        r.bezier_curve_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        r.close_path();
        assert_eq!(reflected.to_string(), "M2,1L4,3Q6,5,8,7C2,1,4,3,6,5Z");

        // The clockwise quarter arc from (11, 0) to (10, 1) around (10, 0)
        // mirrors to the anticlockwise one from (0, 11) to (1, 10).
        let mut arc = SvgPath::with_digits(3);
        arc.arc(10.0, 0.0, 1.0, 0.0, FRAC_PI_2, false);
        assert_eq!(arc.to_string(), "M11,0A1,1,0,0,1,10,1");
        let mut reflected = SvgPath::with_digits(3);
        Reflect(&mut reflected).arc(10.0, 0.0, 1.0, 0.0, FRAC_PI_2, false);
        assert_eq!(reflected.to_string(), "M0,11A1,1,0,0,0,1,10");
    }
}
