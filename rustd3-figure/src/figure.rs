//! [`Figure`]: scales behind typed handles, and panels in grid cells that
//! use them. Resolving a figure fits its domains and lays out its grid.

mod decorate;
mod draw;
mod facet;
mod fits;
mod held;
mod part;
mod resolve;
mod resolved;
mod save;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) use facet::Remap;
pub use fits::Fits;
pub use held::Held;
pub use resolved::Resolved;
use rustd3_core::locale::Locale;
pub use save::Error;

use crate::Target;
use crate::check::Issue;
use crate::layout::{Cell, Grid};
use crate::legend::{self, Slot};
use crate::panel::Panel;
use crate::scale::{Fit, Fittable, Kind, Scale, ScaleRef, window};
use crate::text::Text;
use crate::theme::Theme;

/// The next figure's id.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// A figure's identity, which its handles carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Id(u64);

/// A figure's identity with those of the figures it took in
/// ([`Figure::facet`]) and where their scales went: what finds a handle's
/// table position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ids {
    own: Id,
    taken: Arc<[(Id, Vec<usize>)]>,
}

impl Ids {
    /// The table position of entry `index` of figure `id`'s table.
    pub(crate) fn position(&self, id: Id, index: usize) -> Option<usize> {
        if id == self.own {
            return Some(index);
        }
        let (_, moved) = self.taken.iter().find(|(taken, _)| *taken == id)?;
        moved.get(index).copied()
    }
}

/// Whether the figures [`Figure::facet`] takes in share their scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Share {
    /// A scale an axis, zoom or brush of the facet is on, at a table
    /// position where a facet taken in before with `Share::Scales` had
    /// such a scale of the same type, becomes that scale: one domain fitted
    /// to every facet's marks, zoomed together in the iced `Plot` widget.
    /// Palettes stay each facet's own.
    Scales,
    /// Every scale stays the facet's own.
    None,
}

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
/// let mut cache = rustd3_figure::cache::Cache::default();
/// let r = fig.resolve(Size::new(640.0, 480.0), &mut cache, &Held::default(), Target::Vector);
/// assert_eq!(r.get_scale(x).map(|s| s.get_domain()), Some([0.0, 10.0]));
/// // No mark gives y data: core's empty domain.
/// assert_eq!(r.get_scale(y).map(|s| s.get_domain()), Some([0.0, 1.0]));
/// assert_eq!(r.get_plot_areas().len(), 1);
/// # }
/// ```
pub struct Figure<'a> {
    id: Id,
    scales: Vec<Entry>,
    panels: Vec<Panel<'a>>,
    /// The cells asked for past the grid's tracks.
    outside: Vec<Cell>,
    /// The panel that stands in for cells past the grid's tracks, never
    /// laid out or drawn.
    detached: Option<Box<Panel<'a>>>,
    /// The positions of the handles of other figures given to figure
    /// slots.
    foreign: Vec<usize>,
    /// The positions of handles fitted with `Fit::Window` on a categorical
    /// scale.
    categorical: Vec<usize>,
    grid: Grid,
    title: Option<Text>,
    theme: Theme,
    locale: Locale,
    legends: Vec<(Slot, legend::Stored<'a>)>,
    /// What its builder found wrong, which `check` reports last.
    noted: Vec<Issue>,
    /// The room past labeled vertical axes, in widths of "0", in place of
    /// the target's.
    axis_room: Option<f64>,
    /// The figures taken in by `facet`, each with the table positions its
    /// scales went to.
    taken: Vec<(Id, Vec<usize>)>,
    /// Per table position in the facets that share scales, the position
    /// here of the first such facet's scale an axis, zoom or brush is on.
    shared: HashMap<usize, usize>,
    /// The slots and facet table positions of the figure-slot legends the
    /// facets that share scales brought, each kept once.
    slotted: Vec<(Slot, usize)>,
}

impl<'a> std::fmt::Debug for Figure<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Figure").finish_non_exhaustive()
    }
}

/// One scale of the table, with how it is fitted.
struct Entry {
    scale: Box<dyn Scale>,
    /// How to fit the scale, when it is fitted.
    fitter: Option<fits::Fitter>,
    /// Whether it fits to what its panels' x shows (`Fit::Visible`).
    visible: bool,
    /// The span of x it shows when it is a window (`Fit::Window`), its
    /// `Kind::Span` boxed.
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
            foreign: Vec::new(),
            categorical: Vec::new(),
            grid: Grid::default(),
            title: None,
            theme: Theme::default(),
            locale: Locale::default(),
            legends: Vec::new(),
            noted: Vec::new(),
            axis_room: None,
            taken: Vec::new(),
            shared: HashMap::new(),
            slotted: Vec::new(),
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
    /// area (see [`Fit::Window`]). Resolving lays the figure out with the
    /// window, measures the plot areas, then fits and lays out again until
    /// the column count it fitted with is the one it laid out. The last
    /// count is remembered in the [`Cache`](crate::cache::Cache), so a
    /// layout that did not move takes one pass. `Fit::Window(())` on a
    /// categorical scale means nothing: the handle fits as `Fit::All`, and
    /// [`check`](Figure::check) reports it.
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
    /// let mut cache = rustd3_figure::cache::Cache::default();
    /// let target = Target::Screen { scale: 1.0 };
    /// let r = fig.resolve(Size::new(640.0, 480.0), &mut cache, &Held::default(), target);
    /// let [left, right] = r.get_scale(t).expect("a handle of this figure").get_domain();
    /// assert!(right >= 19.99 && (right - left - 5.0).abs() < 1e-9);
    /// # }
    /// # Ok::<(), rustd3_core::series::Error>(())
    /// ```
    ///
    /// A `Fit::Visible` handle on y fits to the data whose x lies in the
    /// resolved domain of its marks' x handles, merged across panels. A
    /// recording's index finds that data in O(log n). The domain grows at
    /// once to contain the padded extent, and shrinks only by whole eighths
    /// of its last span, so panning does not make it flicker. That span is
    /// remembered in the [`Cache`](crate::cache::Cache) by handle index and
    /// scale type. Used as x, or over an x that is itself `Fit::Visible`,
    /// the handle fits all its data.
    pub fn fit_with<S: Fittable>(&mut self, s: S, fit: Fit<S::Span>) -> ScaleRef<S> {
        let categorical =
            matches!(fit, Fit::Window(_)) && TypeId::of::<S::Span>() == TypeId::of::<()>();
        let visible = matches!(fit, Fit::Visible);
        let window = match fit {
            Fit::Window(span) if !categorical => window::span(span),
            Fit::Window(_) | Fit::All | Fit::Visible => None,
        };
        let h = self.add(Entry {
            scale: Box::new(s),
            fitter: Some(fits::Fitter::of::<S>()),
            visible,
            window,
        });
        if categorical {
            self.categorical.push(h.get_index());
        }
        h
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
                .get_or_insert_with(|| Box::new(Panel::new(self.id, at)));
        }
        let i = match self.panels.iter().position(|p| *p.cell() == at) {
            Some(i) => i,
            None => {
                self.panels.push(Panel::new(self.id, at));
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

    /// A title centered above the grid.
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
        self
    }

    /// The theme every size, color and margin comes from.
    pub fn theme(&mut self, t: Theme) -> &mut Self {
        self.theme = t;
        self
    }

    /// The locale default tick labels are written in, including offset and
    /// multiplier notation. The default is [`Locale::NEUTRAL`], SI style,
    /// which groups numbers of five or more digits in threes with U+202F.
    /// Labels written by an axis's `.format` are its own.
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

    /// The room every labeled vertical axis keeps past its tick labels, in
    /// widths of "0" at tick-label size, so labels that grow during a zoom
    /// or pan do not move the plot area. By default it is 2 for the screen
    /// (`Target::Screen`, the iced widget) and 0 for files and vector
    /// output; setting it makes every target lay the figure out alike, so
    /// a PNG and the live plot match. A room that is not finite and 0 or
    /// more leaves the figure as it was.
    ///
    /// ```
    /// use rustd3_figure::Figure;
    ///
    /// let mut fig = Figure::new();
    /// fig.axis_room(2.0); // the PNG keeps the room the widget keeps
    /// ```
    pub fn axis_room(&mut self, zeros: f64) -> &mut Self {
        if zeros.is_finite() && zeros >= 0.0 {
            self.axis_room = Some(zeros);
        }
        self
    }

    /// The room past labeled vertical axes drawn for `target`, in widths
    /// of "0".
    pub(crate) fn room(&self, target: Target) -> f64 {
        let default = if target.interactive() { 2.0 } else { 0.0 };
        self.axis_room.unwrap_or(default)
    }

    /// Places guide `g` in figure slot `slot`, outside the grid. A legend
    /// above or below the grid is a row centered on the plot areas, and one
    /// beside it is a column. A colorbar runs along its side, as long as
    /// the plot areas beside it. Guides in one slot stack outward in the
    /// order they are added. A guide of a handle of another figure is not
    /// placed, and [`check`](Figure::check) reports the handle.
    pub fn legend(&mut self, slot: Slot, g: impl legend::Guide<'a>) -> &mut Self {
        let stored = g.erase().0;
        let [h] = stored.handles();
        if h.of(&self.ids()).is_some() {
            self.legends.push((slot, stored));
        } else if !self.foreign.contains(&h.get_index()) {
            self.foreign.push(h.get_index());
        }
        self
    }

    /// The positions of the handles of other figures given to figure
    /// slots, each once.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn foreign(&self) -> &[usize] {
        &self.foreign
    }

    /// The positions of the handles fitted with `Fit::Window` on a
    /// categorical scale, which fit as `Fit::All`.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn categorical(&self) -> &[usize] {
        &self.categorical
    }

    /// Whether handle `h` is of this figure: a handle of another figure
    /// finds no scale here, and what uses it draws nothing. The handles of
    /// a figure taken in by [`facet`](Figure::facet) are this figure's.
    #[must_use]
    pub fn owns<S>(&self, h: ScaleRef<S>) -> bool {
        h.of(&self.ids()).is_some()
    }

    /// This figure's own handle to the scale `h` finds here: `h` itself,
    /// or for a handle of a figure taken in by [`facet`](Figure::facet)
    /// the handle of the scale it became. A widget's listeners and
    /// [`Held`] name scales by table position, so they take this handle.
    /// `None` for a handle of another figure.
    #[must_use]
    pub fn handle<S>(&self, h: ScaleRef<S>) -> Option<ScaleRef<S>> {
        let index = h.of(&self.ids())?;
        Some(h.moved(self.id, index))
    }

    /// The figure's identity with those of the figures it took in.
    pub(crate) fn ids(&self) -> Ids {
        Ids {
            own: self.id,
            taken: self.taken.as_slice().into(),
        }
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

    /// Records `issue`, found by whatever built the figure, for
    /// [`check`](Figure::check) to report.
    #[cfg_attr(
        not(feature = "helpers"),
        expect(dead_code, reason = "the chart helpers note what they find")
    )]
    pub(crate) fn note(&mut self, issue: Issue) {
        self.noted.push(issue);
    }

    /// The issues noted while the figure was built, in order.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn noted(&self) -> &[Issue] {
        &self.noted
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
    #[must_use]
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    /// The locale tick labels are written in, as [`locale`](Figure::locale)
    /// set it.
    #[must_use]
    pub fn get_locale(&self) -> &Locale {
        &self.locale
    }

    /// The title above the grid, as [`title`](Figure::title) set it.
    #[must_use]
    pub fn get_title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    /// Takes the title out of the figure, which then has none: a plot
    /// drawn once per facet lifts its title to the grid, and
    /// [`facet`](Figure::facet) then puts no title on the facet's panel.
    ///
    /// ```
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::text::Text;
    ///
    /// let mut fig = Figure::new();
    /// fig.title("spectra");
    /// assert_eq!(fig.take_title(), Some(Text::from("spectra")));
    /// assert_eq!(fig.get_title(), None);
    /// ```
    pub fn take_title(&mut self) -> Option<Text> {
        self.title.take()
    }

    /// How many grid rows and columns its panels reach: one past the last
    /// row and column a panel's cell spans, `(0, 0)` without panels. A
    /// caller placing figures with [`facet`](Figure::facet) steps by it so
    /// that facets of several panels do not overlap.
    ///
    /// ```
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::layout::Cell;
    ///
    /// let mut fig = Figure::new();
    /// fig.panel_at(Cell::new(0, 0).span(1, 2));
    /// fig.panel(2, 1);
    /// assert_eq!(fig.get_extent(), (3, 2));
    /// ```
    #[must_use]
    pub fn get_extent(&self) -> (usize, usize) {
        self.panels.iter().fold((0, 0), |(rows, cols), p| {
            let cell = p.cell();
            (rows.max(cell.rows().end), cols.max(cell.cols().end))
        })
    }

    /// Appends `entry` to the table and returns its handle.
    fn add<S>(&mut self, entry: Entry) -> ScaleRef<S> {
        let index = u32::try_from(self.scales.len())
            .expect("invariant: a figure holds fewer than 2^32 scales");
        self.scales.push(entry);
        ScaleRef::new(self.id, index)
    }
}
