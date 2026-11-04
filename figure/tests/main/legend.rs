//! Legends: entries in domain order, rows in top and bottom slots centred on
//! the plot areas, columns beside the grid and framed in panel corners, and
//! the room slots take from the grid.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Ordinal};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{Rect, Shape, Size};
use rustd3_figure::legend::{Inset, Legend, Slot, Swatch, legend};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held, Target, axis, mark};

use crate::figure::{cache, line};

/// The tick-label size at notebook, px: legend em.
const EM: f64 = 44.0 / 3.0;

/// The canvas padding, px.
const PAD: f64 = 0.8 * EM;

/// DejaVu Sans's typographic ascender plus descender (1556 + 492 of 2048
/// units) at `size` px: a line box.
fn line_box(size: f64) -> f64 {
    let [ascender, descender] = [1556.0, 492.0].map(|units| units / 2048.0 * size);
    ascender + descender
}

/// A one-panel figure, x and y fitted to 0 … 10 with axes, and a palette
/// over three sources.
fn sources() -> (Figure<'static>, ScaleRef<Ordinal<&'static str, Rgb>>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(
        ["accel x", "accel z", "bearing temp"],
        COLORBLIND,
    ));
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .mark(line(y, [0.0, 10.0]))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    (fig, color)
}

/// `fig` drawn at 640 × 480 for vector output.
fn draw(fig: &Figure<'_>) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    )
}

/// The plot area of `fig`'s first panel at 640 × 480.
fn area(fig: &Figure<'_>) -> Rect {
    crate::figure::resolve(fig, &Held::default()).get_plot_areas()[0]
}

/// The commands of the part with id `id`.
fn commands(list: &List, id: part::Id) -> &[Command] {
    let p = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .unwrap_or_else(|| panic!("no part {id:?}"));
    &list.commands[p.commands.clone()]
}

/// The id of legend `index` in figure slots.
fn slotted(index: u64) -> part::Id {
    part::Id::new(7 << 56 | 0xff_ffff << 32 | index)
}

/// The id of inset legend `index` of panel `panel`.
fn inset(panel: u64, index: u64) -> part::Id {
    part::Id::new(7 << 56 | panel << 32 | index)
}

/// The rectangles and solid colors of the fills among `commands`.
fn fills(commands: &[Command]) -> Vec<(Rect, Brush)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Fill { path, paint } = c {
                Some((path.bounding_box(), paint.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// The texts and baselines (run origins) of the glyph runs among
/// `commands`.
fn labels(commands: &[Command]) -> Vec<(String, f64, f64)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                let t = run.transform.translation();
                Some((run.text.to_string(), t.x, t.y))
            } else {
                None
            }
        })
        .collect()
}

/// An opaque solid brush of `hex`.
fn solid(hex: u32) -> Brush {
    let [_, r, g, b] = hex.to_be_bytes();
    Brush::Solid(Color::from_rgba8(r, g, b, 255))
}

#[test]
fn a_legend_has_one_square_per_key_in_domain_order() {
    let (mut fig, color) = sources();
    fig.legend(Slot::Bottom, legend(color));
    let list = draw(&fig);
    let swatches = fills(commands(&list, slotted(0)));
    assert_eq!(swatches.len(), 3);
    let colors: Vec<Brush> = swatches.iter().map(|(_, b)| b.clone()).collect();
    assert_eq!(colors, [solid(0x0173b2), solid(0xde8f05), solid(0x029e73)]);
    for (rect, _) in &swatches {
        let side = [rect.width(), rect.height()].map(|v| format!("{v:.3}"));
        assert_eq!(side, ["10.267", "10.267"]);
    }
    let texts: Vec<String> = labels(commands(&list, slotted(0)))
        .into_iter()
        .map(|(t, ..)| t)
        .collect();
    assert_eq!(texts, ["accel x", "accel z", "bearing temp"]);
}

#[test]
fn legend_glyphs_draw_dots_and_dashed_lines_in_the_squares_place() {
    let (mut fig, color) = sources();
    fig.legend(Slot::Bottom, legend(color))
        .legend(Slot::Bottom, legend(color).glyph(Swatch::Dot))
        .legend(
            Slot::Bottom,
            legend(color).glyph(Swatch::Line {
                dash: Some([6.0, 3.0]),
            }),
        )
        .legend(
            Slot::Bottom,
            legend(color).glyph(Swatch::Line { dash: None }),
        );
    let list = draw(&fig);
    let squares = fills(commands(&list, slotted(0)));
    let label_starts = |k: u64| -> Vec<(f64, f64)> {
        labels(commands(&list, slotted(k)))
            .into_iter()
            .map(|(_, x, y)| (x, y))
            .collect()
    };
    // Squares and dots stand in a box 0.7 em square on the baseline, 0.8
    // em before their labels; dots are the theme's marker diameter (6 pt,
    // 8 px), centred in it. Legends in one slot stack, each on its row.
    let dots = fills(commands(&list, slotted(1)));
    assert_eq!(dots.len(), 3);
    for (k, swatches) in [(0, &squares), (1, &dots)] {
        for ((swatch, paint), (x, y)) in swatches.iter().zip(label_starts(k)) {
            let center = swatch.center();
            assert!((center.y - (y - 0.35 * EM)).abs() < 1e-9, "{swatch:?} {y}");
            assert!(
                (x - (center.x + 0.35 * EM) - 0.8 * EM).abs() < 1e-9,
                "{x} {swatch:?}"
            );
            let side = if k == 0 { 0.7 * EM } else { 8.0 };
            assert!((swatch.width() - side).abs() < 2e-3, "{swatch:?}");
            assert!((swatch.height() - side).abs() < 2e-3, "{swatch:?}");
            assert!(squares.iter().any(|(_, color)| color == paint));
        }
    }
    // Lines: 2 em long at the square's centre height, the line width (1.5
    // pt), dashed [6, 3] px or solid, labels 0.8 em after them.
    for (k, dash) in [(2, vec![6.0, 3.0]), (3, vec![])] {
        let lines: Vec<_> = commands(&list, slotted(k))
            .iter()
            .filter_map(|c| {
                if let Command::Stroke { path, style, paint } = c {
                    Some((path.bounding_box(), style.clone(), paint.clone()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(lines.len(), 3);
        let starts = label_starts(k);
        for (((bounds, style, paint), (_, color)), (x, y)) in
            lines.iter().zip(&squares).zip(&starts)
        {
            assert_eq!(paint, color);
            assert!((bounds.width() - 2.0 * EM).abs() < 1e-9 && bounds.height() == 0.0);
            assert!((bounds.y0 - (y - 0.35 * EM)).abs() < 1e-9, "{bounds:?} {y}");
            assert!((x - bounds.x1 - 0.8 * EM).abs() < 1e-9);
            assert_eq!(style.width, 1.5 * (4.0 / 3.0));
            assert_eq!(style.dash_pattern.as_slice(), dash.as_slice());
        }
    }
}

/// What each swatch among `commands` (a legend part's) draws: "line" for a
/// stroke, with its dash; "dot" for a fill 8 px across, the theme's marker;
/// "square" for one 0.7 em across; frames are left out.
fn glyphs(commands: &[Command]) -> Vec<(&'static str, Vec<f64>)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { style, .. } = c {
                return Some(("line", style.dash_pattern.to_vec()));
            }
            let Command::Fill { path, .. } = c else {
                return None;
            };
            let width = path.bounding_box().width();
            let kind = if (width - 8.0).abs() < 1e-3 {
                "dot"
            } else if (width - 0.7 * EM).abs() < 1e-3 {
                "square"
            } else {
                return None;
            };
            Some((kind, Vec::new()))
        })
        .collect()
}

#[test]
fn legends_draw_the_swatch_of_each_keys_marks_unless_told_otherwise() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let keys = ["model", "measured", "band", "trend", "unused"];
    let color = fig.scale(scale::ordinal(keys, COLORBLIND));
    let points = [(0.0, 1.0), (1.0, 2.0), (2.0, 1.5)];
    fig.panel(0, 0)
        .mark(
            mark::line(points, x, y)
                .stroke(color, "model")
                .dash([6.0, 3.0]),
        )
        .mark(mark::dots(points, x, y).fill(color, "measured"))
        .mark(mark::ribbon([(0.0, 0.5, 1.5), (2.0, 1.0, 2.0)], x, y).fill(color, "band"))
        .mark(mark::dots(points, x, y).fill(color, "trend"))
        .mark(mark::line(points, x, y).stroke(color, "trend"));
    fig.legend(Slot::Bottom, legend(color))
        .legend(Slot::Bottom, legend(color).glyph(Swatch::Square));
    let list = draw(&fig);
    // A line key draws a line with its dash, a dots key a dot, a ribbon key
    // and a key no mark uses a square; a key drawn as dots and a line
    // draws the line.
    let dashed = vec![6.0, 3.0];
    let expected = [
        ("line", dashed),
        ("dot", vec![]),
        ("square", vec![]),
        ("line", vec![]),
        ("square", vec![]),
    ];
    assert_eq!(glyphs(commands(&list, slotted(0))), expected);
    // `.glyph` sets every key's.
    let squares = vec![("square", Vec::new()); 5];
    assert_eq!(glyphs(commands(&list, slotted(1))), squares);
}

#[test]
fn legend_squares_repeat_the_opacity_their_areas_and_ribbons_fill_at() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let keys = ["band", "half", "area", "faint"];
    let palette = [
        Rgb::hex(0x0173b2),
        Rgb::hex(0xde8f05),
        Rgb::hex(0x029e73),
        Rgb::hex(0xd55e00).with_opacity(0.4),
    ];
    let color = fig.scale(scale::ordinal(keys, palette));
    let rows = [(0.0, 0.5, 1.5), (2.0, 1.0, 2.0)];
    let points = [(0.0, 1.0), (1.0, 2.0), (2.0, 1.5)];
    fig.panel(0, 0)
        // A band: an opaque color at 0.25; a line of the same key after it
        // leaves the square as the ribbon fills.
        .mark(mark::ribbon(rows, x, y).fill(color, "band"))
        .mark(mark::line(points, x, y).stroke(color, "band"))
        .mark(mark::ribbon(rows, x, y).fill(color, "half").opacity(0.5))
        .mark(mark::area(points, x, y).fill(color, "area"))
        // A translucent color: a band keeps its own opacity.
        .mark(mark::ribbon(rows, x, y).fill(color, "faint"));
    fig.legend(Slot::Bottom, legend(color));
    let list = draw(&fig);
    // Brushes hold 8-bit alpha: 0.25, 0.5, 1 and 0.4 of 255.
    let alphas: Vec<u8> = fills(commands(&list, slotted(0)))
        .into_iter()
        .map(|(_, brush)| {
            let Brush::Solid(c) = brush else {
                panic!("a solid square, not {brush:?}")
            };
            c.to_rgba8().a
        })
        .collect();
    assert_eq!(alphas, [64, 128, 255, 102]);
}

#[test]
fn inset_legends_in_one_corner_stack_inward_half_an_em_apart() {
    let (mut fig, color) = sources();
    let key = legend(color);
    fig.panel(0, 0)
        .legend(Inset::BottomRight, key.clone())
        .legend(Inset::TopLeft, key.clone())
        .legend(Inset::BottomRight, key.clone())
        .legend(Inset::TopLeft, key);
    let list = draw(&fig);
    let area = area(&fig);
    let frame = |index| fills(commands(&list, inset(0, index)))[0].0;
    let (low, high) = (frame(0), frame(2));
    assert!((low.y1 - (area.y1 - 0.5 * EM)).abs() < 1e-9, "{low:?}");
    assert!(
        (high.y1 - (low.y0 - 0.5 * EM)).abs() < 1e-9,
        "{high:?} {low:?}"
    );
    assert_eq!((low.x1, high.x1), (area.x1 - 0.5 * EM, area.x1 - 0.5 * EM));
    let (top, below) = (frame(1), frame(3));
    assert!((top.y0 - (area.y0 + 0.5 * EM)).abs() < 1e-9, "{top:?}");
    assert!(
        (below.y0 - (top.y1 + 0.5 * EM)).abs() < 1e-9,
        "{below:?} {top:?}"
    );
}

#[test]
fn a_bottom_slot_raises_the_grid_by_its_row_and_half_an_em() {
    let (bare, _) = sources();
    let (mut fig, color) = sources();
    fig.legend(Slot::Bottom, legend(color));
    let (before, after) = (area(&bare), area(&fig));
    let moved = before.y1 - after.y1;
    // The row: the labels' line box, which holds their ink and the swatch.
    let expected = line_box(EM) + 0.5 * EM;
    // The ink of "l" and "b" tops the ascender by 3e-5 px.
    assert!((moved - expected).abs() < 1e-4, "{moved} {expected}");
    assert_eq!(
        (before.x0, before.x1, before.y0),
        (after.x0, after.x1, after.y0)
    );
}

#[test]
fn one_legend_value_draws_in_a_slot_and_an_inset_and_the_row_centres_on_the_plot_areas() {
    let (mut fig, color) = sources();
    let key = legend(color).title("source");
    fig.panel(0, 0).legend(Inset::TopLeft, key.clone());
    fig.legend(Slot::Bottom, key);
    let list = draw(&fig);
    let area = area(&fig);
    let row = commands(&list, slotted(0));
    let corner = commands(&list, inset(0, 0));
    assert_eq!(labels(row).len(), 4);
    assert_eq!(labels(corner).len(), 4);
    // The row: centred on the plot area (its title's start and its last
    // label's advance end are equally far from the area's ends).
    let texts = labels(row);
    let (_, first_x, baseline) = texts[0].clone();
    assert!(texts.iter().all(|(_, _, y)| (y - baseline).abs() < 1e-9));
    let end = row
        .iter()
        .rev()
        .find_map(|c| {
            if let Command::Glyphs(run) = c {
                let last = run.glyphs.last()?;
                Some(run.transform.translation().x + f64::from(last.x + last.advance))
            } else {
                None
            }
        })
        .expect("a label");
    let (left, right) = (first_x - area.x0, area.x1 - end);
    assert!((left - right).abs() < 1e-4, "{left} {right}");
    // Its bottom (the line box under the baseline) on the padding.
    let descender = 492.0 / 2048.0 * 16.0;
    assert!(
        (baseline + descender - (480.0 - PAD)).abs() < 1e-4,
        "{baseline}"
    );
    // The inset: a frame 0.5 em inside the corner, white at 0.8 with a
    // #cccccc edge, the title centred on top, entries below it.
    let frame = fills(corner)[0].clone();
    assert!((frame.0.x0 - area.x0 - 0.5 * EM).abs() < 1e-9, "{frame:?}");
    assert!((frame.0.y0 - area.y0 - 0.5 * EM).abs() < 1e-9, "{frame:?}");
    assert_eq!(frame.1, Brush::Solid(Color::from_rgba8(255, 255, 255, 204)));
    let edge = corner.iter().find_map(|c| {
        if let Command::Stroke { paint, style, .. } = c {
            Some((paint.clone(), style.width))
        } else {
            None
        }
    });
    assert_eq!(edge, Some((solid(0xcccccc), 1.0)));
    let texts = labels(corner);
    assert_eq!(texts[0].0, "source");
    assert!(texts[1].2 > texts[0].2 && texts[2].2 > texts[1].2);
}

#[test]
fn column_rows_are_their_line_box_and_half_an_em_apart() {
    let (mut fig, color) = sources();
    fig.panel(0, 0).legend(Inset::BottomRight, legend(color));
    let list = draw(&fig);
    let corner = commands(&list, inset(0, 0));
    let baselines: Vec<f64> = labels(corner).into_iter().map(|(.., y)| y).collect();
    let pitch = line_box(EM) + 0.5 * EM;
    for pair in baselines.windows(2) {
        assert!((pair[1] - pair[0] - pitch).abs() < 1e-4, "{baselines:?}");
    }
    // Swatches stand on their baselines, 1.5 em before their labels.
    let swatches: Vec<Rect> = fills(corner).into_iter().skip(1).map(|(r, _)| r).collect();
    let starts: Vec<(f64, f64)> = labels(corner).into_iter().map(|(_, x, y)| (x, y)).collect();
    for (swatch, (x, y)) in swatches.iter().zip(&starts) {
        assert!((swatch.y1 - y).abs() < 1e-9 && (x - swatch.x0 - 1.5 * EM).abs() < 1e-9);
    }
    // The frame sits 0.5 em inside the bottom right corner.
    let area = area(&fig);
    let frame = fills(corner)[0].0;
    assert!((area.x1 - frame.x1 - 0.5 * EM).abs() < 1e-9, "{frame:?}");
    assert!((area.y1 - frame.y1 - 0.5 * EM).abs() < 1e-9, "{frame:?}");
}

#[test]
fn side_slots_hold_columns_centred_on_the_plot_areas_and_stack_outward() {
    let (mut fig, color) = sources();
    let key: Legend<&str> = legend(color);
    fig.legend(Slot::Right, key.clone())
        .legend(Slot::Right, key.clone().title("again"))
        .legend(Slot::Left, key);
    let list = draw(&fig);
    let area = area(&fig);
    let bounds = |id| {
        list.parts
            .iter()
            .find(|p| p.id == id)
            .expect("a legend")
            .bounds
    };
    let (near, far, left) = (bounds(slotted(0)), bounds(slotted(1)), bounds(slotted(2)));
    assert!(near.x0 > area.x1 && far.x0 > near.x1, "{near:?} {far:?}");
    assert!(far.x1 <= 640.0 - PAD + 1e-9, "{far:?}");
    assert!(left.x1 < area.x0 && left.x0 >= PAD - 1e-9, "{left:?}");
    // Centred vertically: the swatch column's middle on the area's.
    let swatches = fills(commands(&list, slotted(0)));
    let (top, bottom) = (swatches[0].0.y0, swatches[2].0.y1);
    let middle = (area.y0 + area.y1) / 2.0;
    assert!(((top + bottom) / 2.0 - middle).abs() < EM, "{top} {bottom}");
}

#[test]
fn a_top_slot_sits_below_the_figure_title_and_above_the_grid() {
    let (mut fig, color) = sources();
    fig.title("Sources").legend(Slot::Top, legend(color));
    let list = draw(&fig);
    let title = list
        .parts
        .iter()
        .find(|p| p.id == part::Id::new(1 << 56))
        .expect("a title")
        .bounds;
    let row = list
        .parts
        .iter()
        .find(|p| p.id == slotted(0))
        .expect("a legend")
        .bounds;
    let area = area(&fig);
    assert!(
        title.y1 < row.y0 && row.y1 < area.y0,
        "{title:?} {row:?} {area:?}"
    );
}

#[test]
fn legend_parts_follow_the_marks_of_their_panel_and_the_figure_title() {
    let (mut fig, color) = sources();
    fig.title("Sources")
        .legend(Slot::Bottom, legend(color))
        .panel(0, 0)
        .legend(Inset::TopRight, legend(color));
    let list = draw(&fig);
    let ids: Vec<part::Id> = list.parts.iter().map(|p| p.id).collect();
    let at = |id| ids.iter().position(|i| *i == id).expect("a part");
    assert!(
        at(inset(0, 0)) < at(part::Id::new(4 << 56)),
        "before the axes"
    );
    assert_eq!(at(slotted(0)), ids.len() - 1, "last, after the title");
    let first = &commands(&list, inset(0, 0))[0];
    let Command::Fill { path, .. } = first else {
        panic!("a frame first: {first:?}");
    };
    assert!(path.bounding_box().area() > 0.0);
}
