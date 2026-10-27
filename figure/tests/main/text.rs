//! Figure text: markup from spans and raw typst.

use rustd3_core::color::Rgb;
use rustd3_figure::text::{self, Text, span};

#[test]
fn a_plain_label_is_one_string_literal() {
    assert_eq!(Text::from("0123456789").typst(), r#"#"0123456789""#);
    assert_eq!(Text::from(String::from("x")).typst(), r#"#"x""#);
}

#[test]
fn markup_characters_in_spans_stay_literal() {
    assert_eq!(Text::from("a $ # _ b").typst(), r##"#"a $ # _ b""##);
}

#[test]
fn backslashes_quotes_and_control_characters_are_escaped() {
    let text = Text::from("a\\b \"q\"\tc\r\nd");
    assert_eq!(text.typst(), r#"#"a\\b \"q\"\tc\nd""#);
}

#[test]
fn a_filled_span_wraps_its_literal_in_text() {
    let title = Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)));
    assert_eq!(
        title.typst(),
        r##"#"Wheel: "#text(fill: rgb("#c44e52"), "RL")"##
    );
}

#[test]
fn a_translucent_fill_is_written_as_integer_channels() {
    let text = Text::from(span("x").fill(Rgb::new(10.4, 20.6, 300.0).with_opacity(0.5)));
    assert_eq!(text.typst(), r#"#text(fill: rgb(10, 21, 255, 128), "x")"#);
}

#[test]
fn a_bold_span_is_strong_inside_its_fill() {
    assert_eq!(Text::from(span("b").bold()).typst(), r#"#strong("b")"#);
    assert_eq!(
        Text::from(span("b").bold().fill(Rgb::hex(0x4c72b0))).typst(),
        r##"#text(fill: rgb("#4c72b0"), strong("b"))"##
    );
}

#[test]
fn raw_markup_is_inserted_verbatim() {
    assert_eq!(
        text::typst("RMS [$m slash s^2$]").typst(),
        "RMS [$m slash s^2$]"
    );
}

#[test]
fn a_span_before_markup_ends_its_expression() {
    let text = Text::from("a").push(text::typst("[b]")).push("c");
    assert_eq!(text.typst(), r##"#"a";[b]#"c""##);
}

#[test]
fn a_text_without_characters_is_empty() {
    assert!(Text::default().is_empty());
    assert!(Text::from("").push(text::typst("")).is_empty());
    assert!(!Text::from("").push(" ").is_empty());
}
