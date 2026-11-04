//! d3's tests of d3-shape, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::shape;

use super::support::{Polygon, assert_in_delta, assert_path_eq, drawn};

// d3-shape/test/symbol-test.js: symbol() returns a default symbol shape
#[test]
fn symbol_returns_a_default_symbol_shape() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Circle.draw(64.0, ctx)),
        &"M4.513517,0A4.513517,4.513517,0,1,1,-4.513517,0A4.513517,4.513517,0,1,1,4.513517,0",
    ); // d3-shape/test/symbol-test.js:11:3
}

// d3-shape/test/symbol-test.js: symbol.size(size) observes the specified size constant
#[test]
fn symbol_size_size_observes_the_specified_size_constant() {
    assert_path_eq(&drawn(|ctx| shape::Symbol::Circle.draw(0.0, ctx)), &"M0,0"); // d3-shape/test/symbol-test.js:42:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Circle.draw(std::f64::consts::PI, ctx)),
        &"M1,0A1,1,0,1,1,-1,0A1,1,0,1,1,1,0",
    ); // d3-shape/test/symbol-test.js:43:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Circle.draw(4.0 * std::f64::consts::PI, ctx)),
        &"M2,0A2,2,0,1,1,-2,0A2,2,0,1,1,2,0",
    ); // d3-shape/test/symbol-test.js:44:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolCircle) generates the expected path
#[test]
fn symbol_type_symbolcircle_generates_the_expected_path() {
    assert_path_eq(&drawn(|ctx| shape::Symbol::Circle.draw(0.0, ctx)), &"M0,0"); // d3-shape/test/symbol-test.js:55:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Circle.draw(20.0, ctx)),
        &"M2.523133,0A2.523133,2.523133,0,1,1,-2.523133,0A2.523133,2.523133,0,1,1,2.523133,0",
    ); // d3-shape/test/symbol-test.js:56:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolCross) generates a polygon with the specified size
#[test]
fn symbol_type_symbolcross_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Cross.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:61:16
    shape::Symbol::Cross.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:62:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolCross) generates the expected path
#[test]
fn symbol_type_symbolcross_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Cross.draw(0.0, ctx)),
        &"M0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0Z",
    ); // d3-shape/test/symbol-test.js:67:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Cross.draw(20.0, ctx)),
        &"M-3,-1L-1,-1L-1,-3L1,-3L1,-1L3,-1L3,1L1,1L1,3L-1,3L-1,1L-3,1Z",
    ); // d3-shape/test/symbol-test.js:68:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolDiamond) generates a polygon with the specified size
#[test]
fn symbol_type_symboldiamond_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Diamond.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:73:16
    shape::Symbol::Diamond.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:74:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolDiamond) generates the expected path
#[test]
fn symbol_type_symboldiamond_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Diamond.draw(0.0, ctx)),
        &"M0,0L0,0L0,0L0,0Z",
    ); // d3-shape/test/symbol-test.js:79:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Diamond.draw(10.0, ctx)),
        &"M0,-2.942831L1.699044,0L0,2.942831L-1.699044,0Z",
    ); // d3-shape/test/symbol-test.js:80:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolStar) generates a polygon with the specified size
#[test]
fn symbol_type_symbolstar_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Star.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:97:16
    shape::Symbol::Star.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:98:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolStar) generates the expected path
#[test]
fn symbol_type_symbolstar_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Star.draw(0.0, ctx)),
        &"M0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0Z",
    ); // d3-shape/test/symbol-test.js:103:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Star.draw(10.0, ctx)),
        &"M0,-2.984649L0.670095,-0.922307L2.838570,-0.922307L1.084237,0.352290L1.754333,2.414632L0,1.140035L-1.754333,2.414632L-1.084237,0.352290L-2.838570,-0.922307L-0.670095,-0.922307Z",
    ); // d3-shape/test/symbol-test.js:104:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolSquare) generates a polygon with the specified size
#[test]
fn symbol_type_symbolsquare_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Square.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:109:16
    shape::Symbol::Square.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:110:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolSquare) generates the expected path
#[test]
fn symbol_type_symbolsquare_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Square.draw(0.0, ctx)),
        &"M0,0h0v0h0Z",
    ); // d3-shape/test/symbol-test.js:115:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Square.draw(4.0, ctx)),
        &"M-1,-1h2v2h-2Z",
    ); // d3-shape/test/symbol-test.js:116:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Square.draw(16.0, ctx)),
        &"M-2,-2h4v4h-4Z",
    ); // d3-shape/test/symbol-test.js:117:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolTriangle) generates a polygon with the specified size
#[test]
fn symbol_type_symboltriangle_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Triangle.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:128:16
    shape::Symbol::Triangle.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:129:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolTriangle) generates the expected path
#[test]
fn symbol_type_symboltriangle_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Triangle.draw(0.0, ctx)),
        &"M0,0L0,0L0,0Z",
    ); // d3-shape/test/symbol-test.js:134:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Triangle.draw(10.0, ctx)),
        &"M0,-2.774528L2.402811,1.387264L-2.402811,1.387264Z",
    ); // d3-shape/test/symbol-test.js:135:3
}

// d3-shape/test/symbol-test.js: symbol.type(symbolWye) generates a polygon with the specified size
#[test]
fn symbol_type_symbolwye_generates_a_polygon_with_the_specified_size() {
    let mut p = Polygon::default();
    shape::Symbol::Wye.draw(1.0, &mut p);
    assert_in_delta(&p.area(), &1.0, 1e-6); // d3-shape/test/symbol-test.js:146:16
    shape::Symbol::Wye.draw(240.0, &mut p);
    assert_in_delta(&p.area(), &240.0, 1e-6); // d3-shape/test/symbol-test.js:147:18
}

// d3-shape/test/symbol-test.js: symbol.type(symbolWye) generates the expected path
#[test]
fn symbol_type_symbolwye_generates_the_expected_path() {
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Wye.draw(0.0, ctx)),
        &"M0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0L0,0Z",
    ); // d3-shape/test/symbol-test.js:152:3
    assert_path_eq(
        &drawn(|ctx| shape::Symbol::Wye.draw(10.0, ctx)),
        &"M0.853360,0.492688L0.853360,2.199408L-0.853360,2.199408L-0.853360,0.492688L-2.331423,-0.360672L-1.478063,-1.838735L0,-0.985375L1.478063,-1.838735L2.331423,-0.360672Z",
    ); // d3-shape/test/symbol-test.js:153:3
}

// d3-shape/test/symbol-test.js: symbol(type, size) is equivalent to symbol().type(type).size(size)
#[test]
fn symbol_type_size_is_equivalent_to_symbol_type_type_size_size() {
    assert_eq!(
        drawn(|ctx| shape::Symbol::Cross.draw(16.0, ctx)),
        drawn(|ctx| shape::Symbol::Cross.draw(16.0, ctx))
    ); // d3-shape/test/symbol-test.js:165:3
}

// d3-shape/test/symbols-test.js: symbolsFill is the array of symbol types
#[test]
fn symbolsfill_is_the_array_of_symbol_types() {
    assert_eq!(
        shape::Symbol::FILL,
        [
            shape::Symbol::Circle,
            shape::Symbol::Cross,
            shape::Symbol::Diamond,
            shape::Symbol::Square,
            shape::Symbol::Star,
            shape::Symbol::Triangle,
            shape::Symbol::Wye
        ]
    ); // d3-shape/test/symbols-test.js:9:3
}
