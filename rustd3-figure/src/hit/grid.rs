//! [`Grid`]: one panel's points in uniform buckets.

use kurbo::{Point, Rect};

/// A bucket's side, px.
const BUCKET: f64 = 32.0;

/// One panel's points in a uniform grid of 32 px buckets. The grid covers
/// the panel's plot area and every point reported, since a line's samples
/// may lie outside the plot area. Each bucket lists its points in the
/// order reported.
#[derive(Debug, Clone)]
pub(crate) struct Grid {
    origin: Point,
    cols: usize,
    rows: usize,
    buckets: Vec<Vec<u32>>,
    /// Each point's mark, datum index and place, in the order reported.
    points: Vec<(usize, usize, Point)>,
}

impl Grid {
    /// The grid over plot area `area` of `points`, each a (mark, datum
    /// index, figure px) triple, in the order reported. Points that are not
    /// finite are left out.
    pub(crate) fn new(area: Rect, points: Vec<(usize, usize, Point)>) -> Grid {
        let points: Vec<(usize, usize, Point)> =
            points.into_iter().filter(|(.., p)| p.is_finite()).collect();
        let extent = points.iter().fold(area, |r, (.., p)| r.union_pt(*p));
        let cells = |length: f64| (length / BUCKET).floor().max(0.0) as usize + 1;
        let (cols, rows) = (cells(extent.width()), cells(extent.height()));
        let mut grid = Grid {
            origin: extent.origin(),
            cols,
            rows,
            buckets: vec![Vec::new(); cols * rows],
            points: Vec::new(),
        };
        for (i, (.., p)) in points.iter().enumerate() {
            let (c, r) = grid.bucket(*p);
            let i = u32::try_from(i).expect("invariant: a panel reports fewer than 2^32 points");
            grid.buckets[r * cols + c].push(i);
        }
        grid.points = points;
        grid
    }

    /// The number of points.
    pub(crate) fn len(&self) -> usize {
        self.points.len()
    }

    /// The mark, datum index, place and distance of the point nearest `at`
    /// within `radius` (edges included). Only the buckets the radius
    /// overlaps are searched, and of equally near points the first
    /// reported wins.
    pub(crate) fn nearest(&self, at: Point, radius: f64) -> Option<(usize, usize, Point, f64)> {
        if !(at.is_finite() && radius >= 0.0) || self.points.is_empty() {
            return None;
        }
        let (c0, r0) = self.bucket(at - (radius, radius));
        let (c1, r1) = self.bucket(at + (radius, radius));
        let mut best: Option<(u32, f64)> = None;
        for r in r0..=r1 {
            for c in c0..=c1 {
                for &i in &self.buckets[r * self.cols + c] {
                    let d = self.points[i as usize].2.distance(at);
                    let better = best.is_none_or(|(j, e)| d < e || (d == e && i < j));
                    if d <= radius && better {
                        best = Some((i, d));
                    }
                }
            }
        }
        best.map(|(i, d)| {
            let (mark, index, p) = self.points[i as usize];
            (mark, index, p, d)
        })
    }

    /// The bucket holding `p`, clamped to the grid.
    fn bucket(&self, p: Point) -> (usize, usize) {
        let at = |v: f64, n: usize| {
            // Saturates for far points; negative ones are bucket 0.
            ((v / BUCKET).floor().max(0.0) as usize).min(n - 1)
        };
        (
            at(p.x - self.origin.x, self.cols),
            at(p.y - self.origin.y, self.rows),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nearest_point_within_the_radius_wins_and_ties_go_to_the_first() {
        let points = vec![
            (0, 0, Point::new(10.0, 10.0)),
            (0, 1, Point::new(20.0, 10.0)),
            (1, 0, Point::new(200.0, 200.0)),
            (1, 1, Point::new(14.0, 12.0)),
        ];
        let grid = Grid::new(Rect::new(0.0, 0.0, 300.0, 300.0), points);
        let (mark, index, _, d) = grid.nearest(Point::new(12.0, 11.0), 5.0).expect("a hit");
        assert_eq!((mark, index), (0, 0));
        assert!((d - 5.0_f64.sqrt()).abs() < 1e-12);
        assert!(grid.nearest(Point::new(100.0, 100.0), 5.0).is_none());
        // Far from every bucket: still searched by clamping.
        let far = grid.nearest(Point::new(-30.0, 10.0), 40.0).expect("a hit");
        assert_eq!((far.0, far.1), (0, 0));
    }

    #[test]
    fn points_outside_the_plot_area_widen_the_grid() {
        let grid = Grid::new(
            Rect::new(0.0, 0.0, 64.0, 64.0),
            vec![(0, 0, Point::new(-50.0, 100.0))],
        );
        assert_eq!((grid.cols, grid.rows), (4, 4));
        let hit = grid.nearest(Point::new(-52.0, 101.0), 3.0).expect("a hit");
        assert_eq!(hit.1, 0);
    }
}
