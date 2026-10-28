//! `Fit::Window`'s quantization, in `f64` for numbers and in `i128`
//! nanoseconds for instants, and [`Memory`]: the column counts the last
//! resolve laid each windowed handle out in, from which the next one
//! starts.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use rustd3_core::jiff::{SignedDuration, Timestamp};

use crate::scale::visible::instant;

/// `span` as a window's span, boxed: an `f64` that is finite and
/// positive, or a positive `SignedDuration`; `None` for any other (a
/// categorical scale's `()`).
pub(crate) fn span<S: Copy + 'static>(span: S) -> Option<Box<dyn Any>> {
    let any = &span as &dyn Any;
    if let Some(&number) = any.downcast_ref::<f64>() {
        return (number.is_finite() && number > 0.0).then(|| Box::new(number) as Box<dyn Any>);
    }
    let duration = *any.downcast_ref::<SignedDuration>()?;
    duration
        .is_positive()
        .then(|| Box::new(duration) as Box<dyn Any>)
}

/// The window of span `span` ending at `last`, `columns` columns wide, as
/// `[D; 2]` boxed: [`ends`] for numbers, [`instants`] for a
/// `SignedDuration` after a `Timestamp`; `None` for other types and where
/// those give none.
pub(crate) fn shown(last: &dyn Any, span: &dyn Any, columns: u32) -> Option<Box<dyn Any>> {
    if let (Some(&last), Some(&span)) = (last.downcast_ref::<f64>(), span.downcast_ref::<f64>()) {
        return ends(last, span, columns).map(|e| Box::new(e) as Box<dyn Any>);
    }
    let last = *last.downcast_ref::<Timestamp>()?;
    let span = *span.downcast_ref::<SignedDuration>()?;
    instants(last, span, columns).map(|e| Box::new(e) as Box<dyn Any>)
}

/// [`ends`] in `i128` nanoseconds: each column is `span / columns`
/// rounded to whole nanoseconds, `right` the first multiple of it from the
/// epoch at or past `last`, and `left` lies `columns` columns before it,
/// so appends move the window by whole columns and the columns' edges
/// repeat exactly. `None` when `span` is not positive, `columns` is 0, a
/// column rounds to no nanosecond or an end leaves jiff's range.
pub(crate) fn instants(
    last: Timestamp,
    span: SignedDuration,
    columns: u32,
) -> Option<[Timestamp; 2]> {
    let span = span.as_nanos();
    let columns = i128::from(columns);
    if span <= 0 || columns == 0 {
        return None;
    }
    let col = (span + columns / 2) / columns;
    if col == 0 {
        return None;
    }
    let right = -(-last.as_nanosecond()).div_euclid(col) * col;
    Some([instant(right - columns * col)?, instant(right)?])
}

/// The newest `span` of x ending at `last` in a plot area `columns` device
/// columns wide, `[left, right]`: with `col = span / columns`, `right` is
/// the first whole multiple of `col` at or past `last` and `left` lies
/// `columns` columns before it, so appends move the window by whole
/// columns. `None` when `span` is not finite and positive, `columns` is 0
/// or an end is not finite.
pub(crate) fn ends(last: f64, span: f64, columns: u32) -> Option<[f64; 2]> {
    if !(span.is_finite() && span > 0.0 && columns > 0 && last.is_finite()) {
        return None;
    }
    let columns = f64::from(columns);
    let col = span / columns;
    let right = (last / col).ceil() * col;
    let left = right - columns * col;
    (left.is_finite() && right.is_finite() && left < right).then_some([left, right])
}

/// The column count each windowed handle was last laid out in, keyed by
/// its index in the scale table and its scale's type, which survive a
/// figure's rebuild: kept in [`figure::Cache`](crate::Cache), so a resolve
/// whose layout did not move fits each window once.
#[derive(Debug, Default)]
pub(crate) struct Memory {
    columns: HashMap<(usize, TypeId), u32>,
}

impl Memory {
    /// The column count handle `key` was last laid out in.
    pub(crate) fn get(&self, key: (usize, TypeId)) -> Option<u32> {
        self.columns.get(&key).copied()
    }

    /// Remembers that handle `key` was laid out in `columns` columns.
    pub(crate) fn set(&mut self, key: (usize, TypeId), columns: u32) {
        self.columns.insert(key, columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `a` within 1e-12 of `b`, relative.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12 * b.abs().max(1.0)
    }

    #[test]
    fn a_window_ends_at_the_first_whole_column_past_the_last_sample() {
        let [left, right] = ends(123.456, 10.0, 1000).expect("a window");
        assert!(
            close(right, 123.46) && close(left, 113.46),
            "{left}, {right}"
        );
        let [left, right] = ends(123.556, 10.0, 1000).expect("a window");
        assert!(
            close(right, 123.56) && close(left, 113.56),
            "{left}, {right}"
        );
        assert_eq!(ends(123.5561, 10.0, 1000).map(|w| w[1]), Some(right));
    }

    #[test]
    fn a_window_needs_a_positive_span_and_a_column() {
        assert_eq!(ends(1.0, 0.0, 100), None);
        assert_eq!(ends(1.0, -1.0, 100), None);
        assert_eq!(ends(1.0, f64::NAN, 100), None);
        assert_eq!(ends(1.0, 10.0, 0), None);
        assert_eq!(ends(f64::INFINITY, 10.0, 100), None);
    }

    #[test]
    fn a_time_window_ends_at_the_first_whole_column_in_nanoseconds() {
        let at = |s: &str| s.parse::<Timestamp>().expect("an instant");
        let ten = SignedDuration::from_secs(10);
        // Columns of 10 ms: 12:00:03.456789 rounds up to .46.
        let window = instants(at("2026-10-25T12:00:03.456789Z"), ten, 1000);
        let expected = [at("2026-10-25T11:59:53.46Z"), at("2026-10-25T12:00:03.46Z")];
        assert_eq!(window, Some(expected));
        let on = instants(at("2026-10-25T12:00:03.46Z"), ten, 1000);
        assert_eq!(on, Some(expected));
        // 1 s over 3 columns: 333,333,333 ns each, the span 1 ns short.
        let [left, right] = instants(at("2026-10-25T12:00:00Z"), SignedDuration::from_secs(1), 3)
            .expect("a window");
        assert_eq!(right.duration_since(left).as_nanos(), 999_999_999);
        assert_eq!(right.as_nanosecond() % 333_333_333, 0);
        assert_eq!(
            instants(at("2026-10-25T12:00:00Z"), SignedDuration::from_nanos(1), 3),
            None
        );
        assert_eq!(instants(at("2026-10-25T12:00:00Z"), -ten, 3), None);
        assert_eq!(instants(Timestamp::MAX, ten, 1000), None);
    }

    #[test]
    fn spans_of_numbers_and_durations_are_kept_when_positive() {
        let number = span(5.0).and_then(|s| s.downcast_ref::<f64>().copied());
        assert_eq!(number, Some(5.0));
        assert!(span(0.0).is_none() && span(f64::NAN).is_none() && span(()).is_none());
        assert!(span(SignedDuration::from_secs(60)).is_some());
        assert!(span(SignedDuration::ZERO).is_none());
        let last = Timestamp::UNIX_EPOCH;
        let shown = shown(&last, &SignedDuration::from_secs(10), 10).expect("a window");
        let ends = shown.downcast_ref::<[Timestamp; 2]>().expect("instants");
        assert_eq!(ends[1], last);
        assert!(super::shown(&1.0, &SignedDuration::from_secs(1), 10).is_none());
    }

    #[test]
    fn a_handle_remembers_its_columns_by_index_and_type() {
        let mut memory = Memory::default();
        let key = (2, TypeId::of::<f64>());
        assert_eq!(memory.get(key), None);
        memory.set(key, 1000);
        assert_eq!(memory.get(key), Some(1000));
        assert_eq!(memory.get((2, TypeId::of::<f32>())), None);
    }
}
