//! Radial lines and areas against d3-shape 3.2.0's `lineRadial()` and
//! `areaRadial()`, run in JavaScript and printed by its default 3-digit path.

use std::f64::consts::{FRAC_PI_2, PI};

use rustd3_core::path::SvgPath;
use rustd3_core::shape::{Curve, area, line};

/// What `draw` writes into a 3-digit path.
fn drawn(draw: impl FnOnce(&mut SvgPath)) -> String {
    let mut path = SvgPath::with_digits(3);
    draw(&mut path);
    path.to_string()
}

#[test]
fn a_radial_line_places_each_point_at_its_angle_and_radius() {
    let d = drawn(|p| line::radial().draw([(0.0, 1.0), (FRAC_PI_2, 2.0)], p));
    assert_eq!(d, "M0,-1L2,0");
}

#[test]
fn a_radial_line_curves_the_placed_points() {
    let points = [(0.0, 1.0), (1.0, 2.0), (2.0, 1.5), (3.0, 2.5)];
    let d = drawn(|p| {
        line::radial()
            .curve(Curve::Cardinal { tension: 0.0 })
            .draw(points, p)
    });
    assert_eq!(
        d,
        "M0,-1C0,-1,1.456,-1.351,1.683,-1.081C1.91,-0.81,1.586,0.032,1.364,0.624\
         C1.142,1.217,0.353,2.475,0.353,2.475"
    );
}

#[test]
fn none_breaks_a_radial_line() {
    let points = [
        Some((0.0, 1.0)),
        None,
        Some((PI, 1.0)),
        Some((PI * 1.5, 2.0)),
    ];
    assert_eq!(drawn(|p| line::radial().draw(points, p)), "M0,-1ZM0,1L-2,0");
}

#[test]
fn a_radial_area_runs_out_along_the_outer_radius_and_back_along_the_inner() {
    let d = drawn(|p| area::radial().draw([(0.0, 1.0, 2.0), (FRAC_PI_2, 1.0, 3.0)], p));
    assert_eq!(d, "M0,-2L3,0L1,0L0,-1Z");
}

#[test]
fn two_tuples_take_the_inner_radius_and_both_edges_the_curve() {
    let items = [(0.0, 2.0), (1.0, 3.0), (2.0, 2.5)];
    let d = drawn(|p| {
        area::radial()
            .inner_radius(1.0)
            .curve(Curve::Basis)
            .draw(items, p);
    });
    assert_eq!(
        d,
        "M0,-2L0.421,-1.937C0.841,-1.874,1.683,-1.747,2.062,-1.241\
         C2.441,-0.734,2.357,0.153,2.315,0.597L2.273,1.04L0.909,0.416\
         L0.898,0.257C0.887,0.097,0.864,-0.221,0.713,-0.458\
         C0.561,-0.694,0.28,-0.847,0.14,-0.923L0,-1Z"
    );
}

#[test]
fn none_breaks_a_radial_area_into_rings() {
    let items = [
        Some((0.0, 1.0, 2.0)),
        None,
        Some((2.0, 1.0, 3.0)),
        Some((3.0, 1.5, 2.0)),
    ];
    assert_eq!(
        drawn(|p| area::radial().draw(items, p)),
        "M0,-2L0,-1ZM2.728,1.248L0.282,1.98L0.212,1.485L0.909,0.416Z"
    );
}
