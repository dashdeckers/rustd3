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
///
/// ```
/// use rustd3_figure::text;
///
/// // An app's own font files go first; bytes that hold no face are
/// // rejected with their position.
/// let bad = text::Fonts::with([b"not a font".to_vec()]).unwrap_err();
/// assert_eq!(bad, text::Error::Font { index: 0 });
/// ```
pub struct Fonts {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    faces: Vec<(Font, FontData)>,
    /// Family names as typst reports them, in fallback order, each once.
    families: Vec<String>,
    /// The text face's ascender and descender, em.
    line: [f64; 2],
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
    /// DejaVu Sans and DejaVu Sans Mono in two weights each and New
    /// Computer Modern Math, from `rustd3-fonts`; parsed once per process.
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
    /// on.
    ///
    /// # Errors
    ///
    /// [`Error::Font`] with the position in `extra` of bytes that hold no
    /// face.
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
        let line = faces.first().map_or([0.0, 0.0], |(font, _)| line(font));
        Fonts {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(faces.iter().map(|(font, _)| font))),
            faces,
            families,
            line,
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

    /// The line box a row of labels of `size` px shares, in px: the text
    /// face's ascender above the baseline and descender below it. The text
    /// face is the first in fallback order, and its metrics are read as
    /// typst reads them (the OS/2 typographic metrics, else hhea's). Both
    /// are 0 without faces.
    pub(crate) fn line(&self, size: f64) -> [f64; 2] {
        self.line.map(|em| em * size)
    }
}

/// `font`'s ascender above the baseline and descender below it, em, as
/// typst reads them: the OS/2 typographic metrics, else hhea's.
fn line(font: &Font) -> [f64; 2] {
    let Ok(face) = ttf_parser::Face::parse(font.data(), font.index()) else {
        return [0.0, 0.0];
    };
    let ascender = face.typographic_ascender().unwrap_or(face.ascender());
    let descender = face.typographic_descender().unwrap_or(face.descender());
    let em = f64::from(face.units_per_em());
    [f64::from(ascender) / em, -f64::from(descender) / em]
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
            [
                "DejaVu Sans",
                "New Computer Modern Math",
                "DejaVu Sans Mono"
            ]
        );
    }

    #[test]
    fn the_family_constants_name_bundled_families() {
        let bundled = Fonts::bundled();
        for family in [rustd3_fonts::SANS, rustd3_fonts::MONO] {
            assert!(bundled.families().iter().any(|f| f == family), "{family}");
        }
    }

    #[test]
    fn user_families_come_first_and_each_family_once() {
        let fonts = Fonts::with([rustd3_fonts::NEW_CM_MATH.data().to_vec()]).unwrap();
        assert_eq!(
            fonts.families(),
            [
                "New Computer Modern Math",
                "DejaVu Sans",
                "DejaVu Sans Mono"
            ]
        );
        assert_eq!(fonts.faces.len(), 6);
    }

    #[test]
    fn the_line_box_is_dejavu_sans_typographic_ascender_and_descender() {
        // OS/2 sTypoAscender 1556 and sTypoDescender −492 of 2048 units.
        let [ascender, descender] = Fonts::bundled().line(2048.0);
        assert!((ascender - 1556.0).abs() < 1e-9, "{ascender}");
        assert!((descender - 492.0).abs() < 1e-9, "{descender}");
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
