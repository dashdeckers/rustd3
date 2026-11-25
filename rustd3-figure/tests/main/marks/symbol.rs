//! Dots drawn as symbols: d3's symbols at the area of the circle they
//! replace, hollow dots and line symbols as outlines one patch edge wide,
//! and legend swatches that repeat both.

use std::f64::consts::PI;

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::shape::Symbol;
use rustd3_figure::kurbo::{BezPath, Point, Shape};
use rustd3_figure::legend::{Slot, legend};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::dots;
use rustd3_figure::{Figure, Theme};

use super::{draw, fitted, marked, role, solid};

/// A dot's radius by default: half the theme's marker size.
fn radius() -> f64 {
    Theme::default().get_marker() / 2.0
}

/// An outline's width: the theme's patch edge.
fn width() -> f64 {
    Theme::default().get_patch_width()
}

/// The markers commands among `list`'s marks, as (shape, points).
fn markers(list: &List) -> Vec<(&BezPath, usize)> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            let Command::Markers {
                shape, at, sizes, ..
            } = c
            else {
                return None;
            };
            assert!(sizes.is_none(), "stamped at one size");
            Some((shape, at.len()))
        })
        .collect()
}

/// Half the width of `shape`'s bounding box.
fn half(shape: &BezPath) -> f64 {
    shape.bounding_box().width() / 2.0
}

#[test]
fn a_symbol_has_the_area_of_the_circle_it_replaces() {
    let r = radius();
    let shape = |symbol| {
        let (mut fig, x, y) = fitted();
        fig.panel(0, 0)
            .mark(dots([(0.0, 0.0)], x, y).symbol(symbol));
        let list = draw(&fig);
        let shapes = markers(&list);
        assert_eq!(shapes.len(), 1, "{symbol:?}");
        shapes[0].0.clone()
    };
    for symbol in [
        Symbol::Square,
        Symbol::Diamond,
        Symbol::Triangle,
        Symbol::Star,
    ] {
        let area = shape(symbol).area().abs();
        assert!((area - PI * r * r).abs() < 1e-6, "{symbol:?}: {area}");
    }
    // Observable Plot's hexagon is as wide as the circle instead.
    let hexagon = shape(Symbol::Hexagon);
    assert!((half(&hexagon) - r).abs() < 1e-9, "{}", half(&hexagon));
}

#[test]
fn a_hollow_dot_is_a_ring_one_patch_edge_wide() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(dots([(0.0, 0.0)], x, y).hollow(true));
    let list = draw(&fig);
    let (r, w) = (radius(), width());
    let (ring, _) = markers(&list)[0];
    assert_eq!(ring.winding(Point::ORIGIN), 0, "the inside is empty");
    assert_ne!(ring.winding(Point::new(r, 0.0)), 0, "the outline is filled");
    assert!((half(ring) - (r + w / 2.0)).abs() < 1e-3, "{}", half(ring));
    // The ring from r − w/2 to r + w/2: the circumference times the width.
    let area = ring.area().abs();
    assert!((area - 2.0 * PI * r * w).abs() < 1e-2, "{area}");
}

#[test]
fn a_line_symbol_is_drawn_as_its_outline_without_hollow() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0)], x, y).symbol(Symbol::Plus));
    let list = draw(&fig);
    let (plus, _) = markers(&list)[0];
    // d3's plus at the circle's area; round caps reach half a width past
    // each arm's end.
    let (r, w) = (radius(), width());
    let size = PI * r * r;
    let arm = (size - (size / 7.0).min(2.0)).sqrt() * 0.87559;
    assert!(
        (half(plus) - (arm + w / 2.0)).abs() < 1e-3,
        "{}",
        half(plus)
    );
    assert_ne!(plus.winding(Point::ORIGIN), 0, "the arms cross filled");
    assert_eq!(
        plus.winding(Point::new(arm / 2.0, arm / 2.0)),
        0,
        "between the arms is empty"
    );
}

#[test]
fn hollow_dots_of_a_size_channel_keep_one_outline_width() {
    let (mut fig, x, y) = fitted();
    let size = fig.scale(scale::linear().domain([0.0, 10.0]).range([2.0, 12.0]));
    fig.panel(0, 0).mark(
        dots([(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)], x, y)
            .size_by(size, [0.0, 5.0, 0.0])
            .hollow(true),
    );
    let list = draw(&fig);
    let groups = markers(&list);
    // Radius 2 for the first and last dot, 7 for the middle one: one
    // command per radius, in the order the radii first appear.
    assert_eq!(groups.iter().map(|g| g.1).collect::<Vec<_>>(), [2, 1]);
    let w = width();
    for ((shape, _), r) in groups.iter().zip([2.0, 7.0]) {
        assert!(
            (half(shape) - (r + w / 2.0)).abs() < 1e-3,
            "{r}: {}",
            half(shape)
        );
    }
}

#[test]
fn legend_swatches_repeat_the_symbol_and_hollowness_of_their_dots() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let kind = fig.scale(scale::ordinal(["a", "b", "c"], COLORBLIND));
    fig.panel(0, 0)
        .mark(
            dots([(0.0, 0.0)], x, y)
                .fill(kind, "a")
                .symbol(Symbol::Diamond)
                .hollow(true),
        )
        .mark(
            dots([(1.0, 1.0)], x, y)
                .fill(kind, "b")
                .symbol(Symbol::Square),
        )
        .mark(dots([(2.0, 2.0)], x, y).fill(kind, "c"));
    fig.legend(Slot::Right, legend(kind));
    let list = draw(&fig);
    let swatches: Vec<&Command> = list
        .parts
        .iter()
        .filter(|p| role(p.id) == 7)
        .flat_map(|p| &list.commands[p.commands.clone()])
        .filter(|c| !matches!(c, Command::Glyphs(_)))
        .collect();
    let of = |i: usize| {
        swatches
            .iter()
            .find(|c| {
                matches!(c, Command::Fill { paint, .. } | Command::Stroke { paint, .. }
                    if *paint == solid(COLORBLIND[i]))
            })
            .copied()
            .unwrap_or_else(|| panic!("no swatch for key {i}: {swatches:?}"))
    };
    // A hollow diamond: stroked, taller than wide by √3.
    let Command::Stroke { path, .. } = of(0) else {
        panic!("not stroked: {:?}", of(0));
    };
    let b = path.bounding_box();
    assert!(
        (b.height() / b.width() - 3.0_f64.sqrt()).abs() < 1e-6,
        "{b:?}"
    );
    // A filled square, with the area of the dot beside it.
    let Command::Fill { path: square, .. } = of(1) else {
        panic!("not filled: {:?}", of(1));
    };
    let Command::Fill { path: dot, .. } = of(2) else {
        panic!("not filled: {:?}", of(2));
    };
    let s = square.bounding_box();
    assert!((s.width() - s.height()).abs() < 1e-9, "{s:?}");
    assert!((square.area().abs() - dot.area().abs()).abs() < 1e-2);
}
