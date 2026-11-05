//! The guard band: geometry clipped in `f64` to the plot area grown by its
//! own width and height on every side, so that no coordinate handed to a
//! backend exceeds three times the panel's size, whatever the zoom. The
//! figure's marks clip polylines (Liang–Barsky) and polygons
//! (Sutherland–Hodgman) to it. A custom mark clips its filled shapes with
//! [`polygon`] and clamps its points to [`band`].
//!
//! ```
//! use rustd3_figure::kurbo::Point;
//! use rustd3_figure::mark::guard;
//!
//! // A triangle reaching far past a 600 × 400 px plot area.
//! let band = guard::band(600.0, 400.0);
//! let points = [Point::new(300.0, 200.0), Point::new(1e7, 200.0), Point::new(300.0, 1e7)];
//! let clipped = guard::polygon(&points, band);
//! assert!(clipped.iter().all(|p| p.x <= 1200.0 && p.y <= 800.0));
//! ```

use kurbo::{Point, Rect};

/// The guard band of a plot area `w × h` px in panel coordinates
/// (`ctx.get_rect()`'s width and height for a mark): `[−w, 2w] × [−h, 2h]`.
#[must_use]
pub fn band(w: f64, h: f64) -> Rect {
    Rect::new(-w, -h, 2.0 * w, 2.0 * h)
}

/// The polylines through `points` (`None` breaks the line) with every
/// segment clipped to `rect`, as items for a line generator. A segment
/// wholly outside is dropped and breaks the line, one that leaves `rect`
/// ends its run where it leaves, and one that enters starts a run where it
/// enters. Points inside `rect` are kept bit for bit.
pub(crate) fn clip(
    points: impl IntoIterator<Item = Option<Point>>,
    rect: Rect,
) -> Vec<Option<Point>> {
    let mut out = Vec::new();
    let mut previous: Option<Point> = None;
    let mut open = false;
    for point in points {
        let Some(p) = point else {
            if open {
                out.push(None);
                open = false;
            }
            previous = None;
            continue;
        };
        match previous {
            None => {
                if within(rect, p) {
                    out.push(Some(p));
                    open = true;
                }
            }
            Some(q) => match segment(q, p, rect) {
                None => {
                    if open {
                        out.push(None);
                        open = false;
                    }
                }
                Some((a, b)) => {
                    if !open {
                        out.push(Some(a));
                    }
                    out.push(Some(b));
                    open = b == p;
                    if !open {
                        out.push(None);
                    }
                }
            },
        }
        previous = Some(p);
    }
    out
}

/// The closed polygon through `points` clipped to `rect`
/// (Sutherland–Hodgman), such as a [`band`]. Returns the vertices of its
/// part inside, in order, with edges along `rect` where it was cut, or
/// none when no part is inside. Points inside `rect` are kept bit for bit.
#[must_use]
pub fn polygon(points: &[Point], rect: Rect) -> Vec<Point> {
    let mut out = points.to_vec();
    for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
        let Some(&last) = out.last() else {
            break;
        };
        let input = std::mem::take(&mut out);
        let mut previous = last;
        for p in input {
            match (side.keeps(previous, rect), side.keeps(p, rect)) {
                (true, true) => out.push(p),
                (true, false) => out.push(side.cut(previous, p, rect)),
                (false, true) => out.extend([side.cut(previous, p, rect), p]),
                (false, false) => {}
            }
            previous = p;
        }
    }
    out
}

/// A side of the clip rectangle, which keeps the half-plane inside it.
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl Side {
    /// Whether `p` lies on `rect`'s side of this edge, the edge included.
    fn keeps(self, p: Point, rect: Rect) -> bool {
        match self {
            Side::Left => p.x >= rect.x0,
            Side::Right => p.x <= rect.x1,
            Side::Top => p.y >= rect.y0,
            Side::Bottom => p.y <= rect.y1,
        }
    }

    /// Where the segment from `a` to `b`, which crosses this edge of
    /// `rect`, crosses it.
    fn cut(self, a: Point, b: Point, rect: Rect) -> Point {
        let at_x = |x: f64| Point::new(x, a.y + (b.y - a.y) * (x - a.x) / (b.x - a.x));
        let at_y = |y: f64| Point::new(a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y), y);
        match self {
            Side::Left => at_x(rect.x0),
            Side::Right => at_x(rect.x1),
            Side::Top => at_y(rect.y0),
            Side::Bottom => at_y(rect.y1),
        }
    }
}

/// The part of the segment from `a` to `b` inside `rect` (Liang–Barsky),
/// with `a` and `b` themselves where they lie inside; `None` when no part
/// is inside or the clipped ends are not finite.
fn segment(a: Point, b: Point, rect: Rect) -> Option<(Point, Point)> {
    let d = b - a;
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    let edges = [
        (-d.x, a.x - rect.x0),
        (d.x, rect.x1 - a.x),
        (-d.y, a.y - rect.y0),
        (d.y, rect.y1 - a.y),
    ];
    for (p, q) in edges {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    if t0 > t1 {
        return None;
    }
    let start = if t0 == 0.0 { a } else { a + d * t0 };
    let end = if t1 == 1.0 { b } else { a + d * t1 };
    (start.is_finite() && end.is_finite()).then_some((start, end))
}

/// Whether `p` lies in `rect`, edges included.
fn within(rect: Rect, p: Point) -> bool {
    (rect.x0..=rect.x1).contains(&p.x) && (rect.y0..=rect.y1).contains(&p.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The unit square.
    fn unit() -> Rect {
        Rect::new(0.0, 0.0, 1.0, 1.0)
    }

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn a_segment_inside_is_kept_bit_for_bit() {
        let (a, b) = (p(0.1, 0.3), p(0.7, 0.9));
        assert_eq!(segment(a, b, unit()), Some((a, b)));
    }

    #[test]
    fn a_crossing_segment_is_cut_at_both_edges() {
        assert_eq!(
            segment(p(-1.0, 0.5), p(2.0, 0.5), unit()),
            Some((p(0.0, 0.5), p(1.0, 0.5)))
        );
        assert_eq!(
            segment(p(0.5, 2.0), p(0.5, -2.0), unit()),
            Some((p(0.5, 1.0), p(0.5, 0.0)))
        );
        assert_eq!(
            segment(p(-1.0, -1.0), p(1.0, 1.0), unit()),
            Some((p(0.0, 0.0), p(1.0, 1.0)))
        );
    }

    #[test]
    fn a_segment_wholly_outside_is_dropped() {
        assert_eq!(segment(p(-1.0, 2.0), p(2.0, 2.0), unit()), None);
        assert_eq!(segment(p(-1.0, 0.5), p(-0.5, 0.5), unit()), None);
        assert_eq!(segment(p(2.0, -1.0), p(3.0, 0.5), unit()), None);
        assert_eq!(segment(p(-0.5, 0.75), p(0.5, 1.75), unit()), None);
    }

    #[test]
    fn a_segment_through_a_corner_touches_it() {
        let corner = p(0.0, 1.0);
        assert_eq!(
            segment(p(-0.5, 0.5), p(0.5, 1.5), unit()),
            Some((corner, corner))
        );
    }

    #[test]
    fn a_segment_whose_cut_overflows_is_dropped() {
        assert_eq!(segment(p(-1e308, 0.5), p(1e308, 0.5), unit()), None);
    }

    #[test]
    fn a_polyline_breaks_where_it_leaves_and_restarts_where_it_enters() {
        let points = [p(0.5, 0.5), p(3.0, 0.5), p(3.0, 0.25), p(0.5, 0.25)];
        assert_eq!(
            clip(points.map(Some), unit()),
            [
                Some(p(0.5, 0.5)),
                Some(p(1.0, 0.5)),
                None,
                Some(p(1.0, 0.25)),
                Some(p(0.5, 0.25)),
            ]
        );
    }

    #[test]
    fn gaps_and_segments_wholly_outside_break_the_polyline() {
        let points = [
            Some(p(0.125, 0.125)),
            Some(p(0.25, 0.125)),
            None,
            Some(p(0.25, 0.25)),
            Some(p(0.25, 4.25)),
            Some(p(2.5, 4.25)),
            Some(p(2.5, 0.5)),
            Some(p(0.5, 0.5)),
        ];
        assert_eq!(
            clip(points, unit()),
            [
                Some(p(0.125, 0.125)),
                Some(p(0.25, 0.125)),
                None,
                Some(p(0.25, 0.25)),
                Some(p(0.25, 1.0)),
                None,
                Some(p(1.0, 0.5)),
                Some(p(0.5, 0.5)),
            ]
        );
    }

    #[test]
    fn a_lone_point_is_kept_inside_and_dropped_outside() {
        assert_eq!(clip([Some(p(0.5, 0.5))], unit()), [Some(p(0.5, 0.5))]);
        assert!(clip([Some(p(5.0, 0.5)), None], unit()).is_empty());
    }

    #[test]
    fn a_polygon_inside_is_kept_bit_for_bit() {
        let square = [p(0.1, 0.1), p(0.9, 0.1), p(0.9, 0.7), p(0.1, 0.7)];
        assert_eq!(polygon(&square, unit()), square);
    }

    #[test]
    fn a_polygon_crossing_the_band_is_cut_along_its_edges() {
        let tall = [p(0.25, -1.0), p(0.75, -1.0), p(0.75, 2.0), p(0.25, 2.0)];
        assert_eq!(
            polygon(&tall, unit()),
            [p(0.25, 1.0), p(0.25, 0.0), p(0.75, 0.0), p(0.75, 1.0)]
        );
        let corner = [p(0.5, 0.5), p(3.0, 0.5), p(0.5, 3.0)];
        let clipped = polygon(&corner, unit());
        assert!(clipped.iter().all(|q| within(unit(), *q)), "{clipped:?}");
        assert_eq!(clipped.len(), 4);
    }

    #[test]
    fn a_polygon_wholly_outside_is_empty() {
        let away = [p(2.0, 2.0), p(3.0, 2.0), p(3.0, 3.0)];
        assert!(polygon(&away, unit()).is_empty());
        assert!(polygon(&[], unit()).is_empty());
    }

    #[test]
    fn the_guard_band_is_the_plot_area_grown_by_its_size() {
        assert_eq!(band(600.0, 400.0), Rect::new(-600.0, -400.0, 1200.0, 800.0));
    }
}
