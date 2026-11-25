//! The faces rustd3's figures are set in, for an app's own text: an app
//! that loads [`all`] and names [`SANS`] as its default font draws its
//! interface in the faces its plots are drawn in. The `Plot` widget needs
//! none of this: it draws the figure's own glyphs, in the same faces as
//! the static outputs. An app passes them as
//! `iced::application(boot, update, view).fonts(rustd3_iced::font::all())
//! .font(rustd3_iced::font::SANS)`.
//!
//! ```
//! use iced::font::Family;
//! use rustd3_iced::font;
//!
//! assert_eq!(font::all().count(), 5);
//! assert_eq!(font::MONO.family, Family::Name("DejaVu Sans Mono"));
//! ```

use std::borrow::Cow;

use iced_core::Font;
use iced_core::font::Weight;

/// DejaVu Sans, the family of figure text (`rustd3_fonts::SANS`).
pub const SANS: Font = Font::new(rustd3_fonts::SANS);

/// DejaVu Sans Bold.
pub const SANS_BOLD: Font = SANS.weight(Weight::Bold);

/// DejaVu Sans Mono, the family of `raw` figure text (`rustd3_fonts::MONO`).
pub const MONO: Font = Font::new(rustd3_fonts::MONO);

/// DejaVu Sans Mono Bold.
pub const MONO_BOLD: Font = MONO.weight(Weight::Bold);

/// The bytes of every bundled face (`rustd3_fonts::ALL`), as iced's
/// `application(…).fonts(…)` and `iced::font::load` take them. They are
/// borrowed from the binary, not copied.
pub fn all() -> impl Iterator<Item = Cow<'static, [u8]>> {
    rustd3_fonts::ALL
        .into_iter()
        .map(|font| Cow::Borrowed(font.data()))
}
