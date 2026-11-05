//! Easing curves against d3-ease's values.

use rustd3_core::ease::{Ease, Mode};

/// `actual` within 1e-15 of `expected`, relative.
fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1e-15 * expected.abs().max(1.0)
}

#[test]
fn the_default_is_cubic_in_out() {
    assert_eq!(Ease::default(), Ease::Cubic(Mode::InOut));
    assert_eq!(Ease::default().apply(0.25), 0.0625);
    assert_eq!(Ease::Cubic(Mode::In).apply(0.25), 0.015625);
}

#[test]
fn elastic_bounce_and_back_match_d3() {
    assert!(close(
        Ease::elastic(Mode::Out).apply(0.3),
        0.875_855_327_468_230_7
    ));
    assert_eq!(Ease::Bounce(Mode::Out).apply(0.5), 0.765625);
    assert!(close(Ease::back(Mode::In).apply(0.5), -0.087_697_5));
}

#[test]
fn poly_sin_exp_and_circle_match_d3() {
    let poly = Ease::Poly {
        mode: Mode::In,
        exponent: 2.0,
    };
    assert_eq!(poly.apply(0.5), 0.25);
    assert!(close(
        Ease::Sin(Mode::InOut).apply(0.25),
        0.146_446_609_406_726_2
    ));
    assert!(close(
        Ease::Exp(Mode::Out).apply(0.5),
        0.969_696_969_696_969_7
    ));
    assert!(close(
        Ease::Circle(Mode::In).apply(0.5),
        0.133_974_596_215_561_4
    ));
}

#[test]
fn the_parametric_constructors_carry_d3s_defaults() {
    assert_eq!(
        Ease::poly(Mode::In),
        Ease::Poly {
            mode: Mode::In,
            exponent: 3.0
        }
    );
    assert_eq!(
        Ease::back(Mode::Out),
        Ease::Back {
            mode: Mode::Out,
            overshoot: 1.70158
        }
    );
    assert_eq!(
        Ease::elastic(Mode::InOut),
        Ease::Elastic {
            mode: Mode::InOut,
            amplitude: 1.0,
            period: 0.3
        }
    );
}

#[test]
fn an_elastic_amplitude_below_one_acts_as_one() {
    let low = Ease::Elastic {
        mode: Mode::In,
        amplitude: 0.4,
        period: 0.3,
    };
    assert_eq!(low.apply(0.2), Ease::elastic(Mode::In).apply(0.2));
}

#[test]
fn every_curve_starts_at_zero_and_ends_at_one() {
    for mode in [Mode::In, Mode::Out, Mode::InOut] {
        for ease in [
            Ease::Quad(mode),
            Ease::Cubic(mode),
            Ease::Sin(mode),
            Ease::Exp(mode),
            Ease::Circle(mode),
            Ease::Bounce(mode),
            Ease::poly(mode),
            Ease::back(mode),
            Ease::elastic(mode),
        ] {
            assert!(ease.apply(0.0).abs() < 1e-12, "{ease:?} at 0");
            assert!((ease.apply(1.0) - 1.0).abs() < 1e-12, "{ease:?} at 1");
        }
    }
}
