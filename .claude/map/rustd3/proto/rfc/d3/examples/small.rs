//! Every small example from the RFC, runnable. `cargo run --example small`.

use d3::color::{Hsl, Rgb, scheme};
use d3::interpolate::Interpolate;
use d3::path::{PathContext, svg};
use d3::scale::{self, TickSource};
use d3::shape::{self, Curve};
use d3::{axis, fmt, format, join, time};
use jiff::{Timestamp, tz::TimeZone};

#[derive(Debug, Clone)]
struct Reading {
    t: Timestamp,
    value: Option<f64>,
}

fn main() {
    // E1: a linear scale, applied and inverted.
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    println!("E1 x(25) = {}, x.invert(480) = {}", x.apply(25.0), x.invert(480.0));

    // E2: the range decides the output type.
    let steel: Rgb = "steelblue".parse().unwrap();
    let heat = scale::linear().domain([0.0, 1.0]).range([Rgb::new(255.0, 255.0, 255.0), steel]);
    println!("E2 heat(0.5) = {}", heat.apply(0.5));

    // E3: nice domains, ticks and their format.
    let y = scale::linear().domain([0.13, 9.71]).range([300.0, 0.0]).nice(5);
    let f = y.tick_format(5, None).unwrap();
    let labels: Vec<String> = y.ticks(5).iter().map(|v| f.format(*v)).collect();
    println!("E3 domain {:?}, labels {:?}", y.get_domain(), labels);

    // E4: band scales return Option for unknown keys.
    let bands = scale::band(["a", "b", "c"]).range([0.0, 300.0]).padding(0.1);
    println!("E4 a -> {:?}, z -> {:?}, bandwidth {:.2}", bands.apply(&"a"), bands.apply(&"z"), bands.bandwidth());

    // E5: ordinal scales, explicit domain vs d3's implicit growth.
    let color = scale::ordinal(["cpu", "mem"], scheme::CATEGORY10);
    println!("E5 cpu -> {}, disk -> {:?}", color.apply(&"cpu").unwrap(), color.apply(&"disk"));
    let mut grow = scale::ordinal(Vec::<&str>::new(), scheme::CATEGORY10);
    let disk = *grow.apply_or_insert("disk");
    let net = *grow.apply_or_insert("net");
    println!("E5 implicit: disk -> {disk}, net -> {net}");

    // E6: a sequential scale over a color interpolator.
    let seq = scale::sequential([0.0, 50.0], scheme::blues);
    println!("E6 seq(25) = {}", seq.apply(25.0));

    // E7: a line from data with gaps: iterators instead of accessors.
    let t0: Timestamp = "2026-10-13T08:00:00Z".parse().unwrap();
    let data: Vec<Reading> = [Some(1.0), Some(3.0), None, Some(2.0), Some(4.0)]
        .iter()
        .enumerate()
        .map(|(i, v)| Reading { t: t0 + jiff::SignedDuration::from_mins(15 * i as i64), value: *v })
        .collect();
    let tx = time::time(TimeZone::UTC).domain([data[0].t, data[4].t]).range([0.0, 400.0]);
    let ty = scale::linear().domain([0.0, 4.0]).range([100.0, 0.0]);
    let d = svg(|p| {
        shape::line().draw(data.iter().map(|r| r.value.map(|v| (tx.apply(r.t), ty.apply(v)))), p)
    });
    println!("E7 {d}");

    // E8: the same with d3-style accessors borrowing the scales.
    let d = svg(|p| {
        shape::line_of()
            .x(|r: &Reading| tx.apply(r.t))
            .y(|r: &Reading| ty.apply(r.value.unwrap_or(0.0)))
            .defined(|r: &Reading| r.value.is_some())
            .draw(&data, p)
    });
    println!("E8 {d}");

    // E9: curves are an enum.
    let pts = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
    for c in [Curve::Step, Curve::MonotoneX] {
        println!("E9 {c:?}: {}", svg(|p| shape::line().curve(c).draw(pts, p)));
    }

    // E10: an area between a baseline and values.
    let d = svg(|p| shape::area().draw([(0.0, 10.0, 4.0), (1.0, 10.0, 2.0), (2.0, 10.0, 5.0)].map(Some), p));
    println!("E10 {d}");

    // E11: pie layout and arcs, with indices instead of datum references.
    let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    let donut = shape::arc().inner_radius(40.0).outer_radius(80.0);
    let first = &slices[0];
    println!(
        "E11 slice {} spans {:.3}..{:.3} rad, centroid ({:.1}, {:.1})",
        first.index, first.start_angle, first.end_angle, donut.centroid(first).0, donut.centroid(first).1
    );

    // E12: stack over columns, not row objects with dynamic keys.
    let (apples, bananas) = ([3.0, 2.0, 1.0], [1.0, 1.0, 4.0]);
    let layers = shape::stack(&[&apples, &bananas]);
    println!("E12 bananas sit on apples: {:?}", layers[1]);

    // E13: a compile-time checked format, and number output.
    let money = fmt!("$,.2f");
    println!("E13 {} | {} | {}", money.format(-1234.5), fmt!(".3s").format(0.000_042), format::format(".1%").unwrap().format(0.123));

    // E14: a runtime specifier is a Result.
    println!("E14 {:?}", format::format("10..f").map(|_| ()));

    // E15: time ticks in an explicit zone, with d3's multi-scale labels.
    let berlin = TimeZone::get("Europe/Berlin").unwrap();
    let day: [Timestamp; 2] = ["2026-10-24T22:00:00Z".parse().unwrap(), "2026-10-25T23:00:00Z".parse().unwrap()];
    let st = time::time(berlin).domain(day).range([0.0, 800.0]);
    let labels: Vec<String> = st.ticks(8).iter().map(time::Time::tick_label).collect();
    println!("E15 interval {:?}: {:?}", st.tick_interval(8), labels);

    // E16: an axis as data, drawn by whatever renders text.
    let ax = axis::axis(axis::Orient::Bottom, &scale::linear().domain([0.0, 1.0]).range([0.0, 200.0]), 4);
    let ticks: Vec<(f64, &str)> = ax.ticks.iter().map(|t| (t.position, t.label.as_str())).collect();
    println!("E16 {:?}", ticks);
    println!("E16 band ticks {:?}", bands.tick_values(0).iter().map(|k| bands.position(k)).collect::<Vec<_>>());

    // E17: colors convert, parse and interpolate as values.
    let hsl = Hsl { h: 200.0, s: 0.6, l: 0.5, opacity: 1.0 };
    println!("E17 {} | {} | {:?}", Rgb::from(hsl), steel.brighter(1.0), "#zzz".parse::<Rgb>());
    println!("E17 midpoint {}", Rgb::hex(0x000000).lerp(&Rgb::hex(0xffffff), 0.5));

    // E18: the keyed join replaces selection.data(data, key).
    println!("E18 {:?}", join::join(["a", "b", "c"], ["b", "c", "d"]));

    // E19: a custom PathContext: count commands instead of drawing.
    #[derive(Default)]
    struct Count(usize);
    impl PathContext for Count {
        fn move_to(&mut self, _: f64, _: f64) { self.0 += 1 }
        fn line_to(&mut self, _: f64, _: f64) { self.0 += 1 }
        fn quadratic_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64) { self.0 += 1 }
        fn bezier_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: f64) { self.0 += 1 }
        fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) { self.0 += 1 }
        fn close_path(&mut self) { self.0 += 1 }
    }
    let mut c = Count::default();
    shape::line().curve(Curve::Basis).draw(pts, &mut c);
    println!("E19 basis over 4 points = {} commands", c.0);
}
