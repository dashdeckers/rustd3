//! [`Panel`]: one plot area in a grid cell.

use std::fmt::Display;

use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

use crate::axis::{self, Axis};
use crate::layout::Cell;
use crate::legend::{self, Inset, Legend};
use crate::mark::Mark;
use crate::scale::{Kind, Position};
use crate::text::Text;

/// One plot area in a grid cell: its marks, axes and inset legends, which
/// refer to the figure's scales only through handles, its title and its
/// border.
pub struct Panel<'a> {
    cell: Cell,
    marks: Vec<Box<dyn Mark<'a>>>,
    axes: Vec<Box<dyn axis::Draw<'a>>>,
    insets: Vec<(Inset, Box<dyn legend::Draw>)>,
    title: Option<Text>,
    border: Option<Rgb>,
}

impl<'a> Panel<'a> {
    /// An empty panel in grid cell `cell`.
    pub(crate) fn new(cell: Cell) -> Panel<'a> {
        Panel {
            cell,
            marks: Vec::new(),
            axes: Vec::new(),
            insets: Vec::new(),
            title: None,
            border: None,
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

    /// Places legend `l` in corner `at` of the plot area: a framed column
    /// 0.5 em from the corner, drawn over the marks.
    pub fn legend<K>(&mut self, at: Inset, l: Legend<K>) -> &mut Self
    where
        K: PartialEq + Clone + Display + 'static,
    {
        self.insets.push((at, Box::new(l)));
        self
    }

    /// A title centred above the plot area and its top axes.
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
        self
    }

    /// A frame of color `c` around the plot area, as wide as the axis
    /// lines.
    pub fn border(&mut self, c: Rgb) -> &mut Self {
        self.border = Some(c);
        self
    }

    /// The grid cell.
    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    /// The marks, in insertion order.
    pub(crate) fn marks(&self) -> &[Box<dyn Mark<'a>>] {
        &self.marks
    }

    /// The axes, in insertion order.
    pub(crate) fn axes(&self) -> &[Box<dyn axis::Draw<'a>>] {
        &self.axes
    }

    /// The inset legends, in the order they were added.
    pub(crate) fn insets(&self) -> &[(Inset, Box<dyn legend::Draw>)] {
        &self.insets
    }

    /// The title.
    pub(crate) fn heading(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    /// The border's color.
    pub(crate) fn frame(&self) -> Option<Rgb> {
        self.border
    }
}
