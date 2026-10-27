//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use crate::axis::Labels;
use crate::figure;
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
    areas: Vec<kurbo::Rect>,
    panels: Vec<Laid>,
    title: Option<Key>,
}

/// One panel as laid out, for drawing.
pub(super) struct Laid {
    /// Each axis's labels, with its ticks' positions along the plot area.
    pub(super) axes: Vec<(Labels, Vec<Option<f64>>)>,
    /// The title's key.
    pub(super) title: Option<Key>,
    /// How far the decorations on top reach, the title aside.
    pub(super) top: f64,
}

impl Resolved {
    /// The resolved scales of figure `figure`, before layout.
    pub(super) fn new(figure: figure::Id, scales: Vec<Box<dyn Scale>>, target: Target) -> Resolved {
        Resolved {
            figure,
            scales,
            target,
            areas: Vec::new(),
            panels: Vec::new(),
            title: None,
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

    /// Sets the layout: the plot areas, the panels and the figure title.
    pub(super) fn place(&mut self, areas: Vec<kurbo::Rect>, panels: Vec<Laid>, title: Option<Key>) {
        self.areas = areas;
        self.panels = panels;
        self.title = title;
    }
}
