//! [`Sizes`]: a legend of a size handle's radii.

use rustd3_core::scale::Ticks;

use crate::figure::Resolved;
use crate::legend::{self, Entry, Erased, Guide, Stored, Swatch};
use crate::scale::{Kind, ScaleRef};
use crate::text::Text;
use crate::theme::Theme;

/// A legend of the radii a size handle gives (see [`sizes`]).
pub struct Sizes<S> {
    handle: ScaleRef<S>,
    count: usize,
    values: Option<Vec<f64>>,
}

/// A legend of `handle`, a numeric handle whose output is a radius in px
/// (a dots mark's [`size_by`](crate::mark::Dots::size_by)): one entry per
/// tick of its resolved scale for about [`count`](Sizes::count) ticks (4
/// by default), or per [`values`](Sizes::values) value, inside its domain,
/// each a dot of that value's radius in the theme's ink (matplotlib's
/// `legend_elements` draws them black, so the legend reads as size alone)
/// beside the tick's label; a value whose radius is not positive has no
/// entry. The ticks are d3's: sizes over 1 … 100 at
/// count 4 give 20, 40, 60, 80, 100, where matplotlib's `legend_elements`
/// (`MaxNLocator`) gives 25, 50, 75, 100. Rows grow to hold their dots.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::legend;
/// use rustd3_figure::legend::{Inset, Slot};
///
/// let mut fig = Figure::new();
/// let radius = fig.scale(scale::sqrt().domain([1.0, 100.0]).range([2.0, 12.0]));
/// fig.legend(Slot::Right, legend::sizes(radius).count(3));
/// fig.panel(0, 0)
///     .legend(Inset::TopLeft, legend::sizes(radius).values([10.0, 100.0]));
/// ```
pub fn sizes<S>(handle: ScaleRef<S>) -> Sizes<S>
where
    S: Kind<Domain = f64, Output = f64> + Ticks<Domain = f64>,
{
    Sizes {
        handle,
        count: 4,
        values: None,
    }
}

impl<S> Sizes<S> {
    /// About `n` entries: the scale's ticks for count `n`.
    pub fn count(self, n: usize) -> Self {
        Self { count: n, ..self }
    }

    /// One entry per value in `v` inside the domain, in place of the
    /// scale's ticks, labelled as its ticks are.
    pub fn values(self, v: impl IntoIterator<Item = f64>) -> Self {
        Self {
            values: Some(v.into_iter().collect()),
            ..self
        }
    }
}

impl<'a, S> Guide<'a> for Sizes<S>
where
    S: Kind<Domain = f64, Output = f64> + Ticks<Domain = f64>,
{
    fn erase(self) -> Erased<'a> {
        Erased(Stored::Keys(Box::new(self)))
    }
}

impl<S> legend::Draw for Sizes<S>
where
    S: Kind<Domain = f64, Output = f64> + Ticks<Domain = f64>,
{
    fn scale(&self) -> ScaleRef<()> {
        self.handle.erased()
    }

    /// One dot per entry of the radius the scale gives it, skipped when
    /// that radius is not finite and positive.
    fn entries(&self, resolved: &Resolved, theme: &Theme) -> Vec<Entry> {
        let Some(scale) = resolved.scale(self.handle) else {
            return Vec::new();
        };
        let values = self.values.as_deref();
        super::entries(self.handle, self.count, values, resolved)
            .into_iter()
            .filter_map(|(v, label)| {
                let r = Ticks::position(scale, &v, 0.0).filter(|r| r.is_finite() && *r > 0.0)?;
                Some(Entry {
                    color: Some(theme.get_ink()),
                    label,
                    glyph: Swatch::Dot,
                    hatch: None,
                    size: Some(2.0 * r),
                })
            })
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        None
    }
}
