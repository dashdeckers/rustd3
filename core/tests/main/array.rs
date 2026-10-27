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

/// `f`'s result, failing the test if it takes a second or more (a loop
/// d3's bail-outs should have prevented).
fn within_a_second<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || send.send(f()));
    receive
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("finished within a second")
}

#[test]
fn quantiles_follow_r7_between_order_statistics() {
    let even = [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];
    let at = |p| array::quantile(even, p);
    assert_eq!(
        (at(0.25), at(0.5), at(0.75)),
        (Some(7.25), Some(9.0), Some(14.5))
    );
    assert_eq!(array::quantile([1.0, 2.0, 3.0], NAN), None);
    assert_eq!(array::quantile_sorted(&[], 0.5), None);
    assert_eq!(array::median([1.0, NAN, 3.0, 2.0]), Some(2.0));
}

#[test]
fn a_range_steps_from_its_start() {
    assert_eq!(
        array::range(0.0, 1.0, 0.2),
        [0.0, 0.2, 0.4, 0.6000000000000001, 0.8]
    );
}

#[test]
fn a_degenerate_range_is_empty_without_looping() {
    assert!(within_a_second(|| array::range(0.0, 1.0, 0.0)).is_empty());
    assert!(within_a_second(|| array::range(0.0, f64::INFINITY, 1.0)).is_empty());
    assert!(within_a_second(|| array::range(0.0, 1e300, 1.0)).is_empty());
}

#[test]
fn bisection_finds_both_sides_of_equal_values() {
    let sorted = [1.0, 2.0, 2.0, 3.0];
    assert_eq!(array::bisect_left(&sorted, 2.0), 1);
    assert_eq!(array::bisect_right(&sorted, 2.0), 3);
}

#[test]
fn bisecting_nan_gives_the_length() {
    assert_eq!(array::bisect_left(&[1.0, 2.0, 3.0], NAN), 3);
    assert_eq!(array::bisect_right(&[1.0, 2.0, 3.0], NAN), 3);
}
