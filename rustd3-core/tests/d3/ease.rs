//! d3's tests of d3-ease, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::ease::{Ease, Mode};

use super::support::{assert_in_delta, in_out, out};

// d3-ease/test/back-test.js: easeBack is an alias for easeBackInOut
#[test]
fn easeback_is_an_alias_for_easebackinout() {
    assert_eq!(Ease::back(Mode::InOut), Ease::back(Mode::InOut)); // d3-ease/test/back-test.js:7:3
}

// d3-ease/test/back-test.js: easeBackIn(t) returns the expected results
#[test]
fn easebackin_t_returns_the_expected_results() {
    assert_eq!(f64::abs(Ease::back(Mode::In).apply(0.0)), 0.000000); // d3-ease/test/back-test.js:11:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.1), &-0.014314, 1e-6); // d3-ease/test/back-test.js:12:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.2), &-0.046451, 1e-6); // d3-ease/test/back-test.js:13:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.3), &-0.080200, 1e-6); // d3-ease/test/back-test.js:14:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.4), &-0.099352, 1e-6); // d3-ease/test/back-test.js:15:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.5), &-0.087698, 1e-6); // d3-ease/test/back-test.js:16:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.6), &-0.029028, 1e-6); // d3-ease/test/back-test.js:17:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.7), &0.092868, 1e-6); // d3-ease/test/back-test.js:18:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.8), &0.294198, 1e-6); // d3-ease/test/back-test.js:19:3
    assert_in_delta(&Ease::back(Mode::In).apply(0.9), &0.591172, 1e-6); // d3-ease/test/back-test.js:20:3
    assert_eq!(Ease::back(Mode::In).apply(1.0), 1.000000); // d3-ease/test/back-test.js:21:3
}

// d3-ease/test/back-test.js: easeBackOut(t) returns the expected results
#[test]
fn easebackout_t_returns_the_expected_results() {
    let back_out = out(Ease::back(Mode::In));
    assert_eq!(Ease::back(Mode::Out).apply(0.0), back_out(0.0)); // d3-ease/test/back-test.js:31:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.1), &back_out(0.1), 1e-6); // d3-ease/test/back-test.js:32:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.2), &back_out(0.2), 1e-6); // d3-ease/test/back-test.js:33:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.3), &back_out(0.3), 1e-6); // d3-ease/test/back-test.js:34:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.4), &back_out(0.4), 1e-6); // d3-ease/test/back-test.js:35:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.5), &back_out(0.5), 1e-6); // d3-ease/test/back-test.js:36:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.6), &back_out(0.6), 1e-6); // d3-ease/test/back-test.js:37:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.7), &back_out(0.7), 1e-6); // d3-ease/test/back-test.js:38:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.8), &back_out(0.8), 1e-6); // d3-ease/test/back-test.js:39:3
    assert_in_delta(&Ease::back(Mode::Out).apply(0.9), &back_out(0.9), 1e-6); // d3-ease/test/back-test.js:40:3
    assert_eq!(Ease::back(Mode::Out).apply(1.0), back_out(1.0)); // d3-ease/test/back-test.js:41:3
}

// d3-ease/test/back-test.js: easeBackInOut(t) returns the expected results
#[test]
fn easebackinout_t_returns_the_expected_results() {
    let back_in_out = in_out(Ease::back(Mode::In));
    assert_eq!(Ease::back(Mode::InOut).apply(0.0), back_in_out(0.0)); // d3-ease/test/back-test.js:51:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.1), &back_in_out(0.1), 1e-6); // d3-ease/test/back-test.js:52:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.2), &back_in_out(0.2), 1e-6); // d3-ease/test/back-test.js:53:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.3), &back_in_out(0.3), 1e-6); // d3-ease/test/back-test.js:54:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.4), &back_in_out(0.4), 1e-6); // d3-ease/test/back-test.js:55:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.5), &back_in_out(0.5), 1e-6); // d3-ease/test/back-test.js:56:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.6), &back_in_out(0.6), 1e-6); // d3-ease/test/back-test.js:57:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.7), &back_in_out(0.7), 1e-6); // d3-ease/test/back-test.js:58:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.8), &back_in_out(0.8), 1e-6); // d3-ease/test/back-test.js:59:3
    assert_in_delta(&Ease::back(Mode::InOut).apply(0.9), &back_in_out(0.9), 1e-6); // d3-ease/test/back-test.js:60:3
    assert_eq!(Ease::back(Mode::InOut).apply(1.0), back_in_out(1.0)); // d3-ease/test/back-test.js:61:3
}

// d3-ease/test/bounce-test.js: easeBounce is an alias for easeBounceOut
#[test]
fn easebounce_is_an_alias_for_easebounceout() {
    assert_eq!(Ease::Bounce(Mode::Out), Ease::Bounce(Mode::Out)); // d3-ease/test/bounce-test.js:7:3
}

// d3-ease/test/bounce-test.js: easeBounceIn(t) returns the expected results
#[test]
fn easebouncein_t_returns_the_expected_results() {
    assert_eq!(Ease::Bounce(Mode::In).apply(0.0), 0.000000); // d3-ease/test/bounce-test.js:11:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.1), &0.011875, 1e-6); // d3-ease/test/bounce-test.js:12:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.2), &0.060000, 1e-6); // d3-ease/test/bounce-test.js:13:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.3), &0.069375, 1e-6); // d3-ease/test/bounce-test.js:14:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.4), &0.227500, 1e-6); // d3-ease/test/bounce-test.js:15:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.5), &0.234375, 1e-6); // d3-ease/test/bounce-test.js:16:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.6), &0.090000, 1e-6); // d3-ease/test/bounce-test.js:17:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.7), &0.319375, 1e-6); // d3-ease/test/bounce-test.js:18:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.8), &0.697500, 1e-6); // d3-ease/test/bounce-test.js:19:3
    assert_in_delta(&Ease::Bounce(Mode::In).apply(0.9), &0.924375, 1e-6); // d3-ease/test/bounce-test.js:20:3
    assert_eq!(Ease::Bounce(Mode::In).apply(1.0), 1.000000); // d3-ease/test/bounce-test.js:21:3
}

// d3-ease/test/bounce-test.js: easeBounceOut(t) returns the expected results
#[test]
fn easebounceout_t_returns_the_expected_results() {
    let bounce_out = out(Ease::Bounce(Mode::In));
    assert_eq!(Ease::Bounce(Mode::Out).apply(0.0), bounce_out(0.0)); // d3-ease/test/bounce-test.js:31:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.1), &bounce_out(0.1), 1e-6); // d3-ease/test/bounce-test.js:32:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.2), &bounce_out(0.2), 1e-6); // d3-ease/test/bounce-test.js:33:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.3), &bounce_out(0.3), 1e-6); // d3-ease/test/bounce-test.js:34:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.4), &bounce_out(0.4), 1e-6); // d3-ease/test/bounce-test.js:35:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.5), &bounce_out(0.5), 1e-6); // d3-ease/test/bounce-test.js:36:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.6), &bounce_out(0.6), 1e-6); // d3-ease/test/bounce-test.js:37:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.7), &bounce_out(0.7), 1e-6); // d3-ease/test/bounce-test.js:38:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.8), &bounce_out(0.8), 1e-6); // d3-ease/test/bounce-test.js:39:3
    assert_in_delta(&Ease::Bounce(Mode::Out).apply(0.9), &bounce_out(0.9), 1e-6); // d3-ease/test/bounce-test.js:40:3
    assert_eq!(Ease::Bounce(Mode::Out).apply(1.0), bounce_out(1.0)); // d3-ease/test/bounce-test.js:41:3
}

// d3-ease/test/bounce-test.js: easeBounceInOut(t) returns the expected results
#[test]
fn easebounceinout_t_returns_the_expected_results() {
    let bounce_in_out = in_out(Ease::Bounce(Mode::In));
    assert_eq!(Ease::Bounce(Mode::InOut).apply(0.0), bounce_in_out(0.0)); // d3-ease/test/bounce-test.js:51:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.1),
        &bounce_in_out(0.1),
        1e-6,
    ); // d3-ease/test/bounce-test.js:52:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.2),
        &bounce_in_out(0.2),
        1e-6,
    ); // d3-ease/test/bounce-test.js:53:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.3),
        &bounce_in_out(0.3),
        1e-6,
    ); // d3-ease/test/bounce-test.js:54:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.4),
        &bounce_in_out(0.4),
        1e-6,
    ); // d3-ease/test/bounce-test.js:55:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.5),
        &bounce_in_out(0.5),
        1e-6,
    ); // d3-ease/test/bounce-test.js:56:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.6),
        &bounce_in_out(0.6),
        1e-6,
    ); // d3-ease/test/bounce-test.js:57:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.7),
        &bounce_in_out(0.7),
        1e-6,
    ); // d3-ease/test/bounce-test.js:58:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.8),
        &bounce_in_out(0.8),
        1e-6,
    ); // d3-ease/test/bounce-test.js:59:3
    assert_in_delta(
        &Ease::Bounce(Mode::InOut).apply(0.9),
        &bounce_in_out(0.9),
        1e-6,
    ); // d3-ease/test/bounce-test.js:60:3
    assert_eq!(Ease::Bounce(Mode::InOut).apply(1.0), bounce_in_out(1.0)); // d3-ease/test/bounce-test.js:61:3
}

// d3-ease/test/circle-test.js: easeCircle is an alias for easeCircleInOut
#[test]
fn easecircle_is_an_alias_for_easecircleinout() {
    assert_eq!(Ease::Circle(Mode::InOut), Ease::Circle(Mode::InOut)); // d3-ease/test/circle-test.js:7:3
}

// d3-ease/test/circle-test.js: easeCircleIn(t) returns the expected results
#[test]
fn easecirclein_t_returns_the_expected_results() {
    assert_eq!(Ease::Circle(Mode::In).apply(0.0), 0.000000); // d3-ease/test/circle-test.js:11:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.1), &0.005013, 1e-6); // d3-ease/test/circle-test.js:12:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.2), &0.020204, 1e-6); // d3-ease/test/circle-test.js:13:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.3), &0.046061, 1e-6); // d3-ease/test/circle-test.js:14:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.4), &0.083485, 1e-6); // d3-ease/test/circle-test.js:15:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.5), &0.133975, 1e-6); // d3-ease/test/circle-test.js:16:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.6), &0.200000, 1e-6); // d3-ease/test/circle-test.js:17:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.7), &0.285857, 1e-6); // d3-ease/test/circle-test.js:18:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.8), &0.400000, 1e-6); // d3-ease/test/circle-test.js:19:3
    assert_in_delta(&Ease::Circle(Mode::In).apply(0.9), &0.564110, 1e-6); // d3-ease/test/circle-test.js:20:3
    assert_eq!(Ease::Circle(Mode::In).apply(1.0), 1.000000); // d3-ease/test/circle-test.js:21:3
}

// d3-ease/test/circle-test.js: easeCircleOut(t) returns the expected results
#[test]
fn easecircleout_t_returns_the_expected_results() {
    let circle_out = out(Ease::Circle(Mode::In));
    assert_eq!(Ease::Circle(Mode::Out).apply(0.0), circle_out(0.0)); // d3-ease/test/circle-test.js:31:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.1), &circle_out(0.1), 1e-6); // d3-ease/test/circle-test.js:32:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.2), &circle_out(0.2), 1e-6); // d3-ease/test/circle-test.js:33:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.3), &circle_out(0.3), 1e-6); // d3-ease/test/circle-test.js:34:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.4), &circle_out(0.4), 1e-6); // d3-ease/test/circle-test.js:35:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.5), &circle_out(0.5), 1e-6); // d3-ease/test/circle-test.js:36:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.6), &circle_out(0.6), 1e-6); // d3-ease/test/circle-test.js:37:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.7), &circle_out(0.7), 1e-6); // d3-ease/test/circle-test.js:38:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.8), &circle_out(0.8), 1e-6); // d3-ease/test/circle-test.js:39:3
    assert_in_delta(&Ease::Circle(Mode::Out).apply(0.9), &circle_out(0.9), 1e-6); // d3-ease/test/circle-test.js:40:3
    assert_eq!(Ease::Circle(Mode::Out).apply(1.0), circle_out(1.0)); // d3-ease/test/circle-test.js:41:3
}

// d3-ease/test/circle-test.js: easeCircleInOut(t) returns the expected results
#[test]
fn easecircleinout_t_returns_the_expected_results() {
    let circle_in_out = in_out(Ease::Circle(Mode::In));
    assert_eq!(Ease::Circle(Mode::InOut).apply(0.0), circle_in_out(0.0)); // d3-ease/test/circle-test.js:51:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.1),
        &circle_in_out(0.1),
        1e-6,
    ); // d3-ease/test/circle-test.js:52:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.2),
        &circle_in_out(0.2),
        1e-6,
    ); // d3-ease/test/circle-test.js:53:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.3),
        &circle_in_out(0.3),
        1e-6,
    ); // d3-ease/test/circle-test.js:54:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.4),
        &circle_in_out(0.4),
        1e-6,
    ); // d3-ease/test/circle-test.js:55:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.5),
        &circle_in_out(0.5),
        1e-6,
    ); // d3-ease/test/circle-test.js:56:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.6),
        &circle_in_out(0.6),
        1e-6,
    ); // d3-ease/test/circle-test.js:57:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.7),
        &circle_in_out(0.7),
        1e-6,
    ); // d3-ease/test/circle-test.js:58:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.8),
        &circle_in_out(0.8),
        1e-6,
    ); // d3-ease/test/circle-test.js:59:3
    assert_in_delta(
        &Ease::Circle(Mode::InOut).apply(0.9),
        &circle_in_out(0.9),
        1e-6,
    ); // d3-ease/test/circle-test.js:60:3
    assert_eq!(Ease::Circle(Mode::InOut).apply(1.0), circle_in_out(1.0)); // d3-ease/test/circle-test.js:61:3
}

// d3-ease/test/cubic-test.js: easeCubic is an alias for easeCubicInOut
#[test]
fn easecubic_is_an_alias_for_easecubicinout() {
    assert_eq!(Ease::Cubic(Mode::InOut), Ease::Cubic(Mode::InOut)); // d3-ease/test/cubic-test.js:7:3
}

// d3-ease/test/cubic-test.js: easeCubicIn(t) returns the expected results
#[test]
fn easecubicin_t_returns_the_expected_results() {
    assert_eq!(Ease::Cubic(Mode::In).apply(0.0), 0.000); // d3-ease/test/cubic-test.js:11:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.1), &0.001, 1e-6); // d3-ease/test/cubic-test.js:12:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.2), &0.008, 1e-6); // d3-ease/test/cubic-test.js:13:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.3), &0.027, 1e-6); // d3-ease/test/cubic-test.js:14:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.4), &0.064, 1e-6); // d3-ease/test/cubic-test.js:15:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.5), &0.125, 1e-6); // d3-ease/test/cubic-test.js:16:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.6), &0.216, 1e-6); // d3-ease/test/cubic-test.js:17:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.7), &0.343, 1e-6); // d3-ease/test/cubic-test.js:18:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.8), &0.512, 1e-6); // d3-ease/test/cubic-test.js:19:3
    assert_in_delta(&Ease::Cubic(Mode::In).apply(0.9), &0.729, 1e-6); // d3-ease/test/cubic-test.js:20:3
    assert_eq!(Ease::Cubic(Mode::In).apply(1.0), 1.000); // d3-ease/test/cubic-test.js:21:3
}

// d3-ease/test/cubic-test.js: easeCubicOut(t) returns the expected results
#[test]
fn easecubicout_t_returns_the_expected_results() {
    let cubic_out = out(Ease::Cubic(Mode::In));
    assert_eq!(Ease::Cubic(Mode::Out).apply(0.0), cubic_out(0.0)); // d3-ease/test/cubic-test.js:31:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.1), &cubic_out(0.1), 1e-6); // d3-ease/test/cubic-test.js:32:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.2), &cubic_out(0.2), 1e-6); // d3-ease/test/cubic-test.js:33:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.3), &cubic_out(0.3), 1e-6); // d3-ease/test/cubic-test.js:34:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.4), &cubic_out(0.4), 1e-6); // d3-ease/test/cubic-test.js:35:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.5), &cubic_out(0.5), 1e-6); // d3-ease/test/cubic-test.js:36:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.6), &cubic_out(0.6), 1e-6); // d3-ease/test/cubic-test.js:37:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.7), &cubic_out(0.7), 1e-6); // d3-ease/test/cubic-test.js:38:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.8), &cubic_out(0.8), 1e-6); // d3-ease/test/cubic-test.js:39:3
    assert_in_delta(&Ease::Cubic(Mode::Out).apply(0.9), &cubic_out(0.9), 1e-6); // d3-ease/test/cubic-test.js:40:3
    assert_eq!(Ease::Cubic(Mode::Out).apply(1.0), cubic_out(1.0)); // d3-ease/test/cubic-test.js:41:3
}

// d3-ease/test/cubic-test.js: easeCubicInOut(t) returns the expected results
#[test]
fn easecubicinout_t_returns_the_expected_results() {
    let cubic_in_out = in_out(Ease::Cubic(Mode::In));
    assert_eq!(Ease::Cubic(Mode::InOut).apply(0.0), cubic_in_out(0.0)); // d3-ease/test/cubic-test.js:51:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.1),
        &cubic_in_out(0.1),
        1e-6,
    ); // d3-ease/test/cubic-test.js:52:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.2),
        &cubic_in_out(0.2),
        1e-6,
    ); // d3-ease/test/cubic-test.js:53:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.3),
        &cubic_in_out(0.3),
        1e-6,
    ); // d3-ease/test/cubic-test.js:54:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.4),
        &cubic_in_out(0.4),
        1e-6,
    ); // d3-ease/test/cubic-test.js:55:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.5),
        &cubic_in_out(0.5),
        1e-6,
    ); // d3-ease/test/cubic-test.js:56:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.6),
        &cubic_in_out(0.6),
        1e-6,
    ); // d3-ease/test/cubic-test.js:57:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.7),
        &cubic_in_out(0.7),
        1e-6,
    ); // d3-ease/test/cubic-test.js:58:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.8),
        &cubic_in_out(0.8),
        1e-6,
    ); // d3-ease/test/cubic-test.js:59:3
    assert_in_delta(
        &Ease::Cubic(Mode::InOut).apply(0.9),
        &cubic_in_out(0.9),
        1e-6,
    ); // d3-ease/test/cubic-test.js:60:3
    assert_eq!(Ease::Cubic(Mode::InOut).apply(1.0), cubic_in_out(1.0)); // d3-ease/test/cubic-test.js:61:3
}

// d3-ease/test/elastic-test.js: easeElastic is an alias for easeElasticOut
#[test]
fn easeelastic_is_an_alias_for_easeelasticout() {
    assert_eq!(Ease::elastic(Mode::Out), Ease::elastic(Mode::Out)); // d3-ease/test/elastic-test.js:7:3
}

// d3-ease/test/elastic-test.js: easeElasticIn(t) returns the expected results
#[test]
fn easeelasticin_t_returns_the_expected_results() {
    assert_eq!(f64::abs(Ease::elastic(Mode::In).apply(0.0)), 0.000000); // d3-ease/test/elastic-test.js:11:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.1), &0.000978, 1e-6); // d3-ease/test/elastic-test.js:12:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.2), &-0.001466, 1e-6); // d3-ease/test/elastic-test.js:13:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.3), &-0.003421, 1e-6); // d3-ease/test/elastic-test.js:14:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.4), &0.014663, 1e-6); // d3-ease/test/elastic-test.js:15:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.5), &-0.015152, 1e-6); // d3-ease/test/elastic-test.js:16:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.6), &-0.030792, 1e-6); // d3-ease/test/elastic-test.js:17:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.7), &0.124145, 1e-6); // d3-ease/test/elastic-test.js:18:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.8), &-0.124633, 1e-6); // d3-ease/test/elastic-test.js:19:3
    assert_in_delta(&Ease::elastic(Mode::In).apply(0.9), &-0.249756, 1e-6); // d3-ease/test/elastic-test.js:20:3
    assert_eq!(Ease::elastic(Mode::In).apply(1.0), 1.000000); // d3-ease/test/elastic-test.js:21:3
}

// d3-ease/test/elastic-test.js: easeElasticIn(t) is the same as elasticIn.amplitude(1).period(0.3)(t)
#[test]
fn easeelasticin_t_is_the_same_as_elasticin_amplitude_1_period_0_3_t() {
    assert_eq!(
        Ease::elastic(Mode::In).apply(0.1),
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.0,
            period: 0.3
        }
        .apply(0.1)
    ); // d3-ease/test/elastic-test.js:30:3
    assert_eq!(
        Ease::elastic(Mode::In).apply(0.2),
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.0,
            period: 0.3
        }
        .apply(0.2)
    ); // d3-ease/test/elastic-test.js:31:3
    assert_eq!(
        Ease::elastic(Mode::In).apply(0.3),
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.0,
            period: 0.3
        }
        .apply(0.3)
    ); // d3-ease/test/elastic-test.js:32:3
}

// d3-ease/test/elastic-test.js: easeElasticIn.amplitude(a)(t) is the same as elasticIn(t) if a <= 1
#[test]
fn easeelasticin_amplitude_a_t_is_the_same_as_elasticin_t_if_a_1() {
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: -1.0,
            period: 0.3
        }
        .apply(0.1),
        Ease::elastic(Mode::In).apply(0.1)
    ); // d3-ease/test/elastic-test.js:36:3
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 0.4,
            period: 0.3
        }
        .apply(0.2),
        Ease::elastic(Mode::In).apply(0.2)
    ); // d3-ease/test/elastic-test.js:37:3
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 0.8,
            period: 0.3
        }
        .apply(0.3),
        Ease::elastic(Mode::In).apply(0.3)
    ); // d3-ease/test/elastic-test.js:38:3
}

// d3-ease/test/elastic-test.js: easeElasticIn.amplitude(1.3)(t) returns the expected results
#[test]
fn easeelasticin_amplitude_1_3_t_returns_the_expected_results() {
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3
        }
        .apply(0.0),
        0.000000
    ); // d3-ease/test/elastic-test.js:47:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.1),
        &0.000978,
        1e-6,
    ); // d3-ease/test/elastic-test.js:48:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.2),
        &-0.003576,
        1e-6,
    ); // d3-ease/test/elastic-test.js:49:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.3),
        &0.001501,
        1e-6,
    ); // d3-ease/test/elastic-test.js:50:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.4),
        &0.014663,
        1e-6,
    ); // d3-ease/test/elastic-test.js:51:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.5),
        &-0.036951,
        1e-6,
    ); // d3-ease/test/elastic-test.js:52:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.6),
        &0.013510,
        1e-6,
    ); // d3-ease/test/elastic-test.js:53:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.7),
        &0.124145,
        1e-6,
    ); // d3-ease/test/elastic-test.js:54:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.8),
        &-0.303950,
        1e-6,
    ); // d3-ease/test/elastic-test.js:55:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3,
        }
        .apply(0.9),
        &0.109580,
        1e-6,
    ); // d3-ease/test/elastic-test.js:56:3
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.3,
            period: 0.3
        }
        .apply(1.0),
        1.000000
    ); // d3-ease/test/elastic-test.js:57:3
}

// d3-ease/test/elastic-test.js: easeElasticIn.amplitude(1.5).period(1)(t) returns the expected results
#[test]
fn easeelasticin_amplitude_1_5_period_1_t_returns_the_expected_results() {
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0
        }
        .apply(0.0),
        0.000000
    ); // d3-ease/test/elastic-test.js:61:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.1),
        &0.000148,
        1e-6,
    ); // d3-ease/test/elastic-test.js:62:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.2),
        &-0.002212,
        1e-6,
    ); // d3-ease/test/elastic-test.js:63:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.3),
        &-0.009390,
        1e-6,
    ); // d3-ease/test/elastic-test.js:64:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.4),
        &-0.021498,
        1e-6,
    ); // d3-ease/test/elastic-test.js:65:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.5),
        &-0.030303,
        1e-6,
    ); // d3-ease/test/elastic-test.js:66:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.6),
        &-0.009352,
        1e-6,
    ); // d3-ease/test/elastic-test.js:67:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.7),
        &0.093642,
        1e-6,
    ); // d3-ease/test/elastic-test.js:68:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.8),
        &0.342077,
        1e-6,
    ); // d3-ease/test/elastic-test.js:69:3
    assert_in_delta(
        &Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0,
        }
        .apply(0.9),
        &0.732374,
        1e-6,
    ); // d3-ease/test/elastic-test.js:70:3
    assert_eq!(
        Ease::Elastic {
            mode: Mode::In,
            amplitude: 1.5,
            period: 1.0
        }
        .apply(1.0),
        1.000000
    ); // d3-ease/test/elastic-test.js:71:3
}

// d3-ease/test/elastic-test.js: easeElasticOut(t) returns the expected results
#[test]
fn easeelasticout_t_returns_the_expected_results() {
    let elastic_out = out(Ease::elastic(Mode::In));
    assert_eq!(Ease::elastic(Mode::Out).apply(0.0), elastic_out(0.0)); // d3-ease/test/elastic-test.js:76:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.1),
        &elastic_out(0.1),
        1e-6,
    ); // d3-ease/test/elastic-test.js:77:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.2),
        &elastic_out(0.2),
        1e-6,
    ); // d3-ease/test/elastic-test.js:78:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.3),
        &elastic_out(0.3),
        1e-6,
    ); // d3-ease/test/elastic-test.js:79:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.4),
        &elastic_out(0.4),
        1e-6,
    ); // d3-ease/test/elastic-test.js:80:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.5),
        &elastic_out(0.5),
        1e-6,
    ); // d3-ease/test/elastic-test.js:81:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.6),
        &elastic_out(0.6),
        1e-6,
    ); // d3-ease/test/elastic-test.js:82:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.7),
        &elastic_out(0.7),
        1e-6,
    ); // d3-ease/test/elastic-test.js:83:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.8),
        &elastic_out(0.8),
        1e-6,
    ); // d3-ease/test/elastic-test.js:84:3
    assert_in_delta(
        &Ease::elastic(Mode::Out).apply(0.9),
        &elastic_out(0.9),
        1e-6,
    ); // d3-ease/test/elastic-test.js:85:3
    assert_eq!(Ease::elastic(Mode::Out).apply(1.0), elastic_out(1.0)); // d3-ease/test/elastic-test.js:86:3
}

// d3-ease/test/elastic-test.js: easeElasticInOut(t) returns the expected results
#[test]
fn easeelasticinout_t_returns_the_expected_results() {
    let elastic_in_out = in_out(Ease::elastic(Mode::In));
    assert_eq!(Ease::elastic(Mode::InOut).apply(0.0), elastic_in_out(0.0)); // d3-ease/test/elastic-test.js:96:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.1),
        &elastic_in_out(0.1),
        1e-6,
    ); // d3-ease/test/elastic-test.js:97:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.2),
        &elastic_in_out(0.2),
        1e-6,
    ); // d3-ease/test/elastic-test.js:98:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.3),
        &elastic_in_out(0.3),
        1e-6,
    ); // d3-ease/test/elastic-test.js:99:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.4),
        &elastic_in_out(0.4),
        1e-6,
    ); // d3-ease/test/elastic-test.js:100:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.5),
        &elastic_in_out(0.5),
        1e-6,
    ); // d3-ease/test/elastic-test.js:101:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.6),
        &elastic_in_out(0.6),
        1e-6,
    ); // d3-ease/test/elastic-test.js:102:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.7),
        &elastic_in_out(0.7),
        1e-6,
    ); // d3-ease/test/elastic-test.js:103:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.8),
        &elastic_in_out(0.8),
        1e-6,
    ); // d3-ease/test/elastic-test.js:104:3
    assert_in_delta(
        &Ease::elastic(Mode::InOut).apply(0.9),
        &elastic_in_out(0.9),
        1e-6,
    ); // d3-ease/test/elastic-test.js:105:3
    assert_eq!(Ease::elastic(Mode::InOut).apply(1.0), elastic_in_out(1.0)); // d3-ease/test/elastic-test.js:106:3
}

// d3-ease/test/exp-test.js: easeExp is an alias for easeExpInOut
#[test]
fn easeexp_is_an_alias_for_easeexpinout() {
    assert_eq!(Ease::Exp(Mode::InOut), Ease::Exp(Mode::InOut)); // d3-ease/test/exp-test.js:7:3
}

// d3-ease/test/exp-test.js: easeExpIn(t) returns the expected results
#[test]
fn easeexpin_t_returns_the_expected_results() {
    assert_eq!(Ease::Exp(Mode::In).apply(0.0), 0.000000); // d3-ease/test/exp-test.js:11:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.1), &0.000978, 1e-6); // d3-ease/test/exp-test.js:12:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.2), &0.002933, 1e-6); // d3-ease/test/exp-test.js:13:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.3), &0.006843, 1e-6); // d3-ease/test/exp-test.js:14:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.4), &0.014663, 1e-6); // d3-ease/test/exp-test.js:15:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.5), &0.030303, 1e-6); // d3-ease/test/exp-test.js:16:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.6), &0.061584, 1e-6); // d3-ease/test/exp-test.js:17:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.7), &0.124145, 1e-6); // d3-ease/test/exp-test.js:18:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.8), &0.249267, 1e-6); // d3-ease/test/exp-test.js:19:3
    assert_in_delta(&Ease::Exp(Mode::In).apply(0.9), &0.499511, 1e-6); // d3-ease/test/exp-test.js:20:3
    assert_eq!(Ease::Exp(Mode::In).apply(1.0), 1.000000); // d3-ease/test/exp-test.js:21:3
}

// d3-ease/test/exp-test.js: easeExpOut(t) returns the expected results
#[test]
fn easeexpout_t_returns_the_expected_results() {
    let exp_out = out(Ease::Exp(Mode::In));
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.0), &exp_out(0.0), 1e-6); // d3-ease/test/exp-test.js:31:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.1), &exp_out(0.1), 1e-6); // d3-ease/test/exp-test.js:32:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.2), &exp_out(0.2), 1e-6); // d3-ease/test/exp-test.js:33:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.3), &exp_out(0.3), 1e-6); // d3-ease/test/exp-test.js:34:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.4), &exp_out(0.4), 1e-6); // d3-ease/test/exp-test.js:35:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.5), &exp_out(0.5), 1e-6); // d3-ease/test/exp-test.js:36:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.6), &exp_out(0.6), 1e-6); // d3-ease/test/exp-test.js:37:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.7), &exp_out(0.7), 1e-6); // d3-ease/test/exp-test.js:38:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.8), &exp_out(0.8), 1e-6); // d3-ease/test/exp-test.js:39:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(0.9), &exp_out(0.9), 1e-6); // d3-ease/test/exp-test.js:40:3
    assert_in_delta(&Ease::Exp(Mode::Out).apply(1.0), &exp_out(1.0), 1e-6); // d3-ease/test/exp-test.js:41:3
}

// d3-ease/test/exp-test.js: easeExpInOut(t) returns the expected results
#[test]
fn easeexpinout_t_returns_the_expected_results() {
    let exp_in_out = in_out(Ease::Exp(Mode::In));
    assert_eq!(Ease::Exp(Mode::InOut).apply(0.0), exp_in_out(0.0)); // d3-ease/test/exp-test.js:51:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.1), &exp_in_out(0.1), 1e-6); // d3-ease/test/exp-test.js:52:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.2), &exp_in_out(0.2), 1e-6); // d3-ease/test/exp-test.js:53:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.3), &exp_in_out(0.3), 1e-6); // d3-ease/test/exp-test.js:54:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.4), &exp_in_out(0.4), 1e-6); // d3-ease/test/exp-test.js:55:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.5), &exp_in_out(0.5), 1e-6); // d3-ease/test/exp-test.js:56:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.6), &exp_in_out(0.6), 1e-6); // d3-ease/test/exp-test.js:57:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.7), &exp_in_out(0.7), 1e-6); // d3-ease/test/exp-test.js:58:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.8), &exp_in_out(0.8), 1e-6); // d3-ease/test/exp-test.js:59:3
    assert_in_delta(&Ease::Exp(Mode::InOut).apply(0.9), &exp_in_out(0.9), 1e-6); // d3-ease/test/exp-test.js:60:3
    assert_eq!(Ease::Exp(Mode::InOut).apply(1.0), exp_in_out(1.0)); // d3-ease/test/exp-test.js:61:3
}

// d3-ease/test/linear-test.js: easeLinear(t) returns the expected results
#[test]
fn easelinear_t_returns_the_expected_results() {
    assert_eq!(Ease::Linear.apply(0.0), 0.0); // d3-ease/test/linear-test.js:6:3
    assert_in_delta(&Ease::Linear.apply(0.1), &0.1, 1e-6); // d3-ease/test/linear-test.js:7:3
    assert_in_delta(&Ease::Linear.apply(0.2), &0.2, 1e-6); // d3-ease/test/linear-test.js:8:3
    assert_in_delta(&Ease::Linear.apply(0.3), &0.3, 1e-6); // d3-ease/test/linear-test.js:9:3
    assert_in_delta(&Ease::Linear.apply(0.4), &0.4, 1e-6); // d3-ease/test/linear-test.js:10:3
    assert_in_delta(&Ease::Linear.apply(0.5), &0.5, 1e-6); // d3-ease/test/linear-test.js:11:3
    assert_in_delta(&Ease::Linear.apply(0.6), &0.6, 1e-6); // d3-ease/test/linear-test.js:12:3
    assert_in_delta(&Ease::Linear.apply(0.7), &0.7, 1e-6); // d3-ease/test/linear-test.js:13:3
    assert_in_delta(&Ease::Linear.apply(0.8), &0.8, 1e-6); // d3-ease/test/linear-test.js:14:3
    assert_in_delta(&Ease::Linear.apply(0.9), &0.9, 1e-6); // d3-ease/test/linear-test.js:15:3
    assert_eq!(Ease::Linear.apply(1.0), 1.0); // d3-ease/test/linear-test.js:16:3
}

// d3-ease/test/poly-test.js: easePoly is an alias for easePolyInOut
#[test]
fn easepoly_is_an_alias_for_easepolyinout() {
    assert_eq!(Ease::poly(Mode::InOut), Ease::poly(Mode::InOut)); // d3-ease/test/poly-test.js:7:3
}

// d3-ease/test/poly-test.js: easePolyIn(t) returns the expected results
#[test]
fn easepolyin_t_returns_the_expected_results() {
    assert_eq!(Ease::poly(Mode::In).apply(0.0), 0.000); // d3-ease/test/poly-test.js:11:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.1), &0.001, 1e-6); // d3-ease/test/poly-test.js:12:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.2), &0.008, 1e-6); // d3-ease/test/poly-test.js:13:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.3), &0.027, 1e-6); // d3-ease/test/poly-test.js:14:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.4), &0.064, 1e-6); // d3-ease/test/poly-test.js:15:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.5), &0.125, 1e-6); // d3-ease/test/poly-test.js:16:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.6), &0.216, 1e-6); // d3-ease/test/poly-test.js:17:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.7), &0.343, 1e-6); // d3-ease/test/poly-test.js:18:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.8), &0.512, 1e-6); // d3-ease/test/poly-test.js:19:3
    assert_in_delta(&Ease::poly(Mode::In).apply(0.9), &0.729, 1e-6); // d3-ease/test/poly-test.js:20:3
    assert_eq!(Ease::poly(Mode::In).apply(1.0), 1.000); // d3-ease/test/poly-test.js:21:3
}

// d3-ease/test/poly-test.js: easePolyIn(t) is the same as polyIn.exponent(3)(t)
#[test]
fn easepolyin_t_is_the_same_as_polyin_exponent_3_t() {
    assert_eq!(
        Ease::poly(Mode::In).apply(0.1),
        Ease::Poly {
            mode: Mode::In,
            exponent: 3.0
        }
        .apply(0.1)
    ); // d3-ease/test/poly-test.js:30:3
    assert_eq!(
        Ease::poly(Mode::In).apply(0.2),
        Ease::Poly {
            mode: Mode::In,
            exponent: 3.0
        }
        .apply(0.2)
    ); // d3-ease/test/poly-test.js:31:3
    assert_eq!(
        Ease::poly(Mode::In).apply(0.3),
        Ease::Poly {
            mode: Mode::In,
            exponent: 3.0
        }
        .apply(0.3)
    ); // d3-ease/test/poly-test.js:32:3
}

// d3-ease/test/poly-test.js: easePolyIn.exponent(2.5)(t) returns the expected results
#[test]
fn easepolyin_exponent_2_5_t_returns_the_expected_results() {
    assert_eq!(
        Ease::Poly {
            mode: Mode::In,
            exponent: 2.5
        }
        .apply(0.0),
        0.000000
    ); // d3-ease/test/poly-test.js:41:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.1),
        &0.003162,
        1e-6,
    ); // d3-ease/test/poly-test.js:42:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.2),
        &0.017889,
        1e-6,
    ); // d3-ease/test/poly-test.js:43:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.3),
        &0.049295,
        1e-6,
    ); // d3-ease/test/poly-test.js:44:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.4),
        &0.101193,
        1e-6,
    ); // d3-ease/test/poly-test.js:45:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.5),
        &0.176777,
        1e-6,
    ); // d3-ease/test/poly-test.js:46:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.6),
        &0.278855,
        1e-6,
    ); // d3-ease/test/poly-test.js:47:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.7),
        &0.409963,
        1e-6,
    ); // d3-ease/test/poly-test.js:48:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.8),
        &0.572433,
        1e-6,
    ); // d3-ease/test/poly-test.js:49:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::In,
            exponent: 2.5,
        }
        .apply(0.9),
        &0.768433,
        1e-6,
    ); // d3-ease/test/poly-test.js:50:3
    assert_eq!(
        Ease::Poly {
            mode: Mode::In,
            exponent: 2.5
        }
        .apply(1.0),
        1.000000
    ); // d3-ease/test/poly-test.js:51:3
}

// d3-ease/test/poly-test.js: easePolyOut(t) is the same as polyOut.exponent(3)(t)
#[test]
fn easepolyout_t_is_the_same_as_polyout_exponent_3_t() {
    assert_eq!(
        Ease::poly(Mode::Out).apply(0.1),
        Ease::Poly {
            mode: Mode::Out,
            exponent: 3.0
        }
        .apply(0.1)
    ); // d3-ease/test/poly-test.js:60:3
    assert_eq!(
        Ease::poly(Mode::Out).apply(0.2),
        Ease::Poly {
            mode: Mode::Out,
            exponent: 3.0
        }
        .apply(0.2)
    ); // d3-ease/test/poly-test.js:61:3
    assert_eq!(
        Ease::poly(Mode::Out).apply(0.3),
        Ease::Poly {
            mode: Mode::Out,
            exponent: 3.0
        }
        .apply(0.3)
    ); // d3-ease/test/poly-test.js:62:3
}

// d3-ease/test/poly-test.js: easePolyOut.exponent(2.5)(t) returns the expected results
#[test]
fn easepolyout_exponent_2_5_t_returns_the_expected_results() {
    let poly_out = out(Ease::Poly {
        mode: Mode::In,
        exponent: 2.5,
    });
    assert_eq!(
        Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5
        }
        .apply(0.0),
        poly_out(0.0)
    ); // d3-ease/test/poly-test.js:79:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.1),
        &poly_out(0.1),
        1e-6,
    ); // d3-ease/test/poly-test.js:80:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.2),
        &poly_out(0.2),
        1e-6,
    ); // d3-ease/test/poly-test.js:81:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.3),
        &poly_out(0.3),
        1e-6,
    ); // d3-ease/test/poly-test.js:82:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.4),
        &poly_out(0.4),
        1e-6,
    ); // d3-ease/test/poly-test.js:83:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.5),
        &poly_out(0.5),
        1e-6,
    ); // d3-ease/test/poly-test.js:84:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.6),
        &poly_out(0.6),
        1e-6,
    ); // d3-ease/test/poly-test.js:85:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.7),
        &poly_out(0.7),
        1e-6,
    ); // d3-ease/test/poly-test.js:86:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.8),
        &poly_out(0.8),
        1e-6,
    ); // d3-ease/test/poly-test.js:87:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5,
        }
        .apply(0.9),
        &poly_out(0.9),
        1e-6,
    ); // d3-ease/test/poly-test.js:88:3
    assert_eq!(
        Ease::Poly {
            mode: Mode::Out,
            exponent: 2.5
        }
        .apply(1.0),
        poly_out(1.0)
    ); // d3-ease/test/poly-test.js:89:3
}

// d3-ease/test/poly-test.js: easePolyInOut(t) is the same as polyInOut.exponent(3)(t)
#[test]
fn easepolyinout_t_is_the_same_as_polyinout_exponent_3_t() {
    assert_eq!(
        Ease::poly(Mode::InOut).apply(0.1),
        Ease::Poly {
            mode: Mode::InOut,
            exponent: 3.0
        }
        .apply(0.1)
    ); // d3-ease/test/poly-test.js:98:3
    assert_eq!(
        Ease::poly(Mode::InOut).apply(0.2),
        Ease::Poly {
            mode: Mode::InOut,
            exponent: 3.0
        }
        .apply(0.2)
    ); // d3-ease/test/poly-test.js:99:3
    assert_eq!(
        Ease::poly(Mode::InOut).apply(0.3),
        Ease::Poly {
            mode: Mode::InOut,
            exponent: 3.0
        }
        .apply(0.3)
    ); // d3-ease/test/poly-test.js:100:3
}

// d3-ease/test/poly-test.js: easePolyInOut.exponent(2.5)(t) returns the expected results
#[test]
fn easepolyinout_exponent_2_5_t_returns_the_expected_results() {
    let poly_in_out = in_out(Ease::Poly {
        mode: Mode::In,
        exponent: 2.5,
    });
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.0),
        &poly_in_out(0.0),
        1e-6,
    ); // d3-ease/test/poly-test.js:105:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.1),
        &poly_in_out(0.1),
        1e-6,
    ); // d3-ease/test/poly-test.js:106:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.2),
        &poly_in_out(0.2),
        1e-6,
    ); // d3-ease/test/poly-test.js:107:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.3),
        &poly_in_out(0.3),
        1e-6,
    ); // d3-ease/test/poly-test.js:108:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.4),
        &poly_in_out(0.4),
        1e-6,
    ); // d3-ease/test/poly-test.js:109:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.5),
        &poly_in_out(0.5),
        1e-6,
    ); // d3-ease/test/poly-test.js:110:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.6),
        &poly_in_out(0.6),
        1e-6,
    ); // d3-ease/test/poly-test.js:111:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.7),
        &poly_in_out(0.7),
        1e-6,
    ); // d3-ease/test/poly-test.js:112:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.8),
        &poly_in_out(0.8),
        1e-6,
    ); // d3-ease/test/poly-test.js:113:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(0.9),
        &poly_in_out(0.9),
        1e-6,
    ); // d3-ease/test/poly-test.js:114:3
    assert_in_delta(
        &Ease::Poly {
            mode: Mode::InOut,
            exponent: 2.5,
        }
        .apply(1.0),
        &poly_in_out(1.0),
        1e-6,
    ); // d3-ease/test/poly-test.js:115:3
}

// d3-ease/test/quad-test.js: easeQuad is an alias for easeQuadInOut
#[test]
fn easequad_is_an_alias_for_easequadinout() {
    assert_eq!(Ease::Quad(Mode::InOut), Ease::Quad(Mode::InOut)); // d3-ease/test/quad-test.js:7:3
}

// d3-ease/test/quad-test.js: easeQuadIn(t) returns the expected results
#[test]
fn easequadin_t_returns_the_expected_results() {
    assert_eq!(Ease::Quad(Mode::In).apply(0.0), 0.00); // d3-ease/test/quad-test.js:11:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.1), &0.01, 1e-6); // d3-ease/test/quad-test.js:12:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.2), &0.04, 1e-6); // d3-ease/test/quad-test.js:13:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.3), &0.09, 1e-6); // d3-ease/test/quad-test.js:14:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.4), &0.16, 1e-6); // d3-ease/test/quad-test.js:15:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.5), &0.25, 1e-6); // d3-ease/test/quad-test.js:16:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.6), &0.36, 1e-6); // d3-ease/test/quad-test.js:17:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.7), &0.49, 1e-6); // d3-ease/test/quad-test.js:18:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.8), &0.64, 1e-6); // d3-ease/test/quad-test.js:19:3
    assert_in_delta(&Ease::Quad(Mode::In).apply(0.9), &0.81, 1e-6); // d3-ease/test/quad-test.js:20:3
    assert_eq!(Ease::Quad(Mode::In).apply(1.0), 1.00); // d3-ease/test/quad-test.js:21:3
}

// d3-ease/test/quad-test.js: easeQuadOut(t) returns the expected results
#[test]
fn easequadout_t_returns_the_expected_results() {
    let quad_out = out(Ease::Quad(Mode::In));
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.0), &quad_out(0.0), 1e-6); // d3-ease/test/quad-test.js:31:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.1), &quad_out(0.1), 1e-6); // d3-ease/test/quad-test.js:32:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.2), &quad_out(0.2), 1e-6); // d3-ease/test/quad-test.js:33:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.3), &quad_out(0.3), 1e-6); // d3-ease/test/quad-test.js:34:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.4), &quad_out(0.4), 1e-6); // d3-ease/test/quad-test.js:35:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.5), &quad_out(0.5), 1e-6); // d3-ease/test/quad-test.js:36:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.6), &quad_out(0.6), 1e-6); // d3-ease/test/quad-test.js:37:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.7), &quad_out(0.7), 1e-6); // d3-ease/test/quad-test.js:38:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.8), &quad_out(0.8), 1e-6); // d3-ease/test/quad-test.js:39:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(0.9), &quad_out(0.9), 1e-6); // d3-ease/test/quad-test.js:40:3
    assert_in_delta(&Ease::Quad(Mode::Out).apply(1.0), &quad_out(1.0), 1e-6); // d3-ease/test/quad-test.js:41:3
}

// d3-ease/test/quad-test.js: easeQuadInOut(t) returns the expected results
#[test]
fn easequadinout_t_returns_the_expected_results() {
    let quad_in_out = in_out(Ease::Quad(Mode::In));
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.0), &quad_in_out(0.0), 1e-6); // d3-ease/test/quad-test.js:51:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.1), &quad_in_out(0.1), 1e-6); // d3-ease/test/quad-test.js:52:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.2), &quad_in_out(0.2), 1e-6); // d3-ease/test/quad-test.js:53:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.3), &quad_in_out(0.3), 1e-6); // d3-ease/test/quad-test.js:54:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.4), &quad_in_out(0.4), 1e-6); // d3-ease/test/quad-test.js:55:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.5), &quad_in_out(0.5), 1e-6); // d3-ease/test/quad-test.js:56:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.6), &quad_in_out(0.6), 1e-6); // d3-ease/test/quad-test.js:57:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.7), &quad_in_out(0.7), 1e-6); // d3-ease/test/quad-test.js:58:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.8), &quad_in_out(0.8), 1e-6); // d3-ease/test/quad-test.js:59:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(0.9), &quad_in_out(0.9), 1e-6); // d3-ease/test/quad-test.js:60:3
    assert_in_delta(&Ease::Quad(Mode::InOut).apply(1.0), &quad_in_out(1.0), 1e-6); // d3-ease/test/quad-test.js:61:3
}

// d3-ease/test/sin-test.js: easeSin is an alias for easeSinInOut
#[test]
fn easesin_is_an_alias_for_easesininout() {
    assert_eq!(Ease::Sin(Mode::InOut), Ease::Sin(Mode::InOut)); // d3-ease/test/sin-test.js:7:3
}

// d3-ease/test/sin-test.js: easeSinIn(t) returns the expected results
#[test]
fn easesinin_t_returns_the_expected_results() {
    assert_eq!(Ease::Sin(Mode::In).apply(0.0), 0.000000); // d3-ease/test/sin-test.js:11:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.1), &0.012312, 1e-6); // d3-ease/test/sin-test.js:12:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.2), &0.048943, 1e-6); // d3-ease/test/sin-test.js:13:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.3), &0.108993, 1e-6); // d3-ease/test/sin-test.js:14:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.4), &0.190983, 1e-6); // d3-ease/test/sin-test.js:15:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.5), &0.292893, 1e-6); // d3-ease/test/sin-test.js:16:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.6), &0.412215, 1e-6); // d3-ease/test/sin-test.js:17:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.7), &0.546010, 1e-6); // d3-ease/test/sin-test.js:18:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.8), &0.690983, 1e-6); // d3-ease/test/sin-test.js:19:3
    assert_in_delta(&Ease::Sin(Mode::In).apply(0.9), &0.843566, 1e-6); // d3-ease/test/sin-test.js:20:3
    assert_eq!(Ease::Sin(Mode::In).apply(1.0), 1.000000); // d3-ease/test/sin-test.js:21:3
}

// d3-ease/test/sin-test.js: easeSinOut(t) returns the expected results
#[test]
fn easesinout_t_returns_the_expected_results() {
    let sin_out = out(Ease::Sin(Mode::In));
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.0), &sin_out(0.0), 1e-6); // d3-ease/test/sin-test.js:31:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.1), &sin_out(0.1), 1e-6); // d3-ease/test/sin-test.js:32:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.2), &sin_out(0.2), 1e-6); // d3-ease/test/sin-test.js:33:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.3), &sin_out(0.3), 1e-6); // d3-ease/test/sin-test.js:34:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.4), &sin_out(0.4), 1e-6); // d3-ease/test/sin-test.js:35:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.5), &sin_out(0.5), 1e-6); // d3-ease/test/sin-test.js:36:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.6), &sin_out(0.6), 1e-6); // d3-ease/test/sin-test.js:37:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.7), &sin_out(0.7), 1e-6); // d3-ease/test/sin-test.js:38:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.8), &sin_out(0.8), 1e-6); // d3-ease/test/sin-test.js:39:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(0.9), &sin_out(0.9), 1e-6); // d3-ease/test/sin-test.js:40:3
    assert_in_delta(&Ease::Sin(Mode::Out).apply(1.0), &sin_out(1.0), 1e-6); // d3-ease/test/sin-test.js:41:3
}

// d3-ease/test/sin-test.js: easeSinInOut(t) returns the expected results
#[test]
fn easesininout_t_returns_the_expected_results() {
    let sin_in_out = in_out(Ease::Sin(Mode::In));
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.0), &sin_in_out(0.0), 1e-6); // d3-ease/test/sin-test.js:51:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.1), &sin_in_out(0.1), 1e-6); // d3-ease/test/sin-test.js:52:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.2), &sin_in_out(0.2), 1e-6); // d3-ease/test/sin-test.js:53:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.3), &sin_in_out(0.3), 1e-6); // d3-ease/test/sin-test.js:54:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.4), &sin_in_out(0.4), 1e-6); // d3-ease/test/sin-test.js:55:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.5), &sin_in_out(0.5), 1e-6); // d3-ease/test/sin-test.js:56:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.6), &sin_in_out(0.6), 1e-6); // d3-ease/test/sin-test.js:57:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.7), &sin_in_out(0.7), 1e-6); // d3-ease/test/sin-test.js:58:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.8), &sin_in_out(0.8), 1e-6); // d3-ease/test/sin-test.js:59:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(0.9), &sin_in_out(0.9), 1e-6); // d3-ease/test/sin-test.js:60:3
    assert_in_delta(&Ease::Sin(Mode::InOut).apply(1.0), &sin_in_out(1.0), 1e-6); // d3-ease/test/sin-test.js:61:3
}
