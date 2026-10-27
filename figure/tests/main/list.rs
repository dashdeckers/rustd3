//! The list's types: `check`'s errors, the stroke defaults, `Bez`,
//! bounds, outlines and the dump.

use crate::sink;

use std::f64::consts::{FRAC_PI_2, TAU};
use std::sync::Arc;

use rustd3_core::path::PathContext;
use rustd3_figure::kurbo::{Affine, BezPath, Cap, Join, PathEl, Point, Rect, Shape, Size};
use rustd3_figure::list::{
    self, Bounds, Clip, Command, Error, Glyph, GlyphRun, List, Outlines, Paints, Part, part,
};
use rustd3_figure::path;
use rustd3_figure::peniko::{
    Blob, Brush, Color, FontData, Gradient, ImageAlphaType, ImageData, ImageFormat,
};

fn red() -> Brush {
    Brush::Solid(Color::from_rgb8(255, 0, 0))
}

fn square() -> BezPath {
    Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1)
}

fn fill() -> Command {
    Command::Fill {
        path: square(),
        paint: red(),
    }
}

fn push() -> Command {
    Command::Push {
        transform: Affine::IDENTITY,
        clip: None,
    }
}

fn markers(points: usize) -> Command {
    Command::Markers {
        shape: square(),
        at: vec![Point::ORIGIN; points],
        paint: red(),
        sizes: None,
        paints: None,
    }
}

fn image(width: u32, height: u32, bytes: usize) -> ImageData {
    ImageData {
        data: Blob::new(Arc::new(vec![0_u8; bytes])),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    }
}

fn with(commands: Vec<Command>) -> List {
    List {
        size: Size::new(100.0, 100.0),
        commands,
        ..List::default()
    }
}

fn part(commands: std::ops::Range<usize>) -> Part {
    Part {
        id: part::Id::new(7),
        commands,
        bounds: Rect::ZERO,
        hash: 0,
    }
}

fn run(font: usize) -> GlyphRun {
    GlyphRun {
        font,
        size: 11.0,
        transform: Affine::IDENTITY,
        fill: red(),
        glyphs: vec![Glyph {
            id: 3,
            x: 0.0,
            y: 0.0,
            advance: 3.5,
            text: 0..1,
        }],
        text: Arc::from(" "),
    }
}

#[test]
fn a_balanced_list_with_every_command_passes() {
    let mut list = with(vec![
        fill(),
        push(),
        markers(2),
        Command::Glyphs(run(0)),
        Command::Image {
            rect: Rect::new(0.0, 0.0, 2.0, 2.0),
            image: image(2, 2, 16),
            smooth: false,
        },
        Command::Pop,
    ]);
    list.fonts = vec![FontData::new(Blob::new(Arc::new(Vec::<u8>::new())), 0)];
    list.parts = vec![part(0..1), part(1..6), part(6..6)];
    assert_eq!(list::check(&list), Ok(()));
}

#[test]
fn an_open_group_is_unbalanced() {
    let list = with(vec![push(), push(), Command::Pop]);
    assert_eq!(list::check(&list), Err(Error::Unbalanced { open: 1 }));
}

#[test]
fn a_pop_without_a_push_is_unmatched() {
    let list = with(vec![fill(), Command::Pop]);
    assert_eq!(list::check(&list), Err(Error::UnmatchedPop { command: 1 }));
}

#[test]
fn a_run_outside_the_font_table_is_rejected() {
    let list = with(vec![fill(), Command::Glyphs(run(1))]);
    assert_eq!(
        list::check(&list),
        Err(Error::Font {
            command: 1,
            font: 1,
            fonts: 0
        })
    );
}

#[test]
fn sizes_shorter_than_the_points_are_rejected() {
    let mut command = markers(3);
    if let Command::Markers { sizes, .. } = &mut command {
        *sizes = Some(vec![1.0, 2.0]);
    }
    assert_eq!(
        list::check(&with(vec![command])),
        Err(Error::ChannelLength {
            command: 0,
            expected: 3,
            found: 2
        })
    );
}

#[test]
fn paint_indices_longer_than_the_points_are_rejected() {
    let mut command = markers(2);
    if let Command::Markers { paints, .. } = &mut command {
        *paints = Some(Paints {
            table: vec![red()],
            index: vec![0, 0, 0],
        });
    }
    assert_eq!(
        list::check(&with(vec![fill(), command])),
        Err(Error::ChannelLength {
            command: 1,
            expected: 2,
            found: 3
        })
    );
}

#[test]
fn a_paint_index_outside_the_table_is_rejected() {
    let mut command = markers(3);
    if let Command::Markers { paints, .. } = &mut command {
        *paints = Some(Paints {
            table: vec![red(), red()],
            index: vec![0, 1, 2],
        });
    }
    assert_eq!(
        list::check(&with(vec![command])),
        Err(Error::PaintIndex {
            command: 0,
            point: 2,
            index: 2,
            table: 2
        })
    );
}

#[test]
fn image_data_of_the_wrong_length_is_rejected() {
    let list = with(vec![Command::Image {
        rect: Rect::new(0.0, 0.0, 2.0, 2.0),
        image: image(2, 3, 16),
        smooth: true,
    }]);
    assert_eq!(
        list::check(&list),
        Err(Error::ImageLength {
            command: 0,
            width: 2,
            height: 3,
            expected: 24,
            found: 16
        })
    );
}

#[test]
fn an_image_brush_is_checked_as_an_image() {
    let list = with(vec![Command::Fill {
        path: square(),
        paint: Brush::Image(image(1, 1, 3).into()),
    }]);
    assert!(matches!(
        list::check(&list),
        Err(Error::ImageLength {
            expected: 4,
            found: 3,
            ..
        })
    ));
}

#[test]
fn a_non_finite_coordinate_is_rejected_with_its_value() {
    let mut path = square();
    path.line_to((f64::NAN, 1.0));
    let list = with(vec![fill(), Command::Fill { path, paint: red() }]);
    assert!(matches!(
        list::check(&list),
        Err(Error::NonFinite { command: 1, value }) if value.is_nan()
    ));
    let clip = with(vec![
        Command::Push {
            transform: Affine::IDENTITY,
            clip: Some(Clip::Rect(Rect::new(0.0, 0.0, f64::INFINITY, 1.0))),
        },
        Command::Pop,
    ]);
    assert_eq!(
        list::check(&clip),
        Err(Error::NonFinite {
            command: 0,
            value: f64::INFINITY
        })
    );
    let gradient = Gradient::new_linear((0.0, 0.0), (f64::NEG_INFINITY, 0.0));
    let paint = with(vec![Command::Fill {
        path: square(),
        paint: Brush::Gradient(gradient),
    }]);
    assert_eq!(
        list::check(&paint),
        Err(Error::NonFinite {
            command: 0,
            value: f64::NEG_INFINITY
        })
    );
}

#[test]
fn a_part_past_the_commands_is_out_of_range() {
    let mut list = with(vec![fill()]);
    list.parts = vec![part(0..2)];
    assert_eq!(
        list::check(&list),
        Err(Error::PartRange {
            part: 0,
            start: 0,
            end: 2,
            len: 1
        })
    );
}

#[test]
fn overlapping_parts_are_out_of_order() {
    let mut list = with(vec![fill(), fill(), fill()]);
    list.parts = vec![part(0..2), part(1..3)];
    assert_eq!(
        list::check(&list),
        Err(Error::PartOrder {
            part: 1,
            start: 1,
            previous: 2
        })
    );
}

#[test]
fn a_part_that_splits_a_group_is_unbalanced() {
    let mut list = with(vec![push(), fill(), Command::Pop]);
    list.parts = vec![part(0..2), part(2..3)];
    assert_eq!(
        list::check(&list),
        Err(Error::PartUnbalanced {
            part: 0,
            command: 2
        })
    );
    list.parts = vec![part(1..3)];
    assert_eq!(
        list::check(&list),
        Err(Error::PartUnbalanced {
            part: 0,
            command: 2
        })
    );
}

#[test]
fn strokes_have_butt_caps_and_miter_joins_limited_at_four() {
    let style = list::stroke(1.5);
    assert_eq!(style.width, 1.5);
    assert_eq!((style.start_cap, style.end_cap), (Cap::Butt, Cap::Butt));
    assert_eq!((style.join, style.miter_limit), (Join::Miter, 4.0));
    assert!(style.dash_pattern.is_empty());
}

#[test]
fn dejavu_sans_shapes_the_title_from_its_cmap() {
    let face = ttf_parser::Face::parse(sink::DEJAVU, 0).unwrap();
    assert_eq!(face.units_per_em(), 2048);
    let glyphs = sink::shape("Wheel: RL", 11.0);
    let ids: Vec<u32> = glyphs.iter().map(|g| g.id).collect();
    assert_eq!(ids, [58, 75, 72, 72, 79, 29, 3, 53, 47]);
    let advances: Vec<u16> = ids
        .iter()
        .map(|&id| {
            face.glyph_hor_advance(ttf_parser::GlyphId(u16::try_from(id).unwrap()))
                .unwrap()
        })
        .collect();
    assert_eq!(
        advances,
        [2025, 1298, 1260, 1260, 569, 690, 651, 1423, 1141]
    );
    assert!(
        (glyphs[0].advance - 10.877).abs() < 1e-3,
        "{}",
        glyphs[0].advance
    );
    assert_eq!(glyphs[1].x, glyphs[0].advance);
    assert_eq!(glyphs[5].text, 5..6);
}

#[test]
fn the_kitchen_sink_passes_check() {
    assert_eq!(list::check(&sink::list()), Ok(()));
    assert_eq!(list::check(&sink::with_sweep()), Ok(()));
}

#[test]
fn list_dump_of_the_kitchen_sink() {
    let dump = list::dump(&sink::list(), list::dump::Options { glyphs: true });
    snapshots().check("list_dump_of_the_kitchen_sink.txt", &dump);
}

#[test]
fn a_dump_without_glyphs_prints_font_size_and_text() {
    let dump = list::dump(&sink::list(), list::dump::Options { glyphs: false });
    let line = dump
        .lines()
        .find(|l| l.contains("glyphs"))
        .expect("a glyph run");
    assert!(line.ends_with(r#" glyphs font0 11 "Wheel: RL""#), "{line}");
}

fn snapshots() -> rustd3_testkit::snapshot::Store {
    rustd3_testkit::snapshot::Store::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots"))
}

#[test]
fn an_arc_after_a_point_lines_to_its_start_and_ends_on_the_axis() {
    let p = path::bez(|p| {
        p.move_to(0.0, 0.0);
        p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, false);
    });
    let els = p.elements();
    assert_eq!(els[0], PathEl::MoveTo(Point::new(0.0, 0.0)));
    assert_eq!(els[1], PathEl::LineTo(Point::new(10.0, 0.0)));
    assert!(matches!(els[2], PathEl::CurveTo(..)));
    assert!(
        matches!(els[els.len() - 1], PathEl::CurveTo(_, _, end) if end == Point::new(10.0 * FRAC_PI_2.cos(), 10.0))
    );
    let moves = els
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_)))
        .count();
    assert_eq!(moves, 1, "one subpath");
}

#[test]
fn a_full_circle_starts_and_ends_at_its_first_point() {
    let p = path::bez(|p| p.arc(0.0, 0.0, 10.0, 0.0, TAU, false));
    let els = p.elements();
    assert_eq!(els[0], PathEl::MoveTo(Point::new(10.0, 0.0)));
    assert_eq!(
        els.last().and_then(|e| e.end_point()),
        Some(Point::new(10.0, 0.0))
    );
    let b = p.bounding_box();
    assert!(
        (b.x0 + 10.0).abs() < 1e-3 && (b.y1 - 10.0).abs() < 1e-3,
        "{b:?}"
    );
}

#[test]
fn an_arc_from_the_current_point_draws_no_line() {
    let p = path::bez(|p| {
        p.move_to(10.0, 0.0);
        p.arc(0.0, 0.0, 10.0, 0.0, 1.0, false);
    });
    assert!(!p.elements().iter().any(|e| matches!(e, PathEl::LineTo(_))));
    let near = path::bez(|p| {
        p.move_to(10.0 + 5e-7, 0.0);
        p.arc(0.0, 0.0, 10.0, 0.0, 1.0, false);
    });
    assert!(
        !near
            .elements()
            .iter()
            .any(|e| matches!(e, PathEl::LineTo(_)))
    );
}

#[test]
fn a_negative_radius_draws_nothing_and_a_zero_one_only_its_start() {
    let negative = path::bez(|p| p.arc(0.0, 0.0, -1.0, 0.0, 1.0, false));
    assert!(negative.elements().is_empty());
    let zero = path::bez(|p| p.arc(5.0, 5.0, 0.0, 0.0, 1.0, false));
    assert_eq!(zero.elements(), [PathEl::MoveTo(Point::new(5.0, 5.0))]);
}

#[test]
fn an_anticlockwise_arc_sweeps_the_other_way() {
    let p = path::bez(|p| p.arc(0.0, 0.0, 10.0, 0.0, FRAC_PI_2, true));
    let b = p.bounding_box();
    assert!(
        b.y0 < -9.99 && b.x0 < -9.99,
        "three quarters, through the top: {b:?}"
    );
}

#[test]
fn a_segment_with_no_subpath_starts_one() {
    let p = path::bez(|p| {
        p.line_to(3.0, 4.0);
        p.line_to(5.0, 6.0);
    });
    assert_eq!(
        p.elements(),
        [
            PathEl::MoveTo(Point::new(3.0, 4.0)),
            PathEl::LineTo(Point::new(5.0, 6.0))
        ]
    );
    let closed = path::bez(|p| {
        p.move_to(0.0, 0.0);
        p.line_to(1.0, 0.0);
        p.close_path();
        p.close_path();
    });
    assert_eq!(closed.elements().len(), 4, "close returns to the start");
}

fn bounds(commands: Vec<Command>) -> Bounds {
    let mut list = with(commands);
    list.fonts = sink::fonts();
    Bounds::of(&list, &mut Outlines::default())
}

#[test]
fn bounds_follow_the_group_transforms_and_clips() {
    let b = bounds(vec![
        Command::Push {
            transform: Affine::translate((40.0, 20.0)),
            clip: Some(Clip::Rect(Rect::new(0.0, 0.0, 8.0, 100.0))),
        },
        fill(),
        Command::Push {
            transform: Affine::scale(2.0),
            clip: None,
        },
        fill(),
        Command::Pop,
        Command::Pop,
        fill(),
    ]);
    assert_eq!(b.len(), 7);
    assert_eq!(b.get(0), None);
    assert_eq!(b.get(1), Some(Rect::new(40.0, 20.0, 48.0, 30.0)));
    assert_eq!(b.get(3), Some(Rect::new(40.0, 20.0, 48.0, 40.0)));
    assert_eq!((b.get(4), b.get(5)), (None, None));
    assert_eq!(b.get(6), Some(Rect::new(0.0, 0.0, 10.0, 10.0)));
    assert_eq!(b.get(7), None);
}

#[test]
fn a_command_clipped_away_or_drawing_nothing_has_no_bounds() {
    let b = bounds(vec![
        Command::Push {
            transform: Affine::IDENTITY,
            clip: Some(Clip::Rect(Rect::new(50.0, 50.0, 60.0, 60.0))),
        },
        fill(),
        Command::Pop,
        Command::Fill {
            path: BezPath::new(),
            paint: red(),
        },
        markers(0),
    ]);
    assert_eq!((b.get(1), b.get(3), b.get(4)), (None, None, None));
}

#[test]
fn stroke_bounds_reach_past_the_path_for_joins_and_caps() {
    let line = || {
        let mut p = BezPath::new();
        p.move_to((10.0, 10.0));
        p.line_to((20.0, 10.0));
        p
    };
    let stroke = |style| Command::Stroke {
        path: line(),
        style,
        paint: red(),
    };
    let b = bounds(vec![
        stroke(list::stroke(2.0)),
        stroke(list::stroke(2.0).with_join(Join::Round)),
        stroke(
            list::stroke(2.0)
                .with_join(Join::Bevel)
                .with_caps(Cap::Square),
        ),
    ]);
    assert_eq!(b.get(0), Some(Rect::new(6.0, 6.0, 24.0, 14.0)), "miter 4");
    assert_eq!(
        b.get(1),
        Some(Rect::new(9.0, 9.0, 21.0, 11.0)),
        "half the width"
    );
    let square = b.get(2).unwrap();
    assert!((square.x0 - (10.0 - std::f64::consts::SQRT_2)).abs() < 1e-12);
}

#[test]
fn marker_bounds_are_the_union_of_the_scaled_shapes() {
    let b = bounds(vec![Command::Markers {
        shape: Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1),
        at: vec![Point::new(10.0, 10.0), Point::new(50.0, 20.0)],
        paint: red(),
        sizes: Some(vec![4.0, 10.0]),
        paints: None,
    }]);
    assert_eq!(b.get(0), Some(Rect::new(8.0, 8.0, 55.0, 25.0)));
}

#[test]
fn glyph_bounds_are_the_placed_outlines() {
    let title = sink::run("Wheel: RL", 11.0, Affine::translate((40.0, 270.0)), red());
    let space = sink::run(" ", 11.0, Affine::IDENTITY, red());
    let b = bounds(vec![Command::Glyphs(title), Command::Glyphs(space)]);
    let r = b.get(0).unwrap();
    assert!(
        r.x0 > 40.0 && r.x0 < 41.0,
        "W starts at its pen position: {r:?}"
    );
    assert!(r.y1 > 269.0 && r.y1 <= 270.5, "on the baseline: {r:?}");
    assert!(
        r.y0 > 270.0 - 11.0 && r.y0 < 262.0,
        "cap height below the em: {r:?}"
    );
    assert_eq!(b.get(1), None, "a space has no outline");
}

#[test]
fn outlines_are_in_em_units_with_y_down() {
    let fonts = sink::fonts();
    let mut outlines = Outlines::default();
    let w = outlines.get(&fonts[0], 58).unwrap().bounding_box();
    assert!(w.y1 <= 0.0 && w.y0 < -0.7 && w.y0 > -0.8, "{w:?}");
    assert!(outlines.get(&fonts[0], 3).is_none(), "the space");
    assert!(outlines.get(&fonts[0], 70_000).is_none());
}
