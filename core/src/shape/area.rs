//! d3's area generator ([`Area`](crate::shape::Area)): a curve along the
//! top of each run of items, then back along its baseline; and its radial
//! form ([`Radial`]).

mod radial;

pub use radial::{Radial, radial};

use crate::path::PathContext;
use crate::shape::curve::State;
use crate::shape::{Area, Curve, Span};

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
    /// edges as two closed rings, as d3's do; `Curve::Bundle` draws as
    /// `Curve::Basis`.
    pub fn draw<S: Span>(&self, items: impl IntoIterator<Item = S>, ctx: &mut impl PathContext) {
        let edges = items
            .into_iter()
            .map(|item| item.span(self.baseline).map(|(x, y0, y1)| (x, y0, x, y1)));
        draw_edges(self.curve, edges, ctx);
    }
}

/// d3's area loop (`area.js`, `area(data)`) over `(x0, y0, x1, y1)` items:
/// for each run of `Some` items, `curve` through `(x1, y1)` forward, then
/// through `(x0, y0)` backward. [`Area`] passes `x0 == x1`; the radial area
/// passes edges whose x differ.
fn draw_edges(
    curve: Curve,
    edges: impl IntoIterator<Item = Option<(f64, f64, f64, f64)>>,
    ctx: &mut impl PathContext,
) {
    let mut state = State::area(curve);
    let mut base = Vec::new();
    for edge in edges {
        match edge {
            Some((x0, y0, x1, y1)) => {
                if base.is_empty() {
                    state.area_start();
                    state.line_start();
                }
                base.push((x0, y0));
                state.point(x1, y1, ctx);
            }
            None => end_run(&mut state, &mut base, ctx),
        }
    }
    end_run(&mut state, &mut base, ctx);
}

/// Ends the run whose `(x0, y0)` are in `base`, if one is open: the top's
/// `line_end`, then the baseline drawn in reverse as a second line.
fn end_run(state: &mut State, base: &mut Vec<(f64, f64)>, ctx: &mut impl PathContext) {
    if base.is_empty() {
        return;
    }
    state.line_end(ctx);
    state.line_start();
    for (x, y) in base.drain(..).rev() {
        state.point(x, y, ctx);
    }
    state.line_end(ctx);
    state.area_end();
}
