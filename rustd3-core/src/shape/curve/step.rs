//! d3's curveStep, curveStepBefore and curveStepAfter: one state with the
//! step's position `t`.

use super::Line;
use crate::path::PathContext;

/// d3's Step: horizontal then vertical segments, the vertical one at `t`
/// of the way to the next point (0.5 Step, 0 StepBefore, 1 StepAfter).
pub(crate) struct Step {
    t: f64,
    line: Line,
    /// Points of the run so far, counted up to 2 (d3's `_point`).
    point: u8,
    /// The previous point (d3's `_x`, `_y`).
    last: (f64, f64),
}

impl Step {
    pub(crate) fn new(t: f64) -> Step {
        Step {
            t,
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
        self.last = (f64::NAN, f64::NAN);
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if 0.0 < self.t && self.t < 1.0 && self.point == 2 {
            ctx.line_to(self.last.0, self.last.1);
        }
        if self.line.closes(self.point == 1) {
            ctx.close_path();
        }
        // d3's `_line >= 0`: an area's baseline steps the other way.
        if self.line != Line::Unset {
            self.t = 1.0 - self.t;
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if self.point == 0 {
            self.point = 1;
            self.line.start(ctx, x, y);
        } else {
            self.point = 2;
            let (x0, y0) = self.last;
            if self.t <= 0.0 {
                ctx.line_to(x0, y);
                ctx.line_to(x, y);
            } else {
                let x1 = x0 * (1.0 - self.t) + x * self.t;
                ctx.line_to(x1, y0);
                ctx.line_to(x1, y);
            }
        }
        self.last = (x, y);
    }
}
