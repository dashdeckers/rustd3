//! The bundled time locales against d3-time-format 4.1.0's tables and its
//! own output (`tests/fixtures/time_locales.json`, written with the tables
//! under `TZ=America/Los_Angeles`).

use std::borrow::Cow;

use rustd3_core::time::{Definition, Ladder, Locale};
use rustd3_testkit::zone;
use serde_json::Value;

use super::at;

/// The fixture, read at compile time so the test needs no working directory.
const PROBES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/time_locales.json"
));

/// d3-time-format 4.1.0's JSON table `name`, read at compile time.
macro_rules! table {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/d3/d3-time-format/locale/",
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

/// The locale d3's `timeFormatLocale` builds from the JSON table `json`,
/// labeling ticks with `ladder`.
pub(super) fn from_json(json: &str, ladder: Ladder) -> Locale {
    let table: Value = serde_json::from_str(json).expect("a JSON table");
    let text = |key: &str| Cow::Owned(table[key].as_str().expect("a string field").to_owned());
    let names = |key: &str| -> Vec<Cow<'static, str>> {
        let names = table[key].as_array().expect("an array field");
        names
            .iter()
            .map(|v| Cow::Owned(v.as_str().expect("a name").to_owned()))
            .collect()
    };
    Locale::new(Definition {
        date_time: text("dateTime"),
        date: text("date"),
        time: text("time"),
        periods: names("periods").try_into().expect("two periods"),
        days: names("days").try_into().expect("seven days"),
        short_days: names("shortDays").try_into().expect("seven days"),
        months: names("months").try_into().expect("twelve months"),
        short_months: names("shortMonths").try_into().expect("twelve months"),
        ladder,
    })
}

#[test]
fn bundled_time_locales_equal_their_json() {
    for (name, constant, json) in BUNDLED {
        let ladder = if name == "en-US" {
            Ladder::D3
        } else {
            Ladder::Iso
        };
        assert_eq!(constant, from_json(json, ladder), "{name}");
    }
}

#[test]
fn time_locales_match_d3() {
    let la = zone("America/Los_Angeles");
    let probes: Value = serde_json::from_str(PROBES).expect("the fixture");
    for (name, constant, _) in BUNDLED {
        let locale = &probes[name];
        assert_eq!(
            locale["constant"].as_str(),
            Some(name.replace('-', "_").to_uppercase().as_str())
        );
        let rows = locale["probes"].as_array().expect("probes for the locale");
        assert_eq!(rows.len(), 28, "{name}");
        for row in rows {
            let spec = row[0].as_str().expect("a specifier");
            let t = at(row[1].as_str().expect("an instant"));
            let d3 = row[2].as_str().expect("d3's output");
            assert_eq!(
                constant.format(spec).format(t, &la),
                d3,
                "{name}: {spec} of {t}"
            );
        }
    }
    assert_eq!(probes.as_object().map(|o| o.len()), Some(BUNDLED.len()));
}

#[test]
fn bundled_time_locales_write_the_sections_rows() {
    let la = zone("America/Los_Angeles");
    let t = at("2026-03-08T07:59:30.123Z");
    let f = |locale: &Locale, spec: &str| locale.format(spec).format(t, &la);
    assert_eq!(f(&Locale::EN_US, "%c"), "3/7/2026, 11:59:30 PM");
    assert_eq!(f(&Locale::EN_US, "%x %X"), "3/7/2026 11:59:30 PM");
    assert_eq!(
        f(&Locale::DE_DE, "%c"),
        "Samstag, der  7. März 2026, 23:59:30"
    );
    assert_eq!(f(&Locale::DE_DE, "%a %d %b"), "Sa 07 Mrz");
    assert_eq!(f(&Locale::FR_FR, "%c"), "samedi  7 mars 2026 à 23:59:30");
    assert_eq!(f(&Locale::JA_JP, "%c"), "2026/03/07 土 23:59:30");
    assert_eq!(f(&Locale::RU_RU, "%B %b"), "марта мар");
    assert_eq!(f(&Locale::PL_PL, "%B %b"), "Marzec Marz.");
    assert_eq!(f(&Locale::ZH_CN, "%x"), "2026年3月7日");
}
