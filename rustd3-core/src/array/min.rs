//! d3.min.

/// d3.min: the least of `values` that is not NaN; `None` when there is
/// none, where d3 returns undefined.
pub fn min(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut min = None;
    for v in values {
        // d3's `min > value || (min === undefined && value >= value)`.
        if min.map_or(!v.is_nan(), |m| m > v) {
            min = Some(v);
        }
    }
    min
}

/// d3.minIndex: the position of the least of `values` that is not NaN,
/// the first of equals; `None` when there is none, where d3 returns -1.
///
/// ```
/// use rustd3_core::array::min_index;
///
/// assert_eq!(min_index([3.0, f64::NAN, 1.0, 1.0]), Some(2));
/// assert_eq!(min_index([f64::NAN]), None);
/// ```
pub fn min_index(values: impl IntoIterator<Item = f64>) -> Option<usize> {
    let mut min: Option<(usize, f64)> = None;
    for (i, v) in values.into_iter().enumerate() {
        if min.map_or(!v.is_nan(), |(_, m)| m > v) {
            min = Some((i, v));
        }
    }
    min.map(|(i, _)| i)
}
