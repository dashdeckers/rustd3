//! d3's tests of d3-path, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::path::{PathContext, SvgPath};

use super::support::{assert_path_eq, precision};

// d3-path/test/path-test.js: path is an instanceof path
#[test]
fn path_is_an_instanceof_path() {
    let p = SvgPath::new();
    assert_path_eq(&p.to_string(), &""); // d3-path/test/path-test.js:8:3
}

// d3-path/test/path-test.js: path.moveTo(x, y) appends an M command
#[test]
fn path_moveto_x_y_appends_an_m_command() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50"); // d3-path/test/path-test.js:13:3
    p.line_to(200.0, 100.0);
    assert_path_eq(&p.to_string(), &"M150,50L200,100"); // d3-path/test/path-test.js:15:3
    p.move_to(100.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50L200,100M100,50"); // d3-path/test/path-test.js:17:3
}

// d3-path/test/path-test.js: path.closePath() appends a Z command
#[test]
fn path_closepath_appends_a_z_command() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50"); // d3-path/test/path-test.js:22:3
    p.close_path();
    assert_path_eq(&p.to_string(), &"M150,50Z"); // d3-path/test/path-test.js:24:3
    p.close_path();
    assert_path_eq(&p.to_string(), &"M150,50ZZ"); // d3-path/test/path-test.js:26:3
}

// d3-path/test/path-test.js: path.closePath() does nothing if the path is empty
#[test]
fn path_closepath_does_nothing_if_the_path_is_empty() {
    let mut p = SvgPath::new();
    assert_path_eq(&p.to_string(), &""); // d3-path/test/path-test.js:31:3
    p.close_path();
    assert_path_eq(&p.to_string(), &""); // d3-path/test/path-test.js:33:3
}

// d3-path/test/path-test.js: path.lineTo(x, y) appends an L command
#[test]
fn path_lineto_x_y_appends_an_l_command() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50"); // d3-path/test/path-test.js:38:3
    p.line_to(200.0, 100.0);
    assert_path_eq(&p.to_string(), &"M150,50L200,100"); // d3-path/test/path-test.js:40:3
    p.line_to(100.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50L200,100L100,50"); // d3-path/test/path-test.js:42:3
}

// d3-path/test/path-test.js: path.quadraticCurveTo(x1, y1, x, y) appends a Q command
#[test]
fn path_quadraticcurveto_x1_y1_x_y_appends_a_q_command() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50"); // d3-path/test/path-test.js:47:3
    p.quadratic_curve_to(100.0, 50.0, 200.0, 100.0);
    assert_path_eq(&p.to_string(), &"M150,50Q100,50,200,100"); // d3-path/test/path-test.js:49:3
}

// d3-path/test/path-test.js: path.bezierCurveTo(x1, y1, x, y) appends a C command
#[test]
fn path_beziercurveto_x1_y1_x_y_appends_a_c_command() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 50.0);
    assert_path_eq(&p.to_string(), &"M150,50"); // d3-path/test/path-test.js:54:3
    p.bezier_curve_to(100.0, 50.0, 0.0, 24.0, 200.0, 100.0);
    assert_path_eq(&p.to_string(), &"M150,50C100,50,0,24,200,100"); // d3-path/test/path-test.js:56:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) throws an error if the radius is negative
#[test]
fn path_arc_x_y_radius_startangle_endangle_throws_an_error_if_the_radius_is_negative() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    assert!(
        p.try_arc(100.0, 100.0, -50.0, 0.0, std::f64::consts::PI / 2.0, false)
            .is_err()
    ); // d3-path/test/path-test.js:61:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append only an M command if the radius is zero
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_only_an_m_command_if_the_radius_is_zero() {
    let mut p = SvgPath::new();
    p.try_arc(100.0, 100.0, 0.0, 0.0, std::f64::consts::PI / 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,100"); // d3-path/test/path-test.js:66:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append only an L command if the radius is zero
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_only_an_l_command_if_the_radius_is_zero() {
    let mut p = SvgPath::new();
    p.move_to(0.0, 0.0);
    p.try_arc(100.0, 100.0, 0.0, 0.0, std::f64::consts::PI / 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M0,0L100,100"); // d3-path/test/path-test.js:71:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append only an M command if the angle is zero
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_only_an_m_command_if_the_angle_is_zero() {
    let mut p = SvgPath::new();
    p.try_arc(100.0, 100.0, 0.0, 0.0, 0.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,100"); // d3-path/test/path-test.js:76:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append only an M command if the angle is near zero
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_only_an_m_command_if_the_angle_is_near_zero()
{
    let mut p = SvgPath::new();
    p.try_arc(100.0, 100.0, 0.0, 0.0, 1e-16, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,100"); // d3-path/test/path-test.js:81:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append an M command if the path was empty
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_an_m_command_if_the_path_was_empty() {
    let mut p1 = SvgPath::new();
    p1.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI * 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p1.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:86:3
    let mut p2 = SvgPath::new();
    p2.try_arc(0.0, 50.0, 50.0, -std::f64::consts::PI / 2.0, 0.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p2.to_string(), &"M0,0A50,50,0,0,1,50,50"); // d3-path/test/path-test.js:88:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) may append an L command if the arc doesn’t start at the current point
#[test]
fn path_arc_x_y_radius_startangle_endangle_may_append_an_l_command_if_the_arc_doesn_t_start_at_the_current_point()
 {
    let mut p = SvgPath::new();
    p.move_to(100.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI * 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M100,100L150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:93:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) appends a single A command if the angle is less than π
#[test]
fn path_arc_x_y_radius_startangle_endangle_appends_a_single_a_command_if_the_angle_is_less_than() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI / 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,0,1,100,150"); // d3-path/test/path-test.js:98:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) appends a single A command if the angle is less than τ
#[test]
fn path_arc_x_y_radius_startangle_endangle_appends_a_single_a_command_if_the_angle_is_less_than_2()
{
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI * 1.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,1,1,50,100"); // d3-path/test/path-test.js:103:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, startAngle, endAngle) appends two A commands if the angle is greater than τ
#[test]
fn path_arc_x_y_radius_startangle_endangle_appends_two_a_commands_if_the_angle_is_greater_than() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI * 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:108:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, π/2, false) draws a small clockwise arc
#[test]
fn path_arc_x_y_radius_0_2_false_draws_a_small_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI / 2.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,0,1,100,150"); // d3-path/test/path-test.js:113:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, -π/2, 0, false) draws a small clockwise arc
#[test]
fn path_arc_x_y_radius_2_0_false_draws_a_small_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(100.0, 100.0, 50.0, -std::f64::consts::PI / 2.0, 0.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,0,1,150,100"); // d3-path/test/path-test.js:118:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, ε, true) draws an anticlockwise circle
#[test]
fn path_arc_x_y_radius_0_true_draws_an_anticlockwise_circle() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 1e-16, true)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,0,50,100A50,50,0,1,0,150,100",
    ); // d3-path/test/path-test.js:123:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, ε, false) draws nothing
#[test]
fn path_arc_x_y_radius_0_false_draws_nothing() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 1e-16, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100"); // d3-path/test/path-test.js:128:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, -ε, true) draws nothing
#[test]
fn path_arc_x_y_radius_0_true_draws_nothing() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, -1e-16, true)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100"); // d3-path/test/path-test.js:133:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, -ε, false) draws a clockwise circle
#[test]
fn path_arc_x_y_radius_0_false_draws_a_clockwise_circle() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, -1e-16, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:138:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, τ, true) draws an anticlockwise circle
#[test]
fn path_arc_x_y_radius_0_true_draws_an_anticlockwise_circle_2() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 2.0 * std::f64::consts::PI, true)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,0,50,100A50,50,0,1,0,150,100",
    ); // d3-path/test/path-test.js:143:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, τ, false) draws a clockwise circle
#[test]
fn path_arc_x_y_radius_0_false_draws_a_clockwise_circle_2() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 2.0 * std::f64::consts::PI, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:148:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, τ + ε, true) draws an anticlockwise circle
#[test]
fn path_arc_x_y_radius_0_true_draws_an_anticlockwise_circle_3() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        0.0,
        (2.0 * std::f64::consts::PI) + 1e-13,
        true,
    )
    .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,0,50,100A50,50,0,1,0,150,100",
    ); // d3-path/test/path-test.js:153:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, τ - ε, false) draws a clockwise circle
#[test]
fn path_arc_x_y_radius_0_false_draws_a_clockwise_circle_3() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        0.0,
        (2.0 * std::f64::consts::PI) - 1e-13,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:158:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, τ, 0, true) draws an anticlockwise circle
#[test]
fn path_arc_x_y_radius_0_true_draws_an_anticlockwise_circle_4() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 2.0 * std::f64::consts::PI, true)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,0,50,100A50,50,0,1,0,150,100",
    ); // d3-path/test/path-test.js:163:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, τ, 0, false) draws a clockwise circle
#[test]
fn path_arc_x_y_radius_0_false_draws_a_clockwise_circle_4() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, 2.0 * std::f64::consts::PI, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:168:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, 13π/2, false) draws a clockwise circle
#[test]
fn path_arc_x_y_radius_0_13_2_false_draws_a_clockwise_circle() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        0.0,
        (13.0 * std::f64::consts::PI) / 2.0,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100",
    ); // d3-path/test/path-test.js:173:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 13π/2, 0, false) draws a big clockwise arc
#[test]
fn path_arc_x_y_radius_13_2_0_false_draws_a_big_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 150.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        (13.0 * std::f64::consts::PI) / 2.0,
        0.0,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,150A50,50,0,1,1,150,100"); // d3-path/test/path-test.js:178:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, π/2, 0, false) draws a big clockwise arc
#[test]
fn path_arc_x_y_radius_2_0_false_draws_a_big_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 150.0);
    p.try_arc(100.0, 100.0, 50.0, std::f64::consts::PI / 2.0, 0.0, false)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,150A50,50,0,1,1,150,100"); // d3-path/test/path-test.js:183:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 3π/2, 0, false) draws a small clockwise arc
#[test]
fn path_arc_x_y_radius_3_2_0_false_draws_a_small_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        (3.0 * std::f64::consts::PI) / 2.0,
        0.0,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,0,1,150,100"); // d3-path/test/path-test.js:188:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 15π/2, 0, false) draws a small clockwise arc
#[test]
fn path_arc_x_y_radius_15_2_0_false_draws_a_small_clockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        (15.0 * std::f64::consts::PI) / 2.0,
        0.0,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,0,1,150,100"); // d3-path/test/path-test.js:193:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, π/2, true) draws a big anticlockwise arc
#[test]
fn path_arc_x_y_radius_0_2_true_draws_a_big_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(100.0, 100.0, 50.0, 0.0, std::f64::consts::PI / 2.0, true)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,1,0,100,150"); // d3-path/test/path-test.js:198:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, -π/2, 0, true) draws a big anticlockwise arc
#[test]
fn path_arc_x_y_radius_2_0_true_draws_a_big_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(100.0, 100.0, 50.0, -std::f64::consts::PI / 2.0, 0.0, true)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,1,0,150,100"); // d3-path/test/path-test.js:203:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, -13π/2, 0, true) draws a big anticlockwise arc
#[test]
fn path_arc_x_y_radius_13_2_0_true_draws_a_big_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        (-13.0 * std::f64::consts::PI) / 2.0,
        0.0,
        true,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,1,0,150,100"); // d3-path/test/path-test.js:208:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, -13π/2, 0, false) draws a big clockwise arc
#[test]
fn path_arc_x_y_radius_13_2_0_false_draws_a_big_clockwise_arc_2() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        0.0,
        (-13.0 * std::f64::consts::PI) / 2.0,
        false,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,1,1,100,50"); // d3-path/test/path-test.js:213:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 0, 13π/2, true) draws a big anticlockwise arc
#[test]
fn path_arc_x_y_radius_0_13_2_true_draws_a_big_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        0.0,
        (13.0 * std::f64::consts::PI) / 2.0,
        true,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M150,100A50,50,0,1,0,100,150"); // d3-path/test/path-test.js:218:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, π/2, 0, true) draws a small anticlockwise arc
#[test]
fn path_arc_x_y_radius_2_0_true_draws_a_small_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 150.0);
    p.try_arc(100.0, 100.0, 50.0, std::f64::consts::PI / 2.0, 0.0, true)
        .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,150A50,50,0,0,0,150,100"); // d3-path/test/path-test.js:223:3
}

// d3-path/test/path-test.js: path.arc(x, y, radius, 3π/2, 0, true) draws a big anticlockwise arc
#[test]
fn path_arc_x_y_radius_3_2_0_true_draws_a_big_anticlockwise_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.try_arc(
        100.0,
        100.0,
        50.0,
        (3.0 * std::f64::consts::PI) / 2.0,
        0.0,
        true,
    )
    .expect("d3 test arc");
    assert_path_eq(&p.to_string(), &"M100,50A50,50,0,1,0,150,100"); // d3-path/test/path-test.js:228:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) throws an error if the radius is negative
#[test]
fn path_arcto_x1_y1_x2_y2_radius_throws_an_error_if_the_radius_is_negative() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    assert!(p.arc_to(270.0, 39.0, 163.0, 100.0, -53.0).is_err()); // d3-path/test/path-test.js:247:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends an M command if the path was empty
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_an_m_command_if_the_path_was_empty() {
    let mut p = SvgPath::new();
    p.arc_to(270.0, 39.0, 163.0, 100.0, 53.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M270,39"); // d3-path/test/path-test.js:252:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) does nothing if the previous point was ⟨x1,y1⟩
#[test]
fn path_arcto_x1_y1_x2_y2_radius_does_nothing_if_the_previous_point_was_x1_y1() {
    let mut p = SvgPath::new();
    p.move_to(270.0, 39.0);
    p.arc_to(270.0, 39.0, 163.0, 100.0, 53.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M270,39"); // d3-path/test/path-test.js:257:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends an L command if the previous point, ⟨x1,y1⟩ and ⟨x2,y2⟩ are collinear
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_an_l_command_if_the_previous_point_x1_y1_and_x2_y2_are_collinear()
 {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.arc_to(101.0, 51.0, 102.0, 52.0, 10.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M100,50L101,51"); // d3-path/test/path-test.js:262:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends an L command if ⟨x1,y1⟩ and ⟨x2,y2⟩ are coincident
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_an_l_command_if_x1_y1_and_x2_y2_are_coincident() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 50.0);
    p.arc_to(101.0, 51.0, 101.0, 51.0, 10.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M100,50L101,51"); // d3-path/test/path-test.js:267:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends an L command if the radius is zero
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_an_l_command_if_the_radius_is_zero() {
    let mut p = SvgPath::new();
    p.move_to(270.0, 182.0);
    p.arc_to(270.0, 39.0, 163.0, 100.0, 0.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M270,182L270,39"); // d3-path/test/path-test.js:272:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends L and A commands if the arc does not start at the current point
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_l_and_a_commands_if_the_arc_does_not_start_at_the_current_point()
 {
    let mut p1 = SvgPath::new();
    p1.move_to(270.0, 182.0);
    p1.arc_to(270.0, 39.0, 163.0, 100.0, 53.0)
        .expect("d3 test arcTo");
    assert_path_eq(
        &p1.to_string(),
        &"M270,182L270,130.222686A53,53,0,0,0,190.750991,84.179342",
    ); // d3-path/test/path-test.js:277:3
    let mut p2 = SvgPath::new();
    p2.move_to(270.0, 182.0);
    p2.arc_to(270.0, 39.0, 363.0, 100.0, 53.0)
        .expect("d3 test arcTo");
    assert_path_eq(
        &p2.to_string(),
        &"M270,182L270,137.147168A53,53,0,0,1,352.068382,92.829799",
    ); // d3-path/test/path-test.js:279:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) appends only an A command if the arc starts at the current point
#[test]
fn path_arcto_x1_y1_x2_y2_radius_appends_only_an_a_command_if_the_arc_starts_at_the_current_point()
{
    let mut p = SvgPath::new();
    p.move_to(100.0, 100.0);
    p.arc_to(200.0, 100.0, 200.0, 200.0, 100.0)
        .expect("d3 test arcTo");
    assert_path_eq(&p.to_string(), &"M100,100A100,100,0,0,1,200,200"); // d3-path/test/path-test.js:284:3
}

// d3-path/test/path-test.js: path.arcTo(x1, y1, x2, y2, radius) sets the last point to be the end tangent of the arc
#[test]
fn path_arcto_x1_y1_x2_y2_radius_sets_the_last_point_to_be_the_end_tangent_of_the_arc() {
    let mut p = SvgPath::new();
    p.move_to(100.0, 100.0);
    p.arc_to(200.0, 100.0, 200.0, 200.0, 50.0)
        .expect("d3 test arcTo");
    p.try_arc(150.0, 150.0, 50.0, 0.0, std::f64::consts::PI, false)
        .expect("d3 test arc");
    assert_path_eq(
        &p.to_string(),
        &"M100,100L150,100A50,50,0,0,1,200,150A50,50,0,1,1,100,150",
    ); // d3-path/test/path-test.js:289:3
}

// d3-path/test/path-test.js: path.rect(x, y, w, h) appends M, h, v, h, and Z commands
#[test]
fn path_rect_x_y_w_h_appends_m_h_v_h_and_z_commands() {
    let mut p = SvgPath::new();
    p.move_to(150.0, 100.0);
    p.rect(100.0, 200.0, 50.0, 25.0);
    assert_path_eq(&p.to_string(), &"M150,100M100,200h50v25h-50Z"); // d3-path/test/path-test.js:294:3
}

// d3-path/test/pathRound-test.js: pathRound() defaults to three digits of precision
#[test]
fn pathround_defaults_to_three_digits_of_precision() {
    let mut p = SvgPath::with_digits(3);
    p.move_to(std::f64::consts::PI, std::f64::consts::E);
    assert_eq!(p.to_string(), "M3.142,2.718"); // d3-path/test/pathRound-test.js:7:3
}

// d3-path/test/pathRound-test.js: pathRound(null) is equivalent to pathRound(0)
#[test]
fn pathround_null_is_equivalent_to_pathround_0() {
    let mut p = SvgPath::with_digits(0);
    p.move_to(std::f64::consts::PI, std::f64::consts::E);
    assert_eq!(p.to_string(), "M3,3"); // d3-path/test/pathRound-test.js:13:3
}

// d3-path/test/pathRound-test.js: pathRound(digits) ignores digits if greater than 15
#[test]
fn pathround_digits_ignores_digits_if_greater_than_15() {
    let mut p = SvgPath::with_digits(40);
    p.move_to(std::f64::consts::PI, std::f64::consts::E);
    assert_eq!(p.to_string(), "M3.141592653589793,2.718281828459045"); // d3-path/test/pathRound-test.js:24:3
}

// d3-path/test/pathRound-test.js: pathRound.moveTo(x, y) limits the precision
#[test]
fn pathround_moveto_x_y_limits_the_precision() {
    let mut p = SvgPath::with_digits(1);
    p.move_to(123.456, 789.012);
    assert_eq!(p.to_string(), "M123.5,789"); // d3-path/test/pathRound-test.js:30:3
}

// d3-path/test/pathRound-test.js: pathRound.lineTo(x, y) limits the precision
#[test]
fn pathround_lineto_x_y_limits_the_precision() {
    let mut p = SvgPath::with_digits(1);
    p.move_to(0.0, 0.0);
    p.line_to(123.456, 789.012);
    assert_eq!(p.to_string(), "M0,0L123.5,789"); // d3-path/test/pathRound-test.js:37:3
}

// d3-path/test/pathRound-test.js: pathRound.arc(x, y, r, a0, a1, ccw) limits the precision
#[test]
fn pathround_arc_x_y_r_a0_a1_ccw_limits_the_precision() {
    let mut p0 = SvgPath::new();
    let mut p = SvgPath::with_digits(1);
    p0.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI + 0.0001,
        false,
    )
    .expect("d3 test arc");
    p.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI + 0.0001,
        false,
    )
    .expect("d3 test arc");
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:44:3
    p0.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI - 0.0001,
        false,
    )
    .expect("d3 test arc");
    p.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI - 0.0001,
        false,
    )
    .expect("d3 test arc");
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:47:3
    p0.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI / 2.0,
        true,
    )
    .expect("d3 test arc");
    p.try_arc(
        10.0001,
        10.0001,
        123.456,
        0.0,
        std::f64::consts::PI / 2.0,
        true,
    )
    .expect("d3 test arc");
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:50:3
}

// d3-path/test/pathRound-test.js: pathRound.arcTo(x1, y1, x2, y2, r) limits the precision
#[test]
fn pathround_arcto_x1_y1_x2_y2_r_limits_the_precision() {
    let mut p0 = SvgPath::new();
    let mut p = SvgPath::with_digits(1);
    p0.arc_to(10.0001, 10.0001, 123.456, 456.789, 12345.6789)
        .expect("d3 test arcTo");
    p.arc_to(10.0001, 10.0001, 123.456, 456.789, 12345.6789)
        .expect("d3 test arcTo");
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:57:3
}

// d3-path/test/pathRound-test.js: pathRound.quadraticCurveTo(x1, y1, x, y) limits the precision
#[test]
fn pathround_quadraticcurveto_x1_y1_x_y_limits_the_precision() {
    let mut p0 = SvgPath::new();
    let mut p = SvgPath::with_digits(1);
    p0.quadratic_curve_to(10.0001, 10.0001, 123.456, 456.789);
    p.quadratic_curve_to(10.0001, 10.0001, 123.456, 456.789);
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:64:3
}

// d3-path/test/pathRound-test.js: pathRound.bezierCurveTo(x1, y1, x2, y2, x, y) limits the precision
#[test]
fn pathround_beziercurveto_x1_y1_x2_y2_x_y_limits_the_precision() {
    let mut p0 = SvgPath::new();
    let mut p = SvgPath::with_digits(1);
    p0.bezier_curve_to(10.0001, 10.0001, 123.456, 456.789, 0.007, 0.006);
    p.bezier_curve_to(10.0001, 10.0001, 123.456, 456.789, 0.007, 0.006);
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:71:3
}

// d3-path/test/pathRound-test.js: pathRound.rect(x, y, w, h) limits the precision
#[test]
fn pathround_rect_x_y_w_h_limits_the_precision() {
    let mut p0 = SvgPath::new();
    let mut p = SvgPath::with_digits(1);
    p0.rect(10.0001, 10.0001, 123.456, 456.789);
    p.rect(10.0001, 10.0001, 123.456, 456.789);
    assert_eq!(p.to_string(), precision(&p0.to_string(), 1.0)); // d3-path/test/pathRound-test.js:78:3
}
