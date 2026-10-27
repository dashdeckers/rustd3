//! Figure text through typst: labels written as literal spans or typst
//! markup, with explicit `\n` line breaks, shaped by typst in one document
//! per batch of new labels, kept in a caller-owned [`Cache`], and placed
//! into a drawing list as glyph runs and fills.
//!
//! ```
//! use rustd3_core::color::Rgb;
//! use rustd3_figure::text::{self, Text, span};
//!
//! let title = Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)));
//! assert_eq!(title.typst(), r##"#"Wheel: "#text(fill: rgb("#c44e52"), "RL")"##);
//! let unit = Text::from("RMS [").push(text::typst("$m slash s^2$")).push("]");
//! assert_eq!(unit.typst(), r#"#"RMS [";$m slash s^2$#"]""#);
//! ```

pub mod anchor;
mod batch;
mod cache;
mod convert;
mod error;
mod fonts;
mod key;
mod shaped;
mod span;
mod style;
mod world;

use rustd3_core::color::Rgb;

pub use cache::{Cache, Stats};
pub use error::Error;
pub use fonts::Fonts;
pub use key::Key;
pub use shaped::Shaped;
pub use span::{Span, span};
pub use style::Style;

/// A label's text: literal spans and raw typst markup, in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Text {
    pieces: Vec<Piece>,
}

/// One piece of a [`Text`].
#[derive(Debug, Clone, PartialEq)]
enum Piece {
    /// Literal text.
    Span(Span),
    /// typst markup, inserted verbatim.
    Markup(String),
}

/// Where a label's box sits relative to the point it is drawn at.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Anchor {
    /// Which vertical edge, or the middle, is at the point.
    pub h: anchor::Horizontal,
    /// Which line, or the middle, is at the point.
    pub v: anchor::Vertical,
}

impl From<Span> for Text {
    fn from(span: Span) -> Text {
        Text {
            pieces: vec![Piece::Span(span)],
        }
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Text {
        span(text).into()
    }
}

impl From<String> for Text {
    fn from(text: String) -> Text {
        span(text).into()
    }
}

impl Text {
    /// This text followed by `piece`.
    pub fn push(mut self, piece: impl Into<Text>) -> Self {
        self.pieces.extend(piece.into().pieces);
        self
    }

    /// Whether the text has no characters.
    pub fn is_empty(&self) -> bool {
        self.pieces.iter().all(|piece| match piece {
            Piece::Span(span) => span.is_empty(),
            Piece::Markup(markup) => markup.is_empty(),
        })
    }

    /// The text as typst markup: each span a string literal (see
    /// [`Span`]), markup as it was given. A span followed by markup ends
    /// with `;`, so markup that starts with `(`, `[` or `.` does not
    /// continue the span's expression.
    pub fn typst(&self) -> String {
        let mut out = String::new();
        let mut after_span = false;
        for piece in &self.pieces {
            match piece {
                Piece::Span(span) => {
                    span.markup(&mut out);
                    after_span = true;
                }
                Piece::Markup(markup) => {
                    if after_span {
                        out.push(';');
                    }
                    out.push_str(markup);
                    after_span = false;
                }
            }
        }
        out
    }
}

/// Raw typst markup, math included (`"$m slash s^2$"`), inserted verbatim.
pub fn typst(markup: impl Into<String>) -> Text {
    Text {
        pieces: vec![Piece::Markup(markup.into())],
    }
}

/// `text` as a typst string literal: `\` and `"` escaped, newlines and
/// tabs as `\n` and `\t`, carriage returns dropped.
fn literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `color`'s channels and opacity as 0–255 integers, rounded and clamped.
fn channels(color: Rgb) -> [u8; 4] {
    let c = color.clamp();
    // `clamp` leaves every value a whole number in 0..=255 or in [0, 1].
    [c.r, c.g, c.b, (c.opacity * 255.0).round()].map(|v| v as u8)
}

/// `color` as typst: `rgb("#rrggbb")` when opaque, else `rgb(r, g, b, a)`.
fn rgb(color: Rgb) -> String {
    match channels(color) {
        [r, g, b, 255] => format!("rgb(\"#{r:02x}{g:02x}{b:02x}\")"),
        [r, g, b, a] => format!("rgb({r}, {g}, {b}, {a})"),
    }
}
