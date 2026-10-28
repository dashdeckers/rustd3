//! [`Keyed`]: a palette key applied to the figure's marks a chart draws
//! through.

use crate::mark::{Line, Rect};
use crate::scale::{Paint, Position, ScaleRef};

/// A palette handle and one of its keys, with the palette's type erased:
/// what `.fill(palette, key)` or `.stroke(palette, key)` gives a chart
/// that draws through [`rect`](crate::mark::rect) or
/// [`line`](crate::mark::line).
pub(super) trait Keyed<X: Position<Domain = f64>, Y: Position<Domain = f64>> {
    /// `m` filled with the key's color.
    fn rect(&self, m: Rect<X, Y>) -> Rect<X, Y>;

    /// `m` stroked with the key's color.
    fn line(&self, m: Line<'static, X, Y>) -> Line<'static, X, Y>;
}

/// A palette handle and a key of its domain.
struct Pair<C: Paint> {
    palette: ScaleRef<C>,
    key: C::Domain,
}

impl<C, X, Y> Keyed<X, Y> for Pair<C>
where
    C: Paint,
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    fn rect(&self, m: Rect<X, Y>) -> Rect<X, Y> {
        m.fill(self.palette, self.key.clone())
    }

    fn line(&self, m: Line<'static, X, Y>) -> Line<'static, X, Y> {
        m.stroke(self.palette, self.key.clone())
    }
}

/// `palette`'s key `key`, erased.
pub(super) fn keyed<C, X, Y>(palette: ScaleRef<C>, key: C::Domain) -> Box<dyn Keyed<X, Y>>
where
    C: Paint,
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    Box::new(Pair { palette, key })
}
