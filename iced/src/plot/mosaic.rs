//! [`Mosaic`]: one plot area's images composed into a single image on the
//! CPU, for iced's software renderer.
//!
//! iced's tiny-skia renderer draws every raster image through tiny-skia's
//! high-precision pipeline after clearing and refilling a frame-sized clip
//! mask (`tiny_skia/src/engine.rs:533`, `adjust_clip_mask` at `:776`), and
//! converts every handle it has not drawn the frame before pixel by pixel
//! (`tiny_skia/src/raster.rs`, `Cache::allocate`). While a zoom's preview
//! layer shows, an area draws dozens of scaled, overlapping tiles: a
//! mid-burst frame of the bench's `trace` drew 68 images over 3.1 Mpx of
//! a 1.6 Mpx frame at factor 1, and 332 over 16.8 Mpx of 6.4 at 2,
//! composed in 77 and 565 ms. Laid here first (rows copied for images at
//! 1:1, scaled ones sampled at each pixel's centre, nearest) and drawn as
//! one image, the same frames compose in about a third and a sixth of
//! that (Phase 31g). The GPU renderer draws the images as they are.

use iced_core::image::{self, FilterMethod};
use iced_core::{Color, Rectangle};

use crate::plot::slot;
use crate::plot::tile;

/// A plot area's pixels, laid image by image in drawing order.
pub(crate) struct Mosaic {
    /// Device px [x0, y0, x1, y1], relative to the widget's device origin.
    rect: [i32; 4],
    /// Straight RGBA, row-major, the rect's size.
    rgba: Vec<u8>,
}

/// An image to lay: straight RGBA, its size, and whether every pixel of
/// it is opaque.
#[derive(Clone, Copy)]
pub(crate) struct Source<'a> {
    pub(crate) rgba: &'a [u8],
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) opaque: bool,
}

impl Mosaic {
    /// The area at device rect `rect`, transparent, or filled with `fill`.
    pub(crate) fn new(rect: [i32; 4], fill: Option<Color>) -> Mosaic {
        let [x0, y0, x1, y1] = rect;
        let pixels = usize::try_from((x1 - x0).max(0)).unwrap_or(0)
            * usize::try_from((y1 - y0).max(0)).unwrap_or(0);
        let rgba = match fill {
            Some(color) => color.into_rgba8().repeat(pixels),
            None => vec![0; pixels * 4],
        };
        Mosaic { rect, rgba }
    }

    /// Lays `source` stretched over the device rect `at` (its edges, device
    /// px, relative to the widget's device origin), inside `clip`: each
    /// pixel whose centre lies in `at` and `clip` takes the source pixel
    /// under its centre, over what lies there. An image at whole pixels and
    /// 1:1 copies its rows.
    pub(crate) fn lay(&mut self, source: Source<'_>, at: [f64; 4], clip: [i32; 4]) {
        let (sw, sh) = (source.width as usize, source.height as usize);
        if sw == 0 || sh == 0 || source.rgba.len() != sw * sh * 4 {
            return;
        }
        let [ax0, ay0, ax1, ay1] = at;
        if !(ax1 > ax0 && ay1 > ay0) {
            return;
        }
        // The pixels whose centres x + 0.5 lie in [ax0, ax1).
        let covered = |a: f64, b: f64| ((a - 0.5).ceil() as i32, (b - 0.5).ceil() as i32);
        let (cx0, cx1) = covered(ax0, ax1);
        let (cy0, cy1) = covered(ay0, ay1);
        let [x0, y0, x1, y1] =
            tile::intersect(tile::intersect(clip, self.rect), [cx0, cy0, cx1, cy1]);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let (kx, ky) = (sw as f64 / (ax1 - ax0), sh as f64 / (ay1 - ay0));
        let sample = |p: i32, a: f64, k: f64, n: usize| {
            (((f64::from(p) + 0.5 - a) * k).floor().max(0.0) as usize).min(n - 1)
        };
        let columns: Vec<usize> = (x0..x1).map(|x| sample(x, ax0, kx, sw)).collect();
        // Consecutive source columns: a row's span is one slice.
        let whole = columns.windows(2).all(|w| w[1] == w[0] + 1);
        let width = (self.rect[2] - self.rect[0]) as usize;
        let (left, span) = ((x0 - self.rect[0]) as usize, (x1 - x0) as usize);
        let (pixels, _) = self.rgba.as_chunks_mut::<4>();
        for y in y0..y1 {
            let v = sample(y, ay0, ky, sh);
            let (row, _) = source.rgba[v * sw * 4..][..sw * 4].as_chunks::<4>();
            let start = (y - self.rect[1]) as usize * width + left;
            let out = &mut pixels[start..start + span];
            match (whole, source.opaque) {
                (true, true) => out.copy_from_slice(&row[columns[0]..columns[0] + span]),
                (true, false) => {
                    for (d, s) in out.iter_mut().zip(&row[columns[0]..]) {
                        over(d, *s);
                    }
                }
                (false, true) => {
                    for (d, &u) in out.iter_mut().zip(&columns) {
                        *d = row[u];
                    }
                }
                (false, false) => {
                    for (d, &u) in out.iter_mut().zip(&columns) {
                        over(d, row[u]);
                    }
                }
            }
        }
    }

    /// Draws the mosaic as one image, nearest, on its device pixels, for a
    /// widget whose device origin is `origin` at `factor`.
    pub(crate) fn draw<Renderer>(self, renderer: &mut Renderer, origin: [i32; 2], factor: f32)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let [x0, y0, x1, y1] = self.rect;
        let (Ok(width), Ok(height)) = (u32::try_from(x1 - x0), u32::try_from(y1 - y0)) else {
            return;
        };
        if width == 0 || height == 0 {
            return;
        }
        let bounds: Rectangle = slot::logical(self.rect, origin, factor);
        let handle = image::Handle::from_rgba(width, height, self.rgba);
        let image = image::Image::new(handle).filter_method(FilterMethod::Nearest);
        renderer.draw_image(image, bounds, bounds);
    }

    /// The pixel at device `(x, y)`, straight RGBA.
    #[cfg(test)]
    pub(crate) fn pixel(&self, x: i32, y: i32) -> [u8; 4] {
        let width = (self.rect[2] - self.rect[0]) as usize;
        let i = ((y - self.rect[1]) as usize * width + (x - self.rect[0]) as usize) * 4;
        [0, 1, 2, 3].map(|c| self.rgba[i + c])
    }
}

/// Straight-alpha source-over of pixel `s` onto `d`, as 8-bit RGBA.
fn over(d: &mut [u8; 4], s: [u8; 4]) {
    match s[3] {
        0 => {}
        255 => *d = s,
        a => {
            let sa = u32::from(a);
            let da = u32::from(d[3]) * (255 - sa) / 255;
            let alpha = sa + da;
            for c in 0..3 {
                d[c] = ((u32::from(s[c]) * sa + u32::from(d[c]) * da + alpha / 2) / alpha) as u8;
            }
            d[3] = alpha as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use iced_core::Color;

    use super::{Mosaic, Source};

    /// A `width` × `height` image whose pixel (x, y) is (x, y, 7, 255).
    fn ramp(width: u32, height: u32) -> Vec<u8> {
        (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, 7, 255]))
            .collect()
    }

    fn source(rgba: &[u8], width: u32, height: u32, opaque: bool) -> Source<'_> {
        Source {
            rgba,
            width,
            height,
            opaque,
        }
    }

    #[test]
    fn an_image_at_whole_pixels_copies_its_rows_inside_the_clip() {
        // A 4 × 3 ramp laid 1:1 at (12, 21) in the area [10, 20, 18, 26],
        // clipped to [13, 20, 18, 23]: pixel (x, y) of the area shows the
        // ramp's (x − 12, y − 21) inside the clip, nothing outside it.
        let rgba = ramp(4, 3);
        let mut mosaic = Mosaic::new([10, 20, 18, 26], None);
        mosaic.lay(
            source(&rgba, 4, 3, true),
            [12.0, 21.0, 16.0, 24.0],
            [13, 20, 18, 23],
        );
        assert_eq!(mosaic.pixel(13, 21), [1, 0, 7, 255]);
        assert_eq!(mosaic.pixel(15, 22), [3, 1, 7, 255]);
        assert_eq!(mosaic.pixel(12, 21), [0; 4], "left of the clip");
        assert_eq!(mosaic.pixel(15, 23), [0; 4], "below the clip");
        assert_eq!(mosaic.pixel(16, 21), [0; 4], "right of the image");
    }

    #[test]
    fn a_scaled_image_takes_the_pixel_under_each_centre() {
        // A 4 × 2 ramp stretched over [0, 0, 8, 3]: device column x takes
        // source column floor((x + 0.5) / 2), row y floor((y + 0.5) · 2 / 3).
        let rgba = ramp(4, 2);
        let mut mosaic = Mosaic::new([0, 0, 8, 3], Some(Color::WHITE));
        mosaic.lay(
            source(&rgba, 4, 2, true),
            [0.0, 0.0, 8.0, 3.0],
            [0, 0, 8, 3],
        );
        let columns: Vec<u8> = (0..8).map(|x| mosaic.pixel(x, 0)[0]).collect();
        assert_eq!(columns, [0, 0, 1, 1, 2, 2, 3, 3]);
        let rows: Vec<u8> = (0..3).map(|y| mosaic.pixel(0, y)[1]).collect();
        assert_eq!(rows, [0, 1, 1]);
        // A fractional placement covers the pixels whose centres it holds.
        let mut mosaic = Mosaic::new([0, 0, 8, 1], None);
        mosaic.lay(
            source(&rgba, 4, 2, true),
            [0.6, 0.0, 4.4, 1.0],
            [0, 0, 8, 1],
        );
        let alpha: Vec<u8> = (0..8).map(|x| mosaic.pixel(x, 0)[3]).collect();
        assert_eq!(alpha, [0, 255, 255, 255, 0, 0, 0, 0]);
    }

    #[test]
    fn translucent_pixels_blend_over_what_lies_there() {
        let mut mosaic = Mosaic::new([0, 0, 1, 1], Some(Color::from_rgb8(0, 0, 255)));
        let veil = [255, 255, 255, 128];
        mosaic.lay(
            source(&veil, 1, 1, false),
            [0.0, 0.0, 1.0, 1.0],
            [0, 0, 1, 1],
        );
        assert_eq!(mosaic.pixel(0, 0), [128, 128, 255, 255]);
        // On nothing, a translucent pixel keeps its own color.
        let mut mosaic = Mosaic::new([0, 0, 1, 1], None);
        mosaic.lay(
            source(&veil, 1, 1, false),
            [0.0, 0.0, 1.0, 1.0],
            [0, 0, 1, 1],
        );
        assert_eq!(mosaic.pixel(0, 0), veil);
    }
}
