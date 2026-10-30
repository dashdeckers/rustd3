//! [`Figure`]: a table of scales behind typed handles, panels in grid
//! cells that refer to scales only through handles, a layout, a title and
//! a theme; resolving it fits domains and lays the grid out.

pub(crate) mod cache;
mod decorate;
mod draw;
mod fits;
mod held;
mod part;
mod resolve;
mod resolved;
mod save;

use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicU64, Ordering};

use rustd3_core::locale::Locale;

use crate::layout::{Cell, Grid};
use crate::legend::{self, Slot};
use crate::panel::Panel;
use crate::scale::{Fit, Fittable, Kind, Scale, ScaleRef, window};
use crate::text::Text;
use crate::theme::Theme;

pub use cache::Cache;
pub use fits::Fits;
pub use held::Held;
pub use resolved::Resolved;
pub use save::Error;

/// The next figure's id.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// A figure's identity, which its handles carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Id(u64);

/// A figure: a table of scales, panels in grid cells, a layout, a title
/// and a theme. Built with `&mut`, resolved and drawn with `&`.
///
/// Scales are added once and shared through their handles: two panels that
/// use the same handle show the same domain, each over its own pixel range.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::{Figure, Held, Target};
/// use rustd3_figure::kurbo::Size;
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::linear().domain([0.0, 10.0]));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0);
/// # #[cfg(feature = "fonts")] {
/// let mut cache = rustd3_figure::Cache::default();
/// let r = fig.resolve(Size::new(640.0, 480.0), &mut cache, &Held::default(), Target::Vector);
/// assert_eq!(r.scale(x).get_domain(), [0.0, 10.0]);
/// // No mark gives y data: core's empty domain.
/// assert_eq!(r.scale(y).get_domain(), [0.0, 1.0]);
/// assert_eq!(r.plot_areas().len(), 1);
/// # }
/// ```
pub struct Figure<'a> {
    id: Id,
    scales: Vec<Entry>,
    panels: Vec<Panel<'a>>,
    /// The cells asked for past the grid's tracks, and the panel that
    /// stands in for them, never laid out or drawn.
    outside: Vec<Cell>,
    detached: Option<Box<Panel<'a>>>,
    grid: Grid,
    title: Option<Text>,
    theme: Theme,
    locale: Locale,
    legends: Vec<(Slot, legend::Stored<'a>)>,
}

/// One scale of the table, how to fit it when it is fitted, whether to
/// what its panels' x shows (`Fit::Visible`), and the span of x it shows
/// when it is a window (`Fit::Window`): its `Kind::Span`, boxed.
struct Entry {
    scale: Box<dyn Scale>,
    fitter: Option<fits::Fitter>,
    visible: bool,
    window: Option<Box<dyn Any>>,
}

impl Default for Figure<'_> {
    fn default() -> Self {
        Figure::new()
    }
}

impl<'a> Figure<'a> {
    /// An empty figure with a fresh id, equal grid tracks, no title and
    /// the default theme.
    pub fn new() -> Self {
        Figure {
            id: Id(NEXT.fetch_add(1, Ordering::Relaxed)),
            scales: Vec::new(),
            panels: Vec::new(),
            outside: Vec::new(),
            detached: None,
            grid: Grid::default(),
            title: None,
            theme: Theme::default(),
            locale: Locale::default(),
            legends: Vec::new(),
        }
    }

    /// A scale whose domain is the one it was built with.
    pub fn scale<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(Entry {
            scale: Box::new(s),
            fitter: None,
            visible: false,
            window: None,
        })
    }

    /// A scale fitted to the data of every mark that uses it:
    /// `fit_with(s, Fit::All)`.
    pub fn fit<S: Fittable>(&mut self, s: S) -> ScaleRef<S> {
        self.fit_with(s, Fit::All)
    }

    /// A scale whose domain follows its marks' data as `fit` says.
    ///
    /// A `Fit::Window(span)` handle shows the newest `span` of x, in whole
    /// thinning columns of the widest panel that places it across its plot
    /// area (see [`Fit::Window`]): resolving lays the figure out with the
    /// window, measures the plot areas, and fits and lays out again until
    /// the column count it fitted with is the one it laid out (the last
    /// count is remembered in the [`Cache`], so a layout that did not move
    /// takes one pass). `Fit::Window(())` on a categorical scale (span
    /// `()`) means nothing, and debug builds panic on it.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_core::series::{Clock, Recording};
    /// use rustd3_figure::mark::line;
    /// use rustd3_figure::scale::Fit;
    /// use rustd3_figure::{Figure, Held, Target};
    /// use rustd3_figure::kurbo::Size;
    ///
    /// // 20 s at 100 Hz; the window shows the last 5 s.
    /// let y: Vec<f64> = (0..2000).map(|i| (f64::from(i) * 0.01).sin()).collect();
    /// let rec = Recording::new(Clock::Regular { start: 0.0, period: 0.01 }, [&y])?;
    /// let mut fig = Figure::new();
    /// let t = fig.fit_with(scale::linear(), Fit::Window(5.0));
    /// let v = fig.fit(scale::linear());
    /// fig.panel(0, 0).mark(line(rec.series(0).expect("channel 0"), t, v));
    /// # #[cfg(feature = "fonts")] {
    /// let mut cache = rustd3_figure::Cache::default();
    /// let target = Target::Screen { scale: 1.0 };
    /// let r = fig.resolve(Size::new(640.0, 480.0), &mut cache, &Held::default(), target);
    /// let [left, right] = r.scale(t).get_domain();
    /// assert!(right >= 19.99 && (right - left - 5.0).abs() < 1e-9);
    /// # }
    /// # Ok::<(), rustd3_core::series::Error>(())
    /// ```
    ///
    /// A `Fit::Visible` handle on y fits to the data whose x lies in the
    /// resolved domain of its marks' x handles (read from a recording's
    /// index in O(log n)), merged across panels; its domain grows at once
    /// to contain the padded extent and shrinks only by whole eighths of
    /// its last span, remembered in the [`Cache`] by handle index and
    /// scale type, so panning does not make it flicker. Used as x, or over
    /// an x that is itself `Fit::Visible`, it fits all its data.
    pub fn fit_with<S: Fittable>(&mut self, s: S, fit: Fit<S::Span>) -> ScaleRef<S> {
        debug_assert!(
            !(matches!(fit, Fit::Window(_)) && TypeId::of::<S::Span>() == TypeId::of::<()>()),
            "Fit::Window on a categorical scale"
        );
        let visible = matches!(fit, Fit::Visible);
        let window = match fit {
            Fit::Window(span) => window::span(span),
            Fit::All | Fit::Visible => None,
        };
        self.add(Entry {
            scale: Box::new(s),
            fitter: Some(fits::Fitter::of::<S>()),
            visible,
            window,
        })
    }

    /// The panel in grid cell `(row, col)`, made on first use; panels keep
    /// the order of their first use. `panel(r, c)` is
    /// `panel_at(Cell::new(r, c))`.
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel<'a> {
        self.panel_at(Cell::new(row, col))
    }

    /// The panel spanning `at`, made on first use: the same cell and span
    /// give the same panel. Panels whose cells overlap are laid out over
    /// each other. A cell reaching past the grid's 4,096 tracks a side
    /// gives a panel that is never laid out or drawn, which
    /// [`check`](Figure::check) reports.
    pub fn panel_at(&mut self, at: Cell) -> &mut Panel<'a> {
        if !at.within() {
            if !self.outside.contains(&at) {
                self.outside.push(at.clone());
            }
            return self
                .detached
                .get_or_insert_with(|| Box::new(Panel::new(at)));
        }
        let i = match self.panels.iter().position(|p| *p.cell() == at) {
            Some(i) => i,
            None => {
                self.panels.push(Panel::new(at));
                self.panels.len() - 1
            }
        };
        &mut self.panels[i]
    }

    /// The grid's track weights and gaps.
    pub fn layout(&mut self, grid: Grid) -> &mut Self {
        self.grid = grid;
        self
    }

    /// A title centred above the grid.
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
        self
    }

    /// The theme every size, color and margin comes from.
    pub fn theme(&mut self, t: Theme) -> &mut Self {
        self.theme = t;
        self
    }

    /// The locale default tick labels are written in, offset and
    /// multiplier notation's included; [`Locale::NEUTRAL`] (SI style:
    /// U+202F groups of three from five digits) by default. Labels an
    /// axis's `.format` writes are its own.
    ///
    /// ```
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale;
    /// use rustd3_figure::{Figure, axis};
    ///
    /// let mut fig = Figure::new();
    /// let x = fig.scale(scale::linear().domain([0.0, 12_000.0]));
    /// fig.locale(Locale::EN_US).panel(0, 0).axis(axis::bottom(x));
    /// ```
    pub fn locale(&mut self, l: Locale) -> &mut Self {
        self.locale = l;
        self
    }

    /// Places guide `g` in figure slot `slot`, outside the grid: a legend
    /// as a row centred on the plot areas above or below it, a column
    /// beside it; a colorbar along the side, as long as the plot areas
    /// beside it. Guides in one slot stack outward in the order they are
    /// added.
    pub fn legend(&mut self, slot: Slot, g: impl legend::Guide<'a>) -> &mut Self {
        self.legends.push((slot, g.erase().0));
        self
    }

    /// The guides in figure slots, in the order they were added.
    pub(crate) fn legends(&self) -> &[(Slot, legend::Stored<'a>)] {
        &self.legends
    }

    /// The panels, in the order of their first use.
    pub(crate) fn panels(&self) -> &[Panel<'a>] {
        &self.panels
    }

    /// The cells asked for past the grid's tracks, in the order first
    /// asked.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn outside(&self) -> &[Cell] {
        &self.outside
    }

    /// The grid.
    pub(crate) fn grid(&self) -> &Grid {
        &self.grid
    }

    /// The theme the figure is drawn with, as [`theme`](Figure::theme)
    /// set it.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_figure::{Figure, Theme};
    ///
    /// let mut fig = Figure::new();
    /// fig.theme(Theme::default().ink(Rgb::hex(0xff0000)));
    /// assert_eq!(fig.get_theme().get_ink(), Rgb::hex(0xff0000));
    /// ```
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    /// The title.
    pub(crate) fn heading(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    /// Appends `entry` to the table; its handle.
    fn add<S>(&mut self, entry: Entry) -> ScaleRef<S> {
        let index = u32::try_from(self.scales.len())
            .expect("invariant: a figure holds fewer than 2^32 scales");
        self.scales.push(entry);
        ScaleRef::new(self.id, index)
    }
}
