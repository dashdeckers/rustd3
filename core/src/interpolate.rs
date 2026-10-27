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

mod color;
pub(crate) mod gamma;
mod hue;

pub use color::{cubehelix, cubehelix_long, hcl, hcl_long, hsl, hsl_long, lab, rgb, rgb_gamma};
pub use hue::hue;

use crate::color::{Cubehelix, Hcl, Hsl, Lab};
use crate::js;
use gamma::Channel;

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
