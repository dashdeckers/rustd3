//! PNG encoding of a rendered image.

use crate::raster::{Error, Image};

/// `image` as an 8-bit RGBA PNG with default compression.
///
/// # Errors
///
/// [`Error::Length`] for pixels that are not
/// `width × height × 4` bytes, [`Error::Empty`] for an image of no pixel.
pub fn encode(image: &Image) -> Result<Vec<u8>, Error> {
    let expected = u64::from(image.width) * u64::from(image.height) * 4;
    if u64::try_from(image.rgba.len()).ok() != Some(expected) {
        return Err(Error::Length {
            width: image.width,
            height: image.height,
            found: image.rgba.len(),
        });
    }
    if image.width == 0 || image.height == 0 {
        return Err(Error::Empty(kurbo::Rect::new(
            0.0,
            0.0,
            f64::from(image.width),
            f64::from(image.height),
        )));
    }
    let mut bytes = Vec::new();
    let mut encoder = ::png::Encoder::new(&mut bytes, image.width, image.height);
    encoder.set_color(::png::ColorType::Rgba);
    encoder.set_depth(::png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| {
            w.write_image_data(&image.rgba)?;
            w.finish()
        })
        .expect("invariant: pixels of a positive size and the right length encode into memory");
    Ok(bytes)
}
