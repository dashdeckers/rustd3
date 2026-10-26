//! Spike: typst math -> frame -> glyph outlines -> d3's PathContext.

use std::time::Instant;

use d3::path::{PathContext, svg};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::layout::{Frame, FrameItem, Point, Transform};
use typst_layout::{Page, PagedDocument};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::visualize::{CurveItem, Geometry};
use typst::{Library, LibraryExt, World};

struct Math {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl Math {
    fn new() -> Self {
        let fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        Self {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            main: Source::new(Self::id(), String::new()),
        }
    }

    fn id() -> FileId {
        let path = VirtualPath::new("/label.typ").expect("valid path");
        FileId::new(RootedPath::new(VirtualRoot::Project, path))
    }

    /// Lays out one label: a tight page, so the frame is the label's box.
    fn layout(&mut self, label: &str) -> Page {
        let src = format!(
            "#set page(width: auto, height: auto, margin: 0pt)\n\
             #set text(size: 11pt, top-edge: \"bounds\", bottom-edge: \"bounds\")\n{label}"
        );
        self.main = Source::new(Self::id(), src);
        let doc: PagedDocument = typst::compile(&*self)
            .output
            .expect("label compiles");
        doc.pages()[0].clone()
    }
}

impl World for Math {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
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
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// Glyph outlines in font units, mapped through an affine transform into a PathContext.
struct Pen<'a, P: PathContext> {
    p: &'a mut P,
    t: Transform,
    s: f64,
    x: f64,
    y: f64,
}

impl<P: PathContext> Pen<'_, P> {
    fn at(&self, gx: f32, gy: f32) -> (f64, f64) {
        // Font units are y-up; the page is y-down.
        let (x, y) = (self.x + gx as f64 * self.s, -(self.y + gy as f64 * self.s));
        apply(self.t, x, y)
    }
}

impl<P: PathContext> ttf_parser::OutlineBuilder for Pen<'_, P> {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.at(x, y);
        self.p.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.at(x, y);
        self.p.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let ((x1, y1), (x, y)) = (self.at(x1, y1), self.at(x, y));
        self.p.quadratic_curve_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let ((x1, y1), (x2, y2), (x, y)) = (self.at(x1, y1), self.at(x2, y2), self.at(x, y));
        self.p.bezier_curve_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.p.close_path();
    }
}

fn apply(t: Transform, x: f64, y: f64) -> (f64, f64) {
    (
        t.sx.get() * x + t.kx.get() * y + t.tx.to_pt(),
        t.ky.get() * x + t.sy.get() * y + t.ty.to_pt(),
    )
}

fn translate(t: Transform, p: Point) -> Transform {
    t.pre_concat(Transform::translate(p.x, p.y))
}

/// Walks a typst frame and draws every glyph and shape as filled geometry.
fn draw(frame: &Frame, t: Transform, p: &mut impl PathContext) -> usize {
    let mut glyphs = 0;
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(g) => {
                let t = translate(t, *pos).pre_concat(g.transform);
                glyphs += draw(&g.frame, t, p);
            }
            FrameItem::Text(text) => {
                let t = translate(t, *pos);
                let face = text.font.ttf();
                let s = text.size.to_pt() / text.font.units_per_em();
                let (mut x, mut y) = (0.0, 0.0);
                for g in &text.glyphs {
                    let mut pen = Pen {
                        p: &mut *p,
                        t,
                        s,
                        x: x + g.x_offset.at(text.size).to_pt(),
                        y: y + g.y_offset.at(text.size).to_pt(),
                    };
                    let id = ttf_parser::GlyphId(g.id);
                    if face.outline_glyph(id, &mut pen).is_some() {
                        glyphs += 1;
                    }
                    x += g.x_advance.at(text.size).to_pt();
                    y += g.y_advance.at(text.size).to_pt();
                }
            }
            FrameItem::Shape(shape, _) => {
                let t = translate(t, *pos);
                let half = shape
                    .stroke
                    .as_ref()
                    .map_or(0.0, |s| s.thickness.to_pt() / 2.0);
                match &shape.geometry {
                    // Fraction bars and radical overlines: a stroked line, drawn as a thin rectangle.
                    Geometry::Line(to) => {
                        let (x1, y1) = apply(t, 0.0, -half);
                        let (x2, y2) = apply(t, to.x.to_pt(), -half);
                        let (x3, y3) = apply(t, to.x.to_pt(), half);
                        let (x4, y4) = apply(t, 0.0, half);
                        p.move_to(x1, y1);
                        p.line_to(x2, y2);
                        p.line_to(x3, y3);
                        p.line_to(x4, y4);
                        p.close_path();
                    }
                    Geometry::Rect(size) => {
                        let corners = [(0.0, 0.0), (size.x.to_pt(), 0.0), (size.x.to_pt(), size.y.to_pt()), (0.0, size.y.to_pt())];
                        for (i, (x, y)) in corners.into_iter().enumerate() {
                            let (x, y) = apply(t, x, y);
                            if i == 0 { p.move_to(x, y) } else { p.line_to(x, y) }
                        }
                        p.close_path();
                    }
                    Geometry::Curve(c) => {
                        for item in &c.0 {
                            match *item {
                                CurveItem::Move(a) => {
                                    let (x, y) = apply(t, a.x.to_pt(), a.y.to_pt());
                                    p.move_to(x, y);
                                }
                                CurveItem::Line(a) => {
                                    let (x, y) = apply(t, a.x.to_pt(), a.y.to_pt());
                                    p.line_to(x, y);
                                }
                                CurveItem::Cubic(a, b, c) => {
                                    let (x1, y1) = apply(t, a.x.to_pt(), a.y.to_pt());
                                    let (x2, y2) = apply(t, b.x.to_pt(), b.y.to_pt());
                                    let (x, y) = apply(t, c.x.to_pt(), c.y.to_pt());
                                    p.bezier_curve_to(x1, y1, x2, y2, x, y);
                                }
                                CurveItem::Close => p.close_path(),
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    glyphs
}

fn main() {
    let labels = [
        ("sum", "$ sum_(i=1)^n x_i^2 = (n(n+1)(2n+1))/6 $"),
        ("axis", "Acceleration [$m slash s^2$]"),
        ("psd", "$ S_(x x)(f) = lim_(T -> oo) 1/T bb(E)[abs(hat(x)_T (f))^2] $"),
        ("sqrt", "$ sigma = sqrt(1/N sum_(i=1)^N (x_i - mu)^2) $"),
    ];

    let t0 = Instant::now();
    let mut math = Math::new();
    println!("fonts loaded: {} faces in {:?}", math.fonts.len(), t0.elapsed());

    let out = std::path::Path::new("out");
    std::fs::create_dir_all(out).unwrap();
    for (name, label) in labels {
        let t = Instant::now();
        let page = math.layout(label);
        let frame = &page.frame;
        let layout = t.elapsed();
        let t = Instant::now();
        let mut glyphs = 0;
        let d = svg(|p| glyphs = draw(frame, Transform::identity(), p));
        let paths = t.elapsed();
        let (w, h) = (frame.width().to_pt(), frame.height().to_pt());
        println!(
            "{name:5} layout {:>9.3?}  to paths {:>8.3?}  {w:6.2} x {h:5.2} pt  baseline {:5.2} pt  {glyphs} glyphs  {} bytes of path",
            layout, paths, frame.baseline().to_pt(), d.len()
        );
        // Ours: one filled path through d3's PathContext. Reference: typst's own SVG export.
        let ours = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {w} {h}' width='{}' height='{}'><path d='{d}' fill='black'/></svg>",
            w * 4.0, h * 4.0
        );
        std::fs::write(out.join(format!("{name}_ours.svg")), ours).unwrap();
        let reference = typst_svg::svg(&page, &typst_svg::SvgOptions::default());
        std::fs::write(out.join(format!("{name}_typst.svg")), reference).unwrap();
    }
    // Warm path: the same label again, as a redraw would ask for it.
    let t = Instant::now();
    let _ = math.layout(labels[0].1);
    println!("repeat layout of the first label: {:?}", t.elapsed());
}
