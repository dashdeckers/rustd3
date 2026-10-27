//! vello_cpu rendering: sizes, errors, areas, shifts, bands and culling,
//! group-space paints, and PNG encoding.

use rustd3_figure::kurbo::{Affine, Point, Rect, Shape, Size};
use rustd3_figure::list::{self, Bounds, Command, List, Outlines};
use rustd3_figure::peniko::{Brush, Color, Gradient};
use rustd3_figure::raster::{self, Error, Image, Renderer, Settings, Simd};
use rustd3_testkit::pixels::{self, Diff};

use crate::sink;

/// Single-threaded at the baseline SIMD level: the same pixels anywhere.
pub fn renderer() -> Renderer {
    Renderer::new(Settings {
        threads: 0,
        simd: Simd::Baseline,
    })
}

/// `area` of `list` at `scale`, bounds computed here.
pub fn render(list: &List, area: Rect, scale: f64) -> Image {
    let bounds = Bounds::of(list, &mut Outlines::default());
    renderer()
        .render(list, &bounds, area, scale)
        .expect("the list renders")
}

/// The whole list at `scale`.
pub fn whole(list: &List, scale: f64) -> Image {
    render(
        list,
        Rect::from_origin_size(Point::ORIGIN, list.size),
        scale,
    )
}

/// The pixels of `image` in columns `x..x + w` and rows `y..y + h`.
pub fn crop(image: &Image, x: u32, y: u32, w: u32, h: u32) -> Image {
    let row = |r: u32| {
        let start = ((r * image.width + x) * 4) as usize;
        &image.rgba[start..start + (w * 4) as usize]
    };
    Image {
        width: w,
        height: h,
        rgba: (y..y + h).flat_map(|r| row(r).iter().copied()).collect(),
    }
}

/// How `a` and `b` differ, channels over 16 counted.
pub fn diff(a: &Image, b: &Image) -> Diff {
    let a = pixels::Image::new(a.width, a.height, &a.rgba).expect("whole pixels");
    let b = pixels::Image::new(b.width, b.height, &b.rgba).expect("whole pixels");
    Diff::between(&a, &b, 16).expect("images of one size")
}

fn sized(width: f64, height: f64) -> List {
    List {
        size: Size::new(width, height),
        commands: vec![Command::Fill {
            path: Rect::new(0.0, 0.0, width, height).to_path(0.1),
            paint: sink::rgb(0x336699),
        }],
        ..List::default()
    }
}

#[test]
fn a_render_is_exactly_the_area_times_the_scale_rounded() {
    let list = sized(300.0, 200.0);
    let image = render(&list, Rect::new(0.0, 0.0, 204.8, 100.0), 1.25);
    assert_eq!((image.width, image.height), (256, 125));
    assert_eq!(image.rgba.len(), 256 * 125 * 4);
    let odd = render(&list, Rect::new(10.0, 10.0, 10.6, 11.4), 1.0);
    assert_eq!((odd.width, odd.height), (1, 1));
}

#[test]
fn a_side_above_65408_device_px_is_too_large() {
    assert_eq!(
        raster::render(&sized(70_000.0, 10.0), 1.0),
        Err(Error::TooLarge {
            width: 70_000.0,
            height: 10.0
        })
    );
    assert_eq!(
        raster::render(&sized(40_000.0, 10.0), 2.0),
        Err(Error::TooLarge {
            width: 80_000.0,
            height: 20.0
        })
    );
    assert!(raster::render(&sized(65_408.0, 1.0), 1.0).is_ok());
    assert!(raster::render(&sized(1.0, 65_408.0), 1.0).is_ok());
    assert_eq!(
        raster::render(&sized(65_409.0, 1.0), 1.0),
        Err(Error::TooLarge {
            width: 65_409.0,
            height: 1.0
        })
    );
}

#[test]
fn an_area_of_no_device_pixel_is_empty() {
    let list = sized(10.0, 10.0);
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let empty = Rect::new(0.0, 0.0, 0.0, 10.0);
    assert_eq!(
        renderer().render(&list, &bounds, empty, 2.0),
        Err(Error::Empty(empty))
    );
    let thin = Rect::new(0.0, 0.0, 0.2, 10.0);
    assert_eq!(
        renderer().render(&list, &bounds, thin, 2.0),
        Err(Error::Empty(thin))
    );
    let whole = Rect::new(0.0, 0.0, 10.0, 10.0);
    assert_eq!(
        renderer().render(&list, &bounds, whole, f64::NAN),
        Err(Error::Empty(whole))
    );
}

#[test]
fn a_list_that_fails_check_is_not_rendered() {
    let mut list = sized(10.0, 10.0);
    list.commands.push(Command::Pop);
    assert_eq!(
        raster::render(&list, 1.0),
        Err(Error::List(list::Error::UnmatchedPop { command: 1 }))
    );
}

#[test]
fn an_area_render_equals_the_same_pixels_of_the_whole_render() {
    let list = sink::list();
    let full = whole(&list, 2.0);
    let area = render(&list, Rect::new(100.0, 60.0, 260.0, 180.0), 2.0);
    let d = diff(&area, &crop(&full, 200, 120, 320, 240));
    assert!(d.max() <= 1, "max {}", d.max());
}

#[test]
fn a_whole_device_pixel_shift_renders_the_overlap_identically() {
    let list = sink::list();
    let a = render(&list, Rect::new(0.0, 0.0, 100.0, 100.0), 2.0);
    let b = render(&list, Rect::new(8.0, 0.0, 108.0, 100.0), 2.0);
    let d = diff(&crop(&a, 16, 0, 184, 200), &crop(&b, 0, 0, 184, 200));
    assert!(d.max() <= 1, "max {}", d.max());
    let half = render(&list, Rect::new(8.25, 0.0, 108.25, 100.0), 2.0);
    let h = diff(&crop(&a, 16, 0, 184, 200), &crop(&half, 0, 0, 184, 200));
    assert!(
        h.max() > 16,
        "a half-pixel shift resamples every edge: {}",
        h.max()
    );
}

#[test]
fn bands_rendered_with_one_bounds_match_the_whole_render() {
    let list = sink::list();
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let mut renderer = renderer();
    let full = renderer
        .render(&list, &bounds, Rect::new(0.0, 0.0, 400.0, 300.0), 2.0)
        .unwrap();
    let mut stitched = Vec::new();
    for band in 0..6 {
        let y = 50.0 * f64::from(band);
        let image = renderer
            .render(&list, &bounds, Rect::new(0.0, y, 400.0, y + 50.0), 2.0)
            .unwrap();
        assert_eq!((image.width, image.height), (800, 100));
        stitched.extend(image.rgba);
    }
    let stitched = Image {
        width: 800,
        height: 600,
        rgba: stitched,
    };
    let d = diff(&stitched, &full);
    assert!(d.max() <= 1, "max {}", d.max());
}

#[test]
fn commands_whose_bounds_miss_the_area_are_skipped() {
    let list = sized(100.0, 100.0);
    let elsewhere = List {
        commands: vec![Command::Fill {
            path: Rect::new(60.0, 60.0, 70.0, 70.0).to_path(0.1),
            paint: sink::rgb(0),
        }],
        ..list.clone()
    };
    let bounds = Bounds::of(&elsewhere, &mut Outlines::default());
    let image = renderer()
        .render(&list, &bounds, Rect::new(0.0, 0.0, 20.0, 20.0), 1.0)
        .unwrap();
    assert!(image.rgba.iter().all(|&c| c == 0), "culled by its bounds");
    let drawn = renderer()
        .render(&list, &bounds, Rect::new(50.0, 50.0, 80.0, 80.0), 1.0)
        .unwrap();
    assert_eq!(drawn.rgba[..4], [0x33, 0x66, 0x99, 0xff]);
}

#[test]
fn nothing_drawn_is_transparent() {
    let list = List {
        size: Size::new(4.0, 4.0),
        ..List::default()
    };
    let image = raster::render(&list, 1.0).unwrap();
    assert!(image.rgba.iter().all(|&c| c == 0));
}

#[test]
fn rendering_twice_with_one_renderer_draws_the_same_pixels() {
    let list = sink::list();
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let mut renderer = renderer();
    let area = Rect::new(0.0, 0.0, 400.0, 300.0);
    let first = renderer.render(&list, &bounds, area, 1.0).unwrap();
    let again = renderer.render(&list, &bounds, area, 1.0).unwrap();
    assert_eq!(first, again);
}

#[test]
fn marker_paints_are_positioned_in_the_groups_space() {
    let gradient = Brush::Gradient(Gradient::new_linear((0.0, 0.0), (100.0, 0.0)).with_stops([
        (0.0, Color::from_rgb8(0xff, 0, 0)),
        (1.0, Color::from_rgb8(0, 0, 0xff)),
    ]));
    let square = Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1);
    let at = [20.0, 50.0, 80.0].map(|x| Point::new(x, 20.0));
    let markers = List {
        size: Size::new(100.0, 40.0),
        commands: vec![Command::Markers {
            shape: square.clone(),
            at: at.to_vec(),
            paint: gradient.clone(),
            sizes: Some(vec![16.0; 3]),
            paints: None,
        }],
        ..List::default()
    };
    let fills = List {
        commands: at
            .iter()
            .map(|p| Command::Fill {
                path: (Affine::translate(p.to_vec2()) * Affine::scale(16.0)) * square.clone(),
                paint: gradient.clone(),
            })
            .collect(),
        ..markers.clone()
    };
    let d = diff(&whole(&markers, 2.0), &whole(&fills, 2.0));
    assert!(d.max() <= 1, "max {}", d.max());
}

#[cfg(feature = "png")]
#[test]
fn png_encoding_keeps_every_pixel() {
    let image = whole(&sink::list(), 1.0);
    let bytes = raster::png::encode(&image).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut decoded = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut decoded).unwrap();
    assert_eq!((info.width, info.height), (400, 300));
    assert_eq!(info.color_type, png::ColorType::Rgba);
    assert_eq!(decoded, image.rgba);
}

#[cfg(feature = "png")]
#[test]
fn png_encoding_rejects_pixels_of_the_wrong_length() {
    let image = Image {
        width: 2,
        height: 2,
        rgba: vec![0; 12],
    };
    assert_eq!(
        raster::png::encode(&image),
        Err(Error::Length {
            width: 2,
            height: 2,
            found: 12
        })
    );
}
