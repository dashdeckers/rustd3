//! [`Brush`]: a brush a panel places (`Panel::brush`), as the widget keeps
//! it across rebuilds of the figure: core's `BrushX` or `BrushY` over the
//! panel's plot area in list px, the scale its selections invert through,
//! what it selects, and the gesture in progress.
//!
//! Brushes are kept by their panel's grid cell, the handle they are over
//! (position and scale type) and their axis, and dropped when the rebuilt
//! figure has no such brush, so a figure rebuilt in `view` keeps a gesture
//! alive. Between gestures a brush takes its selection from the domain it
//! selects ([`Brush::sync`]): its target's current domain, or for a brush
//! without a target its own last selection, kept as a domain, so a zoom of
//! either moves the band at the next refresh. During a gesture it never
//! does: the app may not have applied what the gesture published yet (use
//! case 1's bug).

mod listener;
mod machine;

use iced_core::renderer::{self, Quad};
use iced_core::{Color, Point, Rectangle, Size};
pub(crate) use listener::Listener;
use machine::Machine;
use rustd3_core::color::Rgb;
use rustd3_core::interact::{BrushEvent, Pointer};
use rustd3_figure::brush::{Dim, Placed};
use rustd3_figure::kurbo::Rect;
use rustd3_figure::layout;
use rustd3_figure::scale::Scale;

use crate::plot::domains::Key;
use crate::plot::overlay;

/// How close to over's whole range, px, a target's window counts as all of
/// it.
const WHOLE: f64 = 1e-6;

/// The band's fill: its share of the ink's opacity.
const FILL_ALPHA: f32 = 0.1;

/// The band's outline: its share of the ink's opacity.
const OUTLINE_ALPHA: f32 = 0.4;

/// A placed brush as one refresh resolved it.
pub(crate) struct Spot<'r> {
    /// Its panel's flat index.
    pub(crate) panel: usize,
    /// Its panel's grid cell.
    pub(crate) cell: (usize, usize),
    /// What the figure placed.
    pub(crate) placed: Placed,
    /// The handle it is over.
    pub(crate) over: Key,
    /// That handle's resolved scale, before any range.
    pub(crate) scale: &'r dyn Scale,
    /// The panel's plot area, list px.
    pub(crate) area: Rect,
    /// Its target's current scale (the widget's own, published, or as
    /// resolved), if it has a target.
    pub(crate) target: Option<Box<dyn Scale>>,
}

/// One placed brush as the widget keeps it.
pub(crate) struct Brush {
    /// The grid cell of its panel.
    cell: (usize, usize),
    /// The handle it is over.
    over: Key,
    /// What the figure placed, as the last refresh resolved it.
    placed: Placed,
    /// Its panel's flat index in the last refresh.
    panel: usize,
    /// Core's brush; its extent is `extent`.
    machine: Machine,
    /// The plot area the machine was made over, list px.
    extent: Rect,
    /// Over's resolved scale set to the plot area's range, as the last
    /// refresh between gestures had it: selections invert through it, so a
    /// gesture inverts through the scale at its start.
    scale: Box<dyn Scale>,
    /// Without a target: the selection, as a scale of over's type whose
    /// domain it is.
    selected: Option<Box<dyn Scale>>,
    /// During a gesture: the plot areas it holds.
    held: Option<layout::Areas>,
    /// The selection the gestures of this frame last reported, not yet
    /// published.
    fresh: Option<Option<[f64; 2]>>,
}

impl Brush {
    /// The state of the brush at `spot`, without a selection.
    pub(crate) fn new(spot: &Spot<'_>) -> Brush {
        let dim = spot.placed.get_dim();
        Brush {
            cell: spot.cell,
            over: spot.over,
            placed: spot.placed,
            panel: spot.panel,
            machine: Machine::new(dim, spot.area),
            extent: spot.area,
            scale: spot.scale.with_range(range(dim, spot.area)),
            selected: None,
            held: None,
            fresh: None,
        }
    }

    /// Whether this is the state of the brush at `spot`: the same grid
    /// cell, handle and axis.
    pub(crate) fn is(&self, spot: &Spot<'_>) -> bool {
        self.cell == spot.cell
            && self.over == spot.over
            && self.placed.get_dim() == spot.placed.get_dim()
    }

    /// Its panel's flat index in the last refresh.
    pub(crate) fn panel(&self) -> usize {
        self.panel
    }

    /// The handle it is over.
    pub(crate) fn over(&self) -> Key {
        self.over
    }

    /// What the figure placed.
    pub(crate) fn placed(&self) -> &Placed {
        &self.placed
    }

    /// Whether a gesture is in progress.
    pub(crate) fn is_active(&self) -> bool {
        self.machine.is_active()
    }

    /// The plot areas its gesture holds, while one is in progress.
    pub(crate) fn held(&self) -> Option<&layout::Areas> {
        self.held.as_ref().filter(|_| self.is_active())
    }

    /// Brings the state to the refresh that resolved `spot`: its panel and
    /// placed brush, and between gestures its plot area, over's scale, and
    /// the selection the domain it selects shows there: its target's
    /// current domain (none when the target shows all of over's domain, as
    /// at the start), else its own last selection; clipped to the plot
    /// area.
    pub(crate) fn sync(&mut self, spot: &Spot<'_>) {
        let (placed, area) = (spot.placed, spot.area);
        self.panel = spot.panel;
        self.placed = placed;
        if self.is_active() {
            return;
        }
        let dim = placed.get_dim();
        if area != self.extent {
            self.machine = Machine::new(dim, area);
            self.extent = area;
        }
        self.scale = spot.scale.with_range(range(dim, area));
        let window = match placed.get_target() {
            Some(_) => spot.target.as_deref(),
            None => self.selected.as_deref(),
        };
        let selection = window
            .and_then(|window| placed.place(&*self.scale, window))
            .and_then(|ends| band(ends, along(dim, area), placed.get_target().is_some()));
        self.machine.set_selection(selection);
    }

    /// Feeds `pointer` (list px) to the machine; a press that starts a
    /// gesture holds `held`. What d3 emits for it.
    pub(crate) fn feed(
        &mut self,
        pointer: Pointer,
        held: Option<layout::Areas>,
    ) -> Option<BrushEvent> {
        let event = self.machine.handle(pointer);
        if let Some(BrushEvent::Start(_)) = event {
            self.held = held;
        }
        if !self.machine.is_active() {
            self.held = None;
        }
        event
    }

    /// What `selection` selects, as a scale of `template`'s type (the
    /// target's, or over's without a target): the values the scale at the
    /// gesture's start inverts the ends to, over's whole domain for
    /// `None` ([`Placed::window`]).
    pub(crate) fn window(
        &self,
        template: &dyn Scale,
        selection: Option<[f64; 2]>,
    ) -> Option<Box<dyn Scale>> {
        self.placed.window(&*self.scale, template, selection)
    }

    /// A gesture reported `selection`: kept to publish, and for a brush
    /// without a target kept as a domain.
    pub(crate) fn report(&mut self, selection: Option<[f64; 2]>) {
        self.fresh = Some(selection);
        if self.placed.get_target().is_none() {
            self.selected = self
                .window(&*self.scale, selection)
                .filter(|_| selection.is_some());
        }
    }

    /// Clears the selection, as a double click does: reported when there
    /// was one.
    pub(crate) fn clear(&mut self) {
        if self.machine.selection().is_some() {
            self.fresh = Some(None);
        }
        self.machine.set_selection(None);
        self.selected = None;
    }

    /// Draws the selection, the live rectangle during a gesture and the
    /// band between gestures alike, for a widget whose device origin is
    /// `origin` at `factor`: across the plot area (its full height for an
    /// x brush, its full width for a y brush), filled with `ink` at 10% and
    /// outlined one device px wide at 40%, in a layer clipped to the plot
    /// area. Its edges are rounded to device px; the outline's sides and
    /// the fill are quads that do not overlap, so both renderers composite
    /// each pixel once.
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        origin: [i32; 2],
        factor: f32,
        ink: Rgb,
    ) where
        Renderer: renderer::Renderer,
    {
        let Some([a, b]) = self.machine.selection() else {
            return;
        };
        let k = f64::from(factor);
        let device = |v: f64| (v * k).round() as i32;
        let area = [
            self.extent.x0,
            self.extent.y0,
            self.extent.x1,
            self.extent.y1,
        ]
        .map(device);
        let [x0, y0, x1, y1] = area;
        let [x0, y0, x1, y1] = match self.placed.get_dim() {
            Dim::X => [device(a).max(x0), y0, device(b).min(x1), y1],
            Dim::Y => [x0, device(a).max(y0), x1, device(b).min(y1)],
        };
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let rect = |[x0, y0, x1, y1]: [i32; 4]| {
            Rectangle::new(
                Point::new(
                    (origin[0] + x0) as f32 / factor,
                    (origin[1] + y0) as f32 / factor,
                ),
                Size::new((x1 - x0) as f32 / factor, (y1 - y0) as f32 / factor),
            )
        };
        let ink = overlay::color(ink);
        let fill = Color {
            a: ink.a * FILL_ALPHA,
            ..ink
        };
        let line = Color {
            a: ink.a * OUTLINE_ALPHA,
            ..ink
        };
        // Top and bottom rows, then the side columns between them, then
        // the inside.
        let mut quads = vec![([x0, y0, x1, y0 + 1], line)];
        if y1 - y0 > 1 {
            quads.push(([x0, y1 - 1, x1, y1], line));
        }
        if y1 - y0 > 2 {
            quads.push(([x0, y0 + 1, x0 + 1, y1 - 1], line));
            if x1 - x0 > 1 {
                quads.push(([x1 - 1, y0 + 1, x1, y1 - 1], line));
            }
            if x1 - x0 > 2 {
                quads.push(([x0 + 1, y0 + 1, x1 - 1, y1 - 1], fill));
            }
        }
        renderer.with_layer(rect(area), |renderer| {
            for (bounds, color) in quads {
                let quad = Quad {
                    bounds: rect(bounds),
                    ..Quad::default()
                };
                renderer.fill_quad(quad, color);
            }
        });
    }

    /// The selection to publish once, if the frame reported one, and the
    /// scale it inverts through.
    pub(crate) fn take(&mut self) -> Option<(Option<[f64; 2]>, &dyn Scale)> {
        let fresh = self.fresh.take()?;
        Some((fresh, &*self.scale))
    }
}

/// The range a scale has along `dim` over `area`: x from its left, y up
/// from its bottom.
fn range(dim: Dim, area: Rect) -> [f64; 2] {
    match dim {
        Dim::X => [area.x0, area.x1],
        Dim::Y => [area.y1, area.y0],
    }
}

/// `area`'s ends along `dim`, increasing.
fn along(dim: Dim, area: Rect) -> [f64; 2] {
    match dim {
        Dim::X => [area.x0, area.x1],
        Dim::Y => [area.y0, area.y1],
    }
}

/// The band a window at `ends` shows over `[low, high]`: clipped to it;
/// `None` when it lies outside or has no width there, or, when a window
/// covering all of it means no selection (`whole_is_none`, a target that
/// shows all of over's domain), when it covers all of it.
fn band(ends: [f64; 2], [low, high]: [f64; 2], whole_is_none: bool) -> Option<[f64; 2]> {
    let [a, b] = if ends[1] < ends[0] {
        [ends[1], ends[0]]
    } else {
        ends
    };
    if whole_is_none && a <= low + WHOLE && b >= high - WHOLE {
        return None;
    }
    let [a, b] = [a.max(low), b.min(high)];
    (a < b).then_some([a, b])
}

#[cfg(test)]
mod tests {
    use super::band;

    #[test]
    fn a_band_is_clipped_to_the_plot_area_and_a_whole_window_is_none() {
        assert_eq!(band([30.0, 10.0], [0.0, 100.0], true), Some([10.0, 30.0]));
        assert_eq!(band([-20.0, 40.0], [0.0, 100.0], true), Some([0.0, 40.0]));
        assert_eq!(band([120.0, 140.0], [0.0, 100.0], false), None);
        assert_eq!(band([0.0, 100.0 - 1e-9], [0.0, 100.0], true), None);
        assert_eq!(band([-5.0, 105.0], [0.0, 100.0], true), None);
        // Without a target a selection of all of it stays.
        assert_eq!(band([0.0, 100.0], [0.0, 100.0], false), Some([0.0, 100.0]));
    }
}
