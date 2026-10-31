//! d3-interpolate, typed: a value blends with another of its type through
//! [`Interpolate`], where d3 inspects runtime types; d3's interpolators are
//! closures of `t`, and its color interpolators take the color space's type
//! (callers convert, where d3 dispatches on strings).
//!
//! ```
//! use rustd3_core::color::{Hcl, Rgb};
//! use rustd3_core::interpolate::{self, Interpolate};
//!
//! assert_eq!(10.0_f64.lerp(&20.0, 0.25), 12.5);
//! assert_eq!(interpolate::number(10.0, 20.0)(0.25), 12.5);
//! assert_eq!(interpolate::round(10.0, 42.0)(0.3), 20.0);
//! let [white, steel]: [Rgb; 2] = ["white".parse()?, "steelblue".parse()?];
//! let blend = interpolate::hcl(Hcl::from(white), Hcl::from(steel));
//! assert_eq!(blend(0.5).to_string(), "rgb(138, 194, 248)");
//! # Ok::<(), rustd3_core::color::ParseColorError>(())
//! ```

mod basis;
mod color;
pub(crate) mod gamma;
mod hue;
mod piecewise;
mod quantize;

pub(crate) use color::basis_at;

pub use basis::basis;
pub use color::{
    cubehelix, cubehelix_long, hcl, hcl_long, hsl, hsl_long, lab, rgb, rgb_basis, rgb_gamma,
};
pub use hue::hue;
pub use piecewise::piecewise;
pub use quantize::quantize;

use jiff::Timestamp;

use crate::color::{Cubehelix, Hcl, Hsl, Lab};
use crate::js;
use gamma::Channel;

/// A value that blends with another of its type (d3's interpolate, typed).
pub trait Interpolate: Clone {
    /// The value a fraction `t` of the way from `self` to `to`.
    #[must_use]
    fn lerp(&self, to: &Self, t: f64) -> Self;
}

/// Derives [`Interpolate`](trait@Interpolate) field by field for a struct
/// with named fields. `use rustd3_core::interpolate::Interpolate;` imports
/// the trait and the derive together; derive `Clone` as well, the trait's
/// supertrait.
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Debug, Clone, PartialEq, Interpolate)]
/// struct Bar {
///     y: f64,
///     width: f64,
///     opacity: f64,
///     value: f64,
///     #[interpolate(skip)]
///     name: String,
/// }
///
/// let a = Bar { y: 0.0, width: 0.0, opacity: 0.0, value: 0.0, name: "a".into() };
/// let b = Bar { y: 40.0, width: 200.0, opacity: 1.0, value: 10.0, name: "b".into() };
/// let quarter = Bar { y: 10.0, width: 50.0, opacity: 0.25, value: 2.5, name: "a".into() };
/// assert_eq!(a.lerp(&b, 0.25), quarter);
/// assert_eq!(a.lerp(&b, 1.0).name, "b");
/// ```
///
/// A field whose type has no `Interpolate` impl is an error (E0277):
///
/// ```compile_fail,E0277
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Bad {
///     name: String,
/// }
/// ```
///
/// Skipped, it keeps the start value until `t = 1`:
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Good {
///     #[interpolate(skip)]
///     name: String,
/// }
/// ```
///
/// The same holds for a type of your own until it implements the trait:
///
/// ```compile_fail,E0277
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone)]
/// struct Weight(f64);
///
/// #[derive(Clone, Interpolate)]
/// struct Edge {
///     weight: Weight,
/// }
/// ```
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Edge {
///     weight: f64,
/// }
/// ```
///
/// A generic struct interpolates for the parameters its fields can blend:
/// `Mark<f64>` implements the trait, `Mark<String>` does not (E0599).
///
/// ```compile_fail,E0599
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Mark<T> {
///     v: T,
///     #[interpolate(skip)]
///     label: String,
/// }
///
/// let a = Mark { v: String::from("a"), label: String::new() };
/// a.lerp(&a, 0.5);
/// ```
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Mark<T> {
///     v: T,
///     #[interpolate(skip)]
///     label: String,
/// }
///
/// let a = Mark { v: 1.0, label: String::new() };
/// assert_eq!(a.lerp(&Mark { v: 3.0, label: String::new() }, 0.5).v, 2.0);
/// ```
///
/// Only structs with named fields derive it, not enums:
///
/// ```compile_fail
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// enum Shape {
///     Dot { r: f64 },
/// }
/// ```
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct Dot {
///     r: f64,
/// }
/// ```
///
/// nor tuple structs:
///
/// ```compile_fail
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct P(f64, f64);
/// ```
///
/// ```
/// use rustd3_core::interpolate::Interpolate;
///
/// #[derive(Clone, Interpolate)]
/// struct P {
///     x: f64,
///     y: f64,
/// }
/// ```
#[cfg(feature = "derive")]
pub use rustd3_derive::Interpolate;

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

impl Interpolate for Hsl {
    /// d3's interpolateHsl, in HSL: the hue along the shorter arc,
    /// saturation, lightness and opacity linearly, an undefined component
    /// taking the other color's.
    fn lerp(&self, to: &Hsl, t: f64) -> Hsl {
        let [h, s, l, opacity] = Channel::four(
            [self.h, self.s, self.l, self.opacity],
            [to.h, to.s, to.l, to.opacity],
            Channel::hue,
        )
        .map(|c| c.at(t));
        Hsl { h, s, l, opacity }
    }
}

impl Interpolate for Lab {
    /// d3's interpolateLab, in Lab: every channel linearly, an undefined
    /// channel taking the other color's.
    fn lerp(&self, to: &Lab, t: f64) -> Lab {
        let [l, a, b, opacity] = Channel::four(
            [self.l, self.a, self.b, self.opacity],
            [to.l, to.a, to.b, to.opacity],
            Channel::nogamma,
        )
        .map(|c| c.at(t));
        Lab { l, a, b, opacity }
    }
}

impl Interpolate for Hcl {
    /// d3's interpolateHcl, in HCL: the hue along the shorter arc, chroma,
    /// luminance and opacity linearly, an undefined component taking the
    /// other color's.
    fn lerp(&self, to: &Hcl, t: f64) -> Hcl {
        let [h, c, l, opacity] = Channel::four(
            [self.h, self.c, self.l, self.opacity],
            [to.h, to.c, to.l, to.opacity],
            Channel::hue,
        )
        .map(|c| c.at(t));
        Hcl { h, c, l, opacity }
    }
}

impl Interpolate for Cubehelix {
    /// d3's interpolateCubehelix at gamma 1, in cubehelix: the hue along
    /// the shorter arc, the rest linearly.
    fn lerp(&self, to: &Cubehelix, t: f64) -> Cubehelix {
        let [h, s, l, opacity] = Channel::four(
            [self.h, self.s, self.l, self.opacity],
            [to.h, to.s, to.l, to.opacity],
            Channel::hue,
        )
        .map(|c| c.at(t));
        Cubehelix { h, s, l, opacity }
    }
}

impl Interpolate for Timestamp {
    /// d3's interpolateDate in nanoseconds: `self + round((to − self)·t)`,
    /// saturating at [`Timestamp::MIN`] and [`Timestamp::MAX`]. Only the
    /// difference passes through `f64`, never an epoch value.
    ///
    /// ```
    /// use rustd3_core::interpolate::Interpolate;
    /// use rustd3_core::jiff::Timestamp;
    ///
    /// let a: Timestamp = "2026-10-27T00:00:00Z".parse()?;
    /// let b: Timestamp = "2026-10-27T00:00:00.000000003Z".parse()?;
    /// assert_eq!(a.lerp(&b, 0.5).to_string(), "2026-10-27T00:00:00.000000002Z");
    /// assert_eq!(a.lerp(&b, 1e40), Timestamp::MAX);
    /// # Ok::<(), rustd3_core::jiff::Error>(())
    /// ```
    fn lerp(&self, to: &Timestamp, t: f64) -> Timestamp {
        let (a, b) = (self.as_nanosecond(), to.as_nanosecond());
        // A float to integer cast saturates, and NaN casts to 0.
        let step = ((b - a) as f64 * t).round() as i128;
        let (min, max) = (
            Timestamp::MIN.as_nanosecond(),
            Timestamp::MAX.as_nanosecond(),
        );
        let ns = a.saturating_add(step).clamp(min, max);
        Timestamp::from_nanosecond(ns).expect("invariant: clamped into the timestamp range")
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
