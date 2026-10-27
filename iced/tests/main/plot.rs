//! The `Plot` widget through the headless harness on the backend
//! `ICED_TEST_BACKEND` names, against the PNG of the same figure.

use std::time::Duration;

use iced::{Color, Event, Point, Size, mouse};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::kurbo;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Cache, Figure, Held, Target, Theme, axis};
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

/// [`figure`] on a background of `#336699` at 50%: gutters and labels
/// translucent, plot areas opaque.
fn translucent() -> Figure<'static> {
    let mut fig = figure();
    fig.theme(Theme::default().background(Rgb::hex(0x336699).with_opacity(0.5)));
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

/// The widget's first frame of `make()` at `k` against the PNG of the
/// same figure on the screenshot's white: max difference and channels
/// over 16, printed.
fn compare(make: fn() -> Figure<'static>, k: f32) -> Option<(pixels::Diff, Shot, String)> {
    let (shot, backend) = shoot(make(), k)?;
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let png = reference::png_of(&make(), size, f64::from(k), Color::WHITE);
    assert_eq!((shot.width, shot.height), (png.width, png.height));
    let reference = pixels::Image::new(png.width, png.height, &png.rgba)
        .expect("invariant: a render holds width × height pixels");
    let diff = pixels::Diff::between(&shot.image(), &reference, 16)
        .expect("invariant: the shot has the PNG's size");
    println!(
        "plot against its PNG, {backend}, k = {k}, corner {:?}: max {}/255, {} channels over 16 ({})",
        shot.pixel(1, 1),
        diff.max(),
        diff.over(),
        headless::features()
    );
    Some((diff, shot, backend))
}

#[test]
fn plot_matches_the_png_of_its_figure() {
    let Some((diff, _, backend)) = compare(figure, 2.0) else {
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
        let Some((diff, _, backend)) = compare(figure, k) else {
            return;
        };
        assert!(
            diff.max() <= bound(&backend),
            "{backend}, k = {k}: max {}/255",
            diff.max()
        );
    }
}

#[test]
fn translucent_background_matches_the_png() {
    let Some((diff, shot, backend)) = compare(translucent, 2.0) else {
        return;
    };
    assert!(diff.max() <= 4, "{backend}: max {}/255", diff.max());
    // The gutter shows #336699 at 50% over the white under the widget.
    let corner = shot.pixel(1, 1);
    let expected = [153, 178, 204, 255];
    let off = corner
        .iter()
        .zip(expected)
        .map(|(a, b)| a.abs_diff(b))
        .max();
    assert!(off <= Some(1), "{backend}: corner {corner:?}");
}

/// [`figure`] in a pure red ink: axes, labels and the overlay.
fn red() -> Figure<'static> {
    let mut fig = figure();
    fig.theme(Theme::default().ink(Rgb::hex(0xff0000)));
    fig
}

/// Four dots on fixed [0, 10] scales in one panel, one in its bottom left
/// corner, in a red ink.
fn four_dots() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.theme(Theme::default().ink(Rgb::hex(0xff0000)));
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(dots(
            vec![(0.0, 0.0), (2.0, 2.0), (5.0, 5.0), (8.0, 3.0)],
            x,
            y,
        ))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// Panel `i`'s plot area of `fig` at `k`, in device px.
fn plot_area(fig: &Figure<'_>, k: f32, i: usize) -> [u32; 4] {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let resolved = fig.resolve(size, &mut Cache::default(), &Held::default(), target);
    let area = resolved.plot_areas()[i];
    [area.x0, area.y0, area.x1, area.y1].map(|v| (v * f64::from(k)).round() as u32)
}

/// The device pixels where two frames differ.
fn changed(before: &Shot, after: &Shot) -> Vec<(u32, u32)> {
    let width = before.width;
    before
        .rgba
        .chunks_exact(4)
        .zip(after.rgba.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| (i as u32 % width, i as u32 / width))
        .collect()
}

/// A frame of `plot` at `k`, then one after the pointer moved to `at`
/// (logical), the backend's name and whether the move asked for a redraw.
fn hover(plot: Plot<'static, ()>, k: f32, at: Point) -> Option<(Shot, Shot, String, bool)> {
    let mut harness = Harness::new(plot.scale_factor(k), SIZE, k)?;
    let backend = harness.backend();
    let t = headless::epoch();
    let before = harness.frame(t);
    harness.point_at(at);
    harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
    let asked = harness.redraw_requested() == iced::window::RedrawRequest::NextFrame;
    let after = harness.frame(t + Duration::from_millis(16));
    Some((before, after, backend, asked))
}

#[test]
fn crosshair_stays_inside_the_hovered_panel() {
    for k in [2.0, 1.25] {
        let [x0, y0, x1, y1] = plot_area(&red(), k, 0);
        let centre = Point::new((x0 + x1) as f32 / 2.0 / k, (y0 + y1) as f32 / 2.0 / k);
        let Some((before, after, backend, asked)) = hover(Plot::new(red()), k, centre) else {
            return;
        };
        assert!(asked, "{backend}: a hover that changes the overlay redraws");
        let overlay = changed(&before, &after);
        let outside: Vec<_> = overlay
            .iter()
            .filter(|&&(x, y)| !(x0..x1).contains(&x) || !(y0..y1).contains(&y))
            .collect();
        println!(
            "crosshair, {backend}, k = {k}: {} overlay px, {} outside panel 0's plot area",
            overlay.len(),
            outside.len()
        );
        assert!(
            outside.is_empty(),
            "{backend}, k = {k}: {:?}",
            &outside[..outside.len().min(8)]
        );
        // Both lines cross the whole plot area, one device px wide, in red.
        let (cx, cy) = ((centre.x * k).floor() as u32, (centre.y * k).floor() as u32);
        let column = overlay.iter().filter(|&&(x, _)| x == cx).count();
        let row = overlay.iter().filter(|&&(_, y)| y == cy).count();
        assert!(
            column as u32 >= (y1 - y0) * 9 / 10,
            "{backend}, k = {k}: column {column}"
        );
        assert!(
            row as u32 >= (x1 - x0) * 9 / 10,
            "{backend}, k = {k}: row {row}"
        );
        let [r, g, b, _] = after.pixel(cx, y0 + 2);
        assert!(
            r > g && r > b,
            "{backend}, k = {k}: crosshair {r}, {g}, {b}"
        );
    }
}

#[test]
fn highlight_marks_the_nearest_point() {
    let k = 2.0;
    let [x0, y0, x1, y1] = plot_area(&four_dots(), k, 0);
    // (5, 5) at the centre of the plot area; the pointer 5 px right and 4
    // px up of it, logical.
    let expected = ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
    let at = Point::new(expected.0 / k + 5.0, expected.1 / k - 4.0);
    let plot = Plot::new(four_dots()).crosshair(false);
    let Some((before, after, backend, _)) = hover(plot, k, at) else {
        return;
    };
    let ring = changed(&before, &after);
    assert!(!ring.is_empty(), "{backend}: no ring");
    let (lo, hi) = ring
        .iter()
        .fold(((u32::MAX, u32::MAX), (0, 0)), |(lo, hi), &(x, y)| {
            ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)))
        });
    let centre = (
        (lo.0 + hi.0 + 1) as f32 / 2.0,
        (lo.1 + hi.1 + 1) as f32 / 2.0,
    );
    let off = (centre.0 - expected.0).hypot(centre.1 - expected.1);
    println!(
        "highlight, {backend}: ring {} px from {lo:?} to {hi:?}, centre {centre:?}, expected {expected:?}, {off:.3} device px off",
        ring.len()
    );
    assert!(
        off <= 1.0,
        "{backend}: centre {centre:?}, expected {expected:?}"
    );
    // The ring is the point's, not the nearest dot elsewhere: 8 logical px
    // across at 2, plus its border.
    assert!(
        hi.0 - lo.0 <= 20 && hi.1 - lo.1 <= 20,
        "{backend}: {lo:?} … {hi:?}"
    );
}

#[test]
fn the_ring_around_a_corner_point_stays_inside_the_plot_area() {
    for k in [2.0, 1.25] {
        let [x0, y0, x1, y1] = plot_area(&four_dots(), k, 0);
        // (0, 0) on the bottom left corner; the pointer 3 px inside.
        let at = Point::new(x0 as f32 / k + 3.0, y1 as f32 / k - 3.0);
        let Some((before, after, backend, _)) = hover(Plot::new(four_dots()), k, at) else {
            return;
        };
        let overlay = changed(&before, &after);
        let outside = overlay
            .iter()
            .filter(|&&(x, y)| !(x0..x1).contains(&x) || !(y0..y1).contains(&y))
            .count();
        // The ring's quarter inside the area shows: pixels within 6 logical
        // px of the corner beyond the crosshair's lines.
        let (cx, cy) = ((at.x * k).floor() as u32, (at.y * k).floor() as u32);
        let reach = (6.0 * k) as u32;
        let ring = overlay
            .iter()
            .filter(|&&(x, y)| x != cx && y != cy && x < x0 + reach && y + reach > y1)
            .count();
        println!(
            "corner ring, {backend}, k = {k}: {ring} ring px inside, {outside} overlay px outside"
        );
        assert_eq!(outside, 0, "{backend}, k = {k}");
        assert!(ring > 0, "{backend}, k = {k}: no ring");
    }
}
