//! [`Cubehelix`]: d3-color's cubehelix, Green's (2011) color scheme.

use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

use crate::color::{BRIGHTER, DARKER, Hsl, ParseColorError, Rgb};

/// d3's `A`.
const A: f64 = -0.14861;

/// d3's `B`.
const B: f64 = 1.78277;

/// d3's `C`.
const C: f64 = -0.29227;

/// d3's `D`.
const D: f64 = -0.90649;

/// d3's `E`.
const E: f64 = 1.97294;

/// d3's `ED`.
const ED: f64 = E * D;

/// d3's `EB`.
const EB: f64 = E * B;

/// d3's `BC_DA`.
const BC_DA: f64 = B * C - D * A;

/// Green's cubehelix with d3's constants, as `d3.cubehelix`: hue `h` in
/// degrees, saturation `s` and lightness `l` in 0..=1. A NaN hue is d3's
/// undefined one: grays have none, and black and white no saturation
/// either.
///
/// ```
/// use rustd3_core::color::{Cubehelix, Rgb};
///
/// let steel: Cubehelix = "steelblue".parse()?;
/// assert_eq!((steel.h.round(), (steel.s * 100.0).round()), (203.0, 63.0));
/// assert_eq!(Rgb::from(steel).to_string(), "rgb(70, 130, 180)");
/// assert!("gray".parse::<Cubehelix>()?.h.is_nan());
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cubehelix {
    /// Hue, in degrees.
    pub h: f64,
    /// Saturation, the helix's amplitude.
    pub s: f64,
    /// Lightness, 0..=1.
    pub l: f64,
    /// Opacity, 0..=1.
    pub opacity: f64,
}

impl fmt::Display for Cubehelix {
    /// d3's `toString()`: the color converted to [`Rgb`] and formatted as
    /// `rgb(…)` or `rgba(…)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&Rgb::from(*self), f)
    }
}

impl FromStr for Cubehelix {
    type Err = ParseColorError;

    /// d3's `cubehelix(s)`: the CSS color `s` converted from sRGB.
    fn from_str(s: &str) -> Result<Cubehelix, ParseColorError> {
        s.parse::<Rgb>().map(Cubehelix::from)
    }
}

impl From<Rgb> for Cubehelix {
    /// d3's `cubehelixConvert`: an undefined hue when the saturation is 0
    /// or undefined (black and white).
    fn from(c: Rgb) -> Cubehelix {
        let (r, g, b) = (c.r / 255.0, c.g / 255.0, c.b / 255.0);
        let l = (BC_DA * b + ED * r - EB * g) / (BC_DA + ED - EB);
        let bl = b - l;
        let k = (E * (g - l) - C * bl) / D;
        let s = (k * k + bl * bl).sqrt() / (E * l * (1.0 - l));
        let h = if s != 0.0 && !s.is_nan() {
            k.atan2(bl) * (180.0 / PI) - 120.0
        } else {
            f64::NAN
        };
        Cubehelix {
            h: if h < 0.0 { h + 360.0 } else { h },
            s,
            l,
            opacity: c.opacity,
        }
    }
}

impl From<Hsl> for Cubehelix {
    /// Through [`Rgb`], as d3 converts.
    fn from(c: Hsl) -> Cubehelix {
        Cubehelix::from(Rgb::from(c))
    }
}

impl From<Cubehelix> for Rgb {
    /// d3's `cubehelix.rgb()`: an undefined hue counts as 0, an undefined
    /// saturation as gray.
    fn from(c: Cubehelix) -> Rgb {
        let h = if c.h.is_nan() {
            0.0
        } else {
            (c.h + 120.0) * (PI / 180.0)
        };
        let l = c.l;
        let a = if c.s.is_nan() {
            0.0
        } else {
            c.s * l * (1.0 - l)
        };
        let (cosh, sinh) = (h.cos(), h.sin());
        Rgb {
            r: 255.0 * (l + a * (A * cosh + B * sinh)),
            g: 255.0 * (l + a * (C * cosh + D * sinh)),
            b: 255.0 * (l + a * (E * cosh)),
            opacity: c.opacity,
        }
    }
}

impl Cubehelix {
    /// Hue `h`, saturation `s` and lightness `l`, opaque.
    pub const fn new(h: f64, s: f64, l: f64) -> Self {
        Cubehelix {
            h,
            s,
            l,
            opacity: 1.0,
        }
    }

    /// d3's `brighter(k)`: the lightness times (1/0.7)^k.
    pub fn brighter(self, k: f64) -> Self {
        Cubehelix {
            l: self.l * BRIGHTER.powf(k),
            ..self
        }
    }

    /// d3's `darker(k)`: the lightness times 0.7^k.
    pub fn darker(self, k: f64) -> Self {
        Cubehelix {
            l: self.l * DARKER.powf(k),
            ..self
        }
    }
}
