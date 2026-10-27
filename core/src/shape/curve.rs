//! d3-shape's curves: [`Curve`], the closed set of ways a line or an area
//! joins its points, and `State`, the state machine each one draws with
//! (d3's curve objects, verbatim in structure).

mod basis;
mod bump;
mod bundle;
mod cardinal;
mod catmull_rom;
mod linear;
mod monotone;
mod natural;
mod step;

use crate::js;
use crate::path::PathContext;

/// How consecutive points are joined: d3's curve factories.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    /// d3.curveLinear: a straight segment to each point.
    #[default]
    Linear,
    /// d3.curveLinearClosed: straight segments, closed back to the first
    /// point.
    LinearClosed,
    /// d3.curveStep: horizontal and vertical segments, the vertical one
    /// halfway between points.
    Step,
    /// d3.curveStepBefore: the vertical segment at the previous point.
    StepBefore,
    /// d3.curveStepAfter: the vertical segment at the next point.
    StepAfter,
    /// d3.curveBasis: a cubic B-spline with the points as control points,
    /// from the first point to the last.
    Basis,
    /// d3.curveBasisClosed: a closed cubic B-spline.
    BasisClosed,
    /// d3.curveBasisOpen: a cubic B-spline that starts and ends at the
    /// inner knots.
    BasisOpen,
    /// d3.curveBumpX: cubics with horizontal tangents at each point.
    BumpX,
    /// d3.curveBumpY: cubics with vertical tangents at each point.
    BumpY,
    /// d3.curveBundle: a basis spline straightened toward the line from the
    /// first point to the last. d3's default beta is 0.85; beta 1 is
    /// `Basis`. An area draws it as `Basis`.
    Bundle {
        /// The bundle strength in [0, 1]: 0 is the straight line.
        beta: f64,
    },
    /// d3.curveCardinal: a cubic cardinal spline through the points. d3's
    /// default tension is 0.
    Cardinal {
        /// The tension in [0, 1]: 1 draws straight segments.
        tension: f64,
    },
    /// d3.curveCardinalClosed: a closed cardinal spline.
    CardinalClosed {
        /// The tension in [0, 1]: 1 draws straight segments.
        tension: f64,
    },
    /// d3.curveCardinalOpen: a cardinal spline from the second point to the
    /// second-to-last.
    CardinalOpen {
        /// The tension in [0, 1]: 1 draws straight segments.
        tension: f64,
    },
    /// d3.curveCatmullRom: a Catmull–Rom spline through the points. d3's
    /// default alpha is 0.5 (centripetal); alpha 0 is `Cardinal { tension:
    /// 0 }`.
    CatmullRom {
        /// The chord-length exponent in [0, 1]: 0.5 centripetal, 1 chordal.
        alpha: f64,
    },
    /// d3.curveCatmullRomClosed: a closed Catmull–Rom spline.
    CatmullRomClosed {
        /// The chord-length exponent in [0, 1]: 0.5 centripetal, 1 chordal.
        alpha: f64,
    },
    /// d3.curveCatmullRomOpen: a Catmull–Rom spline from the second point to
    /// the second-to-last.
    CatmullRomOpen {
        /// The chord-length exponent in [0, 1]: 0.5 centripetal, 1 chordal.
        alpha: f64,
    },
    /// d3.curveMonotoneX: a cubic spline that stays monotone in y between
    /// points, for points monotone in x.
    MonotoneX,
    /// d3.curveMonotoneY: a cubic spline that stays monotone in x between
    /// points, for points monotone in y.
    MonotoneY,
    /// d3.curveNatural: the natural cubic spline, its second derivative zero
    /// at both ends.
    Natural,
}

/// d3's `_line`, a curve's place in an area: unset for a plain line (d3's
/// `undefined`, and `NaN` after `areaEnd`), `Zero` while it draws an
/// area's top and `One` along the baseline back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Line {
    #[default]
    Unset,
    Zero,
    One,
}

/// The drawing state of one curve: d3's curve object, one variant per
/// family, dispatched by `match`.
pub(crate) enum State {
    Linear(linear::Linear),
    LinearClosed(linear::LinearClosed),
    Step(step::Step),
    Basis(basis::Basis),
    BasisClosed(basis::BasisClosed),
    BasisOpen(basis::BasisOpen),
    Bump(bump::Bump),
    Bundle(bundle::Bundle),
    Cardinal(cardinal::Cardinal),
    CardinalClosed(cardinal::CardinalClosed),
    CardinalOpen(cardinal::CardinalOpen),
    CatmullRom(catmull_rom::CatmullRom),
    CatmullRomClosed(catmull_rom::CatmullRomClosed),
    CatmullRomOpen(catmull_rom::CatmullRomOpen),
    Monotone(monotone::Monotone),
    Natural(natural::Natural),
}

/// d3's `curveRadial` point (`curve/radial.js`): the radial forms place
/// (angle, radius) at `(r·sin a, −r·cos a)` before their curve sees it.
pub(crate) fn radial(angle: f64, radius: f64) -> (f64, f64) {
    (radius * angle.sin(), radius * -angle.cos())
}

/// `$body` with `$family` bound to the family `$state` holds.
macro_rules! each {
    ($state:expr, $family:ident => $body:expr) => {
        match $state {
            State::Linear($family) => $body,
            State::LinearClosed($family) => $body,
            State::Step($family) => $body,
            State::Basis($family) => $body,
            State::BasisClosed($family) => $body,
            State::BasisOpen($family) => $body,
            State::Bump($family) => $body,
            State::Bundle($family) => $body,
            State::Cardinal($family) => $body,
            State::CardinalClosed($family) => $body,
            State::CardinalOpen($family) => $body,
            State::CatmullRom($family) => $body,
            State::CatmullRomClosed($family) => $body,
            State::CatmullRomOpen($family) => $body,
            State::Monotone($family) => $body,
            State::Natural($family) => $body,
        }
    };
}

impl Line {
    /// d3's `_line || (_line !== 0 && lone)`: whether `lineEnd` closes the
    /// path. `lone` is the curve's test for a run that drew one point.
    fn closes(self, lone: bool) -> bool {
        self == Line::One || (self != Line::Zero && lone)
    }

    /// d3's `_line = 1 - _line`, which leaves an unset flag unset.
    fn toggle(self) -> Line {
        match self {
            Line::Zero => Line::One,
            Line::One => Line::Zero,
            Line::Unset => Line::Unset,
        }
    }

    /// d3's `_line ? lineTo : moveTo` for a run's first point: the
    /// baseline continues the ring the top began.
    fn start(self, ctx: &mut impl PathContext, x: f64, y: f64) {
        if self == Line::One {
            ctx.line_to(x, y);
        } else {
            ctx.move_to(x, y);
        }
    }
}

impl State {
    /// The state `curve` draws a line with.
    pub(crate) fn new(curve: Curve) -> State {
        match curve {
            Curve::Linear => State::Linear(linear::Linear::default()),
            Curve::LinearClosed => State::LinearClosed(linear::LinearClosed::default()),
            Curve::Step => State::Step(step::Step::new(0.5)),
            Curve::StepBefore => State::Step(step::Step::new(0.0)),
            Curve::StepAfter => State::Step(step::Step::new(1.0)),
            Curve::Basis => State::Basis(basis::Basis::new()),
            Curve::BasisClosed => State::BasisClosed(basis::BasisClosed::new()),
            Curve::BasisOpen => State::BasisOpen(basis::BasisOpen::new()),
            Curve::BumpX => State::Bump(bump::Bump::new(true)),
            Curve::BumpY => State::Bump(bump::Bump::new(false)),
            Curve::Bundle { beta: 1.0 } => State::Basis(basis::Basis::new()),
            Curve::Bundle { beta } => State::Bundle(bundle::Bundle::new(beta)),
            Curve::Cardinal { tension } => State::Cardinal(cardinal::Cardinal::new(tension)),
            Curve::CardinalClosed { tension } => {
                State::CardinalClosed(cardinal::CardinalClosed::new(tension))
            }
            Curve::CardinalOpen { tension } => {
                State::CardinalOpen(cardinal::CardinalOpen::new(tension))
            }
            // d3 takes the cardinal spline for an alpha that is not truthy.
            Curve::CatmullRom { alpha } if js::truthy(alpha) => {
                State::CatmullRom(catmull_rom::CatmullRom::new(alpha))
            }
            Curve::CatmullRom { .. } => State::Cardinal(cardinal::Cardinal::new(0.0)),
            Curve::CatmullRomClosed { alpha } if js::truthy(alpha) => {
                State::CatmullRomClosed(catmull_rom::CatmullRomClosed::new(alpha))
            }
            Curve::CatmullRomClosed { .. } => {
                State::CardinalClosed(cardinal::CardinalClosed::new(0.0))
            }
            Curve::CatmullRomOpen { alpha } if js::truthy(alpha) => {
                State::CatmullRomOpen(catmull_rom::CatmullRomOpen::new(alpha))
            }
            Curve::CatmullRomOpen { .. } => State::CardinalOpen(cardinal::CardinalOpen::new(0.0)),
            Curve::MonotoneX => State::Monotone(monotone::Monotone::new(false)),
            Curve::MonotoneY => State::Monotone(monotone::Monotone::new(true)),
            Curve::Natural => State::Natural(natural::Natural::new()),
        }
    }

    /// The state `curve` draws an area with: a line's, except that `Bundle`
    /// draws as `Basis` (d3's bundle has no `areaStart` and throws there).
    pub(crate) fn area(curve: Curve) -> State {
        if let Curve::Bundle { .. } = curve {
            State::new(Curve::Basis)
        } else {
            State::new(curve)
        }
    }

    /// d3's `areaStart`: the next line is an area's top.
    pub(crate) fn area_start(&mut self) {
        match self {
            State::Linear(s) => s.area_start(),
            State::Step(s) => s.area_start(),
            State::Basis(s) => s.area_start(),
            State::BasisOpen(s) => s.area_start(),
            State::Bump(s) => s.area_start(),
            State::Cardinal(s) => s.area_start(),
            State::CardinalOpen(s) => s.area_start(),
            State::CatmullRom(s) => s.area_start(),
            State::CatmullRomOpen(s) => s.area_start(),
            State::Monotone(s) => s.area_start(),
            State::Natural(s) => s.area_start(),
            // d3's closed curves ignore areaStart and areaEnd; `State::area`
            // holds no bundle.
            State::LinearClosed(_)
            | State::BasisClosed(_)
            | State::CardinalClosed(_)
            | State::CatmullRomClosed(_)
            | State::Bundle(_) => {}
        }
    }

    /// d3's `areaEnd`.
    pub(crate) fn area_end(&mut self) {
        match self {
            State::Linear(s) => s.area_end(),
            State::Step(s) => s.area_end(),
            State::Basis(s) => s.area_end(),
            State::BasisOpen(s) => s.area_end(),
            State::Bump(s) => s.area_end(),
            State::Cardinal(s) => s.area_end(),
            State::CardinalOpen(s) => s.area_end(),
            State::CatmullRom(s) => s.area_end(),
            State::CatmullRomOpen(s) => s.area_end(),
            State::Monotone(s) => s.area_end(),
            State::Natural(s) => s.area_end(),
            State::LinearClosed(_)
            | State::BasisClosed(_)
            | State::CardinalClosed(_)
            | State::CatmullRomClosed(_)
            | State::Bundle(_) => {}
        }
    }

    /// d3's `lineStart`: a new run of points.
    pub(crate) fn line_start(&mut self) {
        each!(self, s => s.line_start());
    }

    /// d3's `lineEnd`: draws what the run still holds and closes it if the
    /// curve does.
    pub(crate) fn line_end(&mut self, ctx: &mut impl PathContext) {
        each!(self, s => s.line_end(ctx));
    }

    /// d3's `point(x, y)`.
    pub(crate) fn point(&mut self, x: f64, y: f64, ctx: &mut impl PathContext) {
        each!(self, s => s.point(x, y, ctx));
    }
}
