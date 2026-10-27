//! The `Plot` widget through the headless harness on the backend
//! `ICED_TEST_BACKEND` names, against the PNG of the same figure.

use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Color, Event, Point, Size, mouse};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::kurbo;
use rustd3_figure::list::{self, Cells, List};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Cache, Figure, Held, Target, Theme, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Caching, Probe, Render};
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

/// The first frame of `fig` in the widget at factor `k`, rendered
/// inline, and the backend's name; `None` when the backend is skipped.
/// (The cost model predicts 8.7 ms for [`figure`] at 2, over half the
/// first frame's budget, so `Render::Auto` would wait for the worker.)
fn shoot(fig: Figure<'static>, k: f32) -> Option<(Shot, String)> {
    let plot = Plot::<()>::new(fig).scale_factor(k).render(Render::Inline);
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

/// A frame of `plot` at `k`, rendered inline, then one after the pointer
/// moved to `at` (logical), the backend's name and whether the move asked
/// for a redraw.
fn hover(plot: Plot<'static, ()>, k: f32, at: Point) -> Option<(Shot, Shot, String, bool)> {
    let plot = plot.scale_factor(k).render(Render::Inline);
    let mut harness = Harness::new(plot, SIZE, k)?;
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

/// The clock a test's probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn clock() -> Instant {
    Instant::now()
}

/// The cells the raster samples among `samples` rendered, in order.
pub fn rendered(samples: &[Sample]) -> Vec<u32> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Cell(cell),
                ..
            } => Some(*cell),
            Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .collect()
}

/// `fig` drawn at `size` logical px for the screen at `k` as the widget
/// draws it, and its cells.
pub fn partition(fig: &Figure<'_>, size: Size, k: f32) -> (List, Cells) {
    let size = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let k = f64::from(k);
    let mut cache = Cache::default();
    cache.begin();
    let resolved = fig.resolve(
        size,
        &mut cache,
        &Held::default(),
        Target::Screen { scale: k },
    );
    let drawn = fig.draw_resolved(&mut cache, &resolved);
    let (xs, ys) = resolved.edges();
    let cells = list::cells(&drawn, &xs, &ys, k);
    (drawn, cells)
}

/// Dots at `points` on fixed [0, 10] scales in one panel with left and
/// bottom axes.
fn fixed(points: Vec<(f64, f64)>) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(dots(points, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// Eight dots on a falling diagonal, the fourth at `fourth`.
fn eight(fourth: (f64, f64)) -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
    points[3] = fourth;
    points
}

#[test]
fn revision_change_re_renders_everything() {
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = |revision: u64| {
        Plot::<()>::new(figure())
            .scale_factor(k)
            .render(Render::Inline)
            .probe(probe.clone())
            .revision(revision)
    };
    let cells = partition(&figure(), SIZE, k).1.cells.len();
    let all: Vec<u32> = (0..u32::try_from(cells).unwrap()).collect();
    let Some(mut harness) = Harness::new(plot(1), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t = headless::epoch();
    let _ = harness.frame(t);
    assert_eq!(
        rendered(&rx.try_iter().collect::<Vec<_>>()),
        all,
        "{backend}"
    );
    // The same revision and an equal figure: nothing renders.
    harness.rebuild(plot(1));
    let _ = harness.frame(t + Duration::from_millis(16));
    let same = rendered(&rx.try_iter().collect::<Vec<_>>());
    // A new revision: every cell, once.
    harness.rebuild(plot(2));
    let _ = harness.frame(t + Duration::from_millis(32));
    let revised = rendered(&rx.try_iter().collect::<Vec<_>>());
    harness.rebuild(plot(2));
    let _ = harness.frame(t + Duration::from_millis(48));
    let after = rendered(&rx.try_iter().collect::<Vec<_>>());
    println!(
        "revision 1 → 2, {backend}: {cells} cells; {} rendered at the same revision, {} at the new one, {} after",
        same.len(),
        revised.len(),
        after.len()
    );
    assert!(same.is_empty(), "{backend}: {same:?}");
    assert_eq!(revised, all, "{backend}");
    assert!(after.is_empty(), "{backend}: {after:?}");
}

#[test]
fn caching_off_renders_every_cell() {
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = |fourth: (f64, f64), caching: Caching| {
        Plot::<()>::new(fixed(eight(fourth)))
            .scale_factor(k)
            .render(Render::Inline)
            .probe(probe.clone())
            .caching(caching)
    };
    let cells = partition(&fixed(eight((4.0, 5.0))), SIZE, k).1;
    let all: Vec<u32> = (0..u32::try_from(cells.cells.len()).unwrap()).collect();
    // The plot area's cell, which a moved dot changes.
    let area = plot_area(&fixed(eight((4.0, 5.0))), k, 0);
    let inside = cells
        .cell_at([area[0] as i32 + 1, area[1] as i32 + 1])
        .and_then(|i| u32::try_from(i).ok())
        .expect("the plot area is a cell");
    for caching in [Caching::On, Caching::Off] {
        let Some(mut harness) = Harness::new(plot((4.0, 5.0), caching), SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let t = headless::epoch();
        let _ = harness.frame(t);
        assert_eq!(
            rendered(&rx.try_iter().collect::<Vec<_>>()),
            all,
            "{backend}"
        );
        harness.rebuild(plot((4.0, 6.5), caching));
        let _ = harness.frame(t + Duration::from_millis(16));
        let moved = rendered(&rx.try_iter().collect::<Vec<_>>());
        harness.rebuild(plot((4.0, 6.5), caching));
        let _ = harness.frame(t + Duration::from_millis(32));
        let unchanged = rendered(&rx.try_iter().collect::<Vec<_>>());
        println!(
            "{caching:?}, {backend}: a moved dot renders {moved:?} of {} cells; an equal list {unchanged:?}",
            all.len()
        );
        match caching {
            Caching::On => assert_eq!(moved, [inside], "{backend}"),
            Caching::Off => assert_eq!(moved, all, "{backend}"),
        }
        assert!(unchanged.is_empty(), "{backend}: {unchanged:?}");
    }
}
