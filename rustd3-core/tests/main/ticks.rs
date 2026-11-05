//! `ticks`, `tick_increment`, `tick_step` and `nice` against d3-array,
//! and the bail-outs where d3 returns `[NaN]`, throws or allocates.

use rustd3_core::ticks::{nice, tick_increment, tick_step, ticks};
use rustd3_testkit::obs::{ToObs, Tolerance};

/// Tolerant equality of two tick lists, value by value.
fn tolerant_eq(actual: &[f64], expected: &[f64]) -> bool {
    actual
        .to_obs()
        .approx_eq(&expected.to_obs(), &Tolerance::default())
}

#[test]
fn ten_ticks_on_the_unit_interval_are_tenths() {
    let tenths: Vec<f64> = (0..=10).map(|k| f64::from(k) / 10.0).collect();
    assert_eq!(ticks(0.0, 1.0, 10.0), tenths);
}

#[test]
fn ticks_follow_the_direction_of_the_bounds() {
    let evens: Vec<f64> = (-5..=5).map(|k| f64::from(k) * 2.0).collect();
    assert_eq!(ticks(-10.0, 10.0, 10.0), evens);
    assert_eq!(ticks(1.0, 0.0, 5.0), [1.0, 0.8, 0.6, 0.4, 0.2, 0.0]);
}

#[test]
fn nan_bounds_infinite_spans_and_non_positive_counts_give_no_ticks() {
    assert!(ticks(0.0, 1.0, f64::NAN).is_empty());
    assert!(ticks(f64::NAN, 1.0, 10.0).is_empty());
    assert!(ticks(0.0, f64::INFINITY, 10.0).is_empty());
    assert!(ticks(0.0, 1.0, 0.0).is_empty());
    assert!(ticks(0.0, 1.0, -1.0).is_empty());
    assert!(ticks(0.0, 1.0, f64::INFINITY).is_empty());
}

#[test]
fn equal_bounds_and_small_counts_still_give_ticks() {
    assert_eq!(ticks(1.0, 1.0, 10.0), [1.0]);
    assert_eq!(ticks(0.0, 1.0, 0.5), [0.0]);
    assert_eq!(ticks(0.0, 10.0, 1.5), [0.0, 5.0, 10.0]);
    assert_eq!(ticks(1.0, 364.0, 0.5), [200.0]);
}

#[test]
fn ticks_far_from_zero_keep_their_tenths() {
    let expected: Vec<f64> = (10_000_000_010_i64..=10_000_000_020)
        .map(|k| k as f64 / 10.0)
        .collect();
    assert_eq!(ticks(1e9 + 1.0, 1e9 + 2.0, 10.0), expected);
    assert_eq!(expected[1], 1_000_000_001.1);
}

#[test]
fn ticks_up_to_the_largest_float_are_steps_of_two_e307() {
    let expected: Vec<f64> = (0..9).map(|k| f64::from(k) * 2e307).collect();
    assert!(tolerant_eq(&ticks(0.0, f64::MAX, 10.0), &expected));
    assert_eq!(ticks(0.0, f64::MAX, 10.0).len(), 9);
}

#[test]
fn an_overflowing_increment_gives_no_ticks_where_d3_gives_nan() {
    assert!(ticks(-1e308, 1e308, 10.0).is_empty());
}

#[test]
fn a_subnormal_span_gives_no_ticks_where_d3_throws() {
    assert!(ticks(1e-320, 1e-319, 10.0).is_empty());
}

#[test]
fn tick_counts_are_capped_at_ten_times_the_request() {
    assert!(ticks(0.0, 1.0, 1e6).is_empty());
    assert_eq!(ticks(0.0, 1.0, 1000.0).len(), 1001);
}

#[test]
fn tick_increment_is_the_negated_inverse_below_one() {
    assert_eq!(tick_increment(0.0, 1.0, 10.0), -10.0);
    assert_eq!(tick_increment(0.0, 10.0, 10.0), 1.0);
    assert!(tick_increment(0.0, 1.0, f64::NAN).is_nan());
    assert_eq!(tick_increment(1.0, 1.0, 1.0), f64::NEG_INFINITY);
}

#[test]
fn tick_step_is_signed_by_the_direction() {
    assert_eq!(tick_step(0.0, 1.0, 10.0), 0.1);
    assert_eq!(tick_step(1.0, 0.0, 10.0), -0.1);
}

#[test]
fn nice_extends_the_bounds_to_ticks() {
    assert_eq!(nice(0.13, 9.71, 5.0), [0.0, 10.0]);
    assert_eq!(nice(1.1, 10.9, 10.0), [1.0, 11.0]);
    assert_eq!(nice(132.0, 876.0, 5.0), [0.0, 1000.0]);
    assert_eq!(nice(0.0, 1.0, f64::INFINITY), [0.0, 1.0]);
}
