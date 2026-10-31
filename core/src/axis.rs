//! d3-axis for the core level: an axis draws its domain path and tick marks
//! into any [`PathContext`] and returns its labels as positions and
//! strings for the host's own text.
//!
//! ```
//! use rustd3_core::{axis, path, scale};
//!
//! let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
//! let a = axis::bottom(&x).ticks(2);
//! assert_eq!(
//!     path::svg(|p| a.draw(p)),
//!     "M0.5,6L0.5,0.5L960.5,0.5L960.5,6M0.5,0L0.5,6M480.5,0L480.5,6M960.5,0L960.5,6"
//! );
//! let labels = a.labels();
//! assert_eq!((labels[1].x(), labels[1].y(), labels[1].text()), (480.5, 9.0, "50"));
//! ```

mod label;
mod orient;

pub use label::{Anchor, Label};
pub use orient::Orient;

use std::fmt;

use crate::locale::Locale;
use crate::path::PathContext;
use crate::scale::Ticks;
use crate::ticks;

/// An axis over a scale: d3's `axis`, with its tick count, tick values,
/// label format, tick sizes, padding and offset, plus a locale for the
/// default labels and optional minor ticks.
///
/// The format closure borrows for `'s`, as the scale does, so it may use
/// values of the enclosing view.
pub struct Axis<'s, S: Ticks> {
    scale: &'s S,
    orient: Orient,
    count: usize,
    values: Option<Vec<S::Domain>>,
    format: Option<Formatter<'s, S::Domain>>,
    locale: Locale,
    inner: f64,
    outer: f64,
    padding: f64,
    offset: f64,
    minor: bool,
    minor_size: f64,
}

/// A caller's label for a tick value, borrowing for `'s`.
type Formatter<'s, D> = Box<dyn Fn(&D) -> String + 's>;

impl<S: Ticks + fmt::Debug> fmt::Debug for Axis<'_, S>
where
    S::Domain: fmt::Debug,
{
    /// The format prints as `custom` or `None`: a closure has no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Axis")
            .field("scale", &self.scale)
            .field("orient", &self.orient)
            .field("count", &self.count)
            .field("values", &self.values)
            .field("format", &self.format.as_ref().map(|_| "custom"))
            .field("locale", &self.locale)
            .field("inner", &self.inner)
            .field("outer", &self.outer)
            .field("padding", &self.padding)
            .field("offset", &self.offset)
            .field("minor", &self.minor)
            .field("minor_size", &self.minor_size)
            .finish()
    }
}

/// An axis of `scale` on the side `orient` names, with d3's defaults: 10
/// ticks, tick size 6, padding 3, offset 0.5 (d3 at a device pixel ratio
/// of 1), the neutral locale and no minor ticks (size 3 when on).
pub fn new<S: Ticks>(orient: Orient, scale: &S) -> Axis<'_, S> {
    Axis {
        scale,
        orient,
        count: 10,
        values: None,
        format: None,
        locale: Locale::NEUTRAL,
        inner: 6.0,
        outer: 6.0,
        padding: 3.0,
        offset: 0.5,
        minor: false,
        minor_size: 3.0,
    }
}

/// d3.axisTop: ticks and labels above a horizontal line.
#[doc(alias = "axisTop")]
pub fn top<S: Ticks>(scale: &S) -> Axis<'_, S> {
    new(Orient::Top, scale)
}

/// d3.axisRight: ticks and labels right of a vertical line.
#[doc(alias = "axisRight")]
pub fn right<S: Ticks>(scale: &S) -> Axis<'_, S> {
    new(Orient::Right, scale)
}

/// d3.axisBottom: ticks and labels below a horizontal line. The scale must
/// implement [`Ticks`]:
///
/// ```compile_fail
/// use rustd3_core::axis;
///
/// axis::bottom(&"not a scale");
/// ```
///
/// ```
/// use rustd3_core::{axis, scale};
///
/// axis::bottom(&scale::linear());
/// ```
#[doc(alias = "axisBottom")]
pub fn bottom<S: Ticks>(scale: &S) -> Axis<'_, S> {
    new(Orient::Bottom, scale)
}

/// d3.axisLeft: ticks and labels left of a vertical line.
#[doc(alias = "axisLeft")]
pub fn left<S: Ticks>(scale: &S) -> Axis<'_, S> {
    new(Orient::Left, scale)
}

impl<'s, S: Ticks> Axis<'s, S> {
    /// d3's `ticks(count)`: about `count` ticks, which also sets the
    /// default labels' precision.
    #[must_use]
    pub fn ticks(self, count: usize) -> Self {
        Axis { count, ..self }
    }

    /// d3's `tickValues`: these values instead of the scale's ticks; the
    /// default labels still follow the tick count.
    #[must_use]
    pub fn values(self, values: impl IntoIterator<Item = S::Domain>) -> Self {
        Axis {
            values: Some(values.into_iter().collect()),
            ..self
        }
    }

    /// d3's `tickFormat(f)`: each label is `f(value)` instead of the
    /// scale's labels under the locale.
    #[must_use]
    pub fn format(self, f: impl Fn(&S::Domain) -> String + 's) -> Self {
        Axis {
            format: Some(Box::new(f)),
            ..self
        }
    }

    /// The locale of the default labels.
    #[must_use]
    pub fn locale(self, locale: Locale) -> Self {
        Axis { locale, ..self }
    }

    /// d3's `tickSize`: the inner and outer tick size, in pixels.
    #[must_use]
    pub fn tick_size(self, px: f64) -> Self {
        Axis {
            inner: px,
            outer: px,
            ..self
        }
    }

    /// d3's `tickSizeInner`: the length of each tick mark.
    #[must_use]
    pub fn tick_size_inner(self, px: f64) -> Self {
        Axis { inner: px, ..self }
    }

    /// d3's `tickSizeOuter`: the length of the domain path's end marks; 0
    /// draws a plain line.
    #[must_use]
    pub fn tick_size_outer(self, px: f64) -> Self {
        Axis { outer: px, ..self }
    }

    /// The length of each minor tick mark.
    #[must_use]
    pub fn tick_size_minor(self, px: f64) -> Self {
        Axis {
            minor_size: px,
            ..self
        }
    }

    /// d3's `tickPadding`: the gap between a tick and its label.
    #[must_use]
    pub fn tick_padding(self, px: f64) -> Self {
        Axis {
            padding: px,
            ..self
        }
    }

    /// d3's `offset`: added to every position and the range ends, 0.5 to
    /// center one-pixel lines on pixels.
    #[must_use]
    pub fn offset(self, px: f64) -> Self {
        Axis { offset: px, ..self }
    }

    /// Whether minor ticks are drawn, from [`Ticks::minor`].
    #[must_use]
    pub fn minor(self, on: bool) -> Self {
        Axis { minor: on, ..self }
    }

    /// Domain path, then major ticks, then minor ticks, into `ctx`. Ticks
    /// whose position is `None` or not finite are left out.
    pub fn draw(&self, ctx: &mut impl PathContext) {
        let (k, o) = (self.orient.sign(), self.offset);
        let [r0, r1] = self.scale.extent().map(|r| r + o);
        let mut domain = vec![(r0, o), (r1, o)];
        // d3 draws the end marks when tickSizeOuter is truthy: neither 0 nor NaN.
        if self.outer != 0.0 && !self.outer.is_nan() {
            domain.insert(0, (r0, k * self.outer));
            domain.push((r1, k * self.outer));
        }
        for (i, &(along, across)) in domain.iter().enumerate() {
            let (x, y) = self.orient.point(along, across);
            if i == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        let set = self.set();
        self.marks(ctx, set.values(), k * self.inner);
        if self.minor {
            self.marks(ctx, &self.scale.minor(&set), k * self.minor_size);
        }
    }

    /// Where the host draws each label, and what: one per tick whose
    /// position is finite, formatted as one batch.
    #[must_use]
    pub fn labels(&self) -> Vec<Label> {
        let set = self.set();
        let texts = match &self.format {
            Some(f) => set.values().iter().map(f).collect(),
            None => self.scale.labels(&set, &self.locale),
        };
        let across = self.orient.sign() * (self.inner.max(0.0) + self.padding);
        set.values()
            .iter()
            .zip(texts)
            .filter_map(|(value, text)| {
                let (x, y) = self.orient.point(self.place(value)?, across);
                Some(Label {
                    x,
                    y,
                    text,
                    anchor: self.orient.anchor(),
                    dy: self.orient.dy(),
                })
            })
            .collect()
    }

    /// The tick set: the scale's at the count, or the chosen values with
    /// its step and count.
    fn set(&self) -> ticks::Set<S::Domain, S::Step> {
        let set = self.scale.tick_set(self.count);
        match &self.values {
            Some(values) => set.with_values(values.clone()),
            None => set,
        }
    }

    /// d3's `position(d) + offset`, when finite.
    fn place(&self, value: &S::Domain) -> Option<f64> {
        self.scale
            .position(value, self.offset)
            .map(|p| p + self.offset)
            .filter(|p| p.is_finite())
    }

    /// A tick mark `length` across the line at each placed value.
    fn marks(&self, ctx: &mut impl PathContext, values: &[S::Domain], length: f64) {
        for p in values.iter().filter_map(|v| self.place(v)) {
            let (x, y) = self.orient.point(p, 0.0);
            ctx.move_to(x, y);
            let (x, y) = self.orient.point(p, length);
            ctx.line_to(x, y);
        }
    }
}
