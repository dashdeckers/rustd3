//! [`Lab`]: d3-color's lab, CIELAB under D50.

use std::fmt;
use std::str::FromStr;

use crate::color::{Hsl, K, ParseColorError, Rgb};

/// d3's D50 white point, Bradford-adapted: `Xn`, `Yn`, `Zn`.
const WHITE: [f64; 3] = [0.96422, 1.0, 0.82521];

/// d3's `t0`.
const T0: f64 = 4.0 / 29.0;

/// d3's `t1`.
const T1: f64 = 6.0 / 29.0;

/// d3's `t2`.
const T2: f64 = 3.0 * T1 * T1;

/// d3's `t3`.
const T3: f64 = T1 * T1 * T1;

/// CIELAB under D50 with d3's constants, as `d3.lab`: lightness `l` (0 for
/// black, 100 for white), `a` from green to red and `b` from blue to
/// yellow. Channels are neither clamped nor rounded; a NaN channel is d3's
/// undefined one.
///
/// ```
/// use rustd3_core::color::{Lab, Rgb};
///
/// let steel: Lab = "steelblue".parse()?;
/// assert_eq!([steel.l, steel.a, steel.b].map(f64::round), [52.0, -8.0, -33.0]);
/// assert_eq!(Rgb::from(steel.brighter(1.0)).to_string(), "rgb(121, 177, 230)");
/// assert_eq!(Lab::gray(50.0).to_string(), "rgb(119, 119, 119)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    /// Lightness, 0..=100.
    pub l: f64,
    /// Green (negative) to red (positive).
    pub a: f64,
    /// Blue (negative) to yellow (positive).
    pub b: f64,
    /// Opacity, 0..=1.
    pub opacity: f64,
}

impl fmt::Display for Lab {
    /// d3's `toString()`: the color converted to [`Rgb`] and formatted as
    /// `rgb(…)` or `rgba(…)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&Rgb::from(*self), f)
    }
}

impl FromStr for Lab {
    type Err = ParseColorError;

    /// d3's `lab(s)`: the CSS color `s` converted from sRGB.
    fn from_str(s: &str) -> Result<Lab, ParseColorError> {
        s.parse::<Rgb>().map(Lab::from)
    }
}

impl From<Rgb> for Lab {
    /// d3's `labConvert`: sRGB linearized, to XYZ, to Lab. A gray takes
    /// `x = z = y`, so its `a` and `b` are exactly 0.
    fn from(c: Rgb) -> Lab {
        let [r, g, b] = [c.r, c.g, c.b].map(linear);
        let y = xyz_to_lab((0.2225045 * r + 0.7168786 * g + 0.0606169 * b) / WHITE[1]);
        let (x, z) = if r == g && g == b {
            (y, y)
        } else {
            (
                xyz_to_lab((0.4360747 * r + 0.3850649 * g + 0.1430804 * b) / WHITE[0]),
                xyz_to_lab((0.0139322 * r + 0.0971045 * g + 0.7141733 * b) / WHITE[2]),
            )
        };
        Lab {
            l: 116.0 * y - 16.0,
            a: 500.0 * (x - y),
            b: 200.0 * (y - z),
            opacity: c.opacity,
        }
    }
}

impl From<Hsl> for Lab {
    /// Through [`Rgb`], as d3 converts.
    fn from(c: Hsl) -> Lab {
        Lab::from(Rgb::from(c))
    }
}

impl From<Lab> for Rgb {
    /// d3's `lab.rgb()`: Lab to XYZ to linear sRGB, gamma-encoded; an
    /// undefined `a` or `b` counts as 0.
    fn from(c: Lab) -> Rgb {
        let y = (c.l + 16.0) / 116.0;
        let x = if c.a.is_nan() { y } else { y + c.a / 500.0 };
        let z = if c.b.is_nan() { y } else { y - c.b / 200.0 };
        let x = WHITE[0] * lab_to_xyz(x);
        let y = WHITE[1] * lab_to_xyz(y);
        let z = WHITE[2] * lab_to_xyz(z);
        Rgb {
            r: encoded(3.1338561 * x - 1.6168667 * y - 0.4906146 * z),
            g: encoded(-0.9787684 * x + 1.9161415 * y + 0.0334540 * z),
            b: encoded(0.0719453 * x - 0.2289914 * y + 1.4052427 * z),
            opacity: c.opacity,
        }
    }
}

impl Lab {
    /// Lightness `l`, `a` and `b`, opaque.
    #[must_use]
    pub const fn new(l: f64, a: f64, b: f64) -> Self {
        Lab {
            l,
            a,
            b,
            opacity: 1.0,
        }
    }

    /// d3's `gray(l)`: lightness `l` with `a` and `b` 0, opaque.
    #[must_use]
    pub const fn gray(l: f64) -> Self {
        Lab::new(l, 0.0, 0.0)
    }

    /// d3's `brighter(k)`: the lightness plus 18k.
    #[must_use]
    pub fn brighter(self, k: f64) -> Self {
        Lab {
            l: self.l + K * k,
            ..self
        }
    }

    /// d3's `darker(k)`: the lightness minus 18k.
    #[must_use]
    pub fn darker(self, k: f64) -> Self {
        Lab {
            l: self.l - K * k,
            ..self
        }
    }
}

/// d3's `xyz2lab`: a cube root above `t3`, linear below.
fn xyz_to_lab(t: f64) -> f64 {
    if t > T3 {
        t.powf(1.0 / 3.0)
    } else {
        t / T2 + T0
    }
}

/// d3's `lab2xyz`, the inverse of [`xyz_to_lab`].
fn lab_to_xyz(t: f64) -> f64 {
    if t > T1 { t * t * t } else { T2 * (t - T0) }
}

/// d3's `lrgb2rgb`: a linear sRGB channel gamma-encoded into 0..=255.
fn encoded(x: f64) -> f64 {
    255.0
        * if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        }
}

/// d3's `rgb2lrgb`: a channel in 0..=255 linearized into 0..=1.
fn linear(x: f64) -> f64 {
    let x = x / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}
