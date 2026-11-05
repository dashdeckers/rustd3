//! One typst document per batch of labels, one page per label, and the
//! fallback to one document per label.

use std::sync::Arc;

use typst_layout::PagedDocument;

use crate::text::world::World;
use crate::text::{Error, Fonts, Key, Shaped, convert, literal};

/// The labels of `keys`, shaped as pages of one document. When that
/// document fails or its page count is not the label count (raw markup
/// may break pages), each label is compiled alone, so one bad label
/// costs only itself. Every typst compilation adds one to `compiles`.
pub(crate) fn shape(
    fonts: &Arc<Fonts>,
    keys: &[&Key],
    compiles: &mut u64,
) -> Vec<Result<Shaped, Error>> {
    match (compile(fonts, keys, compiles), keys) {
        (Ok(document), _) if document.pages().len() == keys.len() => document
            .pages()
            .iter()
            .zip(keys)
            .map(|(page, key)| Ok(convert::page(fonts, &page.frame, key.fill())))
            .collect(),
        (Ok(document), [key]) => vec![Err(Error::Pages {
            markup: key.as_str().into(),
            pages: document.pages().len(),
        })],
        (Err(message), [key]) => vec![Err(Error::Markup {
            markup: key.as_str().into(),
            message,
        })],
        (Ok(_) | Err(_), _) => keys
            .iter()
            .flat_map(|key| shape(fonts, &[key], compiles))
            .collect(),
    }
}

/// The document of `keys`, or typst's error messages joined by "; ".
fn compile(fonts: &Arc<Fonts>, keys: &[&Key], compiles: &mut u64) -> Result<PagedDocument, String> {
    *compiles += 1;
    let world = World::new(fonts.clone(), source(fonts, keys));
    typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|errors| {
            let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
            messages.join("; ")
        })
}

/// The typst source of `keys`: tight pages without margins, text edges at
/// the ink, every family in fallback order, and each label's snippet on
/// its own page.
fn source(fonts: &Fonts, keys: &[&Key]) -> String {
    let families: Vec<String> = fonts.families().iter().map(|f| literal(f)).collect();
    let mut source = format!(
        "#set page(width: auto, height: auto, margin: 0pt)\n\
         #set text(font: ({}), top-edge: \"bounds\", bottom-edge: \"bounds\")\n",
        families.join(", ")
    );
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            source.push_str("#pagebreak()\n");
        }
        source.push_str(key.as_str());
        source.push('\n');
    }
    source
}

#[cfg(all(test, feature = "fonts"))]
mod tests {
    use rustd3_core::color::Rgb;

    use super::*;
    use crate::text::{Style, Text, anchor, span};

    #[test]
    fn no_spot_check_label_draws_a_typst_warning() {
        let style = Style {
            size: 11.0,
            bold: false,
            fill: Rgb::hex(0),
            align: anchor::Horizontal::Middle,
        };
        let labels: [Text; 8] = [
            "0123456789".into(),
            "line one\nline two".into(),
            Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52))),
            "a $ # _ b".into(),
            "vibration RMS [m/s²]".into(),
            crate::text::typst("RMS [$m slash s^2$]"),
            "漢字".into(),
            Text::from(span("bold").bold()),
        ];
        let keys: Vec<Key> = labels.iter().map(|t| Key::new(t, &style)).collect();
        let keys: Vec<&Key> = keys.iter().collect();
        let fonts = Fonts::bundled();
        let world = World::new(fonts.clone(), source(&fonts, &keys));
        let warned = typst::compile::<PagedDocument>(&world);
        let messages: Vec<&str> = warned.warnings.iter().map(|w| w.message.as_str()).collect();
        assert!(messages.is_empty(), "{messages:?}");
        assert_eq!(warned.output.map(|d| d.pages().len()).ok(), Some(8));
    }
}
