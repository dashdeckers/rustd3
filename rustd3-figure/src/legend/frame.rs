//! [`Frame`]: the box drawn behind a legend.

/// The box behind a legend, 0.4 em of its text past its entries on every
/// side, on the panel's color at 0.8 opacity (matplotlib's
/// `legend.framealpha`) with a gray edge (`legend.edgecolor`, 0.8). By
/// default a legend in a panel corner is [`Square`](Frame::Square) and one
/// in a figure slot [`Off`](Frame::Off).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Frame {
    /// No box.
    Off,
    /// A square box with an edge of 1 px.
    Square,
    /// matplotlib's box (`legend.fancybox`): corners rounded by 0.2 em of
    /// the legend's text, and an edge as wide as the outlines of bars
    /// (`patch.linewidth`, 1 pt at seaborn's notebook) at 0.8 opacity, as
    /// matplotlib's frame alpha also takes the edge.
    Round,
}
