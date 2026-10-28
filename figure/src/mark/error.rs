//! [`Error`]: why a mark draws nothing.

use rustd3_core::series;

/// Why a mark draws nothing: the mark keeps the error from when it was
/// built and [`Figure::check`](crate::Figure::check) reports it
/// ([`Issue::Mark`](crate::check::Issue::Mark)).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// A ribbon over channels of two recordings, which no joint query
    /// thins.
    #[error("a ribbon over channels of two recordings: {left:?} and {right:?}")]
    Recordings {
        /// The lower edge's recording.
        left: series::Id,
        /// The upper edge's recording.
        right: series::Id,
    },
}
