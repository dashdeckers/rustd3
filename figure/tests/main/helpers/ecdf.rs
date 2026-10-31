//! ECDF: the steps against matplotlib's `Axes.ecdf`, plain and
//! complementary.

use rustd3_core::scale::{self, Linear};
use rustd3_figure::helpers::stats::ecdf;
use rustd3_figure::kurbo::{BezPath, PathEl, Point};
use rustd3_figure::list::Command;
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held};
use rustd3_testkit::obs::{ToObs, Tolerance};

use super::{X, numbers};
use crate::figure::resolve;
use crate::marks::{draw, marked};

/// A figure with x over [0, 16] and y over [0, 1].
fn figure() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 16.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    (fig, x, y)
}

/// The vertices of the one line `fig` draws, in data units on x over
/// [0, 16] and y over [0, 1].
fn steps(fig: &Figure<'_>) -> Vec<(f64, f64)> {
    let r = resolve(fig, &Held::default()).plot_areas()[0];
    let (w, h) = (r.width(), r.height());
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
    paths[0]
        .elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => None,
        })
        .map(|p: Point| (p.x / w * 16.0, (h - p.y) / h))
        .collect()
}

/// matplotlib's line at `at` drawn as its `drawstyle` draws it, without
/// repeated vertices.
fn expanded(at: &str, post: bool) -> Vec<(f64, f64)> {
    let (xs, ys) = (numbers(&format!("{at}/x")), numbers(&format!("{at}/y")));
    let mut out: Vec<(f64, f64)> = vec![(xs[0], ys[0])];
    for i in 1..xs.len() {
        let corner = if post {
            (xs[i], ys[i - 1])
        } else {
            (xs[i - 1], ys[i])
        };
        for p in [corner, (xs[i], ys[i])] {
            if out.last() != Some(&p) {
                out.push(p);
            }
        }
    }
    out
}

/// Asserts that `ours` and `theirs` agree within the tolerant compare.
fn assert_steps(ours: &[(f64, f64)], theirs: &[(f64, f64)]) {
    let flat = |v: &[(f64, f64)]| v.iter().flat_map(|(a, b)| [*a, *b]).collect::<Vec<f64>>();
    assert!(
        flat(ours)
            .to_obs()
            .approx_eq(&flat(theirs).to_obs(), &Tolerance::at_least(1e-9)),
        "{ours:?}\n{theirs:?}"
    );
}

#[test]
fn the_steps_match_matplotlibs_ecdf() {
    assert_eq!(
        super::fixture("/ecdf/plain/drawstyle"),
        "steps-post".to_obs()
    );
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(ecdf(X, x, y));
    assert_steps(&steps(&fig), &expanded("/ecdf/plain", true));
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(ecdf(X, x, y).complementary(true));
    assert_eq!(
        super::fixture("/ecdf/complementary/drawstyle"),
        "steps-pre".to_obs()
    );
    assert_steps(&steps(&fig), &expanded("/ecdf/complementary", false));
}

#[test]
fn the_spot_rows_hold_as_the_section_writes_them() {
    let (mut fig, x, y) = figure();
    let shuffled = [
        9.5, 1.2, 14.0, 3.1, 2.9, 4.0, 3.8, 2.3, 4.4, 5.1, 4.7, 6.2, 5.6, 7.9, 3.4,
    ];
    fig.panel(0, 0).mark(ecdf(shuffled, x, y));
    let s = steps(&fig);
    let rises: Vec<(f64, f64)> = s.iter().skip(1).step_by(2).copied().collect();
    let expected = [
        (1.2, 0.066_667),
        (2.3, 0.133_333),
        (2.9, 0.2),
        (3.1, 0.266_667),
    ];
    for ((a, b), (c, d)) in rises.iter().zip(expected) {
        assert!((a - c).abs() < 1e-9 && (b - d).abs() < 1e-6, "{rises:?}");
    }
    let last = rises.last().expect("steps");
    assert!((last.0 - 14.0).abs() < 1e-9 && (last.1 - 1.0).abs() < 1e-9);
}

#[test]
fn y_holds_zero_and_one_and_x_spans_the_values() {
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(ecdf([f64::NAN, 3.0, 1.0, 2.0], x, y));
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.scale(y).unwrap().get_domain(), [0.0, 1.0]);
    let [lo, hi] = r.scale(x).unwrap().get_domain();
    assert!((lo - 0.9).abs() < 1e-12 && (hi - 3.1).abs() < 1e-12);
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(ecdf([f64::NAN], x, y));
    assert!(marked(&draw(&fig)).is_empty());
}

#[cfg(feature = "fonts")]
#[test]
fn fig_check_reports_a_stroke_key_the_palette_lacks() {
    use rustd3_core::scheme::COLORBLIND;
    use rustd3_figure::check::{Issue, size};

    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["a"], COLORBLIND));
    fig.panel(0, 0)
        .mark(ecdf(X, x, y).stroke(color, "a"))
        .mark(ecdf(X, x, y).stroke(color, "b"));
    assert_eq!(
        fig.check(size(640.0, 480.0)),
        [Issue::UnknownKey { panel: 0, mark: 1 }]
    );
}
