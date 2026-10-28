//! Tiles through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: plot areas cut into squares of 256 device px on one grid per
//! area, their pixels against the figure's PNG, and their keys.

use std::collections::HashSet;
use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Color, Event, Point, Size, mouse};
use rustd3_core::scale::{self, Linear};
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::{Scale, ScaleRef};
use rustd3_figure::{Cache, Figure, Held, Target, axis, kurbo, layout, raster, thin};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Caching, Probe, Render};
use rustd3_testkit::pixels;

use crate::gestures::{Message, run_frame};
use crate::headless::{self, Harness, Shot};
use crate::reference;

/// The figures' logical size: plot areas of several tiles at every factor.
pub const SIZE: Size = Size::new(640.0, 400.0);

/// The side of a tile, device px.
pub const TILE: i64 = 256;

/// The clock a test's probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
pub fn clock() -> Instant {
    Instant::now()
}

/// The widget against the PNG, opaque: 1/255 on tiny-skia, 2/255 on wgpu.
pub fn bound(backend: &str) -> u8 {
    if backend == "wgpu" { 2 } else { 1 }
}

/// Dots on a smooth curve and the curve itself over x [0, 100] and y
/// [-1.2, 1.2], with left and bottom axes; the panel zooms both. Nothing
/// overlaps itself densely, so renders at any origin agree (D18.3).
pub fn figure(x: [f64; 2]) -> Figure<'static> {
    handled(x).0
}

/// [`figure`] and its x handle.
fn handled(x: [f64; 2]) -> (Figure<'static>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let hx = fig.scale(scale::linear().domain(x));
    let hy = fig.scale(scale::linear().domain([-1.2, 1.2]));
    let curve: Vec<(f64, f64)> = (0..=400)
        .map(|i| {
            let t = f64::from(i) * 0.25;
            (t, (t * 0.11).sin() * (t * 0.023).cos())
        })
        .collect();
    let points: Vec<(f64, f64)> = curve.iter().step_by(8).copied().collect();
    fig.panel(0, 0)
        .mark(line(curve, hx, hy))
        .mark(dots(points, hx, hy))
        .axis(axis::left(hy))
        .axis(axis::bottom(hx))
        .zoom_x(hx)
        .zoom_y(hy);
    (fig, hx)
}

/// The plot areas of `fig` at `size` on the screen at `k`, device px.
pub fn areas(fig: &Figure<'_>, size: Size, k: f32) -> Vec<[i64; 4]> {
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    fig.resolve(logical, &mut Cache::default(), &Held::default(), target)
        .plot_areas()
        .iter()
        .map(|a| [a.x0, a.y0, a.x1, a.y1].map(|e| (e * f64::from(k)).round() as i64))
        .collect()
}

/// The pixels of `rect` (device px) of an image `width` px wide.
pub fn crop(rgba: &[u8], width: u32, [x0, y0, x1, y1]: [i64; 4]) -> Vec<u8> {
    let row = width as usize * 4;
    (y0..y1)
        .flat_map(|y| {
            let start = y as usize * row + x0 as usize * 4;
            rgba[start..start + (x1 - x0) as usize * 4].iter().copied()
        })
        .collect()
}

/// `shot` against the PNG of `fig` inside device rect `rect`.
pub fn against_png_in(shot: &Shot, fig: &Figure<'_>, k: f32, rect: [i64; 4]) -> pixels::Diff {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let png = reference::png_of(fig, size, f64::from(k), Color::WHITE);
    let (w, h) = ((rect[2] - rect[0]) as u32, (rect[3] - rect[1]) as u32);
    let a = crop(&shot.rgba, shot.width, rect);
    let b = crop(&png.rgba, png.width, rect);
    let a = pixels::Image::new(w, h, &a).expect("invariant: a crop of the shot");
    let b = pixels::Image::new(w, h, &b).expect("invariant: a crop of the PNG");
    pixels::Diff::between(&a, &b, 16).expect("invariant: equal crops")
}

/// The tiles among the raster samples of `samples`: (area, i, j, pixels).
pub fn tiles(samples: &[Sample]) -> Vec<(u32, i32, i32, u32)> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Tile { area, i, j },
                pixels,
                ..
            } => Some((*area, *i, *j, *pixels)),
            Sample::Raster { .. } | Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .collect()
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

#[test]
fn tiles_match_the_whole_area() {
    for k in [1.0, 1.25, 2.0] {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let Some(mut harness) = Harness::new(plot(figure([0.0, 100.0]), k, &probe), SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let shot = harness.frame(headless::epoch());
        let samples: Vec<Sample> = rx.try_iter().collect();
        let area = areas(&figure([0.0, 100.0]), SIZE, k)[0];
        let diff = against_png_in(&shot, &figure([0.0, 100.0]), k, area);
        let count = tiles(&samples).len();
        println!(
            "tiles, {backend}, k = {k}: plot area {area:?} in {count} tiles, against the PNG max {}/255",
            diff.max()
        );
        assert!(count > 1, "{backend}, k = {k}: {count} tiles");
        assert!(
            diff.max() <= bound(&backend),
            "{backend}, k = {k}: max {}/255",
            diff.max()
        );
    }
}

#[test]
fn tiles_cover_the_area_once() {
    for k in [1.0, 1.25, 1.5, 2.0] {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let Some(mut harness) = Harness::new(plot(figure([0.0, 100.0]), k, &probe), SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let _ = harness.frame(headless::epoch());
        let samples: Vec<Sample> = rx.try_iter().collect();
        let [x0, y0, x1, y1] = areas(&figure([0.0, 100.0]), SIZE, k)[0];
        let rendered = tiles(&samples);
        // Tile (0, 0) starts at the area's corner: columns and rows of
        // 256 px over the area grown by one tile on each side (the ring),
        // each key once and over its whole square.
        let (columns, rows) = ((x1 - x0 + TILE - 1) / TILE, (y1 - y0 + TILE - 1) / TILE);
        let keys: HashSet<(u32, i32, i32)> =
            rendered.iter().map(|&(a, i, j, _)| (a, i, j)).collect();
        let expected: HashSet<(u32, i32, i32)> = (-1..=rows)
            .flat_map(|j| (-1..=columns).map(move |i| (0, i as i32, j as i32)))
            .collect();
        let pixels: i64 = rendered.iter().map(|&(.., p)| i64::from(p)).sum();
        let (w, h) = ((columns + 2) * TILE, (rows + 2) * TILE);
        println!(
            "keys, {backend}, k = {k}: {columns} × {rows} tiles and a ring over {} × {} px",
            x1 - x0,
            y1 - y0
        );
        assert_eq!(
            keys.len(),
            rendered.len(),
            "{backend}, k = {k}: each key once"
        );
        assert_eq!(keys, expected, "{backend}, k = {k}");
        assert_eq!(pixels, w * h, "{backend}, k = {k}");
    }
}

/// A larger widget: a plot area of 900 × 640 device px or so at 1.
pub const BIG: Size = Size::new(1000.0, 720.0);

/// One frame at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// Sends a left press at `at` (logical px).
fn press<'a, M: 'a>(harness: &mut Harness<'a, M>, at: Point) {
    harness.point_at(at);
    let moved = Event::Mouse(mouse::Event::CursorMoved { position: at });
    let _ = harness.events(&[
        moved,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    ]);
}

/// Moves the pointer to `at` (logical px).
fn move_to<'a, M: 'a>(harness: &mut Harness<'a, M>, at: Point) {
    harness.point_at(at);
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
}

/// Releases the left button.
fn release<'a, M: 'a>(harness: &mut Harness<'a, M>) {
    let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
}

/// The domain `x` over `width` list px panned by `px` list px, its content
/// following: the scale's own `pan_px`, as a drag applies it.
pub fn panned(x: [f64; 2], width: f64, px: f64) -> [f64; 2] {
    let moved = scale::linear()
        .domain(x)
        .pan_px([0.0, width], px)
        .expect("a linear scale pans");
    moved
        .as_any()
        .downcast_ref::<Linear>()
        .expect("its own type")
        .get_domain()
}

/// The plot areas `fig` holds at `size` and `k` from its first resolve,
/// as a gesture holds them.
fn held(fig: &Figure<'_>, size: Size, k: f32) -> layout::Areas {
    let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    fig.resolve(logical, &mut Cache::default(), &Held::default(), target)
        .areas()
}

/// `shot` against `reference` (an image of the same size).
fn against(shot: &Shot, reference: &raster::Image) -> pixels::Diff {
    let png = pixels::Image::new(reference.width, reference.height, &reference.rgba)
        .expect("invariant: a render holds width × height pixels");
    pixels::Diff::between(&shot.image(), &png, 16).expect("invariant: the shot has the PNG's size")
}

/// The device rect `rect` (device px) in logical px at `k`: its centre.
fn centre([x0, y0, x1, y1]: [i64; 4], k: f32) -> Point {
    Point::new((x0 + x1) as f32 / 2.0 / k, (y0 + y1) as f32 / 2.0 / k)
}

/// The logical size at which [`figure`]'s plot area is `w × h` device px
/// at `k`.
fn sized(w: i64, h: i64, k: f32) -> Size {
    let mut size = BIG;
    for _ in 0..8 {
        let area = areas(&figure([0.0, 100.0]), size, k)[0];
        let (dw, dh) = (w - (area[2] - area[0]), h - (area[3] - area[1]));
        if (dw, dh) == (0, 0) {
            return size;
        }
        size = Size::new(size.width + dw as f32 / k, size.height + dh as f32 / k);
    }
    panic!("no size gives a {w} × {h} plot area at {k}");
}

/// A drag of `d` device px over [`figure`] at `size` and `k`, its
/// overscan ring resident: the tiles that render in the drag's frame
/// (area, i, j), how many gutter cells, and the frame against a fresh
/// render of the snapped domain; also the plot area and the backend.
fn drag_by(k: f32, size: Size, d: i32) -> Option<Dragged> {
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let fig = figure([0.0, 100.0]);
    let area = areas(&fig, size, k)[0];
    let held = held(&fig, size, k);
    let mut harness = Harness::new(plot(figure([0.0, 100.0]), k, &probe), size, k)?;
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let first = tiles(&rx.try_iter().collect::<Vec<_>>()).len();
    let from = centre(area, k);
    press(&mut harness, from);
    let _ = harness.frame(t0 + FRAME);
    let _ = rx.try_iter().count();
    move_to(&mut harness, Point::new(from.x + d as f32 / k, from.y));
    let shot = harness.frame(t0 + FRAME * 2);
    let samples: Vec<Sample> = rx.try_iter().collect();
    let keys: HashSet<(i32, i32)> = tiles(&samples).iter().map(|&(_, i, j, _)| (i, j)).collect();
    let rendered = tiles(&samples).len();
    let cells = samples
        .iter()
        .filter(|s| {
            matches!(
                s,
                Sample::Raster {
                    target: probe::Target::Cell(_),
                    ..
                }
            )
        })
        .count();
    let w = (area[2] - area[0]) as f64 / f64::from(k);
    let domain = panned([0.0, 100.0], w, f64::from(d) / f64::from(k));
    let logical = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let reference =
        reference::png_held(&figure(domain), logical, f64::from(k), Color::WHITE, &held);
    let max = against(&shot, &reference).max();
    release(&mut harness);
    let _ = harness.frame(t0 + FRAME * 3);
    println!(
        "drag of {d} device px over a {} × {} plot area, {backend}, k = {k}: {first} tiles at first, then {rendered} tiles {keys:?} and {cells} cells; against a fresh render of the snapped domain max {max}/255",
        area[2] - area[0],
        area[3] - area[1],
    );
    Some(Dragged {
        keys,
        rendered,
        cells,
        max,
        area,
        backend,
    })
}

/// What [`drag_by`] reports.
struct Dragged {
    keys: HashSet<(i32, i32)>,
    rendered: usize,
    cells: usize,
    max: u8,
    area: [i64; 4],
    backend: String,
}

#[test]
fn pan_renders_only_entering_tiles() {
    // The spot checks: a 900 × 600 device-px plot area, its ring of 6 × 5
    // keys resident, each rendered over its whole square (D31.6). +37 px:
    // no visible tile renders, the column i = −2 enters (its 5 rows, each
    // its square [−475, −219)); −37 px: no key enters, and the ring column
    // i = 4, rendered over [1024, 1280), lies at [987, 1243), its square:
    // no tile renders.
    let size = sized(900, 600, 1.0);
    for (d, entering) in [(37, Some(-2)), (-37, None)] {
        let Some(drag) = drag_by(1.0, size, d) else {
            return;
        };
        let expected: HashSet<(i32, i32)> = entering
            .into_iter()
            .flat_map(|i| (-1..=3).map(move |j| (i, j)))
            .collect();
        assert_eq!(drag.keys, expected, "{}: {d} px", drag.backend);
        assert_eq!(drag.rendered, expected.len(), "{}: {d} px", drag.backend);
        assert!(drag.cells > 0, "the gutter cells showing x render");
        assert!(
            drag.max <= bound(&drag.backend),
            "{}: {d} px, max {}/255",
            drag.backend,
            drag.max
        );
    }
    // At 2× on a taller area, the column i = −2 of every row enters.
    let Some(drag) = drag_by(2.0, BIG, 37) else {
        return;
    };
    let rows = (drag.area[3] - drag.area[1] + TILE - 1) / TILE;
    let entering: HashSet<(i32, i32)> = (-1..=rows as i32).map(|j| (-2, j)).collect();
    assert_eq!(drag.keys, entering, "{}, k = 2", drag.backend);
    assert_eq!(drag.rendered, entering.len(), "{}, k = 2", drag.backend);
    assert!(
        drag.max <= bound(&drag.backend),
        "{}, k = 2: max {}/255",
        drag.backend,
        drag.max
    );
}

/// A figure over one channel of 200,000 samples at 1 kHz (a slow sine
/// with a faster ripple, smooth at any zoom, so nothing overlaps itself
/// densely), x fixed at `x`, y at [−1.5, 1.5], dashed by `dash` when
/// given; left and bottom axes; the panel zooms x. Also its x handle.
fn recorded<'a>(
    rec: &'a Recording<f64>,
    x: [f64; 2],
    dash: &[f64],
) -> (Figure<'a>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let hx = fig.scale(scale::linear().domain(x));
    let hy = fig.scale(scale::linear().domain([-1.5, 1.5]));
    let mut mark = line(rec.series(0).expect("one channel"), hx, hy);
    if !dash.is_empty() {
        mark = mark.dash(dash.iter().copied());
    }
    fig.panel(0, 0)
        .mark(mark)
        .axis(axis::left(hy))
        .axis(axis::bottom(hx))
        .zoom_x(hx);
    (fig, hx)
}

/// The recording [`recorded`] draws.
fn ripple() -> Recording<f64> {
    let y: Vec<f64> = (0..200_000)
        .map(|i| {
            let t = f64::from(i) * 1e-3;
            (t * 0.05).sin() + 0.3 * (t * 0.9).sin()
        })
        .collect();
    let clock = Clock::Regular {
        start: 0.0,
        period: 1e-3,
    };
    Recording::new(clock, [y]).expect("a valid recording")
}

/// Pans the widget over [`recorded`] with x at [80, 120] at factor `k`
/// by each of `moves` (device px from the drag's start), comparing each
/// frame with the figure of the snapped domain drawn as the widget draws
/// it, with the column grid of the drag when `anchored`. The worst
/// difference and the backend.
fn pan_recorded(k: f32, dash: &[f64], moves: &[i32]) -> Option<(u8, String)> {
    let rec = ripple();
    let x = [80.0, 120.0];
    let (fig, hx) = recorded(&rec, x, dash);
    let area = areas(&fig, BIG, k)[0];
    let held = held(&fig, BIG, k);
    let plot = Plot::<()>::new(fig)
        .scale_factor(k)
        .render(Render::Inline)
        .crosshair(false)
        .highlight(false);
    let mut harness = Harness::new(plot, BIG, k)?;
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let from = centre(area, k);
    press(&mut harness, from);
    let _ = harness.frame(t0 + FRAME);
    let w = (area[2] - area[0]) as f64 / f64::from(k);
    let size = kurbo::Size::new(f64::from(BIG.width), f64::from(BIG.height));
    let mut worst = 0;
    for (n, &d) in moves.iter().enumerate() {
        move_to(&mut harness, Point::new(from.x + d as f32 / k, from.y));
        let shot = harness.frame(t0 + FRAME * (n as u32 + 2));
        let px = f64::from(d) / f64::from(k);
        let (moved, _) = recorded(&rec, panned(x, w, px), dash);
        let grid = thin::Grid {
            origin: px,
            density: f64::from(k),
        };
        let grids = vec![(hx.index(), grid)];
        let reference =
            reference::png_held_on(&moved, size, f64::from(k), Color::WHITE, (&held, grids));
        let diff = against(&shot, &reference);
        println!(
            "pan {d} device px, {backend}, k = {k}, dash {dash:?}: max {}/255, {} channels over 16",
            diff.max(),
            diff.over()
        );
        worst = worst.max(diff.max());
    }
    release(&mut harness);
    Some((worst, backend))
}

#[test]
fn thinned_lines_match_a_fresh_render_after_a_pan() {
    for k in [1.0, 2.0] {
        let Some((worst, backend)) = pan_recorded(k, &[], &[20, 57, -30, -300]) else {
            return;
        };
        assert!(
            worst <= bound(&backend),
            "{backend}, k = {k}: max {worst}/255"
        );
    }
}

#[test]
fn dashed_lines_keep_their_phase_across_a_pan() {
    for k in [1.0, 2.0] {
        let Some((worst, backend)) = pan_recorded(k, &[6.0, 3.0], &[0, 20, 57, -30]) else {
            return;
        };
        assert!(
            worst <= bound(&backend),
            "{backend}, k = {k}: max {worst}/255"
        );
    }
}

#[test]
fn zoom_out_preview_uses_the_overscan_ring() {
    let _workers = crate::pipeline::workers();
    for k in [1.0_f32, 2.0] {
        let (tx, rx) = mpsc::channel();
        let area = areas(&figure([0.0, 100.0]), SIZE, k)[0];
        let plot = Plot::<()>::new(figure([0.0, 100.0]))
            .scale_factor(k)
            .render(Render::Worker)
            .crosshair(false)
            .highlight(false)
            .probe(Probe::new(clock, tx));
        let Some(mut harness) = Harness::new(plot, SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let mut t = headless::epoch();
        let _ = harness.frame(t);
        // Wait until the ring is resident: every key over the grown rect,
        // one tile beyond each side, has landed.
        let side = |a: i64, b: i64| ((b - a + TILE - 1) / TILE + 2) as usize;
        let keys = side(area[0], area[2]) * side(area[1], area[3]);
        let mut landed: HashSet<(i32, i32)> = HashSet::new();
        loop {
            let samples: Vec<Sample> = rx.try_iter().collect();
            landed.extend(tiles(&samples).iter().map(|&(_, i, j, _)| (i, j)));
            if landed.len() == keys {
                break;
            }
            assert!(
                harness.wait_for_wake(Duration::from_secs(10)),
                "{backend}, k = {k}: {} of {keys} tiles landed",
                landed.len()
            );
            let _ = harness.events(&[Event::Waken]);
            t += FRAME;
            let _ = harness.frame(t);
        }
        // ×1.2 out about the area's centre: d3's wheel delta is 0.002 per
        // logical px, which iced reports in physical px; the views before
        // it map to the middle 1 / 1.2 of the area, and the ring fills the
        // margins.
        let pixels = 1.2_f64.log2() / 0.002 * f64::from(k);
        harness.point_at(centre(area, k));
        let _ = harness.events(&[Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels {
                x: 0.0,
                y: -pixels as f32,
            },
        })]);
        let mut frames = Vec::new();
        t += FRAME;
        let _ = harness.frame(t);
        loop {
            let samples: Vec<Sample> = rx.try_iter().collect();
            frames.extend(samples.iter().filter_map(|sample| match sample {
                Sample::Frame {
                    missing, previews, ..
                } => Some((*missing, *previews)),
                Sample::Stage { .. } | Sample::Raster { .. } => None,
            }));
            if frames.last() == Some(&(0, 0)) {
                break;
            }
            assert!(
                harness.wait_for_wake(Duration::from_secs(10)),
                "{backend}, k = {k}: no wake, frames {frames:?}"
            );
            let _ = harness.events(&[Event::Waken]);
            t += FRAME;
            let _ = harness.frame(t);
        }
        let visible = (side(area[0], area[2]) - 2) * (side(area[1], area[3]) - 2);
        println!(
            "zoom out ×1.2, {backend}, k = {k}: {keys} tiles resident, then (missing, previews) per frame {frames:?}"
        );
        assert!(
            frames[0].1 as usize >= visible,
            "{backend}, k = {k}: the first frame previews every visible tile, {frames:?}"
        );
        assert!(
            frames.iter().all(|&(missing, _)| missing == 0),
            "{backend}, k = {k}: {frames:?}"
        );
    }
}

/// An app showing [`figure`] over the x domain it last heard.
struct Panned {
    x: [f64; 2],
    k: f32,
}

impl crate::gestures::App for Panned {
    fn update(&mut self, message: Message) {
        if let Message::X(x) = message {
            self.x = x;
        }
    }

    fn view(&self) -> Plot<'static, Message> {
        let (fig, hx) = handled(self.x);
        Plot::new(fig)
            .scale_factor(self.k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false)
            .on_change(hx, Message::X)
    }
}

#[test]
fn drag_settles_on_the_exact_domain() {
    let k = 1.25;
    let mut app = Panned { x: [0.0, 100.0], k };
    let logical = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let w = figure([0.0, 100.0])
        .resolve(logical, &mut Cache::default(), &Held::default(), target)
        .plot_areas()[0]
        .width();
    let area = areas(&figure([0.0, 100.0]), SIZE, k)[0];
    let held = held(&figure([0.0, 100.0]), SIZE, k);
    let Some(mut harness) = Harness::new(crate::gestures::App::view(&app), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = run_frame(&mut harness, &mut app, t0);
    let from = centre(area, k);
    press(&mut harness, from);
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
    // 10.3 logical px at 1.25 are 12.875 device px: the drag shows 13,
    // 10.4 logical px, until it ends.
    let to = Point::new(from.x + 10.3, from.y);
    move_to(&mut harness, to);
    let (mid, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    let raw = f64::from(to.x) - f64::from(from.x);
    let snapped = (raw * f64::from(k)).round() / f64::from(k);
    let [Message::X(during)] = messages[..] else {
        panic!("{backend}: x published once, {messages:?}");
    };
    let close = |a: [f64; 2], b: [f64; 2]| a.iter().zip(&b).all(|(p, q)| (p - q).abs() < 1e-9);
    assert!((snapped - 10.4).abs() < 1e-6, "{snapped}");
    assert!(
        close(during, panned([0.0, 100.0], w, snapped)),
        "{backend}: {during:?}"
    );
    let reference =
        reference::png_held(&figure(during), logical, f64::from(k), Color::WHITE, &held);
    let drag = against(&mid, &reference).max();
    release(&mut harness);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    let [Message::X(exact)] = messages[..] else {
        panic!("{backend}: the exact x published at the end, {messages:?}");
    };
    assert!(
        close(exact, panned([0.0, 100.0], w, raw)),
        "{backend}: {exact:?}"
    );
    assert!(!close(exact, during), "{backend}: the end moves x");
    // Once the layout settles, the frame is the PNG of the exact domain.
    let end = t0 + FRAME * 3 + Duration::from_millis(300);
    let (shot, messages) = run_frame(&mut harness, &mut app, end);
    assert!(messages.is_empty(), "{backend}: {messages:?}");
    let settled = against(
        &shot,
        &reference::png_of(&figure(exact), logical, f64::from(k), Color::WHITE),
    )
    .max();
    println!(
        "drag of 10.3 logical px at 1.25, {backend}: {during:?} during it (10.4 px), {exact:?} at its end; against the PNG max {drag}/255 mid-drag, {settled}/255 settled"
    );
    assert!(
        drag <= bound(&backend),
        "{backend}: max {drag}/255 mid-drag"
    );
    assert!(
        settled <= bound(&backend),
        "{backend}: max {settled}/255 settled"
    );
}

/// Samples per second of [`trace`].
const RATE: f64 = 10_000.0;

/// A trace like the benchmark's (`dev/bench/src/workload/trace.rs`):
/// three channels of `n` samples at 10 kHz, each a slow sine, a 7 Hz tone
/// and uniform noise (a 64-bit LCG seeded per channel), with a 120 Hz
/// burst for 0.2 s every 10 s. Thinned for the screen, its strokes
/// overlap themselves densely (D18.3).
fn trace(n: usize) -> Recording<f64> {
    let tau = std::f64::consts::TAU;
    let channels = [1_u64, 2, 3].map(|seed| {
        let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        (0..n)
            .map(|i| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let unit = (state >> 11) as f64 / (1_u64 << 53) as f64;
                let t = i as f64 / RATE;
                let burst = if t % 10.0 < 0.2 {
                    0.8 * (tau * 120.0 * t).sin()
                } else {
                    0.0
                };
                0.3 * (tau * 0.05 * t).sin()
                    + 0.4 * (tau * 7.0 * t).sin()
                    + 0.25 * (unit - 0.5)
                    + burst
            })
            .collect::<Vec<f64>>()
    });
    let clock = Clock::Regular {
        start: 0.0,
        period: 1.0 / RATE,
    };
    Recording::new(clock, channels).expect("finite samples")
}

/// [`trace`]'s figure as the benchmark lays it out: channels 0 and 1 on
/// the top panel, 2 on the bottom one, x fixed at `x` (s) and shared, y
/// at [−2, 2] per panel; left axes, and bottom axes when `bottom`; both
/// panels zoom x.
fn traced(rec: &Recording<f64>, x: [f64; 2], bottom: bool) -> Figure<'_> {
    let mut fig = Figure::new();
    let time = fig.scale(scale::linear().domain(x));
    let top = fig.scale(scale::linear().domain([-2.0, 2.0]));
    let low = fig.scale(scale::linear().domain([-2.0, 2.0]));
    let channel = |c: usize| rec.series(c).expect("three channels");
    let upper = fig.panel(0, 0);
    upper
        .mark(line(channel(0), time, top))
        .mark(line(channel(1), time, top))
        .axis(axis::left(top))
        .zoom_x(time);
    if bottom {
        upper.axis(axis::bottom(time));
    }
    let lower = fig.panel(1, 0);
    lower
        .mark(line(channel(2), time, low))
        .axis(axis::left(low))
        .zoom_x(time);
    if bottom {
        lower.axis(axis::bottom(time));
    }
    fig
}

/// The largest channel difference of `a` and `b` inside the device rects
/// `rects`, and how many pixels differ by more than 2.
fn inside(a: &Shot, b: &Shot, rects: &[[i64; 4]]) -> (u8, usize) {
    let (mut max, mut over) = (0, 0);
    for &[x0, y0, x1, y1] in rects {
        for y in y0..y1 {
            for x in x0..x1 {
                let (p, q) = (a.pixel(x as u32, y as u32), b.pixel(x as u32, y as u32));
                let d = (0..3).map(|c| p[c].abs_diff(q[c])).max().unwrap_or(0);
                max = max.max(d);
                over += usize::from(d > 2);
            }
        }
    }
    (max, over)
}

/// The bound of translated tiles against tiles re-rendered at the same
/// keys on [`trace`], measured (D18.3, `decisions/phase-31.md`).
const TRACE_BOUND: u8 = 1;

#[test]
fn translated_tiles_match_re_rendered_ones_on_a_trace() {
    // D18.3: vello_cpu's output on the trace depends on the render's
    // origin. A tile renders from its key's square, which moves with the
    // data, so a translated tile and the same key rendered again after
    // the pan (`Caching::Off` renders every tile each frame) share it.
    let rec = trace(400_000);
    let x = [10.0, 30.0];
    for k in [1.0_f32, 2.0] {
        let widget = |caching: Caching| {
            Plot::<()>::new(traced(&rec, x, true))
                .scale_factor(k)
                .render(Render::Inline)
                .caching(caching)
                .crosshair(false)
                .highlight(false)
        };
        let Some(mut kept) = Harness::new(widget(Caching::On), SIZE, k) else {
            return;
        };
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let mut fresh = Harness::new(widget(Caching::Off).probe(probe), SIZE, k)
            .expect("the backend found above");
        let backend = kept.backend();
        let rects = areas(&traced(&rec, x, true), SIZE, k);
        let from = centre(rects[0], k);
        let t0 = headless::epoch();
        for harness in [&mut kept, &mut fresh] {
            let _ = harness.frame(t0);
            press(harness, from);
            let _ = harness.frame(t0 + FRAME);
        }
        let mut worst = 0;
        for (n, d) in [1, 2, 4, 37, -300].into_iter().enumerate() {
            let at = Point::new(from.x + d as f32 / k, from.y);
            let t = t0 + FRAME * (n as u32 + 2);
            move_to(&mut kept, at);
            move_to(&mut fresh, at);
            let _ = rx.try_iter().count();
            let (a, b) = (kept.frame(t), fresh.frame(t));
            let again = tiles(&rx.try_iter().collect::<Vec<_>>()).len();
            let (max, over) = inside(&a, &b, &rects);
            println!(
                "trace, {backend}, k = {k}: drag to {d} device px, translated against {again} re-rendered tiles max {max}/255, {over} px over 2"
            );
            let side = |a: i64, b: i64| ((b - a + TILE - 1) / TILE) as usize;
            let visible: usize = rects
                .iter()
                .map(|r| side(r[0], r[2]) * side(r[1], r[3]))
                .sum();
            assert!(
                again >= visible,
                "{backend}, k = {k}: {again} tiles re-rendered"
            );
            worst = worst.max(max);
        }
        release(&mut kept);
        release(&mut fresh);
        assert!(worst <= TRACE_BOUND, "{backend}, k = {k}: max {worst}/255");
    }
}

/// Lands what the worker renders until it is idle: a wake handled
/// after each result, until none comes for 300 ms.
fn drain(harness: &mut Harness<'_, ()>) {
    while harness.wait_for_wake(Duration::from_millis(300)) {
        let _ = harness.events(&[Event::Waken]);
    }
}

/// What a frame of [`pan_replay`] showed: (missing, previews), and the
/// tiles rendered after it.
type Replayed = (u32, u32, usize);

/// A pan-x replay over [`traced`] (without its bottom axes, so that a pan
/// changes nothing outside the plot areas) at `k` as the benchmark's
/// `pan-x` (`dev/bench/src/scenario.rs`) drives it: the pointer right of
/// centre on the top panel, a press, then every 8 ms a move of 2–8
/// logical px, left for 600 px (at most 60% of the width), then back;
/// `frames` frames 16 ms apart under `Render::Worker`, each followed by
/// the worker's renders (a worker that keeps up). Per frame after the
/// press: (missing, previews), and the tiles rendered after it.
fn pan_replay(rec: &Recording<f64>, k: f32, frames: u32) -> Option<(Vec<Replayed>, String)> {
    let (tx, rx) = mpsc::channel();
    let x = [10.0, 30.0];
    let plot = Plot::<()>::new(traced(rec, x, false))
        .scale_factor(k)
        .render(Render::Worker)
        .crosshair(false)
        .highlight(false)
        .probe(Probe::new(clock, tx));
    let mut harness = Harness::new(plot, SIZE, k)?;
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    drain(&mut harness);
    let width = f64::from(SIZE.width);
    let length = 600.0_f64.min(0.6 * width);
    let mut at = Point::new((0.5 * width + 0.5 * length) as f32, 0.25 * SIZE.height);
    press(&mut harness, at);
    let _ = harness.frame(t0 + FRAME);
    drain(&mut harness);
    let _ = rx.try_iter().count();
    let (mut direction, mut swept, mut moves) = (-1.0_f64, 0.0_f64, 2_u32);
    let mut seen = Vec::new();
    for n in 0..frames {
        for _ in 0..2 {
            if swept >= length {
                direction = -direction;
                swept = 0.0;
            }
            let step = (5.0 + 3.0 * (f64::from(moves) * 0.7).sin()).min(length - swept);
            at.x += (direction * step) as f32;
            swept += step;
            moves += 1;
        }
        move_to(&mut harness, at);
        let _ = harness.frame(t0 + FRAME * (n + 2));
        drain(&mut harness);
        let samples: Vec<Sample> = rx.try_iter().collect();
        let shown = samples.iter().find_map(|sample| match sample {
            Sample::Frame {
                missing, previews, ..
            } => Some((*missing, *previews)),
            Sample::Raster { .. } | Sample::Stage { .. } => None,
        });
        let (missing, previews) = shown.expect("a frame sample");
        seen.push((missing, previews, tiles(&samples).len()));
    }
    release(&mut harness);
    Some((seen, backend))
}

#[test]
fn a_pan_over_a_trace_keeps_the_plot_areas_current() {
    // D22.1: the benchmark's pan-x over the trace showed outdated plot
    // areas on 94% of frames: each frame re-rendered whole plot-area
    // cells, which the worker delivered a frame late. With tiles, the
    // translated images cover the plot areas and only the ring's tiles
    // render, after the frame: a worker that keeps up leaves no frame
    // outdated.
    let _workers = crate::pipeline::workers();
    let rec = trace(400_000);
    for k in [1.0_f32, 2.0] {
        let Some((frames, backend)) = pan_replay(&rec, k, 30) else {
            return;
        };
        let after = &frames[1..];
        let outdated = after
            .iter()
            .filter(|&&(missing, previews, _)| missing + previews > 0)
            .count();
        let ring: usize = after.iter().map(|&(_, _, tiles)| tiles).sum();
        let share = outdated as f64 / after.len() as f64;
        println!(
            "pan-x over the trace, {backend}, k = {k}: {} frames; after the first, {outdated} showed outdated plot-area content ({:.1}%), {ring} tiles rendered after them ({:.1} per frame)",
            frames.len(),
            100.0 * share,
            ring as f64 / after.len() as f64
        );
        assert!(
            share < 0.05,
            "{backend}, k = {k}: {:.1}%, {frames:?}",
            100.0 * share
        );
        assert!(ring > 0, "{backend}, k = {k}: the ring follows the pan");
    }
}
