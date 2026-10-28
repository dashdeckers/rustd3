//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use kurbo::Point;
use rustd3_core::axis::Orient;
use rustd3_core::locale::Locale;

use crate::figure;
use crate::layout::{self, Laid, Layout, Plan};
use crate::legend::Block;
use crate::panel::Panel;
use crate::scale::{Kind, Scale, ScaleRef};
use crate::target::Target;
use crate::text::Key;

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
    /// Per table entry, whether it is a `Fit::Window` handle some panel
    /// places across.
    windowed: Vec<bool>,
}

impl Resolved {
    /// The resolved scales of figure `figure` for `target` and `size`,
    /// labelled in `locale`, before layout.
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
            windowed: Vec::new(),
        }
    }

    /// The resolved scale behind a handle, before any panel's range.
    pub fn scale<S: Kind>(&self, h: ScaleRef<S>) -> &S {
        self.scales[h.of(self.figure)]
            .as_any()
            .downcast_ref()
            .expect("invariant: a handle's scale has the handle's type")
    }

    /// The resolved scale at position `index` of the table, its type
    /// erased (a widget keeps gesture state by position and `TypeId`);
    /// `None` past the table's end.
    pub fn scale_at(&self, index: usize) -> Option<&dyn Scale> {
        self.scales.get(index).map(|scale| &**scale)
    }

    /// Plot-area rects in list px, one per panel in panel order.
    pub fn plot_areas(&self) -> &[kurbo::Rect] {
        &self.areas
    }

    /// The table positions of the handles panel `panel` zooms along x and
    /// along y ([`Panel::zoom_x`], [`Panel::zoom_y`]); empty for a panel
    /// that zooms none and for one that does not exist.
    pub fn zooms(&self, panel: usize) -> (&[usize], &[usize]) {
        self.zooms
            .get(panel)
            .map_or((&[], &[]), |[x, y]| (x.as_slice(), y.as_slice()))
    }

    /// The side of axis `axis` (in insertion order) of panel `panel`, and
    /// its handle's table position; `None` when there is no such axis.
    /// With [`plan`](Resolved::plan)'s offset and thickness it bounds the
    /// strip the axis draws in.
    pub fn axis(&self, panel: usize, axis: usize) -> Option<(Orient, usize)> {
        self.axes.get(panel)?.get(axis).copied()
    }

    /// How layout placed axis `axis` (in insertion order) of panel `panel`
    /// (in the order of first use); `None` when there is no such axis.
    pub fn plan(&self, panel: usize, axis: usize) -> Option<Plan> {
        Some(self.panels.get(panel)?.axes.get(axis)?.plan)
    }

    /// The box of every tick label layout kept, in list px, in panel, axis
    /// and tick order: the label's advance width by its line box (its ink
    /// box, turned, for a rotated label, whose box this is the bounding
    /// rectangle of). No two labels overlap: rotated ones as turned boxes.
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
    pub fn passes(&self) -> usize {
        self.passes
    }

    /// The plot areas with the size they were laid out for, for a gesture
    /// to hold ([`Figure::resolve_held`](crate::Figure::resolve_held)).
    pub fn areas(&self) -> layout::Areas {
        layout::Areas::new(self.areas.clone(), self.size)
    }

    /// True when this resolve used held areas.
    pub fn held(&self) -> bool {
        self.held
    }

    /// What the last placement of the tracks needed and had, px: reaches
    /// and gaps, and the length the tracks fill, across the columns, then
    /// down the rows. Held layouts place nothing: 0 and 0.
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

    /// Records, for each of `panels` (this figure's), the handles it zooms
    /// and its axes' sides and handles, and per table entry whether it is
    /// a window placed across.
    pub(super) fn bind(&mut self, panels: &[Panel<'_>], windowed: Vec<bool>) {
        self.windowed = windowed;
        self.zooms = panels.iter().map(|p| p.zooms(self.figure)).collect();
        self.axes = panels
            .iter()
            .map(|p| p.axes().iter().map(|a| (a.orient(), a.handle())).collect())
            .collect();
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
