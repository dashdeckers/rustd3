//! Tick rules: what a [`scale::Transform`](crate::scale::Transform) calls
//! to state its ticks and minor ticks. Each returns values in the order of
//! its domain or majors, at most 10,000 of them.

use crate::scale::Transform;
use crate::ticks;

/// The most values a built-in minor rule returns.
const MAX_VALUES: usize = 10_000;

/// d3's linear ticks in data space: `ticks(d[0], d[1], count)`.
pub fn linear(domain: [f64; 2], count: usize) -> Vec<f64> {
    ticks::ticks(domain[0], domain[1], count as f64)
}

/// `k` equal steps in transformed space between neighbouring majors (the
/// majors themselves excluded), stopping after 10,000 values.
pub fn subdivide<T: Transform>(transform: &T, major: &[f64], k: usize) -> Vec<f64> {
    major
        .windows(2)
        .flat_map(|pair| {
            let (a, b) = (transform.forward(pair[0]), transform.forward(pair[1]));
            (1..k).map(move |i| a + (b - a) * i as f64 / k as f64)
        })
        .map(|u| transform.inverse(u))
        .take(MAX_VALUES)
        .collect()
}
