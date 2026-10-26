//! What the iced glue does with a figure, minus iced:
//! route pointer input to a panel's gesture, then either
//! publish the scale change or keep it in widget state.

use crate::fig::{Figure, Gesture, Kind, Rect, Resolved};
use crate::fig::{ScaleRef, Value};
use d3::interact::{BrushEvent, BrushX, Pointer};
use d3::interact::{Transform, Zoom};
use std::collections::HashMap;

/// Widget state, as iced keeps it between events.
#[derive(Debug, Default)]
pub struct State {
    /// Domains of scales nobody listens to.
    views: HashMap<usize, [f64; 2]>,
    /// Published domains the app may not have applied yet.
    pending: HashMap<usize, [f64; 2]>,
    active: Option<(usize, Active)>,
}

#[derive(Debug)]
enum Active {
    Zoom(Zoom),
    Brush(BrushX),
}

type Publish<'a, Msg> = Box<dyn Fn([f64; 2]) -> Msg + 'a>;

/// The widget: a figure, its size, and which scale
/// changes become app messages.
pub struct Chart<'a, Msg> {
    fig: &'a Figure,
    size: [f64; 2],
    publish: HashMap<usize, Publish<'a, Msg>>,
}

impl<'a, Msg> Chart<'a, Msg> {
    pub fn new(fig: &'a Figure, size: [f64; 2]) -> Self {
        let publish = HashMap::new();
        Self { fig, size, publish }
    }

    /// Changes to `scale` become messages; the app owns
    /// its domain. Without this, the widget keeps it.
    pub fn on_change<S: Kind>(
        mut self,
        scale: ScaleRef<S>,
        f: impl Fn([S::Domain; 2]) -> Msg + 'a,
    ) -> Self {
        let f = move |d: [f64; 2]| {
            f(d.map(S::Domain::from_f64))
        };
        self.publish.insert(scale.index(), Box::new(f));
        self
    }

    pub fn resolved(&self, state: &State) -> Resolved {
        let mut views = state.views.clone();
        views.extend(&state.pending);
        self.fig.resolve(self.size, &views)
    }

    /// One pointer event, in figure pixels.
    pub fn update(
        &self,
        state: &mut State,
        input: Pointer,
    ) -> Option<Msg> {
        let hover = matches!(input, Pointer::Moved { .. });
        if hover && state.active.is_none() {
            return None;
        }
        let r = self.resolved(state);
        let p = match (&state.active, at(input)) {
            (Some((p, _)), _) => *p,
            (None, Some(pt)) => r
                .rects
                .iter()
                .position(|rect| rect.contains(pt))?,
            (None, None) => return None,
        };
        let gesture = self.fig.panels[p].gesture?;
        let rect = r.rects[p];
        let local = shift(input, rect);
        let ends = !matches!(
            input,
            Pointer::Pressed { .. } | Pointer::Moved { .. }
        );
        let mut active = match state.active.take() {
            Some((_, a)) => a,
            None => match gesture {
                Gesture::ZoomX { .. } => {
                    Active::Zoom(Zoom::default())
                }
                Gesture::BrushX { .. } => {
                    Active::Brush(BrushX::new([0.0, rect.w]))
                }
            },
        };
        let change = match (gesture, &mut active) {
            (Gesture::ZoomX { scale }, Active::Zoom(z)) => {
                let s = r.along(p, scale, true);
                let t = Transform::IDENTITY;
                z.handle(local, t).map(|t| {
                    let d = [0.0, rect.w]
                        .map(|px| s.invert((px - t.x) / t.k));
                    (scale, d)
                })
            }
            (
                Gesture::BrushX { over, sets },
                Active::Brush(b),
            ) => {
                let o = r.along(p, over, true);
                if !b.is_active() {
                    let shown = r.scales[sets]
                        .domain()
                        .map(|v| o.apply(v));
                    b.set_selection(Some(shown));
                }
                match b.handle(local) {
                    Some(
                        BrushEvent::Brush(Some(s))
                        | BrushEvent::End(Some(s)),
                    ) => Some((sets, s.map(|x| o.invert(x)))),
                    Some(BrushEvent::End(None)) => {
                        Some((sets, r.scales[over].domain()))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if ends {
            state.pending.clear();
        } else {
            state.active = Some((p, active));
        }
        let (scale, domain) = change?;
        match self.publish.get(&scale) {
            Some(f) => {
                if !ends {
                    state.pending.insert(scale, domain);
                }
                Some(f(domain))
            }
            None => {
                state.views.insert(scale, domain);
                None
            }
        }
    }
}

fn at(input: Pointer) -> Option<(f64, f64)> {
    match input {
        Pointer::Pressed { x, y }
        | Pointer::Moved { x, y }
        | Pointer::Wheel { x, y, .. }
        | Pointer::DoubleClicked { x, y } => Some((x, y)),
        Pointer::Released => None,
    }
}

fn shift(input: Pointer, r: Rect) -> Pointer {
    let (dx, dy) = (r.x, r.y);
    match input {
        Pointer::Pressed { x, y } => Pointer::Pressed {
            x: x - dx,
            y: y - dy,
        },
        Pointer::Moved { x, y } => Pointer::Moved {
            x: x - dx,
            y: y - dy,
        },
        Pointer::Wheel { x, y, lines } => Pointer::Wheel {
            x: x - dx,
            y: y - dy,
            lines,
        },
        Pointer::DoubleClicked { x, y } => {
            Pointer::DoubleClicked { x: x - dx, y: y - dy }
        }
        Pointer::Released => Pointer::Released,
    }
}
