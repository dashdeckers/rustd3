//! Tiles through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: plot areas cut into squares of 256 device px on one grid per
//! area, their pixels against the figure's PNG, and their keys.

use std::collections::HashSet;
use std::sync::mpsc;

use iced::time::Instant;
use iced::{Color, Size};
use rustd3_core::scale;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Cache, Figure, Held, Target, axis, kurbo};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Probe, Render};
use rustd3_testkit::pixels;

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
    fig
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
        // 256 px, the last ones shorter, each key once.
        let (columns, rows) = ((x1 - x0 + TILE - 1) / TILE, (y1 - y0 + TILE - 1) / TILE);
        let keys: HashSet<(u32, i32, i32)> =
            rendered.iter().map(|&(a, i, j, _)| (a, i, j)).collect();
        let expected: HashSet<(u32, i32, i32)> = (0..rows)
            .flat_map(|j| (0..columns).map(move |i| (0, i as i32, j as i32)))
            .collect();
        let pixels: i64 = rendered.iter().map(|&(.., p)| i64::from(p)).sum();
        println!(
            "keys, {backend}, k = {k}: {columns} × {rows} tiles over {} × {} px",
            x1 - x0,
            y1 - y0
        );
        assert_eq!(
            keys.len(),
            rendered.len(),
            "{backend}, k = {k}: each key once"
        );
        assert_eq!(keys, expected, "{backend}, k = {k}");
        assert_eq!(pixels, (x1 - x0) * (y1 - y0), "{backend}, k = {k}");
    }
}
