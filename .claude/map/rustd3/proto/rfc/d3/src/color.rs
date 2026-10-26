//! Colors as plain value types with conversions, parsing and interpolation.

use crate::interpolate::Interpolate;
use std::fmt;
use std::str::FromStr;

/// sRGB with channels in `0.0..=255.0` and opacity in `0.0..=1.0`, like `d3.rgb`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub opacity: f64,
}

/// Hue (degrees), saturation and lightness, like `d3.hsl`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
    pub opacity: f64,
}

/// A color string that did not parse.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseColorError(pub String);

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a CSS color: {:?}", self.0)
    }
}

impl std::error::Error for ParseColorError {}

impl Rgb {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, opacity: 1.0 }
    }

    /// `#rrggbb` as a `const`, for palettes.
    pub const fn hex(v: u32) -> Self {
        Self::new(((v >> 16) & 0xff) as f64, ((v >> 8) & 0xff) as f64, (v & 0xff) as f64)
    }

    /// d3's `color.brighter(k)`.
    pub fn brighter(self, k: f64) -> Self {
        let f = (1.0 / 0.7f64).powf(k);
        Self { r: self.r * f, g: self.g * f, b: self.b * f, ..self }
    }

    pub fn with_opacity(self, opacity: f64) -> Self {
        Self { opacity, ..self }
    }
}

impl FromStr for Rgb {
    type Err = ParseColorError;

    /// Parses `#rgb`, `#rrggbb` and a few named colors (the real port carries all 148).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseColorError(s.to_owned());
        let named = match s.trim().to_ascii_lowercase().as_str() {
            "steelblue" => Some(0x4682b4),
            "black" => Some(0x000000),
            "white" => Some(0xffffff),
            "tomato" => Some(0xff6347),
            _ => None,
        };
        if let Some(v) = named {
            return Ok(Self::hex(v));
        }
        let hex = s.trim().strip_prefix('#').ok_or_else(err)?;
        let v = u32::from_str_radix(hex, 16).map_err(|_| err())?;
        match hex.len() {
            6 => Ok(Self::hex(v)),
            3 => Ok(Self::hex(((v & 0xf00) * 0x1100) | ((v & 0x0f0) * 0x110) | ((v & 0x00f) * 0x11))),
            _ => Err(err()),
        }
    }
}

impl fmt::Display for Rgb {
    /// `rgb(r, g, b)` or `rgba(r, g, b, a)`, as `color.toString()` prints it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = |v: f64| v.round().clamp(0.0, 255.0);
        if self.opacity >= 1.0 {
            write!(f, "rgb({}, {}, {})", c(self.r), c(self.g), c(self.b))
        } else {
            write!(f, "rgba({}, {}, {}, {})", c(self.r), c(self.g), c(self.b), self.opacity.clamp(0.0, 1.0))
        }
    }
}

impl From<Hsl> for Rgb {
    fn from(c: Hsl) -> Self {
        let h = c.h % 360.0 + if c.h < 0.0 { 360.0 } else { 0.0 };
        let (s, l) = (if h.is_nan() || c.s.is_nan() { 0.0 } else { c.s }, c.l);
        let m2 = l + if l < 0.5 { l } else { 1.0 - l } * s;
        let m1 = 2.0 * l - m2;
        let hue = |h: f64| {
            255.0 * if h < 60.0 {
                m1 + (m2 - m1) * h / 60.0
            } else if h < 180.0 {
                m2
            } else if h < 240.0 {
                m1 + (m2 - m1) * (240.0 - h) / 60.0
            } else {
                m1
            }
        };
        let r = if h >= 240.0 { h - 240.0 } else { h + 120.0 };
        let b = if h < 120.0 { h + 240.0 } else { h - 120.0 };
        Rgb { r: hue(r), g: hue(h), b: hue(b), opacity: c.opacity }
    }
}

impl Interpolate for Rgb {
    /// Plain RGB interpolation (d3's `interpolateRgb` with gamma 1).
    fn lerp(&self, to: &Self, t: f64) -> Self {
        Rgb {
            r: self.r.lerp(&to.r, t),
            g: self.g.lerp(&to.g, t),
            b: self.b.lerp(&to.b, t),
            opacity: self.opacity.lerp(&to.opacity, t),
        }
    }
}

/// Categorical schemes from d3-scale-chromatic.
pub mod scheme {
    use super::Rgb;

    pub const CATEGORY10: [Rgb; 10] = [
        Rgb::hex(0x1f77b4), Rgb::hex(0xff7f0e), Rgb::hex(0x2ca02c), Rgb::hex(0xd62728), Rgb::hex(0x9467bd),
        Rgb::hex(0x8c564b), Rgb::hex(0xe377c2), Rgb::hex(0x7f7f7f), Rgb::hex(0xbcbd22), Rgb::hex(0x17becf),
    ];

    /// A sequential interpolator: `t` in `[0, 1]` to a color (a two-stop stand-in for viridis).
    pub fn blues(t: f64) -> Rgb {
        use crate::interpolate::Interpolate;
        Rgb::hex(0xf7fbff).lerp(&Rgb::hex(0x08306b), t.clamp(0.0, 1.0))
    }
}
