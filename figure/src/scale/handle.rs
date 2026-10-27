//! [`ScaleRef`]: a typed handle to a scale a figure owns.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::figure;

/// A typed handle to a scale in a figure's table: `Copy`, so one handle is
/// placed on every mark and axis that should agree. Its type recovers the
/// concrete scale at draw time. A handle carries its figure's id; using it
/// with another figure panics in debug builds ("handle from another
/// figure").
pub struct ScaleRef<S> {
    figure: figure::Id,
    index: u32,
    kind: PhantomData<fn() -> S>,
}

impl<S> Clone for ScaleRef<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for ScaleRef<S> {}

impl<S> PartialEq for ScaleRef<S> {
    fn eq(&self, other: &Self) -> bool {
        self.figure == other.figure && self.index == other.index
    }
}

impl<S> Eq for ScaleRef<S> {}

impl<S> Hash for ScaleRef<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.figure.hash(state);
        self.index.hash(state);
    }
}

/// `ScaleRef { index: 2 }`.
impl<S> fmt::Debug for ScaleRef<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScaleRef")
            .field("index", &self.index)
            .finish_non_exhaustive()
    }
}

impl<S> ScaleRef<S> {
    /// The handle to entry `index` of figure `figure`'s table.
    pub(crate) fn new(figure: figure::Id, index: u32) -> Self {
        ScaleRef {
            figure,
            index,
            kind: PhantomData,
        }
    }

    /// Position in the scale table (the widget keys state by it and the
    /// scale's `TypeId`).
    pub fn index(self) -> usize {
        self.index as usize
    }

    /// The position in figure `figure`'s table: every use of a handle goes
    /// through here. Debug builds panic for a handle of another figure;
    /// release builds skip the check.
    pub(crate) fn of(self, figure: figure::Id) -> usize {
        debug_assert!(self.figure == figure, "handle from another figure");
        self.index()
    }

    /// The same handle without its scale type, for lists of handles of
    /// several types.
    pub(crate) fn erased(self) -> ScaleRef<()> {
        ScaleRef::new(self.figure, self.index)
    }
}
