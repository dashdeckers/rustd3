//! Figure handles of every scale kind: how each fits, resolves, places or
//! paints its values, and ticks.

use rustd3_core::scale::{self, Band, Point};
use rustd3_figure::kurbo::Size;
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{dots, line, rule_x};
use rustd3_figure::scale::{Kind, Position, Scale};
use rustd3_figure::{Figure, Held, Resolved, Target, axis};

use crate::figure::cache;

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
