//! [`Note`]: text at a point in data coordinates.

use kurbo::{Point, Vec2};
use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::List;
use crate::mark::{Ctx, Mark, at, outside_count, within};
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::{Anchor, Key, Text};
use crate::theme::Theme;

/// Text placed at a point in data coordinates, with an anchor and an
/// offset in px; it takes part in no collision pass.
pub struct Note<X: Position, Y: Position> {
    at: (<X as Kind>::Domain, <Y as Kind>::Domain),
    text: Text,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    anchor: Anchor,
    offset: Vec2,
    fill: Option<Rgb>,
}

impl<X: Position, Y: Position> std::fmt::Debug for Note<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note").finish_non_exhaustive()
    }
}

/// `text` at the point `at` on `x` and `y` (the middle of a band on band
/// scales), its first line's start at the point by default (matplotlib's
/// `text`, `ha="left"`, `va="baseline"`), at the axis-label size (12 pt)
/// in the theme's ink. The point enters fitting; the text itself does not,
/// and no collision pass moves or drops it: notes go where users put
/// them. Like every mark it is clipped to the plot area.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::{dots, note};
/// use rustd3_figure::text::{Anchor, anchor};
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let peak = Anchor { h: anchor::Horizontal::Middle, v: anchor::Vertical::Bottom };
/// fig.panel(0, 0)
///     .mark(dots([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y))
///     .mark(note((1.0, 3.0), "peak", x, y).anchor(peak).offset(0.0, -6.0));
/// ```
pub fn note<X: Position, Y: Position>(
    at: (<X as Kind>::Domain, <Y as Kind>::Domain),
    text: impl Into<Text>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Note<X, Y> {
    Note {
        at,
        text: text.into(),
        x,
        y,
        anchor: Anchor::default(),
        offset: Vec2::ZERO,
        fill: None,
    }
}

impl<X: Position, Y: Position> Note<X, Y> {
    /// Which point of the text's box sits at the note's point; its first
    /// line's start by default. Its lines align by the anchor's horizontal
    /// half.
    #[must_use]
    pub fn anchor(self, a: Anchor) -> Self {
        Self { anchor: a, ..self }
    }

    /// Moves the text `dx` px right and `dy` px down from its point, as
    /// Observable Plot's `dx` and `dy`. An offset that is not finite leaves
    /// it as it was.
    #[must_use]
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        let offset = Vec2::new(dx, dy);
        Self {
            offset: if offset.is_finite() {
                offset
            } else {
                self.offset
            },
            ..self
        }
    }

    /// The text's color, for spans without their own; the theme's ink by
    /// default.
    #[must_use]
    pub fn fill(self, c: Rgb) -> Self {
        Self {
            fill: Some(c),
            ..self
        }
    }

    /// The text's key under `theme`.
    fn key(&self, theme: &Theme) -> Key {
        let fill = self.fill.unwrap_or_else(|| theme.get_ink());
        Key::new(&self.text, &theme.note(self.anchor.h, fill))
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Note<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        fits.add(self.x, &self.at.0);
        fits.add(self.y, &self.at.1);
        let key = self.key(fits.theme());
        fits.label(key);
    }

    /// The text's glyph runs at its point plus the offset, when the point
    /// is placed on both scales and lies within the guard band around the
    /// plot area; nothing otherwise, or when typst rejected the text.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = ctx.band();
        let Some(p) = at(&sx, &self.at.0).zip(at(&sy, &self.at.1)) else {
            return;
        };
        let p = Point::new(p.0, p.1) + self.offset;
        if !within(band, p) {
            return;
        }
        if let Some(shaped) = ctx.label(&self.key(ctx.theme())) {
            shaped.place(list, p, self.anchor, 0.0);
        }
    }

    /// A point outside a band or point domain, which draws nothing.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        let across = ctx
            .scale(self.x)
            .is_some_and(|s| s.position(&self.at.0).is_none());
        let up = ctx
            .scale(self.y)
            .is_some_and(|s| s.position(&self.at.1).is_none());
        outside_count(self.x.index(), usize::from(across), issues);
        outside_count(self.y.index(), usize::from(up), issues);
    }
}
