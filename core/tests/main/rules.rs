//! Phase 5's continuous scales: tick rules and the `ticks::` blocks, the
//! symlog, logit, asinh and func transforms, `piecewise` and
//! `AnyContinuous`, against d3-scale 4.0.2, matplotlib 3.11.2's locators
//! and the fscale prototype.

use rustd3_core::format::{InvalidFormat, Locale};
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
fn symlog_minor_ticks_subdivide_symmetric_linear_majors_through_decades() {
    let s = scale::symlog().domain([-20.0, 20.0]);
    assert_eq!(s.ticks(4), [-20.0, -10.0, 0.0, 10.0, 20.0]);
    let minor = s.minor(4);
    assert_eq!(minor.len(), 16);
    assert!(close(minor[0], -18.0) && close(minor[15], 18.0));
    let t = scale::symlog().domain([-10.0, 20.0]);
    assert_eq!(t.ticks(3), [-10.0, 0.0, 10.0, 20.0]);
    assert_eq!(t.minor(3).len(), 12);
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

/// Whether `actual` and `expected` have equal lengths and close values.
fn all_close(actual: &[f64], expected: &[f64]) -> bool {
    actual
        .to_obs()
        .approx_eq(&expected.to_obs(), &Tolerance::default())
}

/// `logit()` ticks(10) over `domain`.
fn logit_ticks(domain: [f64; 2]) -> Vec<f64> {
    scale::logit().domain(domain).ticks(10)
}

/// `asinh()` ticks(10) over `domain`.
fn asinh_ticks(domain: [f64; 2]) -> Vec<f64> {
    scale::asinh().domain(domain).ticks(10)
}

#[test]
fn logit_ticks_at_one_half_and_decades_towards_zero_and_one() {
    let p = scale::continuous(scale::transform::Logit).domain([0.001, 0.999]);
    let expected = [0.001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999];
    assert!(all_close(&p.ticks(10), &expected));
    assert_eq!(p.ticks(10), logit_ticks([0.001, 0.999]));
}

#[test]
fn logit_labels_its_ticks_in_full() -> Result<(), InvalidFormat> {
    let p = scale::logit().domain([0.001, 0.999]);
    let f = p.tick_format(10, None)?;
    assert_eq!(
        f.format_all(&p.ticks(10)),
        ["0.001", "0.01", "0.1", "0.5", "0.9", "0.99", "0.999"]
    );
    Ok(())
}

#[test]
fn logit_is_symmetric_about_one_half() {
    let p = scale::logit().domain([0.001, 0.999]).range([0.0, 400.0]);
    assert!(close(p.apply(0.5), 200.0));
    assert!(close(p.invert(200.0), 0.5));
}

#[test]
fn logit_thins_its_ideal_ticks_past_the_count() {
    assert!(all_close(
        &logit_ticks([1e-6, 1.0 - 1e-6]),
        &[1e-6, 1e-4, 0.01, 0.5, 0.99, 0.9999, 0.999999]
    ));
}

#[test]
fn logit_ticks_keep_to_the_domain() {
    let mut low: Vec<f64> = (-9..=-1).map(|k| 10f64.powi(k)).collect();
    low.push(0.5);
    assert!(all_close(&logit_ticks([1e-9, 0.5]), &low));
    assert!(all_close(
        &logit_ticks([0.01, 0.99]),
        &[0.01, 0.1, 0.5, 0.9, 0.99]
    ));
    assert!(all_close(
        &logit_ticks([0.002, 0.998]),
        &[0.01, 0.1, 0.5, 0.9, 0.99]
    ));
}

#[test]
fn logit_ticks_linearly_when_fewer_than_two_ideal_ticks_fit() {
    let expected: Vec<f64> = (4..=14).map(|k| f64::from(k) * 0.05).collect();
    assert!(all_close(&logit_ticks([0.2, 0.7]), &expected));
}

#[test]
fn logit_nice_leaves_the_domain_alone() {
    let p = scale::logit().domain([0.013, 0.97]).nice(10);
    assert_eq!(p.get_domain(), [0.013, 0.97]);
}

#[test]
fn asinh_ticks_signed_powers_of_ten() {
    assert_eq!(
        asinh_ticks([-1000.0, 1000.0]),
        [-100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0]
    );
    assert_eq!(
        asinh_ticks([0.0, 1e6]),
        [0.0, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5]
    );
    assert_eq!(asinh_ticks([-3.0, 3.0]), [-1.0, -0.1, 0.0, 0.1, 1.0]);
    assert_eq!(asinh_ticks([0.0, 100.0]), [0.0, 0.1, 1.0, 10.0, 100.0]);
}

#[test]
fn asinh_ticks_every_other_decade_over_eighteen_orders() {
    // matplotlib on AVX-512 numpy keeps ±1e9: its arcsinh(1e9) is one ulp
    // above glibc's, which this port's arithmetic matches (D5.1).
    assert_eq!(
        asinh_ticks([-1e9, 1e9]),
        [-1e8, -1e7, -1e5, -1e3, -10.0, 0.0, 10.0, 1e3, 1e5, 1e7, 1e8]
    );
}

#[test]
fn asinh_ticks_lopsided_and_nearly_symmetric_domains() {
    assert_eq!(asinh_ticks([-50.0, 5.0]), [-10.0, -1.0, -0.1, 0.0, 1.0]);
    assert_eq!(asinh_ticks([1.0, 1e4]), [1.0, 10.0, 100.0, 1000.0]);
    assert_eq!(asinh_ticks([-50.0, 45.0]), [-10.0, -1.0, 0.0, 1.0, 10.0]);
}

#[test]
fn asinh_ticks_follow_the_linear_width() {
    let a = scale::asinh().linear_width(10.0).domain([-1000.0, 1000.0]);
    assert_eq!(a.ticks(10), [-100.0, -10.0, 0.0, 10.0, 100.0]);
}

#[test]
fn asinh_ticks_follow_a_descending_domain() {
    assert_eq!(asinh_ticks([1e4, 1.0]), [1000.0, 100.0, 10.0, 1.0]);
}

#[test]
fn asinh_with_the_transformed_rule_gives_uneven_ticks() {
    let a = scale::asinh()
        .domain([-1000.0, 1000.0])
        .tick_rule(TickRule::Transformed);
    let rounded: Vec<f64> = a
        .ticks(10)
        .iter()
        .map(|v| (v * 100.0).round() / 100.0)
        .collect();
    assert_eq!(rounded, [-201.71, -27.29, -3.63, 0.0, 3.63, 27.29, 201.71]);
}

#[test]
fn asinh_labels_group_thousands() -> Result<(), InvalidFormat> {
    let a = scale::asinh().domain([0.0, 1e6]);
    let f = a.tick_format_in(&Locale::EN_US, 10, None)?;
    assert_eq!(f.format(1e5), "100,000");
    assert_eq!(f.format(0.1), "0.1");
    Ok(())
}

#[test]
fn piecewise_maps_descending_stops_as_d3() {
    let s = scale::piecewise([4.0, 2.0, 1.0], [1.0, 2.0, 4.0]);
    assert_eq!(s.apply(1.5), 3.0);
    assert_eq!(s.invert(1.5), 3.0);
}

#[test]
fn piecewise_ticks_and_maps_by_segment() {
    let s = scale::piecewise([-10.0, 0.0, 100.0], [0.0, 50.0, 100.0]);
    assert_eq!(s.apply(-5.0), 25.0);
    assert_eq!(s.apply(50.0), 75.0);
    assert_eq!(s.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
}

#[test]
fn piecewise_nice_moves_only_the_end_stops() {
    let s = scale::piecewise([-10.3, 0.0, 99.1], [0.0, 1.0, 2.0]).nice(10);
    assert_eq!(s.get_domain(), [-20.0, 0.0, 100.0]);
}

#[test]
fn piecewise_keeps_its_stop_count_through_builders() {
    let s = scale::piecewise([0.0, 1.0, 2.0, 3.0], [0.0, 10.0, 20.0, 40.0])
        .domain([0.0, 1.0, 2.0, 4.0])
        .range([0.0f32, 1.0, 2.0, 3.0]);
    assert_eq!(s.apply(3.0), 2.5);
    assert_eq!(s.get_range(), &[0.0, 1.0, 2.0, 3.0]);
}

#[test]
fn any_continuous_forwards_to_the_log_scale_it_holds() -> Result<(), InvalidFormat> {
    let log = scale::log().domain([1.0, 1e3]).range([0.0, 300.0]);
    let any = scale::AnyContinuous::from(log.clone());
    for x in [1.0, 2.0, 31.6, 999.0] {
        assert_eq!(any.apply(x), log.apply(x));
    }
    assert_eq!(any.invert(150.0), log.invert(150.0));
    assert_eq!(any.ticks(10), log.ticks(10));
    assert_eq!(any.minor(3), log.minor(3));
    let (a, l) = (any.tick_format(10, None)?, log.tick_format(10, None)?);
    assert_eq!(a.format_all(&any.ticks(10)), l.format_all(&log.ticks(10)));
    assert_eq!(any.nice(10).get_domain(), log.nice(10).get_domain());
    Ok(())
}

#[test]
fn any_continuous_takes_every_built_in_scale() {
    let domain = [0.1, 0.9];
    let scales: [scale::AnyContinuous; 7] = [
        scale::linear().domain(domain).into(),
        scale::pow().exponent(2.0).domain(domain).into(),
        scale::log().domain(domain).into(),
        scale::symlog().constant(0.5).domain(domain).into(),
        scale::logit().domain(domain).into(),
        scale::asinh().linear_width(2.0).domain(domain).into(),
        scale::func(f64::sqrt, |y| y * y, TickRule::Data)
            .domain(domain)
            .into(),
    ];
    for s in &scales {
        assert!(close(s.apply(0.1), 0.0));
        assert!(close(s.apply(0.9), 1.0));
        assert!(close(
            s.invert(0.5),
            s.clone().range([0.0, 2.0]).invert(1.0)
        ));
        assert!(!s.ticks(5).is_empty());
    }
}

#[test]
fn any_continuous_keeps_a_negative_log_domain() {
    let any: scale::AnyContinuous = scale::log().domain([-1e3, -1.0]).into();
    let moved = any.domain([-100.0, -10.0]);
    assert_eq!(
        moved.ticks(10),
        scale::log().domain([-100.0, -10.0]).ticks(10)
    );
    assert!(close(moved.apply(-31.622776601683793), 0.5));
}

#[test]
fn symlog_logit_and_asinh_scales_have_aliases() {
    let s: scale::Symlog = scale::symlog();
    let p: scale::Logit = scale::logit().domain([0.1, 0.9]);
    let a: scale::Asinh<f32> = scale::asinh().range([0.0f32, 1.0]);
    assert_eq!(s.apply(1.0), 1.0);
    assert_eq!(p.get_domain(), [0.1, 0.9]);
    assert_eq!(a.apply(1.0), 1.0);
}
