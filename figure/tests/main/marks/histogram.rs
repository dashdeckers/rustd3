//! Histograms: keys counted per band or point, numbers binned as d3 and
//! numpy bin them, held at 0, dodged.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{Rect, Shape};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{Bins, histogram};
use rustd3_figure::{Figure, Held};

use super::{draw, fitted, marked};
use crate::figure::resolve;

/// The section's statistical data, n = 15.
const X: [f64; 15] = [
    1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0,
];

/// The rectangles a list's marks fill, in order.
fn filled(list: &List) -> Vec<Rect> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Fill { path, .. } = c {
                Some(path.bounding_box())
            } else {
                None
            }
        })
        .collect()
}

/// Whether two rectangles agree within 1e-9 px.
fn same(a: Rect, b: Rect) -> bool {
    [a.x0 - b.x0, a.y0 - b.y0, a.x1 - b.x1, a.y1 - b.y1]
        .iter()
        .all(|d| d.abs() < 1e-9)
}

#[test]
fn a_histogram_of_numbers_draws_d3s_nice_sturges_bins_held_at_zero() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(histogram(X, x, y));
    let resolved = resolve(&fig, &Held::default());
    let (dx, dy) = (
        resolved.scale(x).unwrap().get_domain(),
        resolved.scale(y).unwrap().get_domain(),
    );
    assert!(
        (dx[0] + 0.8).abs() < 1e-12 && (dx[1] - 16.8).abs() < 1e-12,
        "{dx:?}"
    );
    assert!(dy[0] == 0.0 && (dy[1] - 5.25).abs() < 1e-12, "{dy:?}");
    let area = resolved.plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let px = |a: f64| (a + 0.8) / 17.6 * w;
    let py = |c: f64| h - c / 5.25 * h;
    let bars = [
        (0.0, 1.0),
        (2.0, 5.0),
        (4.0, 5.0),
        (6.0, 2.0),
        (8.0, 1.0),
        (14.0, 1.0),
    ];
    let expected: Vec<Rect> = bars
        .iter()
        .map(|&(x0, c)| Rect::new(px(x0), py(c), px(x0 + 2.0), h))
        .collect();
    let rects = filled(&draw(&fig));
    assert_eq!(rects.len(), expected.len(), "empty bins draw nothing");
    for (got, want) in rects.iter().zip(&expected) {
        assert!(same(*got, *want), "{got:?} against {want:?}");
    }
}

#[test]
fn count_bins_are_numpys_equal_width_bins() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([1.2, 14.0]));
    let y = fig.scale(scale::linear().domain([0.0, 4.0]));
    fig.panel(0, 0)
        .mark(histogram(X, x, y).bins(Bins::Count(10)));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let heights: Vec<f64> = filled(&draw(&fig))
        .iter()
        .map(|r| (r.height() / h * 4.0 * 1e9).round() / 1e9)
        .collect();
    assert_eq!(heights, [2.0, 3.0, 4.0, 3.0, 1.0, 1.0, 1.0]);
    let first = filled(&draw(&fig))[0];
    assert!((first.width() - w / 10.0).abs() < 1e-9);
}

#[test]
fn a_histogram_of_keys_counts_each_band_and_dodges_into_its_slot() {
    let mut fig = Figure::new();
    let speed = fig.scale(scale::band(0..4).padding(0.1));
    let count = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(["p", "q", "r"], COLORBLIND));
    fig.panel(0, 0).mark(
        histogram([1, 3, 3, 1, 3], speed, count)
            .fill(color, "q")
            .dodge(),
    );
    let resolved = resolve(&fig, &Held::default());
    let dy = resolved.scale(count).unwrap().get_domain();
    assert!(dy[0] == 0.0 && (dy[1] - 3.15).abs() < 1e-12, "{dy:?}");
    let area = resolved.plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let step = w / 4.1;
    let bandwidth = 0.9 * step;
    let start = |k: f64| 0.1 * step + k * step + bandwidth / 3.0;
    let py = |c: f64| h - c / dy[1] * h;
    let expected = [
        Rect::new(start(1.0), py(2.0), start(1.0) + bandwidth / 3.0, h),
        Rect::new(start(3.0), py(3.0), start(3.0) + bandwidth / 3.0, h),
    ];
    let rects = filled(&draw(&fig));
    assert_eq!(rects.len(), 2);
    for (got, want) in rects.iter().zip(expected) {
        assert!(same(*got, want), "{got:?} against {want:?}");
    }
}

#[test]
fn a_histogram_of_point_keys_takes_0_8_of_the_gap_between_its_keys() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::point(["a", "b", "c"]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(histogram(["a", "c", "c"], x, y));
    let w = resolve(&fig, &Held::default()).plot_areas()[0].width();
    let rects = filled(&draw(&fig));
    assert_eq!(rects.len(), 2);
    assert!((rects[0].width() - 0.8 * w).abs() < 1e-9, "{rects:?}");
    assert!((rects[1].center().x - w).abs() < 1e-9, "{rects:?}");
}

#[test]
fn dodged_histograms_of_numbers_share_each_bin() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 4.0]));
    let y = fig.scale(scale::linear().domain([0.0, 4.0]));
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(
            histogram([0.5, 1.5, 3.5], x, y)
                .bins(Bins::Count(2))
                .fill(color, "a")
                .dodge(),
        )
        .mark(
            histogram([1.0, 3.0], x, y)
                .bins(Bins::Count(1))
                .fill(color, "b")
                .dodge(),
        );
    let w = resolve(&fig, &Held::default()).plot_areas()[0].width();
    let px = |v: f64| v / 4.0 * w;
    let spans: Vec<[f64; 2]> = filled(&draw(&fig)).iter().map(|r| [r.x0, r.x1]).collect();
    let expected = [[px(0.5), px(1.25)], [px(2.0), px(2.75)], [px(2.0), px(3.0)]];
    assert_eq!(spans.len(), 3);
    for (got, want) in spans.iter().zip(expected) {
        assert!(
            (got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9,
            "{spans:?} against {expected:?}"
        );
    }
}

#[test]
fn a_histogram_asking_for_more_nice_bins_than_ticks_allow_keeps_them_apart() {
    let values: Vec<f64> = (0..5000).map(|i| f64::from(i) * 0.01).collect();
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 50.0]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(histogram(values, x, y).bins(Bins::Nice(1_000_000)));
    let dy = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(dy[1] < 10.0, "one collapsed bin would count 5,000: {dy:?}");
}

#[cfg(feature = "fonts")]
#[test]
fn keys_outside_the_band_are_reported_with_their_count() {
    use rustd3_figure::check::Issue;
    use rustd3_figure::kurbo::Size;
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(histogram(["a", "z", "z", "y"], x, y));
    assert_eq!(
        fig.check(Size::new(640.0, 480.0)),
        [Issue::OutsideDomain {
            handle: x.index(),
            count: 3
        }]
    );
}
