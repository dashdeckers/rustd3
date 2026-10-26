//! Themes as seaborn splits them: a style (colors and
//! which decorations exist), a context (one scale for
//! every size), a font scale and a palette. Sizes are
//! seaborn's notebook values in points; 1 pt = 4/3 px.

use d3::color::Rgb;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Style {
    DarkGrid,
    WhiteGrid,
    Ticks,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Context {
    Paper,
    Notebook,
    Talk,
    Poster,
}

impl Context {
    pub fn scale(self) -> f64 {
        match self {
            Context::Paper => 0.8,
            Context::Notebook => 1.0,
            Context::Talk => 1.5,
            Context::Poster => 2.0,
        }
    }
}

/// How fitted domains end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Domains {
    /// Extend by this share of the span on each side,
    /// except at zero, which marks start from.
    Margin(f64),
    /// Round outward to tick values, as d3's `nice`.
    Nice,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub style: Style,
    pub context: Context,
    pub font_scale: f64,
    pub domains: Domains,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::seaborn(Style::Ticks, Context::Notebook)
    }
}

const PX_PER_PT: f64 = 4.0 / 3.0;

impl Theme {
    pub fn seaborn(style: Style, context: Context) -> Self {
        Theme {
            style,
            context,
            font_scale: 1.0,
            domains: Domains::Margin(0.05),
        }
    }

    pub fn font_scale(self, font_scale: f64) -> Self {
        Self { font_scale, ..self }
    }

    pub fn domains(self, domains: Domains) -> Self {
        Self { domains, ..self }
    }

    /// Points at this context, in px.
    pub fn pt(&self, v: f64) -> f64 {
        v * PX_PER_PT * self.context.scale()
    }

    fn text(&self, v: f64) -> f64 {
        self.pt(v) * self.font_scale
    }

    pub fn tick_text(&self) -> f64 {
        self.text(11.0)
    }

    pub fn label_text(&self) -> f64 {
        self.text(12.0)
    }

    pub fn title_text(&self) -> f64 {
        self.text(12.0)
    }

    pub fn figure_title_text(&self) -> f64 {
        self.text(14.4)
    }

    /// What `em` gaps are multiples of.
    pub fn em(&self) -> f64 {
        self.tick_text()
    }

    /// Average glyph width, until typst measures.
    pub fn char_w(&self) -> f64 {
        0.6 * self.tick_text()
    }

    /// Tick marks exist only in the ticks style.
    pub fn tick(&self) -> f64 {
        match self.style {
            Style::Ticks => self.pt(6.0),
            _ => 0.0,
        }
    }

    pub fn tick_pad(&self) -> f64 {
        self.pt(3.5)
    }

    pub fn label_pad(&self) -> f64 {
        self.pt(4.0)
    }

    pub fn title_pad(&self) -> f64 {
        self.pt(6.0)
    }

    pub fn axis_width(&self) -> f64 {
        self.pt(1.25)
    }

    pub fn grid_width(&self) -> f64 {
        self.pt(1.0)
    }

    pub fn line_width(&self) -> f64 {
        self.pt(1.5)
    }

    pub fn dot(&self) -> f64 {
        self.pt(1.2)
    }

    pub fn ink(&self) -> Rgb {
        Rgb::hex(0x262626)
    }

    pub fn panel(&self) -> Rgb {
        match self.style {
            Style::DarkGrid => Rgb::hex(0xeaeaf2),
            _ => Rgb::hex(0xffffff),
        }
    }

    /// The axis line; darkgrid has none.
    pub fn edge(&self) -> Option<Rgb> {
        match self.style {
            Style::DarkGrid => None,
            Style::WhiteGrid => Some(Rgb::hex(0xcccccc)),
            Style::Ticks => Some(self.ink()),
        }
    }

    pub fn grid(&self) -> Option<Rgb> {
        match self.style {
            Style::DarkGrid => Some(Rgb::hex(0xffffff)),
            Style::WhiteGrid => Some(Rgb::hex(0xcccccc)),
            Style::Ticks => None,
        }
    }
}
