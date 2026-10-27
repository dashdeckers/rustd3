//! [`Panel`]: one plot area in a grid cell.

use rustd3_core::scale::Ticks;

use crate::axis::{self, Axis};
use crate::mark::Mark;
use crate::scale::{Kind, Position};
use crate::text::Text;

/// One plot area in a grid cell: its marks and axes, which refer to the
/// figure's scales only through handles, and its title.
pub struct Panel<'a> {
    cell: (usize, usize),
    marks: Vec<Box<dyn Mark<'a>>>,
    axes: Vec<Box<dyn axis::Draw<'a>>>,
    title: Option<Text>,
}

impl<'a> Panel<'a> {
    /// An empty panel in grid cell `cell`, (row, column).
    pub(crate) fn new(cell: (usize, usize)) -> Panel<'a> {
        Panel {
            cell,
            marks: Vec::new(),
            axes: Vec::new(),
            title: None,
        }
    }

    /// Adds a mark, drawn in insertion order.
    pub fn mark(&mut self, m: impl Mark<'a>) -> &mut Self {
        self.marks.push(Box::new(m));
        self
    }

    /// Places an axis, given by value or by reference (one axis value is
    /// placed on many panels). It sits at its `outward` offset, 0 by
    /// default.
    pub fn axis<S>(&mut self, a: impl Into<Axis<'a, S>>) -> &mut Self
    where
        S: Position + Ticks<Domain = <S as Kind>::Domain>,
    {
        self.axes.push(Box::new(a.into()));
        self
    }

    /// A title centred above the plot area and its top axes.
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
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

    /// The axes, in insertion order.
    pub(crate) fn axes(&self) -> &[Box<dyn axis::Draw<'a>>] {
        &self.axes
    }

    /// The title.
    pub(crate) fn heading(&self) -> Option<&Text> {
        self.title.as_ref()
    }
}
