//! Box plots: `boxplot::summary` against matplotlib's
//! `cbook.boxplot_stats`, and the chart drawn from it.

use rustd3_core::array;
use rustd3_core::scale::{self, Band, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::helpers::stats::boxplot;
use rustd3_figure::helpers::stats::boxplot::{Summary, Whis};
use rustd3_figure::kurbo::{BezPath, PathEl, Point, Rect, Shape};
use rustd3_figure::list::{Command, List};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held};

use super::{X, assert_fixture};
use crate::figure::resolve;
use crate::marks::{draw, marked};

/// Asserts that `s` is the fixture's summary at `at` (`/boxplot/iqr`).
fn assert_summary(s: &Summary, at: &str) {
    let fields = [
        ("q1", s.q1),
        ("median", s.median),
        ("q3", s.q3),
        ("iqr", s.q3 - s.q1),
        ("mean", s.mean),
        ("whisker_low", s.whisker_low),
        ("whisker_high", s.whisker_high),
        ("notch_low", s.notch_low),
        ("notch_high", s.notch_high),
    ];
    for (name, value) in fields {
        assert_fixture(value, &format!("{at}/{name}"));
    }
    assert_fixture(&s.fliers, &format!("{at}/fliers"));
}

#[test]
fn summaries_match_matplotlibs_boxplot_stats() {
    let tukey = boxplot::summary(&X, Whis::Iqr(1.5)).expect("finite values");
    assert_summary(&tukey, "/boxplot/iqr");
    assert_eq!(
        tukey,
        boxplot::summary(&X, Whis::default()).expect("values")
    );
    let percentiles = boxplot::summary(&X, Whis::Percentiles(5.0, 95.0)).expect("values");
    assert_summary(&percentiles, "/boxplot/percentiles");
    assert_eq!(
        (percentiles.whisker_low, percentiles.whisker_high),
        (2.3, 9.5)
    );
    assert_eq!(percentiles.fliers, [1.2, 14.0]);
    let single = boxplot::summary(&[3.0], Whis::default()).expect("one value");
    assert_summary(&single, "/boxplot/single");
}

#[test]
fn the_spot_rows_hold_as_the_section_writes_them() {
    let s = boxplot::summary(&X, Whis::Iqr(1.5)).expect("finite values");
    assert_eq!((s.q1, s.median, s.q3), (3.25, 4.4, 5.9));
    assert!((s.q3 - s.q1 - 2.65).abs() < 1e-12);
    assert!((s.mean - 5.206_667).abs() < 1e-6);
    assert_eq!(
        (s.whisker_low, s.whisker_high, s.fliers.as_slice()),
        (1.2, 9.5, &[14.0][..])
    );
    assert!((s.notch_low - 3.325_764).abs() < 1e-6 && (s.notch_high - 5.474_236).abs() < 1e-6);
    assert!(boxplot::summary(&[], Whis::default()).is_none());
    assert!(boxplot::summary(&[f64::NAN], Whis::default()).is_none());
    let single = boxplot::summary(&[3.0], Whis::default()).expect("one value");
    let every = [
        single.q1,
        single.median,
        single.q3,
        single.mean,
        single.whisker_low,
        single.whisker_high,
        single.notch_low,
        single.notch_high,
    ];
    assert_eq!(every, [3.0; 8]);
    assert!(single.fliers.is_empty());
}

#[test]
fn cores_quantiles_are_numpys() {
    let quartiles = [0.25, 0.5, 0.75].map(|p| array::quantile(X, p).expect("values"));
    assert_eq!(quartiles, [3.25, 4.4, 5.9]);
    assert_fixture(quartiles, "/quantiles/values");
}

/// A figure with band keys "a" and "b" (no padding) and y over [0, 16].
fn figure() -> (
    Figure<'static>,
    ScaleRef<Band<&'static str>>,
    ScaleRef<Linear>,
) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 16.0]));
    (fig, x, y)
}

/// The plot area of `fig`'s only panel.
fn area(fig: &Figure<'_>) -> Rect {
    resolve(fig, &Held::default()).plot_areas()[0]
}

/// The points of every segment of `path`, in order.
fn vertices(path: &BezPath) -> Vec<Point> {
    path.elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::ClosePath => None,
        })
        .collect()
}

/// The fill, stroke and markers commands of `list`'s marks, in order.
fn commands(list: &List) -> (Vec<&BezPath>, Vec<&BezPath>, Vec<&[Point]>) {
    let (mut fills, mut strokes, mut markers) = (Vec::new(), Vec::new(), Vec::new());
    for command in marked(list) {
        match command {
            Command::Fill { path, .. } => fills.push(path),
            Command::Stroke { path, .. } => strokes.push(path),
            Command::Markers { at, .. } => markers.push(at.as_slice()),
            Command::Glyphs(_)
            | Command::Image { .. }
            | Command::Push { .. }
            | Command::Pop
            | _ => panic!("unexpected command {command:?}"),
        }
    }
    (fills, strokes, markers)
}

/// Whether `a` and `b` are within 1e-9 px.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn a_box_plot_draws_its_box_whiskers_and_fliers_where_the_summary_puts_them() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(boxplot([("a", X.to_vec())], x, y));
    let r = area(&fig);
    let (w, h) = (r.width(), r.height());
    let at = |v: f64| h - v / 16.0 * h;
    let list = draw(&fig);
    let (fills, strokes, markers) = commands(&list);
    assert_eq!((fills.len(), strokes.len(), markers.len()), (1, 1, 1));
    // The box: half of band "a" ([0, w/2]) around its centre w/4.
    let bounds = fills[0].bounding_box();
    let expected = Rect::new(w / 8.0, at(5.9), 3.0 * w / 8.0, at(3.25));
    for (a, b) in [
        (bounds.x0, expected.x0),
        (bounds.y0, expected.y0),
        (bounds.x1, expected.x1),
        (bounds.y1, expected.y1),
    ] {
        assert!(near(a, b), "{bounds:?} vs {expected:?}");
    }
    // Lines reach the whisker ends, 1.2 and 9.5.
    let lines = strokes[0].bounding_box();
    assert!(
        near(lines.y0, at(9.5)) && near(lines.y1, at(1.2)),
        "{lines:?}"
    );
    let points = vertices(strokes[0]);
    let median = at(4.4);
    assert!(
        points.windows(2).any(|p| {
            near(p[0].y, median)
                && near(p[1].y, median)
                && near(p[0].x, w / 8.0)
                && near(p[1].x, 3.0 * w / 8.0)
        }),
        "a median line across the box"
    );
    let caps = points
        .windows(2)
        .filter(|p| near(p[0].y, p[1].y) && near((p[1].x - p[0].x).abs(), w / 8.0))
        .count();
    assert_eq!(caps, 2, "two caps half the box's width");
    assert_eq!(markers[0], [Point::new(w / 4.0, at(14.0))]);
}

#[test]
fn whisker_ends_and_fliers_fit_y_and_the_box_holds_no_zero() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a"]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(boxplot([("a", X.to_vec())], x, y));
    let [lo, hi] = resolve(&fig, &Held::default()).scale(y).get_domain();
    assert!(
        near(lo, 1.2 - 0.05 * 12.8) && near(hi, 14.0 + 0.05 * 12.8),
        "{lo} {hi}"
    );
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a"]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(boxplot([("a", X.to_vec())], x, y).whis(Whis::Iqr(10.0)));
    let [lo, hi] = resolve(&fig, &Held::default()).scale(y).get_domain();
    assert!(
        near(lo, 1.2 - 0.05 * 12.8) && near(hi, 14.0 + 0.05 * 12.8),
        "{lo} {hi}"
    );
}

#[test]
fn notched_boxes_narrow_to_half_their_width_at_the_median() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(boxplot([("a", X.to_vec())], x, y).notch(true));
    let r = area(&fig);
    let (w, h) = (r.width(), r.height());
    let at = |v: f64| h - v / 16.0 * h;
    let list = draw(&fig);
    let (fills, _, _) = commands(&list);
    let s = boxplot::summary(&X, Whis::default()).expect("values");
    let outline = vertices(fills[0]);
    let (x0, x1, centre) = (w / 8.0, 3.0 * w / 8.0, w / 4.0);
    let expected = [
        (x0, at(s.q1)),
        (x1, at(s.q1)),
        (x1, at(s.notch_low)),
        (centre + w / 16.0, at(s.median)),
        (x1, at(s.notch_high)),
        (x1, at(s.q3)),
        (x0, at(s.q3)),
        (x0, at(s.notch_high)),
        (centre - w / 16.0, at(s.median)),
        (x0, at(s.notch_low)),
    ];
    assert_eq!(outline.len(), expected.len(), "{outline:?}");
    for (p, (ex, ey)) in outline.iter().zip(expected) {
        assert!(near(p.x, ex) && near(p.y, ey), "{p:?} vs ({ex}, {ey})");
    }
}

#[test]
fn dodged_box_plots_take_their_keys_part_of_the_band() {
    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["left", "right"], COLORBLIND));
    let low: Vec<f64> = X.iter().map(|v| v / 2.0).collect();
    fig.panel(0, 0)
        .mark(boxplot([("b", low)], x, y).fill(color, "left").dodge())
        .mark(
            boxplot([("b", X.to_vec())], x, y)
                .fill(color, "right")
                .dodge(),
        );
    let w = area(&fig).width();
    let list = draw(&fig);
    let (fills, _, _) = commands(&list);
    // Band "b" is [w/2, w]; each key takes a quarter of the panel, its box
    // half of that.
    let [left, right] = [fills[0], fills[1]].map(|f| f.bounding_box());
    assert!(near(left.x0, w / 2.0 + w / 16.0) && near(left.x1, w / 2.0 + 3.0 * w / 16.0));
    assert!(near(right.x0, 3.0 * w / 4.0 + w / 16.0) && near(right.x1, w - w / 16.0));
}

#[test]
fn means_are_marked_and_empty_groups_draw_nothing() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(boxplot([("a", vec![f64::NAN]), ("b", X.to_vec())], x, y).means(true));
    let r = area(&fig);
    let (w, h) = (r.width(), r.height());
    let list = draw(&fig);
    let (fills, _, markers) = commands(&list);
    let s = boxplot::summary(&X, Whis::default()).expect("values");
    assert!(near(fills[0].bounding_box().x0, w / 2.0 + w / 8.0));
    assert_eq!(markers.len(), 2);
    let [mean] = markers[1] else {
        panic!("one mean: {:?}", markers[1]);
    };
    assert!(near(mean.x, 3.0 * w / 4.0) && near(mean.y, h - s.mean / 16.0 * h));
}

#[cfg(feature = "fonts")]
#[test]
fn fig_check_is_empty_for_box_plots_and_reports_what_they_cannot_draw() {
    use rustd3_figure::axis;
    use rustd3_figure::check::{Issue, size};

    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["left"], COLORBLIND));
    fig.panel(0, 0)
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .mark(boxplot([("a", X.to_vec()), ("b", vec![1.0, 2.0])], x, y).notch(true));
    assert!(fig.check(size(640.0, 480.0)).is_empty());
    fig.panel(0, 0)
        .mark(boxplot([("z", X.to_vec())], x, y).fill(color, "right"));
    let issues = fig.check(size(640.0, 480.0));
    assert!(
        issues.contains(&Issue::UnknownKey { panel: 0, mark: 1 }),
        "{issues:?}"
    );
    assert!(
        issues.contains(&Issue::OutsideDomain {
            handle: x.index(),
            count: 1
        }),
        "{issues:?}"
    );
}
