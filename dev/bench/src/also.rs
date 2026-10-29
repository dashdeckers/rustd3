//! What earlier phases asked the benchmark to measure beside its
//! scenarios (D19.3, D18.3, D15.3, D21.1, D22.4, Phase 31's gutters, Phase
//! 31g's software compositing): figure-level renders, resolves and lists,
//! the widget's tiles drawn on iced's headless renderers, a real gesture
//! frame composed by its software renderer ([`compose`]), its renders per
//! frame of a pan and one tile of a long trace's geometry, timed in this
//! build, written as the report's "Also measured" section.

mod compose;

use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use iced::advanced::renderer::{self as iced_renderer, Headless as _};
use iced::advanced::{Renderer as _, shell};
use iced::{Event, mouse, window};
use iced_runtime::user_interface::{self, UserInterface};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_core::series::{Clock, Recording};
use rustd3_examples::trace as example;
use rustd3_figure::kurbo::{Affine, Point, Rect, Size};
use rustd3_figure::list::{self, Bounds, Command, List, Outlines};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::raster::{self, Image, Settings, Simd};
use rustd3_figure::thin::Thin;
use rustd3_figure::{Cache, Figure, Held, Target, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{Sample, Target as Slot};
use rustd3_iced::plot::{Probe, Render, cost};

use crate::reference::{first_area, median_render_ms, zoomed_tile};
use crate::scenario::WINDOW;
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
        compose::frames(),
        compose::per_images(),
        pan_frames(),
        gutter_costs(),
        tile_geometry(),
        splat_speed(),
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
            Sample::Raster { .. } | Sample::Stage { .. } | Sample::Mismatch { .. } => None,
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

/// Phase 31's gutters: per frame of a `pan-x` replay over the `trace`
/// workload (full size, 1600 × 1000 logical at 1 and 2, `Render::Auto`,
/// the default pool, headless without drawing), the renders of the frame,
/// inline in its update and on the pool until the pool falls silent for
/// 300 ms: tiles and cells (gutter runs and overlays) apart, their time,
/// and the share of frames whose cells all rendered inline; means over 40
/// frames after the press.
fn pan_frames() -> Row {
    let mut parts = Vec::new();
    for backend in ["tiny-skia", "wgpu"] {
        for k in [1.0_f32, 2.0] {
            parts.push(match pan_counts(backend, k) {
                Some(c) => format!(
                    "{backend} at {k}: inline {:.1} tiles + {:.1} cells, {:.2} ms; pool {:.1} tiles + \
                     {:.1} cells, {:.2} ms; cells all inline on {:.0}% of frames",
                    c.inline[0], c.inline[1], c.inline[2], c.pool[0], c.pool[1], c.pool[2], c.share
                ),
                None => format!("{backend} at {k}: no headless renderer"),
            });
        }
    }
    Row {
        item: "Phase 31: renders per frame of a pan-x over trace (1600 × 1000, Auto, pool drained)"
            .to_owned(),
        value: format!("{} (means of 40 frames)", parts.join("; ")),
    }
}

/// What [`pan_counts`] measures: per frame, (tiles, cells, ms) inline and
/// on the pool, and the share of frames whose cells all rendered inline,
/// percent.
struct Counts {
    inline: [f64; 3],
    pool: [f64; 3],
    share: f64,
}

/// [`pan_frames`] on `backend` at factor `k`; `None` without a renderer
/// for it.
fn pan_counts(backend: &str, k: f32) -> Option<Counts> {
    let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced_renderer::Settings::default(),
        Some(backend),
    ))?;
    let data = Workload::Trace.data(1.0);
    let (tx, rx) = std::sync::mpsc::channel();
    let plot = Plot::<()>::new(data.figure(false))
        .scale_factor(k)
        .crosshair(false)
        .highlight(false)
        .probe(Probe::new(Instant::now, tx));
    let bounds = iced::Size::new(WINDOW.width, WINDOW.height);
    let mut ui = UserInterface::build(
        plot,
        bounds,
        user_interface::Cache::default(),
        &mut renderer,
    );
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = shell::Waker::new({
        let wakes = Arc::clone(&wakes);
        move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        }
    });
    let mut driven = Driven {
        renderer,
        bus: shell::Bus::new(),
        waker,
        wakes,
    };
    let length = 600.0_f32.min(0.6 * bounds.width);
    let mut at = iced::Point::new(0.5 * bounds.width + 0.5 * length, 0.27 * bounds.height);
    let mut now = Instant::now();
    driven.update(
        &mut ui,
        &[Event::Window(window::Event::RedrawRequested(now))],
        at,
    );
    driven.drain(&mut ui, at, Duration::from_secs(2));
    now += Duration::from_millis(16);
    let press = [
        Event::Mouse(mouse::Event::CursorMoved { position: at }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Window(window::Event::RedrawRequested(now)),
    ];
    driven.update(&mut ui, &press, at);
    driven.drain(&mut ui, at, Duration::from_millis(300));
    let _ = rx.try_iter().count();
    let (mut direction, mut swept, mut moves) = (-1.0_f32, 0.0_f32, 2_u32);
    let (mut inline, mut pool, mut all_inline) = ([0.0; 3], [0.0; 3], 0_u32);
    let frames = 40;
    for _ in 0..frames {
        let mut events = Vec::new();
        for _ in 0..2 {
            if swept >= length {
                direction = -direction;
                swept = 0.0;
            }
            let step = ((5.0 + 3.0 * (f64::from(moves) * 0.7).sin()) as f32).min(length - swept);
            at.x += direction * step;
            swept += step;
            moves += 1;
            events.push(Event::Mouse(mouse::Event::CursorMoved { position: at }));
        }
        now += Duration::from_millis(16);
        events.push(Event::Window(window::Event::RedrawRequested(now)));
        driven.update(&mut ui, &events, at);
        driven.drain(&mut ui, at, Duration::from_millis(300));
        let mut pooled_cells = 0;
        for sample in rx.try_iter() {
            if let Sample::Raster {
                target,
                duration,
                inline: here,
                ..
            } = sample
            {
                let sum = if here { &mut inline } else { &mut pool };
                match target {
                    Slot::Tile { .. } => sum[0] += 1.0,
                    Slot::Cell(_) => sum[1] += 1.0,
                }
                sum[2] += duration.as_secs_f64() * 1e3;
                if !here && matches!(target, Slot::Cell(_)) {
                    pooled_cells += 1;
                }
            }
        }
        all_inline += u32::from(pooled_cells == 0);
    }
    let n = f64::from(frames);
    Some(Counts {
        inline: inline.map(|v| v / n),
        pool: pool.map(|v| v / n),
        share: 100.0 * f64::from(all_inline) / n,
    })
}

/// A widget driven headlessly: its renderer, the messages it publishes,
/// and a waker that counts its wakes.
struct Driven {
    renderer: iced::Renderer,
    bus: shell::Bus<()>,
    waker: shell::Waker,
    wakes: Arc<AtomicUsize>,
}

impl Driven {
    /// Updates `ui` with `events`, the pointer at `at`.
    fn update(
        &mut self,
        ui: &mut UserInterface<'_, (), iced::Theme, iced::Renderer>,
        events: &[Event],
        at: iced::Point,
    ) {
        let _ = ui.update(
            &window::Headless,
            &self.waker,
            events,
            mouse::Cursor::Available(at),
            &mut self.renderer,
            &mut self.bus,
        );
    }

    /// Lands the pool's results, an `Event::Waken` after each wake, until
    /// no wake comes for `quiet`.
    fn drain(
        &mut self,
        ui: &mut UserInterface<'_, (), iced::Theme, iced::Renderer>,
        at: iced::Point,
        quiet: Duration,
    ) {
        let mut seen = self.wakes.load(Ordering::SeqCst);
        loop {
            let begin = Instant::now();
            while self.wakes.load(Ordering::SeqCst) == seen && begin.elapsed() < quiet {
                std::thread::sleep(Duration::from_millis(1));
            }
            let wakes = self.wakes.load(Ordering::SeqCst);
            if wakes == seen {
                return;
            }
            seen = wakes;
            self.update(ui, &[Event::Waken], at);
        }
    }
}

/// Phase 31's gutters: what one render of the `trace` workload's bottom
/// axis strip (the cell under its bottom plot area, 1600 × 1000 at 1 and
/// 2) spends its time on, as a pool thread renders it (single-threaded):
/// the list as drawn, without its glyph runs, without its strokes, and no
/// command at all (the fixed cost of a render of that size); and the same
/// for the overlay over the plot area's bottom edge (two device rows);
/// medians of 15.
fn gutter_costs() -> Row {
    let data = Workload::Trace.data(1.0);
    let fig = data.figure(false);
    let mut parts = Vec::new();
    for k in [1.0, 2.0] {
        let target = Target::Screen { scale: k };
        let size = Size::new(f64::from(WINDOW.width), f64::from(WINDOW.height));
        let mut cache = Cache::default();
        let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
        let Some(&area) = resolved.plot_areas().get(1) else {
            return failed("Phase 31 gutters", "no bottom plot area");
        };
        let list = fig.draw_resolved(&mut cache, &resolved);
        let (_, ys) = resolved.edges();
        let below = ys
            .iter()
            .copied()
            .filter(|&y| y > area.y1)
            .fold(size.height, f64::min);
        let strip = Rect::new(area.x0, area.y1, area.x1, below);
        let edge = Rect::new(area.x0, area.y1 - 2.0 / k, area.x1, area.y1);
        let without = |drop: fn(&Command) -> bool| List {
            commands: list.commands.iter().filter(|c| !drop(c)).cloned().collect(),
            parts: Vec::new(),
            ..list.clone()
        };
        let lists = [
            ("as drawn", list.clone()),
            (
                "without glyphs",
                without(|c| matches!(c, Command::Glyphs(_))),
            ),
            (
                "without strokes",
                without(|c| matches!(c, Command::Stroke { .. })),
            ),
            ("no command", without(|_| true)),
        ];
        for (what, rect) in [("strip", strip), ("edge overlay", edge)] {
            let times: Vec<String> = lists
                .iter()
                .map(|(name, list)| format!("{name} {:.2}", render_ms(list, rect, k)))
                .collect();
            let glyphs = runs_in(&list, rect);
            parts.push(format!(
                "{what} at {k} ({} × {} px, {glyphs} glyph runs): {} ms",
                (rect.width() * k).round(),
                (rect.height() * k).round(),
                times.join(", ")
            ));
        }
    }
    Row {
        item: "Phase 31: one axis gutter render of trace, single-threaded (1600 × 1000)".to_owned(),
        value: format!("{} (medians of 15)", parts.join("; ")),
    }
}

/// D31.2: the `scatter100k` plot area (1600 × 1000 at 1 and 2) rendered
/// single-threaded as the pool renders it, its markers filled and
/// splatted (`Renderer::splat`), medians of 5, and the largest channel
/// difference.
fn splat_speed() -> Row {
    let data = Workload::Scatter100k.data(1.0);
    let fig = data.figure(false);
    let mut parts = Vec::new();
    for k in [1.0, 2.0] {
        let Some((list, area)) = first_area(&fig, k) else {
            return failed("D31.2", "no plot area");
        };
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let mut renderer = raster::Renderer::new(Settings {
            threads: 0,
            simd: Simd::Detect,
        });
        let mut time = |splat: bool| {
            renderer.splat(splat);
            let mut last = None;
            let times: Vec<f64> = (0..5)
                .map(|_| {
                    let begin = Instant::now();
                    last = renderer.render(&list, &bounds, area, k).ok();
                    begin.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            (percentile(&times, 50.0).unwrap_or(0.0), last)
        };
        let (filled, a) = time(false);
        let (splatted, b) = time(true);
        let (Some(a), Some(b)) = (a, b) else {
            return failed("D31.2", "a render failed");
        };
        let max = a
            .rgba
            .iter()
            .zip(&b.rgba)
            .map(|(p, q)| p.abs_diff(*q))
            .max()
            .unwrap_or(0);
        parts.push(format!(
            "at {k}: filled {filled:.1} ms, splatted {splatted:.1} ms ({:.2}×), max {max}/255",
            filled / splatted
        ));
    }
    Row {
        item: "D31.2: the scatter100k plot area, single-threaded, markers filled and splatted"
            .to_owned(),
        value: format!("{} (medians of 5)", parts.join("; ")),
    }
}

/// Phase 31g: one tile of `trace` mid-zoom, as the pool renders it: the
/// top plot area's layer drawn with the widget's overscan of two tiles at
/// the x domain a zoom of ×1.6 about the area's centre gives (1600 × 1000
/// at 1 and 2), and a 256-device-px tile of it (its second column, across
/// the area's middle) rendered single-threaded (medians of 15): its time,
/// the stroke segments of the commands that meet it against those that
/// reach it (within half the stroke's width and a device px), kurbo's
/// stroke expansion of those commands' whole paths, and the tile's fixed
/// cost (no command).
fn tile_geometry() -> Row {
    let data = Workload::Trace.data(1.0);
    let fig = data.figure(false);
    let mut parts = Vec::new();
    for k in [1.0, 2.0] {
        let (list, tile) = match zoomed_tile(&fig, k) {
            Ok(found) => found,
            Err(why) => return failed("Phase 31g: a trace tile", why),
        };
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let (mut segments, mut reaching, mut paths) = (0, 0, Vec::new());
        // Each open group's space in list px: a path's points are in its
        // group's.
        let mut spaces = vec![Affine::IDENTITY];
        for (i, command) in list.commands.iter().enumerate() {
            let space = spaces.last().copied().unwrap_or(Affine::IDENTITY);
            let (path, style) = match command {
                Command::Stroke { path, style, .. } => (path, style),
                Command::Push { transform, .. } => {
                    spaces.push(space * *transform);
                    continue;
                }
                Command::Pop => {
                    spaces.pop();
                    continue;
                }
                Command::Fill { .. }
                | Command::Markers { .. }
                | Command::Glyphs(_)
                | Command::Image { .. }
                // `Command` is #[non_exhaustive] in rustd3-figure.
                | _ => continue,
            };
            if !bounds
                .get(i)
                .is_some_and(|b| b.intersect(tile).area() > 0.0)
            {
                continue;
            }
            let reach = 0.5 * style.width + 1.0 / k;
            let grown = space
                .inverse()
                .transform_rect_bbox(tile)
                .inflate(reach, reach);
            let mut last = rustd3_figure::kurbo::Point::ORIGIN;
            for element in path.elements() {
                if let rustd3_figure::kurbo::PathEl::LineTo(p) = *element {
                    segments += 1;
                    let seg = Rect::from_points(last, p);
                    reaching += usize::from(
                        seg.x0 <= grown.x1
                            && grown.x0 <= seg.x1
                            && seg.y0 <= grown.y1
                            && grown.y0 <= seg.y1,
                    );
                    last = p;
                } else if let rustd3_figure::kurbo::PathEl::MoveTo(p) = *element {
                    last = p;
                }
            }
            paths.push((path.clone(), style.clone()));
        }
        let render = render_ms(&list, tile, k);
        let expand = {
            let times: Vec<f64> = (0..15)
                .map(|_| {
                    let begin = Instant::now();
                    for (path, style) in &paths {
                        let outline = rustd3_figure::kurbo::stroke(
                            path.elements().iter().copied(),
                            style,
                            &rustd3_figure::kurbo::StrokeOpts::default(),
                            0.25 / k,
                        );
                        std::hint::black_box(outline);
                    }
                    begin.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            percentile(&times, 50.0).unwrap_or(0.0)
        };
        let empty = List {
            commands: Vec::new(),
            parts: Vec::new(),
            ..list.clone()
        };
        let fixed = render_ms(&empty, tile, k);
        parts.push(format!(
            "at {k}: {render:.1} ms; {} stroked paths meet it with {segments} segments, \
             {reaching} of them within its reach; their stroke expansion alone {expand:.1} ms; no \
             command {fixed:.2} ms",
            paths.len()
        ));
    }
    Row {
        item: "Phase 31g: one trace tile (256 device px) mid-zoom, single-threaded".to_owned(),
        value: format!("{} (medians of 15)", parts.join("; ")),
    }
}

/// The warm composition of a `trace` frame mid-zoom by iced's software
/// renderer at factor 1 (Phase 31g's harness, [`compose`]), ms; `None`
/// without a headless tiny-skia renderer.
pub fn composition_ms() -> Option<f64> {
    compose::zoom_frame_ms()
}

/// The median time of 15 renders of `rect` (list px) of `list` at `k`
/// with a single-threaded renderer, after one to warm it, ms.
fn render_ms(list: &List, rect: Rect, k: f64) -> f64 {
    median_render_ms(list, rect, k, false, 15)
}

/// The glyph runs of `list` whose bounds meet `rect` (list px).
fn runs_in(list: &List, rect: Rect) -> usize {
    let bounds = Bounds::of(list, &mut Outlines::default());
    list.commands
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            matches!(c, Command::Glyphs(_))
                && bounds
                    .get(*i)
                    .is_some_and(|b| b.intersect(rect).area() > 0.0)
        })
        .count()
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
