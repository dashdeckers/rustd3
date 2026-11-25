//! [`Span`]: literal text in one weight, color and family.

use rustd3_core::color::Rgb;

use crate::text::{literal, rgb};

/// Literal text, plain or bold, in the label's color and family or its
/// own. Its characters reach typst as a string, so `$`, `#` and `_` stay
/// literal.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    text: String,
    fill: Option<Rgb>,
    font: Option<&'static str>,
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
    #[must_use]
    pub fn fill(self, color: Rgb) -> Self {
        Span {
            fill: Some(color),
            ..self
        }
    }

    /// The same text in font family `family`, as its name table gives it.
    ///
    /// ```
    /// use rustd3_figure::text::{Text, span};
    ///
    /// let readout = Text::from("RMS ").push(span("  0.12").font("DejaVu Sans Mono"));
    /// assert_eq!(readout.typst(), r##"#"RMS "#text(font: "DejaVu Sans Mono", "  0.12")"##);
    /// ```
    #[must_use]
    pub fn font(self, family: &'static str) -> Self {
        Span {
            font: Some(family),
            ..self
        }
    }

    /// The same text in bold.
    #[must_use]
    pub fn bold(self) -> Self {
        Span { bold: true, ..self }
    }

    /// Whether the span has no characters.
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The span as typst markup: `#"lit"`, wrapped in `strong(…)` when bold
    /// and `text(font: "…", fill: rgb(…), …)` when it has a family or a
    /// color of its own.
    pub(crate) fn markup(&self, out: &mut String) {
        let mut code = literal(&self.text);
        if self.bold {
            code = format!("strong({code})");
        }
        if let Some(color) = self.fill {
            code = format!("fill: {}, {code}", rgb(color));
        }
        if let Some(family) = self.font {
            code = format!("font: {}, {code}", literal(family));
        }
        if self.fill.is_some() || self.font.is_some() {
            code = format!("text({code})");
        }
        out.push('#');
        out.push_str(&code);
    }
}

/// A span of literal `text`, plain and in the label's color and family.
pub fn span(text: impl Into<String>) -> Span {
    Span {
        text: text.into(),
        fill: None,
        font: None,
        bold: false,
    }
}
