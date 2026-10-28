//! One channel's NaN-free runs.

use std::fmt;
use std::ops::Range;

/// The sorted, disjoint, half-open ranges of a channel's samples that hold
/// no NaN, each as long as it can be.
#[derive(Clone, Default)]
pub(crate) struct Runs {
    ranges: Vec<Range<usize>>,
}

/// A summary: the number of runs rather than every range.
impl fmt::Debug for Runs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Runs")
            .field("len", &self.ranges.len())
            .finish()
    }
}

impl Runs {
    /// Adds `values`, samples `first..first + values.len()`, to the runs of
    /// the samples before `first`: the last run grows when it ended at
    /// `first` and the first new sample is not NaN.
    pub(crate) fn extend(&mut self, first: usize, values: &[f64]) {
        let mut open = self
            .ranges
            .pop_if(|run| run.end == first)
            .map(|run| run.start);
        for (i, value) in (first..).zip(values) {
            match (value.is_nan(), open) {
                (false, None) => open = Some(i),
                (true, Some(start)) => {
                    self.ranges.push(start..i);
                    open = None;
                }
                (false, Some(_)) | (true, None) => {}
            }
        }
        if let Some(start) = open {
            self.ranges.push(start..first + values.len());
        }
    }

    /// The runs that overlap `samples`, whole: those that end after its
    /// start and start before its end (for an empty `samples`, those that
    /// hold samples on both sides of it).
    pub(crate) fn overlapping(&self, samples: Range<usize>) -> &[Range<usize>] {
        let first = self.ranges.partition_point(|run| run.end <= samples.start);
        let last = self.ranges.partition_point(|run| run.start < samples.end);
        &self.ranges[first..last.max(first)]
    }
}

/// The non-empty intersections of two lists of sorted, disjoint ranges, in
/// order: where samples lie in a range of both.
pub(crate) fn intersect(a: &[Range<usize>], b: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while let (Some(x), Some(y)) = (a.get(i), b.get(j)) {
        let (start, end) = (x.start.max(y.start), x.end.min(y.end));
        if start < end {
            out.push(start..end);
        }
        if x.end <= y.end {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(pieces: &[&[f64]]) -> Vec<Range<usize>> {
        let mut runs = Runs::default();
        let mut first = 0;
        for piece in pieces {
            runs.extend(first, piece);
            first += piece.len();
        }
        runs.ranges
    }

    #[test]
    fn runs_break_at_every_nan() {
        let nan = f64::NAN;
        assert_eq!(runs(&[&[nan, 1.0, 2.0, nan, nan, 3.0]]), [1..3, 5..6]);
        assert_eq!(runs(&[&[nan, nan]]), []);
        assert_eq!(runs(&[&[]]), []);
        assert_eq!(runs(&[&[f64::INFINITY, -0.0, nan, 1.0]]), [0..2, 3..4]);
    }

    #[test]
    fn runs_across_appends_equal_runs_of_the_whole() {
        let nan = f64::NAN;
        let all = [1.0, 2.0, nan, 3.0, 4.0, 5.0, nan, nan, 6.0, 7.0];
        let whole = runs(&[&all]);
        assert_eq!(whole, [0..2, 3..6, 8..10]);
        for a in 0..=all.len() {
            for b in a..=all.len() {
                assert_eq!(
                    runs(&[&all[..a], &all[a..b], &[], &all[b..]]),
                    whole,
                    "{a} {b}"
                );
            }
        }
    }

    #[test]
    fn intersections_keep_what_both_lists_cover() {
        let one = |r: Range<usize>| vec![r];
        assert_eq!(intersect(&one(0..10), &[0..3, 4..10]), [0..3, 4..10]);
        assert_eq!(
            intersect(&[0..2, 3..6, 8..10], &one(1..9)),
            [1..2, 3..6, 8..9]
        );
        assert_eq!(intersect(&one(0..4), &one(4..8)), []);
        assert_eq!(intersect(&one(2..5), &[]), []);
        assert_eq!(intersect(&[0..5, 6..9], &one(4..7)), [4..5, 6..7]);
    }
}
