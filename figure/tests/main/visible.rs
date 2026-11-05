//! `Fit::Visible`: y fitted to what x shows, held still by eighths.

use rustd3_core::scale::{self, Linear};
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::Size;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::{Fit, ScaleRef};
use rustd3_figure::{Figure, Held, Target};

use crate::figure::cache;

/// The ramp y = x at x = 0, 1, …, 100.
fn ramp() -> Vec<(f64, f64)> {
    (0..=100).map(|i| (f64::from(i), f64::from(i))).collect()
}

/// `fig`'s domain of `y` resolved through `cache` with `held`.
fn y_domain(fig: &Figure<'_>, y: ScaleRef<Linear>, cache: &mut Cache, held: &Held) -> [f64; 2] {
    let size = Size::new(640.0, 480.0);
    fig.resolve(size, cache, held, Target::Screen { scale: 1.0 })
        .get_scale(y)
        .expect("a handle of this figure")
        .get_domain()
}

/// Asserts that `a` equals `b` within 1e-9.
fn near(a: [f64; 2], b: [f64; 2]) {
    assert!(
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
        "{a:?} {b:?}"
    );
}

/// The ramp spot rows on `fig`, whose x is `x` over [0, 100] and whose y
/// is `y`, fitted to what x shows.
fn steps(fig: &Figure<'_>, x: ScaleRef<Linear>, y: ScaleRef<Linear>) {
    let mut cache = cache();
    let mut held = Held::default();
    near(y_domain(fig, y, &mut cache, &held), [-5.0, 105.0]);
    // x zoomed to [40, 60]: the padded extent [39, 61] fits after three
    // steps of 110 / 8 = 13.75 per end, in one resolve.
    held.hold(x, scale::linear().domain([40.0, 60.0]));
    near(y_domain(fig, y, &mut cache, &held), [36.25, 63.75]);
    // The next step, 27.5 / 8 = 3.4375, would cut into [39, 61]: it stays.
    near(y_domain(fig, y, &mut cache, &held), [36.25, 63.75]);
    near(y_domain(fig, y, &mut cache, &held), [36.25, 63.75]);
    // x back to [0, 100]: y grows at once.
    held.release(x);
    near(y_domain(fig, y, &mut cache, &held), [-5.0, 105.0]);
}

#[test]
fn a_visible_y_steps_toward_what_x_shows_and_grows_at_once() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0).mark(dots(ramp(), x, y));
    steps(&fig, x, y);
}

#[test]
fn a_visible_y_reads_a_recording_through_the_index() {
    let (xs, ys): (Vec<f64>, Vec<f64>) = ramp().into_iter().unzip();
    let rec = Recording::new(Clock::Explicit(xs), [ys]).expect("a recording");
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("channel 0"), x, y));
    steps(&fig, x, y);
}

#[test]
fn panels_sharing_a_visible_y_merge_what_their_x_scales_show() {
    let mut fig = Figure::new();
    let left = fig.scale(scale::linear().domain([10.0, 20.0]));
    let right = fig.scale(scale::linear().domain([70.0, 80.0]));
    let y = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0).mark(dots(ramp(), left, y));
    fig.panel(0, 1).mark(dots(ramp(), right, y));
    // [10, 80] padded by 5% of 70.
    near(
        y_domain(&fig, y, &mut cache(), &Held::default()),
        [6.5, 83.5],
    );
}

#[test]
fn visible_on_x_fits_all_and_a_y_over_a_visible_x_fits_all() {
    let mut fig = Figure::new();
    let x = fig.fit_with(scale::linear(), Fit::Visible);
    let y = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0).mark(dots(ramp(), x, y));
    let r = fig.resolve(
        Size::new(640.0, 480.0),
        &mut cache(),
        &Held::default(),
        Target::Vector,
    );
    near(r.get_scale(x).unwrap().get_domain(), [-5.0, 105.0]);
    near(r.get_scale(y).unwrap().get_domain(), [-5.0, 105.0]);
}

#[test]
fn with_nothing_in_view_a_visible_y_keeps_its_last_domain() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0).mark(dots(ramp(), x, y));
    let mut cache = cache();
    let mut held = Held::default();
    near(y_domain(&fig, y, &mut cache, &held), [-5.0, 105.0]);
    held.hold(x, scale::linear().domain([200.0, 300.0]));
    near(y_domain(&fig, y, &mut cache, &held), [-5.0, 105.0]);
}
