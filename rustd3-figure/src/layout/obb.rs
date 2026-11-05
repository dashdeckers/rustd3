//! [`Obb`]: an oriented box, and the separating-axis test both collision
//! passes run on tick labels.

use std::ops::Add;

use kurbo::{Point, Rect, Vec2};

/// An oriented box in list px: its four corners in order around it and
/// its two edge directions as unit vectors. An axis-aligned box keeps the
/// exact edges of the rectangle it came from, so [`Obb::overlaps`] decides
/// as the rectangles' own comparison would.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Obb {
    corners: [Point; 4],
    u: Vec2,
    n: Vec2,
}

impl Obb {
    /// `rect`, axis-aligned.
    pub(crate) fn rect(rect: Rect) -> Obb {
        Obb::rotated(rect, 0.0)
    }

    /// `rect` turned counter-clockwise on screen (y down) by `angle`
    /// radians about the origin: x runs along `u = (cos, −sin)` and y
    /// along `n = (sin, cos)`.
    pub(crate) fn rotated(rect: Rect, angle: f64) -> Obb {
        let (sin, cos) = angle.sin_cos();
        let (u, n) = (Vec2::new(cos, -sin), Vec2::new(sin, cos));
        let at = |x: f64, y: f64| (u * x + n * y).to_point();
        Obb {
            corners: [
                at(rect.x0, rect.y0),
                at(rect.x1, rect.y0),
                at(rect.x1, rect.y1),
                at(rect.x0, rect.y1),
            ],
            u,
            n,
        }
    }

    /// The smallest axis-aligned rectangle around the box.
    pub(crate) fn aabb(&self) -> Rect {
        let [first, rest @ ..] = self.corners;
        rest.iter()
            .fold(Rect::from_points(first, first), |r, p| r.union_pt(*p))
    }

    /// Whether the boxes share interior, by the separating-axis test on
    /// the four edge directions of the two: boxes that touch do not
    /// overlap. For two congruent boxes on one axis, turned by θ with
    /// centers `d` apart along it, the test reduces to the spacing rule:
    /// they are disjoint iff `d·|cos θ| ≥ w` or `d·|sin θ| ≥ h`.
    pub(crate) fn overlaps(&self, other: &Obb) -> bool {
        [self.u, self.n, other.u, other.n].iter().all(|&axis| {
            let (a0, a1) = self.project(axis);
            let (b0, b1) = other.project(axis);
            a0 < b1 && b0 < a1
        })
    }

    /// The box's extent along `axis`: the least and greatest projection
    /// of its corners.
    fn project(&self, axis: Vec2) -> (f64, f64) {
        self.corners
            .iter()
            .map(|p| p.to_vec2().dot(axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            })
    }
}

/// The box moved by `v`.
impl Add<Vec2> for Obb {
    type Output = Obb;

    fn add(self, v: Vec2) -> Obb {
        Obb {
            corners: self.corners.map(|p| p + v),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;

    /// A 60 × 15 px label at `deg`, its center `d` px along x from the
    /// origin.
    fn label(deg: f64, d: f64) -> Obb {
        let local = Rect::new(-30.0, -7.5, 30.0, 7.5);
        Obb::rotated(local, deg.to_radians()) + Vec2::new(d, 0.0)
    }

    /// The least spacing at which two of [`label`] at `deg` stop
    /// overlapping, by bisection.
    fn spacing(deg: f64) -> f64 {
        let (mut low, mut high) = (0.0, 100.0);
        for _ in 0..60 {
            let mid = (low + high) / 2.0;
            if label(deg, 0.0).overlaps(&label(deg, mid)) {
                low = mid;
            } else {
                high = mid;
            }
        }
        high
    }

    #[test]
    fn congruent_labels_need_the_spacing_of_the_rule() {
        // min(w / |cos θ|, h / |sin θ|) for a 60 × 15 px box.
        let rows = [(0.0, 60.0), (30.0, 30.0), (45.0, 21.2132), (90.0, 15.0)];
        for (deg, expected) in rows {
            let d = spacing(deg);
            assert!((d - expected).abs() < 1e-4, "{deg}°: {d}");
        }
    }

    #[test]
    fn rotated_extents_follow_sines_and_cosines() {
        // Reach below a bottom axis: w·|sin θ| + h·|cos θ|.
        let rows = [(0.0, 15.0), (30.0, 42.990), (45.0, 53.033), (90.0, 60.0)];
        for (deg, expected) in rows {
            let height = label(deg, 0.0).aabb().height();
            assert!((height - expected).abs() < 1e-3, "{deg}°: {height}");
        }
        let width = label(30.0, 0.0).aabb().width();
        assert!((width - (60.0 * 0.75f64.sqrt() + 7.5)).abs() < 1e-9);
    }

    #[test]
    fn axis_aligned_boxes_decide_as_rectangles_and_touching_is_not_overlap() {
        let a = Obb::rect(Rect::new(0.0, 0.0, 10.0, 10.0));
        assert_eq!(a.aabb(), Rect::new(0.0, 0.0, 10.0, 10.0));
        assert!(!a.overlaps(&Obb::rect(Rect::new(10.0, 0.0, 20.0, 10.0))));
        assert!(a.overlaps(&Obb::rect(Rect::new(9.9, 9.9, 20.0, 20.0))));
        assert!(!a.overlaps(&Obb::rect(Rect::new(0.0, 10.0, 10.0, 20.0))));
    }

    #[test]
    fn a_rotated_box_clears_one_its_aabb_would_hit() {
        // A 60 × 4 bar at 45° from the origin, and a square in the empty
        // corner of its bounding box.
        let bar = Obb::rotated(Rect::new(0.0, -2.0, 60.0, 2.0), FRAC_PI_2 / 2.0);
        let corner = Obb::rect(Rect::new(30.0, -10.0, 40.0, 0.0));
        assert!(
            bar.aabb()
                .intersect(Rect::new(30.0, -10.0, 40.0, 0.0))
                .area()
                > 0.0
        );
        assert!(!bar.overlaps(&corner) && !corner.overlaps(&bar));
        let on = Obb::rect(Rect::new(20.0, -25.0, 25.0, -20.0));
        assert!(bar.overlaps(&on));
    }
}
