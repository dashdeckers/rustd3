//! d3.quantile and d3.quantileSorted.

use crate::array::{max, min, quickselect};

/// d3.quantile: the `p`-quantile of `values` that are not NaN by the R-7
/// method, interpolating between the order statistics around `(n − 1)·p`
/// found by quickselect on a copy; the least value for `p ≤ 0` or fewer
/// than two values, the greatest for `p ≥ 1`. `None` for no values or a
/// NaN `p`.
pub fn quantile(values: impl IntoIterator<Item = f64>, p: f64) -> Option<f64> {
    let mut values: Vec<f64> = values.into_iter().filter(|v| !v.is_nan()).collect();
    let n = values.len();
    if n == 0 || p.is_nan() {
        return None;
    }
    if p <= 0.0 || n < 2 {
        return min(values);
    }
    if p >= 1.0 {
        return max(values);
    }
    let i = (n - 1) as f64 * p;
    let i0 = i.floor();
    let below = i0 as usize;
    quickselect(&mut values, below);
    let value0 = max(values[..=below].iter().copied())?;
    let value1 = min(values[below + 1..].iter().copied())?;
    Some(value0 + (value1 - value0) * (i - i0))
}

/// d3.quantileSorted: the `p`-quantile of `sorted` by the R-7 method, read
/// from the order it is given in (ascending for a meaningful result);
/// `None` for an empty slice or a NaN `p`.
#[must_use]
pub fn quantile_sorted(sorted: &[f64], p: f64) -> Option<f64> {
    let n = sorted.len();
    if n == 0 || p.is_nan() {
        return None;
    }
    if p <= 0.0 || n < 2 {
        return Some(sorted[0]);
    }
    if p >= 1.0 {
        return Some(sorted[n - 1]);
    }
    let i = (n - 1) as f64 * p;
    let i0 = i.floor();
    let (value0, value1) = (sorted[i0 as usize], sorted[i0 as usize + 1]);
    Some(value0 + (value1 - value0) * (i - i0))
}
