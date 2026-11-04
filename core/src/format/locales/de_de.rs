// Generated from d3-format 3.1.2 locale/de-DE.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "German (Germany): `1.234.567,5`, `1.234,50\u{a0}\u{20ac}`."]
    pub const DE_DE: Locale = Locale::from_table(Table {
        decimal: ",",
        thousands: ".",
        grouping: &[3],
        currency: ["", "\u{a0}\u{20ac}"],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
