//! Continuous scales and `scale::tick_format` against d3-scale 4.0.2, and
//! the neutral locale's labels where they differ from d3's `EN_US`.

use rustd3_core::format::{InvalidFormat, Locale};
use rustd3_core::scale;

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
    assert_eq!(error, Err(InvalidFormat("10..f".to_owned())));
}
