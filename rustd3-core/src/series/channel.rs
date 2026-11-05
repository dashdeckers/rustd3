//! One channel of a recording: its samples in chunks, their M4 index and
//! their NaN-free runs.

use std::ops::Range;

use crate::series::chunks::Chunks;
use crate::series::index::Index;
use crate::series::m4::Extremes;
use crate::series::runs::Runs;

/// A channel's index levels as bits and its runs.
#[cfg(test)]
pub(crate) type Layout = (Vec<[Vec<u64>; 4]>, Vec<Range<usize>>);

/// One channel's samples (NaN is a gap), indexed as they arrive.
#[derive(Debug, Clone, Default)]
pub(crate) struct Channel {
    values: Chunks<f64>,
    index: Index,
    runs: Runs,
}

impl Channel {
    /// Sample `i`, if there is one.
    pub(crate) fn get(&self, i: usize) -> Option<f64> {
        self.values.get(i)
    }

    /// The NaN-free runs.
    pub(crate) fn runs(&self) -> &Runs {
        &self.runs
    }

    /// The earliest minimum and maximum of `samples` (within the channel)
    /// among those that pass `valid`, from the index.
    pub(crate) fn extremes(
        &self,
        samples: Range<usize>,
        valid: &impl Fn(f64) -> bool,
    ) -> Option<Extremes> {
        self.index.extremes(&self.values, samples, valid)
    }

    /// Appends `values`, pushing the index nodes and runs they complete.
    pub(crate) fn extend(&mut self, values: &[f64]) {
        let first = self.values.len();
        self.values.extend(values);
        self.index.extend(&self.values);
        self.runs.extend(first, values);
    }

    /// The index's levels as bits and the runs, for exact comparisons.
    #[cfg(test)]
    pub(crate) fn layout(&self) -> Layout {
        (
            self.index.bits(),
            self.runs.overlapping(0..usize::MAX).to_vec(),
        )
    }

    /// Removes the first `chunks` chunks and rebuilds the index and the
    /// runs from the retained samples, through the path appends take.
    pub(crate) fn drop_front(&mut self, chunks: usize) {
        let mut values = std::mem::take(&mut self.values);
        values.drop_front(chunks);
        let mut channel = Self {
            values,
            ..Self::default()
        };
        channel.index.extend(&channel.values);
        for (first, piece) in channel.values.pieces(0..channel.values.len()) {
            channel.runs.extend(first, piece);
        }
        *self = channel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::CHUNK;

    fn channel(values: &[f64]) -> Channel {
        let mut channel = Channel::default();
        channel.extend(values);
        channel
    }

    #[test]
    fn a_channel_after_drop_front_equals_a_fresh_one_of_the_retained_samples() {
        let n = 5 * CHUNK + 40_000;
        let values: Vec<f64> = (0..n)
            .map(|i| match i % 9_973 {
                0..=40 => f64::NAN,
                k => ((i * 7_919) % 1_009) as f64 - k as f64 * 1e-3,
            })
            .collect();
        for chunks in [1, 2, 5] {
            let mut dropped = channel(&values[..CHUNK + 3]);
            dropped.extend(&values[CHUNK + 3..]);
            dropped.drop_front(chunks);
            let fresh = channel(&values[chunks * CHUNK..]);
            assert_eq!(dropped.values.len(), n - chunks * CHUNK);
            assert_eq!(dropped.index.bits(), fresh.index.bits(), "{chunks} chunks");
            assert_eq!(
                dropped.runs.overlapping(0..n),
                fresh.runs.overlapping(0..n),
                "{chunks} chunks"
            );
            let first = 9_973 - chunks * CHUNK % 9_973;
            assert_eq!(dropped.runs.overlapping(0..n)[0], 0..first);
        }
    }
}
