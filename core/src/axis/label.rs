//! [`Label`]: one tick label for the host to draw, and its [`Anchor`].

/// Which point of the text its position names, d3's `text-anchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The start of the text: labels right of a vertical axis.
    Start,
    /// The middle of the text: labels of a horizontal axis.
    Middle,
    /// The end of the text: labels left of a vertical axis.
    End,
}

/// One tick label for the host's text: position in axis coordinates, text
/// anchor and d3's `dy` in em.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) text: String,
    pub(super) anchor: Anchor,
    pub(super) dy: f64,
}

impl Label {
    /// The x of the anchor point, in axis coordinates.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// The y of the anchor point, in axis coordinates, before `dy`.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// The text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Which point of the text (x, y) names.
    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    /// d3's vertical shift of the text, in em of its font size.
    pub fn dy(&self) -> f64 {
        self.dy
    }
}
