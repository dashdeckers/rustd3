//! The `Plot` widget through the headless harness on the backend
//! `ICED_TEST_BACKEND` names, against the PNG of the same figure.

use iced::{Color, Size};
use rustd3_core::scale;
use rustd3_figure::kurbo;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Figure, axis};
use rustd3_iced::Plot;
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};
use crate::reference;

/// The figure's logical size.
const SIZE: Size = Size::new(480.0, 320.0);

/// The pixel bound of `backend` against the PNG: 1/255 on tiny-skia,
/// 2/255 on wgpu.
fn bound(backend: &str) -> u8 {
    if backend == "wgpu" { 2 } else { 1 }
}

/// Two panels side by side under a title: a line of 10k samples and 2k
/// dots, each with left and bottom axes.
fn figure() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.title("Line and dots");
    let (x0, y0) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let (x1, y1) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(samples(10_000, 0), x0, y0))
        .axis(axis::left(y0))
        .axis(axis::bottom(x0));
    fig.panel(0, 1)
        .mark(dots(samples(2_000, 1), x1, y1))
        .axis(axis::left(y1))
        .axis(axis::bottom(x1));
    fig
}

/// `n` samples of a sine with a deterministic wobble, phase `seed`.
fn samples(n: usize, seed: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = ((i * 7919 + seed * 104_729) % 1000) as f64 / 5000.0;
            (t * 10.0, (t * 25.0 + seed as f64).sin() + wobble)
        })
        .collect()
}

/// The first frame of `fig` in the widget at factor `k`, and the
/// backend's name; `None` when the backend is skipped.
fn shoot(fig: Figure<'static>, k: f32) -> Option<(Shot, String)> {
    let plot = Plot::<()>::new(fig).scale_factor(k);
    let mut harness = Harness::new(plot, SIZE, k)?;
    let backend = harness.backend();
    Some((harness.frame(headless::epoch()), backend))
}

/// The widget's first frame at `k` against the PNG of the same figure on
/// the screenshot's white: max difference and channels over 16, printed.
fn compare(k: f32) -> Option<(pixels::Diff, String)> {
    let (shot, backend) = shoot(figure(), k)?;
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let png = reference::png_of(&figure(), size, f64::from(k), Color::WHITE);
    assert_eq!((shot.width, shot.height), (png.width, png.height));
    let reference = pixels::Image::new(png.width, png.height, &png.rgba)
        .expect("invariant: a render holds width × height pixels");
    let diff = pixels::Diff::between(&shot.image(), &reference, 16)
        .expect("invariant: the shot has the PNG's size");
    println!(
        "plot against its PNG, {backend}, k = {k}: max {}/255, {} channels over 16 ({})",
        diff.max(),
        diff.over(),
        headless::features()
    );
    Some((diff, backend))
}

#[test]
fn plot_matches_the_png_of_its_figure() {
    let Some((diff, backend)) = compare(2.0) else {
        return;
    };
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
}

#[test]
fn plot_places_cells_on_device_pixels_at_fractional_scales() {
    for k in [1.25, 1.5] {
        let Some((diff, backend)) = compare(k) else {
            return;
        };
        assert!(
            diff.max() <= bound(&backend),
            "{backend}, k = {k}: max {}/255",
            diff.max()
        );
    }
}
