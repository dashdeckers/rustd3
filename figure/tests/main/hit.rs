//! Hit testing: the points marks draw, found by position.

use rustd3_core::scale::{self, Linear};
use rustd3_figure::kurbo::{Point, Size};
use rustd3_figure::mark::{dots, line, rule_y};
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
