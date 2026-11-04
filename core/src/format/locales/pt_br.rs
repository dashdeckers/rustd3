// Generated from d3-format 3.1.2 locale/pt-BR.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "Portuguese (Brazil): `1.234.567,5`, `R$1.234,50`."]
    pub const PT_BR: Locale = Locale::from_table(Table {
        decimal: ",",
        thousands: ".",
        grouping: &[3],
        currency: ["R$", ""],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
