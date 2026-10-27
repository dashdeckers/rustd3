//! [`Item`], one piece of a compiled format: text or a directive.

use jiff::Zoned;

use crate::time::Locale;
use crate::time::format::{Directive, Pad};

/// Literal text, or a directive with its pad (d3's `newFormat` loop,
/// `locale.js:170–193`, split once at compile time).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    /// Written as is.
    Literal(String),
    /// Written for each instant.
    Directive(Directive, Pad),
}

impl Item {
    /// Writes the item for the instant `at`, names from `locale`.
    pub(crate) fn write(&self, at: &Zoned, locale: &Locale, out: &mut String) {
        match self {
            Item::Literal(text) => out.push_str(text),
            Item::Directive(directive, pad) => directive.write(*pad, at, locale, out),
        }
    }
}
