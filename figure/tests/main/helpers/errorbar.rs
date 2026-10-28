//! Error bars: bars at their keys, caps only when asked.

use rustd3_core::scale::{self, Band, Linear};
use rustd3_figure::helpers::stats::errorbar;
use rustd3_figure::kurbo::{BezPath, PathEl, Point};
use rustd3_figure::list::Command;
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held};

use crate::figure::resolve;
use crate::marks::{draw, marked};

/// A figure with band keys "a" and "b" (no padding) and y over [0, 10].
fn figure() -> (
    Figure<'static>,
    ScaleRef<Band<&'static str>>,
    ScaleRef<Linear>,
) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    (fig, x, y)
}

/// The segments of the one stroke `fig`'s marks draw, and the plot area's
/// width and height.
fn segments(fig: &Figure<'_>) -> (Vec<[Point; 2]>, f64, f64) {
    let r = resolve(fig, &Held::default()).plot_areas()[0];
    let list = draw(fig);
    let paths: Vec<&BezPath> = marked(&list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(paths.len(), 1);
    let mut out = Vec::new();
    let mut from = Point::ZERO;
    for el in paths[0].elements() {
        match el {
            PathEl::MoveTo(p) => from = *p,
            PathEl::LineTo(p) => out.push([from, *p]),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => {
                panic!("not a segment: {el:?}")
            }
        }
    }
    (out, r.width(), r.height())
}

/// Whether `a` and `b` are within 1e-9 px.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn bars_stand_at_their_keys_without_caps_by_default() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(errorbar(
        [
            ("a", 4.0, errorbar::Err::Sym(1.0)),
            ("b", 6.0, errorbar::Err::Asym(2.0, 0.5)),
        ],
        x,
        y,
    ));
    let (segments, w, h) = segments(&fig);
    let at = |v: f64| h - v / 10.0 * h;
    let expected = [
        [(w / 4.0, at(3.0)), (w / 4.0, at(5.0))],
        [(3.0 * w / 4.0, at(4.0)), (3.0 * w / 4.0, at(6.5))],
    ];
    assert_eq!(segments.len(), 2);
    for (s, e) in segments.iter().zip(expected) {
        assert!(
            near(s[0].x, e[0].0)
                && near(s[0].y, e[0].1)
                && near(s[1].x, e[1].0)
                && near(s[1].y, e[1].1),
            "{segments:?}"
        );
    }
}

#[test]
fn caps_reach_their_points_to_each_side() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(errorbar([("a", 4.0, errorbar::Err::Sym(1.0))], x, y).caps(4.0));
    let (segments, w, h) = segments(&fig);
    assert_eq!(segments.len(), 3);
    let reach = 4.0 * 4.0 / 3.0;
    for (cap, v) in segments[1..].iter().zip([3.0, 5.0]) {
        let y = h - v / 10.0 * h;
        assert!(near(cap[0].x, w / 4.0 - reach) && near(cap[1].x, w / 4.0 + reach));
        assert!(near(cap[0].y, y) && near(cap[1].y, y), "{cap:?}");
    }
}

#[test]
fn bar_ends_fit_y_and_bad_errors_draw_nothing() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(errorbar(
        [
            ("a", 4.0, errorbar::Err::Sym(1.0)),
            ("b", 6.0, errorbar::Err::Asym(2.0, 1.0)),
            ("b", 50.0, errorbar::Err::Sym(-1.0)),
        ],
        x,
        y,
    ));
    let [lo, hi] = resolve(&fig, &Held::default()).scale(y).get_domain();
    assert!(near(lo, 3.0 - 0.2) && near(hi, 7.0 + 0.2), "{lo} {hi}");
    let (segments, _, _) = segments(&fig);
    assert_eq!(segments.len(), 2);
}

#[cfg(feature = "fonts")]
#[test]
fn fig_check_is_empty_for_error_bars_and_reports_keys_outside_the_band() {
    use rustd3_figure::axis;
    use rustd3_figure::check::{Issue, size};
    use rustd3_figure::mark::bar_y;

    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .mark(bar_y([("a", 4.0), ("b", 6.0)], x, y))
        .mark(
            errorbar(
                [
                    ("a", 4.0, errorbar::Err::Sym(1.0)),
                    ("b", 6.0, errorbar::Err::Sym(0.5)),
                ],
                x,
                y,
            )
            .caps(3.0),
        );
    assert!(fig.check(size(640.0, 480.0)).is_empty());
    fig.panel(0, 0)
        .mark(errorbar([("z", 4.0, errorbar::Err::Sym(1.0))], x, y));
    assert_eq!(
        fig.check(size(640.0, 480.0)),
        [Issue::OutsideDomain {
            handle: x.index(),
            count: 1
        }]
    );
}
