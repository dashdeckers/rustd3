//! [`Indexed`]: series data through generic marks, and [`SeriesExtent`],
//! what it reads from a recording.

use rustd3_core::series::{Points, m4};

/// What a mark needs to fit and thin `Series` data on this x scale.
///
/// Marks are generic over their x scale, while core's indexed recordings
/// exist only for x types that are a `series::Coordinate`; generic mark
/// code cannot name that bound, so the x scale supplies the bridge through
/// [`Position::indexed`](crate::scale::Position::indexed). Object-safe.
pub trait Indexed<D> {
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
    ) -> Option<SeriesExtent<D>>;

    /// For a `Series` view: M4 over the columns between consecutive
    /// `edges` (domain values in increasing order). `None` when `points`
    /// is not a `Series` view, or `edges` are fewer than two, out of order
    /// or not valid coordinates.
    fn thin(
        &self,
        points: &Points<'_, D, f64>,
        edges: &[D],
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<m4::Lines>;
}

/// What [`Indexed::series_extent`] reads from a recording. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesExtent<D> {
    /// First and last x of the samples considered.
    pub x: [D; 2],
    /// The y extremes; `None` when no sample is valid.
    pub y: Option<[f64; 2]>,
}
