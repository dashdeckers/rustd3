//! The widget's render pipeline through the headless harness on the
//! backend `ICED_TEST_BACKEND` names: cells rendered on the worker against
//! the same cells rendered inline, the newest result winning, previous
//! images staying until their replacement lands, and the worker thread's
//! lifetime.

use std::sync::{Mutex, MutexGuard, PoisonError, mpsc};
use std::time::Duration;

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
use rustd3_iced::plot::{Probe, Render};
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

/// The expensive scatter's dots. The cost model predicts 12.8 + 5 ms at
/// 2, over half of any frame budget; a debug build renders its plot area
/// in about 2.5 s, where 200k dots take a minute (D19.1).
const DOTS: usize = 2_000;

/// The cheap figures' logical size.
const MEDIUM: Size = Size::new(480.0, 320.0);

/// The lock [`WORKERS`] holds; a test that failed holding it poisons
/// nothing the others read.
fn workers() -> MutexGuard<'static, ()> {
    WORKERS.lock().unwrap_or_else(PoisonError::into_inner)
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

/// The raster samples among `samples`: (target cell, generation, inline).
fn rasters(samples: &[Sample]) -> Vec<(u32, u64, bool)> {
    samples
        .iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Cell(cell),
                generation,
                inline,
                ..
            } => Some((*cell, *generation, *inline)),
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
    mut t: iced::time::Instant,
) -> Settled {
    let start = headless::clock();
    let mut first = None;
    let mut received = Vec::new();
    for wakes in 1..=1000 {
        assert!(
            harness.wait_for_wake(Duration::from_secs(10)),
            "no wake within ten seconds"
        );
        let first = *first.get_or_insert_with(|| headless::clock().duration_since(start));
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
        .probe(Probe::new(headless::clock, tx));
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
    let before = headless::threads(WORKER);
    let plot = Plot::<()>::new(scatter(DOTS))
        .scale_factor(2.0)
        .render(Render::Worker);
    let Some(mut harness) = Harness::new(plot, LARGE, 2.0) else {
        return;
    };
    assert_eq!(headless::threads(WORKER), before, "no job, no thread");
    let _ = harness.frame(headless::epoch());
    // The worker, and vello_cpu's pool threads, which take its name.
    let during = headless::threads(WORKER);
    assert!(during > before, "{during} threads, {before} before");
    // The job set is pending: the drop closes the queue and joins the
    // thread after the cell in flight; the pool's threads end once the
    // joined thread dropped its renderer.
    drop(harness);
    let after = (0..10_000)
        .map(|_| {
            let count = headless::threads(WORKER);
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
    let probe = Probe::new(headless::clock, tx);
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
    let newest = rendered.iter().filter(|(_, g, _)| *g == 2).count();
    assert_eq!(
        newest, images as usize,
        "{backend}: every cell of generation 2"
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
    let probe = Probe::new(headless::clock, tx);
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
        let probe = Probe::new(headless::clock, tx);
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
        let rendered = rasters(&settled.samples);
        let [(index, 2, false)] = rendered[..] else {
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
