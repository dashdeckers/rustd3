//! Scales, a line and an axis at the core level, printed as an SVG.

use rustd3::format::{InvalidFormat, Locale};
use rustd3::{axis, path, scale, shape};

fn main() -> Result<(), InvalidFormat> {
    // Scales are values: build them, call them, drop them.
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    let y = scale::linear().domain([0.0, 1.0]).range([100.0, 0.0]);
    assert_eq!(x.apply(25.0), 240.0);
    assert_eq!(x.invert(480.0), 50.0);
    assert_eq!(x.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);

    // Generators draw scaled geometry into any PathContext: here an SVG path.
    let data = [(0.0, 0.5), (50.0, 1.0), (100.0, 0.0)];
    let points = data.map(|(a, b)| (x.apply(a), y.apply(b)));
    let line = path::svg(|p| shape::line().draw(points, p));
    assert_eq!(line, "M0,50L480,0L960,100");

    // An axis is d3's: its domain line and ticks as a path, and its labels.
    let bottom = axis::bottom(&x).ticks(5);
    let ticks = path::svg(|p| bottom.draw(p));
    let labels: Vec<String> = bottom
        .labels()
        .iter()
        .map(|l| l.text().to_owned())
        .collect();
    assert_eq!(labels, ["0", "20", "40", "60", "80", "100"]);

    // Number formats are d3-format's, in the locale you pass; `fmt!` checks
    // its specifier at compile time.
    assert_eq!(Locale::EN_US.format("$,.2f")?.format(1234.5), "$1,234.50");
    assert_eq!(rustd3::fmt!(".1%").format(0.123), "12.3%");

    println!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"980\" height=\"140\">\
         <g transform=\"translate(10,10)\">\
         <path d=\"{line}\" fill=\"none\" stroke=\"steelblue\"/>\
         <path d=\"{ticks}\" transform=\"translate(0,105)\" fill=\"none\" stroke=\"black\"/>\
         </g></svg>"
    );
    Ok(())
}
