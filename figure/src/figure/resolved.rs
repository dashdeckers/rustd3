//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use kurbo::Point;
use rustd3_core::axis::Orient;
use rustd3_core::locale::Locale;

use crate::figure::fits::Reported;
use crate::hatch::Hatch;
use crate::layout::{self, Laid, Layout, Plan};
use crate::legend::{Block, Swatch};
use crate::panel::Panel;
use crate::scale::{Kind, Scale, ScaleRef};
use crate::target::Target;
use crate::text::Key;
use crate::{brush, figure, thin};

/// A figure resolved for one size and target: every scale with its domain
/// fitted or held, every panel's plot area, and per panel its axes' tick
/// positions and label keys, for drawing.
pub struct Resolved {
    figure: figure::Id,
    scales: Vec<Box<dyn Scale>>,
    locale: Locale,
    target: Target,
    size: kurbo::Size,
    areas: Vec<kurbo::Rect>,
    panels: Vec<Laid>,
    legends: Vec<(Block, Point)>,
    room: [[f64; 2]; 2],
    title: Option<Key>,
    passes: usize,
    held: bool,
    /// Per panel, the table positions of the handles it zooms along x and
    /// along y.
    zooms: Vec<[Vec<usize>; 2]>,
    /// Per panel, each axis's side and handle position, in insertion order.
    axes: Vec<Vec<(Orient, usize)>>,
    /// Per panel, the first row and column of its grid cell.
    cells: Vec<(usize, usize)>,
    /// Per panel, its brush.
    brushes: Vec<Option<brush::Placed>>,
    /// Per table entry, whether it is a `Fit::Window` handle some panel
    /// places across.
    windowed: Vec<bool>,
    /// The palette keys marks draw in, as they reported them.
    keys: Rc<[Reported]>,
    /// Column grids lines thin on, by handle index, as held.
    grids: HashMap<usize, thin::Grid>,
    /// How far plot areas reach past the view, list px, as held.
    overscan: f64,
    /// The (panel, handle position) of the handles of other figures marks
    /// reported.
    foreign: Vec<(usize, usize)>,
}

impl std::fmt::Debug for Resolved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolved").finish_non_exhaustive()
    }
}

impl Resolved {
    /// The resolved scales of figure `figure` for `target` and `size`,
    /// labeled in `locale`, before layout.
    pub(super) fn new(
        figure: figure::Id,
        scales: Vec<Box<dyn Scale>>,
        locale: Locale,
        target: Target,
        size: kurbo::Size,
    ) -> Resolved {
        Resolved {
            figure,
            scales,
            locale,
            target,
            size,
            areas: Vec::new(),
            panels: Vec::new(),
            legends: Vec::new(),
            room: [[0.0; 2]; 2],
            title: None,
            passes: 0,
            held: false,
            zooms: Vec::new(),
            axes: Vec::new(),
            cells: Vec::new(),
            brushes: Vec::new(),
            windowed: Vec::new(),
            keys: Rc::from([]),
            grids: HashMap::new(),
            overscan: 0.0,
            foreign: Vec::new(),
        }
    }

    /// The resolved scale behind a handle, before any panel's range;
    /// `None` for a handle of another figure.
    #[must_use]
    pub fn get_scale<S: Kind>(&self, h: ScaleRef<S>) -> Option<&S> {
        self.scales.get(h.of(self.figure)?)?.as_any().downcast_ref()
    }

    /// The resolved scale behind a handle of an axis, guide or brush the
    /// figure placed. It is never missing, because [`Panel`] and
    /// [`Figure`](crate::Figure) drop placements on handles of other
    /// figures, and a `Resolved` is drawn only for its own figure.
    pub(crate) fn own<S: Kind>(&self, h: ScaleRef<S>) -> &S {
        self.get_scale(h)
            .expect("invariant: placed axes, guides and brushes are on their figure's handles")
    }

    /// The resolved scale at position `index` of the table, its type
    /// erased (a widget keeps gesture state by position and `TypeId`);
    /// `None` past the table's end.
    #[must_use]
    pub fn get_scale_at(&self, index: usize) -> Option<&dyn Scale> {
        self.scales.get(index).map(|scale| &**scale)
    }

    /// Plot-area rects in list px, one per panel in panel order.
    #[must_use]
    pub fn get_plot_areas(&self) -> &[kurbo::Rect] {
        &self.areas
    }

    /// The table positions of the handles panel `panel` zooms along x and
    /// along y ([`Panel::zoom_x`], [`Panel::zoom_y`]); empty for a panel
    /// that zooms none and for one that does not exist.
    #[must_use]
    pub fn get_zooms(&self, panel: usize) -> (&[usize], &[usize]) {
        self.zooms
            .get(panel)
            .map_or((&[], &[]), |[x, y]| (x.as_slice(), y.as_slice()))
    }

    /// The side of axis `axis` (in insertion order) of panel `panel`, and
    /// its handle's table position; `None` when there is no such axis.
    /// With [`get_plan`](Resolved::get_plan)'s offset and thickness it
    /// bounds the strip the axis draws in.
    #[must_use]
    pub fn get_axis(&self, panel: usize, axis: usize) -> Option<(Orient, usize)> {
        self.axes.get(panel)?.get(axis).copied()
    }

    /// (row, col) of a flat panel index, as `hit::Hit::panel` reports it:
    /// the first row and column of the panel's grid cell (panels are
    /// numbered in the order of their first use, not row by row); `None`
    /// for a panel the figure does not have.
    #[must_use]
    pub fn get_panel_cell(&self, panel: usize) -> Option<(usize, usize)> {
        self.cells.get(panel).copied()
    }

    /// The brush placed on panel `panel` ([`Panel::brush`]); `None` for a
    /// panel without one and for one that does not exist.
    #[must_use]
    pub fn get_brush(&self, panel: usize) -> Option<&brush::Placed> {
        self.brushes.get(panel)?.as_ref()
    }

    /// How layout placed axis `axis` (in insertion order) of panel `panel`
    /// (in the order of first use); `None` when there is no such axis.
    #[must_use]
    pub fn get_plan(&self, panel: usize, axis: usize) -> Option<Plan> {
        Some(self.panels.get(panel)?.axes.get(axis)?.plan)
    }

    /// The box of every tick label layout kept, in list px, in panel, axis
    /// and tick order. A label's box is its advance width by its line box,
    /// and for a rotated label it is the bounding rectangle of its turned
    /// ink box. No two labels overlap, with rotated ones compared as turned
    /// boxes.
    #[must_use]
    pub fn label_boxes(&self) -> Vec<kurbo::Rect> {
        self.panels
            .iter()
            .flat_map(|laid| &laid.axes)
            .flat_map(|a| a.boxes.iter().zip(&a.keep))
            .filter_map(|(b, keep)| b.filter(|_| *keep).map(|b| b.aabb()))
            .collect()
    }

    /// Every plot-area edge plus 0 and the size, per axis (x, then y), in
    /// list px, increasing and each once: where
    /// [`list::cells`](crate::list::cells) cuts the canvas.
    #[must_use]
    pub fn edges(&self) -> (Vec<f64>, Vec<f64>) {
        let edges = |ends: [f64; 2], of: fn(&kurbo::Rect) -> [f64; 2]| {
            let mut edges: Vec<f64> = ends
                .into_iter()
                .chain(self.areas.iter().flat_map(of))
                .collect();
            edges.sort_by(f64::total_cmp);
            edges.dedup();
            edges
        };
        (
            edges([0.0, self.size.width], |r| [r.x0, r.x1]),
            edges([0.0, self.size.height], |r| [r.y0, r.y1]),
        )
    }

    /// Layout passes until tick counts settled (1..=4).
    #[must_use]
    pub fn get_passes(&self) -> usize {
        self.passes
    }

    /// The plot areas with the size they were laid out for, for a gesture
    /// to hold ([`Figure::resolve_held`](crate::Figure::resolve_held)).
    #[must_use]
    pub fn get_areas(&self) -> layout::Areas {
        layout::Areas::new(self.areas.clone(), self.size)
    }

    /// Whether this resolve used held areas.
    #[must_use]
    pub fn get_held(&self) -> bool {
        self.held
    }

    /// What the last placement of the tracks needed and had, in px, across
    /// the columns and then down the rows. The need is the reaches and
    /// gaps, and the room had is the length the tracks fill. Held layouts
    /// place nothing, so both are 0.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(
            dead_code,
            reason = "fig.check reads it, and checks with the bundled fonts"
        )
    )]
    pub(crate) fn room(&self) -> [[f64; 2]; 2] {
        self.room
    }

    /// The figure this was resolved from.
    pub(super) fn figure(&self) -> figure::Id {
        self.figure
    }

    /// The size the figure was resolved for.
    pub(crate) fn size(&self) -> kurbo::Size {
        self.size
    }

    /// The locale default tick labels are written in.
    pub(crate) fn locale(&self) -> &Locale {
        &self.locale
    }

    /// The target the figure was resolved for.
    pub(crate) fn target(&self) -> Target {
        self.target
    }

    /// The panels as laid out, in panel order.
    pub(crate) fn panels(&self) -> &[Laid] {
        &self.panels
    }

    /// The legends in figure slots and their top-left corners.
    pub(crate) fn legends(&self) -> &[(Block, Point)] {
        &self.legends
    }

    /// The figure title's key.
    pub(crate) fn title(&self) -> Option<&Key> {
        self.title.as_ref()
    }

    /// Whether the handle at table position `index` is a `Fit::Window`
    /// handle some panel places across: its horizontal axes reserve room
    /// for a tick label at either end.
    pub(crate) fn windowed(&self, index: usize) -> bool {
        self.windowed.get(index).copied().unwrap_or(false)
    }

    /// Records, for each of `panels` (this figure's), its grid cell, the
    /// handles it zooms, its axes' sides and handles and its brush, and
    /// per table entry whether it is a window placed across.
    pub(super) fn bind(&mut self, panels: &[Panel<'_>], windowed: Vec<bool>) {
        self.windowed = windowed;
        self.cells = panels
            .iter()
            .map(|p| (p.cell().rows().start, p.cell().cols().start))
            .collect();
        self.brushes = panels.iter().map(|p| p.brushing().copied()).collect();
        self.zooms = panels.iter().map(Panel::zooms).collect();
        self.axes = panels
            .iter()
            .map(|p| p.axes().iter().map(|a| (a.orient(), a.handle())).collect())
            .collect();
    }

    /// Sets the column grids lines thin on and how far plot areas reach
    /// past the view (list px), from what a widget held.
    pub(super) fn thin_on(&mut self, grids: HashMap<usize, thin::Grid>, overscan: f64) {
        self.grids = grids;
        self.overscan = overscan;
    }

    /// The column grid of the handle at table position `index`, if held.
    pub(crate) fn grid(&self, index: usize) -> Option<thin::Grid> {
        self.grids.get(&index).copied()
    }

    /// How far plot areas reach past the view, list px: 0 unless held.
    pub(crate) fn overscan(&self) -> f64 {
        self.overscan
    }

    /// Sets the palette keys marks reported drawing in.
    pub(super) fn keyed(&mut self, keys: Rc<[Reported]>) {
        self.keys = keys;
    }

    /// Sets the (panel, handle position) of the handles of other figures
    /// the figure's marks reported.
    pub(super) fn foreign(&mut self, foreign: Vec<(usize, usize)>) {
        self.foreign = foreign;
    }

    /// The (panel, handle position) of the handles of other figures the
    /// figure's marks reported, each once.
    #[cfg_attr(
        not(feature = "fonts"),
        expect(dead_code, reason = "fig.check reads it, and needs fonts")
    )]
    pub(crate) fn foreign_marks(&self) -> &[(usize, usize)] {
        &self.foreign
    }

    /// The hatch of the first mark that reported one for palette `palette`
    /// (its table position) and a key for which `is` holds.
    pub(crate) fn hatch(&self, palette: usize, is: impl Fn(&dyn Any) -> bool) -> Option<&Hatch> {
        self.keys
            .iter()
            .filter(|k| k.palette == palette && is(&*k.key))
            .find_map(|k| k.hatch.as_ref())
    }

    /// The opacity a legend square repeats for a color of opacity
    /// `opacity`. It is the most opaque fill among the marks that reported
    /// filling a shape in palette `palette`'s (its table position) color
    /// for a key for which `is` holds, or `opacity` when none did.
    pub(crate) fn opacity(
        &self,
        palette: usize,
        is: impl Fn(&dyn Any) -> bool,
        opacity: f64,
    ) -> f64 {
        self.keys
            .iter()
            .filter(|k| k.palette == palette && k.glyph == Swatch::Square && is(&*k.key))
            .map(|k| k.alpha.of(opacity))
            .reduce(f64::max)
            .unwrap_or(opacity)
    }

    /// The legend swatch of the marks that reported drawing in palette
    /// `palette`'s (its table position) color for a key for which `is`
    /// holds. It is a square when any of them fills a shape, else the first
    /// line's swatch, else a dot. `None` when none did.
    pub(crate) fn glyph(&self, palette: usize, is: impl Fn(&dyn Any) -> bool) -> Option<Swatch> {
        let glyphs = || {
            self.keys
                .iter()
                .filter(|k| k.palette == palette && is(&*k.key))
                .map(|k| k.glyph)
        };
        let square = glyphs().find(|g| *g == Swatch::Square);
        let line = || glyphs().find(|g| matches!(g, Swatch::Line { .. }));
        square.or_else(line).or_else(|| glyphs().next())
    }

    /// Sets the layout, the figure title's key and whether the layout
    /// kept held areas.
    pub(super) fn place(&mut self, layout: Layout, title: Option<Key>, held: bool) {
        self.areas = layout.areas;
        self.panels = layout.panels;
        self.legends = layout.legends;
        self.room = layout.room;
        self.passes = layout.passes;
        self.title = title;
        self.held = held;
    }
}
