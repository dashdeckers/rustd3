//! `scale::Diverging` against d3-scale 4.0.2's scaleDiverging and its log,
//! pow, sqrt and symlog variants.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks, transform};
use rustd3_core::scheme;

#[test]
fn each_half_of_the_domain_maps_onto_half_of_0_to_1() {
    let s = scale::diverging([-1.0, 0.0, 4.0], |t| t);
    assert_eq!(
        [s.apply(-0.5), s.apply(0.0), s.apply(2.0)],
        [0.25, 0.5, 0.75]
    );
    assert_eq!([s.apply(-2.0), s.apply(8.0)], [-0.5, 1.5]);
    assert_eq!(s.clone().clamp(true).apply(8.0), 1.0);
    assert!(s.apply(f64::NAN).is_nan());
    assert_eq!(s.unknown(-1.0).apply(f64::NAN), -1.0);
    let heat = scale::diverging([-1.0, 0.0, 4.0], scheme::rd_bu);
    assert_eq!(heat.apply(2.0).to_string(), "rgb(107, 172, 208)");
}

#[test]
fn flat_halves_map_to_the_middle_and_descending_domains_reverse() {
    let s = scale::diverging([2.0, 2.0, 3.0], |t| t);
    assert_eq!([s.apply(-1.2), s.apply(0.6), s.apply(2.4)], [0.5, 0.5, 0.7]);
    let s = s.domain([1.0, 2.0, 2.0]);
    assert_eq!(
        [s.apply(-1.0), s.apply(0.5), s.apply(2.4)],
        [-1.0, -0.25, 0.5]
    );
    let s = s.domain([2.0, 2.0, 2.0]);
    assert_eq!(s.apply(-1.0), 0.5);
    let down = scale::diverging([4.0, 2.0, 1.0], |t| t);
    assert_eq!(
        [down.apply(1.2), down.apply(2.0), down.apply(3.0)],
        [0.9, 0.5, 0.25]
    );
}

#[test]
fn range_spreads_three_values_by_piecewise_lerp() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t| t).range([1.0, 3.0, 10.0]);
    assert_eq!(s.get_range(), [1.0, 3.0, 10.0]);
    assert_eq!([s.apply(0.25), s.apply(0.75)], [2.0, 6.5]);
    let [red, white, blue] = [Rgb::hex(0xff0000), Rgb::hex(0xffffff), Rgb::hex(0x0000ff)];
    let colors = scale::diverging([-1.0, 0.0, 1.0], move |_| white).range([red, white, blue]);
    assert_eq!(colors.apply(-0.5).to_string(), "rgb(255, 128, 128)");
}

#[test]
fn transforms_give_d3s_log_pow_sqrt_and_symlog_variants() {
    let symlog =
        scale::diverging([-100.0, 0.0, 100.0], |t| t).transform(transform::Symlog::new(1.0));
    assert_eq!(symlog.apply(10.0), 0.7597868532412204);
    let log = scale::diverging([-1.0, -2.0, -3.0], |t| t).transform(transform::Log::new(10.0));
    assert_eq!(
        [log.apply(-1.2), log.apply(-2.0)],
        [0.1315172029168969, 0.5]
    );
    let sqrt = scale::diverging([-4.0, 0.0, 16.0], |t| t).transform(transform::Pow::new(0.5));
    assert_eq!(sqrt.apply(4.0), 0.75);
    let square = scale::diverging([-1.0, 0.0, 1.0], |t| t)
        .transform(transform::Pow::new(1.0))
        .exponent(2.0);
    assert_eq!(square.apply(0.5), 0.625);
    assert_eq!(log.base(2.0).get_domain(), [-1.0, -2.0, -3.0]);
    assert_eq!(symlog.constant(10.0).get_domain(), [-100.0, 0.0, 100.0]);
}

#[test]
fn ticks_and_nice_act_on_the_end_stops() {
    let s = scale::diverging([-1.0, 0.0, 4.0], |t| t);
    assert_eq!(s.ticks(5), [-1.0, 0.0, 1.0, 2.0, 3.0, 4.0]);
    assert_eq!(
        scale::diverging([-1.1, 0.0, 4.2], |t| t)
            .nice(10)
            .get_domain(),
        [-1.5, 0.0, 4.5]
    );
    assert_eq!(
        s.labels(&s.tick_set(5), &Locale::EN_US),
        ["−1", "0", "1", "2", "3", "4"]
    );
    assert_eq!(
        s.tick_format(5, Some("+.1f")).map(|f| f.format(2.0)),
        Ok("+2.0".to_owned())
    );
}

#[test]
fn ticks_position_the_middle_stop_at_one_half() {
    let s = scale::diverging([-1.0, 0.0, 4.0], scheme::rd_bu);
    assert_eq!(
        [0.0, 2.0, -1.0].map(|v| s.position(&v, 0.0)),
        [Some(0.5), Some(0.75), Some(0.0)]
    );
    assert_eq!(s.position(&f64::NAN, 0.0), None);
    assert_eq!(s.ends(), [0.0, 1.0]);
    assert_eq!(Ticks::minor(&s, &s.tick_set(5))[..2], [-0.8, -0.6]);
    let shown = format!("{s:?}");
    assert!(
        shown.starts_with("Diverging { domain: [-1.0, 0.0, 4.0]"),
        "{shown}"
    );
}
