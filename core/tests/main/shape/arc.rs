//! Arcs against d3-shape 3.2.0's `arc()`, run in Node and printed by its
//! default 3-digit path.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use rustd3_core::path::SvgPath;
use rustd3_core::shape::{self, Arc, Slice};

/// A slice literal from `start` to `end`, unpadded.
fn slice(start: f64, end: f64) -> Slice {
    Slice {
        datum: 0,
        index: 0,
        value: 0.0,
        start_angle: start,
        end_angle: end,
        pad_angle: 0.0,
    }
}

/// `arc` drawing `slice` as d3-shape prints it, at 3 digits.
fn drawn(arc: Arc, slice: &Slice) -> String {
    let mut path = SvgPath::with_digits(3);
    arc.draw(slice, &mut path);
    path.to_string()
}

#[test]
fn a_donut_slice_from_a_pie_is_one_closed_subpath() {
    let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    let donut = shape::arc().inner_radius(40.0).outer_radius(80.0);
    assert_eq!(
        drawn(donut, &slices[0]),
        "M0,-80A80,80,0,1,1,0,80L0,40A40,40,0,1,0,0,-40Z"
    );
    assert_eq!(donut.centroid(&slices[0]), (60.0, 0.0));
}

#[test]
fn a_circular_sector_runs_out_along_the_ring_and_back_to_the_centre() {
    let arc = shape::arc().outer_radius(100.0);
    assert_eq!(
        drawn(arc, &slice(0.0, FRAC_PI_2)),
        "M0,-100A100,100,0,0,1,100,0L0,0Z"
    );
}

#[test]
fn corners_round_both_rings() {
    let arc = shape::arc()
        .inner_radius(50.0)
        .outer_radius(100.0)
        .corner_radius(10.0);
    assert_eq!(
        drawn(arc, &slice(0.0, FRAC_PI_2)),
        "M0,-89.443A10,10,0,0,1,11.111,-99.381A100,100,0,0,1,99.381,-11.111\
         A10,10,0,0,1,89.443,0L59.161,0A10,10,0,0,1,49.301,-8.333\
         A50,50,0,0,0,8.333,-49.301A10,10,0,0,1,0,-59.161Z"
    );
}

#[test]
fn the_pad_angle_narrows_both_edges_by_one_width() {
    let arc = shape::arc()
        .inner_radius(50.0)
        .outer_radius(100.0)
        .pad_angle(Some(0.1));
    assert_eq!(
        drawn(arc, &slice(0.0, FRAC_PI_2)),
        "M5.588,-99.844A100,100,0,0,1,99.844,-5.588L49.687,-5.588A50,50,0,0,0,5.588,-49.687Z"
    );
}

#[test]
fn the_slice_pad_angle_applies_unless_the_arc_sets_one() {
    let padded = Slice {
        pad_angle: 0.1,
        ..slice(0.0, FRAC_PI_2)
    };
    let arc = shape::arc().inner_radius(50.0).outer_radius(100.0);
    assert_eq!(
        drawn(arc, &padded),
        drawn(arc.pad_angle(Some(0.1)), &slice(0.0, FRAC_PI_2))
    );
    assert_eq!(
        drawn(arc.pad_angle(Some(0.0)), &padded),
        drawn(arc, &slice(0.0, FRAC_PI_2))
    );
}

#[test]
fn a_full_turn_draws_an_annulus_and_a_zero_radius_a_point() {
    let arc = shape::arc().inner_radius(100.0).outer_radius(50.0);
    assert_eq!(
        drawn(arc, &slice(0.0, TAU)),
        "M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z"
    );
    assert_eq!(drawn(shape::arc(), &slice(0.0, PI)), "M0,0Z");
}
