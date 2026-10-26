//! The bundled fonts, parsed with ttf-parser.
#![allow(clippy::unwrap_used, reason = "test helpers outside #[test] fns")]

use rustd3_fonts::{ALL, DEJAVU_SANS, DEJAVU_SANS_BOLD, Font, NEW_CM_MATH};
use ttf_parser::{Face, name_id};

/// Narrow no-break space and minus (the neutral locale's group separator
/// and minus sign), micro sign and euro sign.
const LOCALE_CHARS: [char; 4] = ['\u{202F}', '\u{2212}', '\u{B5}', '\u{20AC}'];

/// What a font's own tables say about it.
struct Parsed {
    family: String,
    bold: bool,
    math: bool,
    glyphs: u16,
    missing: Vec<char>,
}

impl From<Font> for Parsed {
    fn from(font: Font) -> Self {
        let face = Face::parse(font.data(), 0).unwrap();
        let family = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == name_id::FAMILY)
            .find_map(|name| name.to_string())
            .unwrap();
        Parsed {
            family,
            bold: face.is_bold(),
            math: face.tables().math.is_some(),
            glyphs: face.number_of_glyphs(),
            missing: LOCALE_CHARS
                .into_iter()
                .filter(|&c| face.glyph_index(c).is_none())
                .collect(),
        }
    }
}

#[test]
fn dejavu_sans_is_regular_dejavu_sans_with_a_math_table() {
    let parsed = Parsed::from(DEJAVU_SANS);
    assert_eq!(parsed.family, "DejaVu Sans");
    assert!(!parsed.bold);
    assert!(parsed.math);
    assert_eq!(parsed.glyphs, 6_253);
    assert_eq!(parsed.missing, Vec::<char>::new());
}

#[test]
fn dejavu_sans_bold_is_bold_dejavu_sans_without_a_math_table() {
    let parsed = Parsed::from(DEJAVU_SANS_BOLD);
    assert_eq!(parsed.family, "DejaVu Sans");
    assert!(parsed.bold);
    assert!(!parsed.math);
    assert_eq!(parsed.glyphs, 6_196);
    assert_eq!(parsed.missing, Vec::<char>::new());
}

#[test]
fn new_cm_math_is_regular_new_computer_modern_math_with_a_math_table() {
    let parsed = Parsed::from(NEW_CM_MATH);
    assert_eq!(parsed.family, "NewComputerModernMath");
    assert!(!parsed.bold);
    assert!(parsed.math);
    assert_eq!(parsed.glyphs, 8_603);
    assert_eq!(parsed.missing, Vec::<char>::new());
}

#[test]
fn every_font_reports_the_family_and_weight_its_tables_hold() {
    for font in ALL {
        let parsed = Parsed::from(font);
        assert_eq!(font.family(), parsed.family);
        assert_eq!(font.bold(), parsed.bold);
    }
}
