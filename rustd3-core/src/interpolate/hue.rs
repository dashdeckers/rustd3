//! d3.interpolateHue.

use crate::interpolate::gamma::Channel;

/// d3.interpolateHue: hue `a` to hue `b` in degrees along the shorter arc,
/// wrapped into [0, 360). An undefined (NaN) hue takes the other's.
///
/// ```
/// use rustd3_core::interpolate;
///
/// assert_eq!(interpolate::hue(350.0, 10.0)(0.5), 0.0);
/// assert_eq!(interpolate::hue(10.0, 350.0)(0.6), 358.0);
/// assert_eq!(interpolate::hue(f64::NAN, 20.0)(0.5), 20.0);
/// ```
#[doc(alias = "interpolateHue")]
pub fn hue(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync {
    let h = Channel::hue(a, b);
    move |t| {
        let x = h.at(t);
        x - 360.0 * (x / 360.0).floor()
    }
}
