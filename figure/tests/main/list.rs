//! The list's types: `check`'s errors, the stroke defaults.

use std::sync::Arc;

use rustd3_figure::kurbo::{Affine, BezPath, Cap, Join, Point, Rect, Shape, Size};
use rustd3_figure::list::{self, Clip, Command, Error, Glyph, GlyphRun, List, Paints, Part, part};
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
