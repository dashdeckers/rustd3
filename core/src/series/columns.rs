//! Column boundaries in samples, shared by every channel of a recording.

use std::ops::Range;

use crate::series::Id;

/// Column boundaries as sample indices, built by
/// [`Recording::columns`](crate::series::Recording::columns) for one
/// revision of one recording: column k holds samples
/// `edges[k]..edges[k + 1]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Columns {
    recording: Id,
    revision: u64,
    edges: Vec<usize>,
}

impl Columns {
    /// Columns of `recording` at `revision`; `edges` holds at least two
    /// non-decreasing sample indices.
    pub(crate) fn new(recording: Id, revision: u64, edges: Vec<usize>) -> Self {
        Self {
            recording,
            revision,
            edges,
        }
    }

    /// The number of columns, at least one.
    #[expect(
        clippy::len_without_is_empty,
        reason = "a grid has at least one column"
    )]
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len() - 1
    }

    /// The sample index of each edge, non-decreasing.
    #[must_use]
    pub fn get_edges(&self) -> &[usize] {
        &self.edges
    }

    /// The samples the columns cover: the first edge to the last.
    #[must_use]
    pub fn samples(&self) -> Range<usize> {
        self.edges[0]..self.edges[self.edges.len() - 1]
    }

    /// The recording the columns were built for.
    pub(crate) fn recording(&self) -> Id {
        self.recording
    }

    /// That recording's revision when they were built.
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }
}
