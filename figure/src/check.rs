//! Checking a figure before drawing it:
//! [`Figure::check`](crate::Figure::check) lays it out for
//! a size without drawing and lists what does not fit or will not draw
//! well, so a test can assert that a figure fits.
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::check;
//! use rustd3_figure::kurbo::Size;
//! use rustd3_figure::{Figure, axis};
//!
//! let mut fig = Figure::new();
//! let y = fig.scale(scale::linear().domain([0.0, 3.0]));
//! fig.panel(0, 0).axis(axis::left(y).label("vibration RMS [m/s²]"));
//! # #[cfg(feature = "fonts")] {
//! assert!(fig.check(Size::new(640.0, 480.0)).is_empty());
//! let issues = fig.check(Size::new(640.0, 120.0));
//! assert!(matches!(
//!     issues[..],
//!     [check::Issue::Overflow { what: check::Element::AxisTitle, .. }]
//! ));
//! # }
//! ```

#[cfg(feature = "fonts")]
mod find;

/// Collected data larger than this belongs in a recording.
pub(crate) const LARGE: usize = 100_000;

/// Something [`Figure::check`](crate::Figure::check) found.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Issue {
    /// Text longer than the space it labels (it overflows centered).
    Overflow {
        /// What overflows.
        what: Element,
        /// The panel it labels or sits in; `None` for the figure's.
        panel: Option<usize>,
        /// Its length, px.
        needed: f64,
        /// The room it has, px.
        available: f64,
    },
    /// No room left for plot areas after reaches and gaps.
    NoRoom {
        /// Across the grid's columns (`Horizontal`) or rows (`Vertical`).
        axis: Direction,
        /// The px reaches and gaps take.
        needed: f64,
        /// The px the tracks have.
        available: f64,
    },
    /// An axis that the collision passes left fewer than two of its tick
    /// labels: labels too long, or rotated too little, for its length.
    FewLabels {
        /// The axis's panel.
        panel: usize,
        /// The axis's index among its panel's axes, in the order they
        /// were added.
        axis: usize,
        /// The tick labels drawn: 0 or 1.
        kept: usize,
    },
    /// Two legends drawn over each other: insets stacked past their plot
    /// area's middle, or in corners of one too small to hold them apart.
    LegendsOverlap {
        /// The panel whose plot area holds them; `None` for legends in
        /// figure slots.
        panel: Option<usize>,
        /// Their indices among the panel's legends (or the figure's), in
        /// the order they were added.
        legends: [usize; 2],
    },
    /// Owned data over 100,000 points: use a Series.
    LargeOwned {
        /// The mark's panel.
        panel: usize,
        /// The mark's index in its panel.
        mark: usize,
        /// Its number of points.
        points: usize,
    },
    /// Glyphs the fonts lack.
    MissingGlyphs {
        /// The label: its text when it is plain, its typst markup
        /// otherwise.
        text: String,
        /// The glyphs typst drew as .notdef.
        count: usize,
    },
    /// Markup that typst rejected.
    Markup {
        /// The label's typst markup.
        text: String,
        /// typst's message.
        message: String,
    },
    /// A palette key not in the palette's domain.
    UnknownKey {
        /// The mark's panel.
        panel: usize,
        /// The mark's index in its panel.
        mark: usize,
    },
    /// A mark that draws nothing, and why.
    Mark {
        /// The mark's panel.
        panel: usize,
        /// The mark's index in its panel.
        mark: usize,
        /// What the mark kept from when it was built.
        error: crate::mark::Error,
    },
    /// A panel asked for in a cell that reaches past the grid's 4,096
    /// tracks a side ([`layout::Cell`](crate::layout::Cell)): it is not
    /// laid out or drawn. One issue per cell, in the order first asked.
    OutsideGrid {
        /// The cell's first row.
        row: usize,
        /// The cell's first column.
        col: usize,
    },
    /// A scale handle of another figure ([`ScaleRef`](crate::scale::ScaleRef)
    /// carries its figure): the marks, axes, guides, zooms and brushes
    /// given it draw nothing (a palette of another figure leaves its marks
    /// in the theme's gray and its axis untinted; a mark's values on the
    /// figure's own handles still fit them). One issue per panel and
    /// handle.
    ForeignHandle {
        /// The panel it was given to; `None` for a figure slot's guide.
        panel: Option<usize>,
        /// The handle's position in its own figure's table
        /// ([`ScaleRef::get_index`](crate::scale::ScaleRef::get_index)).
        handle: usize,
    },
    /// A handle fitted with `Fit::Window` on a categorical scale, which
    /// has no span to window: it fits as `Fit::All`.
    CategoricalWindow {
        /// The handle's position in the figure's table.
        handle: usize,
    },
    /// Data keys outside a band or point scale's domain, which draw
    /// nothing: one issue per handle, over every mark that uses it.
    OutsideDomain {
        /// The handle's position in the figure's scale table
        /// ([`ScaleRef::get_index`](crate::scale::ScaleRef::get_index)).
        handle: usize,
        /// How many data values lie outside the domain.
        count: usize,
    },
    /// A series name given twice to a chart helper (`helpers::chart`'s
    /// `Line::key` and `Line::series`, `Area::keys`): its series share one
    /// color and one legend entry. One issue per name.
    DuplicateKey {
        /// The name.
        key: String,
    },
}

/// What text overflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Element {
    /// An axis title, longer than its axis.
    AxisTitle,
    /// A panel title, wider than its plot area.
    PanelTitle,
    /// The figure title, wider than the canvas inside its padding.
    FigureTitle,
    /// A legend: in a slot, larger than the canvas inside its padding; in
    /// a panel corner, than the plot area 0.5 em inside its edges.
    Legend,
}

/// An axis of the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Across the columns.
    Horizontal,
    /// Down the rows.
    Vertical,
}
