//! Dodging and jitter: dots and lines in their palette key's sub-band,
//! dots spread across it by a fixed hash.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{PathEl, Point, Size};
use rustd3_figure::list::Command;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Figure, Held, Target};

use super::{draw, marked};
use crate::figure::{cache, resolve};

/// The points of the markers commands a figure's marks drew, in order.
fn points(fig: &Figure<'_>) -> Vec<Vec<Point>> {
    marked(&draw(fig))
        .into_iter()
        .filter_map(|c| {
            if let Command::Markers { at, .. } = c {
                Some(at.clone())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn a_dodged_line_runs_through_the_centres_of_its_keys_sub_bands() {
    let build = || {
        let mut fig = Figure::new();
        let x = fig.scale(scale::band(0..4).padding(0.1));
        let y = fig.scale(scale::linear().domain([0.0, 10.0]));
        let color = fig.scale(scale::ordinal(["p", "q", "r"], COLORBLIND));
        fig.panel(0, 0).mark(
            line([(0, 1.0), (1, 4.0), (2, 2.0), (3, 8.0)], x, y)
                .stroke(color, "r")
                .dodge(),
        );
        fig
    };
    let fig = build();
    let mut cache = cache();
    let probe = Size::new(640.0, 480.0);
    let area = fig
        .resolve(probe, &mut cache, &Held::default(), Target::Vector)
        .get_plot_areas()[0];
    let size = Size::new(300.0 + probe.width - area.width(), 480.0);
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let area = fig
        .resolve(size, &mut cache, &Held::default(), Target::Vector)
        .get_plot_areas()[0];
    assert!((area.width() - 300.0).abs() < 1e-9, "{area:?}");
    let Command::Stroke { path, .. } = marked(&list)[0] else {
        panic!("a stroke");
    };
    let xs: Vec<f64> = path
        .elements()
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(p.x),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => None,
        })
        .collect();
    let step = 300.0 / 4.1;
    let bandwidth = 0.9 * step;
    for (k, x) in xs.iter().enumerate() {
        let third = 0.1 * step + k as f64 * step + 2.5 * bandwidth / 3.0;
        assert!((x - third).abs() < 1e-9, "vertex {k} at {x}, not {third}");
    }
    assert_eq!(xs.len(), 4);
}

#[test]
fn dodged_dots_take_their_slot_and_jitter_spreads_them_across_it() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let color = fig.scale(scale::ordinal(["p", "q"], COLORBLIND));
    let data: Vec<(&str, f64)> = (0..400).map(|i| ("b", f64::from(i % 10))).collect();
    fig.panel(0, 0)
        .mark(
            dots(data.clone(), x, y)
                .fill(color, "q")
                .dodge()
                .jitter(0.8),
        )
        .mark(dots(data.clone(), x, y).fill(color, "q").dodge())
        .mark(
            dots(data, x, y)
                .fill(color, "q")
                .jitter(2.0)
                .jitter(f64::NAN),
        );
    let w = resolve(&fig, &Held::default()).get_plot_areas()[0].width();
    let drawn = points(&fig);
    let slot = (0.75 * w, w);
    let (lo, hi) = (slot.0 + 0.1 * (w / 4.0), slot.0 + 0.9 * (w / 4.0));
    assert!(
        drawn[0]
            .iter()
            .all(|p| (lo - 1e-9..=hi + 1e-9).contains(&p.x)),
        "jittered over 80 % of the slot"
    );
    assert!((drawn[0][0].x - lo).abs() < 1e-9, "index 0 hashes to 0");
    let spread = drawn[0].iter().map(|p| p.x).fold(f64::NAN, f64::max)
        - drawn[0].iter().map(|p| p.x).fold(f64::NAN, f64::min);
    assert!(spread > 0.75 * (hi - lo), "{spread}");
    assert_eq!(drawn[0], points(&fig)[0], "the same on every draw");
    assert!(
        drawn[1]
            .iter()
            .all(|p| (p.x - (slot.0 + w / 8.0)).abs() < 1e-9)
    );
    assert!(drawn[2].iter().all(|p| (0.5 * w..=w).contains(&p.x)));
    assert!(drawn[2].iter().any(|p| p.x < 0.5 * w + 0.05 * w));
}

#[test]
fn dots_on_a_band_y_dodge_and_jitter_along_y() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::band(["a", "b"]));
    let color = fig.scale(scale::ordinal(["p", "q"], COLORBLIND));
    fig.panel(0, 0).mark(
        dots([(5.0, "a"), (6.0, "a")], x, y)
            .fill(color, "p")
            .dodge(),
    );
    let h = resolve(&fig, &Held::default()).get_plot_areas()[0].height();
    let drawn = points(&fig);
    assert!(
        drawn[0]
            .iter()
            .all(|p| (p.y - (0.5 * h + h / 8.0)).abs() < 1e-9),
        "{:?} in a band at the bottom half",
        drawn[0]
    );
}

#[test]
fn dodge_without_a_palette_key_keeps_the_whole_band() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0).mark(dots([("a", 5.0)], x, y).dodge());
    let w = resolve(&fig, &Held::default()).get_plot_areas()[0].width();
    assert!((points(&fig)[0][0].x - 0.25 * w).abs() < 1e-9);
}

#[cfg(feature = "fonts")]
#[test]
fn hover_finds_dodged_dots_where_they_are_drawn() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let color = fig.scale(scale::ordinal(["p", "q"], COLORBLIND));
    fig.panel(0, 0)
        .mark(dots([("a", 5.0)], x, y).fill(color, "p").dodge());
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let resolved = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let area = resolved.get_plot_areas()[0];
    let at = Point::new(area.x0 + area.width() / 8.0, area.y0 + area.height() / 2.0);
    let hit = fig
        .hits(&resolved, &mut cache)
        .nearest(at, 1.0)
        .expect("the dot");
    assert!(hit.distance < 1e-9, "{hit:?}");
}
