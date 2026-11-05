//! [`Rgba`]: straight RGBA pixels as a krilla custom image. Embedding
//! pixels this way gives an image that PDF viewers interpolate.

use std::sync::Arc;

use krilla::image::{BitsPerComponent, CustomImage, ImageColorspace};

/// Straight RGBA pixels split into krilla's two channels, the RGB bytes
/// and then the alpha bytes. Both live in one shared buffer, so cloning is
/// cheap and hashing costs little beyond the bytes.
#[derive(Debug, Clone, Hash)]
pub(crate) struct Rgba {
    width: u32,
    height: u32,
    /// `width × height × 3` color bytes, then `width × height` alpha bytes.
    pixels: Arc<[u8]>,
}

impl Rgba {
    /// `rgba`, `width × height × 4` straight bytes, split into channels.
    pub(crate) fn new(width: u32, height: u32, rgba: &[u8]) -> Rgba {
        let (pixels, _) = rgba.as_chunks::<4>();
        let colors = pixels.iter().flat_map(|p| [p[0], p[1], p[2]]);
        let alphas = pixels.iter().map(|p| p[3]);
        Rgba {
            width,
            height,
            pixels: colors.chain(alphas).collect(),
        }
    }

    /// Where the alpha bytes start.
    fn split(&self) -> usize {
        self.pixels.len() / 4 * 3
    }
}

impl CustomImage for Rgba {
    fn color_channel(&self) -> &[u8] {
        &self.pixels[..self.split()]
    }

    fn alpha_channel(&self) -> Option<&[u8]> {
        Some(&self.pixels[self.split()..])
    }

    fn bits_per_component(&self) -> BitsPerComponent {
        BitsPerComponent::Eight
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn icc_profile(&self) -> Option<&[u8]> {
        None
    }

    fn color_space(&self) -> ImageColorspace {
        ImageColorspace::Rgb
    }
}
