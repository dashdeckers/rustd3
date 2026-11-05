//! `SvgPath` against what `d3.path()` and `d3.pathRound()` print.

use std::f64::consts::{E, FRAC_PI_2, PI, TAU};

use rustd3_core::path::{self, PathContext, SvgPath};
use rustd3_testkit::obs::{Obs, Tolerance};

/// Tolerant equality of two path strings, number by number.
fn tolerant_eq(actual: &str, expected: &str) -> bool {
    Obs::Str(actual.to_owned()).approx_eq(&Obs::Str(expected.to_owned()), &Tolerance::default())
}

#[test]
fn a_line_prints_as_move_and_line_commands() {
    let d = path::svg(|p| {
        p.move_to(150.0, 50.0);
        p.line_to(200.0, 100.0);
    });
    assert_eq!(d, "M150,50L200,100");
}

#[test]
fn a_quarter_arc_prints_its_tiny_end_coordinate_without_exponent() {
    let d = path::svg(|p| {
        p.move_to(0.0, 0.0);
        p.arc(0.0, 0.0, 50.0, 0.0, FRAC_PI_2, false);
    });
    assert_eq!(
        d,
        "M0,0L50,0A50,50,0,0,1,0.000000000000003061616997868383,50"
    );
    assert!(tolerant_eq(
        &d,
        "M0,0L50,0A50,50,0,0,1,3.061616997868383e-15,50"
    ));
}

#[test]
fn a_full_circle_on_an_empty_path_is_two_half_arcs() {
    let d = path::svg(|p| p.arc(100.0, 100.0, 50.0, 0.0, TAU, false));
    assert_eq!(d, "M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100");
}

#[test]
fn arc_to_draws_a_line_then_a_tangent_arc() {
    let mut p = SvgPath::new();
    p.move_to(270.0, 182.0);
    p.arc_to(270.0, 39.0, 363.0, 39.0, 53.0).unwrap();
    assert_eq!(p.to_string(), "M270,182L270,92A53,53,0,0,1,323,39");
}

#[test]
fn rect_prints_relative_sides_and_closes() {
    let mut p = SvgPath::new();
    p.rect(200.0, 100.0, 70.0, 40.0);
    assert_eq!(p.to_string(), "M200,100h70v40h-70Z");
}

#[test]
fn with_digits_rounds_like_path_round() {
    let mut p = SvgPath::with_digits(1);
    p.move_to(123.456, 789.012);
    assert_eq!(p.to_string(), "M123.5,789");
    let mut p = SvgPath::with_digits(3);
    p.move_to(PI, E);
    assert_eq!(p.to_string(), "M3.142,2.718");
}

#[test]
fn with_digits_above_fifteen_leaves_numbers_unrounded() {
    let mut p = SvgPath::with_digits(40);
    p.move_to(PI, E);
    assert_eq!(p.to_string(), "M3.141592653589793,2.718281828459045");
}

#[test]
fn negative_zero_prints_as_zero() {
    let mut p = SvgPath::with_digits(1);
    p.move_to(-0.04, 0.0);
    p.rect(0.0, 0.0, 0.0, 1.0);
    assert_eq!(p.to_string(), "M0,0M0,0h0v1h0Z");
}

#[test]
fn a_negative_radius_is_an_error_and_draws_nothing() {
    let mut p = SvgPath::new();
    assert_eq!(
        p.try_arc(0.0, 0.0, -1.0, 0.0, 1.0, false),
        Err(path::Error::NegativeRadius(-1.0))
    );
    p.arc(0.0, 0.0, -1.0, 0.0, 1.0, false);
    assert_eq!(p.to_string(), "");
    assert_eq!(
        p.arc_to(0.0, 0.0, 1.0, 1.0, -2.0),
        Err(path::Error::NegativeRadius(-2.0))
    );
    assert_eq!(p, SvgPath::new());
}

#[test]
fn close_path_returns_to_the_subpath_start_only_when_drawing() {
    let mut p = SvgPath::new();
    p.close_path();
    assert_eq!(p.to_string(), "");
    p.move_to(1.0, 2.0);
    p.line_to(3.0, 4.0);
    p.close_path();
    p.arc(1.0, 2.0, 0.0, 0.0, 1.0, false);
    assert_eq!(p.to_string(), "M1,2L3,4Z");
}
