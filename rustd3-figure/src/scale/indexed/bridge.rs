//! [`Indexed`]: series data through generic marks.

use rustd3_core::series::{Points, m4};

use crate::scale::indexed::Extent;
use crate::thin;

/// What a mark needs to fit and thin `Series` data on this x scale.
///
/// Marks are generic over their x scale, while core's indexed recordings
/// exist only for x types that are a `series::Coordinate`; generic mark
/// code cannot name that bound, so the x scale supplies the bridge through
/// [`Position::indexed`](crate::scale::Position::indexed). Object-safe.
pub trait Indexed<D>: crate::sealed::Indexed<D> {
    /// The domain value at output position `px` (the scale's inverse).
    fn invert(&self, px: f64) -> D;

    /// For a `Series` view: the first and last x and the y extremes, among
    /// the values that pass `valid`, of the samples whose x lies in `x`
    /// (ends included; all samples for `None`), read from the index in
    /// O(log n). `None` when `points` is not a `Series` view or no sample
    /// lies in `x`.
    fn series_extent(
        &self,
        points: &Points<'_, D, f64>,
        x: Option<[D; 2]>,
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<Extent<D>>;

    /// For a `Series` view: M4 over the columns between consecutive
    /// `edges`, keeping the samples that pass `valid`. The edges are domain
    /// values in increasing order, or decreasing, as a reversed scale
    /// inverts device-pixel edges. It runs through `frame`, which computes
    /// column boundaries once per draw for every line on one clock and
    /// memoizes output across draws by recording, revision, channel and
    /// grid. `None` when `points` is not a `Series` view, or `edges` are
    /// fewer than two, not monotonic or not valid coordinates.
    fn thin(
        &self,
        points: &Points<'_, D, f64>,
        edges: &[D],
        valid: &dyn Fn(f64) -> bool,
        frame: &mut thin::Frame,
    ) -> Option<m4::Lines>;

    /// For two `Series` views of one recording, a ribbon's `lower` and
    /// `upper` edge: M4 over both together (core's `Series::m4_joint`), so
    /// both keep their extremes in every column, through `frame` as
    /// [`thin`](Indexed::thin). `None` by default, and when either is not a
    /// `Series` view, they belong to two recordings, or `edges` are not
    /// valid.
    fn thin_joint(
        &self,
        _lower: &Points<'_, D, f64>,
        _upper: &Points<'_, D, f64>,
        _edges: &[D],
        _valid: &dyn Fn(f64) -> bool,
        _frame: &mut thin::Frame,
    ) -> Option<m4::Lines> {
        None
    }
}
