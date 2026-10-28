//! [`Driver`]: the widget that feeds a replay to the `Plot` as if the user
//! produced it, and times the plot's update and draw calls.
//!
//! A replay's positions are logical px in the plot size it was generated
//! for; the driver places each at the same fractions of the widget's real
//! bounds, so a window the window manager gave another size still gets
//! its gestures on the plot areas.

use std::cell::RefCell;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use iced::advanced::Shell;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget, tree};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, keyboard, mouse, window};
use rustd3_iced::plot::probe::{Sample, Stage, Target};

use crate::record::Record;
use crate::replay::{self, Cue, Delta, Replay};
use crate::system;

/// How long frames keep coming after the last event before the run can
/// end (it ends once every slot is sharp).
pub const SETTLE: Duration = Duration::from_secs(2);

/// How long the driver waits past the settle for every slot to be sharp
/// before it ends the run anyway, unsettled.
const TIMEOUT: Duration = Duration::from_secs(30);

/// The window in which input counts as interaction extends this far past
/// the last event (operational reading 4).
const AFTER_LAST: Duration = Duration::from_millis(300);

/// Feeds a replay to its child as if the user produced it, and times the
/// child's update and draw calls.
pub struct Driver<'a, Message> {
    child: Element<'a, Message, Theme, Renderer>,
    replay: &'a Replay,
    /// The plot size, logical px, the replay's positions were generated for.
    planned: Size,
    records: mpsc::Sender<Record>,
    cue: fn(Cue) -> Message,
    /// The child's probe samples, joined with the driver's timings.
    samples: &'a mpsc::Receiver<Sample>,
    /// The process's first instant, which record times count from.
    epoch: Instant,
}

/// The driver's state in iced's tree.
#[derive(Debug, Default)]
struct State {
    start: Option<Instant>,
    /// The next replay event to inject.
    next: usize,
    /// The synthetic cursor, in window coordinates.
    cursor: Option<iced::Point>,
    /// The widget's logical size in the last pass before the scenario's
    /// own first resize (the size the window manager gave it).
    widget: Size,
    /// A resize cue was injected: the size is the scenario's from here.
    resized: bool,
    modifiers: keyboard::Modifiers,
    /// The last `Frame` sample showed every slot current.
    sharp: bool,
    /// Injected events waiting for the frame that shows them.
    waiting: Vec<Waiting>,
    /// The last frame's record, completed by the draw that follows it.
    pending: RefCell<Option<Record>>,
    /// The update time of the current pass's `RedrawRequested`.
    update_ms: f64,
    first_input: bool,
    window_end: bool,
    done: bool,
}

/// An injected event before its frame is known.
#[derive(Debug)]
struct Waiting {
    event: String,
    scheduled_ms: f64,
    injected_ms: f64,
    update_ms: f64,
    /// `RedrawRequested` passes to skip: 0 for pointer input the widget
    /// applies in this pass, 1 for cues the app applies before the next.
    passes: u8,
}

impl<'a, Message> Driver<'a, Message> {
    /// Drives `child` with `replay`, generated for a plot of `planned`
    /// logical px; records to `records`, cues to the app through `cue`;
    /// `samples` receives the child's probe.
    pub fn new(
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
        replay: &'a Replay,
        planned: Size,
        records: mpsc::Sender<Record>,
        cue: fn(Cue) -> Message,
        samples: &'a mpsc::Receiver<Sample>,
        epoch: Instant,
    ) -> Self {
        Driver {
            child: child.into(),
            replay,
            planned,
            records,
            cue,
            samples,
            epoch,
        }
    }

    fn ms(&self, at: Instant) -> f64 {
        system::since(self.epoch, at)
    }

    fn send(&self, record: Record) {
        let _ = self.records.send(record);
    }

    /// Joins the child's samples with the driver's timings: `Frame`s
    /// become pending frame records, which also settle waiting inputs.
    fn drain(&self, state: &mut State) {
        for sample in self.samples.try_iter() {
            match sample {
                Sample::Stage {
                    frame,
                    stage,
                    duration,
                } => self.send(Record::Stage {
                    frame,
                    stage: match stage {
                        Stage::Resolve => "resolve",
                        Stage::List => "list",
                    }
                    .to_owned(),
                    ms: duration.as_secs_f64() * 1e3,
                }),
                Sample::Raster {
                    frame,
                    generation,
                    target,
                    pixels,
                    work,
                    queued,
                    duration,
                    inline,
                } => {
                    let (cell, tile) = spot(target);
                    self.send(Record::Raster {
                        frame,
                        generation,
                        cell,
                        tile,
                        pixels,
                        work,
                        queued_ms: queued.as_secs_f64() * 1e3,
                        ms: duration.as_secs_f64() * 1e3,
                        inline,
                    });
                }
                Sample::Mismatch { frame, target, max } => {
                    let (cell, tile) = spot(target);
                    self.send(Record::Mismatch {
                        frame,
                        cell,
                        tile,
                        max,
                    });
                }
                Sample::Frame {
                    frame,
                    at,
                    generation,
                    images,
                    missing,
                    previews,
                    uploaded,
                } => {
                    state.sharp = missing + previews == 0;
                    let mut rest = Vec::new();
                    for mut waiting in state.waiting.drain(..) {
                        if waiting.passes == 0 {
                            self.send(Record::Input {
                                event: waiting.event,
                                scheduled_ms: waiting.scheduled_ms,
                                injected_ms: waiting.injected_ms,
                                frame,
                                update_ms: waiting.update_ms,
                            });
                        } else {
                            waiting.passes -= 1;
                            rest.push(waiting);
                        }
                    }
                    state.waiting = rest;
                    let record = Record::Frame {
                        frame,
                        at_ms: self.ms(at),
                        generation,
                        images,
                        missing,
                        previews,
                        uploaded,
                        update_ms: state.update_ms,
                        draw_ms: None,
                    };
                    // A pass that was not drawn goes out without a draw.
                    if let Some(undrawn) = state.pending.replace(Some(record)) {
                        self.send(undrawn);
                    }
                }
            }
        }
    }

    /// Injects `event` as the iced event it stands for, or publishes the
    /// cue; the child's update is timed.
    #[expect(
        clippy::too_many_arguments,
        reason = "the widget's update arguments, passed through"
    )]
    fn inject(
        &mut self,
        state: &mut State,
        timed: replay::Timed,
        now: Instant,
        start: Instant,
        tree: &mut Tree,
        layout: Layout,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let event = match timed.event {
            replay::Event::Move { x, y } => {
                let position = place(x, y, self.planned, layout.bounds());
                state.cursor = Some(position);
                Some(Event::Mouse(mouse::Event::CursorMoved { position }))
            }
            replay::Event::Press(button) => Some(Event::Mouse(mouse::Event::ButtonPressed(
                iced_button(button),
            ))),
            replay::Event::Release(button) => Some(Event::Mouse(mouse::Event::ButtonReleased(
                iced_button(button),
            ))),
            replay::Event::Wheel(delta) => {
                let delta = match delta {
                    Delta::Lines { x, y } => mouse::ScrollDelta::Lines { x, y },
                    Delta::Pixels { x, y } => mouse::ScrollDelta::Pixels { x, y },
                };
                Some(Event::Mouse(mouse::Event::WheelScrolled { delta }))
            }
            replay::Event::Modifiers { ctrl, shift, alt } => {
                let mut modifiers = keyboard::Modifiers::empty();
                modifiers.set(keyboard::Modifiers::CTRL, ctrl);
                modifiers.set(keyboard::Modifiers::SHIFT, shift);
                modifiers.set(keyboard::Modifiers::ALT, alt);
                state.modifiers = modifiers;
                Some(Event::Keyboard(keyboard::Event::ModifiersChanged(
                    modifiers,
                )))
            }
            replay::Event::Append(n) => {
                shell.publish((self.cue)(Cue::Append(n)));
                None
            }
            replay::Event::Resize { width, height } => {
                state.resized = true;
                shell.publish((self.cue)(Cue::Resize { width, height }));
                None
            }
        };
        let scheduled = start + Duration::from_secs_f64(timed.t_ms / 1e3);
        let (update_ms, passes) = match event {
            Some(event) => {
                let begin = Instant::now();
                self.child.update(
                    &mut tree.children[0],
                    &event,
                    layout,
                    cursor(state),
                    renderer,
                    shell,
                    viewport,
                );
                (begin.elapsed().as_secs_f64() * 1e3, 0)
            }
            None => (0.0, 1),
        };
        state.waiting.push(Waiting {
            event: timed.event.to_string(),
            scheduled_ms: self.ms(scheduled),
            injected_ms: self.ms(now),
            update_ms,
            passes,
        });
    }
}

impl<Message> widget::Meta for Driver<'_, Message> {}

impl<Message> Widget<Message, Theme, Renderer> for Driver<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.child.size()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.child));
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.child.layout(&mut tree.children[0], renderer, limits);
        tree.size = tree.children[0].size;
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        _cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // The state leaves the tree while the child updates in it.
        let mut state = std::mem::take(tree.state.downcast_mut::<State>());
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                self.redraw(&mut state, *now, tree, layout, renderer, shell, viewport);
            }
            Event::Window(_) | Event::Waken => {
                self.child.update(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor(&state),
                    renderer,
                    shell,
                    viewport,
                );
                self.drain(&mut state);
            }
            // Real input is not forwarded during a run.
            Event::Mouse(_)
            | Event::Keyboard(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_) => {}
        }
        *tree.state.downcast_mut::<State>() = state;
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let begin = Instant::now();
        self.child.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor(state),
            viewport,
        );
        let ms = begin.elapsed().as_secs_f64() * 1e3;
        if let Some(mut record) = state.pending.take() {
            if let Record::Frame { draw_ms, .. } = &mut record {
                *draw_ms = Some(ms);
            }
            self.send(record);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        self.child
            .mouse_interaction(&tree.children[0], layout, cursor(state), viewport, renderer)
    }
}

impl<Message> Driver<'_, Message> {
    /// One `RedrawRequested(now)` pass: inject every due event, forward
    /// the redraw (timed), join the samples, mark the interaction window,
    /// and either request the next frame or end the run.
    #[expect(
        clippy::too_many_arguments,
        reason = "the widget's update arguments, passed through"
    )]
    fn redraw(
        &mut self,
        state: &mut State,
        now: Instant,
        tree: &mut Tree,
        layout: Layout,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let start = *state.start.get_or_insert_with(|| {
            self.send(system::sys(self.epoch, "start", system::gpu_mb()));
            now
        });
        let elapsed_ms = now.saturating_duration_since(start).as_secs_f64() * 1e3;
        if !state.resized {
            state.widget = layout.bounds().size();
        }
        let events = &self.replay.events;
        while let Some(&timed) = events.get(state.next) {
            if timed.t_ms > elapsed_ms || state.done {
                break;
            }
            if !state.first_input {
                state.first_input = true;
                self.send(system::sys(self.epoch, "first_input", None));
            }
            self.inject(
                state, timed, now, start, tree, layout, renderer, shell, viewport,
            );
            state.next += 1;
        }
        let begin = Instant::now();
        self.child.update(
            &mut tree.children[0],
            &Event::Window(window::Event::RedrawRequested(now)),
            layout,
            cursor(state),
            renderer,
            shell,
            viewport,
        );
        state.update_ms = begin.elapsed().as_secs_f64() * 1e3;
        self.drain(state);
        let end = Duration::from_secs_f64(self.replay.end_ms() / 1e3);
        let after = now.saturating_duration_since(start);
        let replayed = state.next == events.len();
        if replayed && !state.window_end && after >= end + AFTER_LAST {
            state.window_end = true;
            self.send(system::sys(self.epoch, "window_end", None));
        }
        let settled = replayed && after >= end + SETTLE && state.sharp;
        let timed_out = replayed && after >= end + SETTLE + TIMEOUT;
        if !state.done && (settled || timed_out) {
            state.done = true;
            if let Some(record) = state.pending.take() {
                self.send(record);
            }
            self.send(system::sys(self.epoch, "done", system::gpu_mb()));
            self.send(Record::End {
                at_ms: self.ms(now),
                settled,
            });
            shell.publish((self.cue)(Cue::Done {
                widget: [state.widget.width, state.widget.height],
            }));
        } else if !state.done {
            shell.request_redraw();
        }
    }
}

impl<'a, Message: 'a> From<Driver<'a, Message>> for Element<'a, Message, Theme, Renderer> {
    fn from(driver: Driver<'a, Message>) -> Self {
        // `Element::new` is private at the pinned revision; widgets box
        // themselves.
        driver._boxed()
    }
}

/// The synthetic cursor as iced passes cursors.
fn cursor(state: &State) -> mouse::Cursor {
    state
        .cursor
        .map_or(mouse::Cursor::Unavailable, mouse::Cursor::Available)
}

/// The replay's position (`x`, `y`) in a plot of `planned` logical px,
/// placed at the same fractions of `bounds`.
fn place(x: f32, y: f32, planned: Size, bounds: Rectangle) -> iced::Point {
    iced::Point::new(
        bounds.x + x * bounds.width / planned.width,
        bounds.y + y * bounds.height / planned.height,
    )
}

/// A probe target as a raw record names it: a cell's index, or a tile's
/// plot area, column and row.
fn spot(target: Target) -> (Option<u32>, Option<[i32; 3]>) {
    match target {
        Target::Cell(cell) => (Some(cell), None),
        Target::Tile { area, i, j } => {
            (None, Some([i32::try_from(area).unwrap_or(i32::MAX), i, j]))
        }
    }
}

/// The bench's button as iced's.
fn iced_button(button: replay::Button) -> mouse::Button {
    match button {
        replay::Button::Left => mouse::Button::Left,
        replay::Button::Right => mouse::Button::Right,
        replay::Button::Middle => mouse::Button::Middle,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use iced::advanced::renderer::{self, Headless as _};
    use iced::advanced::shell;
    use iced::{Color, Event, Point, Rectangle, Size, mouse, window};
    use iced_runtime::user_interface::{self, UserInterface};
    use rustd3_core::scale;
    use rustd3_figure::Figure;
    use rustd3_iced::Plot;
    use rustd3_iced::plot::{Probe, Render};

    use super::{Driver, place};
    use crate::record::Record;
    use crate::replay::Cue;
    use crate::scenario::{Anchors, Scenario, WINDOW};
    use crate::workload::{Data, Workload};

    /// What the test's app receives: the driver's cues and the x domain
    /// the plot publishes.
    #[derive(Debug, Clone)]
    enum Message {
        Cue(Cue),
        Domain([f64; 2]),
    }

    #[test]
    fn replayed_positions_keep_their_fractions_of_the_plot() {
        let planned = Size::new(1600.0, 1000.0);
        // pan-x's press, right of centre on the top panel.
        let at = |origin, size| place(1100.0, 270.0, planned, Rectangle::new(origin, size));
        assert_eq!(at(Point::ORIGIN, planned), Point::new(1100.0, 270.0));
        assert_eq!(
            at(Point::ORIGIN, Size::new(960.0, 600.0)),
            Point::new(660.0, 162.0)
        );
        // Each axis by its own ratio, from the widget's origin.
        assert_eq!(
            at(Point::new(10.0, 20.0), Size::new(800.0, 1000.0)),
            Point::new(560.0, 290.0)
        );
    }

    #[test]
    fn a_pan_lands_on_the_plot_in_a_window_of_another_size() {
        // The replay is generated for 1600 × 1000 and the window manager
        // gave 960 × 600: the press at x = 1100 would fall outside the
        // widget, and nothing would pan.
        let actual = Size::new(960.0, 600.0);
        let replay = Scenario::PanX.replay(WINDOW, Anchors::STANDARD, 480.0);
        let data = Workload::Trace.data(0.01);
        let Data::Trace(trace) = &data else {
            panic!("trace data");
        };
        let (records, received) = mpsc::channel();
        let (sink, samples) = mpsc::channel();
        let epoch = Instant::now();
        let (replay, samples) = (&replay, &samples);
        let view = move |domain: Option<[f64; 2]>| {
            let mut fig = Figure::new();
            let time = match domain {
                Some(domain) => fig.scale(scale::linear().domain(domain)),
                None => fig.fit(scale::linear()),
            };
            trace.panels(&mut fig, time, "time [s]");
            let plot = Plot::new(fig)
                .scale_factor(1.0)
                .render(Render::Inline)
                .probe(Probe::new(Instant::now, sink.clone()))
                .on_change(time, Message::Domain);
            let records = records.clone();
            Driver::new(plot, replay, WINDOW, records, Message::Cue, samples, epoch)
        };
        let backend = std::env::var("ICED_TEST_BACKEND").ok();
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            renderer::Settings::default(),
            backend.as_deref(),
        ));
        let Some(mut renderer) = renderer else {
            let allowed = std::env::var("RUSTD3_ALLOW_NO_GPU").is_ok_and(|v| v == "1");
            assert!(
                allowed && backend.as_deref() == Some("wgpu"),
                "no headless renderer for backend {backend:?}"
            );
            eprintln!("skipped: wgpu found no device (RUSTD3_ALLOW_NO_GPU=1)");
            return;
        };
        let waker = shell::Waker::new(|| {});
        let mut bus = shell::Bus::new();
        let mut ui = UserInterface::build(
            view(None),
            actual,
            user_interface::Cache::default(),
            &mut renderer,
        );
        let (mut domain, mut domains, mut widget) = (None, Vec::new(), None);
        // Virtual frames at 60 Hz: the replay, the 2 s settle and a margin.
        for k in 0..240_u32 {
            let now = epoch + Duration::from_secs_f64(f64::from(k) / 60.0);
            let redraw = Event::Window(window::Event::RedrawRequested(now));
            let cursor = mouse::Cursor::Unavailable;
            let _ = ui.update(
                &window::Headless,
                &waker,
                &[redraw],
                cursor,
                &mut renderer,
                &mut bus,
            );
            let messages: Vec<Message> = bus.drain().map(|(message, _)| message).collect();
            for message in &messages {
                match *message {
                    Message::Domain(published) => {
                        domain = Some(published);
                        domains.push(published);
                    }
                    Message::Cue(Cue::Done { widget: size }) => widget = Some(size),
                    Message::Cue(Cue::Append(_) | Cue::Resize { .. }) => {}
                }
            }
            if !messages.is_empty() {
                ui = UserInterface::build(view(domain), actual, ui.into_cache(), &mut renderer);
            }
            let style = renderer::Style {
                text_color: Color::BLACK,
            };
            ui.draw(&mut renderer, &iced::Theme::Light, &style, cursor);
            if widget.is_some() {
                break;
            }
        }
        assert_eq!(widget, Some([960.0, 600.0]), "the run ended in the widget");
        // The pan's moves landed on the top plot area: the drag sweeps
        // left, so the x domain moves to later times, its span kept.
        let (Some(&first), Some(&last)) = (domains.first(), domains.last()) else {
            panic!("no domain published: the pan missed the plot areas");
        };
        let span = first[1] - first[0];
        assert!(last[0] - first[0] > 0.1 * span, "{first:?} → {last:?}");
        assert!(((last[1] - last[0]) - span).abs() < 1e-9 * span, "{last:?}");
        let generations: Vec<u64> = received
            .try_iter()
            .filter_map(|record| {
                if let Record::Frame { generation, .. } = record {
                    Some(generation)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            generations.last() > generations.first(),
            "the generation advances: {generations:?}"
        );
    }
}
