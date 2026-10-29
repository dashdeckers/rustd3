//! Long strokes cut to what a render can show. A segment of a stroked
//! path whose reach (half the stroke's width, as far as its joins and caps
//! go, plus the render's antialiasing margin, which the area carries)
//! misses the rendered area cannot touch a pixel of it, so it is dropped
//! before vello_cpu expands the stroke: a tile of a long trace strokes its
//! own part of the line, not all of it. Where a subpath is cut, the cut
//! lies at a vertex farther than the reach from the area, so the cap the
//! piece gains there and the join it loses are both out of sight; dashed
//! pieces start at the phase the whole path has there.

use kurbo::{BezPath, Cap, CubicBez, Join, Line, ParamCurveArclen, PathEl, Point, QuadBez, Rect};

/// The fewest elements a stroked path has before it is cut: shorter ones
/// are stroked whole.
pub(super) const LONG: usize = 64;

/// kurbo's accuracy for the arc lengths it dashes by.
const ACCURACY: f64 = 1e-6;

/// What of `path`, stroked with `style`, may touch `area` (in the path's
/// space, antialiasing margin included): one path whose subpaths are the
/// runs of segments within the stroke's reach of it and, for a dashed
/// style, already dashed, each run from the phase the whole path has
/// where it starts. It is stroked with the style's dashes cleared, in one
/// call, so that pieces that overlap still fill as one path. `None` when
/// nothing is dropped.
pub(super) fn cut(path: &BezPath, style: &kurbo::Stroke, area: Rect) -> Option<BezPath> {
    let elements = path.elements();
    let reach = reach(style);
    let grown = area.inflate(reach, reach);
    // Per element, whether its segment may touch the area (moves: false).
    let mut kept = Vec::with_capacity(elements.len());
    let (mut start, mut current) = (Point::ORIGIN, Point::ORIGIN);
    for element in elements {
        let (bounds, to) = match *element {
            PathEl::MoveTo(p) => {
                start = p;
                current = p;
                kept.push(false);
                continue;
            }
            PathEl::LineTo(p) => (Rect::from_points(current, p), p),
            PathEl::QuadTo(c, p) => (Rect::from_points(current, p).union_pt(c), p),
            PathEl::CurveTo(c, d, p) => (Rect::from_points(current, p).union_pt(c).union_pt(d), p),
            PathEl::ClosePath => (Rect::from_points(current, start), start),
        };
        kept.push(meets(bounds, grown));
        current = to;
    }
    let dropped = elements
        .iter()
        .zip(&kept)
        .any(|(element, &kept)| !kept && !matches!(element, PathEl::MoveTo(_)));
    if !dropped {
        return None;
    }
    let mut out = BezPath::new();
    for range in subpaths(elements) {
        pieces(elements, &kept, range, |piece, before| {
            if style.dash_pattern.is_empty() {
                out.extend(piece.elements().iter().copied());
            } else {
                // kurbo reduces the offset by the pattern's period itself.
                let offset = style.dash_offset + before;
                out.extend(kurbo::dash(
                    piece.elements().iter().copied(),
                    offset,
                    &style.dash_pattern,
                ));
            }
        });
    }
    Some(out)
}

/// How far a stroke with `style` reaches from its path: half its width,
/// times the miter limit for miter joins and √2 for square caps.
fn reach(style: &kurbo::Stroke) -> f64 {
    let join = match style.join {
        Join::Miter => style.miter_limit.max(1.0),
        Join::Round | Join::Bevel => 1.0,
    };
    let caps = [style.start_cap, style.end_cap]
        .map(|cap| match cap {
            Cap::Square => std::f64::consts::SQRT_2,
            Cap::Round | Cap::Butt => 1.0,
        })
        .into_iter()
        .fold(1.0, f64::max);
    0.5 * style.width.abs() * join.max(caps)
}

/// Whether two rects meet (touching counts).
fn meets(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

/// The element ranges of each subpath, from its move.
fn subpaths(elements: &[PathEl]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    for (i, element) in elements.iter().enumerate() {
        if i > from && matches!(element, PathEl::MoveTo(_)) {
            ranges.push(from..i);
            from = i;
        }
    }
    if from < elements.len() {
        ranges.push(from..elements.len());
    }
    ranges
}

/// Calls `piece` with each run of kept segments of the subpath at `range`
/// of `elements`, as a path of its own, and the arc length of the subpath
/// before it. A run that is the whole subpath keeps its close; a cut
/// closing segment becomes a line.
fn pieces(
    elements: &[PathEl],
    kept: &[bool],
    range: std::ops::Range<usize>,
    mut piece: impl FnMut(BezPath, f64),
) {
    let whole = range.clone().skip(1).all(|i| kept[i]);
    let (mut start, mut current) = (Point::ORIGIN, Point::ORIGIN);
    let mut before = 0.0;
    let mut run: Option<(BezPath, f64)> = None;
    for i in range {
        let element = elements[i];
        let to = match element {
            PathEl::MoveTo(p) => {
                start = p;
                current = p;
                continue;
            }
            PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => p,
            PathEl::ClosePath => start,
        };
        if kept[i] {
            let (path, _) = run.get_or_insert_with(|| {
                let mut path = BezPath::new();
                path.move_to(current);
                (path, before)
            });
            if matches!(element, PathEl::ClosePath) && !whole {
                path.line_to(start);
            } else {
                path.push(element);
            }
        } else if let Some((path, at)) = run.take() {
            piece(path, at);
        }
        before += length(current, element, start);
        current = to;
    }
    if let Some((path, at)) = run {
        piece(path, at);
    }
}

/// The arc length of `element`'s segment from `current` (a close's back to
/// `start`), as kurbo dashes by it.
fn length(current: Point, element: PathEl, start: Point) -> f64 {
    match element {
        PathEl::MoveTo(_) => 0.0,
        PathEl::LineTo(p) => Line::new(current, p).arclen(ACCURACY),
        PathEl::QuadTo(c, p) => QuadBez::new(current, c, p).arclen(ACCURACY),
        PathEl::CurveTo(c, d, p) => CubicBez::new(current, c, d, p).arclen(ACCURACY),
        PathEl::ClosePath => Line::new(current, start).arclen(ACCURACY),
    }
}

#[cfg(test)]
mod tests {
    use kurbo::{BezPath, Cap, Join, PathEl, Point, Rect, Stroke};

    use super::cut;

    /// A polyline through `points`.
    fn polyline(points: &[(f64, f64)]) -> BezPath {
        let mut path = BezPath::new();
        for (i, &(x, y)) in points.iter().enumerate() {
            if i == 0 {
                path.move_to((x, y));
            } else {
                path.line_to((x, y));
            }
        }
        path
    }

    /// The points of a path's elements, moves and lines alike.
    fn points(path: &BezPath) -> Vec<(f64, f64)> {
        path.elements()
            .iter()
            .filter_map(|element| match *element {
                PathEl::MoveTo(Point { x, y }) | PathEl::LineTo(Point { x, y }) => Some((x, y)),
                PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => None,
            })
            .collect()
    }

    #[test]
    fn segments_beyond_the_reach_are_dropped() {
        // A line along y = 0 from x = 0 to 100 in steps of 10, width 2
        // (reach 1 with butt caps and round joins): the area [40, 60] keeps
        // the segments that come within 1 of it, [30, 40] to [60, 70].
        let line: Vec<(f64, f64)> = (0..=10).map(|i| (f64::from(i) * 10.0, 0.0)).collect();
        let style = Stroke::new(2.0).with_join(Join::Round).with_caps(Cap::Butt);
        let area = Rect::new(40.0, -5.0, 60.0, 5.0);
        let piece = cut(&polyline(&line), &style, area).expect("a cut");
        assert_eq!(
            points(&piece),
            [
                (30.0, 0.0),
                (40.0, 0.0),
                (50.0, 0.0),
                (60.0, 0.0),
                (70.0, 0.0)
            ]
        );
        // A miter limit of 4 reaches 4: the same segments here.
        let mitered = Stroke::new(2.0)
            .with_join(Join::Miter)
            .with_miter_limit(4.0);
        let piece = cut(&polyline(&line), &mitered, area).expect("a cut");
        assert_eq!(points(&piece).first(), Some(&(30.0, 0.0)));
        // An area the whole line lies in cuts nothing.
        assert!(cut(&polyline(&line), &style, Rect::new(-5.0, -5.0, 105.0, 5.0)).is_none());
        // One the line misses keeps nothing.
        let away = cut(&polyline(&line), &style, Rect::new(0.0, 50.0, 100.0, 60.0));
        assert_eq!(away.map(|p| p.elements().len()), Some(0));
    }

    #[test]
    fn a_dashed_piece_starts_at_the_phase_the_whole_path_has_there() {
        // Dashes 4 on, 6 off from offset 1 over x = 0..100: the whole line
        // dashes [0, 3], [9, 13], [19, 23], [29, 33], … The piece kept for
        // [40, 60], from x = 30, is dashed from phase 31: [30, 33], [39,
        // 43], …, [69, 70], the same dashes inside the reach.
        let line: Vec<(f64, f64)> = (0..=10).map(|i| (f64::from(i) * 10.0, 0.0)).collect();
        let style = Stroke::new(2.0)
            .with_join(Join::Round)
            .with_caps(Cap::Butt)
            .with_dashes(1.0, [4.0, 6.0]);
        let piece = cut(&polyline(&line), &style, Rect::new(40.0, -5.0, 60.0, 5.0)).expect("a cut");
        let mut dashes: Vec<(f64, f64)> = Vec::new();
        let mut from = 0.0;
        for element in piece.elements() {
            match *element {
                PathEl::MoveTo(p) => from = p.x,
                PathEl::LineTo(p) => {
                    if let Some(last) = dashes
                        .last_mut()
                        .filter(|d| (d.1 - from).abs() < 1e-9 && from > d.0)
                    {
                        last.1 = p.x;
                    } else {
                        dashes.push((from, p.x));
                    }
                    from = p.x;
                }
                PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => {}
            }
        }
        dashes.sort_by(|a, b| a.0.total_cmp(&b.0));
        let round = |d: Vec<(f64, f64)>| -> Vec<(i64, i64)> {
            d.into_iter()
                .map(|(a, b)| ((a * 1e6).round() as i64, (b * 1e6).round() as i64))
                .collect()
        };
        assert_eq!(
            round(dashes),
            round(vec![
                (30.0, 33.0),
                (39.0, 43.0),
                (49.0, 53.0),
                (59.0, 63.0),
                (69.0, 70.0)
            ])
        );
    }
}
