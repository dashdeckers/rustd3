//! The reference the widget's pixels are compared with: the PNG of the
//! same figure, rendered by vello_cpu for the screen at the widget's
//! factor and composited onto the screenshot's background.

use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::list::{Bounds, List, Outlines};
use rustd3_figure::raster::{self, Settings, Simd};
use rustd3_figure::{Cache, Figure, Held, Target, layout};

/// `fig` at `size` logical px and `scale` device px per px, as the PNG
/// output renders it (single-threaded, baseline SIMD), composited
/// source-over onto `background`.
pub fn png_of(
    fig: &Figure<'_>,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
) -> raster::Image {
    let list = fig.draw_with(
        &mut Cache::default(),
        size,
        Target::Screen { scale },
        &Held::default(),
    );
    png_of_list(&list, size, scale, background)
}

/// As [`png_of`], laid out in the plot areas `areas` a gesture holds.
pub fn png_held(
    fig: &Figure<'_>,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
    areas: &layout::Areas,
) -> raster::Image {
    let list = fig.draw_held(
        &mut Cache::default(),
        size,
        Target::Screen { scale },
        &Held::default(),
        areas,
    );
    png_of_list(&list, size, scale, background)
}

/// `list` rendered as the PNG output renders it, composited source-over
/// onto `background`.
fn png_of_list(
    list: &List,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
) -> raster::Image {
    let mut image = raster::Renderer::new(Settings {
        threads: 0,
        simd: Simd::Baseline,
    })
    .render(
        list,
        &Bounds::of(list, &mut Outlines::default()),
        Rect::from_origin_size((0.0, 0.0), size),
        scale,
    )
    .expect("the figure renders");
    let [r, g, b, a] = background.into_rgba8().map(f64::from);
    for pixel in image.rgba.chunks_exact_mut(4) {
        let alpha = f64::from(pixel[3]) / 255.0;
        let under = a / 255.0 * (1.0 - alpha);
        let out = alpha + under;
        for (channel, back) in pixel.iter_mut().zip([r, g, b]) {
            let value = (f64::from(*channel) * alpha + back * under) / out;
            *channel = value.round() as u8;
        }
        pixel[3] = (out * 255.0).round() as u8;
    }
    image
}
