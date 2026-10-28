//! Figure handles of every scale kind: how each fits, resolves, places or
//! paints its values, and ticks.

use rustd3_core::color::Rgb;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::{self, Band, Point};
use rustd3_core::scheme;
use rustd3_core::series::{Clock, Recording};
use rustd3_core::time::{self, Time};
use rustd3_figure::kurbo::{PathEl, Size};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::{Fit, Invert, Kind, Paint, Position, Scale};
use rustd3_figure::{Figure, Held, Resolved, Target, axis};

use crate::figure::{self, cache};

/// The theme's tick-label size, 11 pt at 4/3 px per pt: one em.
const EM: f64 = 44.0 / 3.0;

/// The canvas whose one bare panel has a plot area `width` × `height` px:
/// 0.8 em of padding on each side.
fn canvas(width: f64, height: f64) -> Size {
    Size::new(width + 1.6 * EM, height + 1.6 * EM)
}

/// `fig` resolved for vector output at `size`.
fn resolve(fig: &Figure<'_>, size: Size) -> Resolved {
    fig.resolve(size, &mut cache(), &Held::default(), Target::Vector)
}

/// `fig` drawn for vector output at `size`.
fn draw(fig: &Figure<'_>, size: Size) -> List {
    fig.draw_with(&mut cache(), size, Target::Vector, &Held::default())
}

/// The texts of the glyph runs in `list`, in order.
fn texts(list: &List) -> Vec<String> {
    list.commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.text.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// The points of every `Markers` command in `list`.
fn markers(list: &List) -> Vec<rustd3_figure::kurbo::Point> {
    list.commands
        .iter()
        .flat_map(|c| {
            if let Command::Markers { at, .. } = c {
                at.clone()
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// `a` within `tolerance` of `b`.
fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

/// `scale` over `range`, as a panel of that length resolves it.
fn ranged<S: Kind>(scale: &S, range: [f64; 2]) -> S {
    Scale::with_range(scale, range)
        .as_any()
        .downcast_ref::<S>()
        .expect("with_range keeps the type")
        .clone()
}

#[test]
fn band_handles_keep_their_keys_and_place_bands_in_the_panel() {
    let mut fig = Figure::new();
    // `fit` keeps a band's keys, as `scale` does.
    let band = fig.fit(scale::band(["a", "b", "c"]).padding(0.1));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(dots([("a", 0.5), ("z", 0.5)], band, y));
    let size = canvas(300.0, 200.0);
    let r = resolve(&fig, size);
    let area = r.plot_areas()[0];
    assert!(near(area.width(), 300.0, 1e-9), "{area:?}");
    assert_eq!(r.scale(band).get_domain(), ["a", "b", "c"]);
    assert_eq!(Kind::domain(r.scale(band)), None);
    // d3: a step of 300 / 3.1, half the outer padding's leftover first.
    let b: Band<&str> = ranged(r.scale(band), [0.0, area.width()]);
    let start = b.position(&"a").expect("a key");
    assert!(
        near(start, (300.0 - 2.9 * 300.0 / 3.1) / 2.0, 1e-9),
        "{start}"
    );
    assert!(near(start, 9.677, 1e-3) && near(b.bandwidth(), 87.10, 5e-3));
    assert_eq!(b.position(&"z"), None);
    let at = markers(&draw(&fig, size));
    assert_eq!(at.len(), 1, "\"z\" draws nothing");
    // In the panel's coordinates, at the band's middle.
    assert!(near(at[0].x, start + b.bandwidth() / 2.0, 1e-9), "{at:?}");
    // Ticks are the keys.
    fig.panel(0, 0).axis(axis::bottom(band));
    assert_eq!(texts(&draw(&fig, size)), ["a", "b", "c"]);
}

#[test]
fn point_handles_place_their_keys_without_width() {
    let mut fig = Figure::new();
    let p = fig.fit(scale::point(["a", "b", "c"]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).mark(line([("a", 0.0), ("c", 1.0)], p, y));
    let r = resolve(&fig, canvas(300.0, 200.0));
    assert!(near(r.plot_areas()[0].width(), 300.0, 1e-9));
    let ranged: Point<&str> = ranged(r.scale(p), [0.0, 300.0]);
    let at = ["a", "b", "c"].map(|k| ranged.position(&k));
    assert_eq!(at, [Some(0.0), Some(150.0), Some(300.0)]);
    assert_eq!((ranged.bandwidth(), ranged.position(&"z")), (0.0, None));
    assert_eq!(r.scale(p).get_domain(), ["a", "b", "c"]);
    let mut fig = Figure::new();
    let p = fig.scale(scale::point(["low", "mid", "high"]));
    fig.panel(0, 0).axis(axis::left(p));
    assert_eq!(
        texts(&draw(&fig, canvas(300.0, 200.0))),
        ["low", "mid", "high"]
    );
}

#[cfg(feature = "fonts")]
#[test]
fn keys_outside_a_band_domain_are_reported_once_per_handle() {
    use rustd3_figure::check::{Issue, size};
    use rustd3_figure::mark::rule_x;

    let mut fig = Figure::new();
    let band = fig.scale(scale::band(["a", "b", "c"]).padding(0.1));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(dots([("a", 0.5), ("z", 0.5)], band, y));
    let issues = fig.check(size(640.0, 480.0));
    let one = Issue::OutsideDomain {
        handle: band.index(),
        count: 1,
    };
    assert_eq!(issues, [one]);
    // Two more marks over the same handle add to the same count.
    fig.panel(0, 1)
        .mark(line([("y", 0.0), ("z", 1.0), ("b", 0.5)], band, y))
        .mark(rule_x("w", band));
    let issues = fig.check(size(640.0, 480.0));
    let four = Issue::OutsideDomain {
        handle: band.index(),
        count: 4,
    };
    assert_eq!(issues, [four]);
}

/// `s` as an instant.
fn at(s: &str) -> Timestamp {
    s.parse().expect("an instant")
}

/// Part I's Berlin day: local midnight to midnight across the end of
/// daylight saving time, 25 hours.
fn berlin_day() -> [Timestamp; 2] {
    [at("2026-10-24T22:00:00Z"), at("2026-10-25T23:00:00Z")]
}

#[test]
fn fitting_two_timestamps_a_microsecond_apart_pads_by_fifty_nanoseconds() {
    let t = at("2026-10-25T12:00:00.000000001Z");
    let micro = t + SignedDuration::from_micros(1);
    let mut fig = Figure::new();
    let x = fig.fit(time::utc());
    fig.panel(0, 0).mark(figure::line(x, [t, micro]));
    let r = resolve(&fig, canvas(640.0, 480.0));
    let ns = SignedDuration::from_nanos(50);
    assert_eq!(r.scale(x).get_domain(), [t - ns, micro + ns]);
    // An f64 of epoch seconds in 2026 resolves about 240 ns.
    let seconds = t.as_nanosecond() as f64 / 1e9;
    assert!(seconds.next_up() - seconds > 2e-7);
}

#[test]
fn time_handles_fit_their_data_with_five_percent_in_nanoseconds() {
    let mut fig = Figure::new();
    let x = fig.fit(time::utc());
    let empty = fig.fit(time::time(rustd3_testkit::zone("Europe/Berlin")));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    let [a, b] = berlin_day();
    fig.panel(0, 0)
        .mark(dots([(b, 0.5), (a, 0.2)], x, y))
        .mark(figure::line(empty, []));
    let r = resolve(&fig, canvas(640.0, 480.0));
    // 5% of 25 h is 75 min: matplotlib gives the same.
    let fitted = [at("2026-10-24T20:45:00Z"), at("2026-10-26T00:15:00Z")];
    assert_eq!(r.scale(x).get_domain(), fitted);
    assert_eq!(Kind::domain(r.scale(x)), Some(fitted));
    // No data: matplotlib's empty date axis.
    let day = [Timestamp::UNIX_EPOCH, at("1970-01-02T00:00:00Z")];
    assert_eq!(r.scale(empty).get_domain(), day);
    // Positions over the panel and back, to the nanosecond.
    let area = r.plot_areas()[0];
    let ranged: Time = ranged(r.scale(x), [0.0, area.width()]);
    let middle = at("2026-10-25T10:30:00Z");
    let p = ranged.position(&middle).expect("a position");
    assert!(near(p, area.width() / 2.0, 1e-9), "{p}");
    assert_eq!(Invert::invert(&ranged, p), Some(middle));
}

#[test]
fn berlin_labels_follow_the_iso_ladder_through_a_figure_axis() {
    let berlin = rustd3_testkit::zone("Europe/Berlin");
    let mut fig = Figure::new();
    let x = fig.scale(time::time(berlin).domain(berlin_day()));
    fig.panel(0, 0).axis(axis::bottom(x).ticks(8));
    let size = canvas(800.0, 200.0);
    assert_eq!(
        resolve(&fig, size).scale(x).tick_interval(8),
        time::Interval::Hour(3)
    );
    let ladder = [
        "10-25", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00", "10-26",
    ];
    assert_eq!(texts(&draw(&fig, size)), ladder);
}

/// One panel over `domain` in Berlin, about 24 ticks (hourly) on a bottom
/// axis, `width` px of canvas.
fn hourly(domain: [Timestamp; 2], width: f64) -> (Figure<'static>, Size) {
    let berlin = rustd3_testkit::zone("Europe/Berlin");
    let mut fig = Figure::new();
    let x = fig.scale(time::time(berlin).domain(domain));
    fig.panel(0, 0).axis(axis::bottom(x).ticks(24));
    (fig, Size::new(width, 120.0))
}

/// The instants of the tick labels `fig` keeps at `size`: each kept box's
/// centre through the resolved scale, to the minute.
fn kept(fig: &Figure<'_>, size: Size) -> Vec<Timestamp> {
    let r = resolve(fig, size);
    let area = r.plot_areas()[0];
    let x: Time = ranged(
        r.scale_at(0)
            .and_then(|s| s.as_any().downcast_ref::<Time>())
            .expect("a time scale"),
        [area.x0, area.x1],
    );
    r.label_boxes()
        .iter()
        .map(|b| {
            let t = x.invert(b.center().x);
            let minute = SignedDuration::from_mins(1).as_nanos();
            let ns = (t.as_nanosecond() + minute / 2).div_euclid(minute) * minute;
            Timestamp::from_nanosecond(ns).expect("an instant")
        })
        .collect()
}

#[test]
fn time_ticks_keep_parity_while_panning() {
    let day = berlin_day();
    let (fig, size) = hourly(day, 440.0);
    let before = kept(&fig, size);
    assert!(before.len() < 26 && before.len() > 2, "{before:?}");
    let pan = SignedDuration::from_mins(37);
    let (panned, size) = hourly([day[0] + pan, day[1] + pan], 440.0);
    let after = kept(&panned, size);
    let within = |values: &[Timestamp], of: &[Timestamp]| -> Vec<Timestamp> {
        let (low, high) = (of[0], of[of.len() - 1]);
        values
            .iter()
            .copied()
            .filter(|t| (low..=high).contains(t))
            .collect()
    };
    assert_eq!(within(&before, &after), within(&after, &before));
    assert!(within(&after, &before).len() >= before.len() - 1);
}

/// The points of the first `Stroke` command in `list`.
fn stroked(list: &List) -> Vec<rustd3_figure::kurbo::Point> {
    let path = list
        .commands
        .iter()
        .find_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .expect("a stroke");
    path.elements()
        .iter()
        .filter_map(|e| match *e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(p),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => None,
        })
        .collect()
}

/// 1M samples: a sine with a fast wobble.
fn wave() -> Vec<f64> {
    (0..1_000_000)
        .map(|i| {
            let v = f64::from(i);
            (v * 3e-4).sin() + 0.2 * (v * 0.37).sin()
        })
        .collect()
}

#[test]
fn time_lines_thin_through_the_bridge() {
    // Samples 37 µs past every 100 µs from a whole second, so no column
    // edge (every 125 ms) lies within 37 µs of one.
    let t0 = at("2026-10-25T12:00:00Z");
    let start = t0 + SignedDuration::from_micros(37);
    let period = SignedDuration::from_micros(100);
    let y = wave();
    let timed = Recording::new(Clock::Regular { start, period }, [&y]).expect("valid");
    let seconds = Clock::Regular {
        start: 37e-6,
        period: 1e-4,
    };
    let numbered = Recording::new(seconds, [&y]).expect("valid");
    let size = canvas(800.0, 300.0);
    let target = Target::Screen { scale: 1.0 };
    let mut fig = Figure::new();
    let x = fig.scale(time::utc().domain([t0, t0 + SignedDuration::from_secs(100)]));
    let v = fig.scale(scale::linear().domain([-1.5, 1.5]));
    fig.panel(0, 0)
        .mark(line(timed.series(0).expect("channel 0"), x, v));
    let mut twin = Figure::new();
    let x = twin.scale(scale::linear().domain([0.0, 100.0]));
    let v = twin.scale(scale::linear().domain([-1.5, 1.5]));
    twin.panel(0, 0)
        .mark(line(numbered.series(0).expect("channel 0"), x, v));
    let a = stroked(&fig.draw_with(&mut cache(), size, target, &Held::default()));
    let b = stroked(&twin.draw_with(&mut cache(), size, target, &Held::default()));
    let columns = 800;
    assert!(
        a.len() <= 4 * columns + 2 && a.len() > columns,
        "{}",
        a.len()
    );
    assert_eq!(a.len(), b.len());
    for (p, q) in a.iter().zip(&b) {
        assert!(near(p.x, q.x, 1e-6) && p.y == q.y, "{p:?} {q:?}");
    }
}

#[test]
fn a_time_window_moves_by_whole_columns_in_nanoseconds() {
    let t0 = at("2026-10-25T12:00:00Z");
    let period = SignedDuration::from_micros(100);
    let y = vec![0.25; 1_234_561];
    let rec = Recording::new(Clock::Regular { start: t0, period }, [&y]).expect("valid");
    let mut fig = Figure::new();
    let x = fig.fit_with(time::utc(), Fit::Window(SignedDuration::from_secs(10)));
    let v = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("channel 0"), x, v));
    let target = Target::Screen { scale: 1.0 };
    let r = fig.resolve(
        canvas(1000.0, 300.0),
        &mut cache(),
        &Held::default(),
        target,
    );
    assert!(near(r.plot_areas()[0].width(), 1000.0, 1e-9));
    // The last sample at 123.456 s rounds up to the next 10 ms column.
    let ms = |n: i64| t0 + SignedDuration::from_millis(n);
    assert_eq!(r.scale(x).get_domain(), [ms(113_460), ms(123_460)]);
}

#[test]
fn a_visible_y_reads_a_timestamp_recording_through_its_index() {
    let t0 = at("2026-10-25T12:00:00Z");
    let period = SignedDuration::from_millis(1);
    let y: Vec<f64> = (0..100_000).map(f64::from).collect();
    let rec = Recording::new(Clock::Regular { start: t0, period }, [&y]).expect("valid");
    let mut fig = Figure::new();
    let shown = [t0 + period * 1000, t0 + period * 3000];
    let x = fig.scale(time::utc().domain(shown));
    let v = fig.fit_with(scale::linear(), Fit::Visible);
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("channel 0"), x, v));
    let r = resolve(&fig, canvas(640.0, 480.0));
    // Samples 1000 … 3000, ends included, padded by 5% of 2000.
    assert_eq!(r.scale(v).get_domain(), [900.0, 3100.0]);
}

#[test]
fn a_visible_time_handle_steps_in_whole_nanoseconds() {
    let t0 = at("2026-10-25T12:00:00Z");
    let hour = SignedDuration::from_hours(1);
    let points: Vec<(f64, Timestamp)> = (0..=100).map(|i| (f64::from(i), t0 + hour * i)).collect();
    let figure = |shown: [f64; 2]| {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain(shown));
        let t = fig.fit_with(time::utc(), Fit::Visible);
        fig.panel(0, 0).mark(dots(points.clone(), x, t));
        (fig, t)
    };
    let mut cache = cache();
    let mut domain = |shown| {
        let (fig, t) = figure(shown);
        let r = fig.resolve(
            canvas(640.0, 480.0),
            &mut cache,
            &Held::default(),
            Target::Vector,
        );
        r.scale(t).get_domain()
    };
    // Hours 0 … 10 padded by 30 min.
    let half = SignedDuration::from_mins(30);
    let wide = [t0 - half, t0 + hour * 10 + half];
    assert_eq!(domain([0.0, 10.0]), wide);
    // Hours 4 … 6 pad to [3.9 h, 6.1 h]: each end moves in by whole
    // eighths of 11 h (4,950 s) that keep it inside.
    let eighth = SignedDuration::from_secs(4950);
    let shrunk = domain([4.0, 6.0]);
    assert_eq!(shrunk, [wide[0] + eighth * 3, wide[1] - eighth * 3]);
    assert_eq!(domain([0.0, 10.0]), wide);
}

/// `color`'s hex form, when there is one.
fn hex(color: Option<Rgb>) -> Option<String> {
    color.map(|c| c.format_hex())
}

/// d3's `schemeBlues[5]`, as a literal array: core has no k-color schemes.
fn blues5() -> [Rgb; 5] {
    ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"].map(|c| c.parse().expect("a color"))
}

#[test]
fn sequential_handles_paint_as_d3_and_fit_without_margin() {
    let mut fig = Figure::new();
    let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
    let fitted = fig.fit(scale::sequential([0.0, 80.0], scheme::viridis));
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(figure::line(fitted, [3.0, 7.0]))
        .mark(dots([(0.5, 0.5)], x, x).fill(heat, 40.0));
    let r = resolve(&fig, canvas(300.0, 200.0));
    // d3 in Node: scaleSequential([0, 80], interpolateViridis).
    let paints = [0.0, 20.0, 40.0, 80.0, 100.0].map(|v| hex(r.scale(heat).paint(&v)));
    let d3 = ["#440154", "#3b528b", "#21918c", "#fde725", "#fde725"];
    assert_eq!(paints, d3.map(|c| Some(c.to_string())));
    // No margin on color: [3, 7] exactly, 5 at the middle of the ramp.
    assert_eq!(r.scale(fitted).get_domain(), [3.0, 7.0]);
    assert_eq!(hex(r.scale(fitted).paint(&5.0)).as_deref(), Some("#21918c"));
    assert_eq!(r.scale(heat).paint(&f64::NAN), None);
    assert_eq!(
        scale::Ticks::tick_set(r.scale(heat), 4).values(),
        [0.0, 20.0, 40.0, 60.0, 80.0]
    );
    // A palette key on a dot reaches the drawn paint.
    let list = draw(&fig, canvas(300.0, 200.0));
    let painted = list.commands.iter().find_map(|c| {
        if let Command::Markers { paint, .. } = c {
            Some(paint.clone())
        } else {
            None
        }
    });
    let expected = rustd3_figure::peniko::Color::from_rgb8(0x21, 0x91, 0x8c);
    assert_eq!(painted, Some(rustd3_figure::peniko::Brush::Solid(expected)));
}

#[test]
fn diverging_handles_paint_as_d3_and_keep_their_middle() {
    let mut fig = Figure::new();
    let rd_bu = fig.scale(scale::diverging([-1.0, 0.0, 1.0], scheme::rd_bu));
    let fitted = fig.fit(scale::diverging([-1.0, 0.0, 1.0], scheme::rd_bu));
    fig.panel(0, 0).mark(figure::line(fitted, [2.0, 5.0]));
    let r = resolve(&fig, canvas(300.0, 200.0));
    // d3 in Node: scaleDiverging([-1, 0, 1], interpolateRdBu).
    let paints = [-1.0, 0.0, 0.5].map(|v| r.scale(rd_bu).paint(&v).map(|c| c.to_string()));
    let d3 = [
        "rgb(103, 0, 31)",
        "rgb(242, 239, 238)",
        "rgb(107, 172, 208)",
    ];
    assert_eq!(paints, d3.map(|c| Some(c.to_string())));
    // D33.2: data on one side of the middle keeps it, so the ramp does
    // not reverse.
    assert_eq!(r.scale(fitted).get_domain(), [0.0, 0.0, 5.0]);
    assert_eq!(Kind::domain(r.scale(fitted)), Some([0.0, 5.0]));
    assert_eq!(
        scale::Ticks::tick_set(r.scale(rd_bu), 4).values(),
        [-1.0, -0.5, 0.0, 0.5, 1.0]
    );
}

#[test]
fn linear_color_handles_interpolate_their_range_and_fit_without_margin() {
    let red = Rgb::new(255.0, 0.0, 0.0);
    let blue = Rgb::new(0.0, 0.0, 255.0);
    let mut fig = Figure::new();
    let ramp = fig.fit(scale::linear().range([red, blue]));
    fig.panel(0, 0).mark(figure::line(ramp, [2.0, 4.0, 3.0]));
    let r = resolve(&fig, canvas(300.0, 200.0));
    assert_eq!(r.scale(ramp).get_domain(), [2.0, 4.0]);
    let middle = r.scale(ramp).paint(&3.0).expect("a color");
    assert_eq!(middle.to_string(), "rgb(128, 0, 128)");
    assert_eq!(
        scale::Ticks::tick_set(r.scale(ramp), 4).values(),
        [2.0, 2.5, 3.0, 3.5, 4.0]
    );
}

#[test]
fn binned_handles_paint_their_bins_as_d3() {
    let red = Rgb::new(255.0, 0.0, 0.0);
    let white = Rgb::new(255.0, 255.0, 255.0);
    let blue = Rgb::new(0.0, 0.0, 255.0);
    let mut fig = Figure::new();
    let q = fig.scale(scale::quantize([0.0, 100.0], blues5()));
    let fitted = fig.fit(scale::quantize([0.0, 100.0], blues5()));
    let t = fig.fit(scale::threshold([0.0, 1.0], [red, white, blue]));
    let sample = [1.0, 2.0, 3.0, 10.0];
    let quantile = fig.scale(scale::quantile(sample, [red, blue]));
    fig.panel(0, 0)
        .mark(figure::line(fitted, [10.0, 60.0]))
        .mark(figure::line(t, [-5.0, 5.0]));
    let r = resolve(&fig, canvas(300.0, 200.0));
    // d3 in Node: scaleQuantize([0, 100], schemeBlues[5]).
    assert_eq!(r.scale(q).thresholds(), [20.0, 40.0, 60.0, 80.0]);
    let paints = [10.0, 30.0, 99.0].map(|v| hex(r.scale(q).paint(&v)));
    assert_eq!(
        paints,
        ["#eff3ff", "#bdd7e7", "#08519c"].map(|c| Some(c.to_string()))
    );
    assert_eq!(r.scale(fitted).get_domain(), [10.0, 60.0]);
    // d3: scaleThreshold([0, 1], [red, white, blue]); its thresholds stay.
    let bins = [-1.0, 0.0, 0.5, 1.0].map(|v| r.scale(t).paint(&v));
    assert_eq!(bins, [Some(red), Some(white), Some(white), Some(blue)]);
    assert_eq!(r.scale(t).get_domain(), [0.0, 1.0]);
    // The sample is the quantile scale's domain.
    assert_eq!(r.scale(quantile).quantiles(), [2.5]);
    assert_eq!(r.scale(quantile).paint(&3.0), Some(blue));
    // Binned scales tick at their edges.
    assert_eq!(
        scale::Ticks::tick_set(r.scale(q), 5).values(),
        [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
    );
    assert_eq!(scale::Ticks::tick_set(r.scale(t), 5).values(), [0.0, 1.0]);
}
