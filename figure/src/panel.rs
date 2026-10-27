//! [`Panel`]: one plot area in a grid cell.

use crate::mark::Mark;

/// One plot area in a grid cell: its marks, which refer to the figure's
/// scales only through handles.
pub struct Panel<'a> {
    cell: (usize, usize),
    marks: Vec<Box<dyn Mark<'a>>>,
}

impl<'a> Panel<'a> {
    /// An empty panel in grid cell `cell`, (row, column).
    pub(crate) fn new(cell: (usize, usize)) -> Panel<'a> {
        Panel {
            cell,
            marks: Vec::new(),
        }
    }

    /// Adds a mark, drawn in insertion order.
    pub fn mark(&mut self, m: impl Mark<'a>) -> &mut Self {
        self.marks.push(Box::new(m));
        self
    }

    /// The grid cell, (row, column).
    pub(crate) fn cell(&self) -> (usize, usize) {
        self.cell
    }

    /// The marks, in insertion order.
    pub(crate) fn marks(&self) -> &[Box<dyn Mark<'a>>] {
        &self.marks
    }
}
