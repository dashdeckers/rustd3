//! Review H: one value under the neutral default and
//! three d3 locales.
use d3_core::format::{Definition, Locale};
fn def(decimal: &str, thousands: &str, currency: [&str; 2]) -> Locale {
    Locale::new(Definition {
        decimal: Some(decimal.into()),
        thousands: Some(thousands.into()),
        grouping: Some(vec![3]),
        currency: Some(currency.map(String::from)),
        ..Default::default()
    })
}
fn main() {
    let cases = [
        ("neutral", def(".", "\u{202f}", ["", ""])),
        ("en-US (d3 default)", Locale::default()),
        ("de-DE", def(",", ".", ["", "\u{a0}€"])),
        ("fr-FR", def(",", "\u{a0}", ["", "\u{a0}€"])),
    ];
    for (name, l) in &cases {
        let f = |s: &str, v: f64| l.format(s).unwrap().format(v);
        println!(
            "{name:20} {:>14} | {:>10} | {:>8} | {:>12}",
            f(",.1f", 1234567.5).replace('\u{202f}', "\u{2009}"),
            f(",f", 10000.0).replace('\u{202f}', "\u{2009}"),
            f(".1f", -0.5),
            f("$,.2f", 1234.5).replace('\u{a0}', " "),
        );
    }
}
