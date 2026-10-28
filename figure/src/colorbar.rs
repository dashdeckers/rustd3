//! Colorbars: what a legend is to a palette, a [`Colorbar`] is to a
//! continuous or binned color scale. It goes into a figure slot
//! ([`Figure::legend`](crate::Figure::legend)) or a panel corner
//! ([`Panel::legend`](crate::Panel::legend)) like a legend: `em(1)` thick
//! (the tick-label size), as long as the plot areas beside a slot or 40%
//! of a panel's long side in a corner. Its ramp is one image strip drawn
//! with nearest sampling ([`Ramp`]).
//!
//! ```
//! use rustd3_core::{scale, scheme};
//! use rustd3_figure::Figure;
//! use rustd3_figure::colorbar::colorbar;
//! use rustd3_figure::legend::{Inset, Slot};
//!
//! let mut fig = Figure::new();
//! let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
//! fig.legend(Slot::Right, colorbar(heat));
//! fig.panel(0, 0).legend(Inset::TopRight, colorbar(heat));
//! ```

mod block;
mod ramp;
mod strip;

use std::marker::PhantomData;

use crate::figure::Resolved;
use crate::legend::{Erased, Guide, Stored};
use crate::scale::{Paint, ScaleRef};

pub(crate) use block::{Block, Content};
pub use ramp::{Ramp, Samples};
pub(crate) use strip::Strip;

/// The legend of a continuous or binned color scale: a strip of its colors
/// along a bar, from its low end (bottom or left) to its high end.
pub struct Colorbar<'a, S> {
    handle: ScaleRef<S>,
    life: PhantomData<&'a ()>,
}

impl<S> Clone for Colorbar<'_, S> {
    fn clone(&self) -> Self {
        Colorbar {
            handle: self.handle,
            life: PhantomData,
        }
    }
}

/// A colorbar of the color scale behind `handle`.
pub fn colorbar<'a, S>(handle: ScaleRef<S>) -> Colorbar<'a, S>
where
    S: Paint<Domain = f64> + Ramp,
{
    Colorbar {
        handle,
        life: PhantomData,
    }
}

/// A colorbar with its scale type erased: what layout and drawing ask of
/// it.
pub(crate) trait Draw<'a>: 'a {
    /// The strip of its resolved scale.
    fn strip(&self, resolved: &Resolved) -> Strip;
}

impl<'a, S> Draw<'a> for Colorbar<'a, S>
where
    S: Paint<Domain = f64> + Ramp,
{
    fn strip(&self, resolved: &Resolved) -> Strip {
        Strip::of(resolved.scale(self.handle))
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
