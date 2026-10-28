//! [`Context`]: seaborn's `plotting_context` presets.

/// seaborn 0.13.2's `plotting_context` presets: one factor for every size
/// of the theme, text, line widths, tick sizes, pads and `em` gaps alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// 0.8: small figures in print.
    Paper,
    /// 1: the default.
    Notebook,
    /// 1.5: slides.
    Talk,
    /// 2: posters.
    Poster,
}

impl Context {
    /// The factor every size is multiplied by: 0.8, 1, 1.5 or 2.
    ///
    /// ```
    /// use rustd3_figure::theme::Context;
    ///
    /// assert_eq!(Context::Talk.scale(), 1.5);
    /// ```
    pub fn scale(self) -> f64 {
        match self {
            Context::Paper => 0.8,
            Context::Notebook => 1.0,
            Context::Talk => 1.5,
            Context::Poster => 2.0,
        }
    }
}
