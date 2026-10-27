//! [`Theme`]: the sizes, colors and margins a figure is drawn with.

use rustd3_core::color::Rgb;

use crate::text::{Style, anchor};

/// px per pt: the CSS ratio.
const PX_PER_PT: f64 = 4.0 / 3.0;

/// The sizes, colors and margins a figure is drawn with: seaborn 0.13.2's
/// `ticks` style at `notebook` size with the colorblind palette, sizes in
/// px at 4/3 px per pt. Fitted domains get 5% margins.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// seaborn's context scale: notebook 1.
    context: f64,
    /// Text alone scales by this.
    font_scale: f64,
    /// Text, ticks and axis lines: seaborn's ".15".
    ink: Rgb,
    /// Fitted continuous domains get this share of their span per side.
    margin: f64,
}

impl Default for Theme {
    /// seaborn's `ticks` style at `notebook` size: tick labels 11 pt, axis
    /// labels and titles 12 pt, the figure title 14.4 pt, ink `#262626`,
    /// 5% margins.
    fn default() -> Self {
        Theme {
            context: 1.0,
            font_scale: 1.0,
            ink: Rgb::hex(0x262626),
            margin: 0.05,
        }
    }
}

impl Theme {
    /// Tick labels: 11 pt.
    pub(crate) fn tick_text(&self) -> f64 {
        self.text(11.0)
    }

    /// The figure title: matplotlib's `large`, 1.2 × 12 pt.
    pub(crate) fn figure_title_text(&self) -> f64 {
        self.text(14.4)
    }

    /// What `em` lengths are multiples of: the tick-label size.
    pub(crate) fn em(&self) -> f64 {
        self.tick_text()
    }

    /// Below a title: 6 pt.
    pub(crate) fn title_pad(&self) -> f64 {
        self.pt(6.0)
    }

    /// The share of a fitted continuous domain's span added per side.
    pub(crate) fn margin(&self) -> f64 {
        self.margin
    }

    /// The figure title's style: centred lines in ink.
    pub(crate) fn figure_title(&self) -> Style {
        self.style(self.figure_title_text(), anchor::Horizontal::Middle)
    }

    /// Text of `size` px in ink, its lines aligned by `align`.
    fn style(&self, size: f64, align: anchor::Horizontal) -> Style {
        Style {
            size,
            bold: false,
            fill: self.ink,
            align,
        }
    }

    /// `v` pt at this context, in px.
    fn pt(&self, v: f64) -> f64 {
        v * PX_PER_PT * self.context
    }

    /// `v` pt of text at this context and font scale, in px.
    fn text(&self, v: f64) -> f64 {
        self.pt(v) * self.font_scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::em;

    /// `a` within 1e-9 of `b`.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn default_theme_sizes() {
        let theme = Theme::default();
        assert!(
            close(theme.tick_text(), 44.0 / 3.0),
            "{}",
            theme.tick_text()
        );
        assert!(close(theme.em(), theme.tick_text()));
        assert_eq!(format!("{:.3}", theme.em()), "14.667");
        let gap = em(1.6).to_px(theme.em());
        assert_eq!(format!("{gap:.3}"), "23.467");
        assert!(close(theme.figure_title_text(), 19.2));
        assert!(close(theme.title_pad(), 8.0));
        assert_eq!(theme.figure_title().fill, Rgb::hex(0x262626));
        assert_eq!(theme.margin(), 0.05);
    }
}
