//! Lines and areas against what d3-shape 3.2.0 prints, compared after the
//! normalization of d3's `assertPathEqual`; pies, arcs, stacks, symbols
//! and radial shapes in the modules below.

mod arc;
mod pie;
mod stack;

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

/// The points of d3's own cardinal tests.
const ZIGZAG: [(f64, f64); 4] = [(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)];

#[test]
fn cardinal_at_tension_zero_passes_through_every_point() {
    assert_path(
        &line(Curve::Cardinal { tension: 0.0 }, &PTS),
        "M0,0C0,0,0.667,1.833,1,2C1.333,2.167,1.667,0.833,2,1C2.333,1.167,3,3,3,3",
    );
    assert_path(
        &line(Curve::Cardinal { tension: 0.0 }, &ZIGZAG),
        "M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3",
    );
}

#[test]
fn cardinal_tension_shortens_the_tangents() {
    assert_path(
        &line(Curve::Cardinal { tension: 0.5 }, &ZIGZAG),
        "M0,1C0,1,0.833333,3,1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3,3,3,3",
    );
}

#[test]
fn cardinal_closed_closes_the_spline() {
    assert_path(
        &line(Curve::CardinalClosed { tension: 0.0 }, &PTS),
        "M1,2C1.333,2.167,1.667,0.833,2,1C2.333,1.167,3.333,3.167,3,3\
         C2.667,2.833,0.333,0.167,0,0C-0.333,-0.167,0.667,1.833,1,2",
    );
}

#[test]
fn cardinal_open_runs_from_the_second_point_to_the_second_to_last() {
    assert_path(
        &line(Curve::CardinalOpen { tension: 0.0 }, &PTS),
        "M1,2C1.333,2.167,1.667,0.833,2,1",
    );
}

#[test]
fn a_cardinal_area_runs_back_along_its_baseline() {
    assert_path(
        &area(Curve::Cardinal { tension: 0.0 }, &ZIGZAG[..3]),
        "M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1L2,0C2,0,1.333333,0,1,0\
         C0.666667,0,0,0,0,0Z",
    );
}

#[test]
fn a_cardinal_closed_area_draws_two_closed_splines() {
    assert_path(
        &area(Curve::CardinalClosed { tension: 0.0 }, &TOPS),
        "M1,2C1.333,2.167,2.167,4.667,2,5C1.833,5.333,0.167,4.5,0,4C-0.167,3.5,0.667,1.833,1,2\
         M1,0C0.667,0,-0.167,0,0,0C0.167,0,1.833,0,2,0C2.167,0,1.333,0,1,0",
    );
}

#[test]
fn catmull_rom_at_half_alpha_is_centripetal() {
    assert_path(
        &line(Curve::CatmullRom { alpha: 0.5 }, &PTS),
        "M0,0C0,0,0.619,1.938,1,2C1.303,2.049,1.697,0.951,2,1C2.381,1.062,3,3,3,3",
    );
}

#[test]
fn catmull_rom_closed_closes_the_spline() {
    assert_path(
        &line(Curve::CatmullRomClosed { alpha: 0.5 }, &PTS),
        "M1,2C1.303,2.049,1.697,0.951,2,1C2.381,1.062,3.112,2.919,3,3\
         C2.845,3.111,0.155,-0.111,0,0C-0.112,0.081,0.619,1.938,1,2",
    );
}

#[test]
fn catmull_rom_open_runs_from_the_second_point_to_the_second_to_last() {
    assert_path(
        &line(Curve::CatmullRomOpen { alpha: 0.5 }, &PTS),
        "M1,2C1.303,2.049,1.697,0.951,2,1",
    );
}

#[test]
fn catmull_rom_at_alpha_zero_is_the_cardinal_spline_of_tension_zero() {
    let cardinal = [
        Curve::Cardinal { tension: 0.0 },
        Curve::CardinalClosed { tension: 0.0 },
        Curve::CardinalOpen { tension: 0.0 },
    ];
    let catmull_rom = [
        Curve::CatmullRom { alpha: 0.0 },
        Curve::CatmullRomClosed { alpha: 0.0 },
        Curve::CatmullRomOpen { alpha: 0.0 },
    ];
    for (a, b) in cardinal.into_iter().zip(catmull_rom) {
        assert_eq!(line(a, &PTS), line(b, &PTS));
        assert_eq!(area(a, &TOPS), area(b, &TOPS));
    }
}

#[test]
fn monotone_x_passes_through_every_point() {
    assert_path(
        &line(Curve::MonotoneX, &PTS),
        "M0,0C0.333,1,0.667,2,1,2C1.333,2,1.667,1,2,1C2.333,1,2.667,2,3,3",
    );
}

#[test]
fn monotone_x_keeps_flat_runs_flat() {
    let points = [
        (0.0, 200.0),
        (100.0, 100.0),
        (200.0, 100.0),
        (300.0, 300.0),
        (400.0, 300.0),
    ];
    assert_path(
        &line(Curve::MonotoneX, &points),
        "M0,200C33.333333,150,66.666667,100,100,100C133.333333,100,166.666667,100,200,100\
         C233.333333,100,266.666667,300,300,300C333.333333,300,366.666667,300,400,300",
    );
}

#[test]
fn monotone_x_handles_duplicate_x_values() {
    let first = [(0.0, 200.0), (0.0, 100.0), (100.0, 100.0), (200.0, 0.0)];
    assert_path(
        &line(Curve::MonotoneX, &first),
        "M0,200C0,200,0,100,0,100C33.333333,100,66.666667,100,100,100\
         C133.333333,100,166.666667,50,200,0",
    );
    let middle = [(0.0, 200.0), (100.0, 100.0), (100.0, 0.0), (200.0, 0.0)];
    assert_path(
        &line(Curve::MonotoneX, &middle),
        "M0,200C33.333333,183.333333,66.666667,166.666667,100,100C100,100,100,0,100,0\
         C133.333333,0,166.666667,0,200,0",
    );
    let last = [(0.0, 200.0), (100.0, 100.0), (200.0, 100.0), (200.0, 0.0)];
    assert_path(
        &line(Curve::MonotoneX, &last),
        "M0,200C33.333333,150,66.666667,100,100,100C133.333333,100,166.666667,100,200,100\
         C200,100,200,0,200,0",
    );
}

#[test]
fn monotone_x_handles_segments_of_infinite_slope() {
    let down = [(0.0, 200.0), (100.0, 150.0), (100.0, 50.0), (200.0, 0.0)];
    assert_path(
        &line(Curve::MonotoneX, &down),
        "M0,200C33.333333,191.666667,66.666667,183.333333,100,150C100,150,100,50,100,50\
         C133.333333,16.666667,166.666667,8.333333,200,0",
    );
    let back = [(200.0, 0.0), (100.0, 50.0), (100.0, 150.0), (0.0, 200.0)];
    assert_path(
        &line(Curve::MonotoneX, &back),
        "M200,0C166.666667,8.333333,133.333333,16.666667,100,50C100,50,100,150,100,150\
         C66.666667,183.333333,33.333333,191.666667,0,200",
    );
}

#[test]
fn monotone_x_ignores_coincident_points() {
    let points = [
        (0.0, 200.0),
        (50.0, 200.0),
        (100.0, 100.0),
        (150.0, 0.0),
        (200.0, 0.0),
    ];
    let expected = line(Curve::MonotoneX, &points);
    for i in 0..points.len() {
        let mut doubled = points.to_vec();
        doubled.insert(i, points[i]);
        assert_eq!(line(Curve::MonotoneX, &doubled), expected);
    }
}

#[test]
fn monotone_y_passes_through_every_point() {
    assert_path(
        &line(Curve::MonotoneY, &PTS),
        "M0,0C0.5,0.667,1,1.333,1,2C1,1.667,2,1.333,2,1C2,1.667,2.5,2.333,3,3",
    );
}

#[test]
fn monotone_y_is_monotone_x_of_the_swapped_points_swapped_back() {
    let numbers = |d: &str| -> Vec<f64> {
        tokenize(d)
            .into_iter()
            .filter_map(|token| match token {
                Token::Num { value, .. } => Some(value),
                Token::Text(_) => None,
            })
            .collect()
    };
    let letters = |d: &str| -> String { d.chars().filter(char::is_ascii_alphabetic).collect() };
    let swapped: Vec<_> = PTS.iter().map(|&(x, y)| (y, x)).collect();
    let x = line(Curve::MonotoneX, &swapped);
    let y = line(Curve::MonotoneY, &PTS);
    let mut back = numbers(&x);
    back.chunks_mut(2).for_each(|pair| pair.swap(0, 1));
    assert_eq!(numbers(&y), back);
    assert_eq!(letters(&y), letters(&x));
}

#[test]
fn a_monotone_x_area_runs_back_along_its_baseline() {
    assert_path(
        &area(Curve::MonotoneX, &TOPS),
        "M0,4C0.333,3,0.667,2,1,2C1.333,2,1.667,3.5,2,5L2,0C1.667,0,1.333,0,1,0\
         C0.667,0,0.333,0,0,0Z",
    );
}

#[test]
fn natural_passes_through_every_point() {
    assert_path(
        &line(Curve::Natural, &PTS),
        "M0,0C0.333,1,0.667,2,1,2C1.333,2,1.667,1,2,1C2.333,1,2.667,2,3,3",
    );
    assert_path(
        &line(Curve::Natural, &ZIGZAG),
        "M0,1C0.333333,2.111111,0.666667,3.222222,1,3C1.333333,2.777778,1.666667,1.222222,2,1\
         C2.333333,0.777778,2.666667,1.888889,3,3",
    );
}

#[test]
fn natural_draws_one_or_two_points_as_a_closed_move_or_a_segment() {
    assert_path(&line(Curve::Natural, &ZIGZAG[..1]), "M0,1Z");
    assert_path(&line(Curve::Natural, &ZIGZAG[..2]), "M0,1L1,3");
    assert_path(&area(Curve::Natural, &ZIGZAG[..1]), "M0,1L0,0Z");
}
