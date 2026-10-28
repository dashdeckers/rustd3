//! [`Indexed`]: series data through generic marks, and [`SeriesExtent`],
//! what it reads from a recording.

use rustd3_core::series::{Coordinate, Points, m4};

use crate::thin;

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
    /// `edges` (domain values in increasing order, or decreasing, as a
    /// reversed scale inverts device-pixel edges), keeping the samples that
    /// pass `valid`, through `frame`: column boundaries computed once per
    /// draw for every line on one clock, and output memoized across draws
    /// by recording, revision, channel and grid. `None` when `points` is
    /// not a `Series` view, or `edges` are fewer than two, not monotonic or
    /// not valid coordinates.
    fn thin(
        &self,
        points: &Points<'_, D, f64>,
        edges: &[D],
        valid: &dyn Fn(f64) -> bool,
        frame: &mut thin::Frame,
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

/// [`Indexed::series_extent`] for any coordinate: the samples of `points`
/// whose x lies in `[low, high]` when `x` holds `(low, past)` with `past`
/// the first x beyond `high` (`None` past the coordinate's last value),
/// every sample for `None`.
pub(crate) fn extent<X: Coordinate>(
    points: &Points<'_, X, f64>,
    x: Option<(X, Option<X>)>,
    valid: &dyn Fn(f64) -> bool,
) -> Option<SeriesExtent<X>> {
    let series = points.series()?;
    let recording = series.recording();
    let samples = match x {
        None => 0..series.len(),
        Some((low, past)) => {
            let end = past.map_or(series.len(), |past| recording.locate(past));
            recording.locate(low)..end
        }
    };
    let last = samples.end.checked_sub(1)?;
    let x = [recording.x(samples.start)?, recording.x(last)?];
    let y = series
        .extremes(samples, valid)
        .map(|e| [e.min().1, e.max().1]);
    Some(SeriesExtent { x, y })
}

/// [`Indexed::thin`] for any coordinate: the edges put in increasing
/// order, their grid keyed by `bits` of each edge.
pub(crate) fn thin<X: Coordinate, B: IntoIterator<Item = u64>>(
    points: &Points<'_, X, f64>,
    edges: &[X],
    bits: impl Fn(&X) -> B,
    valid: &dyn Fn(f64) -> bool,
    frame: &mut thin::Frame,
) -> Option<m4::Lines> {
    let series = points.series()?;
    let reversed: Vec<X>;
    let edges = if edges.first() > edges.last() {
        reversed = edges.iter().rev().copied().collect();
        &reversed
    } else {
        edges
    };
    let grid = thin::grid(edges.iter().flat_map(bits));
    frame.lines(series, edges, grid, valid)
}
