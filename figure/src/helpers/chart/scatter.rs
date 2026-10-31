//! [`Scatter`]: a scatter plot.

use rustd3_core::scale;
use rustd3_core::series::{Data, Points};

use crate::Figure;
use crate::helpers::chart::setup::Setup;
use crate::mark;

/// A scatter plot: dots on fitted linear x and y scales (see [`scatter`]).
pub struct Scatter<'a> {
    points: Points<'a, f64, f64>,
    setup: Setup,
}

/// A scatter plot of `points`: one [`mark::dots`] in the theme palette's
/// first color at the theme's marker size on linear x and y scales fitted
/// to them, a bottom and a left axis with minor ticks. One series, so no
/// legend; sized and colored dots with their legends are
/// [`dots`](mark::dots)' `size_by` and `fill_by` with
/// [`helpers::legend`](crate::helpers::legend) on a figure of your own.
///
/// ```
/// use rustd3_figure::helpers::chart;
///
/// let points = (0..40).map(|i| {
///     let t = f64::from(i);
///     (t, (t * 0.4).sin() + 0.05 * t)
/// });
/// let fig = chart::scatter(points).x_label("t").y_label("signal").figure();
/// ```
pub fn scatter<'a>(points: impl IntoIterator<Item = (f64, f64)>) -> Scatter<'a> {
    Scatter {
        points: points.into_points(),
        setup: Setup::default(),
    }
}

impl<'a> Scatter<'a> {
    /// The figure: one panel with the dots, its axes and title.
    #[must_use]
    pub fn figure(self) -> Figure<'a> {
        let Scatter { points, setup } = self;
        let mut fig = setup.figure();
        let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
        let panel = fig.panel(0, 0);
        panel.mark(mark::dots(points, x, y));
        setup.axes(panel, x, y);
        fig
    }
}

finish!(impl['a] Scatter<'a>);
