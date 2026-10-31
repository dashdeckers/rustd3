//! Part identities: what a figure composes for each of its elements.

/// The identity of a [`Part`](crate::list::Part), stable across draws of
/// the same figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(u64);

impl Id {
    /// The identity `value`.
    #[must_use]
    pub const fn new(value: u64) -> Id {
        Id(value)
    }
}
