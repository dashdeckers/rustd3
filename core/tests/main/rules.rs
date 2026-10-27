//! Phase 5's continuous scales: tick rules and the `ticks::` blocks, the
//! symlog, logit, asinh and func transforms, `piecewise` and
//! `AnyContinuous`, against d3-scale 4.0.2, matplotlib 3.11.2's locators
//! and the fscale prototype.

use rustd3_core::scale::{self, TickRule};
use rustd3_core::ticks;

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
