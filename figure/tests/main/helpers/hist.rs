//! Histogram options: `hist::table` against matplotlib's `hist` and numpy's
//! bins, `hist::stacked`, and the `hist` chart.

use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::helpers::stats::hist;
use rustd3_figure::helpers::stats::hist::{Bins, Table};
use rustd3_figure::kurbo::{Rect, Shape};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::histogram;
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held};

use super::{X, assert_fixture, numbers};
use crate::figure::resolve;
use crate::marks::{draw, marked, role};

/// Asserts that `t` is the fixture's table at `at` (`/hist/count10`).
fn assert_table(t: &Table, at: &str) {
    assert_fixture(&t.edges, &format!("{at}/edges"));
    assert_fixture(&t.counts, &format!("{at}/counts"));
}

/// The fixture's spans `[y0, y1]` per bin at `pointer`.
fn spans(pointer: &str) -> Vec<[f64; 2]> {
    (0..3)
        .map(|i| {
            let span = numbers(&format!("{pointer}/{i}"));
            [span[0], span[1]]
        })
        .collect()
}

/// `hist::table` of X.
fn table(bins: Bins, density: bool, cumulative: bool) -> Table {
    hist::table(&X, None, &bins, density, cumulative).expect("a table")
}

#[test]
fn tables_match_matplotlibs_hist() {
    assert_table(&table(Bins::Count(10), false, false), "/hist/count10");
    assert_table(
        &table(Bins::Count(10), true, false),
        "/hist/count10_density",
    );
    assert_table(&table(Bins::Auto, false, false), "/hist/auto");
    assert_table(&table(Bins::Count(5), false, false), "/hist/count5");
    assert_table(
        &table(Bins::Count(5), false, true),
        "/hist/count5_cumulative",
    );
    assert_table(
        &table(Bins::Count(5), true, true),
        "/hist/count5_cumulative_density",
    );
    let weights = numbers("/hist/count5_weights/weights");
    let weighted = hist::table(&X, Some(&weights), &Bins::Count(5), false, false).expect("a table");
    assert_table(&weighted, "/hist/count5_weights");
    let flat = numbers("/hist/auto_flat/values");
    let auto = hist::table(&flat, None, &Bins::Auto, false, false).expect("a table");
    assert_table(&auto, "/hist/auto_flat");
    assert_table(&table(Bins::Nice(5), false, false), "/hist/nice");
}

#[test]
fn the_spot_rows_hold_as_the_section_writes_them() {
    let ten = table(Bins::Count(10), false, false);
    assert_eq!(
        ten.counts,
        [2.0, 3.0, 4.0, 3.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]
    );
    assert_eq!((ten.edges[0], ten.edges[10]), (1.2, 14.0));
    let auto = table(Bins::Auto, false, false);
    assert_eq!(
        auto.counts.len(),
        6,
        "Freedman–Diaconis wins over Sturges' 5"
    );
    assert!((auto.edges[1] - 3.333_333).abs() < 1e-6);
    let five = table(Bins::Count(5), false, false);
    assert_eq!(five.counts, [5.0, 7.0, 1.0, 1.0, 1.0]);
    assert_eq!(
        table(Bins::Count(5), false, true).counts,
        [5.0, 12.0, 13.0, 14.0, 15.0]
    );
    let weights: Vec<f64> = (1..=15).map(f64::from).collect();
    let weighted = hist::table(&X, Some(&weights), &Bins::Count(5), false, false);
    assert_eq!(
        weighted.map(|t| t.counts),
        Ok(vec![15.0, 63.0, 13.0, 14.0, 15.0])
    );
    let nice = table(Bins::Nice(5), false, false);
    assert_eq!(nice.counts, [1.0, 5.0, 5.0, 2.0, 1.0, 0.0, 0.0, 1.0]);
    assert_eq!(
        nice.edges,
        (0..=8).map(|i| 2.0 * f64::from(i)).collect::<Vec<_>>()
    );
    let bad = hist::table(
        &X,
        None,
        &Bins::Edges(vec![0.0, 1.0, 1.0, 2.0]),
        false,
        false,
    );
    assert_eq!(
        bad,
        Err(hist::Error::Edge {
            index: 2,
            value: 1.0
        })
    );
}

#[test]
fn densities_have_unit_area_and_values_outside_edges_are_dropped() {
    let t = table(Bins::Auto, true, false);
    let area: f64 = t
        .edges
        .windows(2)
        .zip(&t.counts)
        .map(|(e, c)| (e[1] - e[0]) * c)
        .sum();
    assert!((area - 1.0).abs() < 1e-12);
    let edged = hist::table(&X, None, &Bins::Edges(vec![2.0, 4.0, 6.0]), false, false);
    assert_eq!(edged.map(|t| t.counts), Ok(vec![5.0, 5.0]));
    let cumulative = table(Bins::Count(5), true, true);
    assert!((cumulative.counts[4] - 1.0).abs() < 1e-12);
}

/// A figure with x over [0.5, 3.5] and y over [0, 4].
fn figure() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.5, 3.5]));
    let y = fig.scale(scale::linear().domain([0.0, 4.0]));
    (fig, x, y)
}

/// Each mark part's filled rectangles in data units, on `fig`'s x over
/// [0.5, 3.5] and y over [0, 4].
fn rects(fig: &Figure<'_>, list: &List) -> Vec<Vec<([f64; 2], [f64; 2])>> {
    let r = resolve(fig, &Held::default()).get_plot_areas()[0];
    let (w, h) = (r.width(), r.height());
    let data = |b: Rect| {
        let x = |px: f64| 0.5 + px / w * 3.0;
        let y = |px: f64| (h - px) / h * 4.0;
        ([x(b.x0), x(b.x1)], [y(b.y1), y(b.y0)])
    };
    list.parts
        .iter()
        .filter(|p| role(p.id) == 6)
        .map(|p| {
            list.commands[p.commands.clone()]
                .iter()
                .filter_map(|c| {
                    if let Command::Fill { path, .. } = c {
                        Some(data(path.bounding_box()))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

/// Whether `a` and `b` are within 1e-9.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn stacked_groups_are_one_rect_mark_each_over_shared_edges() {
    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    let groups = [
        ("a", numbers("/stacked/groups/a")),
        ("b", numbers("/stacked/groups/b")),
    ];
    let edges = numbers("/stacked/counts/edges");
    assert_eq!(edges, [0.5, 1.5, 2.5, 3.5]);
    hist::stacked(groups, x, y, color)
        .bins(Bins::Edges(edges))
        .add_to(fig.panel(0, 0));
    let list = draw(&fig);
    let marks = rects(&fig, &list);
    assert_eq!(marks.len(), 2, "two rect marks");
    // Bins with no height draw nothing: b's first, [1, 1].
    let expected = |group: &str| -> Vec<([f64; 2], [f64; 2])> {
        let spans = spans(&format!("/stacked/counts/{group}"));
        (0..3)
            .map(|i| ([0.5 + i as f64, 1.5 + i as f64], spans[i]))
            .filter(|(_, [y0, y1])| y1 > y0)
            .collect()
    };
    for (got, want) in marks.iter().zip([expected("a"), expected("b")]) {
        assert_eq!(got.len(), want.len());
        for ((gx, gy), (wx, wy)) in got.iter().zip(&want) {
            assert!(
                near(gx[0], wx[0])
                    && near(gx[1], wx[1])
                    && near(gy[0], wy[0])
                    && near(gy[1], wy[1]),
                "{got:?} vs {want:?}"
            );
        }
    }
    assert_eq!(
        expected("b"),
        [([1.5, 2.5], [2.0, 3.0]), ([2.5, 3.5], [1.0, 3.0])]
    );
}

#[test]
fn stacked_densities_divide_by_every_groups_count() {
    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    let groups = [
        ("a", numbers("/stacked/groups/a")),
        ("b", numbers("/stacked/groups/b")),
    ];
    hist::stacked(groups, x, y, color)
        .bins(Bins::Edges(vec![0.5, 1.5, 2.5, 3.5]))
        .density(true)
        .add_to(fig.panel(0, 0));
    let list = draw(&fig);
    let marks = rects(&fig, &list);
    let tops: Vec<f64> = marks.iter().flatten().map(|(_, [_, y1])| *y1).collect();
    let (a, b) = (spans("/stacked/density/a"), spans("/stacked/density/b"));
    // a's three tops, then b's two non-empty ones.
    let expected = [a[0][1], a[1][1], a[2][1], b[1][1], b[2][1]];
    assert_eq!(tops.len(), expected.len());
    assert!(
        tops.iter().zip(expected).all(|(t, e)| near(*t, e)),
        "{tops:?}"
    );
}

#[test]
fn a_hist_holds_its_base_and_draws_bars_or_its_outline() {
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(hist(X, x, y).bins(Bins::Count(5)))
        .mark(hist(X, x, y).bins(Bins::Count(5)).step(true));
    let r = resolve(&fig, &Held::default());
    let [lo, hi] = r.get_scale(y).unwrap().get_domain();
    assert_eq!(lo, 0.0, "held at 0");
    assert!(near(hi, 7.0 * 1.05));
    let list = draw(&fig);
    let fills = marked(&list)
        .iter()
        .filter(|c| matches!(c, Command::Fill { .. }))
        .count();
    let strokes: Vec<_> = marked(&list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    assert_eq!((fills, strokes.len()), (5, 1));
    // Up from 0, along five tops, down to 0: 12 vertices.
    assert_eq!(strokes[0].elements().len(), 12);
}

#[test]
fn a_hist_by_default_bins_as_histogram_does() {
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(hist(X, x, y));
    let ours = draw(&fig);
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(histogram(X, x, y));
    let theirs = draw(&fig);
    assert_eq!(marked(&ours), marked(&theirs));
}

#[cfg(feature = "fonts")]
#[test]
fn fig_check_reports_tables_that_cannot_be_made() {
    use rustd3_figure::check::Issue;
    use rustd3_figure::kurbo::Size;
    use rustd3_figure::mark;

    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(hist(X, x, y).bins(Bins::Edges(vec![0.0, f64::INFINITY])))
        .mark(hist(X, x, y).weights([1.0, 2.0]))
        .mark(hist(X, x, y).density(true));
    let issues = fig.check(Size::new(640.0, 480.0));
    let errors: Vec<_> = issues
        .iter()
        .filter_map(|i| {
            if let Issue::Mark { mark, error, .. } = i {
                Some((*mark, error.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        errors,
        [
            (
                0,
                mark::Error::Edge {
                    index: 1,
                    value: f64::INFINITY
                }
            ),
            (
                1,
                mark::Error::Weights {
                    values: 15,
                    weights: 2
                }
            ),
        ]
    );
    let (mut fig, x, y) = figure();
    let color = fig.scale(scale::ordinal(["a"], COLORBLIND));
    hist::stacked([("a", X.to_vec())], x, y, color)
        .bins(Bins::Edges(vec![3.0, 1.0]))
        .add_to(fig.panel(0, 0));
    fig.panel(0, 0).mark(hist(X, x, y).fill(color, "a"));
    let issues = fig.check(Size::new(640.0, 480.0));
    assert!(
        matches!(
            issues[..],
            [Issue::Mark {
                mark: 0,
                error: mark::Error::Edge { index: 1, .. },
                ..
            }]
        ),
        "{issues:?}"
    );
}
