//! [`Rule`]: a line across the plot area at one value, and [`Place`], where
//! along it its label sits.

use kurbo::{Point, Shape, Vec2};
use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark, at, dashes, positive, stroke};
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::{Anchor, Key, Text, anchor};
use crate::theme::Theme;

/// A line across the plot area at one value of one scale: vertical for
/// [`rule_x`], horizontal for [`rule_y`], with an optional label beside it.
pub struct Rule<S: Position> {
    value: <S as Kind>::Domain,
    scale: ScaleRef<S>,
    along: Along,
    color: Option<Rgb>,
    width: Option<f64>,
    dash: Vec<f64>,
    label: Option<Text>,
    place: Place,
    offset: Vec2,
}

impl<S: Position> std::fmt::Debug for Rule<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rule").finish_non_exhaustive()
    }
}

/// Where along a rule its label sits: at the start of the line (the bottom
/// of a vertical rule, the left end of a horizontal one), its middle, or its
/// end (the top, the right end), the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Place {
    /// The bottom of a vertical rule, the left end of a horizontal one.
    Start,
    /// The middle of the line.
    Middle,
    /// The top of a vertical rule, the right end of a horizontal one.
    #[default]
    End,
}

/// The direction a rule's value is measured in.
#[derive(Clone, Copy)]
enum Along {
    /// An x value: a vertical line.
    X,
    /// A y value: a horizontal line.
    Y,
}

/// A vertical line across the plot area at `value` of `x`.
pub fn rule_x<X: Position>(value: <X as Kind>::Domain, x: ScaleRef<X>) -> Rule<X> {
    Rule::new(value, x, Along::X)
}

/// A horizontal line across the plot area at `value` of `y`. Its value
/// enters fitting, so a fitted handle grows to show it.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::color::Rgb;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::rule_y;
///
/// let mut fig = Figure::new();
/// let count = fig.fit(scale::linear());
/// fig.panel(0, 0).mark(
///     rule_y(20.0, count)
///         .dash([4.0, 3.0])
///         .color(Rgb::hex(0x262626))
///         .label("min. count"),
/// );
/// ```
pub fn rule_y<Y: Position>(value: <Y as Kind>::Domain, y: ScaleRef<Y>) -> Rule<Y> {
    Rule::new(value, y, Along::Y)
}

impl<S: Position> Rule<S> {
    /// A rule at `value` of `scale` along `along`, in the theme palette's
    /// first color and line width, solid, unlabeled.
    fn new(value: <S as Kind>::Domain, scale: ScaleRef<S>, along: Along) -> Self {
        Rule {
            value,
            scale,
            along,
            color: None,
            width: None,
            dash: Vec::new(),
            label: None,
            place: Place::End,
            offset: Vec2::ZERO,
        }
    }

    /// The line's and the label's color; by default the theme palette's
    /// first.
    #[must_use]
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Some(c),
            ..self
        }
    }

    /// The line's width in px; by default the theme's line width. A width
    /// that is not finite and positive leaves it as it was.
    #[must_use]
    pub fn width(self, px: f64) -> Self {
        Self {
            width: positive(px).or(self.width),
            ..self
        }
    }

    /// Dashes: on and off lengths in px, from the line's start. A pattern
    /// with a negative or non-finite length, or none longer than 0, leaves
    /// the line as it was.
    #[must_use]
    pub fn dash(self, pattern: impl IntoIterator<Item = f64>) -> Self {
        let dash = dashes(pattern).unwrap_or(self.dash);
        Self { dash, ..self }
    }

    /// Text beside the line, at tick-label size in the line's color: right
    /// of a vertical rule, above a horizontal one, at its top or right end
    /// unless [`label_at`](Rule::label_at) places it elsewhere.
    #[must_use]
    pub fn label(self, text: impl Into<Text>) -> Self {
        Self {
            label: Some(text.into()),
            ..self
        }
    }

    /// Where along the line the label sits; its end (the top or the right
    /// end) by default. A label at the start or end sits a tick pad in from
    /// the plot area's edge.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::{Place, rule_y};
    ///
    /// let mut fig = Figure::new();
    /// let y = fig.fit(scale::linear());
    /// fig.panel(0, 0).mark(rule_y(2.5, y).label("limit").label_at(Place::Start));
    /// ```
    #[must_use]
    pub fn label_at(self, place: Place) -> Self {
        Self { place, ..self }
    }

    /// Moves the label `dx` px right and `dy` px down from where
    /// [`label_at`](Rule::label_at) puts it, as a note's offset moves a
    /// note: a positive `dy` puts a horizontal rule's label below the line.
    /// An offset that is not finite leaves it as it was.
    #[must_use]
    pub fn label_offset(self, dx: f64, dy: f64) -> Self {
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

    /// The label's key under `theme`, if the rule has a label.
    fn key(&self, theme: &Theme) -> Option<Key> {
        // Lines align as the label's box: by its start beside a vertical
        // rule, by where it sits along a horizontal one.
        let align = match (self.along, self.place) {
            (Along::X, _) | (Along::Y, Place::Start) => anchor::Horizontal::Start,
            (Along::Y, Place::Middle) => anchor::Horizontal::Middle,
            (Along::Y, Place::End) => anchor::Horizontal::End,
        };
        let color = self.color.unwrap_or_else(|| theme.get_color());
        let text = self.label.as_ref()?;
        Some(Key::new(text, &theme.mark_label(align, color)))
    }
}

impl<'a, S: Position> Mark<'a> for Rule<S> {
    fn fit(&self, fits: &mut Fits) {
        fits.add(self.scale, &self.value);
        match self.along {
            Along::X => fits.across(self.scale),
            Along::Y => {}
        }
        if let Some(key) = self.key(fits.get_theme()) {
            fits.label(key);
        }
    }

    /// A stroke across the plot area and the label's runs, when the value
    /// is placed within half the line's width of the plot area; nothing
    /// otherwise. On raster targets the line covers whole device pixels.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let theme = ctx.get_theme();
        let (w, h) = (ctx.get_rect().width(), ctx.get_rect().height());
        let width = self.width.unwrap_or(theme.get_line_width());
        let (pos, length) = match self.along {
            Along::X => (ctx.x(self.scale).and_then(|s| at(&s, &self.value)), w),
            Along::Y => (ctx.y(self.scale).and_then(|s| at(&s, &self.value)), h),
        };
        let Some(pos) = pos.filter(|p| (-width / 2.0..=length + width / 2.0).contains(p)) else {
            return;
        };
        let snap = ctx.snap();
        let (pos, width) = (snap.line(pos, width), snap.width(width));
        let pad = theme.tick_pad();
        let (line, label, corner) = match self.along {
            Along::X => {
                let (y, v) = match self.place {
                    Place::Start => (h - pad, anchor::Vertical::Bottom),
                    Place::Middle => (h / 2.0, anchor::Vertical::Middle),
                    Place::End => (pad, anchor::Vertical::Top),
                };
                (
                    kurbo::Line::new((pos, 0.0), (pos, h)),
                    Point::new(pos + pad, y),
                    (anchor::Horizontal::Start, v),
                )
            }
            Along::Y => {
                let (x, across) = match self.place {
                    Place::Start => (pad, anchor::Horizontal::Start),
                    Place::Middle => (w / 2.0, anchor::Horizontal::Middle),
                    Place::End => (w - pad, anchor::Horizontal::End),
                };
                (
                    kurbo::Line::new((0.0, pos), (w, pos)),
                    Point::new(x, pos - pad),
                    (across, anchor::Vertical::Bottom),
                )
            }
        };
        let color = self.color.unwrap_or_else(|| theme.get_color());
        list.commands.push(Command::Stroke {
            path: line.to_path(0.1),
            style: stroke(width, &self.dash),
            paint: list::brush(color),
        });
        if let Some(key) = self.key(theme)
            && let Some(shaped) = ctx.get_label(&key)
        {
            let (h, v) = corner;
            shaped.place(list, label + self.offset, Anchor { h, v }, 0.0);
        }
    }

    /// A value outside a band or point domain, which draws nothing.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if ctx
            .scale(self.scale)
            .is_some_and(|s| s.position(&self.value).is_none())
        {
            issues.push(Issue::OutsideDomain {
                handle: self.scale.get_index(),
                count: 1,
            });
        }
    }
}
