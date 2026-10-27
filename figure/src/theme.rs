//! [`Theme`]: the sizes, colors and margins a figure is drawn with.

use rustd3_core::color::Rgb;
use rustd3_core::scheme;

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
    /// Behind each plot area.
    panel: Rgb,
    /// Behind the whole figure.
    background: Rgb,
    /// Fitted continuous domains get this share of their span per side.
    margin: f64,
    /// Marks without a key take its first color.
    palette: &'static [Rgb],
    /// Marks whose key their palette lacks.
    gray: Rgb,
}

impl Default for Theme {
    /// seaborn's `ticks` style at `notebook` size: tick labels 11 pt, axis
    /// labels and titles 12 pt, the figure title 14.4 pt, ink `#262626`
    /// for text, ticks and axis lines, white panels and figure, no grid,
    /// 5% margins; lines 1.5 pt wide, markers 6 pt across; seaborn's
    /// colorblind palette, and its gray `#949494` for keys a palette lacks.
    fn default() -> Self {
        Theme {
            context: 1.0,
            font_scale: 1.0,
            ink: Rgb::hex(0x262626),
            panel: Rgb::hex(0xffffff),
            background: Rgb::hex(0xffffff),
            margin: 0.05,
            palette: &scheme::COLORBLIND,
            gray: Rgb::hex(0x949494),
        }
    }
}

impl Theme {
    /// Axis lines, ticks and borders: 1.25 pt.
    pub(crate) fn axis_width(&self) -> f64 {
        self.pt(1.25)
    }

    /// Axis lines: the ink, in the ticks style.
    pub(crate) fn edge(&self) -> Rgb {
        self.ink
    }

    /// Behind each plot area.
    pub(crate) fn panel(&self) -> Rgb {
        self.panel
    }

    /// Behind the whole figure.
    pub(crate) fn background(&self) -> Rgb {
        self.background
    }

    /// Tick labels: 11 pt.
    pub(crate) fn tick_text(&self) -> f64 {
        self.text(11.0)
    }

    /// Axis labels and panel titles: 12 pt.
    pub(crate) fn label_text(&self) -> f64 {
        self.text(12.0)
    }

    /// The figure title: matplotlib's `large`, 1.2 × 12 pt.
    pub(crate) fn figure_title_text(&self) -> f64 {
        self.text(14.4)
    }

    /// What `em` lengths are multiples of: the tick-label size.
    pub(crate) fn em(&self) -> f64 {
        self.tick_text()
    }

    /// Tick marks: 6 pt.
    pub(crate) fn tick(&self) -> f64 {
        self.pt(6.0)
    }

    /// Between a tick and its label: 3.5 pt.
    pub(crate) fn tick_pad(&self) -> f64 {
        self.pt(3.5)
    }

    /// Between tick labels and the axis label: 4 pt.
    pub(crate) fn label_pad(&self) -> f64 {
        self.pt(4.0)
    }

    /// Below a title: 6 pt.
    pub(crate) fn title_pad(&self) -> f64 {
        self.pt(6.0)
    }

    /// Text, ticks and axis lines: seaborn's ".15".
    pub(crate) fn ink(&self) -> Rgb {
        self.ink
    }

    /// The share of a fitted continuous domain's span added per side.
    pub(crate) fn margin(&self) -> f64 {
        self.margin
    }

    /// Lines and rules: 1.5 pt.
    pub(crate) fn line_width(&self) -> f64 {
        self.pt(1.5)
    }

    /// A marker's diameter: 6 pt.
    pub(crate) fn marker(&self) -> f64 {
        self.pt(6.0)
    }

    /// The palette's first color: what marks without a key are drawn in.
    pub(crate) fn color(&self) -> Rgb {
        *self
            .palette
            .first()
            .expect("invariant: a theme's palette is not empty")
    }

    /// What a mark whose key its palette lacks is drawn in.
    pub(crate) fn gray(&self) -> Rgb {
        self.gray
    }

    /// A tick label's style: its lines aligned by `align`, in `fill`.
    pub(crate) fn tick_label(&self, align: anchor::Horizontal, fill: Rgb) -> Style {
        style(self.tick_text(), align, fill)
    }

    /// An axis label's style: centred lines in `fill`.
    pub(crate) fn axis_label(&self, fill: Rgb) -> Style {
        style(self.label_text(), anchor::Horizontal::Middle, fill)
    }

    /// A panel title's style: centred lines in ink.
    pub(crate) fn title(&self) -> Style {
        style(self.label_text(), anchor::Horizontal::Middle, self.ink)
    }

    /// A mark's label (a rule's): tick-label size, its lines aligned by
    /// `align`, in `fill`.
    pub(crate) fn mark_label(&self, align: anchor::Horizontal, fill: Rgb) -> Style {
        style(self.tick_text(), align, fill)
    }

    /// The figure title's style: centred lines in ink.
    pub(crate) fn figure_title(&self) -> Style {
        style(
            self.figure_title_text(),
            anchor::Horizontal::Middle,
            self.ink,
        )
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

/// Regular text of `size` px in `fill`, its lines aligned by `align`.
fn style(size: f64, align: anchor::Horizontal, fill: Rgb) -> Style {
    Style {
        size,
        bold: false,
        fill,
        align,
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
        assert!(close(theme.label_text(), 16.0));
        assert!(close(theme.figure_title_text(), 19.2));
        let pads = [theme.tick(), theme.tick_pad(), theme.label_pad()];
        assert_eq!(pads.map(|v| format!("{v:.3}")), ["8.000", "4.667", "5.333"]);
        assert!(close(theme.title_pad(), 8.0));
        assert_eq!(format!("{:.3}", theme.axis_width()), "1.667");
        assert_eq!(theme.ink(), Rgb::hex(0x262626));
        assert_eq!(theme.edge(), theme.ink());
        assert_eq!([theme.panel(), theme.background()], [Rgb::hex(0xffffff); 2]);
        assert_eq!(theme.margin(), 0.05);
        assert!(close(theme.line_width(), 2.0));
        assert!(close(theme.marker(), 8.0));
        assert_eq!(theme.color(), Rgb::hex(0x0173b2));
        assert_eq!(theme.gray(), scheme::COLORBLIND[7]);
    }
}
