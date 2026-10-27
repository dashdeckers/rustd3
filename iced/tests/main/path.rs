//! `IcedPath` against `SvgPath`: lyon's events from
//! `canvas::Path::raw().iter()` reach the endpoints d3's path data names,
//! and arcs become cubics of at most a quarter turn in one subpath.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use iced_widget::canvas;
use iced_widget::canvas::path::lyon_path::PathEvent;
use iced_widget::canvas::path::lyon_path::math::Point;
use rustd3_core::path::{self, PathContext, SvgPath};

/// The events of the iced path `draw` builds through `IcedPath`.
fn events(draw: impl FnOnce(&mut dyn PathContext)) -> Vec<PathEvent> {
    rustd3_iced::path(|p| draw(p)).raw().iter().collect()
}

/// The endpoints iced reaches, in order: every subpath start and every
/// segment's end.
fn vertices(events: &[PathEvent]) -> Vec<(f64, f64)> {
    events
        .iter()
        .filter_map(|event| match *event {
            PathEvent::Begin { at } => Some(at),
            PathEvent::Line { to, .. }
            | PathEvent::Quadratic { to, .. }
            | PathEvent::Cubic { to, .. } => Some(to),
            PathEvent::End { .. } => None,
        })
        .map(|p| (f64::from(p.x), f64::from(p.y)))
        .collect()
}

/// The endpoints SVG path data names: the last pair of each `M`, `L`,
/// `Q`, `C` and `A` command.
fn svg_endpoints(data: &str) -> Vec<(f64, f64)> {
    let mut endpoints = Vec::new();
    let mut rest = data;
    while let Some(command) = rest.chars().next() {
        let body_end = rest[1..]
            .find(|c: char| c.is_ascii_alphabetic())
            .map_or(rest.len(), |i| i + 1);
        let numbers: Vec<f64> = rest[1..body_end]
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("a number in path data"))
            .collect();
        if command != 'Z' {
            let n = numbers.len();
            endpoints.push((numbers[n - 2], numbers[n - 1]));
        }
        rest = &rest[body_end..];
    }
    endpoints
}

/// Counts the events of each kind: begins, lines, cubics, closing ends.
fn census(events: &[PathEvent]) -> (usize, usize, usize, usize) {
    let mut counts = (0, 0, 0, 0);
    for event in events {
        match event {
            PathEvent::Begin { .. } => counts.0 += 1,
            PathEvent::Line { .. } => counts.1 += 1,
            PathEvent::Cubic { .. } => counts.2 += 1,
            PathEvent::End { close: true, .. } => counts.3 += 1,
            PathEvent::Quadratic { .. } | PathEvent::End { close: false, .. } => {}
        }
    }
    counts
}

/// The point of the cubic `event` at parameter `t`.
fn cubic_at(event: &PathEvent, t: f32) -> (f64, f64) {
    let PathEvent::Cubic {
        from,
        ctrl1,
        ctrl2,
        to,
    } = *event
    else {
        panic!("not a cubic: {event:?}");
    };
    let s = 1.0 - t;
    let p = |a: Point, b: Point, c: Point, d: Point| {
        let x = s * s * s * a.x + 3.0 * s * s * t * b.x + 3.0 * s * t * t * c.x + t * t * t * d.x;
        let y = s * s * s * a.y + 3.0 * s * s * t * b.y + 3.0 * s * t * t * c.y + t * t * t * d.y;
        (f64::from(x), f64::from(y))
    };
    p(from, ctrl1, ctrl2, to)
}

/// Asserts that `a` and `b` are within `tolerance` in both coordinates.
fn near(a: (f64, f64), b: (f64, f64), tolerance: f64) {
    assert!(
        (a.0 - b.0).abs() <= tolerance && (a.1 - b.1).abs() <= tolerance,
        "{a:?} is not within {tolerance} of {b:?}"
    );
}

#[test]
fn a_quarter_arc_on_an_empty_path_is_one_cubic() {
    let events = events(|p| p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, false));
    assert_eq!(census(&events), (1, 0, 1, 0));
    let points = vertices(&events);
    near(points[0], (10.0, 0.0), 1e-3);
    near(points[1], (0.0, 10.0), 1e-3);
}

#[test]
fn an_arc_after_a_move_joins_it_with_a_line() {
    let events = events(|p| {
        p.move_to(0.0, 0.0);
        p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, false);
    });
    assert_eq!(census(&events), (1, 1, 1, 0));
    assert!(matches!(events[1], PathEvent::Line { .. }));
    let points = vertices(&events);
    near(points[1], (10.0, 0.0), 1e-3);
    near(points[2], (0.0, 10.0), 1e-3);
}

#[test]
fn an_anticlockwise_quarter_sweeps_three_quarters_in_three_cubics() {
    let events = events(|p| p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, true));
    assert_eq!(census(&events), (1, 0, 3, 0));
    // The middle cubic runs from −π/2 to −π, through −3π/4.
    let half = 10.0 * (0.75 * PI).cos();
    near(cubic_at(&events[2], 0.5), (half, half), 0.01);
    near(*vertices(&events).last().unwrap(), (0.0, 10.0), 1e-3);
}

#[test]
fn a_full_turn_is_four_cubics_back_to_the_start() {
    let events = events(|p| p.arc(0.0, 0.0, 10.0, 0.0, TAU, false));
    assert_eq!(census(&events), (1, 0, 4, 0));
    near(*vertices(&events).last().unwrap(), (10.0, 0.0), 1e-3);
}

#[test]
fn a_donut_slice_is_one_closed_subpath() {
    let events = events(|p| {
        p.move_to(10.0, 0.0);
        p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, false);
        p.line_to(0.0, 5.0);
        p.arc(0.0, 0.0, 5.0, FRAC_PI_2, 0.0, true);
        p.close_path();
    });
    let (begins, _, _, closes) = census(&events);
    assert_eq!((begins, closes), (1, 1));
    assert!(matches!(
        events.last(),
        Some(PathEvent::End { close: true, .. })
    ));

    // iced's own arc starts a subpath of its own for each arc.
    let split = canvas::Path::new(|b| {
        b.move_to(iced_core::Point::new(10.0, 0.0));
        b.arc(canvas::path::Arc {
            center: iced_core::Point::ORIGIN,
            radius: 10.0,
            start_angle: iced_core::Radians(0.0),
            end_angle: iced_core::Radians(std::f32::consts::FRAC_PI_2),
        });
        b.line_to(iced_core::Point::new(0.0, 5.0));
        b.arc(canvas::path::Arc {
            center: iced_core::Point::ORIGIN,
            radius: 5.0,
            start_angle: iced_core::Radians(std::f32::consts::FRAC_PI_2),
            end_angle: iced_core::Radians(0.0),
        });
        b.close();
    });
    let split: Vec<PathEvent> = split.raw().iter().collect();
    assert!(census(&split).0 > 1, "{split:?}");
}

#[test]
fn a_zero_radius_arc_is_only_its_point() {
    let events = events(|p| p.arc(5.0, 5.0, 0.0, 0.0, PI, false));
    assert_eq!(census(&events), (1, 0, 0, 0));
    assert_eq!(events.len(), 2);
    near(vertices(&events)[0], (5.0, 5.0), 0.0);
}

#[test]
fn a_negative_radius_arc_draws_nothing() {
    let events = events(|p| p.arc(5.0, 5.0, -1.0, 0.0, PI, false));
    assert!(events.is_empty(), "{events:?}");
}

#[test]
fn the_iced_path_reaches_every_endpoint_of_the_svg_path() {
    let scripts: [fn(&mut dyn PathContext); 6] = [
        |p| {
            p.move_to(1.0, 2.0);
            p.line_to(30.0, 4.0);
            p.quadratic_curve_to(40.0, 10.0, 50.0, 4.0);
            p.bezier_curve_to(60.0, 0.0, 70.0, 20.0, 80.0, 4.0);
            p.close_path();
        },
        |p| p.arc(100.0, 50.0, 20.0, 0.3, 2.9, false),
        |p| p.arc(100.0, 50.0, 20.0, 0.3, 2.9, true),
        |p| {
            p.move_to(0.0, 0.0);
            p.arc(100.0, 50.0, 20.0, -1.0, 9.0, false);
            p.line_to(5.0, 5.0);
        },
        |p| {
            p.move_to(120.0, 50.0);
            p.arc(100.0, 50.0, 20.0, 0.0, -7.0, true);
            p.arc(100.0, 50.0, 10.0, 1.0, 1.0, false);
            p.close_path();
        },
        |p| {
            p.arc(0.0, 0.0, 30.0, -FRAC_PI_2, 2.0, false);
            p.line_to(0.0, 0.0);
            p.close_path();
            p.line_to(-10.0, -10.0);
        },
    ];
    for script in scripts {
        let mut svg = SvgPath::new();
        script(&mut svg);
        let wanted = svg_endpoints(&svg.to_string());
        let got = vertices(&events(script));
        let mut at = got.iter();
        for &point in &wanted {
            assert!(
                at.any(|&g| (g.0 - point.0).abs() <= 1e-3 && (g.1 - point.1).abs() <= 1e-3),
                "{point:?} of {svg} not reached in order by {got:?}"
            );
        }
        near(*got.last().unwrap(), *wanted.last().unwrap(), 1e-3);
    }
    assert_eq!(path::svg(|p| p.arc(0.0, 0.0, -1.0, 0.0, 1.0, false)), "");
}
