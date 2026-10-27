//! d3-shape's curves: [`Curve`], the closed set of ways a line or an area
//! joins its points, and `State`, the state machine each one draws with
//! (d3's curve objects, verbatim in structure).

mod basis;
mod bump;
mod bundle;
mod linear;
mod step;

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
            // d3's closed curves ignore areaStart and areaEnd; `State::area`
            // holds no bundle.
            State::LinearClosed(_) | State::BasisClosed(_) | State::Bundle(_) => {}
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
            State::LinearClosed(_) | State::BasisClosed(_) | State::Bundle(_) => {}
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
