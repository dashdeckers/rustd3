//! One-dimensional brushes: values placed on a panel, like axes. A panel
//! with a brush selects an interval of a handle's domain by dragging over
//! its plot area; in the iced `Plot` widget the selection is published
//! with `Plot::on_brush`, and with [`Brush::sets`] it becomes the domain
//! of another handle (d3's focus and context). File outputs ignore
//! brushes: they have no input.
//!
//! A brush needs a typed inverse from pixels to its handle's domain
//! ([`Invert`]): continuous and time handles have one, band and point
//! handles do not, so a brush over a band handle does not compile. The
//! handle it sets must take the same domain values: a selection of
//! numbers cannot set a time handle (E0271, type mismatch resolving
//! `<Time as Kind>::Domain == f64`):
//!
//! ```compile_fail,E0271
//! use rustd3_core::{scale, time};
//! use rustd3_figure::{Figure, brush};
//!
//! let mut fig = Figure::new();
//! let full = fig.scale(scale::linear().domain([0.0, 100.0]));
//! let focus = fig.fit(time::utc());
//! fig.panel(1, 0).brush(brush::x(full).sets(focus)); // E0271
//! ```
//!
//! Two time handles agree:
//!
//! ```
//! use rustd3_core::jiff::{SignedDuration, Timestamp};
//! use rustd3_core::time;
//! use rustd3_figure::{Figure, brush};
//!
//! let start: Timestamp = "2026-09-01T00:00:00Z".parse().expect("an instant");
//! let end = start + SignedDuration::from_hours(719);
//! let mut fig = Figure::new();
//! let focus = fig.fit(time::utc());
//! let full = fig.scale(time::utc().domain([start, end]));
//! fig.panel(0, 0);
//! fig.panel(1, 0).brush(brush::x(full).sets(focus));
//! ```

mod dim;
mod placed;

pub use dim::Dim;
pub use placed::Placed;

use crate::scale::{Invert, Kind, Position, ScaleRef};

/// A brush over handle `over` (type `A`) whose selection sets the domain
/// of a handle of type `B`; `B` is `A` until [`sets`](Brush::sets) names a
/// target. Built with [`x`] or [`y`] and placed with `Panel::brush`.
pub struct Brush<A, B> {
    dim: Dim,
    over: ScaleRef<A>,
    target: Option<ScaleRef<B>>,
}

impl<A, B> std::fmt::Debug for Brush<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Brush").finish_non_exhaustive()
    }
}

impl<A, B> Clone for Brush<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, B> Copy for Brush<A, B> {}

/// A brush across the plot area over `over`: its selection is an interval
/// of `over`'s domain along x, and spans the plot area's height.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::{Figure, brush};
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::linear().domain([0.0, 10.0]));
/// fig.panel(0, 0).brush(brush::x(x));
/// ```
#[must_use]
pub fn x<A: Invert>(over: ScaleRef<A>) -> Brush<A, A> {
    Brush {
        dim: Dim::X,
        over,
        target: None,
    }
}

/// A brush up the plot area over `over`: its selection is an interval of
/// `over`'s domain along y, and spans the plot area's width.
#[must_use]
pub fn y<A: Invert>(over: ScaleRef<A>) -> Brush<A, A> {
    Brush {
        dim: Dim::Y,
        over,
        target: None,
    }
}

impl<A: Invert> Brush<A, A> {
    /// The selection becomes the domain of `target` (d3's focus and
    /// context): a handle whose domain takes the same values as `over`'s.
    /// An empty selection gives it `over`'s whole domain.
    #[must_use]
    pub fn sets<B>(self, target: ScaleRef<B>) -> Brush<A, B>
    where
        B: Position<Domain = <A as Kind>::Domain>,
    {
        Brush {
            dim: self.dim,
            over: self.over,
            target: Some(target),
        }
    }
}

impl<A, B> From<Brush<A, B>> for Placed
where
    A: Invert,
    B: Position<Domain = <A as Kind>::Domain>,
{
    fn from(brush: Brush<A, B>) -> Placed {
        Placed::new::<A, B>(
            brush.dim,
            brush.over.erased(),
            brush.target.map(ScaleRef::erased),
        )
    }
}
