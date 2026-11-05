//! [`BrushY`], d3's `brushY()`.

use crate::interact::brush::engine::{Dim, Engine};
use crate::interact::{BrushEvent, Pointer};

/// d3.brushY(): the same as [`BrushX`](crate::interact::BrushX) over y,
/// with handles `n` and `s`; the selection spans the extent's x.
///
/// A new selection, clamped to the extent, then a flip (d3's trace T1
/// along y):
///
/// ```
/// use rustd3_core::interact::{BrushEvent, BrushY, Button, Modifiers, Pointer};
///
/// let mut brush = BrushY::new([[0.0, 0.0], [100.0, 50.0]]);
/// let press = Pointer::Pressed { x: 10.0, y: 20.0, button: Button::Left, modifiers: Modifiers::NONE };
/// assert_eq!(brush.handle(press), Some(BrushEvent::Start(Some([20.0, 20.0]))));
/// let to = |y| Pointer::Moved { x: 10.0, y };
/// // dy = 40 stops at the extent's bottom, 30 below the press.
/// assert_eq!(brush.handle(to(60.0)), Some(BrushEvent::Brush(Some([20.0, 50.0]))));
/// assert_eq!(brush.handle(to(10.0)), Some(BrushEvent::Brush(Some([10.0, 20.0]))));
/// let release = Pointer::Released { button: Button::Left };
/// assert_eq!(brush.handle(release), Some(BrushEvent::End(Some([10.0, 20.0]))));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BrushY {
    engine: Engine,
}

impl BrushY {
    /// d3's `brushY().extent(extent)`: the area a press can start in,
    /// [[x0, y0], [x1, y1]], without a selection.
    #[must_use]
    pub fn new(extent: [[f64; 2]; 2]) -> Self {
        BrushY {
            engine: Engine::new(Dim::Y, extent),
        }
    }

    /// d3's `brush.handleSize(px)`: the handles' height, 6 by default.
    #[must_use]
    pub fn handle_size(self, px: f64) -> Self {
        BrushY {
            engine: self.engine.handle_size(px),
        }
    }

    /// Feeds one event; what d3 emits for it. A move emits only when it
    /// changes the selection.
    pub fn handle(&mut self, input: Pointer) -> Option<BrushEvent> {
        self.engine.handle(input)
    }

    /// The selected y interval.
    #[must_use]
    pub fn get_selection(&self) -> Option<[f64; 2]> {
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
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.engine.is_active()
    }
}
