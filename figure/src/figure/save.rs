//! Writing a figure to files, and [`Error`], why that failed.

#[cfg(feature = "fonts")]
use std::path::Path;
use std::path::PathBuf;

use crate::cache::Cache;
#[cfg(feature = "fonts")]
use crate::figure::Figure;
#[cfg(all(feature = "png", feature = "fonts"))]
use crate::figure::Held;
use crate::list;
#[cfg(all(feature = "png", feature = "fonts"))]
use crate::list::{Bounds, Outlines};
#[cfg(feature = "pdf")]
use crate::pdf;
#[cfg(feature = "raster")]
use crate::raster;
#[cfg(all(feature = "png", feature = "fonts"))]
use crate::target::Target;
#[cfg(feature = "fonts")]
use crate::{lower, svg};

/// Why a figure could not be saved.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The file could not be written.
    #[error("cannot write {}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// What the system reported.
        source: std::io::Error,
    },
    /// The figure's list fails [`list::check`], or holds what the output
    /// cannot write.
    #[error(transparent)]
    List(#[from] list::Error),
    /// The render failed: too large, or nothing to render.
    #[cfg(feature = "raster")]
    #[error(transparent)]
    Raster(#[from] raster::Error),
    /// The list could not be written as PDF.
    #[cfg(feature = "pdf")]
    #[error(transparent)]
    Pdf(#[from] pdf::Error),
}

#[cfg(feature = "fonts")]
impl Figure<'_> {
    /// Writes the figure as SVG to `path`: drawn for vector output at
    /// `size` logical px with a fresh cache ([`Figure::draw`]), lowered
    /// for SVG ([`svg::CAPS`]: sweep gradients become images) and written
    /// with text as glyph outlines. Returns what lowering could not draw
    /// faithfully.
    ///
    /// # Errors
    ///
    /// [`Error::List`] for a list the SVG writer rejects, [`Error::Io`]
    /// when the file cannot be written.
    pub fn save_svg(
        &self,
        path: impl AsRef<Path>,
        size: kurbo::Size,
    ) -> Result<Vec<lower::Warning>, Error> {
        let lowered = lower::lower(&self.draw(size), svg::CAPS)?;
        let svg = svg::write(&lowered.list, &svg::Options::default())?;
        write(path.as_ref(), svg.as_bytes())?;
        Ok(lowered.warnings)
    }

    /// Writes the figure as an 8-bit RGBA PNG of `size × scale` device px
    /// to `path`: drawn for [`Target::File`](crate::Target::File) with a
    /// fresh cache, so long lines thin to four columns per device pixel,
    /// and rendered by vello_cpu on its own pool
    /// ([`raster::Settings::parallel`]).
    ///
    /// # Errors
    ///
    /// [`Error::Raster`] for a size of no device pixel or beyond 65,408
    /// device px a side, [`Error::Io`] when the file cannot be written.
    #[cfg(feature = "png")]
    pub fn save_png(
        &self,
        path: impl AsRef<Path>,
        size: kurbo::Size,
        scale: f64,
    ) -> Result<(), Error> {
        let target = Target::File { scale };
        let list = self.draw_with(&mut Cache::default(), size, target, &Held::default());
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let area = kurbo::Rect::from_origin_size(kurbo::Point::ORIGIN, size);
        let mut renderer = raster::Renderer::new(raster::Settings::parallel());
        let image = renderer.render(&list, &bounds, area, scale)?;
        write(path.as_ref(), &raster::png::encode(&image)?)
    }

    /// Writes the figure as a one-page PDF of `size × 0.75` points to
    /// `path`: drawn for vector output at `size` logical px with a fresh
    /// cache ([`Figure::draw`]) and written by [`pdf::write`] with text as
    /// selectable text in subset fonts.
    ///
    /// # Errors
    ///
    /// [`Error::Pdf`] for a size that is not positive and finite or what
    /// PDF does not draw, [`Error::Io`] when the file cannot be written.
    #[cfg(feature = "pdf")]
    pub fn save_pdf(&self, path: impl AsRef<Path>, size: kurbo::Size) -> Result<(), Error> {
        let pdf = pdf::write(&self.draw(size), &pdf::Options::default())?;
        write(path.as_ref(), &pdf)
    }
}

/// Writes `bytes` to `path`.
#[cfg(feature = "fonts")]
fn write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    std::fs::write(path, bytes).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}
