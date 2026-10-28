//! What earlier phases asked the benchmark to measure beside its
//! scenarios (D19.3, D18.3, D15.3, D21.1, D22.4): figure-level renders,
//! resolves and lists, and the widget's tiles drawn on iced's headless
//! renderers, timed in this build, written as the report's "Also
//! measured" section.

use std::fmt::Write as _;
use std::time::Instant;

use iced::advanced::renderer::{self as iced_renderer, Headless as _};
use iced::advanced::{Renderer as _, shell};
use iced::{Event, mouse, window};
use iced_runtime::user_interface::{self, UserInterface};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_core::series::{Clock, Recording};
use rustd3_examples::trace as example;
use rustd3_figure::kurbo::{Point, Rect, Size};
use rustd3_figure::list::{self, Bounds, List, Outlines};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::raster::{self, Image, Settings, Simd};
use rustd3_figure::thin::Thin;
use rustd3_figure::{Cache, Figure, Held, Target, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::Sample;
use rustd3_iced::plot::{Probe, Render, cost};

use crate::stats::percentile;
use crate::workload::Workload;

/// One measured item.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// What was measured, with the decision that asked for it.
    pub item: String,
    /// The result.
    pub value: String,
}

/// Measures every item and returns the report section.
pub fn measure() -> String {
    let rows = [
        dense_scatter(),
        origin_dashes(),
        thin_line(),
        windowed_resolve(),
        scatter_lists(),
        tile_draw(),
    ];
    let mut md = String::from(
        "## Also measured\n\nFigure-level items earlier phases asked for, measured in this build \
         and process (not in a child):\n\n| Item | Result |\n| --- | --- |\n",
    );
    for row in rows {
        let _ = writeln!(md, "| {} | {} |", row.item, row.value);
    }
    md
}

/// D19.3: the 200k-dot plot-area cell of `iced/tests/main/pipeline.rs`'s
/// scatter at 1600 × 1000 logical at 2, rendered as the widget's worker
/// renders it (vello_cpu's pool), against the cost model.
fn dense_scatter() -> Row {
    let n = 200_000;
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let points: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = ((i * 7919) % 1000) as f64 / 1000.0;
            (t * 10.0, (t * 25.0).sin() + wobble)
        })
        .collect();
    fig.panel(0, 0)
        .mark(dots(points, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    let (k, size) = (2.0, Size::new(1600.0, 1000.0));
    let (list, cells) = cut(&fig, size, k);
    let Some(cell) = cells.cells.iter().max_by_key(|cell| cell.work) else {
        return failed("D19.3: 200k-dot plot-area cell, release", "no cell");
    };
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let mut renderer = raster::Renderer::new(Settings::parallel());
    let area = scaled(cell.rect, k);
    let times: Vec<f64> = (0..3)
        .map(|_| {
            let begin = Instant::now();
            let _ = renderer.render(&list, &bounds, area, k);
            begin.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    let measured = percentile(&times, 50.0).unwrap_or(0.0);
    let pixels = (cell.rect.width() * cell.rect.height()) as u64;
    let model = cost::model_ms(pixels, cell.work);
    Row {
        item: "D19.3: the 200k-dot plot-area cell (1600 × 1000 at 2), vello_cpu's pool".to_owned(),
        value: format!(
            "{measured:.1} ms (median of 3; {pixels} px, {} items); model {model:.1} ms; calibration \
             {:.3}",
            cell.work,
            measured / model
        ),
    }
}

/// D18.3: the trace example at 2×: its cells rendered at their own
/// origins, stitched, against the whole canvas rendered at once
/// (single-threaded, baseline SIMD, as D18.3 measured).
fn origin_dashes() -> Row {
    let rec = example::recording(example::SAMPLES);
    let fig = example::trace(&rec);
    let k = example::SCALE;
    let (list, cells) = cut(&fig, example::SIZE, k);
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let mut renderer = raster::Renderer::new(Settings {
        threads: 0,
        simd: Simd::Baseline,
    });
    let whole = Rect::from_origin_size(Point::ORIGIN, list.size);
    let Ok(whole) = renderer.render(&list, &bounds, whole, k) else {
        return failed("D18.3", "the whole canvas did not render");
    };
    let (mut over, mut over16, mut max) = (0_u64, 0_u64, 0_u8);
    for cell in &cells.cells {
        let Ok(image) = renderer.render(&list, &bounds, scaled(cell.rect, k), k) else {
            return failed("D18.3", "a cell did not render");
        };
        let (x0, y0) = (cell.rect.x0 as u32, cell.rect.y0 as u32);
        for y in 0..image.height {
            for x in 0..image.width {
                let diff = channel_diff(&image, x, y, &whole, x0 + x, y0 + y);
                max = max.max(diff);
                over += u64::from(diff > 2);
                over16 += u64::from(diff > 16);
            }
        }
    }
    Row {
        item: format!(
            "D18.3: the trace example ({} × {} at 2, 3 × 1M samples), {} cells stitched against the \
             whole render",
            example::SIZE.width,
            example::SIZE.height,
            cells.cells.len()
        ),
        value: format!("{over} px over 2/255 ({over16} over 16/255), max {max}/255"),
    }
}

/// D15.3: a 1 px line thinned at one column per device px on a 2×
/// display, against every sample stroked: the share of pixel columns
/// whose top or bottom ink moves more than 1 px, and of pixels off by
/// more than 16/255 (the prototype's view of the trace, as Phase 15).
fn thin_line() -> Row {
    let (x, y, _) = example::signal(1_000_000, 42);
    let Ok(rec) = Recording::new(Clock::Explicit(x), [y]) else {
        return failed("D15.3", "the recording");
    };
    let k = 2.0;
    let render = |thin: Thin, target: Target| -> Option<Image> {
        let mut fig = Figure::new();
        let t = fig.scale(scale::linear().domain([0.0, 100.0]));
        let v = fig.scale(scale::linear().domain([-2.4, 2.4]));
        fig.panel(0, 0).mark(
            line(rec.series(0)?, t, v)
                .width(1.0)
                .color(Rgb::new(30.0, 60.0, 140.0))
                .thin(thin),
        );
        let pad = 2.0 * 0.8 * 44.0 / 3.0;
        let size = Size::new(1000.0 + pad, 240.0 + pad);
        let mut cache = Cache::default();
        let list = fig.draw_with(&mut cache, size, target, &Held::default());
        let area = *fig
            .resolve(size, &mut cache, &Held::default(), target)
            .plot_areas()
            .first()?;
        let bounds = Bounds::of(&list, &mut Outlines::default());
        raster::Renderer::new(Settings {
            threads: 0,
            simd: Simd::Baseline,
        })
        .render(&list, &bounds, area, k)
        .ok()
    };
    let thinned = render(Thin::Auto, Target::Screen { scale: k });
    let every = render(Thin::Off, Target::Vector);
    let (Some(thinned), Some(every)) = (thinned, every) else {
        return failed("D15.3", "a render failed");
    };
    if (thinned.width, thinned.height) != (every.width, every.height) {
        return failed("D15.3", "the two renders differ in size");
    }
    let pixels = (0..thinned.height)
        .flat_map(|y| (0..thinned.width).map(move |x| (x, y)))
        .filter(|&(x, y)| channel_diff(&thinned, x, y, &every, x, y) > 16)
        .count() as f64
        / f64::from(thinned.width * thinned.height);
    let (a, b) = (ink(&thinned), ink(&every));
    let columns = a
        .iter()
        .zip(&b)
        .filter(|(p, q)| match (p, q) {
            (Some((t, b)), Some((u, c))) => t.abs_diff(*u) > 1 || b.abs_diff(*c) > 1,
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
        })
        .count() as f64
        / f64::from(thinned.width);
    Row {
        item: "D15.3: a 1 px line thinned at one column per device px at 2×, against every sample"
            .to_owned(),
        value: format!(
            "{:.1}% of columns off by more than 1 px (Phase 15: 24.3%); {:.2}% of pixels off by \
             more than 16/255 (4.32%)",
            100.0 * columns,
            100.0 * pixels
        ),
    }
}

/// D21.1: resolving the `trace` workload with x on `Fit::Window(10.0)` at
/// 1600 × 1000 at 2: from a fresh cache (two rounds: the window's column
/// count is not known yet) and in a steady layout after an append (one
/// round), with the labels the text cache looked up per resolve.
fn windowed_resolve() -> Row {
    let target = Target::Screen { scale: 2.0 };
    let size = Size::new(1600.0, 1000.0);
    let mut data = Workload::Trace.data(1.0);
    let lookups = |cache: &Cache| {
        let stats = cache.stats().text;
        stats.hits + stats.misses
    };
    let mut fresh = Vec::new();
    let (mut fresh_lookups, mut fresh_passes) = (0, 0);
    for _ in 0..5 {
        let fig = data.figure(true);
        let mut cache = Cache::default();
        let begin = Instant::now();
        let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
        fresh.push(begin.elapsed().as_secs_f64() * 1e3);
        fresh_lookups = lookups(&cache);
        fresh_passes = resolved.passes();
    }
    let mut cache = Cache::default();
    let _ = data
        .figure(true)
        .resolve(size, &mut cache, &Held::default(), target);
    let mut steady = Vec::new();
    let (mut steady_lookups, mut steady_passes) = (0, 0);
    for _ in 0..5 {
        data.append(1_000);
        let fig = data.figure(true);
        let before = lookups(&cache);
        cache.begin();
        let begin = Instant::now();
        let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
        steady.push(begin.elapsed().as_secs_f64() * 1e3);
        steady_lookups = lookups(&cache) - before;
        steady_passes = resolved.passes();
    }
    let median = |values: &[f64]| percentile(values, 50.0).unwrap_or(0.0);
    Row {
        item:
            "D21.1: resolving the windowed trace (1600 × 1000 at 2), fresh cache against a steady \
               layout after an append"
                .to_owned(),
        value: format!(
            "fresh {:.2} ms, {fresh_lookups} label lookups, last round {fresh_passes} layout \
             pass(es); steady {:.2} ms, {steady_lookups} lookups, {steady_passes} pass(es) \
             (medians of 5; a lookup is one label in one layout pass of one round)",
            median(&fresh),
            median(&steady)
        ),
    }
}

/// D22.4 (4): the UI thread's work per frame of a pan on the scatters at
/// full size, 1600 × 1000 at 1, as the widget does it: resolving with both
/// handles held at the pan's domains (3 px further each frame) and
/// drawing the list (the probe's resolve and list stages), then the
/// list's bounds, its cells and the hit index; medians of 9 frames over a
/// warm cache.
fn scatter_lists() -> Row {
    let target = Target::Screen { scale: 1.0 };
    let size = Size::new(1600.0, 1000.0);
    let mut parts = Vec::new();
    for workload in [Workload::Scatter100k, Workload::Scatter1m] {
        let data = workload.data(1.0);
        let fig = data.figure(false);
        let mut cache = Cache::default();
        let fresh = fig.resolve(size, &mut cache, &Held::default(), target);
        let Some(&area) = fresh.plot_areas().first() else {
            return failed("D22.4 (4)", "no plot area");
        };
        let ranges = [(0, [0.0, area.width()]), (1, [area.height(), 0.0])];
        let mut stages: [Vec<f64>; 5] = Default::default();
        for frame in 0..10 {
            let mut held = Held::default();
            for (index, range) in ranges {
                let dx = if index == 0 {
                    3.0 * f64::from(frame)
                } else {
                    0.0
                };
                let Some(scale) = fresh.scale_at(index).and_then(|s| s.pan_px(range, dx)) else {
                    return failed("D22.4 (4)", "a handle that does not pan");
                };
                held.hold_erased(index, scale);
            }
            let mut clock = Instant::now();
            let mut lap = |stage: &mut Vec<f64>| {
                let now = Instant::now();
                if frame > 0 {
                    stage.push((now - clock).as_secs_f64() * 1e3);
                }
                clock = now;
            };
            cache.begin();
            let resolved = fig.resolve(size, &mut cache, &held, target);
            lap(&mut stages[0]);
            let list = fig.draw_resolved(&mut cache, &resolved);
            lap(&mut stages[1]);
            let _ = Bounds::of(&list, &mut Outlines::default());
            lap(&mut stages[2]);
            let (xs, ys) = resolved.edges();
            let _ = list::cells(&list, &xs, &ys, 1.0);
            lap(&mut stages[3]);
            let _ = fig.hits(&resolved, &mut cache);
            lap(&mut stages[4]);
        }
        let [resolve, draw, bounds, cells, hits] =
            stages.map(|times| percentile(&times, 50.0).unwrap_or(0.0));
        parts.push(format!(
            "{}: resolve {resolve:.2} ms, list {draw:.2} ms; then bounds {bounds:.2}, cells \
             {cells:.2}, hits {hits:.2} ms",
            workload.name()
        ));
    }
    Row {
        item: "D22.4 (4): a pan's UI-thread work on the scatters (1600 × 1000 at 1, full size)"
            .to_owned(),
        value: format!("{} (medians of 9 frames)", parts.join("; ")),
    }
}

/// D22.4 (3): a 1600 × 1000 device-px plot area of tiles drawn per frame
/// of a pan at factor 1 on iced's headless renderers: a drag in steps of 3
/// px, every visible tile translated (drawn at whole device pixels, no
/// resampling); per frame the widget's `draw` and the renderer composing
/// the frame (`screenshot`, which on wgpu also reads it back), the ring's
/// renders in the update before them excluded; medians of 30 frames.
fn tile_draw() -> Row {
    let parts: Vec<String> = ["tiny-skia", "wgpu"]
        .into_iter()
        .map(|backend| match tile_frames(backend) {
            Some(([draw, compose], [images, empty])) => format!(
                "{backend}: draw {draw:.2} ms, compose {compose:.2} ms for {images} images \
                 (an empty figure's frame, one image: {empty:.2} ms)"
            ),
            None => format!("{backend}: no headless renderer"),
        })
        .collect();
    Row {
        item: "D22.4 (3): a 1600 × 1000 plot area of tiles drawn per frame of a pan (headless, \
               factor 1)"
            .to_owned(),
        value: format!("{} (medians of 30 frames)", parts.join("; ")),
    }
}

/// The medians of [`tile_draw`]'s two stages on `backend`, ms, the images
/// a frame draws, and the median compose of an empty figure's frame of
/// the same size; `None` without a renderer for it.
fn tile_frames(backend: &str) -> Option<([f64; 2], [f64; 2])> {
    let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced_renderer::Settings::default(),
        Some(backend),
    ))?;
    renderer.hint(iced_renderer::Scale {
        window: 1.0,
        application: 1.0,
    });
    let curve: Vec<(f64, f64)> = (0..10_000)
        .map(|i| {
            let t = f64::from(i) * 0.01;
            (t, (t * 0.7).sin() + 0.2 * (t * 5.0).sin())
        })
        .collect();
    let figure = || {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([20.0, 80.0]));
        let y = fig.scale(scale::linear().domain([-1.5, 1.5]));
        fig.panel(0, 0)
            .mark(line(curve.clone(), x, y))
            .axis(axis::left(y))
            .axis(axis::bottom(x))
            .zoom_x(x);
        fig
    };
    // The window whose plot area is 1600 × 1000.
    let target = Target::Screen { scale: 1.0 };
    let mut size = Size::new(1700.0, 1080.0);
    let mut area = Rect::ZERO;
    for _ in 0..8 {
        let resolved = figure().resolve(size, &mut Cache::default(), &Held::default(), target);
        area = *resolved.plot_areas().first()?;
        let (dw, dh) = (1600.0 - area.width(), 1000.0 - area.height());
        if dw.abs() < 0.5 && dh.abs() < 0.5 {
            break;
        }
        size = Size::new(size.width + dw, size.height + dh);
    }
    let bounds = iced::Size::new(size.width as f32, size.height as f32);
    let physical = iced::Size::new(bounds.width.round() as u32, bounds.height.round() as u32);
    let (tx, rx) = std::sync::mpsc::channel();
    let plot = Plot::<()>::new(figure())
        .scale_factor(1.0)
        .render(Render::Inline)
        .crosshair(false)
        .highlight(false)
        .probe(Probe::new(Instant::now, tx));
    let mut ui = UserInterface::build(
        plot,
        bounds,
        user_interface::Cache::default(),
        &mut renderer,
    );
    let waker = shell::Waker::new(|| {});
    let mut bus = shell::Bus::new();
    let mut at = iced::Point::new(area.center().x as f32, area.center().y as f32);
    let style = iced_renderer::Style {
        text_color: iced::Color::BLACK,
    };
    let (mut draws, mut composes) = (Vec::new(), Vec::new());
    for frame in 0..34 {
        let mut events = match frame {
            0 => Vec::new(),
            1 => vec![
                Event::Mouse(mouse::Event::CursorMoved { position: at }),
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            ],
            _ => {
                at.x -= 3.0;
                vec![Event::Mouse(mouse::Event::CursorMoved { position: at })]
            }
        };
        events.push(Event::Window(
            window::Event::RedrawRequested(Instant::now()),
        ));
        renderer.tick();
        let _ = ui.update(
            &window::Headless,
            &waker,
            &events,
            mouse::Cursor::Available(at),
            &mut renderer,
            &mut bus,
        );
        let begin = Instant::now();
        ui.draw(
            &mut renderer,
            &iced::Theme::Light,
            &style,
            mouse::Cursor::Available(at),
        );
        let drawn = Instant::now();
        let _ = renderer.screenshot(physical, 1.0, iced::Color::WHITE);
        let composed = Instant::now();
        if frame >= 4 {
            draws.push((drawn - begin).as_secs_f64() * 1e3);
            composes.push((composed - drawn).as_secs_f64() * 1e3);
        }
    }
    let shown = rx
        .try_iter()
        .filter_map(|sample| match sample {
            Sample::Frame { images, .. } => Some(images),
            Sample::Raster { .. } | Sample::Stage { .. } => None,
        })
        .last()?;
    let mut empty = UserInterface::build(
        Plot::<()>::new(Figure::new()).render(Render::Inline),
        bounds,
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut blank = Vec::new();
    for frame in 0..14 {
        let events = [Event::Window(
            window::Event::RedrawRequested(Instant::now()),
        )];
        renderer.tick();
        let _ = empty.update(
            &window::Headless,
            &waker,
            &events,
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut bus,
        );
        empty.draw(
            &mut renderer,
            &iced::Theme::Light,
            &style,
            mouse::Cursor::Unavailable,
        );
        let begin = Instant::now();
        let _ = renderer.screenshot(physical, 1.0, iced::Color::WHITE);
        if frame >= 4 {
            blank.push(begin.elapsed().as_secs_f64() * 1e3);
        }
    }
    Some((
        [percentile(&draws, 50.0)?, percentile(&composes, 50.0)?],
        [f64::from(shown), percentile(&blank, 50.0)?],
    ))
}

/// The list of `fig` at `size` for the screen at `k`, and its cells.
fn cut(fig: &Figure<'_>, size: Size, k: f64) -> (List, list::Cells) {
    let target = Target::Screen { scale: k };
    let mut cache = Cache::default();
    let list = fig.draw_with(&mut cache, size, target, &Held::default());
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let (xs, ys) = resolved.edges();
    let cells = list::cells(&list, &xs, &ys, k);
    (list, cells)
}

/// A device-px rect in list px.
fn scaled(rect: Rect, k: f64) -> Rect {
    Rect::new(rect.x0 / k, rect.y0 / k, rect.x1 / k, rect.y1 / k)
}

/// The largest channel difference between two pixels.
fn channel_diff(a: &Image, ax: u32, ay: u32, b: &Image, bx: u32, by: u32) -> u8 {
    let i = ((ay * a.width + ax) * 4) as usize;
    let j = ((by * b.width + bx) * 4) as usize;
    (0..4)
        .map(|c| a.rgba[i + c].abs_diff(b.rgba[j + c]))
        .max()
        .unwrap_or(0)
}

/// Each pixel column's topmost and bottommost row at least half covered
/// by the (30, 60, 140) line: red at most 142, as Phase 15 measures.
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

/// A row for an item that could not be measured.
fn failed(item: &str, why: &str) -> Row {
    Row {
        item: item.to_owned(),
        value: format!("not measured: {why}"),
    }
}
