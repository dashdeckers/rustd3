//! [`Key`]: a label's identity in the text cache.

use std::sync::Arc;

use peniko::Color;

use crate::text::{Style, Text, anchor, channels};

/// A label as typst sets it: its text with size, weight, color and line
/// alignment, as one typst snippet. Equal snippets shape alike, so the
/// text cache keys on it.
///
/// ```
/// use rustd3_core::color::Rgb;
/// use rustd3_figure::text::{Key, Style, Text, anchor};
///
/// let style = Style {
///     size: 11.0,
///     bold: false,
///     fill: Rgb::hex(0x262626),
///     align: anchor::Horizontal::Middle,
/// };
/// assert_eq!(
///     Key::new(&Text::from("x"), &style).as_str(),
///     r#"#align(center, box(text(size: 11.0000pt, weight: "regular", fill: rgb(38, 38, 38, 255))[#"x"]))"#
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    snippet: Arc<str>,
    /// The style's fill, which also paints text typst fills with a
    /// gradient or tiling.
    fill: [u8; 4],
}

impl Key {
    /// `text` set in `style`: `#align(…, box(text(size: …pt, weight: …,
    /// fill: rgb(r, g, b, a))[…]))`, the size in px to four places. The
    /// box keeps typst's baseline of the label's first line in the frame.
    pub fn new(text: &Text, style: &Style) -> Key {
        let align = match style.align {
            anchor::Horizontal::Start => "start",
            anchor::Horizontal::Middle => "center",
            anchor::Horizontal::End => "end",
        };
        let weight = if style.bold { "bold" } else { "regular" };
        let fill = channels(style.fill);
        let [r, g, b, a] = fill;
        let snippet = format!(
            "#align({align}, box(text(size: {:.4}pt, weight: \"{weight}\", fill: rgb({r}, {g}, {b}, {a}))[{}]))",
            style.size,
            text.typst()
        );
        Key {
            snippet: snippet.into(),
            fill,
        }
    }

    /// The typst snippet.
    pub fn as_str(&self) -> &str {
        &self.snippet
    }

    /// The label's own markup, as [`Text::typst`] wrote it: the snippet
    /// inside its style.
    #[cfg(feature = "fonts")]
    pub(crate) fn markup(&self) -> &str {
        // The snippet is `#align(…, box(text(…, fill: rgb(…))[markup]))`:
        // the first ")[" closes the style.
        let start = self.snippet.find(")[").map_or(0, |i| i + 2);
        let end = self.snippet.len().saturating_sub(3).max(start);
        &self.snippet[start..end]
    }

    /// The style's fill.
    pub(crate) fn fill(&self) -> Color {
        let [r, g, b, a] = self.fill;
        Color::from_rgba8(r, g, b, a)
    }
}
