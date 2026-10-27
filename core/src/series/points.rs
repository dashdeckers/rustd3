//! What a mark holds: its data as pairs, without bounds on their types.

use std::any::Any;

use crate::series::{Coordinate, Recording, Series, Slices};

/// What a mark holds: collected pairs, borrowed slices or a borrowed
/// recording channel. Opaque: the indexed case exists only for
/// `X: Coordinate, Y = f64`, yet `Points` itself needs no bound.
pub struct Points<'a, X, Y> {
    repr: Repr<'a, X, Y>,
}

enum Repr<'a, X, Y> {
    Owned(Vec<(X, Y)>),
    Slices(Slices<'a, X, Y>),
    /// A bound-free view of the recording, the channel, and the identity
    /// (`Y` is `f64` whenever this variant exists).
    Series(&'a dyn Source<X>, usize, fn(f64) -> Y),
}

/// A recording seen without `X: Coordinate`, so that `Points` needs no
/// bound; [`Points::series`] recovers the recording through `any`.
pub(crate) trait Source<X> {
    /// The number of samples.
    fn len(&self) -> usize;
    /// Sample `i` of `channel`.
    fn get(&self, channel: usize, i: usize) -> Option<(X, f64)>;
    /// The recording itself, to downcast.
    fn any(&self) -> &(dyn Any + 'static);
}

impl<X: Coordinate> Source<X> for Recording<X> {
    fn len(&self) -> usize {
        Recording::len(self)
    }

    fn get(&self, channel: usize, i: usize) -> Option<(X, f64)> {
        self.series(channel)?.get(i)
    }

    fn any(&self) -> &(dyn Any + 'static) {
        self
    }
}

impl<'a, X, Y> Points<'a, X, Y> {
    /// Collected pairs.
    pub(crate) fn collected(pairs: Vec<(X, Y)>) -> Self {
        Self {
            repr: Repr::Owned(pairs),
        }
    }

    /// Borrowed columns.
    pub(crate) fn borrowed(slices: Slices<'a, X, Y>) -> Self {
        Self {
            repr: Repr::Slices(slices),
        }
    }

    /// The number of pairs; for slices, the shorter length.
    pub fn len(&self) -> usize {
        match &self.repr {
            Repr::Owned(pairs) => pairs.len(),
            Repr::Slices(slices) => slices.len(),
            Repr::Series(source, ..) => source.len(),
        }
    }

    /// Whether there are no pairs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, X: Coordinate> Points<'a, X, f64> {
    /// One channel of a recording, borrowed with its index.
    pub(crate) fn indexed(series: Series<'a, X>) -> Self {
        let repr = Repr::Series(series.recording(), series.channel(), std::convert::identity);
        Self { repr }
    }
}

impl<X: Copy, Y: Copy> Points<'_, X, Y> {
    /// Pair `i`, if there is one.
    pub fn get(&self, i: usize) -> Option<(X, Y)> {
        match &self.repr {
            Repr::Owned(pairs) => pairs.get(i).copied(),
            Repr::Slices(slices) => slices.get(i),
            Repr::Series(source, channel, y) => source.get(*channel, i).map(|(x, v)| (x, y(v))),
        }
    }

    /// Every pair, in order.
    pub fn iter(&self) -> impl Iterator<Item = (X, Y)> + '_ {
        (0..self.len()).map_while(|i| self.get(i))
    }
}

impl<'a, X: Coordinate, Y> Points<'a, X, Y> {
    /// The indexed view, for thinning and fitting to what is visible;
    /// `None` for collected pairs and slices.
    pub fn series(&self) -> Option<Series<'a, X>> {
        match &self.repr {
            Repr::Series(source, channel, _) => {
                let source: &'a dyn Source<X> = *source;
                let recording = source
                    .any()
                    .downcast_ref::<Recording<X>>()
                    .expect("invariant: a series' source is a recording of its x type");
                Some(Series::new(recording, *channel))
            }
            Repr::Owned(_) | Repr::Slices(_) => None,
        }
    }
}
