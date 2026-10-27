//! Lines and areas against what d3-shape 3.2.0 prints, compared after the
//! normalization of d3's `assertPathEqual`.

use rustd3_core::path::{self, PathContext};
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

/// A `PathContext` that counts the commands drawn into it.
#[derive(Default)]
struct Count(usize);

impl PathContext for Count {
    fn move_to(&mut self, _x: f64, _y: f64) {
        self.0 += 1;
    }

    fn line_to(&mut self, _x: f64, _y: f64) {
        self.0 += 1;
    }

    fn quadratic_curve_to(&mut self, _x1: f64, _y1: f64, _x: f64, _y: f64) {
        self.0 += 1;
    }

    fn bezier_curve_to(&mut self, _x1: f64, _y1: f64, _x2: f64, _y2: f64, _x: f64, _y: f64) {
        self.0 += 1;
    }

    fn arc(&mut self, _x: f64, _y: f64, _r: f64, _start: f64, _end: f64, _anticlockwise: bool) {
        self.0 += 1;
    }

    fn close_path(&mut self) {
        self.0 += 1;
    }
}

#[test]
fn step_puts_the_vertical_segment_halfway() {
    assert_path(
        &line(Curve::Step, &PTS),
        "M0,0L0.5,0L0.5,2L1.5,2L1.5,1L2.5,1L2.5,3L3,3",
    );
}

#[test]
fn step_before_puts_the_vertical_segment_first() {
    assert_path(
        &line(Curve::StepBefore, &PTS),
        "M0,0L0,2L1,2L1,1L2,1L2,3L3,3",
    );
}

#[test]
fn step_after_puts_the_vertical_segment_last() {
    assert_path(
        &line(Curve::StepAfter, &PTS),
        "M0,0L1,0L1,2L2,2L2,1L3,1L3,3",
    );
}

#[test]
fn a_step_area_steps_its_baseline_the_other_way() {
    assert_path(
        &area(Curve::StepAfter, &TOPS),
        "M0,4L1,4L1,2L2,2L2,5L2,0L2,0L1,0L1,0L0,0Z",
    );
}

#[test]
fn basis_runs_from_the_first_point_to_the_last() {
    assert_path(
        &line(Curve::Basis, &PTS),
        "M0,0L0.167,0.333C0.333,0.667,0.667,1.333,1,1.5C1.333,1.667,1.667,1.333,2,1.5\
         C2.333,1.667,2.667,2.333,2.833,2.667L3,3",
    );
}

#[test]
fn basis_draws_six_commands_for_four_points() {
    let mut count = Count::default();
    shape::line().curve(Curve::Basis).draw(PTS, &mut count);
    assert_eq!(count.0, 6);
}

#[test]
fn basis_closed_closes_the_spline() {
    assert_path(
        &line(Curve::BasisClosed, &PTS),
        "M1,1.5C1.333,1.667,1.667,1.333,2,1.5C2.333,1.667,2.667,2.333,2.333,2.167\
         C2,2,1,1,0.667,0.833C0.333,0.667,0.667,1.333,1,1.5",
    );
}

#[test]
fn basis_open_starts_and_ends_at_the_inner_knots() {
    assert_path(
        &line(Curve::BasisOpen, &PTS),
        "M1,1.5C1.333,1.667,1.667,1.333,2,1.5",
    );
}

#[test]
fn a_basis_closed_area_draws_two_closed_splines() {
    assert_path(
        &area(Curve::BasisClosed, &TOPS),
        "M1,2.833C1.333,3,1.667,4,1.5,4.333C1.333,4.667,0.667,4.333,0.5,3.833\
         C0.333,3.333,0.667,2.667,1,2.833M1,0C0.667,0,0.333,0,0.5,0C0.667,0,1.333,0,1.5,0\
         C1.667,0,1.333,0,1,0",
    );
}

#[test]
fn bump_x_leaves_and_reaches_each_point_horizontally() {
    assert_path(
        &line(Curve::BumpX, &PTS),
        "M0,0C0.5,0,0.5,2,1,2C1.5,2,1.5,1,2,1C2.5,1,2.5,3,3,3",
    );
}

#[test]
fn bump_y_leaves_and_reaches_each_point_vertically() {
    assert_path(
        &line(Curve::BumpY, &PTS),
        "M0,0C0,1,1,1,1,2C1,1.5,2,1.5,2,1C2,2,3,2,3,3",
    );
}

#[test]
fn bundle_at_d3s_default_beta_straightens_the_basis_a_little() {
    assert_path(
        &line(Curve::Bundle { beta: 0.85 }, &PTS),
        "M0,0L0.167,0.308C0.333,0.617,0.667,1.233,1,1.425C1.333,1.617,1.667,1.383,2,1.575\
         C2.333,1.767,2.667,2.383,2.833,2.692L3,3",
    );
}

#[test]
fn bundle_at_half_beta_straightens_the_basis_halfway() {
    assert_path(
        &line(Curve::Bundle { beta: 0.5 }, &PTS),
        "M0,0L0.167,0.25C0.333,0.5,0.667,1,1,1.25C1.333,1.5,1.667,1.5,2,1.75\
         C2.333,2,2.667,2.5,2.833,2.75L3,3",
    );
}

#[test]
fn bundle_at_beta_one_is_basis() {
    assert_eq!(
        line(Curve::Bundle { beta: 1.0 }, &PTS),
        line(Curve::Basis, &PTS)
    );
}

#[test]
fn a_bundle_area_draws_as_basis() {
    assert_eq!(
        area(Curve::Bundle { beta: 0.5 }, &TOPS),
        area(Curve::Basis, &TOPS)
    );
}
