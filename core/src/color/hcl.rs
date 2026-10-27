//! [`Hcl`]: d3-color's hcl, CIELAB in cylindrical form.

use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

use crate::color::{Hsl, K, Lab, ParseColorError, Rgb};

/// d3's hcl, also its `lch`: [`Lab`] in cylindrical form, with hue `h` in
/// degrees, chroma `c` and luminance `l`. A NaN hue is d3's undefined one:
/// grays have none, and black and white no chroma either.
///
/// ```
/// use rustd3_core::color::{Hcl, Rgb};
///
/// let steel: Hcl = "steelblue".parse()?;
/// assert_eq!([steel.h, steel.c, steel.l].map(f64::round), [256.0, 34.0, 52.0]);
/// assert_eq!(Rgb::from(Hcl::new(200.0, 40.0, 60.0)).to_string(), "rgb(0, 163, 167)");
/// let white: Hcl = "white".parse()?;
/// assert!(white.h.is_nan() && white.c.is_nan() && white.l == 100.0);
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hcl {
    /// Hue, in degrees.
    pub h: f64,
    /// Chroma.
    pub c: f64,
    /// Luminance, 0..=100.
    pub l: f64,
    /// Opacity, 0..=1.
    pub opacity: f64,
}

impl fmt::Display for Hcl {
    /// d3's `toString()`: the color converted to [`Rgb`] and formatted as
    /// `rgb(…)` or `rgba(…)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&Rgb::from(*self), f)
    }
}

impl FromStr for Hcl {
    type Err = ParseColorError;

    /// d3's `hcl(s)`: the CSS color `s` converted from sRGB through Lab.
    fn from_str(s: &str) -> Result<Hcl, ParseColorError> {
        s.parse::<Rgb>().map(Hcl::from)
    }
}

impl From<Lab> for Hcl {
    /// d3's `hclConvert`: with `a` and `b` both 0 the hue is undefined, and
    /// the chroma too unless the luminance lies strictly between 0 and 100.
    fn from(c: Lab) -> Hcl {
        if c.a == 0.0 && c.b == 0.0 {
            let chroma = if 0.0 < c.l && c.l < 100.0 {
                0.0
            } else {
                f64::NAN
            };
            return Hcl {
                h: f64::NAN,
                c: chroma,
                l: c.l,
                opacity: c.opacity,
            };
        }
        let h = c.b.atan2(c.a) * (180.0 / PI);
        Hcl {
            h: if h < 0.0 { h + 360.0 } else { h },
            c: (c.a * c.a + c.b * c.b).sqrt(),
            l: c.l,
            opacity: c.opacity,
        }
    }
}

impl From<Rgb> for Hcl {
    /// Through [`Lab`], as d3 converts.
    fn from(c: Rgb) -> Hcl {
        Hcl::from(Lab::from(c))
    }
}

impl From<Hsl> for Hcl {
    /// Through [`Rgb`] and [`Lab`], as d3 converts.
    fn from(c: Hsl) -> Hcl {
        Hcl::from(Lab::from(Rgb::from(c)))
    }
}

impl From<Hcl> for Lab {
    /// d3's `hcl2lab`: an undefined hue gives `a` and `b` 0.
    fn from(c: Hcl) -> Lab {
        if c.h.is_nan() {
            return Lab {
                opacity: c.opacity,
                ..Lab::gray(c.l)
            };
        }
        let h = c.h * (PI / 180.0);
        Lab {
            l: c.l,
            a: h.cos() * c.c,
            b: h.sin() * c.c,
            opacity: c.opacity,
        }
    }
}

impl From<Hcl> for Rgb {
    /// d3's `hcl.rgb()`: through [`Lab`].
    fn from(c: Hcl) -> Rgb {
        Rgb::from(Lab::from(c))
    }
}

impl Hcl {
    /// Hue `h`, chroma `c` and luminance `l`, opaque.
    pub const fn new(h: f64, c: f64, l: f64) -> Self {
        Hcl {
            h,
            c,
            l,
            opacity: 1.0,
        }
    }

    /// d3's `lch(l, c, h)`: the same color with its arguments in the
    /// opposite order.
    pub const fn lch(l: f64, c: f64, h: f64) -> Self {
        Hcl::new(h, c, l)
    }

    /// d3's `brighter(k)`: the luminance plus 18k.
    pub fn brighter(self, k: f64) -> Self {
        Hcl {
            l: self.l + K * k,
            ..self
        }
    }

    /// d3's `darker(k)`: the luminance minus 18k.
    pub fn darker(self, k: f64) -> Self {
        Hcl {
            l: self.l - K * k,
            ..self
        }
    }
}
