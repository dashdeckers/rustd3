//! d3's tests of d3-format, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::format;

use super::support::assert_d3_eq;

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-001 locale
#[test]
fn formatlocale_can_format_numbers_using_ar_001_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(-1234.56),
        "−١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:10:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-AE locale
#[test]
fn formatlocale_can_format_numbers_using_ar_ae_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " د.إ.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ د.إ."
    ); // d3-format/test/arabicLocale-test.js:14:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-BH locale
#[test]
fn formatlocale_can_format_numbers_using_ar_bh_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " د.ب.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ د.ب."
    ); // d3-format/test/arabicLocale-test.js:18:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-DJ locale
#[test]
fn formatlocale_can_format_numbers_using_ar_dj_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["‏Fdj ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "‏Fdj ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:22:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-DZ locale
#[test]
fn formatlocale_can_format_numbers_using_ar_dz_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ",".into(),
            thousands: ".".into(),
            grouping: vec![3].into(),
            currency: ["د.ج. ".into(), "".into()],
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "د.ج. 1.234,56"
    ); // d3-format/test/arabicLocale-test.js:26:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-EG locale
#[test]
fn formatlocale_can_format_numbers_using_ar_eg_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ج.م.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ج.م."
    ); // d3-format/test/arabicLocale-test.js:30:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-EH locale
#[test]
fn formatlocale_can_format_numbers_using_ar_eh_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            thousands: ",".into(),
            grouping: vec![3].into(),
            currency: ["د.م. ".into(), "".into()],
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "د.م. 1,234.56"
    ); // d3-format/test/arabicLocale-test.js:34:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-ER locale
#[test]
fn formatlocale_can_format_numbers_using_ar_er_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["Nfk ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "Nfk ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:38:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-IL locale
#[test]
fn formatlocale_can_format_numbers_using_ar_il_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["₪ ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "₪ ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:42:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-IQ locale
#[test]
fn formatlocale_can_format_numbers_using_ar_iq_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " د.ع.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ د.ع."
    ); // d3-format/test/arabicLocale-test.js:46:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-JO locale
#[test]
fn formatlocale_can_format_numbers_using_ar_jo_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " د.أ.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ د.أ."
    ); // d3-format/test/arabicLocale-test.js:50:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-KM locale
#[test]
fn formatlocale_can_format_numbers_using_ar_km_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ف.ج.ق.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ف.ج.ق."
    ); // d3-format/test/arabicLocale-test.js:54:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-KW locale
#[test]
fn formatlocale_can_format_numbers_using_ar_kw_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " د.ك.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ د.ك."
    ); // d3-format/test/arabicLocale-test.js:58:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-LB locale
#[test]
fn formatlocale_can_format_numbers_using_ar_lb_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ل.ل.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ل.ل."
    ); // d3-format/test/arabicLocale-test.js:62:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-MA locale
#[test]
fn formatlocale_can_format_numbers_using_ar_ma_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ",".into(),
            thousands: ".".into(),
            grouping: vec![3].into(),
            currency: ["د.م. ".into(), "".into()],
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "د.م. 1.234,56"
    ); // d3-format/test/arabicLocale-test.js:66:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-MR locale
#[test]
fn formatlocale_can_format_numbers_using_ar_mr_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " أ.م.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ أ.م."
    ); // d3-format/test/arabicLocale-test.js:70:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-OM locale
#[test]
fn formatlocale_can_format_numbers_using_ar_om_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ر.ع.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ر.ع."
    ); // d3-format/test/arabicLocale-test.js:74:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-PS locale
#[test]
fn formatlocale_can_format_numbers_using_ar_ps_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["₪ ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "₪ ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:78:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-QA locale
#[test]
fn formatlocale_can_format_numbers_using_ar_qa_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ر.ق.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ر.ق."
    ); // d3-format/test/arabicLocale-test.js:82:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-SA locale
#[test]
fn formatlocale_can_format_numbers_using_ar_sa_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ر.س.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ر.س."
    ); // d3-format/test/arabicLocale-test.js:86:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-SD locale
#[test]
fn formatlocale_can_format_numbers_using_ar_sd_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ج.س.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ج.س."
    ); // d3-format/test/arabicLocale-test.js:90:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-SO locale
#[test]
fn formatlocale_can_format_numbers_using_ar_so_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["‏S ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "‏S ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:94:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-SS locale
#[test]
fn formatlocale_can_format_numbers_using_ar_ss_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["£ ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "£ ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:98:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-SY locale
#[test]
fn formatlocale_can_format_numbers_using_ar_sy_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ل.س.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ل.س."
    ); // d3-format/test/arabicLocale-test.js:102:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-TD locale
#[test]
fn formatlocale_can_format_numbers_using_ar_td_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["‏FCFA ".into(), "".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "‏FCFA ١٬٢٣٤٫٥٦"
    ); // d3-format/test/arabicLocale-test.js:106:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-TN locale
#[test]
fn formatlocale_can_format_numbers_using_ar_tn_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ",".into(),
            thousands: ".".into(),
            grouping: vec![3].into(),
            currency: ["د.ت. ".into(), "".into()],
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "د.ت. 1.234,56"
    ); // d3-format/test/arabicLocale-test.js:110:3
}

// d3-format/test/arabicLocale-test.js: formatLocale() can format numbers using ar-YE locale
#[test]
fn formatlocale_can_format_numbers_using_ar_ye_locale() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "٫".into(),
            thousands: "٬".into(),
            grouping: vec![3].into(),
            currency: ["".into(), " ر.ى.".into()],
            numerals: Some([
                "٠".into(),
                "١".into(),
                "٢".into(),
                "٣".into(),
                "٤".into(),
                "٥".into(),
                "٦".into(),
                "٧".into(),
                "٨".into(),
                "٩".into()
            ]),
            ..Default::default()
        })
        .format("$,.2f")
        .expect("d3 test")
        .format(1234.56),
        "١٬٢٣٤٫٥٦ ر.ى."
    ); // d3-format/test/arabicLocale-test.js:114:3
}

// d3-format/test/defaultLocale-test.js: formatDefaultLocale(definition) returns the new default locale
#[test]
fn formatdefaultlocale_definition_returns_the_new_default_locale() {
    let locale = format::Locale::new(format::Definition {
        decimal: ",".into(),
        thousands: ".".into(),
        grouping: vec![3].into(),
        currency: ["".into(), " €".into()],
        percent: " %".into(),
        ..Default::default()
    });
    assert_eq!(
        locale.format("$,.2f").expect("d3 test").format(12345678.90),
        "12.345.678,90 €"
    ); // d3-format/test/defaultLocale-test.js:22:5
    assert_eq!(
        locale.format(",.0%").expect("d3 test").format(12345678.90),
        "1.234.567.890 %"
    ); // d3-format/test/defaultLocale-test.js:23:5
    let _ = format::Locale::new(format::Definition {
        decimal: ".".into(),
        thousands: ",".into(),
        grouping: vec![3].into(),
        currency: ["$".into(), "".into()],
        ..Default::default()
    });
}

// d3-format/test/format-test.js: format(specifier).toString() returns the normalized specifier
#[test]
fn format_specifier_tostring_returns_the_normalized_specifier() {
    assert_eq!(
        format::Locale::EN_US
            .format("d")
            .expect("d3 test")
            .to_string(),
        " >-d"
    ); // d3-format/test/format-test.js:9:3
}

// d3-format/test/format-test.js: format(specifier) throws an error for invalid formats
#[test]
fn format_specifier_throws_an_error_for_invalid_formats() {
    assert!(
        format::Locale::EN_US
            .format("foo")
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/format-test.js:13:3
    assert!(
        format::Locale::EN_US
            .format(".-2s")
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/format-test.js:14:3
    assert!(
        format::Locale::EN_US
            .format(".f")
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/format-test.js:15:3
}

// d3-format/test/format-test.js: format(",.") unreasonable precision values are clamped to reasonable values
#[test]
fn format_unreasonable_precision_values_are_clamped_to_reasonable_values() {
    assert_eq!(
        format::Locale::EN_US
            .format(".30f")
            .expect("d3 test")
            .format(0.0),
        "0.00000000000000000000"
    ); // d3-format/test/format-test.js:19:3
    assert_eq!(
        format::Locale::EN_US
            .format(".0g")
            .expect("d3 test")
            .format(1.0),
        "1"
    ); // d3-format/test/format-test.js:20:3
}

// d3-format/test/format-test.js: format("s") handles very small and very large values
#[test]
fn format_s_handles_very_small_and_very_large_values() {
    assert_eq!(
        format::Locale::EN_US
            .format("s")
            .expect("d3 test")
            .format(f64::from_bits(1)),
        "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005y"
    ); // d3-format/test/format-test.js:24:3
    assert_eq!(
        format::Locale::EN_US
            .format("s")
            .expect("d3 test")
            .format(f64::MAX),
        "179769000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000Y"
    ); // d3-format/test/format-test.js:25:3
}

// d3-format/test/format-test.js: format("n") is equivalent to format(",g")
#[test]
fn format_n_is_equivalent_to_format_g() {
    assert_eq!(
        format::Locale::EN_US
            .format("n")
            .expect("d3 test")
            .format(123456.78),
        "123,457"
    ); // d3-format/test/format-test.js:29:3
    assert_eq!(
        format::Locale::EN_US
            .format(",g")
            .expect("d3 test")
            .format(123456.78),
        "123,457"
    ); // d3-format/test/format-test.js:30:3
}

// d3-format/test/format-test.js: format("012") is equivalent to format("0=12")
#[test]
fn format_012_is_equivalent_to_format_0_12() {
    assert_eq!(
        format::Locale::EN_US
            .format("012")
            .expect("d3 test")
            .format(123.456),
        "00000123.456"
    ); // d3-format/test/format-test.js:34:3
    assert_eq!(
        format::Locale::EN_US
            .format("0=12")
            .expect("d3 test")
            .format(123.456),
        "00000123.456"
    ); // d3-format/test/format-test.js:35:3
}

// d3-format/test/format-trim-test.js: format("~r") trims insignificant zeros
#[test]
fn format_r_trims_insignificant_zeros() {
    let f = format::Locale::EN_US.format("~r").expect("d3 test");
    assert_eq!(f.format(1.0), "1"); // d3-format/test/format-trim-test.js:6:3
    assert_eq!(f.format(0.1), "0.1"); // d3-format/test/format-trim-test.js:7:3
    assert_eq!(f.format(0.01), "0.01"); // d3-format/test/format-trim-test.js:8:3
    assert_eq!(f.format(10.0001), "10.0001"); // d3-format/test/format-trim-test.js:9:3
    assert_eq!(f.format(123.45), "123.45"); // d3-format/test/format-trim-test.js:10:3
    assert_eq!(f.format(123.456), "123.456"); // d3-format/test/format-trim-test.js:11:3
    assert_eq!(f.format(123.4567), "123.457"); // d3-format/test/format-trim-test.js:12:3
    assert_eq!(f.format(0.000009), "0.000009"); // d3-format/test/format-trim-test.js:13:3
    assert_eq!(f.format(0.0000009), "0.0000009"); // d3-format/test/format-trim-test.js:14:3
    assert_eq!(f.format(0.00000009), "0.00000009"); // d3-format/test/format-trim-test.js:15:3
    assert_eq!(f.format(0.111119), "0.111119"); // d3-format/test/format-trim-test.js:16:3
    assert_eq!(f.format(0.1111119), "0.111112"); // d3-format/test/format-trim-test.js:17:3
    assert_eq!(f.format(0.11111119), "0.111111"); // d3-format/test/format-trim-test.js:18:3
}

// d3-format/test/format-trim-test.js: format("~e") trims insignificant zeros
#[test]
fn format_e_trims_insignificant_zeros() {
    let f = format::Locale::EN_US.format("~e").expect("d3 test");
    assert_eq!(f.format(0.0), "0e+0"); // d3-format/test/format-trim-test.js:23:3
    assert_eq!(f.format(42.0), "4.2e+1"); // d3-format/test/format-trim-test.js:24:3
    assert_eq!(f.format(42000000.0), "4.2e+7"); // d3-format/test/format-trim-test.js:25:3
    assert_eq!(f.format(0.042), "4.2e-2"); // d3-format/test/format-trim-test.js:26:3
    assert_eq!(f.format(-4.0), "−4e+0"); // d3-format/test/format-trim-test.js:27:3
    assert_eq!(f.format(-42.0), "−4.2e+1"); // d3-format/test/format-trim-test.js:28:3
    assert_eq!(f.format(42000000000.0), "4.2e+10"); // d3-format/test/format-trim-test.js:29:3
    assert_eq!(f.format(0.00000000042), "4.2e-10"); // d3-format/test/format-trim-test.js:30:3
}

// d3-format/test/format-trim-test.js: format(".4~e") trims insignificant zeros
#[test]
fn format_4_e_trims_insignificant_zeros() {
    let f = format::Locale::EN_US.format(".4~e").expect("d3 test");
    assert_eq!(f.format(0.00000000012345), "1.2345e-10"); // d3-format/test/format-trim-test.js:35:3
    assert_eq!(f.format(0.00000000012340), "1.234e-10"); // d3-format/test/format-trim-test.js:36:3
    assert_eq!(f.format(0.00000000012300), "1.23e-10"); // d3-format/test/format-trim-test.js:37:3
    assert_eq!(f.format(-0.00000000012345), "−1.2345e-10"); // d3-format/test/format-trim-test.js:38:3
    assert_eq!(f.format(-0.00000000012340), "−1.234e-10"); // d3-format/test/format-trim-test.js:39:3
    assert_eq!(f.format(-0.00000000012300), "−1.23e-10"); // d3-format/test/format-trim-test.js:40:3
    assert_eq!(f.format(12345000000.0), "1.2345e+10"); // d3-format/test/format-trim-test.js:41:3
    assert_eq!(f.format(12340000000.0), "1.234e+10"); // d3-format/test/format-trim-test.js:42:3
    assert_eq!(f.format(12300000000.0), "1.23e+10"); // d3-format/test/format-trim-test.js:43:3
    assert_eq!(f.format(-12345000000.0), "−1.2345e+10"); // d3-format/test/format-trim-test.js:44:3
    assert_eq!(f.format(-12340000000.0), "−1.234e+10"); // d3-format/test/format-trim-test.js:45:3
    assert_eq!(f.format(-12300000000.0), "−1.23e+10"); // d3-format/test/format-trim-test.js:46:3
}

// d3-format/test/format-trim-test.js: format("~s") trims insignificant zeros
#[test]
fn format_s_trims_insignificant_zeros() {
    let f = format::Locale::EN_US.format("~s").expect("d3 test");
    assert_eq!(f.format(0.0), "0"); // d3-format/test/format-trim-test.js:51:3
    assert_eq!(f.format(1.0), "1"); // d3-format/test/format-trim-test.js:52:3
    assert_eq!(f.format(10.0), "10"); // d3-format/test/format-trim-test.js:53:3
    assert_eq!(f.format(100.0), "100"); // d3-format/test/format-trim-test.js:54:3
    assert_eq!(f.format(999.5), "999.5"); // d3-format/test/format-trim-test.js:55:3
    assert_eq!(f.format(999500.0), "999.5k"); // d3-format/test/format-trim-test.js:56:3
    assert_eq!(f.format(1000.0), "1k"); // d3-format/test/format-trim-test.js:57:3
    assert_eq!(f.format(1400.0), "1.4k"); // d3-format/test/format-trim-test.js:58:3
    assert_eq!(f.format(1500.0), "1.5k"); // d3-format/test/format-trim-test.js:59:3
    assert_eq!(f.format(1500.5), "1.5005k"); // d3-format/test/format-trim-test.js:60:3
    assert_eq!(f.format(1e-15), "1f"); // d3-format/test/format-trim-test.js:61:3
    assert_eq!(f.format(1e-12), "1p"); // d3-format/test/format-trim-test.js:62:3
    assert_eq!(f.format(1e-9), "1n"); // d3-format/test/format-trim-test.js:63:3
    assert_eq!(f.format(1e-6), "1µ"); // d3-format/test/format-trim-test.js:64:3
    assert_eq!(f.format(1e-3), "1m"); // d3-format/test/format-trim-test.js:65:3
    assert_eq!(f.format(1e0), "1"); // d3-format/test/format-trim-test.js:66:3
    assert_eq!(f.format(1e3), "1k"); // d3-format/test/format-trim-test.js:67:3
    assert_eq!(f.format(1e6), "1M"); // d3-format/test/format-trim-test.js:68:3
    assert_eq!(f.format(1e9), "1G"); // d3-format/test/format-trim-test.js:69:3
    assert_eq!(f.format(1e12), "1T"); // d3-format/test/format-trim-test.js:70:3
    assert_eq!(f.format(1e15), "1P"); // d3-format/test/format-trim-test.js:71:3
}

// d3-format/test/format-trim-test.js: format("~%") trims insignificant zeros
#[test]
fn format_trims_insignificant_zeros() {
    let f = format::Locale::EN_US.format("~%").expect("d3 test");
    assert_eq!(f.format(0.0), "0%"); // d3-format/test/format-trim-test.js:76:3
    assert_eq!(f.format(0.1), "10%"); // d3-format/test/format-trim-test.js:77:3
    assert_eq!(f.format(0.01), "1%"); // d3-format/test/format-trim-test.js:78:3
    assert_eq!(f.format(0.001), "0.1%"); // d3-format/test/format-trim-test.js:79:3
    assert_eq!(f.format(0.0001), "0.01%"); // d3-format/test/format-trim-test.js:80:3
}

// d3-format/test/format-trim-test.js: trimming respects commas
#[test]
fn trimming_respects_commas() {
    let f = format::Locale::EN_US.format(",~g").expect("d3 test");
    assert_eq!(f.format(10000.0), "10,000"); // d3-format/test/format-trim-test.js:85:3
    assert_eq!(f.format(10000.1), "10,000.1"); // d3-format/test/format-trim-test.js:86:3
}

// d3-format/test/format-type-%-test.js: format("%") can output a whole percentage
#[test]
fn format_can_output_a_whole_percentage() {
    let f = format::Locale::EN_US.format(".0%").expect("d3 test");
    assert_eq!(f.format(0.0), "0%"); // d3-format/test/format-type-%-test.js:6:3
    assert_eq!(f.format(0.042), "4%"); // d3-format/test/format-type-%-test.js:7:3
    assert_eq!(f.format(0.42), "42%"); // d3-format/test/format-type-%-test.js:8:3
    assert_eq!(f.format(4.2), "420%"); // d3-format/test/format-type-%-test.js:9:3
    assert_eq!(f.format(-0.042), "−4%"); // d3-format/test/format-type-%-test.js:10:3
    assert_eq!(f.format(-0.42), "−42%"); // d3-format/test/format-type-%-test.js:11:3
    assert_eq!(f.format(-4.2), "−420%"); // d3-format/test/format-type-%-test.js:12:3
}

// d3-format/test/format-type-%-test.js: format(".%") can output a percentage with precision
#[test]
fn format_can_output_a_percentage_with_precision() {
    let f1 = format::Locale::EN_US.format(".1%").expect("d3 test");
    assert_eq!(f1.format(0.234), "23.4%"); // d3-format/test/format-type-%-test.js:17:3
    let f2 = format::Locale::EN_US.format(".2%").expect("d3 test");
    assert_eq!(f2.format(0.234), "23.40%"); // d3-format/test/format-type-%-test.js:19:3
}

// d3-format/test/format-type-%-test.js: format("%") fill respects suffix
#[test]
fn format_fill_respects_suffix() {
    assert_eq!(
        format::Locale::EN_US
            .format("020.0%")
            .expect("d3 test")
            .format(42.0),
        "0000000000000004200%"
    ); // d3-format/test/format-type-%-test.js:23:3
    assert_eq!(
        format::Locale::EN_US
            .format("20.0%")
            .expect("d3 test")
            .format(42.0),
        "               4200%"
    ); // d3-format/test/format-type-%-test.js:24:3
}

// d3-format/test/format-type-%-test.js: format("^%") align center puts suffix adjacent to number
#[test]
fn format_align_center_puts_suffix_adjacent_to_number() {
    assert_eq!(
        format::Locale::EN_US
            .format("^21.0%")
            .expect("d3 test")
            .format(0.42),
        "         42%         "
    ); // d3-format/test/format-type-%-test.js:28:3
    assert_eq!(
        format::Locale::EN_US
            .format("^21,.0%")
            .expect("d3 test")
            .format(422.0),
        "       42,200%       "
    ); // d3-format/test/format-type-%-test.js:29:3
    assert_eq!(
        format::Locale::EN_US
            .format("^21,.0%")
            .expect("d3 test")
            .format(-422.0),
        "      −42,200%       "
    ); // d3-format/test/format-type-%-test.js:30:3
}

// d3-format/test/format-type-b-test.js: format("b") binary
#[test]
fn format_b_binary() {
    assert_eq!(
        format::Locale::EN_US
            .format("b")
            .expect("d3 test")
            .format(10.0),
        "1010"
    ); // d3-format/test/format-type-b-test.js:5:3
}

// d3-format/test/format-type-b-test.js: format("#b") binary with prefix
#[test]
fn format_b_binary_with_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("#b")
            .expect("d3 test")
            .format(10.0),
        "0b1010"
    ); // d3-format/test/format-type-b-test.js:9:3
}

// d3-format/test/format-type-d-test.js: format("d") can zero fill
#[test]
fn format_d_can_zero_fill() {
    let f = format::Locale::EN_US.format("08d").expect("d3 test");
    assert_eq!(f.format(0.0), "00000000"); // d3-format/test/format-type-d-test.js:6:3
    assert_eq!(f.format(42.0), "00000042"); // d3-format/test/format-type-d-test.js:7:3
    assert_eq!(f.format(42000000.0), "42000000"); // d3-format/test/format-type-d-test.js:8:3
    assert_eq!(f.format(420000000.0), "420000000"); // d3-format/test/format-type-d-test.js:9:3
    assert_eq!(f.format(-4.0), "−0000004"); // d3-format/test/format-type-d-test.js:10:3
    assert_eq!(f.format(-42.0), "−0000042"); // d3-format/test/format-type-d-test.js:11:3
    assert_eq!(f.format(-4200000.0), "−4200000"); // d3-format/test/format-type-d-test.js:12:3
    assert_eq!(f.format(-42000000.0), "−42000000"); // d3-format/test/format-type-d-test.js:13:3
}

// d3-format/test/format-type-d-test.js: format("d") can space fill
#[test]
fn format_d_can_space_fill() {
    let f = format::Locale::EN_US.format("8d").expect("d3 test");
    assert_eq!(f.format(0.0), "       0"); // d3-format/test/format-type-d-test.js:18:3
    assert_eq!(f.format(42.0), "      42"); // d3-format/test/format-type-d-test.js:19:3
    assert_eq!(f.format(42000000.0), "42000000"); // d3-format/test/format-type-d-test.js:20:3
    assert_eq!(f.format(420000000.0), "420000000"); // d3-format/test/format-type-d-test.js:21:3
    assert_eq!(f.format(-4.0), "      −4"); // d3-format/test/format-type-d-test.js:22:3
    assert_eq!(f.format(-42.0), "     −42"); // d3-format/test/format-type-d-test.js:23:3
    assert_eq!(f.format(-4200000.0), "−4200000"); // d3-format/test/format-type-d-test.js:24:3
    assert_eq!(f.format(-42000000.0), "−42000000"); // d3-format/test/format-type-d-test.js:25:3
}

// d3-format/test/format-type-d-test.js: format("d") can underscore fill
#[test]
fn format_d_can_underscore_fill() {
    let f = format::Locale::EN_US.format("_>8d").expect("d3 test");
    assert_eq!(f.format(0.0), "_______0"); // d3-format/test/format-type-d-test.js:30:3
    assert_eq!(f.format(42.0), "______42"); // d3-format/test/format-type-d-test.js:31:3
    assert_eq!(f.format(42000000.0), "42000000"); // d3-format/test/format-type-d-test.js:32:3
    assert_eq!(f.format(420000000.0), "420000000"); // d3-format/test/format-type-d-test.js:33:3
    assert_eq!(f.format(-4.0), "______−4"); // d3-format/test/format-type-d-test.js:34:3
    assert_eq!(f.format(-42.0), "_____−42"); // d3-format/test/format-type-d-test.js:35:3
    assert_eq!(f.format(-4200000.0), "−4200000"); // d3-format/test/format-type-d-test.js:36:3
    assert_eq!(f.format(-42000000.0), "−42000000"); // d3-format/test/format-type-d-test.js:37:3
}

// d3-format/test/format-type-d-test.js: format("d") can zero fill with sign and group
#[test]
fn format_d_can_zero_fill_with_sign_and_group() {
    let f = format::Locale::EN_US.format("+08,d").expect("d3 test");
    assert_eq!(f.format(0.0), "+0,000,000"); // d3-format/test/format-type-d-test.js:42:3
    assert_eq!(f.format(42.0), "+0,000,042"); // d3-format/test/format-type-d-test.js:43:3
    assert_eq!(f.format(42000000.0), "+42,000,000"); // d3-format/test/format-type-d-test.js:44:3
    assert_eq!(f.format(420000000.0), "+420,000,000"); // d3-format/test/format-type-d-test.js:45:3
    assert_eq!(f.format(-4.0), "−0,000,004"); // d3-format/test/format-type-d-test.js:46:3
    assert_eq!(f.format(-42.0), "−0,000,042"); // d3-format/test/format-type-d-test.js:47:3
    assert_eq!(f.format(-4200000.0), "−4,200,000"); // d3-format/test/format-type-d-test.js:48:3
    assert_eq!(f.format(-42000000.0), "−42,000,000"); // d3-format/test/format-type-d-test.js:49:3
}

// d3-format/test/format-type-d-test.js: format("d") always uses zero precision
#[test]
fn format_d_always_uses_zero_precision() {
    let f = format::Locale::EN_US.format(".2d").expect("d3 test");
    assert_eq!(f.format(0.0), "0"); // d3-format/test/format-type-d-test.js:54:3
    assert_eq!(f.format(42.0), "42"); // d3-format/test/format-type-d-test.js:55:3
    assert_eq!(f.format(-4.2), "−4"); // d3-format/test/format-type-d-test.js:56:3
}

// d3-format/test/format-type-d-test.js: format("d") rounds non-integers
#[test]
fn format_d_rounds_non_integers() {
    let f = format::Locale::EN_US.format("d").expect("d3 test");
    assert_eq!(f.format(4.2), "4"); // d3-format/test/format-type-d-test.js:61:3
}

// d3-format/test/format-type-d-test.js: format(",d") can group thousands
#[test]
fn format_d_can_group_thousands() {
    let f = format::Locale::EN_US.format(",d").expect("d3 test");
    assert_eq!(f.format(0.0), "0"); // d3-format/test/format-type-d-test.js:66:3
    assert_eq!(f.format(42.0), "42"); // d3-format/test/format-type-d-test.js:67:3
    assert_eq!(f.format(42000000.0), "42,000,000"); // d3-format/test/format-type-d-test.js:68:3
    assert_eq!(f.format(420000000.0), "420,000,000"); // d3-format/test/format-type-d-test.js:69:3
    assert_eq!(f.format(-4.0), "−4"); // d3-format/test/format-type-d-test.js:70:3
    assert_eq!(f.format(-42.0), "−42"); // d3-format/test/format-type-d-test.js:71:3
    assert_eq!(f.format(-4200000.0), "−4,200,000"); // d3-format/test/format-type-d-test.js:72:3
    assert_eq!(f.format(-42000000.0), "−42,000,000"); // d3-format/test/format-type-d-test.js:73:3
    assert_eq!(f.format(1e21), "1,000,000,000,000,000,000,000"); // d3-format/test/format-type-d-test.js:74:3
    assert_eq!(f.format(1.3e27), "1,300,000,000,000,000,000,000,000,000"); // d3-format/test/format-type-d-test.js:75:3
    assert_eq!(
        f.format(1.3e107),
        "130,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:76:3
}

// d3-format/test/format-type-d-test.js: format("0,d") can group thousands and zero fill
#[test]
fn format_0_d_can_group_thousands_and_zero_fill() {
    assert_eq!(
        format::Locale::EN_US
            .format("01,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:80:3
    assert_eq!(
        format::Locale::EN_US
            .format("01,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:81:3
    assert_eq!(
        format::Locale::EN_US
            .format("02,d")
            .expect("d3 test")
            .format(0.0),
        "00"
    ); // d3-format/test/format-type-d-test.js:82:3
    assert_eq!(
        format::Locale::EN_US
            .format("03,d")
            .expect("d3 test")
            .format(0.0),
        "000"
    ); // d3-format/test/format-type-d-test.js:83:3
    assert_eq!(
        format::Locale::EN_US
            .format("04,d")
            .expect("d3 test")
            .format(0.0),
        "0,000"
    ); // d3-format/test/format-type-d-test.js:84:3
    assert_eq!(
        format::Locale::EN_US
            .format("05,d")
            .expect("d3 test")
            .format(0.0),
        "0,000"
    ); // d3-format/test/format-type-d-test.js:85:3
    assert_eq!(
        format::Locale::EN_US
            .format("06,d")
            .expect("d3 test")
            .format(0.0),
        "00,000"
    ); // d3-format/test/format-type-d-test.js:86:3
    assert_eq!(
        format::Locale::EN_US
            .format("08,d")
            .expect("d3 test")
            .format(0.0),
        "0,000,000"
    ); // d3-format/test/format-type-d-test.js:87:3
    assert_eq!(
        format::Locale::EN_US
            .format("013,d")
            .expect("d3 test")
            .format(0.0),
        "0,000,000,000"
    ); // d3-format/test/format-type-d-test.js:88:3
    assert_eq!(
        format::Locale::EN_US
            .format("021,d")
            .expect("d3 test")
            .format(0.0),
        "0,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:89:3
    assert_eq!(
        format::Locale::EN_US
            .format("013,d")
            .expect("d3 test")
            .format(-42000000.0),
        "−0,042,000,000"
    ); // d3-format/test/format-type-d-test.js:90:3
    assert_eq!(
        format::Locale::EN_US
            .format("012,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:91:3
    assert_eq!(
        format::Locale::EN_US
            .format("013,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:92:3
    assert_eq!(
        format::Locale::EN_US
            .format("014,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:93:3
    assert_eq!(
        format::Locale::EN_US
            .format("015,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:94:3
}

// d3-format/test/format-type-d-test.js: format("0,d") can group thousands and zero fill with overflow
#[test]
fn format_0_d_can_group_thousands_and_zero_fill_with_overflow() {
    assert_eq!(
        format::Locale::EN_US
            .format("01,d")
            .expect("d3 test")
            .format(1.0),
        "1"
    ); // d3-format/test/format-type-d-test.js:98:3
    assert_eq!(
        format::Locale::EN_US
            .format("01,d")
            .expect("d3 test")
            .format(1.0),
        "1"
    ); // d3-format/test/format-type-d-test.js:99:3
    assert_eq!(
        format::Locale::EN_US
            .format("02,d")
            .expect("d3 test")
            .format(12.0),
        "12"
    ); // d3-format/test/format-type-d-test.js:100:3
    assert_eq!(
        format::Locale::EN_US
            .format("03,d")
            .expect("d3 test")
            .format(123.0),
        "123"
    ); // d3-format/test/format-type-d-test.js:101:3
    assert_eq!(
        format::Locale::EN_US
            .format("05,d")
            .expect("d3 test")
            .format(12345.0),
        "12,345"
    ); // d3-format/test/format-type-d-test.js:102:3
    assert_eq!(
        format::Locale::EN_US
            .format("08,d")
            .expect("d3 test")
            .format(12345678.0),
        "12,345,678"
    ); // d3-format/test/format-type-d-test.js:103:3
    assert_eq!(
        format::Locale::EN_US
            .format("013,d")
            .expect("d3 test")
            .format(1234567890123.0),
        "1,234,567,890,123"
    ); // d3-format/test/format-type-d-test.js:104:3
}

// d3-format/test/format-type-d-test.js: format(",d") can group thousands and space fill
#[test]
fn format_d_can_group_thousands_and_space_fill() {
    assert_eq!(
        format::Locale::EN_US
            .format("1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:108:3
    assert_eq!(
        format::Locale::EN_US
            .format("1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:109:3
    assert_eq!(
        format::Locale::EN_US
            .format("2,d")
            .expect("d3 test")
            .format(0.0),
        " 0"
    ); // d3-format/test/format-type-d-test.js:110:3
    assert_eq!(
        format::Locale::EN_US
            .format("3,d")
            .expect("d3 test")
            .format(0.0),
        "  0"
    ); // d3-format/test/format-type-d-test.js:111:3
    assert_eq!(
        format::Locale::EN_US
            .format("5,d")
            .expect("d3 test")
            .format(0.0),
        "    0"
    ); // d3-format/test/format-type-d-test.js:112:3
    assert_eq!(
        format::Locale::EN_US
            .format("8,d")
            .expect("d3 test")
            .format(0.0),
        "       0"
    ); // d3-format/test/format-type-d-test.js:113:3
    assert_eq!(
        format::Locale::EN_US
            .format("13,d")
            .expect("d3 test")
            .format(0.0),
        "            0"
    ); // d3-format/test/format-type-d-test.js:114:3
    assert_eq!(
        format::Locale::EN_US
            .format("21,d")
            .expect("d3 test")
            .format(0.0),
        "                    0"
    ); // d3-format/test/format-type-d-test.js:115:3
}

// d3-format/test/format-type-d-test.js: format(",d") can group thousands and space fill with overflow
#[test]
fn format_d_can_group_thousands_and_space_fill_with_overflow() {
    assert_eq!(
        format::Locale::EN_US
            .format("1,d")
            .expect("d3 test")
            .format(1.0),
        "1"
    ); // d3-format/test/format-type-d-test.js:119:3
    assert_eq!(
        format::Locale::EN_US
            .format("1,d")
            .expect("d3 test")
            .format(1.0),
        "1"
    ); // d3-format/test/format-type-d-test.js:120:3
    assert_eq!(
        format::Locale::EN_US
            .format("2,d")
            .expect("d3 test")
            .format(12.0),
        "12"
    ); // d3-format/test/format-type-d-test.js:121:3
    assert_eq!(
        format::Locale::EN_US
            .format("3,d")
            .expect("d3 test")
            .format(123.0),
        "123"
    ); // d3-format/test/format-type-d-test.js:122:3
    assert_eq!(
        format::Locale::EN_US
            .format("5,d")
            .expect("d3 test")
            .format(12345.0),
        "12,345"
    ); // d3-format/test/format-type-d-test.js:123:3
    assert_eq!(
        format::Locale::EN_US
            .format("8,d")
            .expect("d3 test")
            .format(12345678.0),
        "12,345,678"
    ); // d3-format/test/format-type-d-test.js:124:3
    assert_eq!(
        format::Locale::EN_US
            .format("13,d")
            .expect("d3 test")
            .format(1234567890123.0),
        "1,234,567,890,123"
    ); // d3-format/test/format-type-d-test.js:125:3
}

// d3-format/test/format-type-d-test.js: format("<d") align left
#[test]
fn format_d_align_left() {
    assert_eq!(
        format::Locale::EN_US
            .format("<1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:129:3
    assert_eq!(
        format::Locale::EN_US
            .format("<1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:130:3
    assert_eq!(
        format::Locale::EN_US
            .format("<2,d")
            .expect("d3 test")
            .format(0.0),
        "0 "
    ); // d3-format/test/format-type-d-test.js:131:3
    assert_eq!(
        format::Locale::EN_US
            .format("<3,d")
            .expect("d3 test")
            .format(0.0),
        "0  "
    ); // d3-format/test/format-type-d-test.js:132:3
    assert_eq!(
        format::Locale::EN_US
            .format("<5,d")
            .expect("d3 test")
            .format(0.0),
        "0    "
    ); // d3-format/test/format-type-d-test.js:133:3
    assert_eq!(
        format::Locale::EN_US
            .format("<8,d")
            .expect("d3 test")
            .format(0.0),
        "0       "
    ); // d3-format/test/format-type-d-test.js:134:3
    assert_eq!(
        format::Locale::EN_US
            .format("<13,d")
            .expect("d3 test")
            .format(0.0),
        "0            "
    ); // d3-format/test/format-type-d-test.js:135:3
    assert_eq!(
        format::Locale::EN_US
            .format("<21,d")
            .expect("d3 test")
            .format(0.0),
        "0                    "
    ); // d3-format/test/format-type-d-test.js:136:3
}

// d3-format/test/format-type-d-test.js: format(">d") align right
#[test]
fn format_d_align_right() {
    assert_eq!(
        format::Locale::EN_US
            .format(">1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:140:3
    assert_eq!(
        format::Locale::EN_US
            .format(">1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:141:3
    assert_eq!(
        format::Locale::EN_US
            .format(">2,d")
            .expect("d3 test")
            .format(0.0),
        " 0"
    ); // d3-format/test/format-type-d-test.js:142:3
    assert_eq!(
        format::Locale::EN_US
            .format(">3,d")
            .expect("d3 test")
            .format(0.0),
        "  0"
    ); // d3-format/test/format-type-d-test.js:143:3
    assert_eq!(
        format::Locale::EN_US
            .format(">5,d")
            .expect("d3 test")
            .format(0.0),
        "    0"
    ); // d3-format/test/format-type-d-test.js:144:3
    assert_eq!(
        format::Locale::EN_US
            .format(">8,d")
            .expect("d3 test")
            .format(0.0),
        "       0"
    ); // d3-format/test/format-type-d-test.js:145:3
    assert_eq!(
        format::Locale::EN_US
            .format(">13,d")
            .expect("d3 test")
            .format(0.0),
        "            0"
    ); // d3-format/test/format-type-d-test.js:146:3
    assert_eq!(
        format::Locale::EN_US
            .format(">21,d")
            .expect("d3 test")
            .format(0.0),
        "                    0"
    ); // d3-format/test/format-type-d-test.js:147:3
    assert_eq!(
        format::Locale::EN_US
            .format(">21,d")
            .expect("d3 test")
            .format(1000.0),
        "                1,000"
    ); // d3-format/test/format-type-d-test.js:148:3
    assert_eq!(
        format::Locale::EN_US
            .format(">21,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:149:3
}

// d3-format/test/format-type-d-test.js: format("^d") align center
#[test]
fn format_d_align_center() {
    assert_eq!(
        format::Locale::EN_US
            .format("^1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:153:3
    assert_eq!(
        format::Locale::EN_US
            .format("^1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:154:3
    assert_eq!(
        format::Locale::EN_US
            .format("^2,d")
            .expect("d3 test")
            .format(0.0),
        "0 "
    ); // d3-format/test/format-type-d-test.js:155:3
    assert_eq!(
        format::Locale::EN_US
            .format("^3,d")
            .expect("d3 test")
            .format(0.0),
        " 0 "
    ); // d3-format/test/format-type-d-test.js:156:3
    assert_eq!(
        format::Locale::EN_US
            .format("^5,d")
            .expect("d3 test")
            .format(0.0),
        "  0  "
    ); // d3-format/test/format-type-d-test.js:157:3
    assert_eq!(
        format::Locale::EN_US
            .format("^8,d")
            .expect("d3 test")
            .format(0.0),
        "   0    "
    ); // d3-format/test/format-type-d-test.js:158:3
    assert_eq!(
        format::Locale::EN_US
            .format("^13,d")
            .expect("d3 test")
            .format(0.0),
        "      0      "
    ); // d3-format/test/format-type-d-test.js:159:3
    assert_eq!(
        format::Locale::EN_US
            .format("^21,d")
            .expect("d3 test")
            .format(0.0),
        "          0          "
    ); // d3-format/test/format-type-d-test.js:160:3
    assert_eq!(
        format::Locale::EN_US
            .format("^21,d")
            .expect("d3 test")
            .format(1000.0),
        "        1,000        "
    ); // d3-format/test/format-type-d-test.js:161:3
    assert_eq!(
        format::Locale::EN_US
            .format("^21,d")
            .expect("d3 test")
            .format(1e21),
        "1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:162:3
}

// d3-format/test/format-type-d-test.js: format("=+,d") pad after sign
#[test]
fn format_d_pad_after_sign() {
    assert_eq!(
        format::Locale::EN_US
            .format("=+1,d")
            .expect("d3 test")
            .format(0.0),
        "+0"
    ); // d3-format/test/format-type-d-test.js:166:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+1,d")
            .expect("d3 test")
            .format(0.0),
        "+0"
    ); // d3-format/test/format-type-d-test.js:167:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+2,d")
            .expect("d3 test")
            .format(0.0),
        "+0"
    ); // d3-format/test/format-type-d-test.js:168:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+3,d")
            .expect("d3 test")
            .format(0.0),
        "+ 0"
    ); // d3-format/test/format-type-d-test.js:169:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+5,d")
            .expect("d3 test")
            .format(0.0),
        "+   0"
    ); // d3-format/test/format-type-d-test.js:170:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+8,d")
            .expect("d3 test")
            .format(0.0),
        "+      0"
    ); // d3-format/test/format-type-d-test.js:171:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+13,d")
            .expect("d3 test")
            .format(0.0),
        "+           0"
    ); // d3-format/test/format-type-d-test.js:172:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+21,d")
            .expect("d3 test")
            .format(0.0),
        "+                   0"
    ); // d3-format/test/format-type-d-test.js:173:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+21,d")
            .expect("d3 test")
            .format(1e21),
        "+1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:174:3
}

// d3-format/test/format-type-d-test.js: format("=+$,d") pad after sign with currency
#[test]
fn format_d_pad_after_sign_with_currency() {
    assert_eq!(
        format::Locale::EN_US
            .format("=+$1,d")
            .expect("d3 test")
            .format(0.0),
        "+$0"
    ); // d3-format/test/format-type-d-test.js:178:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$1,d")
            .expect("d3 test")
            .format(0.0),
        "+$0"
    ); // d3-format/test/format-type-d-test.js:179:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$2,d")
            .expect("d3 test")
            .format(0.0),
        "+$0"
    ); // d3-format/test/format-type-d-test.js:180:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$3,d")
            .expect("d3 test")
            .format(0.0),
        "+$0"
    ); // d3-format/test/format-type-d-test.js:181:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$5,d")
            .expect("d3 test")
            .format(0.0),
        "+$  0"
    ); // d3-format/test/format-type-d-test.js:182:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$8,d")
            .expect("d3 test")
            .format(0.0),
        "+$     0"
    ); // d3-format/test/format-type-d-test.js:183:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$13,d")
            .expect("d3 test")
            .format(0.0),
        "+$          0"
    ); // d3-format/test/format-type-d-test.js:184:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$21,d")
            .expect("d3 test")
            .format(0.0),
        "+$                  0"
    ); // d3-format/test/format-type-d-test.js:185:3
    assert_eq!(
        format::Locale::EN_US
            .format("=+$21,d")
            .expect("d3 test")
            .format(1e21),
        "+$1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:186:3
}

// d3-format/test/format-type-d-test.js: format(" ,d") a space can denote positive numbers
#[test]
fn format_d_a_space_can_denote_positive_numbers() {
    assert_eq!(
        format::Locale::EN_US
            .format(" 1,d")
            .expect("d3 test")
            .format(-1.0),
        "−1"
    ); // d3-format/test/format-type-d-test.js:190:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 1,d")
            .expect("d3 test")
            .format(0.0),
        " 0"
    ); // d3-format/test/format-type-d-test.js:191:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 2,d")
            .expect("d3 test")
            .format(0.0),
        " 0"
    ); // d3-format/test/format-type-d-test.js:192:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 3,d")
            .expect("d3 test")
            .format(0.0),
        "  0"
    ); // d3-format/test/format-type-d-test.js:193:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 5,d")
            .expect("d3 test")
            .format(0.0),
        "    0"
    ); // d3-format/test/format-type-d-test.js:194:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 8,d")
            .expect("d3 test")
            .format(0.0),
        "       0"
    ); // d3-format/test/format-type-d-test.js:195:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 13,d")
            .expect("d3 test")
            .format(0.0),
        "            0"
    ); // d3-format/test/format-type-d-test.js:196:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 21,d")
            .expect("d3 test")
            .format(0.0),
        "                    0"
    ); // d3-format/test/format-type-d-test.js:197:3
    assert_eq!(
        format::Locale::EN_US
            .format(" 21,d")
            .expect("d3 test")
            .format(1e21),
        " 1,000,000,000,000,000,000,000"
    ); // d3-format/test/format-type-d-test.js:198:3
}

// d3-format/test/format-type-d-test.js: format("-,d") explicitly only use a sign for negative numbers
#[test]
fn format_d_explicitly_only_use_a_sign_for_negative_numbers() {
    assert_eq!(
        format::Locale::EN_US
            .format("-1,d")
            .expect("d3 test")
            .format(-1.0),
        "−1"
    ); // d3-format/test/format-type-d-test.js:202:3
    assert_eq!(
        format::Locale::EN_US
            .format("-1,d")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:203:3
    assert_eq!(
        format::Locale::EN_US
            .format("-2,d")
            .expect("d3 test")
            .format(0.0),
        " 0"
    ); // d3-format/test/format-type-d-test.js:204:3
    assert_eq!(
        format::Locale::EN_US
            .format("-3,d")
            .expect("d3 test")
            .format(0.0),
        "  0"
    ); // d3-format/test/format-type-d-test.js:205:3
    assert_eq!(
        format::Locale::EN_US
            .format("-5,d")
            .expect("d3 test")
            .format(0.0),
        "    0"
    ); // d3-format/test/format-type-d-test.js:206:3
    assert_eq!(
        format::Locale::EN_US
            .format("-8,d")
            .expect("d3 test")
            .format(0.0),
        "       0"
    ); // d3-format/test/format-type-d-test.js:207:3
    assert_eq!(
        format::Locale::EN_US
            .format("-13,d")
            .expect("d3 test")
            .format(0.0),
        "            0"
    ); // d3-format/test/format-type-d-test.js:208:3
    assert_eq!(
        format::Locale::EN_US
            .format("-21,d")
            .expect("d3 test")
            .format(0.0),
        "                    0"
    ); // d3-format/test/format-type-d-test.js:209:3
}

// d3-format/test/format-type-d-test.js: format("d") can format negative zero as zero
#[test]
fn format_d_can_format_negative_zero_as_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format("1d")
            .expect("d3 test")
            .format(-0.0),
        "0"
    ); // d3-format/test/format-type-d-test.js:213:3
    assert_eq!(
        format::Locale::EN_US
            .format("1d")
            .expect("d3 test")
            .format(-1e-12),
        "0"
    ); // d3-format/test/format-type-d-test.js:214:3
}

// d3-format/test/format-type-e-test.js: format("e") can output exponent notation
#[test]
fn format_e_can_output_exponent_notation() {
    let f = format::Locale::EN_US.format("e").expect("d3 test");
    assert_eq!(f.format(0.0), "0.000000e+0"); // d3-format/test/format-type-e-test.js:6:3
    assert_eq!(f.format(42.0), "4.200000e+1"); // d3-format/test/format-type-e-test.js:7:3
    assert_eq!(f.format(42000000.0), "4.200000e+7"); // d3-format/test/format-type-e-test.js:8:3
    assert_eq!(f.format(420000000.0), "4.200000e+8"); // d3-format/test/format-type-e-test.js:9:3
    assert_eq!(f.format(-4.0), "−4.000000e+0"); // d3-format/test/format-type-e-test.js:10:3
    assert_eq!(f.format(-42.0), "−4.200000e+1"); // d3-format/test/format-type-e-test.js:11:3
    assert_eq!(f.format(-4200000.0), "−4.200000e+6"); // d3-format/test/format-type-e-test.js:12:3
    assert_eq!(f.format(-42000000.0), "−4.200000e+7"); // d3-format/test/format-type-e-test.js:13:3
    assert_eq!(
        format::Locale::EN_US
            .format(".0e")
            .expect("d3 test")
            .format(42.0),
        "4e+1"
    ); // d3-format/test/format-type-e-test.js:14:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3e")
            .expect("d3 test")
            .format(42.0),
        "4.200e+1"
    ); // d3-format/test/format-type-e-test.js:15:3
}

// d3-format/test/format-type-e-test.js: format("e") can format negative zero as zero
#[test]
fn format_e_can_format_negative_zero_as_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format("1e")
            .expect("d3 test")
            .format(-0.0),
        "0.000000e+0"
    ); // d3-format/test/format-type-e-test.js:19:3
    assert_eq!(
        format::Locale::EN_US
            .format("1e")
            .expect("d3 test")
            .format(-1e-12),
        "−1.000000e-12"
    ); // d3-format/test/format-type-e-test.js:20:3
}

// d3-format/test/format-type-e-test.js: format(",e") does not group Infinity
#[test]
fn format_e_does_not_group_infinity() {
    assert_eq!(
        format::Locale::EN_US
            .format(",e")
            .expect("d3 test")
            .format(f64::INFINITY),
        "Infinity"
    ); // d3-format/test/format-type-e-test.js:24:3
}

// d3-format/test/format-type-e-test.js: format(".3e") can format negative infinity
#[test]
fn format_3e_can_format_negative_infinity() {
    assert_eq!(
        format::Locale::EN_US
            .format(".3e")
            .expect("d3 test")
            .format(-f64::INFINITY),
        "−Infinity"
    ); // d3-format/test/format-type-e-test.js:28:3
}

// d3-format/test/format-type-f-test.js: format("f") can output fixed-point notation
#[test]
fn format_f_can_output_fixed_point_notation() {
    assert_eq!(
        format::Locale::EN_US
            .format(".1f")
            .expect("d3 test")
            .format(0.49),
        "0.5"
    ); // d3-format/test/format-type-f-test.js:5:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2f")
            .expect("d3 test")
            .format(0.449),
        "0.45"
    ); // d3-format/test/format-type-f-test.js:6:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3f")
            .expect("d3 test")
            .format(0.4449),
        "0.445"
    ); // d3-format/test/format-type-f-test.js:7:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5f")
            .expect("d3 test")
            .format(0.444449),
        "0.44445"
    ); // d3-format/test/format-type-f-test.js:8:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1f")
            .expect("d3 test")
            .format(100.0),
        "100.0"
    ); // d3-format/test/format-type-f-test.js:9:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2f")
            .expect("d3 test")
            .format(100.0),
        "100.00"
    ); // d3-format/test/format-type-f-test.js:10:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3f")
            .expect("d3 test")
            .format(100.0),
        "100.000"
    ); // d3-format/test/format-type-f-test.js:11:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5f")
            .expect("d3 test")
            .format(100.0),
        "100.00000"
    ); // d3-format/test/format-type-f-test.js:12:3
}

// d3-format/test/format-type-f-test.js: format("+$,f") can output a currency with comma-grouping and sign
#[test]
fn format_f_can_output_a_currency_with_comma_grouping_and_sign() {
    let f = format::Locale::EN_US.format("+$,.2f").expect("d3 test");
    assert_eq!(f.format(0.0), "+$0.00"); // d3-format/test/format-type-f-test.js:17:3
    assert_eq!(f.format(0.429), "+$0.43"); // d3-format/test/format-type-f-test.js:18:3
    assert_eq!(f.format(-0.429), "−$0.43"); // d3-format/test/format-type-f-test.js:19:3
    assert_eq!(f.format(-1.0), "−$1.00"); // d3-format/test/format-type-f-test.js:20:3
    assert_eq!(f.format(1e4), "+$10,000.00"); // d3-format/test/format-type-f-test.js:21:3
}

// d3-format/test/format-type-f-test.js: format(",.f") can group thousands, space fill, and round to significant digits
#[test]
fn format_f_can_group_thousands_space_fill_and_round_to_significant_digits() {
    assert_eq!(
        format::Locale::EN_US
            .format("10,.1f")
            .expect("d3 test")
            .format(123456.49),
        " 123,456.5"
    ); // d3-format/test/format-type-f-test.js:25:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.2f")
            .expect("d3 test")
            .format(1234567.449),
        "1,234,567.45"
    ); // d3-format/test/format-type-f-test.js:26:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.3f")
            .expect("d3 test")
            .format(12345678.4449),
        "12,345,678.445"
    ); // d3-format/test/format-type-f-test.js:27:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.5f")
            .expect("d3 test")
            .format(123456789.444449),
        "123,456,789.44445"
    ); // d3-format/test/format-type-f-test.js:28:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.1f")
            .expect("d3 test")
            .format(123456.0),
        " 123,456.0"
    ); // d3-format/test/format-type-f-test.js:29:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.2f")
            .expect("d3 test")
            .format(1234567.0),
        "1,234,567.00"
    ); // d3-format/test/format-type-f-test.js:30:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.3f")
            .expect("d3 test")
            .format(12345678.0),
        "12,345,678.000"
    ); // d3-format/test/format-type-f-test.js:31:3
    assert_eq!(
        format::Locale::EN_US
            .format("10,.5f")
            .expect("d3 test")
            .format(123456789.0),
        "123,456,789.00000"
    ); // d3-format/test/format-type-f-test.js:32:3
}

// d3-format/test/format-type-f-test.js: format("f") can display integers in fixed-point notation
#[test]
fn format_f_can_display_integers_in_fixed_point_notation() {
    assert_eq!(
        format::Locale::EN_US
            .format("f")
            .expect("d3 test")
            .format(42.0),
        "42.000000"
    ); // d3-format/test/format-type-f-test.js:36:3
}

// d3-format/test/format-type-f-test.js: format("f") can format negative zero as zero
#[test]
fn format_f_can_format_negative_zero_as_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format("f")
            .expect("d3 test")
            .format(-0.0),
        "0.000000"
    ); // d3-format/test/format-type-f-test.js:40:3
    assert_eq!(
        format::Locale::EN_US
            .format("f")
            .expect("d3 test")
            .format(-1e-12),
        "0.000000"
    ); // d3-format/test/format-type-f-test.js:41:3
}

// d3-format/test/format-type-f-test.js: format("+f") signs negative zero correctly
#[test]
fn format_f_signs_negative_zero_correctly() {
    assert_eq!(
        format::Locale::EN_US
            .format("+f")
            .expect("d3 test")
            .format(-0.0),
        "−0.000000"
    ); // d3-format/test/format-type-f-test.js:45:3
    assert_eq!(
        format::Locale::EN_US
            .format("+f")
            .expect("d3 test")
            .format(0.0),
        "+0.000000"
    ); // d3-format/test/format-type-f-test.js:46:3
    assert_eq!(
        format::Locale::EN_US
            .format("+f")
            .expect("d3 test")
            .format(-1e-12),
        "−0.000000"
    ); // d3-format/test/format-type-f-test.js:47:3
    assert_eq!(
        format::Locale::EN_US
            .format("+f")
            .expect("d3 test")
            .format(1e-12),
        "+0.000000"
    ); // d3-format/test/format-type-f-test.js:48:3
}

// d3-format/test/format-type-f-test.js: format("f") can format negative infinity
#[test]
fn format_f_can_format_negative_infinity() {
    assert_eq!(
        format::Locale::EN_US
            .format("f")
            .expect("d3 test")
            .format(-f64::INFINITY),
        "−Infinity"
    ); // d3-format/test/format-type-f-test.js:52:3
}

// d3-format/test/format-type-f-test.js: format(",f") does not group Infinity
#[test]
fn format_f_does_not_group_infinity() {
    assert_eq!(
        format::Locale::EN_US
            .format(",f")
            .expect("d3 test")
            .format(f64::INFINITY),
        "Infinity"
    ); // d3-format/test/format-type-f-test.js:56:3
}

// d3-format/test/format-type-g-test.js: format("g") can output general notation
#[test]
fn format_g_can_output_general_notation() {
    assert_eq!(
        format::Locale::EN_US
            .format(".1g")
            .expect("d3 test")
            .format(0.049),
        "0.05"
    ); // d3-format/test/format-type-g-test.js:5:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1g")
            .expect("d3 test")
            .format(0.49),
        "0.5"
    ); // d3-format/test/format-type-g-test.js:6:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2g")
            .expect("d3 test")
            .format(0.449),
        "0.45"
    ); // d3-format/test/format-type-g-test.js:7:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3g")
            .expect("d3 test")
            .format(0.4449),
        "0.445"
    ); // d3-format/test/format-type-g-test.js:8:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5g")
            .expect("d3 test")
            .format(0.444449),
        "0.44445"
    ); // d3-format/test/format-type-g-test.js:9:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1g")
            .expect("d3 test")
            .format(100.0),
        "1e+2"
    ); // d3-format/test/format-type-g-test.js:10:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2g")
            .expect("d3 test")
            .format(100.0),
        "1.0e+2"
    ); // d3-format/test/format-type-g-test.js:11:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3g")
            .expect("d3 test")
            .format(100.0),
        "100"
    ); // d3-format/test/format-type-g-test.js:12:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5g")
            .expect("d3 test")
            .format(100.0),
        "100.00"
    ); // d3-format/test/format-type-g-test.js:13:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5g")
            .expect("d3 test")
            .format(100.2),
        "100.20"
    ); // d3-format/test/format-type-g-test.js:14:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2g")
            .expect("d3 test")
            .format(0.002),
        "0.0020"
    ); // d3-format/test/format-type-g-test.js:15:3
}

// d3-format/test/format-type-g-test.js: format(",g") can group thousands with general notation
#[test]
fn format_g_can_group_thousands_with_general_notation() {
    let f = format::Locale::EN_US.format(",.12g").expect("d3 test");
    assert_eq!(f.format(0.0), "0.00000000000"); // d3-format/test/format-type-g-test.js:20:3
    assert_eq!(f.format(42.0), "42.0000000000"); // d3-format/test/format-type-g-test.js:21:3
    assert_eq!(f.format(42000000.0), "42,000,000.0000"); // d3-format/test/format-type-g-test.js:22:3
    assert_eq!(f.format(420000000.0), "420,000,000.000"); // d3-format/test/format-type-g-test.js:23:3
    assert_eq!(f.format(-4.0), "−4.00000000000"); // d3-format/test/format-type-g-test.js:24:3
    assert_eq!(f.format(-42.0), "−42.0000000000"); // d3-format/test/format-type-g-test.js:25:3
    assert_eq!(f.format(-4200000.0), "−4,200,000.00000"); // d3-format/test/format-type-g-test.js:26:3
    assert_eq!(f.format(-42000000.0), "−42,000,000.0000"); // d3-format/test/format-type-g-test.js:27:3
}

// d3-format/test/format-type-n-test.js: format("n") is an alias for ",g"
#[test]
fn format_n_is_an_alias_for_g() {
    let f = format::Locale::EN_US.format(".12n").expect("d3 test");
    assert_eq!(f.format(0.0), "0.00000000000"); // d3-format/test/format-type-n-test.js:6:3
    assert_eq!(f.format(42.0), "42.0000000000"); // d3-format/test/format-type-n-test.js:7:3
    assert_eq!(f.format(42000000.0), "42,000,000.0000"); // d3-format/test/format-type-n-test.js:8:3
    assert_eq!(f.format(420000000.0), "420,000,000.000"); // d3-format/test/format-type-n-test.js:9:3
    assert_eq!(f.format(-4.0), "−4.00000000000"); // d3-format/test/format-type-n-test.js:10:3
    assert_eq!(f.format(-42.0), "−42.0000000000"); // d3-format/test/format-type-n-test.js:11:3
    assert_eq!(f.format(-4200000.0), "−4,200,000.00000"); // d3-format/test/format-type-n-test.js:12:3
    assert_eq!(f.format(-42000000.0), "−42,000,000.0000"); // d3-format/test/format-type-n-test.js:13:3
    assert_eq!(f.format(0.0042), "0.00420000000000"); // d3-format/test/format-type-n-test.js:14:3
    assert_eq!(f.format(0.42), "0.420000000000"); // d3-format/test/format-type-n-test.js:15:3
    assert_eq!(f.format(1e21), "1.00000000000e+21"); // d3-format/test/format-type-n-test.js:16:3
}

// d3-format/test/format-type-n-test.js: format("n") uses zero padding
#[test]
fn format_n_uses_zero_padding() {
    assert_eq!(
        format::Locale::EN_US
            .format("01.0n")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-n-test.js:20:3
    assert_eq!(
        format::Locale::EN_US
            .format("02.0n")
            .expect("d3 test")
            .format(0.0),
        "00"
    ); // d3-format/test/format-type-n-test.js:21:3
    assert_eq!(
        format::Locale::EN_US
            .format("03.0n")
            .expect("d3 test")
            .format(0.0),
        "000"
    ); // d3-format/test/format-type-n-test.js:22:3
    assert_eq!(
        format::Locale::EN_US
            .format("05.0n")
            .expect("d3 test")
            .format(0.0),
        "0,000"
    ); // d3-format/test/format-type-n-test.js:23:3
    assert_eq!(
        format::Locale::EN_US
            .format("08.0n")
            .expect("d3 test")
            .format(0.0),
        "0,000,000"
    ); // d3-format/test/format-type-n-test.js:24:3
    assert_eq!(
        format::Locale::EN_US
            .format("013.0n")
            .expect("d3 test")
            .format(0.0),
        "0,000,000,000"
    ); // d3-format/test/format-type-n-test.js:25:3
    assert_eq!(
        format::Locale::EN_US
            .format("021.0n")
            .expect("d3 test")
            .format(0.0),
        "0,000,000,000,000,000"
    ); // d3-format/test/format-type-n-test.js:26:3
    assert_eq!(
        format::Locale::EN_US
            .format("013.8n")
            .expect("d3 test")
            .format(-42000000.0),
        "−0,042,000,000"
    ); // d3-format/test/format-type-n-test.js:27:3
}

// d3-format/test/format-type-none-test.js: format(".[precision]") uses significant precision and trims insignificant zeros
#[test]
fn format_precision_uses_significant_precision_and_trims_insignificant_zeros() {
    assert_eq!(
        format::Locale::EN_US
            .format(".1")
            .expect("d3 test")
            .format(4.9),
        "5"
    ); // d3-format/test/format-type-none-test.js:5:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1")
            .expect("d3 test")
            .format(0.49),
        "0.5"
    ); // d3-format/test/format-type-none-test.js:6:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2")
            .expect("d3 test")
            .format(4.9),
        "4.9"
    ); // d3-format/test/format-type-none-test.js:7:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2")
            .expect("d3 test")
            .format(0.49),
        "0.49"
    ); // d3-format/test/format-type-none-test.js:8:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2")
            .expect("d3 test")
            .format(0.449),
        "0.45"
    ); // d3-format/test/format-type-none-test.js:9:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3")
            .expect("d3 test")
            .format(4.9),
        "4.9"
    ); // d3-format/test/format-type-none-test.js:10:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3")
            .expect("d3 test")
            .format(0.49),
        "0.49"
    ); // d3-format/test/format-type-none-test.js:11:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3")
            .expect("d3 test")
            .format(0.449),
        "0.449"
    ); // d3-format/test/format-type-none-test.js:12:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3")
            .expect("d3 test")
            .format(0.4449),
        "0.445"
    ); // d3-format/test/format-type-none-test.js:13:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(0.444449),
        "0.44445"
    ); // d3-format/test/format-type-none-test.js:14:3
}

// d3-format/test/format-type-none-test.js: format(".[precision]") does not trim significant zeros
#[test]
fn format_precision_does_not_trim_significant_zeros() {
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(10.0),
        "10"
    ); // d3-format/test/format-type-none-test.js:18:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(100.0),
        "100"
    ); // d3-format/test/format-type-none-test.js:19:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(1000.0),
        "1000"
    ); // d3-format/test/format-type-none-test.js:20:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(21010.0),
        "21010"
    ); // d3-format/test/format-type-none-test.js:21:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(1.10001),
        "1.1"
    ); // d3-format/test/format-type-none-test.js:22:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(1.10001e6),
        "1.1e+6"
    ); // d3-format/test/format-type-none-test.js:23:3
    assert_eq!(
        format::Locale::EN_US
            .format(".6")
            .expect("d3 test")
            .format(1.10001),
        "1.10001"
    ); // d3-format/test/format-type-none-test.js:24:3
    assert_eq!(
        format::Locale::EN_US
            .format(".6")
            .expect("d3 test")
            .format(1.10001e6),
        "1.10001e+6"
    ); // d3-format/test/format-type-none-test.js:25:3
}

// d3-format/test/format-type-none-test.js: format(".[precision]") also trims the decimal point if there are only insignificant zeros
#[test]
fn format_precision_also_trims_the_decimal_point_if_there_are_only_insignificant_zeros() {
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(1.00001),
        "1"
    ); // d3-format/test/format-type-none-test.js:29:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5")
            .expect("d3 test")
            .format(1.00001e6),
        "1e+6"
    ); // d3-format/test/format-type-none-test.js:30:3
    assert_eq!(
        format::Locale::EN_US
            .format(".6")
            .expect("d3 test")
            .format(1.00001),
        "1.00001"
    ); // d3-format/test/format-type-none-test.js:31:3
    assert_eq!(
        format::Locale::EN_US
            .format(".6")
            .expect("d3 test")
            .format(1.00001e6),
        "1.00001e+6"
    ); // d3-format/test/format-type-none-test.js:32:3
}

// d3-format/test/format-type-none-test.js: format("$") can output a currency
#[test]
fn format_can_output_a_currency() {
    let f = format::Locale::EN_US.format("$").expect("d3 test");
    assert_eq!(f.format(0.0), "$0"); // d3-format/test/format-type-none-test.js:37:3
    assert_eq!(f.format(0.042), "$0.042"); // d3-format/test/format-type-none-test.js:38:3
    assert_eq!(f.format(0.42), "$0.42"); // d3-format/test/format-type-none-test.js:39:3
    assert_eq!(f.format(4.2), "$4.2"); // d3-format/test/format-type-none-test.js:40:3
    assert_eq!(f.format(-0.042), "−$0.042"); // d3-format/test/format-type-none-test.js:41:3
    assert_eq!(f.format(-0.42), "−$0.42"); // d3-format/test/format-type-none-test.js:42:3
    assert_eq!(f.format(-4.2), "−$4.2"); // d3-format/test/format-type-none-test.js:43:3
}

// d3-format/test/format-type-none-test.js: format("($") can output a currency with parentheses for negative values
#[test]
fn format_can_output_a_currency_with_parentheses_for_negative_values() {
    let f = format::Locale::EN_US.format("($").expect("d3 test");
    assert_eq!(f.format(0.0), "$0"); // d3-format/test/format-type-none-test.js:48:3
    assert_eq!(f.format(0.042), "$0.042"); // d3-format/test/format-type-none-test.js:49:3
    assert_eq!(f.format(0.42), "$0.42"); // d3-format/test/format-type-none-test.js:50:3
    assert_eq!(f.format(4.2), "$4.2"); // d3-format/test/format-type-none-test.js:51:3
    assert_eq!(f.format(-0.042), "($0.042)"); // d3-format/test/format-type-none-test.js:52:3
    assert_eq!(f.format(-0.42), "($0.42)"); // d3-format/test/format-type-none-test.js:53:3
    assert_eq!(f.format(-4.2), "($4.2)"); // d3-format/test/format-type-none-test.js:54:3
}

// d3-format/test/format-type-none-test.js: format("") can format negative zero as zero
#[test]
fn format_can_format_negative_zero_as_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format("")
            .expect("d3 test")
            .format(-0.0),
        "0"
    ); // d3-format/test/format-type-none-test.js:58:3
}

// d3-format/test/format-type-none-test.js: format("") can format negative infinity
#[test]
fn format_can_format_negative_infinity() {
    assert_eq!(
        format::Locale::EN_US
            .format("")
            .expect("d3 test")
            .format(-f64::INFINITY),
        "−Infinity"
    ); // d3-format/test/format-type-none-test.js:62:3
}

// d3-format/test/format-type-o-test.js: format("o") octal
#[test]
fn format_o_octal() {
    assert_eq!(
        format::Locale::EN_US
            .format("o")
            .expect("d3 test")
            .format(10.0),
        "12"
    ); // d3-format/test/format-type-o-test.js:5:3
}

// d3-format/test/format-type-o-test.js: format("#o") octal with prefix
#[test]
fn format_o_octal_with_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("#o")
            .expect("d3 test")
            .format(10.0),
        "0o12"
    ); // d3-format/test/format-type-o-test.js:9:3
}

// d3-format/test/format-type-p-test.js: format("p") can output a percentage
#[test]
fn format_p_can_output_a_percentage() {
    let f = format::Locale::EN_US.format("p").expect("d3 test");
    assert_eq!(f.format(0.00123), "0.123000%"); // d3-format/test/format-type-p-test.js:6:3
    assert_eq!(f.format(0.0123), "1.23000%"); // d3-format/test/format-type-p-test.js:7:3
    assert_eq!(f.format(0.123), "12.3000%"); // d3-format/test/format-type-p-test.js:8:3
    assert_eq!(f.format(0.234), "23.4000%"); // d3-format/test/format-type-p-test.js:9:3
    assert_eq!(f.format(1.23), "123.000%"); // d3-format/test/format-type-p-test.js:10:3
    assert_eq!(f.format(-0.00123), "−0.123000%"); // d3-format/test/format-type-p-test.js:11:3
    assert_eq!(f.format(-0.0123), "−1.23000%"); // d3-format/test/format-type-p-test.js:12:3
    assert_eq!(f.format(-0.123), "−12.3000%"); // d3-format/test/format-type-p-test.js:13:3
    assert_eq!(f.format(-1.23), "−123.000%"); // d3-format/test/format-type-p-test.js:14:3
}

// d3-format/test/format-type-p-test.js: format("+p") can output a percentage with rounding and sign
#[test]
fn format_p_can_output_a_percentage_with_rounding_and_sign() {
    let f = format::Locale::EN_US.format("+.2p").expect("d3 test");
    assert_eq!(f.format(0.00123), "+0.12%"); // d3-format/test/format-type-p-test.js:19:3
    assert_eq!(f.format(0.0123), "+1.2%"); // d3-format/test/format-type-p-test.js:20:3
    assert_eq!(f.format(0.123), "+12%"); // d3-format/test/format-type-p-test.js:21:3
    assert_eq!(f.format(1.23), "+120%"); // d3-format/test/format-type-p-test.js:22:3
    assert_eq!(f.format(-0.00123), "−0.12%"); // d3-format/test/format-type-p-test.js:23:3
    assert_eq!(f.format(-0.0123), "−1.2%"); // d3-format/test/format-type-p-test.js:24:3
    assert_eq!(f.format(-0.123), "−12%"); // d3-format/test/format-type-p-test.js:25:3
    assert_eq!(f.format(-1.23), "−120%"); // d3-format/test/format-type-p-test.js:26:3
}

// d3-format/test/format-type-r-test.js: format("r") can round to significant digits
#[test]
fn format_r_can_round_to_significant_digits() {
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.049),
        "0.05"
    ); // d3-format/test/format-type-r-test.js:5:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(-0.049),
        "−0.05"
    ); // d3-format/test/format-type-r-test.js:6:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.49),
        "0.5"
    ); // d3-format/test/format-type-r-test.js:7:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(-0.49),
        "−0.5"
    ); // d3-format/test/format-type-r-test.js:8:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2r")
            .expect("d3 test")
            .format(0.449),
        "0.45"
    ); // d3-format/test/format-type-r-test.js:9:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3r")
            .expect("d3 test")
            .format(0.4449),
        "0.445"
    ); // d3-format/test/format-type-r-test.js:10:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3r")
            .expect("d3 test")
            .format(1.00),
        "1.00"
    ); // d3-format/test/format-type-r-test.js:11:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3r")
            .expect("d3 test")
            .format(0.9995),
        "1.00"
    ); // d3-format/test/format-type-r-test.js:12:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5r")
            .expect("d3 test")
            .format(0.444449),
        "0.44445"
    ); // d3-format/test/format-type-r-test.js:13:3
    assert_eq!(
        format::Locale::EN_US
            .format("r")
            .expect("d3 test")
            .format(123.45),
        "123.450"
    ); // d3-format/test/format-type-r-test.js:14:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(123.45),
        "100"
    ); // d3-format/test/format-type-r-test.js:15:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2r")
            .expect("d3 test")
            .format(123.45),
        "120"
    ); // d3-format/test/format-type-r-test.js:16:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3r")
            .expect("d3 test")
            .format(123.45),
        "123"
    ); // d3-format/test/format-type-r-test.js:17:3
    assert_eq!(
        format::Locale::EN_US
            .format(".4r")
            .expect("d3 test")
            .format(123.45),
        "123.5"
    ); // d3-format/test/format-type-r-test.js:18:3
    assert_eq!(
        format::Locale::EN_US
            .format(".5r")
            .expect("d3 test")
            .format(123.45),
        "123.45"
    ); // d3-format/test/format-type-r-test.js:19:3
    assert_eq!(
        format::Locale::EN_US
            .format(".6r")
            .expect("d3 test")
            .format(123.45),
        "123.450"
    ); // d3-format/test/format-type-r-test.js:20:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.9),
        "0.9"
    ); // d3-format/test/format-type-r-test.js:21:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.09),
        "0.09"
    ); // d3-format/test/format-type-r-test.js:22:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.949),
        "0.9"
    ); // d3-format/test/format-type-r-test.js:23:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.0949),
        "0.09"
    ); // d3-format/test/format-type-r-test.js:24:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.0000000129),
        "0.00000001"
    ); // d3-format/test/format-type-r-test.js:25:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2r")
            .expect("d3 test")
            .format(0.0000000129),
        "0.000000013"
    ); // d3-format/test/format-type-r-test.js:26:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2r")
            .expect("d3 test")
            .format(0.00000000129),
        "0.0000000013"
    ); // d3-format/test/format-type-r-test.js:27:3
    assert_eq!(
        format::Locale::EN_US
            .format(".3r")
            .expect("d3 test")
            .format(0.00000000129),
        "0.00000000129"
    ); // d3-format/test/format-type-r-test.js:28:3
    assert_eq!(
        format::Locale::EN_US
            .format(".4r")
            .expect("d3 test")
            .format(0.00000000129),
        "0.000000001290"
    ); // d3-format/test/format-type-r-test.js:29:3
    assert_eq!(
        format::Locale::EN_US
            .format(".10r")
            .expect("d3 test")
            .format(0.9999999999),
        "0.9999999999"
    ); // d3-format/test/format-type-r-test.js:30:3
    assert_eq!(
        format::Locale::EN_US
            .format(".15r")
            .expect("d3 test")
            .format(0.999999999999999),
        "0.999999999999999"
    ); // d3-format/test/format-type-r-test.js:31:3
}

// d3-format/test/format-type-r-test.js: format("r") can round zero
#[test]
fn format_r_can_round_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format(".2r")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-r-test.js:35:3
    assert_eq!(
        format::Locale::EN_US
            .format(".1r")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-r-test.js:36:3
    assert_eq!(
        format::Locale::EN_US
            .format("r")
            .expect("d3 test")
            .format(0.0),
        "0"
    ); // d3-format/test/format-type-r-test.js:37:3
}

// d3-format/test/format-type-r-test.js: format("r") can round very small numbers
#[test]
fn format_r_can_round_very_small_numbers() {
    let f = format::Locale::EN_US.format(".2r").expect("d3 test");
    assert_eq!(f.format(1e-22), "0.00000000000000000000010"); // d3-format/test/format-type-r-test.js:42:3
}

// d3-format/test/format-type-s-test.js: format("s") outputs SI-prefix notation with default precision 6
#[test]
fn format_s_outputs_si_prefix_notation_with_default_precision_6() {
    let f = format::Locale::EN_US.format("s").expect("d3 test");
    assert_eq!(f.format(0.0), "0.00000"); // d3-format/test/format-type-s-test.js:6:3
    assert_eq!(f.format(1.0), "1.00000"); // d3-format/test/format-type-s-test.js:7:3
    assert_eq!(f.format(10.0), "10.0000"); // d3-format/test/format-type-s-test.js:8:3
    assert_eq!(f.format(100.0), "100.000"); // d3-format/test/format-type-s-test.js:9:3
    assert_eq!(f.format(999.5), "999.500"); // d3-format/test/format-type-s-test.js:10:3
    assert_eq!(f.format(999500.0), "999.500k"); // d3-format/test/format-type-s-test.js:11:3
    assert_eq!(f.format(1000.0), "1.00000k"); // d3-format/test/format-type-s-test.js:12:3
    assert_eq!(f.format(100.0), "100.000"); // d3-format/test/format-type-s-test.js:13:3
    assert_eq!(f.format(1400.0), "1.40000k"); // d3-format/test/format-type-s-test.js:14:3
    assert_eq!(f.format(1500.5), "1.50050k"); // d3-format/test/format-type-s-test.js:15:3
    assert_eq!(f.format(0.00001), "10.0000µ"); // d3-format/test/format-type-s-test.js:16:3
    assert_eq!(f.format(0.000001), "1.00000µ"); // d3-format/test/format-type-s-test.js:17:3
}

// d3-format/test/format-type-s-test.js: format("s") does not get confused by NaN, Infinity, etc.
#[test]
fn format_s_does_not_get_confused_by_nan_infinity_etc() {
    let f = format::Locale::EN_US.format("s").expect("d3 test");
    assert_eq!(f.format(999500.0), "999.500k"); // d3-format/test/format-type-s-test.js:22:3
    assert_eq!(f.format(f64::INFINITY), "Infinity"); // d3-format/test/format-type-s-test.js:23:3
    assert_eq!(f.format(f64::NAN), "NaN"); // d3-format/test/format-type-s-test.js:24:3
}

// d3-format/test/format-type-s-test.js: format("[.precision]s") outputs SI-prefix notation with precision significant digits
#[test]
fn format_precision_s_outputs_si_prefix_notation_with_precision_significant_digits() {
    let f1 = format::Locale::EN_US.format(".3s").expect("d3 test");
    assert_eq!(f1.format(0.0), "0.00"); // d3-format/test/format-type-s-test.js:29:3
    assert_eq!(f1.format(1.0), "1.00"); // d3-format/test/format-type-s-test.js:30:3
    assert_eq!(f1.format(10.0), "10.0"); // d3-format/test/format-type-s-test.js:31:3
    assert_eq!(f1.format(100.0), "100"); // d3-format/test/format-type-s-test.js:32:3
    assert_eq!(f1.format(999.5), "1.00k"); // d3-format/test/format-type-s-test.js:33:3
    assert_eq!(f1.format(999500.0), "1.00M"); // d3-format/test/format-type-s-test.js:34:3
    assert_eq!(f1.format(1000.0), "1.00k"); // d3-format/test/format-type-s-test.js:35:3
    assert_eq!(f1.format(1500.5), "1.50k"); // d3-format/test/format-type-s-test.js:36:3
    assert_eq!(f1.format(145500000.0), "146M"); // d3-format/test/format-type-s-test.js:37:3
    assert_eq!(f1.format(145999999.99999347), "146M"); // d3-format/test/format-type-s-test.js:38:3
    assert_eq!(f1.format(1e26), "100Y"); // d3-format/test/format-type-s-test.js:39:3
    assert_eq!(f1.format(0.000001), "1.00µ"); // d3-format/test/format-type-s-test.js:40:3
    assert_eq!(f1.format(0.009995), "10.0m"); // d3-format/test/format-type-s-test.js:41:3
    let f2 = format::Locale::EN_US.format(".4s").expect("d3 test");
    assert_eq!(f2.format(999.5), "999.5"); // d3-format/test/format-type-s-test.js:43:3
    assert_eq!(f2.format(999500.0), "999.5k"); // d3-format/test/format-type-s-test.js:44:3
    assert_eq!(f2.format(0.009995), "9.995m"); // d3-format/test/format-type-s-test.js:45:3
}

// d3-format/test/format-type-s-test.js: format("s") formats numbers smaller than 1e-24 with yocto
#[test]
fn format_s_formats_numbers_smaller_than_1e_24_with_yocto() {
    let f = format::Locale::EN_US.format(".8s").expect("d3 test");
    assert_eq!(f.format(1.29e-30), "0.0000013y"); // d3-format/test/format-type-s-test.js:50:3
    assert_eq!(f.format(1.29e-29), "0.0000129y"); // d3-format/test/format-type-s-test.js:51:3
    assert_eq!(f.format(1.29e-28), "0.0001290y"); // d3-format/test/format-type-s-test.js:52:3
    assert_eq!(f.format(1.29e-27), "0.0012900y"); // d3-format/test/format-type-s-test.js:53:3
    assert_eq!(f.format(1.29e-26), "0.0129000y"); // d3-format/test/format-type-s-test.js:54:3
    assert_eq!(f.format(1.29e-25), "0.1290000y"); // d3-format/test/format-type-s-test.js:55:3
    assert_eq!(f.format(1.29e-24), "1.2900000y"); // d3-format/test/format-type-s-test.js:56:3
    assert_eq!(f.format(1.29e-23), "12.900000y"); // d3-format/test/format-type-s-test.js:57:3
    assert_eq!(f.format(1.29e-22), "129.00000y"); // d3-format/test/format-type-s-test.js:58:3
    assert_eq!(f.format(1.29e-21), "1.2900000z"); // d3-format/test/format-type-s-test.js:59:3
    assert_eq!(f.format(-1.29e-30), "−0.0000013y"); // d3-format/test/format-type-s-test.js:60:3
    assert_eq!(f.format(-1.29e-29), "−0.0000129y"); // d3-format/test/format-type-s-test.js:61:3
    assert_eq!(f.format(-1.29e-28), "−0.0001290y"); // d3-format/test/format-type-s-test.js:62:3
    assert_eq!(f.format(-1.29e-27), "−0.0012900y"); // d3-format/test/format-type-s-test.js:63:3
    assert_eq!(f.format(-1.29e-26), "−0.0129000y"); // d3-format/test/format-type-s-test.js:64:3
    assert_eq!(f.format(-1.29e-25), "−0.1290000y"); // d3-format/test/format-type-s-test.js:65:3
    assert_eq!(f.format(-1.29e-24), "−1.2900000y"); // d3-format/test/format-type-s-test.js:66:3
    assert_eq!(f.format(-1.29e-23), "−12.900000y"); // d3-format/test/format-type-s-test.js:67:3
    assert_eq!(f.format(-1.29e-22), "−129.00000y"); // d3-format/test/format-type-s-test.js:68:3
    assert_eq!(f.format(-1.29e-21), "−1.2900000z"); // d3-format/test/format-type-s-test.js:69:3
}

// d3-format/test/format-type-s-test.js: format("s") formats numbers larger than 1e24 with yotta
#[test]
fn format_s_formats_numbers_larger_than_1e24_with_yotta() {
    let f = format::Locale::EN_US.format(".8s").expect("d3 test");
    assert_eq!(f.format(1.23e+21), "1.2300000Z"); // d3-format/test/format-type-s-test.js:74:3
    assert_eq!(f.format(1.23e+22), "12.300000Z"); // d3-format/test/format-type-s-test.js:75:3
    assert_eq!(f.format(1.23e+23), "123.00000Z"); // d3-format/test/format-type-s-test.js:76:3
    assert_eq!(f.format(1.23e+24), "1.2300000Y"); // d3-format/test/format-type-s-test.js:77:3
    assert_eq!(f.format(1.23e+25), "12.300000Y"); // d3-format/test/format-type-s-test.js:78:3
    assert_eq!(f.format(1.23e+26), "123.00000Y"); // d3-format/test/format-type-s-test.js:79:3
    assert_eq!(f.format(1.23e+27), "1230.0000Y"); // d3-format/test/format-type-s-test.js:80:3
    assert_eq!(f.format(1.23e+28), "12300.000Y"); // d3-format/test/format-type-s-test.js:81:3
    assert_eq!(f.format(1.23e+29), "123000.00Y"); // d3-format/test/format-type-s-test.js:82:3
    assert_eq!(f.format(1.23e+30), "1230000.0Y"); // d3-format/test/format-type-s-test.js:83:3
    assert_eq!(f.format(-1.23e+21), "−1.2300000Z"); // d3-format/test/format-type-s-test.js:84:3
    assert_eq!(f.format(-1.23e+22), "−12.300000Z"); // d3-format/test/format-type-s-test.js:85:3
    assert_eq!(f.format(-1.23e+23), "−123.00000Z"); // d3-format/test/format-type-s-test.js:86:3
    assert_eq!(f.format(-1.23e+24), "−1.2300000Y"); // d3-format/test/format-type-s-test.js:87:3
    assert_eq!(f.format(-1.23e+25), "−12.300000Y"); // d3-format/test/format-type-s-test.js:88:3
    assert_eq!(f.format(-1.23e+26), "−123.00000Y"); // d3-format/test/format-type-s-test.js:89:3
    assert_eq!(f.format(-1.23e+27), "−1230.0000Y"); // d3-format/test/format-type-s-test.js:90:3
    assert_eq!(f.format(-1.23e+28), "−12300.000Y"); // d3-format/test/format-type-s-test.js:91:3
    assert_eq!(f.format(-1.23e+29), "−123000.00Y"); // d3-format/test/format-type-s-test.js:92:3
    assert_eq!(f.format(-1.23e+30), "−1230000.0Y"); // d3-format/test/format-type-s-test.js:93:3
}

// d3-format/test/format-type-s-test.js: format("$s") outputs SI-prefix notation with a currency symbol
#[test]
fn format_s_outputs_si_prefix_notation_with_a_currency_symbol() {
    let f1 = format::Locale::EN_US.format("$.2s").expect("d3 test");
    assert_eq!(f1.format(0.0), "$0.0"); // d3-format/test/format-type-s-test.js:98:3
    assert_eq!(f1.format(2.5e5), "$250k"); // d3-format/test/format-type-s-test.js:99:3
    assert_eq!(f1.format(-2.5e8), "−$250M"); // d3-format/test/format-type-s-test.js:100:3
    assert_eq!(f1.format(2.5e11), "$250G"); // d3-format/test/format-type-s-test.js:101:3
    let f2 = format::Locale::EN_US.format("$.3s").expect("d3 test");
    assert_eq!(f2.format(0.0), "$0.00"); // d3-format/test/format-type-s-test.js:103:3
    assert_eq!(f2.format(1.0), "$1.00"); // d3-format/test/format-type-s-test.js:104:3
    assert_eq!(f2.format(10.0), "$10.0"); // d3-format/test/format-type-s-test.js:105:3
    assert_eq!(f2.format(100.0), "$100"); // d3-format/test/format-type-s-test.js:106:3
    assert_eq!(f2.format(999.5), "$1.00k"); // d3-format/test/format-type-s-test.js:107:3
    assert_eq!(f2.format(999500.0), "$1.00M"); // d3-format/test/format-type-s-test.js:108:3
    assert_eq!(f2.format(1000.0), "$1.00k"); // d3-format/test/format-type-s-test.js:109:3
    assert_eq!(f2.format(1500.5), "$1.50k"); // d3-format/test/format-type-s-test.js:110:3
    assert_eq!(f2.format(145500000.0), "$146M"); // d3-format/test/format-type-s-test.js:111:3
    assert_eq!(f2.format(145999999.9999347), "$146M"); // d3-format/test/format-type-s-test.js:112:3
    assert_eq!(f2.format(1e26), "$100Y"); // d3-format/test/format-type-s-test.js:113:3
    assert_eq!(f2.format(0.000001), "$1.00µ"); // d3-format/test/format-type-s-test.js:114:3
    assert_eq!(f2.format(0.009995), "$10.0m"); // d3-format/test/format-type-s-test.js:115:3
    let f3 = format::Locale::EN_US.format("$.4s").expect("d3 test");
    assert_eq!(f3.format(999.5), "$999.5"); // d3-format/test/format-type-s-test.js:117:3
    assert_eq!(f3.format(999500.0), "$999.5k"); // d3-format/test/format-type-s-test.js:118:3
    assert_eq!(f3.format(0.009995), "$9.995m"); // d3-format/test/format-type-s-test.js:119:3
}

// d3-format/test/format-type-s-test.js: format("s") SI-prefix notation precision is consistent for small and large numbers
#[test]
fn format_s_si_prefix_notation_precision_is_consistent_for_small_and_large_numbers() {
    let f1 = format::Locale::EN_US.format(".0s").expect("d3 test");
    assert_eq!(f1.format(1e-5), "10µ"); // d3-format/test/format-type-s-test.js:124:3
    assert_eq!(f1.format(1e-4), "100µ"); // d3-format/test/format-type-s-test.js:125:3
    assert_eq!(f1.format(1e-3), "1m"); // d3-format/test/format-type-s-test.js:126:3
    assert_eq!(f1.format(1e-2), "10m"); // d3-format/test/format-type-s-test.js:127:3
    assert_eq!(f1.format(1e-1), "100m"); // d3-format/test/format-type-s-test.js:128:3
    assert_eq!(f1.format(1e+0), "1"); // d3-format/test/format-type-s-test.js:129:3
    assert_eq!(f1.format(1e+1), "10"); // d3-format/test/format-type-s-test.js:130:3
    assert_eq!(f1.format(1e+2), "100"); // d3-format/test/format-type-s-test.js:131:3
    assert_eq!(f1.format(1e+3), "1k"); // d3-format/test/format-type-s-test.js:132:3
    assert_eq!(f1.format(1e+4), "10k"); // d3-format/test/format-type-s-test.js:133:3
    assert_eq!(f1.format(1e+5), "100k"); // d3-format/test/format-type-s-test.js:134:3
    let f2 = format::Locale::EN_US.format(".4s").expect("d3 test");
    assert_eq!(f2.format(1e-5), "10.00µ"); // d3-format/test/format-type-s-test.js:136:3
    assert_eq!(f2.format(1e-4), "100.0µ"); // d3-format/test/format-type-s-test.js:137:3
    assert_eq!(f2.format(1e-3), "1.000m"); // d3-format/test/format-type-s-test.js:138:3
    assert_eq!(f2.format(1e-2), "10.00m"); // d3-format/test/format-type-s-test.js:139:3
    assert_eq!(f2.format(1e-1), "100.0m"); // d3-format/test/format-type-s-test.js:140:3
    assert_eq!(f2.format(1e+0), "1.000"); // d3-format/test/format-type-s-test.js:141:3
    assert_eq!(f2.format(1e+1), "10.00"); // d3-format/test/format-type-s-test.js:142:3
    assert_eq!(f2.format(1e+2), "100.0"); // d3-format/test/format-type-s-test.js:143:3
    assert_eq!(f2.format(1e+3), "1.000k"); // d3-format/test/format-type-s-test.js:144:3
    assert_eq!(f2.format(1e+4), "10.00k"); // d3-format/test/format-type-s-test.js:145:3
    assert_eq!(f2.format(1e+5), "100.0k"); // d3-format/test/format-type-s-test.js:146:3
}

// d3-format/test/format-type-s-test.js: format("0[width],s") will group thousands due to zero fill
#[test]
fn format_0_width_s_will_group_thousands_due_to_zero_fill() {
    let f = format::Locale::EN_US.format("020,s").expect("d3 test");
    assert_eq!(f.format(42.0), "000,000,000,042.0000"); // d3-format/test/format-type-s-test.js:151:3
    assert_eq!(f.format(42e12), "00,000,000,042.0000T"); // d3-format/test/format-type-s-test.js:152:3
}

// d3-format/test/format-type-s-test.js: format(",s") will group thousands for very large numbers
#[test]
fn format_s_will_group_thousands_for_very_large_numbers() {
    let f = format::Locale::EN_US.format(",s").expect("d3 test");
    assert_eq!(f.format(42e30), "42,000,000Y"); // d3-format/test/format-type-s-test.js:157:3
}

// d3-format/test/format-type-x-test.js: format("x") returns the expected hexadecimal (lowercase) string
#[test]
fn format_x_returns_the_expected_hexadecimal_lowercase_string() {
    assert_eq!(
        format::Locale::EN_US
            .format("x")
            .expect("d3 test")
            .format(3735928559.0),
        "deadbeef"
    ); // d3-format/test/format-type-x-test.js:5:3
}

// d3-format/test/format-type-x-test.js: format("#x") returns the expected hexadecimal (lowercase) string with prefix
#[test]
fn format_x_returns_the_expected_hexadecimal_lowercase_string_with_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("#x")
            .expect("d3 test")
            .format(3735928559.0),
        "0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:9:3
}

// d3-format/test/format-type-x-test.js: format(",x") groups thousands
#[test]
fn format_x_groups_thousands() {
    assert_eq!(
        format::Locale::EN_US
            .format(",x")
            .expect("d3 test")
            .format(3735928559.0),
        "de,adb,eef"
    ); // d3-format/test/format-type-x-test.js:13:3
}

// d3-format/test/format-type-x-test.js: format(",x") groups thousands
#[test]
fn format_x_groups_thousands_2() {
    assert_eq!(
        format::Locale::EN_US
            .format(",x")
            .expect("d3 test")
            .format(3735928559.0),
        "de,adb,eef"
    ); // d3-format/test/format-type-x-test.js:17:3
}

// d3-format/test/format-type-x-test.js: format("#,x") does not group the prefix
#[test]
fn format_x_does_not_group_the_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("#,x")
            .expect("d3 test")
            .format(46685601519.0),
        "0xade,adb,eef"
    ); // d3-format/test/format-type-x-test.js:21:3
}

// d3-format/test/format-type-x-test.js: format("+#x") puts the sign before the prefix
#[test]
fn format_x_puts_the_sign_before_the_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("+#x")
            .expect("d3 test")
            .format(3735928559.0),
        "+0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:25:3
    assert_eq!(
        format::Locale::EN_US
            .format("+#x")
            .expect("d3 test")
            .format(-3735928559.0),
        "−0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:26:3
    assert_eq!(
        format::Locale::EN_US
            .format(" #x")
            .expect("d3 test")
            .format(3735928559.0),
        " 0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:27:3
    assert_eq!(
        format::Locale::EN_US
            .format(" #x")
            .expect("d3 test")
            .format(-3735928559.0),
        "−0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:28:3
}

// d3-format/test/format-type-x-test.js: format("$,x") formats hexadecimal currency
#[test]
fn format_x_formats_hexadecimal_currency() {
    assert_eq!(
        format::Locale::EN_US
            .format("$,x")
            .expect("d3 test")
            .format(3735928559.0),
        "$de,adb,eef"
    ); // d3-format/test/format-type-x-test.js:32:3
}

// d3-format/test/format-type-x-test.js: format("[.precision]x") always has precision zero
#[test]
fn format_precision_x_always_has_precision_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format(".2x")
            .expect("d3 test")
            .format(3735928559.0),
        "deadbeef"
    ); // d3-format/test/format-type-x-test.js:36:3
    assert_eq!(
        format::Locale::EN_US
            .format(".2x")
            .expect("d3 test")
            .format(-4.2),
        "−4"
    ); // d3-format/test/format-type-x-test.js:37:3
}

// d3-format/test/format-type-x-test.js: format("x") rounds non-integers
#[test]
fn format_x_rounds_non_integers() {
    assert_eq!(
        format::Locale::EN_US
            .format("x")
            .expect("d3 test")
            .format(2.4),
        "2"
    ); // d3-format/test/format-type-x-test.js:41:3
}

// d3-format/test/format-type-x-test.js: format("x") can format negative zero as zero
#[test]
fn format_x_can_format_negative_zero_as_zero() {
    assert_eq!(
        format::Locale::EN_US
            .format("x")
            .expect("d3 test")
            .format(-0.0),
        "0"
    ); // d3-format/test/format-type-x-test.js:45:3
    assert_eq!(
        format::Locale::EN_US
            .format("x")
            .expect("d3 test")
            .format(-1e-12),
        "0"
    ); // d3-format/test/format-type-x-test.js:46:3
}

// d3-format/test/format-type-x-test.js: format("x") does not consider -0xeee to be positive
#[test]
fn format_x_does_not_consider_0xeee_to_be_positive() {
    assert_eq!(
        format::Locale::EN_US
            .format("x")
            .expect("d3 test")
            .format(-3822.0),
        "−eee"
    ); // d3-format/test/format-type-x-test.js:50:3
}

// d3-format/test/format-type-x-test.js: format("X") returns the expected hexadecimal (uppercase) string
#[test]
fn format_x_returns_the_expected_hexadecimal_uppercase_string() {
    assert_eq!(
        format::Locale::EN_US
            .format("X")
            .expect("d3 test")
            .format(3735928559.0),
        "DEADBEEF"
    ); // d3-format/test/format-type-x-test.js:54:3
}

// d3-format/test/format-type-x-test.js: format("#X") returns the expected hexadecimal (uppercase) string with prefix
#[test]
fn format_x_returns_the_expected_hexadecimal_uppercase_string_with_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("#X")
            .expect("d3 test")
            .format(3735928559.0),
        "0xDEADBEEF"
    ); // d3-format/test/format-type-x-test.js:58:3
}

// d3-format/test/format-type-x-test.js: format("X") can format negative zero as zero
#[test]
fn format_x_can_format_negative_zero_as_zero_2() {
    assert_eq!(
        format::Locale::EN_US
            .format("X")
            .expect("d3 test")
            .format(-0.0),
        "0"
    ); // d3-format/test/format-type-x-test.js:62:3
    assert_eq!(
        format::Locale::EN_US
            .format("X")
            .expect("d3 test")
            .format(-1e-12),
        "0"
    ); // d3-format/test/format-type-x-test.js:63:3
}

// d3-format/test/format-type-x-test.js: format("X") does not consider -0xeee to be positive
#[test]
fn format_x_does_not_consider_0xeee_to_be_positive_2() {
    assert_eq!(
        format::Locale::EN_US
            .format("X")
            .expect("d3 test")
            .format(-3822.0),
        "−EEE"
    ); // d3-format/test/format-type-x-test.js:67:3
}

// d3-format/test/format-type-x-test.js: format("#[width]x") considers the prefix
#[test]
fn format_width_x_considers_the_prefix() {
    assert_eq!(
        format::Locale::EN_US
            .format("20x")
            .expect("d3 test")
            .format(3735928559.0),
        "            deadbeef"
    ); // d3-format/test/format-type-x-test.js:71:3
    assert_eq!(
        format::Locale::EN_US
            .format("#20x")
            .expect("d3 test")
            .format(3735928559.0),
        "          0xdeadbeef"
    ); // d3-format/test/format-type-x-test.js:72:3
    assert_eq!(
        format::Locale::EN_US
            .format("020x")
            .expect("d3 test")
            .format(3735928559.0),
        "000000000000deadbeef"
    ); // d3-format/test/format-type-x-test.js:73:3
    assert_eq!(
        format::Locale::EN_US
            .format("#020x")
            .expect("d3 test")
            .format(3735928559.0),
        "0x0000000000deadbeef"
    ); // d3-format/test/format-type-x-test.js:74:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(",.0s", value)(number) formats with the SI prefix appropriate to the specified value
#[test]
fn formatprefix_0s_value_number_formats_with_the_si_prefix_appropriate_to_the_specified_value() {
    assert_eq!(
        format::Locale::EN_US
            .format_prefix(",.0s", 1e-6)
            .expect("d3 test")
            .format(0.00042),
        "420µ"
    ); // d3-format/test/formatPrefix-test.js:5:3
    assert_eq!(
        format::Locale::EN_US
            .format_prefix(",.0s", 1e-6)
            .expect("d3 test")
            .format(0.0042),
        "4,200µ"
    ); // d3-format/test/formatPrefix-test.js:6:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(",.3s", value)(number) formats with the SI prefix appropriate to the specified value
#[test]
fn formatprefix_3s_value_number_formats_with_the_si_prefix_appropriate_to_the_specified_value() {
    assert_eq!(
        format::Locale::EN_US
            .format_prefix(",.3s", 1e-3)
            .expect("d3 test")
            .format(0.00042),
        "0.420m"
    ); // d3-format/test/formatPrefix-test.js:10:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(",.0s", value)(number) uses yocto for very small reference values
#[test]
fn formatprefix_0s_value_number_uses_yocto_for_very_small_reference_values() {
    assert_eq!(
        format::Locale::EN_US
            .format_prefix(",.0s", 1e-27)
            .expect("d3 test")
            .format(1e-24),
        "1y"
    ); // d3-format/test/formatPrefix-test.js:14:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(",.0s", value)(number) uses yotta for very small reference values
#[test]
fn formatprefix_0s_value_number_uses_yotta_for_very_small_reference_values() {
    assert_eq!(
        format::Locale::EN_US
            .format_prefix(",.0s", 1e27)
            .expect("d3 test")
            .format(1e24),
        "1Y"
    ); // d3-format/test/formatPrefix-test.js:18:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(" $12,.1s", value)(number) formats with the specified SI prefix
#[test]
fn formatprefix_12_1s_value_number_formats_with_the_specified_si_prefix() {
    let f = format::Locale::EN_US
        .format_prefix(" $12,.1s", 1e6)
        .expect("d3 test");
    assert_eq!(f.format(-42e6), "     −$42.0M"); // d3-format/test/formatPrefix-test.js:24:3
    assert_eq!(f.format(4.2e6), "       $4.2M"); // d3-format/test/formatPrefix-test.js:25:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix(" $12,.1s", value)(number) matches format(" $12,.2s")(number) when the units are the same
#[test]
fn formatprefix_12_1s_value_number_matches_format_12_2s_number_when_the_units_are_the_same() {
    let fp = format::Locale::EN_US
        .format_prefix(" $12,.1s", 1e6)
        .expect("d3 test");
    let f = format::Locale::EN_US.format(" $12,.2s").expect("d3 test");
    assert_eq!(fp.format(4.2e6), "       $4.2M"); // d3-format/test/formatPrefix-test.js:32:3
    assert_eq!(f.format(4.2e6), "       $4.2M"); // d3-format/test/formatPrefix-test.js:33:3
}

// d3-format/test/formatPrefix-test.js: formatPrefix("($~s", value)(number) formats with the SI prefix inside parentheses
#[test]
fn formatprefix_s_value_number_formats_with_the_si_prefix_inside_parentheses() {
    assert_eq!(
        format::Locale::EN_US
            .format_prefix("($~s", 1e3)
            .expect("d3 test")
            .format(1e3),
        "$1k"
    ); // d3-format/test/formatPrefix-test.js:37:3
    assert_eq!(
        format::Locale::EN_US
            .format_prefix("($~s", 1e3)
            .expect("d3 test")
            .format(-1e3),
        "($1k)"
    ); // d3-format/test/formatPrefix-test.js:38:3
}

// d3-format/test/formatSpecifier-test.js: formatSpecifier(specifier) throws an error for invalid formats
#[test]
fn formatspecifier_specifier_throws_an_error_for_invalid_formats() {
    assert!(
        "foo"
            .parse::<format::Specifier>()
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/formatSpecifier-test.js:5:3
    assert!(
        ".-2s"
            .parse::<format::Specifier>()
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/formatSpecifier-test.js:6:3
    assert!(
        ".f".parse::<format::Specifier>()
            .map(|v| v.to_string())
            .is_err()
    ); // d3-format/test/formatSpecifier-test.js:7:3
}

// d3-format/test/formatSpecifier-test.js: formatSpecifier("") has the expected defaults
#[test]
fn formatspecifier_has_the_expected_defaults() {
    let s = "".parse::<format::Specifier>().expect("d3 test");
    assert_eq!(s.get_fill().to_string(), " "); // d3-format/test/formatSpecifier-test.js:17:3
    assert_eq!(s.get_align().to_string(), ">"); // d3-format/test/formatSpecifier-test.js:18:3
    assert_eq!(s.get_sign().to_string(), "-"); // d3-format/test/formatSpecifier-test.js:19:3
    assert_eq!(
        s.get_symbol().map(|s| s.to_string()).unwrap_or_default(),
        ""
    ); // d3-format/test/formatSpecifier-test.js:20:3
    assert!(!s.get_zero()); // d3-format/test/formatSpecifier-test.js:21:3
    assert_eq!(s.get_width(), None); // d3-format/test/formatSpecifier-test.js:22:3
    assert!(!s.get_comma()); // d3-format/test/formatSpecifier-test.js:23:3
    assert_eq!(s.get_precision(), None); // d3-format/test/formatSpecifier-test.js:24:3
    assert!(!s.get_trim()); // d3-format/test/formatSpecifier-test.js:25:3
    assert_eq!(s.get_kind().map(|k| k.to_string()).unwrap_or_default(), ""); // d3-format/test/formatSpecifier-test.js:26:3
}

// d3-format/test/formatSpecifier-test.js: formatSpecifier(specifier) preserves unknown types
#[test]
fn formatspecifier_specifier_preserves_unknown_types() {
    let s = "q".parse::<format::Specifier>().expect("d3 test");
    assert!(!s.get_trim()); // d3-format/test/formatSpecifier-test.js:31:3
    assert_eq!(s.get_kind().map(|k| k.to_string()).unwrap_or_default(), "q"); // d3-format/test/formatSpecifier-test.js:32:3
}

// d3-format/test/formatSpecifier-test.js: formatSpecifier(specifier) preserves shorthand
#[test]
fn formatspecifier_specifier_preserves_shorthand() {
    let s = "".parse::<format::Specifier>().expect("d3 test");
    assert!(!s.get_trim()); // d3-format/test/formatSpecifier-test.js:37:3
    assert_eq!(s.get_kind().map(|k| k.to_string()).unwrap_or_default(), ""); // d3-format/test/formatSpecifier-test.js:38:3
}

// d3-format/test/formatSpecifier-test.js: new FormatSpecifier({}) has the expected defaults
#[test]
fn new_formatspecifier_has_the_expected_defaults() {
    let s = format::Specifier::default();
    assert_eq!(s.get_fill().to_string(), " "); // d3-format/test/formatSpecifier-test.js:68:3
    assert_eq!(s.get_align().to_string(), ">"); // d3-format/test/formatSpecifier-test.js:69:3
    assert_eq!(s.get_sign().to_string(), "-"); // d3-format/test/formatSpecifier-test.js:70:3
    assert_eq!(
        s.get_symbol().map(|s| s.to_string()).unwrap_or_default(),
        ""
    ); // d3-format/test/formatSpecifier-test.js:71:3
    assert!(!s.get_zero()); // d3-format/test/formatSpecifier-test.js:72:3
    assert_eq!(s.get_width(), None); // d3-format/test/formatSpecifier-test.js:73:3
    assert!(!s.get_comma()); // d3-format/test/formatSpecifier-test.js:74:3
    assert_eq!(s.get_precision(), None); // d3-format/test/formatSpecifier-test.js:75:3
    assert!(!s.get_trim()); // d3-format/test/formatSpecifier-test.js:76:3
    assert_eq!(s.get_kind().map(|k| k.to_string()).unwrap_or_default(), ""); // d3-format/test/formatSpecifier-test.js:77:3
}

// d3-format/test/locale-test.js: formatLocale({decimal: decimal}) observes the specified decimal point
#[test]
fn formatlocale_decimal_decimal_observes_the_specified_decimal_point() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "|".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(2.0),
        "002|00"
    ); // d3-format/test/locale-test.js:11:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: "/".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(2.0),
        "002/00"
    ); // d3-format/test/locale-test.js:12:3
}

// d3-format/test/locale-test.js: formatLocale({currency: [prefix, suffix]}) observes the specified currency prefix and suffix
#[test]
fn formatlocale_currency_prefix_suffix_observes_the_specified_currency_prefix_and_suffix() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            currency: ["฿".into(), "".into()],
            ..Default::default()
        })
        .format("$06.2f")
        .expect("d3 test")
        .format(2.0),
        "฿02.00"
    ); // d3-format/test/locale-test.js:16:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            currency: ["".into(), "฿".into()],
            ..Default::default()
        })
        .format("$06.2f")
        .expect("d3 test")
        .format(2.0),
        "02.00฿"
    ); // d3-format/test/locale-test.js:17:3
}

// d3-format/test/locale-test.js: formatLocale({currency: [prefix, suffix]}) places the currency suffix after the SI suffix
#[test]
fn formatlocale_currency_prefix_suffix_places_the_currency_suffix_after_the_si_suffix() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ",".into(),
            currency: ["".into(), " €".into()],
            ..Default::default()
        })
        .format("$.3s")
        .expect("d3 test")
        .format(1.2e9),
        "1,20G €"
    ); // d3-format/test/locale-test.js:21:3
}

// d3-format/test/locale-test.js: formatLocale({grouping: undefined}) does not perform any grouping
#[test]
fn formatlocale_grouping_undefined_does_not_perform_any_grouping() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "000000002.00"
    ); // d3-format/test/locale-test.js:25:3
}

// d3-format/test/locale-test.js: formatLocale({grouping: [sizes…]}) observes the specified group sizes
#[test]
fn formatlocale_grouping_sizes_observes_the_specified_group_sizes() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![3].into(),
            thousands: ",".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "0,000,002.00"
    ); // d3-format/test/locale-test.js:29:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![2].into(),
            thousands: ",".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "0,00,00,02.00"
    ); // d3-format/test/locale-test.js:30:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![2, 3].into(),
            thousands: ",".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "00,000,02.00"
    ); // d3-format/test/locale-test.js:31:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![3, 2, 2, 2, 2, 2, 2].into(),
            thousands: ",".into(),
            ..Default::default()
        })
        .format(",d")
        .expect("d3 test")
        .format(1e12),
        "10,00,00,00,00,000"
    ); // d3-format/test/locale-test.js:32:3
}

// d3-format/test/locale-test.js: formatLocale(…) can format numbers using the Indian numbering system.
#[test]
fn formatlocale_can_format_numbers_using_the_indian_numbering_system() {
    let format = format::Locale::new(format::Definition {
        decimal: ".".into(),
        thousands: ",".into(),
        grouping: vec![3, 2, 2, 2, 2, 2, 2, 2, 2, 2].into(),
        currency: ["₹".into(), "".into()],
        ..Default::default()
    })
    .format(",")
    .expect("d3 test");
    assert_eq!(format.format(10.0), "10"); // d3-format/test/locale-test.js:37:3
    assert_eq!(format.format(100.0), "100"); // d3-format/test/locale-test.js:38:3
    assert_eq!(format.format(1000.0), "1,000"); // d3-format/test/locale-test.js:39:3
    assert_eq!(format.format(10000.0), "10,000"); // d3-format/test/locale-test.js:40:3
    assert_eq!(format.format(100000.0), "1,00,000"); // d3-format/test/locale-test.js:41:3
    assert_eq!(format.format(1000000.0), "10,00,000"); // d3-format/test/locale-test.js:42:3
    assert_eq!(format.format(10000000.0), "1,00,00,000"); // d3-format/test/locale-test.js:43:3
    assert_eq!(format.format(10000000.4543), "1,00,00,000.4543"); // d3-format/test/locale-test.js:44:3
    assert_eq!(format.format(1000.321), "1,000.321"); // d3-format/test/locale-test.js:45:3
    assert_eq!(format.format(10.5), "10.5"); // d3-format/test/locale-test.js:46:3
    assert_eq!(format.format(-10.0), "−10"); // d3-format/test/locale-test.js:47:3
    assert_eq!(format.format(-100.0), "−100"); // d3-format/test/locale-test.js:48:3
    assert_eq!(format.format(-1000.0), "−1,000"); // d3-format/test/locale-test.js:49:3
    assert_eq!(format.format(-10000.0), "−10,000"); // d3-format/test/locale-test.js:50:3
    assert_eq!(format.format(-100000.0), "−1,00,000"); // d3-format/test/locale-test.js:51:3
    assert_eq!(format.format(-1000000.0), "−10,00,000"); // d3-format/test/locale-test.js:52:3
    assert_eq!(format.format(-10000000.0), "−1,00,00,000"); // d3-format/test/locale-test.js:53:3
    assert_eq!(format.format(-10000000.4543), "−1,00,00,000.4543"); // d3-format/test/locale-test.js:54:3
    assert_eq!(format.format(-1000.321), "−1,000.321"); // d3-format/test/locale-test.js:55:3
    assert_eq!(format.format(-10.5), "−10.5"); // d3-format/test/locale-test.js:56:3
}

// d3-format/test/locale-test.js: formatLocale({thousands: separator}) observes the specified group separator
#[test]
fn formatlocale_thousands_separator_observes_the_specified_group_separator() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![3].into(),
            thousands: " ".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "0 000 002.00"
    ); // d3-format/test/locale-test.js:60:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            grouping: vec![3].into(),
            thousands: "/".into(),
            ..Default::default()
        })
        .format("012,.2f")
        .expect("d3 test")
        .format(2.0),
        "0/000/002.00"
    ); // d3-format/test/locale-test.js:61:3
}

// d3-format/test/locale-test.js: formatLocale({percent: percent}) observes the specified percent sign
#[test]
fn formatlocale_percent_percent_observes_the_specified_percent_sign() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            percent: "!".into(),
            ..Default::default()
        })
        .format("06.2%")
        .expect("d3 test")
        .format(2.0),
        "200.00!"
    ); // d3-format/test/locale-test.js:65:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            percent: "﹪".into(),
            ..Default::default()
        })
        .format("06.2%")
        .expect("d3 test")
        .format(2.0),
        "200.00﹪"
    ); // d3-format/test/locale-test.js:66:3
}

// d3-format/test/locale-test.js: formatLocale({minus: minus}) observes the specified minus sign
#[test]
fn formatlocale_minus_minus_observes_the_specified_minus_sign() {
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            minus: "-".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(-2.0),
        "-02.00"
    ); // d3-format/test/locale-test.js:70:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            minus: "−".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(-2.0),
        "−02.00"
    ); // d3-format/test/locale-test.js:71:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            minus: "➖".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(-2.0),
        "➖02.00"
    ); // d3-format/test/locale-test.js:72:3
    assert_eq!(
        format::Locale::new(format::Definition {
            decimal: ".".into(),
            ..Default::default()
        })
        .format("06.2f")
        .expect("d3 test")
        .format(-2.0),
        "−02.00"
    ); // d3-format/test/locale-test.js:73:3
}

// d3-format/test/locale-test.js: locale data is valid
#[test]
fn locale_data_is_valid() {
    let tables: [(&[&str], format::Definition); 59] = [
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " د.إ.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " د.ب.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["‏Fdj ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["د.ج. ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ج.م.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["د.م. ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["Nfk ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["₪ ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " د.ع.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " د.أ.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ف.ج.ق.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " د.ك.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ل.ل.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["د.ل. ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["د.م. ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " أ.م.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ر.ع.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["₪ ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ر.ق.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ر.س.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ج.س.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["‏S ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["£ ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ل.س.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["‏FCFA ".into(), "".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["د.ت. ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "numerals"],
            format::Definition {
                decimal: "٫".into(),
                thousands: "٬".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ر.ى.".into()],
                numerals: Some([
                    "٠".into(),
                    "١".into(),
                    "٢".into(),
                    "٣".into(),
                    "٤".into(),
                    "٥".into(),
                    "٦".into(),
                    "٧".into(),
                    "٨".into(),
                    "٩".into(),
                ]),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " Kč".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " kr".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: "'".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " CHF".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["$".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["£".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["€".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3, 2, 2, 2, 2, 2, 2, 2, 2, 2].into(),
                currency: ["₹".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["$".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "percent"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["Bs ".into(), "".into()],
                percent: " %".into(),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["$".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), "$".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency", "percent"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                percent: " %".into(),
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["₪".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " Ft".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["€".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["".into(), "円".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["₩".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ден.".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["€ ".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), "zł".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["R$".into(), "".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ₽".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: ".".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " €".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " kr".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ",".into(),
                thousands: " ".into(),
                grouping: vec![3].into(),
                currency: ["".into(), " ₴.".into()],
                ..Default::default()
            },
        ),
        (
            &["decimal", "thousands", "grouping", "currency"],
            format::Definition {
                decimal: ".".into(),
                thousands: ",".into(),
                grouping: vec![3].into(),
                currency: ["¥".into(), "".into()],
                ..Default::default()
            },
        ),
    ];
    for (keys, definition) in tables {
        assert!(keys.contains(&"currency")); // d3-format/test/locale-test.js:86:5
        assert!(keys.contains(&"decimal")); // d3-format/test/locale-test.js:87:5
        assert!(keys.contains(&"grouping")); // d3-format/test/locale-test.js:88:5
        assert!(keys.contains(&"thousands")); // d3-format/test/locale-test.js:89:5
        let _ = format::Locale::new(definition);
    }
}

// d3-format/test/precisionFixed-test.js: precisionFixed(number) returns the expected value
#[test]
fn precisionfixed_number_returns_the_expected_value() {
    assert_d3_eq(&format::precision_fixed(8.9), &0.0); // d3-format/test/precisionFixed-test.js:5:3
    assert_d3_eq(&format::precision_fixed(1.1), &0.0); // d3-format/test/precisionFixed-test.js:6:3
    assert_d3_eq(&format::precision_fixed(0.89), &1.0); // d3-format/test/precisionFixed-test.js:7:3
    assert_d3_eq(&format::precision_fixed(0.11), &1.0); // d3-format/test/precisionFixed-test.js:8:3
    assert_d3_eq(&format::precision_fixed(0.089), &2.0); // d3-format/test/precisionFixed-test.js:9:3
    assert_d3_eq(&format::precision_fixed(0.011), &2.0); // d3-format/test/precisionFixed-test.js:10:3
}

// d3-format/test/precisionFixed-test.js: precisionFixed(0) returns NaN
#[test]
fn precisionfixed_0_returns_nan() {
    assert!(format::precision_fixed(0.0).is_none()); // d3-format/test/precisionFixed-test.js:14:3
}

// d3-format/test/precisionFixed-test.js: precisionFixed(NaN) returns NaN
#[test]
fn precisionfixed_nan_returns_nan() {
    assert!(format::precision_fixed(f64::NAN).is_none()); // d3-format/test/precisionFixed-test.js:18:3
}

// d3-format/test/precisionFixed-test.js: precisionFixed(Infinity) returns NaN
#[test]
fn precisionfixed_infinity_returns_nan() {
    assert!(format::precision_fixed(f64::INFINITY).is_none()); // d3-format/test/precisionFixed-test.js:22:3
    assert!(format::precision_fixed(-f64::INFINITY).is_none()); // d3-format/test/precisionFixed-test.js:23:3
}

// d3-format/test/precisionPrefix-test.js: precisionPrefix(step, value) returns the expected precision when value is less than one yocto
#[test]
fn precisionprefix_step_value_returns_the_expected_precision_when_value_is_less_than_one_yocto() {
    assert_d3_eq(&format::precision_prefix(1e-24, 1e-24), &0.0); // d3-format/test/precisionPrefix-test.js:29:3
    assert_d3_eq(&format::precision_prefix(1e-25, 1e-25), &1.0); // d3-format/test/precisionPrefix-test.js:30:3
    assert_d3_eq(&format::precision_prefix(1e-26, 1e-26), &2.0); // d3-format/test/precisionPrefix-test.js:31:3
    assert_d3_eq(&format::precision_prefix(1e-27, 1e-27), &3.0); // d3-format/test/precisionPrefix-test.js:32:3
    assert_d3_eq(&format::precision_prefix(1e-28, 1e-28), &4.0); // d3-format/test/precisionPrefix-test.js:33:3
}

// d3-format/test/precisionPrefix-test.js: precisionPrefix(step, value) returns the expected precision when value is greater than than one yotta
#[test]
fn precisionprefix_step_value_returns_the_expected_precision_when_value_is_greater_than_than_one_yotta()
 {
    assert_d3_eq(&format::precision_prefix(1e24, 1e24), &0.0); // d3-format/test/precisionPrefix-test.js:37:3
    assert_d3_eq(&format::precision_prefix(1e24, 1e25), &0.0); // d3-format/test/precisionPrefix-test.js:38:3
    assert_d3_eq(&format::precision_prefix(1e24, 1e26), &0.0); // d3-format/test/precisionPrefix-test.js:39:3
    assert_d3_eq(&format::precision_prefix(1e24, 1e27), &0.0); // d3-format/test/precisionPrefix-test.js:40:3
    assert_d3_eq(&format::precision_prefix(1e23, 1e27), &1.0); // d3-format/test/precisionPrefix-test.js:41:3
}

// d3-format/test/precisionPrefix-test.js: precisionPrefix(0, value) returns NaN
#[test]
fn precisionprefix_0_value_returns_nan() {
    assert!(format::precision_prefix(0.0, 1.0).is_none()); // d3-format/test/precisionPrefix-test.js:45:3
}

// d3-format/test/precisionPrefix-test.js: precisionPrefix(NaN, value) returns NaN
#[test]
fn precisionprefix_nan_value_returns_nan() {
    assert!(format::precision_prefix(f64::NAN, 1.0).is_none()); // d3-format/test/precisionPrefix-test.js:49:3
}

// d3-format/test/precisionPrefix-test.js: precisionPrefix(Infinity, value) returns NaN
#[test]
fn precisionprefix_infinity_value_returns_nan() {
    assert!(format::precision_prefix(f64::INFINITY, 1.0).is_none()); // d3-format/test/precisionPrefix-test.js:53:3
    assert!(format::precision_prefix(-f64::INFINITY, 1.0).is_none()); // d3-format/test/precisionPrefix-test.js:54:3
}

// d3-format/test/precisionRound-test.js: precisionRound(step, max) returns the expected value
#[test]
fn precisionround_step_max_returns_the_expected_value() {
    assert_d3_eq(&format::precision_round(0.1, 1.1), &2.0); // d3-format/test/precisionRound-test.js:5:3
    assert_d3_eq(&format::precision_round(0.01, 0.99), &2.0); // d3-format/test/precisionRound-test.js:6:3
    assert_d3_eq(&format::precision_round(0.01, 1.00), &2.0); // d3-format/test/precisionRound-test.js:7:3
    assert_d3_eq(&format::precision_round(0.01, 1.01), &3.0); // d3-format/test/precisionRound-test.js:8:3
}

// d3-format/test/precisionRound-test.js: precisionRound(0, max) returns NaN
#[test]
fn precisionround_0_max_returns_nan() {
    assert!(format::precision_round(0.0, 1.0).is_none()); // d3-format/test/precisionRound-test.js:12:3
}

// d3-format/test/precisionRound-test.js: precisionRound(NaN, max) returns NaN
#[test]
fn precisionround_nan_max_returns_nan() {
    assert!(format::precision_round(f64::NAN, 1.0).is_none()); // d3-format/test/precisionRound-test.js:16:3
}

// d3-format/test/precisionRound-test.js: precisionRound(Infinity, max) returns NaN
#[test]
fn precisionround_infinity_max_returns_nan() {
    assert!(format::precision_round(f64::INFINITY, 1.0).is_none()); // d3-format/test/precisionRound-test.js:20:3
    assert!(format::precision_round(-f64::INFINITY, 1.0).is_none()); // d3-format/test/precisionRound-test.js:21:3
}
