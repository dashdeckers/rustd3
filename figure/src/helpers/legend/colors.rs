//! [`Colors`]: a legend of a continuous color handle's colors.

use rustd3_core::scale::Ticks;

use crate::figure::Resolved;
use crate::legend::{self, Entry, Erased, Guide, Stored, Swatch};
use crate::scale::{Paint, ScaleRef};
use crate::text::Text;
use crate::theme::Theme;

/// A legend of the colors a continuous color handle gives (see
/// [`colors`]).
pub struct Colors<C> {
    handle: ScaleRef<C>,
    count: usize,
}

impl<C> std::fmt::Debug for Colors<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Colors").finish_non_exhaustive()
    }
}

/// A legend of `handle`, a continuous color handle (a dots mark's
/// [`fill_by`](crate::mark::Dots::fill_by)): one entry per tick of its
/// resolved scale for about [`count`](Colors::count) ticks (5 by default),
/// inside its domain, each a dot of the theme's marker size in that
/// value's color beside the tick's label. Colors over 1 … 100 at count 5
/// give 20, 40, 60, 80, 100, as matplotlib's `legend_elements` does.
///
/// ```
/// use rustd3_core::{scale, scheme};
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::legend;
/// use rustd3_figure::legend::Slot;
///
/// let mut fig = Figure::new();
/// let heat = fig.scale(scale::sequential([1.0, 100.0], scheme::viridis));
/// fig.legend(Slot::Bottom, legend::colors(heat).count(3));
/// ```
#[must_use]
pub fn colors<C>(handle: ScaleRef<C>) -> Colors<C>
where
    C: Paint<Domain = f64> + Ticks<Domain = f64>,
{
    Colors { handle, count: 5 }
}

impl<C> Colors<C> {
    /// About `n` entries: the scale's ticks for count `n`.
    #[must_use]
    pub fn count(self, n: usize) -> Self {
        Self { count: n, ..self }
    }
}

impl<'a, C> Guide<'a> for Colors<C>
where
    C: Paint<Domain = f64> + Ticks<Domain = f64>,
{
    fn erase(self) -> Erased<'a> {
        Erased(Stored::Keys(Box::new(self)))
    }
}

impl<C> legend::Draw for Colors<C>
where
    C: Paint<Domain = f64> + Ticks<Domain = f64>,
{
    fn scale(&self) -> ScaleRef<()> {
        self.handle.erased()
    }

    /// One dot per entry in the color the scale gives it; one it gives
    /// none (`None`) takes the theme's gray.
    fn entries(&self, resolved: &Resolved, _theme: &Theme) -> Vec<Entry> {
        let Some(scale) = resolved.scale(self.handle) else {
            return Vec::new();
        };
        super::entries(self.handle, self.count, None, resolved)
            .into_iter()
            .map(|(v, label)| Entry {
                color: scale.paint(&v),
                label,
                glyph: Swatch::Dot,
                hatch: None,
                size: None,
            })
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        None
    }
}
