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
    /// Text, ticks and axis lines in `c`; seaborn's ".15", `#262626`, by
    /// default. The iced widget draws its crosshair and highlight in it. A
    /// color with a channel or an opacity that is not finite leaves the
    /// theme as it was.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_figure::Theme;
    ///
    /// let red = Rgb::hex(0xff0000);
    /// assert_eq!(Theme::default().ink(red).get_ink(), red);
    /// assert_eq!(Theme::default().ink(Rgb::new(f64::NAN, 0.0, 0.0)), Theme::default());
    /// ```
    pub fn ink(self, c: Rgb) -> Self {
        if finite(c) {
            Theme { ink: c, ..self }
        } else {
            self
        }
    }

    /// Behind the whole figure, `c` at any opacity: under a translucent
    /// background what the figure is drawn over shows through, such as an
    /// app's own background under the iced widget. White by default. A
    /// color with a channel or an opacity that is not finite leaves the
    /// theme as it was.
    pub fn background(self, c: Rgb) -> Self {
        if finite(c) {
            Theme {
                background: c,
                ..self
            }
        } else {
            self
        }
    }

    /// Text, ticks and axis lines.
    pub fn get_ink(&self) -> Rgb {
        self.ink
    }

    /// Behind the whole figure.
    pub fn get_background(&self) -> Rgb {
        self.background
    }

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

    /// Minor ticks, and ticks whose label the scale's format leaves empty:
    /// 4 pt.
    pub(crate) fn minor_tick(&self) -> f64 {
        self.pt(4.0)
    }

    /// Minor ticks' width: 1 pt.
    pub(crate) fn minor_width(&self) -> f64 {
        self.pt(1.0)
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

    /// The share of a fitted continuous domain's span added per side.
    pub(crate) fn margin(&self) -> f64 {
        self.margin
    }

    /// Lines and rules: 1.5 pt.
    pub(crate) fn line_width(&self) -> f64 {
        self.pt(1.5)
    }

    /// Outlines of filled shapes: 1 pt, seaborn's `patch.linewidth`.
    pub(crate) fn patch_width(&self) -> f64 {
        self.pt(1.0)
    }

    /// The outline of a density: 0.75 pt.
    pub(crate) fn density_width(&self) -> f64 {
        self.pt(0.75)
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

    /// A note's text: the axis-label size (matplotlib's `font.size` at
    /// seaborn's notebook, 12 pt), its lines aligned by `align`, in `fill`.
    pub(crate) fn note(&self, align: anchor::Horizontal, fill: Rgb) -> Style {
        style(self.label_text(), align, fill)
    }

    /// A legend entry's label: tick-label size (matplotlib's
    /// `legend.fontsize` at seaborn's notebook, 11 pt), in ink.
    pub(crate) fn legend_label(&self) -> Style {
        style(self.tick_text(), anchor::Horizontal::Start, self.ink)
    }

    /// A legend's title: 12 pt (`legend.title_fontsize`), centred lines in
    /// ink.
    pub(crate) fn legend_title(&self) -> Style {
        style(self.label_text(), anchor::Horizontal::Middle, self.ink)
    }

    /// Behind an inset legend: the panel's color at matplotlib's
    /// `legend.framealpha`, 0.8.
    pub(crate) fn legend_face(&self) -> Rgb {
        self.panel.with_opacity(0.8)
    }

    /// An inset legend's edge: matplotlib's `legend.edgecolor`, gray 0.8.
    pub(crate) fn legend_edge(&self) -> Rgb {
        Rgb::hex(0xcccccc)
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

/// Whether every channel and the opacity of `c` are finite.
fn finite(c: Rgb) -> bool {
    [c.r, c.g, c.b, c.opacity].iter().all(|v| v.is_finite())
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
        let minor = [theme.minor_tick(), theme.minor_width()];
        assert_eq!(minor.map(|v| format!("{v:.3}")), ["5.333", "1.333"]);
        assert!(close(theme.title_pad(), 8.0));
        assert_eq!(format!("{:.3}", theme.axis_width()), "1.667");
        assert_eq!(theme.get_ink(), Rgb::hex(0x262626));
        assert_eq!(theme.edge(), theme.get_ink());
        assert_eq!(
            [theme.panel(), theme.get_background()],
            [Rgb::hex(0xffffff); 2]
        );
        assert_eq!(theme.margin(), 0.05);
        assert!(close(theme.line_width(), 2.0));
        assert!(close(theme.marker(), 8.0));
        assert_eq!(theme.color(), Rgb::hex(0x0173b2));
        assert_eq!(theme.gray(), scheme::COLORBLIND[7]);
    }

    #[test]
    fn ink_and_background_builders_set_what_the_readers_return() {
        let red = Rgb::hex(0xff0000);
        let half = Rgb::hex(0x336699).with_opacity(0.5);
        let theme = Theme::default().ink(red).background(half);
        assert_eq!((theme.get_ink(), theme.get_background()), (red, half));
        // The edge and every text style follow the ink.
        assert_eq!(theme.edge(), red);
        assert_eq!(theme.title().fill, red);
        assert_eq!(theme.figure_title().fill, red);
        assert_eq!(theme.legend_label().fill, red);
        // The panel keeps its own color.
        assert_eq!(theme.panel(), Rgb::hex(0xffffff));
    }

    #[test]
    fn colors_that_are_not_finite_leave_the_theme_as_it_was() {
        let theme = Theme::default();
        for bad in [
            Rgb::new(f64::NAN, 0.0, 0.0),
            Rgb::new(0.0, f64::INFINITY, 0.0),
            Rgb::hex(0xff0000).with_opacity(f64::NAN),
        ] {
            assert_eq!(theme.clone().ink(bad), theme);
            assert_eq!(theme.clone().background(bad), theme);
        }
    }
}
