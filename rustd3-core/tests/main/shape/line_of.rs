//! `LineOf`, the accessor line, against the iterator-first line it wraps.

use rustd3_core::shape::{self, Curve};
use rustd3_core::{path, scale};

/// Readings: a time and a value that may be missing.
struct Reading {
    t: f64,
    value: Option<f64>,
}

const READINGS: [Reading; 5] = [
    Reading {
        t: 0.0,
        value: Some(25.0),
    },
    Reading {
        t: 1.0,
        value: Some(75.0),
    },
    Reading {
        t: 2.0,
        value: None,
    },
    Reading {
        t: 3.0,
        value: Some(50.0),
    },
    Reading {
        t: 4.0,
        value: Some(100.0),
    },
];

#[test]
fn the_option_b_line_equals_option_a() {
    let tx = scale::linear().domain([0.0, 4.0]).range([0.0, 400.0]);
    let ty = scale::linear().domain([0.0, 100.0]).range([100.0, 0.0]);
    let option_a = path::svg(|p| {
        let points = READINGS.iter().map(|r| {
            let x = tx.apply(r.t);
            r.value.map(|v| (x, ty.apply(v)))
        });
        shape::line().draw(points, p);
    });
    let option_b = path::svg(|p| {
        shape::line_of()
            .x(|r: &Reading| tx.apply(r.t))
            .y(|r: &Reading| ty.apply(r.value.unwrap_or(0.0)))
            .defined(|r: &Reading| r.value.is_some())
            .draw(&READINGS, p);
    });
    assert_eq!(option_b, "M0,75L100,25M300,50L400,0");
    assert_eq!(option_b, option_a);
}

#[test]
fn the_curve_applies_as_on_the_line() {
    let data = [0.0, 2.0, 1.0, 3.0];
    let wrapped = path::svg(|p| {
        shape::line_of()
            .x(|v: &f64| *v)
            .y(|v: &f64| v * v)
            .curve(Curve::MonotoneX)
            .draw(&data, p);
    });
    let plain = path::svg(|p| {
        shape::line()
            .curve(Curve::MonotoneX)
            .draw(data.map(|v| (v, v * v)), p);
    });
    assert_eq!(wrapped, plain);
}

#[test]
fn the_default_accessors_put_every_datum_at_the_origin() {
    let d = path::svg(|p| shape::line_of().draw(&["a", "b"], p));
    assert_eq!(d, "M0,0L0,0");
    let none = path::svg(|p| shape::line_of::<f64>().draw(&[], p));
    assert_eq!(none, "");
}
