//! d3's lineRadial: a line through (angle, radius) points.

use crate::path::PathContext;
use crate::shape::{Curve, Vertex, curve};

/// d3.lineRadial() over already-scaled (angle, radius) points, built by
/// consuming builders. Angles are radians clockwise from 12 o'clock.
///
/// ```
/// use std::f64::consts::FRAC_PI_2;
///
/// use rustd3_core::path::SvgPath;
/// use rustd3_core::shape::line;
///
/// let mut p = SvgPath::with_digits(3);
/// line::radial().draw([(0.0, 1.0), (FRAC_PI_2, 2.0)], &mut p);
/// assert_eq!(p.to_string(), "M0,-1L2,0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radial {
    curve: Curve,
}

/// d3.lineRadial(): straight segments, `None` items breaking the line.
pub fn radial() -> Radial {
    Radial::default()
}

impl Radial {
    /// Joins the points with `curve`, applied after the points are placed
    /// (d3's `curveRadial(curve)`).
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve }
    }

    /// Draws `points`, items `(angle, radius)`, into `ctx` as d3's
    /// `lineRadial()(data)` does: each point placed at `(r·sin a, −r·cos a)`,
    /// then drawn as [`Line::draw`](crate::shape::Line::draw) draws.
    pub fn draw<V: Vertex>(&self, points: impl IntoIterator<Item = V>, ctx: &mut impl PathContext) {
        let placed = points
            .into_iter()
            .map(|p| p.vertex().map(|(a, r)| curve::radial(a, r)));
        crate::shape::line().curve(self.curve).draw(placed, ctx);
    }
}
