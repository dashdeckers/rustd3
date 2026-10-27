//! Thinning at draw time: M4 per column of a recording channel, its memo
//! across draws, and lines drawn as given.

use std::sync::OnceLock;

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Continuous, Linear, Log};
use rustd3_core::series::{Clock, Recording, m4};
use rustd3_figure::kurbo::{BezPath, PathEl, Point, Rect, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::mark::line;
#[cfg(feature = "raster")]
use rustd3_figure::raster::Image;
use rustd3_figure::scale::{Indexed, Position, Scale, ScaleRef};
use rustd3_figure::thin::Thin;
use rustd3_figure::{Figure, Held, Resolved, Target, axis};

use crate::figure::cache;
use crate::marks::marked;
#[cfg(feature = "raster")]
use crate::raster::vello::render;
use crate::trace;

/// `n` samples at 1 kHz on three channels: a ramp with a wobble, its
/// negation, and a tone around 0 that dips below it.
fn recording(n: u32) -> Recording<f64> {
    let a: Vec<f64> = (0..n)
        .map(|i| f64::from(i) * 1e-3 + (f64::from(i) * 0.37).sin())
        .collect();
    let b: Vec<f64> = a.iter().map(|v| -v).collect();
    let c: Vec<f64> = (0..n).map(|i| (f64::from(i) * 0.001).sin() + 0.5).collect();
    Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 1e-3,
        },
        [a, b, c],
    )
    .expect("a valid recording")
}

/// The stroked paths of a list's marks, in order.
fn paths(list: &List) -> Vec<&BezPath> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

/// The vertices of a polyline path.
fn vertices(path: &BezPath) -> usize {
    path.elements()
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_) | PathEl::LineTo(_)))
        .count()
}

#[test]
fn a_second_draw_of_an_unchanged_figure_runs_no_m4_query_and_no_typst_compile() {
    let rec = recording(200_000);
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    for c in 0..3 {
        fig.panel(c, 0)
            .mark(line(rec.series(c).expect("a channel"), t, v))
            .axis(axis::left(v))
            .axis(axis::bottom(t));
    }
    let mut cache = cache();
    let size = Size::new(800.0, 600.0);
    let held = Held::default();
    let first = fig.draw_with(&mut cache, size, Target::Screen { scale: 1.0 }, &held);
    let before = cache.stats();
    assert_eq!(before.m4_queries, 3);
    let second = fig.draw_with(&mut cache, size, Target::Screen { scale: 1.0 }, &held);
    let after = cache.stats();
    assert_eq!(after.m4_queries, before.m4_queries);
    assert_eq!(after.text.compiles, before.text.compiles);
    assert_eq!(first, second);
}

#[test]
fn an_append_reruns_m4_for_the_lines_of_that_recording() {
    let mut rec = recording(100_000);
    let mut cache = cache();
    let size = Size::new(800.0, 600.0);
    for (round, queries) in [(0, 1), (1, 1), (2, 2)] {
        if round == 2 {
            rec.extend(&[], [[0.0], [0.0], [0.0]]).expect("an append");
        }
        let mut fig = Figure::new();
        let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
        fig.panel(0, 0).mark(line(rec.series(0).expect("a"), t, v));
        fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
        assert_eq!(cache.stats().m4_queries, queries, "round {round}");
    }
}

#[test]
fn line_thin_off_on_a_million_samples_draws_every_sample() {
    let rec = recording(1_000_000);
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("a"), t, v).thin(Thin::Off));
    let mut cache = cache();
    let list = fig.draw_with(
        &mut cache,
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(vertices(paths(&list)[0]), 1_000_000);
    assert_eq!(cache.stats().m4_queries, 0);
}

#[test]
fn owned_points_are_drawn_as_given() {
    let pairs: Vec<(f64, f64)> = (0..20_000)
        .map(|i| (f64::from(i), f64::from(i % 7)))
        .collect();
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(line(pairs, t, v));
    let mut cache = cache();
    let list = fig.draw_with(
        &mut cache,
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(vertices(paths(&list)[0]), 20_000);
    assert_eq!(cache.stats().m4_queries, 0);
}

/// The number of points M4 keeps for channel `c` of `rec` in a panel `w`
/// px wide on `x`, at `per_px` columns per px, by core directly.
fn expected<S: Position<Domain = f64> + Scale>(
    rec: &Recording<f64>,
    c: usize,
    x: &S,
    w: f64,
    per_px: f64,
    valid: impl Fn(f64) -> bool,
) -> usize {
    let ranged = x.with_range([0.0, w]);
    let ranged = ranged.as_any().downcast_ref::<S>().expect("its type");
    let indexed = ranged.indexed().expect("indexed");
    let n = (w * per_px).round() as u32;
    let edges: Vec<f64> = (0..=n)
        .map(|i| Indexed::invert(indexed, f64::from(i) / per_px))
        .collect();
    let columns = rec.columns(&edges).expect("valid edges");
    rec.series(c)
        .expect("a channel")
        .m4(&columns, valid)
        .expect("fresh columns")
        .points()
}

#[test]
fn columns_on_a_log_x_scale_invert_device_pixel_edges() {
    let x: Vec<f64> = (0..100_000)
        .map(|i| 10f64.powf(f64::from(i) * 4e-5))
        .collect();
    let y: Vec<f64> = (0..100_000).map(|i| (f64::from(i) * 0.21).sin()).collect();
    let rec = Recording::new(Clock::Explicit(x), [y]).expect("a valid recording");
    let mut fig = Figure::new();
    let t = fig.fit(scale::log());
    let v = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(rec.series(0).expect("a"), t, v));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let target = Target::Screen { scale: 2.0 };
    let list = fig.draw_with(&mut cache, size, target, &Held::default());
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let w = resolved.plot_areas()[0].width();
    let want = expected::<Log>(&rec, 0, resolved.scale(t), w, 2.0, |_| true);
    assert_eq!(vertices(paths(&list)[0]), want);
    assert!(want < 4 * (2.0 * w).round() as usize + 2, "{want}");
}

#[test]
fn a_channel_on_a_log_and_a_linear_y_is_thinned_for_each() {
    let rec = recording(100_000);
    let size = Size::new(640.0, 480.0);
    let draw = |both: bool| -> Vec<BezPath> {
        let mut fig = Figure::new();
        let t = fig.fit(scale::linear());
        if both {
            let linear = fig.fit(scale::linear());
            fig.panel(0, 0)
                .mark(line(rec.series(2).expect("c"), t, linear));
        }
        let log = fig.fit(scale::log());
        fig.panel(1, 0)
            .mark(line(rec.series(2).expect("c"), t, log));
        let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
        paths(&list).into_iter().cloned().collect()
    };
    let both = draw(true);
    let alone = draw(false);
    assert_eq!(both.len(), 2);
    assert_ne!(vertices(&both[0]), vertices(&both[1]));
    assert_eq!(both[1].elements(), alone[0].elements());
}

/// `signal(1_000_000, 42)` on an explicit clock, and its impacts.
fn vibration() -> &'static (Recording<f64>, Vec<usize>) {
    static TRACE: OnceLock<(Recording<f64>, Vec<usize>)> = OnceLock::new();
    TRACE.get_or_init(|| {
        let (x, y, impacts) = trace::signal(1_000_000, 42);
        let rec = Recording::new(Clock::Explicit(x), [y]).expect("a valid recording");
        (rec, impacts)
    })
}

/// The prototype's view of the vibration trace (`proto/thin2`): x over
/// [0, 100] s, y over [−2.4, 2.4] in a plot area 1000 × 240 px, stroked
/// `width` px wide (the prototype's 1, or the theme's 2) in (30, 60, 140);
/// the figure's size; its x and y handles.
fn prototype(thin: Thin, width: f64) -> (Figure<'static>, Size, [ScaleRef<Linear>; 2]) {
    let (rec, _) = vibration();
    let mut fig = Figure::new();
    let t = fig.scale(scale::linear().domain([0.0, 100.0]));
    let v = fig.scale(scale::linear().domain([-2.4, 2.4]));
    fig.panel(0, 0).mark(
        line(rec.series(0).expect("the trace"), t, v)
            .width(width)
            .color(Rgb::new(30.0, 60.0, 140.0))
            .thin(thin),
    );
    let pad = 2.0 * 0.8 * 44.0 / 3.0;
    (fig, Size::new(1000.0 + pad, 240.0 + pad), [t, v])
}

/// `h`'s resolved scale over `range`, as a mark sees it.
fn ranged(resolved: &Resolved, h: ScaleRef<Linear>, range: [f64; 2]) -> Linear {
    let scale = Scale::with_range(resolved.scale(h), range);
    scale
        .as_any()
        .downcast_ref::<Linear>()
        .expect("its type")
        .clone()
}

/// What the prototype's figure draws for `target`: the thinned path, the
/// plot area, the list, and core's M4 at `per_px` columns per list px
/// over the same edges.
struct Thinned {
    path: BezPath,
    area: Rect,
    list: List,
    lines: m4::Lines,
    places: Vec<Point>,
}

fn thinned(target: Target, per_px: f64, width: f64) -> Thinned {
    let (rec, _) = vibration();
    let (fig, size, [t, v]) = prototype(Thin::Auto, width);
    let mut cache = cache();
    let list = fig.draw_with(&mut cache, size, target, &Held::default());
    assert_eq!(cache.stats().m4_queries, 1);
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let area = resolved.plot_areas()[0];
    let x = ranged(&resolved, t, [0.0, area.width()]);
    let y = ranged(&resolved, v, [area.height(), 0.0]);
    let n = (area.width() * per_px).round() as u32;
    let edges: Vec<f64> = (0..=n)
        .map(|i| Continuous::invert(&x, f64::from(i) / per_px))
        .collect();
    let columns = rec.columns(&edges).expect("valid edges");
    let series = rec.series(0).expect("the trace");
    let lines = series.m4(&columns, f64::is_finite).expect("fresh columns");
    let places = lines
        .iter()
        .flatten()
        .map(|&i| {
            let (a, b) = series.get(i).expect("a sample");
            Point::new(x.apply(a), y.apply(b))
        })
        .collect();
    let path = paths(&list)[0].clone();
    Thinned {
        path,
        area,
        list,
        lines,
        places,
    }
}

/// The vertices of a path, in order.
fn points(path: &BezPath) -> Vec<Point> {
    path.elements()
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => None,
        })
        .collect()
}

#[test]
fn the_trace_on_screen_keeps_four_points_per_column_and_every_impact() {
    let (_, impacts) = vibration();
    let thinned = thinned(Target::Screen { scale: 1.0 }, 1.0, 1.0);
    list::check(&thinned.list).expect("a valid list");
    assert_eq!(thinned.area.width(), 1000.0);
    let lines = &thinned.lines;
    assert_eq!(points(&thinned.path), thinned.places);
    assert!(lines.points() <= 4 * 1000 + 2 * lines.len() + 2);
    let kept: Vec<usize> = lines.iter().flatten().copied().collect();
    let found = impacts
        .iter()
        .filter(|i| kept.binary_search(i).is_ok())
        .count();
    // Performance's 3,949 came from edges `100 × c / 1000`; the scale's
    // inverse rounds 246 of the 1,001 edges an ulp apart from those, which
    // moves 127 boundaries that fall exactly on a sample by one (D15.1).
    assert_eq!((lines.len(), lines.points(), found), (1, 3947, 50));
}

#[test]
fn the_trace_for_vector_output_takes_four_columns_per_px() {
    let thinned = thinned(Target::Vector, 4.0, 1.0);
    list::check(&thinned.list).expect("a valid list");
    assert_eq!(thinned.area.width(), 1000.0);
    assert_eq!(points(&thinned.path), thinned.places);
    let lines = &thinned.lines;
    assert!(lines.points() <= 4 * 4000 + 2 * lines.len() + 2);
    // Performance: 15,429, with edges `100 × c / 4000` (see above).
    assert_eq!(lines.points(), 15_427);
}

/// The plot area of the prototype's figure with every sample stroked,
/// rendered at `scale`: 1 px wide at 1×, the theme's 2 px at 2×. What
/// thinned lines are measured against.
#[cfg(feature = "raster")]
fn every_sample(scale: f64) -> &'static Image {
    static RENDERS: [OnceLock<Image>; 2] = [OnceLock::new(), OnceLock::new()];
    RENDERS[usize::from(scale > 1.0)].get_or_init(|| {
        let (fig, size, _) = prototype(Thin::Off, scale);
        let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
        assert_eq!(vertices(paths(&list)[0]), 1_000_000);
        render(&list, list.parts[1].bounds, scale)
    })
}

/// The share of pixels with a channel that differs by more than 16/255.
#[cfg(feature = "raster")]
fn pixels_off(a: &Image, b: &Image) -> f64 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    let pixels = a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4));
    let off = pixels
        .filter(|(p, q)| p.iter().zip(*q).any(|(c, d)| c.abs_diff(*d) > 16))
        .count();
    off as f64 / f64::from(a.width * a.height)
}

/// Each pixel column's topmost and bottommost inked row: a pixel at least
/// half covered by the line, its red channel half-way or more from white
/// to the line's 30. A fainter anti-aliasing fringe is not where a reader
/// sees the band end; counting it moves a column's end by a whole impact
/// whenever a 1 px impact line sits near a pixel edge (D15.1).
#[cfg(feature = "raster")]
fn ink(image: &Image) -> Vec<Option<(u32, u32)>> {
    (0..image.width)
        .map(|x| {
            let inked = |y: &u32| image.rgba[((y * image.width + x) * 4) as usize] <= 142;
            let top = (0..image.height).find(inked)?;
            let bottom = (0..image.height).rev().find(inked)?;
            Some((top, bottom))
        })
        .collect()
}

/// The share of pixel columns whose top or bottom ink is more than 1 px
/// away from the other image's.
#[cfg(feature = "raster")]
fn columns_off(a: &Image, b: &Image) -> f64 {
    let off = ink(a)
        .into_iter()
        .zip(ink(b))
        .filter(|(p, q)| match (p, q) {
            (Some((t, b)), Some((u, c))) => t.abs_diff(*u) > 1 || b.abs_diff(*c) > 1,
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
        })
        .count();
    off as f64 / f64::from(a.width)
}

#[cfg(feature = "raster")]
#[test]
fn the_trace_for_vector_output_is_within_5_percent_of_every_sample_stroked() {
    let thinned = thinned(Target::Vector, 4.0, 1.0);
    let image = render(&thinned.list, thinned.area, 1.0);
    let off = pixels_off(&image, every_sample(1.0));
    assert!(off <= 0.05, "{:.2}% of pixels off", 100.0 * off);
}

#[cfg(feature = "raster")]
#[test]
fn the_trace_on_screen_is_within_12_percent_and_its_columns_within_3_percent() {
    let thinned = thinned(Target::Screen { scale: 1.0 }, 1.0, 1.0);
    let image = render(&thinned.list, thinned.area, 1.0);
    let off = pixels_off(&image, every_sample(1.0));
    let columns = columns_off(&image, every_sample(1.0));
    assert!(off <= 0.12, "{:.2}% of pixels off", 100.0 * off);
    assert!(columns <= 0.03, "{:.2}% of columns off", 100.0 * columns);
}

/// At 2× the line has the theme's width: a 1 px line at one column per
/// device px stripes at 2×, 24% of its columns off (D15.3).
#[cfg(feature = "raster")]
#[test]
fn the_trace_at_2x_keeps_the_bounds_per_device_pixel() {
    let file = thinned(Target::File { scale: 2.0 }, 8.0, 2.0);
    let image = render(&file.list, file.area, 2.0);
    let off = pixels_off(&image, every_sample(2.0));
    assert!(off <= 0.05, "file: {:.2}% of pixels off", 100.0 * off);
    let screen = thinned(Target::Screen { scale: 2.0 }, 2.0, 2.0);
    let image = render(&screen.list, screen.area, 2.0);
    let off = pixels_off(&image, every_sample(2.0));
    let columns = columns_off(&image, every_sample(2.0));
    assert!(off <= 0.12, "screen: {:.2}% of pixels off", 100.0 * off);
    assert!(
        columns <= 0.03,
        "screen: {:.2}% of columns off",
        100.0 * columns
    );
}
