//! Figure axes: values built once per quantity from a handle and placed on
//! every panel that shows it. `axis::bottom(h)` takes a figure handle,
//! where core's `axis::bottom(&x)` takes the scale itself.
//!
//! An axis needs a scale that places values and has ticks; band keys are
//! labelled through `Display`, so an axis on a band whose key has none
//! does not compile (E0277):
//!
//! ```compile_fail
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
mod placed;

use std::rc::Rc;

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::Ticks;
use rustd3_core::ticks;

use crate::figure::Resolved;
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::text::Text;

pub(crate) use draw::Draw;
pub(crate) use labels::Labels;
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
    tint: Option<Tint<'a>>,
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
            tint: self.tint.clone(),
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
pub fn bottom<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Bottom, h)
}

/// An axis left of the panel: ticks left, labels right-aligned to them.
pub fn left<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Left, h)
}

/// An axis right of the panel: ticks right, labels left-aligned to them.
pub fn right<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Right, h)
}

/// An axis above the panel: ticks up, labels centred over them.
pub fn top<'a, S>(h: ScaleRef<S>) -> Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    new(Orient::Top, h)
}

impl<'a, S: Kind> Axis<'a, S> {
    /// d3's `axis.tickValues(…)`: these values instead of the scale's
    /// ticks; default labels still follow the tick count.
    pub fn values(self, values: impl IntoIterator<Item = S::Domain>) -> Self {
        Axis {
            values: Some(values.into_iter().collect()),
            ..self
        }
    }

    /// d3's `axis.tickFormat(…)`: each label is `f(value)`.
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
    pub fn ticks(self, count: usize) -> Self {
        Axis {
            count: Some(count),
            ..self
        }
    }

    /// A title along the axis, outside its tick labels.
    pub fn label(self, label: impl Into<Text>) -> Self {
        Axis {
            label: Some(label.into()),
            ..self
        }
    }

    /// Moves the axis `px` away from the plot area, as matplotlib's outward
    /// spines. Without it the axis sits on the plot area, or, after an axis
    /// on the same side, 8 px past that axis's decorations.
    pub fn outward(self, px: f64) -> Self {
        Axis {
            outward: Some(px),
            ..self
        }
    }

    /// Colors the axis line, ticks, labels and title with `key`'s color in
    /// `palette`, to tie a twin axis to its series.
    pub fn color<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        let tint: Tint<'a> =
            Rc::new(move |resolved: &Resolved| resolved.scale(palette).paint(&key));
        Axis {
            tint: Some(tint),
            ..self
        }
    }

    /// With or without tick labels and title (without, the axis is its line
    /// and ticks), and drawn whatever its panel's place: without it, an
    /// axis that a panel further out on its side shows too is drawn
    /// without labels, and not at all when it is stacked or moved outward.
    pub fn labels(self, labels: bool) -> Self {
        Axis {
            labels: Some(labels),
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

    /// The labels of `scale`'s about `count` ticks: the format's, else the
    /// scale's own under the neutral locale.
    fn texts(&self, scale: &S, count: usize) -> Vec<String> {
        let set = self.set(scale, count);
        match &self.format {
            Some(format) => set.values().iter().map(|v| format(v)).collect(),
            None => scale.labels(&set, &Locale::default()),
        }
    }
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
        tint: None,
    }
}

/// Whether `orient` is a horizontal axis (top or bottom).
pub(crate) fn across(orient: Orient) -> bool {
    match orient {
        Orient::Top | Orient::Bottom => true,
        Orient::Left | Orient::Right => false,
    }
}
