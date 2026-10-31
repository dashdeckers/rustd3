//! Number formats under the neutral locale, d3's `EN_US`, de-DE and fr-FR
//! built with `Locale::new`, and the bundled locales against d3-format
//! 3.1.2's tables and its own output (`dev/fixtures/format_locales.json`,
//! written by `dev/harness/gen/format_locales.mjs`).

use std::borrow::Cow;

use rustd3_core::format::{
    self, Definition, Locale, Specifier, literal, precision_fixed, precision_prefix,
    precision_round,
};
use serde_json::Value;

/// The fixture, read at compile time so the test needs no working directory.
const PROBES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dev/fixtures/format_locales.json"
));

/// d3-format 3.1.2's JSON table `name`, read at compile time.
macro_rules! table {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dev/harness/d3/d3-format/locale/",
            $name,
            ".json"
        ))
    };
}

/// Each bundled constant, with d3's name for it and the table it is
/// generated from.
const BUNDLED: [(&str, Locale, &str); 11] = [
    ("de-DE", Locale::DE_DE, table!("de-DE")),
    ("en-US", Locale::EN_US, table!("en-US")),
    ("es-ES", Locale::ES_ES, table!("es-ES")),
    ("fr-FR", Locale::FR_FR, table!("fr-FR")),
    ("it-IT", Locale::IT_IT, table!("it-IT")),
    ("ja-JP", Locale::JA_JP, table!("ja-JP")),
    ("nl-NL", Locale::NL_NL, table!("nl-NL")),
    ("pl-PL", Locale::PL_PL, table!("pl-PL")),
    ("pt-BR", Locale::PT_BR, table!("pt-BR")),
    ("ru-RU", Locale::RU_RU, table!("ru-RU")),
    ("zh-CN", Locale::ZH_CN, table!("zh-CN")),
];

/// Tables that load at run time: Indian grouping and Arabic-Indic digits.
const RUNTIME: [(&str, &str); 2] = [("ar-001", table!("ar-001")), ("en-IN", table!("en-IN"))];

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

/// The locale d3's `formatLocale` builds from the JSON table `json`: its
/// fields, `Definition`'s fallbacks for the absent ones, and grouping only
/// with a separator (locale.js 13–20).
fn from_json(json: &str) -> Locale {
    let table: Value = serde_json::from_str(json).expect("a JSON table");
    let text = |key: &str| {
        let value = table.get(key)?.as_str().expect("a string field");
        Some(Cow::Owned(value.to_owned()))
    };
    let texts = |key: &str| -> Option<Vec<Cow<'static, str>>> {
        let values = table.get(key)?.as_array().expect("an array field");
        Some(
            values
                .iter()
                .map(|v| Cow::Owned(v.as_str().expect("a string").to_owned()))
                .collect(),
        )
    };
    let grouping = match (table.get("grouping"), table.get("thousands")) {
        (Some(sizes), Some(_)) => sizes
            .as_array()
            .expect("group sizes")
            .iter()
            .map(|n| usize::try_from(n.as_u64().expect("a group size")).expect("a size that fits"))
            .collect(),
        _ => Vec::new(),
    };
    let Definition {
        decimal,
        thousands,
        currency,
        percent,
        minus,
        nan,
        ..
    } = Definition::default();
    Locale::new(Definition {
        decimal: text("decimal").unwrap_or(decimal),
        thousands: text("thousands").unwrap_or(thousands),
        grouping: Cow::Owned(grouping),
        currency: texts("currency")
            .map_or(currency, |c| c.try_into().expect("two currency strings")),
        numerals: texts("numerals").map(|n| n.try_into().expect("ten numerals")),
        percent: text("percent").unwrap_or(percent),
        minus: text("minus").unwrap_or(minus),
        nan: text("nan").unwrap_or(nan),
        group_from: 1,
    })
}

/// Asserts that `locale` writes each of the fixture's probes for `name`
/// as d3 does.
fn matches_d3(probes: &Value, name: &str, locale: &Locale) {
    let rows = probes[name]["probes"]
        .as_array()
        .expect("probes for the locale");
    assert_eq!(rows.len(), 6, "{name}");
    for row in rows {
        let specifier = row[0].as_str().expect("a specifier");
        let value = row[1].as_f64().expect("a value");
        let d3 = row[2].as_str().expect("d3's output");
        assert_eq!(
            under(locale, specifier, value),
            d3,
            "{name}: {specifier} of {value}"
        );
    }
}

/// Every double-quoted string literal in `source`, decoded as JSON decodes
/// it: d3-format's tests escape only with `\uXXXX` and `\"`.
fn strings(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find('"') {
        let bytes = &rest.as_bytes()[start + 1..];
        let mut end = 0;
        while end < bytes.len() && bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        let literal = &rest[start..(start + end + 2).min(rest.len())];
        found.extend(serde_json::from_str::<String>(literal).ok());
        rest = &rest[(start + end + 2).min(rest.len())..];
    }
    found
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
    let error = format::format("10..f").map_err(|e| e.get_input().to_owned());
    assert_eq!(error, Err("10..f".to_owned()));
    assert!(".f".parse::<Specifier>().is_err());
    assert!("foo".parse::<Specifier>().is_err());
}

#[test]
fn a_width_past_v8s_longest_string_is_an_error() {
    assert!("536870888".parse::<Specifier>().is_ok());
    for text in ["536870889", "1000000000000", "99999999999999999999"] {
        let error = format::format(text).map_err(|e| e.get_input().to_owned());
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

#[test]
fn bundled_locales_equal_their_json() {
    for (name, constant, json) in BUNDLED {
        assert_eq!(constant, from_json(json), "{name}");
    }
}

#[test]
fn bundled_number_locales_match_d3() {
    let probes: Value = serde_json::from_str(PROBES).expect("the fixture");
    for (name, constant, _) in BUNDLED {
        let named = probes[name]["constant"].as_str().expect("a bundled locale");
        assert_eq!(named, name.replace('-', "_").to_uppercase());
        matches_d3(&probes, name, &constant);
    }
    let bundled = probes.as_object().expect("locales by name");
    let bundled = bundled
        .values()
        .filter(|v| v["constant"].is_string())
        .count();
    assert_eq!(bundled, BUNDLED.len());
}

#[test]
fn runtime_number_locales_match_d3() {
    let probes: Value = serde_json::from_str(PROBES).expect("the fixture");
    for (name, json) in RUNTIME {
        assert!(probes[name]["constant"].is_null(), "{name} is not bundled");
        matches_d3(&probes, name, &from_json(json));
    }
    let in_ = from_json(table!("en-IN"));
    assert_eq!(under(&in_, ",", 10_000_000.454_3), "1,00,00,000.4543");
    assert_eq!(under(&in_, ",.1f", 1_234_567.5), "12,34,567.5");
}

#[test]
fn bundled_locales_write_the_rows_part_i_shows() {
    assert_eq!(under(&Locale::DE_DE, ",.1f", 1_234_567.5), "1.234.567,5");
    assert_eq!(under(&Locale::DE_DE, "$,.2f", 1234.5), "1.234,50\u{a0}€");
    assert_eq!(under(&Locale::FR_FR, ".1%", 0.123), "12,3\u{202f}%");
    assert_eq!(under(&Locale::EN_US, "$,.2f", -1234.5), "−$1,234.50");
    assert_eq!(
        under(&Locale::default(), ",.1f", 1_234_567.5),
        "1\u{202f}234\u{202f}567.5"
    );
}

#[test]
fn literal_check_agrees_with_parser() {
    let agrees = |text: &str| {
        let parsed = text.parse::<Specifier>().is_ok();
        assert_eq!(literal::is_valid(text), parsed, "{text:?}");
        parsed
    };
    let tests = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../dev/harness/d3/d3-format/test"
    );
    let mut d3 = Vec::new();
    for file in std::fs::read_dir(tests).expect("d3-format's tests") {
        let source = std::fs::read_to_string(file.expect("a test file").path());
        d3.extend(strings(&source.expect("a test file as UTF-8")));
    }
    let valid = d3.iter().filter(|text| agrees(text)).count();
    assert!(
        d3.len() > 1000 && valid > 300,
        "{} strings, {valid} valid",
        d3.len()
    );
    for text in ["$,.2f", "foo", ".-2s", ".f", "\u{a0}€"] {
        assert!(d3.iter().any(|s| s == text), "{text:?} is in d3's tests");
    }
    let edges = [
        "😀>10",
        "é^10",
        "€>8",
        "\n>10",
        "\r<1",
        "\u{2028}>1",
        "\u{2029}^1",
        "\u{85}>1",
        "536870888",
        "536870889",
        "99999999999999999999",
        "\u{212a}",
        "\u{17f}",
        ".0",
        "~",
    ];
    for text in edges {
        agrees(text);
    }
    const ALPHABET: [char; 23] = [
        '0', '1', ',', '.', '~', '$', '#', '+', '-', '(', ' ', '<', '>', '=', '^', 'e', 'f', 'g',
        'r', 's', '%', 'x', 'z',
    ];
    let mut words = vec![String::new()];
    let mut all = words.clone();
    for _ in 0..4 {
        words = words
            .iter()
            .flat_map(|w| ALPHABET.iter().map(move |c| format!("{w}{c}")))
            .collect();
        all.extend(words.iter().cloned());
    }
    assert_eq!(
        all.len(),
        1 + 23 + 23 * 23 + 23 * 23 * 23 + 23 * 23 * 23 * 23
    );
    let valid = all.iter().filter(|text| agrees(text)).count();
    assert!(valid > 1000, "{valid} of {} valid", all.len());
}

#[test]
fn degenerate_values_under_the_tables_match_d3() {
    // d3-format 3.1.2's formatLocale over the same JSON tables, in Node,
    // but for one row.
    let ar = from_json(table!("ar-001"));
    let in_ = from_json(table!("en-IN"));
    let rows: [(&Locale, &str, f64, &str); 17] = [
        (&Locale::DE_DE, ",.2f", f64::NAN, "NaN"),
        (&Locale::DE_DE, ",.2f", f64::INFINITY, "Infinity"),
        (
            &Locale::DE_DE,
            "$,.2f",
            f64::NEG_INFINITY,
            "−Infinity\u{a0}€",
        ),
        (&Locale::DE_DE, ",.1f", -0.0, "0,0"),
        (&Locale::DE_DE, "+,.1f", -0.0, "−0,0"),
        (&Locale::DE_DE, "", 0.1 + 0.2, "0,3"),
        (&Locale::DE_DE, ".3~e", 12345.0, "1,235e+4"),
        (&Locale::DE_DE, "#x", 255.0, "0xff"),
        (&Locale::DE_DE, "c", 3.5, "3.5"),
        (&Locale::FR_FR, ",d", -0.4, "0"),
        (
            &Locale::FR_FR,
            "020,.1f",
            -1234.5,
            "−00\u{a0}000\u{a0}000\u{a0}001\u{a0}234,5",
        ),
        (&Locale::RU_RU, ".2s", 1e-30, "0,000001y"),
        // d3's toFixed switches to "1e+21円" here; the port writes no
        // exponent notation that JavaScript's printing adds.
        (
            &Locale::JA_JP,
            "$,.0f",
            1e21,
            "1,000,000,000,000,000,000,000円",
        ),
        (&ar, ",.2f", f64::NAN, "NaN"),
        (&ar, "$,.2f", f64::NEG_INFINITY, "−Infinity"),
        (&ar, "020,.1f", -1234.5, "−٠٠٬٠٠٠٬٠٠٠٬٠٠١٬٢٣٤٫٥"),
        (&in_, "015,d", 1_234_567.0, "00,00,12,34,567"),
    ];
    for (locale, specifier, value, d3) in rows {
        assert_eq!(
            under(locale, specifier, value),
            d3,
            "{specifier} of {value}"
        );
    }
}

/// `fmt!`'s doc says it writes no currency, as the neutral locale has
/// none, and shows the route that writes one; the route does.
#[test]
fn fmt_docs_say_the_neutral_locale_has_no_currency_and_show_one_that_does() {
    let source = include_str!("../../src/format/literal.rs");
    let end = source.find("macro_rules! fmt").expect("the macro");
    let mut lines: Vec<&str> = source[..end]
        .lines()
        .rev()
        .take_while(|l| l.starts_with("///") || l.starts_with("#["))
        .collect();
    lines.reverse();
    let words: Vec<&str> = lines
        .iter()
        .flat_map(|l| l.trim_start_matches("///").split_whitespace())
        .collect();
    let doc = words.join(" ");
    for words in [
        "`fmt!` formats under [`Locale::NEUTRAL`](crate::format::Locale::NEUTRAL), which has no currency symbol",
        "`Locale::EN_US.format(\"$,.2f\")` writes \"$1,234.50\"",
    ] {
        assert!(doc.contains(words), "{words:?}");
    }
    let money = Locale::EN_US.format("$,.2f").expect("a specifier");
    assert_eq!(money.format(1234.5), "$1,234.50");
    assert_eq!(rustd3_core::fmt!("$,.2f").format(1234.5), "1234.50");
}

#[test]
fn fmt_takes_the_empty_specifier_and_the_neutral_default() {
    assert_eq!(rustd3_core::fmt!("").format(0.1 + 0.2), "0.3");
    assert_eq!(rustd3_core::fmt!(",d").format(12000.0), "12\u{202f}000");
    assert_eq!(rustd3_core::fmt!(",d").get_locale(), &Locale::NEUTRAL);
    assert_eq!(
        rustd3_core::fmt!("$,.2f").format(-1234.5),
        format::format("$,.2f").unwrap().format(-1234.5)
    );
    assert!(format::format("10..f").is_err());
    assert!(Locale::DE_DE.format("10..f").is_err());
}
