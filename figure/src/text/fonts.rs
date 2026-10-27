//! [`Fonts`]: the faces typst shapes labels with.

use std::fmt;
use std::sync::Arc;

use peniko::{Blob, FontData};
use typst::foundations::Bytes;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

use crate::text::Error;

/// The faces typst shapes labels with, user fonts first in the fallback
/// order, with typst's standard library and font book over them, hashed
/// once. Each face keeps one [`FontData`], so its blob id, which glyph
/// caches and part hashes depend on, is the same in every list.
pub struct Fonts {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    faces: Vec<(Font, FontData)>,
    /// Family names as typst reports them, in fallback order, each once.
    families: Vec<String>,
}

impl fmt::Debug for Fonts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fonts")
            .field("faces", &self.faces.len())
            .field("families", &self.families)
            .finish_non_exhaustive()
    }
}

impl Fonts {
    /// DejaVu Sans, DejaVu Sans Bold and New Computer Modern Math, from
    /// `rustd3-fonts`; parsed once per process.
    #[cfg(feature = "fonts")]
    pub fn bundled() -> Arc<Fonts> {
        static BUNDLED: std::sync::OnceLock<Arc<Fonts>> = std::sync::OnceLock::new();
        BUNDLED
            .get_or_init(|| {
                let faces = rustd3_fonts::ALL
                    .iter()
                    .flat_map(|font| file(Bytes::new(font.data())))
                    .collect();
                Arc::new(Fonts::new(faces))
            })
            .clone()
    }

    /// `extra` font files (TrueType, OpenType or collections) first in the
    /// fallback order, then the bundled fonts when the `fonts` feature is
    /// on. Bytes that hold no face are `Error::Font` with their position
    /// in `extra`.
    pub fn with(extra: impl IntoIterator<Item = Vec<u8>>) -> Result<Fonts, Error> {
        let mut faces = Vec::new();
        for (index, bytes) in extra.into_iter().enumerate() {
            let found = file(Bytes::new(bytes));
            if found.is_empty() {
                return Err(Error::Font { index });
            }
            faces.extend(found);
        }
        #[cfg(feature = "fonts")]
        faces.extend(Fonts::bundled().faces.iter().cloned());
        Ok(Fonts::new(faces))
    }

    fn new(faces: Vec<(Font, FontData)>) -> Fonts {
        let mut families: Vec<String> = Vec::new();
        for (font, _) in &faces {
            let family = &font.info().family;
            if !families.contains(family) {
                families.push(family.clone());
            }
        }
        Fonts {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(faces.iter().map(|(font, _)| font))),
            faces,
            families,
        }
    }

    /// typst's standard library.
    pub(crate) fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// typst's book of the faces, indexed as the faces are.
    pub(crate) fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Face `index` as typst holds it.
    pub(crate) fn font(&self, index: usize) -> Option<Font> {
        self.faces.get(index).map(|(font, _)| font.clone())
    }

    /// Where `font` is among the faces.
    pub(crate) fn index(&self, font: &Font) -> Option<usize> {
        self.faces.iter().position(|(face, _)| face == font)
    }

    /// Face `index` as a list's font table holds it.
    pub(crate) fn data(&self, index: usize) -> &FontData {
        &self.faces[index].1
    }

    /// Family names in fallback order, as typst reports them.
    pub(crate) fn families(&self) -> &[String] {
        &self.families
    }
}

/// Every face typst reads from one font file, all sharing one blob.
fn file(bytes: Bytes) -> Vec<(Font, FontData)> {
    let blob = Blob::new(Arc::new(bytes.clone()));
    Font::iter(bytes)
        .map(|font| {
            let data = FontData::new(blob.clone(), font.index());
            (font, data)
        })
        .collect()
}

#[cfg(all(test, feature = "fonts"))]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_families_are_the_names_typst_reports() {
        assert_eq!(
            Fonts::bundled().families(),
            ["DejaVu Sans", "New Computer Modern Math"]
        );
    }

    #[test]
    fn user_families_come_first_and_each_family_once() {
        let fonts = Fonts::with([rustd3_fonts::NEW_CM_MATH.data().to_vec()]).unwrap();
        assert_eq!(
            fonts.families(),
            ["New Computer Modern Math", "DejaVu Sans"]
        );
        assert_eq!(fonts.faces.len(), 4);
    }

    #[test]
    fn faces_keep_their_blob_across_font_sets() {
        let bundled = Fonts::bundled();
        let with = Fonts::with([]).unwrap();
        assert_eq!(bundled.data(0).data.id(), with.data(0).data.id());
        assert_eq!(bundled.data(0).data.id(), bundled.data(0).clone().data.id());
        assert_ne!(bundled.data(0).data.id(), bundled.data(2).data.id());
    }
}
