//! Image data in the layouts backends take.

use peniko::{ImageAlphaType, ImageData, ImageFormat};

/// The pixels as straight (un-premultiplied) RGBA; the error names a pixel
/// format this crate does not know.
pub(crate) fn straight(image: &ImageData) -> Result<Vec<u8>, &'static str> {
    let mut rgba = rgba(image)?;
    if image.alpha_type == ImageAlphaType::AlphaPremultiplied {
        for px in rgba.chunks_exact_mut(4) {
            let a = u16::from(px[3]);
            for c in &mut px[..3] {
                if let Some(v) = (u16::from(*c) * 255 + a / 2).checked_div(a) {
                    *c = u8::try_from(v).unwrap_or(u8::MAX);
                }
            }
        }
    }
    Ok(rgba)
}

/// The pixels in RGBA channel order, alpha as stored.
fn rgba(image: &ImageData) -> Result<Vec<u8>, &'static str> {
    let mut data = image.data.data().to_vec();
    match image.format {
        ImageFormat::Rgba8 => {}
        ImageFormat::Bgra8 => {
            for px in data.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        // `ImageFormat` is #[non_exhaustive] in peniko.
        _ => return Err("an image format other than RGBA8 and BGRA8"),
    }
    Ok(data)
}
