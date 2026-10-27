//! Samples in fixed-capacity chunks, so appends never move them.

use std::fmt;
use std::ops::Range;

use crate::series::CHUNK;

/// Samples in inner vectors allocated once at capacity [`CHUNK`]: sample i
/// is `chunks[i / CHUNK][i % CHUNK]`, every chunk but the last is full, and
/// none is empty.
pub(crate) struct Chunks<T> {
    chunks: Vec<Vec<T>>,
}

impl<T> Default for Chunks<T> {
    fn default() -> Self {
        Self { chunks: Vec::new() }
    }
}

/// Clones each chunk at full capacity, so the clone's appends never move
/// samples either.
impl<T: Copy> Clone for Chunks<T> {
    fn clone(&self) -> Self {
        let chunks = self.chunks.iter().map(|chunk| full(chunk)).collect();
        Self { chunks }
    }
}

/// A summary: a recording's Debug output does not list millions of samples.
impl<T> fmt::Debug for Chunks<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chunks")
            .field("len", &self.len())
            .field("chunks", &self.chunks.len())
            .finish()
    }
}

impl<T> Chunks<T> {
    /// The number of samples.
    pub(crate) fn len(&self) -> usize {
        self.chunks
            .last()
            .map_or(0, |last| (self.chunks.len() - 1) * CHUNK + last.len())
    }

    /// Removes the first `chunks` chunks (all of them if there are fewer).
    pub(crate) fn drop_front(&mut self, chunks: usize) {
        self.chunks.drain(..chunks.min(self.chunks.len()));
    }
}

impl<T: Copy> Chunks<T> {
    /// Sample `i`, if there is one.
    pub(crate) fn get(&self, i: usize) -> Option<T> {
        self.chunks.get(i / CHUNK)?.get(i % CHUNK).copied()
    }

    /// `slice::partition_point` over all samples: the first index whose
    /// sample fails `pred`, which holds on a prefix. Binary-searches the
    /// chunks by their last samples, then one chunk.
    pub(crate) fn partition_point(&self, mut pred: impl FnMut(&T) -> bool) -> usize {
        let full = self
            .chunks
            .partition_point(|chunk| chunk.last().is_some_and(&mut pred));
        self.chunks.get(full).map_or(self.len(), |chunk| {
            full * CHUNK + chunk.partition_point(pred)
        })
    }

    /// The samples of `range`, which must lie in one chunk (as every block
    /// does), as one slice.
    pub(crate) fn slice(&self, range: Range<usize>) -> &[T] {
        let start = range.start % CHUNK;
        &self.chunks[range.start / CHUNK][start..start + range.len()]
    }

    /// The samples of `range` (within `len`) as contiguous pieces, each
    /// with the index of its first sample.
    pub(crate) fn pieces(&self, range: Range<usize>) -> impl Iterator<Item = (usize, &[T])> {
        let mut at = range.start;
        std::iter::from_fn(move || {
            (at < range.end).then(|| {
                let first = at;
                let start = first % CHUNK;
                let n = (range.end - first).min(CHUNK - start);
                at += n;
                (first, &self.chunks[first / CHUNK][start..start + n])
            })
        })
    }

    /// Appends `values`, filling the last chunk before starting a new one.
    pub(crate) fn extend(&mut self, mut values: &[T]) {
        while !values.is_empty() {
            if self.chunks.last().is_none_or(|last| last.len() == CHUNK) {
                self.chunks.push(Vec::with_capacity(CHUNK));
            }
            let last = self.chunks.len() - 1;
            let chunk = &mut self.chunks[last];
            let (now, rest) = values.split_at((CHUNK - chunk.len()).min(values.len()));
            chunk.extend_from_slice(now);
            values = rest;
        }
    }
}

/// A copy of `chunk` with room for a whole chunk.
fn full<T: Copy>(chunk: &[T]) -> Vec<T> {
    let mut copy = Vec::with_capacity(CHUNK);
    copy.extend_from_slice(chunk);
    copy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(n: usize) -> Chunks<usize> {
        let mut chunks = Chunks::default();
        chunks.extend(&(0..n).collect::<Vec<_>>());
        chunks
    }

    #[test]
    fn appends_fill_the_last_chunk_before_starting_another() {
        let mut chunks = filled(CHUNK - 1);
        chunks.extend(&[7, 8, 9]);
        assert_eq!(chunks.len(), CHUNK + 2);
        assert_eq!(chunks.chunks.len(), 2);
        assert_eq!(chunks.chunks[0].len(), CHUNK);
        assert_eq!(chunks.get(CHUNK - 1), Some(7));
        assert_eq!(chunks.get(CHUNK + 1), Some(9));
        assert_eq!(chunks.get(CHUNK + 2), None);
    }

    #[test]
    fn chunks_and_their_clones_keep_a_whole_chunk_of_capacity() {
        let chunks = filled(CHUNK + 10);
        let clone = chunks.clone();
        for chunk in chunks.chunks.iter().chain(&clone.chunks) {
            assert_eq!(chunk.capacity(), CHUNK);
        }
        assert_eq!(clone.get(CHUNK + 9), Some(CHUNK + 9));
    }

    #[test]
    fn pieces_split_at_chunk_boundaries() {
        let chunks = filled(2 * CHUNK + 5);
        let pieces: Vec<_> = chunks
            .pieces(CHUNK - 2..2 * CHUNK + 3)
            .map(|(first, piece)| (first, piece.len(), piece[0]))
            .collect();
        assert_eq!(
            pieces,
            [
                (CHUNK - 2, 2, CHUNK - 2),
                (CHUNK, CHUNK, CHUNK),
                (2 * CHUNK, 3, 2 * CHUNK)
            ]
        );
        assert_eq!(chunks.pieces(5..5).count(), 0);
        assert_eq!(chunks.slice(CHUNK + 64..CHUNK + 128)[0], CHUNK + 64);
    }

    #[test]
    fn dropping_removes_whole_chunks_from_the_front() {
        let mut chunks = filled(2 * CHUNK + 5);
        chunks.drop_front(2);
        assert_eq!(chunks.len(), 5);
        assert_eq!(chunks.get(0), Some(2 * CHUNK));
        chunks.drop_front(9);
        assert_eq!(chunks.len(), 0);
    }
}
