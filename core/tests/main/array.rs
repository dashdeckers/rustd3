//! `array`'s statistics, quantiles, ranges, bisection and bins against
//! d3-array 3.2.4 (d3 7.9.0 in Node).

use rustd3_core::array;

const NAN: f64 = f64::NAN;

#[test]
fn reductions_skip_nan_as_d3_skips_undefined() {
    assert_eq!(array::mean([1.0, 2.0, NAN, 4.0]), Some(2.3333333333333335));
    assert_eq!(array::extent([3.0, NAN, -1.0, 7.0]), Some([-1.0, 7.0]));
    assert_eq!(array::count([3.0, NAN, -1.0, 7.0]), 3);
}

#[test]
fn reductions_of_nothing_are_none_and_a_sum_is_zero() {
    assert_eq!(array::min(Vec::new()), None);
    assert_eq!(array::max(Vec::new()), None);
    assert_eq!(array::mean([NAN]), None);
    assert_eq!(array::sum(Vec::new()), 0.0);
}

#[test]
fn a_sum_adds_in_order() {
    assert_eq!(array::sum([0.1, 0.2, 0.3]), 0.6000000000000001);
}

#[test]
fn variance_is_over_n_minus_one_and_deviation_its_root() {
    assert_eq!(array::variance([5.0, 1.0, 2.0, 3.0, 4.0]), Some(2.5));
    assert_eq!(
        array::deviation([5.0, 1.0, 2.0, 3.0, 4.0]),
        Some(1.5811388300841898)
    );
    assert_eq!(array::variance([1.0]), None);
}
