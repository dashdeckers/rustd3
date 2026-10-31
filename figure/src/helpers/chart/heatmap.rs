//! [`Heatmap`]: a grid of values in color, with a colorbar.

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_core::scheme;

use crate::Figure;
use crate::colorbar::colorbar;
use crate::helpers::chart::setup::Setup;
use crate::legend::Slot;
use crate::mark::{Grid, image};

/// A heatmap: a grid's values through a sequential color scale, with a
/// colorbar (see [`heatmap`]).
pub struct Heatmap {
    grid: Grid,
    scheme: fn(f64) -> Rgb,
    /// The rows' and the columns' keys, when given.
    keys: Option<(Vec<String>, Vec<String>)>,
    setup: Setup,
}

/// A heatmap of `grid`: an [`image`] of its cells through a sequential
/// color scale fitted to its values (viridis unless
/// [`colors`](Heatmap::colors) says otherwise), filling the plot area, and
/// a [`colorbar`] of the scale right of it. Its axes are linear over the
/// grid's extent, row 0 at the extent's first y (at the bottom unless the
/// extent runs down), with minor ticks; [`keys`](Heatmap::keys) puts band
/// axes of row and column names in their place.
///
/// ```
/// use rustd3_figure::helpers::chart;
/// use rustd3_figure::mark::Grid;
///
/// let values = (0..24).map(|i| f64::from(i % 6) * f64::from(i / 6)).collect();
/// let grid = Grid::new(6, 4, values)?.extent([0.0, 3.0], [0.0, 2.0]);
/// let fig = chart::heatmap(grid).x_label("x [m]").y_label("y [m]").figure();
/// # Ok::<(), rustd3_figure::mark::Error>(())
/// ```
#[must_use]
pub fn heatmap(grid: Grid) -> Heatmap {
    Heatmap {
        grid,
        scheme: scheme::viridis,
        keys: None,
        setup: Setup::default(),
    }
}

impl Heatmap {
    /// The color scheme the values map onto, low to high: viridis by
    /// default, any of core's sequential schemes (`scheme::magma`,
    /// `scheme::blues`, …) or a function of its own over [0, 1].
    ///
    /// ```
    /// use rustd3_core::scheme;
    /// use rustd3_figure::helpers::chart;
    /// use rustd3_figure::mark::Grid;
    ///
    /// let grid = Grid::new(2, 2, vec![0.0, 1.0, 2.0, 3.0])?;
    /// let fig = chart::heatmap(grid).colors(scheme::magma).figure();
    /// # Ok::<(), rustd3_figure::mark::Error>(())
    /// ```
    #[must_use]
    pub fn colors(self, scheme: fn(f64) -> Rgb) -> Self {
        Heatmap { scheme, ..self }
    }

    /// Names the grid's rows and columns, in order: band axes of the names
    /// replace the linear ones, the cells fill the bands, and rows run top
    /// to bottom as a table's do (seaborn's `heatmap`), row 0 on top. The
    /// names count the grid's rows and columns; the grid's extent is
    /// replaced by one unit per name.
    ///
    /// ```
    /// use rustd3_figure::helpers::chart;
    /// use rustd3_figure::mark::Grid;
    ///
    /// let r = vec![1.0, 0.8, -0.2, 0.8, 1.0, 0.1, -0.2, 0.1, 1.0];
    /// let names: Vec<String> = ["speed", "load", "temp"].map(String::from).to_vec();
    /// let fig = chart::heatmap(Grid::new(3, 3, r)?)
    ///     .keys(names.clone(), names)
    ///     .title("correlation")
    ///     .figure();
    /// # Ok::<(), rustd3_figure::mark::Error>(())
    /// ```
    #[must_use]
    pub fn keys(self, rows: Vec<String>, cols: Vec<String>) -> Self {
        Heatmap {
            keys: Some((rows, cols)),
            ..self
        }
    }

    /// The figure: one panel with the image, its axes and title, and the
    /// colorbar.
    #[must_use]
    pub fn figure<'a>(self) -> Figure<'a> {
        let Heatmap {
            grid,
            scheme,
            keys,
            setup,
        } = self;
        let mut fig = setup.figure();
        let heat = fig.fit(scale::sequential([0.0, 1.0], scheme));
        match keys {
            None => {
                let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
                let panel = fig.panel(0, 0);
                panel.mark(image(grid, x, y, heat));
                setup.axes(panel, x, y);
            }
            Some((rows, cols)) => {
                let (n, m) = (rows.len() as f64, cols.len() as f64);
                let x = fig.scale(scale::linear().domain([0.0, m]));
                let y = fig.scale(scale::linear().domain([0.0, n]));
                let across = fig.scale(scale::band(cols));
                let up = fig.scale(scale::band(rows.into_iter().rev()));
                let panel = fig.panel(0, 0);
                panel.mark(image(grid.extent([0.0, m], [n, 0.0]), x, y, heat));
                setup.axes(panel, across, up);
            }
        }
        fig.legend(Slot::Right, colorbar(heat));
        fig
    }
}

finish!(impl[] Heatmap);
