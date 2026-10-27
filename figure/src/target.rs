//! [`Target`]: what a figure is drawn for.

/// What a figure is resolved and drawn for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// SVG, PDF: list px, no snapping.
    Vector,
    /// iced: device px = list px × `scale`.
    Screen {
        /// Device pixels per list px.
        scale: f64,
    },
}

impl Target {
    /// Device pixels per list px: 1 for vector output.
    pub(crate) fn scale_factor(self) -> f64 {
        match self {
            Target::Vector => 1.0,
            Target::Screen { scale } => scale,
        }
    }
}
