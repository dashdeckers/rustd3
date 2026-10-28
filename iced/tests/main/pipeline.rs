//! The widget's render pipeline through the headless harness on the
//! backend `ICED_TEST_BACKEND` names: cells rendered on the worker against
//! the same cells rendered inline, `Render::Auto`'s choice, the newest
//! result winning, previous
//! images staying until their replacement lands, and the worker thread's
//! lifetime.

use std::sync::{Mutex, MutexGuard, PoisonError, mpsc};
use std::time::Duration;

use iced::time::Instant;
use iced::window::RedrawRequest;
use iced::{Color, Event, Size};
use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::legend::{self, Slot};
use rustd3_figure::list;
use rustd3_figure::mark::dots;
use rustd3_figure::{Cache, Figure, Held, Target, Theme, axis, kurbo};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::{self, Sample};
use rustd3_iced::plot::{Probe, Render, cost};
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};
use crate::reference;

/// The worker thread's name in rustd3-iced.
const WORKER: &str = "rustd3-plot";

/// Tests that start a worker run one at a time, so that a thread count
/// sees only the counting test's own.
static WORKERS: Mutex<()> = Mutex::new(());

/// The expensive scatter's logical size: 6.4 million device px at 2.
const LARGE: Size = Size::new(1600.0, 1000.0);

/// The expensive scatter's dots. The cost model predicts 12.8 + 500 ms
/// at 2; with vello_cpu's crates optimized in the dev profile, a debug
/// build renders its plot area in about a second (D19.1).
const DOTS: usize = 200_000;

/// The cheap figures' logical size.
const MEDIUM: Size = Size::new(480.0, 320.0);

/// The cheap refresh's logical size: 256,000 device px at 2.
const SMALL: Size = Size::new(320.0, 200.0);

/// The lock [`WORKERS`] holds; a test that failed holding it poisons
/// nothing the others read.
pub fn workers() -> MutexGuard<'static, ()> {
    WORKERS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The clock a test's probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn clock() -> Instant {
    Instant::now()
}

/// Threads of this process named `name` (Linux: `/proc/self/task`).
#[cfg(target_os = "linux")]
fn threads(name: &str) -> usize {
    std::fs::read_dir("/proc/self/task")
        .expect("Linux lists a process's threads in /proc/self/task")
        .filter_map(Result::ok)
        .filter_map(|task| std::fs::read_to_string(task.path().join("comm")).ok())
        .filter(|comm| comm.trim_end() == name)
        .count()
}

/// `n` dots of a sine with a deterministic wobble on fitted scales, with
/// left and bottom axes.
fn scatter(n: usize) -> Figure<'static> {
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
    fig
}

/// 500 dots on fixed [0, 10] scales on a background of `background`.
fn tinted(background: u32) -> Figure<'static> {
    let mut fig = Figure::new();
    fig.theme(Theme::default().background(Rgb::hex(background)));
    fig.title("Tinted");
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(dots(wobble(500), x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// Two series of dots on fixed [0, 10] scales, with a legend titled
/// `title` right of the panel; the entries are wider than any one-letter
/// title, so the title changes no layout.
fn legend(title: &'static str) -> Figure<'static> {
    let mut fig = Figure::new();
    let color = fig.scale(scale::ordinal(
        ["first series", "second series"],
        COLORBLIND,
    ));
    fig.legend(Slot::Right, legend::legend(color).title(title));
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let points = wobble(200);
    let (first, second) = points.split_at(100);
    fig.panel(0, 0)
        .mark(dots(first.to_vec(), x, y).fill(color, "first series"))
        .mark(dots(second.to_vec(), x, y).fill(color, "second series"))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// `n` points inside [1, 9]².
fn wobble(n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let w = ((i * 7919) % 1000) as f64 / 1000.0;
            (1.0 + 8.0 * t, 5.0 + 3.0 * (t * 25.0).sin() * w)
        })
        .collect()
}

/// The device rect of cell `index` of `fig`'s partition at `size` and `k`.
fn cell(fig: &Figure<'_>, size: Size, k: f32, index: u32) -> [u32; 4] {
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
    let rect = list::cells(&drawn, &xs, &ys, k).cells[index as usize].rect;
    [rect.x0, rect.y0, rect.x1, rect.y1].map(|edge| edge as u32)
}

/// The pixel bound of `backend` against the PNG: 1/255 on tiny-skia,
/// 2/255 on wgpu.
fn bound(backend: &str) -> u8 {
    if backend == "wgpu" { 2 } else { 1 }
}

/// The widget's frame against the PNG of `fig` on the screenshot's white.
fn against_png(shot: &Shot, fig: &Figure<'_>, size: Size, k: f32) -> pixels::Diff {
    let size = kurbo::Size::new(f64::from(size.width), f64::from(size.height));
    let png = reference::png_of(fig, size, f64::from(k), Color::WHITE);
    let png = pixels::Image::new(png.width, png.height, &png.rgba)
        .expect("invariant: a render holds width × height pixels");
    pixels::Diff::between(&shot.image(), &png, 16).expect("invariant: the shot has the PNG's size")
}

/// The raster samples among `samples`: (target, generation, inline).
fn rasters(samples: &[Sample]) -> Vec<(probe::Target, u64, bool)> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target,
                generation,
                inline,
                ..
            } => Some((*target, *generation, *inline)),
            Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .collect()
}

/// The last `Frame` sample among `samples`: (generation, images,
/// missing, previews).
fn last_frame(samples: &[Sample]) -> (u64, u32, u32, u32) {
    samples
        .iter()
        .rev()
        .find_map(|sample| match sample {
            Sample::Frame {
                generation,
                images,
                missing,
                previews,
                ..
            } => Some((*generation, *images, *missing, *previews)),
            Sample::Stage { .. } | Sample::Raster { .. } => None,
        })
        .expect("a frame sample")
}

/// The cost model's time for the cells the raster samples among
/// `samples` rendered, ms; with `ring`, the tiles of the overscan ring
/// (see [`in_view`]) too.
fn modelled(samples: &[Sample], view: Option<&[[i64; 4]]>) -> f64 {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                pixels,
                work,
                target,
                ..
            } => view
                .is_none_or(|areas| in_view(*target, areas))
                .then(|| cost::model_ms(u64::from(*pixels), *work)),
            Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .sum()
}

/// Whether `target` shows in the canvas: a cell, or a tile whose square
/// meets its plot area (device rect among `areas`) on a grid anchored at
/// the area's corner; not a tile of the overscan ring.
fn in_view(target: probe::Target, areas: &[[i64; 4]]) -> bool {
    match target {
        probe::Target::Cell(_) => true,
        probe::Target::Tile { area, i, j } => {
            let [x0, y0, x1, y1] = areas[area as usize];
            let side = |a: i64, b: i64| (b - a + 255) / 256;
            (0..side(x0, x1)).contains(&i64::from(i)) && (0..side(y0, y1)).contains(&i64::from(j))
        }
    }
}

/// What each `Frame` sample among `samples` reported: (missing, previews).
fn frames(samples: &[Sample]) -> Vec<(u32, u32)> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Frame {
                missing, previews, ..
            } => Some((*missing, *previews)),
            Sample::Stage { .. } | Sample::Raster { .. } => None,
        })
        .collect()
}

/// What [`settle`] waited for: the first frame that showed every slot
/// current, the samples received meanwhile, the wakes it took and the
/// time from its start to the first wake.
struct Settled {
    shot: Shot,
    samples: Vec<Sample>,
    wakes: usize,
    first_wake: Duration,
}

/// Frames of `harness` after each wake, the `Waken` injected before
/// each and the frames `t` + 16 ms apart, until a frame shows every slot
/// current. Panics when a wake takes longer than ten seconds.
fn settle<Message: 'static>(
    harness: &mut Harness<'_, Message>,
    probe: &mpsc::Receiver<Sample>,
    mut t: Instant,
) -> Settled {
    let start = clock();
    let mut first = None;
    let mut received = Vec::new();
    for wakes in 1..=1000 {
        assert!(
            harness.wait_for_wake(Duration::from_secs(10)),
            "no wake within ten seconds"
        );
        let first = *first.get_or_insert_with(|| clock().duration_since(start));
        harness.events(&[Event::Waken]);
        t += Duration::from_millis(16);
        let shot = harness.frame(t);
        let samples: Vec<Sample> = probe.try_iter().collect();
        let current = frames(&samples).last() == Some(&(0, 0));
        received.extend(samples);
        if current {
            return Settled {
                shot,
                samples: received,
                wakes,
                first_wake: first,
            };
        }
    }
    panic!("a thousand wakes without a current frame")
}

#[test]
fn worker_mode_matches_inline_mode() {
    let _workers = workers();
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let plot = Plot::<()>::new(scatter(DOTS))
        .scale_factor(k)
        .render(Render::Worker)
        .probe(Probe::new(clock, tx));
    let Some(mut harness) = Harness::new(plot, LARGE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let first: Vec<Sample> = rx.try_iter().collect();
    let (missing, _) = *frames(&first).last().expect("a frame sample");
    assert!(
        missing > 0,
        "{backend}: the first frame waits for the worker"
    );
    let Settled {
        shot: worker,
        wakes,
        first_wake,
        ..
    } = settle(&mut harness, &rx, t0);
    let inline = Plot::<()>::new(scatter(DOTS))
        .scale_factor(k)
        .render(Render::Inline);
    let mut harness = Harness::new(inline, LARGE, k).expect("the backend found above");
    let inline = harness.frame(t0);
    let diff = pixels::Diff::between(&worker.image(), &inline.image(), 16)
        .expect("invariant: both shots have the widget's size");
    println!(
        "worker against inline, {backend}, {DOTS} dots at {k}: first wake after {first_wake:?}, \
         current after {wakes} wakes, max {}/255, {} channels over 16",
        diff.max(),
        diff.over()
    );
    assert!(diff.max() <= 1, "{backend}: max {}/255", diff.max());
}

#[cfg(target_os = "linux")]
#[test]
fn worker_exits_on_drop() {
    let _workers = workers();
    let before = threads(WORKER);
    let plot = Plot::<()>::new(scatter(DOTS))
        .scale_factor(2.0)
        .render(Render::Worker);
    let Some(mut harness) = Harness::new(plot, LARGE, 2.0) else {
        return;
    };
    assert_eq!(threads(WORKER), before, "no job, no thread");
    let _ = harness.frame(headless::epoch());
    // The worker, and vello_cpu's pool threads, which take its name.
    let during = threads(WORKER);
    assert!(during > before, "{during} threads, {before} before");
    // The job set is pending: the drop closes the queue and joins the
    // thread after the cell in flight; the pool's threads end once the
    // joined thread dropped its renderer.
    drop(harness);
    let after = (0..10_000)
        .map(|_| {
            let count = threads(WORKER);
            if count > before {
                std::thread::sleep(Duration::from_millis(1));
            }
            count
        })
        .find(|&count| count == before);
    println!("worker threads: {before} before, {during} with a job set, {after:?} after the drop");
    assert_eq!(after, Some(before));
}

#[test]
fn newest_result_wins() {
    let _workers = workers();
    let (k, t0) = (2.0, headless::epoch());
    let frame = Duration::from_millis(16);
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = |fig, render| {
        Plot::<()>::new(fig)
            .scale_factor(k)
            .render(render)
            .probe(probe.clone())
    };
    // Two refreshes on the worker before a drain: the second set replaces
    // the first; a new background changes every cell.
    let Some(mut harness) = Harness::new(plot(tinted(0xffffff), Render::Worker), MEDIUM, k) else {
        return;
    };
    let backend = harness.backend();
    let _ = harness.frame(t0);
    harness.rebuild(plot(tinted(0xf2e6d9), Render::Worker));
    let _ = harness.frame(t0 + frame);
    let settled = settle(&mut harness, &rx, t0 + frame);
    let (generation, images, missing, previews) = last_frame(&settled.samples);
    assert_eq!((generation, missing, previews), (2, 0, 0), "{backend}");
    let rendered = rasters(&settled.samples);
    let view = crate::tiles::areas(&tinted(0xf2e6d9), MEDIUM, k);
    let newest = rendered
        .iter()
        .filter(|(target, g, _)| *g == 2 && in_view(*target, &view))
        .count();
    assert_eq!(
        newest, images as usize,
        "{backend}: every cell and tile in view of generation 2"
    );
    let diff = against_png(&settled.shot, &tinted(0xf2e6d9), MEDIUM, k);
    println!(
        "two worker refreshes, {backend}: {} results of generation 1 and {newest} of 2 in {} wakes; \
         against generation 2's PNG max {}/255",
        rendered.len() - newest,
        settled.wakes,
        diff.max()
    );
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );

    // A result of generation 1 drained after generation 2 is shown: the
    // drain is held until the worker sent one, then generation 2 renders
    // inline, then the `Waken` arrives.
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let plot = |fig, render| {
        Plot::<()>::new(fig)
            .scale_factor(k)
            .render(render)
            .probe(probe.clone())
    };
    let mut harness =
        Harness::new(plot(tinted(0xffffff), Render::Worker), MEDIUM, k).expect("found above");
    let _ = harness.frame(t0);
    assert!(harness.wait_for_wake(Duration::from_secs(10)));
    harness.rebuild(plot(tinted(0xf2e6d9), Render::Inline));
    let shown = harness.frame(t0 + frame);
    let inline: Vec<Sample> = rx.try_iter().collect();
    let (_, images, missing, previews) = last_frame(&inline);
    assert_eq!((missing, previews), (0, 0), "{backend}");
    harness.events(&[Event::Waken]);
    assert_eq!(
        harness.redraw_requested(),
        RedrawRequest::Wait,
        "{backend}: a dropped result redraws nothing"
    );
    let after = harness.frame(t0 + frame * 2);
    let drained: Vec<Sample> = rx.try_iter().collect();
    let late = rasters(&drained);
    assert!(!late.is_empty(), "{backend}: the held results");
    assert!(
        late.iter().all(|&(_, g, inline)| g == 1 && !inline),
        "{late:?}"
    );
    assert_eq!(last_frame(&drained), (2, images, 0, 0), "{backend}");
    assert!(
        after.rgba == shown.rgba,
        "{backend}: generation 2 stays on screen"
    );
    println!(
        "{backend}: {} results of generation 1 drained after generation 2, dropped",
        late.len()
    );
}

#[test]
fn previous_image_stays_until_the_new_one_arrives() {
    let _workers = workers();
    for k in [1.0, 2.0] {
        let t0 = headless::epoch();
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let plot = |title| {
            Plot::<()>::new(legend(title))
                .scale_factor(k)
                .render(Render::Worker)
                .probe(probe.clone())
        };
        let Some(mut harness) = Harness::new(plot("A"), MEDIUM, k) else {
            return;
        };
        let backend = harness.backend();
        let _ = harness.frame(t0);
        let old = settle(&mut harness, &rx, t0).shot;
        // A new legend title: one gutter cell renders on the worker.
        harness.rebuild(plot("B"));
        let t = t0 + Duration::from_secs(1);
        let pending = harness.frame(t);
        let planned: Vec<Sample> = rx.try_iter().collect();
        let (generation, _, missing, previews) = last_frame(&planned);
        assert_eq!(
            (generation, missing, previews),
            (2, 0, 1),
            "{backend}, k = {k}"
        );
        assert!(
            pending.rgba == old.rgba,
            "{backend}, k = {k}: the old title stays until the new one lands"
        );
        let settled = settle(&mut harness, &rx, t);
        // Tiles of the first frame's ring may still render, after it
        // settled: of generation 2 when the new set requeued them.
        let rendered: Vec<_> = rasters(&settled.samples)
            .into_iter()
            .filter(|&(target, generation, _)| {
                generation == 2 && matches!(target, probe::Target::Cell(_))
            })
            .collect();
        let [(probe::Target::Cell(index), 2, false)] = rendered[..] else {
            panic!("{backend}, k = {k}: one cell of generation 2, {rendered:?}");
        };
        let [x0, y0, x1, y1] = cell(&legend("B"), MEDIUM, k, index);
        let changed: Vec<(u32, u32)> = old
            .rgba
            .chunks_exact(4)
            .zip(settled.shot.rgba.chunks_exact(4))
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| (i as u32 % old.width, i as u32 / old.width))
            .collect();
        let outside = changed
            .iter()
            .filter(|&&(x, y)| !(x0..x1).contains(&x) || !(y0..y1).contains(&y))
            .count();
        let diff = against_png(&settled.shot, &legend("B"), MEDIUM, k);
        println!(
            "legend title, {backend}, k = {k}: cell {index} at {:?}, {} px changed, {outside} \
             outside it; against the new PNG max {}/255",
            [x0, y0, x1, y1],
            changed.len(),
            diff.max()
        );
        assert!(!changed.is_empty() && outside == 0, "{backend}, k = {k}");
        assert!(
            diff.max() <= bound(&backend),
            "{backend}: max {}/255",
            diff.max()
        );
    }
}

#[test]
fn cheap_refresh_renders_inline() {
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let plot = Plot::<()>::new(tinted(0xffffff))
        .scale_factor(k)
        .probe(Probe::new(clock, tx));
    let Some(mut harness) = Harness::new(plot, SMALL, k) else {
        return;
    };
    let backend = harness.backend();
    let _ = harness.frame(headless::epoch());
    let samples: Vec<Sample> = rx.try_iter().collect();
    let (generation, images, missing, previews) = last_frame(&samples);
    let model = modelled(&samples, None);
    println!(
        "cheap refresh, {backend}: {images} cells and tiles, model {model:.2} ms with the ring, inline"
    );
    assert_eq!((generation, missing, previews), (1, 0, 0), "{backend}");
    assert!(model <= 8.35, "{backend}: model {model} ms");
    // The tiles of the overscan ring render too, after those in view.
    let rendered = rasters(&samples);
    let view = crate::tiles::areas(&tinted(0xffffff), SMALL, k);
    let shown = rendered
        .iter()
        .filter(|&&(target, ..)| in_view(target, &view))
        .count();
    assert_eq!(shown, images as usize);
    assert!(rendered.len() > shown);
    assert!(
        rendered.iter().all(|&(_, _, inline)| inline),
        "{rendered:?}"
    );
    assert_eq!(harness.wakes(), 0, "{backend}: no worker, no wake");
}

#[test]
fn expensive_refresh_uses_the_worker() {
    let _workers = workers();
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let plot = Plot::<()>::new(scatter(DOTS))
        .scale_factor(k)
        .probe(Probe::new(clock, tx));
    let Some(mut harness) = Harness::new(plot, LARGE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let planned: Vec<Sample> = rx.try_iter().collect();
    let (_, images, missing, _) = last_frame(&planned);
    assert!(missing > 0, "{backend}: the first frame waits for the pool");
    // What the frame renders inline fits half the first frame's budget,
    // in order: the gutters and overlays, cheap beside the dense tiles.
    let ahead = rasters(&planned);
    let inline = modelled(&planned, None);
    assert!(ahead.iter().all(|&(_, _, inline)| inline), "{ahead:?}");
    assert!(inline <= 8.35, "{backend}: {inline} ms inline");
    assert!(harness.wait_for_wake(Duration::from_secs(10)));
    harness.events(&[Event::Waken]);
    assert_eq!(
        harness.redraw_requested(),
        RedrawRequest::NextFrame,
        "{backend}: a shown result redraws"
    );
    // The first drain may have taken every result; else the rest come.
    let t = t0 + Duration::from_millis(16);
    let _ = harness.frame(t);
    let mut samples: Vec<Sample> = rx.try_iter().collect();
    if frames(&samples).last() != Some(&(0, 0)) {
        samples.extend(settle(&mut harness, &rx, t).samples);
    }
    let rendered = rasters(&samples);
    assert!(
        rendered.iter().all(|&(_, _, inline)| !inline),
        "{rendered:?}"
    );
    let view = crate::tiles::areas(&scatter(DOTS), LARGE, k);
    let shown = ahead
        .iter()
        .chain(&rendered)
        .filter(|&&(target, ..)| in_view(target, &view))
        .count();
    assert_eq!(
        shown,
        (images + missing) as usize,
        "{backend}: every cell and tile once"
    );
    samples.extend(planned);
    let model = modelled(&samples, Some(&view));
    // The plot area's tiles reach past it by up to a tile less a pixel:
    // their pixels on top of the area's.
    let past: i64 = view
        .iter()
        .map(|[x0, y0, x1, y1]| {
            let side = |a: i64, b: i64| (b - a + 255) / 256 * 256;
            side(*x0, *x1) * side(*y0, *y1) - (x1 - x0) * (y1 - y0)
        })
        .sum();
    let section = 513.0 + past as f64 / cost::PIXELS_PER_MS;
    println!(
        "expensive refresh, {backend}: {images} cells and tiles inline ({inline:.2} ms), {missing} in view on the pool, model {model:.1} ms ({section:.1} expected)"
    );
    // 6.4e6 px / 5e5 + 200k dots / 400 ≈ 513 ms, and the axes' labels;
    // the tiles' pixels past the area on top.
    assert!(
        (model / section - 1.0).abs() < 0.01,
        "{backend}: model {model} ms"
    );
}

#[test]
fn threads_sets_the_pool_size() {
    // Each size starts a pool of as many threads at its first jobs, the
    // last pool stopped; its threads render single-threaded, so no pool of
    // vello_cpu's own takes their name. Sizes clamp to 1..=64.
    let _workers = workers();
    let before = threads(WORKER);
    let k = 1.0;
    let plot = |n: usize, revision: u64| {
        Plot::<()>::new(tinted(0xffffff))
            .scale_factor(k)
            .render(Render::Worker)
            .threads(n)
            .revision(revision)
    };
    let Some(mut harness) = Harness::new(plot(1, 0), SMALL, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let mut seen = Vec::new();
    for (revision, (n, size)) in [(1, 1), (2, 2), (3, 3), (0, 1), (65, 64)]
        .into_iter()
        .enumerate()
    {
        // A new revision renders every slot again, on the pool.
        harness.rebuild(plot(n, revision as u64));
        let _ = harness.frame(t0 + Duration::from_millis(16) * revision as u32);
        let count = threads(WORKER) - before;
        seen.push((n, count));
        assert_eq!(count, size, "{backend}: threads({n})");
    }
    drop(harness);
    println!("pool sizes, {backend}: (asked, threads) {seen:?}");
}

/// The tiles in view of plot area 0 of `area` (device px, its grid at its
/// corner) among the raster samples of `samples`, in arrival order, each
/// with its square's centre's distance from `at` (device px).
fn nearest(samples: &[Sample], area: [i64; 4], at: (f64, f64)) -> Vec<f64> {
    let side = |a: i64, b: i64| (b - a + 255) / 256;
    let (columns, rows) = (side(area[0], area[2]), side(area[1], area[3]));
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Tile { area: 0, i, j },
                ..
            } if (0..columns).contains(&i64::from(*i)) && (0..rows).contains(&i64::from(*j)) => {
                let x = area[0] as f64 + 256.0 * f64::from(*i) + 128.0;
                let y = area[1] as f64 + 256.0 * f64::from(*j) + 128.0;
                Some((x - at.0).hypot(y - at.1))
            }
            Sample::Raster { .. } | Sample::Stage { .. } | Sample::Frame { .. } => None,
        })
        .collect()
}

#[test]
fn tiles_near_the_cursor_render_first() {
    // A wheel step begins every plot area's grid anew: the pool takes the
    // new tiles in view nearest the cursor first. With one thread they
    // land in that order; with two, each lands at most one place early.
    let _workers = workers();
    let k = 1.0;
    let fig = || crate::tiles::figure([0.0, 100.0]);
    let area = crate::tiles::areas(&fig(), crate::tiles::BIG, k)[0];
    let at = ((area[0] + 60) as f64, (area[1] + 50) as f64);
    for n in [1, 2] {
        let (tx, rx) = mpsc::channel();
        let plot = Plot::<()>::new(fig())
            .scale_factor(k)
            .render(Render::Worker)
            .threads(n)
            .crosshair(false)
            .highlight(false)
            .probe(Probe::new(clock, tx));
        let Some(mut harness) = Harness::new(plot, crate::tiles::BIG, k) else {
            return;
        };
        let backend = harness.backend();
        let t0 = headless::epoch();
        let _ = harness.frame(t0);
        let _ = settle(&mut harness, &rx, t0);
        let _ = rx.try_iter().count();
        let cursor = iced::Point::new(at.0 as f32 / k, at.1 as f32 / k);
        harness.point_at(cursor);
        let _ = harness.events(&[
            Event::Mouse(iced::mouse::Event::CursorMoved { position: cursor }),
            Event::Mouse(iced::mouse::Event::WheelScrolled {
                delta: iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -40.0 },
            }),
        ]);
        let _ = harness.frame(t0 + Duration::from_millis(1_000));
        // Results only: no frame, so the wheel gesture stays open and
        // nothing plans again.
        let mut samples = Vec::new();
        while harness.wait_for_wake(Duration::from_millis(500)) {
            let _ = harness.events(&[Event::Waken]);
            samples.extend(rx.try_iter());
        }
        let order = nearest(&samples, area, at);
        let mut sorted = order.clone();
        sorted.sort_by(f64::total_cmp);
        let ranks: Vec<usize> = order
            .iter()
            .map(|d| {
                sorted
                    .iter()
                    .position(|s| s == d)
                    .expect("a distance of the set")
            })
            .collect();
        println!(
            "zoom at {at:?}, {backend}, {n} thread(s): tiles in view by rank of distance, in arrival order {ranks:?}"
        );
        assert!(order.len() >= 6, "{backend}: {} tiles in view", order.len());
        for (place, &rank) in ranks.iter().enumerate() {
            assert!(
                rank < place + n,
                "{backend}, {n} thread(s): rank {rank} landed {place}th, {ranks:?}"
            );
        }
    }
}

#[test]
fn large_marker_sets_splat_on_screen() {
    // A markers command of 10,000 points or more renders from stamps on
    // screen (within D31.2's 32/255 of the PNG, which fills every dot);
    // one point fewer fills them as the PNG does.
    let k = 1.0;
    for (n, bound_of) in [(9_999, None), (10_000, Some(32))] {
        let plot = Plot::<()>::new(scatter(n))
            .scale_factor(k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false);
        let Some(mut harness) = Harness::new(plot, MEDIUM, k) else {
            return;
        };
        let backend = harness.backend();
        let shot = harness.frame(headless::epoch());
        let diff = against_png(&shot, &scatter(n), MEDIUM, k);
        println!(
            "{n} dots, {backend}: against the PNG max {}/255, {} channels over 16",
            diff.max(),
            diff.over()
        );
        let limit = bound_of.unwrap_or_else(|| bound(&backend));
        assert!(
            diff.max() <= limit,
            "{backend}, {n} dots: max {}/255",
            diff.max()
        );
        if bound_of.is_some() {
            assert!(
                diff.max() > bound(&backend),
                "{backend}: splatted dots differ a little"
            );
        }
    }
}
