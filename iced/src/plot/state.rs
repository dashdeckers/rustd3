//! [`State`]: what the widget keeps in iced's widget tree between frames,
//! and the refresh that renders the cells whose content changed.

use std::sync::Arc;

use iced_core::shell::Waker;
use iced_core::time::Instant;
use iced_core::{Event, Point, Rectangle, Size, image, mouse};
use rustd3_core::interact::{self, BrushEvent, Button, Modifiers, Pointer};
use rustd3_figure::list::{self, Bounds, Outlines};
use rustd3_figure::{Cache, Figure, Resolved, Target, Theme, hit, kurbo, raster};

use crate::plot::area::Area;
use crate::plot::brush::{self, Brush};
use crate::plot::budget::Budget;
use crate::plot::cost;
use crate::plot::domains::{Domains, Key};
use crate::plot::gesture::{self, Gesture, Track};
use crate::plot::hover::{self, Hover};
use crate::plot::input::Input;
use crate::plot::job::{Done, Job, Stamp};
use crate::plot::listener::Listener;
use crate::plot::overlay::{self, Overlay};
use crate::plot::preview::Basis;
use crate::plot::probe::{self, Probe, Sample, Stage};
use crate::plot::region::{self, Along, Region};
use crate::plot::slot::{self, Band, Slot};
use crate::plot::tile::{self, Tile};
use crate::plot::worker::Worker;
use crate::plot::{Caching, Render, handle};
use crate::pointer::Tracker;

/// The widget's tree state: the figure's cache, the rasterizer, and the
/// slots of the partition's cells. It borrows nothing of the figure.
pub(crate) struct State {
    /// Laid-out labels, thinned lines and `Fit::Visible` memory.
    cache: Cache,
    /// vello_cpu with its own pool, for cells rendered inline.
    raster: raster::Renderer,
    /// Glyph outlines the list's command bounds are measured with.
    outlines: Outlines,
    /// The widget's logical size from the last layout.
    size: Size,
    /// The window's factor from `Opened` or `Rescaled`.
    window: Option<f32>,
    /// The factor the slots were planned at; `None` before the first
    /// refresh.
    factor: Option<f32>,
    /// Whether the figure was rebuilt or the widget resized since the
    /// last refresh.
    stale: bool,
    /// The partition's cells, row-major, and what each shows; cells inside
    /// a plot area show nothing of their own (its tiles do).
    slots: Vec<Slot>,
    /// Per partition cell, the plot area that holds it, if one does.
    owner: Vec<Option<usize>>,
    /// Bumped whenever the content the slots hold changes.
    generation: u64,
    /// The generation that built the slots: results rendered for an older
    /// one belong to another partition.
    floor: u64,
    /// `RedrawRequested` events handled.
    frame: u64,
    /// The frame budget, from the last two `RedrawRequested` instants, and
    /// the cost model's calibration.
    budget: Budget,
    /// The plot areas, in panel order: their device rects, relative to the
    /// widget's device origin, and their tiles.
    areas: Vec<Area>,
    /// Grids begun: each area's grid gets a new number.
    grids: u64,
    /// The hit index of the last list, and the hover last reported.
    hits: hover::Hits,
    /// The pointer where the last move left it, logical px in the window;
    /// `None` when it left or something above covers it.
    pointer: Option<Point>,
    /// The crosshair and highlight where the pointer is.
    overlay: Overlay,
    /// The render thread, started with the first job set it gets.
    worker: Option<Worker>,
    /// Turns iced events into pointer input in the widget's logical px.
    tracker: Tracker,
    /// Pointer input waiting for the next `RedrawRequested`.
    input: Input,
    /// The scales gestures changed, and each handle's zoom.
    domains: Domains,
    /// The zoom or pan gesture in progress.
    gesture: Option<Gesture>,
    /// The brushes the figure places, as the last refresh resolved them;
    /// one of them may have a gesture in progress.
    brushes: Vec<Brush>,
    /// The last resolve: the scales a gesture starts from.
    resolved: Option<Resolved>,
    /// Its plot areas and axis strips, and the handles each panel zooms.
    map: region::Map,
    /// The keyboard modifiers last reported.
    modifiers: Modifiers,
    /// Published handles a double click reset: the next resolve publishes
    /// their fitted domains.
    resets: Vec<Key>,
    /// The `Plot::revision` the slots were last planned at.
    revision: u64,
}

/// What queuing a pointer asks of the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Queued {
    /// Capture the event: the widget handles it.
    pub(crate) capture: bool,
    /// Request a redraw, which applies the queue.
    pub(crate) redraw: bool,
}

/// How a `RedrawRequested` renders and reports what it did.
pub(crate) struct Pipeline<'a> {
    /// Where changed cells render.
    pub(crate) render: Render,
    /// Which cells count as changed.
    pub(crate) caching: Caching,
    /// The app's revision: a new one changes every cell.
    pub(crate) revision: u64,
    /// Where stage timings and frame reports go, if anywhere.
    pub(crate) probe: Option<&'a Probe>,
    /// Wakes the UI when the worker has results; the worker keeps a clone.
    pub(crate) waker: &'a Waker,
}

impl Default for State {
    fn default() -> Self {
        State {
            cache: cache(),
            raster: raster::Renderer::new(raster::Settings::parallel()),
            outlines: Outlines::default(),
            size: Size::ZERO,
            window: None,
            factor: None,
            stale: true,
            slots: Vec::new(),
            owner: Vec::new(),
            generation: 0,
            floor: 0,
            frame: 0,
            budget: Budget::default(),
            areas: Vec::new(),
            grids: 0,
            hits: hover::Hits::default(),
            pointer: None,
            overlay: Overlay::default(),
            worker: None,
            tracker: Tracker::default(),
            input: Input::default(),
            domains: Domains::default(),
            gesture: None,
            brushes: Vec::new(),
            resolved: None,
            map: region::Map::default(),
            modifiers: Modifiers::NONE,
            resets: Vec::new(),
            revision: 0,
        }
    }
}

impl State {
    /// A new view built a new figure, which may or may not differ.
    pub(crate) fn rebuilt(&mut self) {
        self.stale = true;
    }

    /// The layout's logical size; a new one marks the list for rebuilding.
    pub(crate) fn resize(&mut self, size: Size) {
        if size != self.size {
            self.size = size;
            self.stale = true;
        }
    }

    /// The window's factor, from `Opened` or `Rescaled`.
    pub(crate) fn rescale(&mut self, factor: f32) {
        self.window = Some(factor);
    }

    /// The window's factor, when a window event gave one.
    pub(crate) fn window_factor(&self) -> Option<f32> {
        self.window
    }

    /// The pointer input `event` makes over a widget laid out at `bounds`
    /// at `factor`, in the widget's logical px.
    pub(crate) fn translate(
        &mut self,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        factor: f32,
    ) -> Option<Pointer> {
        self.tracker = std::mem::take(&mut self.tracker).scale_factor(factor);
        self.tracker.translate(event, bounds, cursor)
    }

    /// A press waits for the next `RedrawRequested` to be timed.
    pub(crate) fn wants_redraw(&self) -> bool {
        self.tracker.wants_redraw()
    }

    /// Queues `pointer` for the next `RedrawRequested` when a gesture
    /// reads it, and says what the shell should do: a left press over a
    /// region with zoomable handles or a brushed plot area, or during a
    /// gesture (a press ends a gesture whose release was lost), and a
    /// wheel step over a region with zoomable handles or during a drag,
    /// are captured and redraw; a move during a drag or a brush gesture,
    /// queued or active, and a left release redraw; a modifier change
    /// waits for the next redraw. Wheel steps during a brush gesture are
    /// left alone.
    pub(crate) fn queue(&mut self, pointer: Pointer) -> Queued {
        let capture = Queued {
            capture: true,
            redraw: true,
        };
        let redraw = Queued {
            capture: false,
            redraw: true,
        };
        let dragging = self.is_dragging();
        let brushing = self.is_brushing();
        let queued = match pointer {
            Pointer::Wheel { x, y, .. } => (!brushing
                && (dragging || self.zoomable(kurbo::Point::new(x, y))))
            .then_some(capture),
            Pointer::Pressed {
                x,
                y,
                button: Button::Left,
                ..
            } => {
                let at = kurbo::Point::new(x, y);
                (dragging || brushing || self.zoomable(at) || self.brush_at(at).is_some())
                    .then_some(capture)
            }
            Pointer::Moved { .. } => {
                (dragging || brushing || self.input.has_press()).then_some(redraw)
            }
            Pointer::Released {
                button: Button::Left,
            } => Some(redraw),
            // A double click comes from translating `RedrawRequested`,
            // which applies it next.
            Pointer::Modifiers(_) | Pointer::DoubleClicked { .. } => Some(Queued::default()),
            Pointer::Pressed { .. } | Pointer::Released { .. } | Pointer::Left => None,
            _ => None,
        };
        if queued.is_some() {
            self.input.push(pointer);
        }
        queued.unwrap_or_default()
    }

    /// Handles one `RedrawRequested(now)`: counts the frame, applies the
    /// queued input, refreshes `figure` at `factor` when it was rebuilt,
    /// the widget resized, the factor changed or input changed a domain,
    /// and reports what the following draw shows to the pipeline's probe.
    /// Returns the number of cells whose content changed and the messages
    /// for `listeners` and the brush listeners `brushes`. When applying the
    /// input published a message, the list is not built in this pass: the
    /// runtime runs the app's `update`, rebuilds the view and sends
    /// `RedrawRequested` again (`winit/src/lib.rs:788-830`).
    pub(crate) fn redraw<Message>(
        &mut self,
        figure: &Figure<'_>,
        factor: f32,
        now: Instant,
        pipeline: &Pipeline<'_>,
        (listeners, brushes): (&[Listener<'_, Message>], &[brush::Listener<'_, Message>]),
    ) -> (usize, Vec<Message>) {
        self.frame += 1;
        self.budget.frame(now);
        let mut messages = self.apply(now, listeners, brushes);
        let changed = if messages.is_empty() {
            let (changed, published) = self.refresh(figure, factor, pipeline, listeners);
            messages = published;
            changed
        } else {
            0
        };
        let report = self.report(now);
        if let Some(probe) = pipeline.probe {
            probe.send(report);
        }
        (changed, messages)
    }

    /// When the wheel gesture in progress ends, if one is: the widget
    /// asks for a redraw then.
    pub(crate) fn wheel_end(&self) -> Option<Instant> {
        self.gesture.as_ref().and_then(Gesture::ends_at)
    }

    /// Applies the queued input at `now`, once per `RedrawRequested`:
    /// gesture boundaries, then each pointer fed to the gesture's tracks or
    /// brush; accepted steps are stored, pending for `listeners`' handles
    /// and owned otherwise. A left press over a brushed plot area starts a
    /// brush gesture, elsewhere over a region with zoomable handles a drag,
    /// which its release ends; a wheel step with no gesture starts a wheel
    /// gesture on its region (during a drag it feeds the drag's tracks,
    /// during a brush gesture nothing), which ends at the first redraw
    /// `gesture::WHEEL_END` after its last step. Returns one message per
    /// listened handle that changed (the latest domain) and one per brush
    /// whose selection changed, for `brushes`.
    fn apply<Message>(
        &mut self,
        now: Instant,
        listeners: &[Listener<'_, Message>],
        brushes: &[brush::Listener<'_, Message>],
    ) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut fresh = Vec::new();
        let mut changed = false;
        let queued = self.input.take();
        let double = queued.iter().find_map(|pointer| match *pointer {
            Pointer::DoubleClicked { x, y } => Some(kurbo::Point::new(x, y)),
            Pointer::Pressed { .. }
            | Pointer::Moved { .. }
            | Pointer::Released { .. }
            | Pointer::Wheel { .. }
            | Pointer::Modifiers(_)
            | Pointer::Left => None,
            _ => None,
        });
        if let Some(at) = double {
            // The rest of the queue belongs to the double click: the drag
            // its second press would start ends without moving anything.
            for pointer in &queued {
                if let Pointer::Modifiers(modifiers) = *pointer {
                    self.modifiers = modifiers;
                }
            }
            self.end(&mut fresh, listeners, &mut messages);
            changed |= self.unbrush(&mut fresh, listeners, &mut messages);
            changed |= self.reset(self.map.at(at), listeners);
        }
        let queued = if double.is_some() { Vec::new() } else { queued };
        for pointer in queued {
            match pointer {
                Pointer::Wheel { .. } if self.is_brushing() => {}
                Pointer::Wheel { x, y, .. } => {
                    let region = self.map.at(kurbo::Point::new(x, y));
                    let feeds = self
                        .gesture
                        .as_ref()
                        .is_some_and(|g| g.is_drag() || g.region() == region);
                    if !feeds {
                        self.end(&mut fresh, listeners, &mut messages);
                        self.begin(region, gesture::Kind::Wheel { last: now });
                    }
                    if let Some(gesture) = &mut self.gesture {
                        gesture.wheeled(now);
                    }
                    changed |= self.feed(pointer, &mut fresh, listeners);
                }
                Pointer::Pressed {
                    x,
                    y,
                    button: Button::Left,
                    ..
                } => {
                    // A press ends a wheel gesture, and a drag or a brush
                    // gesture whose release was lost.
                    self.end(&mut fresh, listeners, &mut messages);
                    changed |= self.unbrush(&mut fresh, listeners, &mut messages);
                    let at = kurbo::Point::new(x, y);
                    match self.brush_at(at) {
                        Some(index) => {
                            changed |=
                                self.stroke(index, pointer, &mut fresh, listeners, &mut messages);
                        }
                        None => {
                            self.begin(self.map.at(at), gesture::Kind::Drag);
                            changed |= self.feed(pointer, &mut fresh, listeners);
                        }
                    }
                }
                Pointer::Moved { .. }
                | Pointer::Released {
                    button: Button::Left,
                } if self.is_brushing() => {
                    changed |= self.stroke_on(pointer, &mut fresh, listeners, &mut messages);
                }
                Pointer::Moved { .. } if self.is_dragging() => {
                    changed |= self.feed(pointer, &mut fresh, listeners);
                }
                Pointer::Released {
                    button: Button::Left,
                } if self.is_dragging() => {
                    changed |= self.feed(pointer, &mut fresh, listeners);
                    self.end(&mut fresh, listeners, &mut messages);
                }
                Pointer::Modifiers(modifiers) => {
                    self.modifiers = modifiers;
                    self.feed(pointer, &mut fresh, listeners);
                    // Core's brush accepts them and changes nothing.
                    changed |= self.stroke_on(pointer, &mut fresh, listeners, &mut messages);
                }
                Pointer::Pressed { .. }
                | Pointer::Moved { .. }
                | Pointer::Released { .. }
                | Pointer::DoubleClicked { .. }
                | Pointer::Left => {}
                _ => {}
            }
        }
        if self.wheel_end().is_some_and(|end| now >= end) {
            self.end(&mut fresh, listeners, &mut messages);
        }
        self.flush(&mut fresh, listeners, &mut messages);
        self.publish(brushes, &mut messages);
        if changed {
            self.generation += 1;
            self.stale = true;
        }
        messages
    }

    /// Feeds `pointer` to brush `index`, a press that starts its gesture
    /// holding the plot areas of the last resolve, and applies what it
    /// reports: a selection is kept for `on_brush`, and with a target it
    /// sets the target's domain as a gesture's step does (published for
    /// `listeners`' handles, owned otherwise; the key goes to `fresh`); at
    /// its end what it published goes out and is the app's from then on,
    /// and the layout settles. True when it set a domain.
    fn stroke<Message>(
        &mut self,
        index: usize,
        pointer: Pointer,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) -> bool {
        let pressed = matches!(pointer, Pointer::Pressed { .. });
        let held = self
            .resolved
            .as_ref()
            .filter(|_| pressed)
            .map(Resolved::areas);
        let Some(event) = self
            .brushes
            .get_mut(index)
            .and_then(|brush| brush.feed(pointer, held))
        else {
            return false;
        };
        let (selection, end) = match event {
            BrushEvent::Start(_) => return false,
            BrushEvent::Brush(selection) => (selection, false),
            BrushEvent::End(selection) => (selection, true),
        };
        let brush = &mut self.brushes[index];
        brush.report(selection);
        let set = self.resolved.as_ref().and_then(|resolved| {
            let target = brush.placed().target()?;
            let key = Key::new(target, resolved.scale_at(target)?);
            let template = self.domains.current(key, resolved)?;
            Some((key, brush.window(&*template, selection)?))
        });
        let changed = match set {
            Some((key, window)) => {
                let listened = listeners.iter().any(|listener| listener.key() == key);
                self.domains.store(key, window, listened);
                if !fresh.contains(&key) {
                    fresh.push(key);
                }
                true
            }
            None => false,
        };
        if end {
            self.flush(fresh, listeners, messages);
            self.domains.release();
            self.stale = true;
        }
        changed
    }

    /// Feeds `pointer` to the brush whose gesture is in progress, if one
    /// is, as [`stroke`](State::stroke) does.
    fn stroke_on<Message>(
        &mut self,
        pointer: Pointer,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) -> bool {
        match self.brushes.iter().position(Brush::is_active) {
            Some(index) => self.stroke(index, pointer, fresh, listeners, messages),
            None => false,
        }
    }

    /// Ends the brush gesture in progress, if any, as its left release
    /// would.
    fn unbrush<Message>(
        &mut self,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) -> bool {
        let release = Pointer::Released {
            button: Button::Left,
        };
        self.stroke_on(release, fresh, listeners, messages)
    }

    /// One message per brush whose selection this frame changed, for the
    /// listener of the handle it is over among `listeners`: the latest.
    fn publish<Message>(
        &mut self,
        listeners: &[brush::Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) {
        for brush in &mut self.brushes {
            let over = brush.over();
            let Some((selection, scale)) = brush.take() else {
                continue;
            };
            if let Some(listener) = listeners.iter().find(|listener| listener.key() == over) {
                messages.extend(listener.publish(scale, selection));
            }
        }
    }

    /// Resets every handle the panel of `region` zooms, and its brush's
    /// target, that a gesture touched: the widget's own scales go (the
    /// figure refits them and the hold ends), published ones leave
    /// `pending` and the next resolve publishes their fitted domains; their
    /// zooms and preview views start afresh. The panel's brush loses its
    /// selection, reported to `on_brush` if it had one. True when a handle
    /// was reset.
    fn reset<Message>(&mut self, region: Region, listeners: &[Listener<'_, Message>]) -> bool {
        let Some(panel) = region.panel() else {
            return false;
        };
        let mut keys: Vec<Key> = self
            .map
            .tracks(Region::Area { panel })
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        if let Some(brush) = self.brushes.iter_mut().find(|brush| brush.panel() == panel) {
            brush.clear();
            let target = brush.placed().target().and_then(|target| {
                let resolved = self.resolved.as_ref()?;
                Some(Key::new(target, resolved.scale_at(target)?))
            });
            keys.extend(target.filter(|key| !keys.contains(key)));
        }
        let mut reset = false;
        for key in keys {
            if !self.domains.is_touched(key) {
                continue;
            }
            if listeners.iter().any(|listener| listener.key() == key) {
                self.resets.push(key);
            }
            self.domains.reset(key);
            reset = true;
        }
        if reset {
            self.stale = true;
        }
        reset
    }

    /// Starts a gesture of `kind` over `region` when it has zoomable
    /// handles: each track starts from its handle's current scale (owned,
    /// else pending, else as last resolved), and the plot areas of the
    /// last resolve are held.
    fn begin(&mut self, region: Region, kind: gesture::Kind) {
        let (Some(resolved), Some(area)) = (
            &self.resolved,
            region.panel().and_then(|panel| self.map.area(panel)),
        ) else {
            return;
        };
        let mut tracks = Vec::new();
        for (key, along) in self.map.tracks(region) {
            let Some(start) = self.domains.current(key, resolved) else {
                continue;
            };
            // Ends a wheel burst of an earlier gesture, whose anchor would
            // be in another gesture's frame.
            let _ = self.domains.zoom(key).handle(
                Pointer::Modifiers(self.modifiers),
                interact::Transform::IDENTITY,
            );
            let base = self.domains.view(key).map;
            tracks.push(Track::new(key, along, start, base));
        }
        if !tracks.is_empty() {
            let held = Some(resolved.areas());
            self.gesture = Some(Gesture::new(region, kind, tracks, held, area));
        }
    }

    /// Feeds `pointer` to the gesture's tracks and stores each accepted
    /// step, published (pending) for a handle one of `listeners` listens
    /// to, else owned; the keys go to `fresh`. True when a step was
    /// accepted.
    fn feed<Message>(
        &mut self,
        pointer: Pointer,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
    ) -> bool {
        let Some(gesture) = &mut self.gesture else {
            return false;
        };
        let steps = gesture.feed(pointer, &mut self.domains);
        let accepted = !steps.is_empty();
        for step in steps {
            let key = step.key;
            let listened = listeners.iter().any(|listener| listener.key() == key);
            self.domains.store(key, step.scale, listened);
            self.domains.move_view(key, step.view);
            if !fresh.contains(&key) {
                fresh.push(key);
            }
        }
        accepted
    }

    /// One message per key in `fresh` that one of `listeners` listens to:
    /// its published domain.
    fn flush<Message>(
        &self,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) {
        for key in fresh.drain(..) {
            let listener = listeners.iter().find(|listener| listener.key() == key);
            if let (Some(listener), Some(scale)) = (listener, self.domains.published(key)) {
                messages.extend(listener.publish(scale));
            }
        }
    }

    /// Ends the gesture in progress, if any: what it published goes out,
    /// published domains are the app's from now on, the layout is held no
    /// more and settles at the next refresh.
    fn end<Message>(
        &mut self,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) {
        if self.gesture.take().is_some() {
            self.flush(fresh, listeners, messages);
            self.domains.release();
            self.stale = true;
        }
    }

    /// Whether a drag is in progress.
    fn is_dragging(&self) -> bool {
        self.gesture.as_ref().is_some_and(Gesture::is_drag)
    }

    /// Whether a brush gesture is in progress.
    fn is_brushing(&self) -> bool {
        self.brushes.iter().any(Brush::is_active)
    }

    /// The brush of the plot area at `at` (list px), if its panel has one.
    fn brush_at(&self, at: kurbo::Point) -> Option<usize> {
        let Region::Area { panel } = self.map.at(at) else {
            return None;
        };
        self.brushes.iter().position(|brush| brush.panel() == panel)
    }

    /// Brings the brushes to `resolved`: one state per placed brush, kept
    /// by its panel's grid cell, the handle it is over and its axis, and
    /// made for brushes it did not have; states of brushes the figure no
    /// longer places go. Between gestures each takes its selection from
    /// the domain it selects (a target's as the widget shows it now).
    fn sync(&mut self, resolved: &Resolved) {
        let mut kept = Vec::with_capacity(self.brushes.len());
        for (panel, &area) in resolved.plot_areas().iter().enumerate() {
            let Some(&placed) = resolved.brush(panel) else {
                continue;
            };
            let Some(scale) = resolved.scale_at(placed.over()) else {
                continue;
            };
            let target = placed.target().and_then(|target| {
                let key = Key::new(target, resolved.scale_at(target)?);
                self.domains.current(key, resolved)
            });
            let spot = brush::Spot {
                panel,
                cell: resolved.panel_cell(panel),
                placed,
                over: Key::new(placed.over(), scale),
                scale,
                area,
                target,
            };
            let mut brush = match self.brushes.iter().position(|brush| brush.is(&spot)) {
                Some(index) => self.brushes.swap_remove(index),
                None => Brush::new(&spot),
            };
            brush.sync(&spot);
            kept.push(brush);
        }
        self.brushes = kept;
    }

    /// Whether the region at `at` (list px) has zoomable handles.
    fn zoomable(&self, at: kurbo::Point) -> bool {
        !self.map.tracks(self.map.at(at)).is_empty()
    }

    /// Resolves `figure` once at the widget's size for the screen at
    /// `factor`, with the gestures' domains as overrides (pending winning
    /// over owned) and, during a gesture, in the plot areas it holds;
    /// drops what the widget keeps for handles whose type changed. When a
    /// double click reset published handles, returns their fitted domains
    /// as messages for `listeners` and draws nothing in this pass.
    /// Otherwise draws the figure, cuts the list into cells at every
    /// plot-area edge, and renders the cells whose content hash changed:
    /// inline, shown at once, or as a job set that replaces the worker's
    /// pending one, as `pipeline.render` asks or, for `Auto`, as the cost
    /// model predicts against the frame budget. Returns how many changed.
    /// A new `pipeline.revision` refreshes as a rebuilt figure does.
    fn refresh<Message>(
        &mut self,
        figure: &Figure<'_>,
        factor: f32,
        pipeline: &Pipeline<'_>,
        listeners: &[Listener<'_, Message>],
    ) -> (usize, Vec<Message>) {
        if !self.stale && self.factor == Some(factor) && pipeline.revision == self.revision {
            return (0, Vec::new());
        }
        self.stale = false;
        self.factor = Some(factor);
        let k = f64::from(factor);
        let size = kurbo::Size::new(f64::from(self.size.width), f64::from(self.size.height));
        if !((size.width * k).round() >= 1.0 && (size.height * k).round() >= 1.0) {
            self.slots.clear();
            self.owner.clear();
            self.areas.clear();
            self.hits.rebuild(hit::Index::default());
            self.map = region::Map::default();
            self.resolved = None;
            return (0, Vec::new());
        }
        let probe = pipeline.probe;
        let target = Target::Screen { scale: k };
        let start = probe.map(Probe::now);
        self.cache.begin();
        let held = self.domains.held();
        let areas = self
            .gesture
            .as_ref()
            .and_then(Gesture::held)
            .or_else(|| self.brushes.iter().find_map(Brush::held));
        let resolved = match areas {
            Some(areas) => figure.resolve_held(size, &mut self.cache, &held, target, areas),
            None => figure.resolve(size, &mut self.cache, &held, target),
        };
        self.domains.prune(&resolved);
        self.map = region::Map::of(&resolved);
        self.sync(&resolved);
        // A reset's published handles get their fitted domains; the app
        // rebuilds the view with them, and the next pass draws.
        let fitted: Vec<Message> = std::mem::take(&mut self.resets)
            .into_iter()
            .filter_map(|key| {
                let listener = listeners.iter().find(|listener| listener.key() == key)?;
                listener.publish(resolved.scale_at(key.index())?)
            })
            .collect();
        if !fitted.is_empty() {
            self.stale = true;
            self.resolved = Some(resolved);
            return (0, fitted);
        }
        let changed = self.repaint(figure, &resolved, k, pipeline, start);
        self.resolved = Some(resolved);
        (changed, Vec::new())
    }

    /// Draws `resolved` (of `figure`, the probe's clock read at `start`
    /// before resolving) into a list, cuts it into cells at every plot-area
    /// edge and renders the cells whose content hash changed, as
    /// [`refresh`](State::refresh) describes. Returns how many changed.
    fn repaint(
        &mut self,
        figure: &Figure<'_>,
        resolved: &Resolved,
        k: f64,
        pipeline: &Pipeline<'_>,
        start: Option<Instant>,
    ) -> usize {
        let probe = pipeline.probe;
        let middle = probe.map(Probe::now);
        let list = figure.draw_resolved(&mut self.cache, resolved);
        if let (Some(probe), Some(start), Some(middle)) = (probe, start, middle) {
            let end = probe.now();
            for (stage, from, to) in [(Stage::Resolve, start, middle), (Stage::List, middle, end)] {
                probe.send(Sample::Stage {
                    frame: self.frame,
                    stage,
                    duration: to.saturating_duration_since(from),
                });
            }
        }
        let (xs, ys) = resolved.edges();
        let cells = list::cells(&list, &xs, &ys, k);
        let lists = resolved.plot_areas();
        let rects: Vec<[i32; 4]> = lists.iter().map(|area| device(*area, k)).collect();
        let views: Vec<[Option<Basis>; 2]> = (0..rects.len()).map(|p| self.views(p)).collect();
        let plan = self.plan(
            &cells.cells,
            (&rects, lists),
            &views,
            (pipeline.caching, pipeline.revision),
        );
        if plan.is_empty() {
            return 0;
        }
        self.hits.rebuild(figure.hits(resolved, &mut self.cache));
        let bounds = Arc::new(Bounds::of(&list, &mut self.outlines));
        let list = Arc::new(list);
        let enqueued = probe.map(|probe| Stamp::now(probe.clock()));
        let generation = self.generation;
        // A tile's work is its share of its plot area's, by pixels.
        let works: Vec<u64> = (0..rects.len())
            .map(|p| {
                (0..cells.cells.len())
                    .filter(|&c| self.owner[c] == Some(p))
                    .map(|c| cells.cells[c].work)
                    .sum()
            })
            .collect();
        let job = |at: At, slots: &[Slot], areas: &[Area]| -> Job {
            let (target, grid, slot, work, view) = match at {
                At::Cell(index) => {
                    let target = probe::Target::Cell(
                        u32::try_from(index).expect("invariant: fewer than 2^32 cells"),
                    );
                    (target, 0, &slots[index], cells.cells[index].work, [None; 2])
                }
                At::Tile(p, t) => {
                    let tile: &Tile = &areas[p].tiles()[t];
                    let key = tile.key();
                    let target = probe::Target::Tile {
                        area: key.area,
                        i: key.i,
                        j: key.j,
                    };
                    let (px, all) = (
                        slot::pixels(tile.slot().rect()),
                        slot::pixels(areas[p].rect()),
                    );
                    let work = u64::try_from(
                        u128::from(works[p]) * u128::from(px) / u128::from(all.max(1)),
                    )
                    .unwrap_or(u64::MAX);
                    (target, areas[p].serial(), tile.slot(), work, views[p])
                }
            };
            let rect = slot.rect();
            Job {
                target,
                grid,
                generation,
                hash: slot.hash(),
                list: Arc::clone(&list),
                bounds: Arc::clone(&bounds),
                rect,
                bands: slot::bands(rect),
                scale: k,
                work,
                enqueued,
                view,
            }
        };
        let model: f64 = plan
            .iter()
            .map(|&at| {
                let job = job(at, &self.slots, &self.areas);
                cost::model_ms(slot::pixels(job.rect), job.work)
            })
            .sum();
        let inline = match pipeline.render {
            Render::Inline => true,
            Render::Auto if self.budget.is_inline(model) => true,
            Render::Auto | Render::Worker => !self.start(pipeline.waker),
        };
        if inline {
            for &at in &plan {
                let done = job(at, &self.slots, &self.areas).run(&mut self.raster);
                self.land(done, true, probe);
            }
            // Nothing left for the worker: its rest is out of date.
            if let Some(worker) = &self.worker
                && self.waiting().is_empty()
            {
                worker.replace(Vec::new());
            }
        } else {
            // The new set replaces the pending one, whose unfinished jobs
            // the worker abandons: every slot and tile still waiting
            // renders again, tiles first.
            let order = self.waiting();
            for &at in &order {
                match at {
                    At::Cell(index) => self.slots[index].requeue(generation),
                    At::Tile(p, t) => self.areas[p].tiles_mut()[t].slot_mut().requeue(generation),
                }
            }
            let jobs = order
                .into_iter()
                .map(|at| job(at, &self.slots, &self.areas))
                .collect();
            if let Some(worker) = &self.worker {
                worker.replace(jobs);
            }
        }
        plan.len()
    }

    /// Starts the worker unless it runs, waking the UI through `waker`.
    /// False when the OS cannot start a thread: cells then render inline.
    fn start(&mut self, waker: &Waker) -> bool {
        if self.worker.is_none() {
            self.worker = Worker::start(waker.clone()).ok();
        }
        self.worker.is_some()
    }

    /// The views of the handles leading each axis of `panel` now, each
    /// started when it has none.
    fn views(&mut self, panel: usize) -> [Option<Basis>; 2] {
        [Along::X, Along::Y].map(|along| {
            let key = self.map.lead(panel, along)?;
            Some(Basis {
                key,
                view: self.domains.view(key),
            })
        })
    }

    /// The views of the handles leading each axis of `panel` now, as
    /// started; an axis whose handle has none has no basis.
    fn peek(&self, panel: usize) -> [Option<Basis>; 2] {
        [Along::X, Along::Y].map(|along| {
            let key = self.map.lead(panel, along)?;
            Some(Basis {
                key,
                view: self.domains.peek(key)?,
            })
        })
    }

    /// Every cell and tile waiting for a render: tiles first, area by
    /// area, then the cells.
    fn waiting(&self) -> Vec<At> {
        let tiles = self.areas.iter().enumerate().flat_map(|(p, area)| {
            area.tiles()
                .iter()
                .enumerate()
                .filter(|(_, tile)| tile.slot().pending().is_some())
                .map(move |(t, _)| At::Tile(p, t))
        });
        let cells = (0..self.slots.len())
            .filter(|&c| self.slots[c].pending().is_some())
            .map(At::Cell);
        tiles.chain(cells).collect()
    }

    /// Applies the worker's results in arrival order, each reported to
    /// `probe`. True when any was shown, so the widget redraws.
    pub(crate) fn drain(&mut self, probe: Option<&Probe>) -> bool {
        let Some(worker) = &self.worker else {
            return false;
        };
        let mut shown = false;
        for done in worker.drain() {
            shown |= self.land(done, false, probe);
        }
        shown
    }

    /// Plans a refresh whose partition cells are `cells` and plot areas
    /// `areas` (device rects and list px), whose axes show the views
    /// `views` (per area): rebuilds the slots and the areas when their
    /// rects differ (every slot and tile new and changed); else moves each
    /// area to its views (a new grid when they moved) and finds the cells
    /// outside plot areas whose content hash differs from their slot's, and
    /// the areas whose content (the hash of the cells inside them) did.
    /// With `caching` off, every cell and tile when any changed; every one
    /// when `revision` is not the one the slots were planned at. If any
    /// changed, bumps the generation, stores the new hashes and marks the
    /// changed cells and tiles pending. Returns them, tiles first.
    fn plan(
        &mut self,
        cells: &[list::Cell],
        (rects, lists): (&[[i32; 4]], &[kurbo::Rect]),
        views: &[[Option<Basis>; 2]],
        (caching, revision): (Caching, u64),
    ) -> Vec<At> {
        let cell_rects: Vec<[i32; 4]> = cells.iter().map(|cell| slot::device(cell.rect)).collect();
        let owner: Vec<Option<usize>> = cell_rects
            .iter()
            .map(|r| rects.iter().position(|a| tile::intersect(*r, *a) == *r))
            .collect();
        let hashes: Vec<u64> = (0..rects.len())
            .map(|p| {
                let inside = cells.iter().zip(&owner).filter(|(_, o)| **o == Some(p));
                inside.fold(FNV_BASIS, |hash, (cell, _)| fnv(hash, cell.hash))
            })
            .collect();
        let same = self.slots.len() == cell_rects.len()
            && self
                .slots
                .iter()
                .zip(&cell_rects)
                .all(|(s, r)| s.rect() == *r)
            && self.areas.len() == rects.len()
            && self.areas.iter().zip(rects).all(|(a, r)| a.rect() == *r);
        let mut changed: Vec<usize> = if same {
            (0..cells.len())
                .filter(|&c| owner[c].is_none() && self.slots[c].hash() != cells[c].hash)
                .collect()
        } else {
            self.slots = cell_rects
                .iter()
                .zip(cells)
                .map(|(&rect, cell)| Slot::new(rect, cell.hash))
                .collect();
            self.owner = owner;
            self.areas = (0..rects.len())
                .map(|p| {
                    self.grids += 1;
                    let index = u32::try_from(p).expect("invariant: fewer than 2^32 plot areas");
                    Area::new(index, (rects[p], lists[p]), hashes[p], self.grids, views[p])
                })
                .collect();
            self.floor = self.generation + 1;
            (0..cells.len())
                .filter(|&c| self.owner[c].is_none())
                .collect()
        };
        let mut dirty = !same;
        let mut grids = self.grids;
        for (p, area) in self.areas.iter_mut().enumerate() {
            if same && area.follow(views[p], grids + 1) {
                grids += 1;
            }
            dirty |= area.is_stale(hashes[p]);
        }
        self.grids = grids;
        dirty |= !changed.is_empty();
        let every = match caching {
            Caching::On => revision != self.revision,
            Caching::Off => revision != self.revision || dirty,
        };
        self.revision = revision;
        if every {
            changed = (0..cells.len())
                .filter(|&c| self.owner[c].is_none())
                .collect();
        }
        if !(dirty || every) {
            return Vec::new();
        }
        self.generation += 1;
        let generation = self.generation;
        for &c in &changed {
            self.slots[c].change(cells[c].hash, generation);
        }
        let mut plan = Vec::new();
        for (p, area) in self.areas.iter_mut().enumerate() {
            area.change(hashes[p], generation, every);
            plan.extend(
                area.tiles()
                    .iter()
                    .enumerate()
                    .filter(|(_, tile)| tile.slot().pending() == Some(generation))
                    .map(|(t, _)| At::Tile(p, t)),
            );
        }
        plan.extend(changed.into_iter().map(At::Cell));
        plan
    }

    /// Applies a rendered job, reported to `probe` as rendered `inline` or
    /// not: shown when it belongs to this partition and is newer than its
    /// slot's image. True when it is shown.
    fn land(&mut self, done: Done, inline: bool, probe: Option<&Probe>) -> bool {
        if let Some(duration) = done.duration {
            self.budget
                .calibrate(duration, cost::model_ms(done.pixels(), done.work));
        }
        if let Some(probe) = probe {
            probe.send(Sample::Raster {
                frame: self.frame,
                generation: done.generation,
                target: done.target,
                pixels: u32::try_from(done.pixels()).unwrap_or(u32::MAX),
                work: done.work,
                queued: done.queued.unwrap_or_default(),
                duration: done.duration.unwrap_or_default(),
                inline,
            });
        }
        if done.generation < self.floor {
            return false;
        }
        let bands: Result<Vec<Band>, raster::Error> = done
            .rows
            .into_iter()
            .zip(done.bands)
            .map(|(rows, band)| band.map(|image| Band::new(rows, handle::from_image(image))))
            .collect();
        // A cell renders unless its list fails `list::check`.
        debug_assert!(
            bands.is_ok(),
            "a cell failed to render: {:?}",
            bands.as_ref().err()
        );
        match done.target {
            probe::Target::Cell(index) => {
                let slot = usize::try_from(index)
                    .ok()
                    .and_then(|index| self.slots.get_mut(index));
                slot.is_some_and(|slot| {
                    slot.land(done.generation, done.hash, bands.ok(), done.view)
                })
            }
            probe::Target::Tile { area, i, j } => {
                let area = usize::try_from(area)
                    .ok()
                    .and_then(|area| self.areas.get_mut(area));
                area.is_some_and(|area| {
                    let at = (done.grid, i, j);
                    area.land(at, (done.generation, done.hash), bands.ok(), done.view)
                })
            }
        }
    }

    /// What the following draw shows; reports each newly shown image's
    /// bytes once.
    fn report(&mut self, at: Instant) -> Sample {
        let gutters = || {
            self.slots
                .iter()
                .zip(&self.owner)
                .filter(|(_, owner)| owner.is_none())
                .map(|(slot, _)| slot)
        };
        let count = |test: fn(&Slot) -> bool| {
            u32::try_from(gutters().filter(|s| test(s)).count()).unwrap_or(u32::MAX)
        };
        let (mut missing, mut previews) = (count(Slot::is_missing), count(Slot::is_preview));
        let mut images = u32::try_from(gutters().count()).unwrap_or(u32::MAX) - missing;
        if let Some(factor) = self.factor {
            for (p, area) in self.areas.iter().enumerate() {
                let shown = area.shown(factor, self.peek(p));
                images += shown.images;
                missing += shown.missing;
                previews += shown.previews;
            }
        }
        let gutters: u64 = self.slots.iter_mut().map(Slot::take_fresh).sum();
        let tiles: u64 = self.areas.iter_mut().map(Area::take_fresh).sum();
        Sample::Frame {
            frame: self.frame,
            at,
            generation: self.generation,
            images,
            missing,
            previews,
            uploaded: gutters + tiles,
        }
    }

    /// The pointer moved to `at` (logical, in the window; `None` when it
    /// left or something above covers it) over a widget laid out at
    /// `bounds`: hovers there as [`hover`](State::hover) does, and says
    /// whether the widget should redraw; no cell renders.
    pub(crate) fn moved(
        &mut self,
        at: Option<Point>,
        bounds: Rectangle,
        shown: overlay::Shown,
    ) -> bool {
        self.pointer = at;
        self.hover(bounds, shown)
    }

    /// Hit-tests the pointer's last position over a widget laid out at
    /// `bounds` with the cached index: the panel whose plot area holds it,
    /// and that panel's drawn point nearest it within 12 logical px, for
    /// the overlay `shown` asks for and for `on_hover`. While a gesture is
    /// active nothing is hit-tested: the overlay shows no ring and nothing
    /// changes for `on_hover`. True when the overlay or the hovered point
    /// changed.
    pub(crate) fn hover(&mut self, bounds: Rectangle, shown: overlay::Shown) -> bool {
        let Some(factor) = self.factor else {
            return false;
        };
        let busy = self.gesture.is_some() || self.is_brushing();
        let [x0, y0] = origin(bounds, factor);
        let k = f64::from(factor);
        let mut hover = None;
        let next = match self.pointer {
            None => Overlay::default(),
            Some(at) => {
                let device = Point::new(at.x * factor - x0 as f32, at.y * factor - y0 as f32);
                let inside = |&[x0, y0, x1, y1]: &[i32; 4]| {
                    (x0 as f32..x1 as f32).contains(&device.x)
                        && (y0 as f32..y1 as f32).contains(&device.y)
                };
                let panel = self.areas.iter().map(Area::rect).position(|r| inside(&r));
                let near = kurbo::Point::new(f64::from(device.x) / k, f64::from(device.y) / k);
                let hit = panel
                    .filter(|_| !busy)
                    .and_then(|panel| self.hits.nearest(near, panel));
                hover = hit.map(|hit| Hover {
                    panel: hit.panel,
                    mark: hit.mark,
                    index: hit.index,
                    at: Point::new(at.x - bounds.x, at.y - bounds.y),
                });
                let point = hit.map(|hit| Point::new((hit.at.x * k) as f32, (hit.at.y * k) as f32));
                Overlay::new(device, panel.map(|i| self.areas[i].rect()), point, shown)
            }
        };
        let pending = !busy && self.hits.set(hover);
        let changed = next != self.overlay;
        self.overlay = next;
        changed || pending
    }

    /// What `on_hover` hears after this frame, once per change of the
    /// hovered point (`Some(None)` once when it was lost); nothing while a
    /// gesture is active.
    pub(crate) fn reported(&mut self) -> Option<Option<Hover>> {
        if self.gesture.is_some() || self.is_brushing() {
            return None;
        }
        self.hits.report()
    }

    /// Draws every image of a cell outside the plot areas (the theme
    /// background where a cell has none), each plot area's tiles over its
    /// preview layer mapped from the views it was planned at to the views
    /// now, then each brush's selection and the hover overlay in the
    /// theme's ink, for a widget laid out at `bounds`.
    pub(crate) fn draw<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle, theme: &Theme)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let Some(factor) = self.factor else {
            return;
        };
        let origin = origin(bounds, factor);
        let background = overlay::color(theme.get_background());
        for (slot, _) in self
            .slots
            .iter()
            .zip(&self.owner)
            .filter(|(_, o)| o.is_none())
        {
            slot.draw(renderer, origin, factor, background);
        }
        for (p, area) in self.areas.iter().enumerate() {
            area.draw(renderer, (origin, factor), self.peek(p), background);
        }
        for brush in &self.brushes {
            brush.draw(renderer, origin, factor, theme.get_ink());
        }
        self.overlay.draw(renderer, origin, factor, theme.get_ink());
    }

    /// The cursor over a widget laid out at `bounds` with the pointer at
    /// `at` (logical, in the window): a grabbing hand during a drag, a
    /// crosshair during a brush gesture and over a brushed plot area (d3's
    /// overlay cursor), a grab hand over a plot area with zoomable handles,
    /// the resize arrow of the axis's direction over a zoomable axis, a
    /// crosshair over other plot areas, none elsewhere.
    pub(crate) fn interaction(&self, at: Option<Point>, bounds: Rectangle) -> mouse::Interaction {
        if self.is_dragging() {
            return mouse::Interaction::Grabbing;
        }
        if self.is_brushing() {
            return mouse::Interaction::Crosshair;
        }
        let Some(at) = at else {
            return mouse::Interaction::None;
        };
        let at = kurbo::Point::new(f64::from(at.x - bounds.x), f64::from(at.y - bounds.y));
        let region = self.map.at(at);
        let zoomable = !self.map.tracks(region).is_empty();
        let brushed = |panel: usize| self.brushes.iter().any(|brush| brush.panel() == panel);
        match region {
            Region::Area { panel } if brushed(panel) => mouse::Interaction::Crosshair,
            Region::Area { .. } if zoomable => mouse::Interaction::Grab,
            Region::Area { .. } => mouse::Interaction::Crosshair,
            Region::Axis {
                along: Along::X, ..
            } if zoomable => mouse::Interaction::ResizingHorizontally,
            Region::Axis {
                along: Along::Y, ..
            } if zoomable => mouse::Interaction::ResizingVertically,
            Region::Axis { .. } | Region::Outside => mouse::Interaction::None,
        }
    }
}

/// A job's slot: a partition cell, or tile `t` of plot area `p`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Cell(usize),
    Tile(usize, usize),
}

/// FNV-1a 64's offset basis.
const FNV_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// `hash` with the eight bytes of `word` folded in, FNV-1a 64.
fn fnv(hash: u64, word: u64) -> u64 {
    word.to_le_bytes().iter().fold(hash, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// A plot area's rect in list px as device px at `k`, each edge rounded.
fn device(area: kurbo::Rect, k: f64) -> [i32; 4] {
    [area.x0, area.y0, area.x1, area.y1].map(|edge| (edge * k).round() as i32)
}

/// The widget's device origin: its layout position times the factor,
/// rounded, so that a widget at a fractional logical offset still puts
/// every cell on whole device pixels.
fn origin(bounds: Rectangle, factor: f32) -> [i32; 2] {
    [bounds.x, bounds.y].map(|v| (v * factor).round() as i32)
}

/// A figure cache over the bundled fonts.
#[cfg(feature = "fonts")]
pub(super) fn cache() -> Cache {
    Cache::default()
}

/// Without the `fonts` feature, a figure cache over the fonts
/// rustd3-figure has, which may be none: labels then draw no glyphs.
#[cfg(not(feature = "fonts"))]
pub(super) fn cache() -> Cache {
    let fonts = rustd3_figure::text::Fonts::with(Vec::<Vec<u8>>::new())
        .expect("invariant: no font files, no font error");
    Cache::new(std::sync::Arc::new(fonts))
}

#[cfg(test)]
mod gestures;

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, mpsc};

    use iced_core::shell::Waker;
    use iced_core::time::Instant;
    use iced_core::{Point, Rectangle, Size};
    use rustd3_figure::{Figure, Held, Target, kurbo};

    use super::{Pipeline, State};
    use crate::plot::area::Area;
    use crate::plot::overlay::{Overlay, Shown};
    use crate::plot::probe::{self, Probe, Sample, Stage};
    use crate::plot::slot::Slot;
    use crate::plot::tile::Tile;
    use crate::plot::{Caching, Render, fixture};

    /// The factors layout, snapping and placement are checked at.
    const FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

    #[expect(clippy::disallowed_methods, reason = "test clock")]
    fn clock() -> Instant {
        Instant::now()
    }

    /// A state laid out at `width` × `height` logical px.
    fn state(width: f32, height: f32) -> State {
        let mut state = State::default();
        state.resize(Size::new(width, height));
        state
    }

    /// One inline `RedrawRequested` without a probe; the number of cells
    /// whose content changed.
    fn refresh(state: &mut State, fig: &Figure<'_>, k: f32) -> usize {
        let pipeline = Pipeline {
            render: Render::Inline,
            caching: Caching::On,
            revision: 0,
            probe: None,
            waker: &Waker::noop(),
        };
        state.redraw::<()>(fig, k, clock(), &pipeline, (&[], &[])).0
    }

    /// One inline `RedrawRequested` reported to a fresh probe; its
    /// samples.
    fn probed(state: &mut State, fig: &Figure<'_>, k: f32) -> Vec<Sample> {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let pipeline = Pipeline {
            render: Render::Inline,
            caching: Caching::On,
            revision: 0,
            probe: Some(&probe),
            waker: &Waker::noop(),
        };
        state.redraw::<()>(fig, k, clock(), &pipeline, (&[], &[]));
        rx.try_iter().collect()
    }

    /// A waker that counts its wakes.
    fn counting() -> (Waker, Arc<AtomicUsize>) {
        let wakes = Arc::new(AtomicUsize::new(0));
        let waker = Waker::new({
            let wakes = Arc::clone(&wakes);
            move || {
                wakes.fetch_add(1, Ordering::SeqCst);
            }
        });
        (waker, wakes)
    }

    /// Drains the worker's results into `state` after each wake, reported
    /// to `probe`, until no slot waits for a render; at most ten seconds
    /// per wake.
    fn settle(state: &mut State, wakes: &AtomicUsize, probe: Option<&Probe>) {
        let mut seen = 0;
        while !state.waiting().is_empty() {
            let woke = (0..10_000).any(|_| {
                wakes.load(Ordering::SeqCst) > seen || {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    false
                }
            });
            assert!(woke, "no wake within ten seconds");
            seen = wakes.load(Ordering::SeqCst);
            state.drain(probe);
        }
    }

    /// The targets of the raster samples among `samples`.
    fn rasters(samples: &[Sample]) -> Vec<probe::Target> {
        samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Raster { target, .. } => Some(*target),
                Sample::Stage { .. } | Sample::Frame { .. } => None,
            })
            .collect()
    }

    /// The device rects of what shows an image of its own: the cells
    /// outside plot areas, then the plot areas' tiles.
    fn shown(state: &State) -> Vec<[i32; 4]> {
        let gutters = state
            .slots
            .iter()
            .zip(&state.owner)
            .filter(|(_, owner)| owner.is_none())
            .map(|(slot, _)| slot.rect());
        let tiles = state
            .areas
            .iter()
            .flat_map(|area| area.tiles().iter().map(|tile| tile.slot().rect()));
        gutters.chain(tiles).collect()
    }

    /// Every target of `state`, as one refresh that renders everything
    /// reports them: the tiles area by area, then the cells outside plot
    /// areas.
    fn targets(state: &State) -> Vec<probe::Target> {
        let tiles = state.areas.iter().flat_map(|area| {
            area.tiles().iter().map(|tile| {
                let key = tile.key();
                probe::Target::Tile {
                    area: key.area,
                    i: key.i,
                    j: key.j,
                }
            })
        });
        let cells = (0..state.slots.len())
            .filter(|&c| state.owner[c].is_none())
            .map(|c| probe::Target::Cell(u32::try_from(c).unwrap()));
        tiles.chain(cells).collect()
    }

    #[test]
    fn cells_partition_the_canvas() {
        let fig = fixture::grid();
        for k in FACTORS {
            let mut state = state(640.0, 480.0);
            let rendered = refresh(&mut state, &fig, k);
            let canvas = [0, 0, (640.0 * k).round() as i32, (480.0 * k).round() as i32];
            let cells: Vec<[i32; 4]> = state.slots.iter().map(|slot| slot.rect()).collect();
            // Four plot areas cut each axis into at most nine tracks.
            assert!(cells.len() > 4, "k = {k}: {} cells", cells.len());
            // The cells outside plot areas and the plot areas' tiles render,
            // and partition the canvas as the cells do.
            let rects = shown(&state);
            assert_eq!(rendered, rects.len());
            let area = |[x0, y0, x1, y1]: [i32; 4]| i64::from(x1 - x0) * i64::from(y1 - y0);
            assert_eq!(cells.iter().copied().map(area).sum::<i64>(), area(canvas));
            for r in &rects {
                assert!(r[0] < r[2] && r[1] < r[3], "k = {k}: empty cell {r:?}");
                assert!(
                    r[0] >= canvas[0]
                        && r[1] >= canvas[1]
                        && r[2] <= canvas[2]
                        && r[3] <= canvas[3],
                    "k = {k}: {r:?} outside {canvas:?}"
                );
            }
            for (i, a) in rects.iter().enumerate() {
                for b in &rects[i + 1..] {
                    let overlap =
                        a[0].max(b[0]) < a[2].min(b[2]) && a[1].max(b[1]) < a[3].min(b[3]);
                    assert!(!overlap, "k = {k}: {a:?} and {b:?} overlap");
                }
            }
            // Disjoint and inside the canvas, they cover it when their
            // areas add up to its area.
            let covered: i64 = rects.iter().copied().map(area).sum();
            assert_eq!(covered, area(canvas), "k = {k}");
        }
    }

    #[test]
    fn rebuild_of_an_equal_figure_renders_nothing() {
        let mut state = state(640.0, 480.0);
        let first = probed(&mut state, &fixture::grid(), 1.25);
        assert_eq!(rasters(&first), targets(&state));
        let images = |state: &State| -> Vec<_> {
            let slots = state.slots.iter().map(Slot::images);
            let tiles = state
                .areas
                .iter()
                .flat_map(|area| area.tiles().iter().map(|tile| tile.slot().images()));
            slots.chain(tiles).collect()
        };
        let shown = images(&state);
        // The view is rebuilt: a new figure, equal to the last.
        state.rebuilt();
        let again = probed(&mut state, &fixture::grid(), 1.25);
        assert!(rasters(&again).is_empty(), "{again:?}");
        assert_eq!(images(&state), shown);
        assert_eq!(state.generation, 1);
        // Nothing rebuilt, resized or rescaled: no refresh at all.
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.25), 0);
        // A new size renders every cell again.
        state.resize(Size::new(600.0, 480.0));
        assert!(refresh(&mut state, &fixture::grid(), 1.25) > 0);
        assert_eq!(state.generation, 2);
    }

    #[test]
    fn unchanged_cells_are_skipped() {
        let dots: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
        let mut moved = dots.clone();
        moved[3] = (4.0, 6.5);
        let mut state = state(480.0, 320.0);
        let first = probed(&mut state, &fixture::pair(dots), 2.0);
        assert_eq!(rasters(&first), targets(&state));
        state.rebuilt();
        let again = probed(&mut state, &fixture::pair(moved), 2.0);
        // Only the dots panel's tiles render.
        let tiles: Vec<probe::Target> = targets(&state)
            .into_iter()
            .filter(|target| matches!(target, probe::Target::Tile { area: 1, .. }))
            .collect();
        assert!(!tiles.is_empty());
        assert_eq!(rasters(&again), tiles);
        assert_eq!(state.generation, 2);
        for (c, slot) in state.slots.iter().enumerate() {
            let expected = state.owner[c].is_none().then_some(1);
            assert_eq!(slot.generation(), expected, "slot {c}");
        }
        for (p, area) in state.areas.iter().enumerate() {
            for tile in area.tiles() {
                let expected = if p == 1 { 2 } else { 1 };
                assert_eq!(tile.slot().generation(), Some(expected), "area {p}");
            }
        }
    }

    #[test]
    fn probe_reports_every_stage() {
        let mut state = state(640.0, 480.0);
        let samples = probed(&mut state, &fixture::grid(), 2.0);
        let stages: Vec<Stage> = samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Stage {
                    frame: 1, stage, ..
                } => Some(*stage),
                Sample::Stage { .. } | Sample::Raster { .. } | Sample::Frame { .. } => None,
            })
            .collect();
        assert_eq!(stages, [Stage::Resolve, Stage::List]);
        let cells = rasters(&samples);
        let expected = targets(&state);
        assert_eq!(
            cells, expected,
            "one raster sample per tile and cell, in order"
        );
        let (mut pixels, mut work, mut ms) = (0, 0, 0.0);
        for sample in &samples {
            if let Sample::Raster {
                frame,
                generation,
                pixels: p,
                work: w,
                duration,
                inline,
                ..
            } = sample
            {
                assert_eq!((*frame, *generation, *inline), (1, 1, true));
                pixels += u64::from(*p);
                work += *w;
                ms += duration.as_secs_f64() * 1e3;
            }
        }
        assert_eq!(pixels, 1280 * 960);
        let frames: Vec<_> = samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Frame {
                    frame,
                    generation,
                    images,
                    missing,
                    previews,
                    uploaded,
                    ..
                } => Some((*frame, *generation, *images, *missing, *previews, *uploaded)),
                Sample::Stage { .. } | Sample::Raster { .. } => None,
            })
            .collect();
        let slots = u32::try_from(expected.len()).unwrap();
        assert_eq!(frames, [(1, 1, slots, 0, 0, 1280 * 960 * 4)]);
        assert!(matches!(samples.last(), Some(Sample::Frame { .. })));
        println!(
            "inline: {} samples ({} rasters); {pixels} px and {work} items in {ms:.1} ms: {:.0} px/ms",
            samples.len(),
            cells.len(),
            pixels as f64 / ms
        );
        // A redraw that refreshes nothing still reports its frame.
        let idle = probed(&mut state, &fixture::grid(), 2.0);
        assert!(
            matches!(
                idle.as_slice(),
                [Sample::Frame {
                    frame: 2,
                    uploaded: 0,
                    ..
                }]
            ),
            "{idle:?}"
        );
        // The same refresh on the worker: the frame shows no image, and
        // every cell reports when its result is drained.
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let (waker, wakes) = counting();
        let pipeline = Pipeline {
            render: Render::Worker,
            caching: Caching::On,
            revision: 0,
            probe: Some(&probe),
            waker: &waker,
        };
        let mut state = self::state(640.0, 480.0);
        state.redraw::<()>(&fixture::grid(), 2.0, clock(), &pipeline, (&[], &[]));
        let planned: Vec<Sample> = rx.try_iter().collect();
        let stages = planned
            .iter()
            .filter(|sample| matches!(sample, Sample::Stage { .. }))
            .count();
        assert_eq!(stages, 2);
        assert!(rasters(&planned).is_empty());
        assert!(
            matches!(
                planned.last(),
                Some(Sample::Frame { frame: 1, generation: 1, images: 0, missing, previews: 0, uploaded: 0, .. })
                    if *missing == slots
            ),
            "{planned:?}"
        );
        settle(&mut state, &wakes, Some(&probe));
        let drained: Vec<Sample> = rx.try_iter().collect();
        // One worker renders the set in order: tiles, then cells.
        assert_eq!(
            rasters(&drained),
            expected,
            "one raster sample per tile and cell"
        );
        let (mut pixels, mut ms) = (0, 0.0);
        for sample in &drained {
            if let Sample::Raster {
                frame,
                inline,
                pixels: p,
                duration,
                ..
            } = sample
            {
                assert_eq!((*frame, *inline), (1, false));
                pixels += u64::from(*p);
                ms += duration.as_secs_f64() * 1e3;
            }
        }
        println!(
            "worker: {} samples after {} wakes; {pixels} px in {ms:.1} ms: {:.0} px/ms",
            drained.len(),
            wakes.load(Ordering::SeqCst),
            pixels as f64 / ms
        );
        state.redraw::<()>(&fixture::grid(), 2.0, clock(), &pipeline, (&[], &[]));
        let shown: Vec<Sample> = rx.try_iter().collect();
        assert!(
            matches!(
                shown.as_slice(),
                [Sample::Frame { frame: 2, generation: 1, images, missing: 0, previews: 0, uploaded, .. }]
                    if *images == slots && *uploaded == 1280 * 960 * 4
            ),
            "{shown:?}"
        );
    }

    #[test]
    fn every_slot_lands_after_a_replaced_set() {
        // A set replaced before it finished: the slots it would have
        // rendered render in the new set.
        let dots: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
        let mut moved = dots.clone();
        moved[3] = (4.0, 6.5);
        let (waker, wakes) = counting();
        let pipeline = Pipeline {
            render: Render::Worker,
            caching: Caching::On,
            revision: 0,
            probe: None,
            waker: &waker,
        };
        let mut state = state(480.0, 320.0);
        state.redraw::<()>(&fixture::pair(dots), 2.0, clock(), &pipeline, (&[], &[]));
        state.rebuilt();
        state.redraw::<()>(&fixture::pair(moved), 2.0, clock(), &pipeline, (&[], &[]));
        assert_eq!(state.generation, 2);
        settle(&mut state, &wakes, None);
        let gutters = state
            .slots
            .iter()
            .zip(&state.owner)
            .filter(|(_, o)| o.is_none());
        assert!(gutters.into_iter().all(|(slot, _)| slot.is_current()));
        let tiles = state.areas.iter().flat_map(Area::tiles);
        assert!(tiles.into_iter().all(|tile| tile.slot().is_current()));
    }

    #[test]
    fn a_refresh_resolves_the_figure_once() {
        // One resolve of the grid over a warm cache: every label a hit.
        let fig = fixture::grid();
        let mut cache = super::cache();
        let (size, target) = (
            kurbo::Size::new(640.0, 480.0),
            Target::Screen { scale: 1.5 },
        );
        cache.begin();
        let _ = fig.resolve(size, &mut cache, &Held::default(), target);
        let warm = cache.stats().text.hits;
        cache.begin();
        let _ = fig.resolve(size, &mut cache, &Held::default(), target);
        let one = cache.stats().text.hits - warm;
        assert!(one > 0);
        let mut state = state(640.0, 480.0);
        refresh(&mut state, &fig, 1.5);
        let before = state.cache.stats().text.hits;
        state.rebuilt();
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.5), 0);
        assert_eq!(state.cache.stats().text.hits - before, one);
    }

    #[test]
    fn cell_images_have_their_device_size() {
        let fig = fixture::grid();
        for k in [1.25, 1.5] {
            let mut state = state(640.0, 480.0);
            refresh(&mut state, &fig, k);
            let gutters = state
                .slots
                .iter()
                .zip(&state.owner)
                .filter(|(_, o)| o.is_none());
            let tiles = state.areas.iter().flat_map(Area::tiles).map(Tile::slot);
            for slot in gutters.map(|(slot, _)| slot).chain(tiles) {
                let [x0, y0, x1, y1] = slot.rect();
                let images = slot.images();
                assert_eq!(images.len(), 1, "k = {k}: every cell is one band");
                let (rows, width, height, rgba) = &images[0];
                assert_eq!(*rows, 0..y1 - y0);
                assert_eq!(
                    (*width as i32, *height as i32),
                    (x1 - x0, y1 - y0),
                    "k = {k}"
                );
                assert_eq!(rgba.len(), (*width * *height * 4) as usize);
            }
        }
    }

    #[test]
    fn hover_renders_no_cell() {
        let fig = fixture::grid();
        let mut state = state(640.0, 480.0);
        assert!(refresh(&mut state, &fig, 1.25) > 0);
        let shown = Shown {
            crosshair: true,
            highlight: true,
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(640.0, 480.0));
        let [x0, y0, x1, y1] = state.areas[0].rect();
        let centre = Point::new((x0 + x1) as f32 / 2.5, (y0 + y1) as f32 / 2.5);
        assert!(state.moved(Some(centre), bounds, shown));
        assert!(!state.moved(Some(centre), bounds, shown), "the same place");
        // The redraw the hover asks for renders nothing, and neither does
        // the rebuild an app's message would cause.
        assert_eq!(refresh(&mut state, &fig, 1.25), 0);
        state.rebuilt();
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.25), 0);
        assert!(state.moved(None, bounds, shown), "the pointer left");
        assert_eq!(refresh(&mut state, &fig, 1.25), 0);
    }

    #[test]
    fn hovering_outside_every_plot_area_shows_nothing() {
        let mut state = state(640.0, 480.0);
        refresh(&mut state, &fixture::grid(), 1.0);
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(640.0, 480.0));
        let shown = Shown {
            crosshair: true,
            highlight: true,
        };
        assert!(!state.moved(Some(Point::new(1.0, 1.0)), bounds, shown));
        assert_eq!(state.overlay, Overlay::default());
        // With nothing shown, moving inside a plot area changes no part
        // (what is hovered may change, for `on_hover`).
        let none = Shown {
            crosshair: false,
            highlight: false,
        };
        let [x0, y0, ..] = state.areas[0].rect();
        let inside = Point::new(x0 as f32 + 3.0, y0 as f32 + 3.0);
        assert!(state.moved(Some(inside), bounds, none), "the panel changed");
        let before = state.overlay.clone();
        let further = Point::new(x0 as f32 + 9.0, y0 as f32 + 9.0);
        state.moved(Some(further), bounds, none);
        assert_eq!(state.overlay, before);
    }

    #[test]
    fn a_widget_of_no_device_pixel_has_no_cells() {
        let mut state = state(0.0, 480.0);
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.0), 0);
        assert!(state.slots.is_empty());
    }
}
