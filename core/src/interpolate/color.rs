//! d3-interpolate's color interpolators: two colors blended in one color
//! space, returned as [`Rgb`]. Each takes its space's type; callers convert
//! (`Hsl::from(rgb)`), where d3 dispatches on strings.

use crate::color::{Cubehelix, Hcl, Hsl, Lab, Rgb};
use crate::interpolate::Interpolate;
use crate::interpolate::gamma::Channel;

/// d3.interpolateRgb: `a` to `b` channel by channel in sRGB; an undefined
/// channel takes the other color's.
///
/// ```
/// use rustd3_core::color::Rgb;
/// use rustd3_core::interpolate;
///
/// let steel: Rgb = "steelblue".parse()?;
/// let brown: Rgb = "brown".parse()?;
/// assert_eq!(interpolate::rgb(steel, brown)(0.5).to_string(), "rgb(118, 86, 111)");
/// assert_eq!(interpolate::rgb_gamma(2.2, steel, brown)(0.5).to_string(), "rgb(128, 98, 134)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[doc(alias = "interpolateRgb")]
pub fn rgb(a: Rgb, b: Rgb) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    rgb_gamma(1.0, a, b)
}

/// d3.interpolateRgb.gamma(gamma): the sRGB channels blended along d3's
/// gamma curve `(a^γ + t·(b^γ − a^γ))^(1/γ)`, the opacity linearly; gamma
/// 1 is [`rgb`].
pub fn rgb_gamma(gamma: f64, a: Rgb, b: Rgb) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    let [r, g, blue] =
        [(a.r, b.r), (a.g, b.g), (a.b, b.b)].map(|(x, y)| Channel::gamma(gamma, x, y));
    let opacity = Channel::nogamma(a.opacity, b.opacity);
    move |t| Rgb {
        r: r.at(t),
        g: g.at(t),
        b: blue.at(t),
        opacity: opacity.at(t),
    }
}

/// d3.interpolateHsl: the hue along the shorter arc, saturation,
/// lightness and opacity linearly; an undefined component takes the other
/// color's.
///
/// ```
/// use rustd3_core::color::Hsl;
/// use rustd3_core::interpolate;
///
/// let [steel, brown]: [Hsl; 2] = ["steelblue".parse()?, "brown".parse()?];
/// assert_eq!(interpolate::hsl(steel, brown)(0.5).to_string(), "rgb(141, 55, 173)");
/// assert_eq!(interpolate::hsl_long(steel, brown)(0.5).to_string(), "rgb(87, 173, 55)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[doc(alias = "interpolateHsl")]
pub fn hsl(a: Hsl, b: Hsl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    hsl_by(Channel::hue, a, b)
}

/// d3.interpolateHslLong: [`hsl`] with the hue blended linearly, the long
/// way round when that is longer.
#[doc(alias = "interpolateHslLong")]
pub fn hsl_long(a: Hsl, b: Hsl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    hsl_by(Channel::nogamma, a, b)
}

/// d3.interpolateLab: every channel linearly in CIELAB; an undefined
/// channel takes the other color's.
///
/// ```
/// use rustd3_core::color::Lab;
/// use rustd3_core::interpolate;
///
/// let [steel, brown]: [Lab; 2] = ["steelblue".parse()?, "brown".parse()?];
/// assert_eq!(interpolate::lab(steel, brown)(0.5).to_string(), "rgb(139, 93, 108)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[doc(alias = "interpolateLab")]
pub fn lab(a: Lab, b: Lab) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    move |t| Rgb::from(a.lerp(&b, t))
}

/// d3.interpolateHcl: the hue along the shorter arc, chroma, luminance
/// and opacity linearly. An undefined hue or chroma takes the other
/// color's, so white blends into a color along that color's hue.
///
/// ```
/// use rustd3_core::color::Hcl;
/// use rustd3_core::interpolate;
///
/// let [white, steel]: [Hcl; 2] = ["white".parse()?, "steelblue".parse()?];
/// assert_eq!(interpolate::hcl(white, steel)(0.5).to_string(), "rgb(138, 194, 248)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[doc(alias = "interpolateHcl")]
pub fn hcl(a: Hcl, b: Hcl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    hcl_by(Channel::hue, a, b)
}

/// d3.interpolateHclLong: [`hcl`] with the hue blended linearly.
#[doc(alias = "interpolateHclLong")]
pub fn hcl_long(a: Hcl, b: Hcl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    hcl_by(Channel::nogamma, a, b)
}

/// d3.interpolateCubehelix.gamma(gamma): the hue along the shorter arc,
/// saturation and opacity linearly, and the lightness evaluated at
/// `t^gamma`; d3.interpolateCubehelix is gamma 1.
///
/// ```
/// use rustd3_core::color::Cubehelix;
/// use rustd3_core::interpolate;
///
/// let [purple, orange]: [Cubehelix; 2] = ["purple".parse()?, "orange".parse()?];
/// assert_eq!(interpolate::cubehelix(3.0, purple, orange)(0.5).to_string(), "rgb(207, 4, 32)");
/// # Ok::<(), rustd3_core::color::ParseColorError>(())
/// ```
#[doc(alias = "interpolateCubehelix")]
pub fn cubehelix(
    gamma: f64,
    a: Cubehelix,
    b: Cubehelix,
) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    cubehelix_by(Channel::hue, gamma, a, b)
}

/// d3.interpolateCubehelixLong.gamma(gamma): [`cubehelix`] with the hue
/// blended linearly.
#[doc(alias = "interpolateCubehelixLong")]
pub fn cubehelix_long(
    gamma: f64,
    a: Cubehelix,
    b: Cubehelix,
) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    cubehelix_by(Channel::nogamma, gamma, a, b)
}

/// The HSL blend with the hue blended by `hue`.
fn hsl_by(
    hue: fn(f64, f64) -> Channel,
    a: Hsl,
    b: Hsl,
) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    let [h, s, l, opacity] =
        Channel::four([a.h, a.s, a.l, a.opacity], [b.h, b.s, b.l, b.opacity], hue);
    move |t| {
        Rgb::from(Hsl {
            h: h.at(t),
            s: s.at(t),
            l: l.at(t),
            opacity: opacity.at(t),
        })
    }
}

/// The HCL blend with the hue blended by `hue`.
fn hcl_by(
    hue: fn(f64, f64) -> Channel,
    a: Hcl,
    b: Hcl,
) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    let [h, c, l, opacity] =
        Channel::four([a.h, a.c, a.l, a.opacity], [b.h, b.c, b.l, b.opacity], hue);
    move |t| {
        Rgb::from(Hcl {
            h: h.at(t),
            c: c.at(t),
            l: l.at(t),
            opacity: opacity.at(t),
        })
    }
}

/// The cubehelix blend with the hue blended by `hue` and the lightness at
/// `t^gamma`.
fn cubehelix_by(
    hue: fn(f64, f64) -> Channel,
    gamma: f64,
    a: Cubehelix,
    b: Cubehelix,
) -> impl Fn(f64) -> Rgb + Clone + Send + Sync {
    let [h, s, l, opacity] =
        Channel::four([a.h, a.s, a.l, a.opacity], [b.h, b.s, b.l, b.opacity], hue);
    move |t| {
        Rgb::from(Cubehelix {
            h: h.at(t),
            s: s.at(t),
            l: l.at(t.powf(gamma)),
            opacity: opacity.at(t),
        })
    }
}
