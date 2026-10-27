//! `scale::Ticks` for continuous scales: tick sets, positions, batch labels
//! under `NEUTRAL` and `EN_US`, minor ticks and extents, against d3-scale
//! 4.0.2 where d3 has the value.

use rustd3_core::format::InvalidFormat;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, AnyContinuous, TickRule, Ticks};
use rustd3_testkit::obs::{ToObs, Tolerance};

/// The labels of `scale`'s tick set at `count` under `locale`.
fn labels<S: Ticks>(scale: &S, count: usize, locale: &Locale) -> Vec<String> {
    scale.labels(&scale.tick_set(count), locale)
}

/// Tolerant equality of two lists of numbers.
fn close(actual: &[f64], expected: &[f64]) -> bool {
    actual
        .to_obs()
        .approx_eq(&expected.to_obs(), &Tolerance::default())
}

/// The labels that are not blank, in order.
fn shown(labels: &[String]) -> Vec<&str> {
    labels
        .iter()
        .map(String::as_str)
        .filter(|l| !l.is_empty())
        .collect()
}

#[test]
fn labels_to_eight_thousand_group_under_en_us_only() {
    let x = scale::linear().domain([0.0, 8000.0]);
    assert_eq!(
        labels(&x, 5, &Locale::EN_US),
        ["0", "2,000", "4,000", "6,000", "8,000"]
    );
    assert_eq!(
        labels(&x, 5, &Locale::NEUTRAL),
        ["0", "2000", "4000", "6000", "8000"]
    );
}

#[test]
fn labels_to_twelve_thousand_group_under_both_locales() {
    let x = scale::linear().domain([0.0, 12000.0]);
    assert_eq!(
        labels(&x, 5, &Locale::EN_US),
        ["0", "2,000", "4,000", "6,000", "8,000", "10,000", "12,000"]
    );
    assert_eq!(
        labels(&x, 5, &Locale::NEUTRAL),
        [
            "0",
            "2\u{202f}000",
            "4\u{202f}000",
            "6\u{202f}000",
            "8\u{202f}000",
            "10\u{202f}000",
            "12\u{202f}000"
        ]
    );
}

#[test]
fn labels_around_zero_carry_the_minus_sign_and_the_batch_grouping() {
    let x = scale::linear().domain([-1e6, 1e6]);
    assert_eq!(
        labels(&x, 5, &Locale::EN_US),
        [
            "\u{2212}1,000,000",
            "\u{2212}500,000",
            "0",
            "500,000",
            "1,000,000"
        ]
    );
    assert_eq!(
        labels(&x, 5, &Locale::NEUTRAL),
        [
            "\u{2212}1\u{202f}000\u{202f}000",
            "\u{2212}500\u{202f}000",
            "0",
            "500\u{202f}000",
            "1\u{202f}000\u{202f}000"
        ]
    );
}

#[test]
fn the_default_scale_labels_tenths() {
    let tenths: Vec<String> = (0..=10)
        .map(|k| format!("{:.1}", f64::from(k) / 10.0))
        .collect();
    assert_eq!(labels(&scale::linear(), 10, &Locale::EN_US), tenths);
}

#[test]
fn a_nice_domain_labels_its_round_ticks() {
    let y = scale::linear()
        .domain([0.13, 9.71])
        .range([300.0, 0.0])
        .nice(5);
    assert_eq!(y.get_domain(), [0.0, 10.0]);
    assert_eq!(
        labels(&y, 5, &Locale::NEUTRAL),
        ["0", "2", "4", "6", "8", "10"]
    );
}

#[test]
fn a_tick_set_holds_values_step_and_count() {
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    let set = x.tick_set(8);
    let tens: Vec<f64> = (0..=10).map(|k| f64::from(k) * 10.0).collect();
    assert_eq!(set.values(), tens);
    assert_eq!(*set.step(), 10.0);
    assert_eq!(set.count(), 8);
    let positions: Vec<f64> = set
        .values()
        .iter()
        .map(|v| {
            x.position(v, 0.5)
                .expect("a continuous scale places every value")
        })
        .collect();
    let pixels: Vec<f64> = (0..=10).map(|k| f64::from(k) * 96.0).collect();
    assert!(close(&positions, &pixels), "{positions:?}");
    assert_eq!(x.extent(), [0.0, 960.0]);
}

#[test]
fn log_labels_keep_one_and_two_per_decade_at_count_ten() {
    let x = scale::log().domain([1.0, 1e4]);
    let set = x.tick_set(10);
    assert_eq!(set.values().len(), 37);
    let all = x.labels(&set, &Locale::EN_US);
    assert_eq!(
        shown(&all),
        ["1", "2", "10", "20", "100", "200", "1k", "2k", "10k"]
    );
    assert_eq!(all.iter().filter(|l| l.is_empty()).count(), 28);
}

#[test]
fn log_labels_keep_the_powers_at_count_five() {
    let x = scale::log().domain([1.0, 1e4]);
    let set = x.tick_set(5);
    assert_eq!(set.values(), x.tick_set(10).values());
    assert_eq!(
        shown(&x.labels(&set, &Locale::EN_US)),
        ["1", "10", "100", "1k", "10k"]
    );
}

#[test]
fn log_labels_over_two_decades_blank_six_to_nine() {
    let x = scale::log().domain([1.0, 100.0]);
    assert_eq!(
        labels(&x, 10, &Locale::EN_US),
        [
            "1", "2", "3", "4", "5", "", "", "", "", "10", "20", "30", "40", "50", "", "", "", "",
            "100"
        ]
    );
}

#[test]
fn linear_minor_ticks_subdivide_each_step_by_five() {
    let x = scale::linear().domain([0.0, 10.0]);
    let set = x.tick_set(5);
    assert_eq!(set.values(), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    let minor = Ticks::minor(&x, &set);
    let expected: Vec<f64> = (0..50)
        .filter(|k| k % 10 != 0 && k % 2 == 0)
        .map(|k| f64::from(k) / 5.0)
        .collect();
    assert_eq!(minor.len(), 20);
    assert!(close(&minor, &expected), "{minor:?}");
    assert!(close(&minor[..5], &[0.4, 0.8, 1.2, 1.6, 2.4]));
}

#[test]
fn linear_minor_ticks_reach_past_the_outer_majors_to_the_domain_ends() {
    let x = scale::linear().domain([0.5, 9.5]);
    let set = x.tick_set(5);
    assert_eq!(set.values(), [2.0, 4.0, 6.0, 8.0]);
    let minor = Ticks::minor(&x, &set);
    assert!(close(&minor[..3], &[0.8, 1.2, 1.6]), "{minor:?}");
    assert!(
        close(&minor[minor.len() - 3..], &[8.4, 8.8, 9.2]),
        "{minor:?}"
    );
    assert_eq!(minor.len(), 3 + 12 + 3);
}

#[test]
fn log_minor_ticks_fill_every_decade_inside_the_domain() {
    let x = scale::log().domain([3.0, 3e5]);
    let set = x.tick_set(4);
    assert_eq!(set.values(), [10.0, 100.0, 1e3, 1e4, 1e5]);
    let minor = Ticks::minor(&x, &set);
    let mut expected: Vec<f64> = (3..10).map(f64::from).collect();
    for k in 1..5 {
        expected.extend((2..10).map(|m| f64::from(m) * 10f64.powi(k)));
    }
    expected.extend([2e5, 3e5]);
    assert_eq!(minor, expected);
}

#[test]
fn log_minor_ticks_are_none_when_every_multiple_is_a_major() {
    let x = scale::log().domain([1.0, 1e4]);
    assert!(Ticks::minor(&x, &x.tick_set(10)).is_empty());
}

#[test]
fn symlog_minor_ticks_keep_their_decades_up_to_the_domain_end() {
    let x = scale::symlog().domain([0.0, 5000.0]);
    let set = x.tick_set(10);
    assert_eq!(set.values(), [0.0, 1.0, 10.0, 100.0, 1000.0]);
    let mut expected = Vec::new();
    for k in 0..3 {
        expected.extend((2..10).map(|m| f64::from(m) * 10f64.powi(k)));
    }
    expected.extend([2000.0, 3000.0, 4000.0, 5000.0]);
    assert_eq!(Ticks::minor(&x, &set), expected);
}

#[test]
fn fewer_than_two_majors_have_no_minor_ticks() {
    let x = scale::linear().domain([0.0, 10.0]);
    let one = x.tick_set(5).with_values(vec![5.0]);
    assert!(Ticks::minor(&x, &one).is_empty());
}

#[test]
fn symlog_ticks_and_labels_follow_its_decade_rule() {
    let x = scale::symlog().domain([0.0, 1e6]);
    let set = x.tick_set(10);
    assert_eq!(set.values(), [0.0, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]);
    assert_eq!(*set.step(), 1e5);
    assert_eq!(
        x.labels(&set, &Locale::EN_US),
        [
            "0",
            "1",
            "10",
            "100",
            "1,000",
            "10,000",
            "100,000",
            "1,000,000"
        ]
    );
    assert_eq!(
        x.labels(&set, &Locale::NEUTRAL)[4],
        "1\u{202f}000",
        "the axis groups every label once one has five digits"
    );
}

#[test]
fn a_custom_tick_rule_sets_the_values_and_their_spacing_sets_the_format() {
    let x = scale::linear()
        .domain([0.0, 1.2])
        .tick_rule(TickRule::custom(|[a, b], _| {
            (0..)
                .map(|i| a + 0.3 * f64::from(i))
                .take_while(|v| *v <= b + 1e-9)
                .collect()
        }));
    let set = x.tick_set(5);
    assert!(close(set.values(), &[0.0, 0.3, 0.6, 0.9, 1.2]));
    assert_eq!(*set.step(), 0.2);
    assert_eq!(
        x.labels(&set, &Locale::EN_US),
        ["0.0", "0.3", "0.6", "0.9", "1.2"]
    );
}

#[test]
fn chosen_values_keep_the_step_and_count_of_the_set() {
    let x = scale::linear();
    let set = x.tick_set(10).with_values(vec![0.25, 0.5]);
    assert_eq!(*set.step(), 0.1);
    assert_eq!(set.count(), 10);
    assert_eq!(x.labels(&set, &Locale::EN_US), ["0.3", "0.5"]);
}

#[test]
fn piecewise_scales_tick_on_their_end_stops() {
    let x = scale::piecewise([0.0, 50.0, 100.0], [0.0, 400.0, 500.0]);
    let set = x.tick_set(5);
    assert_eq!(set.values(), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    assert_eq!(*set.step(), 20.0);
    assert_eq!(x.position(&20.0, 0.5), Some(160.0));
    assert_eq!(x.position(&60.0, 0.5), Some(420.0));
    assert_eq!(x.extent(), [0.0, 500.0]);
    assert_eq!(
        labels(&x, 5, &Locale::NEUTRAL),
        ["0", "20", "40", "60", "80", "100"]
    );
}

#[test]
fn a_runtime_scale_ticks_and_labels_as_its_transform() {
    let log = scale::log().domain([1.0, 100.0]).range([0.0, 300.0]);
    let any: AnyContinuous = log.clone().into();
    assert_eq!(any.tick_set(10), log.tick_set(10));
    assert_eq!(
        labels(&any, 10, &Locale::NEUTRAL),
        labels(&log, 10, &Locale::NEUTRAL)
    );
    assert_eq!(
        Ticks::minor(&any, &any.tick_set(3)),
        Ticks::minor(&log, &log.tick_set(3))
    );
    assert_eq!(any.extent(), [0.0, 300.0]);
}

#[test]
fn logit_and_asinh_label_with_their_own_formats() {
    let p = scale::logit().domain([0.001, 0.999]);
    assert_eq!(
        labels(&p, 10, &Locale::NEUTRAL),
        ["0.001", "0.01", "0.1", "0.5", "0.9", "0.99", "0.999"]
    );
    let a = scale::asinh().domain([-1000.0, 1000.0]);
    assert_eq!(
        labels(&a, 10, &Locale::EN_US),
        [
            "\u{2212}100",
            "\u{2212}10",
            "\u{2212}1",
            "0",
            "1",
            "10",
            "100"
        ]
    );
}

#[test]
fn a_position_off_the_domain_is_returned_for_the_axis_to_skip() {
    let x = scale::log().domain([1.0, 10.0]);
    let at = x
        .position(&-1.0, 0.5)
        .expect("continuous positions are Some");
    assert!(at.is_nan());
}

#[test]
fn rebinding_labels_keeps_the_specifier_and_the_si_prefix() -> Result<(), InvalidFormat> {
    let x = scale::linear().domain([0.0, 1e6]);
    let set = x.tick_set(5);
    let neutral = x.tick_format(5, Some("$s"))?;
    assert_eq!(neutral.format(500_000.0), "0.5M");
    assert_eq!(
        neutral.labels(&set, &Locale::EN_US),
        ["$0.0M", "$0.2M", "$0.4M", "$0.6M", "$0.8M", "$1.0M"]
    );
    assert_eq!(neutral.labels(&set, &Locale::NEUTRAL)[5], "1.0M");
    Ok(())
}

#[test]
fn symlog_decades_label_every_tick_in_full() {
    let x = scale::symlog().constant(0.05).domain([0.0, 500.0]);
    let set = x.tick_set(10);
    assert_eq!(set.values(), [0.0, 0.05, 0.5, 5.0, 50.0, 500.0]);
    assert_eq!(
        x.labels(&set, &Locale::NEUTRAL),
        ["0", "0.05", "0.5", "5", "50", "500"]
    );
    let tiny = scale::symlog().constant(1e-6).domain([0.0, 1e-3]);
    assert_eq!(
        labels(&tiny, 10, &Locale::NEUTRAL),
        ["0", "0.000001", "0.00001", "0.0001", "0.001"]
    );
}

#[test]
fn symlog_linear_ticks_keep_d3s_linear_labels() -> Result<(), InvalidFormat> {
    let x = scale::symlog().domain([-3.0, 3.0]);
    let all = labels(&x, 10, &Locale::NEUTRAL);
    assert_eq!(all.len(), 13);
    assert_eq!(
        (all[0].as_str(), all[1].as_str()),
        ("\u{2212}3.0", "\u{2212}2.5")
    );
    assert_eq!(x.tick_format(10, None)?.format(0.5), "0.5");
    Ok(())
}

#[test]
fn a_transformed_rule_labels_to_the_places_of_the_smallest_gap() -> Result<(), InvalidFormat> {
    let x = scale::sqrt()
        .domain([0.0, 1.0])
        .tick_rule(TickRule::Transformed);
    let expected = ["0.00", "0.04", "0.16", "0.36", "0.64", "1.00"];
    assert_eq!(labels(&x, 5, &Locale::NEUTRAL), expected);
    assert_eq!(x.tick_format(5, None)?.format_all(&x.ticks(5)), expected);
    assert_eq!(x.tick_format(5, Some(".1f"))?.format(0.04), "0.0");
    Ok(())
}

#[test]
fn a_func_scale_labels_to_the_places_of_its_rules_smallest_gap() -> Result<(), InvalidFormat> {
    let x = scale::func(f64::sqrt, |y| y * y, TickRule::Transformed).domain([0.0, 1.0]);
    assert!(close(&x.ticks(5), &[0.0, 0.04, 0.16, 0.36, 0.64, 1.0]));
    let expected = ["0.00", "0.04", "0.16", "0.36", "0.64", "1.00"];
    assert_eq!(labels(&x, 5, &Locale::NEUTRAL), expected);
    assert_eq!(x.tick_format(5, None)?.format_all(&x.ticks(5)), expected);
    assert_eq!(x.tick_format(5, Some(".1f"))?.format(0.04), "0.0");
    Ok(())
}

#[test]
fn a_func_scale_under_the_data_rule_keeps_d3s_linear_labels() {
    for domain in [[0.0, 1.0], [-3.0, 1200.0], [1e9, 1e9 + 1.0]] {
        let x = scale::func(|x| x, |y| y, TickRule::Data).domain(domain);
        let linear = scale::linear().domain(domain);
        for count in [2, 5, 10] {
            assert_eq!(
                labels(&x, count, &Locale::EN_US),
                labels(&linear, count, &Locale::EN_US),
                "{domain:?} at {count}"
            );
        }
    }
}

#[test]
fn a_log_scale_under_the_data_rule_labels_every_tick() {
    let x = scale::log().domain([1.0, 1000.0]).tick_rule(TickRule::Data);
    assert_eq!(
        labels(&x, 10, &Locale::EN_US),
        [
            "100", "200", "300", "400", "500", "600", "700", "800", "900", "1,000"
        ]
    );
}

#[test]
fn a_gap_with_float_noise_adds_no_place() {
    assert_eq!(0.3 - 0.2, 0.09999999999999998);
    assert_eq!(1e9 + 0.3 - (1e9 + 0.2), 0.09999990463256836);
    for domain in [[0.0, 1.0], [1e9, 1e9 + 1.0]] {
        let x = scale::linear().domain(domain);
        let ruled = x.clone().tick_rule(TickRule::Data);
        assert_eq!(
            labels(&ruled, 10, &Locale::EN_US),
            labels(&x, 10, &Locale::EN_US),
            "{domain:?}"
        );
    }
    let near = scale::linear().domain([1e9, 1e9 + 1.0]);
    assert_eq!(labels(&near, 10, &Locale::EN_US)[1], "1,000,000,000.1");
}

#[test]
fn a_gap_below_the_ticks_resolution_keeps_the_transforms_labels() {
    let x = scale::linear()
        .domain([1.0, 2.0])
        .tick_rule(TickRule::custom(|_, _| vec![1.0, 1.0 + 1e-13, 2.0]));
    assert_eq!(labels(&x, 10, &Locale::EN_US), ["1.0", "1.0", "2.0"]);
}

#[test]
fn a_rule_with_fewer_than_two_ticks_keeps_the_transforms_labels() {
    let x = scale::log()
        .domain([1.0, 100.0])
        .tick_rule(TickRule::custom(|_, _| vec![50.0]));
    assert_eq!(labels(&x, 10, &Locale::EN_US), ["50"]);
    let blank = scale::log()
        .domain([1.0, 100.0])
        .tick_rule(TickRule::custom(|_, _| vec![70.0]));
    assert_eq!(labels(&blank, 10, &Locale::EN_US), [""]);
}

/// matplotlib 3.11.2's `LogLocator(subs='auto', numticks=5).tick_values`
/// over [1, 1e12], [1e-4, 1e4] and [1, 1e9] is empty: under a major stride
/// above one decade it draws no minor ticks.
#[test]
fn log_minor_ticks_leave_thinned_decades_empty() {
    let wide = scale::log().domain([1.0, 1e12]);
    let set = wide.tick_set(5);
    assert_eq!(
        set.values(),
        [1.0, 100.0, 1e4, 1e6, 1e8, 1e10, 1e12],
        "every other decade"
    );
    assert!(Ticks::minor(&wide, &set).is_empty());
    assert!(wide.minor(5).is_empty());
    let around_one = scale::log().domain([1e-4, 1e4]);
    let set = around_one.tick_set(5);
    assert_eq!(set.values(), [1e-4, 1e-2, 1.0, 100.0, 1e4]);
    assert!(Ticks::minor(&around_one, &set).is_empty());
    assert!(around_one.minor(5).is_empty());
}

#[test]
fn symlog_minor_ticks_leave_thinned_decades_empty() {
    let x = scale::symlog().domain([0.0, 1e9]);
    let set = x.tick_set(5);
    assert_eq!(set.values(), [0.0, 1.0, 100.0, 1e4, 1e6, 1e8]);
    assert!(Ticks::minor(&x, &set).is_empty());
    assert!(x.minor(5).is_empty());
}

#[test]
fn log_minor_ticks_fill_only_gaps_of_at_most_one_decade() {
    let x = scale::log()
        .domain([1.0, 1000.0])
        .tick_rule(TickRule::custom(|_, _| vec![1.0, 10.0, 1000.0]));
    let ones: Vec<f64> = (2..10).map(f64::from).collect();
    assert_eq!(x.minor(10), ones);
    assert_eq!(Ticks::minor(&x, &x.tick_set(10)), ones);
}
