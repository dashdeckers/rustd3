//! The widget's render pipeline through the headless harness on the
//! backend `ICED_TEST_BACKEND` names: cells rendered on the worker against
//! the same cells rendered inline, and the worker thread's lifetime.

use std::sync::{Mutex, MutexGuard, PoisonError, mpsc};
use std::time::Duration;

use iced::{Event, Size};
use rustd3_core::scale;
use rustd3_figure::mark::dots;
use rustd3_figure::{Figure, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::Sample;
use rustd3_iced::plot::{Probe, Render};
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};

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

/// Frames of `harness` after each wake, the `Waken` injected before
/// each and the frames `t` + 16 ms apart, until a frame shows every slot
/// current: that frame, the number of wakes it took and the time from
/// `start` to the first wake. Panics when a wake takes longer than ten
/// seconds.
fn settle<Message: 'static>(
    harness: &mut Harness<'_, Message>,
    probe: &mpsc::Receiver<Sample>,
    mut t: iced::time::Instant,
    start: iced::time::Instant,
) -> (Shot, usize, Duration) {
    let mut first = None;
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
        if frames(&samples).last() == Some(&(0, 0)) {
            return (shot, wakes, first);
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
    let start = headless::clock();
    let _ = harness.frame(t0);
    let first: Vec<Sample> = rx.try_iter().collect();
    let (missing, _) = *frames(&first).last().expect("a frame sample");
    assert!(
        missing > 0,
        "{backend}: the first frame waits for the worker"
    );
    let (worker, wakes, first_wake) = settle(&mut harness, &rx, t0, start);
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
