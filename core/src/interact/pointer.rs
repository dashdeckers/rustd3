//! [`Pointer`]: renderer-agnostic pointer input, with its wheel [`Delta`]
//! and [`Button`].

use crate::interact::Modifiers;

/// Pointer input in the chart's own pixel coordinates, as the behaviors
/// read it; the host translates its events into it (rustd3-iced's
/// `pointer::Tracker` for iced).
///
/// Plain data. It is `#[non_exhaustive]`, so matches from other crates
/// carry a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Pointer {
    /// A button went down at (`x`, `y`) with `modifiers` held.
    Pressed {
        /// The pointer's horizontal position.
        x: f64,
        /// The pointer's vertical position.
        y: f64,
        /// The button pressed.
        button: Button,
        /// The keyboard modifiers held.
        modifiers: Modifiers,
    },
    /// The pointer moved to (`x`, `y`), inside the area or, during a drag,
    /// outside it.
    Moved {
        /// The pointer's horizontal position.
        x: f64,
        /// The pointer's vertical position.
        y: f64,
    },
    /// A button went up, wherever the pointer is.
    Released {
        /// The button released.
        button: Button,
    },
    /// A wheel or trackpad step at (`x`, `y`) with `modifiers` held.
    Wheel {
        /// The pointer's horizontal position.
        x: f64,
        /// The pointer's vertical position.
        y: f64,
        /// The step, in the DOM's convention.
        delta: Delta,
        /// The keyboard modifiers held.
        modifiers: Modifiers,
    },
    /// A double click at (`x`, `y`).
    DoubleClicked {
        /// The pointer's horizontal position.
        x: f64,
        /// The pointer's vertical position.
        y: f64,
    },
    /// The keyboard modifiers changed (shift-double-click zooms out;
    /// brushes accept and ignore them).
    Modifiers(Modifiers),
    /// The pointer left the area without a button held.
    Left,
}

/// A wheel step in the DOM's convention: positive `y` scrolls down (d3
/// zooms out), positive `x` scrolls right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delta {
    /// Steps in lines, as a mouse wheel reports them (the DOM's
    /// `deltaMode` 1).
    Lines {
        /// Lines to the right.
        x: f64,
        /// Lines down.
        y: f64,
    },
    /// Steps in logical pixels, as a trackpad reports them (`deltaMode` 0).
    Pixels {
        /// Pixels to the right.
        x: f64,
        /// Pixels down.
        y: f64,
    },
}

/// A pointer button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// The primary button, which gestures use.
    Left,
    /// The secondary button.
    Right,
    /// The wheel button.
    Middle,
    /// The back button.
    Back,
    /// The forward button.
    Forward,
    /// Any other button, by the host's number.
    Other(u16),
}
