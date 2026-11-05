//! [`World`]: what typst sees while it compiles one batch.

use std::sync::Arc;

use typst::Library;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;

use crate::text::Fonts;

/// One batch's world: the fonts and the batch's source, no files and no
/// clock. A plain value, built per batch. typst's memoization still hits
/// across batches because the library and book hash once in [`Fonts`].
pub(crate) struct World {
    fonts: Arc<Fonts>,
    main: Source,
}

impl World {
    /// The world in which `source` is the main and only file.
    pub(crate) fn new(fonts: Arc<Fonts>, source: String) -> World {
        let path = VirtualPath::new("/labels.typ")
            .expect("invariant: an absolute path of one plain segment is valid");
        let id = RootedPath::new(VirtualRoot::Project, path).intern();
        World {
            fonts,
            main: Source::new(id, source),
        }
    }
}

impl typst::World for World {
    fn library(&self) -> &LazyHash<Library> {
        self.fonts.library()
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
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
        self.fonts.font(index)
    }

    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

#[cfg(all(test, feature = "fonts"))]
mod tests {
    use typst_layout::PagedDocument;

    use super::*;

    fn compile(source: &str) -> Result<usize, String> {
        let world = World::new(Fonts::bundled(), source.into());
        typst::compile::<PagedDocument>(&world)
            .output
            .map(|doc| doc.pages().len())
            .map_err(|errors| errors.iter().map(|e| e.message.to_string()).collect())
    }

    #[test]
    fn a_source_compiles_against_the_bundled_fonts() {
        assert_eq!(compile("a#pagebreak()b"), Ok(2));
    }

    #[test]
    fn the_world_has_no_files_and_no_clock() {
        assert!(compile("#image(\"a.png\")").is_err());
        assert!(compile("#datetime.today()").is_err());
    }
}
