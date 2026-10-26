//! What does "typst for all text" cost per frame? Tick labels, laid out and turned into fills.

use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use d3::path::{PathContext, svg};
use lyon::path::Path as LyonPath;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration as TDuration};
use typst::layout::{Frame, FrameItem, Point};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook, FontVariant};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

const PX: f64 = 96.0 / 72.0;

struct Labels {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl Labels {
    fn new() -> Self {
        let fonts: Vec<Font> = typst_assets::fonts().flat_map(|d| Font::iter(Bytes::new(d))).collect();
        Self {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            main: Source::new(id(), String::new()),
        }
    }

    /// Lays out labels, one page each. Plain text goes in as a typst string, so `#`, `$`, `_` are inert.
    fn layout(&mut self, labels: &[String]) -> Vec<Frame> {
        let body: Vec<String> = labels
            .iter()
            .map(|l| if l.starts_with('$') { l.clone() } else { format!("#{l:?}") })
            .collect();
        let src = format!(
            "#set page(width: auto, height: auto, margin: 0pt)\n\
             #set text(size: 11pt, top-edge: \"bounds\", bottom-edge: \"bounds\")\n{}",
            body.join("\n#pagebreak()\n")
        );
        self.main = Source::new(id(), src);
        let doc: PagedDocument = typst::compile(&*self).output.expect("labels compile");
        doc.pages().iter().map(|p| p.frame.clone()).collect()
    }
}

fn id() -> FileId {
    FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new("/labels.typ").unwrap()))
}

impl World for Labels {
    fn library(&self) -> &LazyHash<Library> { &self.library }
    fn book(&self) -> &LazyHash<FontBook> { &self.book }
    fn main(&self) -> FileId { self.main.id() }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() { Ok(self.main.clone()) } else { Err(FileError::AccessDenied) }
    }
    fn file(&self, _: FileId) -> FileResult<Bytes> { Err(FileError::AccessDenied) }
    fn font(&self, i: usize) -> Option<Font> { self.fonts.get(i).cloned() }
    fn today(&self, _: Option<TDuration>) -> Option<Datetime> { None }
}

/// Glyph outlines from font units into any PathContext, at an offset and scale.
struct Pen<'a, P: PathContext> { p: &'a mut P, s: f64, x: f64, y: f64 }
impl<P: PathContext> Pen<'_, P> {
    fn at(&self, x: f32, y: f32) -> (f64, f64) { (self.x + x as f64 * self.s, self.y - y as f64 * self.s) }
}
impl<P: PathContext> ttf_parser::OutlineBuilder for Pen<'_, P> {
    fn move_to(&mut self, x: f32, y: f32) { let (x, y) = self.at(x, y); self.p.move_to(x, y) }
    fn line_to(&mut self, x: f32, y: f32) { let (x, y) = self.at(x, y); self.p.line_to(x, y) }
    fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
        let ((a, b), (x, y)) = (self.at(a, b), self.at(x, y));
        self.p.quadratic_curve_to(a, b, x, y)
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        let ((a, b), (c, d), (x, y)) = (self.at(a, b), self.at(c, d), self.at(x, y));
        self.p.bezier_curve_to(a, b, c, d, x, y)
    }
    fn close(&mut self) { self.p.close_path() }
}

/// Text items of a label frame, in pixels. (Groups and shapes are skipped: plain labels have none.)
fn draw(frame: &Frame, origin: Point, p: &mut impl PathContext, glyphs: &mut Vec<(usize, u16)>) {
    for (pos, item) in frame.items() {
        let pos = origin + *pos;
        match item {
            FrameItem::Group(g) => draw(&g.frame, pos + Point::new(g.transform.tx, g.transform.ty), p, glyphs),
            FrameItem::Text(t) => {
                let s = t.size.to_pt() / t.font.units_per_em() * PX;
                let mut x = pos.x.to_pt() * PX;
                for g in &t.glyphs {
                    let mut pen = Pen { p: &mut *p, s, x: x + g.x_offset.at(t.size).to_pt() * PX, y: pos.y.to_pt() * PX };
                    t.font.ttf().outline_glyph(ttf_parser::GlyphId(g.id), &mut pen);
                    glyphs.push((t.font.font() as *const Font as usize, g.id));
                    x += g.x_advance.at(t.size).to_pt() * PX;
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct Count(usize);
impl PathContext for Count {
    fn move_to(&mut self, _: f64, _: f64) { self.0 += 1 }
    fn line_to(&mut self, _: f64, _: f64) { self.0 += 1 }
    fn quadratic_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64) { self.0 += 1 }
    fn bezier_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: f64) { self.0 += 1 }
    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) { self.0 += 1 }
    fn close_path(&mut self) { self.0 += 1 }
}

/// lyon, as iced's wgpu canvas uses it: build a path, then tessellate the fill.
#[derive(Default)]
struct Lyon { b: Option<lyon::path::path::Builder>, open: bool }
impl Lyon {
    fn b(&mut self) -> &mut lyon::path::path::Builder { self.b.get_or_insert_with(LyonPath::builder) }
    fn finish(mut self) -> LyonPath { if self.open { self.b().end(false); } self.b.take().unwrap_or_else(LyonPath::builder).build() }
}
impl PathContext for Lyon {
    fn move_to(&mut self, x: f64, y: f64) {
        if self.open { self.b().end(false); }
        self.b().begin(lyon::math::point(x as f32, y as f32));
        self.open = true;
    }
    fn line_to(&mut self, x: f64, y: f64) { self.b().line_to(lyon::math::point(x as f32, y as f32)); }
    fn quadratic_curve_to(&mut self, a: f64, b: f64, x: f64, y: f64) {
        self.b().quadratic_bezier_to(lyon::math::point(a as f32, b as f32), lyon::math::point(x as f32, y as f32));
    }
    fn bezier_curve_to(&mut self, a: f64, b: f64, c: f64, d: f64, x: f64, y: f64) {
        self.b().cubic_bezier_to(lyon::math::point(a as f32, b as f32), lyon::math::point(c as f32, d as f32), lyon::math::point(x as f32, y as f32));
    }
    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) {}
    fn close_path(&mut self) { if self.open { self.b().end(true); self.open = false; } }
}

fn tessellate(path: &LyonPath) -> usize {
    let mut out: VertexBuffers<lyon::math::Point, u32> = VertexBuffers::new();
    FillTessellator::new()
        .tessellate_path(path, &FillOptions::tolerance(0.1), &mut BuffersBuilder::new(&mut out, |v: FillVertex| v.position()))
        .expect("tessellates");
    out.indices.len() / 3
}

fn time<T>(runs: u32, mut f: impl FnMut() -> T) -> Duration {
    let t = Instant::now();
    for _ in 0..runs { black_box(f()); }
    t.elapsed() / runs
}

fn ticks(from: f64) -> Vec<String> {
    (0..40).map(|i| format!("{:.1}", from + i as f64 * 0.1)).collect()
}

fn main() {
    let mut w = Labels::new();
    let labels = ticks(1000.0);
    w.layout(&labels); // warm the library and fonts once, as a running app would be

    println!("40 tick labels \"1000.0\" … \"1003.9\", 11 pt, 2-core container, release build");
    println!("typst layout:");
    let mut k = 0;
    println!("  one document per label   {:>9.2?}  (all new labels)", time(5, || { k += 1; for l in ticks(5000.0 + k as f64 * 9.1) { w.layout(&[l]); } }));
    let mut k = 0;
    println!("  one document, 40 pages   {:>9.2?}  (all new labels)", time(10, || { k += 1; w.layout(&ticks(9000.0 + k as f64 * 9.1)) }));
    println!("  same labels again        {:>9.2?}  (memoized)", time(50, || w.layout(&labels)));
    let mut k = 0;
    println!("  panned by one tick       {:>9.2?}  (39 of 40 labels already seen)", time(10, || { k += 1; w.layout(&ticks(1000.0 + k as f64 * 0.1)) }));
    let mut k = 0;
    println!("  zoomed: all new labels   {:>9.2?}", time(10, || { k += 1; w.layout(&ticks(2000.0 + k as f64 * 7.3)) }));
    let math: Vec<String> = ["$10^3$", "$alpha_1$", "$sqrt(x)$", "$1/2$", "$x^2$"].map(String::from).to_vec();
    println!("  5 small math labels      {:>9.2?}", time(10, || w.layout(&math)));

    // Fast path for plain labels: shape with rustybuzz directly, same font typst picked.
    let frames = w.layout(&labels);
    let idx = w.book.select("libertinus serif", FontVariant::default()).expect("font");
    let font = w.fonts[idx].clone();
    let face = rustybuzz::Face::from_slice(font.data().as_slice(), font.index()).expect("face");
    let shaped = time(200, || {
        labels.iter().map(|l| {
            let mut b = rustybuzz::UnicodeBuffer::new();
            b.push_str(l);
            rustybuzz::shape(&face, &[], b).glyph_positions().iter().map(|p| p.x_advance).sum::<i32>()
        }).sum::<i32>()
    });
    println!("rustybuzz, 40 labels       {:>9.2?}  (no typst layout)", shaped);

    let mut glyphs = Vec::new();
    let into_count = time(200, || { let mut c = Count::default(); glyphs.clear(); for f in &frames { draw(f, Point::zero(), &mut c, &mut glyphs); } c.0 });
    let into_svg = time(50, || { glyphs.clear(); frames.iter().map(|f| svg(|p| draw(f, Point::zero(), p, &mut glyphs)).len()).sum::<usize>() });
    let unique: HashSet<_> = glyphs.iter().copied().collect();
    println!("outlines → PathContext:");
    println!("  counting sink            {:>9.2?}  ({} glyphs, {} distinct)", into_count, glyphs.len(), unique.len());
    println!("  SVG path strings         {:>9.2?}", into_svg);

    let paths: Vec<LyonPath> = frames.iter().map(|f| { let mut l = Lyon::default(); draw(f, Point::zero(), &mut l, &mut Vec::new()); l.finish() }).collect();
    let tris: usize = paths.iter().map(tessellate).sum();
    println!("lyon fill tessellation (wgpu canvas path):");
    println!("  all 40 labels            {:>9.2?}  ({tris} triangles)", time(50, || paths.iter().map(tessellate).sum::<usize>()));
    // Glyph cache: tessellate each distinct glyph once; a frame then only translates meshes.
    let digits: Vec<LyonPath> = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "."]
        .iter()
        .map(|g| { let f = &w.layout(&[g.to_string()])[0]; let mut l = Lyon::default(); draw(f, Point::zero(), &mut l, &mut Vec::new()); l.finish() })
        .collect();
    println!("  11 distinct glyphs once  {:>9.2?}", time(50, || digits.iter().map(tessellate).sum::<usize>()));

    let mut pix = tiny_skia::Pixmap::new(400, 24).unwrap();
    let mut paint = tiny_skia::Paint::default();
    paint.anti_alias = true;
    let sk: Vec<tiny_skia::Path> = frames.iter().filter_map(|f| {
        let mut b = Sk(tiny_skia::PathBuilder::new());
        draw(f, Point::new(typst::layout::Abs::pt(4.0), typst::layout::Abs::pt(2.0)), &mut b, &mut Vec::new());
        b.0.finish()
    }).collect();
    println!("tiny-skia anti-aliased fill:");
    println!("  all 40 labels            {:>9.2?}", time(50, || for p in &sk { pix.fill_path(p, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None); }));
}

struct Sk(tiny_skia::PathBuilder);
impl PathContext for Sk {
    fn move_to(&mut self, x: f64, y: f64) { self.0.move_to(x as f32, y as f32) }
    fn line_to(&mut self, x: f64, y: f64) { self.0.line_to(x as f32, y as f32) }
    fn quadratic_curve_to(&mut self, a: f64, b: f64, x: f64, y: f64) { self.0.quad_to(a as f32, b as f32, x as f32, y as f32) }
    fn bezier_curve_to(&mut self, a: f64, b: f64, c: f64, d: f64, x: f64, y: f64) { self.0.cubic_to(a as f32, b as f32, c as f32, d as f32, x as f32, y as f32) }
    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) {}
    fn close_path(&mut self) { self.0.close() }
}
