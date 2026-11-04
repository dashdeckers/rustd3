// Generated from d3-format 3.1.2 locale/en-US.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "English (United States): `1,234,567.5`, `$1,234.50`. d3's default locale."]
    pub const EN_US: Locale = Locale::from_table(Table {
        decimal: ".",
        thousands: ",",
        grouping: &[3],
        currency: ["$", ""],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
