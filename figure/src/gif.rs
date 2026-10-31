//! Animated GIF output (feature `gif`): figures drawn for raster files,
//! rendered by vello_cpu and encoded frame by frame, with one figure
//! cache for the whole sequence so that labels repeated between frames are
//! shaped once.

#[cfg(feature = "fonts")]
use std::time::Duration;

#[cfg(feature = "fonts")]
use kurbo::{Point, Rect, Size};

#[cfg(feature = "fonts")]
use crate::cache::Cache;
#[cfg(feature = "fonts")]
use crate::figure::{Figure, Held};
#[cfg(feature = "fonts")]
use crate::list::{Bounds, Outlines};
use crate::raster;
#[cfg(feature = "fonts")]
use crate::target::Target;
#[cfg(feature = "fonts")]
use crate::text;

/// The longest side of a GIF, in device px: its sizes are `u16`.
#[cfg(feature = "fonts")]
const MAX_SIDE: f64 = 65_535.0;

/// How often a GIF plays, as its loop count says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    /// Without end.
    Forever,
    /// The loop count `n`: most viewers play the frames `n + 1` times, and
    /// `Times(0)` once.
    Times(u16),
}

/// Writes figures as the frames of an animated GIF of one size: each frame
/// drawn for [`Target::File`] at the writer's scale, rendered by vello_cpu
/// on its own pool ([`raster::Settings::parallel`]) and quantized to its
/// own palette of at most 256 colors.
///
/// ```no_run
/// use std::time::Duration;
///
/// use rustd3_core::scale;
/// use rustd3_figure::gif::{Repeat, Writer};
/// use rustd3_figure::kurbo::Size;
/// use rustd3_figure::{Figure, axis};
///
/// let file = std::fs::File::create("zoom.gif")?;
/// let mut gif = Writer::new(file, Size::new(320.0, 240.0), 1.0, Repeat::Forever)?;
/// for i in 0..30 {
///     let mut fig = Figure::new();
///     let x = fig.scale(scale::linear().domain([0.0, 100.0 - 3.0 * f64::from(i)]));
///     fig.panel(0, 0).axis(axis::bottom(x));
///     gif.frame(&fig, Duration::from_millis(40))?;
/// }
/// gif.finish()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "fonts")]
pub struct Writer<W: std::io::Write> {
    encoder: ::gif::Encoder<W>,
    size: Size,
    scale: f64,
    cache: Cache,
    outlines: Outlines,
    renderer: raster::Renderer,
}

#[cfg(feature = "fonts")]
impl<W: std::io::Write> std::fmt::Debug for Writer<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Writer").finish_non_exhaustive()
    }
}

/// Why a GIF cannot be written.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A size whose device px exceed a GIF's `u16` sides.
    #[error("{width} × {height} device px exceeds GIF's 65,535 limit")]
    Size {
        /// Device px across, rounded.
        width: u64,
        /// Device px down, rounded.
        height: u64,
    },
    /// A figure whose theme background is not opaque.
    #[error("figure has a translucent background; GIF has 1-bit alpha")]
    Translucent,
    /// What the gif crate reported, input and output aside.
    #[error("gif: {0}")]
    Encoding(String),
    /// The output could not be written.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// The frame could not be rendered: its list fails
    /// [`list::check`](crate::list::check), or it is larger than vello_cpu
    /// renders (65,408 device px a side) or of no device pixel.
    #[error(transparent)]
    Raster(#[from] raster::Error),
}

#[cfg(feature = "fonts")]
impl<W: std::io::Write> Writer<W> {
    /// A GIF of `size` logical px at `scale` device px each, its header and
    /// loop count written to `out` now, before any frame.
    ///
    /// # Errors
    ///
    /// [`Error::Size`] when `size × scale` exceeds GIF's 65,535 px a side,
    /// [`Error::Io`] or [`Error::Encoding`] when the header cannot be
    /// written.
    pub fn new(out: W, size: Size, scale: f64, repeat: Repeat) -> Result<Self, Error> {
        let (width, height) = ((size.width * scale).round(), (size.height * scale).round());
        if !(width <= MAX_SIDE && height <= MAX_SIDE) {
            return Err(Error::Size {
                width: width as u64,
                height: height as u64,
            });
        }
        // At most 65,535 by the check above; a side below one px renders
        // as `raster::Error::Empty` at the first frame.
        let mut encoder =
            ::gif::Encoder::new(out, width as u16, height as u16, &[]).map_err(encoding)?;
        encoder
            .set_repeat(match repeat {
                Repeat::Forever => ::gif::Repeat::Infinite,
                Repeat::Times(n) => ::gif::Repeat::Finite(n),
            })
            .map_err(encoding)?;
        Ok(Writer {
            encoder,
            size,
            scale,
            cache: Cache::default(),
            outlines: Outlines::default(),
            renderer: raster::Renderer::new(raster::Settings::parallel()),
        })
    }

    /// Draws, rasterizes (vello_cpu, `Settings::parallel()`) and encodes one frame.
    ///
    /// The frame shows for `delay` rounded to centiseconds, at least 2
    /// (browsers show shorter delays as 10). A figure whose theme background
    /// is translucent is [`Error::Translucent`]: GIF's alpha is one bit.
    ///
    /// # Errors
    ///
    /// [`Error::Translucent`] for a translucent theme background,
    /// [`Error::Raster`] for a frame that cannot be rendered, [`Error::Io`]
    /// or [`Error::Encoding`] when it cannot be written.
    pub fn frame(&mut self, fig: &Figure<'_>, delay: Duration) -> Result<(), Error> {
        if text::channels(fig.get_theme().get_background())[3] < u8::MAX {
            return Err(Error::Translucent);
        }
        let target = Target::File { scale: self.scale };
        let list = fig.draw_with(&mut self.cache, self.size, target, &Held::default());
        let bounds = Bounds::of(&list, &mut self.outlines);
        let area = Rect::from_origin_size(Point::ORIGIN, self.size);
        let mut image = self.renderer.render(&list, &bounds, area, self.scale)?;
        let side =
            |v: u32| u16::try_from(v).expect("invariant: renders are at most 65,408 px a side");
        let mut frame = ::gif::Frame::from_rgba_speed(
            side(image.width),
            side(image.height),
            &mut image.rgba,
            10,
        );
        frame.delay = centiseconds(delay);
        self.encoder.write_frame(&frame).map_err(encoding)
    }

    /// Writes the trailer and returns the output.
    ///
    /// # Errors
    ///
    /// [`Error::Io`] or [`Error::Encoding`] when the trailer cannot be
    /// written.
    pub fn finish(self) -> Result<W, Error> {
        self.encoder.into_inner().map_err(encoding)
    }
}

/// `delay` in GIF's centiseconds: rounded, at least 2, at most `u16::MAX`.
#[cfg(feature = "fonts")]
fn centiseconds(delay: Duration) -> u16 {
    // In 2..=65,535 after the clamp.
    (delay.as_secs_f64() * 100.0).round().clamp(2.0, 65_535.0) as u16
}

/// The gif crate's error: io errors as [`Error::Io`].
#[cfg(feature = "fonts")]
fn encoding(e: ::gif::EncodingError) -> Error {
    if let ::gif::EncodingError::Io(io) = e {
        Error::Io(io)
    } else {
        Error::Encoding(e.to_string())
    }
}
