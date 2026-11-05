//! d3.quantize: evenly spaced samples of an interpolator.

/// The most samples: lengths of 2³¹ or more are not allocated.
const MOST: usize = i32::MAX as usize;

/// d3.quantize(interpolator, n): `f` sampled at `i / (n − 1)` for `i` in
/// `0..n`, so the first sample is `f(0)` and the last `f(1)`. A single
/// sample is `f(NaN)`, as in d3. It is empty when `n` is 2³¹ or more, a
/// length that [`array::range`](crate::array::range) does not allocate
/// either; d3 throws from 2³² on.
///
/// ```
/// use rustd3_core::interpolate;
///
/// let samples = interpolate::quantize(interpolate::number(0.0, 10.0), 4);
/// assert_eq!(samples, [0.0, 3.333333333333333, 6.666666666666666, 10.0]);
/// ```
pub fn quantize<T>(f: impl Fn(f64) -> T, n: usize) -> Vec<T> {
    if n > MOST {
        return Vec::new();
    }
    let last = n as f64 - 1.0;
    (0..n).map(|i| f(i as f64 / last)).collect()
}
