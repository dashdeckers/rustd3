//! [`Panel`]: one plot area in a grid cell.

use std::fmt::Display;

use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

use crate::axis::{self, Axis};
use crate::figure;
use crate::layout::Cell;
use crate::legend::{self, Inset, Legend};
use crate::mark::Mark;
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::Text;

/// One plot area in a grid cell: its marks, axes and inset legends, which
/// refer to the figure's scales only through handles, its title, its
/// border, and the handles gestures over it rescale.
pub struct Panel<'a> {
    cell: Cell,
    marks: Vec<Box<dyn Mark<'a>>>,
    axes: Vec<Box<dyn axis::Draw<'a>>>,
    insets: Vec<(Inset, Box<dyn legend::Draw>)>,
    title: Option<Text>,
    border: Option<Rgb>,
    /// The handles zoomed along x, then along y, each once.
    zooms: [Vec<ScaleRef<()>>; 2],
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
            zooms: [Vec::new(), Vec::new()],
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

    /// Gestures over the plot area rescale `h` along x: in the iced `Plot`
    /// widget, a wheel zooms it around the cursor and a left-button drag
    /// pans it, and every panel that uses `h` follows. Any number of
    /// handles per axis: `.zoom_y(vib).zoom_y(temp)` zooms twin axes
    /// together. Only continuous scales change.
    pub fn zoom_x<S: Kind>(&mut self, h: ScaleRef<S>) -> &mut Self {
        self.zoom(0, h.erased())
    }

    /// Gestures over the plot area rescale `h` along y, as
    /// [`zoom_x`](Panel::zoom_x) does along x.
    pub fn zoom_y<S: Kind>(&mut self, h: ScaleRef<S>) -> &mut Self {
        self.zoom(1, h.erased())
    }

    /// The table positions of the handles zoomed along x and along y, in
    /// the order they were added, for figure `figure`.
    pub(crate) fn zooms(&self, figure: figure::Id) -> [Vec<usize>; 2] {
        self.zooms
            .each_ref()
            .map(|handles| handles.iter().map(|h| h.of(figure)).collect())
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

    /// Adds `h` to the handles zoomed along axis `along` (0: x, 1: y)
    /// unless it is there.
    fn zoom(&mut self, along: usize, h: ScaleRef<()>) -> &mut Self {
        if !self.zooms[along].contains(&h) {
            self.zooms[along].push(h);
        }
        self
    }
}
