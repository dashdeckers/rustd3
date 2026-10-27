//! d3's curveCardinal, curveCardinalClosed and curveCardinalOpen: cubic
//! cardinal splines through the points.

use super::Line;
use crate::path::PathContext;

const UNSET: (f64, f64) = (f64::NAN, f64::NAN);

/// d3's curveCardinal.
pub(crate) struct Cardinal {
    line: Line,
    /// Points of the run so far, counted up to 3 (d3's `_point`).
    point: u8,
    window: Window,
}

/// d3's curveCardinalClosed: no area flag; the first three points are
/// drawn again at the end.
pub(crate) struct CardinalClosed {
    point: u8,
    window: Window,
    /// The run's first three points (d3's `_x3` … `_y5`).
    first: [(f64, f64); 3],
}

/// d3's curveCardinalOpen: from the second point to the second-to-last.
pub(crate) struct CardinalOpen {
    line: Line,
    /// Points of the run so far, counted up to 4.
    point: u8,
    window: Window,
}

/// The last three points (d3's `_x0, _y0` … `_x2, _y2`) and d3's `_k`,
/// (1 − tension) / 6.
struct Window {
    k: f64,
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
}

impl Cardinal {
    pub(crate) fn new(tension: f64) -> Cardinal {
        Cardinal {
            line: Line::Unset,
            point: 0,
            window: Window::new(tension),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.window.clear();
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        match self.point {
            2 => ctx.line_to(self.window.p2.0, self.window.p2.1),
            3 => self.window.bezier(ctx, self.window.p1),
            _ => {}
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
            1 => {
                self.point = 2;
                self.window.p1 = (x, y);
            }
            _ => {
                self.point = 3;
                self.window.bezier(ctx, (x, y));
            }
        }
        self.window.push((x, y));
    }
}

impl CardinalClosed {
    pub(crate) fn new(tension: f64) -> CardinalClosed {
        CardinalClosed {
            point: 0,
            window: Window::new(tension),
            first: [UNSET; 3],
        }
    }

    pub(crate) fn line_start(&mut self) {
        self.window.clear();
        self.first = [UNSET; 3];
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        let (x3, y3) = self.first[0];
        match self.point {
            1 => {
                ctx.move_to(x3, y3);
                ctx.close_path();
            }
            2 => {
                ctx.line_to(x3, y3);
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
            0 | 2 => {
                self.first[usize::from(self.point)] = (x, y);
                self.point += 1;
            }
            1 => {
                self.point = 2;
                self.first[1] = (x, y);
                ctx.move_to(x, y);
            }
            _ => self.window.bezier(ctx, (x, y)),
        }
        self.window.push((x, y));
    }
}

impl CardinalOpen {
    pub(crate) fn new(tension: f64) -> CardinalOpen {
        CardinalOpen {
            line: Line::Unset,
            point: 0,
            window: Window::new(tension),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.window.clear();
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
                let (x2, y2) = self.window.p2;
                self.line.start(ctx, x2, y2);
            }
            _ => {
                self.point = 4;
                self.window.bezier(ctx, (x, y));
            }
        }
        self.window.push((x, y));
    }
}

impl Window {
    fn new(tension: f64) -> Window {
        Window {
            k: (1.0 - tension) / 6.0,
            p0: UNSET,
            p1: UNSET,
            p2: UNSET,
        }
    }

    fn clear(&mut self) {
        (self.p0, self.p1, self.p2) = (UNSET, UNSET, UNSET);
    }

    /// d3's cardinal `point(that, x, y)`: the segment from `p1` to `p2`,
    /// its tangents from `p0` and the next point `p`.
    fn bezier(&self, ctx: &mut impl PathContext, (x, y): (f64, f64)) {
        let ((x0, y0), (x1, y1), (x2, y2), k) = (self.p0, self.p1, self.p2, self.k);
        ctx.bezier_curve_to(
            x1 + k * (x2 - x0),
            y1 + k * (y2 - y0),
            x2 + k * (x1 - x),
            y2 + k * (y1 - y),
            x2,
            y2,
        );
    }

    /// Slides the window on to `p`.
    fn push(&mut self, p: (f64, f64)) {
        (self.p0, self.p1, self.p2) = (self.p1, self.p2, p);
    }
}
