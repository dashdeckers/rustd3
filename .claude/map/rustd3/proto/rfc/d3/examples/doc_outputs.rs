//! Prints every value an RFC output comment claims. `cargo run --example doc_outputs`.
use d3::color::{Hsl, Rgb, scheme};
use d3::interpolate::Interpolate;
use d3::path::svg;
use d3::scale;
use d3::shape::{self, Curve};
use d3::{fmt, format, join, time};
use jiff::{Timestamp, tz::TimeZone};

#[derive(Debug, Clone)]
struct Reading {
    t: Timestamp,
    value: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0: Timestamp = "2026-10-13T08:00:00Z".parse()?;
    let data: Vec<Reading> = [Some(1.0), Some(3.0), None, Some(2.0), Some(4.0)]
        .iter()
        .enumerate()
        .map(|(i, v)| Reading { t: t0 + jiff::SignedDuration::from_mins(15 * i as i64), value: *v })
        .collect();
    let tx = time::time(TimeZone::UTC).domain([data[0].t, data[4].t]).range([0.0, 400.0]);
    let ty = scale::linear().domain([0.0, 4.0]).range([100.0, 0.0]);
    let d = svg(|p| {
        let points = data.iter().map(|r| {
            let x = tx.apply(r.t);
            r.value.map(|v| (x, ty.apply(v)))
        });
        shape::line().draw(points, p)
    });
    println!(".3811 {d}");
    let d = svg(|p| {
        shape::line_of()
            .x(|r: &Reading| tx.apply(r.t))
            .y(|r: &Reading| ty.apply(r.value.unwrap_or(0.0)))
            .defined(|r: &Reading| r.value.is_some())
            .draw(&data, p)
    });
    println!(".4116 {d}");

    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    println!(".6129 {} {}", x.apply(25.0), x.invert(480.0));

    let steel: Rgb = "steelblue".parse()?;
    let white = Rgb::new(255.0, 255.0, 255.0);
    let heat = scale::linear().domain([0.0, 1.0]).range([white, steel]);
    println!(".6431 {}", heat.apply(0.5));

    let y = scale::linear().domain([0.13, 9.71]).range([300.0, 0.0]).nice(5);
    let f = y.tick_format(5, None)?;
    println!(".6889 {:?} {:?}", y.get_domain(), y.ticks(5).iter().map(|v| f.format(*v)).collect::<Vec<_>>());

    let bands = scale::band(["a", "b", "c"]).range([0.0, 300.0]).padding(0.1);
    println!(".7216 {:?} {:?} {:.2}", bands.apply(&"a"), bands.apply(&"z"), bands.bandwidth());

    let color = scale::ordinal(["cpu", "mem"], scheme::CATEGORY10);
    println!(".7647 {:?} / Some({}) {:?}", color.apply(&"cpu"), color.apply(&"cpu").unwrap(), color.apply(&"disk"));
    let mut grow = scale::ordinal(Vec::<&str>::new(), scheme::CATEGORY10);
    let disk = *grow.apply_or_insert("disk");
    let net = *grow.apply_or_insert("net");
    println!(".7647 {disk} {net}");

    let seq = scale::sequential([0.0, 50.0], scheme::blues);
    println!(".8409 {}", seq.apply(25.0));

    let pts = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
    println!(".10206 {}", svg(|p| shape::line().curve(Curve::Step).draw(pts, p)));

    let rows = [(0.0, 10.0, 4.0), (1.0, 10.0, 2.0), (2.0, 10.0, 5.0)];
    println!(".10459 {}", svg(|p| shape::area().draw(rows.map(Some), p)));

    let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    let donut = shape::arc().inner_radius(40.0).outer_radius(80.0);
    println!(".10756 index {} angles {}..{} centroid {:?}", slices[0].index, slices[0].start_angle, slices[0].end_angle, donut.centroid(&slices[0]));

    let apples = [3.0, 2.0, 1.0];
    let bananas = [1.0, 1.0, 4.0];
    let layers = shape::stack(&[&apples, &bananas]);
    println!(".11123 {:?}", layers[1]);

    println!(".12011 {:#?}", join::join(["a", "b", "c"], ["b", "c", "d"]));

    let money = fmt!("$,.2f");
    println!(".12857 {} | {} | {} | {:?}", money.format(-1234.5), fmt!(".3s").format(0.000_042), format::format(".1%")?.format(0.123), format::format("10..f").map(|_| ()));

    let berlin = TimeZone::get("Europe/Berlin")?;
    let day: [Timestamp; 2] = ["2026-10-24T22:00:00Z".parse()?, "2026-10-25T23:00:00Z".parse()?];
    let st = time::time(berlin).domain(day).range([0.0, 800.0]);
    println!(".13429 {:?} {:?}", st.tick_interval(8), st.ticks(8).iter().map(time::Time::tick_label).collect::<Vec<_>>());

    let hsl = Hsl { h: 200.0, s: 0.6, l: 0.5, opacity: 1.0 };
    println!(".14533 {} | {} | {:?} | {}", Rgb::from(hsl), steel.brighter(1.0), "#zzz".parse::<Rgb>(), Rgb::hex(0x000000).lerp(&Rgb::hex(0xffffff), 0.5));
    Ok(())
}
