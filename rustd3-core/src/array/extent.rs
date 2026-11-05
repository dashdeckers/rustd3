//! d3.extent.

/// d3.extent: the least and greatest of `values` that are not NaN, in one
/// pass; `None` when there is none, where d3 returns `[undefined,
/// undefined]`.
pub fn extent(values: impl IntoIterator<Item = f64>) -> Option<[f64; 2]> {
    let mut extent: Option<[f64; 2]> = None;
    for v in values {
        match &mut extent {
            None if v.is_nan() => {}
            None => extent = Some([v, v]),
            Some([min, max]) => {
                if *min > v {
                    *min = v;
                }
                if *max < v {
                    *max = v;
                }
            }
        }
    }
    extent
}
