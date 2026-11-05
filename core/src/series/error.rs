//! What a recording rejects, carrying the rejected value.

use crate::series::{Coordinate, Id};

/// Why a recording, an append or a column grid was rejected.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error<X: Coordinate = f64> {
    /// A channel's length differs from the clock's (or the first channel's).
    #[error("channel {channel} has {found} samples, expected {expected}")]
    Length {
        /// The channel, counted from 0.
        channel: usize,
        /// The samples every channel must have.
        expected: usize,
        /// The samples this one has.
        found: usize,
    },
    /// An append with another number of channels than the recording has.
    #[error("expected {expected} channels, got {found}")]
    Channels {
        /// The recording's channels.
        expected: usize,
        /// The append's.
        found: usize,
    },
    /// An explicit x is smaller than the one before it.
    #[error("x is not ordered at sample {index}: {next:?} after {previous:?}")]
    Unordered {
        /// The sample, counted from the first sample ever appended.
        index: u64,
        /// The x before it.
        previous: X,
        /// Its x.
        next: X,
    },
    /// An x that may not be stored (not finite, for `f64`; jiff's last
    /// instant, for `Timestamp`).
    #[error("x is not valid at sample {index}: {value:?}")]
    Invalid {
        /// The sample, counted from the first sample ever appended.
        index: u64,
        /// Its x.
        value: X,
    },
    /// A regular clock's period that is not finite and positive.
    #[error("a regular clock needs a finite positive period, got {0:?}")]
    Period(X::Period),
    /// x values passed to a recording on a regular clock, with their count.
    #[error("a regular clock takes no x values, got {0}")]
    Regular(usize),
    /// A column edge that is not valid, or smaller than the one before it.
    #[error("column edge {index} is invalid or out of order: {value:?}")]
    Edge {
        /// The edge, counted from 0.
        index: usize,
        /// Its value.
        value: X,
    },
    /// Fewer than two column edges, with their count.
    #[error("need at least 2 column edges, got {0}")]
    Edges(usize),
    /// Columns queried against a recording or revision they were not built
    /// for. `recording` and `built` are the columns' own; `current` is the
    /// revision of the recording queried.
    #[error("columns were built for revision {built} of {recording:?}, which is at {current}")]
    Stale {
        /// The recording the columns were built for.
        recording: Id,
        /// Its revision then.
        built: u64,
        /// The queried recording's revision.
        current: u64,
    },
    /// A joint query ([`Series::m4_joint`](crate::series::Series::m4_joint))
    /// over channels of two recordings.
    #[error("joint query over two recordings: {left:?} and {right:?}")]
    Recordings {
        /// The recording of the channel queried.
        left: Id,
        /// The recording of the channel passed with it.
        right: Id,
    },
    /// More retained samples than the index's offsets can address.
    #[error("a recording holds at most {max} samples, got {len}")]
    Capacity {
        /// The samples the recording would hold.
        len: u64,
        /// `u32::MAX`.
        max: u64,
    },
}
