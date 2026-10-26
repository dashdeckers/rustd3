//! Why a path operation was rejected.

/// A path operation d3 rejects by throwing.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// An arc with a radius below zero.
    #[error("negative radius: {0}")]
    NegativeRadius(f64),
}
