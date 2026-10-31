//! Colorbars: what a legend is to a palette, a [`Colorbar`] is to a
//! continuous or binned color scale. It goes into a figure slot
//! ([`Figure::legend`](crate::Figure::legend)) or a panel corner
//! ([`Panel::legend`](crate::Panel::legend)) like a legend: `em(1)` thick
//! (the tick-label size), as long as the plot areas beside a slot or 40%
//! of a panel's long side in a corner. Its ramp is one image strip drawn
//! with nearest sampling ([`Ramp`]), labelled by the scale's own ticks in
//! the figure's locale, outlined by the theme's axis line, with
//! matplotlib's end triangles on request ([`Extend`]).
//!
//! ```
//! use rustd3_core::{scale, scheme};
//! use rustd3_figure::Figure;
//! use rustd3_figure::colorbar::{Extend, colorbar};
//! use rustd3_figure::legend::{Inset, Slot};
//!
//! let mut fig = Figure::new();
//! let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
//! fig.legend(Slot::Right, colorbar(heat).title("temp [°C]"));
//! fig.panel(0, 0)
//!     .legend(Inset::TopRight, colorbar(heat).ticks(3).extend(Extend::Both));
//! ```
//!
//! A colorbar needs a scale that maps numbers to colors along a ramp; an
//! ordinal palette has keys, which a [`legend`](crate::legend::legend)
//! shows, so a colorbar of one does not compile (E0277, `Ordinal<&str,
//! Rgb>: Ramp` is not satisfied; rustc also reports E0271 for the keys,
//! which are not `f64`):
//!
//! ```compile_fail,E0277
//! use rustd3_core::scale;
//! use rustd3_core::scheme::COLORBLIND;
//! use rustd3_figure::Figure;
//! use rustd3_figure::colorbar::colorbar;
//! use rustd3_figure::legend::Slot;
//!
//! let mut fig = Figure::new();
//! let keys = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
//! fig.legend(Slot::Right, colorbar(keys));
//! ```
//!
//! ```
//! use rustd3_core::{scale, scheme};
//! use rustd3_figure::Figure;
//! use rustd3_figure::colorbar::colorbar;
//! use rustd3_figure::legend::Slot;
//!
//! let mut fig = Figure::new();
//! let heat = fig.scale(scale::sequential([0.0, 1.0], scheme::viridis));
//! fig.legend(Slot::Right, colorbar(heat));
//! ```

mod block;
mod edges;
mod extend;
mod ramp;
mod side;
mod strip;

use std::rc::Rc;

use rustd3_core::scale::Ticks;

use crate::figure::Resolved;
use crate::legend::{Erased, Guide, Stored};
use crate::scale::{Paint, ScaleRef};
use crate::text::Text;

pub(crate) use block::{Block, Content};
pub use extend::Extend;
pub use ramp::{Ramp, Samples};
pub(crate) use side::Side;
pub(crate) use strip::Strip;

/// A label for each tick value, in place of the scale's.
type Format<'a> = Rc<dyn Fn(f64) -> String + 'a>;

/// The legend of a continuous or binned color scale: a strip of its colors
/// along a bar, from its low end (bottom or left) to its high end, its
/// ticks and their labels on the side away from the grid or, in a corner,
/// right of a vertical bar and below a horizontal one, and its title past
/// them along the bar. Built once and placed in any number of slots and
/// corners (`Clone` shares its format).
pub struct Colorbar<'a, S> {
    handle: ScaleRef<S>,
    title: Option<Text>,
    extend: Extend,
    count: Option<usize>,
    format: Option<Format<'a>>,
}

impl<'a, S> std::fmt::Debug for Colorbar<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Colorbar").finish_non_exhaustive()
    }
}

impl<S> Clone for Colorbar<'_, S> {
    fn clone(&self) -> Self {
        Colorbar {
            handle: self.handle,
            title: self.title.clone(),
            extend: self.extend,
            count: self.count,
            format: self.format.clone(),
        }
    }
}

/// A colorbar of the color scale behind `handle`.
#[must_use]
pub fn colorbar<'a, S>(handle: ScaleRef<S>) -> Colorbar<'a, S>
where
    S: Paint<Domain = f64> + Ramp,
{
    Colorbar {
        handle,
        title: None,
        extend: Extend::Neither,
        count: None,
        format: None,
    }
}

impl<'a, S> Colorbar<'a, S> {
    /// A title along the bar, past its tick labels, as an axis's.
    #[must_use]
    pub fn title(self, t: impl Into<Text>) -> Self {
        Colorbar {
            title: Some(t.into()),
            ..self
        }
    }

    /// End triangles for the values the scale clamps; none by default.
    #[must_use]
    pub fn extend(self, e: Extend) -> Self {
        Colorbar { extend: e, ..self }
    }

    /// About `count` ticks from the scale. Without it the count follows
    /// the bar's length as an axis's does: one tick per 80 px across, per
    /// 35 px up, at least 2. Binned scales tick at their edges whatever the
    /// count.
    #[must_use]
    pub fn ticks(self, count: usize) -> Self {
        Colorbar {
            count: Some(count),
            ..self
        }
    }

    /// Each label is `f(value)`, borrowing for `'a` as an axis's format
    /// does.
    #[must_use]
    pub fn format(self, f: impl Fn(f64) -> String + 'a) -> Self {
        let format: Format<'a> = Rc::new(f);
        Colorbar {
            format: Some(format),
            ..self
        }
    }
}

/// A colorbar with its scale type erased: what layout and drawing ask of
/// it.
pub(crate) trait Draw<'a>: 'a {
    /// Its handle's position in the figure's scale table.
    fn handle(&self) -> usize;

    /// Its handle, its scale type erased.
    fn scale(&self) -> ScaleRef<()>;

    /// The strip of its resolved scale.
    fn strip(&self, resolved: &Resolved) -> Strip;

    /// The tick count `.ticks(n)` set.
    fn count(&self) -> Option<usize>;

    /// Its end triangles.
    fn extend(&self) -> Extend;

    /// The title along the bar.
    fn title(&self) -> Option<&Text>;

    /// Whether `.format` sets its labels.
    fn formats(&self) -> bool;

    /// The labels of the resolved scale's about `count` ticks: the
    /// format's; for a binned scale (`binned`), its edges' at the places
    /// they need ([`edges::labels`]); else the scale's own; in the figure's
    /// locale.
    fn texts(&self, resolved: &Resolved, count: usize, binned: bool) -> Vec<String>;

    /// Each of about `count` ticks' share of the bar from its low end,
    /// aligned with [`texts`](Draw::texts): the scale's tick position;
    /// `None` where it has none or it is not finite.
    fn shares(&self, resolved: &Resolved, count: usize) -> Vec<Option<f64>>;
}

impl<'a, S> Draw<'a> for Colorbar<'a, S>
where
    S: Paint<Domain = f64> + Ramp,
{
    fn handle(&self) -> usize {
        self.handle.index()
    }

    fn scale(&self) -> ScaleRef<()> {
        self.handle.erased()
    }

    fn strip(&self, resolved: &Resolved) -> Strip {
        Strip::of(resolved.own(self.handle))
    }

    fn count(&self) -> Option<usize> {
        self.count
    }

    fn extend(&self) -> Extend {
        self.extend
    }

    fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    fn formats(&self) -> bool {
        self.format.is_some()
    }

    fn texts(&self, resolved: &Resolved, count: usize, binned: bool) -> Vec<String> {
        let scale = resolved.own(self.handle);
        let set = scale.tick_set(count);
        match &self.format {
            Some(format) => set.values().iter().map(|v| format(*v)).collect(),
            None if binned => edges::labels(set.values(), resolved.locale()),
            None => scale.labels(&set, resolved.locale()),
        }
    }

    fn shares(&self, resolved: &Resolved, count: usize) -> Vec<Option<f64>> {
        let scale = resolved.own(self.handle);
        scale
            .tick_set(count)
            .values()
            .iter()
            .map(|v| Ticks::position(scale, v, 0.0).filter(|p| p.is_finite()))
            .collect()
    }
}

impl<'a, S> Guide<'a> for Colorbar<'a, S>
where
    S: Paint<Domain = f64> + Ramp,
{
    fn erase(self) -> Erased<'a> {
        Erased(Stored::Bar(Box::new(self)))
    }
}
