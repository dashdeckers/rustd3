//! PDF output (feature `pdf`): the list written by krilla on one page of
//! its size in points, glyph runs as real, selectable text in subset fonts.

mod image;

use std::sync::Arc;

use krilla::geom;
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{self, FillRule};
use krilla::surface::Surface;
use krilla::text::{Font, GlyphId, KrillaGlyph};
use krilla::{Data, Document};
use kurbo::{Affine, BezPath, Cap, Join, PathEl, Point, Rect, Shape};
use peniko::color::Srgb;
use peniko::{Brush, Color, Extend, Gradient, GradientKind, ImageData};

use crate::list::{self, Clip, Command, GlyphRun, List, Paints};
use crate::lower::Caps;
use crate::pixels;

/// What [`write()`] draws natively: everything, so no list needs lowering.
pub const CAPS: Caps = Caps {
    glyphs: true,
    markers: true,
    clip_paths: true,
    sweep: true,
};

/// Points per list px: a list px is a CSS px, 1/96 in, and a point 1/72 in.
const PT: f64 = 0.75;

/// How [`write()`] writes a list.
#[derive(Debug, Clone, Default)]
pub struct Options {
    outlined: bool,
    title: Option<String>,
}

/// Why a list cannot be written as PDF.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A list whose size is not positive and finite.
    #[error("page size {width} × {height} px is not positive and finite")]
    Size {
        /// The list's width, px.
        width: f64,
        /// The list's height, px.
        height: f64,
    },
    /// A font of the list's table that krilla cannot read or embed.
    #[error("font {index} of the list could not be loaded by krilla")]
    Font {
        /// Its index in [`List::fonts`].
        index: usize,
    },
    /// A command holding what PDF output does not draw: an image brush, or
    /// an image format other than RGBA8 and BGRA8.
    #[error("command {index}: {what} cannot be written to PDF")]
    Unsupported {
        /// The command.
        index: usize,
        /// What it holds.
        what: &'static str,
    },
    /// What krilla reported while writing the document.
    #[error("krilla: {0}")]
    Krilla(String),
    /// A list that fails [`list::check`].
    #[error(transparent)]
    List(#[from] list::Error),
}

impl Options {
    /// Glyphs as filled outlines instead of text (no fonts embedded).
    #[must_use]
    pub fn outlined(self, yes: bool) -> Self {
        Options {
            outlined: yes,
            ..self
        }
    }

    /// The document's title, written to its metadata.
    #[must_use]
    pub fn title(self, t: impl Into<String>) -> Self {
        Options {
            title: Some(t.into()),
            ..self
        }
    }
}

/// `list` as a one-page PDF of `size × 0.75` points, list px kept as the
/// page's units by a root `scale(0.75)`: a 960 × 760 px list is a 720 ×
/// 570 pt page, which `pdftoppm -r 96` rasterizes at 960 × 760 px.
///
/// Every command is drawn natively ([`CAPS`]): glyph runs as text in
/// their fonts, subset and embedded, selectable through each run's source
/// text (or as outlines with [`Options::outlined`]); markers as their shape
/// stamped per point; images nearest or interpolated; path clips as clips.
/// Paints are placed in their group's space for every command, as the list
/// positions them. Two limits are krilla's: radial gradients pad whatever
/// their extend, and stops interpolate in sRGB whatever the gradient's
/// color space. A group clipped to an area of nothing draws nothing. The document
/// has no creation date and metadata only with a title, so the same list
/// writes the same bytes.
///
/// ```
/// use rustd3_figure::kurbo::{Rect, Shape, Size};
/// use rustd3_figure::list::{Command, List};
/// use rustd3_figure::peniko::{Brush, Color};
/// use rustd3_figure::pdf;
///
/// let list = List {
///     size: Size::new(960.0, 760.0),
///     commands: vec![Command::Fill {
///         path: Rect::new(0.0, 0.0, 960.0, 760.0).to_path(0.1),
///         paint: Brush::Solid(Color::from_rgb8(0x4c, 0x72, 0xb0)),
///     }],
///     ..List::default()
/// };
/// let bytes = pdf::write(&list, &pdf::Options::default())?;
/// assert!(bytes.starts_with(b"%PDF-"));
/// assert!(bytes.windows(22).any(|w| w == b"/MediaBox[0 0 720 570]"));
/// # Ok::<(), pdf::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::Size`] for a list whose size is not positive and finite,
/// [`Error::Unsupported`] for what PDF does not draw, [`Error::Font`] for a
/// font krilla cannot embed, [`Error::List`] for a list that fails
/// [`list::check`], [`Error::Krilla`] for what krilla
/// reports.
pub fn write(list: &List, options: &Options) -> Result<Vec<u8>, Error> {
    list::check(list)?;
    let (width, height) = (list.size.width, list.size.height);
    let settings = Some((width, height))
        .filter(|(w, h)| w.is_finite() && h.is_finite())
        .and_then(|(w, h)| PageSettings::from_wh((w * PT) as f32, (h * PT) as f32))
        .ok_or(Error::Size { width, height })?;
    let fonts = list
        .fonts
        .iter()
        .enumerate()
        .map(|(index, font)| {
            let bytes: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(font.data.clone());
            Font::new(Data::from(bytes), font.index).ok_or(Error::Font { index })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut document = Document::new();
    if let Some(title) = &options.title {
        document.set_metadata(Metadata::new().title(title.clone()));
    }
    let mut page = document.start_page_with(settings);
    let mut surface = page.surface();
    surface.push_transform(&transform(Affine::scale(PT)));
    let mut replay = Replay {
        fonts: &fonts,
        outlined: options.outlined,
        open: Vec::new(),
    };
    let drawn = list
        .commands
        .iter()
        .enumerate()
        .try_for_each(|(i, command)| replay.command(&mut surface, i, command));
    // krilla panics on a surface dropped with pushes open, so a command
    // that fails leaves none.
    let open: usize = replay.open.iter().flatten().sum();
    for _ in 0..=open {
        surface.pop();
    }
    surface.finish();
    page.finish();
    drawn?;
    document.finish().map_err(|e| {
        let font = if let krilla::error::KrillaError::Font(font, _) = &e {
            fonts.iter().position(|f| f == font)
        } else {
            None
        };
        font.map_or_else(
            || Error::Krilla(e.to_string()),
            |index| Error::Font { index },
        )
    })
}

/// One list replayed onto a page's surface.
struct Replay<'a> {
    /// One per list font.
    fonts: &'a [Font],
    outlined: bool,
    /// How many surface pops each open group owes (1, or 2 with a clip),
    /// or `None` for a group clipped to nothing, whose commands are skipped.
    open: Vec<Option<usize>>,
}

impl Replay<'_> {
    fn command(
        &mut self,
        surface: &mut Surface<'_>,
        index: usize,
        command: &Command,
    ) -> Result<(), Error> {
        let hidden = self.open.contains(&None);
        match command {
            Command::Push { transform: t, clip } => {
                let clip = clip.as_ref().map(clip_path);
                if hidden || clip.as_ref().is_some_and(Option::is_none) {
                    self.open.push(if hidden { Some(0) } else { None });
                    return Ok(());
                }
                surface.push_transform(&transform(*t));
                let mut pops = 1;
                if let Some(Some(path)) = &clip {
                    surface.push_clip_path(path, &FillRule::NonZero);
                    pops = 2;
                }
                self.open.push(Some(pops));
            }
            Command::Pop => {
                for _ in 0..self.open.pop().flatten().unwrap_or(0) {
                    surface.pop();
                }
            }
            Command::Fill { .. }
            | Command::Stroke { .. }
            | Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. }
                if hidden => {}
            Command::Fill { path, paint } => {
                if let Some(path) = convert(path) {
                    surface.set_stroke(None);
                    surface.set_fill(Some(fill(index, paint, Affine::IDENTITY)?));
                    surface.draw_path(&path);
                }
            }
            Command::Stroke { path, style, paint } => {
                // PDF draws a zero width as the thinnest line a device
                // shows; every other backend draws nothing.
                if let Some(path) = convert(path).filter(|_| style.width > 0.0) {
                    surface.set_fill(None);
                    surface.set_stroke(Some(stroke(index, style, paint)?));
                    surface.draw_path(&path);
                }
            }
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } => {
                if let Some(shape) = convert(shape) {
                    let marks = Marks {
                        at,
                        paint,
                        sizes: sizes.as_deref(),
                        paints: paints.as_ref(),
                    };
                    marks.draw(surface, index, &shape)?;
                }
            }
            Command::Glyphs(run) => self.glyphs(surface, index, run)?,
            Command::Image {
                rect,
                image,
                smooth,
            } => stretch(surface, index, *rect, image, *smooth)?,
        }
        Ok(())
    }

    /// A run under its transform, each glyph normalized by the run's size:
    /// advance and offset from the pen, which is the sum of the advances
    /// before it. A glyph whose bytes are not a slice of the run's text
    /// maps to no text.
    fn glyphs(
        &mut self,
        surface: &mut Surface<'_>,
        index: usize,
        run: &GlyphRun,
    ) -> Result<(), Error> {
        let size = run.size;
        if size <= 0.0 || run.transform.determinant() == 0.0 {
            return Ok(());
        }
        let fill = fill(index, &run.fill, run.transform.inverse())?;
        let mut pen = 0.0;
        let glyphs: Vec<KrillaGlyph> = run
            .glyphs
            .iter()
            .map(|g| {
                let bytes = g.text.start as usize..g.text.end as usize;
                let bytes = if run.text.get(bytes.clone()).is_some() {
                    bytes
                } else {
                    0..0
                };
                let glyph = KrillaGlyph::new(
                    GlyphId::new(g.id),
                    g.advance / size,
                    (g.x - pen) / size,
                    -g.y / size,
                    0.0,
                    bytes,
                    None,
                );
                pen += g.advance;
                glyph
            })
            .collect();
        surface.set_stroke(None);
        surface.set_fill(Some(fill));
        surface.push_transform(&transform(run.transform));
        surface.draw_glyphs(
            geom::Point::from_xy(0.0, 0.0),
            &glyphs,
            self.fonts[run.font].clone(),
            &run.text,
            size,
            self.outlined,
        );
        surface.pop();
        Ok(())
    }
}

/// A markers command's channels.
struct Marks<'a> {
    at: &'a [Point],
    paint: &'a Brush,
    sizes: Option<&'a [f64]>,
    paints: Option<&'a Paints>,
}

impl Marks<'_> {
    /// `shape` stamped at each point under a translation and its size;
    /// the fill changes only where the paint index does, and per point for
    /// a gradient, whose transform undoes the point's placement.
    fn draw(
        &self,
        surface: &mut Surface<'_>,
        index: usize,
        shape: &geom::Path,
    ) -> Result<(), Error> {
        surface.set_stroke(None);
        let mut current = None;
        for (k, p) in self.at.iter().enumerate() {
            let size = self.sizes.map_or(1.0, |s| s[k]);
            if size == 0.0 {
                continue;
            }
            let place = Affine::translate(p.to_vec2()) * Affine::scale(size);
            let slot = self.paints.map_or(0, |p| p.index[k]);
            let brush = self.paints.map_or(self.paint, |p| &p.table[slot as usize]);
            if current != Some(slot) || !matches!(brush, Brush::Solid(_)) {
                surface.set_fill(Some(fill(index, brush, place.inverse())?));
                current = Some(slot);
            }
            surface.push_transform(&transform(place));
            surface.draw_path(shape);
            surface.pop();
        }
        Ok(())
    }
}

/// `image` stretched over `rect`: nearest pixels through krilla's RGBA8
/// image, which PDF viewers do not interpolate, or interpolated through a
/// custom image that asks for it.
fn stretch(
    surface: &mut Surface<'_>,
    index: usize,
    rect: Rect,
    image: &ImageData,
    smooth: bool,
) -> Result<(), Error> {
    let rect = rect.abs();
    let Some(size) = geom::Size::from_wh(rect.width() as f32, rect.height() as f32) else {
        return Ok(());
    };
    if image.width == 0 || image.height == 0 {
        return Ok(());
    }
    let rgba = pixels::straight(image).map_err(|what| Error::Unsupported { index, what })?;
    let pdf_image = if smooth {
        krilla::image::Image::from_custom(image::Rgba::new(image.width, image.height, &rgba), true)
            .map_err(Error::Krilla)?
    } else {
        krilla::image::Image::from_rgba8(rgba, image.width, image.height)
    };
    surface.push_transform(&transform(Affine::translate(rect.origin().to_vec2())));
    surface.draw_image(pdf_image, size);
    surface.pop();
    Ok(())
}

/// A fill of `brush` under the nonzero rule, a gradient placed by
/// `inverse` (the inverse of the command's own placement, so that it sits
/// in the group's space).
fn fill(index: usize, brush: &Brush, inverse: Affine) -> Result<paint::Fill, Error> {
    let (paint, opacity) = paint(index, brush, inverse)?;
    Ok(paint::Fill {
        paint,
        opacity,
        rule: FillRule::NonZero,
    })
}

/// A stroke of `style` in `brush`: width, the start cap at both ends,
/// join, miter limit and dashes, all explicit.
fn stroke(index: usize, style: &kurbo::Stroke, brush: &Brush) -> Result<paint::Stroke, Error> {
    let (paint, opacity) = paint(index, brush, Affine::IDENTITY)?;
    let dash = (!style.dash_pattern.is_empty()).then(|| paint::StrokeDash {
        array: style.dash_pattern.iter().map(|d| *d as f32).collect(),
        offset: style.dash_offset as f32,
    });
    Ok(paint::Stroke {
        paint,
        width: style.width as f32,
        miter_limit: style.miter_limit as f32,
        line_cap: match style.start_cap {
            Cap::Butt => paint::LineCap::Butt,
            Cap::Round => paint::LineCap::Round,
            Cap::Square => paint::LineCap::Square,
        },
        line_join: match style.join {
            Join::Miter => paint::LineJoin::Miter,
            Join::Round => paint::LineJoin::Round,
            Join::Bevel => paint::LineJoin::Bevel,
        },
        opacity,
        dash,
    })
}

/// krilla's paint of `brush` and its opacity: an sRGB color and its alpha,
/// or a gradient, opaque but for its stops' own opacities.
fn paint(
    index: usize,
    brush: &Brush,
    inverse: Affine,
) -> Result<(paint::Paint, NormalizedF32), Error> {
    match brush {
        Brush::Solid(color) => Ok((rgb(*color).into(), unit(color.components[3]))),
        Brush::Gradient(gradient) => Ok(self::gradient(gradient, inverse)),
        Brush::Image(_) => Err(Error::Unsupported {
            index,
            what: "an image brush",
        }),
    }
}

/// `gradient` as krilla's, its extend as the spread method, angles in
/// degrees; a gradient without stops is transparent.
fn gradient(gradient: &Gradient, inverse: Affine) -> (paint::Paint, NormalizedF32) {
    if gradient.stops.is_empty() {
        return (
            krilla::color::rgb::Color::black().into(),
            NormalizedF32::ZERO,
        );
    }
    let stops: Vec<paint::Stop> = gradient
        .stops
        .iter()
        .map(|stop| {
            let color = stop.color.to_alpha_color::<Srgb>();
            paint::Stop {
                offset: unit(stop.offset),
                color: rgb(color).into(),
                opacity: unit(color.components[3]),
            }
        })
        .collect();
    let spread_method = match gradient.extend {
        Extend::Pad => paint::SpreadMethod::Pad,
        Extend::Repeat => paint::SpreadMethod::Repeat,
        Extend::Reflect => paint::SpreadMethod::Reflect,
    };
    let transform = transform(inverse);
    let paint = match gradient.kind {
        GradientKind::Linear(p) => paint::LinearGradient {
            x1: p.start.x as f32,
            y1: p.start.y as f32,
            x2: p.end.x as f32,
            y2: p.end.y as f32,
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
        GradientKind::Radial(p) => paint::RadialGradient {
            fx: p.start_center.x as f32,
            fy: p.start_center.y as f32,
            fr: p.start_radius,
            cx: p.end_center.x as f32,
            cy: p.end_center.y as f32,
            cr: p.end_radius,
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
        GradientKind::Sweep(p) => paint::SweepGradient {
            cx: p.center.x as f32,
            cy: p.center.y as f32,
            start_angle: p.start_angle.to_degrees(),
            end_angle: p.end_angle.to_degrees(),
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
    };
    (paint, NormalizedF32::ONE)
}

/// The color's sRGB channels in 8 bits.
fn rgb(color: Color) -> krilla::color::rgb::Color {
    let [r, g, b, _] = color.to_rgba8().to_u8_array();
    krilla::color::rgb::Color::new(r, g, b)
}

/// `v` clamped to 0..=1.
fn unit(v: f32) -> NormalizedF32 {
    NormalizedF32::new(v.clamp(0.0, 1.0)).unwrap_or(NormalizedF32::ONE)
}

/// kurbo's affine as krilla's transform.
fn transform(t: Affine) -> geom::Transform {
    let [a, b, c, d, e, f] = t.as_coeffs();
    geom::Transform::from_row(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32)
}

/// `path` as krilla's; `None` when it draws nothing (no segment).
fn convert(path: &BezPath) -> Option<geom::Path> {
    let mut builder = geom::PathBuilder::new();
    for element in path.elements() {
        match *element {
            PathEl::MoveTo(a) => builder.move_to(a.x as f32, a.y as f32),
            PathEl::LineTo(a) => builder.line_to(a.x as f32, a.y as f32),
            PathEl::QuadTo(a, b) => builder.quad_to(a.x as f32, a.y as f32, b.x as f32, b.y as f32),
            PathEl::CurveTo(a, b, c) => builder.cubic_to(
                a.x as f32, a.y as f32, b.x as f32, b.y as f32, c.x as f32, c.y as f32,
            ),
            PathEl::ClosePath => builder.close(),
        }
    }
    builder.finish()
}

/// The clip's path, or `None` for a clip of no area, which clips every
/// command away (viewers differ on clips to degenerate paths).
fn clip_path(clip: &Clip) -> Option<geom::Path> {
    let path = match clip {
        Clip::Rect(r) => r.to_path(0.1),
        Clip::Path(p) => p.clone(),
    };
    let area = path.bounding_box();
    if area.width() > 0.0 && area.height() > 0.0 {
        convert(&path)
    } else {
        None
    }
}
