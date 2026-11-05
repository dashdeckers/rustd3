//! d3's curveBasis, curveBasisClosed and curveBasisOpen: the uniform cubic
//! B-spline through the points as control points.

use super::Line;
use crate::path::PathContext;

const UNSET: (f64, f64) = (f64::NAN, f64::NAN);

/// d3's curveBasis, clamped to the first and last point.
pub(crate) struct Basis {
    line: Line,
    /// Points of the run so far, counted up to 3 (d3's `_point`).
    point: u8,
    /// The two previous points (d3's `_x0, _y0` and `_x1, _y1`).
    p0: (f64, f64),
    p1: (f64, f64),
}

/// d3's curveBasisClosed, which has no area flag. The first three points
/// are drawn again at the end.
pub(crate) struct BasisClosed {
    point: u8,
    p0: (f64, f64),
    p1: (f64, f64),
    /// The run's first three points (d3's `_x2` … `_y4`).
    first: [(f64, f64); 3],
}

/// d3's curveBasisOpen: starts at the second point's knot, ends at the
/// second-to-last one.
pub(crate) struct BasisOpen {
    line: Line,
    /// Points of the run so far, counted up to 4.
    point: u8,
    p0: (f64, f64),
    p1: (f64, f64),
}

impl Basis {
    pub(crate) fn new() -> Basis {
        Basis {
            line: Line::Unset,
            point: 0,
            p0: UNSET,
            p1: UNSET,
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        (self.p0, self.p1) = (UNSET, UNSET);
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.point == 3 {
            bezier(ctx, self.p0, self.p1, self.p1);
        }
        if self.point >= 2 {
            ctx.line_to(self.p1.0, self.p1.1);
        }
        if self.line.closes(self.point == 1) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        match self.point {
            0 => {
                self.point = 1;
                self.line.start(ctx, x, y);
            }
            1 => self.point = 2,
            2 => {
                self.point = 3;
                let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
                ctx.line_to((5.0 * x0 + x1) / 6.0, (5.0 * y0 + y1) / 6.0);
                bezier(ctx, self.p0, self.p1, (x, y));
            }
            _ => bezier(ctx, self.p0, self.p1, (x, y)),
        }
        (self.p0, self.p1) = (self.p1, (x, y));
    }
}

impl BasisClosed {
    pub(crate) fn new() -> BasisClosed {
        BasisClosed {
            point: 0,
            p0: UNSET,
            p1: UNSET,
            first: [UNSET; 3],
        }
    }

    pub(crate) fn line_start(&mut self) {
        (self.p0, self.p1, self.first) = (UNSET, UNSET, [UNSET; 3]);
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        let [(x2, y2), (x3, y3), _] = self.first;
        match self.point {
            1 => {
                ctx.move_to(x2, y2);
                ctx.close_path();
            }
            2 => {
                ctx.move_to((x2 + 2.0 * x3) / 3.0, (y2 + 2.0 * y3) / 3.0);
                ctx.line_to((x3 + 2.0 * x2) / 3.0, (y3 + 2.0 * y2) / 3.0);
                ctx.close_path();
            }
            3 => {
                for (x, y) in self.first {
                    self.point(x, y, ctx);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        match self.point {
            0 | 1 => {
                self.first[usize::from(self.point)] = (x, y);
                self.point += 1;
            }
            2 => {
                self.point = 3;
                self.first[2] = (x, y);
                let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
                ctx.move_to((x0 + 4.0 * x1 + x) / 6.0, (y0 + 4.0 * y1 + y) / 6.0);
            }
            _ => bezier(ctx, self.p0, self.p1, (x, y)),
        }
        (self.p0, self.p1) = (self.p1, (x, y));
    }
}

impl BasisOpen {
    pub(crate) fn new() -> BasisOpen {
        BasisOpen {
            line: Line::Unset,
            point: 0,
            p0: UNSET,
            p1: UNSET,
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        (self.p0, self.p1) = (UNSET, UNSET);
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.line.closes(self.point == 3) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        match self.point {
            0 | 1 => self.point += 1,
            2 => {
                self.point = 3;
                let ((x0, y0), (x1, y1)) = (self.p0, self.p1);
                let start = ((x0 + 4.0 * x1 + x) / 6.0, (y0 + 4.0 * y1 + y) / 6.0);
                self.line.start(ctx, start.0, start.1);
            }
            _ => {
                self.point = 4;
                bezier(ctx, self.p0, self.p1, (x, y));
            }
        }
        (self.p0, self.p1) = (self.p1, (x, y));
    }
}

/// d3's basis `point(that, x, y)`: the B-spline segment of `p0`, `p1` and
/// the next point `p`.
fn bezier(
    ctx: &mut impl PathContext,
    (x0, y0): (f64, f64),
    (x1, y1): (f64, f64),
    (x, y): (f64, f64),
) {
    ctx.bezier_curve_to(
        (2.0 * x0 + x1) / 3.0,
        (2.0 * y0 + y1) / 3.0,
        (x0 + 2.0 * x1) / 3.0,
        (y0 + 2.0 * y1) / 3.0,
        (x0 + 4.0 * x1 + x) / 6.0,
        (y0 + 4.0 * y1 + y) / 6.0,
    );
}
