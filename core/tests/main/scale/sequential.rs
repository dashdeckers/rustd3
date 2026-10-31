//! `scale::Sequential` against d3-scale 4.0.2's scaleSequential and its
//! log, pow, sqrt and symlog variants.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks, transform};
use rustd3_core::{interpolate, scheme};

#[test]
fn a_scheme_maps_the_domain_through_0_to_1() {
    let blues = scale::sequential([0.0, 50.0], scheme::blues);
    assert_eq!(blues.apply(25.0).to_string(), "rgb(109, 174, 213)");
    let viridis = scale::sequential([0.0, 80.0], scheme::viridis);
    assert_eq!(viridis.apply(20.0).format_hex(), "#3b528b");
    assert_eq!(
        viridis.get_range().map(Rgb::format_hex),
        ["#440154", "#fde725"]
    );
}

#[test]
fn values_outside_the_domain_extrapolate_unless_clamped() {
    let s = scale::sequential([0.0, 1.0], |t| t);
    assert_eq!(s.apply(2.0), 2.0);
    assert_eq!(s.clone().clamp(true).apply(2.0), 1.0);
    assert_eq!(s.clone().clamp(true).apply(-0.5), 0.0);
    assert!(s.apply(f64::NAN).is_nan());
    assert_eq!(s.unknown(-1.0).apply(f64::NAN), -1.0);
}

#[test]
fn a_flat_domain_maps_to_the_middle_and_a_nan_end_to_nan() {
    let flat = scale::sequential([2.0, 2.0], |t| t);
    assert_eq!([flat.apply(-1.2), flat.apply(2.4)], [0.5, 0.5]);
    let undefined = scale::sequential([f64::NAN, 2.0], |t| t);
    assert!(undefined.apply(0.6).is_nan());
}

#[test]
fn range_sets_a_lerp_interpolator() {
    let s = scale::sequential([0.0, 1.0], |t| t).range([0.0, 10.0]);
    assert_eq!(s.apply(0.5), 5.0);
    assert_eq!(s.get_range(), [0.0, 10.0]);
    let white = Rgb::hex(0xffffff);
    let steel = Rgb::hex(0x4682b4);
    let colors = scale::sequential([0.0, 1.0], move |_| white).range([white, steel]);
    assert_eq!(colors.apply(0.5).to_string(), "rgb(163, 193, 218)");
}

#[test]
fn transforms_give_d3s_log_pow_sqrt_and_symlog_variants() {
    let log =
        scale::sequential([1.0, 1000.0], scheme::viridis).transform(transform::Log::new(10.0));
    assert_eq!(log.apply(10.0).format_hex(), "#31688e");
    let reflected = scale::sequential([-1000.0, -1.0], |t| t).transform(transform::Log::new(10.0));
    assert_eq!(reflected.apply(-10.0), 0.6666666666666666);
    let sqrt = scale::sequential([0.0, 100.0], |t| t).transform(transform::Pow::new(0.5));
    assert_eq!(sqrt.apply(25.0), 0.5);
    assert_eq!(sqrt.exponent(2.0).domain([0.0, 10.0]).apply(5.0), 0.25);
    let symlog = scale::sequential([0.0, 100.0], |t| t).transform(transform::Symlog::new(1.0));
    assert_eq!(symlog.apply(10.0), 0.5195737064824407);
    let binary = scale::sequential([1.0, 100.0], |t| t)
        .transform(transform::Log::new(10.0))
        .base(2.0);
    assert_eq!(binary.ticks(5), [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]);
    assert_eq!(symlog.constant(10.0).get_domain(), [0.0, 100.0]);
}

#[test]
fn ticks_nice_and_formats_act_on_the_domain() -> Result<(), rustd3_core::format::InvalidFormat> {
    let s = scale::sequential([0.0, 50.0], |t| t);
    assert_eq!(s.ticks(5), [0.0, 10.0, 20.0, 30.0, 40.0, 50.0]);
    assert_eq!(
        scale::sequential([0.13, 9.71], |t| t).nice(5).get_domain(),
        [0.0, 10.0]
    );
    let wide = scale::sequential([0.0, 1e4], |t| t);
    let f = wide.tick_format_in(&rustd3_core::format::Locale::EN_US, 5, None)?;
    assert_eq!(
        f.format_all(&wide.ticks(5)),
        ["0", "2,000", "4,000", "6,000", "8,000", "10,000"]
    );
    assert_eq!(wide.tick_format(5, Some(".1f"))?.format(2000.0), "2000.0");
    Ok(())
}

#[test]
fn ticks_position_in_0_to_1_for_a_colorbar() {
    let s = scale::sequential([0.0, 50.0], scheme::blues);
    assert_eq!(s.position(&25.0, 0.0), Some(0.5));
    assert_eq!(s.position(&f64::NAN, 0.0), None);
    assert_eq!(s.position(&75.0, 0.0), Some(1.5));
    assert_eq!(s.clone().clamp(true).position(&75.0, 0.0), Some(1.5));
    assert_eq!(s.extent(), [0.0, 1.0]);
    let set = s.tick_set(5);
    assert_eq!(
        (set.get_values(), *set.get_step(), set.get_count()),
        (&[0.0, 10.0, 20.0, 30.0, 40.0, 50.0][..], 10.0, 5)
    );
}

#[test]
fn log_labels_blank_as_d3s_and_minors_fill_decades() {
    let log =
        scale::sequential([1.0, 1000.0], scheme::viridis).transform(transform::Log::new(10.0));
    let set = log.tick_set(10);
    let labels = log.labels(&set, &Locale::EN_US);
    let shown: Vec<&str> = labels
        .iter()
        .map(String::as_str)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        shown,
        ["1", "2", "3", "10", "20", "30", "100", "200", "300", "1k"]
    );
    assert_eq!(labels.len(), 28);
    let linear = scale::sequential([0.0, 10.0], |t| t);
    assert_eq!(linear.minor(&linear.tick_set(5))[..4], [0.4, 0.8, 1.2, 1.6]);
}

#[test]
fn a_piecewise_interpolator_feeds_a_sequential_scale() {
    let [red, green, blue] = [Rgb::hex(0xff0000), Rgb::hex(0x008000), Rgb::hex(0x0000ff)];
    let spread = interpolate::piecewise(&[red, green, blue], |a, b| interpolate::rgb(*a, *b));
    let s = scale::sequential([0.0, 100.0], spread);
    assert_eq!(s.apply(75.0).to_string(), "rgb(0, 64, 128)");
}

#[test]
fn clones_share_the_interpolator_and_debug_prints_the_rest() {
    let s = scale::sequential([0.0, 2.0], |t| t * 2.0).clamp(true);
    let copy = s.clone().domain([0.0, 4.0]);
    assert_eq!((s.apply(1.0), copy.apply(1.0)), (1.0, 0.5));
    let shown = format!("{s:?}");
    assert!(
        shown.contains("domain: [0.0, 2.0]") && shown.contains("clamp: true"),
        "{shown}"
    );
}
