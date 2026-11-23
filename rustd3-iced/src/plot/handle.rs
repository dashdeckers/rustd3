//! [`from_image`]: a rendered cell's pixels as an iced image handle.

use iced_core::image;
use rustd3_figure::raster;

/// Moves a straight-alpha raster image into an iced handle (no copy).
///
/// iced handles carry straight alpha (the software renderer premultiplies
/// on load, `vello_cpu/src/raster.rs:139-149`), which is what the figure's
/// rasterizer returns, so
/// the bytes move into `Handle::from_rgba` as they are. Every call makes a
/// new handle id and so a new texture (`core/src/image.rs:134`).
pub(crate) fn from_image(image: raster::Image) -> image::Handle {
    image::Handle::from_rgba(image.width, image.height, image.rgba)
}

#[cfg(test)]
mod tests {
    use iced_core::image::Handle;
    use rustd3_figure::raster::Image;

    use super::from_image;

    #[test]
    fn a_960_by_640_image_moves_into_its_handle_without_a_copy() {
        let rgba = vec![7_u8; 960 * 640 * 4];
        let at = rgba.as_ptr();
        let handle = from_image(Image {
            width: 960,
            height: 640,
            rgba,
        });
        let Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } = handle
        else {
            panic!("an image becomes an RGBA handle");
        };
        assert_eq!((width, height, pixels.len()), (960, 640, 2_457_600));
        assert_eq!(pixels.as_ptr(), at);
    }
}
