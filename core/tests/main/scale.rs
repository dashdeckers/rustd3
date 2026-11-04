//! Continuous scales and `scale::tick_format` against d3-scale 4.0.2, and
//! the neutral locale's labels where they differ from d3's `EN_US`.

mod diverging;
mod identity;
mod sequential;
mod ticks;

use rustd3_core::format::{InvalidFormat, Locale};
use rustd3_core::scale;
use rustd3_testkit::obs::{ToObs, Tolerance};

/// `values` formatted by `scale::tick_format` under `EN_US`.
fn labels(domain: [f64; 2], count: usize, specifier: Option<&str>, values: &[f64]) -> Vec<String> {
    let f =
        scale::tick_format(&Locale::EN_US, domain, count, specifier).expect("a valid specifier");
    values.iter().map(|&v| f.format(v)).collect()
}

#[test]
fn linear_maps_and_inverts() {
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    assert_eq!(x.apply(25.0), 240.0);
    assert_eq!(x.invert(480.0), 50.0);
}

#[test]
fn nice_extends_the_domain_to_the_ticks_and_labels_follow_them() -> Result<(), InvalidFormat> {
    let y = scale::linear()
        .domain([0.13, 9.71])
        .range([300.0, 0.0])
        .nice(5);
    assert_eq!(y.get_domain(), [0.0, 10.0]);
    let f = y.tick_format(5, None)?;
    assert_eq!(f.format_all(&y.ticks(5)), ["0", "2", "4", "6", "8", "10"]);
    Ok(())
}

#[test]
fn nice_keeps_the_direction_of_the_domain() {
    let nice = |domain| scale::linear().domain(domain).nice(10).get_domain();
    assert_eq!(nice([0.13, 9.71]), [0.0, 10.0]);
    assert_eq!(nice([9.71, 0.13]), [10.0, 0.0]);
    assert_eq!(nice([0.0, 0.96]), [0.0, 1.0]);
    assert_eq!(nice([1.1, 10.9]), [1.0, 11.0]);
}

#[test]
fn nice_leaves_an_empty_domain_alone() {
    let nice = |domain| scale::linear().domain(domain).nice(10).get_domain();
    assert_eq!(nice([0.0, 0.0]), [0.0, 0.0]);
    assert_eq!(nice([0.5, 0.5]), [0.5, 0.5]);
}

#[test]
fn default_linear_ticks_label_with_one_decimal() -> Result<(), InvalidFormat> {
    let x = scale::linear();
    let f = x.tick_format_in(&Locale::EN_US, 10, None)?;
    let tenths: Vec<String> = (0..=10)
        .map(|k| format!("{:.1}", f64::from(k) / 10.0))
        .collect();
    assert_eq!(f.format_all(&x.ticks(10)), tenths);
    Ok(())
}

#[test]
fn a_percent_specifier_drops_two_places() -> Result<(), InvalidFormat> {
    let f = scale::linear().tick_format_in(&Locale::EN_US, 10, Some("+%"))?;
    assert_eq!(f.format(0.3), "+30%");
    Ok(())
}

#[test]
fn large_ticks_group_under_en_us_and_the_neutral_locale() -> Result<(), InvalidFormat> {
    let x = scale::linear().domain([0.0, 1e6]);
    let ticks = x.ticks(5);
    let en = x.tick_format_in(&Locale::EN_US, 5, None)?;
    assert_eq!(
        en.format_all(&ticks),
        ["0", "200,000", "400,000", "600,000", "800,000", "1,000,000"]
    );
    let neutral = x.tick_format(5, None)?;
    assert_eq!(
        neutral.format_all(&ticks),
        [
            "0",
            "200\u{202f}000",
            "400\u{202f}000",
            "600\u{202f}000",
            "800\u{202f}000",
            "1\u{202f}000\u{202f}000"
        ]
    );
    Ok(())
}

#[test]
fn si_specifiers_take_the_prefix_of_the_larger_end() -> Result<(), InvalidFormat> {
    let x = scale::linear().domain([0.0, 1e6]);
    assert_eq!(
        x.tick_format_in(&Locale::EN_US, 5, Some("s"))?
            .format(500_000.0),
        "0.5M"
    );
    assert_eq!(
        x.tick_format_in(&Locale::EN_US, 5, Some("$s"))?
            .format(500_000.0),
        "$0.5M"
    );
    Ok(())
}

#[test]
fn ticks_far_from_zero_keep_their_halves() -> Result<(), InvalidFormat> {
    let x = scale::linear().domain([1e9 + 1.0, 1e9 + 2.0]);
    assert_eq!(
        x.ticks(3),
        [1_000_000_001.0, 1_000_000_001.5, 1_000_000_002.0]
    );
    let f = x.tick_format_in(&Locale::EN_US, 3, None)?;
    assert_eq!(f.format(1e9 + 1.5), "1,000,000,001.5");
    Ok(())
}

#[test]
fn clamp_keeps_apply_and_invert_within_the_domain() {
    let x = scale::linear()
        .domain([0.0, 10.0])
        .range([0.0, 100.0])
        .clamp(true);
    assert_eq!(x.apply(20.0), 100.0);
    assert_eq!(x.invert(200.0), 10.0);
}

#[test]
fn an_empty_domain_or_range_maps_to_the_middle() {
    let x = scale::linear().domain([0.0, 0.0]).range([1.0, 2.0]);
    assert_eq!(x.apply(0.0), 1.5);
    let y = scale::linear().domain([1.0, 2.0]).range([0.0, 0.0]);
    assert_eq!(y.invert(0.0), 1.5);
}

#[test]
fn range_round_rounds_the_output() {
    assert_eq!(scale::linear().range_round([0.0, 10.0]).apply(0.59), 6.0);
}

#[test]
fn range_round_keeps_the_unknown_value() {
    let x = scale::linear().unknown(-1.0).range_round([0.0, 100.0]);
    assert_eq!(x.apply(f64::NAN), -1.0);
    assert_eq!(x.apply(0.555), 56.0);
}

#[test]
fn with_range_keeps_the_interpolator_clamp_and_unknown_value() {
    let x = scale::linear()
        .range_round([0.0, 10.0])
        .with_range([0.0, 10.0]);
    assert_eq!(x.apply(0.59), 6.0);
    let kept = scale::linear()
        .clamp(true)
        .unknown(-1.0)
        .with_range([0.0, 10.0]);
    assert_eq!(kept.apply(2.0), 10.0);
    assert_eq!(kept.apply(f64::NAN), -1.0);
}

#[test]
fn admits_follows_the_transform_for_the_current_domain() {
    let log = scale::log();
    assert!(log.admits(1e-300) && !log.admits(0.0) && !log.admits(-5.0));
    let negative = scale::log().domain([-100.0, -1.0]);
    assert!(negative.admits(-5.0) && !negative.admits(5.0));
    let logit = scale::logit().domain([0.01, 0.99]);
    assert!(logit.admits(0.5) && !logit.admits(0.0) && !logit.admits(1.0));
    assert!(scale::sqrt().admits(-4.0));
    assert!(!scale::linear().admits(f64::NAN));
}

#[test]
fn admits_ignores_clamping() {
    let clamped = scale::log().domain([1.0, 10.0]).clamp(true);
    assert!(clamped.apply(-5.0).is_finite());
    assert!(!clamped.admits(-5.0));
    assert!(clamped.admits(1000.0));
}

#[test]
fn range_resets_the_interpolator_and_the_unknown_value() {
    let x = scale::linear()
        .unknown(-1.0)
        .range_round([0.0, 10.0])
        .range([0.0f64, 10.0]);
    assert_eq!(x.apply(0.25), 2.5);
    assert!(x.apply(f64::NAN).is_nan());
}

#[test]
fn nan_flows_through_unless_an_unknown_value_is_set() {
    let x = scale::linear().clamp(true);
    assert!(x.apply(f64::NAN).is_nan());
    assert_eq!(x.unknown(-1.0).apply(f64::NAN), -1.0);
}

#[test]
fn a_new_range_type_resets_the_interpolator() {
    let x = scale::linear()
        .range_round([0.0, 10.0])
        .range([0.0f32, 10.0]);
    assert_eq!(x.apply(0.59), 5.9);
}

#[test]
fn minor_ticks_subdivide_the_majors_by_five() {
    let x = scale::linear().domain([0.0, 2.0]);
    assert_eq!(x.ticks(2), [0.0, 1.0, 2.0]);
    assert_eq!(x.minor(2), [0.2, 0.4, 0.6, 0.8, 1.2, 1.4, 1.6, 1.8]);
}

#[test]
fn tick_format_sets_fixed_precision_from_the_step() {
    assert_eq!(labels([0.0, 1.0], 10, None, &[0.2]), ["0.2"]);
    assert_eq!(labels([0.0, 1.0], 20, None, &[0.2]), ["0.20"]);
    assert_eq!(labels([-100.0, 100.0], 10, None, &[-20.0]), ["\u{2212}20"]);
    assert_eq!(labels([0.19, 0.21], 10, Some("+%"), &[0.2]), ["+20.0%"]);
}

#[test]
fn tick_format_sets_significant_digits_from_the_step() {
    assert_eq!(labels([0.0, 9.0], 10, Some("e"), &[2.1]), ["2e+0"]);
    assert_eq!(labels([0.0, 100.0], 10, Some("r"), &[2.11e6]), ["2000000"]);
    assert_eq!(labels([0.0, 0.9], 10, Some("p"), &[0.21]), ["20%"]);
}

#[test]
fn tick_format_keeps_a_precision_the_specifier_sets() {
    assert_eq!(labels([0.0, 1.0], 10, Some(".3f"), &[0.2]), ["0.200"]);
}

#[test]
fn tick_format_rejects_a_malformed_specifier() {
    let error = scale::tick_format(&Locale::EN_US, [0.0, 1.0], 10, Some("10..f"));
    assert_eq!(
        error.map_err(|e| e.get_input().to_owned()),
        Err("10..f".to_owned())
    );
}

#[test]
fn pow_raises_the_domain_and_inverts_through_the_root() {
    let x = scale::pow()
        .exponent(2.0)
        .domain([0.0, 10.0])
        .range([0.0, 100.0]);
    assert_eq!(x.apply(5.0), 25.0);
    assert_eq!(x.invert(25.0), 5.0);
}

#[test]
fn pow_keeps_the_sign_and_sqrt_is_exponent_one_half() {
    assert_eq!(scale::pow().exponent(0.5).apply(-4.0), -2.0);
    let s = scale::sqrt().domain([0.0, 100.0]).range([0.0, 10.0]);
    assert_eq!(s.apply(25.0), 5.0);
    assert_eq!(scale::sqrt().apply(0.5), std::f64::consts::FRAC_1_SQRT_2);
}

#[test]
fn pow_ticks_and_nice_are_linear_in_data_space() {
    let x = scale::pow().exponent(2.0).domain([0.0, 10.0]);
    let units: Vec<f64> = (0..=10).map(f64::from).collect();
    assert_eq!(x.ticks(10), units);
    let nice = scale::pow().exponent(2.0).domain([0.13, 9.71]).nice(10);
    assert_eq!(nice.get_domain(), [0.0, 10.0]);
}

#[test]
fn a_new_exponent_keeps_the_domain_and_range() {
    let x = scale::pow()
        .domain([1.0, 2.0])
        .range([3.0, 4.0])
        .exponent(0.5);
    assert_eq!(x.get_domain(), [1.0, 2.0]);
    assert_eq!(x.get_range(), &[3.0, 4.0]);
    assert_eq!(x.apply(2.0), 4.0);
}

#[test]
fn a_clamped_pow_scale_inverts_within_the_domain() {
    let x = scale::pow().exponent(0.5).domain([1.0, 20.0]).clamp(true);
    assert_eq!(x.invert(0.0), 1.0);
    assert_eq!(x.invert(1.0), 20.0);
}

/// `count` copies of `""`.
fn blanks(count: usize) -> Vec<String> {
    vec![String::new(); count]
}

/// `labels` with `blanks(8)` after every one but the last.
fn decades(labels: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        out.push((*label).to_owned());
        if i + 1 < labels.len() {
            out.extend(blanks(8));
        }
    }
    out
}

#[test]
fn log_base_two_ticks_at_powers_of_two() -> Result<(), InvalidFormat> {
    let x = scale::log().base(2.0).domain([1.0, 1024.0]);
    let ticks = x.ticks(10);
    let powers: Vec<f64> = (0..=10).map(|k| 2f64.powi(k)).collect();
    assert_eq!(ticks, powers);
    let en = x
        .tick_format_in(&Locale::EN_US, 10, None)?
        .format_all(&ticks);
    assert_eq!(en.last().map(String::as_str), Some("1,024"));
    let neutral = x.tick_format(10, None)?.format_all(&ticks);
    assert_eq!(neutral.last().map(String::as_str), Some("1024"));
    Ok(())
}

#[test]
fn log_ticks_count_one_to_nine_per_decade_and_label_the_powers() -> Result<(), InvalidFormat> {
    let x = scale::log().domain([1.0, 1e6]);
    let ticks = x.ticks(10);
    assert_eq!(ticks.len(), 55);
    let f = x.tick_format_in(&Locale::EN_US, 10, None)?;
    assert_eq!(
        f.format_all(&ticks),
        decades(&["1", "10", "100", "1k", "10k", "100k", "1M"])
    );
    assert_eq!(x.ticks(5), [1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]);
    Ok(())
}

#[test]
fn log_labels_thin_to_about_the_count() -> Result<(), InvalidFormat> {
    let x = scale::log().domain([1.0, 100.0]);
    let ticks = x.ticks(10);
    let ten = x
        .tick_format_in(&Locale::EN_US, 10, None)?
        .format_all(&ticks);
    let mut expected: Vec<String> = Vec::new();
    for leading in [["1", "2", "3", "4", "5"], ["10", "20", "30", "40", "50"]] {
        expected.extend(leading.iter().map(|s| (*s).to_owned()));
        expected.extend(blanks(4));
    }
    expected.push("100".to_owned());
    assert_eq!(ten, expected);
    let five = x
        .tick_format_in(&Locale::EN_US, 5, None)?
        .format_all(&ticks);
    let mut expected: Vec<String> = Vec::new();
    for leading in [["1", "2"], ["10", "20"]] {
        expected.extend(leading.iter().map(|s| (*s).to_owned()));
        expected.extend(blanks(7));
    }
    expected.push("100".to_owned());
    assert_eq!(five, expected);
    Ok(())
}

#[test]
fn log_nice_extends_to_powers_of_the_base_on_either_side_of_zero() {
    let nice = |domain| scale::log().domain(domain).nice(10).get_domain();
    assert_eq!(nice([0.15, 68.0]), [0.1, 100.0]);
    assert_eq!(nice([-68.0, -0.15]), [-100.0, -0.1]);
    assert_eq!(nice([0.5, 0.5]), [0.1, 1.0]);
}

#[test]
fn log_maps_through_the_natural_logarithm() {
    let x = scale::log().domain([1.0, 10.0]).range([0.0, 1.0]);
    assert_eq!(x.apply(5.0), 0.6989700043360187);
    // JavaScript prints 3.16227766016838, one ulp above the nearest double: V8's
    // exp is not correctly rounded at ln(10)/2.
    let invert = x.invert(0.5);
    assert!(
        invert
            .to_obs()
            .approx_eq(&3.16227766016838.to_obs(), &Tolerance::default())
    );
}

#[test]
fn log_ticks_reflect_a_negative_domain() {
    let x = scale::log().domain([-100.0, -1.0]);
    let mut expected = vec![-100.0];
    expected.extend((1..=9).rev().map(|k| -10.0 * f64::from(k)));
    expected.extend((1..=9).rev().map(|k| -f64::from(k)));
    assert_eq!(x.ticks(10), expected);
}

#[test]
fn log_ticks_are_exact_below_one() {
    assert_eq!(
        scale::log().domain([0.5, 1.5]).ticks(10),
        [0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    );
    assert_eq!(
        scale::log().domain([0.15, 0.68]).ticks(10),
        [0.2, 0.3, 0.4, 0.5, 0.6]
    );
}

#[test]
fn log_base_e_ticks_at_powers_of_e() -> Result<(), InvalidFormat> {
    let x = scale::log().base(std::f64::consts::E).domain([1.0, 100.0]);
    let ticks = x.ticks(10);
    assert_eq!(
        ticks,
        [
            1.0,
            std::f64::consts::E,
            7.38905609893065,
            20.085536923187668,
            54.598150033144236
        ]
    );
    let f = x.tick_format_in(&Locale::EN_US, 10, None)?;
    assert_eq!(
        f.format_all(&ticks),
        [
            "1",
            "2.71828182846",
            "7.38905609893",
            "20.0855369232",
            "54.5981500331"
        ]
    );
    Ok(())
}

#[test]
fn log_labels_keep_a_given_specifier() -> Result<(), InvalidFormat> {
    let x = scale::log().domain([1.0, 1000.0]);
    let f = x.tick_format_in(&Locale::EN_US, 5, Some("+,"))?;
    assert_eq!(
        f.format_all(&x.ticks(10)),
        decades(&["+1", "+10", "+100", "+1,000"])
    );
    Ok(())
}

#[test]
fn log_ticks_over_many_decades_skip_decades() {
    assert_eq!(scale::log().domain([1.0, 1e100]).ticks(10).len(), 11);
    assert_eq!(scale::log().domain([1e-300, 1e300]).ticks(10).len(), 13);
}

#[test]
fn log_ticks_bail_out_where_the_log_of_an_end_is_not_finite() {
    assert!(scale::log().domain([0.0, 10.0]).ticks(10).is_empty());
    assert!(
        scale::log()
            .domain([1.0, f64::INFINITY])
            .ticks(10)
            .is_empty()
    );
    assert!(scale::log().base(1.0).ticks(10).is_empty());
    assert!(scale::log().base(0.0).ticks(10).is_empty());
    assert!(scale::log().base(-2.0).ticks(10).is_empty());
}

#[test]
fn a_log_domain_straddling_zero_gives_nan_no_ticks_and_no_nice() {
    let x = scale::log().domain([-1.0, 10.0]);
    assert!(x.apply(5.0).is_nan());
    assert!(x.ticks(10).is_empty());
    assert_eq!(x.nice(10).get_domain(), [-1.0, 10.0]);
}

#[test]
fn log_ticks_within_one_decade_keep_d3s_multiples_at_a_count_of_one() {
    let x = scale::log().domain([286.9252014, 329.4978332]);
    assert_eq!(x.ticks(1), [300.0]);
    assert_eq!(x.ticks(4), [290.0, 300.0, 310.0, 320.0]);
}

#[test]
fn log_minor_ticks_fill_the_decades_between_the_majors() {
    assert!(scale::log().domain([1.0, 1000.0]).minor(10).is_empty());
    let minor = scale::log().domain([1.0, 1e6]).minor(5);
    let expected: Vec<f64> = (0..=5)
        .flat_map(|k| (2..=9).map(move |m| f64::from(m) * 10f64.powi(k)))
        .collect();
    assert_eq!(minor.len(), 48);
    assert_eq!(minor, expected);
}

#[test]
fn log_minor_ticks_follow_the_order_of_a_negative_domain() {
    let minor = scale::log().domain([-1e3, -1.0]).minor(3);
    assert_eq!(minor.len(), 24);
    assert_eq!(minor[..2], [-900.0, -800.0]);
    assert_eq!(minor[23], -2.0);
}

#[test]
fn log_minor_ticks_of_a_base_beyond_two_to_the_53_end() {
    // Multiples past 2^53 do not move by one; the minor ticks stop there
    // instead of counting forever.
    let base = 9.999_999_999_997_266e299;
    let x = scale::log()
        .base(base)
        .domain([-base, -1.0])
        .tick_rule(scale::TickRule::Transformed);
    assert_eq!(x.ticks(3).len(), 4);
    let minor = scale::Ticks::minor(&x, &scale::Ticks::tick_set(&x, 3));
    assert!(
        minor.iter().all(|m| m.is_finite() && *m < -1.0),
        "{minor:?}"
    );
}

#[test]
fn log_minor_ticks_of_a_non_integer_base_of_three_or_more_subdivide() {
    let x = scale::log().base(std::f64::consts::PI).domain([1.0, 100.0]);
    assert_eq!(x.minor(10).len(), 16);
    let e = scale::log().base(std::f64::consts::E).domain([1.0, 100.0]);
    assert!(e.minor(10).is_empty(), "a base below 3, as matplotlib");
}
