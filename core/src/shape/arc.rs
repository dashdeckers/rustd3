//! d3's arc generator: circular and annular sectors with rounded corners
//! and padding, drawn from a [`Slice`].

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::js;
use crate::path::PathContext;
use crate::shape::Slice;

/// d3-shape's tolerance (`math.js`): smaller radii and angles are none.
const EPSILON: f64 = 1e-12;

/// d3.arc() over [`Slice`]s, built by consuming builders: radii here, angles
/// and the default pad angle from the slice. Angles run clockwise from 12
/// o'clock; an inner radius above the outer one is swapped.
///
/// ```
/// use rustd3_core::path::SvgPath;
/// use rustd3_core::shape;
///
/// let slices = shape::pie().layout([3.0, 1.0, 2.0]);
/// let donut = shape::arc().inner_radius(40.0).outer_radius(80.0);
/// // d3-shape's generators print 3 digits.
/// let mut path = SvgPath::with_digits(3);
/// donut.draw(&slices[0], &mut path);
/// assert_eq!(path.to_string(), "M0,-80A80,80,0,1,1,0,80L0,40A40,40,0,1,0,0,-40Z");
/// assert_eq!(donut.centroid(&slices[0]), (60.0, 0.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Arc {
    inner_radius: f64,
    outer_radius: f64,
    corner_radius: f64,
    /// `None`: √(r₀² + r₁²).
    pad_radius: Option<f64>,
    /// `None`: the slice's own.
    pad_angle: Option<f64>,
}

/// d3's `cornerTangents`: a corner circle of radius `rc` tangent to the ring
/// of radius `r1` and to the line from (x0, y0) to (x1, y1).
struct Corner {
    cx: f64,
    cy: f64,
    x01: f64,
    y01: f64,
    x11: f64,
    y11: f64,
}

impl Arc {
    /// The inner radius (d3's default reads the datum; here 0).
    pub fn inner_radius(self, inner_radius: f64) -> Self {
        Self {
            inner_radius,
            ..self
        }
    }

    /// The outer radius (d3's default reads the datum; here 0).
    pub fn outer_radius(self, outer_radius: f64) -> Self {
        Self {
            outer_radius,
            ..self
        }
    }

    /// The corners' radius (default 0), at most half the ring's width and
    /// smaller where the sector is too narrow.
    pub fn corner_radius(self, corner_radius: f64) -> Self {
        Self {
            corner_radius,
            ..self
        }
    }

    /// The radius the pad angle is measured at, so that padding is a
    /// constant width along both edges; `None` (the default) is
    /// √(r₀² + r₁²).
    pub fn pad_radius(self, pad_radius: Option<f64>) -> Self {
        Self { pad_radius, ..self }
    }

    /// The pad angle for every slice; `None` (the default) takes each
    /// slice's `pad_angle`.
    pub fn pad_angle(self, pad_angle: Option<f64>) -> Self {
        Self { pad_angle, ..self }
    }

    /// Draws `slice` into `ctx` as d3's `arc(d)` does (`arc.js:88–227`): a
    /// point when the outer radius is not above 1e-12, a circle or annulus
    /// when the span reaches τ, otherwise the sector with its corners
    /// rounded and its edges padded, closed.
    pub fn draw(&self, slice: &Slice, ctx: &mut impl PathContext) {
        let (mut r0, mut r1) = (self.inner_radius, self.outer_radius);
        let a0 = slice.start_angle - FRAC_PI_2;
        let a1 = slice.end_angle - FRAC_PI_2;
        let da = (a1 - a0).abs();
        let cw = a1 > a0;
        if r1 < r0 {
            (r0, r1) = (r1, r0);
        }
        if r1 > EPSILON && da > TAU - EPSILON {
            ctx.move_to(r1 * a0.cos(), r1 * a0.sin());
            ctx.arc(0.0, 0.0, r1, a0, a1, !cw);
            if r0 > EPSILON {
                ctx.move_to(r0 * a1.cos(), r0 * a1.sin());
                ctx.arc(0.0, 0.0, r0, a1, a0, cw);
            }
        } else if r1 > EPSILON {
            self.sector(slice, [r0, r1], [a0, a1], ctx);
        } else {
            ctx.move_to(0.0, 0.0);
        }
        ctx.close_path();
    }

    /// d3's `arc.centroid(d)`: the midpoint of the slice's centre line, at
    /// the mean radius and the mean angle.
    pub fn centroid(&self, slice: &Slice) -> (f64, f64) {
        let r = (self.inner_radius + self.outer_radius) / 2.0;
        let a = (slice.start_angle + slice.end_angle) / 2.0 - PI / 2.0;
        (a.cos() * r, a.sin() * r)
    }

    /// The circular or annular sector of `arc.js:120–224`, with `r0 ≤ r1`.
    fn sector(
        &self,
        slice: &Slice,
        [r0, r1]: [f64; 2],
        [a0, a1]: [f64; 2],
        ctx: &mut impl PathContext,
    ) {
        let da = (a1 - a0).abs();
        let cw = a1 > a0;
        let (mut a01, mut a11, mut a00, mut a10) = (a0, a1, a0, a1);
        let (mut da0, mut da1) = (da, da);
        let ap = self.pad_angle.unwrap_or(slice.pad_angle) / 2.0;
        let rp = if ap > EPSILON {
            self.pad_radius
                .unwrap_or_else(|| (r0 * r0 + r1 * r1).sqrt())
        } else {
            0.0
        };
        let rc = js::min((r1 - r0).abs() / 2.0, self.corner_radius);
        let (mut rc0, mut rc1) = (rc, rc);

        // Padding: since r1 ≥ r0, da1 ≥ da0.
        if rp > EPSILON {
            let mut p0 = asin(rp / r0 * ap.sin());
            let mut p1 = asin(rp / r1 * ap.sin());
            da0 -= p0 * 2.0;
            if da0 > EPSILON {
                p0 *= if cw { 1.0 } else { -1.0 };
                a00 += p0;
                a10 -= p0;
            } else {
                da0 = 0.0;
                a00 = (a0 + a1) / 2.0;
                a10 = a00;
            }
            da1 -= p1 * 2.0;
            if da1 > EPSILON {
                p1 *= if cw { 1.0 } else { -1.0 };
                a01 += p1;
                a11 -= p1;
            } else {
                da1 = 0.0;
                a01 = (a0 + a1) / 2.0;
                a11 = a01;
            }
        }

        let (x01, y01) = (r1 * a01.cos(), r1 * a01.sin());
        let (x10, y10) = (r0 * a10.cos(), r0 * a10.sin());
        let (x11, y11) = (r1 * a11.cos(), r1 * a11.sin());
        let (x00, y00) = (r0 * a00.cos(), r0 * a00.sin());

        // Corners narrower than the sector allows: restrict them, or drop
        // them when the edges do not intersect (the arc is too small).
        if rc > EPSILON && da < PI {
            if let Some((ox, oy)) = intersect([x01, y01, x00, y00], [x11, y11, x10, y10]) {
                let (ax, ay) = (x01 - ox, y01 - oy);
                let (bx, by) = (x11 - ox, y11 - oy);
                let cos =
                    (ax * bx + ay * by) / ((ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt());
                let kc = 1.0 / (acos(cos) / 2.0).sin();
                let lc = (ox * ox + oy * oy).sqrt();
                rc0 = js::min(rc, (r0 - lc) / (kc - 1.0));
                rc1 = js::min(rc, (r1 - lc) / (kc + 1.0));
            } else {
                rc0 = 0.0;
                rc1 = 0.0;
            }
        }

        // The outer ring: with rounded corners, a circular arc, or collapsed
        // to a line.
        if da1 > EPSILON && rc1 > EPSILON {
            let t0 = Corner::new([x00, y00, x01, y01], r1, rc1, cw);
            let t1 = Corner::new([x11, y11, x10, y10], r1, rc1, cw);
            ctx.move_to(t0.cx + t0.x01, t0.cy + t0.y01);
            if rc1 < rc {
                // The corners have merged.
                ctx.arc(
                    t0.cx,
                    t0.cy,
                    rc1,
                    t0.y01.atan2(t0.x01),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            } else {
                ctx.arc(
                    t0.cx,
                    t0.cy,
                    rc1,
                    t0.y01.atan2(t0.x01),
                    t0.y11.atan2(t0.x11),
                    !cw,
                );
                let start = (t0.cy + t0.y11).atan2(t0.cx + t0.x11);
                let end = (t1.cy + t1.y11).atan2(t1.cx + t1.x11);
                ctx.arc(0.0, 0.0, r1, start, end, !cw);
                ctx.arc(
                    t1.cx,
                    t1.cy,
                    rc1,
                    t1.y11.atan2(t1.x11),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            }
        } else if da1 > EPSILON {
            ctx.move_to(x01, y01);
            ctx.arc(0.0, 0.0, r1, a01, a11, !cw);
        } else {
            ctx.move_to(x01, y01);
        }

        // The inner ring: with rounded corners, an arc, or a point (a
        // circular sector, or an annular one collapsed by its padding).
        let ring = r0 > EPSILON && da0 > EPSILON;
        if ring && rc0 > EPSILON {
            let t0 = Corner::new([x10, y10, x11, y11], r0, -rc0, cw);
            let t1 = Corner::new([x01, y01, x00, y00], r0, -rc0, cw);
            ctx.line_to(t0.cx + t0.x01, t0.cy + t0.y01);
            if rc0 < rc {
                ctx.arc(
                    t0.cx,
                    t0.cy,
                    rc0,
                    t0.y01.atan2(t0.x01),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            } else {
                ctx.arc(
                    t0.cx,
                    t0.cy,
                    rc0,
                    t0.y01.atan2(t0.x01),
                    t0.y11.atan2(t0.x11),
                    !cw,
                );
                let start = (t0.cy + t0.y11).atan2(t0.cx + t0.x11);
                let end = (t1.cy + t1.y11).atan2(t1.cx + t1.x11);
                ctx.arc(0.0, 0.0, r0, start, end, cw);
                ctx.arc(
                    t1.cx,
                    t1.cy,
                    rc0,
                    t1.y11.atan2(t1.x11),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            }
        } else if ring {
            ctx.arc(0.0, 0.0, r0, a10, a00, cw);
        } else {
            ctx.line_to(x10, y10);
        }
    }
}

impl Corner {
    /// d3's `cornerTangents(x0, y0, x1, y1, r1, rc, cw)`: of the two
    /// tangent circles, the one nearer the edge's midpoint.
    fn new([x0, y0, x1, y1]: [f64; 4], r1: f64, rc: f64, cw: bool) -> Corner {
        let (x01, y01) = (x0 - x1, y0 - y1);
        let lo = (if cw { rc } else { -rc }) / (x01 * x01 + y01 * y01).sqrt();
        let (ox, oy) = (lo * y01, -lo * x01);
        let (x11, y11) = (x0 + ox, y0 + oy);
        let (x10, y10) = (x1 + ox, y1 + oy);
        let (x00, y00) = ((x11 + x10) / 2.0, (y11 + y10) / 2.0);
        let (dx, dy) = (x10 - x11, y10 - y11);
        let d2 = dx * dx + dy * dy;
        let r = r1 - rc;
        let big_d = x11 * y10 - x10 * y11;
        let sign = if dy < 0.0 { -1.0 } else { 1.0 };
        let d = sign * js::max(0.0, r * r * d2 - big_d * big_d).sqrt();
        let (mut cx, mut cy) = ((big_d * dy - dx * d) / d2, (-big_d * dx - dy * d) / d2);
        let (cx1, cy1) = ((big_d * dy + dx * d) / d2, (-big_d * dx + dy * d) / d2);
        let (dx0, dy0) = (cx - x00, cy - y00);
        let (dx1, dy1) = (cx1 - x00, cy1 - y00);
        if dx0 * dx0 + dy0 * dy0 > dx1 * dx1 + dy1 * dy1 {
            (cx, cy) = (cx1, cy1);
        }
        Corner {
            cx,
            cy,
            x01: -ox,
            y01: -oy,
            x11: cx * (r1 / r - 1.0),
            y11: cy * (r1 / r - 1.0),
        }
    }
}

/// d3's `intersect`: where the line through the first pair of points meets
/// the line through the second, or `None` when they are (nearly) parallel.
fn intersect([x0, y0, x1, y1]: [f64; 4], [x2, y2, x3, y3]: [f64; 4]) -> Option<(f64, f64)> {
    let (x10, y10) = (x1 - x0, y1 - y0);
    let (x32, y32) = (x3 - x2, y3 - y2);
    let t = y32 * x10 - x32 * y10;
    if t * t < EPSILON {
        return None;
    }
    let t = (x32 * (y0 - y2) - y32 * (x0 - x2)) / t;
    Some((x0 + t * x10, y0 + t * y10))
}

/// d3's `acos`, clamped: 0 above 1, π below −1.
fn acos(x: f64) -> f64 {
    if x > 1.0 {
        0.0
    } else if x < -1.0 {
        PI
    } else {
        x.acos()
    }
}

/// d3's `asin`, clamped: ±π/2 beyond ±1.
fn asin(x: f64) -> f64 {
    if x >= 1.0 {
        FRAC_PI_2
    } else if x <= -1.0 {
        -FRAC_PI_2
    } else {
        x.asin()
    }
}
