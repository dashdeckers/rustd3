//! d3's stack layout over columns ([`Stack`]), with
//! d3's six orders ([`Order`]) and five offsets ([`Offset`]).

mod offset;
mod order;

pub use offset::Offset;
pub use order::Order;

use crate::shape::Stack;

impl Stack {
    /// The order layers stack in: d3's `stack.order`.
    #[must_use]
    pub fn order(self, order: Order) -> Self {
        Self { order, ..self }
    }

    /// Where the stack's baseline is: d3's `stack.offset`.
    #[must_use]
    pub fn offset(self, offset: Offset) -> Self {
        Self { offset, ..self }
    }

    /// d3's `stack(data)` over columns, one layer per column (`stack.js`):
    /// layers indexed like `columns`, each `[y0, y1]` per row. Rows are the
    /// shortest column's length; longer columns are truncated.
    ///
    /// ```
    /// use rustd3_core::shape::{self, stack};
    ///
    /// let layers = shape::Stack::default()
    ///     .offset(stack::Offset::Expand)
    ///     .layout(&[&[3.0, 2.0, 1.0], &[1.0, 1.0, 4.0]]);
    /// assert_eq!(layers[0], [[0.0, 0.75], [0.0, 2.0 / 3.0], [0.0, 0.2]]);
    /// ```
    #[must_use]
    pub fn layout(&self, columns: &[&[f64]]) -> Vec<Vec<[f64; 2]>> {
        let mut layers = unstacked(columns);
        let order = self.order.apply(&layers);
        self.offset.apply(&mut layers, &order);
        layers
    }

    /// The stacking order `layout` used, bottom first: d3's `series[i].index`
    /// inverted, so `stacking_order(…)[0]` is the column at the baseline.
    #[must_use]
    pub fn stacking_order(&self, columns: &[&[f64]]) -> Vec<usize> {
        self.order.apply(&unstacked(columns))
    }
}

/// Each column as d3's series before ordering: `[0, value]` per row, the
/// rows cut to the shortest column.
fn unstacked(columns: &[&[f64]]) -> Vec<Vec<[f64; 2]>> {
    let rows = columns.iter().map(|c| c.len()).min().unwrap_or(0);
    columns
        .iter()
        .map(|c| c[..rows].iter().map(|&v| [0.0, v]).collect())
        .collect()
}
