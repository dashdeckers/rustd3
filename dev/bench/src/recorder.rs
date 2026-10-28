//! The `record` command's app: the `trace` window, whose real mouse and
//! modifier events are written as a replay, timed from the first one,
//! until Escape.

use std::path::PathBuf;
use std::time::Instant;

use iced::keyboard::key::Named;
use iced::{Element, Event, Subscription, Task, keyboard, mouse};
use rustd3_iced::Plot;

use crate::replay::{self, Button, Delta, Replay, Timed};
use crate::scenario::WINDOW;
use crate::workload::{Data, Workload};

/// The recorder's state.
struct Recorder {
    data: Data,
    events: Vec<Timed>,
    first: Option<Instant>,
    out: PathBuf,
}

/// An event the recorder listens to.
#[derive(Debug, Clone)]
enum Message {
    Input(Event),
}

/// Opens the `trace` window and records into `out` until Escape.
pub fn record(out: PathBuf) -> iced::Result {
    let boot = move || Recorder {
        data: Workload::Trace.data(1.0),
        events: Vec::new(),
        first: None,
        out: out.clone(),
    };
    iced::application(boot, Recorder::update, Recorder::view)
        .title("rustd3-bench record: Escape stops")
        .window_size(WINDOW)
        .subscription(Recorder::subscription)
        .run()
}

impl Recorder {
    fn view(&self) -> Element<'_, Message> {
        Plot::new(self.data.figure(false)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _, _| match event {
            Event::Mouse(_) | Event::Keyboard(_) => Some(Message::Input(event)),
            Event::Window(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_)
            | Event::Waken => None,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let Message::Input(event) = message;
        let now = Instant::now();
        let event = match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                ..
            }) => {
                let replay = Replay {
                    events: std::mem::take(&mut self.events),
                };
                match replay.write(&self.out) {
                    Ok(()) => println!(
                        "{} events written to {}",
                        replay.events.len(),
                        self.out.display()
                    ),
                    Err(error) => eprintln!("rustd3-bench record: {error}"),
                }
                return iced::exit();
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                Some(replay::Event::Modifiers {
                    ctrl: m.control(),
                    shift: m.shift(),
                    alt: m.alt(),
                })
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => Some(replay::Event::Move {
                x: position.x,
                y: position.y,
            }),
            Event::Mouse(mouse::Event::ButtonPressed(b)) => button(b).map(replay::Event::Press),
            Event::Mouse(mouse::Event::ButtonReleased(b)) => button(b).map(replay::Event::Release),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                Some(replay::Event::Wheel(match delta {
                    mouse::ScrollDelta::Lines { x, y } => Delta::Lines { x, y },
                    mouse::ScrollDelta::Pixels { x, y } => Delta::Pixels { x, y },
                }))
            }
            Event::Mouse(_)
            | Event::Keyboard(_)
            | Event::Window(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_)
            | Event::Waken => None,
        };
        if let Some(event) = event {
            let first = *self.first.get_or_insert(now);
            self.events.push(Timed {
                t_ms: now.saturating_duration_since(first).as_secs_f64() * 1e3,
                event,
            });
        }
        Task::none()
    }
}

/// The bench's button for iced's, if it has one.
fn button(button: mouse::Button) -> Option<Button> {
    match button {
        mouse::Button::Left => Some(Button::Left),
        mouse::Button::Right => Some(Button::Right),
        mouse::Button::Middle => Some(Button::Middle),
        mouse::Button::Back | mouse::Button::Forward | mouse::Button::Other(_) => None,
    }
}
