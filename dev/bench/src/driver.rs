//! [`Driver`]: the widget that feeds a replay to the `Plot` as if the user
//! produced it, and times the plot's update and draw calls.

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
    /// Drives `child` with `replay`, records to `records`, cues to the app
    /// through `cue`; `samples` receives the child's probe.
    pub fn new(
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
        replay: &'a Replay,
        records: mpsc::Sender<Record>,
        cue: fn(Cue) -> Message,
        samples: &'a mpsc::Receiver<Sample>,
        epoch: Instant,
    ) -> Self {
        Driver {
            child: child.into(),
            replay,
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
                    target: Target::Cell(cell),
                    pixels,
                    work,
                    queued,
                    duration,
                    inline,
                } => self.send(Record::Raster {
                    frame,
                    generation,
                    cell,
                    pixels,
                    work,
                    queued_ms: queued.as_secs_f64() * 1e3,
                    ms: duration.as_secs_f64() * 1e3,
                    inline,
                }),
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
        let origin = layout.bounds().position();
        let event = match timed.event {
            replay::Event::Move { x, y } => {
                let position = iced::Point::new(origin.x + x, origin.y + y);
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
            shell.publish((self.cue)(Cue::Done));
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

/// The bench's button as iced's.
fn iced_button(button: replay::Button) -> mouse::Button {
    match button {
        replay::Button::Left => mouse::Button::Left,
        replay::Button::Right => mouse::Button::Right,
        replay::Button::Middle => mouse::Button::Middle,
    }
}
