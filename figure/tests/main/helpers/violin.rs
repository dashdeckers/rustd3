//! Violins: their estimate against matplotlib's `violin_stats`, and the
//! chart.

use rustd3_core::scale::{self, Band, Linear};
use rustd3_figure::helpers::stats::boxplot::{self, Whis};
use rustd3_figure::helpers::stats::violin;
use rustd3_figure::helpers::stats::violin::Inner;
use rustd3_figure::kurbo::{BezPath, Rect, Shape};
use rustd3_figure::list::{Command, List};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::stats::Kde;
use rustd3_figure::{Figure, Held};

use super::{X, assert_fixture};
use crate::figure::resolve;
use crate::marks::{draw, marked};

#[test]
fn the_estimate_matches_matplotlibs_violin_stats() {
    let kde = Kde::new(X).expect("values that vary");
    assert_fixture(kde.bandwidth(), "/violin/bandwidth");
    assert!((kde.bandwidth() - 3.229_654 * 15f64.powf(-0.2)).abs() < 1e-6);
    let grid = kde.grid(100, 0.0);
    let coords: Vec<f64> = grid.iter().map(|(v, _)| *v).collect();
    let density: Vec<f64> = grid.iter().map(|(_, d)| *d).collect();
    assert_fixture(&coords, "/violin/coords");
    assert_fixture(&density, "/violin/vals");
    assert_eq!((coords[0], coords[99]), (1.2, 14.0));
    let peak = grid
        .iter()
        .copied()
        .fold((f64::NAN, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    assert_fixture([peak.0, peak.1], "/violin/peak");
    assert!((peak.0 - 4.044_444).abs() < 1e-6 && (peak.1 - 0.138_084).abs() < 1e-6);
    let s = boxplot::summary(&X, Whis::default()).expect("values");
    assert_fixture(s.mean, "/violin/mean");
    assert_fixture(s.median, "/violin/median");
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

/// The fills and strokes of `list`'s marks, with each stroke's width and
/// whether it is dashed.
fn commands(list: &List) -> (Vec<&BezPath>, Vec<(&BezPath, f64, bool)>) {
    let (mut fills, mut strokes) = (Vec::new(), Vec::new());
    for command in marked(list) {
        match command {
            Command::Fill { path, .. } => fills.push(path),
            Command::Stroke { path, style, .. } => {
                strokes.push((path, style.width, !style.dash_pattern.is_empty()));
            }
            Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. }
            | Command::Push { .. }
            | Command::Pop
            | _ => panic!("unexpected command {command:?}"),
        }
    }
    (fills, strokes)
}

/// Whether `a` and `b` are within 1e-9 px.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn the_largest_density_spans_the_slot_and_other_groups_share_its_scale() {
    let (mut fig, x, y) = figure();
    let wide: Vec<f64> = X.iter().map(|v| 1.5 * v).collect();
    fig.panel(0, 0)
        .mark(violin([("a", X.to_vec()), ("b", wide.clone())], x, y).inner(Inner::None));
    let r = area(&fig);
    let (w, h) = (r.width(), r.height());
    let list = draw(&fig);
    let (fills, strokes) = commands(&list);
    assert_eq!((fills.len(), strokes.len()), (1, 1));
    // Group a holds the peak: its body spans band "a" exactly at 4.044444.
    let peak_a = Kde::new(X).expect("varies").grid(100, 0.0);
    let peak_b = Kde::new(wide.iter().copied())
        .expect("varies")
        .grid(100, 0.0);
    let max = |g: &[(f64, f64)]| g.iter().map(|(_, d)| *d).fold(0.0, f64::max);
    assert!(max(&peak_a) > max(&peak_b));
    let bounds = fills[0].bounding_box();
    assert!(near(bounds.x0, 0.0), "{bounds:?}");
    let ratio = max(&peak_b) / max(&peak_a);
    assert!(
        near(bounds.x1, w / 2.0 + w / 4.0 + ratio * w / 4.0),
        "{bounds:?}"
    );
    assert!(near(bounds.y1, h - 1.2 / 16.0 * h), "{bounds:?}");
    let top = wide.iter().copied().fold(f64::MIN, f64::max);
    assert!(near(bounds.y0, h - top / 16.0 * h), "{bounds:?}");
    assert!(
        near(strokes[0].1, 1.0),
        "0.75 pt outlines: {}",
        strokes[0].1
    );
}

#[test]
fn flat_groups_draw_a_line_across_the_slot_and_empty_ones_nothing() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(violin([("a", vec![3.0, 3.0]), ("b", vec![])], x, y));
    let r = area(&fig);
    let (w, h) = (r.width(), r.height());
    let list = draw(&fig);
    let (fills, strokes) = commands(&list);
    assert!(fills.is_empty());
    assert_eq!(strokes.len(), 1);
    let line = strokes[0].0.bounding_box();
    let expected = Rect::new(0.0, h - 3.0 / 16.0 * h, w / 2.0, h - 3.0 / 16.0 * h);
    assert!(near(line.x0, expected.x0) && near(line.x1, expected.x1) && near(line.y0, expected.y0));
}

#[test]
fn inner_box_plots_and_quartiles_draw_in_their_own_strokes() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(violin([("a", X.to_vec())], x, y));
    let list = draw(&fig);
    let (_, strokes) = commands(&list);
    // The outline, then whiskers, box and median.
    let widths: Vec<f64> = strokes.iter().map(|s| s.1).collect();
    let box_width = 4.5 * 1.25 * 4.0 / 3.0;
    let expected = [1.0, box_width / 3.0, box_width, box_width / 5.0];
    assert_eq!(widths.len(), expected.len());
    assert!(
        widths.iter().zip(expected).all(|(a, b)| near(*a, b)),
        "{widths:?}"
    );
    let h = area(&fig).height();
    let whisker = strokes[1].0.bounding_box();
    assert!(near(whisker.y0, h - 9.5 / 16.0 * h) && near(whisker.y1, h - 1.2 / 16.0 * h));

    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(violin([("a", X.to_vec())], x, y).inner(Inner::Quartiles));
    let list = draw(&fig);
    let (_, strokes) = commands(&list);
    assert_eq!(strokes.len(), 4);
    assert!(strokes[1..].iter().all(|s| s.2), "dashed quartiles");
    let median = strokes[2].0.bounding_box();
    assert!(near(median.y0, h - 4.4 / 16.0 * h));
}

#[test]
fn the_bodies_ends_fit_y_and_cut_runs_past_the_data() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a"]));
    let (plain, cut) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(violin([("a", X.to_vec())], x, plain))
        .mark(violin([("a", X.to_vec())], x, cut).cut(2.0).points(50));
    let r = resolve(&fig, &Held::default());
    let [lo, hi] = r.scale(plain).unwrap().get_domain();
    assert!(near(lo, 1.2 - 0.05 * 12.8) && near(hi, 14.0 + 0.05 * 12.8));
    let h = Kde::new(X).expect("varies").bandwidth();
    let (a, b) = (1.2 - 2.0 * h, 14.0 + 2.0 * h);
    let [lo, hi] = r.scale(cut).unwrap().get_domain();
    assert!(near(lo, a - 0.05 * (b - a)) && near(hi, b + 0.05 * (b - a)));
}

#[cfg(feature = "fonts")]
#[test]
fn fig_check_is_empty_for_violins() {
    use rustd3_core::scheme::COLORBLIND;
    use rustd3_figure::axis;
    use rustd3_figure::kurbo::Size;

    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["left", "right"], COLORBLIND));
    fig.panel(0, 0)
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .mark(
            violin([("a", X.to_vec())], x, y)
                .fill(color, "left")
                .dodge(),
        )
        .mark(
            violin([("a", X.to_vec()), ("b", vec![2.0])], x, y)
                .fill(color, "right")
                .dodge(),
        );
    assert!(fig.check(Size::new(640.0, 480.0)).is_empty());
}
