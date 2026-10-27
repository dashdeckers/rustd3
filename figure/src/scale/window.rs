//! `Fit::Window`'s quantization, and [`Memory`]: the column counts the
//! last resolve laid each windowed handle out in, from which the next one
//! starts.

use std::any::TypeId;
use std::collections::HashMap;

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
    fn a_handle_remembers_its_columns_by_index_and_type() {
        let mut memory = Memory::default();
        let key = (2, TypeId::of::<f64>());
        assert_eq!(memory.get(key), None);
        memory.set(key, 1000);
        assert_eq!(memory.get(key), Some(1000));
        assert_eq!(memory.get((2, TypeId::of::<f32>())), None);
    }
}
