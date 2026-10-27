//! Raster outputs: the list rendered by vello_cpu at any scale and over
//! any area (feature `raster`), encoded as PNG (`png`), or rendered by the
//! tiny-skia fallback (`tiny-skia`); and the device-pixel snapping helpers
//! raster targets lay out with.

#[cfg(feature = "png")]
pub mod png;
pub mod snap;
#[cfg(feature = "raster")]
mod vello;

use kurbo::Rect;

use crate::list;
#[cfg(feature = "raster")]
use crate::list::{Bounds, List, Outlines};

/// The longest side of a render, in device px. vello_cpu 0.3.0 sizes are
/// `u16`, but its spans round widths up to 128 px and its tiles heights up
/// to 4 px, and either overflows past 65,535: widths from 65,409 and
/// heights from 65,533 panic (`vello_cpu-0.3.0/src/span.rs:84-90`,
/// `vello_common-0.3.0/src/util.rs:222-238`).
#[cfg(feature = "raster")]
const MAX_SIDE: f64 = 65_408.0;
/// The longest side of an image in a list: vello_cpu's `u16` pixmaps.
#[cfg(feature = "raster")]
const MAX_IMAGE: f64 = 65_535.0;

/// Pixels: straight (un-premultiplied) RGBA, row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Width in px.
    pub width: u32,
    /// Height in px.
    pub height: u32,
    /// `width × height × 4` bytes.
    pub rgba: Vec<u8>,
}

/// Why a list cannot be rendered.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// A render with a side above 65,408 device px, or an image in the
    /// list with a side above 65,535 px.
    #[error("{width} × {height} px is larger than vello_cpu takes")]
    TooLarge {
        /// Width in device px.
        width: f64,
        /// Height in device px.
        height: f64,
    },
    /// An area of no device pixel at the scale asked for.
    #[error("empty area {0:?}")]
    Empty(Rect),
    /// Pixels whose length is not `width × height × 4`.
    #[error("{found} bytes for {width} × {height} px")]
    Length {
        /// The image's width.
        width: u32,
        /// The image's height.
        height: u32,
        /// The data's length.
        found: usize,
    },
    /// A list that fails [`list::check`], or holds what the backend cannot
    /// draw.
    #[error(transparent)]
    List(#[from] list::Error),
}

/// How a [`Renderer`] runs vello_cpu.
#[cfg(feature = "raster")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Simd {
    /// The best SIMD level the CPU offers.
    #[default]
    Detect,
    /// No SIMD beyond the baseline: the same pixels on every machine.
    Baseline,
}

/// vello_cpu's threads and SIMD level. Default: threads 0
/// (single-threaded), [`Simd::Detect`].
#[cfg(feature = "raster")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Settings {
    /// vello_cpu's worker threads: 0 renders on the calling thread, `n`
    /// starts vello_cpu's own pool of `n`.
    pub threads: u16,
    /// The SIMD level.
    pub simd: Simd,
}

#[cfg(feature = "raster")]
impl Settings {
    /// vello_cpu's own pool: the available parallelism − 1, clamped to
    /// 1..=8; what file outputs and the widget's worker use.
    pub fn parallel() -> Settings {
        let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let threads = cores.saturating_sub(1).clamp(1, 8);
        Settings {
            threads: u16::try_from(threads).expect("invariant: at most 8"),
            simd: Simd::Detect,
        }
    }
}

/// Renders lists with vello_cpu, keeping one render context, its
/// resources and the images of the last render between calls.
#[cfg(feature = "raster")]
#[derive(Debug)]
pub struct Renderer {
    context: vello_cpu::RenderContext,
    resources: vello_cpu::Resources,
    images: vello::Images,
}

#[cfg(feature = "raster")]
const _: () = {
    fn assert<T: Send>() {}
    let _ = assert::<Renderer>;
};

#[cfg(feature = "raster")]
impl Renderer {
    /// A renderer with `settings`; a pool, if any, starts now.
    pub fn new(settings: Settings) -> Self {
        let level = match settings.simd {
            Simd::Detect => vello_cpu::Level::try_detect().unwrap_or(vello_cpu::Level::baseline()),
            Simd::Baseline => vello_cpu::Level::baseline(),
        };
        let render = vello_cpu::RenderSettings {
            level,
            num_threads: settings.threads,
        };
        Renderer {
            context: vello_cpu::RenderContext::new_with(1, 1, render),
            resources: vello_cpu::Resources::new(),
            images: vello::Images::default(),
        }
    }

    /// The part of `list` inside `area` (list px) at `scale`: exactly
    /// round(area.width × scale) × round(area.height × scale) device px, at
    /// most 65,408 a side,
    /// transparent where nothing is drawn. Commands whose `bounds` miss
    /// `area` grown by one device px are skipped, markers point by point.
    ///
    /// `bounds` must be [`Bounds::of`] this list.
    pub fn render(
        &mut self,
        list: &List,
        bounds: &Bounds,
        area: Rect,
        scale: f64,
    ) -> Result<Image, Error> {
        list::check(list)?;
        debug_assert_eq!(bounds.len(), list.commands.len());
        let (width, height) = device(area, scale)?;
        self.context.reset_and_resize(width, height);
        vello::Replay {
            context: &mut self.context,
            resources: &mut self.resources,
            images: &mut self.images,
            list,
            bounds,
            area,
            scale,
        }
        .run()?;
        self.context.flush();
        let mut pixmap = vello_cpu::Pixmap::new(width, height);
        self.context.render(&mut pixmap, &mut self.resources);
        Ok(Image {
            width: u32::from(width),
            height: u32::from(height),
            rgba: pixmap.take_rgba8(peniko::ImageAlphaType::Alpha),
        })
    }
}

/// The whole of `list` at `scale` with [`Settings::default`], its
/// [`Bounds`] computed here.
#[cfg(feature = "raster")]
pub fn render(list: &List, scale: f64) -> Result<Image, Error> {
    let bounds = Bounds::of(list, &mut Outlines::default());
    let area = Rect::from_origin_size(kurbo::Point::ORIGIN, list.size);
    Renderer::new(Settings::default()).render(list, &bounds, area, scale)
}

/// The device size of `area` at `scale`, each side rounded:
/// [`Error::Empty`] below one px (or not a number), [`Error::TooLarge`]
/// above 65,408.
#[cfg(feature = "raster")]
pub(crate) fn device(area: Rect, scale: f64) -> Result<(u16, u16), Error> {
    let (width, height) = (
        (area.width() * scale).round(),
        (area.height() * scale).round(),
    );
    if !(width >= 1.0 && height >= 1.0) {
        return Err(Error::Empty(area));
    }
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(Error::TooLarge { width, height });
    }
    // Both in 1..=65,408 by the checks above.
    Ok((width as u16, height as u16))
}
