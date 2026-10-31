//! [`Fill`]: what a chart's shapes are filled in.

use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::mark::Ctx;
use crate::scale::{Paint, ScaleRef};

/// What a chart's shapes are filled in: the theme palette's first color,
/// or a palette handle's color for a key, looked up when the figure draws.
pub(super) struct Fill(Option<Box<dyn Lookup>>);

/// A palette handle and a key of its domain, with the palette's type
/// erased.
trait Lookup {
    /// The key's color in the resolved palette; `None` when it has none.
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb>;

    /// The key's slot among the palette's keys and their count, which
    /// dodged charts sit in; `None` when the palette has none.
    fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)>;
}

/// A palette handle and a key of its domain.
struct Keyed<C: Paint> {
    palette: ScaleRef<C>,
    key: C::Domain,
}

impl<C: Paint> Lookup for Keyed<C> {
    fn paint(&self, ctx: &Ctx<'_>) -> Option<Rgb> {
        ctx.get_raw(self.palette)?.paint(&self.key)
    }

    fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        ctx.get_raw(self.palette)?.slot(&self.key)
    }
}

impl Fill {
    /// The theme palette's first color.
    pub(super) fn theme() -> Fill {
        Fill(None)
    }

    /// `palette`'s color for `key`.
    pub(super) fn keyed<C: Paint>(palette: ScaleRef<C>, key: C::Domain) -> Fill {
        Fill(Some(Box::new(Keyed { palette, key })))
    }

    /// The color to draw in: the key's, the theme's gray for a key the
    /// palette lacks, the theme palette's first color without a key.
    pub(super) fn resolve(&self, ctx: &Ctx<'_>) -> Rgb {
        match &self.0 {
            None => ctx.get_theme().get_color(),
            Some(lookup) => lookup
                .paint(ctx)
                .unwrap_or_else(|| ctx.get_theme().get_gray()),
        }
    }

    /// The key's slot among its palette's keys and their count; `None`
    /// without a key.
    pub(super) fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        self.0.as_ref().and_then(|lookup| lookup.slot(ctx))
    }

    /// Reports a key its palette lacks.
    pub(super) fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if self.0.as_ref().is_some_and(|l| l.paint(ctx).is_none()) {
            issues.push(Issue::UnknownKey {
                panel: ctx.get_panel(),
                mark: ctx.get_mark(),
            });
        }
    }
}
