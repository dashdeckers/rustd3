//! d3-shape's curves: [`Curve`], the closed set of ways a line or an area
//! joins its points, and `State`, the state machine each one draws with
//! (d3's curve objects, verbatim in structure).

mod linear;

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
}

/// `$body` with `$family` bound to the family `$state` holds.
macro_rules! each {
    ($state:expr, $family:ident => $body:expr) => {
        match $state {
            State::Linear($family) => $body,
            State::LinearClosed($family) => $body,
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
    /// The state `curve` draws a line or an area with.
    pub(crate) fn new(curve: Curve) -> State {
        match curve {
            Curve::Linear => State::Linear(linear::Linear::default()),
            Curve::LinearClosed => State::LinearClosed(linear::LinearClosed::default()),
        }
    }

    /// d3's `areaStart`: the next line is an area's top.
    pub(crate) fn area_start(&mut self) {
        match self {
            State::Linear(s) => s.area_start(),
            // d3's closed curves ignore areaStart and areaEnd.
            State::LinearClosed(_) => {}
        }
    }

    /// d3's `areaEnd`.
    pub(crate) fn area_end(&mut self) {
        match self {
            State::Linear(s) => s.area_end(),
            State::LinearClosed(_) => {}
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
