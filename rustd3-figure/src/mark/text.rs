//! [`Note`]: text at a point in data coordinates, and [`Avoid`], how notes
//! make way for each other.

use kurbo::{Point, Vec2};
use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::List;
use crate::mark::{Ctx, Mark, at, outside_count, positive, within};
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::{Anchor, Key, Style, Text};
use crate::theme::Theme;

/// What a note does when its box would overlap that of a note drawn
/// before it in the same panel ([`Note::avoid`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Avoid {
    /// The note steps up or down by its own height, to the nearest place
    /// that is free.
    Step,
    /// The note is not drawn.
    Drop,
}

/// Text placed at a point in data coordinates, with an anchor and an
/// offset in px. It takes part in no layout pass, and makes way for other
/// notes only when told to ([`Note::avoid`]).
pub struct Note<X: Position, Y: Position> {
    at: (<X as Kind>::Domain, <Y as Kind>::Domain),
    text: Text,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    anchor: Anchor,
    offset: Vec2,
    fill: Option<Rgb>,
    size: Option<f64>,
    bold: bool,
    font: Option<&'static str>,
    avoid: Option<Avoid>,
}

impl<X: Position, Y: Position> std::fmt::Debug for Note<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note").finish_non_exhaustive()
    }
}

/// `text` at the point `at` on `x` and `y` (the middle of a band on band
/// scales), at the axis-label size (12 pt) in the theme's ink. By default
/// the first line's start sits at the point (matplotlib's `text`,
/// `ha="left"`, `va="baseline"`). The point enters fitting but the text
/// does not, and nothing moves or drops it unless [`Note::avoid`] says so,
/// so notes go where users put them. Like every mark it is clipped to the
/// plot area.
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
        size: None,
        bold: false,
        font: None,
        avoid: None,
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
    pub fn color(self, c: Rgb) -> Self {
        Self {
            fill: Some(c),
            ..self
        }
    }

    /// The text's size in px (typst's `pt`, as in a label's markup); by
    /// default the axis-label size, 12 pt (16 px at seaborn's notebook). A
    /// size that is not finite and positive leaves it as it was.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::note;
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// fig.panel(0, 0).mark(note((1.0, 3.0), "peak", x, y).size(9.0).bold(true));
    /// ```
    #[must_use]
    pub fn size(self, px: f64) -> Self {
        Self {
            size: positive(px).or(self.size),
            ..self
        }
    }

    /// With the whole text bold, or regular, the default; a bold span stays
    /// bold.
    #[must_use]
    pub fn bold(self, yes: bool) -> Self {
        Self { bold: yes, ..self }
    }

    /// The text's font family, as its name table gives it; the theme's by
    /// default ([`Theme::font`]). In DejaVu Sans Mono, bundled with the
    /// `fonts` feature, padded columns line up. An empty name leaves it as
    /// it was.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::note;
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// let table = "peak    3.00\nmean   12.50";
    /// fig.panel(0, 0).mark(note((1.0, 3.0), table, x, y).font("DejaVu Sans Mono"));
    /// ```
    #[must_use]
    pub fn font(self, family: &'static str) -> Self {
        Self {
            font: Some(family).filter(|f| !f.is_empty()).or(self.font),
            ..self
        }
    }

    /// Makes way for the notes of the panel drawn before this one that
    /// also avoid. A note without this call is drawn where it is put and
    /// is avoided by none.
    ///
    /// With [`Avoid::Step`], a note whose box comes within 1 px of an
    /// earlier one moves up or down by its height plus 2 px, as often as
    /// it takes and at most four times, to the nearest free place inside
    /// the plot area. A note that finds none stays where it was put. With
    /// [`Avoid::Drop`] it is not drawn. Marks draw in insertion order, so
    /// the notes added first keep their places.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::{Avoid, dots, note};
    ///
    /// let points = [(1.0, 1.10), (1.1, 1.12), (2.0, 1.4)];
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// fig.panel(0, 0).mark(dots(points, x, y));
    /// for (at, value) in points {
    ///     let label = note((at, value), format!("{value:.2}"), x, y);
    ///     fig.panel(0, 0).mark(label.offset(6.0, 0.0).avoid(Avoid::Step));
    /// }
    /// ```
    #[must_use]
    pub fn avoid(self, a: Avoid) -> Self {
        Self {
            avoid: Some(a),
            ..self
        }
    }

    /// Where the note is drawn when its box at `p` is `bounds`, among the
    /// boxes `taken` in a plot area `area`: `p`, moved as its [`Avoid`]
    /// says, or `None` for a note that is dropped. The box it takes is
    /// added to `taken`.
    fn settle(
        &self,
        p: Point,
        bounds: kurbo::Rect,
        area: kurbo::Rect,
        taken: &mut Vec<kurbo::Rect>,
    ) -> Option<Point> {
        let Some(avoid) = self.avoid else {
            return Some(p);
        };
        let free = |b: kurbo::Rect| !taken.iter().any(|t| overlaps(*t, b));
        let step = bounds.height() + STEP_GAP;
        let moved = match avoid {
            _ if free(bounds) => Some(Vec2::ZERO),
            Avoid::Drop => None,
            Avoid::Step => {
                let tries = (1..=STEPS).flat_map(|k| [-f64::from(k), f64::from(k)]);
                let inside = |b: kurbo::Rect| area.union(b) == area;
                let at = tries
                    .map(|k| Vec2::new(0.0, k * step))
                    .find(|by| free(bounds + *by) && inside(bounds + *by));
                Some(at.unwrap_or(Vec2::ZERO))
            }
        }?;
        taken.push(bounds + moved);
        Some(p + moved)
    }

    /// The text's key under `theme`.
    fn key(&self, theme: &Theme) -> Key {
        let fill = self.fill.unwrap_or_else(|| theme.get_ink());
        let style = theme.note(self.anchor.h, fill);
        let style = Style {
            size: self.size.unwrap_or(style.size),
            bold: self.bold,
            font: self.font.or(style.font),
            ..style
        };
        Key::new(&self.text, &style)
    }
}

/// How often a stepping note moves each way before it gives up.
const STEPS: u32 = 4;

/// The space a stepping note leaves between its box and the one it steps
/// past, in px.
const STEP_GAP: f64 = 2.0;

/// Whether boxes `a` and `b` come within 1 px of each other.
fn overlaps(a: kurbo::Rect, b: kurbo::Rect) -> bool {
    let a = a.inflate(1.0, 1.0);
    a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1
}

impl<'a, X: Position, Y: Position> Mark<'a> for Note<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        fits.add(self.x, &self.at.0);
        fits.add(self.y, &self.at.1);
        let key = self.key(fits.get_theme());
        fits.label(key);
    }

    /// The text's glyph runs at its point plus the offset, moved as its
    /// [`Avoid`] says, when the point is placed on both scales and lies
    /// within the guard band around the plot area. Nothing otherwise, for a
    /// note that is dropped, or when typst rejected the text.
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
        let Some(shaped) = ctx.get_label(&self.key(ctx.get_theme())) else {
            return;
        };
        let bounds = shaped.bounds(self.anchor) + p.to_vec2();
        let area = ctx.get_rect().with_origin(Point::ORIGIN);
        if let Some(p) = self.settle(p, bounds, area, &mut ctx.taken()) {
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
        outside_count(self.x.get_index(), usize::from(across), issues);
        outside_count(self.y.get_index(), usize::from(up), issues);
    }
}
