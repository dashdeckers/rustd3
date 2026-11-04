//! Pies against d3-shape 3.2.0's `pie()`, run in JavaScript.

use std::f64::consts::{PI, TAU};

use rustd3_core::path::SvgPath;
use rustd3_core::shape::{self, pie};

/// Each slice as (index, start, end), in input order.
fn rows(slices: &[shape::Slice]) -> Vec<(usize, f64, f64)> {
    slices
        .iter()
        .map(|s| (s.index, s.start_angle, s.end_angle))
        .collect()
}

#[test]
fn pie_and_donut_examples() {
    let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    assert_eq!((slices[0].datum, slices[0].index), (0, 0));
    assert_eq!((slices[0].start_angle, slices[0].end_angle), (0.0, PI));
    assert_eq!((slices[1].datum, slices[1].index), (1, 2));
    assert!((slices[1].start_angle - 5.0 * PI / 3.0).abs() < 1e-12);
    assert!((slices[1].end_angle - TAU).abs() < 1e-12);
    let donut = shape::arc().inner_radius(40.0).outer_radius(80.0);
    assert_eq!(donut.centroid(&slices[0]), (60.0, 0.0));
    let mut path = SvgPath::with_digits(3);
    donut.draw(&slices[0], &mut path);
    assert_eq!(
        path.to_string(),
        "M0,-80A80,80,0,1,1,0,80L0,40A40,40,0,1,0,0,-40Z"
    );
}

#[test]
fn a_pie_draws_the_largest_value_first_and_returns_slices_in_input_order() {
    let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    assert_eq!(
        rows(&slices),
        [
            (0, 0.0, PI),
            (2, 5.235987755982988, 6.283185307179585),
            (1, PI, 5.235987755982988)
        ]
    );
    assert!(slices.iter().enumerate().all(|(i, s)| s.datum == i));
    assert_eq!(slices[1].value, 1.0);
}

#[test]
fn input_order_lays_slices_out_as_given() {
    let slices = shape::pie().sort(pie::Sort::Input).layout([3.0, 1.0, 2.0]);
    assert_eq!(
        rows(&slices),
        [
            (0, 0.0, PI),
            (1, PI, 4.1887902047863905),
            (2, 4.1887902047863905, TAU)
        ]
    );
}

#[test]
fn every_slice_carries_the_pad_angle() {
    let slices = shape::pie().pad_angle(0.1).layout([1.0, 1.0]);
    assert_eq!(rows(&slices), [(0, 0.0, PI), (1, PI, TAU)]);
    assert!(slices.iter().all(|s| s.pad_angle == 0.1));
}

#[test]
fn nan_values_sort_last_both_ways_and_span_no_angle() {
    let values = [1.0, f64::NAN, 2.0, f64::NAN];
    let index = |sort| -> Vec<usize> {
        shape::pie()
            .sort(sort)
            .layout(values)
            .iter()
            .map(|s| s.index)
            .collect()
    };
    assert_eq!(index(pie::Sort::Descending), [1, 2, 0, 3]);
    assert_eq!(index(pie::Sort::Ascending), [0, 2, 1, 3]);
    let slices = shape::pie().layout(values);
    assert_eq!(slices[1].start_angle, slices[1].end_angle);
    assert_eq!(slices[3].end_angle, TAU);
}

#[test]
fn signed_zeros_compare_equal_and_keep_their_order() {
    let slices = shape::pie()
        .sort(pie::Sort::Ascending)
        .layout([0.0, -0.0, 1.0]);
    assert_eq!(
        slices.iter().map(|s| s.index).collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

#[test]
fn a_zero_sum_puts_every_slice_at_the_start_angle() {
    let slices = shape::pie().start_angle(1.0).layout([0.0, -1.0]);
    assert_eq!(rows(&slices), [(0, 1.0, 1.0), (1, 1.0, 1.0)]);
}

#[test]
fn the_span_is_clamped_to_tau_and_the_pad_to_the_span_per_value() {
    let wide = shape::pie().end_angle(7.0).layout([1.0, 2.0]);
    assert_eq!(wide[1].start_angle, 0.0);
    assert_eq!(wide[0].end_angle, TAU);
    let padded = shape::pie()
        .end_angle(-PI)
        .pad_angle(f64::INFINITY)
        .layout([1.0, 2.0, 3.0]);
    assert_eq!(padded[0].pad_angle, PI / 3.0);
    assert_eq!(padded[0].end_angle, -PI);
}

#[test]
fn no_values_lay_out_no_slices() {
    assert!(shape::pie().layout([]).is_empty());
}
