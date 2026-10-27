//! Number formats under the neutral locale, d3's `EN_US`, and de-DE and
//! fr-FR built with `Locale::new`.

use std::borrow::Cow;

use rustd3_core::format::{
    self, Definition, Locale, Specifier, precision_fixed, precision_prefix, precision_round,
};

/// `value` formatted by `specifier` under `locale`.
fn under(locale: &Locale, specifier: &str, value: f64) -> String {
    locale
        .format(specifier)
        .expect("a valid specifier")
        .format(value)
}

/// `value` formatted by `specifier` under the neutral locale.
fn neutral(specifier: &str, value: f64) -> String {
    format::format(specifier)
        .expect("a valid specifier")
        .format(value)
}

/// A locale with d3's fallbacks but the given separators and currency.
fn locale(decimal: &'static str, thousands: &'static str, currency: [&'static str; 2]) -> Locale {
    Locale::new(Definition {
        decimal: Cow::Borrowed(decimal),
        thousands: Cow::Borrowed(thousands),
        grouping: Cow::Borrowed(&[3]),
        currency: currency.map(Cow::Borrowed),
        ..Definition::default()
    })
}

#[test]
fn en_us_groups_with_commas_and_writes_dollars() {
    let en = Locale::EN_US;
    assert_eq!(under(&en, ",.2f", -1234.5), "−1,234.50");
    assert_eq!(under(&en, "$,.2f", -1234.5), "−$1,234.50");
    assert_eq!(under(&en, ",.1f", 1_234_567.5), "1,234,567.5");
    assert_eq!(under(&en, ",d", 10_000.0), "10,000");
    assert_eq!(under(&en, "~s", 1500.0), "1.5k");
    assert_eq!(under(&en, ".2e", 12345.0), "1.23e+4");
}

#[test]
#[expect(clippy::approx_constant, reason = "the spot check's value, not π")]
fn en_us_pads_signs_and_hides_negative_zero() {
    let en = Locale::EN_US;
    assert_eq!(under(&en, "08.2f", -3.14159), "−0003.14");
    assert_eq!(under(&en, "+.0f", 0.0), "+0");
    assert_eq!(under(&en, "(.2f", -1.5), "(1.50)");
    assert_eq!(under(&en, ".1f", -0.04), "0.0");
}

#[test]
fn en_us_formats_without_a_type_and_in_other_bases() {
    let en = Locale::EN_US;
    assert_eq!(under(&en, "", 0.1 + 0.2), "0.3");
    assert_eq!(under(&en, "#x", 255.0), "0xff");
    assert_eq!(under(&en, "b", 5.0), "101");
}

#[test]
fn exact_ties_round_up_as_in_javascript() {
    let en = Locale::EN_US;
    assert_eq!(under(&en, ".0f", 2.5), "3");
    assert_eq!(under(&en, ".0%", 0.125), "13%");
    assert_eq!(under(&en, ".1e", 0.25), "2.5e-1");
    assert_eq!(under(&en, ".2r", 0.125), "0.13");
    assert_eq!(under(&en, ".2s", 125.0), "130");
}

#[test]
fn the_neutral_default_writes_si_prefixes_and_percent() {
    assert_eq!(neutral(".3s", 0.000_042), "42.0µ");
    assert_eq!(neutral(".1%", 0.123), "12.3%");
}

#[test]
fn an_invalid_specifier_is_an_error_carrying_it() {
    let error = format::format("10..f").map_err(|e| e.input().to_owned());
    assert_eq!(error, Err("10..f".to_owned()));
    assert!(".f".parse::<Specifier>().is_err());
    assert!("foo".parse::<Specifier>().is_err());
}

#[test]
fn a_width_past_v8s_longest_string_is_an_error() {
    assert!("536870888".parse::<Specifier>().is_ok());
    for text in ["536870889", "1000000000000", "99999999999999999999"] {
        let error = format::format(text).map_err(|e| e.input().to_owned());
        assert_eq!(
            error,
            Err(text.to_owned()),
            "d3 throws a RangeError when {text} pads"
        );
    }
}

#[test]
fn the_neutral_default_groups_from_five_integer_digits() {
    assert_eq!(neutral(",.1f", 1_234_567.5), "1\u{202f}234\u{202f}567.5");
    assert_eq!(neutral(",.2f", -12345.678), "−12\u{202f}345.68");
    assert_eq!(neutral(",d", 8000.0), "8000");
    assert_eq!(neutral(",d", 12000.0), "12\u{202f}000");
    assert_eq!(neutral("08,d", 1234.0), "00001234");
}

#[test]
fn the_neutral_default_writes_no_currency() {
    assert_eq!(neutral("$,.2f", -1234.5), "−1234.50");
    assert_eq!(neutral("$,.2f", 1234.5), "1234.50");
}

#[test]
fn de_de_swaps_point_and_comma() {
    let de = locale(",", ".", ["", "\u{a0}€"]);
    assert_eq!(under(&de, ",.1f", 1_234_567.5), "1.234.567,5");
    assert_eq!(under(&de, "$,.2f", 1234.5), "1.234,50\u{a0}€");
}

#[test]
fn fr_fr_groups_with_no_break_spaces() {
    let fr = locale(",", "\u{a0}", ["", "\u{a0}€"]);
    assert_eq!(under(&fr, ",.1f", 1_234_567.5), "1\u{a0}234\u{a0}567,5");
    assert_eq!(under(&fr, ",d", 12000.0), "12\u{a0}000");
}

#[test]
fn format_prefix_fixes_the_si_prefix_of_a_reference_value() {
    let en = Locale::EN_US;
    assert_eq!(en.format_prefix(",.0", 1e3).unwrap().format(1e3), "1k");
    assert_eq!(
        en.format_prefix(".1", 1e6).unwrap().format(1_234_567.0),
        "1.2M"
    );
    assert_eq!(
        format::format_prefix(".1", 1e6).unwrap().to_string(),
        " >-.1f"
    );
}

#[test]
fn precision_suggestions_are_none_where_d3_returns_nan() {
    assert_eq!(precision_fixed(0.01), Some(2));
    assert_eq!(precision_fixed(1.0), Some(0));
    assert_eq!(precision_fixed(0.0), None);
    assert_eq!(precision_prefix(1e5, 1.3e6), Some(1));
    assert_eq!(precision_round(0.01, 1.01), Some(3));
    assert_eq!(precision_round(f64::NAN, 1.0), None);
}

#[test]
fn specifiers_print_normalized() {
    let s: Specifier = ".2s".parse().unwrap();
    assert_eq!(s.to_string(), " >-.2s");
    assert_eq!(Specifier::default().to_string(), " >-");
    assert_eq!(s.with_precision(3).with_trim(true).to_string(), " >-.3~s");
    assert_eq!(format::format("d").unwrap().to_string(), " >-d");
}

#[test]
fn numerals_replace_every_digit() {
    const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
    let ar = Locale::new(Definition {
        decimal: Cow::Borrowed("٫"),
        thousands: Cow::Borrowed("٬"),
        grouping: Cow::Borrowed(&[3]),
        numerals: Some(DIGITS.map(Cow::Borrowed)),
        ..Definition::default()
    });
    // d3-format's arabicLocale test, ar-001.
    assert_eq!(under(&ar, "$,.2f", -1234.56), "−١٬٢٣٤٫٥٦");
}
