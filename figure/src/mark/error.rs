//! [`Error`]: why a mark draws nothing.

use rustd3_core::series;

/// Why a mark draws nothing, or why a mark's input could not be built
/// ([`GridLength`](Error::GridLength)). A mark keeps the error from when it
/// was built, and [`Figure::check`](crate::Figure::check) reports it as
/// [`Issue::Mark`](crate::check::Issue::Mark).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
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
    /// A density over fewer than two finite values, or over values that
    /// are all equal, which have no kernel density estimate.
    #[error("a density needs two or more finite values that differ, got {values} finite values")]
    Density {
        /// The number of finite values.
        values: usize,
    },
    /// A grid whose values do not fill its cells
    /// ([`Grid::new`](crate::mark::Grid::new)).
    #[error("a grid of {cols} × {rows} cells got {len} values")]
    GridLength {
        /// The grid's column count.
        cols: usize,
        /// The grid's row count.
        rows: usize,
        /// The number of values given.
        len: usize,
    },
    /// A per-point channel (`fill_by`, `size_by`) whose length differs
    /// from the mark's point count. The channel is never truncated; the
    /// mark draws nothing.
    #[error("channel has {values} values for {points} points")]
    ChannelLength {
        /// The mark's point count.
        points: usize,
        /// The channel's value count.
        values: usize,
    },
    /// The first histogram bin edge that is not finite or not above the
    /// one before it (the `helpers` feature's histograms).
    #[error("bin edge {index} is not finite or not above the one before: {value}")]
    Edge {
        /// The edge's index.
        index: usize,
        /// The edge's value.
        value: f64,
    },
    /// A histogram's weights whose count differs from its values' (the
    /// `helpers` feature's histograms).
    #[error("{weights} weights for {values} values")]
    Weights {
        /// The number of values given.
        values: usize,
        /// The number of weights given.
        weights: usize,
    },
}
