//! Zoom and brush as renderer-agnostic state machines: pointer input in, new state out.

use crate::scale::Linear;
use crate::time::Time;

/// Pointer input in the chart's own pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pointer {
    Pressed { x: f64, y: f64 },
    Moved { x: f64, y: f64 },
    Released,
    /// `lines` is positive when scrolling down (zooming out), as browsers report `deltaY`.
    Wheel { x: f64, y: f64, lines: f64 },
    DoubleClicked { x: f64, y: f64 },
}

/// d3-zoom's `ZoomTransform`: screen = data * k + (x, y).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub k: f64,
    pub x: f64,
    pub y: f64,
}

impl Transform {
    pub const IDENTITY: Self = Self { k: 1.0, x: 0.0, y: 0.0 };

    pub fn apply(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (x * self.k + self.x, y * self.k + self.y)
    }

    pub fn invert(&self, (x, y): (f64, f64)) -> (f64, f64) {
        ((x - self.x) / self.k, (y - self.y) / self.k)
    }

    /// Scales by `factor` while keeping the screen point `anchor` fixed.
    pub fn scale_around(&self, factor: f64, anchor: (f64, f64)) -> Self {
        let (px, py) = self.invert(anchor);
        let k = self.k * factor;
        Self { k, x: anchor.0 - px * k, y: anchor.1 - py * k }
    }

}

/// A continuous scale a zoom can rescale: d3's `transform.rescaleX(x)` for any such scale.
pub trait Rescale: Sized {
    /// The scale whose domain is what the transform now shows.
    fn rescale_x(&self, t: &Transform) -> Self;
}

impl Rescale for Linear<f64> {
    fn rescale_x(&self, t: &Transform) -> Self {
        let visible = self.get_range().map(|r| self.invert((r - t.x) / t.k));
        self.clone().domain(visible)
    }
}

impl Rescale for Time {
    fn rescale_x(&self, t: &Transform) -> Self {
        let visible = self.get_range().map(|r| self.invert((r - t.x) / t.k));
        self.clone().domain(visible)
    }
}

/// `d3.zoom()`: wheel, drag-to-pan and double-click, limited to a scale extent.
#[derive(Debug, Clone)]
pub struct Zoom {
    scale_extent: [f64; 2],
    grab: Option<(f64, f64)>,
}

impl Default for Zoom {
    fn default() -> Self {
        Self { scale_extent: [0.0, f64::INFINITY], grab: None }
    }
}

impl Zoom {
    pub fn scale_extent(self, scale_extent: [f64; 2]) -> Self {
        Self { scale_extent, ..self }
    }

    /// Feeds one pointer event; returns the new transform if it changed.
    pub fn handle(&mut self, input: Pointer, t: Transform) -> Option<Transform> {
        let clamp = |t: Transform, anchor| {
            let k = t.k.clamp(self.scale_extent[0], self.scale_extent[1]);
            Transform::scale_around(&t, k / t.k, anchor)
        };
        match input {
            Pointer::Wheel { x, y, lines } => {
                let factor = 2f64.powf(-lines * 0.05);
                Some(clamp(t.scale_around(factor, (x, y)), (x, y)))
            }
            Pointer::DoubleClicked { x, y } => Some(clamp(t.scale_around(2.0, (x, y)), (x, y))),
            Pointer::Pressed { x, y } => {
                self.grab = Some((x, y));
                None
            }
            Pointer::Moved { x, y } => {
                let (gx, gy) = self.grab?;
                self.grab = Some((x, y));
                Some(Transform { x: t.x + x - gx, y: t.y + y - gy, ..t })
            }
            Pointer::Released => {
                self.grab = None;
                None
            }
        }
    }
}

/// What a brush reports, like d3's `start`, `brush` and `end` events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushEvent {
    Start(Option<[f64; 2]>),
    Brush(Option<[f64; 2]>),
    End(Option<[f64; 2]>),
}

#[derive(Debug, Clone, Copy)]
enum Drag {
    Creating { origin: f64 },
    Moving { grab: f64, from: [f64; 2] },
}

/// `d3.brushX()`: a one-dimensional selection in pixels within `extent`.
#[derive(Debug, Clone)]
pub struct BrushX {
    extent: [f64; 2],
    selection: Option<[f64; 2]>,
    drag: Option<Drag>,
}

impl BrushX {
    pub fn new(extent: [f64; 2]) -> Self {
        Self { extent, selection: None, drag: None }
    }

    pub fn selection(&self) -> Option<[f64; 2]> {
        self.selection
    }

    /// Whether a gesture is in progress; outside state must not overwrite the selection then.
    pub fn is_active(&self) -> bool {
        self.drag.is_some()
    }

    /// `brush.move`: set the selection programmatically, e.g. after a zoom.
    pub fn set_selection(&mut self, selection: Option<[f64; 2]>) {
        self.selection = selection;
    }

    pub fn handle(&mut self, input: Pointer) -> Option<BrushEvent> {
        let [lo, hi] = self.extent;
        match input {
            Pointer::Pressed { x, .. } => {
                let x = x.clamp(lo, hi);
                self.drag = Some(match self.selection {
                    Some([a, b]) if (a..=b).contains(&x) => Drag::Moving { grab: x, from: [a, b] },
                    _ => {
                        self.selection = None;
                        Drag::Creating { origin: x }
                    }
                });
                Some(BrushEvent::Start(self.selection))
            }
            Pointer::Moved { x, .. } => {
                let x = x.clamp(lo, hi);
                self.selection = Some(match self.drag? {
                    Drag::Creating { origin } => [origin.min(x), origin.max(x)],
                    Drag::Moving { grab, from: [a, b] } => {
                        let d = (x - grab).clamp(lo - a, hi - b);
                        [a + d, b + d]
                    }
                });
                Some(BrushEvent::Brush(self.selection))
            }
            Pointer::Released => {
                self.drag.take()?;
                if matches!(self.selection, Some([a, b]) if a == b) {
                    self.selection = None;
                }
                Some(BrushEvent::End(self.selection))
            }
            Pointer::Wheel { .. } | Pointer::DoubleClicked { .. } => None,
        }
    }
}
