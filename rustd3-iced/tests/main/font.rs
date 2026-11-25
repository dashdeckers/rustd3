//! `rustd3_iced::font`: the bundled faces load into iced's font system and
//! its fonts find them by name.

use iced::advanced::graphics::text::cosmic_text::{Style, fontdb};
use iced::advanced::graphics::text::font_system;
use iced::font::Weight;
use rustd3_iced::font::{MONO, MONO_BOLD, SANS, SANS_BOLD, all};

#[test]
fn every_bundled_face_loads_and_each_font_is_found_by_its_name() {
    assert_eq!(all().count(), rustd3_fonts::ALL.len());
    let mut system = font_system().write().expect("the font system");
    for bytes in all() {
        system.load_font(bytes);
    }
    let db = system.raw().db();
    for font in [SANS, SANS_BOLD, MONO, MONO_BOLD] {
        let iced::font::Family::Name(name) = font.family else {
            panic!("{font:?} names no family");
        };
        let bold = font.weight == Weight::Bold;
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(name)],
            weight: if bold {
                fontdb::Weight::BOLD
            } else {
                fontdb::Weight::NORMAL
            },
            stretch: fontdb::Stretch::Normal,
            style: Style::Normal,
        };
        let id = db.query(&query).expect("a face of the family");
        let face = db.face(id).expect("the face");
        assert!(
            face.families.iter().any(|(family, _)| family == name),
            "{:?}",
            face.families
        );
        assert_eq!(face.weight.0 >= 600, bold, "{name}");
        assert_eq!(face.monospaced, name == rustd3_fonts::MONO, "{name}");
    }
}
