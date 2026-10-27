//! What marks take as data.

use crate::series::{Coordinate, Points, Series, Slices};

/// Mark data: collected pairs, a borrowed indexed view, or borrowed slices.
/// No bound on `X` or `Y`: band keys (`PartialEq` only) work.
///
/// A borrowed `Vec` yields references, not pairs, so it is rejected (E0271):
///
/// ```compile_fail,E0271
/// use rustd3_core::series::{Data, Points};
///
/// fn mark<'a, X, Y>(data: impl Data<'a, X, Y>) -> Points<'a, X, Y> {
///     data.into_points()
/// }
///
/// let pairs: Vec<(f64, f64)> = vec![(0.0, 1.0), (1.0, 3.0)];
/// mark(&pairs);
/// ```
///
/// Copy the pairs out instead, or pass two columns with
/// [`slices`](crate::series::slices):
///
/// ```
/// use rustd3_core::series::{self, Data, Points};
///
/// fn mark<'a, X, Y>(data: impl Data<'a, X, Y>) -> Points<'a, X, Y> {
///     data.into_points()
/// }
///
/// let pairs: Vec<(f64, f64)> = vec![(0.0, 1.0), (1.0, 3.0)];
/// assert_eq!(mark(pairs.iter().copied()).len(), 2);
/// assert_eq!(mark(series::slices(&[0.0, 1.0], &["a", "b"])).get(1), Some((1.0, "b")));
/// ```
///
/// A generic function that calls `into_points` without naming `Points<'a,
/// X, Y>` (or a type that holds it) in its signature needs `X: 'a, Y: 'a`
/// spelled out: borrowed slices are part of what `Points` can hold.
pub trait Data<'a, X, Y> {
    /// The data as a mark holds it.
    fn into_points(self) -> Points<'a, X, Y>;
}

/// Collected: small, computed or filtered data.
impl<'a, X, Y, I: IntoIterator<Item = (X, Y)>> Data<'a, X, Y> for I {
    fn into_points(self) -> Points<'a, X, Y> {
        Points::collected(self.into_iter().collect())
    }
}

/// Indexed: one channel, the view by value, nothing copied.
impl<'a, X: Coordinate> Data<'a, X, f64> for Series<'a, X> {
    fn into_points(self) -> Points<'a, X, f64> {
        Points::from_series(self)
    }
}

/// Borrowed columns without an index.
impl<'a, X, Y> Data<'a, X, Y> for Slices<'a, X, Y> {
    fn into_points(self) -> Points<'a, X, Y> {
        Points::borrowed(self)
    }
}
