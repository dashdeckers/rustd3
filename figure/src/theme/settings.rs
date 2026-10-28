//! [`Theme`]: the sizes, colors and domain rule a figure is drawn with.

use rustd3_core::color::Rgb;
use rustd3_core::scheme;

use crate::text::{self, anchor};
use crate::theme::{Context, Domains, Style};

/// px per pt: the CSS ratio.
const PX_PER_PT: f64 = 4.0 / 3.0;

/// seaborn's ".15": text, tick labels, tick marks and, in the white
/// styles, axis lines.
const INK: Rgb = Rgb::hex(0x262626);

/// What a mark whose key its palette lacks is drawn in: seaborn's gray
/// (the colorblind palette's eighth color).
const GRAY: Rgb = Rgb::hex(0x949494);

/// The sizes, colors and domain rule a figure is drawn with: seaborn
/// 0.13.2's split of a [`Style`], a [`Context`] that scales every size, a
/// font scale for text alone, a palette and [`Domains`]. Sizes are
/// seaborn's `plotting_context` values in points at 4/3 px per pt, times
/// the context's factor (text also times the font scale):
///
/// | Setting | pt at notebook |
/// | --- | --- |
/// | text, axis labels, panel titles, notes | 12 |
/// | tick labels, legend entries (`em`) | 11 |
/// | legend titles | 12 |
/// | figure title | 14.4 (matplotlib's `large`) |
/// | axis lines and major ticks: width | 1.25 |
/// | grid lines: width | 1 |
/// | lines and rules: width | 1.5 |
/// | outlines of bars and rects: width | 1 |
/// | major ticks: length | 6 |
/// | minor ticks: length / width | 4 / 1 |
/// | markers: diameter | 6 |
/// | pads: tick label, axis label, title | 3.5, 4, 6 |
///
/// Fields are private and set by builders only. A builder given input it
/// rejects leaves the theme as it was: there is no theme error type.
///
/// ```
/// use rustd3_figure::Theme;
/// use rustd3_figure::theme::{Context, Domains, Style, palette};
///
/// assert_eq!(
///     Theme::default(),
///     Theme::seaborn(Style::Ticks, Context::Notebook)
///         .palette(palette::COLORBLIND)
///         .domains(Domains::Margin(0.05)),
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Colors, and which decorations are drawn.
    style: Style,
    /// The factor every size is multiplied by.
    context: Context,
    /// The factor text alone is multiplied by.
    font_scale: f64,
    /// Marks without a key take its first color.
    palette: Vec<Rgb>,
    /// How fitted domains end.
    domains: Domains,
    /// Text, tick labels, tick marks and the white styles' axis lines.
    ink: Rgb,
    /// Behind the whole figure.
    background: Rgb,
}

impl Default for Theme {
    /// seaborn's `ticks` style at `notebook` size with the colorblind
    /// palette and 5% margins: tick labels 11 pt, axis labels and titles
    /// 12 pt, the figure title 14.4 pt, ink `#262626` for text, ticks and
    /// axis lines, white panels and figure, no grid; lines 1.5 pt wide,
    /// markers 6 pt across.
    fn default() -> Self {
        Theme::seaborn(Style::Ticks, Context::Notebook)
    }
}

impl Theme {
    /// seaborn's `style` at `context`, with the colorblind palette, 5%
    /// margins, the ink `#262626` and a white figure.
    pub fn seaborn(style: Style, context: Context) -> Self {
        Theme {
            style,
            context,
            font_scale: 1.0,
            palette: scheme::COLORBLIND.to_vec(),
            domains: Domains::Margin(0.05),
            ink: INK,
            background: Rgb::hex(0xffffff),
        }
    }

    /// The palette marks without a key are drawn from (its first color);
    /// the colors are copied. An empty palette, or one with a color whose
    /// channels or opacity are not finite, leaves the theme as it was.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_figure::Theme;
    /// use rustd3_figure::theme::palette;
    ///
    /// let mine = vec![Rgb::hex(0x336699)];
    /// let _ = Theme::default().palette(palette::DEEP).palette(&mine);
    /// assert_eq!(Theme::default().palette(Vec::<Rgb>::new()), Theme::default());
    /// ```
    pub fn palette(self, colors: impl AsRef<[Rgb]>) -> Self {
        let colors = colors.as_ref();
        if colors.is_empty() || !colors.iter().copied().all(finite) {
            return self;
        }
        Theme {
            palette: colors.to_vec(),
            ..self
        }
    }

    /// Scales text only, by `k`: tick labels, axis labels, titles, legends
    /// and notes, and with them the `em` lengths of gaps; line widths,
    /// tick sizes and pads stay. A `k` that is not finite and positive
    /// leaves the theme as it was.
    pub fn font_scale(self, k: f64) -> Self {
        if k.is_finite() && k > 0.0 {
            Theme {
                font_scale: k,
                ..self
            }
        } else {
            self
        }
    }

    /// How fitted continuous and time domains end: margins or nice
    /// values. `Domains::Margin(m)` with `m` outside [0, 0.5) leaves the
    /// theme as it was.
    pub fn domains(self, d: Domains) -> Self {
        match d {
            Domains::Margin(m) if !(0.0..0.5).contains(&m) => self,
            Domains::Margin(_) | Domains::Nice => Theme { domains: d, ..self },
        }
    }

    /// Text, tick labels, tick marks and the white styles' axis lines in
    /// `c`; seaborn's ".15", `#262626`, by default. The iced widget draws
    /// its crosshair and highlight in it. A color with a channel or an
    /// opacity that is not finite leaves the theme as it was.
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

    /// Text, tick labels, tick marks and the white styles' axis lines.
    pub fn get_ink(&self) -> Rgb {
        self.ink
    }

    /// Behind the whole figure.
    pub fn get_background(&self) -> Rgb {
        self.background
    }

    /// Axis lines and borders: 1.25 pt.
    pub(crate) fn axis_width(&self) -> f64 {
        self.pt(1.25)
    }

    /// Major ticks' width: 1.25 pt.
    pub(crate) fn tick_width(&self) -> f64 {
        self.pt(1.25)
    }

    /// Grid lines' width: 1 pt.
    pub(crate) fn grid_width(&self) -> f64 {
        self.pt(1.0)
    }

    /// Axis lines' color; `None` in the styles that draw none.
    pub(crate) fn edge(&self) -> Option<Rgb> {
        self.style.edge(self.ink)
    }

    /// Grid lines' color; `None` in the styles without a grid.
    pub(crate) fn grid(&self) -> Option<Rgb> {
        self.style.grid()
    }

    /// Whether tick marks are drawn (the ticks style); without them tick
    /// labels keep their distance from the axis.
    pub(crate) fn ticks(&self) -> bool {
        self.style.ticks()
    }

    /// Behind each plot area.
    pub(crate) fn panel(&self) -> Rgb {
        self.style.panel()
    }

    /// Tick labels: 11 pt.
    pub(crate) fn tick_text(&self) -> f64 {
        self.text(11.0)
    }

    /// Axis labels, panel titles and notes: 12 pt.
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

    /// Tick marks: 6 pt; the room they take whether drawn or not.
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

    /// How fitted continuous and time domains end.
    pub(crate) fn get_domains(&self) -> Domains {
        self.domains
    }

    /// The width lines and rules are stroked at, px: 1.5 pt at this
    /// context. A custom mark that draws lines, whiskers or error bars
    /// strokes them at this width to match the line and rule marks.
    pub fn get_line_width(&self) -> f64 {
        self.pt(1.5)
    }

    /// The width outlines of filled shapes are stroked at, px: 1 pt at this
    /// context, seaborn's `patch.linewidth`. A custom mark that outlines
    /// boxes or bars strokes their edges at this width, as the rect mark
    /// does.
    pub fn get_patch_width(&self) -> f64 {
        self.pt(1.0)
    }

    /// The width a density's outline is stroked at, px: 0.75 pt at this
    /// context. A custom mark that outlines a filled curve (a violin, a
    /// ridge) strokes it at this width to match the density marks.
    pub fn get_density_width(&self) -> f64 {
        self.pt(0.75)
    }

    /// A marker's diameter, px: 6 pt at this context. A custom mark that
    /// draws points (fliers, means) sizes them by it, as dots without a
    /// radius are.
    pub fn get_marker(&self) -> f64 {
        self.pt(6.0)
    }

    /// The palette's first color. A custom mark without a key draws in it,
    /// as every built-in mark does.
    pub fn get_color(&self) -> Rgb {
        *self
            .palette
            .first()
            .expect("invariant: a theme's palette is not empty")
    }

    /// seaborn's gray, `#949494`. A custom mark whose key its palette lacks
    /// draws in it, as the built-in marks do (and
    /// [`Figure::check`](crate::Figure::check) reports the key).
    pub fn get_gray(&self) -> Rgb {
        GRAY
    }

    /// A tick label's style: its lines aligned by `align`, in `fill`.
    pub(crate) fn tick_label(&self, align: anchor::Horizontal, fill: Rgb) -> text::Style {
        style(self.tick_text(), align, fill)
    }

    /// An axis label's style: centred lines in `fill`.
    pub(crate) fn axis_label(&self, fill: Rgb) -> text::Style {
        style(self.label_text(), anchor::Horizontal::Middle, fill)
    }

    /// A panel title's style: centred lines in ink.
    pub(crate) fn title(&self) -> text::Style {
        style(self.label_text(), anchor::Horizontal::Middle, self.ink)
    }

    /// A mark's label (a rule's): tick-label size, its lines aligned by
    /// `align`, in `fill`.
    pub(crate) fn mark_label(&self, align: anchor::Horizontal, fill: Rgb) -> text::Style {
        style(self.tick_text(), align, fill)
    }

    /// A note's text: the axis-label size (matplotlib's `font.size` at
    /// seaborn's notebook, 12 pt), its lines aligned by `align`, in `fill`.
    pub(crate) fn note(&self, align: anchor::Horizontal, fill: Rgb) -> text::Style {
        style(self.label_text(), align, fill)
    }

    /// A legend entry's label: tick-label size (matplotlib's
    /// `legend.fontsize` at seaborn's notebook, 11 pt), in ink.
    pub(crate) fn legend_label(&self) -> text::Style {
        style(self.tick_text(), anchor::Horizontal::Start, self.ink)
    }

    /// A legend's title: 12 pt (`legend.title_fontsize`), centred lines in
    /// ink.
    pub(crate) fn legend_title(&self) -> text::Style {
        style(self.label_text(), anchor::Horizontal::Middle, self.ink)
    }

    /// Behind an inset legend: the panel's color at matplotlib's
    /// `legend.framealpha`, 0.8.
    pub(crate) fn legend_face(&self) -> Rgb {
        self.panel().with_opacity(0.8)
    }

    /// An inset legend's edge: matplotlib's `legend.edgecolor`, gray 0.8.
    pub(crate) fn legend_edge(&self) -> Rgb {
        Rgb::hex(0xcccccc)
    }

    /// The figure title's style: centred lines in ink.
    pub(crate) fn figure_title(&self) -> text::Style {
        style(
            self.figure_title_text(),
            anchor::Horizontal::Middle,
            self.ink,
        )
    }

    /// `v` pt at this context, in px.
    pub(crate) fn pt(&self, v: f64) -> f64 {
        v * PX_PER_PT * self.context.scale()
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
fn style(size: f64, align: anchor::Horizontal, fill: Rgb) -> text::Style {
    text::Style {
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

    /// `px` in pt.
    fn pt(px: f64) -> f64 {
        px / PX_PER_PT
    }

    /// Every size the tables set, in pt: text, tick labels (and legend
    /// entries), legend titles, the figure title; axis line, grid, line,
    /// patch edge, major tick and minor tick widths; major and minor tick
    /// sizes; the marker.
    fn sizes(theme: &Theme) -> [f64; 13] {
        [
            theme.label_text(),
            theme.tick_text(),
            theme.legend_title().size,
            theme.figure_title_text(),
            theme.axis_width(),
            theme.grid_width(),
            theme.get_line_width(),
            theme.get_patch_width(),
            theme.tick_width(),
            theme.minor_width(),
            theme.tick(),
            theme.minor_tick(),
            theme.get_marker(),
        ]
        .map(pt)
    }

    #[test]
    fn every_size_is_seaborns_notebook_value_times_the_context() {
        let notebook = [
            12.0, 11.0, 12.0, 14.4, 1.25, 1.0, 1.5, 1.0, 1.25, 1.0, 6.0, 4.0, 6.0,
        ];
        for context in [
            Context::Paper,
            Context::Notebook,
            Context::Talk,
            Context::Poster,
        ] {
            let theme = Theme::seaborn(Style::Ticks, context);
            let k = context.scale();
            for (got, want) in sizes(&theme).into_iter().zip(notebook) {
                assert!(
                    close(got, want * k),
                    "{context:?}: {got} against {}",
                    want * k
                );
            }
            assert!(close(theme.em(), theme.tick_text()));
            let pads = [theme.tick_pad(), theme.label_pad(), theme.title_pad()].map(pt);
            assert!(close(pads[0], 3.5 * k) && close(pads[1], 4.0 * k) && close(pads[2], 6.0 * k));
        }
    }

    #[test]
    fn the_spot_rows_of_the_contexts() {
        let talk = Theme::seaborn(Style::Ticks, Context::Talk);
        assert!(close(talk.tick_text(), 22.0), "{}", talk.tick_text());
        let paper = Theme::seaborn(Style::Ticks, Context::Paper);
        assert!(close(pt(paper.get_line_width()), 1.2) && close(pt(paper.tick()), 4.8));
        let poster = Theme::seaborn(Style::Ticks, Context::Poster);
        assert!(close(pt(poster.label_text()), 24.0) && close(pt(poster.axis_width()), 2.5));
    }

    #[test]
    fn the_font_scale_scales_text_alone() {
        let base = Theme::default();
        let scaled = Theme::default().font_scale(1.1);
        assert!(
            close(pt(scaled.tick_text()), 12.1),
            "{}",
            pt(scaled.tick_text())
        );
        assert!(close(scaled.em(), scaled.tick_text()));
        let text = |t: &Theme| {
            [
                t.label_text(),
                t.tick_text(),
                t.legend_title().size,
                t.figure_title_text(),
            ]
        };
        for (s, b) in text(&scaled).into_iter().zip(text(&base)) {
            assert!(close(s, 1.1 * b));
        }
        assert_eq!(sizes(&scaled)[4..], sizes(&base)[4..]);
        let pads = |t: &Theme| [t.tick_pad(), t.label_pad(), t.title_pad()];
        assert_eq!(pads(&scaled), pads(&base));
    }

    #[test]
    fn styles_set_the_panel_axis_lines_grid_and_tick_marks() {
        let white = Rgb::hex(0xffffff);
        let gray = Rgb::hex(0xcccccc);
        let dark = Rgb::hex(0xeaeaf2);
        let rows = [
            (Style::DarkGrid, dark, None, Some(white), false),
            (Style::WhiteGrid, white, Some(gray), Some(gray), false),
            (Style::Dark, dark, None, None, false),
            (Style::White, white, Some(INK), None, false),
            (Style::Ticks, white, Some(INK), None, true),
        ];
        for (style, panel, edge, grid, ticks) in rows {
            let theme = Theme::seaborn(style, Context::Notebook);
            assert_eq!(
                (theme.panel(), theme.edge(), theme.grid(), theme.ticks()),
                (panel, edge, grid, ticks),
                "{style:?}"
            );
            assert_eq!(
                (theme.get_ink(), theme.get_background()),
                (INK, white),
                "{style:?}"
            );
            // Tick marks or not, labels keep their distance.
            assert!(close(theme.tick(), 8.0));
        }
    }

    #[test]
    fn default_theme_sizes() {
        let theme = Theme::default();
        assert!(
            close(theme.tick_text(), 44.0 / 3.0),
            "{}",
            theme.tick_text()
        );
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
        assert_eq!(theme.edge(), Some(theme.get_ink()));
        assert_eq!(theme.get_domains(), Domains::Margin(0.05));
        assert!(close(theme.get_line_width(), 2.0));
        assert!(close(theme.get_marker(), 8.0));
        assert_eq!(theme.get_color(), Rgb::hex(0x0173b2));
        assert_eq!(theme.get_gray(), scheme::COLORBLIND[7]);
        assert_eq!(theme.palette, scheme::COLORBLIND);
        assert_eq!(
            (theme.style, theme.context, theme.font_scale),
            (Style::Ticks, Context::Notebook, 1.0)
        );
    }

    #[test]
    fn ink_and_background_builders_set_what_the_readers_return() {
        let red = Rgb::hex(0xff0000);
        let half = Rgb::hex(0x336699).with_opacity(0.5);
        let theme = Theme::default().ink(red).background(half);
        assert_eq!((theme.get_ink(), theme.get_background()), (red, half));
        // The edge and every text style follow the ink.
        assert_eq!(theme.edge(), Some(red));
        assert_eq!(theme.title().fill, red);
        assert_eq!(theme.figure_title().fill, red);
        assert_eq!(theme.legend_label().fill, red);
        // The panel keeps its own color, and whitegrid its gray edge.
        assert_eq!(theme.panel(), Rgb::hex(0xffffff));
        let grid = Theme::seaborn(Style::WhiteGrid, Context::Notebook).ink(red);
        assert_eq!(grid.edge(), Some(Rgb::hex(0xcccccc)));
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
            assert_eq!(theme.clone().palette([Rgb::hex(0x336699), bad]), theme);
        }
    }

    #[test]
    fn builders_reject_what_they_document() {
        let theme = Theme::default();
        for k in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(theme.clone().font_scale(k), theme);
        }
        for m in [-0.01, 0.5, 0.7, f64::NAN] {
            assert_eq!(theme.clone().domains(Domains::Margin(m)), theme);
        }
        assert_eq!(theme.clone().palette(Vec::<Rgb>::new()), theme);
        let kept = theme.clone().domains(Domains::Margin(0.0)).font_scale(0.5);
        assert_eq!(kept.get_domains(), Domains::Margin(0.0));
        assert!(close(kept.font_scale, 0.5));
        let deep = theme.palette(scheme::DEEP);
        assert_eq!(deep.get_color(), scheme::DEEP[0]);
    }
}
