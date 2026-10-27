//! [`Span`]: literal text in one weight and color.

use rustd3_core::color::Rgb;

use crate::text::{literal, rgb};

/// Literal text, plain or bold, in the label's color or its own. Its
/// characters reach typst as a string, so `$`, `#` and `_` stay literal.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    text: String,
    fill: Option<Rgb>,
    bold: bool,
}

impl From<&str> for Span {
    fn from(text: &str) -> Span {
        span(text)
    }
}

impl From<String> for Span {
    fn from(text: String) -> Span {
        span(text)
    }
}

impl Span {
    /// The same text in `color`.
    pub fn fill(self, color: Rgb) -> Self {
        Span {
            fill: Some(color),
            ..self
        }
    }

    /// The same text in bold.
    pub fn bold(self) -> Self {
        Span { bold: true, ..self }
    }

    /// Whether the span has no characters.
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The span as typst markup: `#"lit"`, wrapped in `strong(…)` when bold
    /// and `text(fill: rgb(…), …)` when filled.
    pub(crate) fn markup(&self, out: &mut String) {
        let mut code = literal(&self.text);
        if self.bold {
            code = format!("strong({code})");
        }
        if let Some(color) = self.fill {
            code = format!("text(fill: {}, {code})", rgb(color));
        }
        out.push('#');
        out.push_str(&code);
    }
}

/// A span of literal `text`, plain and in the label's color.
pub fn span(text: impl Into<String>) -> Span {
    Span {
        text: text.into(),
        fill: None,
        bold: false,
    }
}
