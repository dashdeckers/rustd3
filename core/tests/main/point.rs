//! `scale::Point`, d3's scalePoint: the spot checks of d3-scale 4.0.2.

use rustd3_core::axis;
use rustd3_core::scale::{self, Ticks};

/// The positions of `a`, `b` and `c`.
fn abc(s: &scale::Point<&'static str>) -> [Option<f64>; 3] {
    ["a", "b", "c"].map(|k| s.apply(&k))
}

#[test]
fn points_span_the_range_with_zero_bandwidth() {
    let s = scale::point(["a", "b", "c"]).range([0.0, 1.0]);
    assert_eq!(abc(&s), [Some(0.0), Some(0.5), Some(1.0)]);
    assert_eq!((s.bandwidth(), s.step()), (0.0, 0.5));
}

#[test]
fn padding_is_the_outer_padding_in_steps() {
    let s = scale::point(["a", "b", "c"]).range([0.0, 1.0]).padding(1.0);
    assert_eq!(abc(&s), [Some(0.25), Some(0.5), Some(0.75)]);
    assert_eq!(s.step(), 0.25);
}

#[test]
fn round_floors_the_step_and_rounds_the_first_point() {
    let s = scale::point(["a", "b", "c"])
        .range([0.0, 100.0])
        .round(true)
        .padding(0.5);
    assert_eq!(abc(&s), [Some(17.0), Some(50.0), Some(83.0)]);
    assert_eq!(s.step(), 33.0);
}

#[test]
fn range_round_is_range_with_rounding() {
    let s = scale::point(["a", "b", "c"])
        .range_round([0.0, 100.0])
        .padding(0.5);
    assert_eq!(abc(&s), [Some(17.0), Some(50.0), Some(83.0)]);
    assert_eq!(s.step(), 33.0);
    assert_eq!(s.get_range(), [0.0, 100.0]);
}

#[test]
fn align_zero_puts_the_outer_space_after_the_points() {
    let s = scale::point(["a", "b", "c"])
        .range([0.0, 100.0])
        .padding(0.5)
        .align(0.0);
    assert_eq!(
        abc(&s),
        [Some(0.0), Some(33.333333333333336), Some(66.66666666666667)]
    );
    assert_eq!(s.step(), 33.333333333333336);
}

#[test]
fn a_point_scale_is_a_band_scale_with_inner_padding_one() {
    let p = scale::point(["foo", "bar"])
        .range([0.0, 960.0])
        .padding(0.5);
    let b = scale::band(["foo", "bar"])
        .range([0.0, 960.0])
        .padding_inner(1.0)
        .padding_outer(0.5);
    for key in ["foo", "bar", "baz"] {
        assert_eq!(p.apply(&key), b.apply(&key));
    }
    assert_eq!((p.step(), p.bandwidth()), (b.step(), b.bandwidth()));
    assert_eq!(p.get_domain(), ["foo", "bar"]);
}

#[test]
fn a_bottom_axis_puts_ticks_at_the_points() {
    let s = scale::point(["a", "b", "c"]).range([0.0, 100.0]);
    let labels = axis::bottom(&s).labels();
    let xs: Vec<f64> = labels.iter().map(|l| l.x()).collect();
    assert_eq!(xs, [0.5, 50.5, 100.5]);
    assert_eq!(s.extent(), [0.0, 100.0]);
    assert_eq!(s.tick_set(3).values(), ["a", "b", "c"]);
}
