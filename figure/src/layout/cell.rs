//! [`Cell`]: where a panel sits in the grid.

use std::ops::Range;

/// Where a panel sits in the grid: the rows and columns it spans. A
/// spanning panel's reaches count against its outer tracks, and its plot
/// area runs from its first track's start to its last track's end.
///
/// ```
/// use rustd3_figure::Figure;
/// use rustd3_figure::layout::Cell;
///
/// let mut fig = Figure::new();
/// fig.panel(0, 0);
/// fig.panel(0, 1);
/// // A panel across the whole second row.
/// fig.panel_at(Cell::new(1, 0).span(1, 2));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    rows: Range<usize>,
    cols: Range<usize>,
}

impl Cell {
    /// The cell in row `row` and column `col`, one track each way.
    pub fn new(row: usize, col: usize) -> Cell {
        Cell {
            rows: row..row + 1,
            cols: col..col + 1,
        }
    }

    /// The same first row and column, spanning `rows` rows and `cols`
    /// columns; a span of 0 is taken as 1.
    pub fn span(self, rows: usize, cols: usize) -> Cell {
        let (row, col) = (self.rows.start, self.cols.start);
        Cell {
            rows: row..row + rows.max(1),
            cols: col..col + cols.max(1),
        }
    }

    /// The rows it spans.
    pub(crate) fn rows(&self) -> Range<usize> {
        self.rows.clone()
    }

    /// The columns it spans.
    pub(crate) fn cols(&self) -> Range<usize> {
        self.cols.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_keeps_the_first_track_and_takes_at_least_one() {
        let cell = Cell::new(1, 0).span(1, 2);
        assert_eq!((cell.rows(), cell.cols()), (1..2, 0..2));
        assert_eq!(Cell::new(2, 3).span(0, 0), Cell::new(2, 3));
    }
}
