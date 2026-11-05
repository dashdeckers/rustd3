//! `array`'s statistics, quantiles, ranges, bisection and bins against
//! d3-array 3.2.4 (d3 7.9.0 in JavaScript).

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

/// Each bucket as (x0, x1, how many values).
fn lengths(buckets: &[array::bin::Bucket]) -> Vec<(f64, f64, usize)> {
    buckets
        .iter()
        .map(|b| (b.x0, b.x1, b.indices.len()))
        .collect()
}

const FIBONACCI: [f64; 9] = [0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0];

#[test]
fn the_default_bin_nices_the_extent_and_widens_the_last_bucket() {
    let expected = [
        (0.0, 5.0, 5),
        (5.0, 10.0, 2),
        (10.0, 15.0, 1),
        (15.0, 20.0, 0),
        (20.0, 25.0, 1),
    ];
    assert_eq!(lengths(&array::bin().layout(&FIBONACCI)), expected);
}

#[test]
fn a_fixed_domain_drops_the_values_outside_it() {
    let bin = array::bin()
        .domain([0.0, 20.0])
        .thresholds(array::bin::Thresholds::Count(4));
    let expected = [
        (0.0, 5.0, 5),
        (5.0, 10.0, 2),
        (10.0, 15.0, 1),
        (15.0, 20.0, 0),
    ];
    assert_eq!(lengths(&bin.layout(&FIBONACCI)), expected);
}

#[test]
fn given_thresholds_cut_the_extent() {
    let bin = array::bin().thresholds(array::bin::Thresholds::Values(vec![5.0, 10.0]));
    let expected = [(0.0, 5.0, 5), (5.0, 10.0, 2), (10.0, 21.0, 2)];
    assert_eq!(lengths(&bin.layout(&FIBONACCI)), expected);
}

#[test]
fn buckets_hold_the_indices_of_their_values() {
    let buckets = array::bin().layout(&[3.0, 12.0, 1.0]);
    assert_eq!(buckets[0].indices, [0, 2]);
}

#[test]
fn the_threshold_rules_count_as_d3() {
    let values = array::range(0.0, 20.0, 1.0);
    assert_eq!(array::bin::sturges(&values), 6);
    assert_eq!(array::bin::scott(&values, 0.0, 19.0), 3);
    assert_eq!(array::bin::freedman_diaconis(&values, 0.0, 19.0), 3);
    let at = |n: f64| array::bin::sturges(&array::range(0.0, n, 1.0));
    assert_eq!(
        [at(1.0), at(8.0), at(1024.0), at(1_048_576.0)],
        [1, 4, 11, 21]
    );
}

#[test]
fn infinite_or_missing_values_bin_as_d3_without_hanging() {
    let (inf, minus) = (f64::INFINITY, f64::NEG_INFINITY);
    let layout = |data: Vec<f64>| within_a_second(move || lengths(&array::bin().layout(&data)));
    assert_eq!(layout(vec![inf, minus]), [(minus, inf, 2)]);
    assert_eq!(layout(vec![1.0, inf]), [(1.0, inf, 2)]);
    assert_eq!(layout(vec![minus, 0.0, 1.0, inf]), [(minus, inf, 4)]);
    assert_eq!(layout(vec![inf]), [(inf, inf, 1)]);
    for nothing in [vec![NAN, NAN], Vec::new()] {
        let [(x0, x1, n)] = layout(nothing)[..] else {
            panic!("one bucket");
        };
        assert!(x0.is_nan() && x1.is_nan() && n == 0);
    }
}

/// d3-array's `bin-test.js` "bin(data) assigns floating point values to
/// the correct bins", which the translator skips (a `for…of` loop). n bins
/// over `ticks(1, 2, n)` hold one value each. From n = 10,000 the
/// 10,000-tick ceiling applies. `ticks` then gives no values, and over the
/// same 10,001 values (k / n for k from n to 2n, as d3 computes them) the
/// thresholds are capped too, so one bucket holds them all where d3 has
/// one per value.
#[test]
fn floating_point_ticks_land_one_per_bin() {
    for n in [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000] {
        let values = rustd3_core::ticks::ticks(1.0, 2.0, f64::from(n));
        let buckets = array::bin()
            .thresholds(array::bin::Thresholds::Count(usize::try_from(n).unwrap()))
            .layout(&values);
        assert!(
            buckets.iter().all(|b| b.indices.len() == 1),
            "{n} bins: {:?}",
            buckets.iter().map(|b| b.indices.len()).collect::<Vec<_>>()
        );
    }
    for n in [10_000, 20_000, 50_000] {
        assert!(rustd3_core::ticks::ticks(1.0, 2.0, f64::from(n)).is_empty());
        let values: Vec<f64> = (n..=2 * n).map(|k| f64::from(k) / f64::from(n)).collect();
        let buckets = array::bin()
            .thresholds(array::bin::Thresholds::Count(usize::try_from(n).unwrap()))
            .layout(&values);
        assert_eq!(buckets.len(), 1, "{n}");
        assert_eq!(buckets[0].indices.len(), values.len(), "{n}");
    }
}
