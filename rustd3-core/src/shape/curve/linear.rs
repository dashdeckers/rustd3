//! d3's curveLinear and curveLinearClosed.

use super::Line;
use crate::path::PathContext;

/// d3's curveLinear.
#[derive(Default)]
pub(crate) struct Linear {
    line: Line,
    /// Points of the run so far, counted up to 2 (d3's `_point`).
    point: u8,
}

/// d3's curveLinearClosed: no area flag, closed at every line end.
#[derive(Default)]
pub(crate) struct LinearClosed {
    /// Whether the run has its first point (d3's `_point`).
    started: bool,
}

impl Linear {
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
        if self.point == 0 {
            self.line.start(ctx, x, y);
        } else {
            ctx.line_to(x, y);
        }
        self.point = (self.point + 1).min(2);
    }
}

impl LinearClosed {
    pub(crate) fn line_start(&mut self) {
        self.started = false;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.started {
            ctx.close_path();
        }
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if self.started {
            ctx.line_to(x, y);
        } else {
            self.started = true;
            ctx.move_to(x, y);
        }
    }
}
