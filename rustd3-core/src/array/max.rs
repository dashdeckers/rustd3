//! d3.max.

/// d3.max: the greatest of `values` that is not NaN; `None` when there is
/// none, where d3 returns undefined.
pub fn max(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut max = None;
    for v in values {
        // d3's `max < value || (max === undefined && value >= value)`.
        if max.map_or(!v.is_nan(), |m| m < v) {
            max = Some(v);
        }
    }
    max
}

/// d3.maxIndex: the position of the greatest of `values` that is not NaN,
/// the first of equals; `None` when there is none, where d3 returns -1.
///
/// ```
/// use rustd3_core::array::max_index;
///
/// assert_eq!(max_index([3.0, f64::NAN, 5.0, 5.0]), Some(2));
/// assert_eq!(max_index([f64::NAN]), None);
/// ```
pub fn max_index(values: impl IntoIterator<Item = f64>) -> Option<usize> {
    let mut max: Option<(usize, f64)> = None;
    for (i, v) in values.into_iter().enumerate() {
        if max.map_or(!v.is_nan(), |(_, m)| m < v) {
            max = Some((i, v));
        }
    }
    max.map(|(i, _)| i)
}
