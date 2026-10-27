//! The tiny-skia fallback against vello_cpu.

use rustd3_figure::kurbo::{Rect, Shape, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::raster::{Error, skia};

use crate::sink;

/// At 1.25 the 16-texel checkerboard spans 40 device px, so texel edges
/// fall on pixel centres and the two nearest samplers break those ties
/// differently (164/255 there, 46/255 elsewhere); 1, 1.5 and 2 have none.
#[cfg(feature = "raster")]
#[test]
fn vello_cpu_and_tiny_skia_agree_on_the_kitchen_sink() {
    use crate::raster::vello::{diff, whole};

    let list = sink::with_sweep();
    for scale in [1.0, 1.5, 2.0] {
        let vello = whole(&list, scale);
        let skia = skia::render(&list, scale).unwrap();
        assert_eq!((skia.width, skia.height), (vello.width, vello.height));
        let d = diff(&skia, &vello);
        assert!(
            d.max() <= 64,
            "at {scale}: max {}, worst tile at {:?}",
            d.max(),
            d.worst_tile()
        );
    }
}

#[test]
fn tiny_skia_draws_the_glyphs_as_outlines() {
    let list = sink::list();
    let image = skia::render(&list, 1.0).unwrap();
    let ink = |x: u32, y: u32| image.rgba[((y * image.width + x) * 4) as usize] < 0x80;
    let title = (40..100).any(|x| (260..272).any(|y| ink(x, y)));
    assert!(title, "the title is drawn");
}

#[test]
fn tiny_skia_sizes_and_errors_follow_vello_cpu() {
    let list = List {
        size: Size::new(204.8, 100.0),
        ..List::default()
    };
    let image = skia::render(&list, 1.25).unwrap();
    assert_eq!((image.width, image.height), (256, 125));
    assert!(matches!(skia::render(&list, 0.0), Err(Error::Empty(_))));
    let wide = List {
        size: Size::new(70_000.0, 10.0),
        ..List::default()
    };
    assert!(matches!(
        skia::render(&wide, 1.0),
        Err(Error::TooLarge { .. })
    ));
}

#[test]
fn tiny_skia_rejects_image_brushes() {
    let list = List {
        size: Size::new(10.0, 10.0),
        commands: vec![Command::Fill {
            path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
            paint: rustd3_figure::peniko::Brush::Image(
                sink::image(1, 1, vec![0, 0, 0, 255]).into(),
            ),
        }],
        ..List::default()
    };
    assert_eq!(
        skia::render(&list, 1.0),
        Err(Error::List(list::Error::Unsupported {
            command: 0,
            what: "an image brush"
        }))
    );
}
