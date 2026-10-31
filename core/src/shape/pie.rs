//! d3's pie layout ([`Pie`]) and its drawing order
//! ([`Sort`]).

use std::cmp::Ordering;
use std::f64::consts::TAU;

use crate::js;
use crate::shape::{Pie, Slice};

/// The order wedges are laid out in, clockwise from the start angle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Largest value first: d3's default `sortValues(descending)`.
    #[default]
    Descending,
    /// Smallest value first: `sortValues(ascending)`.
    Ascending,
    /// In input order: d3's `sort(null)`.
    Input,
}

impl Default for Pie {
    fn default() -> Self {
        Self {
            sort: Sort::Descending,
            start_angle: 0.0,
            end_angle: TAU,
            pad_angle: 0.0,
        }
    }
}

impl Pie {
    /// The drawing order: d3's `sortValues` and `sort(null)`.
    #[must_use]
    pub fn sort(self, sort: Sort) -> Self {
        Self { sort, ..self }
    }

    /// Where the first wedge starts, in radians clockwise from 12 o'clock
    /// (d3's default 0).
    #[must_use]
    pub fn start_angle(self, start_angle: f64) -> Self {
        Self {
            start_angle,
            ..self
        }
    }

    /// Where the last wedge ends (d3's default τ); the span is clamped to
    /// ±τ.
    #[must_use]
    pub fn end_angle(self, end_angle: f64) -> Self {
        Self { end_angle, ..self }
    }

    /// The padding between neighbouring wedges (d3's default 0), at most
    /// the span over the number of values.
    #[must_use]
    pub fn pad_angle(self, pad_angle: f64) -> Self {
        Self { pad_angle, ..self }
    }

    /// d3's `pie(data)` (`pie.js:15–52`): one slice per value, returned in
    /// input order (`slices[i].datum == i`), each spanning its share of the
    /// angle in drawing order. Values that are not positive (NaN included)
    /// span only the padding; when no value is positive every slice starts
    /// and ends where the previous one did. `Descending` and `Ascending`
    /// sort stably, ±0 equal and NaN last.
    ///
    /// ```
    /// use rustd3_core::shape;
    ///
    /// let slices = shape::pie().layout([3.0, 1.0, 2.0]);
    /// assert_eq!((slices[0].index, slices[0].start_angle), (0, 0.0));
    /// assert_eq!(slices[1].index, 2);
    /// assert_eq!(slices[2].end_angle, slices[1].start_angle);
    /// ```
    pub fn layout(&self, values: impl IntoIterator<Item = f64>) -> Vec<Slice> {
        let values: Vec<f64> = values.into_iter().collect();
        let n = values.len() as f64;
        let span = (self.end_angle - self.start_angle).clamp(-TAU, TAU);
        let pad = js::min(span.abs() / n, self.pad_angle);
        let signed_pad = if span < 0.0 { -pad } else { pad };
        let sum = values.iter().filter(|&&v| v > 0.0).fold(0.0, |s, v| s + v);
        let k = if js::truthy(sum) {
            (span - n * signed_pad) / sum
        } else {
            0.0
        };
        let mut order: Vec<usize> = (0..values.len()).collect();
        match self.sort {
            Sort::Descending => {
                order.sort_by(|&i, &j| nan_last(values[i], values[j], |a, b| b.partial_cmp(&a)));
            }
            Sort::Ascending => {
                order.sort_by(|&i, &j| nan_last(values[i], values[j], |a, b| a.partial_cmp(&b)));
            }
            Sort::Input => {}
        }
        let mut slices = Vec::with_capacity(values.len());
        let mut angle = self.start_angle;
        for (index, datum) in order.into_iter().enumerate() {
            let value = values[datum];
            let share = if value > 0.0 { value * k } else { 0.0 };
            let end_angle = angle + share + signed_pad;
            slices.push(Slice {
                datum,
                index,
                value,
                start_angle: angle,
                end_angle,
                pad_angle: pad,
            });
            angle = end_angle;
        }
        slices.sort_by_key(|slice| slice.datum);
        slices
    }
}

/// `a` against `b` by `by` when both are numbers, NaN after every number:
/// d3's comparators call ±0 equal and leave NaN's place to the engine.
fn nan_last(a: f64, b: f64, by: fn(f64, f64) -> Option<Ordering>) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => by(a, b).unwrap_or(Ordering::Equal),
    }
}
