//! What [`Indexed`](crate::scale::Indexed), the bridge from generic marks
//! to series data, reads from a recording: [`Extent`].

use rustd3_core::series::{Coordinate, Points, m4};

use crate::thin;

// The trait, re-exported as `scale::Indexed` only.
pub(in crate::scale) mod bridge;

/// What [`Indexed::series_extent`](crate::scale::Indexed::series_extent)
/// reads from a recording. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent<D> {
    /// First and last x of the samples considered.
    pub x: [D; 2],
    /// The y extremes; `None` when no sample is valid.
    pub y: Option<[f64; 2]>,
}

/// [`Indexed::series_extent`](crate::scale::Indexed::series_extent) for
/// any coordinate. It reads the samples of `points` whose x lies in
/// `[low, high]`, where `x` holds `(low, past)` and `past` is the first x
/// beyond `high` (`None` past the coordinate's last value). With `x` of
/// `None`, it reads every sample.
pub(crate) fn extent<X: Coordinate>(
    points: &Points<'_, X, f64>,
    x: Option<(X, Option<X>)>,
    valid: &dyn Fn(f64) -> bool,
) -> Option<Extent<X>> {
    let series = points.series()?;
    let recording = series.get_recording();
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
        .map(|e| [e.get_min().1, e.get_max().1]);
    Some(Extent { x, y })
}

/// [`Indexed::thin`](crate::scale::Indexed::thin) for any coordinate, and
/// [`Indexed::thin_joint`](crate::scale::Indexed::thin_joint) with
/// an `upper` edge. The edges are put in increasing order, and their grid
/// is keyed by `bits` of each edge.
pub(crate) fn thin<X: Coordinate, B: IntoIterator<Item = u64>>(
    points: &Points<'_, X, f64>,
    upper: Option<&Points<'_, X, f64>>,
    edges: &[X],
    bits: impl Fn(&X) -> B,
    valid: &dyn Fn(f64) -> bool,
    frame: &mut thin::Frame,
) -> Option<m4::Lines> {
    let series = points.series()?;
    let upper = match upper {
        Some(upper) => Some(upper.series()?),
        None => None,
    };
    let reversed: Vec<X>;
    let edges = if edges.first() > edges.last() {
        reversed = edges.iter().rev().copied().collect();
        &reversed
    } else {
        edges
    };
    let grid = thin::grid(edges.iter().flat_map(bits));
    frame.lines(series, upper, edges, grid, valid)
}
