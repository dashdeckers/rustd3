//! `interact::Transform`, d3's `ZoomTransform`: a uniform scale and a
//! translation, as a value.

use std::fmt;

use crate::js;

/// d3's `ZoomTransform`: screen = data · k + (x, y). Not the trait under
/// continuous scales, [`scale::Transform`](crate::scale::Transform).
///
/// Plain data. d3's `transform.rescaleX(x)` is
/// [`x.rescale_x(&t)`](crate::interact::Rescale::rescale_x).
///
/// ```
/// use rustd3_core::interact;
///
/// let t = interact::Transform::IDENTITY.translate(2.0, 3.0).scale(2.0);
/// assert_eq!(t, interact::Transform { k: 2.0, x: 2.0, y: 3.0 });
/// assert_eq!(t.apply((4.0, 5.0)), (10.0, 13.0));
/// assert_eq!(t.invert((4.0, 5.0)), (1.0, 1.0));
/// assert_eq!(t.to_string(), "translate(2,3) scale(2)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The scale factor.
    pub k: f64,
    /// The horizontal translation, in screen units.
    pub x: f64,
    /// The vertical translation, in screen units.
    pub y: f64,
}

impl fmt::Display for Transform {
    /// d3's `toString()`: `translate(x,y) scale(k)`, −0 printed as 0, as
    /// JavaScript prints it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Transform { k, x, y } = *self;
        write!(
            f,
            "translate({},{}) scale({})",
            js::Number(x),
            js::Number(y),
            js::Number(k)
        )
    }
}

impl Default for Transform {
    /// [`Transform::IDENTITY`], so that a widget's state holding a transform
    /// can derive `Default`.
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Transform {
    /// d3's `zoomIdentity`: k = 1, no translation.
    #[doc(alias = "zoomIdentity")]
    pub const IDENTITY: Self = Transform {
        k: 1.0,
        x: 0.0,
        y: 0.0,
    };

    /// d3's `transform.scale(k)`: the scale factor multiplied by `k`, the
    /// translation kept.
    #[must_use]
    pub fn scale(self, k: f64) -> Self {
        Transform {
            k: self.k * k,
            ..self
        }
    }

    /// d3's `transform.translate(x, y)`: translated by (`x`, `y`) in data
    /// units, which this scale factor multiplies. A zero translation
    /// returns the transform unchanged, as in d3 (so −0 and an infinite
    /// `k` stay as they are).
    #[must_use]
    pub fn translate(self, x: f64, y: f64) -> Self {
        if x == 0.0 && y == 0.0 {
            return self;
        }
        Transform {
            x: self.x + self.k * x,
            y: self.y + self.k * y,
            ..self
        }
    }

    /// d3's `transform.apply(point)`: the screen position of a data point.
    #[must_use]
    pub fn apply(self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.apply_x(x), self.apply_y(y))
    }

    /// d3's `transform.applyX(x)`.
    #[must_use]
    pub fn apply_x(self, x: f64) -> f64 {
        x * self.k + self.x
    }

    /// d3's `transform.applyY(y)`.
    #[must_use]
    pub fn apply_y(self, y: f64) -> f64 {
        y * self.k + self.y
    }

    /// d3's `transform.invert(location)`: the data point at a screen
    /// position.
    #[must_use]
    pub fn invert(self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.invert_x(x), self.invert_y(y))
    }

    /// d3's `transform.invertX(x)`.
    #[must_use]
    pub fn invert_x(self, x: f64) -> f64 {
        (x - self.x) / self.k
    }

    /// d3's `transform.invertY(y)`.
    #[must_use]
    pub fn invert_y(self, y: f64) -> f64 {
        (y - self.y) / self.k
    }
}
