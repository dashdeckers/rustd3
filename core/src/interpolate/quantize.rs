//! d3.quantize: evenly spaced samples of an interpolator.

/// d3.quantize(interpolator, n): `f` sampled at `i / (n − 1)` for `i` in
/// `0..n`, so the first sample is `f(0)` and the last `f(1)`; one sample
/// is `f(NaN)`, as in d3.
///
/// ```
/// use rustd3_core::interpolate;
///
/// let samples = interpolate::quantize(interpolate::number(0.0, 10.0), 4);
/// assert_eq!(samples, [0.0, 3.333333333333333, 6.666666666666666, 10.0]);
/// ```
pub fn quantize<T>(f: impl Fn(f64) -> T, n: usize) -> Vec<T> {
    let last = n as f64 - 1.0;
    (0..n).map(|i| f(i as f64 / last)).collect()
}
