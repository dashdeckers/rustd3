//! Figure axes: values built once per quantity from a handle and placed on
//! every panel that shows it. `axis::bottom(h)` takes a figure handle,
//! where core's `axis::bottom(&x)` takes the scale itself.
//!
//! An axis needs a scale that places values and has ticks; band keys are
//! labelled through `Display`, so an axis on a band whose key has none
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

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};
use rustd3_core::ticks;

use crate::figure::Resolved;
use crate::layout::collide;
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::text::Text;

pub(crate) use draw::Draw;
pub(crate) use labels::Labels;
use offset::Notation;
pub(crate) use placed::Placed;

/// A label for each tick value, in place of the scale's.
type Format<'a, D> = Rc<dyn Fn(&D) -> String + 'a>;

/// The color of a twin axis, read from the resolved palette.
type Tint<'a> = Rc<dyn Fn(&Resolved) -> Option<Rgb> + 'a>;

/// An axis of one quantity: its side, its handle, and how its ticks are
/// chosen and labelled. Built once and placed on many panels (`From<&Axis>`
/// clones it); closures are shared, never boxed per panel, and may borrow
/// for `'a`.
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
    count: Option<usize>,
    label: Option<Text>,
    outward: Option<f64>,
    labels: Option<bool>,
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
            count: self.count,
            label: self.label.clone(),
            outward: self.outward,
            labels: self.labels,
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

/// An axis below the panel: ticks down, labels centred under them.
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

/// An axis above the panel: ticks up, labels centred over them.
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
            Rc::new(move |resolved: &Resolved| resolved.scale(palette)?.paint(&key));
        Axis {
            tint: Some(tint),
            palette: Some(palette.erased()),
            ..self
        }
    }

    /// With or without tick labels and title (without, the axis is its line
    /// and ticks), and drawn whatever its panel's place: without it, an
    /// axis that a panel further out on its side shows too is drawn
    /// without labels, and not at all when it is stacked or moved outward.
    #[must_use]
    pub fn labels(self, labels: bool) -> Self {
        Axis {
            labels: Some(labels),
            ..self
        }
    }

    /// With or without offset and multiplier notation, on by default. A
    /// continuous axis whose ticks lie evenly on d3's grid and whose domain
    /// `[a, b]` sits more than 10⁴ spans from zero labels each tick by its
    /// distance from an offset with the most trailing zeros, and shows the
    /// offset ("+1e9") at its end; otherwise, when the domain's largest
    /// magnitude is below 10⁻⁴ or at least 10⁶, it labels ticks in units of
    /// that power of ten ("1e−7"), as matplotlib's `ScalarFormatter`.
    /// Axes with `.format` keep their labels.
    #[must_use]
    pub fn offset(self, on: bool) -> Self {
        Axis { offset: on, ..self }
    }

    /// With or without minor ticks, off by default: core's
    /// [`Ticks::minor`] values for the axis's ticks (fifths of a step on
    /// linear axes, 2–9 of each decade on log and symlog ones, none on
    /// band, point and time axes), drawn at the theme's minor tick size and
    /// width (4 pt and 1 pt at notebook) in the styles that draw tick
    /// marks, without labels or grid lines.
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
    /// (matplotlib's `autofmt_xdate`: 30°, `ha="right"`), at their start
    /// for negative ones; top axes mirror; left and right axes rotate
    /// about the label's center. Layout measures the turned boxes: a label
    /// of ink size w × h reaches w·|sin θ| + h·|cos θ| past a bottom or
    /// top axis, and both collision passes test the turned boxes. The run
    /// is shaped once; the rotation is its transform. A `deg` that is not
    /// finite leaves the rotation as it was.
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

    /// The labels of `scale`'s about `count` ticks: the format's; else,
    /// under a notation, d3's linear labels of what each tick reads as over
    /// the domain mapped the same way; else the scale's own; both written
    /// in `locale`.
    fn texts(&self, scale: &S, count: usize, locale: &Locale) -> Vec<String> {
        let set = self.set(scale, count);
        if let Some(format) = &self.format {
            return set.values().iter().map(|v| format(v)).collect();
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
        let values = set.values().iter().map(number).collect::<Option<_>>()?;
        let domain = Kind::domain(scale)?;
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
        count: None,
        label: None,
        outward: None,
        labels: None,
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
