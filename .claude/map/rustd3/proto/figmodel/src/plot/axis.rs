//! Axes are values: define one per quantity, then place it
//! on as many panels as share that quantity.

use super::draw::{self, Anchor, List};
use super::text::Text;
use super::{Ctx, Paint, Resolved, ScaleRef, Ticks};
use d3::axis::Orient;
use d3::color::Rgb;
use kurbo::{Affine, BezPath};
use std::rc::Rc;

type Format<D> = Rc<dyn Fn(&D) -> String>;
type Tint = Rc<dyn Fn(&Ctx) -> Rgb>;

pub struct Axis<S: Ticks> {
    orient: Orient,
    scale: ScaleRef<S>,
    values: Option<Rc<[S::Domain]>>,
    format: Option<Format<S::Domain>>,
    label: Option<Text>,
    count: Option<usize>,
    labels: Option<bool>,
    outward: Option<f64>,
    tint: Option<Tint>,
}

impl<S: Ticks> Clone for Axis<S> {
    fn clone(&self) -> Self {
        Self {
            orient: self.orient,
            scale: self.scale,
            values: self.values.clone(),
            format: self.format.clone(),
            label: self.label.clone(),
            count: self.count,
            labels: self.labels,
            outward: self.outward,
            tint: self.tint.clone(),
        }
    }
}

impl<S: Ticks> From<&Axis<S>> for Axis<S> {
    fn from(a: &Axis<S>) -> Self {
        a.clone()
    }
}

fn axis<S: Ticks>(orient: Orient, scale: ScaleRef<S>) -> Axis<S> {
    Axis {
        orient,
        scale,
        values: None,
        format: None,
        label: None,
        count: None,
        labels: None,
        outward: None,
        tint: None,
    }
}

pub fn bottom<S: Ticks>(s: ScaleRef<S>) -> Axis<S> {
    axis(Orient::Bottom, s)
}

pub fn left<S: Ticks>(s: ScaleRef<S>) -> Axis<S> {
    axis(Orient::Left, s)
}

pub fn right<S: Ticks>(s: ScaleRef<S>) -> Axis<S> {
    axis(Orient::Right, s)
}

pub fn top<S: Ticks>(s: ScaleRef<S>) -> Axis<S> {
    axis(Orient::Top, s)
}

impl<S: Ticks> Axis<S> {
    /// d3's `axis.tickValues(…)`.
    pub fn values(
        self,
        v: impl IntoIterator<Item = S::Domain>,
    ) -> Self {
        let values = Some(v.into_iter().collect());
        Self { values, ..self }
    }

    /// d3's `axis.tickFormat(…)`.
    pub fn format(
        self,
        f: impl Fn(&S::Domain) -> String + 'static,
    ) -> Self {
        let format: Format<S::Domain> = Rc::new(f);
        Self { format: Some(format), ..self }
    }

    /// d3's `axis.ticks(count)`. Without it, the count
    /// follows the axis length.
    pub fn ticks(self, count: usize) -> Self {
        Self { count: Some(count), ..self }
    }

    /// A title along the axis.
    pub fn label(self, label: impl Into<Text>) -> Self {
        Self { label: Some(label.into()), ..self }
    }

    /// Moves the axis this many pixels away from the
    /// panel, as matplotlib's outward spines. Without it,
    /// layout stacks axes on one side automatically.
    pub fn outward(self, px: f64) -> Self {
        Self { outward: Some(px), ..self }
    }

    /// Colors line, ticks and labels through a palette,
    /// to tie a twin axis to its series.
    pub fn color<C: Paint>(
        self,
        c: ScaleRef<C>,
        key: C::Domain,
    ) -> Self {
        let tint: Tint = Rc::new(move |ctx: &Ctx| {
            let s = ctx.raw(c);
            s.paint(&key).unwrap_or(super::GRAY)
        });
        Self { tint: Some(tint), ..self }
    }

    /// The same axis with or without tick labels and
    /// title. Without it, layout labels a shared axis on
    /// the outer panels only.
    pub fn labels(&self, labels: bool) -> Self {
        Self { labels: Some(labels), ..self.clone() }
    }
}

/// What layout and drawing need from an axis, erased.
pub(crate) trait Draw {
    fn orient(&self) -> Orient;
    fn scale(&self) -> usize;
    fn outward(&self) -> Option<f64>;
    fn labels(&self) -> Option<bool>;
    fn title(&self) -> Option<&Text>;
    /// A tick count the user fixed, if any.
    fn count(&self) -> Option<usize>;
    /// Tick labels, from the resolved domain alone.
    fn texts(&self, r: &Resolved, count: usize) -> Vec<String>;
    /// Each tick's position along the panel, aligned
    /// with `texts`.
    fn along(&self, c: &Ctx, count: usize) -> Vec<Option<f64>>;
    fn ink(&self, c: &Ctx) -> Rgb;
    fn tinted(&self) -> bool;
}

impl<S: Ticks> Axis<S> {
    fn tick_values(&self, s: &S, count: usize) -> Vec<S::Domain> {
        match &self.values {
            Some(v) => v.to_vec(),
            None => s.tick_values(count),
        }
    }
}

impl<S: Ticks> Draw for Axis<S> {
    fn orient(&self) -> Orient {
        self.orient
    }
    fn scale(&self) -> usize {
        self.scale.index
    }
    fn outward(&self) -> Option<f64> {
        self.outward
    }
    fn labels(&self) -> Option<bool> {
        self.labels
    }
    fn title(&self) -> Option<&Text> {
        self.label.as_ref()
    }
    fn count(&self) -> Option<usize> {
        self.count
    }
    fn texts(&self, r: &Resolved, count: usize) -> Vec<String> {
        let s = r.raw(self.scale);
        self.tick_values(&s, count)
            .iter()
            .map(|v| match &self.format {
                Some(f) => f(v),
                None => s.tick_label(v, count),
            })
            .collect()
    }
    fn along(&self, c: &Ctx, count: usize) -> Vec<Option<f64>> {
        let s = if across(self.orient) {
            c.x(self.scale)
        } else {
            c.y(self.scale)
        };
        let half = s.bandwidth() / 2.0;
        self.tick_values(&s, count)
            .iter()
            .map(|v| s.position(v).map(|p| p + half))
            .collect()
    }
    fn tinted(&self) -> bool {
        self.tint.is_some()
    }

    fn ink(&self, c: &Ctx) -> Rgb {
        self.tint
            .as_ref()
            .map_or(c.r.theme.ink(), |t| t(c))
    }
}

/// Line height of 10 px text, and the gap before a title.

/// Approximate width of 10 px sans text; typst measures
/// it in the real backend.
/// The tick count for an axis this long when the user
/// did not fix one: about one tick per 80 px across and
/// per 35 px up, Observable Plot's defaults.
pub(crate) fn auto_count(o: Orient, len: f64) -> usize {
    let per = if across(o) { 80.0 } else { 35.0 };
    ((len / per).round() as usize).max(2)
}

pub(crate) fn text_w(r: &Resolved, s: &str) -> f64 {
    let t = Text::from(s);
    r.measure.0.width(&t, r.theme.tick_text(), false)
}

fn across(o: Orient) -> bool {
    matches!(o, Orient::Bottom | Orient::Top)
}

/// How far the axis reaches past its own line.
pub(crate) fn thickness(
    a: &dyn Draw,
    r: &Resolved,
    labeled: bool,
    count: usize,
) -> f64 {
    let t = &r.theme;
    if !labeled {
        return t.tick();
    }
    let label = if across(a.orient()) {
        1.2 * t.tick_text()
    } else {
        let texts = a.texts(r, count);
        let w = texts.iter().map(|s| text_w(r, s));
        w.fold(0.0, f64::max)
    };
    let title = if a.title().is_some() {
        t.label_pad() + 1.2 * t.label_text()
    } else {
        0.0
    };
    t.tick() + t.tick_pad() + label + title
}

/// A tick label: where it is drawn, relative to the axis
/// origin, and its box in figure coordinates.
pub(crate) struct Label {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub anchor: &'static str,
    pub bbox: [f64; 4],
}

/// The axis origin in panel coordinates.
fn origin(o: Orient, c: &Ctx, offset: f64) -> (f64, f64) {
    let r = c.rect;
    match o {
        Orient::Bottom => (0.0, r.h + offset),
        Orient::Right => (r.w + offset, 0.0),
        Orient::Top => (0.0, -offset),
        Orient::Left => (-offset, 0.0),
    }
}

pub(crate) fn labels(
    a: &dyn Draw,
    c: &Ctx,
    offset: f64,
    count: usize,
) -> Vec<Label> {
    let (ox, oy) = origin(a.orient(), c, offset);
    let (fx, fy) = (c.rect.x + ox, c.rect.y + oy);
    let t = &c.r.theme;
    let (d, s) = (t.tick() + t.tick_pad(), t.tick_text());
    a.texts(c.r, count)
        .into_iter()
        .zip(a.along(c, count))
        .filter_map(|(text, p)| {
            let p = p?;
            let w = text_w(c.r, &text);
            let (x, y, anchor, bx) = match a.orient() {
                Orient::Bottom => (p, d + 0.9 * s, "middle", p - w / 2.0),
                Orient::Top => (p, -d - 0.2 * s, "middle", p - w / 2.0),
                Orient::Left => (-d, p + 0.35 * s, "end", -d - w),
                Orient::Right => (d, p + 0.35 * s, "start", d),
            };
            let bbox = [fx + bx, fy + y - 0.9 * s, fx + bx + w, fy + y + 0.2 * s];
            Some(Label { text, x, y, anchor, bbox })
        })
        .collect()
}

pub(crate) fn draw(
    a: &dyn Draw,
    c: &Ctx,
    offset: f64,
    labeled: bool,
    count: usize,
    keep: &[bool],
    o: &mut List,
) {
    use Orient::*;
    let orient = a.orient();
    let r = c.rect;
    let (ox, oy) = origin(orient, c, offset);
    let ink = a.ink(c);
    let k = if matches!(orient, Top | Left) { -1.0 } else { 1.0 };
    let len = if across(orient) { r.w } else { r.h };
    let th = &c.r.theme;
    let mut ticks = BezPath::new();
    let tick = th.tick();
    for p in a.along(c, count).into_iter().flatten() {
        if tick == 0.0 {
            break;
        }
        let t = tick * k;
        if across(orient) {
            ticks.move_to((p, 0.0));
            ticks.line_to((p, t));
        } else {
            ticks.move_to((0.0, p));
            ticks.line_to((t, p));
        }
    }
    o.push(Affine::translate((ox, oy)));
    // The axis line takes the style's edge color, or
    // the tint of a twin axis.
    let edge = if a.tinted() { Some(ink) } else { th.edge() };
    if let Some(e) = edge {
        let line = if across(orient) {
            kurbo::Line::new((0.0, 0.0), (len, 0.0))
        } else {
            kurbo::Line::new((0.0, 0.0), (0.0, len))
        };
        o.stroke(line, th.axis_width(), None, e);
    }
    if tick > 0.0 {
        o.stroke(ticks, th.axis_width(), None, ink);
    }
    if labeled {
        let ls = labels(a, c, offset, count);
        for (l, _) in ls.iter().zip(keep).filter(|(_, k)| **k) {
            let anchor = match l.anchor {
                "end" => Anchor::End,
                "start" => Anchor::Start,
                _ => Anchor::Middle,
            };
            o.label(
                draw::Label::new(l.text.as_str(), l.x, l.y)
                    .anchor(anchor)
                    .size(th.tick_text() as f32)
                    .fill(ink),
            );
        }
        if let Some(t) = a.title() {
            let reach = if across(orient) {
                1.2 * th.tick_text()
            } else {
                let w = ls.iter().map(|l| text_w(c.r, &l.text));
                w.fold(0.0, f64::max)
            };
            let at = th.tick() + th.tick_pad() + reach + th.label_pad();
            // The baseline sits outside the glyphs' body.
            let lt = th.label_text();
            let base = if k < 0.0 { -at } else { at + 0.9 * lt };
            let l = if across(orient) {
                draw::Label::new(t.clone(), len / 2.0, base)
            } else {
                draw::Label::new(t.clone(), base, len / 2.0)
                    .angle(-std::f64::consts::FRAC_PI_2)
            };
            o.label(l.anchor(Anchor::Middle).size(lt as f32).fill(ink));
        }
    }
    o.pop();
}

/// Grid lines at the axis's ticks, across the panel.
pub(crate) fn grid(
    a: &dyn Draw,
    c: &Ctx,
    count: usize,
    color: Rgb,
    width: f64,
    o: &mut List,
) {
    let r = c.rect;
    let mut d = BezPath::new();
    for p in a.along(c, count).into_iter().flatten() {
        if across(a.orient()) {
            d.move_to((p, 0.0));
            d.line_to((p, r.h));
        } else {
            d.move_to((0.0, p));
            d.line_to((r.w, p));
        }
    }
    o.stroke(d, width, None, color);
}
