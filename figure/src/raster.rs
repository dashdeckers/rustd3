//! Raster outputs: the list rendered by vello_cpu at any scale and over
//! any area (feature `raster`), encoded as PNG (`png`), or rendered by the
//! tiny-skia fallback (`tiny-skia`); and the device-pixel snapping helpers
//! raster targets lay out with.

#[cfg(feature = "raster")]
mod cull;
#[cfg(feature = "png")]
pub mod png;
#[cfg(feature = "tiny-skia")]
pub mod skia;
pub mod snap;
#[cfg(feature = "raster")]
mod splat;
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
#[cfg(any(feature = "raster", feature = "tiny-skia"))]
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
    /// 1..=8; what file outputs use. The `Plot` widget renders
    /// single-threaded, inline and on its own pool alike.
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
    /// The SIMD level, which marker stamps are made at too.
    level: vello_cpu::Level,
    /// Whether markers are splatted ([`Renderer::splat`]).
    splat: bool,
    /// The marker stamps splatting made, kept between renders.
    stamps: splat::Stamps,
    /// Whether long stroked paths are cut to the area rendered; off only
    /// in the tests that show cutting changes no pixel.
    cut: bool,
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
            level,
            splat: false,
            stamps: splat::Stamps::default(),
            cut: true,
        }
    }

    /// Strokes long paths whole (`false`) or cut to the area (the
    /// default), to compare the two.
    #[cfg(test)]
    pub(crate) fn cut(&mut self, on: bool) -> &mut Renderer {
        self.cut = on;
        self
    }

    /// Composite markers from pre-rasterized coverage stamps (default
    /// off): a markers command of solid paints at one size, in groups that
    /// only scale and translate and clip to rectangles, lays each point's
    /// stamp at the nearest eighth-pixel phase instead of filling its
    /// shape, within 32/255 of the filled markers and several times
    /// faster for large marker sets. Everything else renders as with
    /// splatting off. The `Plot` widget turns it on for screen renders of
    /// markers commands of at least 10,000 points; file outputs never do.
    pub fn splat(&mut self, on: bool) -> &mut Renderer {
        self.splat = on;
        self
    }

    /// The part of `list` inside `area` (list px) at `scale`: exactly
    /// round(area.width × scale) × round(area.height × scale) device px, at
    /// most 65,408 a side,
    /// transparent where nothing is drawn. Commands whose `bounds` miss
    /// `area` grown by one device px are skipped, markers point by point,
    /// and a long stroked path is cut to the segments whose stroke may
    /// reach the area (dashes keep their phase), so a tile of a long line
    /// strokes its own part of it.
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
        let mut pixmap = vello_cpu::Pixmap::new(width, height);
        vello::Replay {
            context: &mut self.context,
            resources: &mut self.resources,
            images: &mut self.images,
            list,
            bounds,
            area,
            scale,
            splat: self.splat.then_some((&mut self.stamps, self.level)),
            cull: self.cut,
        }
        .run(&mut pixmap)?;
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
#[cfg(any(feature = "raster", feature = "tiny-skia"))]
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

#[cfg(all(test, feature = "raster"))]
mod tests {
    use kurbo::{Affine, BezPath, Point, Rect, Size};
    use peniko::Brush;
    use peniko::color::{AlphaColor, Srgb};

    use super::{Image, Renderer, Settings};
    use crate::list::{self, Bounds, Clip, Command, List, Outlines};

    /// A trace-like polyline: `n` columns of a zigzag across `width` px,
    /// four vertices per column, with a NaN-like gap (a new subpath) at
    /// every 997th column; dashed or not, under a group translated by
    /// (30, 20) and clipped to the canvas.
    fn trace(n: usize, width: f64, dashed: bool) -> List {
        let mut path = BezPath::new();
        for c in 0..n {
            let x = width * c as f64 / n as f64;
            let wobble = |k: usize| ((c * 7919 + k * 104_729) % 1000) as f64 / 1000.0;
            let points = [
                Point::new(x, 120.0 + 60.0 * wobble(0)),
                Point::new(x + 0.2, 40.0 + 30.0 * wobble(1)),
                Point::new(x + 0.4, 200.0 + 30.0 * wobble(2)),
                Point::new(x + 0.6, 120.0 + 60.0 * wobble(3)),
            ];
            for (k, p) in points.into_iter().enumerate() {
                if k == 0 && (c == 0 || c % 997 == 0) {
                    path.move_to(p);
                } else {
                    path.line_to(p);
                }
            }
        }
        let mut style = list::stroke(1.5);
        if dashed {
            style = style.with_dashes(1.25, [7.0, 3.0, 2.0, 3.0]);
        }
        let paint = Brush::Solid(AlphaColor::<Srgb>::from_rgb8(30, 60, 140));
        let size = Size::new(width + 60.0, 280.0);
        List {
            size,
            commands: vec![
                Command::Push {
                    transform: Affine::translate((30.0, 20.0)),
                    clip: Some(Clip::Rect(Rect::from_origin_size(Point::ORIGIN, size))),
                },
                Command::Stroke { path, style, paint },
                Command::Pop,
            ],
            ..List::default()
        }
    }

    /// `area` of `list` at `k`, cut or stroked whole.
    fn render(list: &List, area: Rect, k: f64, cut: bool) -> Image {
        let bounds = Bounds::of(list, &mut Outlines::default());
        let mut renderer = Renderer::new(Settings::default());
        renderer.cut(cut);
        renderer
            .render(list, &bounds, area, k)
            .expect("a renderable list")
    }

    /// The largest channel difference between two images of one size.
    fn max_diff(a: &Image, b: &Image) -> u8 {
        assert_eq!((a.width, a.height), (b.width, b.height));
        a.rgba
            .iter()
            .zip(&b.rgba)
            .map(|(p, q)| p.abs_diff(*q))
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn a_long_line_cut_to_each_tile_draws_what_it_drew_whole() {
        // 3,000 columns (12,000 vertices, four subpaths) over 2,000 px,
        // rendered as 256-device-px tiles at 1, 1.25 and 2: each tile with
        // the line cut to its reach against the line stroked whole, at the
        // same origin, solid and dashed (the dashes keep their phase across
        // the cut).
        for dashed in [false, true] {
            let list = trace(3_000, 2_000.0, dashed);
            for k in [1.0, 1.25, 2.0] {
                let side = 256.0 / k;
                let mut worst = 0;
                for i in [0, 3, 7] {
                    for j in [0, 1] {
                        let (x, y) = (f64::from(i) * side, f64::from(j) * side);
                        let area = Rect::new(x, y, x + side, y + side);
                        let cut = render(&list, area, k, true);
                        let whole = render(&list, area, k, false);
                        worst = worst.max(max_diff(&cut, &whole));
                    }
                }
                assert!(worst <= 1, "dashed {dashed}, k {k}: max {worst}/255");
            }
        }
    }
}
