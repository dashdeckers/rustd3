//! d3's tests of d3-zoom, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::interact;

use super::support::assert_d3_eq;

// d3-zoom/test/transform-test.js: zoomIdentity transform contains k = 1, x = y = 0
#[test]
fn zoomidentity_transform_contains_k_1_x_y_0() {
    assert_eq!(
        interact::Transform::IDENTITY,
        interact::Transform {
            k: 1.0,
            x: 0.0,
            y: 0.0
        }
    ); // d3-zoom/test/transform-test.js:5:3
}

// d3-zoom/test/transform-test.js: transform.scale(k) returns a new transform scaled with k
#[test]
fn transform_scale_k_returns_a_new_transform_scaled_with_k() {
    let transform = interact::Transform::IDENTITY.scale(2.5);
    assert_eq!(
        transform.scale(2.0),
        interact::Transform {
            k: 5.0,
            x: 0.0,
            y: 0.0
        }
    ); // d3-zoom/test/transform-test.js:10:3
}

// d3-zoom/test/transform-test.js: transform.translate(x, y) returns a new transform translated with x and y
#[test]
fn transform_translate_x_y_returns_a_new_transform_translated_with_x_and_y() {
    let transform = interact::Transform::IDENTITY.translate(2.0, 3.0);
    assert_eq!(
        transform.translate(-4.0, 4.0),
        interact::Transform {
            k: 1.0,
            x: -2.0,
            y: 7.0
        }
    ); // d3-zoom/test/transform-test.js:15:3
    assert_eq!(
        transform.scale(2.0).translate(-4.0, 4.0),
        interact::Transform {
            k: 2.0,
            x: -6.0,
            y: 11.0
        }
    ); // d3-zoom/test/transform-test.js:16:3
}

// d3-zoom/test/transform-test.js: transform.apply([x, y]) returns the transformation of the specified point
#[test]
fn transform_apply_x_y_returns_the_transformation_of_the_specified_point() {
    assert_d3_eq(
        &interact::Transform::IDENTITY
            .translate(2.0, 3.0)
            .scale(2.0)
            .apply((4.0, 5.0)),
        &vec![10.0, 13.0],
    ); // d3-zoom/test/transform-test.js:20:3
}

// d3-zoom/test/transform-test.js: transform.applyX(x) returns the transformation of the specified x-coordinate
#[test]
fn transform_applyx_x_returns_the_transformation_of_the_specified_x_coordinate() {
    assert_eq!(
        interact::Transform::IDENTITY
            .translate(2.0, 0.0)
            .scale(2.0)
            .apply_x(4.0),
        10.0
    ); // d3-zoom/test/transform-test.js:24:3
}

// d3-zoom/test/transform-test.js: transform.applyY(y) returns the transformation of the specified y-coordinate
#[test]
fn transform_applyy_y_returns_the_transformation_of_the_specified_y_coordinate() {
    assert_eq!(
        interact::Transform::IDENTITY
            .translate(0.0, 3.0)
            .scale(2.0)
            .apply_y(5.0),
        13.0
    ); // d3-zoom/test/transform-test.js:28:3
}

// d3-zoom/test/transform-test.js: transform.invert([x, y]) returns the inverse transformation of the specified point
#[test]
fn transform_invert_x_y_returns_the_inverse_transformation_of_the_specified_point() {
    assert_d3_eq(
        &interact::Transform::IDENTITY
            .translate(2.0, 3.0)
            .scale(2.0)
            .invert((4.0, 5.0)),
        &vec![1.0, 1.0],
    ); // d3-zoom/test/transform-test.js:32:3
}

// d3-zoom/test/transform-test.js: transform.invertX(x) returns the inverse transformation of the specified x-coordinate
#[test]
fn transform_invertx_x_returns_the_inverse_transformation_of_the_specified_x_coordinate() {
    assert_eq!(
        interact::Transform::IDENTITY
            .translate(2.0, 0.0)
            .scale(2.0)
            .invert_x(4.0),
        1.0
    ); // d3-zoom/test/transform-test.js:36:3
}

// d3-zoom/test/transform-test.js: transform.invertY(y) returns the inverse transformation of the specified y-coordinate
#[test]
fn transform_inverty_y_returns_the_inverse_transformation_of_the_specified_y_coordinate() {
    assert_eq!(
        interact::Transform::IDENTITY
            .translate(0.0, 3.0)
            .scale(2.0)
            .invert_y(5.0),
        1.0
    ); // d3-zoom/test/transform-test.js:40:3
}

// d3-zoom/test/transform-test.js: transform.toString() returns a string representing the SVG transform
#[test]
fn transform_tostring_returns_a_string_representing_the_svg_transform() {
    assert_eq!(
        interact::Transform::IDENTITY.to_string(),
        "translate(0,0) scale(1)"
    ); // d3-zoom/test/transform-test.js:48:3
}
