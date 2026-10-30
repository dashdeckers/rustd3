//! Fitting: a fitted handle's domain spans its marks' admitted data,
//! padded 5% through core's `scale::Pad`, held at sticky values; held
//! handles skip it. The rows are matplotlib 3.11.2's `get_ylim` for the
//! same data, except where noted.

use rustd3_core::scale::{self, Continuous, Linear, Log};
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::mark;
use rustd3_figure::scale::{Fit, Kind, ScaleRef};
use rustd3_figure::{Cache, Figure, Held, Resolved, Target, axis};

use crate::figure::{Values, cache, line, resolve};

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
    // matplotlib gives ±0.055; a unit domain reads better.
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
fn visible_and_window_fit_as_all_on_a_handle_no_mark_places_across() {
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

#[cfg(not(debug_assertions))]
#[test]
fn a_foreign_handle_fits_unchecked_in_release_builds() {
    let mut first = Figure::new();
    let foreign = first.fit(scale::linear());
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(foreign, [0.0, 10.0]));
    // Same index and type: release builds skip the check, so the data
    // lands on this figure's scale.
    assert_domain(
        resolve(&fig, &Held::default()).scale(y).get_domain(),
        [-0.5, 10.5],
    );
}

/// The canvas padding on both sides at notebook size: 2 × 0.8 em.
const PAD: f64 = 2.0 * 0.8 * 44.0 / 3.0;

/// A canvas whose one panel without axes has a plot area 1000 × 240 px.
const BARE: Size = Size::new(1000.0 + PAD, 240.0 + PAD);

/// The screen at 1×: one thinning column per px.
const SCREEN: Target = Target::Screen { scale: 1.0 };

/// `n` samples at 10 kHz whose last x is exactly `last`, of a 7 Hz tone,
/// on an explicit clock.
fn tone(n: u32, last: f64) -> Recording<f64> {
    let x: Vec<f64> = (0..n).map(|i| last - f64::from(n - 1 - i) * 1e-4).collect();
    let y: Vec<f64> = x.iter().map(|t| (t * 44.0).sin()).collect();
    Recording::new(Clock::Explicit(x), [y]).expect("a valid recording")
}

/// Appends `n` samples of the tone after `rec`'s last, 1e-4 s apart.
fn append(rec: &mut Recording<f64>, n: u32) {
    let last = rec
        .series(0)
        .and_then(|s| s.get(s.len() - 1))
        .expect("a sample")
        .0;
    let x: Vec<f64> = (1..=n).map(|i| last + f64::from(i) * 1e-4).collect();
    let y: Vec<f64> = x.iter().map(|t| (t * 44.0).sin()).collect();
    rec.extend(&x, [y]).expect("a valid append");
}

/// A line over `rec` on an x handle showing its last 10 s (`Fit::Window`)
/// and a fixed y, with no axes; the x handle.
fn windowed(rec: &Recording<f64>) -> (Figure<'_>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let t = fig.fit_with(scale::linear(), Fit::Window(10.0));
    let v = fig.scale(scale::linear().domain([-2.0, 2.0]));
    fig.panel(0, 0)
        .mark(mark::line(rec.series(0).expect("a channel"), t, v));
    (fig, t)
}

/// `fig` resolved on [`BARE`] for the screen at 1× through `cache`; its
/// plot area is 1000 px wide.
fn bare(fig: &Figure<'_>, cache: &mut Cache) -> Resolved {
    let r = fig.resolve(BARE, cache, &Held::default(), SCREEN);
    assert_eq!(r.plot_areas()[0].width(), 1000.0);
    r
}

#[test]
fn fit_window_moves_in_whole_columns() {
    let mut rec = tone(10_000, 123.456);
    let mut cache = cache();
    let first = {
        let (fig, t) = windowed(&rec);
        bare(&fig, &mut cache).scale(t).get_domain()
    };
    // col = 10 / 1000 = 0.01; right = ceil(12,345.6) · 0.01.
    assert_domain(first, [113.46, 123.46]);
    append(&mut rec, 1000);
    let second = {
        let (fig, t) = windowed(&rec);
        bare(&fig, &mut cache).scale(t).get_domain()
    };
    assert_domain(second, [113.56, 123.56]);
    let moved = (second[1] - first[1]) / 0.01;
    assert!((moved - 10.0).abs() < 1e-9, "{moved} columns");
    // One sample more, at 123.5561: the window stays.
    rec.extend(&[123.5561], [[0.5]]).expect("a valid append");
    let (fig, t) = windowed(&rec);
    assert_eq!(bare(&fig, &mut cache).scale(t).get_domain(), second);
    // A fresh cache fits the same window.
    assert_eq!(
        bare(&fig, &mut crate::figure::cache())
            .scale(t)
            .get_domain(),
        second
    );
}

#[test]
fn fit_window_with_no_data_is_all() {
    let rec = Recording::new(Clock::Explicit(Vec::new()), [Vec::<f64>::new()]).expect("empty");
    let (fig, t) = windowed(&rec);
    assert_eq!(bare(&fig, &mut cache()).scale(t).get_domain(), [0.0, 1.0]);
}

#[test]
fn fit_window_column_edges_shift_by_whole_columns() {
    let mut rec = tone(100_000, 37.0001);
    let mut cache = cache();
    // The thinning columns' edges: the inverse of the device px edges.
    let mut edges = |rec: &Recording<f64>| -> Vec<f64> {
        let (fig, t) = windowed(rec);
        let r = bare(&fig, &mut cache);
        let x: Linear = r.scale(t).clone().range([0.0, 1000.0]);
        (0..=1000)
            .map(|i| Continuous::invert(&x, f64::from(i)))
            .collect()
    };
    let before = edges(&rec);
    append(&mut rec, 1000);
    let after = edges(&rec);
    let col = 0.01;
    let shift = (after[0] - before[0]) / col;
    let k = shift.round();
    println!("1,000 samples at 10 kHz moved the column edges by {shift} columns");
    assert!((shift - k).abs() < 1e-9 && k == 10.0, "{shift}");
    let k = k as usize;
    let worst = after[..=1000 - k]
        .iter()
        .zip(&before[k..])
        .map(|(a, b)| (a - b).abs() / col)
        .fold(0.0, f64::max);
    assert!(worst < 1e-9, "edges apart by {worst} columns");
}

#[test]
fn fit_window_on_a_y_handle_fits_as_all() {
    let rec = tone(1000, 5.0);
    let mut fig = Figure::new();
    let t = fig.fit(scale::linear());
    let v = fig.fit_with(scale::linear(), Fit::Window(0.5));
    fig.panel(0, 0)
        .mark(mark::line(rec.series(0).expect("a channel"), t, v));
    let r = bare(&fig, &mut cache());
    // The tone spans [−1, 1] within its 0.1 s: padded 5%.
    let [low, high] = r.scale(v).get_domain();
    assert!(low < -1.0 && high > 1.0, "{low}, {high}");
    assert_domain(r.scale(t).get_domain(), [4.895105, 5.004995]);
}

#[test]
fn a_window_follows_the_widest_panel_that_places_it() {
    let rec = tone(10_000, 123.456);
    let mut fig = Figure::new();
    let t = fig.fit_with(scale::linear(), Fit::Window(10.0));
    let v = fig.scale(scale::linear().domain([-2.0, 2.0]));
    let series = rec.series(0).expect("a channel");
    fig.panel(0, 0).mark(mark::line(series, t, v));
    fig.panel(0, 1).mark(mark::line(series, t, v));
    fig.layout(rustd3_figure::layout::Grid::new().cols([3.0, 1.0]));
    let r = fig.resolve(BARE, &mut cache(), &Held::default(), SCREEN);
    let widest = r.plot_areas()[0].width();
    assert!(widest > r.plot_areas()[1].width());
    let [left, right] = r.scale(t).get_domain();
    let col = 10.0 / widest;
    assert!(
        ((right / col) - (right / col).round()).abs() < 1e-6,
        "{right}"
    );
    assert!((right - left - 10.0).abs() < 1e-9 && right >= 123.456 && right - 123.456 < col);
}

#[test]
fn a_windowed_axis_holds_the_layout_while_ticks_slide() {
    // Ticks every second pass the right end as the window moves 0.1 s per
    // append; the axis reserves half a label at both ends all along.
    let mut rec = tone(10_000, 113.95);
    let mut cache = cache();
    let mut areas: Vec<Rect> = Vec::new();
    for _ in 0..20 {
        {
            let mut fig = Figure::new();
            let t = fig.fit_with(scale::linear(), Fit::Window(10.0));
            let v = fig.scale(scale::linear().domain([-2.0, 2.0]));
            fig.panel(0, 0)
                .mark(mark::line(rec.series(0).expect("a channel"), t, v))
                .axis(axis::left(v))
                .axis(axis::bottom(t));
            let size = Size::new(640.0, 300.0);
            let r = fig.resolve(size, &mut cache, &Held::default(), SCREEN);
            areas.push(r.plot_areas()[0]);
        }
        append(&mut rec, 1000);
    }
    assert!(areas.iter().all(|a| *a == areas[0]), "{areas:?}");
}
