//! The two halves of an [`Anchor`](crate::text::Anchor): which point of a
//! label's box sits at the place it is drawn.

/// Which vertical edge of the box, or its middle, is the anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Horizontal {
    /// The left edge: text starts at the anchor.
    #[default]
    Start,
    /// The middle: text is centred on the anchor.
    Middle,
    /// The right edge: text ends at the anchor.
    End,
}

/// Which horizontal line of the box, or its middle, is the anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Vertical {
    /// The first line's baseline.
    #[default]
    Baseline,
    /// The top of the ink.
    Top,
    /// Halfway between the top and the bottom of the ink.
    Middle,
    /// The bottom of the ink.
    Bottom,
}
