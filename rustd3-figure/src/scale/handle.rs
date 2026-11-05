//! [`ScaleRef`]: a typed handle to a scale a figure owns.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::figure;

/// A typed handle to a scale in a figure's table: `Copy`, so one handle is
/// placed on every mark and axis that should agree. Its type recovers the
/// concrete scale at draw time. A handle carries its figure's id: used
/// with another figure it finds no scale, what uses it draws nothing, and
/// [`Figure::check`](crate::Figure::check) reports it
/// ([`Issue::ForeignHandle`](crate::check::Issue::ForeignHandle)).
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::{dots, line};
///
/// let mut fig = Figure::new();
/// let x = fig.fit(scale::linear());
/// let y = fig.scale(scale::log().domain([1.0, 100.0]));
/// // Two panels share x by sharing the handle; a copy is the same handle.
/// fig.panel(0, 0).mark(line([(0.0, 1.0), (1.0, 10.0)], x, y));
/// fig.panel(1, 0).mark(dots([(0.5, 50.0)], x, y));
/// assert!(fig.owns(x) && !Figure::new().owns(x));
/// ```
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

    /// The handle's position in its figure's scale table.
    #[must_use]
    pub fn get_index(self) -> usize {
        self.index as usize
    }

    /// The position in figure `figure`'s table: every use of a handle goes
    /// through here. `None` for a handle of another figure.
    pub(crate) fn of(self, figure: figure::Id) -> Option<usize> {
        (self.figure == figure).then(|| self.get_index())
    }

    /// The same handle without its scale type, for lists of handles of
    /// several types.
    pub(crate) fn erased(self) -> ScaleRef<()> {
        ScaleRef::new(self.figure, self.index)
    }
}
