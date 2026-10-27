//! Phase 5's continuous scales: tick rules and the `ticks::` blocks, the
//! symlog, logit, asinh and func transforms, `piecewise` and
//! `AnyContinuous`, against d3-scale 4.0.2, matplotlib 3.11.2's locators
//! and the fscale prototype.

use rustd3_core::scale::{self, TickRule};
use rustd3_core::ticks;
use rustd3_testkit::obs::{ToObs, Tolerance};

/// Frequency laid out by period: Part I's user transform.
#[derive(Clone, Copy)]
struct Reciprocal;

impl scale::Transform for Reciprocal {
    fn forward(&self, x: f64) -> f64 {
        1.0 / x
    }

    fn inverse(&self, y: f64) -> f64 {
        1.0 / y
    }

    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
        ticks::log(10.0, d, n)
    }
}

/// Multiples of 3 from the first end stop to the last.
fn every_third([a, b]: [f64; 2], _count: usize) -> Vec<f64> {
    (0..)
        .map(|i| a + 3.0 * f64::from(i))
        .take_while(|v| *v <= b)
        .collect()
}

#[test]
fn a_custom_rule_replaces_the_transforms_ticks() {
    let s = scale::linear()
        .domain([0.0, 12.0])
        .tick_rule(TickRule::custom(every_third));
    assert_eq!(s.ticks(5), [0.0, 3.0, 6.0, 9.0, 12.0]);
}

#[test]
fn a_custom_rule_prints_without_its_function() {
    assert_eq!(format!("{:?}", TickRule::custom(every_third)), "Custom(..)");
    assert_eq!(format!("{:?}", TickRule::Data), "Data");
}

#[test]
fn the_tick_rule_survives_a_change_of_range_type() {
    let s = scale::linear()
        .tick_rule(TickRule::custom(every_third))
        .domain([0.0, 6.0])
        .range([0.0f32, 1.0]);
    assert_eq!(s.ticks(10), [0.0, 3.0, 6.0]);
}

#[test]
fn the_transformed_rule_ticks_in_transformed_space() {
    let s = scale::sqrt()
        .domain([0.0, 100.0])
        .tick_rule(TickRule::Transformed);
    assert_eq!(s.ticks(5), [0.0, 4.0, 16.0, 36.0, 64.0, 100.0]);
    assert_eq!(
        ticks::transformed(&scale::transform::Identity, [0.0, 1.0], 5),
        ticks::linear([0.0, 1.0], 5)
    );
}

#[test]
fn the_data_rule_gives_a_log_scale_d3s_linear_ticks() {
    let s = scale::log().domain([1.0, 1000.0]).tick_rule(TickRule::Data);
    assert_eq!(s.ticks(5), [200.0, 400.0, 600.0, 800.0, 1000.0]);
}

#[test]
fn minor_ticks_fall_between_the_majors_of_the_rule() {
    let s = scale::linear()
        .domain([0.0, 6.0])
        .tick_rule(TickRule::custom(every_third));
    assert_eq!(s.minor(10), [0.6, 1.2, 1.8, 2.4, 3.6, 4.2, 4.8, 5.4]);
}

#[test]
fn a_user_transform_ticks_by_decades() {
    let s = scale::continuous(Reciprocal).domain([1.0, 100.0]);
    let mut expected: Vec<f64> = (1..10).map(f64::from).collect();
    expected.extend((1..10).map(|k| f64::from(k) * 10.0));
    expected.push(100.0);
    assert_eq!(s.ticks(10), expected);
    let wide = scale::continuous(Reciprocal).domain([1.0, 1e6]);
    assert_eq!(wide.ticks(5), [1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]);
}

#[test]
fn decades_thin_to_every_other_decade_over_eighteen_orders() {
    assert_eq!(
        ticks::decades([-1e9, 1e9], 1.0, 10),
        [
            -1e8, -1e6, -1e4, -100.0, -1.0, 0.0, 1.0, 100.0, 1e4, 1e6, 1e8
        ]
    );
}

#[test]
fn decades_keep_every_decade_near_the_count() {
    assert_eq!(
        ticks::decades([0.0, 1e6], 1.0, 10),
        [0.0, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]
    );
    assert_eq!(
        ticks::decades([-1000.0, 1000.0], 1.0, 10),
        [-1000.0, -100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0, 1000.0]
    );
}

#[test]
fn decades_follow_the_domain_order_and_scale_with_the_constant() {
    assert_eq!(
        ticks::decades([100.0, -10.0], 2.0, 10),
        [20.0, 2.0, 0.0, -2.0]
    );
}

#[test]
fn decades_bail_out_on_a_bad_constant_or_an_infinite_end() {
    assert!(ticks::decades([0.0, 100.0], 0.0, 10).is_empty());
    assert!(ticks::decades([0.0, 100.0], f64::NAN, 10).is_empty());
    assert!(ticks::decades([0.0, f64::INFINITY], 1.0, 10).is_empty());
    assert!(ticks::decades([f64::NAN, 1.0], 1.0, 10).is_empty());
}

#[test]
fn decades_walk_at_most_four_hundred_decades() {
    let z = ticks::decades([0.0, f64::MAX], 1e-300, 1000);
    assert_eq!(z.len(), 401);
    assert!((z[400] / 1e99 - 1.0).abs() < 1e-12);
}

/// Whether `actual` is within the tolerance of d3's printed `expected`.
fn close(actual: f64, expected: f64) -> bool {
    actual
        .to_obs()
        .approx_eq(&expected.to_obs(), &Tolerance::default())
}

/// `symlog()` ticks(10) over `domain`.
fn symlog_ticks(domain: [f64; 2]) -> Vec<f64> {
    scale::symlog().domain(domain).ticks(10)
}

#[test]
fn symlog_ticks_a_million_by_decades() {
    assert_eq!(
        symlog_ticks([0.0, 1e6]),
        [0.0, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]
    );
}

#[test]
fn symlog_with_the_data_rule_ticks_as_d3() {
    let s = scale::symlog().domain([0.0, 1e6]).tick_rule(TickRule::Data);
    let expected: Vec<f64> = (0..=10).map(|k| f64::from(k) * 1e5).collect();
    assert_eq!(s.ticks(10), expected);
}

#[test]
fn symlog_ticks_both_sides_of_zero_by_decades() {
    assert_eq!(
        symlog_ticks([-1000.0, 1000.0]),
        [-1000.0, -100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0, 1000.0]
    );
    assert_eq!(
        symlog_ticks([-1e9, 1e9]),
        [
            -1e8, -1e6, -1e4, -100.0, -1.0, 0.0, 1.0, 100.0, 1e4, 1e6, 1e8
        ]
    );
}

#[test]
fn symlog_ticks_linearly_under_a_hundred_constants() {
    let threes: Vec<f64> = (-6..=6).map(|k| f64::from(k) * 0.5).collect();
    assert_eq!(symlog_ticks([-3.0, 3.0]), threes);
    let fives: Vec<f64> = (-1..=10).map(|k| f64::from(k) * 5.0).collect();
    assert_eq!(symlog_ticks([-5.0, 50.0]), fives);
}

#[test]
fn symlog_decades_keep_to_the_domain() {
    assert_eq!(symlog_ticks([0.0, 100.0]), [0.0, 1.0, 10.0, 100.0]);
    assert_eq!(symlog_ticks([1.0, 1e4]), [1.0, 10.0, 100.0, 1e3, 1e4]);
    assert_eq!(
        symlog_ticks([-1e4, 10.0]),
        [-1e4, -1e3, -100.0, -10.0, -1.0, 0.0, 1.0, 10.0]
    );
}

#[test]
fn symlog_falls_back_to_linear_ticks_inside_one_decade() {
    let expected: Vec<f64> = (10..=20).map(|k| f64::from(k) * 100.0).collect();
    assert_eq!(symlog_ticks([1000.0, 2000.0]), expected);
}

#[test]
fn symlog_minor_ticks_are_whole_multiples_per_decade() {
    let minor: Vec<f64> = scale::symlog()
        .domain([-1e3, 1e3])
        .minor(10)
        .into_iter()
        .filter(|v| 0.0 < *v && *v < 12.0)
        .collect();
    assert_eq!(minor, [2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
}

#[test]
fn symlog_minor_ticks_follow_a_descending_domain() {
    let minor = scale::symlog().domain([-10.0, -100.0]).minor(10);
    let expected: Vec<f64> = (2..10).map(|m| -10.0 * f64::from(m)).collect();
    assert_eq!(minor, expected);
}

#[test]
fn symlog_minor_ticks_subdivide_linear_majors_in_data_space() {
    let minor = scale::symlog().domain([0.0, 1.0]).minor(5);
    assert_eq!(minor.len(), 20);
    assert!(close(minor[0], 0.04));
    assert!(close(minor[4], 0.24));
}

#[test]
fn symlog_maps_and_inverts_as_d3() {
    let s = scale::symlog().domain([-10.0, 10.0]).range([0.0, 1.0]);
    assert!(close(s.apply(2.0), 0.729078454995663));
    assert!(close(
        s.clone().constant(2.0).apply(2.0),
        0.6934264036172708
    ));
    assert!(close(s.invert(0.8), 3.215369133091901));
}

#[test]
fn func_ticks_by_the_rule_it_was_given() {
    let data = scale::func(f64::sqrt, |y| y * y, TickRule::Data).domain([0.0, 100.0]);
    assert_eq!(data.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    let transformed = scale::func(f64::sqrt, |y| y * y, TickRule::Transformed).domain([0.0, 100.0]);
    assert_eq!(transformed.ticks(5), [0.0, 4.0, 16.0, 36.0, 64.0, 100.0]);
}

#[test]
fn func_maps_through_its_closures_and_prints_its_rule() {
    let s = scale::func(f64::sqrt, |y| y * y, TickRule::Data)
        .domain([0.0, 100.0])
        .range([0.0, 10.0]);
    assert_eq!(s.apply(25.0), 5.0);
    assert_eq!(s.invert(5.0), 25.0);
    assert!(format!("{s:?}").contains("Func { rule: Data, .. }"));
}
