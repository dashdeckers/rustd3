// Generated from d3-time-format 4.1.0 locale/it-IT.json. Do not edit.

use crate::time::locale::Table;
use crate::time::{Ladder, Locale};

impl Locale {
    #[doc = "Italian (Italy): `%c` writes `Sabato  7 Marzo 2026, 23:59:30`."]
    pub const IT_IT: Locale = Locale::from_table(Table {
        date_time: "%A %e %B %Y, %X",
        date: "%d/%m/%Y",
        time: "%H:%M:%S",
        periods: ["AM", "PM"],
        days: ["Domenica", "Luned\u{ec}", "Marted\u{ec}", "Mercoled\u{ec}", "Gioved\u{ec}", "Venerd\u{ec}", "Sabato"],
        short_days: ["Dom", "Lun", "Mar", "Mer", "Gio", "Ven", "Sab"],
        months: ["Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno", "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre"],
        short_months: ["Gen", "Feb", "Mar", "Apr", "Mag", "Giu", "Lug", "Ago", "Set", "Ott", "Nov", "Dic"],
        ladder: Ladder::Iso,
    });
}
