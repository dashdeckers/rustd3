//! Phase 31's invalidation by dependency through the headless harness on
//! the backend `ICED_TEST_BACKEND` names: a pan keeps what does not depend
//! on the panned handle, a style change during a pan re-renders the plot
//! area, and gutter cells that change together merge into one image.

use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Color, Event, Point, mouse};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::list;
use rustd3_figure::mark::line;
use rustd3_figure::{Cache, Figure, Held, Target, axis, kurbo};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Probe, Render};
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};
use crate::reference;
use crate::tiles::{self, bound, clock};

/// One frame at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// The curve the figures draw.
fn curve() -> Vec<(f64, f64)> {
    (0..=400)
        .map(|i| {
            let t = f64::from(i) * 0.25;
            (t, (t * 0.11).sin() * (t * 0.023).cos())
        })
        .collect()
}

/// Two panels side by side, each a curve over its own x handle (both at
/// `x`) with left and bottom axes; each panel zooms its own x.
fn pair(x: [f64; 2]) -> Figure<'static> {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([-1.2, 1.2]));
    for col in 0..2 {
        let hx = fig.scale(scale::linear().domain(x));
        fig.panel(0, col)
            .mark(line(curve(), hx, y))
            .axis(axis::left(y))
            .axis(axis::bottom(hx))
            .zoom_x(hx);
    }
    fig
}

/// One panel: the curve over x `x` in `color`, with left and bottom axes;
/// the panel zooms x.
fn colored(x: [f64; 2], color: Rgb) -> Figure<'static> {
    let mut fig = Figure::new();
    let hx = fig.scale(scale::linear().domain(x));
    let hy = fig.scale(scale::linear().domain([-1.2, 1.2]));
    fig.panel(0, 0)
        .mark(line(curve(), hx, hy).color(color).width(2.0))
        .axis(axis::left(hy))
        .axis(axis::bottom(hx))
        .zoom_x(hx);
    fig
}

/// The widget over `fig` at `k`, inline, without the overlay, reporting
/// to `probe`.
fn plot(fig: Figure<'static>, k: f32, probe: &Probe) -> Plot<'static, ()> {
    Plot::new(fig)
        .scale_factor(k)
        .render(Render::Inline)
        .crosshair(false)
        .highlight(false)
        .probe(probe.clone())
}

/// The raster samples' targets among `samples`.
fn targets(samples: &[Sample]) -> Vec<probe::Target> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster { target, .. } => Some(*target),
            Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .collect()
}

/// The images of the last frame sample among `samples`.
fn images(samples: &[Sample]) -> u32 {
    samples
        .iter()
        .rev()
        .find_map(|sample| match sample {
            Sample::Frame { images, .. } => Some(*images),
            Sample::Stage { .. } | Sample::Raster { .. } => None,
        })
        .expect("a frame sample")
}

/// `shot` against `reference`, a render of the same size.
fn against(shot: &Shot, reference: &rustd3_figure::raster::Image) -> u8 {
    let png = pixels::Image::new(reference.width, reference.height, &reference.rgba)
        .expect("invariant: a render holds width × height pixels");
    pixels::Diff::between(&shot.image(), &png, 16)
        .expect("invariant: the shot has the reference's size")
        .max()
}

/// Presses at `from` (logical px), then moves to `to`, a frame each from
/// `t0`; the samples of the move's frame and its screenshot.
fn drag(
    harness: &mut Harness<'_, ()>,
    rx: &mpsc::Receiver<Sample>,
    (from, to): (Point, Point),
    t0: Instant,
) -> (Vec<Sample>, Shot) {
    harness.point_at(from);
    let _ = harness.events(&[
        Event::Mouse(mouse::Event::CursorMoved { position: from }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    ]);
    let _ = harness.frame(t0 + FRAME);
    let _ = rx.try_iter().count();
    harness.point_at(to);
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: to })]);
    let shot = harness.frame(t0 + FRAME * 2);
    (rx.try_iter().collect(), shot)
}

#[test]
fn unchanged_panels_are_kept() {
    // A drag over the left panel pans its own x: the right panel, its
    // tiles and the gutter cells that show only its axes, render nothing.
    let k = 1.0;
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let Some(mut harness) = Harness::new(plot(pair([0.0, 100.0]), k, &probe), tiles::BIG, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let first = targets(&rx.try_iter().collect::<Vec<_>>());
    let areas = tiles::areas(&pair([0.0, 100.0]), tiles::BIG, k);
    let from = Point::new(
        ((areas[0][0] + areas[0][2]) / 2) as f32 / k,
        ((areas[0][1] + areas[0][3]) / 2) as f32 / k,
    );
    let to = Point::new(from.x + 37.0, from.y);
    let (samples, _) = drag(&mut harness, &rx, (from, to), t0);
    let rendered = targets(&samples);
    let rows = (areas[0][3] - areas[0][1] + 255) / 256;
    let right: Vec<_> = rendered
        .iter()
        .filter(|t| matches!(t, probe::Target::Tile { area: 1, .. }))
        .collect();
    let left: Vec<_> = rendered
        .iter()
        .filter(|t| matches!(t, probe::Target::Tile { area: 0, .. }))
        .collect();
    let cells: HashSet<u32> = rendered
        .iter()
        .filter_map(|t| match t {
            probe::Target::Cell(c) => Some(*c),
            probe::Target::Tile { .. } => None,
        })
        .collect();
    let gutters: HashSet<u32> = first
        .iter()
        .filter_map(|t| match t {
            probe::Target::Cell(c) => Some(*c),
            probe::Target::Tile { .. } => None,
        })
        .collect();
    println!(
        "a drag over the left of two panels, {backend}: {} left tiles, {} right tiles, {} of {} gutter and overlay slots render",
        left.len(),
        right.len(),
        cells.len(),
        gutters.len()
    );
    assert!(right.is_empty(), "{backend}: {right:?}");
    assert_eq!(
        left.len() as i64,
        rows + 2,
        "{backend}: the entering column of the left area"
    );
    assert!(
        !cells.is_empty() && cells.len() < gutters.len(),
        "{backend}: {cells:?}"
    );
}

#[test]
fn style_change_during_a_pan_re_renders() {
    // A drag by whole pixels translates the tiles; when the figure comes
    // back with another color mid-drag, its positions moved but its style
    // hash did not stay: every tile of the area renders, and the frame is
    // the new color's at the snapped domain.
    let blue = Rgb::new(30.0, 60.0, 140.0);
    let red = Rgb::new(200.0, 30.0, 30.0);
    for k in [1.0_f32, 2.0] {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let size = tiles::SIZE;
        let Some(mut harness) = Harness::new(plot(colored([0.0, 100.0], blue), k, &probe), size, k)
        else {
            return;
        };
        let backend = harness.backend();
        let t0 = headless::epoch();
        let _ = harness.frame(t0);
        let all = tiles::tiles(&rx.try_iter().collect::<Vec<_>>()).len();
        let area = tiles::areas(&colored([0.0, 100.0], blue), size, k)[0];
        let from = Point::new(
            (area[0] + area[2]) as f32 / 2.0 / k,
            (area[1] + area[3]) as f32 / 2.0 / k,
        );
        let held = {
            let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
            let target = Target::Screen {
                scale: f64::from(k),
            };
            colored([0.0, 100.0], blue)
                .resolve(logical, &mut Cache::default(), &Held::default(), target)
                .areas()
        };
        let (samples, _) = drag(
            &mut harness,
            &rx,
            (from, Point::new(from.x + 10.0, from.y)),
            t0,
        );
        let kept = tiles::tiles(&samples).len();
        // The same drag goes on over the figure in red.
        harness.rebuild(plot(colored([0.0, 100.0], red), k, &probe));
        let to = Point::new(from.x + 30.0, from.y);
        harness.point_at(to);
        let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: to })]);
        let shot = harness.frame(t0 + FRAME * 3);
        let samples: Vec<Sample> = rx.try_iter().collect();
        let restyled = tiles::tiles(&samples).len();
        let w = (area[2] - area[0]) as f64 / f64::from(k);
        let domain = tiles::panned([0.0, 100.0], w, 30.0);
        let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
        let reference = reference::png_held(
            &colored(domain, red),
            logical,
            f64::from(k),
            Color::WHITE,
            &held,
        );
        let max = against(&shot, &reference);
        println!(
            "a style change mid-drag, {backend}, k = {k}: {all} tiles at first, {kept} on the translated frame, {restyled} after the new color; against a fresh render max {max}/255"
        );
        assert!(kept < all, "{backend}, k = {k}: the translation kept tiles");
        assert_eq!(restyled, all, "{backend}, k = {k}: every tile renders");
        assert!(max <= bound(&backend), "{backend}, k = {k}: max {max}/255");
    }
}

#[test]
fn gutters_merge_into_fewer_images() {
    // The two panels' partition: three columns and rows of cells around
    // two plot areas. The cells beside and under each area that its axes'
    // labels reach into merge along their row, so fewer images render than
    // there are gutter cells, and the frame is the PNG's.
    for k in [1.0_f32, 1.25, 2.0] {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let size = tiles::SIZE;
        let Some(mut harness) = Harness::new(plot(pair([0.0, 100.0]), k, &probe), size, k) else {
            return;
        };
        let backend = harness.backend();
        let shot = harness.frame(headless::epoch());
        let samples: Vec<Sample> = rx.try_iter().collect();
        let fig = pair([0.0, 100.0]);
        let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
        let target = Target::Screen {
            scale: f64::from(k),
        };
        let mut cache = Cache::default();
        let resolved = fig.resolve(logical, &mut cache, &Held::default(), target);
        let list = fig.draw_resolved(&mut cache, &resolved);
        let (xs, ys) = resolved.edges();
        let partition = list::cells(&list, &xs, &ys, f64::from(k));
        let areas = tiles::areas(&fig, size, k);
        let inside = |rect: kurbo::Rect| {
            areas.iter().any(|a| {
                rect.x0 as i64 >= a[0]
                    && rect.y0 as i64 >= a[1]
                    && rect.x1 as i64 <= a[2]
                    && rect.y1 as i64 <= a[3]
            })
        };
        let gutters = partition
            .cells
            .iter()
            .filter(|cell| !inside(cell.rect))
            .count();
        // A gutter run reports its first cell; an overlay, the plot-area
        // cell holding its corner.
        let runs = targets(&samples)
            .iter()
            .filter(|t| match t {
                probe::Target::Cell(c) => !inside(partition.cells[*c as usize].rect),
                probe::Target::Tile { .. } => false,
            })
            .count();
        let png = reference::png_of(&fig, logical, f64::from(k), Color::WHITE);
        let max = against(&shot, &png);
        println!(
            "gutters, {backend}, k = {k}: {gutters} gutter cells shown by {runs} images ({} images in all); against the PNG max {max}/255",
            images(&samples)
        );
        assert!(runs < gutters, "{backend}, k = {k}: {runs} of {gutters}");
        assert!(max <= bound(&backend), "{backend}, k = {k}: max {max}/255");
    }
}
