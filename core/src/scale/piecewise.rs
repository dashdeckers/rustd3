//! d3's polylinear scales: [`piecewise`], a linear scale with N ≥ 2 stops.

use crate::interpolate::Interpolate;
use crate::scale::{Continuous, transform};

/// A linear scale with N ≥ 2 stops: each domain stop maps to the range
/// value at the same index, and values between stops interpolate within
/// their segment (d3's polymap). Domain and range lengths are checked by
/// the compiler, where d3 truncates silently; `piecewise([0.0], [1.0])`
/// fails to compile when the function is instantiated. Nice and ticks act
/// on the end stops, as in d3.
///
/// ```
/// use rustd3_core::scale;
///
/// let s = scale::piecewise([-10.0, 0.0, 100.0], [0.0, 50.0, 100.0]);
/// assert_eq!(s.apply(-5.0), 25.0);
/// assert_eq!(s.apply(50.0), 75.0);
/// assert_eq!(s.invert(75.0), 50.0);
/// ```
///
/// A domain and range of different lengths do not compile:
///
/// ```compile_fail
/// use rustd3_core::scale;
///
/// scale::piecewise([0.0, 50.0, 100.0], [1.0, 2.0]);
/// ```
///
/// ```
/// use rustd3_core::scale;
///
/// scale::piecewise([0.0, 50.0, 100.0], [1.0, 2.0, 3.0]);
/// ```
#[doc(alias = "polylinear")]
pub fn piecewise<R: Interpolate, const N: usize>(
    domain: [f64; N],
    range: [R; N],
) -> Continuous<transform::Identity, R, N> {
    const { assert!(N >= 2, "piecewise needs at least two stops") };
    Continuous::with_stops(transform::Identity, domain, range)
}
