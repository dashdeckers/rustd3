// Generated from d3-time-format 4.1.0 locale/ja-JP.json. Do not edit.

use crate::time::locale::Table;
use crate::time::{Ladder, Locale};

impl Locale {
    #[doc = "Japanese (Japan): `%c` writes `2026/03/07 \u{571f} 23:59:30`."]
    pub const JA_JP: Locale = Locale::from_table(Table {
        date_time: "%x %a %X",
        date: "%Y/%m/%d",
        time: "%H:%M:%S",
        periods: ["AM", "PM"],
        days: ["\u{65e5}\u{66dc}\u{65e5}", "\u{6708}\u{66dc}\u{65e5}", "\u{706b}\u{66dc}\u{65e5}", "\u{6c34}\u{66dc}\u{65e5}", "\u{6728}\u{66dc}\u{65e5}", "\u{91d1}\u{66dc}\u{65e5}", "\u{571f}\u{66dc}\u{65e5}"],
        short_days: ["\u{65e5}", "\u{6708}", "\u{706b}", "\u{6c34}", "\u{6728}", "\u{91d1}", "\u{571f}"],
        months: ["1\u{6708}", "2\u{6708}", "3\u{6708}", "4\u{6708}", "5\u{6708}", "6\u{6708}", "7\u{6708}", "8\u{6708}", "9\u{6708}", "10\u{6708}", "11\u{6708}", "12\u{6708}"],
        short_months: ["1\u{6708}", "2\u{6708}", "3\u{6708}", "4\u{6708}", "5\u{6708}", "6\u{6708}", "7\u{6708}", "8\u{6708}", "9\u{6708}", "10\u{6708}", "11\u{6708}", "12\u{6708}"],
        ladder: Ladder::Iso,
    });
}
