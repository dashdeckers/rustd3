//! SVG output: the list written as SVG elements, text as glyph outlines
//! defined once each and used by every run (or, on request, as `<text>`),
//! ids in command order.

mod base64;
pub(crate) mod number;
mod paint;

use std::collections::BTreeSet;
use std::fmt::Write;

use kurbo::{Affine, BezPath, Cap, Join, Point, Rect, Shape};
use peniko::{Brush, FontData, ImageData};

use crate::list::{self, Clip, Command, GlyphRun, List, Outlines, Paints};
use crate::lower::Caps;
use crate::pixels;

/// What [`write()`] draws natively: everything but sweep gradients, which
/// [`lower`](crate::lower::lower) rasterizes first.
pub const CAPS: Caps = Caps {
    glyphs: true,
    markers: true,
    clip_paths: true,
    sweep: false,
};

/// How [`write()`] writes a list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// How glyph runs are written.
    pub text: Text,
}

/// How glyph runs are written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Text {
    /// Every glyph used is one `<path>` in `<defs>` at 1000 units per em,
    /// which runs `<use>`: the file needs no fonts and draws exactly the
    /// glyphs that were measured.
    #[default]
    Outlines,
    /// Each solid run is one `<text>` element with its source text, for
    /// editing: family and weight from the font's `name` table, one x per
    /// glyph when glyphs map one to one onto characters, else the first
    /// glyph's. A viewer re-shapes the text with whatever fonts it has, so
    /// the drawing may differ from what was measured.
    Elements,
}

/// `list` as an SVG document.
///
/// The list must need no lowering for SVG: a sweep gradient or an image
/// brush is `Err(list::Error::Unsupported { command, what })`. Paints
/// interpolate in sRGB; a stroke takes its start cap at both ends.
/// Markers and glyph runs with a gradient paint are written as paths in
/// their group's space, where the list positions every paint.
///
/// ```
/// use rustd3_figure::kurbo::{Rect, Shape, Size};
/// use rustd3_figure::list::{Command, List};
/// use rustd3_figure::peniko::{Brush, Color};
/// use rustd3_figure::svg;
///
/// let list = List {
///     size: Size::new(10.0, 10.0),
///     commands: vec![Command::Fill {
///         path: Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1),
///         paint: Brush::Solid(Color::from_rgb8(0x4c, 0x72, 0xb0)),
///     }],
///     ..List::default()
/// };
/// let svg = svg::write(&list, &svg::Options::default())?;
/// assert!(svg.contains("<path d='M0,0L10,0L10,10L0,10Z' fill='#4c72b0'/>"));
/// # Ok::<(), rustd3_figure::list::Error>(())
/// ```
///
/// # Errors
///
/// The [`list::Error`] of [`list::check`] for a list
/// that fails it, and [`list::Error::Unsupported`] for a sweep gradient
/// (which lowering for [`CAPS`] turns into an image) or an image brush.
pub fn write(list: &List, options: &Options) -> Result<String, list::Error> {
    list::check(list)?;
    let faces = match options.text {
        Text::Outlines => Vec::new(),
        Text::Elements => list.fonts.iter().map(Face::read).collect(),
    };
    let mut writer = Writer {
        list,
        text: options.text,
        faces,
        out: String::new(),
        outlines: Outlines::default(),
        ids: Ids::default(),
        open: Vec::new(),
    };
    writer.header();
    for (i, command) in list.commands.iter().enumerate() {
        writer.command(i, command)?;
    }
    writer.out.push_str("</svg>\n");
    Ok(writer.out)
}

/// The last id of each kind, assigned in command order.
#[derive(Debug, Default)]
struct Ids {
    markers: usize,
    clips: usize,
    paints: usize,
    images: usize,
}

/// A font as `<text>` names it.
struct Face {
    /// The typographic family, else the family, from the `name` table.
    family: Option<String>,
    bold: bool,
}

impl Face {
    /// `font`'s family and weight; neither for a font ttf-parser cannot read.
    fn read(font: &FontData) -> Face {
        let Ok(face) = ttf_parser::Face::parse(font.data.data(), font.index) else {
            return Face {
                family: None,
                bold: false,
            };
        };
        let name = |id: u16| {
            face.names()
                .into_iter()
                .filter(|name| name.name_id == id)
                .find_map(|name| name.to_string())
        };
        Face {
            family: name(ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
                .or_else(|| name(ttf_parser::name_id::FAMILY)),
            bold: face.is_bold(),
        }
    }
}

/// One document being written.
struct Writer<'a> {
    list: &'a List,
    text: Text,
    /// Per list font, for [`Text::Elements`].
    faces: Vec<Face>,
    out: String,
    outlines: Outlines,
    ids: Ids,
    /// How many `<g>` each open group wrote.
    open: Vec<usize>,
}

impl Writer<'_> {
    /// The root element and the glyph outlines every solid run uses when
    /// runs are written as outlines.
    fn header(&mut self) {
        let (w, h) = (
            number::px(self.list.size.width),
            number::px(self.list.size.height),
        );
        let _ = write!(
            self.out,
            "<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' \
             width='{w}' height='{h}' viewBox='0 0 {w} {h}'>"
        );
        let used: BTreeSet<(usize, u32)> = self
            .list
            .commands
            .iter()
            .filter_map(|c| match c {
                Command::Glyphs(run)
                    if self.text == Text::Outlines && matches!(run.fill, Brush::Solid(_)) =>
                {
                    Some(run)
                }
                Command::Glyphs(_)
                | Command::Fill { .. }
                | Command::Stroke { .. }
                | Command::Markers { .. }
                | Command::Image { .. }
                | Command::Push { .. }
                | Command::Pop => None,
            })
            .flat_map(|run| run.glyphs.iter().map(|g| (run.font, g.id)))
            .collect();
        let mut defs = String::new();
        for (font, id) in used {
            if let Some(outline) = self.outlines.get(&self.list.fonts[font], id) {
                let _ = write!(
                    defs,
                    "<path id='g{font}-{id}' d='{}'/>",
                    number::path(&(Affine::scale(1000.0) * outline.clone()), number::px)
                );
            }
        }
        if !defs.is_empty() {
            let _ = write!(self.out, "<defs>{defs}</defs>");
        }
    }

    fn command(&mut self, i: usize, command: &Command) -> Result<(), list::Error> {
        match command {
            Command::Fill { path, paint } => {
                let attrs = self.paint(i, "fill", paint)?;
                let _ = write!(
                    self.out,
                    "<path d='{}'{attrs}/>",
                    number::path(path, number::px)
                );
            }
            Command::Stroke { path, style, paint } => {
                let attrs = self.paint(i, "stroke", paint)?;
                let _ = write!(
                    self.out,
                    "<path d='{}' fill='none'{attrs}{}/>",
                    number::path(path, number::px),
                    stroke(style)
                );
            }
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } => self.markers(i, shape, at, paint, sizes.as_deref(), paints.as_ref())?,
            Command::Glyphs(run) => self.glyphs(i, run)?,
            Command::Image {
                rect,
                image,
                smooth,
            } => self.image(i, *rect, image, *smooth)?,
            Command::Push { transform, clip } => {
                if *transform == Affine::IDENTITY {
                    self.out.push_str("<g>");
                } else {
                    let _ = write!(self.out, "<g transform='{}'>", number::matrix(*transform));
                }
                let mut groups = 1;
                if let Some(clip) = clip {
                    self.ids.clips += 1;
                    let id = self.ids.clips;
                    let d = match clip {
                        Clip::Rect(r) => number::path(&r.to_path(0.1), number::px),
                        Clip::Path(p) => number::path(p, number::px),
                    };
                    let _ = write!(
                        self.out,
                        "<clipPath id='c{id}'><path d='{d}'/></clipPath><g clip-path='url(#c{id})'>"
                    );
                    groups = 2;
                }
                self.open.push(groups);
            }
            Command::Pop => {
                for _ in 0..self.open.pop().unwrap_or(0) {
                    self.out.push_str("</g>");
                }
            }
        }
        Ok(())
    }

    /// `paint` as the attribute `attr`, its gradient element written first.
    fn paint(&mut self, command: usize, attr: &str, paint: &Brush) -> Result<String, list::Error> {
        let written = paint::write(attr, paint, self.ids.paints + 1)
            .map_err(|what| list::Error::Unsupported { command, what })?;
        if let Some(def) = written.def {
            self.ids.paints += 1;
            let _ = write!(self.out, "<defs>{def}</defs>");
        }
        Ok(written.attrs)
    }

    /// The shape once in `<defs>`; points grouped by paint into `<g>` runs
    /// of `<use>`, or of paths in the group's space for a gradient.
    fn markers(
        &mut self,
        command: usize,
        shape: &BezPath,
        at: &[Point],
        paint: &Brush,
        sizes: Option<&[f64]>,
        paints: Option<&Paints>,
    ) -> Result<(), list::Error> {
        self.ids.markers += 1;
        let id = self.ids.markers;
        let _ = write!(
            self.out,
            "<defs><path id='m{id}' d='{}'/></defs>",
            number::path(shape, number::unit)
        );
        let index = |k: usize| paints.map_or(0, |p| p.index[k]);
        let brush = |k: usize| paints.map_or(paint, |p| &p.table[p.index[k] as usize]);
        let mut start = 0;
        while start < at.len() {
            let end = (start..at.len())
                .find(|&k| index(k) != index(start))
                .unwrap_or(at.len());
            let fill = brush(start);
            let attrs = self.paint(command, "fill", fill)?;
            let _ = write!(self.out, "<g{attrs}>");
            for (k, p) in at.iter().enumerate().take(end).skip(start) {
                let size = sizes.map(|s| s[k]);
                match (fill, size) {
                    (Brush::Solid(_), None) => {
                        let _ = write!(
                            self.out,
                            "<use xlink:href='#m{id}' x='{}' y='{}'/>",
                            number::px(p.x),
                            number::px(p.y)
                        );
                    }
                    (Brush::Solid(_), Some(s)) => {
                        let _ = write!(
                            self.out,
                            "<use xlink:href='#m{id}' transform='translate({} {}) scale({})'/>",
                            number::px(p.x),
                            number::px(p.y),
                            number::unit(s)
                        );
                    }
                    (Brush::Gradient(_) | Brush::Image(_), size) => {
                        let place =
                            Affine::translate(p.to_vec2()) * Affine::scale(size.unwrap_or(1.0));
                        let _ = write!(
                            self.out,
                            "<path d='{}'/>",
                            number::path(&(place * shape.clone()), number::px)
                        );
                    }
                }
            }
            self.out.push_str("</g>");
            start = end;
        }
        Ok(())
    }

    /// A solid run as `<use>` of the glyph defs under the run's transform
    /// scaled to 1000 units per em, or as `<text>`; any other run as one
    /// path in its group's space.
    fn glyphs(&mut self, command: usize, run: &GlyphRun) -> Result<(), list::Error> {
        let attrs = self.paint(command, "fill", &run.fill)?;
        if !matches!(run.fill, Brush::Solid(_)) {
            let path = self.outlines.run(&self.list.fonts, run);
            let _ = write!(
                self.out,
                "<path d='{}'{attrs}/>",
                number::path(&path, number::px)
            );
            return Ok(());
        }
        if self.text == Text::Elements {
            self.element(run, &attrs);
            return Ok(());
        }
        let k = f64::from(run.size) / 1000.0;
        let _ = write!(
            self.out,
            "<g transform='{} scale({})'{attrs}>",
            number::matrix(run.transform),
            number::fine(k)
        );
        let font = &self.list.fonts[run.font];
        for glyph in &run.glyphs {
            if self.outlines.get(font, glyph.id).is_some() {
                let _ = write!(
                    self.out,
                    "<use xlink:href='#g{}-{}' x='{}' y='{}'/>",
                    run.font,
                    glyph.id,
                    number::px(f64::from(glyph.x) / k),
                    number::px(f64::from(glyph.y) / k)
                );
            }
        }
        self.out.push_str("</g>");
        Ok(())
    }

    /// A solid run as `<text>`, whitespace preserved so that each x stays
    /// with its glyph's character.
    fn element(&mut self, run: &GlyphRun, attrs: &str) {
        let chars = run.text.char_indices();
        let one_to_one = run.text.chars().count() == run.glyphs.len()
            && run.glyphs.iter().zip(chars).all(|(glyph, (i, c))| {
                let text = glyph.text.start as usize..glyph.text.end as usize;
                text == (i..i + c.len_utf8())
            });
        let xs: Vec<String> = if one_to_one {
            run.glyphs
                .iter()
                .map(|g| number::px(f64::from(g.x)))
                .collect()
        } else {
            vec![number::px(
                run.glyphs.first().map_or(0.0, |g| f64::from(g.x)),
            )]
        };
        let face = &self.faces[run.font];
        let family = face
            .family
            .as_deref()
            .map(|f| format!(" font-family='{}'", xml(f)))
            .unwrap_or_default();
        let weight = if face.bold { " font-weight='bold'" } else { "" };
        let _ = write!(
            self.out,
            "<text transform='{}' x='{}' y='0'{family} font-size='{}'{weight}{attrs} \
             xml:space='preserve'>{}</text>",
            number::matrix(run.transform),
            xs.join(" "),
            number::unit(f64::from(run.size)),
            xml(&run.text)
        );
    }

    /// `<image>` with the pixels as a base64 PNG data URI.
    fn image(
        &mut self,
        command: usize,
        rect: Rect,
        image: &ImageData,
        smooth: bool,
    ) -> Result<(), list::Error> {
        if image.width == 0 || image.height == 0 {
            return Ok(());
        }
        let rgba =
            pixels::straight(image).map_err(|what| list::Error::Unsupported { command, what })?;
        self.ids.images += 1;
        let r = rect.abs();
        let style = if smooth {
            ""
        } else {
            " style='image-rendering:pixelated'"
        };
        let _ = write!(
            self.out,
            "<image id='i{}' x='{}' y='{}' width='{}' height='{}' preserveAspectRatio='none'{style} \
             href='data:image/png;base64,{}'/>",
            self.ids.images,
            number::px(r.x0),
            number::px(r.y0),
            number::px(r.width()),
            number::px(r.height()),
            base64::encode(&png(image.width, image.height, &rgba))
        );
        Ok(())
    }
}

/// Stroke attributes beyond SVG's defaults (butt caps, miter joins, limit
/// 4); the start cap stands for both ends.
fn stroke(style: &kurbo::Stroke) -> String {
    let mut s = format!(" stroke-width='{}'", number::unit(style.width));
    match style.start_cap {
        Cap::Butt => {}
        Cap::Round => s.push_str(" stroke-linecap='round'"),
        Cap::Square => s.push_str(" stroke-linecap='square'"),
    }
    match style.join {
        Join::Miter if style.miter_limit != 4.0 => {
            let _ = write!(
                s,
                " stroke-miterlimit='{}'",
                number::unit(style.miter_limit)
            );
        }
        Join::Miter => {}
        Join::Round => s.push_str(" stroke-linejoin='round'"),
        Join::Bevel => s.push_str(" stroke-linejoin='bevel'"),
    }
    if !style.dash_pattern.is_empty() {
        let dashes: Vec<String> = style
            .dash_pattern
            .iter()
            .map(|d| number::unit(*d))
            .collect();
        let _ = write!(s, " stroke-dasharray='{}'", dashes.join(" "));
        if style.dash_offset != 0.0 {
            let _ = write!(
                s,
                " stroke-dashoffset='{}'",
                number::unit(style.dash_offset)
            );
        }
    }
    s
}

/// `text` escaped for an XML attribute in single quotes or for content.
fn xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// Straight RGBA as an 8-bit PNG with default compression.
fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = ::png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(::png::ColorType::Rgba);
    encoder.set_depth(::png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| {
            w.write_image_data(rgba)?;
            w.finish()
        })
        .expect("invariant: a checked image of positive size encodes into memory");
    bytes
}
