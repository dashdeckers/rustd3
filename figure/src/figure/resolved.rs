//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use crate::figure;
use crate::layout::{Laid, Layout};
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
    passes: usize,
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
            passes: 0,
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

    /// Layout passes until tick counts settled (1..=4).
    pub fn passes(&self) -> usize {
        self.passes
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

    /// Sets the layout and the figure title's key.
    pub(super) fn place(&mut self, layout: Layout, title: Option<Key>) {
        self.areas = layout.areas;
        self.panels = layout.panels;
        self.passes = layout.passes;
        self.title = title;
    }
}
