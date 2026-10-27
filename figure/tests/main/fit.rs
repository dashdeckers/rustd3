//! Fitting: a fitted handle's domain spans its marks' admitted data,
//! padded 5% through core's `scale::Pad`, held at sticky values; held
//! handles skip it. The rows are matplotlib 3.11.2's `get_ylim` for the
//! same data, except where noted.

use rustd3_core::scale::{self, Log};
use rustd3_figure::scale::{Fit, Kind, ScaleRef};
use rustd3_figure::{Figure, Held};

use crate::figure::{Values, line, resolve};

/// `a` within 1e-12 of `b`, relative.
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
}

/// Asserts `domain` equals `expected` within 1e-12, relative.
fn assert_domain(domain: [f64; 2], expected: [f64; 2]) {
    assert!(
        close(domain[0], expected[0]) && close(domain[1], expected[1]),
        "{domain:?}, expected {expected:?}"
    );
}

/// The y domain fitted with `fit` to a mark of `values`, with a sticky
/// base at 0 for `bars`.
fn fitted(values: &[f64], bars: bool, fit: Fit<f64>) -> [f64; 2] {
    let mut fig = Figure::new();
    let y = fig.fit_with(scale::linear(), fit);
    fig.panel(0, 0).mark(Values {
        on: y,
        values: values.to_vec(),
        base: bars.then_some(0.0),
    });
    resolve(&fig, &Held::default()).scale(y).get_domain()
}

/// The domain of a fitted log handle over `values`.
fn fitted_log(values: &[f64]) -> [f64; 2] {
    let mut fig = Figure::new();
    let y: ScaleRef<Log> = fig.fit(scale::log());
    fig.panel(0, 0).mark(line(y, values.iter().copied()));
    resolve(&fig, &Held::default()).scale(y).get_domain()
}

#[test]
fn a_flat_line_is_widened_by_nonsingular_then_padded() {
    // nonsingular [4.75, 5.25], then ±0.025.
    assert_domain(fitted(&[5.0, 5.0], false, Fit::All), [4.725, 5.275]);
}

#[test]
fn a_flat_line_at_zero_spans_a_tenth() {
    assert_domain(fitted(&[0.0, 0.0], false, Fit::All), [-0.055, 0.055]);
}

#[test]
fn a_line_is_padded_by_five_percent_per_side() {
    assert_domain(fitted(&[0.13, 9.71], false, Fit::All), [-0.349, 10.189]);
}

#[test]
fn lines_are_not_sticky() {
    assert_domain(fitted(&[0.0, 3.0], false, Fit::All), [-0.15, 3.15]);
}

#[test]
fn a_line_across_zero_is_padded_on_both_sides() {
    assert_domain(fitted(&[-2.0, 8.0], false, Fit::All), [-2.5, 8.5]);
}

#[test]
fn a_line_far_from_zero_keeps_its_precision() {
    assert_domain(
        fitted(&[1e9, 1e9 + 1.0], false, Fit::All),
        [999_999_999.95, 1_000_000_001.05],
    );
}

#[test]
fn bars_stick_at_their_base() {
    assert_domain(fitted(&[3.0, 7.0], true, Fit::All), [0.0, 7.35]);
}

#[test]
fn nan_is_not_admitted() {
    assert_domain(fitted(&[f64::NAN, 4.0], false, Fit::All), [3.78, 4.22]);
}

#[test]
fn no_data_gives_the_unit_domain() {
    // matplotlib gives ±0.055 (D10.2 (5)).
    assert_eq!(fitted(&[], false, Fit::All), [0.0, 1.0]);
}

#[test]
fn a_log_handle_pads_in_decades() {
    assert_domain(
        fitted_log(&[1.0, 10.0, 100.0, 1000.0]),
        [10f64.powf(-0.15), 10f64.powf(3.15)],
    );
    assert_eq!(
        fitted_log(&[1.0, 1000.0]).map(|v| format!("{v:.5}")),
        ["0.70795", "1412.53754"]
    );
}

#[test]
fn a_log_handle_does_not_admit_negative_values() {
    let domain = fitted_log(&[-1.0, 10.0, 1000.0]);
    assert_domain(domain, [10f64.powf(0.9), 10f64.powf(3.1)]);
    assert_eq!(domain.map(|v| format!("{v:.2}")), ["7.94", "1258.93"]);
}

#[test]
fn a_held_handle_skips_fitting_until_released() {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(y, (0..=10).map(f64::from)));
    let mut held = Held::default();
    held.hold(y, scale::linear().domain([2.0, 4.0]));
    assert_eq!(resolve(&fig, &held).scale(y).get_domain(), [2.0, 4.0]);
    held.release(y);
    assert_domain(resolve(&fig, &held).scale(y).get_domain(), [-0.5, 10.5]);
}

#[test]
fn visible_and_window_fit_as_all_for_now() {
    assert_domain(fitted(&[0.0, 3.0], false, Fit::Visible), [-0.15, 3.15]);
    assert_domain(fitted(&[0.0, 3.0], false, Fit::Window(5.0)), [-0.15, 3.15]);
}

#[test]
fn one_handle_spans_the_marks_of_every_panel() {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(y, [0.0, 1.0]));
    fig.panel(0, 1).mark(line(y, [9.0, 10.0]));
    let r = resolve(&fig, &Held::default());
    assert_domain(r.scale(y).get_domain(), [-0.5, 10.5]);
    assert_eq!(Kind::domain(r.scale(y)), Some(r.scale(y).get_domain()));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Fit::Window on a categorical scale")]
fn fit_with_window_on_a_band_handle_panics() {
    let mut fig = Figure::new();
    fig.fit_with(scale::band(["a", "b"]), Fit::Window(()));
}
