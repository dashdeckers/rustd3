//! Areas and ribbons: held bases, polygons, and ribbons over two channels
//! of a recording thinned together.

use rustd3_core::color::Rgb;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::series::{Clock, Recording, m4};
use rustd3_core::time;
use rustd3_figure::kurbo::{BezPath, PathEl, Point, Size};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{area, ribbon, ribbon_series};
use rustd3_figure::scale::{Scale, ScaleRef};
use rustd3_figure::thin::Thin;
use rustd3_figure::{Figure, Held, Resolved, Target};

use super::{draw, fitted, marked, solid};
use crate::figure::{cache, resolve};

/// The single fill a figure's marks drew, as closed polygons.
fn polygons(list: &List) -> Vec<Vec<Point>> {
    let fills: Vec<&BezPath> = marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Fill { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(fills.len(), 1, "one fill");
    let mut out: Vec<Vec<Point>> = Vec::new();
    for e in fills[0].elements() {
        match e {
            PathEl::MoveTo(p) => out.push(vec![*p]),
            PathEl::LineTo(p) => out.last_mut().expect("a subpath").push(*p),
            PathEl::ClosePath => {}
            PathEl::QuadTo(..) | PathEl::CurveTo(..) => panic!("not a polygon: {e:?}"),
        }
    }
    out
}

/// `a` and `b` within 1e-9.
fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

#[test]
fn ribbons_are_bands_at_a_quarter_opacity_and_areas_opaque() {
    let (mut fig, x, y) = fitted();
    let red = Rgb::hex(0xc44e52);
    let (rows, points) = ([(0.0, 1.0, 2.0), (1.0, 0.5, 3.0)], [(0.0, 1.0), (1.0, 3.0)]);
    fig.panel(0, 0)
        .mark(area(points, x, y))
        .mark(ribbon(rows, x, y))
        .mark(area(points, x, y).color(red).opacity(0.4))
        .mark(ribbon(rows, x, y).color(red.with_opacity(0.5)).opacity(0.5))
        .mark(ribbon(rows, x, y).opacity(1.0).opacity(f64::NAN))
        .mark(area(points, x, y).opacity(2.0))
        .mark(ribbon(rows, x, y).color(red.with_opacity(0.5)));
    let list = draw(&fig);
    let paints: Vec<_> = marked(&list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Fill { paint, .. } = c {
                Some(paint.clone())
            } else {
                None
            }
        })
        .collect();
    let blue = COLORBLIND[0];
    let expected = [
        solid(blue),
        solid(blue.with_opacity(0.25)),
        solid(red.with_opacity(0.4)),
        solid(red.with_opacity(0.25)),
        solid(blue),
        solid(blue),
        // A translucent color keeps its own as a band.
        solid(red.with_opacity(0.5)),
    ];
    assert_eq!(paints, expected);
}

#[test]
fn an_area_is_held_at_its_base() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(area([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 3.15]), "{domain:?}");
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(
        area([(0.0, 1.0), (1.0, 3.0)], x, y)
            .base(-1.0)
            .base(f64::NAN),
    );
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [-1.0, 3.2]), "{domain:?}");
}

#[test]
fn an_area_is_one_polygon_per_run_down_to_its_base() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 4.0]));
    let y = fig.scale(scale::linear().domain([0.0, 4.0]));
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0).mark(
        area(
            [
                (0.0, 1.0),
                (1.0, 3.0),
                (2.0, f64::NAN),
                (3.0, 2.0),
                (4.0, 2.0),
            ],
            x,
            y,
        )
        .base(1.0)
        .fill(color, "b"),
    );
    let list = draw(&fig);
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let p = |a: f64, b: f64| Point::new(a / 4.0 * w, h - b / 4.0 * h);
    assert_eq!(
        polygons(&list),
        [
            vec![p(0.0, 1.0), p(0.0, 1.0), p(1.0, 3.0), p(1.0, 1.0)],
            vec![p(3.0, 1.0), p(3.0, 2.0), p(4.0, 2.0), p(4.0, 1.0)],
        ]
    );
    let Command::Fill { paint, .. } = marked(&list)[0] else {
        panic!("a fill");
    };
    assert_eq!(*paint, solid(COLORBLIND[1]));
}

#[test]
fn an_area_on_a_log_scale_reaches_down_past_the_plot_area() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::log().domain([1.0, 100.0]));
    fig.panel(0, 0).mark(area([(0.0, 10.0), (1.0, 50.0)], x, y));
    let list = draw(&fig);
    let h = resolve(&fig, &Held::default()).plot_areas()[0].height();
    let polygons = polygons(&list);
    assert_eq!(polygons.len(), 1);
    assert_eq!(polygons[0][0].y, 2.0 * h, "the guard band's bottom");
}

#[test]
fn a_ribbon_of_rows_fits_both_edges_and_draws_upper_then_lower() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(ribbon(
        [(0.0, 1.0, 2.0), (1.0, 0.5, 3.0), (2.0, f64::NAN, 1.0)],
        x,
        y,
    ));
    let resolved = resolve(&fig, &Held::default());
    let domain = resolved.scale(y).unwrap().get_domain();
    assert!(close(domain, [0.375, 3.125]), "{domain:?}");
    let list = draw(&fig);
    let polygons = polygons(&list);
    assert_eq!(polygons.len(), 1);
    let ys: Vec<f64> = polygons[0].iter().map(|p| p.y).collect();
    let sy = |v: f64| {
        let h = resolved.plot_areas()[0].height();
        h * (domain[1] - v) / (domain[1] - domain[0])
    };
    let expected = [sy(2.0), sy(3.0), sy(0.5), sy(1.0)];
    assert!(
        ys.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-9),
        "{ys:?} against {expected:?}"
    );
}

#[test]
fn a_ribbon_reaching_zero_holds_it_and_stands_on_the_axis() {
    // A stacked area's first layer: rows from 0.
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(ribbon([(0.0, 0.0, 2.0), (1.0, 0.0, 4.0)], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 4.2]), "{domain:?}");
    // A ribbon whose upper edge reaches 0 from below holds it at the top.
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(ribbon([(0.0, -2.0, 0.0), (1.0, -1.0, -0.5)], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [-2.1, 0.0]), "{domain:?}");
    // Two channels of a recording: the lower one's least value is 0.
    let low: Vec<f64> = (0..1000).map(|i| f64::from(i % 10)).collect();
    let high: Vec<f64> = low.iter().map(|v| v + 2.0).collect();
    let rec = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 1.0,
        },
        [low, high],
    )
    .expect("a valid recording");
    let (lower, upper) = (rec.series(0).expect("low"), rec.series(1).expect("high"));
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(ribbon_series(lower, upper, x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 11.55]), "{domain:?}");
}

/// 200,000 samples at 1 kHz on two channels: a wobbling lower edge and an
/// upper edge that never dips below it, with a gap in the upper edge.
fn band() -> Recording<f64> {
    let lower: Vec<f64> = (0..200_000)
        .map(|i| (f64::from(i) * 0.37).sin() + f64::from(i) * 1e-5)
        .collect();
    let upper: Vec<f64> = lower
        .iter()
        .enumerate()
        .map(|(i, v)| match i {
            120_000..120_500 => f64::NAN,
            _ => v + 0.25 + (i as f64 * 0.11).cos().abs(),
        })
        .collect();
    let clock = Clock::Regular {
        start: 0.0,
        period: 1e-3,
    };
    Recording::new(clock, [lower, upper]).expect("a valid recording")
}

/// `h`'s resolved scale over `range`, as a mark sees it.
fn ranged(resolved: &Resolved, h: ScaleRef<Linear>, range: [f64; 2]) -> Linear {
    let scale = Scale::with_range(resolved.scale(h).expect("a handle of this figure"), range);
    scale
        .as_any()
        .downcast_ref::<Linear>()
        .expect("its type")
        .clone()
}

#[test]
fn a_ribbon_over_two_channels_is_thinned_jointly_and_keeps_lower_below_upper() {
    let rec = band();
    let (lower, upper) = (rec.series(0).expect("lower"), rec.series(1).expect("upper"));
    let size = Size::new(640.0, 480.0);
    for scale in [1.0, 2.0] {
        let target = Target::Screen { scale };
        let mut fig = Figure::new();
        let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
        fig.panel(0, 0).mark(ribbon_series(lower, upper, t, v));
        let mut cache = cache();
        let list = fig.draw_with(&mut cache, size, target, &Held::default());
        assert_eq!(cache.stats().m4_queries, 1);
        let again = fig.draw_with(&mut cache, size, target, &Held::default());
        assert_eq!((cache.stats().m4_queries, &again), (1, &list));

        let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
        let area = resolved.plot_areas()[0];
        let sx = ranged(&resolved, t, [0.0, area.width()]);
        let sy = ranged(&resolved, v, [area.height(), 0.0]);
        let n = (area.width() * scale).round() as u32;
        let edges: Vec<f64> = (0..=n).map(|i| sx.invert(f64::from(i) / scale)).collect();
        let columns = rec.columns(&edges).expect("valid edges");
        let lines: m4::Lines = lower
            .m4_joint(upper, &columns, f64::is_finite)
            .expect("one recording");
        let at = |i: usize, s: usize| {
            let (a, b) = rec.series(s).expect("a channel").get(i).expect("a sample");
            Point::new(sx.apply(a), sy.apply(b))
        };
        let expected: Vec<Vec<Point>> = lines
            .iter()
            .map(|line| {
                let top = line.iter().map(|&i| at(i, 1));
                top.chain(line.iter().rev().map(|&i| at(i, 0))).collect()
            })
            .collect();
        let drawn = polygons(&list);
        assert_eq!(drawn, expected, "at {scale}×");
        assert_eq!(drawn.len(), 2, "the gap splits the ribbon");

        let per_column = 6 * columns.len() + 4;
        let kept: usize = drawn.iter().map(Vec::len).sum();
        assert!(kept <= 2 * per_column, "{kept} vertices");
        for polygon in &drawn {
            let m = polygon.len() / 2;
            for k in 0..m {
                let (high, low) = (polygon[k], polygon[2 * m - 1 - k]);
                assert_eq!(high.x, low.x);
                assert!(high.y <= low.y, "upper below lower at {high:?}, {low:?}");
            }
        }
        for column in columns.edges().windows(2) {
            let samples = column[0]..column[1];
            if samples.start < 120_500 && samples.end > 120_000 {
                continue;
            }
            let raw = |s: usize, top: bool| {
                let series = rec.series(s).expect("a channel");
                let e = series.extremes(samples.clone(), |_| true)?;
                Some(if top { e.max().0 } else { e.min().0 })
            };
            let (Some(high), Some(low)) = (raw(1, true), raw(0, false)) else {
                continue;
            };
            let kept = |i: usize| lines.iter().flatten().any(|&j| j == i);
            assert!(kept(high) && kept(low), "column {samples:?}");
        }
    }
}

#[test]
fn a_ribbon_thin_off_or_of_rows_draws_every_row() {
    let rec = band();
    let (lower, upper) = (rec.series(0).expect("lower"), rec.series(1).expect("upper"));
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(ribbon_series(lower, upper, t, v).thin(Thin::Off));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let vertices: usize = polygons(&list).iter().map(Vec::len).sum();
    assert_eq!(vertices, 2 * (200_000 - 500));
    let rows: Vec<(f64, f64, f64)> = (0..20_000)
        .map(|i| (f64::from(i), 0.0, f64::from(i % 5)))
        .collect();
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(ribbon(rows, x, y));
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let vertices: usize = polygons(&list).iter().map(Vec::len).sum();
    assert_eq!(vertices, 40_000);
    assert_eq!(cache.stats().m4_queries, 0);
}

#[test]
fn a_ribbon_over_a_time_recording_is_thinned_too() {
    let start: Timestamp = "2026-10-28T00:00:00Z".parse().expect("an instant");
    let clock = Clock::Regular {
        start,
        period: SignedDuration::from_millis(10),
    };
    let low: Vec<f64> = (0..50_000).map(|i| (f64::from(i) * 0.3).sin()).collect();
    let high: Vec<f64> = low.iter().map(|v| v + 1.0).collect();
    let rec = Recording::new(clock, [low, high]).expect("a valid recording");
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(time::utc()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(ribbon_series(
        rec.series(0).expect("low"),
        rec.series(1).expect("high"),
        t,
        v,
    ));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(
        &mut cache,
        size,
        Target::Screen { scale: 1.0 },
        &Held::default(),
    );
    assert_eq!(cache.stats().m4_queries, 1);
    let vertices: usize = polygons(&list).iter().map(Vec::len).sum();
    assert!(vertices < 2 * (6 * 640 + 4), "{vertices}");
}

#[test]
fn a_ribbon_over_two_recordings_draws_nothing() {
    let (a, b) = (band(), band());
    let mut fig = Figure::new();
    let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0).mark(ribbon_series(
        a.series(0).expect("lower"),
        b.series(1).expect("upper"),
        t,
        v,
    ));
    assert!(marked(&draw(&fig)).is_empty());
    #[cfg(feature = "fonts")]
    {
        use rustd3_figure::check::{Issue, size};
        use rustd3_figure::mark::Error;
        assert_eq!(
            fig.check(size(640.0, 480.0)),
            [Issue::Mark {
                panel: 0,
                mark: 0,
                error: Error::Recordings {
                    left: a.id(),
                    right: b.id()
                }
            }]
        );
    }
}
