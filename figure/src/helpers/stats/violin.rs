//! [`Inner`]: what a violin draws inside its body. The chart is
//! [`stats::violin`](super::violin()).

pub(super) mod chart;

/// What a violin draws inside its body, in ink, as seaborn's `inner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Inner {
    /// Nothing.
    None,
    /// A miniature box plot: a thick line from `q1` to `q3`, a thin one
    /// between the whisker ends (1.5 × IQR), and the median as a short
    /// dash in the theme's background color.
    #[default]
    Box,
    /// Dashed lines across the body at the quartiles.
    Quartiles,
}
