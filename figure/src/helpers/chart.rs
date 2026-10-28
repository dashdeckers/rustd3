//! One-call chart presets, matplotlib's and seaborn's conveniences as
//! consuming builders that end in a figure: [`line()`] (one or more
//! series), [`scatter()`], [`bar()`] (grouped, stacked or diverging, see
//! [`bar::Mode`]), [`area()`] (stacked), [`pie()`] (and donuts) and
//! [`heatmap()`] (with a colorbar).
//!
//! Every preset takes the same finishing builders: `.title(t)` (the panel's
//! title), `.x_label(t)` and `.y_label(t)` (the axes' labels), `.size(w, h)`
//! (640 × 480 px by default, matplotlib's 6.4 × 4.8 in at 100 dpi),
//! `.theme(theme)` and `.minor(on)` (minor ticks, on by default: the axes
//! go through [`axes::defaults`](crate::helpers::axes::defaults)), and ends
//! in `.figure()`, a [`Figure`](crate::Figure) to change further, or in a
//! file at its size, `.save_svg(path)` or `.save_png(path, scale)`.
//!
//! Series take their colors from the theme's palette
//! ([`Theme::get_palette`](crate::Theme::get_palette)) through an ordinal
//! scale over their keys, and a legend of the keys goes right of the plot
//! area when there is more than one series. Each preset is ordinary code
//! over the figure's handles and marks, a copy of which makes the chart a
//! preset does not; the pie's margin (half an em) is the one thing the
//! presets read from the theme that a copy outside this crate names
//! itself.
//!
//! ```
//! use rustd3_figure::helpers::chart;
//!
//! let fig = chart::bar(
//!     [("2024", vec![3.0, 1.0]), ("2025", vec![2.0, 4.0])],
//!     vec!["apples".into(), "pears".into()],
//! )
//! .mode(chart::bar::Mode::Stacked)
//! .title("fruit sold")
//! .figure();
//! ```

/// The finishing builders every preset shares (see the module's docs), for
/// the preset type `$preset` with the generics and bounds `$g`; each preset
/// holds its [`setup::Setup`] as `setup` and has its own `figure`.
macro_rules! finish {
    (impl[$($g:tt)*] $preset:ty) => {
        impl<$($g)*> $preset {
            /// The plot area's title (the panel's), centred over it.
            pub fn title(mut self, t: impl Into<crate::text::Text>) -> Self {
                self.setup.title = Some(t.into());
                self
            }

            /// The label of the axis along x, under its tick labels.
            pub fn x_label(mut self, t: impl Into<crate::text::Text>) -> Self {
                self.setup.x_label = Some(t.into());
                self
            }

            /// The label of the axis along y, beside its tick labels.
            pub fn y_label(mut self, t: impl Into<crate::text::Text>) -> Self {
                self.setup.y_label = Some(t.into());
                self
            }

            /// The size the files are written at, logical px: 640 × 480 by
            /// default, matplotlib's 6.4 × 4.8 in at 100 dpi. A side that is
            /// not finite and positive leaves the size as it was.
            pub fn size(mut self, w: f64, h: f64) -> Self {
                self.setup.size(w, h);
                self
            }

            /// The theme the figure is drawn in; [`Theme::default`]
            /// (seaborn's ticks at notebook, colorblind palette) unless
            /// set.
            ///
            /// [`Theme::default`]: crate::Theme::default
            pub fn theme(mut self, t: crate::Theme) -> Self {
                self.setup.theme = t;
                self
            }

            /// Minor ticks on the axes that have them, as
            /// [`axes::defaults`](crate::helpers::axes::defaults) turns them
            /// on; on by default.
            pub fn minor(mut self, on: bool) -> Self {
                self.setup.minor = on;
                self
            }

            /// Writes the figure as SVG to `path` at its size
            /// ([`Figure::save_svg`](crate::Figure::save_svg)); a preset
            /// draws nothing that lowering changes.
            #[cfg(feature = "fonts")]
            pub fn save_svg(self, path: impl AsRef<std::path::Path>) -> Result<(), crate::Error> {
                let size = self.setup.size;
                self.figure().save_svg(path, size).map(|_| ())
            }

            /// Writes the figure as a PNG of its size × `scale` device px to
            /// `path` ([`Figure::save_png`](crate::Figure::save_png)).
            #[cfg(all(feature = "png", feature = "fonts"))]
            pub fn save_png(
                self,
                path: impl AsRef<std::path::Path>,
                scale: f64,
            ) -> Result<(), crate::Error> {
                let size = self.setup.size;
                self.figure().save_png(path, size, scale)
            }
        }
    };
}

mod area;
pub mod bar;
mod heatmap;
mod line;
mod pie;
mod scatter;
mod setup;

pub use area::{Area, area};
pub use bar::preset::{Bar, bar};
pub use heatmap::{Heatmap, heatmap};
pub use line::{Line, line};
pub use pie::{Pie, pie};
pub use scatter::{Scatter, scatter};
