//! Labels → glyph runs. One typst document per list,
//! one page per label; each page's text items become
//! runs that keep their font, positions and source text.

use figmodel::plot::draw::{
    Anchor, Command, Glyph, GlyphRun, Label, List,
};
use kurbo::Affine;
use peniko::{Blob, Color, FontData};
use std::collections::HashMap;
use std::sync::Arc;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::layout::{Frame, FrameItem, Point as TPoint};
use typst::syntax::{
    FileId, RootedPath, Source, VirtualPath, VirtualRoot,
};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::visualize::Paint;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

pub struct Shaper {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

struct Doc<'a> {
    s: &'a Shaper,
    main: Source,
}

impl World for Doc<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.s.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.s.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, _: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }
    fn font(&self, i: usize) -> Option<Font> {
        self.s.fonts.get(i).cloned()
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

const FONTS: [&str; 2] = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
];

impl Shaper {
    /// DejaVu Sans for text, typst's bundled math font.
    pub fn new() -> Self {
        let mut fonts: Vec<Font> = FONTS
            .iter()
            .flat_map(|p| {
                let d = std::fs::read(p).expect("font");
                Font::iter(Bytes::new(d)).collect::<Vec<_>>()
            })
            .collect();
        fonts.extend(
            typst_assets::fonts()
                .filter(|_| true)
                .flat_map(|d| Font::iter(Bytes::new(d)))
                .filter(|f| f.info().family.contains("Math")),
        );
        Shaper {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
        }
    }

    /// Replaces every `Label` in `l` with glyph runs.
    pub fn shape(&self, l: &mut List) {
        let labels: Vec<&Label> = l
            .commands
            .iter()
            .filter_map(|c| match c {
                Command::Label(x) => Some(x),
                _ => None,
            })
            .collect();
        if labels.is_empty() {
            return;
        }
        let mut src = String::from(
            "#set page(width: auto, height: auto, \
             margin: 0pt)\n\
             #set text(font: \"DejaVu Sans\", \
             top-edge: \"ascender\", \
             bottom-edge: \"descender\")\n",
        );
        for (i, x) in labels.iter().enumerate() {
            if i > 0 {
                src += "#pagebreak()\n";
            }
            let [r, g, b, a] = x.fill.to_rgba8().to_u8_array();
            let w = if x.bold { "bold" } else { "regular" };
            src += &format!(
                "#text(size: {}pt, weight: \"{w}\", \
                 fill: rgb({r}, {g}, {b}, {a}))[{}]\n",
                x.size,
                x.text.typst()
            );
        }
        let id = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("/labels.typ").unwrap(),
        ));
        let doc = Doc { s: self, main: Source::new(id, src) };
        let out: PagedDocument = typst::compile(&doc)
            .output
            .expect("labels compile");
        let pages = out.pages();
        assert_eq!(pages.len(), labels.len(), "one page each");

        // Fonts are numbered in order of first use.
        let mut table: HashMap<Font, usize> = HashMap::new();
        let mut runs: Vec<Vec<GlyphRun>> = Vec::new();
        for (x, page) in labels.iter().zip(pages) {
            let f = &page.frame;
            let mut items = Vec::new();
            collect(f, TPoint::zero(), &mut items);
            let base = items
                .first()
                .map_or(0.0, |(p, _)| p.y.to_pt());
            let w = f.width().to_pt();
            let dx = match x.anchor {
                Anchor::Start => 0.0,
                Anchor::Middle => -w / 2.0,
                Anchor::End => -w,
            };
            let place = Affine::translate(x.at.to_vec2())
                * Affine::rotate(x.angle)
                * Affine::translate((dx, -base));
            let mut out = Vec::new();
            for (p, t) in items {
                let n = table.len();
                let font = *table
                    .entry(t.font.font().clone())
                    .or_insert(n);
                let size = t.size.to_pt();
                let mut pen = 0.0;
                let glyphs = t
                    .glyphs
                    .iter()
                    .map(|g| {
                        let adv = g.x_advance.at(t.size).to_pt();
                        let q = Glyph {
                            id: g.id as u32,
                            x: (pen + g.x_offset.at(t.size).to_pt())
                                as f32,
                            y: -g.y_offset.at(t.size).to_pt() as f32,
                            advance: adv as f32,
                            text: (
                                g.range.start as u32,
                                g.range.end as u32,
                            ),
                        };
                        pen += adv;
                        q
                    })
                    .collect();
                let fill = match &t.fill {
                    Paint::Solid(c) => {
                        let [r, g, b, a] = c.to_vec4_u8();
                        Color::from_rgba8(r, g, b, a)
                    }
                    _ => x.fill,
                };
                out.push(GlyphRun {
                    font,
                    size: size as f32,
                    transform: place
                        * Affine::translate((
                            p.x.to_pt(),
                            p.y.to_pt(),
                        )),
                    fill,
                    glyphs,
                    text: Arc::from(t.text.as_str()),
                });
            }
            runs.push(out);
        }
        let mut fonts: Vec<(usize, Font)> =
            table.into_iter().map(|(f, i)| (i, f)).collect();
        fonts.sort_by_key(|(i, _)| *i);
        l.fonts = fonts
            .into_iter()
            .map(|(_, f)| {
                let blob = Blob::new(Arc::new(f.data().clone()));
                FontData::new(blob, f.index())
            })
            .collect();
        let mut runs = runs.into_iter();
        let old = std::mem::take(&mut l.commands);
        for c in old {
            match c {
                Command::Label(_) => l.commands.extend(
                    runs.next().unwrap().into_iter().map(Command::Glyphs),
                ),
                c => l.commands.push(c),
            }
        }
    }
}

/// Text items of a frame with their absolute positions.
fn collect<'a>(
    f: &'a Frame,
    at: TPoint,
    out: &mut Vec<(TPoint, &'a typst::text::TextItem)>,
) {
    for (p, item) in f.items() {
        let p = at + *p;
        match item {
            FrameItem::Text(t) => out.push((p, t)),
            // Labels carry no transforms; math may nest.
            FrameItem::Group(g) => collect(&g.frame, p, out),
            _ => {}
        }
    }
}

impl Shaper {
    /// Widths of several labels from one typst
    /// document, one page each: (markup, size px, bold).
    pub fn widths(&self, labels: &[(String, f64, bool)]) -> Vec<f64> {
        if labels.is_empty() {
            return vec![];
        }
        let mut src = String::from(
            "#set page(width: auto, height: auto, margin: 0pt)\n\
             #set text(font: \"DejaVu Sans\")\n",
        );
        for (i, (m, size, bold)) in labels.iter().enumerate() {
            if i > 0 {
                src += "#pagebreak()\n";
            }
            let w = if *bold { "bold" } else { "regular" };
            src += &format!("#text(size: {size}pt, weight: \"{w}\")[{m}]\n");
        }
        let id = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("/measure.typ").unwrap(),
        ));
        let doc = Doc { s: self, main: Source::new(id, src) };
        let out: PagedDocument =
            typst::compile(&doc).output.expect("labels compile");
        out.pages().iter().map(|p| p.frame.width().to_pt()).collect()
    }
}
