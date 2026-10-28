//! Hit testing: the points marks draw, found by position.

use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::{self, COLORBLIND};
use rustd3_figure::kurbo::{Point, Size};
use rustd3_figure::mark::{
    Bins, Grid, area, bar_x, bar_y, bar_y_spans, density_y, dots, histogram, image, line, rect,
    ribbon, rule_y,
};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held, Target};

use crate::figure::cache;
use crate::trace;

/// A one-panel figure whose scales map values to figure px at 640 × 480
/// (the plot area fills the padded canvas: no axes), and its handles.
fn identity() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let pad = 0.8 * 44.0 / 3.0;
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([pad, 640.0 - pad]));
    let y = fig.scale(scale::linear().domain([480.0 - pad, pad]));
    fig.panel(0, 0);
    (fig, x, y)
}

#[test]
fn the_nearest_dot_within_the_radius_is_found_with_its_index() {
    let (mut fig, x, y) = identity();
    fig.panel(0, 0)
        .mark(dots([(10.0, 10.0), (20.0, 10.0), (200.0, 200.0)], x, y))
        .mark(rule_y(100.0, y));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let index = fig.hits(&r, &mut cache);
    assert_eq!(index.len(), 3, "rules report none");
    let hit = index.nearest(Point::new(12.0, 11.0), 5.0).expect("a dot");
    assert_eq!((hit.panel, hit.mark, hit.index), (0, 0, 0));
    assert!((hit.distance - 5.0_f64.sqrt()).abs() < 1e-9, "{hit:?}");
    assert!((hit.at - Point::new(10.0, 10.0)).hypot() < 1e-9, "{hit:?}");
    assert_eq!(index.nearest(Point::new(100.0, 100.0), 5.0), None);
}

#[test]
fn equally_near_points_go_to_the_first_reported() {
    let (mut fig, x, y) = identity();
    fig.panel(0, 0)
        .mark(dots([(50.0, 50.0)], x, y))
        .mark(dots([(30.0, 30.0), (50.0, 50.0)], x, y));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let hit = fig
        .hits(&r, &mut cache)
        .nearest(Point::new(51.0, 50.0), 2.0)
        .expect("a dot");
    assert_eq!((hit.mark, hit.index), (0, 0));
}

#[test]
fn lines_report_their_points_and_thinned_ones_their_kept_samples_without_a_query() {
    let (mut fig, x, y) = identity();
    fig.panel(0, 0)
        .mark(line([(100.0, 100.0), (300.0, 200.0), (900.0, 300.0)], x, y));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let index = fig.hits(&r, &mut cache);
    // The third point lies outside the plot area.
    assert_eq!(index.len(), 2);
    let hit = index
        .nearest(Point::new(299.0, 201.0), 3.0)
        .expect("a vertex");
    assert_eq!(hit.index, 1);

    let rec = trace::recording(100_000);
    let fig = trace::trace(&rec);
    let mut cache = crate::figure::cache();
    let target = Target::Screen { scale: 1.0 };
    let held = Held::default();
    fig.draw_with(&mut cache, trace::SIZE, target, &held);
    let queries = cache.stats().m4_queries;
    let r = fig.resolve(trace::SIZE, &mut cache, &held, target);
    let index = fig.hits(&r, &mut cache);
    assert_eq!(cache.stats().m4_queries, queries, "read from the memo");
    // M4 keeps at most four samples per column of each line's panel.
    let columns: f64 = r.plot_areas().iter().map(|a| a.width()).sum();
    assert!(index.len() > 1000 && (index.len() as f64) <= 3.0 * 4.0 * columns + 12.0);
}

/// The hit index of `fig` resolved at 640 × 480 for vector output.
fn indexed(fig: &Figure<'_>) -> rustd3_figure::hit::Index {
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    fig.hits(&r, &mut cache)
}

#[test]
fn bars_are_found_at_the_centre_of_their_value_end() {
    let (mut fig, x, y) = identity();
    fig.panel(0, 0)
        .mark(bar_y(
            [(100.0, 200.0), (300.0, 150.0), (400.0, 1000.0)],
            x,
            y,
        ))
        .mark(bar_x([(400.0, 50.0)], x, y))
        .mark(bar_y_spans([(500.0, [300.0, 100.0])], x, y));
    let index = indexed(&fig);
    assert_eq!(index.len(), 4, "the third bar ends outside the plot area");
    for (at, mark, i) in [
        (Point::new(100.0, 200.0), 0, 0),
        (Point::new(300.0, 150.0), 0, 1),
        (Point::new(400.0, 50.0), 1, 0),
        (Point::new(500.0, 100.0), 2, 0),
    ] {
        let hit = index.nearest(at, 1.0).expect("a bar");
        assert_eq!((hit.mark, hit.index), (mark, i), "{at:?}");
        assert!((hit.at - at).hypot() < 1e-9, "{hit:?}");
    }
}

#[test]
fn histogram_bars_are_found_at_the_centre_of_their_top_dodged_or_binned() {
    let mut fig = Figure::new();
    let keys = fig.scale(scale::band(["a", "b"]));
    let numbers = fig.scale(scale::linear().domain([0.0, 4.0]));
    let count = fig.scale(scale::linear().domain([0.0, 4.0]));
    let color = fig.scale(scale::ordinal(["p", "q"], COLORBLIND));
    fig.panel(0, 0).mark(
        histogram(["b", "a", "b"], keys, count)
            .fill(color, "q")
            .dodge(),
    );
    fig.panel(1, 0)
        .mark(histogram([0.5, 1.5, 1.5], numbers, count).bins(Bins::Count(2)));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let index = fig.hits(&r, &mut cache);
    assert_eq!(index.len(), 4);
    let [top, bottom] = [r.plot_areas()[0], r.plot_areas()[1]];
    let place = |area: rustd3_figure::kurbo::Rect, x: f64, y: f64| {
        Point::new(area.x0 + x * area.width(), area.y0 + y * area.height())
    };
    for (at, panel, i) in [
        (place(top, 7.0 / 8.0, 0.5), 0, 0),
        (place(top, 3.0 / 8.0, 0.75), 0, 1),
        (place(bottom, 0.75 / 4.0, 0.75), 1, 0),
        (place(bottom, 1.25 / 4.0, 0.5), 1, 1),
    ] {
        let hit = index.nearest(at, 1.0).expect("a bar");
        assert_eq!((hit.panel, hit.mark, hit.index), (panel, 0, i), "{at:?}");
        assert!((hit.at - at).hypot() < 1e-9, "{hit:?} against {at:?}");
    }
}

#[test]
fn areas_ribbons_densities_rects_and_images_report_no_hits() {
    let (mut fig, x, y) = identity();
    let heat = fig.scale(scale::sequential([0.0, 1.0], scheme::viridis));
    let grid = Grid::new(1, 1, vec![0.5]).expect("one value");
    fig.panel(0, 0)
        .mark(area([(100.0, 100.0), (200.0, 200.0)], x, y))
        .mark(ribbon([(100.0, 100.0, 200.0), (200.0, 150.0, 250.0)], x, y))
        .mark(density_y([100.0, 150.0, 200.0, 260.0], y, x))
        .mark(rect([([100.0, 200.0], [100.0, 200.0])], x, y))
        .mark(image(
            grid.extent([100.0, 300.0], [100.0, 300.0]),
            x,
            y,
            heat,
        ));
    let index = indexed(&fig);
    assert!(index.is_empty(), "{} points", index.len());
}
