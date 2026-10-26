//! Rich text as spans. The real backend lowers a `Text`
//! to typst markup (`Text::typst`); this sketch also
//! writes SVG `<tspan>`s so it can render without typst.

use d3::color::Rgb;
use std::fmt::Write;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Text {
    spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    text: String,
    fill: Option<Rgb>,
    bold: bool,
}

pub fn span(text: impl Into<String>) -> Span {
    Span { text: text.into(), fill: None, bold: false }
}

impl Span {
    pub fn fill(self, fill: Rgb) -> Self {
        Self { fill: Some(fill), ..self }
    }

    pub fn bold(self) -> Self {
        Self { bold: true, ..self }
    }
}

impl Text {
    pub fn push(mut self, s: impl Into<Span>) -> Self {
        self.spans.push(s.into());
        self
    }

    /// Characters, for width estimates until typst
    /// measures text here.
    pub(crate) fn len(&self) -> usize {
        self.spans.iter().map(|s| s.text.chars().count()).sum()
    }

    pub fn svg(&self) -> String {
        let mut o = String::new();
        for s in &self.spans {
            let fill = s
                .fill
                .map(|c| format!(" fill='{c}'"))
                .unwrap_or_default();
            let bold = if s.bold { " font-weight='bold'" } else { "" };
            let _ = write!(
                o,
                "<tspan{fill}{bold}>{}</tspan>",
                xml(&s.text)
            );
        }
        o
    }

    /// The same text as typst markup. Each span becomes a
    /// string literal, so user text needs no escaping
    /// beyond `\` and `"`.
    pub fn typst(&self) -> String {
        let mut o = String::new();
        for s in &self.spans {
            let lit = s
                .text
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            let mut e = format!("\"{lit}\"");
            if s.bold {
                e = format!("strong({e})");
            }
            if let Some(c) = s.fill {
                let hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    c.r.round() as u8,
                    c.g.round() as u8,
                    c.b.round() as u8
                );
                e = format!("text(fill: rgb(\"{hex}\"), {e})");
            }
            let _ = write!(o, "#{e}");
        }
        o
    }
}

impl From<&str> for Span {
    fn from(s: &str) -> Self {
        span(s)
    }
}

impl From<String> for Span {
    fn from(s: String) -> Self {
        span(s)
    }
}

impl From<Span> for Text {
    fn from(s: Span) -> Self {
        Text { spans: vec![s] }
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        span(s).into()
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        span(s).into()
    }
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
