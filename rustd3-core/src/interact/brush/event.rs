//! [`BrushEvent`]: what a brush reports.

/// What a brush reports: d3's `start`, `brush` and `end` events, each
/// with the selection after it, `None` when there is none. `S` is the
/// selection. The default is an interval, and a 2-D form would use
/// `[[f64; 2]; 2]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushEvent<S = [f64; 2]> {
    /// A gesture started (a press on the overlay, the selection or a
    /// handle).
    Start(Option<S>),
    /// The gesture changed the selection.
    Brush(Option<S>),
    /// The gesture ended; an empty selection ends as `None`.
    End(Option<S>),
}
