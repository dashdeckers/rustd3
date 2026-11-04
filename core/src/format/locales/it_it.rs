// Generated from d3-format 3.1.2 locale/it-IT.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "Italian (Italy): `1.234.567,5`, `\u{20ac}1.234,50`."]
    pub const IT_IT: Locale = Locale::from_table(Table {
        decimal: ",",
        thousands: ".",
        grouping: &[3],
        currency: ["\u{20ac}", ""],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
