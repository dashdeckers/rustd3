//! [`Machine`]: core's brush along the placed brush's axis.

use rustd3_core::interact::{BrushEvent, BrushX, BrushY, Pointer};
use rustd3_figure::brush::Dim;
use rustd3_figure::kurbo::Rect;

/// Core's `BrushX` or `BrushY`, as the placed brush's axis asks.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Machine {
    X(BrushX),
    Y(BrushY),
}

impl Machine {
    /// A brush along `dim` over `extent` (list px), without a selection.
    pub(crate) fn new(dim: Dim, extent: Rect) -> Machine {
        let extent = [[extent.x0, extent.y0], [extent.x1, extent.y1]];
        match dim {
            Dim::X => Machine::X(BrushX::new(extent)),
            Dim::Y => Machine::Y(BrushY::new(extent)),
        }
    }

    /// Feeds one event; what d3 emits for it.
    pub(crate) fn handle(&mut self, input: Pointer) -> Option<BrushEvent> {
        match self {
            Machine::X(brush) => brush.handle(input),
            Machine::Y(brush) => brush.handle(input),
        }
    }

    /// The selected interval along the axis, list px.
    pub(crate) fn selection(&self) -> Option<[f64; 2]> {
        match self {
            Machine::X(brush) => brush.selection(),
            Machine::Y(brush) => brush.selection(),
        }
    }

    /// d3's `brush.move` without events.
    pub(crate) fn set_selection(&mut self, selection: Option<[f64; 2]>) {
        match self {
            Machine::X(brush) => brush.set_selection(selection),
            Machine::Y(brush) => brush.set_selection(selection),
        }
    }

    /// Whether a gesture is in progress.
    pub(crate) fn is_active(&self) -> bool {
        match self {
            Machine::X(brush) => brush.is_active(),
            Machine::Y(brush) => brush.is_active(),
        }
    }
}
