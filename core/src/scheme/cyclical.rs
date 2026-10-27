//! d3-scale-chromatic's cyclical schemes: sinebow and rainbow.

use std::f64::consts::PI;

use crate::color::{Cubehelix, Rgb};

/// d3's `interpolateSinebow` (Stoddard's sinebow): with
/// `u = (0.5 − t)·π`, the channels `255·sin²(u)`, `255·sin²(u + π/3)` and
/// `255·sin²(u + 2π/3)`.
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::sinebow(0.3).to_string(), "rgb(88, 252, 42)");
/// assert_eq!(scheme::sinebow(0.0).to_string(), scheme::sinebow(1.0).to_string());
/// ```
#[doc(alias = "interpolateSinebow")]
pub fn sinebow(t: f64) -> Rgb {
    let u = (0.5 - t) * PI;
    let channel = |x: f64| {
        let s = x.sin();
        255.0 * s * s
    };
    Rgb::new(
        channel(u),
        channel(u + PI / 3.0),
        channel(u + PI * 2.0 / 3.0),
    )
}

/// d3's `interpolateRainbow`, the less-angry rainbow: `t` outside [0, 1]
/// wrapped, then cubehelix with `h = 360t − 100`, `s = 1.5 − 1.5·|t − ½|`
/// and `l = 0.8 − 0.9·|t − ½|`.
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::rainbow(0.3).to_string(), "rgb(255, 120, 71)");
/// assert_eq!(scheme::rainbow(1.3), scheme::rainbow(0.30000000000000004));
/// ```
#[doc(alias = "interpolateRainbow")]
pub fn rainbow(t: f64) -> Rgb {
    let t = if (0.0..=1.0).contains(&t) {
        t
    } else {
        t - t.floor()
    };
    let ts = (t - 0.5).abs();
    Rgb::from(Cubehelix::new(
        360.0 * t - 100.0,
        1.5 - 1.5 * ts,
        0.8 - 0.9 * ts,
    ))
}
