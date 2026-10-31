//! `Renderer::splat`: markers composited from eighth-pixel
//! coverage stamps against the markers filled point by point.

use rustd3_core::{scale, scheme};
use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::list::{Bounds, Command, List, Outlines};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::raster::{Image, Renderer, Settings, Simd};
use rustd3_figure::{Figure, Held, Target, axis};

use super::vello::diff;

/// Splatted markers stay within 32/255 of filled ones.
const BOUND: u8 = 32;

/// `n` dots of a wobbling sine with a curve drawn over them and axes,
/// whose centres fall on any sub-pixel phase; `colors` paints them from
/// viridis by a channel.
fn scatter(n: usize, colors: bool) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([-1.5, 2.5]));
    let points: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = ((i * 7919) % 1000) as f64 / 1000.0;
            (t * 10.0, (t * 25.0).sin() + wobble)
        })
        .collect();
    let curve: Vec<(f64, f64)> = (0..=200)
        .map(|i| {
            let t = f64::from(i) * 0.05;
            (t, (t * 2.5).sin())
        })
        .collect();
    let mut mark = dots(points, x, y);
    if colors {
        let heat = fig.scale(scale::sequential([0.0, 1.0], scheme::viridis));
        mark = mark.fill_by(heat, (0..n).map(|i| (i % 7) as f64 / 6.0));
    }
    fig.panel(0, 0)
        .mark(mark)
        .mark(line(curve, x, y).width(2.0))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// `fig` drawn for the screen at `k` over 480 × 320: the list and its
/// plot area.
fn drawn(fig: &Figure<'_>, k: f64) -> (List, Rect) {
    let size = Size::new(480.0, 320.0);
    let target = Target::Screen { scale: k };
    let mut cache = Cache::default();
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let area = resolved.plot_areas()[0];
    (fig.draw_resolved(&mut cache, &resolved), area)
}

/// `area` of `list` at `k`, splatted or filled.
fn render(list: &List, area: Rect, k: f64, splat: bool) -> Image {
    let bounds = Bounds::of(list, &mut Outlines::default());
    let mut renderer = Renderer::new(Settings {
        threads: 0,
        simd: Simd::Baseline,
    });
    renderer
        .splat(splat)
        .render(list, &bounds, area, k)
        .expect("a render")
}

#[test]
fn splatted_markers_match_filled_ones() {
    for k in [1.0, 1.25, 2.0] {
        for colors in [false, true] {
            let (list, area) = drawn(&scatter(5_000, colors), k);
            // The plot area and the whole canvas: markers inside a clipped
            // group, with the line drawn after them and the axes around.
            let whole = Rect::from_origin_size((0.0, 0.0), list.size);
            for rect in [area, whole] {
                let filled = render(&list, rect, k, false);
                let splatted = render(&list, rect, k, true);
                let d = diff(&filled, &splatted);
                println!(
                    "splat at {k}, colors {colors}, {rect:?}: max {}/255, {} channels over 16",
                    d.max(),
                    d.over()
                );
                assert!(
                    d.max() <= BOUND,
                    "k = {k}, colors {colors}: max {}/255",
                    d.max()
                );
            }
        }
    }
}

#[test]
fn splatting_leaves_lists_without_markers_unchanged() {
    // A curve and axes alone: splatting has nothing to do.
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([-1.5, 1.5]));
    let curve: Vec<(f64, f64)> = (0..=100)
        .map(|i| (f64::from(i) * 0.1, (f64::from(i) * 0.1).sin()))
        .collect();
    fig.panel(0, 0)
        .mark(line(curve, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    let (list, _) = drawn(&fig, 2.0);
    assert!(
        !list
            .commands
            .iter()
            .any(|c| matches!(c, Command::Markers { .. })),
        "no markers"
    );
    let whole = Rect::from_origin_size((0.0, 0.0), list.size);
    let a = render(&list, whole, 2.0, false);
    let b = render(&list, whole, 2.0, true);
    assert_eq!(a, b);
}

#[test]
fn splatted_dots_on_eighth_pixels_match_within_the_compositing() {
    // Centres on eighth pixels at 1×: each stamp is the dot's own coverage
    // at its phase, so only the compositing's rounding differs.
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 76.0]));
    let y = fig.scale(scale::linear().domain([0.0, 76.0]));
    let points: Vec<(f64, f64)> = (0..400)
        .map(|i| {
            let t = f64::from(i);
            let (a, b) = (
                38.0 + (t * 0.37).sin() * 30.0,
                38.0 + (t * 0.23).cos() * 30.0,
            );
            ((a * 8.0).round() / 8.0, (b * 8.0).round() / 8.0)
        })
        .collect();
    fig.panel(0, 0).mark(dots(points, x, y));
    let size = Size::new(100.0, 100.0);
    let target = Target::Screen { scale: 1.0 };
    let mut cache = Cache::default();
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let area = resolved.plot_areas()[0];
    assert_eq!(area.width(), 76.0, "one data unit per device px");
    let list = fig.draw_resolved(&mut cache, &resolved);
    let d = diff(
        &render(&list, area, 1.0, false),
        &render(&list, area, 1.0, true),
    );
    println!("dots on eighth pixels: max {}/255", d.max());
    assert!(d.max() <= 4, "max {}/255", d.max());
}
