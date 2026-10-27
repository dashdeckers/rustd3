//! Marks: the [`Mark`] trait every mark implements, and [`Ctx`], what a
//! mark sees while it draws one panel.

use crate::figure::{Fits, Resolved};
use crate::list::List;
use crate::scale::{Kind, Position, ScaleRef};
use crate::theme::Theme;

/// A mark: data drawn through the figure's scales in one panel. Its data
/// may borrow for `'a` (a recording's
/// [`Points`](rustd3_core::series::Points), say), so a figure that borrows
/// is built where the data lives.
pub trait Mark<'a>: 'a {
    /// Reports the values each fitted handle must span ([`Fits::add`]),
    /// and the values margins stop at ([`Fits::stick`]).
    fn fit(&self, fits: &mut Fits);

    /// Appends the mark's commands to `list`, in list px.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List);
}

/// What a mark sees while it draws one panel: the resolved scales over the
/// panel's pixel ranges, its plot area, the theme and the device scale.
pub struct Ctx<'r> {
    resolved: &'r Resolved,
    theme: &'r Theme,
    rect: kurbo::Rect,
}

impl Ctx<'_> {
    /// `h`'s resolved scale across the plot area, over [0, w].
    pub fn x<S: Position>(&self, h: ScaleRef<S>) -> S {
        self.ranged(h, [0.0, self.rect.width()])
    }

    /// `h`'s resolved scale up the plot area, over [h, 0].
    pub fn y<S: Position>(&self, h: ScaleRef<S>) -> S {
        self.ranged(h, [self.rect.height(), 0.0])
    }

    /// `h`'s resolved scale as it is, before any panel's range (a palette,
    /// say).
    pub fn raw<S: Kind>(&self, h: ScaleRef<S>) -> S {
        self.resolved.scale(h).clone()
    }

    /// The plot area, in list px.
    pub fn rect(&self) -> kurbo::Rect {
        self.rect
    }

    /// The figure's theme.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Device pixels per list px of the target drawn for: 1 for vector
    /// output.
    pub fn scale_factor(&self) -> f64 {
        self.resolved.target().scale_factor()
    }

    /// `h`'s resolved scale over `range`.
    fn ranged<S: Kind>(&self, h: ScaleRef<S>, range: [f64; 2]) -> S {
        self.resolved
            .scale(h)
            .with_range(range)
            .as_any()
            .downcast_ref::<S>()
            .expect("invariant: a scale's with_range keeps its type")
            .clone()
    }
}
