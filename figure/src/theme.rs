//! What a [`Theme`] is built from, seaborn 0.13.2's split:
//! a [`Style`] (colors, and which of grid lines, axis lines and tick marks
//! are drawn), a [`Context`] that scales every size at once, a font scale,
//! a [`palette`], and how fitted domains end ([`Domains`]).
//!
//! ```
//! use rustd3_figure::theme::{Context, Domains, Style, palette};
//! use rustd3_figure::{Figure, Theme};
//!
//! let mut fig = Figure::new();
//! // seaborn's structure, with this crate's defaults.
//! fig.theme(Theme::default());
//!
//! fig.theme(
//!     Theme::seaborn(Style::WhiteGrid, Context::Paper)
//!         .palette(palette::DEEP)
//!         .font_scale(1.1)
//!         .domains(Domains::Nice),
//! );
//! ```

mod context;
mod domains;
pub mod palette;
pub(crate) mod settings;
mod style;

pub use context::Context;
pub use domains::Domains;
pub(crate) use settings::Theme;
pub use style::Style;
