//! Hit testing for hover: the points marks draw, in a bucket grid per
//! panel. [`Figure::hits`](crate::Figure::hits) builds an [`Index`] from
//! what each mark reports to a [`Sink`] in
//! [`Mark::hits`](crate::mark::Mark::hits).
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
//! let area = r.get_plot_areas()[0];
//! let near = Point::new(area.x0 + 0.7 * area.width(), area.y0 + 0.5 * area.height());
//! let hit = index.nearest(near, 8.0).expect("the second dot");
//! assert_eq!((hit.panel, hit.mark, hit.index), (0, 0, 1));
//! # }
//! ```

mod grid;
mod sink;

use std::cell::RefCell;

pub(crate) use grid::Grid;
use kurbo::Point;
use rustd3_core::color::Rgb;
pub use sink::Sink;

use crate::cache::Cache;
use crate::figure::{Figure, Resolved};
use crate::mark::Ctx;

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
    /// Per panel, its plot area and its curves, for readouts.
    curves: Vec<(kurbo::Rect, Vec<Curve>)>,
}

/// A curve a mark reported ([`Sink::series`]): its mark, label, color, and
/// points sorted by x.
#[derive(Debug, Clone)]
struct Curve {
    mark: usize,
    label: Option<String>,
    color: Rgb,
    points: Vec<(Point, Option<[f64; 2]>)>,
}

/// What the curves of the panel under a position show at its x
/// ([`Index::read`]). Plain data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Readout {
    /// The panel's index, in the order of first use.
    pub panel: usize,
    /// The position read at, figure px.
    pub at: Point,
    /// Each curve that spans the position's x, in mark order.
    pub readings: Vec<Reading>,
}

/// One curve at a readout's x. Plain data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Reading {
    /// The mark's index in its panel, in insertion order.
    pub mark: usize,
    /// Its legend label: its key, when it is a string.
    pub label: Option<String>,
    /// The color it is drawn in.
    pub color: Rgb,
    /// The curve at the readout's x, figure px, between the two points
    /// it passes through on either side.
    pub at: Point,
    /// That place in the data, x and y, interpolated as the curve is drawn
    /// straight between them; `None` for a curve on scales whose domain
    /// is not numbers.
    pub data: Option<[f64; 2]>,
}

impl Figure<'_> {
    /// The points the marks of `resolved`'s panels draw, in figure px, with
    /// each panel's points in a grid of 32 px buckets.
    ///
    /// Each mark reports its points in
    /// [`Mark::hits`](crate::mark::Mark::hits). Dots report every point
    /// drawn. Lines report the points they pass through inside the plot
    /// area; a thinned line reports its kept samples, read through
    /// `cache`'s M4 memo, so a figure drawn with the same cache queries
    /// nothing again. Bars and histogram bars report the center of each
    /// bar's value end inside the plot area (the top of a vertical bar, the
    /// right end of a horizontal one). Rules, areas, ribbons, densities and
    /// rects report none. Lines also report their label, color and
    /// data, which [`Index::read`] reads at any x.
    pub fn hits(&self, resolved: &Resolved, cache: &mut Cache) -> Index {
        let (text, frame) = cache.start();
        let frame = RefCell::new(frame);
        let mut panels = Vec::new();
        let mut curves = Vec::new();
        for (i, (panel, area)) in self
            .panels()
            .iter()
            .zip(resolved.get_plot_areas())
            .enumerate()
        {
            let taken = RefCell::new(Vec::new());
            let ctx = Ctx::new(resolved, self.get_theme(), *area, text, &frame, &taken);
            let mut points = Vec::new();
            let mut lines = Vec::new();
            for (j, mark) in panel.marks().iter().enumerate() {
                let mut sink = Sink::new(area.origin());
                mark.hits(&ctx.at(i, j), &mut sink);
                let (reported, series) = sink.into_parts();
                if let Some((label, color)) = series {
                    let mut points: Vec<(Point, Option<[f64; 2]>)> = reported
                        .iter()
                        .filter(|(_, p, _)| p.is_finite())
                        .map(|(_, p, d)| (*p, *d))
                        .collect();
                    points.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
                    lines.push(Curve {
                        mark: j,
                        label,
                        color,
                        points,
                    });
                }
                points.extend(reported.into_iter().map(|(k, p, _)| (j, k, p)));
            }
            panels.push(Grid::new(*area, points));
            curves.push((*area, lines));
        }
        cache.end(frame.into_inner());
        Index { panels, curves }
    }
}

impl Index {
    /// The point nearest `at` (figure px) within `radius` px, edges
    /// included. Of equally near points, the one reported first wins, in
    /// panel, then mark, then point order.
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

    /// What the curves of the panel whose plot area holds `at` (figure px,
    /// edges included) show at its x: each line's place, data, label and
    /// color there, straight between the two points it passes through on
    /// either side. A curve that does not span the x is left out. `None`
    /// outside every plot area. Lines read their points inside the plot
    /// area, as hit testing does, so a readout follows the zoomed view.
    #[must_use]
    pub fn read(&self, at: Point) -> Option<Readout> {
        if !at.is_finite() {
            return None;
        }
        let inside =
            |r: &kurbo::Rect| (r.x0..=r.x1).contains(&at.x) && (r.y0..=r.y1).contains(&at.y);
        let panel = self.curves.iter().position(|(area, _)| inside(area))?;
        let readings = self.curves[panel]
            .1
            .iter()
            .filter_map(|curve| curve.at(at.x))
            .collect();
        Some(Readout {
            panel,
            at,
            readings,
        })
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

impl Curve {
    /// The curve at `x` figure px, between the last point at or left of it
    /// and the first right of it; `None` when it does not span `x`.
    fn at(&self, x: f64) -> Option<Reading> {
        let after = self.points.partition_point(|(p, _)| p.x <= x);
        let (a, da) = *self.points.get(after.checked_sub(1)?)?;
        let (p, data) = match self.points.get(after) {
            Some(&(b, db)) => {
                let t = (x - a.x) / (b.x - a.x);
                let data = da
                    .zip(db)
                    .map(|(da, db)| [0, 1].map(|k| da[k] + t * (db[k] - da[k])));
                (a.lerp(b, t), data)
            }
            None if a.x == x => (a, da),
            None => return None,
        };
        Some(Reading {
            mark: self.mark,
            label: self.label.clone(),
            color: self.color,
            at: p,
            data,
        })
    }
}
