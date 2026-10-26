//! Every compilable RFC code block, verbatim as displayed. `cargo build --example doc_snippets`.
#![allow(unused, path_statements)]

use d3::color::{Hsl, Rgb, scheme};
use d3::interact::{BrushEvent, BrushX, Pointer, Rescale, Transform, Zoom};
use d3::interpolate::Interpolate;
use d3::path::svg;
use d3::scale::{self, Linear};
use d3::shape::{self, Curve};
use d3::anim::Timing;
use d3::{fmt, format, join, time};
use jiff::{Timestamp, tz::TimeZone};
use std::time::Duration;

type R = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
struct Reading {
    t: Timestamp,
    value: Option<f64>,
}

enum Message {
    Show([f64; 2]),
}

fn b3811() -> R {
    let t0: Timestamp = "2026-10-13T08:00:00Z".parse()?;
    let data: Vec<Reading> = [Some(1.0), Some(3.0), None, Some(2.0), Some(4.0)]
        .iter()
        .enumerate()
        .map(|(i, v)| Reading { t: t0 + jiff::SignedDuration::from_mins(15 * i as i64), value: *v })
        .collect();
    let tx = time::time(TimeZone::UTC).domain([data[0].t, data[4].t]).range([0.0, 400.0]);
    let ty = scale::linear().domain([0.0, 4.0]).range([100.0, 0.0]);
    // ---
    let d = svg(|p| {
        let points = data
            .iter()
            .map(|r| {
                let x = tx.apply(r.t);
                r.value.map(|v| (x, ty.apply(v)))
            });
        shape::line()
            .draw(points, p)
    });
    // → M0,75L100,25M300,50L400,0
    Ok(())
}

fn b4116() -> R {
    let t0: Timestamp = "2026-10-13T08:00:00Z".parse()?;
    let data: Vec<Reading> = [Some(1.0), Some(3.0), None, Some(2.0), Some(4.0)]
        .iter()
        .enumerate()
        .map(|(i, v)| Reading { t: t0 + jiff::SignedDuration::from_mins(15 * i as i64), value: *v })
        .collect();
    let tx = time::time(TimeZone::UTC).domain([data[0].t, data[4].t]).range([0.0, 400.0]);
    let ty = scale::linear().domain([0.0, 4.0]).range([100.0, 0.0]);
    // ---
    let d = svg(|p| {
        shape::line_of()
            .x(|r: &Reading| tx.apply(r.t))
            .y(|r: &Reading| {
                ty.apply(r.value.unwrap_or(0.0))
            })
            .defined(|r: &Reading| r.value.is_some())
            .draw(&data, p)
    });
    // → M0,75L100,25M300,50L400,0
    Ok(())
}

fn b6129() -> R {
    let x = scale::linear()
        .domain([0.0, 100.0])
        .range([0.0, 960.0]);
    x.apply(25.0);    // 240
    x.invert(480.0);  // 50
    Ok(())
}

fn b6431() -> R {
    let steel: Rgb = "steelblue".parse()?;
    let white = Rgb::new(255.0, 255.0, 255.0);
    let heat = scale::linear()
        .domain([0.0, 1.0])
        .range([white, steel]);
    heat.apply(0.5);  // rgb(163, 193, 218)
    Ok(())
}

fn b6889() -> R {
    let y = scale::linear()
        .domain([0.13, 9.71])
        .range([300.0, 0.0])
        .nice(5);
    let f = y.tick_format(5, None)?;
    y.get_domain();  // [0.0, 10.0]
    y.ticks(5)
        .iter()
        .map(|v| f.format(*v));
    // → "0" "2" "4" "6" "8" "10"
    Ok(())
}

fn b7216() -> R {
    let bands = scale::band(["a", "b", "c"])
        .range([0.0, 300.0])
        .padding(0.1);
    bands.apply(&"a");  // Some(9.677…)
    bands.apply(&"z");  // None
    bands.bandwidth();  // 87.10
    Ok(())
}

fn b7647() -> R {
    let color = scale::ordinal(
        ["cpu", "mem"],
        scheme::CATEGORY10,
    );
    color.apply(&"cpu");
    // → Some(rgb(31, 119, 180))
    color.apply(&"disk");  // None

    let mut grow = scale::ordinal(
        Vec::<&str>::new(),
        scheme::CATEGORY10,
    );
    let disk = *grow.apply_or_insert("disk");
    // → rgb(31, 119, 180)
    let net = *grow.apply_or_insert("net");
    // → rgb(255, 127, 14)
    Ok(())
}

fn b8409() -> R {
    let seq = scale::sequential(
        [0.0, 50.0],
        scheme::blues,
    );
    seq.apply(25.0);  // rgb(128, 150, 181)
    Ok(())
}

fn b10206() -> R {
    let pts = [
        (0.0, 0.0),
        (1.0, 2.0),
        (2.0, 1.0),
        (3.0, 3.0),
    ];
    svg(|p| {
        shape::line()
            .curve(Curve::Step)
            .draw(pts, p)
    });
    // → M0,0L0.5,0L0.5,2L1.5,2
    //   L1.5,1L2.5,1L2.5,3L3,3
    Ok(())
}

fn b10459() -> R {
    let rows = [
        (0.0, 10.0, 4.0),
        (1.0, 10.0, 2.0),
        (2.0, 10.0, 5.0),
    ];
    svg(|p| {
        shape::area()
            .draw(rows.map(Some), p)
    });
    // → M0,4L1,2L2,5L2,10L1,10L0,10Z
    Ok(())
}

fn b10756() -> R {
    let slices = shape::pie()
        .layout([3.0, 1.0, 2.0]);
    let donut = shape::arc()
        .inner_radius(40.0)
        .outer_radius(80.0);
    // slices[0]: index 0, angles 0..π
    // donut.centroid(&slices[0]) == (60.0, 0.0)
    Ok(())
}

fn b11123() -> R {
    let apples = [3.0, 2.0, 1.0];
    let bananas = [1.0, 1.0, 4.0];
    let layers = shape::stack(&[&apples, &bananas]);
    // layers[1], bananas on top of apples:
    // [[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]]
    Ok(())
}

fn b12011() -> R {
    join::join(["a", "b", "c"], ["b", "c", "d"]);
    // → Join {
    //       enter: ["d"],
    //       update: ["b", "c"],
    //       exit: ["a"],
    //   }
    Ok(())
}

fn b12857() -> R {
    let money = fmt!("$,.2f");
    money.format(-1234.5);  // −$1,234.50

    fmt!(".3s")
        .format(0.000_042);  // 42.0µ

    format::format(".1%")?
        .format(0.123);  // 12.3%

    format::format("10..f");
    // → Err(InvalidFormat("10..f"))
    Ok(())
}

fn b13429() -> R {
    let berlin = TimeZone::get("Europe/Berlin")?;
    let day: [Timestamp; 2] = [
        "2026-10-24T22:00:00Z".parse()?,
        "2026-10-25T23:00:00Z".parse()?,
    ];
    let st = time::time(berlin)
        .domain(day)
        .range([0.0, 800.0]);
    st.tick_interval(8);  // Hour(3)
    st.ticks(8)
        .iter()
        .map(time::Time::tick_label);
    // → "Oct 25" "03 AM" "06 AM" "09 AM"
    //   "12 PM" "03 PM" "06 PM" "09 PM"
    //   "Mon 26"
    Ok(())
}

fn b16762() -> R {
    let timing = Timing {
        duration: Duration::from_millis(750),
        stagger: Duration::from_millis(20),
        // no delay, cubic in-out: d3's defaults
        ..Timing::default()
    };
    Ok(())
}

fn b14533() -> R {
    let steel: Rgb = "steelblue".parse()?;
    // ---
    let hsl = Hsl {
        h: 200.0,
        s: 0.6,
        l: 0.5,
        opacity: 1.0,
    };
    Rgb::from(hsl);       // rgb(51, 153, 204)
    steel.brighter(1.0);  // rgb(100, 186, 255)

    "#zzz".parse::<Rgb>();
    // → Err(ParseColorError("#zzz"))

    Rgb::hex(0x000000)
        .lerp(&Rgb::hex(0xffffff), 0.5);
    // → rgb(128, 128, 128)
    Ok(())
}

fn b15679() -> R {
    let full = scale::linear().domain([0.0, 100.0]).range([0.0, 900.0]);
    let current = Transform { k: 1.0, x: 0.0, y: 0.0 };
    // ---
    let mut zoom = Zoom::default()
        .scale_extent([1.0, 50.0]);
    let wheel = Pointer::Wheel {
        x: 450.0,
        y: 150.0,
        lines: -20.0,
    };
    if let Some(t) = zoom.handle(wheel, current) {
        // d3's transform.rescaleX(x)
        let visible = full
            .rescale_x(&t)
            .get_domain();
    }
    Ok(())
}

fn b16201(brush: &mut BrushX, input: Pointer, x: &Linear, full: [f64; 2]) -> Option<Message> {
    Some(match brush.handle(input)? {
        BrushEvent::Brush(Some([a, b]))
        | BrushEvent::End(Some([a, b])) => {
            Message::Show([
                x.invert(a),
                x.invert(b),
            ])
        }
        BrushEvent::End(None) => Message::Show(full),
        _ => return None,
    })
}

mod b8907 {
    pub trait PathContext {
        fn move_to(&mut self, x: f64, y: f64);
        fn line_to(&mut self, x: f64, y: f64);
        fn quadratic_curve_to(
            &mut self,
            x1: f64, y1: f64,
            x: f64, y: f64,
        );
        fn bezier_curve_to(
            &mut self,
            x1: f64, y1: f64,
            x2: f64, y2: f64,
            x: f64, y: f64,
        );
        fn arc(
            &mut self,
            x: f64, y: f64, r: f64,
            start: f64, end: f64,
            anticlockwise: bool,
        );
        fn close_path(&mut self);
    }
}

fn main() {
    b3811().unwrap();
    b4116().unwrap();
    b6129().unwrap();
    b6431().unwrap();
    b6889().unwrap();
    b7216().unwrap();
    b7647().unwrap();
    b8409().unwrap();
    b10206().unwrap();
    b10459().unwrap();
    b10756().unwrap();
    b11123().unwrap();
    b12011().unwrap();
    b12857().unwrap();
    b13429().unwrap();
    b16762().unwrap();
    b14533().unwrap();
    b15679().unwrap();
}
