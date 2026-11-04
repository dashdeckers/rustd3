// Generated from d3-format 3.1.2 locale/pl-PL.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "Polish (Poland): `1.234.567,5`, `1.234,50z\u{142}`."]
    pub const PL_PL: Locale = Locale::from_table(Table {
        decimal: ",",
        thousands: ".",
        grouping: &[3],
        currency: ["", "z\u{142}"],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
