//! [`Rule`]: a line across the plot area at one value.

use kurbo::{Point, Shape};
use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark, at, dashes, positive, stroke};
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::{Anchor, Key, Text, anchor};
use crate::theme::Theme;

/// A line across the plot area at one value of one scale: vertical for
/// [`rule_x`], horizontal for [`rule_y`], with an optional label at its
/// top or right end.
pub struct Rule<S: Position> {
    value: <S as Kind>::Domain,
    scale: ScaleRef<S>,
    along: Along,
    color: Option<Rgb>,
    width: Option<f64>,
    dash: Vec<f64>,
    label: Option<Text>,
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
    /// first color and line width, solid, unlabelled.
    fn new(value: <S as Kind>::Domain, scale: ScaleRef<S>, along: Along) -> Self {
        Rule {
            value,
            scale,
            along,
            color: None,
            width: None,
            dash: Vec::new(),
            label: None,
        }
    }

    /// The line's and the label's color; by default the theme palette's
    /// first.
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Some(c),
            ..self
        }
    }

    /// The line's width in px; by default the theme's line width. A width
    /// that is not finite and positive leaves it as it was.
    pub fn width(self, px: f64) -> Self {
        Self {
            width: positive(px).or(self.width),
            ..self
        }
    }

    /// Dashes: on and off lengths in px, from the line's start. A pattern
    /// with a negative or non-finite length, or none longer than 0, leaves
    /// the line as it was.
    pub fn dash(self, pattern: impl IntoIterator<Item = f64>) -> Self {
        let dash = dashes(pattern).unwrap_or(self.dash);
        Self { dash, ..self }
    }

    /// Text at the line's top end (`rule_x`, beside the line) or right end
    /// (`rule_y`, above it), at tick-label size in the line's color.
    pub fn label(self, text: impl Into<Text>) -> Self {
        Self {
            label: Some(text.into()),
            ..self
        }
    }

    /// The label's key under `theme`, if the rule has a label.
    fn key(&self, theme: &Theme) -> Option<Key> {
        let align = match self.along {
            Along::X => anchor::Horizontal::Start,
            Along::Y => anchor::Horizontal::End,
        };
        let color = self.color.unwrap_or_else(|| theme.color());
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
        if let Some(key) = self.key(fits.theme()) {
            fits.label(key);
        }
    }

    /// A stroke across the plot area and the label's runs, when the value
    /// is placed within half the line's width of the plot area; nothing
    /// otherwise. On raster targets the line covers whole device pixels.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let theme = ctx.theme();
        let (w, h) = (ctx.rect().width(), ctx.rect().height());
        let width = self.width.unwrap_or(theme.line_width());
        let (pos, length) = match self.along {
            Along::X => (at(&ctx.x(self.scale), &self.value), w),
            Along::Y => (at(&ctx.y(self.scale), &self.value), h),
        };
        let Some(pos) = pos.filter(|p| (-width / 2.0..=length + width / 2.0).contains(p)) else {
            return;
        };
        let snap = ctx.snap();
        let (pos, width) = (snap.line(pos, width), snap.width(width));
        let (line, label, corner) = match self.along {
            Along::X => (
                kurbo::Line::new((pos, 0.0), (pos, h)),
                Point::new(pos + theme.tick_pad(), theme.tick_pad()),
                (anchor::Horizontal::Start, anchor::Vertical::Top),
            ),
            Along::Y => (
                kurbo::Line::new((0.0, pos), (w, pos)),
                Point::new(w - theme.tick_pad(), pos - theme.tick_pad()),
                (anchor::Horizontal::End, anchor::Vertical::Bottom),
            ),
        };
        let color = self.color.unwrap_or_else(|| theme.color());
        list.commands.push(Command::Stroke {
            path: line.to_path(0.1),
            style: stroke(width, &self.dash),
            paint: list::brush(color),
        });
        if let Some(key) = self.key(theme)
            && let Some(shaped) = ctx.label(&key)
        {
            let (h, v) = corner;
            shaped.place(list, label, Anchor { h, v }, 0.0);
        }
    }

    /// A value outside a band or point domain, which draws nothing.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if ctx.scale(self.scale).position(&self.value).is_none() {
            issues.push(Issue::OutsideDomain {
                handle: self.scale.index(),
                count: 1,
            });
        }
    }
}
