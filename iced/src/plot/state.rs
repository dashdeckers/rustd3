//! [`State`]: what the widget keeps in iced's widget tree between frames,
//! and the refresh that renders the cells whose content changed.

use std::cell::Cell;
use std::sync::Arc;

use iced_core::shell::Waker;
use iced_core::time::Instant;
use iced_core::{Event, Point, Rectangle, Size, image, mouse};
use rustd3_core::interact::{self, BrushEvent, Button, Modifiers, Pointer};
use rustd3_figure::list::{self, Command, List, Outlines};
use rustd3_figure::scale::Scale;
use rustd3_figure::{Cache, Figure, Resolved, Target, Theme, hit, kurbo, raster, thin};

use crate::plot::area::{Area, Moved};
use crate::plot::brush::{self, Brush};
use crate::plot::budget::Budget;
use crate::plot::domains::{Domains, Key};
use crate::plot::gesture::{self, Gesture, Track};
use crate::plot::hover::{self, Hover};
use crate::plot::input::Input;
use crate::plot::job::{Done, Job, Spot, Stamp};
use crate::plot::layer::{self, Layers};
use crate::plot::listener::Listener;
use crate::plot::merge::{self, Run};
use crate::plot::overlay::{self, Overlay};
use crate::plot::preview::Basis;
use crate::plot::probe::{self, Probe, Sample, Stage};
use crate::plot::region::{self, Along, Region};
use crate::plot::slot::{self, Band, Slot};
use crate::plot::tile::{self, Tile};
use crate::plot::worker::Pool;
use crate::plot::{Caching, Render, handle};
use crate::pointer::Tracker;

/// The widget's tree state: the figure's cache, the rasterizer, and the
/// slots of the partition's cells. It borrows nothing of the figure.
pub(crate) struct State {
    /// Laid-out labels, thinned lines and `Fit::Visible` memory.
    cache: Cache,
    /// The renderer of what renders inline, single-threaded whatever the
    /// pool's size: vello_cpu's own pool costs about 4 ms a render on two
    /// cores whatever its size, and the pool is the widget's only
    /// parallelism.
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
    /// The partition's cells, row-major: their device rects.
    cells: Vec<[i32; 4]>,
    /// Per partition cell, the plot area that holds it, if one does.
    owner: Vec<Option<usize>>,
    /// The partition's cells outside the plot areas, merged into runs of
    /// one slot each; cells inside a plot area show nothing of their own
    /// (its tiles do).
    gutters: Vec<Run>,
    /// Gutter images of an earlier layout: each stays drawn under the runs
    /// that meet it until every one of them shows an image of its own.
    former: Vec<Slot>,
    /// What the last repaint drew from, which every render reads.
    drawn: Option<Drawn>,
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
    /// Whether the hit index is not of the list shown: built only for a
    /// widget that hovers, and not while a gesture lasts.
    unhit: bool,
    /// The pointer where the last move left it, logical px in the window;
    /// `None` when it left or something above covers it.
    pointer: Option<Point>,
    /// The crosshair and highlight where the pointer is.
    overlay: Overlay,
    /// The render pool, started with the first jobs it gets.
    pool: Option<Pool>,
    /// Where the last gesture input was, list px: the pool renders the
    /// tiles nearest it first.
    focus: Option<kurbo::Point>,
    /// Whether tiles of the overscan ring wait for the gesture in progress
    /// to end: the first refresh after it renders them.
    deferred: bool,
    /// Whether the last draw grew its layer by half a px: draws alternate
    /// while a gesture lasts ([`draw`](State::draw)).
    grown: Cell<bool>,
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
    /// Wakes the UI when the pool has results; each of its threads keeps
    /// a clone.
    pub(crate) waker: &'a Waker,
    /// The render pool's size.
    pub(crate) threads: usize,
    /// Whether anything reads the hit index: the highlight or `on_hover`.
    pub(crate) hover: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            cache: cache(),
            raster: raster::Renderer::new(raster::Settings::default()),
            outlines: Outlines::default(),
            size: Size::ZERO,
            window: None,
            factor: None,
            stale: true,
            cells: Vec::new(),
            owner: Vec::new(),
            gutters: Vec::new(),
            former: Vec::new(),
            drawn: None,
            generation: 0,
            floor: 0,
            frame: 0,
            budget: Budget::default(),
            areas: Vec::new(),
            grids: 0,
            hits: hover::Hits::default(),
            unhit: false,
            pointer: None,
            overlay: Overlay::default(),
            pool: None,
            focus: None,
            deferred: false,
            grown: Cell::new(false),
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
        for pointer in &queued {
            if let Pointer::Wheel { x, y, .. }
            | Pointer::Pressed { x, y, .. }
            | Pointer::Moved { x, y } = *pointer
            {
                self.focus = Some(kurbo::Point::new(x, y));
            }
        }
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
            let gesture = Gesture::new(region, kind, tracks, held, area);
            self.gesture = Some(match (kind, self.factor) {
                (gesture::Kind::Drag, Some(factor)) => gesture.snap(f64::from(factor)),
                (gesture::Kind::Drag | gesture::Kind::Wheel { .. }, _) => gesture,
            });
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
        self.keep(steps, fresh, listeners)
    }

    /// Stores each of `steps`, published (pending) for a handle one of
    /// `listeners` listens to, else owned; the keys go to `fresh`. True
    /// when there was one.
    fn keep<Message>(
        &mut self,
        steps: Vec<gesture::Step>,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
    ) -> bool {
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

    /// Ends the gesture in progress, if any: a drag's unsnapped transforms
    /// apply once, what it published goes out, published domains are the
    /// app's from now on, the layout is held no more and settles at the
    /// next refresh.
    fn end<Message>(
        &mut self,
        fresh: &mut Vec<Key>,
        listeners: &[Listener<'_, Message>],
        messages: &mut Vec<Message>,
    ) {
        if let Some(mut gesture) = self.gesture.take() {
            self.keep(gesture.exact(), fresh, listeners);
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

    /// Whether a gesture is in progress: a drag, a wheel gesture until its
    /// end or a brush gesture. Its refreshes render only the tiles that
    /// meet the plot areas; the ring waits for it to end.
    fn is_active(&self) -> bool {
        self.gesture.is_some() || self.is_brushing()
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
            let Some(cell) = resolved.panel_cell(panel) else {
                continue;
            };
            let target = placed.target().and_then(|target| {
                let key = Key::new(target, resolved.scale_at(target)?);
                self.domains.current(key, resolved)
            });
            let spot = brush::Spot {
                panel,
                cell,
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
    /// Otherwise first moves each plot area's tile grid to the views now
    /// (by whole device pixels during a drag), resolves with an overscan of
    /// two tiles and, during a drag, the column grids of the moved areas,
    /// keeps each translation only where the lead handles' scales moved as
    /// the grid did ([`confirm`](State::confirm)), and renders what changed (see
    /// [`repaint`](State::repaint)): inline, shown at once, or as jobs
    /// that replace the pool's waiting ones, as `pipeline.render` asks or,
    /// for `Auto`, as the cost model predicts against the frame budget.
    /// Returns how many cells, tiles and overlays changed. A new
    /// `pipeline.revision` refreshes as a rebuilt figure does.
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
        let rescaled = self.factor != Some(factor);
        self.stale = false;
        self.factor = Some(factor);
        let k = f64::from(factor);
        let size = kurbo::Size::new(f64::from(self.size.width), f64::from(self.size.height));
        if !((size.width * k).round() >= 1.0 && (size.height * k).round() >= 1.0) {
            self.cells.clear();
            self.owner.clear();
            self.gutters.clear();
            self.former.clear();
            self.drawn = None;
            self.areas.clear();
            self.hits.rebuild(hit::Index::default());
            self.map = region::Map::default();
            self.resolved = None;
            return (0, Vec::new());
        }
        let probe = pipeline.probe;
        let target = Target::Screen { scale: k };
        let start = probe.map(Probe::now);
        // The grids move before the list is drawn: a drag's lines thin on
        // the moved columns.
        let moves = if rescaled { Vec::new() } else { self.follow(k) };
        self.cache.begin();
        let mut held = self.domains.held();
        held.overscan(f64::from(tile::OVERSCAN) / k)
            .grids(self.thin_grids(k));
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
        let moves = self.confirm(moves, &resolved, k);
        let changed = self.repaint(figure, &resolved, (k, &moves), pipeline, start);
        for p in 0..self.areas.len() {
            let leads = self.leads(p, &resolved);
            self.areas[p].lead(leads.map(|lead| lead.map(|(key, scale)| (key, scale.boxed()))));
        }
        self.resolved = Some(resolved);
        (changed, Vec::new())
    }

    /// The two-point test of each area [`follow`](State::follow)
    /// translated by the views: the scales `resolved` gives its
    /// lead handles, against those its layer was last drawn with, must
    /// have moved by the grid's whole pixels at `k` at its edges. An area
    /// whose scales moved otherwise (a `Fit::Visible` y refitting during
    /// an x pan, a domain the app changed mid-drag) begins its grid anew
    /// where it lies, so every tile renders.
    fn confirm(&mut self, mut moves: Vec<Moved>, resolved: &Resolved, k: f64) -> Vec<Moved> {
        for (p, moved) in moves.iter_mut().enumerate() {
            let Moved::Translated(by) = *moved else {
                continue;
            };
            let now = self.leads(p, resolved);
            let views = self.peek(p);
            let Some(area) = self.areas.get_mut(p) else {
                continue;
            };
            if !area.keeps(now, by, k) {
                self.grids += 1;
                area.restart(self.grids, (k, views));
                *moved = Moved::Anew;
            }
        }
        moves
    }

    /// The scales `resolved` gives the handles leading each axis of
    /// `panel`, by handle.
    fn leads<'r>(&self, panel: usize, resolved: &'r Resolved) -> [Option<(Key, &'r dyn Scale)>; 2] {
        [Along::X, Along::Y].map(|along| {
            let key = self.map.lead(panel, along)?;
            Some((key, resolved.scale_at(key.index())?))
        })
    }

    /// Moves each plot area's grid to the views its axes show now at `k`;
    /// how each moved.
    fn follow(&mut self, k: f64) -> Vec<Moved> {
        let views: Vec<[Option<Basis>; 2]> = (0..self.areas.len()).map(|p| self.views(p)).collect();
        let mut grids = self.grids;
        let moves = self
            .areas
            .iter_mut()
            .zip(views)
            .map(|(area, now)| {
                let moved = area.follow(now, k, grids + 1);
                if moved == Moved::Anew {
                    grids += 1;
                }
                moved
            })
            .collect();
        self.grids = grids;
        moves
    }

    /// The column grids lines thin on: for each plot area, its x lead
    /// handle's columns from the tile grid's origin at one per device px
    /// (list px from the area's left edge at `k`: 0 until a drag moves the
    /// grid), so that every tile of one grid thins on the same columns and
    /// starts dashes at the same sample.
    fn thin_grids(&self, k: f64) -> Vec<(usize, thin::Grid)> {
        let mut grids: Vec<(usize, thin::Grid)> = Vec::new();
        for (p, area) in self.areas.iter().enumerate() {
            let moved = area.origin()[0] - area.rect()[0];
            let Some(key) = self.map.lead(p, Along::X) else {
                continue;
            };
            if grids.iter().all(|(index, _)| *index != key.index()) {
                let origin = f64::from(moved) / k;
                grids.push((key.index(), thin::Grid { origin, density: k }));
            }
        }
        grids
    }

    /// Draws `resolved` (of `figure`, the probe's clock read at `start`
    /// before resolving) into a list at `k` and splits it into the plot
    /// areas' layers and the chrome ([`Layers`]); cuts the chrome into
    /// cells at every plot-area edge; plans the gutter runs, the plot
    /// areas' tiles (their grids having moved as `moves` says) and the
    /// overlays of the chrome over the plot areas; and renders what
    /// changed ([`render`](State::render)): the visible tiles, then the
    /// gutters and overlays, then the ring of tiles around each area.
    /// Returns how many changed.
    fn repaint(
        &mut self,
        figure: &Figure<'_>,
        resolved: &Resolved,
        (k, moves): (f64, &[Moved]),
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
        let lists = resolved.plot_areas();
        let rects: Vec<[i32; 4]> = lists.iter().map(|area| device(*area, k)).collect();
        let layers = Layers::split(&list, rects.len(), &mut self.outlines);
        let (xs, ys) = resolved.edges();
        let cells = list::cells(&layers.chrome.list, &xs, &ys, k);
        let views: Vec<[Option<Basis>; 2]> = (0..rects.len()).map(|p| self.views(p)).collect();
        let overlays: Vec<Vec<([i32; 4], u64)>> = rects
            .iter()
            .map(|&rect| overlays(&layers.fixed.list, rect, k))
            .collect();
        let contents: Vec<(u64, u64)> = layers
            .layers
            .iter()
            .map(|layer| (layer.hash, layer.style))
            .collect();
        let plan = self.plan(
            Partition {
                cells: &cells,
                chrome: &layers.chrome.list,
                resolved,
            },
            Planned {
                k,
                rects: &rects,
                lists,
                views: &views,
                contents: &contents,
                overlays: &overlays,
                moves,
            },
            (pipeline.caching, pipeline.revision),
        );
        let verify = pipeline.caching == Caching::Verify;
        // A list whose cells all hash as before draws the same points. The
        // index is built only for a widget that hovers, and not while a
        // gesture lasts (it hit-tests nothing then): the first refresh
        // after builds it.
        self.unhit |= !plan.is_empty();
        if self.unhit && pipeline.hover && !self.is_active() {
            self.hits.rebuild(figure.hits(resolved, &mut self.cache));
            self.unhit = false;
        }
        // The first refresh after a gesture renders the ring it left.
        let settle = self.deferred && !self.is_active();
        if plan.is_empty() && !verify && !settle {
            return 0;
        }
        // A tile's work is its share of its layer's over the area, by the
        // pixels of the area it covers (the ring's tiles count pixels
        // only); an overlay's, its chrome's over its rect.
        let areas = rects
            .iter()
            .zip(&layers.layers)
            .map(|(&rect, layer)| work(&layer.source.list, rect, k))
            .collect();
        let splat = layers
            .layers
            .iter()
            .map(|layer| {
                layer
                    .source
                    .list
                    .commands
                    .iter()
                    .any(|command| match command {
                        Command::Markers { at, .. } => at.len() >= SPLAT,
                        Command::Fill { .. }
                        | Command::Stroke { .. }
                        | Command::Glyphs(_)
                        | Command::Image { .. }
                        | Command::Push { .. }
                        | Command::Pop
                        // `Command` is #[non_exhaustive] in rustd3-figure.
                        | _ => false,
                    })
            })
            .collect();
        self.drawn = Some(Drawn {
            cells: cells.cells.iter().map(|cell| cell.work).collect(),
            areas,
            splat,
            layers,
            k,
        });
        self.render(pipeline);
        plan.len()
    }

    /// Renders everything waiting, what the last refresh changed and what
    /// an earlier job set left, at the current generation, in
    /// [`waiting`](State::waiting)'s order: inline under `Render::Inline`,
    /// on the pool under `Render::Worker`, and under `Render::Auto` inline
    /// each job whose predicted time still fits half the frame budget with
    /// those before it, the rest on the pool, whose waiting jobs they
    /// replace (it abandons them); while a wheel gesture or a brush lasts,
    /// every tile goes to the pool. Under `Caching::Verify` every kept or
    /// translated slot renders again after them, as a check. Everything
    /// renders inline when the pool cannot start.
    fn render(&mut self, pipeline: &Pipeline<'_>) {
        let probe = pipeline.probe;
        let enqueued = probe.map(|probe| Stamp::now(probe.clock()));
        let generation = self.generation;
        let todo = self.waiting();
        self.deferred = self.is_active()
            && self.areas.iter().any(|area| {
                area.tiles()
                    .iter()
                    .any(|tile| tile.slot().pending().is_some() && !area.shows(tile))
            });
        // An earlier set's jobs render again at this generation.
        for &at in &todo {
            self.requeue(at, generation);
        }
        let jobs: Vec<Job> = todo
            .into_iter()
            .filter_map(|at| self.job(at, generation, enqueued))
            .collect();
        let mut predicted = 0.0;
        // During a wheel gesture or a brush the UI thread renders no tile:
        // a zoom step's tiles would take its frames, and the pool shows
        // them as they land. A drag's tiles are the column entering the
        // area, which renders inline when it fits as idle renders do: on
        // the pool it lands a frame later, and on the software renderer a
        // frame is long.
        let pooled = self.is_active() && !self.is_dragging();
        let (mut here, mut there): (Vec<Job>, Vec<Job>) = match pipeline.render {
            Render::Inline => (jobs, Vec::new()),
            Render::Worker => (Vec::new(), jobs),
            Render::Auto => jobs.into_iter().partition(|job| {
                if pooled && matches!(job.spot, Spot::Tile { .. }) {
                    return false;
                }
                let ms = self
                    .budget
                    .predict(job.spot.kind(), slot::pixels(job.rect), job.work);
                let fits = self.budget.is_inline(predicted + ms);
                if fits {
                    predicted += ms;
                }
                fits
            }),
        };
        if pipeline.caching == Caching::Verify {
            let checks = self
                .kept()
                .into_iter()
                .filter_map(|at| self.check(at, enqueued));
            match pipeline.render {
                Render::Inline => here.extend(checks),
                Render::Worker | Render::Auto => there.extend(checks),
            }
        }
        if !there.is_empty() && !self.start(pipeline) {
            // No thread: everything renders here.
            here.append(&mut there);
        }
        for job in here {
            let done = job.run(&mut self.raster);
            self.land(done, true, probe);
        }
        if let Some(pool) = &self.pool {
            pool.replace(there);
        }
    }

    /// Every slot that shows its current content and waits for no render
    /// (kept, or translated with its plot area): gutter runs, tiles,
    /// overlays.
    fn kept(&self) -> Vec<At> {
        let idle = |slot: &Slot| slot.is_current() && slot.pending().is_none();
        let gutters = (0..self.gutters.len())
            .filter(|&g| idle(self.gutters[g].slot()))
            .map(At::Gutter);
        let tiles = self.areas.iter().enumerate().flat_map(move |(p, area)| {
            area.tiles()
                .iter()
                .enumerate()
                .filter(move |(_, tile)| idle(tile.slot()) && tile.covers(area.origin()))
                .map(move |(t, _)| At::Tile(p, t))
        });
        let overlays = self.areas.iter().enumerate().flat_map(move |(p, area)| {
            (0..area.overlays().len())
                .filter(move |&o| idle(&area.overlays()[o]))
                .map(move |o| At::Overlay(p, o))
        });
        gutters.chain(tiles).chain(overlays).collect()
    }

    /// The check of the image `at` shows: a job rendering it again for the
    /// generation and content of that image.
    fn check(&self, at: At, enqueued: Option<Stamp>) -> Option<Job> {
        let slot = match at {
            At::Gutter(g) => self.gutters.get(g)?.slot(),
            At::Tile(p, t) => self.areas.get(p)?.tiles().get(t)?.slot(),
            At::Overlay(p, o) => self.areas.get(p)?.overlays().get(o)?,
        };
        let (generation, hash) = slot.shows()?;
        let job = self.job(at, generation, enqueued)?;
        Some(Job {
            hash,
            check: true,
            ..job
        })
    }

    /// The job that renders `at` for `generation` from what the last
    /// repaint drew, its queue time measured from `enqueued`; `None`
    /// before a repaint drew anything or for a slot that is gone.
    fn job(&self, at: At, generation: u64, enqueued: Option<Stamp>) -> Option<Job> {
        let drawn = self.drawn.as_ref()?;
        let k = drawn.k;
        let splat = match at {
            At::Tile(p, _) => drawn.splat.get(p).copied().unwrap_or(false),
            At::Gutter(_) | At::Overlay(..) => false,
        };
        let (target, spot, rect, hash, source, work, view) = match at {
            At::Gutter(g) => {
                let run = self.gutters.get(g)?;
                let cells = run.cells();
                let target = probe::Target::Cell(
                    u32::try_from(cells.start).expect("invariant: fewer than 2^32 cells"),
                );
                let work = cells
                    .map(|c| drawn.cells.get(c).copied().unwrap_or(0))
                    .sum();
                let slot = run.slot();
                let source = &drawn.layers.chrome;
                (
                    target,
                    Spot::Gutter(g),
                    slot.rect(),
                    slot.hash(),
                    source,
                    work,
                    [None; 2],
                )
            }
            At::Tile(p, t) => {
                let area = self.areas.get(p)?;
                let tile: &Tile = area.tiles().get(t)?;
                let key = tile.key();
                let target = probe::Target::Tile {
                    area: key.area,
                    i: key.i,
                    j: key.j,
                };
                let spot = Spot::Tile {
                    area: p,
                    grid: area.serial(),
                    i: key.i,
                    j: key.j,
                    at: area.origin(),
                };
                let all = u128::from(slot::pixels(area.rect()).max(1));
                let px = u128::from(slot::pixels(tile::intersect(tile.rect(), area.rect())));
                let layer = u128::from(drawn.areas.get(p).copied().unwrap_or(0));
                let work = u64::try_from(layer * px / all).unwrap_or(u64::MAX);
                let source = &drawn.layers.layers.get(p)?.source;
                (
                    target,
                    spot,
                    tile.rect(),
                    tile.slot().hash(),
                    source,
                    work,
                    self.peek(p),
                )
            }
            At::Overlay(p, o) => {
                let slot = self.areas.get(p)?.overlays().get(o)?;
                let rect = slot.rect();
                let [x, y, ..] = rect;
                let cell = self
                    .cells
                    .iter()
                    .position(|&[x0, y0, x1, y1]| (x0..x1).contains(&x) && (y0..y1).contains(&y));
                let target = probe::Target::Cell(
                    cell.and_then(|c| u32::try_from(c).ok()).unwrap_or(u32::MAX),
                );
                let work = work(&drawn.layers.fixed.list, rect, k);
                (
                    target,
                    Spot::Overlay { area: p },
                    rect,
                    slot.hash(),
                    &drawn.layers.fixed,
                    work,
                    [None; 2],
                )
            }
        };
        Some(Job {
            target,
            spot,
            generation,
            hash,
            list: Arc::clone(&source.list),
            bounds: Arc::clone(&source.bounds),
            rect,
            bands: slot::bands(rect),
            scale: k,
            work,
            splat,
            check: false,
            enqueued,
            view,
        })
    }

    /// Starts the pool of `pipeline.threads` threads unless it runs at
    /// that size, waking the UI through `pipeline.waker`; a pool of another
    /// size stops first, its results landing. False when the OS cannot
    /// start a thread: everything then renders inline.
    fn start(&mut self, pipeline: &Pipeline<'_>) -> bool {
        if let Some(pool) = self.pool.take_if(|pool| pool.size() != pipeline.threads) {
            for done in pool.stop() {
                self.land(done, false, pipeline.probe);
            }
        }
        if self.pool.is_none() {
            self.pool = Pool::start(pipeline.threads, pipeline.waker).ok();
        }
        self.pool.is_some()
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

    /// Everything waiting for a render, in the order it renders: the
    /// tiles that meet their plot area and do not show their part of it
    /// exact, then the gutters and the overlays, then the ring of tiles
    /// around each area and the tiles whose part inside it is exact; the
    /// tiles of each group nearest the last gesture input first (each
    /// area's centre without one). During a gesture the gutters and the
    /// overlays go first, so that on the pool the axes render ahead of the
    /// tiles each step replaces and keep step with the data;
    /// and the ring waits: each step of a zoom would render it all again,
    /// two to three times the tiles that show.
    fn waiting(&self) -> Vec<At> {
        let active = self.is_active();
        let tiles = |ring: bool| {
            let mut tiles: Vec<(f64, At)> = self
                .areas
                .iter()
                .enumerate()
                .flat_map(move |(p, area)| {
                    area.tiles()
                        .iter()
                        .enumerate()
                        .filter(move |(_, tile)| {
                            let shows = area.shows(tile);
                            let late = !shows || area.is_exact(tile);
                            tile.slot().pending().is_some() && late == ring && (shows || !active)
                        })
                        .map(move |(t, tile)| (self.distance(area, tile), At::Tile(p, t)))
                })
                .collect();
            tiles.sort_by(|a, b| a.0.total_cmp(&b.0));
            tiles.into_iter().map(|(_, at)| at)
        };
        let gutters = (0..self.gutters.len())
            .filter(|&g| self.gutters[g].slot().pending().is_some())
            .map(At::Gutter);
        let overlays = self.areas.iter().enumerate().flat_map(|(p, area)| {
            (0..area.overlays().len())
                .filter(move |&o| area.overlays()[o].pending().is_some())
                .map(move |o| At::Overlay(p, o))
        });
        if active {
            gutters
                .chain(overlays)
                .chain(tiles(false))
                .chain(tiles(true))
                .collect()
        } else {
            tiles(false)
                .chain(gutters)
                .chain(overlays)
                .chain(tiles(true))
                .collect()
        }
    }

    /// How far `tile`'s centre lies from the last gesture input, or from
    /// `area`'s centre without one, device px.
    fn distance(&self, area: &Area, tile: &Tile) -> f64 {
        let centre = |[x0, y0, x1, y1]: [i32; 4]| {
            kurbo::Point::new(
                (f64::from(x0) + f64::from(x1)) / 2.0,
                (f64::from(y0) + f64::from(y1)) / 2.0,
            )
        };
        let k = self.factor.map_or(1.0, f64::from);
        let focus = self
            .focus
            .map_or_else(|| centre(area.rect()), |at| (at.to_vec2() * k).to_point());
        centre(tile.rect()).distance(focus)
    }

    /// A pending render of `at` moves to `generation`: the job set it was
    /// in was replaced by one that renders it again.
    fn requeue(&mut self, at: At, generation: u64) {
        let slot = match at {
            At::Gutter(g) => self.gutters[g].slot_mut(),
            At::Tile(p, t) => self.areas[p].tiles_mut()[t].slot_mut(),
            At::Overlay(p, o) => &mut self.areas[p].overlays_mut()[o],
        };
        slot.requeue(generation);
    }

    /// Applies the pool's results in arrival order, each reported to
    /// `probe`. True when any was shown, so the widget redraws.
    pub(crate) fn drain(&mut self, probe: Option<&Probe>) -> bool {
        let Some(pool) = &self.pool else {
            return false;
        };
        let mut shown = false;
        for done in pool.drain() {
            shown |= self.land(done, false, probe);
        }
        shown
    }

    /// Plans a refresh whose partition is `partition`, for the plot areas
    /// `planned` describes: rebuilds the cells, the gutter runs and the
    /// areas when their rects differ (every run, tile and overlay new and
    /// changed; at the factor the images shown were rendered at, the old
    /// runs' images stay drawn under the new runs and the old areas' join
    /// the new ones' preview layers until theirs land); else
    /// finds the gutter runs whose content hash differs
    /// from their slot's, each area's tiles that its layer's hash, its
    /// style and its grid's move leave stale, and the overlays whose rects
    /// or hashes changed. With `caching` off, every one when any changed;
    /// every one when `revision` is not the one the slots were planned at.
    /// If any changed, bumps the generation, stores the new hashes and
    /// marks them pending. Returns them, tiles first, area by area, then
    /// the gutters, then the overlays.
    fn plan(
        &mut self,
        partition: Partition<'_>,
        planned: Planned<'_>,
        (caching, revision): (Caching, u64),
    ) -> Vec<At> {
        let Planned {
            k,
            rects,
            lists,
            views,
            contents,
            overlays,
            moves,
        } = planned;
        let cells = &partition.cells.cells;
        let cell_rects: Vec<[i32; 4]> = cells.iter().map(|cell| slot::device(cell.rect)).collect();
        let hashes: Vec<u64> = cells.iter().map(|cell| cell.hash).collect();
        let same = self.cells == cell_rects
            && self.areas.len() == rects.len()
            && self.areas.iter().zip(rects).all(|(a, r)| a.rect() == *r)
            && moves.len() == rects.len();
        let mut changed: Vec<usize> = if same {
            (0..self.gutters.len())
                .filter(|&g| {
                    let run = &self.gutters[g];
                    run.slot().hash() != run.hash(&hashes)
                })
                .collect()
        } else {
            // What the old layout shows stays until the new slots' images
            // land, when it was rendered at this factor.
            let carry = self.drawn.as_ref().is_some_and(|drawn| drawn.k == k);
            self.owner = cell_rects
                .iter()
                .map(|r| rects.iter().position(|a| tile::intersect(*r, *a) == *r))
                .collect();
            let across = partition.cells.cols.len().saturating_sub(1).max(1);
            let deps = merge::deps(partition.cells, partition.chrome, partition.resolved);
            let runs = merge::runs(&cell_rects, across, &self.owner, &deps)
                .into_iter()
                .map(|run| {
                    let hash = run
                        .clone()
                        .fold(layer::FNV, |h, c| layer::fnv(h, hashes[c]));
                    Run::new(run, &cell_rects, hash)
                })
                .collect();
            let old = std::mem::replace(&mut self.gutters, runs);
            if carry {
                let imaged = old.into_iter().map(Run::into_slot);
                self.former.extend(imaged.filter(|slot| !slot.is_missing()));
            } else {
                self.former.clear();
            }
            self.prune();
            self.cells = cell_rects;
            let old = std::mem::take(&mut self.areas);
            // The same panels, in panel order.
            let inherit = carry && old.len() == rects.len();
            self.areas = (0..rects.len())
                .map(|p| {
                    self.grids += 1;
                    let index = u32::try_from(p).expect("invariant: fewer than 2^32 plot areas");
                    Area::new(
                        index,
                        (rects[p], lists[p]),
                        contents[p],
                        self.grids,
                        views[p],
                    )
                })
                .collect();
            if inherit {
                for (p, (area, old)) in self.areas.iter_mut().zip(old).enumerate() {
                    area.inherit(old, (k, views[p]));
                }
            }
            self.floor = self.generation + 1;
            (0..self.gutters.len()).collect()
        };
        let moved = |p: usize| if same { moves[p] } else { Moved::Anew };
        let dirty = !same
            || !changed.is_empty()
            || self.areas.iter().enumerate().any(|(p, area)| {
                moved(p) != Moved::Still
                    || area.is_stale(contents[p].0)
                    || area.overlays_differ(&overlays[p])
            });
        let every = match caching {
            Caching::On | Caching::Verify => revision != self.revision,
            Caching::Off => revision != self.revision || dirty,
        };
        self.revision = revision;
        if every {
            changed = (0..self.gutters.len()).collect();
        }
        if !(dirty || every) {
            return Vec::new();
        }
        self.generation += 1;
        let generation = self.generation;
        for &g in &changed {
            let hash = self.gutters[g].hash(&hashes);
            self.gutters[g].slot_mut().change(hash, generation);
        }
        let mut tiles = Vec::new();
        let mut layered = Vec::new();
        for (p, area) in self.areas.iter_mut().enumerate() {
            area.change(contents[p], generation, (moved(p), every));
            tiles.extend(
                area.tiles()
                    .iter()
                    .enumerate()
                    .filter(|(_, tile)| tile.slot().pending() == Some(generation))
                    .map(|(t, _)| At::Tile(p, t)),
            );
            let fixed = area.overlay(&overlays[p], generation, every);
            layered.extend(fixed.into_iter().map(|o| At::Overlay(p, o)));
        }
        tiles
            .into_iter()
            .chain(changed.into_iter().map(At::Gutter))
            .chain(layered)
            .collect()
    }

    /// Applies a rendered job, reported to `probe` as rendered `inline` or
    /// not: its time joins the cost model's fit (every render but
    /// `Caching::Verify`'s checks, which are not the work the model
    /// predicts); it is shown when it belongs to this partition and
    /// is newer than its slot's image. True when it is shown.
    fn land(&mut self, done: Done, inline: bool, probe: Option<&Probe>) -> bool {
        if done.check {
            self.verify(done, probe);
            return false;
        }
        self.budget
            .calibrate(done.spot.kind(), done.elapsed, done.pixels(), done.work);
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
        let bands = bands.ok();
        match done.spot {
            Spot::Gutter(index) => {
                let shown = self.gutters.get_mut(index).is_some_and(|run| {
                    run.slot_mut()
                        .land(done.generation, done.hash, bands, done.view)
                });
                if shown {
                    self.prune();
                }
                shown
            }
            Spot::Tile {
                area,
                grid,
                i,
                j,
                at,
            } => {
                let views = (
                    done.view,
                    self.factor.map_or(1.0, f64::from),
                    self.peek(area),
                );
                self.areas.get_mut(area).is_some_and(|area| {
                    let tile = (grid, i, j, at);
                    let image = (done.rect, bands);
                    area.land(tile, (done.generation, done.hash), image, views)
                })
            }
            Spot::Overlay { area } => self.areas.get_mut(area).is_some_and(|area| {
                area.land_overlay(done.rect, (done.generation, done.hash), bands)
            }),
        }
    }

    /// Applies a check (`Caching::Verify`): the fresh render against the
    /// image its slot shows, which it replaces when they differ by more
    /// than 1/255, reported to `probe` as a mismatch.
    fn verify(&mut self, done: Done, probe: Option<&Probe>) {
        if done.generation < self.floor {
            return;
        }
        let Ok(bands) = done
            .rows
            .into_iter()
            .zip(done.bands)
            .map(|(rows, band)| band.map(|image| Band::new(rows, handle::from_image(image))))
            .collect::<Result<Vec<Band>, raster::Error>>()
        else {
            return;
        };
        let shown = (done.generation, done.hash);
        let max = match done.spot {
            Spot::Gutter(g) => self
                .gutters
                .get_mut(g)
                .and_then(|run| run.slot_mut().check(shown, bands)),
            Spot::Tile {
                area,
                grid,
                i,
                j,
                at,
            } => self
                .areas
                .get_mut(area)
                .and_then(|area| area.check((grid, i, j, at), shown, (done.rect, bands))),
            Spot::Overlay { area } => self
                .areas
                .get_mut(area)
                .and_then(|area| area.check_overlay(done.rect, shown, bands)),
        };
        if let (Some(max), Some(probe)) = (max.filter(|&max| max > 1), probe) {
            probe.send(Sample::Mismatch {
                frame: self.frame,
                target: done.target,
                max,
            });
        }
    }

    /// Drops each gutter image of an earlier layout that no run waits
    /// under: every run that meets it shows an image of its own.
    fn prune(&mut self) {
        let gutters = &self.gutters;
        self.former.retain(|former| {
            gutters
                .iter()
                .any(|run| run.slot().is_missing() && tile::meets(run.slot().rect(), former.rect()))
        });
    }

    /// What the following draw shows; reports each newly shown image's
    /// bytes once. A gutter run without an image of its own that an image
    /// of an earlier layout meets shows that image: a preview.
    fn report(&mut self, at: Instant) -> Sample {
        let gutters = || self.gutters.iter().map(Run::slot);
        let under = |slot: &Slot| {
            slot.is_missing()
                && self
                    .former
                    .iter()
                    .any(|former| tile::meets(former.rect(), slot.rect()))
        };
        let count = |test: &dyn Fn(&Slot) -> bool| {
            u32::try_from(gutters().filter(|s| test(s)).count()).unwrap_or(u32::MAX)
        };
        let mut missing = count(&|slot| slot.is_missing() && !under(slot));
        let mut previews = count(&|slot| slot.is_preview() || under(slot));
        let mut images = u32::try_from(gutters().count()).unwrap_or(u32::MAX) - missing;
        if let Some(factor) = self.factor {
            for (p, area) in self.areas.iter().enumerate() {
                let shown = area.shown(factor, self.peek(p));
                images += shown.images;
                missing += shown.missing;
                previews += shown.previews;
            }
        }
        let gutters: u64 = self
            .gutters
            .iter_mut()
            .map(|run| run.slot_mut().take_fresh())
            .sum();
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
    /// active, or the index is not of the list shown (until the refresh
    /// after a gesture builds it), nothing is hit-tested: the overlay
    /// shows no ring and nothing changes for `on_hover`. True when the
    /// overlay or the hovered point changed.
    pub(crate) fn hover(&mut self, bounds: Rectangle, shown: overlay::Shown) -> bool {
        let Some(factor) = self.factor else {
            return false;
        };
        let busy = self.is_active() || self.unhit;
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
    /// gesture is active or the index is not of the list shown.
    pub(crate) fn reported(&mut self) -> Option<Option<Hover>> {
        if self.is_active() || self.unhit {
            return None;
        }
        self.hits.report()
    }

    /// Draws every image of a gutter run over the images an earlier layout
    /// left (the theme background where a run has none), each plot area's
    /// tiles over its
    /// preview layer mapped from the views it was planned at to the views
    /// now, then each brush's selection and the hover overlay in the
    /// theme's ink, for a widget laid out at `bounds`. While a gesture
    /// lasts, and while previews or an earlier layout's gutter images
    /// show, the images are drawn in one layer over the widget whose
    /// bounds alternate between frames by half a logical px, so that
    /// iced's software renderer redraws the widget once a frame; on that
    /// renderer an area that shows a preview layer is drawn as one image
    /// composed on the CPU ([`Area::draw_mosaic`]).
    pub(crate) fn draw<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle, theme: &Theme)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let Some(factor) = self.factor else {
            return;
        };
        let origin = origin(bounds, factor);
        let background = overlay::color(theme.get_background());
        // iced's software renderer redraws each damaged rect of a frame
        // apart, every image that meets it once per rect, each draw
        // clearing a frame-sized clip mask (`tiny_skia/src/lib.rs:72-160`,
        // `engine.rs:533`), and finds a moved image's damage image by image
        // (`layer.rs:289-294`): a frame of a pan or zoom, where every image
        // moves, damaged many small rects. A layer whose bounds changed is
        // damaged as a whole (`layer.rs:206-209`), so while a gesture lasts,
        // and while images of an earlier grid or layout show (mapped to the
        // views now, they move with them; at a settle all of them move at
        // once), the images go in one layer whose bounds alternate by half
        // a px: one rect, each image drawn once.
        let moving =
            self.is_active() || !self.former.is_empty() || self.areas.iter().any(Area::has_preview);
        // Grown when the last draw was not, so that the first draw of a
        // gesture differs from the idle one before it too.
        let grown = moving && !self.grown.get();
        self.grown.set(grown);
        let layer = if grown { bounds.expand(0.5) } else { bounds };
        // A renderer that reports no scale is iced's software one
        // (`scale_factor`): while an area shows a preview layer, its scaled
        // and overlapping tiles are composed on the CPU and drawn as one
        // image (`Mosaic`), each image costing tiny-skia a frame-sized clip
        // mask and a filtered pass.
        let software = renderer.scale().is_none();
        renderer.with_layer(layer, |renderer| {
            for run in &self.gutters {
                run.slot().fill(renderer, origin, factor, background);
            }
            for former in &self.former {
                former.draw(renderer, origin, factor);
            }
            for run in &self.gutters {
                run.slot().draw(renderer, origin, factor);
            }
            for (p, area) in self.areas.iter().enumerate() {
                let at = (origin, factor);
                if software && area.has_preview() {
                    area.draw_mosaic(renderer, at, self.peek(p), background);
                } else {
                    area.draw(renderer, at, self.peek(p), background);
                }
            }
        });
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

/// A job's slot: gutter run `g`, tile `t` of plot area `p`, or overlay
/// `o` of plot area `p`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Gutter(usize),
    Tile(usize, usize),
    Overlay(usize, usize),
}

/// What a refresh plans the gutters from: the partition's cells of the
/// chrome, the chrome they were cut from and the resolve that drew it.
struct Partition<'a> {
    cells: &'a list::Cells,
    chrome: &'a List,
    resolved: &'a Resolved,
}

/// What a refresh plans the plot areas from: the factor, and per area its
/// device rect and list px, the views its axes show, its layer's hash and
/// style hash, its overlays' rects and hashes, and how its grid moved.
struct Planned<'a> {
    k: f64,
    rects: &'a [[i32; 4]],
    lists: &'a [kurbo::Rect],
    views: &'a [[Option<Basis>; 2]],
    contents: &'a [(u64, u64)],
    overlays: &'a [Vec<([i32; 4], u64)>],
    moves: &'a [Moved],
}

/// What a repaint drew from: the lists every job renders (each plot
/// area's layer, the chrome, the chrome without the figure's background)
/// at `k` device px per list px, the raster work of each partition cell,
/// and of each layer over its plot area, and whether each layer's tiles
/// splat their markers.
struct Drawn {
    layers: Layers,
    k: f64,
    cells: Vec<u64>,
    areas: Vec<u64>,
    splat: Vec<bool>,
}

/// The points of a markers command from which the widget splats it
/// (`raster::Renderer::splat`): the large marker sets on screen.
pub(crate) const SPLAT: usize = 10_000;

/// Where the parts of `fixed` (the chrome without the figure's
/// background) reach into the plot area at device rect `area` at `k`:
/// each part's device bounds, out by a device px for antialiasing as
/// cells take them, cut to the area, and cut into disjoint rects (each
/// rect less those before it), so that an overlay's pixel is composited
/// once; each with an FNV-1a hash of its rect and the hashes of the parts
/// that reach into it.
fn overlays(fixed: &List, area: [i32; 4], k: f64) -> Vec<([i32; 4], u64)> {
    let reach: Vec<([i32; 4], u64)> = fixed
        .parts
        .iter()
        .filter(|part| part.bounds.area() > 0.0)
        .filter_map(|part| {
            let b = part.bounds.scale_from_origin(k).inflate(1.0, 1.0);
            let device = [b.x0.floor(), b.y0.floor(), b.x1.ceil(), b.y1.ceil()].map(|e| e as i32);
            let [x0, y0, x1, y1] = tile::intersect(device, area);
            (x0 < x1 && y0 < y1).then_some(([x0, y0, x1, y1], part.hash))
        })
        .collect();
    let mut rects: Vec<[i32; 4]> = Vec::new();
    for &(rect, _) in &reach {
        let mut pieces = vec![rect];
        for &taken in &rects {
            pieces = pieces
                .into_iter()
                .flat_map(|piece| subtract(piece, taken))
                .collect();
        }
        rects.extend(pieces);
    }
    rects.sort_unstable();
    rects
        .into_iter()
        .map(|rect| {
            let hash = rect
                .iter()
                .fold(layer::FNV, |h, &e| layer::fnv(h, e as u64));
            let meets = |r: [i32; 4]| {
                let [x0, y0, x1, y1] = tile::intersect(r, rect);
                x0 < x1 && y0 < y1
            };
            let hash = reach
                .iter()
                .filter(|(r, _)| meets(*r))
                .fold(hash, |h, &(_, part)| layer::fnv(h, part));
            (rect, hash)
        })
        .collect()
}

/// Device rect `a` less `b`: up to four disjoint rects (the bands above
/// and below `b`, then the parts left and right of it between them); `a`
/// when they do not overlap.
fn subtract(a: [i32; 4], b: [i32; 4]) -> Vec<[i32; 4]> {
    let [x0, y0, x1, y1] = tile::intersect(a, b);
    if x0 >= x1 || y0 >= y1 {
        return vec![a];
    }
    [
        [a[0], a[1], a[2], y0],
        [a[0], y1, a[2], a[3]],
        [a[0], y0, x0, y1],
        [x1, y0, a[2], y1],
    ]
    .into_iter()
    .filter(|r| r[0] < r[2] && r[1] < r[3])
    .collect()
}

/// The raster work of `list` over device rect `rect` at `k`: what its
/// parts there count as one cell (`list::cells`).
fn work(list: &List, [x0, y0, x1, y1]: [i32; 4], k: f64) -> u64 {
    let edges = |a: i32, b: i32| [f64::from(a) / k, f64::from(b) / k];
    list::cells(list, &edges(x0, x1), &edges(y0, y1), k)
        .cells
        .first()
        .map_or(0, |cell| cell.work)
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
    use crate::plot::budget::{Budget, Kind};
    use crate::plot::merge::Run;
    use crate::plot::overlay::{Overlay, Shown};
    use crate::plot::probe::{self, Probe, Sample, Stage};
    use crate::plot::slot::Slot;
    use crate::plot::tile::{self, Tile};
    #[cfg(target_os = "linux")]
    use crate::plot::worker::Pool;
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
            threads: 1,
            hover: true,
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
            threads: 1,
            hover: true,
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

    /// Drains the pool's results into `state` after each wake, reported
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

    /// A raster target as a key that sorts: cells (and overlays) first.
    fn order(target: probe::Target) -> (u8, i64, i64, i64) {
        match target {
            probe::Target::Cell(c) => (0, i64::from(c), 0, 0),
            probe::Target::Tile { area, i, j } => (1, i64::from(area), i64::from(i), i64::from(j)),
        }
    }

    /// The targets of the raster samples among `samples`, sorted.
    fn rasters(samples: &[Sample]) -> Vec<probe::Target> {
        let mut targets: Vec<probe::Target> = samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Raster { target, .. } => Some(*target),
                Sample::Stage { .. } | Sample::Frame { .. } | Sample::Mismatch { .. } => None,
            })
            .collect();
        targets.sort_by_key(|&target| order(target));
        targets
    }

    /// The device rects of what shows in the canvas: the gutter runs,
    /// then each plot area's tiles that meet it, cut to it.
    fn shown(state: &State) -> Vec<[i32; 4]> {
        let gutters = state.gutters.iter().map(|run| run.slot().rect());
        let tiles = state.areas.iter().flat_map(|area| {
            area.tiles()
                .iter()
                .filter(|tile| area.shows(tile))
                .map(|tile| tile::intersect(tile.rect(), area.rect()))
        });
        gutters.chain(tiles).collect()
    }

    /// Every slot of `state`: the gutter runs (as their first cell), each
    /// plot area's tiles (its ring included) and its overlays (as the cell
    /// holding their corner).
    fn slots(state: &State) -> Vec<(probe::Target, &Slot)> {
        let cells = state.gutters.iter().map(|run| {
            let first = u32::try_from(run.cells().start).unwrap();
            (probe::Target::Cell(first), run.slot())
        });
        let tiles = state.areas.iter().flat_map(|area| {
            area.tiles().iter().map(|tile| {
                let key = tile.key();
                let target = probe::Target::Tile {
                    area: key.area,
                    i: key.i,
                    j: key.j,
                };
                (target, tile.slot())
            })
        });
        let overlays = state.areas.iter().flat_map(|area| {
            area.overlays().iter().map(|slot| {
                let [x, y, ..] = slot.rect();
                let cell = state
                    .cells
                    .iter()
                    .position(|&[x0, y0, x1, y1]| (x0..x1).contains(&x) && (y0..y1).contains(&y))
                    .expect("an overlay lies in a cell");
                (probe::Target::Cell(u32::try_from(cell).unwrap()), slot)
            })
        });
        cells.chain(tiles).chain(overlays).collect()
    }

    /// Every target of `state`, sorted: what one refresh that renders
    /// everything reports.
    fn targets(state: &State) -> Vec<probe::Target> {
        let mut targets: Vec<probe::Target> = slots(state).into_iter().map(|(t, _)| t).collect();
        targets.sort_by_key(|&target| order(target));
        targets
    }

    #[test]
    fn cells_partition_the_canvas() {
        let fig = fixture::grid();
        for k in FACTORS {
            let mut state = state(640.0, 480.0);
            let rendered = refresh(&mut state, &fig, k);
            let canvas = [0, 0, (640.0 * k).round() as i32, (480.0 * k).round() as i32];
            let cells = state.cells.clone();
            // Four plot areas cut each axis into at most nine tracks.
            assert!(cells.len() > 4, "k = {k}: {} cells", cells.len());
            // The cells outside plot areas, the plot areas' tiles and their
            // overlays render; the cells and the tiles that meet a plot area,
            // cut to it, partition the canvas as the cells do.
            assert_eq!(rendered, slots(&state).len());
            let rects = shown(&state);
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
            slots(state)
                .into_iter()
                .map(|(_, slot)| slot.images())
                .collect()
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
        for (g, run) in state.gutters.iter().enumerate() {
            assert_eq!(run.slot().generation(), Some(1), "run {g}");
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
                Sample::Stage { .. }
                | Sample::Raster { .. }
                | Sample::Frame { .. }
                | Sample::Mismatch { .. } => None,
            })
            .collect();
        assert_eq!(stages, [Stage::Resolve, Stage::List]);
        let cells = rasters(&samples);
        let expected = targets(&state);
        assert_eq!(
            cells, expected,
            "one raster sample per cell, tile and overlay"
        );
        // Every slot's pixels, and what the frame shows of them: the cells,
        // the tiles that meet a plot area and the overlays.
        let all: u64 = slots(&state)
            .iter()
            .map(|(_, slot)| crate::plot::slot::pixels(slot.rect()))
            .sum();
        let visible = slots(&state).len()
            - state
                .areas
                .iter()
                .map(|area| area.tiles().iter().filter(|tile| !area.shows(tile)).count())
                .sum::<usize>();
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
        assert_eq!(pixels, all);
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
                Sample::Stage { .. } | Sample::Raster { .. } | Sample::Mismatch { .. } => None,
            })
            .collect();
        let slots = u32::try_from(visible).unwrap();
        assert_eq!(frames, [(1, 1, slots, 0, 0, all * 4)]);
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
            threads: 1,
            hover: true,
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
        assert_eq!(rasters(&drained), expected, "one raster sample per slot");
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
                    if *images == slots && *uploaded == all * 4
            ),
            "{shown:?}"
        );
    }

    #[test]
    fn renders_calibrate_the_cost_model_without_a_probe() {
        // The widget times its own renders, so one refresh that no
        // probe watches moves the fit off its seed.
        let mut state = state(480.0, 320.0);
        assert!(refresh(&mut state, &fixture::grid(), 2.0) > 0);
        let seeded = Budget::default();
        for kind in [Kind::Tile, Kind::Cell] {
            for (pixels, work) in [(65_536, 0), (65_536, 400), (614_400, 2_000)] {
                assert_ne!(
                    state.budget.predict(kind, pixels, work),
                    seeded.predict(kind, pixels, work),
                    "{kind:?}: {pixels} px, {work} items"
                );
            }
        }
    }

    /// The threads of this process named `name`: Linux names a thread
    /// after the one that started it unless it is given a name, as
    /// vello_cpu's own pool's are not.
    #[cfg(target_os = "linux")]
    fn named(name: &str) -> usize {
        std::fs::read_dir("/proc/self/task")
            .expect("the process's threads")
            .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("comm")).ok())
            .filter(|comm| comm.trim_end() == name)
            .count()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn inline_renders_start_no_thread_whatever_the_pool_size() {
        // What renders inline renders single-threaded, so the
        // pool's threads (named `rustd3-plot`) are the widget's only ones.
        // In a thread of its own name, a state renders inline, then on a
        // pool of `threads`; no thread keeps that name from it. typst lays
        // text out on rayon's global pool, whose threads take the name of
        // the thread that first uses it, and std names the pool's threads
        // from inside them: the global pool starts here, and the count is
        // read until the pool's threads have their names.
        refresh(&mut state(480.0, 320.0), &fixture::grid(), 2.0);
        for threads in [1, 4] {
            let name = format!("rd3-inline-{threads}");
            let spawned = std::thread::Builder::new().name(name.clone());
            let own = name.clone();
            let counted = spawned
                .spawn(move || {
                    let (waker, wakes) = counting();
                    let pipeline = |render| Pipeline {
                        render,
                        caching: Caching::On,
                        revision: 0,
                        probe: None,
                        waker: &waker,
                        threads,
                        hover: true,
                    };
                    let mut state = state(480.0, 320.0);
                    let inline = pipeline(Render::Inline);
                    state.redraw::<()>(&fixture::grid(), 2.0, clock(), &inline, (&[], &[]));
                    state.resize(Size::new(470.0, 320.0));
                    let pool = pipeline(Render::Worker);
                    state.redraw::<()>(&fixture::grid(), 2.0, clock(), &pool, (&[], &[]));
                    settle(&mut state, &wakes, None);
                    assert_eq!(state.pool.as_ref().map(Pool::size), Some(threads));
                    let mut counted = named(&own);
                    for _ in 0..200 {
                        if counted == 1 {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        counted = named(&own);
                    }
                    counted
                })
                .expect("a thread");
            let counted = counted.join().expect("no panic");
            assert_eq!(counted, 1, "{threads} pool threads: {counted} named {name}");
        }
    }

    #[test]
    fn verify_checks_leave_the_cost_model_as_it_was() {
        // A check renders at background priority and is not the
        // work the model predicts. A rebuild of an equal figure keeps every
        // slot, so its refresh renders nothing but checks.
        let pipeline = Pipeline {
            render: Render::Inline,
            caching: Caching::Verify,
            revision: 0,
            probe: None,
            waker: &Waker::noop(),
            threads: 1,
            hover: true,
        };
        let mut state = state(480.0, 320.0);
        let first = state.redraw::<()>(&fixture::grid(), 2.0, clock(), &pipeline, (&[], &[]));
        assert!(first.0 > 0);
        let rows = [(65_536, 0), (65_536, 400), (614_400, 2_000)];
        let predict = |state: &State| {
            [Kind::Tile, Kind::Cell]
                .map(|kind| rows.map(|(pixels, work)| state.budget.predict(kind, pixels, work)))
        };
        let fitted = predict(&state);
        let kept = state.kept().len();
        assert!(kept > 0);
        state.rebuilt();
        let checked = state.redraw::<()>(&fixture::grid(), 2.0, clock(), &pipeline, (&[], &[]));
        assert_eq!(checked.0, 0, "{kept} slots kept, none changed");
        assert_eq!(predict(&state), fitted);
    }

    #[test]
    fn verify_replaces_a_differing_image() {
        let dots: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let pipeline = Pipeline {
            render: Render::Inline,
            caching: Caching::Verify,
            revision: 0,
            probe: Some(&probe),
            waker: &Waker::noop(),
            threads: 1,
            hover: true,
        };
        let mismatches = |samples: &[Sample]| -> Vec<(probe::Target, u8)> {
            samples
                .iter()
                .filter_map(|sample| match sample {
                    Sample::Mismatch { target, max, .. } => Some((*target, *max)),
                    Sample::Stage { .. } | Sample::Raster { .. } | Sample::Frame { .. } => None,
                })
                .collect()
        };
        let mut state = state(480.0, 320.0);
        state.redraw::<()>(
            &fixture::pair(dots.clone()),
            2.0,
            clock(),
            &pipeline,
            (&[], &[]),
        );
        assert!(mismatches(&rx.try_iter().collect::<Vec<_>>()).is_empty());
        // A rebuild of an equal figure keeps every slot: each is checked,
        // and all agree.
        state.rebuilt();
        state.redraw::<()>(
            &fixture::pair(dots.clone()),
            2.0,
            clock(),
            &pipeline,
            (&[], &[]),
        );
        assert!(mismatches(&rx.try_iter().collect::<Vec<_>>()).is_empty());
        // A resident tile corrupted in place: the next check replaces it
        // with a fresh render, the image it had.
        let (p, t) = (1, 7);
        let key = state.areas[p].tiles()[t].key();
        let before = state.areas[p].tiles()[t].slot().images();
        state.areas[p].tiles_mut()[t].slot_mut().corrupt();
        assert_ne!(state.areas[p].tiles()[t].slot().images(), before);
        state.rebuilt();
        state.redraw::<()>(&fixture::pair(dots), 2.0, clock(), &pipeline, (&[], &[]));
        let found = mismatches(&rx.try_iter().collect::<Vec<_>>());
        let target = probe::Target::Tile {
            area: key.area,
            i: key.i,
            j: key.j,
        };
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, target);
        assert!(found[0].1 > 1, "{found:?}");
        assert_eq!(state.areas[p].tiles()[t].slot().images(), before);
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
            threads: 1,
            hover: true,
        };
        let mut state = state(480.0, 320.0);
        state.redraw::<()>(&fixture::pair(dots), 2.0, clock(), &pipeline, (&[], &[]));
        state.rebuilt();
        state.redraw::<()>(&fixture::pair(moved), 2.0, clock(), &pipeline, (&[], &[]));
        assert_eq!(state.generation, 2);
        settle(&mut state, &wakes, None);
        assert!(state.gutters.iter().all(|run| run.slot().is_current()));
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
            let gutters = state.gutters.iter().map(Run::slot);
            let tiles = state.areas.iter().flat_map(Area::tiles).map(Tile::slot);
            for slot in gutters.chain(tiles) {
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
        assert!(state.cells.is_empty() && state.gutters.is_empty());
    }
}
