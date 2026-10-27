//! Part hashes: equal for equal drawings, changed for the part whose data
//! or place changed.

use std::collections::BTreeSet;

use rustd3_core::scale;
use rustd3_figure::kurbo::{Affine, BezPath, Point, Rect, Shape, Size};
use rustd3_figure::layout::{Grid, em};
use rustd3_figure::list::{self, Cells, Clip, Command, List, Part, part};
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

/// The cells of `fig` drawn for `target` at 640 × 480, cut at its plot-area
/// edges.
fn cells(fig: &Figure<'_>, target: Target) -> (List, Cells) {
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache(), size, target, &Held::default());
    let (xs, ys) = fig
        .resolve(size, &mut cache(), &Held::default(), target)
        .edges();
    let scale = match target {
        Target::Vector => 1.0,
        Target::File { scale } | Target::Screen { scale } => scale,
    };
    let cells = list::cells(&list, &xs, &ys, scale);
    (list, cells)
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
fn a_figure_drawn_twice_hashes_every_cell_equally() {
    let fig = scatter::scatter(scatter::POINTS);
    let target = Target::File { scale: 2.0 };
    let (_, first) = cells(&fig, target);
    let (_, second) = cells(&scatter::scatter(scatter::POINTS), target);
    assert_eq!(first, second);
    // One panel: 3 × 3 cells.
    assert_eq!((first.cols.len(), first.rows.len()), (4, 4));
    assert_eq!(first.cells.len(), 9);
}

#[test]
fn changing_one_marks_data_changes_the_hashes_of_the_cells_it_touches_only() {
    let target = Target::File { scale: 1.5 };
    let (before_list, before) = cells(&two_marks(vec![(5.0, 5.0)]), target);
    let (after_list, after) = cells(&two_marks(vec![(5.0, 6.0)]), target);
    let mark = |list: &List| {
        list.parts
            .iter()
            .position(|p| p.id == part::Id::new(6 << 56 | 1))
            .expect("the second mark") as u32
    };
    let touched: BTreeSet<usize> = [(&before, mark(&before_list)), (&after, mark(&after_list))]
        .into_iter()
        .flat_map(|(cells, mark)| {
            cells
                .cells
                .iter()
                .enumerate()
                .filter(move |(_, c)| c.parts.contains(&mark))
                .map(|(i, _)| i)
        })
        .collect();
    let changed: BTreeSet<usize> = before
        .cells
        .iter()
        .zip(&after.cells)
        .enumerate()
        .filter(|(_, (a, b))| a.hash != b.hash)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(changed, touched);
    // The dots sit inside the plot area: one cell of nine.
    assert_eq!(changed.len(), 1);
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

#[test]
fn cells_are_half_open_and_none_outside_the_canvas() {
    let cells = list::cells(&List::default(), &[0.0, 100.0, 200.0], &[0.0, 50.0], 1.0);
    assert_eq!(cells.cells.len(), 2);
    assert_eq!(cells.cell_at([100, 10]), Some(1));
    assert_eq!(cells.cell_at([99, 49]), Some(0));
    assert_eq!(cells.cell_at([200, 10]), None);
    assert_eq!(cells.cell_at([10, 50]), None);
    assert_eq!(cells.cell_at([-1, 10]), None);
    assert_eq!(cells.cells[1].rect, Rect::new(100.0, 0.0, 200.0, 50.0));
}

/// A part over `commands` with `bounds`.
fn part_over(commands: std::ops::Range<usize>, bounds: Rect) -> Part {
    Part {
        id: part::Id::new(commands.start as u64),
        commands,
        bounds,
        hash: 7,
    }
}

#[test]
fn a_stroke_of_33_elements_and_5600_markers_are_5603_work_items() {
    let mut stroke = BezPath::new();
    stroke.move_to((0.0, 0.0));
    for i in 1..33 {
        stroke.line_to((f64::from(i), 1.0));
    }
    assert_eq!(stroke.elements().len(), 33);
    let at: Vec<Point> = (0..5600)
        .map(|i| Point::new(f64::from(i % 50), 2.0))
        .collect();
    let brush = Brush::Solid(Color::from_rgba8(0, 0, 0, 255));
    let list = List {
        size: Size::new(100.0, 100.0),
        commands: vec![
            Command::Stroke {
                path: stroke,
                style: list::stroke(1.0),
                paint: brush.clone(),
            },
            Command::Markers {
                shape: Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1),
                at,
                paint: brush,
                sizes: None,
                paints: None,
            },
        ],
        parts: vec![part_over(0..2, Rect::new(0.0, 0.0, 50.0, 3.0))],
        ..List::default()
    };
    let cells = list::cells(&list, &[0.0, 100.0], &[0.0, 100.0], 1.0);
    assert_eq!(cells.cells[0].work, 1 + 33 / 16 + 5600);
    assert_eq!(cells.cells[0].work, 5603);
}

#[test]
fn parts_land_in_every_cell_their_bounds_inflated_by_a_device_pixel_touch() {
    let list = List {
        size: Size::new(200.0, 100.0),
        parts: vec![
            // Inside cell 0, 0.4 px from its right edge: inflated, it
            // touches cell 1 too.
            part_over(0..0, Rect::new(10.0, 10.0, 49.6, 20.0)),
            // In cell 1 alone.
            part_over(0..0, Rect::new(60.0, 60.0, 90.0, 90.0)),
            // Draws nothing.
            part_over(0..0, Rect::ZERO),
        ],
        ..List::default()
    };
    let cells = list::cells(&list, &[0.0, 50.0, 100.0], &[0.0, 100.0], 2.0);
    assert_eq!(
        (cells.cols.clone(), cells.rows.clone()),
        (vec![0, 100, 200], vec![0, 200])
    );
    let parts: Vec<&[u32]> = cells.cells.iter().map(|c| c.parts.as_slice()).collect();
    assert_eq!(parts, [&[0][..], &[0, 1][..]]);
    assert_ne!(cells.cells[0].hash, cells.cells[1].hash);
}

#[test]
fn a_clipped_part_stops_at_its_clip_on_whole_device_pixels() {
    let brush = Brush::Solid(Color::from_rgba8(0, 0, 0, 255));
    let clipped = |clip: Rect| List {
        size: Size::new(200.0, 100.0),
        commands: vec![
            Command::Push {
                transform: Affine::translate((50.0, 0.0)),
                clip: Some(Clip::Rect(clip)),
            },
            Command::Fill {
                path: Rect::new(-10.0, 10.0, 30.0, 20.0).to_path(0.1),
                paint: brush.clone(),
            },
            Command::Pop,
        ],
        // The fill reaches the clip's left edge, x = 50 in list px.
        parts: vec![part_over(1..2, Rect::new(50.0, 10.0, 80.0, 20.0))],
        ..List::default()
    };
    let parts = |list: &List| -> Vec<Vec<u32>> {
        list::cells(list, &[0.0, 50.0, 100.0], &[0.0, 100.0], 2.0)
            .cells
            .iter()
            .map(|c| c.parts.clone())
            .collect()
    };
    // A clip on device pixels: the inflated bounds stop at it.
    let aligned = clipped(Rect::new(0.0, 0.0, 50.0, 100.0));
    assert_eq!(parts(&aligned), [vec![], vec![0]]);
    // A clip a quarter px left of the edge reaches the cell before it.
    let between = clipped(Rect::new(-0.25, 0.0, 50.0, 100.0));
    assert_eq!(parts(&between), [vec![0], vec![0]]);
    // Outside every clip the bounds grow by a device px either way.
    let mut free = clipped(Rect::new(0.0, 0.0, 50.0, 100.0));
    free.commands = vec![free.commands[1].clone()];
    free.parts = vec![part_over(0..1, Rect::new(50.0, 10.0, 80.0, 20.0))];
    assert_eq!(parts(&free), [vec![0], vec![0]]);
}

#[test]
fn a_wheel_like_grid_cuts_the_canvas_into_nine_by_nine_cells() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.layout(
        Grid::new()
            .cols([1.0, 0.25, 1.0, 0.25])
            .rows([1.0, 0.3, 1.0, 0.3])
            .col_gaps([em(0.4), em(1.6), em(0.4)])
            .row_gaps([em(0.4), em(1.6), em(0.4)]),
    );
    for r in 0..4 {
        for c in 0..4 {
            fig.panel(r, c).axis(axis::left(y)).axis(axis::bottom(x));
        }
    }
    let size = Size::new(960.0, 760.0);
    let target = Target::File { scale: 2.0 };
    let list = fig.draw_with(&mut cache(), size, target, &Held::default());
    let (xs, ys) = fig
        .resolve(size, &mut cache(), &Held::default(), target)
        .edges();
    assert_eq!((xs.len(), ys.len()), (10, 10));
    let cells = list::cells(&list, &xs, &ys, 2.0);
    assert_eq!((cells.cols.len(), cells.rows.len()), (10, 10));
    assert_eq!(cells.cells.len(), 81);
    assert_eq!(*cells.cols.last().expect("edges"), 1920);
    assert_eq!(*cells.rows.last().expect("edges"), 1520);
}
