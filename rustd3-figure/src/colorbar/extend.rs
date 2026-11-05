//! [`Extend`]: matplotlib's end triangles for the values a colorbar's
//! scale clamps, and their geometry.

use kurbo::{BezPath, Point, Rect};

/// A triangle's length as a share of the bar's interior: matplotlib's
/// default `extendfrac`.
const FRACTION: f64 = 0.05;

/// Which ends of a colorbar get a triangle for the values beyond its
/// scale's domain, filled with the scale's clamped end color (matplotlib's
/// `extend`). Each triangle is 5% of the bar's interior long, and the bar
/// with its triangles keeps the length its place gives it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Extend {
    /// No triangles: the default.
    #[default]
    Neither,
    /// A triangle at the low end (bottom or left).
    Min,
    /// A triangle at the high end (top or right).
    Max,
    /// A triangle at each end.
    Both,
}

impl Extend {
    /// Splits a bar `length` px long, triangles included, into its
    /// interior's length and its triangles' lengths `[low end, high end]`.
    /// Each triangle is 5% of the interior, as matplotlib shrinks the
    /// interior to fit them.
    pub(crate) fn split(self, length: f64) -> (f64, [f64; 2]) {
        let ends = match self {
            Extend::Neither => [false, false],
            Extend::Min => [true, false],
            Extend::Max => [false, true],
            Extend::Both => [true, true],
        };
        let count = ends.iter().filter(|e| **e).count() as f64;
        let interior = length / (1.0 + FRACTION * count);
        (
            interior,
            ends.map(|e| if e { FRACTION * interior } else { 0.0 }),
        )
    }
}

/// The triangle of `length` px at the low (`high` false) or high end of
/// bar `interior`, vertical (low end at the bottom) or horizontal (low end
/// at the left): its base the bar's end, its tip on the bar's center line.
pub(crate) fn triangle(interior: Rect, vertical: bool, high: bool, length: f64) -> BezPath {
    let c = interior.center();
    let [a, b, tip] = match (vertical, high) {
        (true, false) => [
            Point::new(interior.x0, interior.y1),
            Point::new(interior.x1, interior.y1),
            Point::new(c.x, interior.y1 + length),
        ],
        (true, true) => [
            Point::new(interior.x0, interior.y0),
            Point::new(interior.x1, interior.y0),
            Point::new(c.x, interior.y0 - length),
        ],
        (false, false) => [
            Point::new(interior.x0, interior.y0),
            Point::new(interior.x0, interior.y1),
            Point::new(interior.x0 - length, c.y),
        ],
        (false, true) => [
            Point::new(interior.x1, interior.y0),
            Point::new(interior.x1, interior.y1),
            Point::new(interior.x1 + length, c.y),
        ],
    };
    let mut path = BezPath::new();
    path.move_to(a);
    path.line_to(b);
    path.line_to(tip);
    path.close_path();
    path
}

/// The outline of bar `interior` with triangles of `[low, high]` px at its
/// ends (none where 0), vertical or horizontal, as one closed path.
pub(crate) fn outline(interior: Rect, vertical: bool, [low, high]: [f64; 2]) -> BezPath {
    let Rect { x0, y0, x1, y1 } = interior;
    let c = interior.center();
    let mut path = BezPath::new();
    path.move_to((x0, y0));
    if vertical {
        if high > 0.0 {
            path.line_to((c.x, y0 - high));
        }
        path.line_to((x1, y0));
        path.line_to((x1, y1));
        if low > 0.0 {
            path.line_to((c.x, y1 + low));
        }
        path.line_to((x0, y1));
    } else {
        path.line_to((x1, y0));
        if high > 0.0 {
            path.line_to((x1 + high, c.y));
        }
        path.line_to((x1, y1));
        path.line_to((x0, y1));
        if low > 0.0 {
            path.line_to((x0 - low, c.y));
        }
    }
    path.close_path();
    path
}

#[cfg(test)]
mod tests {
    use kurbo::Shape;

    use super::*;

    #[test]
    fn triangles_are_5_percent_of_the_interior_and_fit_the_bar() {
        assert_eq!(Extend::Neither.split(300.0), (300.0, [0.0, 0.0]));
        let (interior, ends) = Extend::Both.split(330.0);
        assert!((interior - 300.0).abs() < 1e-9);
        assert!((ends[0] - 15.0).abs() < 1e-9 && (ends[1] - 15.0).abs() < 1e-9);
        let (interior, ends) = Extend::Max.split(315.0);
        assert!((interior - 300.0).abs() < 1e-9 && ends[0] == 0.0);
        assert!((ends[1] - 15.0).abs() < 1e-9);
    }

    #[test]
    fn a_triangle_points_away_from_its_end() {
        let bar = Rect::new(0.0, 10.0, 20.0, 310.0);
        let low = triangle(bar, true, false, 15.0).bounding_box();
        assert_eq!(low, Rect::new(0.0, 310.0, 20.0, 325.0));
        let high = triangle(bar, true, true, 15.0).bounding_box();
        assert_eq!(high, Rect::new(0.0, -5.0, 20.0, 10.0));
        let left = triangle(bar, false, false, 5.0).bounding_box();
        assert_eq!(left, Rect::new(-5.0, 10.0, 0.0, 310.0));
    }

    #[test]
    fn the_outline_runs_round_the_bar_and_its_triangles() {
        let bar = Rect::new(0.0, 10.0, 20.0, 310.0);
        let both = outline(bar, true, [15.0, 15.0]);
        assert_eq!(both.bounding_box(), Rect::new(0.0, -5.0, 20.0, 325.0));
        assert_eq!(both.elements().len(), 7);
        assert_eq!(outline(bar, false, [0.0, 0.0]).bounding_box(), bar);
    }
}
