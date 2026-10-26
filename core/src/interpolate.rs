//! d3-interpolate, typed: a value blends with another of its type through
//! [`Interpolate`], where d3 inspects runtime types, and d3's number
//! interpolators are closures of `t`.
//!
//! ```
//! use rustd3_core::interpolate::{self, Interpolate};
//!
//! assert_eq!(10.0_f64.lerp(&20.0, 0.25), 12.5);
//! assert_eq!(interpolate::number(10.0, 20.0)(0.25), 12.5);
//! assert_eq!(interpolate::round(10.0, 42.0)(0.3), 20.0);
//! ```

use crate::js;

/// A value that blends with another of its type (d3's interpolate, typed).
pub trait Interpolate: Clone {
    /// The value a fraction `t` of the way from `self` to `to`.
    fn lerp(&self, to: &Self, t: f64) -> Self;
}

impl Interpolate for f64 {
    /// d3's interpolateNumber, `self · (1 − t) + to · t`: exact at t = 0
    /// and t = 1.
    fn lerp(&self, to: &f64, t: f64) -> f64 {
        self * (1.0 - t) + to * t
    }
}

impl Interpolate for f32 {
    /// Through `f64`.
    fn lerp(&self, to: &f32, t: f64) -> f32 {
        f64::from(*self).lerp(&f64::from(*to), t) as f32
    }
}

/// d3.interpolateNumber: `a` blended with `b` at `t`.
#[doc(alias = "interpolateNumber")]
pub fn number(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync {
    move |t| a.lerp(&b, t)
}

/// d3.interpolateRound: the blend of `a` and `b` at `t`, rounded half
/// toward +∞ as JavaScript's `Math.round` does; `a` and `b` are not
/// rounded first.
#[doc(alias = "interpolateRound")]
pub fn round(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync {
    move |t| js::round(a.lerp(&b, t))
}
