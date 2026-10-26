//! Checks the figure sketch's typst lowering of rich text:
//! the markup `Text::typst` produced, compiled to SVG.

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

struct One {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl World for One {
    fn library(&self) -> &LazyHash<Library> { &self.library }
    fn book(&self) -> &LazyHash<FontBook> { &self.book }
    fn main(&self) -> FileId { self.main.id() }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() { Ok(self.main.clone()) } else { Err(FileError::AccessDenied) }
    }
    fn file(&self, _: FileId) -> FileResult<Bytes> { Err(FileError::AccessDenied) }
    fn font(&self, i: usize) -> Option<Font> { self.fonts.get(i).cloned() }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> { None }
}

fn main() {
    let markup = std::env::args().nth(1).expect("markup");
    let fonts: Vec<Font> = typst_assets::fonts().flat_map(|d| Font::iter(Bytes::new(d))).collect();
    let id = FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new("/t.typ").unwrap()));
    let src = format!("#set page(width: auto, height: auto, margin: 2pt)\n#set text(size: 12pt, weight: \"bold\")\n{markup}");
    let world = One {
        library: LazyHash::new(Library::builder().build()),
        book: LazyHash::new(FontBook::from_fonts(&fonts)),
        fonts,
        main: Source::new(id, src),
    };
    let doc: PagedDocument = typst::compile(&world).output.expect("compiles");
    let svg = typst_svg::svg(&doc.pages()[0], &typst_svg::SvgOptions::default());
    let red = svg.matches("#c44e52").count();
    println!("compiled; {} bytes of SVG; fills #c44e52: {red}", svg.len());
    std::fs::write("out/rich.svg", svg).unwrap();
}
