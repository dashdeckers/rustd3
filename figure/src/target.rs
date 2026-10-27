//! [`Target`]: what a figure is drawn for.

/// What a figure is resolved and drawn for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// SVG, PDF: list px, 4 thinning columns per px, no snapping.
    Vector,
    /// PNG, GIF: device px = list px × `scale`; 4 thinning columns per
    /// device px.
    File {
        /// Device pixels per list px.
        scale: f64,
    },
    /// iced: device px = list px × `scale`; 1 thinning column per device
    /// px.
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
            Target::File { scale } | Target::Screen { scale } => scale,
        }
    }

    /// Whether the figure is drawn for the screen, where gestures change it.
    pub(crate) fn interactive(self) -> bool {
        match self {
            Target::Vector | Target::File { .. } => false,
            Target::Screen { .. } => true,
        }
    }

    /// Thinning columns per list px: four per list pixel for vector
    /// output, four per device pixel for files, one per device pixel on
    /// screen. Files are thinned finer than the screen because M4's zigzag
    /// leaves lighter stripes inside dense bands that overlapping segments
    /// fill: at four columns per pixel they nearly vanish.
    pub(crate) fn columns(self) -> f64 {
        match self {
            Target::Vector => 4.0,
            Target::File { scale } => 4.0 * scale,
            Target::Screen { scale } => scale,
        }
    }
}
