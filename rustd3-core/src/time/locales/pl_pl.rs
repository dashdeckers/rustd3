// Generated from d3-time-format 4.1.0 locale/pl-PL.json. Do not edit.

use crate::time::locale::Table;
use crate::time::{Ladder, Locale};

impl Locale {
    #[doc = "Polish (Poland): `%c` writes `Sobota,  7 Marzec 2026, 23:59:30`."]
    pub const PL_PL: Locale = Locale::from_table(Table {
        date_time: "%A, %e %B %Y, %X",
        date: "%d/%m/%Y",
        time: "%H:%M:%S",
        periods: ["AM", "PM"],
        days: ["Niedziela", "Poniedzia\u{142}ek", "Wtorek", "\u{15a}roda", "Czwartek", "Pi\u{105}tek", "Sobota"],
        short_days: ["Niedz.", "Pon.", "Wt.", "\u{15a}r.", "Czw.", "Pt.", "Sob."],
        months: ["Stycze\u{144}", "Luty", "Marzec", "Kwiecie\u{144}", "Maj", "Czerwiec", "Lipiec", "Sierpie\u{144}", "Wrzesie\u{144}", "Pa\u{17a}dziernik", "Listopad", "Grudzie\u{144}"],
        short_months: ["Stycz.", "Luty", "Marz.", "Kwie.", "Maj", "Czerw.", "Lipc.", "Sierp.", "Wrz.", "Pa\u{17a}dz.", "Listop.", "Grudz."],
        ladder: Ladder::Iso,
    });
}
