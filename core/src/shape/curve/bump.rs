//! d3's curveBumpX and curveBumpY: a cubic to each point whose tangents
//! are horizontal (X) or vertical (Y) at both ends.

use super::Line;
use crate::path::PathContext;

/// d3's Bump, horizontal for BumpX.
pub(crate) struct Bump {
    horizontal: bool,
    line: Line,
    /// Points of the run so far, counted up to 2 (d3's `_point`).
    point: u8,
    /// The previous point (d3's `_x0`, `_y0`).
    last: (f64, f64),
}

impl Bump {
    pub(crate) fn new(horizontal: bool) -> Bump {
        Bump {
            horizontal,
            line: Line::Unset,
            point: 0,
            last: (f64::NAN, f64::NAN),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.line.closes(self.point == 1) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        let (x0, y0) = self.last;
        if self.point == 0 {
            self.point = 1;
            self.line.start(ctx, x, y);
        } else if self.horizontal {
            self.point = 2;
            let xm = (x0 + x) / 2.0;
            ctx.bezier_curve_to(xm, y0, xm, y, x, y);
        } else {
            self.point = 2;
            let ym = (y0 + y) / 2.0;
            ctx.bezier_curve_to(x0, ym, x, ym, x, y);
        }
        self.last = (x, y);
    }
}
