//! Hit testing: the points marks draw, in a bucket grid per panel, for
//! hover. [`Figure::hits`](crate::Figure::hits) builds an [`Index`] from
//! what each mark reports to a [`Sink`] ([`Mark::hits`](crate::mark::Mark::hits)).
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::kurbo::{Point, Size};
//! use rustd3_figure::mark::dots;
//! use rustd3_figure::{Figure, Held, Target};
//!
//! let mut fig = Figure::new();
//! let x = fig.scale(scale::linear().domain([0.0, 10.0]));
//! let y = fig.scale(scale::linear().domain([0.0, 10.0]));
//! fig.panel(0, 0).mark(dots([(2.0, 3.0), (7.0, 5.0)], x, y));
//! # #[cfg(feature = "fonts")] {
//! let mut cache = rustd3_figure::cache::Cache::default();
//! let size = Size::new(640.0, 480.0);
//! let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
//! let index = fig.hits(&r, &mut cache);
//! let area = r.plot_areas()[0];
//! let near = Point::new(area.x0 + 0.7 * area.width(), area.y0 + 0.5 * area.height());
//! let hit = index.nearest(near, 8.0).expect("the second dot");
//! assert_eq!((hit.panel, hit.mark, hit.index), (0, 0, 1));
//! # }
//! ```

mod grid;
mod sink;

use std::cell::RefCell;

use kurbo::Point;

use crate::cache::Cache;
use crate::figure::{Figure, Resolved};
use crate::mark::Ctx;

pub(crate) use grid::Grid;
pub use sink::Sink;

/// A point a mark drew, found near a position. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Hit {
    /// The panel's index, in the order of first use.
    pub panel: usize,
    /// The mark's index in its panel, in insertion order.
    pub mark: usize,
    /// The datum's index in the mark's data (a sample's index in its
    /// recording).
    pub index: usize,
    /// Where the point is drawn, figure px.
    pub at: Point,
    /// Its distance from the position asked about, px.
    pub distance: f64,
}

/// The points a figure's marks draw, per panel in a uniform grid of 32 px
/// buckets.
///
/// ```
/// # #[cfg(feature = "fonts")] {
/// use rustd3_core::scale;
/// use rustd3_figure::kurbo::Size;
/// use rustd3_figure::mark::dots;
/// use rustd3_figure::{Figure, Held, Target};
/// use rustd3_figure::cache::Cache;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// fig.panel(0, 0).mark(dots([(0.0, 0.0), (1.0, 1.0)], x, y));
/// let mut cache = Cache::default();
/// let resolved = fig.resolve(Size::new(320.0, 240.0), &mut cache, &Held::default(), Target::Vector);
/// let index = fig.hits(&resolved, &mut cache);
/// assert_eq!(index.len(), 2);
/// let far = index.nearest((-100.0, -100.0).into(), 12.0);
/// assert!(far.is_none());
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Index {
    panels: Vec<Grid>,
}

impl Figure<'_> {
    /// The points the marks of `resolved`'s panels draw, each panel's in a
    /// grid of 32 px buckets, in figure px ([`Mark::hits`](crate::mark::Mark::hits)):
    /// dots report every point drawn, lines the points they pass through
    /// inside the plot area (a thinned line its kept samples, read through
    /// `cache`'s M4 memo, so a figure drawn with the same cache queries
    /// nothing again), bars and histogram bars the centre of each bar's
    /// value end inside the plot area (the top of a vertical bar, the
    /// right end of a horizontal one); rules, areas, ribbons, densities and
    /// rects none.
    pub fn hits(&self, resolved: &Resolved, cache: &mut Cache) -> Index {
        let (text, frame) = cache.start();
        let frame = RefCell::new(frame);
        let panels = self
            .panels()
            .iter()
            .zip(resolved.plot_areas())
            .enumerate()
            .map(|(i, (panel, area))| {
                let ctx = Ctx::new(resolved, self.get_theme(), *area, text, &frame);
                let mut points = Vec::new();
                for (j, mark) in panel.marks().iter().enumerate() {
                    let mut sink = Sink::new(area.origin());
                    mark.hits(&ctx.at(i, j), &mut sink);
                    points.extend(sink.into_points().into_iter().map(|(k, p)| (j, k, p)));
                }
                Grid::new(*area, points)
            })
            .collect();
        cache.end(frame.into_inner());
        Index { panels }
    }
}

impl Index {
    /// The point nearest `at` (figure px) within `radius` px, edges
    /// included: searched in the buckets the radius overlaps; of equally
    /// near points, the one reported first (panel, then mark, then point
    /// order).
    #[must_use]
    pub fn nearest(&self, at: Point, radius: f64) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        for (panel, grid) in self.panels.iter().enumerate() {
            if let Some((mark, index, p, distance)) = grid.nearest(at, radius)
                && best.is_none_or(|b| distance < b.distance)
            {
                best = Some(Hit {
                    panel,
                    mark,
                    index,
                    at: p,
                    distance,
                });
            }
        }
        best
    }

    /// The number of points indexed.
    pub fn len(&self) -> usize {
        self.panels.iter().map(Grid::len).sum()
    }

    /// Whether no point is indexed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
