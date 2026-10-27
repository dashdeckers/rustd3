//! [`Rgb`]: d3-color's rgb, a plain sRGB value.

use std::fmt;
use std::str::FromStr;

use crate::color::parse::{self, Parsed};
use crate::color::{BRIGHTER, DARKER, Hsl, ParseColorError};
use crate::interpolate::{self, Interpolate};
use crate::js;

/// sRGB, channels in 0..=255, opacity in 0..=1, as `d3.rgb`. The channels
/// are neither rounded nor clamped until the color is formatted or
/// [`clamp`](Rgb::clamp)ed; a NaN channel is d3's undefined one.
///
/// ```
/// use rustd3_core::color::{Hsl, Rgb};
///
/// let steel: Rgb = "steelblue".parse()?;
/// assert_eq!(steel.brighter(1.0).to_string(), "rgb(100, 186, 255)");
/// let hsl = Hsl { h: 200.0, s: 0.6, l: 0.5, opacity: 1.0 };
/// assert_eq!(Rgb::from(hsl).to_string(), "rgb(51, 153, 204)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
///
/// Colors blend through [`Interpolate`], so a color range makes a
/// `Linear<Rgb>`, which has no `invert`:
///
/// ```compile_fail
/// use rustd3_core::color::Rgb;
/// use rustd3_core::scale;
///
/// let heat = scale::linear().range([Rgb::hex(0xffffff), Rgb::hex(0x4682b4)]);
/// heat.invert(0.5);
/// ```
///
/// ```
/// use rustd3_core::color::Rgb;
/// use rustd3_core::scale;
///
/// let heat = scale::linear().range([Rgb::hex(0xffffff), Rgb::hex(0x4682b4)]);
/// assert_eq!(heat.apply(0.5).to_string(), "rgb(163, 193, 218)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    /// Red, 0..=255.
    pub r: f64,
    /// Green, 0..=255.
    pub g: f64,
    /// Blue, 0..=255.
    pub b: f64,
    /// Opacity, 0..=1.
    pub opacity: f64,
}

impl fmt::Display for Rgb {
    /// d3's `formatRgb`: `rgb(r, g, b)`, or `rgba(r, g, b, a)` when the
    /// opacity, clamped to [0, 1] with NaN as 1, is below 1; channels
    /// rounded and clamped to 0..=255, NaN as 0.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let a = super::clamp_opacity(self.opacity);
        let [r, g, b] = [self.r, self.g, self.b].map(clamp_channel);
        if a == 1.0 {
            write!(f, "rgb({r}, {g}, {b})")
        } else {
            write!(f, "rgba({r}, {g}, {b}, {})", js::Number(a))
        }
    }
}

impl FromStr for Rgb {
    type Err = ParseColorError;

    /// d3's `rgb(s)`: the CSS color `s` in sRGB, `hsl(…)` forms converted.
    fn from_str(s: &str) -> Result<Rgb, ParseColorError> {
        match parse::parse(s) {
            Some(Parsed::Rgb(c)) => Ok(c),
            Some(Parsed::Hsl(c)) => Ok(Rgb::from(c)),
            None => Err(ParseColorError::new(s)),
        }
    }
}

impl From<Hsl> for Rgb {
    /// d3's `hsl.rgb()` (FvD 13.37): the hue taken modulo 360 into
    /// [0, 360), an undefined hue or saturation as gray.
    fn from(c: Hsl) -> Rgb {
        let h = c.h % 360.0 + if c.h < 0.0 { 360.0 } else { 0.0 };
        let s = if h.is_nan() || c.s.is_nan() { 0.0 } else { c.s };
        let l = c.l;
        let m2 = l + if l < 0.5 { l } else { 1.0 - l } * s;
        let m1 = 2.0 * l - m2;
        let channel = |h: f64| {
            let v = if h < 60.0 {
                m1 + (m2 - m1) * h / 60.0
            } else if h < 180.0 {
                m2
            } else if h < 240.0 {
                m1 + (m2 - m1) * (240.0 - h) / 60.0
            } else {
                m1
            };
            v * 255.0
        };
        Rgb {
            r: channel(if h >= 240.0 { h - 240.0 } else { h + 120.0 }),
            g: channel(h),
            b: channel(if h < 120.0 { h + 240.0 } else { h - 120.0 }),
            opacity: c.opacity,
        }
    }
}

impl Interpolate for Rgb {
    /// d3's `interpolateRgb` at gamma 1 ([`interpolate::rgb`]), channel by
    /// channel and for the opacity (d3's `nogamma`): `a + t·(b − a)`, or the
    /// constant `a` when `b − a` is 0 or NaN, `b` when `a` is NaN, so an
    /// undefined channel takes the other color's.
    fn lerp(&self, to: &Rgb, t: f64) -> Rgb {
        interpolate::rgb(*self, *to)(t)
    }
}

impl Rgb {
    /// Channels `r`, `g`, `b`, opaque.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Rgb {
            r,
            g,
            b,
            opacity: 1.0,
        }
    }

    /// The opaque color `0xrrggbb`; bits above the lowest 24 are ignored.
    pub const fn hex(v: u32) -> Self {
        Rgb::new(
            ((v >> 16) & 0xff) as f64,
            ((v >> 8) & 0xff) as f64,
            (v & 0xff) as f64,
        )
    }

    /// The same channels at `opacity`.
    pub fn with_opacity(self, opacity: f64) -> Self {
        Rgb { opacity, ..self }
    }

    /// d3's `brighter(k)`: the channels times (1/0.7)^k.
    pub fn brighter(self, k: f64) -> Self {
        self.scaled(BRIGHTER.powf(k))
    }

    /// d3's `darker(k)`: the channels times 0.7^k.
    pub fn darker(self, k: f64) -> Self {
        self.scaled(DARKER.powf(k))
    }

    /// d3's `clamp()`: channels rounded and clamped to 0..=255 (NaN as 0),
    /// opacity clamped to [0, 1] (NaN as 1).
    pub fn clamp(self) -> Self {
        let [r, g, b] = [self.r, self.g, self.b].map(|c| f64::from(clamp_channel(c)));
        Rgb {
            r,
            g,
            b,
            opacity: super::clamp_opacity(self.opacity),
        }
    }

    /// d3's `displayable()`: each channel in [−0.5, 255.5), so it rounds
    /// into 0..=255, and the opacity in [0, 1].
    pub fn displayable(self) -> bool {
        let channel = |c: f64| (-0.5..255.5).contains(&c);
        channel(self.r) && channel(self.g) && channel(self.b) && (0.0..=1.0).contains(&self.opacity)
    }

    /// d3's `formatHex()`: `#rrggbb` of the rounded, clamped channels.
    pub fn format_hex(self) -> String {
        let [r, g, b] = [self.r, self.g, self.b].map(clamp_channel);
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// d3's `formatHex8()`: `#rrggbbaa`, the opacity (NaN as 1) scaled to
    /// 0..=255 as a channel.
    pub fn format_hex8(self) -> String {
        let opacity = if self.opacity.is_nan() {
            1.0
        } else {
            self.opacity
        };
        let a = clamp_channel(opacity * 255.0);
        format!("{}{a:02x}", self.format_hex())
    }

    /// The channels times `k`, the opacity kept.
    fn scaled(self, k: f64) -> Self {
        Rgb {
            r: self.r * k,
            g: self.g * k,
            b: self.b * k,
            ..self
        }
    }
}

/// d3's `clampi`: `Math.round(c)`, NaN as 0, clamped to 0..=255.
fn clamp_channel(c: f64) -> u8 {
    let r = js::round(c);
    if r.is_nan() {
        0
    } else {
        // In range after the clamp, so the cast is exact.
        r.clamp(0.0, 255.0) as u8
    }
}
