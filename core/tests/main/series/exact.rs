//! M4 from the index against a raw scan of the same samples, over the
//! prototypes' traces (`proto/thin2`): a vibration-like signal and a slow
//! signal with gaps of 1 to 50,000 samples, 1,000,000 samples each.

use std::sync::OnceLock;

use rustd3_core::series::{Clock, Recording};

use super::{Lcg, m4_scan, thinned_by_index};

/// The traces' sample count.
const N: usize = 1_000_000;

/// A trace as the prototype built it, and its recording on an explicit
/// clock of the same x.
struct Trace {
    x: Vec<f64>,
    y: Vec<f64>,
    rec: Recording<f64>,
    /// The one-sample impacts, sorted (the signal trace only).
    impacts: Vec<usize>,
}

impl Trace {
    fn new(x: Vec<f64>, y: Vec<f64>, impacts: Vec<usize>) -> Self {
        let rec = Recording::new(Clock::Explicit(x.clone()), [&y]).expect("a valid trace");
        Self { x, y, rec, impacts }
    }

    /// The view from the first sample to just past the last.
    fn full(&self) -> [f64; 2] {
        [self.x[0], self.x[N - 1] + 1e-4]
    }

    /// M4 by the index and by the raw scan over `view` in `columns`
    /// columns: column edges in samples and polylines, each way.
    fn both(&self, view: [f64; 2], columns: usize, valid: impl Fn(f64) -> bool) -> [Thinned; 2] {
        let edges = edges(view, columns);
        let index = thinned_by_index(&self.rec, &edges, &valid);
        [index, m4_scan(&self.x, &self.y, &edges, &valid)]
    }

    /// The index's polylines, asserted equal to the raw scan's.
    fn agree(
        &self,
        view: [f64; 2],
        columns: usize,
        valid: impl Fn(f64) -> bool,
    ) -> Vec<Vec<usize>> {
        let [index, scan] = self.both(view, columns, valid);
        assert_eq!(index, scan, "view {view:?}, {columns} columns");
        index.1
    }

    /// How many of `views` the index and the raw scan agree on.
    fn same(&self, views: &[[f64; 2]], columns: usize) -> usize {
        let same = views.iter().filter(|view| {
            let [index, scan] = self.both(**view, columns, |_| true);
            index == scan
        });
        same.count()
    }
}

/// Column edges in samples and polylines.
type Thinned = (Vec<usize>, Vec<Vec<usize>>);

/// The prototype's data-space column edges: `columns + 1` of them.
fn edges(view: [f64; 2], columns: usize) -> Vec<f64> {
    (0..=columns)
        .map(|c| view[0] + (view[1] - view[0]) * c as f64 / columns as f64)
        .collect()
}

/// `proto/thin2/src/main.rs`, `signal(n)`: drift, a 7 Hz tone, noise,
/// 0.2 s bursts every 10 s and 50 one-sample impacts at 10 kHz, seed 42.
fn signal() -> &'static Trace {
    static TRACE: OnceLock<Trace> = OnceLock::new();
    TRACE.get_or_init(|| {
        let mut u = Lcg(42);
        let tau = std::f64::consts::TAU;
        let x: Vec<f64> = (0..N).map(|i| i as f64 / 10_000.0).collect();
        let mut y: Vec<f64> = x
            .iter()
            .map(|t| {
                let burst = if t % 10.0 < 0.2 {
                    0.8 * (tau * 120.0 * t).sin()
                } else {
                    0.0
                };
                0.3 * (tau * 0.05 * t).sin()
                    + 0.4 * (tau * 7.0 * t).sin()
                    + 0.25 * (u.next() - 0.5)
                    + burst
            })
            .collect();
        let mut impacts: Vec<usize> = (0..50).map(|_| (u.next() * N as f64) as usize).collect();
        impacts.sort_unstable();
        for (k, &i) in impacts.iter().enumerate() {
            let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
            y[i] = sign * (1.4 + 0.6 * u.next());
        }
        Trace::new(x, y, impacts)
    })
}

/// `proto/thin2/src/bin/gaps.rs`: a 0.1 Hz tone with noise, seed 9, with
/// four gaps each of 1, 10, 100, 1,000, 10,000 and 50,000 samples (some
/// overlap), the first sample a gap and the last NaN.
fn gaps() -> &'static Trace {
    static TRACE: OnceLock<Trace> = OnceLock::new();
    TRACE.get_or_init(|| {
        let mut u = Lcg(9);
        let x: Vec<f64> = (0..N).map(|i| i as f64 / 10_000.0).collect();
        let mut y: Vec<f64> = x
            .iter()
            .map(|t| 0.6 * (std::f64::consts::TAU * 0.1 * t).sin() + 0.3 * (u.next() - 0.5))
            .collect();
        for len in [1, 10, 100, 1_000, 10_000, 50_000] {
            for _ in 0..4 {
                let start = (u.next() * (N - len) as f64) as usize;
                y[start..start + len].fill(f64::NAN);
            }
        }
        y[0] = f64::NAN;
        y[N - 1] = f64::NAN;
        Trace::new(x, y, Vec::new())
    })
}

#[test]
fn m4_from_the_index_equals_a_raw_scan_in_the_signal_traces_51_views() {
    let trace = signal();
    let mut views = vec![trace.full()];
    for k in 0..50 {
        let a = trace.x[k * N / 60];
        let b = trace.x[(k * N / 60 + 1 + k * 997 * (N / 1_000_000)).min(N - 1)];
        views.push([a, b.max(a + 2e-4)]);
    }
    let same = trace.same(&views, 1000);
    println!("{same}/51 views");
    assert_eq!(same, 51);
}

#[test]
fn the_full_view_keeps_every_impact_at_one_two_and_four_columns_per_pixel() {
    let trace = signal();
    for (columns, points) in [(1000, 3_949), (2000, 7_791), (4000, 15_429)] {
        let lines = trace.agree(trace.full(), columns, |_| true);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].len(), points, "{columns} columns");
        let kept = trace
            .impacts
            .iter()
            .filter(|i| lines[0].binary_search(i).is_ok())
            .count();
        assert_eq!(kept, 50, "{columns} columns");
    }
}

#[test]
fn a_view_of_2000_samples_adds_the_two_outer_neighbours() {
    let trace = signal();
    let view = [trace.x[N / 2], trace.x[N / 2 + 2000]];
    let lines = trace.agree(view, 1000, |_| true);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].len(), 1_981);
    assert_eq!((lines[0][0], lines[0][1_980]), (N / 2 - 1, N / 2 + 2000));
}

#[test]
fn m4_from_the_index_equals_a_raw_scan_in_the_gap_traces_41_views() {
    let trace = gaps();
    let mut views = vec![trace.full()];
    for k in 0..40 {
        let a = trace.x[k * N / 41];
        views.push([a, a + 0.05 + 2.0 * k as f64]);
    }
    let same = trace.same(&views, 1000);
    println!("{same}/41 views");
    assert_eq!(same, 41);
    for view in &views {
        let [(_, lines), _] = trace.both(*view, 1000, |_| true);
        for line in &lines {
            assert!(
                line.iter().all(|&i| !trace.y[i].is_nan()),
                "a NaN point in {view:?}"
            );
            for pair in line.windows(2) {
                let crossed = trace.y[pair[0]..=pair[1]].iter().any(|v| v.is_nan());
                assert!(!crossed, "a segment across a gap in {view:?}");
            }
        }
    }
    let series = trace.rec.series(0).expect("channel 0");
    assert_eq!(series.runs(0..N).count(), 19);
    let lines = trace.agree(trace.full(), 1000, |_| true);
    assert_eq!(
        (lines.len(), lines.iter().map(Vec::len).sum::<usize>()),
        (19, 3_124)
    );
}

#[test]
fn the_predicate_agrees_with_a_raw_scan() {
    for trace in [signal(), gaps()] {
        let full = trace.full();
        for view in [
            full,
            [full[0] + 20.0, full[0] + 20.5],
            [full[0] + 31.3, full[0] + 80.1],
        ] {
            for columns in [1, 7, 1000] {
                trace.agree(view, columns, |y| y > 0.0);
                trace.agree(view, columns, |y| y.abs() < 0.3);
            }
        }
    }
}

#[test]
fn columns_across_chunk_boundaries_agree_with_a_raw_scan() {
    for trace in [signal(), gaps()] {
        for chunk in [1, 2, 7, 15] {
            let boundary = chunk * 65_536;
            for (before, after, columns) in [
                (6, 70, 7),
                (64, 64, 2),
                (1, 1, 1),
                (3_000, 200, 50),
                (130, 1, 3),
            ] {
                let view = [trace.x[boundary - before], trace.x[boundary + after]];
                trace.agree(view, columns, |_| true);
                trace.agree(view, columns, |y| y < 0.1);
            }
        }
    }
}
