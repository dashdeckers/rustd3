//! d3's tests of d3-shape, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::shape::{self, Curve};

use super::support::{assert_d3_eq, assert_path_eq, drawn};

// d3-shape/test/curve/basis-test.js: line.curve(curveBasis)(data) generates the expected path
#[test]
fn line_curve_curvebasis_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::Basis);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/basis-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/basis-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/basis-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1L0.166667,1.333333C0.333333,1.666667,0.666667,2.333333,1,2.333333C1.333333,2.333333,1.666667,1.666667,1.833333,1.333333L2,1",
    ); // d3-shape/test/curve/basis-test.js:10:3
}

// d3-shape/test/curve/basis-test.js: area.curve(curveBasis)(data) generates the expected path
#[test]
fn area_curve_curvebasis_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::Basis);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/basis-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/basis-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/curve/basis-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1L0.166667,1.333333C0.333333,1.666667,0.666667,2.333333,1,2.333333C1.333333,2.333333,1.666667,1.666667,1.833333,1.333333L2,1L2,0L1.833333,0C1.666667,0,1.333333,0,1,0C0.666667,0,0.333333,0,0.166667,0L0,0Z",
    ); // d3-shape/test/curve/basis-test.js:18:3
}

// d3-shape/test/curve/basisClosed-test.js: line.curve(curveBasisClosed)(data) generates the expected path
#[test]
fn line_curve_curvebasisclosed_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::BasisClosed);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/basisClosed-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 0.0)], ctx)), &"M0,0Z"); // d3-shape/test/curve/basisClosed-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 0.0), (0.0, 10.0)], ctx)),
        &"M0,6.666667L0,3.333333Z",
    ); // d3-shape/test/curve/basisClosed-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0)], ctx)),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,6.666667,8.333333C6.666667,6.666667,3.333333,3.333333,1.666667,3.333333C0,3.333333,0,6.666667,1.666667,8.333333",
    ); // d3-shape/test/curve/basisClosed-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0), (10.0, 0.0)],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333C10,6.666667,10,3.333333,8.333333,1.666667C6.666667,0,3.333333,0,1.666667,1.666667C0,3.333333,0,6.666667,1.666667,8.333333",
    ); // d3-shape/test/curve/basisClosed-test.js:11:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![
                    (0.0, 0.0),
                    (0.0, 10.0),
                    (10.0, 10.0),
                    (10.0, 0.0),
                    (0.0, 0.0),
                ],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333C10,6.666667,10,3.333333,8.333333,1.666667C6.666667,0,3.333333,0,1.666667,0C0,0,0,0,0,1.666667C0,3.333333,0,6.666667,1.666667,8.333333",
    ); // d3-shape/test/curve/basisClosed-test.js:12:3
}

// d3-shape/test/curve/basisOpen-test.js: line.curve(curveBasisOpen)(data) generates the expected path
#[test]
fn line_curve_curvebasisopen_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::BasisOpen);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/basisOpen-test.js:7:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 0.0)], ctx)), None); // d3-shape/test/curve/basisOpen-test.js:8:3
    assert_eq!(
        drawn(|ctx| l.draw(vec![(0.0, 0.0), (0.0, 10.0)], ctx)),
        None
    ); // d3-shape/test/curve/basisOpen-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0)], ctx)),
        &"M1.666667,8.333333Z",
    ); // d3-shape/test/curve/basisOpen-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0), (10.0, 0.0)],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333",
    ); // d3-shape/test/curve/basisOpen-test.js:11:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![
                    (0.0, 0.0),
                    (0.0, 10.0),
                    (10.0, 10.0),
                    (10.0, 0.0),
                    (0.0, 0.0),
                ],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333C10,6.666667,10,3.333333,8.333333,1.666667",
    ); // d3-shape/test/curve/basisOpen-test.js:12:3
}

// d3-shape/test/curve/basisOpen-test.js: area.curve(curveBasisOpen)(data) generates the expected path
#[test]
fn area_curve_curvebasisopen_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::BasisOpen);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/basisOpen-test.js:17:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/basisOpen-test.js:18:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/basisOpen-test.js:19:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0)], ctx)),
        &"M1.666667,8.333333L1.666667,0Z",
    ); // d3-shape/test/curve/basisOpen-test.js:20:3
    assert_path_eq(
        &drawn(|ctx| {
            a.draw(
                vec![(0.0, 0.0), (0.0, 10.0), (10.0, 10.0), (10.0, 0.0)],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333L8.333333,0C6.666667,0,3.333333,0,1.666667,0Z",
    ); // d3-shape/test/curve/basisOpen-test.js:21:3
    assert_path_eq(
        &drawn(|ctx| {
            a.draw(
                vec![
                    (0.0, 0.0),
                    (0.0, 10.0),
                    (10.0, 10.0),
                    (10.0, 0.0),
                    (0.0, 0.0),
                ],
                ctx,
            )
        }),
        &"M1.666667,8.333333C3.333333,10,6.666667,10,8.333333,8.333333C10,6.666667,10,3.333333,8.333333,1.666667L8.333333,0C10,0,10,0,8.333333,0C6.666667,0,3.333333,0,1.666667,0Z",
    ); // d3-shape/test/curve/basisOpen-test.js:22:3
}

// d3-shape/test/curve/bumpX-test.js: line.curve(curveBumpX)(data) generates the expected path
#[test]
fn line_curve_curvebumpx_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::BumpX);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/bumpX-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/bumpX-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1C0.500000,1,0.500000,3,1,3",
    ); // d3-shape/test/curve/bumpX-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.500000,1,0.500000,3,1,3C1.500000,3,1.500000,1,2,1",
    ); // d3-shape/test/curve/bumpX-test.js:10:3
}

// d3-shape/test/curve/bumpX-test.js: area.curve(curveBumpX)(data) generates the expected path
#[test]
fn area_curve_curvebumpx_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::BumpX);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/bumpX-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/bumpX-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1C0.500000,1,0.500000,3,1,3L1,0C0.500000,0,0.500000,0,0,0Z",
    ); // d3-shape/test/curve/bumpX-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.500000,1,0.500000,3,1,3C1.500000,3,1.500000,1,2,1L2,0C1.500000,0,1.500000,0,1,0C0.500000,0,0.500000,0,0,0Z",
    ); // d3-shape/test/curve/bumpX-test.js:18:3
}

// d3-shape/test/curve/bumpY-test.js: line.curve(curveBumpY)(data) generates the expected path
#[test]
fn line_curve_curvebumpy_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::BumpY);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/bumpY-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/bumpY-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1C0,2,1,2,1,3",
    ); // d3-shape/test/curve/bumpY-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,2,1,2,1,3C1,2,2,2,2,1",
    ); // d3-shape/test/curve/bumpY-test.js:10:3
}

// d3-shape/test/curve/bumpY-test.js: area.curve(curveBumpY)(data) generates the expected path
#[test]
fn area_curve_curvebumpy_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::BumpY);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/bumpY-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/bumpY-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1C0,2,1,2,1,3L1,0C1,0,0,0,0,0Z",
    ); // d3-shape/test/curve/bumpY-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,2,1,2,1,3C1,2,2,2,2,1L2,0C2,0,1,0,1,0C1,0,0,0,0,0Z",
    ); // d3-shape/test/curve/bumpY-test.js:18:3
}

// d3-shape/test/curve/bundle-test.js: line.curve(curveBundle) uses a default beta of 0.85
#[test]
fn line_curve_curvebundle_uses_a_default_beta_of_0_85() {
    let l = shape::line().curve(Curve::Bundle { beta: 0.85 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::Bundle { beta: 0.85 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/bundle-test.js:7:3
}

// d3-shape/test/curve/bundle-test.js: line.curve(curveBundle.beta(beta)) uses the specified beta
#[test]
fn line_curve_curvebundle_beta_beta_uses_the_specified_beta() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::line()
                .curve(Curve::Bundle { beta: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M0,1L0.166667,1.222222C0.333333,1.444444,0.666667,1.888889,1,2C1.333333,2.111111,1.666667,1.888889,2,2C2.333333,2.111111,2.666667,2.555556,2.833333,2.777778L3,3",
    ); // d3-shape/test/curve/bundle-test.js:11:3
}

// d3-shape/test/curve/cardinal-test.js: line.curve(curveCardinal)(data) generates the expected path
#[test]
fn line_curve_curvecardinal_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::Cardinal { tension: 0.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinal-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/cardinal-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/cardinal-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1",
    ); // d3-shape/test/curve/cardinal-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3",
    ); // d3-shape/test/curve/cardinal-test.js:11:3
}

// d3-shape/test/curve/cardinal-test.js: line.curve(curveCardinal) uses a default tension of zero
#[test]
fn line_curve_curvecardinal_uses_a_default_tension_of_zero() {
    let l = shape::line().curve(Curve::Cardinal { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::Cardinal { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinal-test.js:16:3
}

// d3-shape/test/curve/cardinal-test.js: line.curve(curveCardinal.tension(tension)) uses the specified tension
#[test]
fn line_curve_curvecardinal_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::line()
                .curve(Curve::Cardinal { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M0,1C0,1,0.833333,3,1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3,3,3,3",
    ); // d3-shape/test/curve/cardinal-test.js:20:3
}

// d3-shape/test/curve/cardinal-test.js: area.curve(curveCardinal)(data) generates the expected path
#[test]
fn area_curve_curvecardinal_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::Cardinal { tension: 0.0 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinal-test.js:30:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/cardinal-test.js:31:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/curve/cardinal-test.js:32:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1L2,0C2,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z",
    ); // d3-shape/test/curve/cardinal-test.js:33:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3L3,0C3,0,2.333333,0,2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z",
    ); // d3-shape/test/curve/cardinal-test.js:34:3
}

// d3-shape/test/curve/cardinal-test.js: area.curve(curveCardinal) uses a default tension of zero
#[test]
fn area_curve_curvecardinal_uses_a_default_tension_of_zero() {
    let a = shape::area().curve(Curve::Cardinal { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::area()
            .curve(Curve::Cardinal { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinal-test.js:39:3
}

// d3-shape/test/curve/cardinal-test.js: area.curve(curveCardinal.tension(tension)) uses the specified tension
#[test]
fn area_curve_curvecardinal_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::area()
                .curve(Curve::Cardinal { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M0,1C0,1,0.833333,3,1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3,3,3,3L3,0C3,0,2.166667,0,2,0C1.833333,0,1.166667,0,1,0C0.833333,0,0,0,0,0Z",
    ); // d3-shape/test/curve/cardinal-test.js:43:3
}

// d3-shape/test/curve/cardinalClosed-test.js: line.curve(curveCardinalClosed)(data) generates the expected path
#[test]
fn line_curve_curvecardinalclosed_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CardinalClosed { tension: 0.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinalClosed-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/cardinalClosed-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M1,3L0,1Z",
    ); // d3-shape/test/curve/cardinalClosed-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3C1.333333,3,2.166667,1.333333,2,1C1.833333,0.666667,0.166667,0.666667,0,1C-0.166667,1.333333,0.666667,3,1,3",
    ); // d3-shape/test/curve/cardinalClosed-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3.333333,3,3,3C2.666667,3,0.333333,1,0,1C-0.333333,1,0.666667,3,1,3",
    ); // d3-shape/test/curve/cardinalClosed-test.js:11:3
}

// d3-shape/test/curve/cardinalClosed-test.js: line.curve(curveCardinalClosed) uses a default tension of zero
#[test]
fn line_curve_curvecardinalclosed_uses_a_default_tension_of_zero() {
    let l = shape::line().curve(Curve::CardinalClosed { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::CardinalClosed { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinalClosed-test.js:16:3
}

// d3-shape/test/curve/cardinalClosed-test.js: line.curve(curveCardinalClosed.tension(tension)) uses the specified tension
#[test]
fn line_curve_curvecardinalclosed_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::line()
                .curve(Curve::CardinalClosed { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3.166667,3,3,3C2.833333,3,0.166667,1,0,1C-0.166667,1,0.833333,3,1,3",
    ); // d3-shape/test/curve/cardinalClosed-test.js:20:3
}

// d3-shape/test/curve/cardinalClosed-test.js: area.curve(curveCardinalClosed)(data) generates the expected path
#[test]
fn area_curve_curvecardinalclosed_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::CardinalClosed { tension: 0.0 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinalClosed-test.js:30:3
    assert_d3_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1ZM0,0Z"); // d3-shape/test/curve/cardinalClosed-test.js:31:3
    assert_d3_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M1,3L0,1ZM0,0L1,0Z",
    ); // d3-shape/test/curve/cardinalClosed-test.js:32:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3C1.333333,3,2.166667,1.333333,2,1C1.833333,0.666667,0.166667,0.666667,0,1C-0.166667,1.333333,0.666667,3,1,3M1,0C0.666667,0,-0.166667,0,0,0C0.166667,0,1.833333,0,2,0C2.166667,0,1.333333,0,1,0",
    ); // d3-shape/test/curve/cardinalClosed-test.js:33:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3.333333,3,3,3C2.666667,3,0.333333,1,0,1C-0.333333,1,0.666667,3,1,3M2,0C1.666667,0,1.333333,0,1,0C0.666667,0,-0.333333,0,0,0C0.333333,0,2.666667,0,3,0C3.333333,0,2.333333,0,2,0",
    ); // d3-shape/test/curve/cardinalClosed-test.js:34:3
}

// d3-shape/test/curve/cardinalClosed-test.js: area.curve(curveCardinalClosed) uses a default tension of zero
#[test]
fn area_curve_curvecardinalclosed_uses_a_default_tension_of_zero() {
    let a = shape::area().curve(Curve::CardinalClosed { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::area()
            .curve(Curve::CardinalClosed { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinalClosed-test.js:39:3
}

// d3-shape/test/curve/cardinalClosed-test.js: area.curve(curveCardinalClosed.tension(tension)) uses the specified tension
#[test]
fn area_curve_curvecardinalclosed_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::area()
                .curve(Curve::CardinalClosed { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3.166667,3,3,3C2.833333,3,0.166667,1,0,1C-0.166667,1,0.833333,3,1,3M2,0C1.833333,0,1.166667,0,1,0C0.833333,0,-0.166667,0,0,0C0.166667,0,2.833333,0,3,0C3.166667,0,2.166667,0,2,0",
    ); // d3-shape/test/curve/cardinalClosed-test.js:43:3
}

// d3-shape/test/curve/cardinalOpen-test.js: line.curve(curveCardinalOpen)(data) generates the expected path
#[test]
fn line_curve_curvecardinalopen_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CardinalOpen { tension: 0.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:7:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:8:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3Z",
    ); // d3-shape/test/curve/cardinalOpen-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1",
    ); // d3-shape/test/curve/cardinalOpen-test.js:11:3
}

// d3-shape/test/curve/cardinalOpen-test.js: line.curve(curveCardinalOpen) uses a default tension of zero
#[test]
fn line_curve_curvecardinalopen_uses_a_default_tension_of_zero() {
    let l = shape::line().curve(Curve::CardinalOpen { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::CardinalOpen { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinalOpen-test.js:16:3
}

// d3-shape/test/curve/cardinalOpen-test.js: line.curve(curveCardinalOpen.tension(tension)) uses the specified tension
#[test]
fn line_curve_curvecardinalopen_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::line()
                .curve(Curve::CardinalOpen { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M1,3C1.166667,3,1.833333,1,2,1",
    ); // d3-shape/test/curve/cardinalOpen-test.js:20:3
}

// d3-shape/test/curve/cardinalOpen-test.js: area.curve(curveCardinalOpen)(data) generates the expected path
#[test]
fn area_curve_curvecardinalopen_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::CardinalOpen { tension: 0.0 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:30:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:31:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/cardinalOpen-test.js:32:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3L1,0Z",
    ); // d3-shape/test/curve/cardinalOpen-test.js:33:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1L2,0C1.666667,0,1.333333,0,1,0Z",
    ); // d3-shape/test/curve/cardinalOpen-test.js:34:3
}

// d3-shape/test/curve/cardinalOpen-test.js: area.curve(curveCardinalOpen) uses a default tension of zero
#[test]
fn area_curve_curvecardinalopen_uses_a_default_tension_of_zero() {
    let a = shape::area().curve(Curve::CardinalOpen { tension: 0.0 });
    assert_eq!(
        drawn(|ctx| shape::area()
            .curve(Curve::CardinalOpen { tension: 0.0 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/cardinalOpen-test.js:39:3
}

// d3-shape/test/curve/cardinalOpen-test.js: area.curve(curveCardinalOpen.tension(tension)) uses the specified tension
#[test]
fn area_curve_curvecardinalopen_tension_tension_uses_the_specified_tension() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::area()
                .curve(Curve::CardinalOpen { tension: 0.5 })
                .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)
        }),
        &"M1,3C1.166667,3,1.833333,1,2,1L2,0C1.833333,0,1.166667,0,1,0Z",
    ); // d3-shape/test/curve/cardinalOpen-test.js:43:3
}

// d3-shape/test/curve/catmullRom-test.js: line.curve(curveCatmullRom)(data) generates the expected path
#[test]
fn line_curve_curvecatmullrom_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRom { alpha: 0.5 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRom-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/catmullRom-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/catmullRom-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1",
    ); // d3-shape/test/curve/catmullRom-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3",
    ); // d3-shape/test/curve/catmullRom-test.js:11:3
}

// d3-shape/test/curve/catmullRom-test.js: line.curve(curveCatmullRom.alpha(1))(data) generates the expected path
#[test]
fn line_curve_curvecatmullrom_alpha_1_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRom { alpha: 1.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRom-test.js:16:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/catmullRom-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/catmullRom-test.js:18:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1",
    ); // d3-shape/test/curve/catmullRom-test.js:19:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3",
    ); // d3-shape/test/curve/catmullRom-test.js:20:3
}

// d3-shape/test/curve/catmullRom-test.js: line.curve(curveCatmullRom) uses a default alpha of 0.5 (centripetal)
#[test]
fn line_curve_curvecatmullrom_uses_a_default_alpha_of_0_5_centripetal() {
    let l = shape::line().curve(Curve::CatmullRom { alpha: 0.5 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::CatmullRom { alpha: 0.5 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/catmullRom-test.js:25:3
}

// d3-shape/test/curve/catmullRom-test.js: area.curve(curveCatmullRom.alpha(0))(data) generates the expected path
#[test]
fn area_curve_curvecatmullrom_alpha_0_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::CatmullRom { alpha: 0.0 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRom-test.js:35:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/catmullRom-test.js:36:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/curve/catmullRom-test.js:37:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1L2,0C2,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z",
    ); // d3-shape/test/curve/catmullRom-test.js:38:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3L3,0C3,0,2.333333,0,2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z",
    ); // d3-shape/test/curve/catmullRom-test.js:39:3
}

// d3-shape/test/curve/catmullRom-test.js: area.curve(curveCatmullRom) uses a default alpha of 0.5 (centripetal)
#[test]
fn area_curve_curvecatmullrom_uses_a_default_alpha_of_0_5_centripetal() {
    let a = shape::area().curve(Curve::CatmullRom { alpha: 0.5 });
    assert_eq!(
        drawn(|ctx| shape::area()
            .curve(Curve::CatmullRom { alpha: 0.5 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/catmullRom-test.js:44:3
}

// d3-shape/test/curve/catmullRomClosed-test.js: line.curve(curveCatmullRomClosed)(data) generates the expected path
#[test]
fn line_curve_curvecatmullromclosed_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRomClosed { alpha: 0.5 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomClosed-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/catmullRomClosed-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M1,3L0,1Z",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3C1.333333,3,2.200267,1.324038,2,1C1.810600,0.693544,0.189400,0.693544,0,1C-0.200267,1.324038,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3.160469,2.858341,3,3C2.796233,3.179882,0.203767,0.820118,0,1C-0.160469,1.141659,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:11:3
}

// d3-shape/test/curve/catmullRomClosed-test.js: line.curve(curveCatmullRomClosed.alpha(0))(data) generates the expected path
#[test]
fn line_curve_curvecatmullromclosed_alpha_0_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRomClosed { alpha: 0.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomClosed-test.js:16:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/catmullRomClosed-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M1,3L0,1Z",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:18:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3C1.333333,3,2.166667,1.333333,2,1C1.833333,0.666667,0.166667,0.666667,0,1C-0.166667,1.333333,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:19:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3.333333,3,3,3C2.666667,3,0.333333,1,0,1C-0.333333,1,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:20:3
}

// d3-shape/test/curve/catmullRomClosed-test.js: line.curve(curveCatmullRomClosed.alpha(1))(data) generates the expected path
#[test]
fn line_curve_curvecatmullromclosed_alpha_1_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRomClosed { alpha: 1.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomClosed-test.js:25:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/catmullRomClosed-test.js:26:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M1,3L0,1Z",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:27:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3C1.333333,3,2.236068,1.314757,2,1C1.788854,0.718473,0.211146,0.718473,0,1C-0.236068,1.314757,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:28:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3.031652,2.746782,3,3C2.948962,3.408301,0.051038,0.591699,0,1C-0.031652,1.253218,0.666667,3,1,3",
    ); // d3-shape/test/curve/catmullRomClosed-test.js:29:3
}

// d3-shape/test/curve/catmullRomClosed-test.js: line.curve(curveCatmullRomClosed) uses a default alpha of 0.5 (centripetal)
#[test]
fn line_curve_curvecatmullromclosed_uses_a_default_alpha_of_0_5_centripetal() {
    let l = shape::line().curve(Curve::CatmullRomClosed { alpha: 0.5 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::CatmullRomClosed { alpha: 0.5 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/catmullRomClosed-test.js:34:3
}

// d3-shape/test/curve/catmullRomOpen-test.js: line.curve(curveCatmullRomOpen)(data) generates the expected path
#[test]
fn line_curve_curvecatmullromopen_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRomOpen { alpha: 0.5 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:7:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:8:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3Z",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:11:3
}

// d3-shape/test/curve/catmullRomOpen-test.js: line.curve(curveCatmullRomOpen.alpha(1))(data) generates the expected path
#[test]
fn line_curve_curvecatmullromopen_alpha_1_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::CatmullRomOpen { alpha: 1.0 });
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:16:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:17:3
    assert_eq!(drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:18:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3Z",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:19:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:20:3
}

// d3-shape/test/curve/catmullRomOpen-test.js: line.curve(curveCatmullRomOpen) uses a default alpha of 0.5 (centripetal)
#[test]
fn line_curve_curvecatmullromopen_uses_a_default_alpha_of_0_5_centripetal() {
    let l = shape::line().curve(Curve::CatmullRomOpen { alpha: 0.5 });
    assert_eq!(
        drawn(|ctx| shape::line()
            .curve(Curve::CatmullRomOpen { alpha: 0.5 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/catmullRomOpen-test.js:25:3
}

// d3-shape/test/curve/catmullRomOpen-test.js: area.curve(curveCatmullRomOpen.alpha(0.5))(data) generates the expected path
#[test]
fn area_curve_curvecatmullromopen_alpha_0_5_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::CatmullRomOpen { alpha: 0.5 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:35:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:36:3
    assert_eq!(drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)), None); // d3-shape/test/curve/catmullRomOpen-test.js:37:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M1,3L1,0Z",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:38:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M1,3C1.333333,3,1.666667,1,2,1L2,0C1.666667,0,1.333333,0,1,0Z",
    ); // d3-shape/test/curve/catmullRomOpen-test.js:39:3
}

// d3-shape/test/curve/catmullRomOpen-test.js: area.curve(curveCatmullRomOpen) uses a default alpha of 0.5 (centripetal)
#[test]
fn area_curve_curvecatmullromopen_uses_a_default_alpha_of_0_5_centripetal() {
    let a = shape::area().curve(Curve::CatmullRomOpen { alpha: 0.5 });
    assert_eq!(
        drawn(|ctx| shape::area()
            .curve(Curve::CatmullRomOpen { alpha: 0.5 })
            .draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx))
    ); // d3-shape/test/curve/catmullRomOpen-test.js:44:3
}

// d3-shape/test/curve/linear-test.js: line.curve(curveLinear)(data) generates the expected path
#[test]
fn line_curve_curvelinear_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::Linear);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/linear-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/linear-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,3",
    ); // d3-shape/test/curve/linear-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,3L4,5",
    ); // d3-shape/test/curve/linear-test.js:10:3
}

// d3-shape/test/curve/linear-test.js: area.curve(curveLinear)(data) generates the expected path
#[test]
fn area_curve_curvelinear_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::Linear);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/linear-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/linear-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,3L2,0L0,0Z",
    ); // d3-shape/test/curve/linear-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,3L4,5L4,0L2,0L0,0Z",
    ); // d3-shape/test/curve/linear-test.js:18:3
}

// d3-shape/test/curve/linearClosed-test.js: line.curve(curveLinearClosed)(data) generates the expected path
#[test]
fn line_curve_curvelinearclosed_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::LinearClosed);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/linearClosed-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/linearClosed-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,3Z",
    ); // d3-shape/test/curve/linearClosed-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,3L4,5Z",
    ); // d3-shape/test/curve/linearClosed-test.js:10:3
}

// d3-shape/test/curve/monotoneX-test.js: line.curve(curveMonotoneX)(data) generates the expected path
#[test]
fn line_curve_curvemonotonex_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::MonotoneX);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/monotoneX-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/monotoneX-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/monotoneX-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,2,2,1",
    ); // d3-shape/test/curve/monotoneX-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,2.666667,2,3,3",
    ); // d3-shape/test/curve/monotoneX-test.js:11:3
}

// d3-shape/test/curve/monotoneX-test.js: line.curve(curveMonotoneX)(data) preserves monotonicity in y
#[test]
fn line_curve_curvemonotonex_data_preserves_monotonicity_in_y() {
    let l = shape::line().curve(Curve::MonotoneX);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![
                    (0.0, 200.0),
                    (100.0, 100.0),
                    (200.0, 100.0),
                    (300.0, 300.0),
                    (400.0, 300.0),
                ],
                ctx,
            )
        }),
        &"M0,200C33.333333,150,66.666667,100,100,100C133.333333,100,166.666667,100,200,100C233.333333,100,266.666667,300,300,300C333.333333,300,366.666667,300,400,300",
    ); // d3-shape/test/curve/monotoneX-test.js:16:3
}

// d3-shape/test/curve/monotoneX-test.js: line.curve(curveMonotoneX)(data) handles duplicate x-values
#[test]
fn line_curve_curvemonotonex_data_handles_duplicate_x_values() {
    let l = shape::line().curve(Curve::MonotoneX);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 200.0), (0.0, 100.0), (100.0, 100.0), (200.0, 0.0)],
                ctx,
            )
        }),
        &"M0,200C0,200,0,100,0,100C33.333333,100,66.666667,100,100,100C133.333333,100,166.666667,50,200,0",
    ); // d3-shape/test/curve/monotoneX-test.js:21:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 200.0), (100.0, 100.0), (100.0, 0.0), (200.0, 0.0)],
                ctx,
            )
        }),
        &"M0,200C33.333333,183.333333,66.666667,166.666667,100,100C100,100,100,0,100,0C133.333333,0,166.666667,0,200,0",
    ); // d3-shape/test/curve/monotoneX-test.js:22:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 200.0), (100.0, 100.0), (200.0, 100.0), (200.0, 0.0)],
                ctx,
            )
        }),
        &"M0,200C33.333333,150,66.666667,100,100,100C133.333333,100,166.666667,100,200,100C200,100,200,0,200,0",
    ); // d3-shape/test/curve/monotoneX-test.js:23:3
}

// d3-shape/test/curve/monotoneX-test.js: line.curve(curveMonotoneX)(data) handles segments of infinite slope
#[test]
fn line_curve_curvemonotonex_data_handles_segments_of_infinite_slope() {
    let l = shape::line().curve(Curve::MonotoneX);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 200.0), (100.0, 150.0), (100.0, 50.0), (200.0, 0.0)],
                ctx,
            )
        }),
        &"M0,200C33.333333,191.666667,66.666667,183.333333,100,150C100,150,100,50,100,50C133.333333,16.666667,166.666667,8.333333,200,0",
    ); // d3-shape/test/curve/monotoneX-test.js:28:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(200.0, 0.0), (100.0, 50.0), (100.0, 150.0), (0.0, 200.0)],
                ctx,
            )
        }),
        &"M200,0C166.666667,8.333333,133.333333,16.666667,100,50C100,50,100,150,100,150C66.666667,183.333333,33.333333,191.666667,0,200",
    ); // d3-shape/test/curve/monotoneX-test.js:29:3
}

// d3-shape/test/curve/monotoneX-test.js: line.curve(curveMonotoneX)(data) ignores coincident points
#[test]
fn line_curve_curvemonotonex_data_ignores_coincident_points() {
    let l = shape::line().curve(Curve::MonotoneX);
    let p = drawn(|ctx| {
        l.draw(
            vec![
                (0.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (200.0, 0.0),
            ],
            ctx,
        )
    });
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (0.0, 200.0),
                (0.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (200.0, 0.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneX-test.js:35:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (0.0, 200.0),
                (50.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (200.0, 0.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneX-test.js:36:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (0.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (200.0, 0.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneX-test.js:37:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (0.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (150.0, 0.0),
                (200.0, 0.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneX-test.js:38:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (0.0, 200.0),
                (50.0, 200.0),
                (100.0, 100.0),
                (150.0, 0.0),
                (200.0, 0.0),
                (200.0, 0.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneX-test.js:39:3
}

// d3-shape/test/curve/monotoneX-test.js: area.curve(curveMonotoneX)(data) generates the expected path
#[test]
fn area_curve_curvemonotonex_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::MonotoneX);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/monotoneX-test.js:44:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/monotoneX-test.js:45:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/curve/monotoneX-test.js:46:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,2,2,1L2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0.333333,0,0,0Z",
    ); // d3-shape/test/curve/monotoneX-test.js:47:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,2.666667,2,3,3L3,0C2.666667,0,2.333333,0,2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0.333333,0,0,0Z",
    ); // d3-shape/test/curve/monotoneX-test.js:48:3
}

// d3-shape/test/curve/monotoneY-test.js: line.curve(curveMonotoneY)(data) generates the expected path
#[test]
fn line_curve_curvemonotoney_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::MonotoneY);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/monotoneY-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(1.0, 0.0)], ctx)), &"M1,0Z"); // d3-shape/test/curve/monotoneY-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(1.0, 0.0), (3.0, 1.0)], ctx)),
        &"M1,0L3,1",
    ); // d3-shape/test/curve/monotoneY-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(1.0, 0.0), (3.0, 1.0), (1.0, 2.0)], ctx)),
        &"M1,0C2,0.333333,3,0.666667,3,1C3,1.333333,2,1.666667,1,2",
    ); // d3-shape/test/curve/monotoneY-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(1.0, 0.0), (3.0, 1.0), (1.0, 2.0), (3.0, 3.0)], ctx)),
        &"M1,0C2,0.333333,3,0.666667,3,1C3,1.333333,1,1.666667,1,2C1,2.333333,2,2.666667,3,3",
    ); // d3-shape/test/curve/monotoneY-test.js:11:3
}

// d3-shape/test/curve/monotoneY-test.js: line.curve(curveMonotoneY)(data) preserves monotonicity in y
#[test]
fn line_curve_curvemonotoney_data_preserves_monotonicity_in_y() {
    let l = shape::line().curve(Curve::MonotoneY);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![
                    (200.0, 0.0),
                    (100.0, 100.0),
                    (100.0, 200.0),
                    (300.0, 300.0),
                    (300.0, 400.0),
                ],
                ctx,
            )
        }),
        &"M200,0C150,33.333333,100,66.666667,100,100C100,133.333333,100,166.666667,100,200C100,233.333333,300,266.666667,300,300C300,333.333333,300,366.666667,300,400",
    ); // d3-shape/test/curve/monotoneY-test.js:16:3
}

// d3-shape/test/curve/monotoneY-test.js: line.curve(curveMonotoneY)(data) handles duplicate x-values
#[test]
fn line_curve_curvemonotoney_data_handles_duplicate_x_values() {
    let l = shape::line().curve(Curve::MonotoneY);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(200.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 200.0)],
                ctx,
            )
        }),
        &"M200,0C200,0,100,0,100,0C100,33.333333,100,66.666667,100,100C100,133.333333,50,166.666667,0,200",
    ); // d3-shape/test/curve/monotoneY-test.js:21:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(200.0, 0.0), (100.0, 100.0), (0.0, 100.0), (0.0, 200.0)],
                ctx,
            )
        }),
        &"M200,0C183.333333,33.333333,166.666667,66.666667,100,100C100,100,0,100,0,100C0,133.333333,0,166.666667,0,200",
    ); // d3-shape/test/curve/monotoneY-test.js:22:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(200.0, 0.0), (100.0, 100.0), (100.0, 200.0), (0.0, 200.0)],
                ctx,
            )
        }),
        &"M200,0C150,33.333333,100,66.666667,100,100C100,133.333333,100,166.666667,100,200C100,200,0,200,0,200",
    ); // d3-shape/test/curve/monotoneY-test.js:23:3
}

// d3-shape/test/curve/monotoneY-test.js: line.curve(curveMonotoneY)(data) handles segments of infinite slope
#[test]
fn line_curve_curvemonotoney_data_handles_segments_of_infinite_slope() {
    let l = shape::line().curve(Curve::MonotoneY);
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(200.0, 0.0), (150.0, 100.0), (50.0, 100.0), (0.0, 200.0)],
                ctx,
            )
        }),
        &"M200,0C191.666667,33.333333,183.333333,66.666667,150,100C150,100,50,100,50,100C16.666667,133.333333,8.333333,166.666667,0,200",
    ); // d3-shape/test/curve/monotoneY-test.js:28:3
    assert_path_eq(
        &drawn(|ctx| {
            l.draw(
                vec![(0.0, 200.0), (50.0, 100.0), (150.0, 100.0), (200.0, 0.0)],
                ctx,
            )
        }),
        &"M0,200C8.333333,166.666667,16.666667,133.333333,50,100C50,100,150,100,150,100C183.333333,66.666667,191.666667,33.333333,200,0",
    ); // d3-shape/test/curve/monotoneY-test.js:29:3
}

// d3-shape/test/curve/monotoneY-test.js: line.curve(curveMonotoneY)(data) ignores coincident points
#[test]
fn line_curve_curvemonotoney_data_ignores_coincident_points() {
    let l = shape::line().curve(Curve::MonotoneY);
    let p = drawn(|ctx| {
        l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 200.0),
            ],
            ctx,
        )
    });
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 0.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 200.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneY-test.js:35:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 50.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 200.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneY-test.js:36:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 200.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneY-test.js:37:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 150.0),
                (0.0, 200.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneY-test.js:38:3
    assert_eq!(
        drawn(|ctx| l.draw(
            vec![
                (200.0, 0.0),
                (200.0, 50.0),
                (100.0, 100.0),
                (0.0, 150.0),
                (0.0, 200.0),
                (0.0, 200.0)
            ],
            ctx
        )),
        p
    ); // d3-shape/test/curve/monotoneY-test.js:39:3
}

// d3-shape/test/curve/monotoneY-test.js: area.curve(curveMonotoneY)(data) generates the expected path
#[test]
fn area_curve_curvemonotoney_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::MonotoneY);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/monotoneY-test.js:44:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(1.0, 0.0)], ctx)), &"M1,0L1,0Z"); // d3-shape/test/curve/monotoneY-test.js:45:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(1.0, 0.0), (3.0, 1.0)], ctx)),
        &"M1,0L3,1L3,0L1,0Z",
    ); // d3-shape/test/curve/monotoneY-test.js:46:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(1.0, 0.0), (3.0, 1.0), (1.0, 2.0)], ctx)),
        &"M1,0C2,0.333333,3,0.666667,3,1C3,1.333333,2,1.666667,1,2L1,0C1,0,3,0,3,0C3,0,1,0,1,0Z",
    ); // d3-shape/test/curve/monotoneY-test.js:47:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(1.0, 0.0), (3.0, 1.0), (1.0, 2.0), (3.0, 3.0)], ctx)),
        &"M1,0C2,0.333333,3,0.666667,3,1C3,1.333333,1,1.666667,1,2C1,2.333333,2,2.666667,3,3L3,0C3,0,1,0,1,0C1,0,3,0,3,0C3,0,1,0,1,0Z",
    ); // d3-shape/test/curve/monotoneY-test.js:48:3
}

// d3-shape/test/curve/natural-test.js: line.curve(curveNatural)(data) generates the expected path
#[test]
fn line_curve_curvenatural_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::Natural);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/natural-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/natural-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3",
    ); // d3-shape/test/curve/natural-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,2,2,1",
    ); // d3-shape/test/curve/natural-test.js:10:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0.333333,2.111111,0.666667,3.222222,1,3C1.333333,2.777778,1.666667,1.222222,2,1C2.333333,0.777778,2.666667,1.888889,3,3",
    ); // d3-shape/test/curve/natural-test.js:11:3
}

// d3-shape/test/curve/natural-test.js: area.curve(curveNatural)(data) generates the expected path
#[test]
fn area_curve_curvenatural_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::Natural);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/natural-test.js:16:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/natural-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/curve/natural-test.js:18:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,2,2,1L2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0.333333,0,0,0Z",
    ); // d3-shape/test/curve/natural-test.js:19:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0.333333,2.111111,0.666667,3.222222,1,3C1.333333,2.777778,1.666667,1.222222,2,1C2.333333,0.777778,2.666667,1.888889,3,3L3,0C2.666667,0,2.333333,0,2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0.333333,0,0,0Z",
    ); // d3-shape/test/curve/natural-test.js:20:3
}

// d3-shape/test/curve/step-test.js: line.curve(curveStep)(data) generates the expected path
#[test]
fn line_curve_curvestep_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::Step);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/step-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/step-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L1,1L1,3L2,3",
    ); // d3-shape/test/curve/step-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L1,1L1,3L3,3L3,5L4,5",
    ); // d3-shape/test/curve/step-test.js:10:3
}

// d3-shape/test/curve/step-test.js: area.curve(curveStep)(data) generates the expected path
#[test]
fn area_curve_curvestep_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::Step);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/step-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/step-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L1,1L1,3L2,3L2,0L1,0L1,0L0,0Z",
    ); // d3-shape/test/curve/step-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L1,1L1,3L3,3L3,5L4,5L4,0L3,0L3,0L1,0L1,0L0,0Z",
    ); // d3-shape/test/curve/step-test.js:18:3
}

// d3-shape/test/curve/stepAfter-test.js: line.curve(curveStepAfter)(data) generates the expected path
#[test]
fn line_curve_curvestepafter_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::StepAfter);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/stepAfter-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/stepAfter-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,1L2,3",
    ); // d3-shape/test/curve/stepAfter-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,1L2,3L4,3L4,5",
    ); // d3-shape/test/curve/stepAfter-test.js:10:3
}

// d3-shape/test/curve/stepAfter-test.js: area.curve(curveStepAfter)(data) generates the expected path
#[test]
fn area_curve_curvestepafter_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::StepAfter);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/stepAfter-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/stepAfter-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,1L2,3L2,0L2,0L0,0Z",
    ); // d3-shape/test/curve/stepAfter-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,1L2,3L4,3L4,5L4,0L4,0L2,0L2,0L0,0Z",
    ); // d3-shape/test/curve/stepAfter-test.js:18:3
}

// d3-shape/test/curve/stepBefore-test.js: line.curve(curveStepBefore)(data) generates the expected path
#[test]
fn line_curve_curvestepbefore_data_generates_the_expected_path() {
    let l = shape::line().curve(Curve::StepBefore);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/stepBefore-test.js:7:3
    assert_path_eq(&drawn(|ctx| l.draw(vec![(0.0, 1.0)], ctx)), &"M0,1Z"); // d3-shape/test/curve/stepBefore-test.js:8:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L0,3L2,3",
    ); // d3-shape/test/curve/stepBefore-test.js:9:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L0,3L2,3L2,5L4,5",
    ); // d3-shape/test/curve/stepBefore-test.js:10:3
}

// d3-shape/test/curve/stepBefore-test.js: area.curve(curveStepBefore)(data) generates the expected path
#[test]
fn area_curve_curvestepbefore_data_generates_the_expected_path() {
    let a = shape::area().curve(Curve::StepBefore);
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/curve/stepBefore-test.js:15:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/curve/stepBefore-test.js:16:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L0,3L2,3L2,0L0,0L0,0Z",
    ); // d3-shape/test/curve/stepBefore-test.js:17:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L0,3L2,3L2,5L4,5L4,0L2,0L2,0L0,0L0,0Z",
    ); // d3-shape/test/curve/stepBefore-test.js:18:3
}

// d3-shape/test/line-test.js: line() returns a default line shape
#[test]
fn line_returns_a_default_line_shape() {
    let l = shape::line();
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,3L4,5",
    ); // d3-shape/test/line-test.js:12:3
}

// d3-shape/test/line-test.js: line.curve(curve) sets the curve method
#[test]
fn line_curve_curve_sets_the_curve_method() {
    let l = shape::line().curve(Curve::LinearClosed);
    assert_eq!(drawn(|ctx| l.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/line-test.js:63:3
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0)], ctx)),
        &"M0,1L2,3Z",
    ); // d3-shape/test/line-test.js:64:3
}

// d3-shape/test/area-test.js: area() returns a default area shape
#[test]
fn area_returns_a_default_area_shape() {
    let a = shape::area();
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,1L2,3L4,5L4,0L2,0L0,0Z",
    ); // d3-shape/test/area-test.js:14:3
}

// d3-shape/test/area-test.js: area.curve(curve) sets the curve method
#[test]
fn area_curve_curve_sets_the_curve_method() {
    let a = shape::area().curve(Curve::Cardinal { tension: 0.0 });
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3L3,0C3,0,2.333333,0,2,0C1.666667,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z",
    ); // d3-shape/test/area-test.js:106:3
}

// d3-shape/test/area-test.js: area.curve(curveCardinal.tension(tension)) sets the cardinal spline tension
#[test]
fn area_curve_curvecardinal_tension_tension_sets_the_cardinal_spline_tension() {
    let a = shape::area().curve(Curve::Cardinal { tension: 0.1 });
    assert_eq!(drawn(|ctx| a.draw(Vec::<(f64, f64)>::new(), ctx)), None); // d3-shape/test/area-test.js:111:3
    assert_path_eq(&drawn(|ctx| a.draw(vec![(0.0, 1.0)], ctx)), &"M0,1L0,0Z"); // d3-shape/test/area-test.js:112:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0)], ctx)),
        &"M0,1L1,3L1,0L0,0Z",
    ); // d3-shape/test/area-test.js:113:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0)], ctx)),
        &"M0,1C0,1,0.700000,3,1,3C1.300000,3,2,1,2,1L2,0C2,0,1.300000,0,1,0C0.700000,0,0,0,0,0Z",
    ); // d3-shape/test/area-test.js:114:3
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (1.0, 3.0), (2.0, 1.0), (3.0, 3.0)], ctx)),
        &"M0,1C0,1,0.700000,3,1,3C1.300000,3,1.700000,1,2,1C2.300000,1,3,3,3,3L3,0C3,0,2.300000,0,2,0C1.700000,0,1.300000,0,1,0C0.700000,0,0,0,0,0Z",
    ); // d3-shape/test/area-test.js:115:3
}

// d3-shape/test/lineRadial-test.js: lineRadial() returns a default radial line shape
#[test]
fn lineradial_returns_a_default_radial_line_shape() {
    let l = shape::line::radial();
    assert_path_eq(
        &drawn(|ctx| l.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,-1L2.727892,1.248441L-3.784012,3.268218",
    ); // d3-shape/test/lineRadial-test.js:12:3
}

// d3-shape/test/areaRadial-test.js: areaRadial() returns a default radial area shape
#[test]
fn arearadial_returns_a_default_radial_area_shape() {
    let a = shape::area::radial();
    assert_path_eq(
        &drawn(|ctx| a.draw(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], ctx)),
        &"M0,-1L2.727892,1.248441L-3.784012,3.268218L0,0L0,0L0,0Z",
    ); // d3-shape/test/areaRadial-test.js:14:3
}
