//! d3's areaRadial: an area between an inner and an outer radius.

use crate::path::PathContext;
use crate::shape::{Curve, Span, curve};

/// d3.areaRadial() over already-scaled items, built by consuming builders.
/// Angles are radians clockwise from 12 o'clock.
///
/// ```
/// use std::f64::consts::FRAC_PI_2;
///
/// use rustd3_core::path::SvgPath;
/// use rustd3_core::shape::area;
///
/// let mut p = SvgPath::with_digits(3);
/// area::radial().draw([(0.0, 1.0, 2.0), (FRAC_PI_2, 1.0, 3.0)], &mut p);
/// assert_eq!(p.to_string(), "M0,-2L3,0L1,0L0,-1Z");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radial {
    curve: Curve,
    /// The inner radius of `(angle, outer)` items; 0 is d3's default.
    inner_radius: f64,
}

/// d3.areaRadial(): straight segments down to the center, `None` items
/// breaking the area.
#[must_use]
pub fn radial() -> Radial {
    Radial::default()
}

impl Radial {
    /// Joins the points of both edges with `curve`, applied after the points
    /// are placed (d3's `curveRadial(curve)`).
    #[must_use]
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve, ..self }
    }

    /// A constant inner radius for `(angle, outer)` items: d3's
    /// `areaRadial().innerRadius(r)`. 3-tuple items carry their own.
    #[must_use]
    pub fn inner_radius(self, inner_radius: f64) -> Self {
        Self {
            inner_radius,
            ..self
        }
    }

    /// Draws `items` into `ctx` as d3's `areaRadial()(data)` does: items are
    /// `(angle, inner, outer)`, `(angle, outer)` over the inner radius, or
    /// `None`. As [`Area::draw`](crate::shape::Area::draw) draws, the outer
    /// edge runs forward and the inner one back, each point placed at
    /// `(r·sin a, −r·cos a)`.
    pub fn draw<S: Span>(&self, items: impl IntoIterator<Item = S>, ctx: &mut impl PathContext) {
        let edges = items.into_iter().map(|item| {
            item.span(self.inner_radius).map(|(a, inner, outer)| {
                let ((x0, y0), (x1, y1)) = (curve::radial(a, inner), curve::radial(a, outer));
                (x0, y0, x1, y1)
            })
        });
        super::draw_edges(self.curve, edges, ctx);
    }
}
