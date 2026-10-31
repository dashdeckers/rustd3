//! The number-format and time-format parsers on random specifiers: random
//! characters of each mini-language mixed with arbitrary Unicode. Parsing
//! never panics, and a parsed number format writes every `f64`, from NaN
//! and ±infinity to subnormals and 1e±300; a time format writes
//! timestamps across jiff's whole range in every bundled locale and in
//! zones with odd offsets.

use rustd3_core::format::{self, precision_fixed, precision_prefix, precision_round};
use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::locale::Locale;
use rustd3_core::time;
use rustd3_testkit::hostile::{self, Rng};

const CASES: usize = 600;

/// Characters of d3-format specifiers, digits weighted.
const NUMBER: &str = " <>=^+-($#00123456789,..~eEfgGrsdx%pbocXn";

/// Characters of d3-time-format specifiers, `%` weighted.
const TIME: &str = "%%%%aAbBcdefgGHIjLmMpqQsSuUVwWxXyYZ-_0 :/T";

/// Widths beyond this pad strings of that many characters, which tests
/// memory, not robustness; such formats are parsed but not written.
const WIDEST: usize = 100_000;

const VALUES: [f64; 16] = [
    0.0,
    -0.0,
    5e-324,
    -5e-324,
    f64::MIN_POSITIVE,
    1e-300,
    -1e-300,
    0.5,
    1e21,
    -1e21,
    1e300,
    -1e300,
    f64::MAX,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
];

static LOCALES: [Locale; 12] = [
    Locale::NEUTRAL,
    Locale::EN_US,
    Locale::DE_DE,
    Locale::FR_FR,
    Locale::ES_ES,
    Locale::IT_IT,
    Locale::PT_BR,
    Locale::NL_NL,
    Locale::PL_PL,
    Locale::RU_RU,
    Locale::JA_JP,
    Locale::ZH_CN,
];

fn writes_every_value(f: &format::Format, rng: &mut Rng) {
    for v in VALUES {
        f.format(v);
    }
    for _ in 0..8 {
        f.format(rng.f64());
    }
    f.format_all(&VALUES);
}

#[test]
fn number_formats_parse_any_text_and_write_every_value() {
    hostile::run("number format", 0xF0_47, CASES, |rng| {
        let spec = rng.string(NUMBER, 12);
        let locale = rng.pick(&LOCALES);
        let width = spec
            .parse::<format::Specifier>()
            .ok()
            .and_then(|s| s.width());
        if width.is_some_and(|w| w > WIDEST) {
            return;
        }
        if let Ok(f) = locale.number().format(&spec) {
            writes_every_value(&f, rng);
        }
        if let Ok(f) = locale.number().format_prefix(&spec, rng.f64()) {
            writes_every_value(&f, rng);
        }
        if let Ok(f) = format::format(&spec) {
            writes_every_value(&f, rng);
        }
        let step = rng.f64();
        precision_fixed(step);
        precision_prefix(step, rng.f64());
        precision_round(step, rng.f64());
    });
}

/// A timestamp anywhere in jiff's range or at its ends.
fn instant(rng: &mut Rng) -> Timestamp {
    let (min, max) = (
        Timestamp::MIN.as_nanosecond(),
        Timestamp::MAX.as_nanosecond(),
    );
    let ns = match rng.below(4) {
        0 => min,
        1 => max,
        2 => i128::from(rng.bits() >> 4) * if rng.chance(0.5) { 1 } else { -1 },
        _ => min + i128::from(rng.bits()) * ((max - min) / i128::from(u64::MAX)),
    };
    Timestamp::from_nanosecond(ns.clamp(min, max)).unwrap_or(Timestamp::UNIX_EPOCH)
}

#[test]
fn time_formats_write_any_instant_in_every_locale_and_zone() {
    let zones = [
        TimeZone::UTC,
        rustd3_testkit::zone("Asia/Kathmandu"),
        rustd3_testkit::zone("Pacific/Chatham"),
        rustd3_testkit::zone("America/St_Johns"),
        TimeZone::fixed(rustd3_core::jiff::tz::Offset::MAX),
        TimeZone::fixed(rustd3_core::jiff::tz::Offset::MIN),
    ];
    hostile::run("time format", 0x7F_47, CASES, |rng| {
        let spec = rng.string(TIME, 16);
        let t = instant(rng);
        let zone = rng.pick(&zones);
        time::format(&spec).format(t, zone);
        for locale in &LOCALES {
            locale.time().format(&spec).format(t, zone);
        }
        // ISO strings keep milliseconds and, through d3's `%Y`, four-digit
        // years: they read back within a millisecond in years 0 to 9999.
        let iso = time::iso_format(t);
        let year = t.to_zoned(TimeZone::UTC).year();
        if (0..=9999).contains(&year) {
            let back = time::iso_parse(&iso).map(|b| t.as_nanosecond() - b.as_nanosecond());
            assert!(
                back.as_ref().is_ok_and(|ns| (0..1_000_000).contains(ns)),
                "{iso}: {back:?}"
            );
        }
        let _ = time::iso_parse(&rng.string("0123456789-:TZ.+", 30));
    });
}
