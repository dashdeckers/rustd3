//! Thinning of long recordings at draw time, by M4 per device-pixel
//! column. Every line on one clock shares a draw's column boundaries
//! ([`Frame`]), and M4 output is memoized across draws in the figure's
//! [`Cache`](crate::cache::Cache).
//!
//! A line is thinned by default ([`Thin::Auto`]) when its x scale has the
//! bridge to core's index
//! ([`Position::indexed`](crate::scale::Position::indexed)) and its data
//! is a `Series` view. It gets one column per device pixel on screen, four
//! per device pixel for PNG and GIF, and four per list pixel for SVG and
//! PDF. Column edges lie where the x scale's inverse puts the device-pixel
//! edges, so log and time axes get the right columns. Owned points and
//! slices are drawn as given.
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_core::series::{Clock, Recording};
//! use rustd3_figure::Figure;
//! use rustd3_figure::mark::line;
//! use rustd3_figure::thin::Thin;
//!
//! let y: Vec<f64> = (0..100_000).map(|i| (f64::from(i) * 0.01).sin()).collect();
//! let rec = Recording::new(Clock::Regular { start: 0.0, period: 1e-3 }, [&y])?;
//! let mut fig = Figure::new();
//! let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
//! let series = rec.series(0).expect("channel 0");
//! fig.panel(0, 0)
//!     .mark(line(series, t, v))
//!     .mark(line(series, t, v).thin(Thin::Off));
//! # Ok::<(), rustd3_core::series::Error>(())
//! ```

mod frame;
mod grid;
mod memo;

pub use frame::Frame;
pub use grid::Grid;
pub(crate) use memo::Memo;

use crate::scale::Indexed;

/// Whether a line thins its data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Thin {
    /// M4 per column when the x scale has an index bridge and the data is
    /// a `Series` view; as given otherwise.
    #[default]
    Auto,
    /// Every point, joined by the line's curve.
    Off,
}

/// The column edges of a plot area `width` list px wide at `per_px`
/// columns per list px: `n = round(width × per_px)` columns, edge `i` at
/// `indexed`'s inverse of `i / per_px`, for `i` in `0..=n`, in the scale's
/// order (decreasing for a reversed domain). `None` below one column.
pub(crate) fn edges<D>(indexed: &dyn Indexed<D>, width: f64, per_px: f64) -> Option<Vec<D>> {
    let n = (width * per_px).round();
    if !(1.0..=f64::from(u32::MAX)).contains(&n) {
        return None;
    }
    let n = n as u32;
    Some(
        (0..=n)
            .map(|i| indexed.invert(f64::from(i) / per_px))
            .collect(),
    )
}

/// How far, in columns, an edge on a grid is moved left before it is
/// inverted (and before a dash anchor's column is searched).
pub(crate) const LEAD: f64 = 1e-6;

/// The column edges on `grid` of every column that intersects the view
/// `[lo, hi]` (list px along the plot area): edge `c` at `indexed`'s
/// inverse of `grid.origin + c / grid.density`, from the first column's
/// left edge to the last one's right edge, in the scale's order. `None`
/// when no column intersects the view.
///
/// Each edge is first moved a millionth of a column to the left. A pan
/// inverts the same edge through another scale, which can land it an ulp
/// either side of a sample lying exactly on it (round domains and widths
/// put many there). The sample would then change columns between the two
/// draws. A millionth of a column is far above that error, so such a
/// sample falls in the column to its right in both.
pub(crate) fn edges_on<D>(indexed: &dyn Indexed<D>, view: [f64; 2], grid: Grid) -> Option<Vec<D>> {
    let (first, last) = grid.span(view)?;
    let lead = LEAD / grid.density;
    Some(
        (first..=last + 1)
            .map(|c| indexed.invert(grid.edge(c) - lead))
            .collect(),
    )
}

/// A column grid's key: FNV-1a 64 over its edges' bits.
pub(crate) fn grid(bits: impl IntoIterator<Item = u64>) -> u64 {
    bits.into_iter()
        .flat_map(u64::to_le_bytes)
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;
    use crate::scale::Position;

    #[test]
    fn edges_invert_device_pixel_edges() {
        let x = scale::linear().domain([0.0, 100.0]).range([0.0, 1000.0]);
        let indexed = x.indexed().expect("continuous scales are indexed");
        let linear = edges(indexed, 1000.0, 4.0).expect("columns");
        assert_eq!(linear.len(), 4001);
        assert_eq!((linear[0], linear[4000]), (0.0, 100.0));
        assert!((linear[1] - 0.025).abs() < 1e-15);
        let log = scale::log().domain([1.0, 1000.0]).range([0.0, 300.0]);
        let indexed = log.indexed().expect("continuous scales are indexed");
        let decades = edges(indexed, 300.0, 1.0).expect("columns");
        assert!((decades[100] - 10.0).abs() < 1e-12, "{}", decades[100]);
        assert_eq!(edges(indexed, 0.4, 1.0), None);
    }

    #[test]
    fn a_reversed_domain_gives_decreasing_edges() {
        let x = scale::linear().domain([10.0, 0.0]).range([0.0, 10.0]);
        let indexed = x.indexed().expect("continuous scales are indexed");
        let edges = edges(indexed, 10.0, 0.5).expect("columns");
        let expected = [10.0, 8.0, 6.0, 4.0, 2.0, 0.0];
        assert_eq!(edges.len(), expected.len());
        for (e, x) in edges.iter().zip(expected) {
            assert!((e - x).abs() < 1e-12, "{edges:?}");
        }
    }

    #[test]
    fn grids_hash_every_edge() {
        let a = grid([1.0_f64, 2.0].map(f64::to_bits));
        assert_eq!(a, grid([1.0_f64, 2.0].map(f64::to_bits)));
        assert_ne!(a, grid([1.0_f64, 2.5].map(f64::to_bits)));
        assert_ne!(a, grid([2.0_f64, 1.0].map(f64::to_bits)));
    }
}
