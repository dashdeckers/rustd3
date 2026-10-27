//! The figure locale and the batch rule: Part I's Formats, time and color
//! table under `NEUTRAL` and `EN_US`, and axes whose labels group all or
//! nothing.

use rustd3_core::format::InvalidFormat;
use rustd3_core::locale::Locale;
use rustd3_core::scale;

/// The labels of a linear scale over `domain` at `count`, as one batch.
fn tick_labels(locale: &Locale, domain: [f64; 2], count: usize) -> Vec<String> {
    let x = scale::linear().domain(domain);
    x.tick_format_in(locale.number(), count, None)
        .expect("the default specifier parses")
        .format_all(&x.ticks(count))
}

/// `value` formatted by `specifier` under `locale`'s number half.
fn single(locale: &Locale, specifier: &str, value: f64) -> Result<String, InvalidFormat> {
    Ok(locale.number().format(specifier)?.format(value))
}

#[test]
fn a_number_locale_makes_a_figure_locale() {
    use std::borrow::Cow;

    use rustd3_core::format::{self, Definition};

    // The number half only: the time half stays neutral (D11.2), where
    // EN_US pairs d3's.
    assert_eq!(
        Locale::from(format::Locale::EN_US).number(),
        Locale::EN_US.number()
    );
    assert_eq!(Locale::from(format::Locale::NEUTRAL), Locale::default());
    let de = Locale::from(format::Locale::new(Definition {
        decimal: Cow::Borrowed(","),
        thousands: Cow::Borrowed("."),
        grouping: Cow::Borrowed(&[3]),
        ..Definition::default()
    }));
    assert_eq!(
        tick_labels(&de, [0.0, 12000.0], 5),
        ["0", "2.000", "4.000", "6.000", "8.000", "10.000", "12.000"]
    );
    assert_eq!(tick_labels(&de, [0.0, 1.0], 2), ["0,0", "0,5", "1,0"]);
}

#[test]
fn the_default_locale_is_neutral() {
    assert_eq!(Locale::default(), Locale::NEUTRAL);
    assert_eq!(
        Locale::NEUTRAL.number(),
        &rustd3_core::format::Locale::NEUTRAL
    );
    assert_eq!(Locale::EN_US.number(), &rustd3_core::format::Locale::EN_US);
}

#[test]
fn a_grouped_fixed_value_follows_the_table() -> Result<(), InvalidFormat> {
    assert_eq!(
        single(&Locale::NEUTRAL, ",.1f", 1234567.5)?,
        "1\u{202f}234\u{202f}567.5"
    );
    assert_eq!(single(&Locale::EN_US, ",.1f", 1234567.5)?, "1,234,567.5");
    Ok(())
}

#[test]
fn currency_follows_the_table() -> Result<(), InvalidFormat> {
    assert_eq!(single(&Locale::NEUTRAL, "$,.2f", 1234.5)?, "1234.50");
    assert_eq!(single(&Locale::EN_US, "$,.2f", 1234.5)?, "$1,234.50");
    Ok(())
}

#[test]
fn ticks_to_eight_thousand_are_not_grouped_under_neutral() {
    assert_eq!(
        tick_labels(&Locale::NEUTRAL, [0.0, 8000.0], 5),
        ["0", "2000", "4000", "6000", "8000"]
    );
    assert_eq!(
        tick_labels(&Locale::EN_US, [0.0, 8000.0], 5),
        ["0", "2,000", "4,000", "6,000", "8,000"]
    );
}

#[test]
fn ticks_to_twelve_thousand_are_all_grouped_under_neutral() {
    assert_eq!(
        tick_labels(&Locale::NEUTRAL, [0.0, 12000.0], 5),
        [
            "0",
            "2\u{202f}000",
            "4\u{202f}000",
            "6\u{202f}000",
            "8\u{202f}000",
            "10\u{202f}000",
            "12\u{202f}000"
        ]
    );
    assert_eq!(
        tick_labels(&Locale::EN_US, [0.0, 12000.0], 5),
        ["0", "2,000", "4,000", "6,000", "8,000", "10,000", "12,000"]
    );
}

#[test]
fn single_values_keep_the_per_value_rule() -> Result<(), InvalidFormat> {
    let f = Locale::NEUTRAL.number().format(",")?;
    assert_eq!(f.format(8000.0), "8000");
    assert_eq!(f.format(12000.0), "12\u{202f}000");
    assert_eq!(f.format_all(&[8000.0]), ["8000"]);
    assert_eq!(f.format_all(&[12000.0]), ["12\u{202f}000"]);
    Ok(())
}

#[test]
fn the_batch_counts_digits_as_printed() -> Result<(), InvalidFormat> {
    // 9999.6 prints as 10000 at no decimals; SI scaling leaves 12.
    let fixed = Locale::NEUTRAL.number().format(",.0f")?;
    assert_eq!(
        fixed.format_all(&[5000.0, 9999.6]),
        ["5\u{202f}000", "10\u{202f}000"]
    );
    let si = Locale::NEUTRAL.number().format(",s")?;
    assert_eq!(si.format_all(&[2000.0, 12000.0]), ["2.00000k", "12.0000k"]);
    Ok(())
}

#[test]
fn fractions_and_formats_without_a_comma_are_never_grouped() -> Result<(), InvalidFormat> {
    let fraction = Locale::NEUTRAL.number().format(",.5f")?;
    assert_eq!(
        fraction.format_all(&[0.12345, 12345.0]),
        ["0.12345", "12\u{202f}345.00000"]
    );
    let plain = Locale::NEUTRAL.number().format(".0f")?;
    assert_eq!(plain.format_all(&[8000.0, 12000.0]), ["8000", "12000"]);
    Ok(())
}

#[test]
fn an_empty_batch_has_no_labels() -> Result<(), InvalidFormat> {
    assert!(
        Locale::NEUTRAL
            .number()
            .format(",")?
            .format_all(&[])
            .is_empty()
    );
    Ok(())
}

#[test]
fn bundled_tables_write_the_de_de_and_fr_fr_rows_of_part_i() -> Result<(), InvalidFormat> {
    use rustd3_core::format;

    let de = Locale::from(format::Locale::DE_DE);
    assert_eq!(single(&de, ",.1f", 1234567.5)?, "1.234.567,5");
    assert_eq!(single(&de, "$,.2f", 1234.5)?, "1.234,50\u{a0}€");
    assert_eq!(
        tick_labels(&de, [0.0, 8000.0], 5)[1..],
        ["2.000", "4.000", "6.000", "8.000"]
    );
    assert_eq!(tick_labels(&de, [0.0, 12000.0], 5)[6], "12.000");
    let fr = Locale::from(format::Locale::FR_FR);
    assert_eq!(single(&fr, ",.1f", 1234567.5)?, "1\u{a0}234\u{a0}567,5");
    assert_eq!(single(&fr, "$,.2f", 1234.5)?, "1\u{a0}234,50\u{a0}€");
    assert_eq!(tick_labels(&fr, [0.0, 8000.0], 5)[1], "2\u{a0}000");
    assert_eq!(tick_labels(&fr, [0.0, 12000.0], 5)[6], "12\u{a0}000");
    Ok(())
}

#[test]
fn the_constants_pair_the_number_and_time_tables_of_their_language() {
    use rustd3_core::{format, time};

    let pairs = [
        (
            Locale::NEUTRAL,
            format::Locale::NEUTRAL,
            time::Locale::NEUTRAL,
        ),
        (Locale::EN_US, format::Locale::EN_US, time::Locale::EN_US),
        (Locale::DE_DE, format::Locale::DE_DE, time::Locale::DE_DE),
        (Locale::FR_FR, format::Locale::FR_FR, time::Locale::FR_FR),
        (Locale::ES_ES, format::Locale::ES_ES, time::Locale::ES_ES),
        (Locale::IT_IT, format::Locale::IT_IT, time::Locale::IT_IT),
        (Locale::PT_BR, format::Locale::PT_BR, time::Locale::PT_BR),
        (Locale::NL_NL, format::Locale::NL_NL, time::Locale::NL_NL),
        (Locale::PL_PL, format::Locale::PL_PL, time::Locale::PL_PL),
        (Locale::RU_RU, format::Locale::RU_RU, time::Locale::RU_RU),
        (Locale::JA_JP, format::Locale::JA_JP, time::Locale::JA_JP),
        (Locale::ZH_CN, format::Locale::ZH_CN, time::Locale::ZH_CN),
    ];
    for (locale, number, time) in pairs {
        assert_eq!((locale.number(), locale.time()), (&number, &time));
    }
    assert_eq!(Locale::default().time(), &time::Locale::default());
    // A runtime number table keeps the neutral time half (D11.2).
    let runtime = Locale::from(format::Locale::new(format::Definition::default()));
    assert_eq!(runtime.time(), &time::Locale::NEUTRAL);
}
