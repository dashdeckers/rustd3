//! Figure axes: values built once per quantity from a handle and placed on
//! every panel that shows it. `axis::bottom(h)` takes a figure handle,
//! where core's `axis::bottom(&x)` takes the scale itself.
//!
//! An axis needs a scale that places values and has ticks; band keys are
//! labeled through `Display`, so an axis on a band whose key has none
//! does not compile (E0277, `Bin` doesn't implement `Display`):
//!
//! ```compile_fail,E0277
//! use rustd3_core::scale;
//! use rustd3_figure::{Figure, axis};
//!
//! #[derive(Clone, PartialEq)]
//! struct Bin(u8);
//!
//! let mut fig = Figure::new();
//! let speed = fig.scale(scale::band((0..20).map(Bin)));
//! fig.panel(0, 0).axis(axis::bottom(speed));
//! ```
//!
//! ```
//! use std::fmt;
//!
//! use rustd3_core::scale;
//! use rustd3_figure::{Figure, axis};
//!
//! #[derive(Clone, PartialEq)]
//! struct Bin(u8);
//!
//! impl fmt::Display for Bin {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "{}", 10 * u32::from(self.0))
//!     }
//! }
//!
//! let mut fig = Figure::new();
//! let speed = fig.scale(scale::band((0..20).map(Bin)));
//! fig.panel(0, 0).axis(axis::bottom(speed));
//! ```

mod draw;
mod labels;
mod offset;
mod placed;

use std::any::Any;
use std::rc::Rc;

pub(crate) use draw::Draw;
pub(crate) use labels::Labels;
use offset::Notation;
pub(crate) use placed::Placed;
use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};
use rustd3_core::ticks;

use crate::figure::Resolved;
use crate::layout::collide;
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::text::Text;

/// A label for each tick value, in place of the scale's.
type Format<'a, D> = Rc<dyn Fn(&D) -> String + 'a>;

/// The color of a twin axis, read from the resolved palette.
type Tint<'a> = Rc<dyn Fn(&Resolved) -> Option<Rgb> + 'a>;

/// A key of two levels for a band or point scale: an outer one, shared by
/// a group of neighboring keys, and an inner one. It is written as its
/// inner level, so an axis's ticks show that, and [`Axis::group`] names
/// the outer.
///
/// ```
/// use rustd3_figure::axis::Nested;
///
/// assert_eq!(Nested("2026", "Q1").to_string(), "Q1");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nested<O, I>(pub O, pub I);

impl<O, I: std::fmt::Display> std::fmt::Display for Nested<O, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f)
    }
}

/// An axis of one quantity: its side, its handle, and how its ticks are
/// chosen and labeled. It is built once and placed on many panels
/// (`From<&Axis>` clones it). Its closures are shared, never boxed per
/// panel, and may borrow for `'a`.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::{Figure, axis};
///
/// let mut fig = Figure::new();
/// let y = fig.fit(scale::linear());
/// let vib = axis::left(y).ticks(5).label("vibration RMS [m/s²]");
/// fig.panel(0, 0).axis(&vib);
/// fig.panel(1, 0).axis(&vib);
/// ```
pub struct Axis<'a, S: Kind> {
    orient: Orient,
    scale: ScaleRef<S>,
    values: Option<Rc<[S::Domain]>>,
    format: Option<Format<'a, S::Domain>>,
    group: Option<Format<'a, S::Domain>>,
    count: Option<usize>,
    label: Option<Text>,
    outward: Option<f64>,
    labels: Option<bool>,
    keep_label: bool,
    offset: bool,
    minor: bool,
    rotate: f64,
    tint: Option<Tint<'a>>,
    palette: Option<ScaleRef<()>>,
}

impl<'a, S: Kind> std::fmt::Debug for Axis<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Axis").finish_non_exhaustive()
    }
}

impl<S: Kind> Clone for Axis<'_, S> {
    fn clone(&self) -> Self {
        Axis {
            orient: self.orient,
            scale: self.scale,
            values: self.values.clone(),
            format: self.format.clone(),
            group: self.group.clone(),
            count: self.count,
            label: self.label.clone(),
            outward: self.outward,
            labels: self.labels,
            keep_label: self.keep_label,
            offset: self.offset,
            minor: self.minor,
            rotate: self.rotate,
            tint: self.tint.clone(),
            palette: self.palette,
        }
    }
}

/// One axis value placed on many panels.
impl<'a, S: Kind> From<&Axis<'a, S>> for Axis<'a, S> {
    fn from(axis: &Axis<'a, S>) -> Self {
        axis.clone()
    }
}

/// An axis below the panel: ticks down, labels centered under them.
#[must_use]
pub fn bottom<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Bottom, h)
}

/// An axis left of the panel: ticks left, labels right-aligned to them.
#[must_use]
pub fn left<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Left, h)
}

/// An axis right of the panel: ticks right, labels left-aligned to them.
#[must_use]
pub fn right<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Right, h)
}

/// An axis above the panel: ticks up, labels centered over them.
#[must_use]
pub fn top<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Top, h)
}

impl<'a, S: Kind> Axis<'a, S> {
    /// d3's `axis.tickValues(…)`: these values instead of the scale's
    /// ticks; default labels still follow the tick count.
    #[must_use]
    pub fn values(self, values: impl IntoIterator<Item = S::Domain>) -> Self {
        Axis {
            values: Some(values.into_iter().collect()),
            ..self
        }
    }

    /// d3's `axis.tickFormat(…)`: each label is `f(value)`.
    #[must_use]
    pub fn format(self, f: impl Fn(&S::Domain) -> String + 'a) -> Self {
        let format: Format<'a, S::Domain> = Rc::new(f);
        Axis {
            format: Some(format),
            ..self
        }
    }

    /// A second level of labels: `f(value)` names each tick's group, and
    /// each run of neighboring ticks with one name is labeled once, past
    /// the tick labels and centered on the run, under a line that spans
    /// it. An empty name leaves its ticks ungrouped. A group label that
    /// would run into the one before it is not drawn.
    ///
    /// With [`Nested`] keys on a band or point scale, the ticks show the
    /// inner level and this names the outer one:
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::axis::{self, Nested};
    /// use rustd3_figure::Figure;
    ///
    /// let keys = [("2025", "Q3"), ("2025", "Q4"), ("2026", "Q1"), ("2026", "Q2")];
    /// let mut fig = Figure::new();
    /// let quarter = fig.scale(scale::band(keys.map(|(y, q)| Nested(y, q))).padding(0.2));
    /// fig.panel(0, 0)
    ///     .axis(axis::bottom(quarter).group(|key| key.0.to_owned()));
    /// ```
    #[must_use]
    pub fn group(self, f: impl Fn(&S::Domain) -> String + 'a) -> Self {
        let group: Format<'a, S::Domain> = Rc::new(f);
        Axis {
            group: Some(group),
            ..self
        }
    }

    /// d3's `axis.ticks(count)`: about `count` ticks. Without it the count
    /// follows the axis's length: one tick per 80 px across, per 35 px up,
    /// at least 2.
    #[must_use]
    pub fn ticks(self, count: usize) -> Self {
        Axis {
            count: Some(count),
            ..self
        }
    }

    /// A title along the axis, outside its tick labels.
    #[must_use]
    pub fn label(self, label: impl Into<Text>) -> Self {
        Axis {
            label: Some(label.into()),
            ..self
        }
    }

    /// Moves the axis `px` away from the plot area, as matplotlib's outward
    /// spines. Without it the axis sits on the plot area, or, after an axis
    /// on the same side, 8 px past that axis's decorations. A `px` that is
    /// not finite leaves the offset as it was.
    #[must_use]
    pub fn outward(self, px: f64) -> Self {
        Axis {
            outward: if px.is_finite() {
                Some(px)
            } else {
                self.outward
            },
            ..self
        }
    }

    /// Colors the axis line, ticks, labels and title with `key`'s color in
    /// `palette`, to tie a twin axis to its series.
    #[must_use]
    pub fn color<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        let tint: Tint<'a> =
            Rc::new(move |resolved: &Resolved| resolved.get_scale(palette)?.paint(&key));
        Axis {
            tint: Some(tint),
            palette: Some(palette.erased()),
            ..self
        }
    }

    /// With or without tick labels and title; without them the axis is its
    /// line and ticks. Either way the axis is drawn whatever its panel's
    /// place. Without this call, an axis that a panel further out on its
    /// side also shows is drawn without labels, and not at all when it is
    /// stacked or moved outward.
    #[must_use]
    pub fn labels(self, labels: bool) -> Self {
        Axis {
            labels: Some(labels),
            ..self
        }
    }

    /// Keeps the title ([`label`](Axis::label)) wherever the axis is drawn
    /// without its tick labels: an inner axis of a grid that shares the
    /// handle loses its tick labels only, as matplotlib's `sharex` and
    /// `sharey` do. Off by default, when an inner axis loses both.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::{Figure, axis};
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// for col in 0..2 {
    ///     fig.panel(0, col)
    ///         .axis(axis::bottom(x).label("order"))
    ///         // The right panel's y ticks are unlabeled, its title kept.
    ///         .axis(axis::left(y).label("amplitude").keep_label(true));
    /// }
    /// ```
    #[must_use]
    pub fn keep_label(self, yes: bool) -> Self {
        Axis {
            keep_label: yes,
            ..self
        }
    }

    /// With or without offset and multiplier notation, on by default, as in
    /// matplotlib's `ScalarFormatter`. It applies to a continuous axis whose
    /// ticks lie evenly on d3's grid. When the domain `[a, b]` sits more
    /// than 10⁴ spans from zero, each tick is labeled by its distance from
    /// an offset with the most trailing zeros, and the offset ("+1e9")
    /// shows at the axis's end. Otherwise, when the domain's largest
    /// magnitude is below 10⁻⁴ or at least 10⁶, ticks are labeled in units
    /// of that power of ten ("1e−7"). Axes with `.format` keep their labels.
    #[must_use]
    pub fn offset(self, on: bool) -> Self {
        Axis { offset: on, ..self }
    }

    /// With or without minor ticks, off by default. Minor ticks are core's
    /// [`Ticks::minor`] values for the axis's ticks (fifths of a step on
    /// linear axes, 2–9 of each decade on log and symlog ones, none on
    /// band, point and time axes). They are drawn at the theme's minor tick
    /// size and width (4 pt and 1 pt at notebook) in the styles that draw
    /// tick marks, without labels or grid lines.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::{Figure, axis};
    ///
    /// let mut fig = Figure::new();
    /// let y = fig.fit(scale::log());
    /// fig.panel(0, 0).axis(axis::left(y).minor(true));
    /// ```
    #[must_use]
    pub fn minor(self, on: bool) -> Self {
        Axis { minor: on, ..self }
    }

    /// Rotates tick labels counter-clockwise by `deg` (matplotlib's sign).
    /// Labels anchor at their end for positive angles on bottom axes
    /// (matplotlib's `autofmt_xdate`: 30°, `ha="right"`), and at their
    /// start for negative ones. Top axes mirror this. Beside left and right
    /// axes a turned label ends or starts at its tick, centered on it
    /// vertically. Layout measures the turned boxes: a label of ink size
    /// w × h reaches w·|sin θ| + h·|cos θ| past a bottom or top axis, and
    /// both collision passes test the turned boxes. The run is shaped once,
    /// and the rotation is its transform. A `deg` that is not finite leaves
    /// the rotation as it was.
    ///
    /// ```
    /// use rustd3_core::time;
    /// use rustd3_figure::{Figure, axis};
    ///
    /// let mut fig = Figure::new();
    /// let x = fig.fit(time::utc());
    /// fig.panel(0, 0).axis(axis::bottom(x).rotate(30.0));
    /// ```
    #[must_use]
    pub fn rotate(self, deg: f64) -> Self {
        Axis {
            rotate: if deg.is_finite() { deg } else { self.rotate },
            ..self
        }
    }
}

impl<S> Axis<'_, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    /// The tick set: the scale's about `count` ticks, or the chosen values
    /// with the scale's step and count.
    fn set(&self, scale: &S, count: usize) -> ticks::Set<<S as Kind>::Domain, <S as Ticks>::Step> {
        let set = scale.tick_set(count);
        match &self.values {
            Some(values) => set.with_values(values.to_vec()),
            None => set,
        }
    }

    /// The labels of `scale`'s about `count` ticks. They are the format's
    /// when the axis has one. Otherwise, under a notation, they are d3's
    /// linear labels of what each tick reads as, over the domain mapped the
    /// same way, and failing that the scale's own. Both of the last two are
    /// written in `locale`.
    fn texts(&self, scale: &S, count: usize, locale: &Locale) -> Vec<String> {
        let set = self.set(scale, count);
        if let Some(format) = &self.format {
            return set.get_values().iter().map(|v| format(v)).collect();
        }
        let mapped = self.notation(scale, count).zip(self.numbers(scale, count));
        let Some((notation, (values, domain))) = mapped else {
            return scale.labels(&set, locale);
        };
        let [a, b] = domain.map(|d| notation.apply(d));
        let values = values.into_iter().map(|t| notation.apply(t)).collect();
        let step = ticks::tick_step(a, b, count as f64);
        let linear = scale::linear().domain([a, b]);
        linear.labels(&ticks::Set::new(values, step, count), locale)
    }

    /// The group name of each of `scale`'s about `count` ticks; empty
    /// without `.group`.
    fn groups(&self, scale: &S, count: usize) -> Vec<String> {
        let Some(group) = &self.group else {
            return Vec::new();
        };
        let set = self.set(scale, count);
        set.get_values().iter().map(|v| group(v)).collect()
    }

    /// The notation of `scale`'s about `count` ticks: none with
    /// `.offset(false)` or `.format`, for a domain that is not numeric, or
    /// for ticks that do not lie evenly on d3's grid (log and symlog
    /// decades).
    fn notation(&self, scale: &S, count: usize) -> Option<Notation> {
        if !self.offset || self.format.is_some() {
            return None;
        }
        let (values, domain) = self.numbers(scale, count)?;
        collide::grid(&values, domain, count)?;
        Notation::of(domain)
    }

    /// The tick values and the domain's ends as numbers, when the domain is
    /// `f64`.
    fn numbers(&self, scale: &S, count: usize) -> Option<(Vec<f64>, [f64; 2])> {
        let set = self.set(scale, count);
        let values = set.get_values().iter().map(number).collect::<Option<_>>()?;
        let domain = Kind::ends(scale)?;
        let domain = (&domain as &dyn Any).downcast_ref::<[f64; 2]>()?;
        Some((values, *domain))
    }
}

/// `v` as a number, when the domain is `f64`.
fn number<D: 'static>(v: &D) -> Option<f64> {
    (v as &dyn Any).downcast_ref::<f64>().copied()
}

/// An axis on side `orient` of handle `h`, with d3's defaults.
fn new<'a, S: Kind>(orient: Orient, h: ScaleRef<S>) -> Axis<'a, S> {
    Axis {
        orient,
        scale: h,
        values: None,
        format: None,
        group: None,
        count: None,
        label: None,
        outward: None,
        labels: None,
        keep_label: false,
        offset: true,
        minor: false,
        rotate: 0.0,
        tint: None,
        palette: None,
    }
}

/// Whether `orient` is a horizontal axis (top or bottom).
pub(crate) fn across(orient: Orient) -> bool {
    match orient {
        Orient::Top | Orient::Bottom => true,
        Orient::Left | Orient::Right => false,
    }
}
