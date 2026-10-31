//! M4 thinning: per column, the first, minimum, maximum and last sample of
//! each NaN-free run, from the index.

use std::ops::Range;

/// Indices of the minimum and maximum of a sample range, with values; the
/// earliest sample wins ties.
///
/// ```
/// use rustd3_core::series::{Clock, Recording};
///
/// let rec = Recording::new(Clock::Regular { start: 0.0, period: 1.0 }, [[3.0, 1.0, f64::NAN, 5.0, 1.0]])?;
/// let channel = rec.series(0).expect("one channel");
/// let e = channel.extremes(0..5, |_| true).expect("finite values");
/// assert_eq!((e.min(), e.max()), ((1, 1.0), (3, 5.0)));
/// # Ok::<(), rustd3_core::series::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extremes {
    min: (usize, f64),
    max: (usize, f64),
}

/// Thinned polylines: sample indices, one slice per NaN-free run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lines {
    indices: Vec<usize>,
    ends: Vec<usize>,
}

impl Extremes {
    pub(crate) fn new(min: (usize, f64), max: (usize, f64)) -> Self {
        Self { min, max }
    }

    /// The minimum's sample index and value.
    #[must_use]
    pub fn min(&self) -> (usize, f64) {
        self.min
    }

    /// The maximum's sample index and value.
    #[must_use]
    pub fn max(&self) -> (usize, f64) {
        self.max
    }
}

impl Lines {
    /// M4 over column `edges` (sample indices e₀ … e_c, non-decreasing) for
    /// the `runs` that overlap the view [e₀, e_c). Each run yields one
    /// polyline: the sample before the view when the run holds it, then per
    /// column the sorted, deduplicated first and last of the run's samples
    /// in it and the minimum and maximum of each channel `extremes` gives
    /// (one channel, or a joint query's two; `None` when no sample
    /// qualifies), then the sample after the view when the run holds it. A
    /// gap is never crossed.
    pub(crate) fn thin(
        runs: &[Range<usize>],
        edges: &[usize],
        mut extremes: impl FnMut(Range<usize>) -> [Option<Extremes>; 2],
    ) -> Self {
        let mut lines = Self {
            indices: Vec::new(),
            ends: Vec::new(),
        };
        let (Some(&start), Some(&end)) = (edges.first(), edges.last()) else {
            return lines;
        };
        for run in runs {
            if run.start < start {
                lines.indices.push(start - 1);
            }
            let (from, to) = (run.start.max(start), run.end.min(end));
            let first = edges[1..].partition_point(|&edge| edge <= from);
            for column in edges[first..]
                .windows(2)
                .take_while(|column| column[0] < to)
            {
                let (a, b) = (column[0].max(run.start), column[1].min(run.end));
                if a >= b {
                    continue;
                }
                let mut picks = [a, b - 1, a, a, a, a];
                for (pair, found) in picks[2..]
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip(extremes(a..b))
                {
                    if let Some(found) = found {
                        *pair = [found.min.0, found.max.0];
                    }
                }
                picks.sort_unstable();
                for pick in picks {
                    if lines.indices.last() != Some(&pick) {
                        lines.indices.push(pick);
                    }
                }
            }
            if run.end > end {
                lines.indices.push(end);
            }
            lines.ends.push(lines.indices.len());
        }
        lines
    }

    /// The number of polylines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// Whether there are no polylines.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// The number of points across all polylines.
    #[must_use]
    pub fn points(&self) -> usize {
        self.indices.len()
    }

    /// Each polyline's sample indices, increasing.
    pub fn iter(&self) -> impl Iterator<Item = &[usize]> + '_ {
        let mut start = 0;
        self.ends.iter().map(move |&end| {
            let line = &self.indices[start..end];
            start = end;
            line
        })
    }
}
