//! [`BrushX`], d3's `brushX()`.

use crate::interact::BrushEvent;
use crate::interact::Pointer;
use crate::interact::brush::engine::{Dim, Engine};

/// d3.brushX(): selects an x interval inside `extent` = [[x0, y0], [x1,
/// y1]]. A press on the overlay starts a selection, on the selection drags
/// it, on a handle resizes it, and an edge dragged past the other flips
/// the handle (d3-brush 3.0.0, `brush.js`).
///
/// The caller feeds it [`Pointer`] input and stores nothing else; a
/// selection that ends with zero width ends as `None`. d3's filter
/// applies: presses of other buttons or with ctrl held do nothing. Other
/// modifiers are accepted and ignored (no keyboard modes in v0.1), and a
/// gesture continues after [`Pointer::Left`] until the left button is
/// released.
///
/// A new selection, then a flip (d3's trace T1):
///
/// ```
/// use rustd3_core::interact::{BrushEvent, BrushX, Button, Modifiers, Pointer};
///
/// let mut brush = BrushX::new([[0.0, 0.0], [100.0, 50.0]]);
/// let press = Pointer::Pressed { x: 20.0, y: 10.0, button: Button::Left, modifiers: Modifiers::NONE };
/// assert_eq!(brush.handle(press), Some(BrushEvent::Start(Some([20.0, 20.0]))));
/// let to = |x| Pointer::Moved { x, y: 10.0 };
/// assert_eq!(brush.handle(to(60.0)), Some(BrushEvent::Brush(Some([20.0, 60.0]))));
/// // The e edge passes w: the handle flips.
/// assert_eq!(brush.handle(to(10.0)), Some(BrushEvent::Brush(Some([10.0, 20.0]))));
/// let release = Pointer::Released { button: Button::Left };
/// assert_eq!(brush.handle(release), Some(BrushEvent::End(Some([10.0, 20.0]))));
/// assert_eq!(brush.selection(), Some([10.0, 20.0]));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BrushX {
    engine: Engine,
}

impl BrushX {
    /// d3's `brushX().extent(extent)`: the area a press can start in,
    /// [[x0, y0], [x1, y1]], without a selection.
    pub fn new(extent: [[f64; 2]; 2]) -> Self {
        BrushX {
            engine: Engine::new(Dim::X, extent),
        }
    }

    /// d3's `brush.handleSize(px)`: the handles' width, 6 by default.
    pub fn handle_size(self, px: f64) -> Self {
        BrushX {
            engine: self.engine.handle_size(px),
        }
    }

    /// Feeds one event; what d3 emits for it. A move emits only when it
    /// changes the selection.
    pub fn handle(&mut self, input: Pointer) -> Option<BrushEvent> {
        self.engine.handle(input)
    }

    /// The selected x interval.
    pub fn selection(&self) -> Option<[f64; 2]> {
        self.engine.selection()
    }

    /// d3's `brush.move` without events (the caller already knows). During
    /// a gesture it replaces the selection the gesture compares against,
    /// as d3's does (`brush.js:479, 505`).
    pub fn set_selection(&mut self, selection: Option<[f64; 2]>) {
        self.engine.set_selection(selection);
    }

    /// A gesture is in progress: an app syncs the brush from its own state
    /// only between gestures.
    pub fn is_active(&self) -> bool {
        self.engine.is_active()
    }
}
