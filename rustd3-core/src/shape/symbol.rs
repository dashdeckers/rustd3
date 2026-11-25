//! d3's symbols (`symbol/*.js`), seven designed to be filled and seven to
//! be stroked with the circle in both, and Observable Plot's hexagon.

use std::f64::consts::{PI, TAU};

use crate::path::PathContext;

/// One of d3's symbols, or Observable Plot's hexagon, drawn centered at the
/// origin with a given area. The fill symbols ([`Symbol::FILL`]) are closed
/// shapes to fill; the stroke symbols ([`Symbol::STROKE`]) are designed to be
/// stroked 1.5 px wide with round caps, and three of them are open lines
/// ([`Symbol::is_open`]).
///
/// ```
/// use rustd3_core::path;
/// use rustd3_core::shape::Symbol;
///
/// assert_eq!(path::svg(|p| Symbol::Square.draw(64.0, p)), "M-4,-4h8v8h-8Z");
/// assert_eq!(Symbol::FILL[0], Symbol::Circle);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    /// A circle.
    Circle,
    /// A Greek cross, arms as wide as the center square.
    Cross,
    /// A rhombus, taller than wide.
    Diamond,
    /// An axis-aligned square.
    Square,
    /// A five-pointed star.
    Star,
    /// An equilateral triangle pointing up.
    Triangle,
    /// A Y shape of three arms.
    Wye,
    /// Two crossing lines, upright: d3's `symbolPlus`, for stroking.
    Plus,
    /// Two crossing lines, diagonal: d3's `symbolTimes`, for stroking.
    Times,
    /// A triangle pointing up, sized for stroking: d3's `symbolTriangle2`.
    Triangle2,
    /// Three crossing lines: d3's `symbolAsterisk`, for stroking.
    Asterisk,
    /// A square sized for stroking: d3's `symbolSquare2`.
    Square2,
    /// A square on its corner, sized for stroking: d3's `symbolDiamond2`.
    Diamond2,
    /// A hexagon standing on a corner, as wide as the circle of the same
    /// size and so a little larger: Observable Plot's `hexagon`.
    Hexagon,
}

impl Symbol {
    /// d3.symbolsFill, in d3's order: circle cross diamond square star
    /// triangle wye.
    pub const FILL: [Symbol; 7] = [
        Symbol::Circle,
        Symbol::Cross,
        Symbol::Diamond,
        Symbol::Square,
        Symbol::Star,
        Symbol::Triangle,
        Symbol::Wye,
    ];

    /// d3.symbolsStroke, in d3's order: circle plus times triangle2
    /// asterisk square2 diamond2.
    pub const STROKE: [Symbol; 7] = [
        Symbol::Circle,
        Symbol::Plus,
        Symbol::Times,
        Symbol::Triangle2,
        Symbol::Asterisk,
        Symbol::Square2,
        Symbol::Diamond2,
    ];

    /// Whether the symbol is lines without an inside (plus, times,
    /// asterisk), which can only be stroked.
    #[must_use]
    pub fn is_open(self) -> bool {
        matches!(self, Symbol::Plus | Symbol::Times | Symbol::Asterisk)
    }

    /// Draws the symbol centered at the origin with area `size` (d3's
    /// default 64), as d3's symbol types draw into a context.
    pub fn draw(self, size: f64, ctx: &mut impl PathContext) {
        match self {
            Symbol::Circle => {
                let r = (size / PI).sqrt();
                ctx.move_to(r, 0.0);
                ctx.arc(0.0, 0.0, r, 0.0, TAU, false);
            }
            Symbol::Cross => cross(size, ctx),
            Symbol::Diamond => {
                let tan30 = (1.0_f64 / 3.0).sqrt();
                let y = (size / (tan30 * 2.0)).sqrt();
                let x = y * tan30;
                ctx.move_to(0.0, -y);
                ctx.line_to(x, 0.0);
                ctx.line_to(0.0, y);
                ctx.line_to(-x, 0.0);
                ctx.close_path();
            }
            Symbol::Square => {
                let w = size.sqrt();
                let x = -w / 2.0;
                ctx.rect(x, x, w, w);
            }
            Symbol::Star => star(size, ctx),
            Symbol::Triangle => {
                let sqrt3 = 3.0_f64.sqrt();
                let y = -(size / (sqrt3 * 3.0)).sqrt();
                ctx.move_to(0.0, y * 2.0);
                ctx.line_to(-sqrt3 * y, -y);
                ctx.line_to(sqrt3 * y, -y);
                ctx.close_path();
            }
            Symbol::Wye => wye(size, ctx),
            Symbol::Plus => {
                let r = (size - (size / 7.0).min(2.0)).sqrt() * 0.87559;
                ctx.move_to(-r, 0.0);
                ctx.line_to(r, 0.0);
                ctx.move_to(0.0, r);
                ctx.line_to(0.0, -r);
            }
            Symbol::Times => {
                let r = (size - (size / 6.0).min(1.7)).sqrt() * 0.6189;
                ctx.move_to(-r, -r);
                ctx.line_to(r, r);
                ctx.move_to(-r, r);
                ctx.line_to(r, -r);
            }
            Symbol::Triangle2 => {
                let s = size.sqrt() * 0.6824;
                let (t, u) = (s / 2.0, s * 3.0_f64.sqrt() / 2.0);
                ctx.move_to(0.0, -s);
                ctx.line_to(u, t);
                ctx.line_to(-u, t);
                ctx.close_path();
            }
            Symbol::Asterisk => asterisk(size, ctx),
            Symbol::Square2 => {
                let r = size.sqrt() * 0.4431;
                ctx.move_to(r, r);
                ctx.line_to(r, -r);
                ctx.line_to(-r, -r);
                ctx.line_to(-r, r);
                ctx.close_path();
            }
            Symbol::Diamond2 => {
                let r = size.sqrt() * 0.62625;
                ctx.move_to(0.0, -r);
                ctx.line_to(r, 0.0);
                ctx.line_to(0.0, r);
                ctx.line_to(-r, 0.0);
                ctx.close_path();
            }
            Symbol::Hexagon => hexagon(size, ctx),
        }
    }
}

/// `symbol/cross.js`: twelve corners around arms of half-width r.
fn cross(size: f64, ctx: &mut impl PathContext) {
    let r = (size / 5.0).sqrt() / 2.0;
    ctx.move_to(-3.0 * r, -r);
    let corners = [
        (-r, -r),
        (-r, -3.0 * r),
        (r, -3.0 * r),
        (r, -r),
        (3.0 * r, -r),
        (3.0 * r, r),
        (r, r),
        (r, 3.0 * r),
        (-r, 3.0 * r),
        (-r, r),
        (-3.0 * r, r),
    ];
    for (x, y) in corners {
        ctx.line_to(x, y);
    }
    ctx.close_path();
}

/// `symbol/star.js`: five points at radius r, the inner corners at
/// kr · r, the area fixed by ka.
fn star(size: f64, ctx: &mut impl PathContext) {
    const KA: f64 = 0.890_813_091_529_285_2;
    let kr = (PI / 10.0).sin() / (7.0 * PI / 10.0).sin();
    let kx = (TAU / 10.0).sin() * kr;
    let ky = -(TAU / 10.0).cos() * kr;
    let r = (size * KA).sqrt();
    let (x, y) = (kx * r, ky * r);
    ctx.move_to(0.0, -r);
    ctx.line_to(x, y);
    for i in 1..5 {
        let a = TAU * f64::from(i) / 5.0;
        let (c, s) = (a.cos(), a.sin());
        ctx.line_to(s * r, -c * r);
        ctx.line_to(c * x - s * y, s * x + c * y);
    }
    ctx.close_path();
}

/// `symbol/wye.js`: one arm, then the arm turned by ±120°.
fn wye(size: f64, ctx: &mut impl PathContext) {
    let (c, s) = (-0.5, 3.0_f64.sqrt() / 2.0);
    let k = 1.0 / 12.0_f64.sqrt();
    let a = (k / 2.0 + 1.0) * 3.0;
    let r = (size / a).sqrt();
    let (x0, y0) = (r / 2.0, r * k);
    let (x1, y1) = (x0, r * k + r);
    let (x2, y2) = (-x1, y1);
    ctx.move_to(x0, y0);
    ctx.line_to(x1, y1);
    ctx.line_to(x2, y2);
    for (x, y) in [(x0, y0), (x1, y1), (x2, y2)] {
        ctx.line_to(c * x - s * y, s * x + c * y);
    }
    for (x, y) in [(x0, y0), (x1, y1), (x2, y2)] {
        ctx.line_to(c * x + s * y, c * y - s * x);
    }
    ctx.close_path();
}

/// `symbol/asterisk.js`: one upright line and two turned by ±60°.
fn asterisk(size: f64, ctx: &mut impl PathContext) {
    let r = (size + (size / 28.0).min(0.75)).sqrt() * 0.59436;
    let t = r / 2.0;
    let u = t * 3.0_f64.sqrt();
    ctx.move_to(0.0, r);
    ctx.line_to(0.0, -r);
    ctx.move_to(-u, -t);
    ctx.line_to(u, t);
    ctx.move_to(-u, t);
    ctx.line_to(u, -t);
}

/// Observable Plot's `symbolHexagon` (`src/symbol.js`): half-width rx as a
/// circle's radius of the same area, corners at ±ry and ±ry / 2.
fn hexagon(size: f64, ctx: &mut impl PathContext) {
    let rx = (size / PI).sqrt();
    let ry = rx * 2.0 / 3.0_f64.sqrt();
    let hy = ry / 2.0;
    ctx.move_to(0.0, ry);
    ctx.line_to(rx, hy);
    ctx.line_to(rx, -hy);
    ctx.line_to(0.0, -ry);
    ctx.line_to(-rx, -hy);
    ctx.line_to(-rx, hy);
    ctx.close_path();
}
