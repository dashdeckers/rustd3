//! [`Line`]: a line chart of one or more series.

use rustd3_core::scale::{self, AnyContinuous};
use rustd3_core::series::{Data, Points};

use crate::Figure;
use crate::helpers::chart::setup::{self, Setup};
use crate::legend::{self, Slot, Swatch};
use crate::mark;

/// A line chart: one line per series on fitted x and y scales (see
/// [`line()`]).
pub struct Line<'a> {
    /// Each series' points and key, in order.
    series: Vec<(Points<'a, f64, f64>, String)>,
    x: AnyContinuous,
    y: AnyContinuous,
    setup: Setup,
}

impl<'a> std::fmt::Debug for Line<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line").finish_non_exhaustive()
    }
}

/// A line chart of `data` (pairs, two slices or a recording channel, as
/// [`mark::line`] takes them; a channel thins to M4's points per column),
/// more series added with [`series`](Line::series): each a
/// [`mark::line`] in its key's color on linear x and y scales fitted to
/// every series, a bottom and a left axis with minor ticks, and a legend
/// of lines right of the plot area when there is more than one series.
/// The first series' key is "0", pandas' name for an unnamed column, until
/// [`key`](Line::key) names it. Keys are the legend's labels.
///
/// A line over one channel of a recording:
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::series::{Clock, Recording};
/// use rustd3_figure::helpers;
/// use rustd3_figure::text;
///
/// let samples: Vec<f64> = (0..2000).map(|i| 50.0 * (f64::from(i) * 0.01).sin().powi(3)).collect();
/// let rec = Recording::new(Clock::Regular { start: 0.0, period: 0.01 }, [&samples])?;
/// let rms = helpers::chart::line(
///     rec.series(0).expect("channel 0"),
/// )
/// .y_scale(scale::symlog().constant(1.0))
/// .minor(true)
/// .title(text::typst(
///     "RMS [$m slash s^2$]",
/// ));
/// # #[cfg(all(feature = "png", feature = "fonts"))] {
/// # let path = std::env::temp_dir().join(format!("rustd3-doc-{}-rms.png", std::process::id()));
/// // 1280 × 960 px.
/// rms.save_png(&path, 2.0)?;
/// # std::fs::remove_file(&path).expect("the file is removed");
/// # }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn line<'a>(data: impl Data<'a, f64, f64>) -> Line<'a> {
    Line {
        series: vec![(data.into_points(), "0".to_string())],
        x: scale::linear().into(),
        y: scale::linear().into(),
        setup: Setup::default(),
    }
}

impl<'a> Line<'a> {
    /// Names the first series `name` in place of "0".
    ///
    /// ```
    /// use rustd3_figure::helpers::chart;
    ///
    /// let t: Vec<f64> = (0..50).map(f64::from).collect();
    /// let fig = chart::line(t.iter().map(|&t| (t, t.sqrt())))
    ///     .key("sqrt")
    ///     .series(t.iter().map(|&t| (t, 0.1 * t)), "linear")
    ///     .figure();
    /// ```
    #[must_use]
    pub fn key(mut self, name: impl Into<String>) -> Self {
        if let Some((_, key)) = self.series.first_mut() {
            *key = name.into();
        }
        self
    }

    /// Another series, `data`, under `key`, drawn after the earlier ones in
    /// the palette's next color. A key given twice is an
    /// [`Issue::DuplicateKey`](crate::check::Issue::DuplicateKey) that
    /// [`Figure::check`] reports; its series share a color and a legend
    /// entry.
    ///
    /// ```
    /// use rustd3_figure::helpers::chart;
    ///
    /// let t: Vec<f64> = (0..50).map(f64::from).collect();
    /// let fig = chart::line(t.iter().map(|&t| (t, t.sqrt())))
    ///     .series(t.iter().map(|&t| (t, 0.1 * t)), "linear")
    ///     .x_label("t [s]")
    ///     .figure();
    /// ```
    #[must_use]
    pub fn series(mut self, data: impl Data<'a, f64, f64>, key: impl Into<String>) -> Self {
        self.series.push((data.into_points(), key.into()));
        self
    }

    /// The x scale, fitted to the series' x: linear by default; log,
    /// symlog, asinh, logit and the others through
    /// [`AnyContinuous`]'s conversions.
    #[must_use]
    pub fn x_scale(self, s: impl Into<AnyContinuous>) -> Self {
        Line {
            x: s.into(),
            ..self
        }
    }

    /// The y scale, fitted to the series' y: linear by default.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::helpers::chart;
    ///
    /// let fig = chart::line((1..=6).map(|k| (f64::from(k), 10f64.powi(k))))
    ///     .y_scale(scale::log())
    ///     .figure();
    /// ```
    #[must_use]
    pub fn y_scale(self, s: impl Into<AnyContinuous>) -> Self {
        Line {
            y: s.into(),
            ..self
        }
    }

    /// The figure: one panel with the lines, its axes and title, and the
    /// legend when there is more than one series.
    #[must_use]
    pub fn figure(self) -> Figure<'a> {
        let Line {
            series,
            x,
            y,
            setup,
        } = self;
        let mut fig = setup.figure();
        setup::duplicates(&mut fig, series.iter().map(|(_, key)| key));
        let (x, y) = (fig.fit(x), fig.fit(y));
        let color = fig.scale(setup.palette(series.iter().map(|(_, key)| key.clone())));
        let many = series.len() > 1;
        let panel = fig.panel(0, 0);
        for (points, key) in series {
            panel.mark(mark::line(points, x, y).stroke(color, key));
        }
        setup.axes(panel, x, y);
        if many {
            fig.legend(
                Slot::Right,
                legend::legend(color).glyph(Swatch::Line { dash: None }),
            );
        }
        fig
    }
}

finish!(impl['a] Line<'a>);
