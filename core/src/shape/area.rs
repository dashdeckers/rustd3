//! d3's area generator: a curve along the top of each run of items, then
//! back along its baseline.

use crate::path::PathContext;
use crate::shape::Curve;
use crate::shape::curve::State;

/// An item an area draws: `(x, y0, y1)`, or `(x, y1)` over the area's
/// baseline, or `None` for a gap.
pub trait Span {
    /// The item as `(x, y0, y1)`, a 2-tuple taking `baseline` as `y0`; `None`
    /// breaks the area (d3's `defined`).
    fn span(self, baseline: f64) -> Option<(f64, f64, f64)>;
}

/// d3.area() over already-scaled items, built by consuming builders.
///
/// ```
/// use rustd3_core::path;
/// use rustd3_core::shape;
///
/// let tops = [(0.0, 4.0), (1.0, 2.0), (2.0, 5.0)];
/// let d = path::svg(|p| shape::area().y0(300.0).draw(tops, p));
/// assert_eq!(d, "M0,4L1,2L2,5L2,300L1,300L0,300Z");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Area {
    curve: Curve,
    /// The `y0` of 2-tuple items; 0 is d3's default.
    baseline: f64,
}

impl Span for (f64, f64, f64) {
    fn span(self, _baseline: f64) -> Option<(f64, f64, f64)> {
        Some(self)
    }
}

impl Span for Option<(f64, f64, f64)> {
    fn span(self, _baseline: f64) -> Option<(f64, f64, f64)> {
        self
    }
}

impl Span for (f64, f64) {
    fn span(self, baseline: f64) -> Option<(f64, f64, f64)> {
        let (x, y1) = self;
        Some((x, baseline, y1))
    }
}

impl Span for Option<(f64, f64)> {
    fn span(self, baseline: f64) -> Option<(f64, f64, f64)> {
        self.and_then(|item| item.span(baseline))
    }
}

impl Area {
    /// Joins the points of both edges with `curve`: d3's `area.curve`.
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve, ..self }
    }

    /// A constant baseline for `(x, y1)` items: d3's `area().y0(300)`.
    /// 3-tuple items carry their own `y0`.
    pub fn y0(self, baseline: f64) -> Self {
        Self { baseline, ..self }
    }

    /// Draws `items` into `ctx` as d3's `area(data)` does: for each run of
    /// `Some` items, the curve through `(x, y1)` forward, then through
    /// `(x, y0)` backward, closed by the curve. A closed curve draws the two
    /// edges as two closed rings, as d3's do.
    pub fn draw<S: Span>(&self, items: impl IntoIterator<Item = S>, ctx: &mut impl PathContext) {
        let mut state = State::new(self.curve);
        let mut base = Vec::new();
        for item in items {
            match item.span(self.baseline) {
                Some((x, y0, y1)) => {
                    if base.is_empty() {
                        state.area_start();
                        state.line_start();
                    }
                    base.push((x, y0));
                    state.point(x, y1, ctx);
                }
                None => end_run(&mut state, &mut base, ctx),
            }
        }
        end_run(&mut state, &mut base, ctx);
    }
}

/// Ends the run whose `(x, y0)` are in `base`, if one is open: the top's
/// `line_end`, then the baseline drawn in reverse as a second line.
fn end_run(state: &mut State, base: &mut Vec<(f64, f64)>, ctx: &mut impl PathContext) {
    if base.is_empty() {
        return;
    }
    state.line_end(ctx);
    state.line_start();
    for (x, y0) in base.drain(..).rev() {
        state.point(x, y0, ctx);
    }
    state.line_end(ctx);
    state.area_end();
}
