//! Part hashes: equal for equal drawings, changed for the part whose data
//! or place changed.

use rustd3_core::scale;
use rustd3_figure::kurbo::{Rect, Shape, Size};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::mark::{Ctx, Mark, dots};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::{Figure, Fits, Held, Target, axis};

use crate::figure::cache;
use crate::scatter;

/// `fig` drawn at 640 × 480 for `target` with a fresh cache.
fn draw(fig: &Figure<'_>, target: Target) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        target,
        &Held::default(),
    )
}

/// Each part's id and hash.
fn hashes(list: &List) -> Vec<(part::Id, u64)> {
    list.parts.iter().map(|p| (p.id, p.hash)).collect()
}

/// Two dots marks on fixed scales, the second over `second`, with axes.
fn two_marks(second: Vec<(f64, f64)>) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .axis(axis::left(y))
        .axis(axis::bottom(x))
        .mark(dots(vec![(1.0, 1.0), (2.0, 4.0)], x, y))
        .mark(dots(second, x, y));
    fig
}

#[test]
fn a_figure_drawn_twice_hashes_every_part_equally() {
    let fig = scatter::scatter(scatter::POINTS);
    for target in [Target::Vector, Target::File { scale: 2.0 }] {
        let first = draw(&fig, target);
        let mut kept = cache();
        let size = Size::new(640.0, 480.0);
        let second = fig.draw_with(&mut kept, size, target, &Held::default());
        let third = fig.draw_with(&mut kept, size, target, &Held::default());
        assert_eq!(hashes(&first), hashes(&second));
        assert_eq!(hashes(&first), hashes(&third));
        let rebuilt = draw(&scatter::scatter(scatter::POINTS), target);
        assert_eq!(hashes(&first), hashes(&rebuilt), "equal data, equal hashes");
        let mut distinct: Vec<u64> = first.parts.iter().map(|p| p.hash).collect();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), first.parts.len(), "{:?}", hashes(&first));
    }
}

#[test]
fn changing_one_marks_data_changes_that_parts_hash_only() {
    let before = draw(&two_marks(vec![(5.0, 5.0)]), Target::Vector);
    let after = draw(&two_marks(vec![(5.0, 6.0)]), Target::Vector);
    let changed: Vec<part::Id> = hashes(&before)
        .into_iter()
        .zip(hashes(&after))
        .filter(|(a, b)| a != b)
        .map(|(a, b)| {
            assert_eq!(a.0, b.0, "the same parts");
            a.0
        })
        .collect();
    // Role 6 (mark), panel 0, the second mark.
    assert_eq!(changed, [part::Id::new(6 << 56 | 1)]);
}

/// A mark that draws the same 10 px square at the plot area's corner
/// whatever the panel's size or scales.
struct Square;

impl Mark<'static> for Square {
    fn fit(&self, _fits: &mut Fits) {}

    fn draw(&self, _ctx: &Ctx<'_>, list: &mut List) {
        list.commands.push(Command::Fill {
            path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
            paint: Brush::Solid(Color::from_rgba8(1, 115, 178, 255)),
        });
    }
}

/// One panel with a square and axes, and a title when `titled`.
fn square(titled: bool) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .axis(axis::left(y))
        .axis(axis::bottom(x))
        .mark(Square);
    if titled {
        fig.title("A title pushes the panel down");
    }
    fig
}

#[test]
fn a_panel_pushed_down_by_a_title_changes_its_parts_bounds_and_hashes() {
    let before = draw(&square(false), Target::Vector);
    let after = draw(&square(true), Target::Vector);
    let mark = |list: &List| {
        let p = list
            .parts
            .iter()
            .find(|p| p.id == part::Id::new(6 << 56))
            .expect("the square");
        (list.commands[p.commands.clone()].to_vec(), p.bounds, p.hash)
    };
    let (commands, bounds, hash) = mark(&before);
    let (moved, moved_bounds, moved_hash) = mark(&after);
    // The same commands, drawn lower: the group they are in moved.
    assert_eq!(commands, moved);
    assert!(moved_bounds.y0 > bounds.y0, "{bounds:?} {moved_bounds:?}");
    assert_ne!(hash, moved_hash);
    // The panel, its left axis and its mark moved and changed; the
    // figure's background and the bottom axis, which did not move, kept
    // their hashes; the title is a new part.
    let moved = [2 << 56, 4 << 56, 6 << 56].map(part::Id::new);
    let kept = [0, 4 << 56 | 1].map(part::Id::new);
    assert_eq!(after.parts.len(), before.parts.len() + 1);
    for (a, b) in before.parts.iter().zip(&after.parts) {
        assert_eq!(a.id, b.id);
        if moved.contains(&a.id) {
            assert!(a.bounds != b.bounds && a.hash != b.hash, "{:?}", a.id);
        } else {
            assert!(kept.contains(&a.id), "{:?}", a.id);
            assert_eq!((a.bounds, a.hash), (b.bounds, b.hash), "{:?}", a.id);
        }
    }
}
