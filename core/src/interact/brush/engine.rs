//! The engine of [`BrushX`](crate::interact::BrushX) and
//! [`BrushY`](crate::interact::BrushY): d3-brush 3.0.0's `brush(dim)` for
//! one pointer and `dim` ∈ {X, Y}, without keyboard modes (`brush.js`).

use crate::interact::{BrushEvent, Button, Pointer};
use crate::js;

/// The coordinate a brush selects: d3's `dim` (`brush.js:25–37`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Dim {
    /// d3's `X`: the selection's x interval; its y spans the extent.
    X,
    /// d3's `Y`: the selection's y interval; its x spans the extent.
    Y,
}

/// d3's `brush(dim)`: the extent, the selection and the gesture in
/// progress.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Engine {
    dim: Dim,
    /// d3's `state.extent`, [[x0, y0], [x1, y1]].
    extent: [[f64; 2]; 2],
    handle_size: f64,
    /// d3's `state.selection`, [[w, n], [e, s]]: along the other axis it
    /// always spans the extent, as `dim.input` and `started` set it.
    selection: Option<[[f64; 2]; 2]>,
    gesture: Option<Gesture>,
}

/// d3's locals of one pointer's gesture (`started`, 334–411) that a
/// one-dimensional brush without keyboard modes reads, along the brushed
/// axis.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Gesture {
    mode: Mode,
    /// d3's `w0` and `e0` (`n0` and `s0` for Y): the edges a move starts
    /// from.
    edges: [f64; 2],
    /// d3's `point0`.
    point0: f64,
    /// d3's local `selection`: what a move compares against, which
    /// `state.selection` replaces while it is set (479, 505).
    selection: [[f64; 2]; 2],
}

/// How a gesture moves the selection: d3's `mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// `MODE_DRAG`, a press on the selection: both edges move.
    Drag,
    /// `MODE_HANDLE`, a press on a handle or the overlay: one edge
    /// follows the pointer.
    Handle(Edge),
}

/// An edge of the selection along the brushed axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    /// `w` (`n` for Y): d3's sign −1.
    Low,
    /// `e` (`s` for Y), and the overlay: d3's sign +1.
    High,
}

impl Engine {
    /// d3's `brush(dim)` over `extent`, handle size 6, no selection.
    pub(super) fn new(dim: Dim, extent: [[f64; 2]; 2]) -> Self {
        Engine {
            dim,
            extent,
            handle_size: 6.0,
            selection: None,
            gesture: None,
        }
    }

    /// d3's `brush.handleSize(px)`.
    pub(super) fn handle_size(self, px: f64) -> Self {
        Engine {
            handle_size: px,
            ..self
        }
    }

    /// Feeds one event; what d3 emits for it.
    pub(super) fn handle(&mut self, input: Pointer) -> Option<BrushEvent> {
        match input {
            Pointer::Pressed {
                x,
                y,
                button,
                modifiers,
            } => {
                // d3's defaultFilter (108–110).
                if button != Button::Left || modifiers.get_ctrl() {
                    return None;
                }
                self.started([x, y])
            }
            Pointer::Moved { x, y } => self.moved([x, y]),
            Pointer::Released { button } => {
                if button != Button::Left {
                    return None;
                }
                self.ended()
            }
            Pointer::Wheel { .. }
            | Pointer::DoubleClicked { .. }
            | Pointer::Modifiers(_)
            | Pointer::Left => None,
        }
    }

    /// d3's `dim.output(state.selection)`.
    pub(super) fn selection(&self) -> Option<[f64; 2]> {
        self.selection.map(|selection| self.output(selection))
    }

    /// d3's `brush.move(group, selection)` without events: `dim.input`
    /// spans the other axis over the extent.
    pub(super) fn set_selection(&mut self, selection: Option<[f64; 2]>) {
        self.selection = selection.map(|[low, high]| self.spanning(low, high));
    }

    /// A gesture is in progress.
    pub(super) fn is_active(&self) -> bool {
        self.gesture.is_some()
    }

    /// d3's `started` (334–411): the hit test gives the mode and the
    /// selection; a press on the overlay starts a new one at the point.
    fn started(&mut self, p: [f64; 2]) -> Option<BrushEvent> {
        let (mode, selection) = self.target(p)?;
        let a = self.dim.index();
        self.selection = Some(selection);
        self.gesture = Some(Gesture {
            mode,
            edges: [selection[0][a], selection[1][a]],
            point0: p[a],
            selection,
        });
        Some(BrushEvent::Start(self.selection()))
    }

    /// d3's `moved` and `move` (413–491) for one pointer.
    fn moved(&mut self, p: [f64; 2]) -> Option<BrushEvent> {
        let mut gesture = self.gesture?;
        let a = self.dim.index();
        let (low, high) = (self.extent[0][a], self.extent[1][a]);
        let [w0, e0] = gesture.edges;
        let dx = p[a] - gesture.point0;
        let (w1, e1) = match gesture.mode {
            Mode::Drag => {
                let dx = js::max(low - w0, js::min(high - e0, dx));
                (w0 + dx, e0 + dx)
            }
            Mode::Handle(Edge::Low) => (w0 + js::max(low - w0, js::min(high - w0, dx)), e0),
            Mode::Handle(Edge::High) => (w0, e0 + js::max(low - e0, js::min(high - e0, dx))),
        };
        // The moving edge passed the other: the handle flips (465–477).
        let (w1, e1) = if e1 < w1 {
            gesture.edges = [e0, w0];
            if let Mode::Handle(edge) = gesture.mode {
                gesture.mode = Mode::Handle(edge.flipped());
            }
            (e1, w1)
        } else {
            (w1, e1)
        };
        if let Some(selection) = self.selection {
            gesture.selection = selection;
        }
        self.gesture = Some(gesture);
        let next = self.spanning(w1, e1);
        if next == gesture.selection {
            return None;
        }
        self.selection = Some(next);
        Some(BrushEvent::Brush(self.selection()))
    }

    /// d3's `ended` (493–508): an empty selection becomes `None`.
    fn ended(&mut self) -> Option<BrushEvent> {
        let gesture = self.gesture.take()?;
        if empty(self.selection.unwrap_or(gesture.selection)) {
            self.selection = None;
        }
        Some(BrushEvent::End(self.selection()))
    }

    /// The topmost element under `p` in d3's DOM order (handles above the
    /// selection above the overlay; `e` above `w`, `s` above `n`), as the
    /// gesture's mode and selection. During a gesture d3 sets
    /// `pointer-events: none` on the group, which only the overlay
    /// overrides (170, 391), so a press then hits only the overlay.
    fn target(&self, p: [f64; 2]) -> Option<(Mode, [[f64; 2]; 2])> {
        if let (None, Some(selection)) = (&self.gesture, self.selection) {
            for edge in [Edge::High, Edge::Low] {
                let (origin, size) = self.handle_rect(selection, edge);
                if covers(origin, size, p) {
                    return Some((Mode::Handle(edge), selection));
                }
            }
            let (origin, size) = rect(selection);
            if covers(origin, size, p) {
                return Some((Mode::Drag, selection));
            }
        }
        let (origin, size) = rect(self.extent);
        let a = self.dim.index();
        covers(origin, size, p).then(|| (Mode::Handle(Edge::High), self.spanning(p[a], p[a])))
    }

    /// `redraw`'s rect of `edge`'s handle (269–273) as its origin and
    /// size: `handle_size` across the edge, the selection's span plus
    /// `handle_size` along it.
    fn handle_rect(&self, selection: [[f64; 2]; 2], edge: Edge) -> ([f64; 2], [f64; 2]) {
        let a = self.dim.index();
        let b = 1 - a;
        let half = self.handle_size / 2.0;
        let at = match edge {
            Edge::Low => selection[0][a],
            Edge::High => selection[1][a],
        };
        let mut origin = [0.0; 2];
        let mut size = [0.0; 2];
        origin[a] = at - half;
        size[a] = self.handle_size;
        origin[b] = selection[0][b] - half;
        size[b] = selection[1][b] - selection[0][b] + self.handle_size;
        (origin, size)
    }

    /// The selection from `low` to `high` along the brushed axis, spanning
    /// the extent along the other: d3's `dim.input`.
    fn spanning(&self, low: f64, high: f64) -> [[f64; 2]; 2] {
        let a = self.dim.index();
        let mut selection = self.extent;
        selection[0][a] = low;
        selection[1][a] = high;
        selection
    }

    /// d3's `dim.output`: the interval along the brushed axis.
    fn output(&self, selection: [[f64; 2]; 2]) -> [f64; 2] {
        let a = self.dim.index();
        [selection[0][a], selection[1][a]]
    }
}

impl Dim {
    /// The index of the coordinate it selects.
    fn index(self) -> usize {
        match self {
            Dim::X => 0,
            Dim::Y => 1,
        }
    }
}

impl Edge {
    /// The other edge: d3's `flipX` (`flipY`) of a handle.
    fn flipped(self) -> Self {
        match self {
            Edge::Low => Edge::High,
            Edge::High => Edge::Low,
        }
    }
}

/// `redraw`'s rect from `corners[0]` to `corners[1]` (261–266) as its
/// origin and size.
fn rect(corners: [[f64; 2]; 2]) -> ([f64; 2], [f64; 2]) {
    let [[x0, y0], [x1, y1]] = corners;
    ([x0, y0], [x1 - x0, y1 - y0])
}

/// Whether an SVG rect at `origin` of `size` covers `p`, edges included;
/// a rect of negative size covers nothing, as SVG draws none.
fn covers(origin: [f64; 2], size: [f64; 2], p: [f64; 2]) -> bool {
    (0..2).all(|i| origin[i] <= p[i] && p[i] <= origin[i] + size[i])
}

/// d3's `empty`: zero width or zero height (131–134).
fn empty(selection: [[f64; 2]; 2]) -> bool {
    selection[0][0] == selection[1][0] || selection[0][1] == selection[1][1]
}
