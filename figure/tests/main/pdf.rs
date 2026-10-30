//! PDF output: units, determinism, images, errors, text and fonts, and
//! the poppler round trip of the kitchen sink and the example figures
//! against vello_cpu's render of the same list.

use std::sync::Arc;

use rustd3_figure::kurbo::{Affine, Rect, Shape, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::pdf::{self, Error, Options};
use rustd3_figure::peniko::{Blob, Brush, Color, FontData, ImageBrush};
use rustd3_figure::raster::Image;

use crate::poppler;
use crate::sink;

/// `list` as PDF with the default options.
fn written(list: &List) -> Vec<u8> {
    pdf::write(list, &Options::default()).expect("the list writes")
}

/// A list of `size` whose background is one white fill.
fn blank(width: f64, height: f64) -> List {
    List {
        size: Size::new(width, height),
        commands: vec![Command::Fill {
            path: Rect::new(0.0, 0.0, width, height).to_path(0.1),
            paint: sink::rgb(0xffffff),
        }],
        ..List::default()
    }
}

/// The distinct colors of each 32-px tile of `image`, the most of any tile.
fn levels_per_tile(image: &Image) -> usize {
    let mut most = 0;
    for ty in (0..image.height).step_by(32) {
        for tx in (0..image.width).step_by(32) {
            let mut seen = std::collections::BTreeSet::new();
            for y in ty..(ty + 32).min(image.height) {
                for x in tx..(tx + 32).min(image.width) {
                    let i = ((y * image.width + x) * 4) as usize;
                    seen.insert([image.rgba[i], image.rgba[i + 1], image.rgba[i + 2]]);
                }
            }
            most = most.max(seen.len());
        }
    }
    most
}

#[test]
fn a_960_by_760_list_is_a_720_by_570_pt_page() {
    let pdf = written(&blank(960.0, 760.0));
    assert!(pdf.windows(22).any(|w| w == b"/MediaBox[0 0 720 570]"));
    let at_72 = poppler::render(&pdf, 72.0);
    assert_eq!((at_72.width, at_72.height), (720, 570));
    let at_192 = poppler::render(&pdf, 192.0);
    assert_eq!((at_192.width, at_192.height), (1920, 1520));
}

#[test]
fn writing_a_list_twice_gives_identical_bytes() {
    let list = sink::with_sweep();
    assert_eq!(written(&list), written(&list));
    let titled = Options::default().title("Kitchen sink");
    let once = pdf::write(&list, &titled).expect("the list writes");
    assert_eq!(once, pdf::write(&list, &titled).expect("the list writes"));
    let back = poppler::render(&once, 192.0);
    assert_eq!((back.width, back.height), (800, 600));
}

#[test]
fn smooth_and_nearest_images_both_round_trip() {
    // A 2 × 2 checker of two greys stretched over 200 px.
    let rgba: Vec<u8> = [0x40, 0xc0, 0xc0, 0x40]
        .into_iter()
        .flat_map(|v| [v, v, v, 0xff])
        .collect();
    let image = |smooth| {
        let mut list = blank(200.0, 200.0);
        list.commands.push(Command::Image {
            rect: Rect::new(0.0, 0.0, 200.0, 200.0),
            image: sink::image(2, 2, rgba.clone()),
            smooth,
        });
        poppler::render(&written(&list), 192.0)
    };
    let nearest = image(false);
    assert_eq!((nearest.width, nearest.height), (400, 400));
    assert!(levels_per_tile(&nearest) <= 2, "nearest keeps hard edges");
    let smooth = image(true);
    assert_eq!((smooth.width, smooth.height), (400, 400));
    assert!(
        levels_per_tile(&smooth) > 2,
        "smooth images are interpolated"
    );
}

#[test]
fn text_is_selectable_in_subset_fonts_unless_outlined() {
    let list = sink::list();
    let pdf = written(&list);
    let text = poppler::text(&pdf);
    for label in ["Wheel: RL", "Speed (km/h)"] {
        assert!(text.contains(label), "{label:?} in {text:?}");
    }
    let fonts = poppler::fonts(&pdf);
    assert_eq!(fonts.len(), 1, "{fonts:?}");
    assert!(fonts[0].name.ends_with("+DejaVuSans"), "{fonts:?}");
    assert!(fonts[0].embedded && fonts[0].subset, "{fonts:?}");

    let outlined = pdf::write(&list, &Options::default().outlined(true)).expect("the list writes");
    assert!(!poppler::text(&outlined).contains("Wheel"));
    assert_eq!(poppler::fonts(&outlined), []);
}

#[test]
fn a_title_is_the_only_metadata() {
    let list = sink::list();
    let titled =
        pdf::write(&list, &Options::default().title("Kitchen sink")).expect("the list writes");
    assert!(titled.windows(12).any(|w| w == b"Kitchen sink"));
    assert!(!written(&list).windows(6).any(|w| w == b"/Title"));
}

#[test]
fn a_page_of_no_area_or_an_infinite_side_is_an_error() {
    for (w, h) in [(0.0, 10.0), (10.0, -1.0), (f64::INFINITY, 10.0)] {
        let list = List {
            size: Size::new(w, h),
            ..List::default()
        };
        let e = pdf::write(&list, &Options::default()).expect_err("no page");
        assert!(
            matches!(e, Error::Size { width, height } if width == w && height == h),
            "{e}"
        );
    }
}

#[test]
fn an_image_brush_inside_a_group_is_unsupported() {
    let mut list = sink::list();
    let at = list.commands.len();
    list.commands.extend([
        Command::Push {
            transform: Affine::translate((10.0, 10.0)),
            clip: Some(list::Clip::Rect(Rect::new(0.0, 0.0, 50.0, 50.0))),
        },
        Command::Fill {
            path: Rect::new(0.0, 0.0, 20.0, 20.0).to_path(0.1),
            paint: Brush::Image(ImageBrush::new(sink::image(1, 1, vec![0, 0, 0, 255]))),
        },
        Command::Pop,
    ]);
    let e = pdf::write(&list, &Options::default()).expect_err("an image brush");
    assert!(
        matches!(e, Error::Unsupported { index, what: "an image brush" } if index == at + 1),
        "{e}"
    );
}

#[test]
fn a_font_krilla_cannot_read_is_an_error_naming_it() {
    let mut list = sink::list();
    list.fonts
        .push(FontData::new(Blob::new(Arc::new(vec![0_u8; 64])), 0));
    let e = pdf::write(&list, &Options::default()).expect_err("an unreadable font");
    assert!(matches!(e, Error::Font { index: 1 }), "{e}");
}

#[test]
fn an_unbalanced_list_fails_its_check() {
    let mut list = sink::list();
    list.commands.push(Command::Pop);
    let e = pdf::write(&list, &Options::default()).expect_err("an unmatched pop");
    assert!(
        matches!(e, Error::List(list::Error::UnmatchedPop { .. })),
        "{e}"
    );
}

#[test]
fn a_clip_of_no_area_clips_everything() {
    let empty = |clip| {
        let mut list = blank(100.0, 100.0);
        list.commands.extend([
            Command::Push {
                transform: Affine::IDENTITY,
                clip: Some(clip),
            },
            Command::Push {
                transform: Affine::translate((1.0, 1.0)),
                clip: None,
            },
            Command::Fill {
                path: Rect::new(0.0, 0.0, 100.0, 100.0).to_path(0.1),
                paint: Brush::Solid(Color::from_rgb8(0, 0, 0)),
            },
            Command::Pop,
            Command::Pop,
            Command::Fill {
                path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
                paint: Brush::Solid(Color::from_rgb8(0, 0, 0)),
            },
        ]);
        poppler::render(&written(&list), 96.0)
    };
    for clip in [
        list::Clip::Path(Default::default()),
        list::Clip::Rect(Rect::new(20.0, 20.0, 20.0, 80.0)),
    ] {
        let back = empty(clip);
        let black = back
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] == 0)
            .count();
        assert_eq!(black, 100, "only the fill after the group draws");
    }
}

/// The spec's bound for a measured tile score: 0.02 when the measurement
/// meets it, else 1.25 × the measurement, never above 0.05.
#[cfg(feature = "raster")]
fn bound(measured: f64) -> f64 {
    assert!(measured <= 0.05, "{measured} is a writer bug, not a bound");
    if measured <= 0.02 {
        0.02
    } else {
        1.25 * measured
    }
}

/// The tile scores of `list`'s PDF with text and with outlined glyphs,
/// each rendered by `pdftoppm -r 192` against vello_cpu's render of the
/// list at 2× (single-threaded, baseline SIMD).
#[cfg(feature = "raster")]
fn round_trip(list: &List) -> [f64; 2] {
    let reference = crate::raster::vello::whole(list, 2.0);
    [false, true].map(|outlined| {
        let pdf =
            pdf::write(list, &Options::default().outlined(outlined)).expect("the list writes");
        let d = crate::raster::vello::diff(&poppler::render(&pdf, 192.0), &reference);
        eprintln!(
            "pdf round trip (outlined {outlined}): tile score {:.4} at {:?}, max {}",
            d.tile_score(),
            d.worst_tile(),
            d.max()
        );
        d.tile_score()
    })
}

/// Asserts the round trip of `list` against the scores measured for it
/// as bounds: outlined glyphs within their measurement, text within 1.25 ×
/// its measurement, which poppler's glyph placement drives.
#[cfg(feature = "raster")]
fn assert_round_trip(name: &str, list: &List, [text, outlined]: [f64; 2]) {
    let [t, o] = round_trip(list);
    assert!(o <= bound(outlined), "{name} outlined: tile score {o:.4}");
    assert!(t <= 1.25 * text, "{name} text: tile score {t:.4}");
}

#[cfg(feature = "raster")]
#[test]
fn the_kitchen_sink_renders_back_through_poppler_within_the_bound() {
    assert_round_trip("sink", &sink::with_sweep(), [0.0310, 0.0124]);
}

/// A heatmap of 24 × 16 cells on viridis with its colorbar, a bold title
/// span and a math axis label: three fonts, two images.
#[cfg(feature = "fonts")]
fn heatmap() -> rustd3_figure::Figure<'static> {
    use rustd3_core::{scale, scheme};
    use rustd3_figure::colorbar::colorbar;
    use rustd3_figure::legend::Slot;
    use rustd3_figure::mark::{Grid, image};
    use rustd3_figure::text::{self, Text, span};
    use rustd3_figure::{Figure, axis};

    let (cols, rows) = (24, 16);
    let values = (0..cols * rows)
        .map(|i| {
            let (c, r) = ((i % cols) as f64, (i / cols) as f64);
            40.0 + 30.0 * (c / 4.0).sin() * (r / 5.0).cos()
        })
        .collect();
    let grid = Grid::new(cols, rows, values)
        .expect("cols × rows values")
        .extent([0.0, 240.0], [0.0, 3.0]);
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.title(Text::from("Bearing ").push(span("temperature").bold()));
    fig.panel(0, 0)
        .axis(axis::left(y).label(text::typst("RMS [$m slash s^2$]")))
        .axis(axis::bottom(x).label("speed [km/h]"))
        .mark(image(grid, x, y, heat));
    fig.legend(Slot::Right, colorbar(heat).title("temp [°C]"));
    fig
}

/// `fig` drawn at `size` for vector output.
#[cfg(feature = "fonts")]
fn vector(fig: &rustd3_figure::Figure<'_>, size: Size) -> List {
    use rustd3_figure::{Held, Target};
    fig.draw_with(
        &mut crate::figure::cache(),
        size,
        Target::Vector,
        &Held::default(),
    )
}

#[cfg(all(feature = "raster", feature = "fonts"))]
#[test]
fn the_example_figures_render_back_through_poppler_within_the_bound() {
    use crate::{scatter, trace};
    let fig = scatter::scatter(scatter::POINTS);
    assert_round_trip("scatter", &vector(&fig, scatter::SIZE), [0.0635, 0.0191]);
    let rec = trace::recording(trace::SAMPLES);
    let fig = trace::trace(&rec);
    assert_round_trip("trace", &vector(&fig, trace::SIZE), [0.0620, 0.0246]);
    let fig = heatmap();
    assert_round_trip(
        "heatmap",
        &vector(&fig, Size::new(640.0, 480.0)),
        [0.0740, 0.0161],
    );
}

#[cfg(feature = "fonts")]
#[test]
fn every_label_is_extracted_and_every_font_embedded_as_a_subset() {
    let list = vector(&heatmap(), Size::new(640.0, 480.0));
    let pdf = written(&list);
    let words = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let extracted = words(&poppler::text(&pdf));
    for c in &list.commands {
        if let Command::Glyphs(run) = c {
            assert!(extracted.contains(&words(&run.text)), "{:?}", run.text);
        }
    }
    assert!(extracted.contains("Bearing temperature"));
    let mut names: Vec<String> = poppler::fonts(&pdf)
        .into_iter()
        .map(|f| {
            assert!(f.embedded && f.subset, "{f:?}");
            let (tag, name) = f.name.split_once('+').expect("a subset tag");
            assert_eq!(tag.len(), 6);
            name.to_string()
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "DejaVuSans",
            "DejaVuSans-Bold",
            "NewCMMath-Regular-Identity-H"
        ]
    );
}
