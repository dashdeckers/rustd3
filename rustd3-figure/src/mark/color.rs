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

/// The opacity an opaque ribbon color is drawn at unless set, so lines
/// drawn over the band show through (seaborn's error bands use 0.2,
/// matplotlib's examples 0.2–0.3).
const BAND: f64 = 0.25;

/// How a filled mark's opacity follows its color's. The fill, and a legend
/// square of its key, draw the color at the opacity [`Alpha::of`] gives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Alpha {
    /// The color's opacity times a factor in [0, 1].
    Times(f64),
    /// A band's opacity: 0.25 for an opaque color, a translucent color's
    /// own.
    Band,
}

/// A palette handle and a key of its domain, with the palette's type
/// erased.
pub(crate) trait Lookup {
    /// The key's color in the resolved palette; `None` when the palette
    /// has none for it.
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb>;

    /// The key's index among the palette's keys and the number of keys,
    /// which place a dodged mark; `None` when the palette has none.
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

impl Alpha {
    /// Drawn as is.
    pub(crate) const OPAQUE: Alpha = Alpha::Times(1.0);

    /// The opacity a color of opacity `opacity` is drawn at.
    pub(crate) fn of(self, opacity: f64) -> f64 {
        match self {
            Alpha::Times(a) => opacity * a,
            Alpha::Band if opacity < 1.0 => opacity,
            Alpha::Band => BAND,
        }
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

    /// The key's index among its palette's keys and the number of keys, for
    /// [`dodge`](super::dodge); `None` without a key.
    pub(crate) fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        match self {
            Color::Theme | Color::Fixed(_) => None,
            Color::Keyed(lookup) => lookup.slot(ctx),
        }
    }

    /// Reports the key with the mark's legend swatch `glyph`, its `hatch`
    /// and its fill's `alpha`. Legends of the key then draw that swatch with
    /// the hatch, and a square swatch at that opacity. A mark without a key
    /// reports nothing.
    pub(crate) fn report(
        &self,
        glyph: Swatch,
        hatch: Option<&Hatch>,
        alpha: Alpha,
        fits: &mut Fits,
    ) {
        if let Color::Keyed(lookup) = self {
            let (palette, key) = lookup.key();
            fits.key(palette, key, (glyph, hatch, alpha));
        }
    }

    /// The color to draw in: a key the palette lacks, or a palette of
    /// another figure, draws the theme's gray.
    pub(crate) fn resolve(&self, ctx: &Ctx<'_>) -> Rgb {
        match self {
            Color::Theme => ctx.get_theme().get_color(),
            Color::Fixed(color) => *color,
            Color::Keyed(lookup) => lookup
                .paint(ctx)
                .unwrap_or_else(|| ctx.get_theme().get_gray()),
        }
    }
}
