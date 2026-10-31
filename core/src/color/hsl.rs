//! [`Hsl`]: d3-color's hsl, hue, saturation and lightness.

use std::fmt;
use std::str::FromStr;

use crate::color::parse::{self, Parsed};
use crate::color::{BRIGHTER, DARKER, ParseColorError, Rgb};
use crate::js;

/// Hue in degrees, saturation and lightness in 0..=1, as `d3.hsl`. A NaN
/// hue or saturation is d3's undefined one: grays have no hue, black and
/// white no saturation either.
///
/// ```
/// use rustd3_core::color::{Hsl, Rgb};
///
/// let steel = Hsl::from(Rgb::hex(0x4682b4));
/// assert_eq!((steel.h, steel.s), (207.27272727272728, 0.44));
/// assert!("#808080".parse::<Hsl>()?.h.is_nan());
/// assert_eq!(steel.format_hsl(), "hsl(207.27272727272728, 44%, 49.01960784313726%)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// Hue, in degrees.
    pub h: f64,
    /// Saturation, 0..=1.
    pub s: f64,
    /// Lightness, 0..=1.
    pub l: f64,
    /// Opacity, 0..=1.
    pub opacity: f64,
}

impl fmt::Display for Hsl {
    /// d3's `toString()`: the color converted to [`Rgb`] and formatted as
    /// `rgb(…)` or `rgba(…)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&Rgb::from(*self), f)
    }
}

impl FromStr for Hsl {
    type Err = ParseColorError;

    /// d3's `hsl(s)`: an `hsl(…)` form keeps its components, any other CSS
    /// color converts from sRGB.
    fn from_str(s: &str) -> Result<Hsl, ParseColorError> {
        match parse::parse(s) {
            Some(Parsed::Hsl(c)) => Ok(c),
            Some(Parsed::Rgb(c)) => Ok(Hsl::from(c)),
            None => Err(ParseColorError::new(s)),
        }
    }
}

impl From<Rgb> for Hsl {
    /// d3's `hslConvert`: NaN hue for grays, NaN saturation as well at
    /// lightness 0 or 1 and beyond.
    fn from(c: Rgb) -> Hsl {
        let (r, g, b) = (c.r / 255.0, c.g / 255.0, c.b / 255.0);
        let min = js::min(js::min(r, g), b);
        let max = js::max(js::max(r, g), b);
        let l = (max + min) / 2.0;
        let spread = max - min;
        let (h, s) = if spread != 0.0 && !spread.is_nan() {
            let h = if r == max {
                (g - b) / spread + if g < b { 6.0 } else { 0.0 }
            } else if g == max {
                (b - r) / spread + 2.0
            } else {
                (r - g) / spread + 4.0
            };
            let s = spread / if l < 0.5 { max + min } else { 2.0 - max - min };
            (h * 60.0, s)
        } else if l > 0.0 && l < 1.0 {
            (f64::NAN, 0.0)
        } else {
            (f64::NAN, f64::NAN)
        };
        Hsl {
            h,
            s,
            l,
            opacity: c.opacity,
        }
    }
}

impl Hsl {
    /// d3's `brighter(k)`: the lightness times (1/0.7)^k.
    #[must_use]
    pub fn brighter(self, k: f64) -> Self {
        Hsl {
            l: self.l * BRIGHTER.powf(k),
            ..self
        }
    }

    /// d3's `darker(k)`: the lightness times 0.7^k.
    #[must_use]
    pub fn darker(self, k: f64) -> Self {
        Hsl {
            l: self.l * DARKER.powf(k),
            ..self
        }
    }

    /// d3's `clamp()`: the hue into [0, 360), saturation and lightness
    /// into [0, 1] (NaN as 0), opacity into [0, 1] (NaN as 1).
    #[must_use]
    pub fn clamp(self) -> Self {
        Hsl {
            h: clamp_hue(self.h),
            s: clamp_unit(self.s),
            l: clamp_unit(self.l),
            opacity: super::clamp_opacity(self.opacity),
        }
    }

    /// d3's `displayable()`: saturation in [0, 1] or undefined, lightness
    /// and opacity in [0, 1].
    #[must_use]
    pub fn displayable(self) -> bool {
        ((0.0..=1.0).contains(&self.s) || self.s.is_nan())
            && (0.0..=1.0).contains(&self.l)
            && (0.0..=1.0).contains(&self.opacity)
    }

    /// d3's `formatHsl()`: `hsl(h, s%, l%)`, or `hsla(h, s%, l%, a)` when
    /// the clamped opacity is below 1, of the clamped components.
    #[must_use]
    pub fn format_hsl(self) -> String {
        let a = super::clamp_opacity(self.opacity);
        let (h, s, l) = (
            js::Number(clamp_hue(self.h)),
            js::Number(clamp_unit(self.s) * 100.0),
            js::Number(clamp_unit(self.l) * 100.0),
        );
        if a == 1.0 {
            format!("hsl({h}, {s}%, {l}%)")
        } else {
            format!("hsla({h}, {s}%, {l}%, {})", js::Number(a))
        }
    }
}

/// d3's `clamph`: the hue modulo 360 into [0, 360), NaN and −0 as 0.
fn clamp_hue(h: f64) -> f64 {
    let h = if h.is_nan() || h == 0.0 { 0.0 } else { h } % 360.0;
    if h < 0.0 { h + 360.0 } else { h }
}

/// d3's `clampt`: clamped to [0, 1], NaN and −0 as 0.
fn clamp_unit(v: f64) -> f64 {
    if v.is_nan() || v == 0.0 {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}
