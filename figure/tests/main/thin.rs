//! Thinning at draw time: M4 per column of a recording channel, its memo
//! across draws, and lines drawn as given.

use rustd3_core::scale::{self, Log};
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::kurbo::{BezPath, PathEl, Size};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::line;
use rustd3_figure::scale::{Indexed, Position, Scale};
use rustd3_figure::thin::Thin;
use rustd3_figure::{Figure, Held, Target, axis};

use crate::figure::cache;
use crate::marks::marked;

/// `n` samples at 1 kHz on three channels: a ramp with a wobble, its
/// negation, and a tone around 0 that dips below it.
fn recording(n: u32) -> Recording<f64> {
    let a: Vec<f64> = (0..n)
        .map(|i| f64::from(i) * 1e-3 + (f64::from(i) * 0.37).sin())
        .collect();
    let b: Vec<f64> = a.iter().map(|v| -v).collect();
    let c: Vec<f64> = (0..n).map(|i| (f64::from(i) * 0.001).sin() + 0.5).collect();
    Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 1e-3,
        },
        [a, b, c],
    )
    .expect("a valid recording")
}

/// The stroked paths of a list's marks, in order.
fn paths(list: &List) -> Vec<&BezPath> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

/// The vertices of a polyline path.
fn vertices(path: &BezPath) -> usize {
    path.elements()
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_) | PathEl::LineTo(_)))
        .count()
}

#[test]
fn a_second_draw_of_an_unchanged_figure_runs_no_m4_query_and_no_typst_compile() {
    let rec = recording(200_000);
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    for c in 0..3 {
        fig.panel(c, 0)
            .mark(line(rec.series(c).expect("a channel"), t, v))
            .axis(axis::left(v))
            .axis(axis::bottom(t));
    }
    let mut cache = cache();
    let size = Size::new(800.0, 600.0);
    let held = Held::default();
    let first = fig.draw_with(&mut cache, size, Target::Screen { scale: 1.0 }, &held);
    let before = cache.stats();
    assert_eq!(before.m4_queries, 3);
    let second = fig.draw_with(&mut cache, size, Target::Screen { scale: 1.0 }, &held);
    let after = cache.stats();
    assert_eq!(after.m4_queries, before.m4_queries);
    assert_eq!(after.text.compiles, before.text.compiles);
    assert_eq!(first, second);
}

#[test]
fn an_append_reruns_m4_for_the_lines_of_that_recording() {
    let mut rec = recording(100_000);
    let mut cache = cache();
    let size = Size::new(800.0, 600.0);
    for (round, queries) in [(0, 1), (1, 1), (2, 2)] {
        if round == 2 {
            rec.extend(&[], [[0.0], [0.0], [0.0]]).expect("an append");
        }
        let mut fig = Figure::new();
        let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
        fig.panel(0, 0).mark(line(rec.series(0).expect("a"), t, v));
        fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
        assert_eq!(cache.stats().m4_queries, queries, "round {round}");
    }
}

#[test]
fn line_thin_off_on_a_million_samples_draws_every_sample() {
    let rec = recording(1_000_000);
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("a"), t, v).thin(Thin::Off));
    let mut cache = cache();
    let list = fig.draw_with(
        &mut cache,
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(vertices(paths(&list)[0]), 1_000_000);
    assert_eq!(cache.stats().m4_queries, 0);
}

#[test]
fn owned_points_are_drawn_as_given() {
    let pairs: Vec<(f64, f64)> = (0..20_000)
        .map(|i| (f64::from(i), f64::from(i % 7)))
        .collect();
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(line(pairs, t, v));
    let mut cache = cache();
    let list = fig.draw_with(
        &mut cache,
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(vertices(paths(&list)[0]), 20_000);
    assert_eq!(cache.stats().m4_queries, 0);
}

/// The number of points M4 keeps for channel `c` of `rec` in a panel `w`
/// px wide on `x`, at `per_px` columns per px, by core directly.
fn expected<S: Position<Domain = f64> + Scale>(
    rec: &Recording<f64>,
    c: usize,
    x: &S,
    w: f64,
    per_px: f64,
    valid: impl Fn(f64) -> bool,
) -> usize {
    let ranged = x.with_range([0.0, w]);
    let ranged = ranged.as_any().downcast_ref::<S>().expect("its type");
    let indexed = ranged.indexed().expect("indexed");
    let n = (w * per_px).round() as u32;
    let edges: Vec<f64> = (0..=n)
        .map(|i| Indexed::invert(indexed, f64::from(i) / per_px))
        .collect();
    let columns = rec.columns(&edges).expect("valid edges");
    rec.series(c)
        .expect("a channel")
        .m4(&columns, valid)
        .expect("fresh columns")
        .points()
}

#[test]
fn columns_on_a_log_x_scale_invert_device_pixel_edges() {
    let x: Vec<f64> = (0..100_000)
        .map(|i| 10f64.powf(f64::from(i) * 4e-5))
        .collect();
    let y: Vec<f64> = (0..100_000).map(|i| (f64::from(i) * 0.21).sin()).collect();
    let rec = Recording::new(Clock::Explicit(x), [y]).expect("a valid recording");
    let mut fig = Figure::new();
    let t = fig.fit(scale::log());
    let v = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(rec.series(0).expect("a"), t, v));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let target = Target::Screen { scale: 2.0 };
    let list = fig.draw_with(&mut cache, size, target, &Held::default());
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let w = resolved.plot_areas()[0].width();
    let want = expected::<Log>(&rec, 0, resolved.scale(t), w, 2.0, |_| true);
    assert_eq!(vertices(paths(&list)[0]), want);
    assert!(want < 4 * (2.0 * w).round() as usize + 2, "{want}");
}

#[test]
fn a_channel_on_a_log_and_a_linear_y_is_thinned_for_each() {
    let rec = recording(100_000);
    let size = Size::new(640.0, 480.0);
    let draw = |both: bool| -> Vec<BezPath> {
        let mut fig = Figure::new();
        let t = fig.fit(scale::linear());
        if both {
            let linear = fig.fit(scale::linear());
            fig.panel(0, 0)
                .mark(line(rec.series(2).expect("c"), t, linear));
        }
        let log = fig.fit(scale::log());
        fig.panel(1, 0)
            .mark(line(rec.series(2).expect("c"), t, log));
        let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
        paths(&list).into_iter().cloned().collect()
    };
    let both = draw(true);
    let alone = draw(false);
    assert_eq!(both.len(), 2);
    assert_ne!(vertices(&both[0]), vertices(&both[1]));
    assert_eq!(both[1].elements(), alone[0].elements());
}
