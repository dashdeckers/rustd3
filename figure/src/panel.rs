//! [`Panel`]: one plot area in a grid cell.

use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

use crate::axis::{self, Axis};
use crate::brush;
use crate::figure;
use crate::layout::Cell;
use crate::legend::{self, Inset};
use crate::mark::Mark;
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::Text;

/// One plot area in a grid cell: its marks, axes and inset legends, which
/// refer to the figure's scales only through handles, its title, its
/// border, the handles gestures over it rescale, and its brush.
pub struct Panel<'a> {
    figure: figure::Id,
    cell: Cell,
    marks: Vec<Box<dyn Mark<'a>>>,
    axes: Vec<Box<dyn axis::Draw<'a>>>,
    insets: Vec<(Inset, legend::Stored<'a>)>,
    title: Option<Text>,
    border: Option<Rgb>,
    /// The handles zoomed along x, then along y, each once.
    zooms: [Vec<ScaleRef<()>>; 2],
    brush: Option<brush::Placed>,
    /// The positions of the handles of other figures that axes, insets,
    /// zooms and the brush were given, which those draw nothing for.
    foreign: Vec<usize>,
}

impl<'a> std::fmt::Debug for Panel<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel").finish_non_exhaustive()
    }
}

impl<'a> Panel<'a> {
    /// An empty panel in grid cell `cell`.
    pub(crate) fn new(figure: figure::Id, cell: Cell) -> Panel<'a> {
        Panel {
            figure,
            cell,
            marks: Vec::new(),
            axes: Vec::new(),
            insets: Vec::new(),
            title: None,
            border: None,
            zooms: [Vec::new(), Vec::new()],
            brush: None,
            foreign: Vec::new(),
        }
    }

    /// Adds a mark, drawn in insertion order.
    pub fn mark(&mut self, m: impl Mark<'a>) -> &mut Self {
        self.marks.push(Box::new(m));
        self
    }

    /// Places an axis, given by value or by reference (one axis value is
    /// placed on many panels). It sits at its `outward` offset, 0 by
    /// default. An axis on a handle of another figure is not placed (its
    /// tint's palette of another figure only leaves it untinted), and
    /// [`Figure::check`](crate::Figure::check) reports the handle.
    pub fn axis<S>(&mut self, a: impl Into<Axis<'a, S>>) -> &mut Self
    where
        S: Position + Ticks<Domain = <S as Kind>::Domain>,
    {
        let a: Box<dyn axis::Draw<'a>> = Box::new(a.into());
        let tint: Vec<_> = a.tint_handle().into_iter().collect();
        if self.admit(&[a.scale()], &tint) {
            self.axes.push(a);
        }
        self
    }

    /// Places guide `g` in corner `at` of the plot area, framed, 0.5 em
    /// from the corner and drawn over the marks: a legend as a column, a
    /// colorbar along the plot area's longer side, 40% of its length. A
    /// guide of a handle of another figure is not placed, and
    /// [`Figure::check`](crate::Figure::check) reports the handle.
    pub fn legend(&mut self, at: Inset, g: impl legend::Guide<'a>) -> &mut Self {
        let stored = g.erase().0;
        if self.admit(&stored.handles(), &[]) {
            self.insets.push((at, stored));
        }
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
    /// together. Only continuous scales change. A handle of another figure
    /// is not zoomed, and [`Figure::check`](crate::Figure::check) reports
    /// it.
    pub fn zoom_x<S: Kind>(&mut self, h: ScaleRef<S>) -> &mut Self {
        self.zoom(0, h.erased())
    }

    /// Gestures over the plot area rescale `h` along y, as
    /// [`zoom_x`](Panel::zoom_x) does along x.
    pub fn zoom_y<S: Kind>(&mut self, h: ScaleRef<S>) -> &mut Self {
        self.zoom(1, h.erased())
    }

    /// Drag in this panel brushes; one brush per panel (a second replaces
    /// it). In the iced `Plot` widget a left-button drag over the plot
    /// area selects an interval along the brush's axis instead of panning,
    /// while the wheel still zooms; file outputs ignore it. A brush over or
    /// setting a handle of another figure is not placed, and
    /// [`Figure::check`](crate::Figure::check) reports the handle.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::{Figure, brush};
    ///
    /// let mut fig = Figure::new();
    /// let full = fig.scale(scale::linear().domain([0.0, 100.0]));
    /// let focus = fig.scale(scale::linear().domain([0.0, 100.0]));
    /// fig.panel(0, 0).zoom_x(focus);
    /// // The overview below sets the window the panel above shows.
    /// fig.panel(1, 0).brush(brush::x(full).sets(focus));
    /// ```
    pub fn brush(&mut self, b: impl Into<brush::Placed>) -> &mut Self {
        let b = b.into();
        if self.admit(&b.handles(), &[]) {
            self.brush = Some(b);
        }
        self
    }

    /// The table positions of the handles zoomed along x and along y, in
    /// the order they were added.
    pub(crate) fn zooms(&self) -> [Vec<usize>; 2] {
        self.zooms
            .each_ref()
            .map(|handles| handles.iter().map(|h| h.index()).collect())
    }

    /// The positions of the handles of other figures this panel's axes,
    /// insets, zooms and brush were given, in the order given, each once.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn foreign(&self) -> &[usize] {
        &self.foreign
    }

    /// The brush placed on the panel, if any.
    pub(crate) fn brushing(&self) -> Option<&brush::Placed> {
        self.brush.as_ref()
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

    /// The inset guides, in the order they were added.
    pub(crate) fn insets(&self) -> &[(Inset, legend::Stored<'a>)] {
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
        if self.admit(&[h], &[]) && !self.zooms[along].contains(&h) {
            self.zooms[along].push(h);
        }
        self
    }

    /// Whether every handle in `needed`, those an element is drawn on, is
    /// of this panel's figure; every handle of another figure in `needed`
    /// and `optional` is recorded for [`Figure::check`](crate::Figure::check).
    fn admit(&mut self, needed: &[ScaleRef<()>], optional: &[ScaleRef<()>]) -> bool {
        for h in needed.iter().chain(optional) {
            if h.of(self.figure).is_none() && !self.foreign.contains(&h.index()) {
                self.foreign.push(h.index());
            }
        }
        needed.iter().all(|h| h.of(self.figure).is_some())
    }
}
