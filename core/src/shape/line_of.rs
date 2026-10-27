//! [`LineOf`]: d3-style accessors over [`line`](crate::shape::line()) (Part
//! I, Accessors, option B).

use std::fmt;

use crate::path::PathContext;
use crate::shape::Curve;

/// d3.line() with accessors: `x`, `y` and `defined` read each datum,
/// borrowing what they need for `'a` (never `'static`), and `draw` maps the
/// data through them into [`shape::line`](crate::shape::line()), so curves
/// and gaps are the line's own. The default accessors return 0 and `true`;
/// there is no area counterpart.
///
/// ```
/// use rustd3_core::{path, scale, shape};
///
/// struct Reading {
///     t: f64,
///     value: Option<f64>,
/// }
/// let data = [
///     Reading { t: 0.0, value: Some(25.0) },
///     Reading { t: 1.0, value: Some(75.0) },
///     Reading { t: 2.0, value: None },
///     Reading { t: 3.0, value: Some(50.0) },
///     Reading { t: 4.0, value: Some(100.0) },
/// ];
/// let tx = scale::linear().domain([0.0, 4.0]).range([0.0, 400.0]);
/// let ty = scale::linear().domain([0.0, 100.0]).range([100.0, 0.0]);
/// let d = path::svg(|p| {
///     shape::line_of()
///         .x(|r: &Reading| tx.apply(r.t))
///         .y(|r: &Reading| ty.apply(r.value.unwrap_or(0.0)))
///         .defined(|r: &Reading| r.value.is_some())
///         .draw(&data, p)
/// });
/// assert_eq!(d, "M0,75L100,25M300,50L400,0");
/// ```
pub struct LineOf<'a, D> {
    x: Box<dyn Fn(&D) -> f64 + 'a>,
    y: Box<dyn Fn(&D) -> f64 + 'a>,
    defined: Box<dyn Fn(&D) -> bool + 'a>,
    curve: Curve,
}

impl<D> Default for LineOf<'_, D> {
    fn default() -> Self {
        Self {
            x: Box::new(|_| 0.0),
            y: Box::new(|_| 0.0),
            defined: Box::new(|_| true),
            curve: Curve::default(),
        }
    }
}

impl<D> fmt::Debug for LineOf<'_, D> {
    /// The curve; the accessors are closures.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineOf")
            .field("curve", &self.curve)
            .finish_non_exhaustive()
    }
}

impl<'a, D> LineOf<'a, D> {
    /// The x of each datum: d3's `line.x`.
    pub fn x(self, x: impl Fn(&D) -> f64 + 'a) -> Self {
        Self {
            x: Box::new(x),
            ..self
        }
    }

    /// The y of each datum: d3's `line.y`.
    pub fn y(self, y: impl Fn(&D) -> f64 + 'a) -> Self {
        Self {
            y: Box::new(y),
            ..self
        }
    }

    /// Whether a datum is drawn; one that is not breaks the line: d3's
    /// `line.defined`.
    pub fn defined(self, defined: impl Fn(&D) -> bool + 'a) -> Self {
        Self {
            defined: Box::new(defined),
            ..self
        }
    }

    /// Joins the points with `curve`: d3's `line.curve`.
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve, ..self }
    }

    /// Draws `data` into `ctx`: each datum through the accessors, `None`
    /// where it is not defined, then [`Line::draw`](crate::shape::Line::draw).
    pub fn draw(&self, data: &[D], ctx: &mut impl PathContext) {
        let points = data
            .iter()
            .map(|d| (self.defined)(d).then(|| ((self.x)(d), (self.y)(d))));
        crate::shape::line().curve(self.curve).draw(points, ctx);
    }
}
