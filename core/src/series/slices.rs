//! Borrowed parallel columns of mark data, without an index.

/// Borrowed parallel slices, unindexed: `series::slices(&x, &y)`. `Copy`.
#[derive(Debug)]
pub struct Slices<'a, X, Y> {
    x: &'a [X],
    y: &'a [Y],
}

/// Copies the two references, whatever `X` and `Y` are.
impl<X, Y> Clone for Slices<'_, X, Y> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X, Y> Copy for Slices<'_, X, Y> {}

/// Two columns as mark data: pair i is `(x[i], y[i])`, up to the shorter
/// length.
///
/// ```
/// use rustd3_core::series::{self, Data};
///
/// let (t, v) = ([0.0, 1.0, 2.0], [5.0, 7.0]);
/// let points = series::slices(&t, &v).into_points();
/// assert_eq!(points.iter().collect::<Vec<_>>(), [(0.0, 5.0), (1.0, 7.0)]);
/// ```
pub fn slices<'a, X, Y>(x: &'a [X], y: &'a [Y]) -> Slices<'a, X, Y> {
    Slices { x, y }
}

impl<X, Y> Slices<'_, X, Y> {
    /// The number of pairs: the shorter slice's length.
    pub(crate) fn len(&self) -> usize {
        self.x.len().min(self.y.len())
    }
}

impl<X: Clone, Y: Clone> Slices<'_, X, Y> {
    /// Pair `i`, cloned, if both slices hold it.
    pub(crate) fn get(&self, i: usize) -> Option<(X, Y)> {
        Some((self.x.get(i)?.clone(), self.y.get(i)?.clone()))
    }
}
