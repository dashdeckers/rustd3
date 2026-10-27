//! [`Frame`]: what one draw thins with.

use std::collections::HashMap;
use std::fmt;

use rustd3_core::series::{self, Columns, Coordinate, Series, m4};

use crate::thin::Memo;
use crate::thin::memo::Key;

/// Values every validity test is tried on for its fingerprint: the edges
/// of the sets the figure's scales admit (all finite values, positive or
/// negative ones for log scales, the open unit interval for logit).
const PROBES: [f64; 18] = [
    f64::NEG_INFINITY,
    -1e300,
    -1e10,
    -1.0,
    -0.5,
    -1e-10,
    -1e-300,
    -0.0,
    0.0,
    1e-300,
    1e-10,
    0.5,
    1.0,
    2.0,
    1e10,
    1e300,
    f64::INFINITY,
    f64::NAN,
];

/// What one draw thins with: the column boundaries computed so far, which
/// every line on one clock shares, and the figure cache's M4 memo, which
/// outlives the draw. [`Indexed::thin`](crate::scale::Indexed::thin) takes
/// it.
pub struct Frame {
    memo: Memo,
    columns: HashMap<(series::Id, u64, u64), Columns>,
    hits: u64,
}

/// `Frame { columns: 1, hits: 2, memo: … }`.
impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("columns", &self.columns.len())
            .field("hits", &self.hits)
            .field("memo", &self.memo)
            .finish()
    }
}

impl Frame {
    /// A draw's frame over `memo`.
    pub(crate) fn new(memo: Memo) -> Frame {
        Frame {
            memo,
            columns: HashMap::new(),
            hits: 0,
        }
    }

    /// The memo, to keep for the next draw.
    pub(crate) fn into_memo(self) -> Memo {
        self.memo
    }

    /// How often column boundaries were found computed in this draw.
    #[cfg(test)]
    pub(crate) fn hits(&self) -> u64 {
        self.hits
    }

    /// M4 of `series` over the columns between consecutive `edges`
    /// (increasing), whose bits hash to `grid`, keeping the samples that
    /// pass `valid`: from the memo when this recording, revision, channel,
    /// grid and test were thinned before, else from the index over column
    /// boundaries computed once per draw for the recording, its revision
    /// and the grid. `None` when `edges` are fewer than two, out of order or
    /// not valid coordinates.
    pub(crate) fn lines<X: Coordinate>(
        &mut self,
        series: Series<'_, X>,
        edges: &[X],
        grid: u64,
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<m4::Lines> {
        let recording = series.recording();
        let key = Key {
            recording: recording.id(),
            revision: recording.revision(),
            channel: series.channel(),
            grid,
            valid: fingerprint(valid),
        };
        if let Some(lines) = self.memo.get(&key) {
            return Some(lines.clone());
        }
        let columns = match self.columns.entry((key.recording, key.revision, grid)) {
            std::collections::hash_map::Entry::Occupied(found) => {
                self.hits += 1;
                found.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(recording.columns(edges).ok()?)
            }
        };
        let lines = series
            .m4(columns, valid)
            .expect("invariant: columns built for this series' recording and revision");
        self.memo.insert(key, lines.clone());
        Some(lines)
    }
}

/// Which of the probe values `valid` admits, as bits.
fn fingerprint(valid: &dyn Fn(f64) -> bool) -> u64 {
    PROBES
        .iter()
        .enumerate()
        .filter(|(_, v)| valid(**v))
        .fold(0, |bits, (i, _)| bits | 1 << i)
}

#[cfg(test)]
mod tests {
    use rustd3_core::series::{Clock, Recording};

    use super::*;

    /// 10,000 samples of a ramp on two channels, the second negated.
    fn ramp() -> Recording<f64> {
        let a: Vec<f64> = (0..10_000).map(f64::from).collect();
        let b: Vec<f64> = a.iter().map(|v| -v).collect();
        Recording::new(
            Clock::Regular {
                start: 0.0,
                period: 1.0,
            },
            [a, b],
        )
        .expect("a valid recording")
    }

    /// 101 edges over the recording, and their grid key.
    fn edges() -> (Vec<f64>, u64) {
        let edges: Vec<f64> = (0..=100).map(|i| f64::from(i) * 100.0).collect();
        let grid = crate::thin::grid(edges.iter().map(|e| e.to_bits()));
        (edges, grid)
    }

    #[test]
    fn two_channels_of_one_recording_compute_column_boundaries_once() {
        let rec = ramp();
        let (edges, grid) = edges();
        let mut frame = Frame::new(Memo::default());
        let a = frame.lines(rec.series(0).expect("a"), &edges, grid, &|_| true);
        let b = frame.lines(rec.series(1).expect("b"), &edges, grid, &|_| true);
        assert_eq!(frame.hits(), 1);
        assert_eq!(frame.into_memo().queries(), 2);
        let columns = rec.columns(&edges).expect("valid edges");
        let core = |c| rec.series(c).expect("channel").m4(&columns, |_| true).ok();
        assert_eq!((a, b), (core(0), core(1)));
    }

    #[test]
    fn the_memo_answers_an_unchanged_view_and_misses_after_an_append() {
        let mut rec = ramp();
        let (edges, grid) = edges();
        let mut memo = Memo::default();
        for _ in 0..2 {
            memo.begin();
            let mut frame = Frame::new(memo);
            frame.lines(rec.series(0).expect("a"), &edges, grid, &|_| true);
            memo = frame.into_memo();
        }
        assert_eq!(memo.queries(), 1);
        rec.extend(&[], [[1.0], [2.0]]).expect("an append");
        let mut frame = Frame::new(memo);
        frame.lines(rec.series(0).expect("a"), &edges, grid, &|_| true);
        assert_eq!(frame.into_memo().queries(), 2);
    }

    #[test]
    fn lines_under_another_validity_test_are_thinned_again() {
        let y: Vec<f64> = (0..10_000).map(|i| (f64::from(i) * 0.7).sin()).collect();
        let clock = Clock::Regular {
            start: 0.0,
            period: 1.0,
        };
        let rec = Recording::new(clock, [y]).expect("a valid recording");
        let (edges, grid) = edges();
        let mut frame = Frame::new(Memo::default());
        let all = frame.lines(rec.series(0).expect("a"), &edges, grid, &|_| true);
        let positive = frame.lines(rec.series(0).expect("a"), &edges, grid, &|v| v > 0.0);
        assert_ne!(all, positive);
        assert_eq!(frame.into_memo().queries(), 2);
    }

    #[test]
    fn memo_entries_unused_for_32_draws_are_dropped() {
        let rec = ramp();
        let (edges, grid) = edges();
        let mut memo = Memo::default();
        memo.begin();
        let mut frame = Frame::new(memo);
        frame.lines(rec.series(0).expect("a"), &edges, grid, &|_| true);
        memo = frame.into_memo();
        for _ in 0..32 {
            memo.begin();
        }
        assert_eq!(
            format!("{memo:?}"),
            "Memo { entries: 1, generation: 33, queries: 1 }"
        );
        memo.begin();
        assert_eq!(
            format!("{memo:?}"),
            "Memo { entries: 0, generation: 34, queries: 1 }"
        );
    }

    #[test]
    fn invalid_edges_thin_nothing() {
        let rec = ramp();
        let mut frame = Frame::new(Memo::default());
        let series = rec.series(0).expect("a");
        assert_eq!(frame.lines(series, &[2.0, 1.0], 7, &|_| true), None);
        assert_eq!(frame.lines(series, &[f64::NAN, 1.0], 8, &|_| true), None);
    }

    #[test]
    fn fingerprints_tell_the_figure_scales_admission_tests_apart() {
        let tests: [&dyn Fn(f64) -> bool; 4] = [
            &|v| v.is_finite(),
            &|v| v.is_finite() && v > 0.0,
            &|v| v.is_finite() && v < 0.0,
            &|v| v > 0.0 && v < 1.0,
        ];
        let prints: Vec<u64> = tests.iter().map(|t| fingerprint(*t)).collect();
        for (i, a) in prints.iter().enumerate() {
            for b in &prints[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
