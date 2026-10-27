//! d3's seven fill symbols (`symbol/*.js`).

use std::f64::consts::{PI, TAU};

use crate::path::PathContext;

/// One of d3's fill symbols, drawn centred at the origin with a given area.
/// The set is closed; d3's stroke symbols are Later.
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
    /// A Greek cross, arms as wide as the centre square.
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

    /// Draws the symbol centred at the origin with area `size` (d3's
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
