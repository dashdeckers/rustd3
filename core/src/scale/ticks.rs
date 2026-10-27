//! [`Ticks`]: what an axis needs from a scale, and its impls for the
//! scales of this crate.

mod continuous;
mod identity;

use crate::format::InvalidFormat;
use crate::locale::Locale;
use crate::scale::{TickFormat, Transform, tick_format};
use crate::ticks;

/// What an axis needs from a scale: d3 duck-types it (`scale.ticks`,
/// `scale.tickFormat`, `scale.bandwidth`), this names it.
///
/// Labels are one batch over the whole set, so the neutral locale groups an
/// axis all or nothing:
///
/// ```
/// use rustd3_core::locale::Locale;
/// use rustd3_core::scale::{self, Ticks};
///
/// let a = scale::linear().domain([0.0, 8000.0]);
/// assert_eq!(a.labels(&a.tick_set(5), &Locale::NEUTRAL), ["0", "2000", "4000", "6000", "8000"]);
/// let b = scale::linear().domain([0.0, 12000.0]);
/// assert_eq!(b.labels(&b.tick_set(5), &Locale::NEUTRAL)[1], "2\u{202f}000");
/// ```
///
/// The method is `tick_set`, not `ticks`, because a scale's inherent
/// `ticks(count)` is d3's and would shadow it; for the same reason a
/// continuous scale's minor ticks of a set are written
/// `Ticks::minor(&scale, &set)`.
pub trait Ticks {
    /// The values ticks mark (`f64`, a band key, a `Timestamp`).
    type Domain: Clone;
    /// The step the scale chooses ticks with: `f64` for continuous and
    /// binned scales, a time interval for time scales, `()` for band,
    /// point and ordinal scales.
    type Step: Clone;

    /// About `count` ticks, with the step and count they were chosen for.
    fn tick_set(&self, count: usize) -> ticks::Set<Self::Domain, Self::Step>;

    /// The tick's position along the range for an axis with `offset`
    /// (d3's `number` / `center`); `None` for a key outside the domain.
    /// A scale whose range is not numeric normalizes it to [0, 1].
    fn position(&self, value: &Self::Domain, offset: f64) -> Option<f64>;

    /// One label per value, as one batch under `locale`.
    fn labels(&self, ticks: &ticks::Set<Self::Domain, Self::Step>, locale: &Locale) -> Vec<String>;

    /// Minor tick values for `.minor(true)`; none by default.
    fn minor(&self, _ticks: &ticks::Set<Self::Domain, Self::Step>) -> Vec<Self::Domain> {
        Vec::new()
    }

    /// First and last range value, d3's `range[0]` and `range[n-1]`;
    /// `[0, 1]` for a range that is not numeric.
    fn extent(&self) -> [f64; 2];
}

/// The labels of `ticks` under `locale` by a scale's default tick format
/// `format` (its `tick_format_in` with no specifier).
///
/// # Panics
///
/// When `format` is an error: a transform's default specifier must parse,
/// as [`Transform::tick_format`] requires.
pub(super) fn labels(
    format: Result<TickFormat, InvalidFormat>,
    ticks: &ticks::Set<f64, f64>,
    locale: &Locale,
) -> Vec<String> {
    format
        .expect("invariant: a transform's default tick format specifier parses")
        .labels(ticks, locale)
}

/// The minor ticks of a scale under `transform` whose end stops are
/// `ends`: the majors extended by one step on each side in transformed
/// space (u₋₁ = 2u₀ − u₁, uₙ₊₁ = 2uₙ − uₙ₋₁), the transform's minor ticks
/// of that list, kept inside the ends and off the majors. An extension
/// that maps back to no finite value, or not past its end (a transform
/// undefined there, such as a square root's negatives), is left out. None
/// for fewer than two majors.
pub(super) fn minor<T: Transform>(transform: &T, ends: [f64; 2], majors: &[f64]) -> Vec<f64> {
    let ([first, second, ..], [.., before, last]) = (majors, majors) else {
        return Vec::new();
    };
    let beyond = |end: f64, inner: f64| {
        let u = 2.0 * transform.forward(end) - transform.forward(inner);
        Some(transform.inverse(u)).filter(|v| v.is_finite() && (v - end) * (end - inner) > 0.0)
    };
    let extended: Vec<f64> = beyond(*first, *second)
        .into_iter()
        .chain(majors.iter().copied())
        .chain(beyond(*last, *before))
        .collect();
    let (low, high) = if ends[1] < ends[0] {
        (ends[1], ends[0])
    } else {
        (ends[0], ends[1])
    };
    transform
        .minor(&extended)
        .into_iter()
        .filter(|v| (low..=high).contains(v) && !majors.contains(v))
        .collect()
}

/// A binned scale's ticks: its bin edges `values`, in order, with the
/// smallest positive gap between neighbours as the step (NaN for none),
/// which sets the labels' precision as d3's `tickFormat` uses its tick
/// step.
pub(super) fn edges(values: Vec<f64>, count: usize) -> ticks::Set<f64, f64> {
    let step = values
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .filter(|gap| gap.is_finite() && *gap > 0.0)
        .min_by(f64::total_cmp)
        .unwrap_or(f64::NAN);
    ticks::Set::new(values, step, count)
}

/// The labels of a binned scale's edges: d3's linear tick format at the
/// set's step (`,f` to [`precision_fixed`](crate::format::precision_fixed)
/// of it) under `locale`, as one batch.
pub(super) fn edge_labels(set: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
    let largest = set.values().iter().fold(0.0, |m: f64, v| m.max(v.abs()));
    tick_format::stepped(locale.number(), *set.step(), largest, None)
        .expect("invariant: `,f` parses")
        .format_all(set.values())
}
