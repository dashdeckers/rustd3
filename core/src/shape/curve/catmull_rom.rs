//! d3's curveCatmullRom, curveCatmullRomClosed and curveCatmullRomOpen:
//! Catmull–Rom splines parameterized by chord length to the power alpha
//! (0.5 centripetal, 1 chordal). Alpha 0 is d3's cardinal spline of
//! tension 0, which `State::new` constructs instead.

use super::Line;
use crate::path::PathContext;

/// d3-shape's `epsilon`: shorter chords leave a tangent as the cardinal
/// one.
const EPSILON: f64 = 1e-12;

const UNSET: (f64, f64) = (f64::NAN, f64::NAN);

/// d3's curveCatmullRom.
pub(crate) struct CatmullRom {
    line: Line,
    /// Points of the run so far, counted up to 3 (d3's `_point`).
    point: u8,
    chords: Chords,
}

/// d3's curveCatmullRomClosed, which has no area flag. The first three
/// points are drawn again at the end.
pub(crate) struct CatmullRomClosed {
    point: u8,
    chords: Chords,
    /// The run's first three points (d3's `_x3` … `_y5`).
    first: [(f64, f64); 3],
}

/// d3's curveCatmullRomOpen: from the second point to the second-to-last.
pub(crate) struct CatmullRomOpen {
    line: Line,
    /// Points of the run so far, counted up to 4.
    point: u8,
    chords: Chords,
}

/// The last three points and the chord lengths between them and to the
/// next one, as `l^alpha` and `l^(2 alpha)` (d3's `_l01_a`, `_l01_2a`, …).
struct Chords {
    alpha: f64,
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    l01_a: f64,
    l12_a: f64,
    l23_a: f64,
    l01_2a: f64,
    l12_2a: f64,
    l23_2a: f64,
}

impl CatmullRom {
    pub(crate) fn new(alpha: f64) -> CatmullRom {
        CatmullRom {
            line: Line::Unset,
            point: 0,
            chords: Chords::new(alpha),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.chords.clear();
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        let (x2, y2) = self.chords.p2;
        match self.point {
            2 => ctx.line_to(x2, y2),
            3 => self.point(x2, y2, ctx),
            _ => {}
        }
        if self.line.closes(self.point == 1) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if self.point != 0 {
            self.chords.measure((x, y));
        }
        match self.point {
            0 => {
                self.point = 1;
                self.line.start(ctx, x, y);
            }
            1 => self.point = 2,
            _ => {
                self.point = 3;
                self.chords.bezier(ctx, (x, y));
            }
        }
        self.chords.push((x, y));
    }
}

impl CatmullRomClosed {
    pub(crate) fn new(alpha: f64) -> CatmullRomClosed {
        CatmullRomClosed {
            point: 0,
            chords: Chords::new(alpha),
            first: [UNSET; 3],
        }
    }

    pub(crate) fn line_start(&mut self) {
        self.chords.clear();
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
        if self.point != 0 {
            self.chords.measure((x, y));
        }
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
            _ => self.chords.bezier(ctx, (x, y)),
        }
        self.chords.push((x, y));
    }
}

impl CatmullRomOpen {
    pub(crate) fn new(alpha: f64) -> CatmullRomOpen {
        CatmullRomOpen {
            line: Line::Unset,
            point: 0,
            chords: Chords::new(alpha),
        }
    }

    pub(crate) fn area_start(&mut self) {
        self.line = Line::Zero;
    }

    pub(crate) fn area_end(&mut self) {
        self.line = Line::Unset;
    }

    pub(crate) fn line_start(&mut self) {
        self.chords.clear();
        self.point = 0;
    }

    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        if self.line.closes(self.point == 3) {
            ctx.close_path();
        }
        self.line = self.line.toggle();
    }

    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        if self.point != 0 {
            self.chords.measure((x, y));
        }
        match self.point {
            0 | 1 => self.point += 1,
            2 => {
                self.point = 3;
                let (x2, y2) = self.chords.p2;
                self.line.start(ctx, x2, y2);
            }
            _ => {
                self.point = 4;
                self.chords.bezier(ctx, (x, y));
            }
        }
        self.chords.push((x, y));
    }
}

impl Chords {
    fn new(alpha: f64) -> Chords {
        Chords {
            alpha,
            p0: UNSET,
            p1: UNSET,
            p2: UNSET,
            l01_a: 0.0,
            l12_a: 0.0,
            l23_a: 0.0,
            l01_2a: 0.0,
            l12_2a: 0.0,
            l23_2a: 0.0,
        }
    }

    /// d3's `lineStart`: no points, every chord 0.
    fn clear(&mut self) {
        (self.p0, self.p1, self.p2) = (UNSET, UNSET, UNSET);
        (self.l01_a, self.l12_a, self.l23_a) = (0.0, 0.0, 0.0);
        (self.l01_2a, self.l12_2a, self.l23_2a) = (0.0, 0.0, 0.0);
    }

    /// Measures the chord from `p2` to the next point `p`.
    fn measure(&mut self, (x, y): (f64, f64)) {
        let (dx, dy) = (self.p2.0 - x, self.p2.1 - y);
        self.l23_2a = (dx * dx + dy * dy).powf(self.alpha);
        self.l23_a = self.l23_2a.sqrt();
    }

    /// d3's catmullRom `point(that, x, y)`: the segment from `p1` to `p2`,
    /// its control points moved by the chords to `p0` and the next point
    /// `p`.
    fn bezier(&self, ctx: &mut impl PathContext, (x, y): (f64, f64)) {
        let ((x0, y0), (mut x1, mut y1), (mut x2, mut y2)) = (self.p0, self.p1, self.p2);
        if self.l01_a > EPSILON {
            let a = 2.0 * self.l01_2a + 3.0 * self.l01_a * self.l12_a + self.l12_2a;
            let n = 3.0 * self.l01_a * (self.l01_a + self.l12_a);
            x1 = (x1 * a - x0 * self.l12_2a + self.p2.0 * self.l01_2a) / n;
            y1 = (y1 * a - y0 * self.l12_2a + self.p2.1 * self.l01_2a) / n;
        }
        if self.l23_a > EPSILON {
            let b = 2.0 * self.l23_2a + 3.0 * self.l23_a * self.l12_a + self.l12_2a;
            let m = 3.0 * self.l23_a * (self.l23_a + self.l12_a);
            x2 = (x2 * b + self.p1.0 * self.l23_2a - x * self.l12_2a) / m;
            y2 = (y2 * b + self.p1.1 * self.l23_2a - y * self.l12_2a) / m;
        }
        ctx.bezier_curve_to(x1, y1, x2, y2, self.p2.0, self.p2.1);
    }

    /// Slides the window and the chords on to `p`.
    fn push(&mut self, p: (f64, f64)) {
        (self.l01_a, self.l12_a) = (self.l12_a, self.l23_a);
        (self.l01_2a, self.l12_2a) = (self.l12_2a, self.l23_2a);
        (self.p0, self.p1, self.p2) = (self.p1, self.p2, p);
    }
}
