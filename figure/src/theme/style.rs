//! [`Style`]: seaborn's `axes_style` presets.

use rustd3_core::color::Rgb;

/// seaborn 0.13.2's `axes_style` presets: the panel's color, and which of
/// grid lines, axis lines and tick marks are drawn. Text, tick labels and
/// tick marks are in the theme's ink (seaborn's ".15", `#262626`) and the
/// figure is white in every style.
///
/// | Style | Panel | Axis line | Grid | Tick marks |
/// | --- | --- | --- | --- | --- |
/// | `DarkGrid` | `#EAEAF2` | none (seaborn's is white) | white | off |
/// | `WhiteGrid` | white | `#CCCCCC` | `#CCCCCC` | off |
/// | `Dark` | `#EAEAF2` | none (seaborn's is white) | none | off |
/// | `White` | white | the ink | none | off |
/// | `Ticks` | white | the ink | none | on, outward |
///
/// Without tick marks, tick labels keep their distance from the axis, as
/// matplotlib keeps its tick pad and size when it hides tick marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// A gray-blue panel with white grid lines.
    DarkGrid,
    /// A white panel with light gray grid lines and axis lines.
    WhiteGrid,
    /// A gray-blue panel.
    Dark,
    /// A white panel with axis lines in the ink.
    White,
    /// A white panel with axis lines and outward tick marks in the ink:
    /// the default.
    Ticks,
}

/// seaborn's dark styles' panel: `#EAEAF2`.
const DARK_PANEL: Rgb = Rgb::hex(0xeaeaf2);

/// seaborn's light gray, ".8".
const LIGHT_GRAY: Rgb = Rgb::hex(0xcccccc);

/// White.
const WHITE: Rgb = Rgb::hex(0xffffff);

impl Style {
    /// The color behind each plot area.
    pub(crate) fn panel(self) -> Rgb {
        match self {
            Style::DarkGrid | Style::Dark => DARK_PANEL,
            Style::WhiteGrid | Style::White | Style::Ticks => WHITE,
        }
    }

    /// The axis lines' color, `ink` where the style draws them in the
    /// ink; `None` where it draws none.
    pub(crate) fn edge(self, ink: Rgb) -> Option<Rgb> {
        match self {
            Style::DarkGrid | Style::Dark => None,
            Style::WhiteGrid => Some(LIGHT_GRAY),
            Style::White | Style::Ticks => Some(ink),
        }
    }

    /// The grid lines' color; `None` without a grid.
    pub(crate) fn grid(self) -> Option<Rgb> {
        match self {
            Style::DarkGrid => Some(WHITE),
            Style::WhiteGrid => Some(LIGHT_GRAY),
            Style::Dark | Style::White | Style::Ticks => None,
        }
    }

    /// Whether tick marks are drawn.
    pub(crate) fn ticks(self) -> bool {
        match self {
            Style::Ticks => true,
            Style::DarkGrid | Style::WhiteGrid | Style::Dark | Style::White => false,
        }
    }
}
