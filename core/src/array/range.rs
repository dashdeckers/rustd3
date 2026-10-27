//! d3.range.

/// d3.range(start, stop, step): `start + i·step` for `i` in `0..n`, `n =
/// ⌈(stop − start)/step⌉`, so `range(0.0, 1.0, 0.2)` ends with
/// 0.6000000000000001 as in d3. Empty, without looping, when `n` is not a
/// positive number below 2³¹ (d3 truncates it to a 32-bit integer): a zero
/// or NaN step, a NaN bound, an infinite length, or a step against the
/// direction from `start` to `stop`.
pub fn range(start: f64, stop: f64, step: f64) -> Vec<f64> {
    let n = ((stop - start) / step).ceil();
    if !(n > 0.0 && n <= f64::from(i32::MAX)) {
        return Vec::new();
    }
    (0..n as usize).map(|i| start + i as f64 * step).collect()
}
