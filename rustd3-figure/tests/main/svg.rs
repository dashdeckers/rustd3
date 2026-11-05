//! The SVG writer: the kitchen sink's snapshot and the writer's spot
//! checks.

use std::f64::consts::TAU;

use rustd3_figure::kurbo::{Affine, Point, Rect, Shape, Size};
use rustd3_figure::list::{self, Clip, Command, Error, List};
use rustd3_figure::peniko::{Brush, Color, Gradient};
use rustd3_figure::svg;

use crate::sink;

fn write(list: &List) -> Result<String, Error> {
    svg::write(list, &svg::Options::default())
}

fn with(commands: Vec<Command>) -> List {
    List {
        size: Size::new(10.0, 10.0),
        commands,
        ..List::default()
    }
}

fn markers() -> Command {
    Command::Markers {
        shape: Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1),
        at: vec![Point::new(1.0, 2.0)],
        paint: sink::rgb(0x55a868),
        sizes: None,
        paints: None,
    }
}

#[test]
fn svg_of_the_kitchen_sink() {
    let svg = write(&sink::list()).unwrap();
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("svg_of_the_kitchen_sink.txt", &svg);
}

#[test]
fn a_filled_rect_is_one_path() {
    let svg = write(&with(vec![Command::Fill {
        path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
        paint: sink::rgb(0x4c72b0),
    }]))
    .unwrap();
    assert_eq!(
        svg,
        "<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' \
         width='10' height='10' viewBox='0 0 10 10'>\
         <path d='M0,0L10,0L10,10L0,10Z' fill='#4c72b0'/></svg>\n"
    );
}

#[test]
fn ids_follow_command_order_per_kind() {
    let svg = write(&with(vec![
        markers(),
        markers(),
        Command::Push {
            transform: Affine::IDENTITY,
            clip: Some(Clip::Path(Rect::new(0.0, 0.0, 5.0, 5.0).to_path(0.1))),
        },
        Command::Pop,
    ]))
    .unwrap();
    let (m1, m2, c1) = (
        svg.find("id='m1'").unwrap(),
        svg.find("id='m2'").unwrap(),
        svg.find("id='c1'").unwrap(),
    );
    assert!(m1 < m2 && m2 < c1, "{svg}");
}

#[test]
fn the_same_list_writes_the_same_file() {
    assert_eq!(write(&sink::list()), write(&sink::list()));
}

#[test]
fn a_sweep_gradient_needs_lowering_first() {
    let sweep = Gradient::new_sweep((5.0, 5.0), 0.0, TAU as f32)
        .with_stops([(0.0, Color::BLACK), (1.0, Color::WHITE)]);
    let list = with(vec![
        markers(),
        Command::Fill {
            path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
            paint: Brush::Gradient(sweep),
        },
    ]);
    assert_eq!(
        write(&list),
        Err(Error::Unsupported {
            command: 1,
            what: "a sweep gradient"
        })
    );
    assert!(write(&sink::with_sweep()).is_err());
}

#[test]
fn a_list_that_fails_check_is_not_written() {
    assert_eq!(
        write(&with(vec![Command::Pop])),
        Err(Error::UnmatchedPop { command: 0 })
    );
}

#[test]
fn per_point_paints_group_consecutive_points() {
    let svg = write(&sink::list()).unwrap();
    let start = svg.find("<defs><path id='m1'").unwrap();
    let markers = &svg[start..svg[start..].find("<defs><path id='m2'").unwrap() + start];
    assert_eq!(markers.matches("<g fill=").count(), 3, "{markers}");
    assert_eq!(markers.matches("<use ").count(), 3);
    assert!(
        markers.contains("transform='translate(150 60) scale(6)'"),
        "{markers}"
    );
}

#[test]
fn gradient_markers_are_paths_in_their_groups_space() {
    let svg = write(&sink::list()).unwrap();
    let start = svg.find("<defs><path id='m2'").unwrap();
    let rest = &svg[start..];
    let group = &rest[..rest.find("</g>").unwrap()];
    assert!(group.contains("fill='url(#p2)'"), "{group}");
    assert!(
        group.contains("<path d='M34,94L46,94L46,106L34,106Z'/>"),
        "{group}"
    );
    assert!(!group.contains("<use "));
}

#[test]
fn runs_use_glyph_outlines_at_1000_units_per_em() {
    let svg = write(&sink::list()).unwrap();
    assert!(svg.contains("<path id='g0-58' d='M"), "the W is defined");
    assert_eq!(svg.matches("id='g0-58'").count(), 1);
    assert!(
        svg.contains("<g transform='matrix(1 0 0 1 40 270) scale(0.011)' fill='#262626'><use xlink:href='#g0-58' x='0' y='0'/>"),
        "{svg}"
    );
    assert!(!svg.contains("#g0-3'"), "a space has no outline to use");
}

#[test]
fn images_are_base64_pngs_and_nearest_ones_are_pixelated() {
    let svg = write(&sink::list()).unwrap();
    assert!(svg.contains(
        "<image id='i1' x='272' y='8' width='32' height='32' preserveAspectRatio='none' \
         style='image-rendering:pixelated' href='data:image/png;base64,iVBORw0KGgo"
    ));
    assert!(svg.contains(
        "<image id='i2' x='200' y='180' width='48' height='32' preserveAspectRatio='none' href="
    ));
}

#[test]
fn an_image_brush_is_not_written() {
    let list = with(vec![Command::Fill {
        path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
        paint: Brush::Image(sink::image(1, 1, vec![0, 0, 0, 255]).into()),
    }]);
    assert_eq!(
        write(&list),
        Err(Error::Unsupported {
            command: 0,
            what: "an image brush"
        })
    );
}

#[test]
fn transparent_paints_carry_their_opacity() {
    let list = with(vec![Command::Stroke {
        path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
        style: list::stroke(2.0).with_dashes(1.0, [4.0, 2.0]),
        paint: Brush::Solid(Color::from_rgba8(0, 0, 255, 128)),
    }]);
    let svg = write(&list).unwrap();
    assert!(
        svg.contains(
            "fill='none' stroke='#0000ff' stroke-opacity='0.502' stroke-width='2' \
             stroke-dasharray='4 2' stroke-dashoffset='1'/>"
        ),
        "{svg}"
    );
}

#[test]
fn a_run_whose_glyphs_are_not_its_characters_has_one_x() {
    let mut run = sink::run("ffi", 10.0, Affine::IDENTITY, sink::rgb(0));
    run.glyphs.truncate(2);
    run.glyphs[1].text = 1..3;
    let list = List {
        size: Size::new(10.0, 10.0),
        fonts: sink::fonts(),
        commands: vec![Command::Glyphs(run)],
        ..List::default()
    };
    let svg = svg::write(&list, &svg::Options::default().text(svg::Text::Elements)).unwrap();
    assert!(
        svg.contains("<text transform='matrix(1 0 0 1 0 0)' x='0' y='0' font-family='DejaVu Sans' font-size='10' fill='#000000' xml:space='preserve'>ffi</text>"),
        "{svg}"
    );
}

#[test]
fn text_elements_escape_markup_characters() {
    let list = List {
        size: Size::new(10.0, 10.0),
        fonts: sink::fonts(),
        commands: vec![Command::Glyphs(sink::run(
            "a<&>'b",
            10.0,
            Affine::IDENTITY,
            sink::rgb(0),
        ))],
        ..List::default()
    };
    let svg = svg::write(&list, &svg::Options::default().text(svg::Text::Elements)).unwrap();
    assert!(svg.contains(">a&lt;&amp;&gt;&apos;b</text>"), "{svg}");
}
