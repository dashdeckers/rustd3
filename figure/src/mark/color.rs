//! [`Color`]: what a mark is drawn in.

use std::any::Any;

use rustd3_core::color::Rgb;

use crate::figure::Fits;
use crate::hatch::Hatch;
use crate::legend::Swatch;
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

    /// The key's slot among the palette's keys and their count, which
    /// dodged marks sit in; `None` when the palette has none.
    fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)>;

    /// The palette's position in the scale table and the key, boxed.
    fn key(&self) -> (ScaleRef<()>, Box<dyn Any>);
}

/// A palette handle and a key of its domain.
struct Keyed<C: Paint> {
    palette: ScaleRef<C>,
    key: C::Domain,
}

impl<C: Paint> Lookup for Keyed<C> {
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb> {
        ctx.scale(self.palette)?.paint(&self.key)
    }

    fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        ctx.scale(self.palette)?.slot(&self.key)
    }

    fn key(&self) -> (ScaleRef<()>, Box<dyn Any>) {
        (self.palette.erased(), Box::new(self.key.clone()))
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

    /// The key's slot among its palette's keys and their count, for
    /// [`dodge`](super::dodge); `None` without a key.
    pub(crate) fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        match self {
            Color::Theme | Color::Fixed(_) => None,
            Color::Keyed(lookup) => lookup.slot(ctx),
        }
    }

    /// Reports the key with the mark's legend swatch `glyph` and its
    /// `hatch`, so legends of the key draw that swatch and repeat the
    /// hatch; marks without a key report nothing.
    pub(crate) fn report(&self, glyph: Swatch, hatch: Option<&Hatch>, fits: &mut Fits) {
        if let Color::Keyed(lookup) = self {
            let (palette, key) = lookup.key();
            fits.key(palette, key, glyph, hatch);
        }
    }

    /// The color to draw in: a key the palette lacks, or a palette of
    /// another figure, draws the theme's gray.
    pub(crate) fn resolve(&self, ctx: &Ctx<'_>) -> Rgb {
        match self {
            Color::Theme => ctx.theme().get_color(),
            Color::Fixed(color) => *color,
            Color::Keyed(lookup) => lookup.paint(ctx).unwrap_or_else(|| ctx.theme().get_gray()),
        }
    }
}
