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

use std::any::TypeId;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::layout::Grid;
use crate::panel::Panel;
use crate::scale::{Fit, Fittable, Kind, Scale, ScaleRef};
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
    grid: Grid,
    title: Option<Text>,
    theme: Theme,
}

/// One scale of the table, and how to fit it when it is fitted.
struct Entry {
    scale: Box<dyn Scale>,
    fitter: Option<fits::Fitter>,
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
            grid: Grid::default(),
            title: None,
            theme: Theme::default(),
        }
    }

    /// A scale whose domain is the one it was built with.
    pub fn scale<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(Box::new(s), None)
    }

    /// A scale fitted to the data of every mark that uses it:
    /// `fit_with(s, Fit::All)`.
    pub fn fit<S: Fittable>(&mut self, s: S) -> ScaleRef<S> {
        self.fit_with(s, Fit::All)
    }

    /// A scale whose domain follows its marks' data as `fit` says.
    /// `Fit::Window` and `Fit::Visible` resolve as `Fit::All` for now.
    /// `Fit::Window(())` on a categorical scale (span `()`) means nothing,
    /// and debug builds panic on it.
    pub fn fit_with<S: Fittable>(&mut self, s: S, fit: Fit<S::Span>) -> ScaleRef<S> {
        debug_assert!(
            !(matches!(fit, Fit::Window(_)) && TypeId::of::<S::Span>() == TypeId::of::<()>()),
            "Fit::Window on a categorical scale"
        );
        self.add(Box::new(s), Some(fits::Fitter::of::<S>()))
    }

    /// The panel in grid cell `(row, col)`, made on first use; panels keep
    /// the order of their first use.
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel<'a> {
        let i = match self.panels.iter().position(|p| p.cell() == (row, col)) {
            Some(i) => i,
            None => {
                self.panels.push(Panel::new((row, col)));
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

    /// The panels, in the order of their first use.
    pub(crate) fn panels(&self) -> &[Panel<'a>] {
        &self.panels
    }

    /// The grid.
    pub(crate) fn grid(&self) -> &Grid {
        &self.grid
    }

    /// The theme.
    pub(crate) fn style(&self) -> &Theme {
        &self.theme
    }

    /// The title.
    pub(crate) fn heading(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    /// Appends `scale` to the table.
    fn add<S>(&mut self, scale: Box<dyn Scale>, fitter: Option<fits::Fitter>) -> ScaleRef<S> {
        let index = u32::try_from(self.scales.len())
            .expect("invariant: a figure holds fewer than 2^32 scales");
        self.scales.push(Entry { scale, fitter });
        ScaleRef::new(self.id, index)
    }
}
