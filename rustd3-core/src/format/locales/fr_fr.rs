// Generated from d3-format 3.1.2 locale/fr-FR.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "French (France): `1\u{a0}234\u{a0}567,5`, `1\u{a0}234,50\u{a0}\u{20ac}`."]
    pub const FR_FR: Locale = Locale::from_table(Table {
        decimal: ",",
        thousands: "\u{a0}",
        grouping: &[3],
        currency: ["", "\u{a0}\u{20ac}"],
        numerals: None,
        percent: "\u{202f}%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
