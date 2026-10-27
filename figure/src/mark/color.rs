//! [`Color`]: what a mark is drawn in.

use rustd3_core::color::Rgb;

use crate::mark::Ctx;
use crate::scale::{Paint, ScaleRef};

/// What a mark is drawn in: the theme palette's first color, a fixed
/// color, or a palette handle's color for a key, looked up at draw time.
pub(crate) enum Color {
    /// The theme palette's first color: a mark without a key.
    Theme,
    /// A fixed color.
    Fixed(Rgb),
    /// A palette's color for a key.
    Keyed(Box<dyn Lookup>),
}

/// A palette handle and a key of its domain, with the palette's type
/// erased.
pub(crate) trait Lookup {
    /// The key's color in the resolved palette; `None` when the palette
    /// has none for it.
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb>;
}

/// A palette handle and a key of its domain.
struct Keyed<C: Paint> {
    palette: ScaleRef<C>,
    key: C::Domain,
}

impl<C: Paint> Lookup for Keyed<C> {
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb> {
        ctx.scale(self.palette).paint(&self.key)
    }
}

impl Color {
    /// `palette`'s color for `key`.
    pub(crate) fn keyed<C: Paint>(palette: ScaleRef<C>, key: C::Domain) -> Color {
        Color::Keyed(Box::new(Keyed { palette, key }))
    }

    /// Whether the color is known: false for a key its palette lacks.
    pub(crate) fn known(&self, ctx: &Ctx<'_>) -> bool {
        match self {
            Color::Theme | Color::Fixed(_) => true,
            Color::Keyed(lookup) => lookup.paint(ctx).is_some(),
        }
    }

    /// The color to draw in: a key the palette lacks draws the theme's
    /// gray.
    pub(crate) fn resolve(&self, ctx: &Ctx<'_>) -> Rgb {
        match self {
            Color::Theme => ctx.theme().color(),
            Color::Fixed(color) => *color,
            Color::Keyed(lookup) => lookup.paint(ctx).unwrap_or_else(|| ctx.theme().gray()),
        }
    }
}
