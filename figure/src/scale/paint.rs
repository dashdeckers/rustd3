//! [`Paint`]: scales that color values.

use rustd3_core::color::Rgb;

use crate::scale::Kind;

/// A scale that colors domain values.
pub trait Paint: Kind<Output = Rgb> {
    /// The color of `v`; `None` when the scale has none for it.
    fn paint(&self, v: &Self::Domain) -> Option<Rgb>;

    /// The key's slot among all keys and their count, so dodged marks line
    /// up in the palette's order; `None` for scales without keys.
    fn slot(&self, _v: &Self::Domain) -> Option<(usize, usize)> {
        None
    }
}
