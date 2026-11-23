//! The `Plot` widget through the headless harness on the backend
//! `ICED_TEST_BACKEND` names, against the PNG of the same figure. The
//! tests cover display and the overlay, `Plot::revision` and
//! `Plot::caching`, and live data. For live data they check the cells an
//! append, a moving window or `drop_front` re-renders, reading the probe's
//! raster samples against the parts that changed.

use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Color, Event, Point, Size, mouse};
use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::cache::Cache;
use rustd3_figure::list::{self, Cells, List, part};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::{Fit, ScaleRef};
use rustd3_figure::{Figure, Held, Target, Theme, axis, kurbo};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Caching, Probe, Render, cost};
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};
use crate::reference;

/// The figure's logical size.
const SIZE: Size = Size::new(480.0, 320.0);

/// The pixel bound of `backend` against the PNG: 1/255 on vello_cpu,
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
/// inline, and the backend's name. `None` when the backend is skipped.
/// Rendering is inline because the cost model predicts 8.7 ms for
/// [`figure`] at 2, over half the first frame's budget, so `Render::Auto`
/// would wait for the worker.
fn shoot(fig: Figure<'static>, k: f32) -> Option<(Shot, String)> {
    let plot = Plot::<()>::new(fig).scale_factor(k).render(Render::Inline);
    let mut harness = Harness::new(plot, SIZE, k)?;
    let backend = harness.backend();
    Some((harness.frame(headless::epoch()), backend))
}

/// Compares the widget's first frame of `make()` at `k` with the PNG of
/// the same figure on the screenshot's white, and prints the max
/// difference and the channels over 16. Returns the difference, the frame
/// and the backend's name.
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

/// One panel of a damped wave with left and bottom axes, as the README's
/// `Plot` app shows it, in a pure red ink.
fn wave_in_red() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.theme(Theme::default().ink(Rgb::hex(0xff0000)));
    let wave: Vec<(f64, f64)> = (0..=2000)
        .map(|i| {
            let t = f64::from(i) / 100.0;
            (t, (-t / 8.0).exp() * t.sin())
        })
        .collect();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(wave, x, y))
        .axis(axis::bottom(x).label("time [s]"))
        .axis(axis::left(y));
    fig
}

/// Whether `pixel` is the red ink, as a spine draws it.
fn is_red(pixel: [u8; 4]) -> bool {
    pixel[0] > 200 && pixel[1] < 60 && pixel[2] < 60
}

/// The columns (`vertical`) or rows of `shot` within `span` that are red
/// along at least 90% of `along`. These are a spine's, which must form one
/// run.
fn spine(
    shot: &Shot,
    vertical: bool,
    span: std::ops::Range<u32>,
    along: std::ops::Range<u32>,
) -> Vec<u32> {
    let n = along.len();
    span.filter(|&a| {
        let red = along
            .clone()
            .filter(|&b| {
                let (x, y) = if vertical { (a, b) } else { (b, a) };
                is_red(shot.pixel(x, y))
            })
            .count();
        red * 10 >= n * 9
    })
    .collect()
}

#[test]
fn spines_have_no_seam_whatever_factor_the_widget_assumes() {
    // The widget is told either the window's factor (right) or another
    // one (wrong: 1 on a scaled window, 2 on an unscaled one). The wrong
    // case is what a headless capture without `.scale_factor` gets. wgpu
    // snaps the scaled images to device pixels. The software renderer
    // resamples them and antialiases their edges
    // (`vello_cpu/src/raster.rs:46-79`), which blends where cells meet, so
    // only the right factor is checked there.
    let mut seams = Vec::new();
    for k in [1.0_f32, 1.25, 1.5, 2.0] {
        for assumed in [k, if k == 1.0 { 2.0 } else { 1.0 }] {
            let plot = Plot::<()>::new(wave_in_red())
                .scale_factor(assumed)
                .render(Render::Inline);
            let Some(mut harness) = Harness::new(plot, SIZE, k) else {
                return;
            };
            let backend = harness.backend();
            if assumed != k && backend != "wgpu" {
                continue;
            }
            let shot = harness.frame(headless::epoch());
            let (w, h) = (shot.width, shot.height);
            let left = spine(&shot, true, 0..w / 3, h / 5..h * 3 / 5);
            let bottom = spine(&shot, false, h / 2..h, w / 3..w * 4 / 5);
            for (side, lines) in [("left", left), ("bottom", bottom)] {
                let run = lines.last().zip(lines.first()).map(|(l, f)| l - f + 1);
                let seen =
                    format!("{backend}, k = {k}, assumed {assumed}: {side} spine at {lines:?}");
                println!("{seen}");
                if lines.is_empty() || run != Some(lines.len() as u32) {
                    seams.push(seen);
                }
            }
        }
    }
    assert!(seams.is_empty(), "{seams:#?}");
}

/// Panel `i`'s plot area of `fig` at `k`, in device px.
fn plot_area(fig: &Figure<'_>, k: f32, i: usize) -> [u32; 4] {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let resolved = fig.resolve(size, &mut Cache::default(), &Held::default(), target);
    let area = resolved.get_plot_areas()[i];
    [area.x0, area.y0, area.x1, area.y1].map(|v| (v * f64::from(k)).round() as u32)
}

/// The device pixels where two frames differ.
fn changed(before: &Shot, after: &Shot) -> Vec<(u32, u32)> {
    let width = before.width;
    before
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.rgba.as_chunks::<4>().0.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| (i as u32 % width, i as u32 / width))
        .collect()
}

/// Two frames of `plot` at `k`, rendered inline, before and after the
/// pointer moves to `at` (logical). Also returns the backend's name and
/// whether the move asked for a redraw.
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
    // (5, 5) at the center of the plot area; the pointer 5 px right and 4
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

/// The cells the raster samples among `samples` rendered for `fig` at
/// `size` and `k`, increasing and each once (see [`spans`]).
pub fn rendered(samples: &[Sample], fig: &Figure<'_>, size: Size, k: f32) -> Vec<u32> {
    let mut rendered: Vec<u32> = spans(samples, fig, size, k).concat();
    rendered.sort_unstable();
    rendered.dedup();
    rendered
}

/// The cells each raster sample among `samples` rendered for `fig` at
/// `size` and `k`. A tile counts as the cell holding its plot area's top
/// left pixel, since each plot area of these figures is one cell. A gutter
/// run reports its first cell and its pixels, and covers the cells after it
/// in its row until their areas reach that pixel count. An overlay reports
/// the plot area's cell, which is larger than the overlay.
fn spans(samples: &[Sample], fig: &Figure<'_>, size: Size, k: f32) -> Vec<Vec<u32>> {
    let cells = partition(fig, size, k).1;
    let areas = plot_areas(fig, size, k).len();
    let of: Vec<u32> = (0..areas)
        .map(|panel| area_cell(fig, size, k, &cells, panel))
        .collect();
    let area = |c: u32| {
        let r = cells.cells[c as usize].rect;
        ((r.x1 - r.x0) * (r.y1 - r.y0)) as u64
    };
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Cell(cell),
                pixels,
                ..
            } => {
                let mut run = vec![*cell];
                let mut covered = area(*cell);
                while covered < u64::from(*pixels) {
                    let next = run[run.len() - 1] + 1;
                    covered += area(next);
                    run.push(next);
                }
                Some(run)
            }
            Sample::Raster {
                target: probe::Target::Tile { area, .. },
                ..
            } => Some(vec![of[*area as usize]]),
            Sample::Stage { .. } | Sample::Frame { .. } | Sample::Mismatch { .. } => None,
        })
        .collect()
}

/// The plot areas of `fig` at `size` on the screen at `k`, list px.
fn plot_areas(fig: &Figure<'_>, size: Size, k: f32) -> Vec<kurbo::Rect> {
    let size = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    fig.resolve(size, &mut Cache::default(), &Held::default(), target)
        .get_plot_areas()
        .to_vec()
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
    let fig = figure();
    let cells_of =
        |rx: &mpsc::Receiver<Sample>| rendered(&rx.try_iter().collect::<Vec<_>>(), &fig, SIZE, k);
    assert_eq!(cells_of(&rx), all, "{backend}");
    // The same revision and an equal figure: nothing renders.
    harness.rebuild(plot(1));
    let _ = harness.frame(t + Duration::from_millis(16));
    let same = cells_of(&rx);
    // A new revision: every cell, once.
    harness.rebuild(plot(2));
    let _ = harness.frame(t + Duration::from_millis(32));
    let revised = cells_of(&rx);
    harness.rebuild(plot(2));
    let _ = harness.frame(t + Duration::from_millis(48));
    let after = cells_of(&rx);
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
    for caching in [Caching::On, Caching::Off, Caching::Verify] {
        let Some(mut harness) = Harness::new(plot((4.0, 5.0), caching), SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let t = headless::epoch();
        let _ = harness.frame(t);
        let fig = fixed(eight((4.0, 5.0)));
        let cells_of = |rx: &mpsc::Receiver<Sample>| {
            rendered(&rx.try_iter().collect::<Vec<_>>(), &fig, SIZE, k)
        };
        assert_eq!(cells_of(&rx), all, "{backend}");
        harness.rebuild(plot((4.0, 6.5), caching));
        let _ = harness.frame(t + Duration::from_millis(16));
        let moved = cells_of(&rx);
        harness.rebuild(plot((4.0, 6.5), caching));
        let _ = harness.frame(t + Duration::from_millis(32));
        let unchanged = cells_of(&rx);
        println!(
            "{caching:?}, {backend}: a moved dot renders {moved:?} of {} cells; an equal list {unchanged:?}",
            all.len()
        );
        // Verify renders as On; its checks report no raster sample.
        match caching {
            Caching::On | Caching::Verify => assert_eq!(moved, [inside], "{backend}"),
            Caching::Off => assert_eq!(moved, all, "{backend}"),
            _ => unreachable!("the modes this test runs"),
        }
        assert!(unchanged.is_empty(), "{backend}: {unchanged:?}");
    }
}

/// Samples per second of the live recordings.
const RATE: f64 = 10_000.0;

/// Samples one append adds: 0.1 s at 10 kHz.
const APPEND: usize = 1_000;

/// Samples `from..from + n` of live channel `seed`: a sine of 1, 2 or
/// 0.5 Hz and amplitude 0.9, 0.7 or 0.5. Its sampled peaks are exactly
/// ± the amplitude (each peak's argument rounds to within an ulp of
/// ±π/2 + 2πk, where the sine rounds to ±1) and one of each lies in every
/// two seconds, so neither appends nor `drop_front` move a fitted y. It is
/// smooth, so no thinned stroke overlaps itself (where vello_cpu's output
/// would depend on the render's origin).
fn wave(from: usize, n: usize, seed: usize) -> Vec<f64> {
    let (hz, amplitude) = [(1.0, 0.9), (2.0, 0.7), (0.5, 0.5)][seed % 3];
    (from..from + n)
        .map(|i| amplitude * (std::f64::consts::TAU * hz * (i as f64 / RATE)).sin())
        .collect()
}

/// `n` samples at 10 kHz from t = 0 on live channels `seeds`, on a
/// regular clock.
fn recording(n: usize, seeds: std::ops::Range<usize>) -> Recording<f64> {
    let clock = Clock::Regular {
        start: 0.0,
        period: 1.0 / RATE,
    };
    Recording::new(clock, seeds.map(|seed| wave(0, n, seed))).expect("a valid recording")
}

/// Appends `n` samples to every channel of `rec`, whose channels are live
/// channels `seeds`, continuing their waves.
fn append(rec: &mut Recording<f64>, seeds: std::ops::Range<usize>, n: usize) {
    let from = usize::try_from(rec.get_offset()).expect("a small offset") + rec.len();
    let channels: Vec<Vec<f64>> = seeds.map(|seed| wave(from, n, seed)).collect();
    rec.extend(&[], channels).expect("a valid append");
}

/// The live channels of the upper recording.
const TOP: std::ops::Range<usize> = 0..1;

/// The live channels of the lower recording.
const BOTTOM: std::ops::Range<usize> = 1..3;

/// How a live trace's x follows its data.
#[derive(Debug, Clone, Copy)]
enum Follow {
    /// A domain the app sets.
    Fixed([f64; 2]),
    /// The newest span of data (`Fit::Window`).
    Window(f64),
}

/// The live trace and its x handle. Every channel of `top` is in the upper
/// panel and every channel of `bottom` in the lower one, in the palette's
/// colors. The panels share x as `follow` says, and each fits its y to its
/// data. Both have left and bottom axes (the upper bottom axis
/// unlabeled), and the upper panel zooms x.
fn trace<'a>(
    top: &'a Recording<f64>,
    bottom: &'a Recording<f64>,
    follow: Follow,
) -> (Figure<'a>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let t = match follow {
        Follow::Fixed(domain) => fig.scale(scale::linear().domain(domain)),
        Follow::Window(span) => fig.fit_with(scale::linear(), Fit::Window(span)),
    };
    let (upper, lower) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let color = fig.scale(scale::ordinal([0_usize, 1, 2], COLORBLIND));
    let panel = fig.panel(0, 0);
    for c in 0..top.channels() {
        panel.mark(line(top.series(c).expect("a channel"), t, upper).stroke(color, c));
    }
    panel
        .axis(axis::left(upper))
        .axis(axis::bottom(t).labels(false))
        .zoom_x(t);
    let panel = fig.panel(1, 0);
    for c in 0..bottom.channels() {
        let channel = bottom.series(c).expect("a channel");
        panel.mark(line(channel, t, lower).stroke(color, top.channels() + c));
    }
    panel
        .axis(axis::left(lower))
        .axis(axis::bottom(t).label("time [s]"));
    (fig, t)
}

/// The widget over [`trace`] at factor `k`, rendering as `render`,
/// without the overlay, reporting to `probe` if given.
fn live<'a>(
    top: &'a Recording<f64>,
    bottom: &'a Recording<f64>,
    follow: Follow,
    k: f32,
    render: Render,
    probe: Option<&Probe>,
) -> Plot<'a, ()> {
    let plot = Plot::new(trace(top, bottom, follow).0)
        .scale_factor(k)
        .render(render)
        .crosshair(false)
        .highlight(false);
    match probe {
        Some(probe) => plot.probe(probe.clone()),
        None => plot,
    }
}

/// The id of part `index` of `role` in panel `panel` (`role << 56 |
/// panel << 32 | index`): 4 for axes, 6 for marks.
fn part_id(role: u64, panel: u64, index: u64) -> part::Id {
    part::Id::new(role << 56 | panel << 32 | index)
}

/// The ids of the parts of `after` that `before` does not have with the
/// same hash, and of those of `before` that `after` lacks, sorted.
fn changed_parts(before: &List, after: &List) -> Vec<part::Id> {
    let differ = |a: &List, b: &List| -> Vec<part::Id> {
        a.parts
            .iter()
            .filter(|p| !b.parts.iter().any(|q| q.id == p.id && q.hash == p.hash))
            .map(|p| p.id)
            .collect()
    };
    let mut ids = differ(after, before);
    ids.extend(differ(before, after));
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// The cells of `cells` (cut from `list`) that list one of the parts
/// `ids`, in order.
fn listing(cells: &Cells, list: &List, ids: &[part::Id]) -> Vec<u32> {
    (0_u32..)
        .zip(&cells.cells)
        .filter(|(_, cell)| {
            cell.parts
                .iter()
                .any(|&p| ids.contains(&list.parts[p as usize].id))
        })
        .map(|(i, _)| i)
        .collect()
}

/// The cell of `cells` holding the top left device pixel of panel
/// `panel`'s plot area of `fig` at `size` and `k`.
fn area_cell(fig: &Figure<'_>, size: Size, k: f32, cells: &Cells, panel: usize) -> u32 {
    let size = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let area = fig
        .resolve(size, &mut Cache::default(), &Held::default(), target)
        .get_plot_areas()[panel];
    let at = [area.x0, area.y0].map(|v| (v * f64::from(k)).round() as i32 + 1);
    cells
        .cell_at(at)
        .and_then(|i| u32::try_from(i).ok())
        .expect("a plot area is a cell")
}

/// The frame `shot` against the PNG of `fig` at `size` and `k` on white.
fn against(shot: &Shot, fig: &Figure<'_>, size: Size, k: f32) -> pixels::Diff {
    let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let png = reference::png_of(fig, logical, f64::from(k), Color::WHITE);
    let png = pixels::Image::new(png.width, png.height, &png.rgba)
        .expect("invariant: a render holds width × height pixels");
    pixels::Diff::between(&shot.image(), &png, 16).expect("invariant: the shot has the PNG's size")
}

/// The live figures' logical size.
const LIVE: Size = Size::new(640.0, 400.0);

#[test]
fn live_append_re_renders_only_touched_cells() {
    let k = 2.0;
    // Channel 0 alone in its own recording; x set wider than the data.
    let (mut top, bottom) = (recording(100_000, TOP), recording(100_000, BOTTOM));
    let follow = Follow::Fixed([0.0, 20.0]);
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
    let Some(mut harness) = Harness::new(plot, LIVE, k) else {
        return;
    };
    let backend = harness.backend();
    let t = headless::epoch();
    let _ = harness.frame(t);
    let first = rendered(
        &rx.try_iter().collect::<Vec<_>>(),
        &trace(&top, &bottom, follow).0,
        LIVE,
        k,
    );
    let (before, _) = partition(&trace(&top, &bottom, follow).0, LIVE, k);
    let parked = harness.park();
    let revision = top.get_revision();
    append(&mut top, TOP, APPEND);
    assert_eq!(top.get_revision(), revision + 1);
    let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
    let mut harness = parked.resume(plot);
    let shot = harness.frame(t + Duration::from_millis(100));
    let samples: Vec<Sample> = rx.try_iter().collect();
    let (fig, _) = trace(&top, &bottom, follow);
    let after = rendered(&samples, &fig, LIVE, k);
    let (list, cells) = partition(&fig, LIVE, k);
    let changed = changed_parts(&before, &list);
    let cell = area_cell(&fig, LIVE, k, &cells, 0);
    println!(
        "append to channel 0, {backend}: {} cells first, then {after:?} (panel 0's plot area is cell {cell}); parts changed {changed:?}",
        first.len()
    );
    assert_eq!(first.len(), cells.cells.len(), "{backend}");
    assert_eq!(
        changed,
        [part_id(6, 0, 0)],
        "{backend}: only channel 0's line"
    );
    assert_eq!(after, [cell], "{backend}");
    let diff = against(&shot, &fig, LIVE, k);
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
}

#[test]
fn fit_window_follows_in_whole_device_columns() {
    let follow = Follow::Window(10.0);
    for k in [1.0, 1.25, 2.0] {
        let (mut top, mut bottom) = (recording(100_000, TOP), recording(100_000, BOTTOM));
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
        let Some(mut harness) = Harness::new(plot, LIVE, k) else {
            return;
        };
        let backend = harness.backend();
        let t = headless::epoch();
        let mut last: Option<([f64; 2], List, Cells)> = None;
        let mut runs: Vec<Vec<u32>> = Vec::new();
        // Every line and both x axes; the y axes and the rest stay.
        let moving = [
            part_id(4, 0, 1),
            part_id(4, 1, 1),
            part_id(6, 0, 0),
            part_id(6, 1, 0),
            part_id(6, 1, 1),
        ];
        for step in 0..=10_u32 {
            if step > 0 {
                let parked = harness.park();
                append(&mut top, TOP, APPEND);
                append(&mut bottom, BOTTOM, APPEND);
                let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
                harness = parked.resume(plot);
            }
            let shot = harness.frame(t + Duration::from_millis(100) * step);
            let samples: Vec<Sample> = rx.try_iter().collect();
            let (fig, x) = trace(&top, &bottom, follow);
            let cells_rendered = rendered(&samples, &fig, LIVE, k);
            if step == 0 {
                runs = spans(&samples, &fig, LIVE, k);
            }
            let target = Target::Screen {
                scale: f64::from(k),
            };
            let size = kurbo::Size::new(f64::from(LIVE.width), f64::from(LIVE.height));
            let resolved = fig.resolve(size, &mut Cache::default(), &Held::default(), target);
            let domain = resolved.get_scale(x).unwrap().get_domain();
            let columns = (resolved.get_plot_areas()[0].width() * f64::from(k)).round();
            let col = 10.0 / columns;
            let (list, cells) = partition(&fig, LIVE, k);
            let diff = against(&shot, &fig, LIVE, k);
            assert!(
                diff.max() <= bound(&backend),
                "{backend}, k = {k}, append {step}: max {}/255",
                diff.max()
            );
            if let Some((previous, before, before_cells)) = &last {
                let moved = (domain[1] - previous[1]) / col;
                let changed = changed_parts(before, &list);
                // The cells the moving parts are in now or were in (a tick
                // leaving the plot area's end takes its pixels in the
                // gutter beside it along), and the cells merged with them.
                let mut touched = listing(&cells, &list, &moving);
                touched.extend(listing(before_cells, before, &moving));
                let mut expected: Vec<u32> = runs
                    .iter()
                    .filter(|run| run.iter().any(|c| touched.contains(c)))
                    .flatten()
                    .copied()
                    .collect();
                expected.sort_unstable();
                expected.dedup();
                println!(
                    "window, {backend}, k = {k}, append {step}: [{:.4}, {:.4}] moved {moved} of {columns} columns; rendered {cells_rendered:?}",
                    domain[0], domain[1]
                );
                assert!(
                    (moved - moved.round()).abs() < 1e-9 && moved >= 1.0,
                    "{backend}, k = {k}: {moved} columns"
                );
                assert!(
                    (moved - columns / 100.0).abs() < 1.0,
                    "{backend}, k = {k}: {moved} columns of {columns}"
                );
                if columns == 1000.0 {
                    assert_eq!(moved.round(), 10.0, "{backend}, k = {k}");
                }
                assert_eq!(changed, moving, "{backend}, k = {k}, append {step}");
                assert_eq!(
                    cells_rendered, expected,
                    "{backend}, k = {k}, append {step}"
                );
            } else {
                assert_eq!(
                    cells_rendered.len(),
                    cells.cells.len(),
                    "{backend}, k = {k}"
                );
            }
            last = Some((domain, list, cells));
        }
    }
}

#[test]
fn ten_hz_appends_stay_sharp() {
    // 800 × 500 at 2: plot areas about 1,400 × 760 device px.
    let (size, k) = (Size::new(800.0, 500.0), 2.0);
    let follow = Follow::Window(10.0);
    let (mut top, mut bottom) = (recording(1_000_000, TOP), recording(1_000_000, BOTTOM));
    // What an append changes, as the cost model predicts it, against half
    // of the longest frame budget.
    let (before, _) = partition(&trace(&top, &bottom, follow).0, size, k);
    let mut grown = (recording(1_000_000, TOP), recording(1_000_000, BOTTOM));
    append(&mut grown.0, TOP, APPEND);
    append(&mut grown.1, BOTTOM, APPEND);
    let (after, cells) = partition(&trace(&grown.0, &grown.1, follow).0, size, k);
    let changed = changed_parts(&before, &after);
    let touched = listing(&cells, &after, &changed);
    let (pixels, work) = touched.iter().fold((0, 0), |(pixels, work), &i| {
        let cell = &cells.cells[i as usize];
        let area = (cell.rect.width() * cell.rect.height()) as u64;
        (pixels + area, work + cell.work)
    });
    let predicted = cost::model_ms(pixels, work);
    println!(
        "ten appends a second: an append changes {} parts and {} of {} cells, {pixels} device px and {work} items: {predicted:.2} ms predicted",
        changed.len(),
        touched.len(),
        cells.cells.len()
    );
    assert!(predicted <= 16.7 / 2.0, "{predicted} ms");
    drop(grown);
    // Where the renders run is forced. The widget fits its cost model to
    // its own renders, here a debug build's, so `Render::Auto`'s choice
    // would depend on the machine; the prediction above is the seed
    // model's. Frames are 100 ms apart.
    let plot = live(&top, &bottom, follow, k, Render::Inline, None);
    let Some(mut harness) = Harness::new(plot, size, k) else {
        return;
    };
    let backend = harness.backend();
    let t = headless::epoch();
    let _ = harness.frame(t);
    let mut worst = 0;
    for step in 1..=10_u32 {
        let parked = harness.park();
        append(&mut top, TOP, APPEND);
        append(&mut bottom, BOTTOM, APPEND);
        harness = parked.resume(live(&top, &bottom, follow, k, Render::Inline, None));
        let shot = harness.frame(t + Duration::from_millis(100) * step);
        // Every changed cell and tile rendered in the frame, so it is the
        // sharp one; a preview would be shifted. It differs from the PNG
        // only where strokes cross, where vello_cpu's output depends on a
        // cell's origin.
        let diff = against(&shot, &trace(&top, &bottom, follow).0, size, k);
        worst = worst.max(diff.max());
        assert_eq!(
            diff.over(),
            0,
            "{backend}, append {step}: max {}/255",
            diff.max()
        );
    }
    println!("ten appends a second, {backend}: ten sharp frames (max {worst}/255 against the PNG)");
}

#[test]
fn drop_front_re_renders_the_cell() {
    let k = 2.0;
    let (mut top, bottom) = (recording(150_000, TOP), recording(100_000, BOTTOM));
    let follow = Follow::Fixed([0.0, 20.0]);
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
    let Some(mut harness) = Harness::new(plot, LIVE, k) else {
        return;
    };
    let backend = harness.backend();
    let t = headless::epoch();
    let _ = harness.frame(t);
    let _ = rx.try_iter().count();
    let (before, _) = partition(&trace(&top, &bottom, follow).0, LIVE, k);
    let parked = harness.park();
    let revision = top.get_revision();
    assert_eq!(top.drop_front(65_536), 65_536);
    assert_eq!(top.get_revision(), revision + 1);
    let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
    let mut harness = parked.resume(plot);
    let shot = harness.frame(t + Duration::from_millis(100));
    let samples: Vec<Sample> = rx.try_iter().collect();
    let (fig, _) = trace(&top, &bottom, follow);
    let after = rendered(&samples, &fig, LIVE, k);
    let (list, cells) = partition(&fig, LIVE, k);
    let changed = changed_parts(&before, &list);
    let cell = area_cell(&fig, LIVE, k, &cells, 0);
    // A fresh recording of the retained samples draws the same pixels.
    let series = top.series(0).expect("a channel");
    let (x, y): (Vec<f64>, Vec<f64>) = (0..series.len())
        .map(|i| series.get(i).expect("a sample"))
        .unzip();
    let fresh = Recording::new(Clock::Explicit(x), [y]).expect("a valid recording");
    let diff = against(&shot, &trace(&fresh, &bottom, follow).0, LIVE, k);
    println!(
        "drop_front of 65,536 samples, {backend}: rendered {after:?} (panel 0's plot area is cell {cell}); parts changed {changed:?}; against a fresh recording max {}/255",
        diff.max()
    );
    assert_eq!(changed, [part_id(6, 0, 0)], "{backend}");
    assert_eq!(after, [cell], "{backend}");
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
}

#[test]
fn a_gesture_holds_a_windowed_handle() {
    let k = 1.0;
    let follow = Follow::Window(10.0);
    let (mut top, mut bottom) = (recording(100_000, TOP), recording(100_000, BOTTOM));
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let area = {
        let (fig, _) = trace(&top, &bottom, follow);
        let size = kurbo::Size::new(f64::from(LIVE.width), f64::from(LIVE.height));
        let target = Target::Screen { scale: 1.0 };
        fig.resolve(size, &mut Cache::default(), &Held::default(), target)
            .get_plot_areas()[0]
    };
    let plot = live(&top, &bottom, follow, k, Render::Inline, Some(&probe));
    let Some(mut harness) = Harness::new(plot, LIVE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let frame = Duration::from_millis(16);
    let fresh = harness.frame(t0);
    // Drag the upper panel 40 px right, then let the layout settle.
    let from = Point::new(area.center().x as f32, area.center().y as f32);
    let left = mouse::Button::Left;
    let moved = |at: Point| Event::Mouse(mouse::Event::CursorMoved { position: at });
    harness.point_at(from);
    let _ = harness.events(&[moved(from), Event::Mouse(mouse::Event::ButtonPressed(left))]);
    let _ = harness.frame(t0 + frame);
    for (i, dx) in [20.0, 40.0].into_iter().enumerate() {
        let at = Point::new(from.x + dx, from.y);
        harness.point_at(at);
        let _ = harness.events(&[moved(at)]);
        let _ = harness.frame(t0 + frame * (i as u32 + 2));
    }
    let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(left))]);
    let _ = harness.frame(t0 + frame * 4);
    let t1 = t0 + Duration::from_secs(1);
    let dragged = harness.frame(t1);
    assert_ne!(dragged.rgba, fresh.rgba, "{backend}: the drag moved x");
    let _ = rx.try_iter().count();
    // An append: the held window stays, and no cell renders.
    let parked = harness.park();
    append(&mut top, TOP, APPEND);
    append(&mut bottom, BOTTOM, APPEND);
    harness = parked.resume(live(&top, &bottom, follow, k, Render::Inline, Some(&probe)));
    let held = harness.frame(t1 + Duration::from_millis(100));
    let samples: Vec<Sample> = rx.try_iter().collect();
    let (fig, _) = trace(&top, &bottom, follow);
    let after = rendered(&samples, &fig, LIVE, k);
    let (_, cells) = partition(&fig, LIVE, k);
    let areas = [0, 1].map(|panel| area_cell(&fig, LIVE, k, &cells, panel));
    println!("held window, {backend}: an append renders {after:?}");
    // The x domain stays: no axis moves, and no gutter cell renders. The
    // samples appended past the view lie in the upper plot area's ring,
    // whose tiles render again; the frame shows the same pixels.
    assert!(
        after.iter().all(|cell| areas.contains(cell)),
        "{backend}: {after:?}"
    );
    assert_eq!(held.rgba, dragged.rgba, "{backend}");
    // A double click over the panel: the window follows the data again.
    let click = |harness: &mut Harness<'_, ()>| {
        let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonPressed(left))]);
        let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(left))]);
    };
    let t2 = t1 + Duration::from_millis(200);
    click(&mut harness);
    let _ = harness.frame(t2);
    click(&mut harness);
    let _ = harness.frame(t2 + Duration::from_millis(200));
    let reset = harness.frame(t2 + Duration::from_secs(1));
    let diff = against(&reset, &trace(&top, &bottom, follow).0, LIVE, k);
    println!(
        "held window, {backend}: after a double click max {}/255 against the windowed figure",
        diff.max()
    );
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
    assert_ne!(reset.rgba, held.rgba, "{backend}");
}

/// Draws one inline frame of `plot` at `size` logical px on a factor-1
/// host. Does nothing when the backend is skipped.
fn one_frame(plot: Plot<'static, ()>, size: Size) {
    let Some(mut harness) = Harness::new(plot, size, 1.0) else {
        return;
    };
    let now = headless::epoch();
    let _ = harness.events(&[Event::Window(iced::window::Event::RedrawRequested(now))]);
    harness.draw();
}

#[test]
fn a_widget_wider_than_the_renderer_draws_renders_at_the_factor_that_fits() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(line(vec![(0.0, 0.0), (1.0, 1.0)], x, y))
        .axis(axis::bottom(x))
        .axis(axis::left(y));
    // 80,000 device px wide at 2: its axis strip is wider than any render.
    let plot = Plot::new(fig).render(Render::Inline).scale_factor(2.0);
    one_frame(plot, Size::new(40_000.0, 200.0));
}

#[test]
fn a_factor_of_a_billion_renders_at_eight() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).axis(axis::bottom(x));
    let plot = Plot::new(fig).render(Render::Inline).scale_factor(1e9);
    one_frame(plot, SIZE);
}

/// A host that draws `content` scaled by `scale` and then moved by
/// `offset` logical px, as a pan-and-zoom surface draws a card. Layout
/// and events pass through unchanged.
struct Surface<W> {
    content: W,
    scale: f32,
    offset: iced::Vector,
}

impl<W> iced::advanced::widget::Meta for Surface<W> {}

impl<W, Message> iced::advanced::Widget<Message, iced::Theme, iced::Renderer> for Surface<W>
where
    W: iced::advanced::Widget<Message, iced::Theme, iced::Renderer>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut iced::advanced::widget::Tree) {
        self.content.diff(tree);
    }

    fn size(&self) -> Size<iced::Length> {
        self.content.size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        _limits: &iced::advanced::layout::Limits,
    ) {
        // The content is as large as it asks, whatever the window shows.
        let limits = iced::advanced::layout::Limits::new(Size::ZERO, Size::INFINITE);
        self.content.layout(tree, renderer, &limits);
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &Event,
        layout: iced::advanced::Layout,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;

        let transformation = iced::Transformation::translate(self.offset.x, self.offset.y)
            * iced::Transformation::scale(self.scale);
        renderer.with_transformation(transformation, |renderer| {
            self.content
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        });
    }
}

/// [`figure`] at [`SIZE`] in a window of factor `k`, magnified by `z` on
/// a [`Surface`] moved by `offset` device px. Returns the part of the
/// first frame the plot covers, the same plot's first frame drawn
/// directly at `k * z`, and the backend's name.
fn magnified(k: f32, z: f32, offset: [f32; 2]) -> Option<(Shot, Shot, String)> {
    let plot = |factor, magnification| {
        Plot::<()>::new(figure())
            .width(SIZE.width)
            .height(SIZE.height)
            .scale_factor(factor)
            .magnification(magnification)
            .render(Render::Inline)
    };
    let mut direct = Harness::new(plot(k * z, 1.0), SIZE, k * z)?;
    let direct = direct.frame(headless::epoch());

    let surface = Surface {
        content: plot(k, z),
        scale: z,
        offset: iced::Vector::new(offset[0] / k, offset[1] / k),
    };
    // Room for the moved plot, in whole device px.
    let [left, top] = offset.map(|d| d.floor() as u32);
    let window = Size::new(
        (direct.width + left + 1) as f32 / k,
        (direct.height + top + 1) as f32 / k,
    );
    let mut harness = Harness::new(surface, window, k)?;
    let backend = harness.backend();
    let whole = harness.frame(headless::epoch());
    let mut rgba = Vec::with_capacity(direct.rgba.len());
    for y in top..top + direct.height {
        for x in left..left + direct.width {
            rgba.extend_from_slice(&whole.pixel(x, y));
        }
    }
    let shown = Shot {
        width: direct.width,
        height: direct.height,
        rgba,
    };
    Some((shown, direct, backend))
}

#[test]
fn a_magnified_plot_matches_the_plot_drawn_at_the_product() {
    // Above and below 1, at whole and fractional products, at the origin
    // and at a whole device offset.
    for (k, z, offset) in [
        (2.0, 1.25, [0.0, 0.0]),
        (2.0, 0.75, [0.0, 0.0]),
        (1.25, 1.3, [7.0, 3.0]),
        (1.25, 0.7, [7.0, 3.0]),
    ] {
        let Some((shown, direct, backend)) = magnified(k, z, offset) else {
            return;
        };
        let diff = pixels::Diff::between(&shown.image(), &direct.image(), 16)
            .expect("invariant: the crop has the direct frame's size");
        println!(
            "magnified against direct, {backend}, k = {k}, z = {z}, offset {offset:?}: \
             max {}/255, {} channels over 16",
            diff.max(),
            diff.over()
        );
        assert!(
            diff.max() <= bound(&backend),
            "{backend}, k = {k}, z = {z}: max {}/255",
            diff.max()
        );
    }
}

#[test]
fn a_magnification_that_is_not_finite_and_positive_draws_as_one() {
    let Some((plain, _)) = shoot(figure(), 2.0) else {
        return;
    };
    for bad in [0.0, -1.5, f32::NAN, f32::INFINITY] {
        let plot = Plot::<()>::new(figure())
            .scale_factor(2.0)
            .magnification(bad)
            .render(Render::Inline);
        let Some(mut harness) = Harness::new(plot, SIZE, 2.0) else {
            return;
        };
        let shot = harness.frame(headless::epoch());
        assert_eq!(shot.rgba, plain.rgba, "{bad}");
    }
}

#[test]
fn a_magnified_factor_above_eight_renders_at_eight() {
    let at = |factor, magnification| {
        let plot = Plot::<()>::new(figure())
            .scale_factor(factor)
            .magnification(magnification)
            .render(Render::Inline);
        let mut harness = Harness::new(plot, Size::new(120.0, 80.0), 8.0)?;
        Some(harness.frame(headless::epoch()))
    };
    let (Some(capped), Some(eight)) = (at(4.0, 2.5), at(8.0, 1.0)) else {
        return;
    };
    assert_eq!(capped.rgba, eight.rgba);
}
