//! The kitchen sink: a hand-built list with every command, paint and clip
//! the backends must agree on, and its font, shaped naively through
//! ttf-parser's cmap (typst arrives in Phase 13).

use std::f64::consts::{FRAC_PI_2, TAU};
use std::sync::Arc;

use rustd3_core::path::PathContext;
use rustd3_figure::kurbo::{Affine, BezPath, Point, Rect, Shape, Size};
use rustd3_figure::list::{
    self, Bounds, Clip, Command, Glyph, GlyphRun, List, Outlines, Paints, Part, part,
};
use rustd3_figure::path;
use rustd3_figure::peniko::{
    Blob, Brush, Color, FontData, Gradient, ImageAlphaType, ImageData, ImageFormat,
};

/// DejaVu Sans, read from the fonts crate's file so that the tests need no
/// feature.
pub const DEJAVU: &[u8] = include_bytes!("../../../fonts/DejaVuSans.ttf");

/// The list's font table: DejaVu Sans.
pub fn fonts() -> Vec<FontData> {
    vec![FontData::new(Blob::new(Arc::new(DEJAVU)), 0)]
}

/// `text` in DejaVu Sans at `size` px, one glyph per char from the cmap,
/// advances from `hmtx`.
pub fn shape(text: &str, size: f32) -> Vec<Glyph> {
    let face = ttf_parser::Face::parse(DEJAVU, 0).expect("DejaVu Sans parses");
    let k = size / f32::from(face.units_per_em());
    let mut x = 0.0;
    text.char_indices()
        .map(|(i, c)| {
            let id = face.glyph_index(c).expect("a glyph for every char");
            let advance = f32::from(face.glyph_hor_advance(id).expect("an advance")) * k;
            let start = u32::try_from(i).expect("short text");
            let end = start + u32::try_from(c.len_utf8()).expect("a char");
            let glyph = Glyph {
                id: u32::from(id.0),
                x,
                y: 0.0,
                advance,
                text: start..end,
            };
            x += advance;
            glyph
        })
        .collect()
}

/// A glyph run of `text` at `size` px placed by `transform`.
pub fn run(text: &str, size: f32, transform: Affine, fill: Brush) -> GlyphRun {
    GlyphRun {
        font: 0,
        size,
        transform,
        fill,
        glyphs: shape(text, size),
        text: Arc::from(text),
    }
}

/// An opaque color.
pub fn rgb(hex: u32) -> Brush {
    let [_, r, g, b] = hex.to_be_bytes();
    Brush::Solid(Color::from_rgb8(r, g, b))
}

/// Straight RGBA pixels as image data.
pub fn image(width: u32, height: u32, rgba: Vec<u8>) -> ImageData {
    ImageData {
        data: Blob::new(Arc::new(rgba)),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    }
}

/// A 16 × 16 checkerboard of single texels, opaque.
fn checkerboard() -> ImageData {
    let rgba = (0..16 * 16)
        .flat_map(|i| {
            if (i % 16 + i / 16) % 2 == 0 {
                [0x4c, 0x72, 0xb0, 0xff]
            } else {
                [0xf0, 0xf0, 0xf0, 0xff]
            }
        })
        .collect();
    image(16, 16, rgba)
}

/// A 4 × 4 ramp of low contrast and partial alpha, for smooth sampling.
fn ramp() -> ImageData {
    let rgba = (0..16_u8)
        .flat_map(|i| [0xc0 + 2 * (i % 4), 0x90 + 3 * (i / 4), 0x60, 0xc8])
        .collect();
    image(4, 4, rgba)
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> BezPath {
    Rect::new(x0, y0, x1, y1).to_path(0.1)
}

/// The kitchen sink: background, a clipped and translated plot group with a
/// linear gradient, dashed, round and square strokes, sized and painted
/// markers, gradient markers, a radial gradient, a rotated group under a
/// path clip, a nearest and a smooth image, and two glyph runs, one
/// rotated; four parts. Without a sweep gradient, which SVG cannot write.
pub fn list() -> List {
    build(false)
}

/// The kitchen sink with a sweep-gradient disc in the plot group.
pub fn with_sweep() -> List {
    build(true)
}

fn build(sweep: bool) -> List {
    let ink = Color::from_rgb8(0x26, 0x26, 0x26);
    let mut commands = vec![Command::Fill {
        path: rect(0.0, 0.0, 400.0, 300.0),
        paint: rgb(0xffffff),
    }];
    let plot_start = commands.len();
    let wash = Gradient::new_linear((0.0, 0.0), (320.0, 0.0)).with_stops([
        (0.0, Color::from_rgb8(0xea, 0xea, 0xf2)),
        (1.0, Color::from_rgb8(0xfa, 0xfa, 0xfc)),
    ]);
    let circle = path::bez(|p| p.arc(0.0, 0.0, 0.5, 0.0, TAU, false));
    let polyline = |points: &[(f64, f64)]| {
        path::bez(|p| {
            p.move_to(points[0].0, points[0].1);
            for &(x, y) in &points[1..] {
                p.line_to(x, y);
            }
        })
    };
    commands.extend([
        Command::Push {
            transform: Affine::translate((40.0, 20.0)),
            clip: Some(Clip::Rect(Rect::new(0.0, 0.0, 320.0, 220.0))),
        },
        Command::Fill {
            path: rect(0.0, 0.0, 320.0, 220.0),
            paint: Brush::Gradient(wash),
        },
        Command::Stroke {
            path: polyline(&[(0.0, 110.0), (320.0, 110.0)]),
            style: list::stroke(1.5).with_dashes(0.0, [6.0, 3.0]),
            paint: rgb(0x4c72b0),
        },
        Command::Stroke {
            path: polyline(&[(20.0, 200.0), (80.0, 120.0), (140.0, 160.0), (200.0, 40.0)]),
            style: list::stroke(2.5)
                .with_caps(rustd3_figure::kurbo::Cap::Round)
                .with_join(rustd3_figure::kurbo::Join::Round),
            paint: rgb(0xdd8452),
        },
        Command::Stroke {
            path: polyline(&[(20.0, 30.0), (60.0, 60.0), (100.0, 20.0)]),
            style: list::stroke(3.0).with_caps(rustd3_figure::kurbo::Cap::Square),
            paint: rgb(0x8c8c8c),
        },
        Command::Markers {
            shape: circle,
            at: vec![
                Point::new(150.0, 60.0),
                Point::new(180.0, 80.0),
                Point::new(210.0, 60.0),
            ],
            paint: rgb(0x000000),
            sizes: Some(vec![6.0, 10.0, 14.0]),
            paints: Some(Paints {
                table: vec![rgb(0x55a868), rgb(0xc44e52)],
                index: vec![0, 1, 0],
            }),
        },
        Command::Markers {
            shape: rect(-0.5, -0.5, 0.5, 0.5),
            at: vec![
                Point::new(40.0, 100.0),
                Point::new(60.0, 100.0),
                Point::new(80.0, 100.0),
                Point::new(100.0, 100.0),
            ],
            paint: Brush::Gradient(Gradient::new_linear((40.0, 0.0), (100.0, 0.0)).with_stops([
                (0.0, Color::from_rgb8(0x64, 0xb5, 0xcd)),
                (1.0, Color::from_rgb8(0x2a, 0x4d, 0x8f)),
            ])),
            sizes: Some(vec![12.0; 4]),
            paints: None,
        },
        Command::Fill {
            path: path::bez(|p| p.arc(250.0, 150.0, 40.0, 0.0, TAU, false)),
            paint: Brush::Gradient(Gradient::new_radial((250.0, 150.0), 40.0).with_stops([
                (0.0, Color::from_rgb8(0x81, 0x72, 0xb3)),
                (1.0, Color::from_rgba8(0x81, 0x72, 0xb3, 0x40)),
            ])),
        },
        Command::Push {
            transform: Affine::rotate_about(0.3, (110.0, 170.0)),
            clip: Some(Clip::Path(polyline(&[
                (70.0, 200.0),
                (110.0, 130.0),
                (150.0, 200.0),
            ]))),
        },
        Command::Fill {
            path: rect(60.0, 140.0, 160.0, 210.0),
            paint: Brush::Solid(Color::from_rgba8(0x93, 0x78, 0x60, 0x99)),
        },
        Command::Pop,
        Command::Image {
            rect: Rect::new(272.0, 8.0, 304.0, 40.0),
            image: checkerboard(),
            smooth: false,
        },
        Command::Image {
            rect: Rect::new(200.0, 180.0, 248.0, 212.0),
            image: ramp(),
            smooth: true,
        },
    ]);
    if sweep {
        commands.push(Command::Fill {
            path: path::bez(|p| p.arc(290.0, 80.0, 24.0, 0.0, TAU, false)),
            paint: Brush::Gradient(
                Gradient::new_sweep((290.0, 80.0), 0.0, TAU as f32).with_stops([
                    (0.0, Color::from_rgb8(0xe0, 0x40, 0x40)),
                    (0.5, Color::from_rgb8(0x40, 0xe0, 0x40)),
                    (1.0, Color::from_rgb8(0x40, 0x40, 0xe0)),
                ]),
            ),
        });
    }
    commands.push(Command::Pop);
    let title_start = commands.len();
    commands.push(Command::Glyphs(run(
        "Wheel: RL",
        11.0,
        Affine::translate((40.0, 270.0)),
        Brush::Solid(ink),
    )));
    let label_start = commands.len();
    commands.push(Command::Glyphs(run(
        "Speed (km/h)",
        10.0,
        Affine::translate((24.0, 160.0)) * Affine::rotate(-FRAC_PI_2),
        Brush::Solid(ink),
    )));
    let mut list = List {
        size: Size::new(400.0, 300.0),
        fonts: fonts(),
        commands,
        parts: Vec::new(),
    };
    let ranges = [
        0..plot_start,
        plot_start..title_start,
        title_start..label_start,
        label_start..list.commands.len(),
    ];
    let bounds = Bounds::of(&list, &mut Outlines::default());
    list.parts = ranges
        .into_iter()
        .zip(1..)
        .map(|(commands, id)| Part {
            id: part::Id::new(id),
            bounds: commands
                .clone()
                .filter_map(|c| bounds.get(c))
                .reduce(|a, b| a.union(b))
                .unwrap_or(Rect::ZERO),
            commands,
            hash: 0,
        })
        .collect();
    list
}
