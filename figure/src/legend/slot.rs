//! [`Slot`]: where a figure-level legend sits.

/// A figure-level place outside the grid, inside the outer padding. Top
/// and bottom legends are one row centered on the grid's plot extent, and
/// left and right ones a column centered on it vertically. Each reserves its
/// box plus 0.5 em from the grid. Legends in one slot stack outward in the
/// order they were added, 0.5 em apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Above the grid, below the figure title.
    Top,
    /// Below the grid.
    Bottom,
    /// Left of the grid.
    Left,
    /// Right of the grid.
    Right,
}
