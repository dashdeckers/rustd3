//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use crate::figure;
use crate::layout::{self, Laid, Layout, Plan};
use crate::scale::{Kind, Scale, ScaleRef};
use crate::target::Target;
use crate::text::Key;

/// A figure resolved for one size and target: every scale with its domain
/// fitted or held, every panel's plot area, and per panel its axes' tick
/// positions and label keys, for drawing.
pub struct Resolved {
    figure: figure::Id,
    scales: Vec<Box<dyn Scale>>,
    target: Target,
    size: kurbo::Size,
    areas: Vec<kurbo::Rect>,
    panels: Vec<Laid>,
    title: Option<Key>,
    passes: usize,
    held: bool,
}

impl Resolved {
    /// The resolved scales of figure `figure` for `target` and `size`,
    /// before layout.
    pub(super) fn new(
        figure: figure::Id,
        scales: Vec<Box<dyn Scale>>,
        target: Target,
        size: kurbo::Size,
    ) -> Resolved {
        Resolved {
            figure,
            scales,
            target,
            size,
            areas: Vec::new(),
            panels: Vec::new(),
            title: None,
            passes: 0,
            held: false,
        }
    }

    /// The resolved scale behind a handle, before any panel's range.
    pub fn scale<S: Kind>(&self, h: ScaleRef<S>) -> &S {
        self.scales[h.of(self.figure)]
            .as_any()
            .downcast_ref()
            .expect("invariant: a handle's scale has the handle's type")
    }

    /// Plot-area rects in list px, one per panel in panel order.
    pub fn plot_areas(&self) -> &[kurbo::Rect] {
        &self.areas
    }

    /// How layout placed axis `axis` (in insertion order) of panel `panel`
    /// (in the order of first use); `None` when there is no such axis.
    pub fn plan(&self, panel: usize, axis: usize) -> Option<Plan> {
        Some(self.panels.get(panel)?.axes.get(axis)?.plan)
    }

    /// The box of every tick label layout kept, in list px, in panel, axis
    /// and tick order: the label's advance width by its ink height. No two
    /// overlap.
    pub fn label_boxes(&self) -> Vec<kurbo::Rect> {
        self.panels
            .iter()
            .flat_map(|laid| &laid.axes)
            .flat_map(|a| a.boxes.iter().zip(&a.keep))
            .filter_map(|(b, keep)| b.filter(|_| *keep))
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

    /// The size the figure was resolved for.
    pub(crate) fn size(&self) -> kurbo::Size {
        self.size
    }

    /// The target the figure was resolved for.
    pub(crate) fn target(&self) -> Target {
        self.target
    }

    /// The panels as laid out, in panel order.
    pub(super) fn panels(&self) -> &[Laid] {
        &self.panels
    }

    /// The figure title's key.
    pub(super) fn title(&self) -> Option<&Key> {
        self.title.as_ref()
    }

    /// Sets the layout, the figure title's key and whether the layout
    /// kept held areas.
    pub(super) fn place(&mut self, layout: Layout, title: Option<Key>, held: bool) {
        self.areas = layout.areas;
        self.panels = layout.panels;
        self.passes = layout.passes;
        self.title = title;
        self.held = held;
    }
}
