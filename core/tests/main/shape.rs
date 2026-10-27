//! Lines and areas against what d3-shape 3.2.0 prints, compared after the
//! normalization of d3's `assertPathEqual`.

use rustd3_core::path;
use rustd3_core::shape::{self, Curve};
use rustd3_testkit::obs::{Token, tokenize};

/// The spot checks' points.
const PTS: [(f64, f64); 4] = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];

/// The area rows' tops over the baseline.
const TOPS: [(f64, f64); 3] = [(0.0, 4.0), (1.0, 2.0), (2.0, 5.0)];

/// d3-shape's `normalizePath`: numbers within 1e-6 of an integer as that
/// integer, every other number with 3 decimals.
fn normalize(path: &str) -> String {
    tokenize(path)
        .into_iter()
        .map(|token| match token {
            Token::Text(text) => text,
            Token::Num { value, .. } if (value - value.round()).abs() < 1e-6 => {
                (value.round() + 0.0).to_string()
            }
            Token::Num { value, .. } => format!("{value:.3}"),
        })
        .collect()
}

/// `assertPathEqual(actual, expected)`.
#[track_caller]
fn assert_path(actual: &str, expected: &str) {
    assert_eq!(normalize(actual), normalize(expected));
}

fn line(curve: Curve, points: &[(f64, f64)]) -> String {
    path::svg(|p| shape::line().curve(curve).draw(points.iter().copied(), p))
}

fn area(curve: Curve, tops: &[(f64, f64)]) -> String {
    path::svg(|p| shape::area().curve(curve).draw(tops.iter().copied(), p))
}

#[test]
fn normalize_rounds_like_assert_path_equal() {
    assert_eq!(
        normalize("M-0.0000001,0.16666666L2.9999999,-1.0005"),
        "M0,0.167L3,-1.000"
    );
}

#[test]
fn linear_joins_points_with_straight_segments() {
    assert_path(&line(Curve::Linear, &PTS), "M0,0L1,2L2,1L3,3");
}

#[test]
fn linear_closed_closes_the_line() {
    assert_path(&line(Curve::LinearClosed, &PTS), "M0,0L1,2L2,1L3,3Z");
}

#[test]
fn a_single_point_line_is_a_closed_move() {
    assert_path(&line(Curve::Linear, &[(0.0, 1.0)]), "M0,1Z");
}

#[test]
fn an_empty_line_draws_no_commands() {
    assert_eq!(line(Curve::Linear, &[]), "");
    assert_eq!(area(Curve::Linear, &[]), "");
}

#[test]
fn none_breaks_the_line_into_runs() {
    let points = [
        Some((0.0, 75.0)),
        Some((100.0, 25.0)),
        None,
        Some((300.0, 50.0)),
        Some((400.0, 0.0)),
    ];
    let d = path::svg(|p| shape::line().draw(points, p));
    assert_path(&d, "M0,75L100,25M300,50L400,0");
}

#[test]
fn leading_trailing_and_repeated_gaps_draw_nothing_extra() {
    let points = [
        None,
        Some((0.0, 1.0)),
        None,
        None,
        Some((2.0, 3.0)),
        Some((4.0, 5.0)),
        None,
    ];
    let d = path::svg(|p| shape::line().draw(points, p));
    assert_eq!(d, "M0,1ZM2,3L4,5");
}

#[test]
fn nan_coordinates_are_drawn_not_skipped() {
    let d = path::svg(|p| shape::line().draw([(0.0, 1.0), (f64::NAN, 2.0)], p));
    assert_eq!(d, "M0,1LNaN,2");
}

#[test]
fn an_area_of_three_tuples_runs_along_y1_and_back_along_y0() {
    let rows = [(0.0, 10.0, 4.0), (1.0, 10.0, 2.0), (2.0, 10.0, 5.0)];
    let d = path::svg(|p| shape::area().draw(rows.map(Some), p));
    assert_path(&d, "M0,4L1,2L2,5L2,10L1,10L0,10Z");
}

#[test]
fn y0_sets_the_baseline_of_two_tuples() {
    let d = path::svg(|p| shape::area().y0(300.0).draw(TOPS, p));
    assert_path(&d, "M0,4L1,2L2,5L2,300L1,300L0,300Z");
}

#[test]
fn the_default_baseline_is_zero() {
    assert_path(&area(Curve::Linear, &TOPS), "M0,4L1,2L2,5L2,0L1,0L0,0Z");
}

#[test]
fn three_tuples_ignore_the_baseline() {
    let rows = [(0.0, 10.0, 4.0), (1.0, 10.0, 2.0)];
    let d = path::svg(|p| shape::area().y0(300.0).draw(rows, p));
    assert_eq!(d, "M0,4L1,2L1,10L0,10Z");
}

#[test]
fn a_single_point_area_is_a_closed_vertical_segment() {
    assert_path(&area(Curve::Linear, &[(0.0, 1.0)]), "M0,1L0,0Z");
}

#[test]
fn none_breaks_the_area_into_closed_rings() {
    let items = [Some((0.0, 1.0)), Some((1.0, 2.0)), None, Some((3.0, 4.0))];
    let d = path::svg(|p| shape::area().draw(items, p));
    assert_eq!(d, "M0,1L1,2L1,0L0,0ZM3,4L3,0Z");
}

#[test]
fn a_linear_closed_area_draws_two_rings_without_a_z_of_its_own() {
    assert_path(
        &area(Curve::LinearClosed, &TOPS),
        "M0,4L1,2L2,5ZM2,0L1,0L0,0Z",
    );
}
