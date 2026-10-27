//! Marks: the [`Mark`] trait every mark implements, [`Ctx`], what a mark
//! sees while it draws one panel, and the figure's marks: [`dots`],
//! [`line()`], [`rule_x`] and [`rule_y`].
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::Figure;
//! use rustd3_figure::mark::{dots, rule_y};
//!
//! let mut fig = Figure::new();
//! let x = fig.fit(scale::linear());
//! let y = fig.fit(scale::linear());
//! fig.panel(0, 0)
//!     .mark(dots([(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)], x, y))
//!     .mark(rule_y(1.5, y).dash([6.0, 3.0]).label("limit"));
//! ```

mod color;
mod dots;
mod guard;
mod line;
mod rule;

use std::any::Any;
use std::cell::{RefCell, RefMut};

use kurbo::{Cap, Join};
use rustd3_core::series::Points;

use crate::figure::{Fits, Resolved};
use crate::layout::Snap;
use crate::list::List;
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::{self, Key, Shaped};
use crate::theme::Theme;
use crate::thin;

pub use dots::{Dots, dots};
pub use line::{Line, line};
pub use rule::{Rule, rule_x, rule_y};

/// A mark: data drawn through the figure's scales in one panel. Its data
/// may borrow for `'a` (a recording's
/// [`Points`], say), so a figure that borrows
/// is built where the data lives.
pub trait Mark<'a>: 'a {
    /// Reports the values each fitted handle must span ([`Fits::add`]),
    /// and the values margins stop at ([`Fits::stick`]).
    fn fit(&self, fits: &mut Fits);

    /// Appends the mark's commands to `list`, in the panel's coordinates:
    /// the plot area spans `[0, w] × [0, h]`, and the figure clips to it.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List);
}

/// What a mark sees while it draws one panel: the resolved scales over the
/// panel's pixel ranges, its plot area, the theme and the device scale.
pub struct Ctx<'r> {
    resolved: &'r Resolved,
    theme: &'r Theme,
    rect: kurbo::Rect,
    text: &'r text::Cache,
    frame: &'r RefCell<thin::Frame>,
}

impl<'r> Ctx<'r> {
    /// What the marks of the panel whose plot area is `rect` see; they
    /// thin through the draw's `frame`.
    pub(crate) fn new(
        resolved: &'r Resolved,
        theme: &'r Theme,
        rect: kurbo::Rect,
        text: &'r text::Cache,
        frame: &'r RefCell<thin::Frame>,
    ) -> Ctx<'r> {
        Ctx {
            resolved,
            theme,
            rect,
            text,
            frame,
        }
    }
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
        self.scale(h).clone()
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

    /// How the target lays hairlines on its pixel grid.
    pub(crate) fn snap(&self) -> Snap {
        Snap::of(self.resolved.target())
    }

    /// Thinning columns per list px: one per device pixel on screen, four
    /// per device pixel for files and per list pixel for vector output.
    pub(crate) fn columns(&self) -> f64 {
        self.resolved.target().columns()
    }

    /// The draw's thinning frame.
    pub(crate) fn frame(&self) -> RefMut<'_, thin::Frame> {
        self.frame.borrow_mut()
    }

    /// `h`'s resolved scale, borrowed.
    pub(crate) fn scale<S: Kind>(&self, h: ScaleRef<S>) -> &S {
        self.resolved.scale(h)
    }

    /// The shaped label of `key`, which the mark reported while fitting;
    /// `None` when typst rejected it.
    pub(crate) fn label(&self, key: &Key) -> Option<&Shaped> {
        self.text.get(key)?.ok()
    }

    /// `h`'s resolved scale over `range`.
    fn ranged<S: Kind>(&self, h: ScaleRef<S>, range: [f64; 2]) -> S {
        self.scale(h)
            .with_range(range)
            .as_any()
            .downcast_ref::<S>()
            .expect("invariant: a scale's with_range keeps its type")
            .clone()
    }
}

/// Where `v` lies on `scale`: at the middle of its band on band scales;
/// `None` for a key outside the domain or a position that is not finite.
fn at<S: Position>(scale: &S, v: &<S as Kind>::Domain) -> Option<f64> {
    let p = scale.position(v)? + scale.bandwidth() / 2.0;
    p.is_finite().then_some(p)
}

/// A channel value as a scale's domain value: `Some` when the domain is
/// `f64`, which it is whenever a mark's data is a recording channel.
fn value<D: Clone + 'static>(v: f64) -> Option<D> {
    (&v as &dyn Any).downcast_ref::<D>().cloned()
}

/// Whether `scale` admits the channel value `v`.
fn admits<S: Kind>(scale: &S, v: f64) -> bool {
    value::<<S as Kind>::Domain>(v).is_some_and(|v| scale.admits(&v))
}

/// Reports `points` on `x` and `y`: a recording channel through the x
/// scale's index (its first and last x, and its y extremes among the values
/// `y` admits), read in O(log n); anything else pair by pair.
fn fit<'a, X: Position, Y: Position>(
    points: &Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fits: &mut Fits,
) {
    let extent = points.indexed().and_then(|channel| {
        let ys = fits.scale(y);
        fits.scale(x)
            .indexed()?
            .series_extent(&channel, None, &|v| admits(ys, v))
    });
    match extent {
        Some(extent) => {
            for v in &extent.x {
                fits.add(x, v);
            }
            for v in extent.y.iter().flatten() {
                if let Some(v) = value(*v) {
                    fits.add(y, &v);
                }
            }
        }
        None => {
            for (a, b) in points.iter() {
                fits.add(x, &a);
                fits.add(y, &b);
            }
        }
    }
}

/// A mark's stroke: `width` px, butt caps, round joins (matplotlib's
/// lines), `dash` from offset 0 at the first point drawn.
fn stroke(width: f64, dash: &[f64]) -> kurbo::Stroke {
    kurbo::Stroke::new(width)
        .with_caps(Cap::Butt)
        .with_join(Join::Round)
        .with_dashes(0.0, dash.iter().copied())
}

/// A dash pattern of on and off lengths in px: `None` unless every length
/// is finite and not negative and their sum is positive.
fn dashes(pattern: impl IntoIterator<Item = f64>) -> Option<Vec<f64>> {
    let pattern: Vec<f64> = pattern.into_iter().collect();
    let valid = pattern.iter().all(|v| v.is_finite() && *v >= 0.0);
    (valid && pattern.iter().sum::<f64>() > 0.0).then_some(pattern)
}

/// A positive, finite length in px.
fn positive(px: f64) -> Option<f64> {
    (px.is_finite() && px > 0.0).then_some(px)
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn marks_sit_at_the_middle_of_a_band() {
        let band = scale::band(["a", "b"]).range([0.0, 100.0]);
        assert_eq!(at(&band, &"b"), Some(75.0));
        assert_eq!(at(&band, &"z"), None);
        let log = scale::log().range([0.0, 100.0]);
        assert_eq!(at(&log, &-1.0), None);
        assert_eq!(at(&log, &10.0), Some(100.0));
    }

    #[test]
    fn channel_values_reach_only_f64_domains() {
        assert_eq!(value::<f64>(2.5), Some(2.5));
        assert_eq!(value::<&str>(2.5), None);
        assert!(admits(&scale::log(), 3.0));
        assert!(!admits(&scale::log(), -3.0));
        assert!(!admits(&scale::band(["a"]), 0.0));
    }

    #[test]
    fn invalid_dash_patterns_and_lengths_are_rejected() {
        assert_eq!(dashes([6.0, 3.0]), Some(vec![6.0, 3.0]));
        assert_eq!(dashes([0.0, 0.0]), None);
        assert_eq!(dashes([4.0, -1.0]), None);
        assert_eq!(dashes([f64::NAN]), None);
        assert_eq!(dashes([]), None);
        assert_eq!(positive(2.0), Some(2.0));
        assert_eq!([0.0, -1.0, f64::INFINITY].map(positive), [None; 3]);
    }

    #[test]
    fn mark_strokes_are_butt_capped_with_round_joins() {
        let s = stroke(2.0, &[6.0, 3.0]);
        assert_eq!(
            (s.start_cap, s.end_cap, s.join),
            (Cap::Butt, Cap::Butt, Join::Round)
        );
        assert_eq!(
            (s.dash_offset, s.dash_pattern.as_slice()),
            (0.0, &[6.0, 3.0][..])
        );
    }
}
