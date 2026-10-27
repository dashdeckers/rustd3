//! What a mark holds: its data as pairs, without bounds on their types.

use std::any::Any;
use std::fmt;

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

/// Clones collected pairs; borrowed slices and recordings stay borrowed.
impl<X: Clone, Y: Clone> Clone for Points<'_, X, Y> {
    fn clone(&self) -> Self {
        let repr = match &self.repr {
            Repr::Owned(pairs) => Repr::Owned(pairs.clone()),
            Repr::Slices(slices) => Repr::Slices(*slices),
            Repr::Series(source, channel, y) => Repr::Series(*source, *channel, *y),
        };
        Self { repr }
    }
}

/// What the points are and how many, whatever `X` and `Y` are:
/// `Points { kind: "collected", len: 2 }`, `"slices"` or `"series"`.
impl<X, Y> fmt::Debug for Points<'_, X, Y> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match &self.repr {
            Repr::Owned(_) => "collected",
            Repr::Slices(_) => "slices",
            Repr::Series(..) => "series",
        };
        f.debug_struct("Points")
            .field("kind", &kind)
            .field("len", &self.len())
            .finish()
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
    pub(crate) fn from_series(series: Series<'a, X>) -> Self {
        let repr = Repr::Series(series.recording(), series.channel(), std::convert::identity);
        Self { repr }
    }
}

impl<'a, X, Y> Points<'a, X, Y> {
    /// The same recording channel as `f64` pairs, for code generic over
    /// `X` and `Y` that thins or fits through an index: `Some` for a
    /// borrowed [`Series`] (a channel holds `f64`, whatever `Y` is named
    /// here), `None` for collected pairs and slices. Needs no bound.
    ///
    /// ```
    /// use rustd3_core::series::{Clock, Data, Points, Recording};
    ///
    /// fn view<'a, X, Y>(points: &Points<'a, X, Y>) -> Option<Points<'a, X, f64>> {
    ///     points.indexed()
    /// }
    ///
    /// let rec = Recording::new(Clock::Explicit(vec![0.0, 1.0]), [[4.0, 2.0]])?;
    /// let series = rec.series(0).expect("channel 0").into_points();
    /// assert_eq!(view(&series).map(|p| p.get(1)), Some(Some((1.0, 2.0))));
    /// assert!(view(&[(0.0, 1.0)].into_points()).is_none());
    /// # Ok::<(), rustd3_core::series::Error>(())
    /// ```
    pub fn indexed(&self) -> Option<Points<'a, X, f64>> {
        match &self.repr {
            Repr::Series(source, channel, _) => Some(Points {
                repr: Repr::Series(*source, *channel, std::convert::identity),
            }),
            Repr::Owned(_) | Repr::Slices(_) => None,
        }
    }
}

impl<X: Clone, Y: Clone> Points<'_, X, Y> {
    /// Pair `i`, if there is one: cloned from collected pairs and slices,
    /// read from a recording.
    pub fn get(&self, i: usize) -> Option<(X, Y)> {
        match &self.repr {
            Repr::Owned(pairs) => pairs.get(i).cloned(),
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
