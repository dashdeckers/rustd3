//! `scale::Band`, d3's scaleBand: the spot checks of d3-scale 4.0.2 and
//! its axis through d3-axis's `center`.

use rustd3_core::axis;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};

/// The band starts of `keys` in order.
fn starts(s: &scale::Band<&'static str>, keys: &[&'static str]) -> Vec<Option<f64>> {
    keys.iter().map(|k| s.apply(k)).collect()
}

#[test]
fn padded_bands_start_after_the_outer_padding() {
    let s = scale::band(["a", "b", "c"])
        .range([0.0, 300.0])
        .padding(0.1);
    assert_eq!(s.apply(&"a"), Some(9.677419354838719));
    assert_eq!(s.apply(&"z"), None);
    assert_eq!(s.bandwidth(), 87.09677419354838);
    assert_eq!(s.step(), 96.77419354838709);
}

#[test]
fn rounded_bands_floor_the_step_and_round_start_and_width() {
    let s = scale::band(["a", "b", "c"])
        .range([0.0, 300.0])
        .padding(0.1)
        .round(true);
    assert_eq!(starts(&s, &["a", "b"]), [Some(11.0), Some(107.0)]);
    assert_eq!((s.bandwidth(), s.step()), (86.0, 96.0));
}

#[test]
fn a_descending_range_puts_the_first_key_last() {
    let s = scale::band(["a", "b", "c"]).range([300.0, 0.0]);
    assert_eq!(
        starts(&s, &["a", "b", "c"]),
        [Some(200.0), Some(100.0), Some(0.0)]
    );
    assert_eq!(s.bandwidth(), 100.0);
}

#[test]
fn duplicate_keys_keep_their_first_position() {
    let s = scale::band(["a", "b", "a", "c"]).range([0.0, 120.0]);
    assert_eq!(s.get_domain(), ["a", "b", "c"]);
    assert_eq!(s.apply(&"c"), Some(80.0));
    assert_eq!(s.bandwidth(), 40.0);
}

#[test]
fn an_empty_domain_spans_the_whole_range() {
    let s = scale::band::<&str>([]).range([0.0, 120.0]);
    assert_eq!((s.step(), s.bandwidth()), (120.0, 120.0));
}

#[test]
fn align_zero_puts_the_outer_space_after_the_bands() {
    let s = scale::band(["a", "b", "c"])
        .range([0.0, 120.0])
        .padding_inner(0.2)
        .padding_outer(0.5)
        .align(0.0);
    assert_eq!(
        starts(&s, &["a", "b", "c"]),
        [Some(0.0), Some(31.578947368421055), Some(63.15789473684211)]
    );
    assert_eq!(s.bandwidth(), 25.263157894736846);
}

#[test]
fn range_round_is_range_with_rounding() {
    let a = scale::band(["a", "b", "c"]).range_round([0.0, 100.0]);
    let b = scale::band(["a", "b", "c"]).range([0.0, 100.0]).round(true);
    assert_eq!(a, b);
    assert_eq!(
        starts(&a, &["a", "b", "c"]),
        [Some(1.0), Some(34.0), Some(67.0)]
    );
    assert_eq!(a.get_range(), [0.0, 100.0]);
}

#[test]
fn inner_padding_is_at_most_one_and_align_is_clamped() {
    let wide = scale::band(["a"]).range([0.0, 960.0]).padding_inner(2.0);
    let one = scale::band(["a"]).range([0.0, 960.0]).padding_inner(1.0);
    assert_eq!(wide, one);
    assert_eq!(wide.bandwidth(), 0.0);
    let s = |a| {
        scale::band(["a", "b"])
            .range([0.0, 100.0])
            .padding(0.5)
            .align(a)
    };
    assert_eq!(s(-3.0), s(0.0));
    assert_eq!(s(3.0), s(1.0));
}

#[test]
fn padding_sets_both_paddings_with_the_inner_one_capped() {
    let s = scale::band(["foo"]).range([0.0, 960.0]).padding(0.5);
    assert_eq!(s.apply(&"foo"), Some(320.0));
    assert_eq!((s.step(), s.bandwidth()), (640.0, 320.0));
    let t = scale::band(["foo"]).range([0.0, 960.0]).padding(2.0);
    let u = scale::band(["foo"])
        .range([0.0, 960.0])
        .padding_inner(1.0)
        .padding_outer(2.0);
    assert_eq!(t, u);
}

#[test]
fn a_bottom_axis_puts_ticks_at_the_band_centers() {
    let s = scale::band(["a", "b", "c"])
        .range([0.0, 300.0])
        .padding(0.1);
    let labels = axis::bottom(&s).labels();
    let xs: Vec<f64> = labels.iter().map(|l| l.get_x()).collect();
    assert_eq!(xs, [53.22580645161291, 150.0, 246.77419354838707]);
    let texts: Vec<&str> = labels.iter().map(|l| l.get_text()).collect();
    assert_eq!(texts, ["a", "b", "c"]);
}

#[test]
fn band_ticks_are_every_key_whatever_the_count() {
    let s = scale::band(["a", "b", "c"]).range([10.0, 20.0]);
    let set = s.tick_set(1);
    assert_eq!(set.get_values(), ["a", "b", "c"]);
    assert_eq!((set.get_step(), set.get_count()), (&(), 1));
    assert_eq!(s.labels(&set, &Locale::EN_US), ["a", "b", "c"]);
    assert_eq!(s.ends(), [10.0, 20.0]);
    assert!(Ticks::minor(&s, &set).is_empty());
    assert_eq!(s.position(&"z", 0.5), None);
}

#[test]
fn a_rounding_band_rounds_the_offset_into_its_bands() {
    let s = scale::band([1, 2]).range([0.0, 9.0]).round(true);
    assert_eq!((s.apply(&1), s.bandwidth()), (Some(1.0), 4.0));
    // max(0, 4 − 2 · 0.75) / 2 = 1.25, rounded to 1.
    assert_eq!(s.position(&1, 0.75), Some(2.0));
    // Unrounded: step and bandwidth 4.5, so the offset is (4.5 − 1.5) / 2.
    let unrounded = scale::band([1, 2]).range([0.0, 9.0]);
    assert_eq!(unrounded.position(&1, 0.75), Some(1.5));
}
