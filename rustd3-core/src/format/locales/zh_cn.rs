// Generated from d3-format 3.1.2 locale/zh-CN.json. Do not edit.

use crate::format::Locale;
use crate::format::locale::Table;

impl Locale {
    #[doc = "Chinese (China): `1,234,567.5`, `\u{a5}1,234.50`."]
    pub const ZH_CN: Locale = Locale::from_table(Table {
        decimal: ".",
        thousands: ",",
        grouping: &[3],
        currency: ["\u{a5}", ""],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 1,
    });
}
