//! The figure locale: what labels are written in, one value for every
//! label a scale, an axis or a figure formats.
//!
//! ```
//! use rustd3_core::locale::Locale;
//! use rustd3_core::scale::{self, Ticks};
//!
//! let a = scale::linear().domain([0.0, 12000.0]);
//! let set = a.tick_set(5);
//! assert_eq!(a.labels(&set, &Locale::NEUTRAL)[1], "2\u{202f}000");
//! assert_eq!(a.labels(&set, &Locale::EN_US)[1], "2,000");
//! ```

use crate::format;

/// The figure locale: the number formats of its labels.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    number: format::Locale,
}

impl Default for Locale {
    /// [`Locale::NEUTRAL`].
    fn default() -> Self {
        Locale::NEUTRAL
    }
}

impl From<format::Locale> for Locale {
    /// The figure locale whose labels write numbers as `number` does: a
    /// runtime table, such as one of d3-format's, labelling an axis.
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// use rustd3_core::format::{self, Definition};
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale::{self, Ticks};
    ///
    /// let de = Locale::from(format::Locale::new(Definition {
    ///     decimal: Cow::Borrowed(","),
    ///     thousands: Cow::Borrowed("."),
    ///     grouping: Cow::Borrowed(&[3]),
    ///     ..Definition::default()
    /// }));
    /// let x = scale::linear().domain([0.0, 12000.0]);
    /// assert_eq!(x.labels(&x.tick_set(5), &de)[1], "2.000");
    /// ```
    fn from(number: format::Locale) -> Self {
        Locale { number }
    }
}

impl Locale {
    /// SI style: a point, U+202F groups of three from five integer digits,
    /// U+2212 for minus. `Default` returns it.
    pub const NEUTRAL: Locale = Locale {
        number: format::Locale::NEUTRAL,
    };

    /// d3's default locale: commas in groups of three, a dollar currency.
    pub const EN_US: Locale = Locale {
        number: format::Locale::EN_US,
    };

    /// The number half, d3's `formatLocale`.
    pub fn number(&self) -> &format::Locale {
        &self.number
    }
}
