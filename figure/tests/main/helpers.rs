//! The `helpers` feature: its statistics against matplotlib 3.11.2, numpy
//! and scipy (`dev/fixtures/stats.json`, written by `stats.py`) through the
//! tolerant `Obs` compare, and its charts and grids as figures.

mod boxplot;
mod subplots;

use rustd3_testkit::fixture::Fixture;
use rustd3_testkit::obs::{Obs, ToObs, Tolerance};

/// The fixture, read at compile time so the test needs no working directory.
const STATS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dev/fixtures/stats.json"
));

/// The section's statistical data, n = 15.
pub const X: [f64; 15] = [
    1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0,
];

/// The fixture's value at `pointer` (`/boxplot/iqr/q1`).
pub fn fixture(pointer: &str) -> Obs {
    Fixture::parse(STATS)
        .expect("the fixture")
        .get(pointer)
        .expect("a value at the pointer")
}

/// Asserts that `actual` equals the fixture's value at `pointer` within
/// the tolerant compare's default bounds (1e-9).
pub fn assert_fixture(actual: impl ToObs, pointer: &str) {
    let (actual, expected) = (actual.to_obs(), fixture(pointer));
    assert!(
        actual.approx_eq(&expected, &Tolerance::default()),
        "{pointer}: {actual:?}, expected {expected:?}"
    );
}
