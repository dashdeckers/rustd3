//! Envelope fills, which draw each thinned line in a plot area as one band
//! polygon per run of device columns instead of stroking M4's polyline.
//! A band's edges lie at each column's M4 extremes, grown by half the line
//! width.

use std::collections::BTreeMap;

use rustd3_figure::kurbo::{Affine, BezPath, PathEl, Point, Rect};
use rustd3_figure::list::{Command, List};

/// The fewest device columns a stroke spans to count as thinned.
const COLUMNS: usize = 8;

/// The fewest vertices per column a thinned stroke has. M4 keeps up to
/// four per column of dense data, while a polyline through sparse points
/// has fewer than one.
const PER_COLUMN: usize = 2;

/// A thinned line over one run of contiguous device columns: per column,
/// the least and greatest device y of the vertices M4 kept in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    /// The first column, device px from the cell's left edge.
    first: i64,
    /// Per column from `first`: the least and greatest y, device px.
    spans: Vec<[f64; 2]>,
}

/// One thinned stroke's envelope.
#[derive(Debug, Clone)]
pub struct Outline {
    /// The `Stroke` command it replaces.
    pub command: usize,
    /// Its bands' outlines, device px of the cell.
    pub path: BezPath,
    /// The transform from the stroke's group space into the cell's device
    /// px.
    pub device: Affine,
}

impl Band {
    /// The bands of `path`, a polyline in the space that `device` maps to
    /// the cell's device px. Its vertices are grouped by device column, one
    /// band per run of contiguous columns. Returns `None` unless the path
    /// is a polyline with at least 90 % of its vertices in subpaths that
    /// have at least two vertices per column over at least eight columns.
    pub fn of(path: &BezPath, device: Affine) -> Option<Vec<Band>> {
        let mut columns: BTreeMap<i64, [f64; 2]> = BTreeMap::new();
        // Per subpath: its vertices and the columns it spans.
        let mut runs: Vec<(usize, BTreeMap<i64, ()>)> = Vec::new();
        for element in path.elements() {
            let p = match *element {
                PathEl::MoveTo(p) => {
                    runs.push((0, BTreeMap::new()));
                    device * p
                }
                PathEl::LineTo(p) => device * p,
                PathEl::ClosePath => continue,
                PathEl::QuadTo(..) | PathEl::CurveTo(..) => return None,
            };
            let run = runs.last_mut()?;
            if !p.is_finite() {
                return None;
            }
            let c = p.x.floor() as i64;
            run.0 += 1;
            run.1.insert(c, ());
            let span = columns.entry(c).or_insert([p.y, p.y]);
            span[0] = span[0].min(p.y);
            span[1] = span[1].max(p.y);
        }
        // Thinned: most vertices lie in runs over many columns with
        // several vertices each (axis ticks are many short runs).
        let vertices: usize = runs.iter().map(|r| r.0).sum();
        let dense: usize = runs
            .iter()
            .filter(|(n, cols)| cols.len() >= COLUMNS && *n >= PER_COLUMN * cols.len())
            .map(|r| r.0)
            .sum();
        if vertices == 0 || 10 * dense < 9 * vertices {
            return None;
        }
        let mut bands: Vec<Band> = Vec::new();
        for (c, span) in columns {
            match bands.last_mut() {
                Some(band) if band.first + band.spans.len() as i64 == c => band.spans.push(span),
                Some(_) | None => bands.push(Band {
                    first: c,
                    spans: vec![span],
                }),
            }
        }
        Some(bands)
    }

    /// Appends to `path` the band's outline in device px, grown by `grow`
    /// device px on every side. At each x, the outline takes the least top
    /// and the greatest bottom of the columns within `grow` of it, so a
    /// column's extremes reach as far as a stroke `2 · grow` wide around
    /// them does.
    pub fn outline(&self, grow: f64, path: &mut BezPath) {
        let first = self.first as f64;
        let n = self.spans.len();
        // Where a column's grown interval starts or ends.
        let mut edges: Vec<f64> = (0..n)
            .flat_map(|i| {
                let left = first + i as f64;
                [left - grow, left + 1.0 + grow]
            })
            .collect();
        edges.sort_by(f64::total_cmp);
        edges.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let mut upper: Vec<Point> = Vec::with_capacity(edges.len());
        let mut lower: Vec<Point> = Vec::with_capacity(edges.len());
        for pair in edges.windows(2) {
            let [p, q] = [pair[0], pair[1]];
            let m = (p + q) / 2.0 - first;
            let lo = (m - 1.0 - grow).ceil().max(0.0) as usize;
            let hi = ((m + grow).floor().max(0.0) as usize).min(n - 1);
            if lo > hi {
                continue;
            }
            let within = &self.spans[lo..=hi];
            let top = within.iter().map(|s| s[0]).fold(f64::INFINITY, f64::min) - grow;
            let bottom = within
                .iter()
                .map(|s| s[1])
                .fold(f64::NEG_INFINITY, f64::max)
                + grow;
            level(&mut upper, p, q, top);
            level(&mut lower, p, q, bottom);
        }
        let Some(&start) = upper.first() else {
            return;
        };
        path.move_to(start);
        for &p in upper.iter().skip(1).chain(lower.iter().rev()) {
            path.line_to(p);
        }
        path.close_path();
    }
}

/// The envelopes of the thinned strokes that `list` draws into the
/// plot-area cell `area` (list px) at `k` device px per list px. A thinned
/// stroke is a solid stroke whose path is a polyline with at least two
/// vertices per device column. Each band is grown by half the stroke's
/// width.
pub fn outlines(list: &List, area: Rect, k: f64) -> Vec<Outline> {
    let cell = Affine::scale(k) * Affine::translate((-area.x0, -area.y0));
    let mut groups = vec![Affine::IDENTITY];
    let mut out = Vec::new();
    for (i, command) in list.commands.iter().enumerate() {
        let to_list = *groups
            .last()
            .expect("invariant: the root group is never popped");
        match command {
            Command::Push { transform, .. } => groups.push(to_list * *transform),
            Command::Pop => {
                groups.pop();
            }
            Command::Stroke { path, style, .. } if style.dash_pattern.is_empty() => {
                let device = cell * to_list;
                let Some(bands) = Band::of(path, device) else {
                    continue;
                };
                let grow = style.width / 2.0 * device.as_coeffs()[0].abs();
                let mut outline = BezPath::new();
                for band in &bands {
                    band.outline(grow, &mut outline);
                }
                out.push(Outline {
                    command: i,
                    path: outline,
                    device,
                });
            }
            Command::Fill { .. }
            | Command::Stroke { .. }
            | Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. }
            | _ => {}
        }
    }
    out
}

/// `list` with each thinned stroke of the plot-area cell `area` (list px)
/// at `k` replaced by a fill of its envelope ([`outlines`]) in its paint.
/// Returns `None` when the cell has no thinned stroke.
pub fn envelope(list: &List, area: Rect, k: f64) -> Option<List> {
    let outlines = outlines(list, area, k);
    if outlines.is_empty() {
        return None;
    }
    let mut out = list.clone();
    for outline in outlines {
        let command = &mut out.commands[outline.command];
        if let Command::Stroke { paint, .. } = command {
            let mut path = outline.path;
            path.apply_affine(outline.device.inverse());
            *command = Command::Fill {
                path,
                paint: paint.clone(),
            };
        }
    }
    Some(out)
}

/// Extends `chain` along y = `y` from x = `p` to `q`, merging with the
/// last level when it is the same.
fn level(chain: &mut Vec<Point>, p: f64, q: f64, y: f64) {
    match chain.last_mut() {
        Some(last) if last.y == y => last.x = q,
        Some(_) | None => {
            chain.push(Point::new(p, y));
            chain.push(Point::new(q, y));
        }
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;
    use rustd3_core::series::{Clock, Recording};
    use rustd3_figure::cache::Cache;
    use rustd3_figure::kurbo::{Affine, BezPath, Point, Shape, Size};
    use rustd3_figure::list::Command;
    use rustd3_figure::mark::line;
    use rustd3_figure::{Figure, Held, Target};

    use super::{Band, envelope, outlines};

    #[test]
    fn a_band_groups_vertices_by_column_and_splits_at_gaps() {
        let mut path = BezPath::new();
        path.move_to((0.2, 5.0));
        for (x, y) in [(0.5, 1.0), (0.9, 3.0), (1.1, 7.0), (1.6, 2.0), (3.5, 4.0)] {
            path.line_to((x, y));
        }
        for i in 0..40 {
            path.line_to((3.5 + f64::from(i) * 0.25, 4.0));
        }
        let bands = Band::of(&path, Affine::IDENTITY).expect("thinned");
        assert_eq!(bands.len(), 2, "column 2 holds no vertex");
        assert_eq!(bands[0].first, 0);
        assert_eq!(bands[0].spans, [[1.0, 5.0], [2.0, 7.0]]);
        assert_eq!(bands[1].first, 3);
        // A sparse polyline is not thinned, nor are many short ticks.
        let mut sparse = BezPath::new();
        sparse.move_to((0.0, 0.0));
        sparse.line_to((100.0, 1.0));
        assert!(Band::of(&sparse, Affine::IDENTITY).is_none());
        let mut ticks = BezPath::new();
        for i in 0..50 {
            ticks.move_to((f64::from(i) * 7.0, 0.0));
            ticks.line_to((f64::from(i) * 7.0, 4.0));
        }
        assert!(Band::of(&ticks, Affine::IDENTITY).is_none());
    }

    #[test]
    fn an_outline_grows_each_column_by_the_half_width_on_every_side() {
        let band = Band {
            first: 10,
            spans: vec![[4.0, 6.0], [0.0, 2.0], [4.0, 6.0]],
        };
        let mut path = BezPath::new();
        band.outline(1.0, &mut path);
        // The middle column's top reaches one px past it on each side.
        assert!(path.contains(Point::new(10.1, -0.5)));
        assert!(path.contains(Point::new(12.9, -0.5)));
        assert!(!path.contains(Point::new(9.9, -0.5)));
        assert!(!path.contains(Point::new(11.5, -1.5)));
        // The outer columns reach one px past the run and below.
        assert!(path.contains(Point::new(9.1, 6.5)));
        assert!(path.contains(Point::new(13.9, 6.5)));
        assert!(!path.contains(Point::new(8.9, 5.0)));
        assert!(!path.contains(Point::new(11.5, 7.5)));
    }

    /// The envelope of a dense thinned line covers, in every device
    /// column, the M4 extremes core's query gives for the same columns,
    /// grown by half the line width.
    #[test]
    fn envelope_covers_every_column_extreme() {
        let n = 200_000;
        let values: Vec<f64> = (0..n)
            .map(|i| {
                let t = f64::from(i) * 1e-3;
                (t * 0.9).sin() + 0.4 * (t * 37.0).sin() + 0.2 * (f64::from(i % 7) - 3.0) / 3.0
            })
            .collect();
        let clock = Clock::Regular {
            start: 0.0,
            period: 1e-3,
        };
        let rec = Recording::new(clock, [values]).expect("a recording");
        let series = rec.series(0).expect("channel 0");
        let (k, width) = (2.0, 2.0);
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 200.0]));
        let y = fig.scale(scale::linear().domain([-2.0, 2.0]));
        fig.panel(0, 0).mark(line(series, x, y).width(width));
        let size = Size::new(480.0, 260.0);
        let target = Target::Screen { scale: k };
        let mut cache = Cache::default();
        let list = fig.draw_with(&mut cache, size, target, &Held::default());
        let area = fig
            .resolve(size, &mut cache, &Held::default(), target)
            .get_plot_areas()[0];
        let found = outlines(&list, area, k);
        assert_eq!(found.len(), 1, "one thinned line");
        let band = &found[0].path;
        let sx = scale::linear()
            .domain([0.0, 200.0])
            .range([0.0, area.width()]);
        let sy = scale::linear()
            .domain([-2.0, 2.0])
            .range([area.height(), 0.0]);
        let columns = (area.width() * k).round() as usize;
        let edges: Vec<f64> = (0..=columns).map(|c| sx.invert(c as f64 / k)).collect();
        let samples = rec.columns(&edges).expect("columns");
        let grow = width / 2.0 * k;
        for c in 0..columns {
            let range = samples.get_edges()[c]..samples.get_edges()[c + 1];
            let extremes = series.extremes(range, |_| true).expect("samples");
            let top = sy.apply(extremes.get_max().1) * k;
            let bottom = sy.apply(extremes.get_min().1) * k;
            let at = c as f64 + 0.5;
            assert!(
                band.contains(Point::new(at, top - grow + 0.01)),
                "column {c}: top {top}"
            );
            assert!(
                band.contains(Point::new(at, bottom + grow - 0.01)),
                "column {c}: bottom {bottom}"
            );
        }
        // The list's stroke becomes a fill in its paint.
        let filled = envelope(&list, area, k).expect("a thinned line");
        let (Command::Stroke { paint, .. }, Command::Fill { paint: fill, .. }) = (
            &list.commands[found[0].command],
            &filled.commands[found[0].command],
        ) else {
            panic!("a stroke replaced by a fill");
        };
        assert_eq!(paint, fill);
    }
}
