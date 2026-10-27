//! [`Resolved`]: a figure's fitted scales and its laid-out plot areas.

use crate::figure;
use crate::scale::{Kind, Scale, ScaleRef};
use crate::target::Target;

/// A figure resolved for one size and target: every scale with its domain
/// fitted or held, and every panel's plot area.
pub struct Resolved {
    figure: figure::Id,
    scales: Vec<Box<dyn Scale>>,
    areas: Vec<kurbo::Rect>,
    target: Target,
}

impl Resolved {
    /// The resolved scales of figure `figure`, before layout.
    pub(super) fn new(figure: figure::Id, scales: Vec<Box<dyn Scale>>, target: Target) -> Resolved {
        Resolved {
            figure,
            scales,
            areas: Vec::new(),
            target,
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

    /// Sets the plot areas.
    pub(super) fn place(&mut self, areas: Vec<kurbo::Rect>) {
        self.areas = areas;
    }
}
