//! d3.piecewise: one interpolator per neighbouring pair of values.

use std::sync::Arc;

/// d3.piecewise(interpolate, values): `interpolator` built for each pair of
/// neighbouring `values`, and `t` in [0, 1] spread over the N − 1
/// segments: segment `i = floor(t·(N − 1))`, clamped to the first and
/// last, evaluated at `t·(N − 1) − i`, so `t` outside [0, 1] extrapolates
/// the end segments. Fewer than two values fail to compile when the
/// function is instantiated, where d3 fails when called.
///
/// ```
/// use rustd3_core::color::Rgb;
/// use rustd3_core::interpolate;
///
/// let numbers = interpolate::piecewise(&[0.0, 2.0, 10.0], |a, b| interpolate::number(*a, *b));
/// assert_eq!((numbers(-1.0), numbers(0.75)), (-4.0, 6.0));
/// let [red, green, blue] = [Rgb::hex(0xff0000), Rgb::hex(0x008000), Rgb::hex(0x0000ff)];
/// let colors = interpolate::piecewise(&[red, green, blue], |a, b| interpolate::rgb(*a, *b));
/// assert_eq!(colors(0.75).to_string(), "rgb(0, 64, 128)");
/// ```
///
/// One value does not compile:
///
/// ```compile_fail
/// use rustd3_core::interpolate;
///
/// interpolate::piecewise(&[1.0], |a, b| interpolate::number(*a, *b))(0.5);
/// ```
///
/// ```
/// use rustd3_core::interpolate;
///
/// interpolate::piecewise(&[1.0, 2.0], |a, b| interpolate::number(*a, *b))(0.5);
/// ```
#[doc(alias = "interpolatePiecewise")]
pub fn piecewise<T, G, U, F, const N: usize>(
    values: &[T; N],
    interpolator: F,
) -> impl Fn(f64) -> U + Clone + Send + Sync + use<T, G, U, F, N>
where
    T: Clone + Send + Sync + 'static,
    G: Fn(f64) -> U + Clone + Send + Sync,
    F: Fn(&T, &T) -> G,
{
    const { assert!(N >= 2, "piecewise needs at least two values") };
    let segments: Arc<[G]> = values
        .windows(2)
        .map(|pair| interpolator(&pair[0], &pair[1]))
        .collect();
    let n = (N - 1) as f64;
    move |t| {
        let t = t * n;
        let i = t.floor().clamp(0.0, n - 1.0);
        // NaN stays NaN through the clamp and casts to 0, where d3 throws.
        segments[i as usize](t - i)
    }
}
