//! `interpolate::{number, round}` and `Interpolate` against d3-interpolate.

use rustd3_core::interpolate::{self, Interpolate};

#[test]
fn number_blends_linearly() {
    assert_eq!(interpolate::number(10.0, 20.0)(0.25), 12.5);
    assert_eq!(interpolate::number(10.0, 42.0)(0.5), 26.0);
}

#[test]
fn number_is_exact_at_both_ends() {
    let i = interpolate::number(2e42, 335.0);
    assert_eq!(i(1.0), 335.0);
    assert_eq!(i(0.0), 2e42);
}

#[test]
fn round_rounds_the_blend_not_its_ends() {
    assert_eq!(interpolate::round(10.0, 42.0)(0.3), 20.0);
    assert_eq!(interpolate::round(2.6, 3.6)(0.6), 3.0);
}

#[test]
fn round_takes_ties_toward_positive_infinity() {
    assert_eq!(interpolate::round(-3.0, -2.0)(0.5), -2.0);
    assert_eq!(interpolate::round(2.0, 3.0)(0.5), 3.0);
}

#[test]
fn f32_blends_through_f64() {
    assert_eq!(1.0_f32.lerp(&3.0, 0.5), 2.0);
    assert_eq!(0.5_f64.lerp(&1.5, 1.0), 1.5);
}
