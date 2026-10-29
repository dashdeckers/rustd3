//! [`Replay`]: time-stamped input a run feeds the `Plot` widget, stored as
//! JSON Lines, one [`Timed`] event per line.

use std::path::Path;
use std::{fmt, fs, io};

use serde::{Deserialize, Serialize};

/// The input of one run, in the order it is injected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    /// Events by increasing `t_ms`.
    pub events: Vec<Timed>,
}

/// One event at `t_ms` after the run's first frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Timed {
    /// When the event is due, in ms after the first frame.
    pub t_ms: f64,
    /// What happens.
    pub event: Event,
}

/// An input event, in logical px relative to the plot's top left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// The pointer moves to (x, y).
    Move {
        /// Logical px from the plot's left edge.
        x: f32,
        /// Logical px from the plot's top edge.
        y: f32,
    },
    /// A button goes down.
    Press(Button),
    /// A button goes up.
    Release(Button),
    /// A wheel or touchpad scroll.
    Wheel(Delta),
    /// The keyboard modifiers change. Missing flags default to false.
    Modifiers {
        /// Control.
        #[serde(default)]
        ctrl: bool,
        /// Shift.
        #[serde(default)]
        shift: bool,
        /// Alt.
        #[serde(default)]
        alt: bool,
    },
    /// The app appends this many samples to every channel.
    Append(u32),
    /// The app resizes the window to this logical size.
    Resize {
        /// Logical width.
        width: f32,
        /// Logical height.
        height: f32,
    },
    /// The app resizes the window to these shares of its real size: the
    /// size the window manager gave it, before the scenario's first resize.
    ResizeBy {
        /// A share of the real width.
        width: f32,
        /// A share of the real height.
        height: f32,
    },
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Button {
    /// The primary button.
    Left,
    /// The secondary button.
    Right,
    /// The wheel button.
    Middle,
}

/// A scroll amount.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delta {
    /// Wheel lines (positive y scrolls up).
    Lines {
        /// Horizontal lines.
        x: f32,
        /// Vertical lines.
        y: f32,
    },
    /// High-resolution pixels (touchpads).
    Pixels {
        /// Horizontal logical px.
        x: f32,
        /// Vertical logical px.
        y: f32,
    },
}

/// What the driver publishes to the app.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue {
    /// Append this many samples to every channel.
    Append(u32),
    /// Resize the window to this logical size.
    Resize {
        /// Logical width.
        width: f32,
        /// Logical height.
        height: f32,
    },
    /// The replay is over and every slot is sharp (or the settle timed
    /// out): write the records and exit.
    Done {
        /// The plot's logical size the replay's positions were placed in:
        /// the last before the scenario's own first resize.
        widget: [f32; 2],
    },
}

/// Why a replay file cannot be read.
#[derive(Debug)]
pub enum Error {
    /// The file cannot be read or written.
    Io {
        /// The file.
        path: String,
        /// What failed.
        source: io::Error,
    },
    /// A line is not a timed event.
    Line {
        /// 1-based line number.
        line: usize,
        /// The rejected text.
        text: String,
        /// What serde said.
        source: serde_json::Error,
    },
    /// Times go backwards, or one is not finite.
    Order {
        /// 1-based line number.
        line: usize,
        /// The rejected time.
        t_ms: f64,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{path}: {source}"),
            Error::Line { line, text, source } => write!(f, "line {line}: {source}: {text}"),
            Error::Order { line, t_ms } => {
                write!(
                    f,
                    "line {line}: t_ms {t_ms} is not finite or before the last"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl Replay {
    /// The replay as JSON Lines.
    pub fn to_jsonl(&self) -> String {
        let mut out = String::new();
        for timed in &self.events {
            let line = serde_json::to_string(timed).expect("invariant: events serialize");
            out.push_str(&line);
            out.push('\n');
        }
        out
    }

    /// Parses JSON Lines; blank lines are skipped and times must not
    /// decrease.
    pub fn from_jsonl(text: &str) -> Result<Replay, Error> {
        let mut events: Vec<Timed> = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            if raw.trim().is_empty() {
                continue;
            }
            let timed: Timed = serde_json::from_str(raw).map_err(|source| Error::Line {
                line: index + 1,
                text: raw.to_owned(),
                source,
            })?;
            let earlier = events.last().is_some_and(|last| timed.t_ms < last.t_ms);
            if earlier || !timed.t_ms.is_finite() {
                return Err(Error::Order {
                    line: index + 1,
                    t_ms: timed.t_ms,
                });
            }
            events.push(timed);
        }
        Ok(Replay { events })
    }

    /// Reads a replay file.
    pub fn read(path: &Path) -> Result<Replay, Error> {
        let text = fs::read_to_string(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })?;
        Replay::from_jsonl(&text)
    }

    /// Writes the replay to `path`.
    pub fn write(&self, path: &Path) -> Result<(), Error> {
        fs::write(path, self.to_jsonl()).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })
    }

    /// When the last event is due, 0 for none.
    pub fn end_ms(&self) -> f64 {
        self.events.last().map_or(0.0, |timed| timed.t_ms)
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Event::Move { .. } => "move",
            Event::Press(_) => "press",
            Event::Release(_) => "release",
            Event::Wheel(Delta::Lines { .. }) => "wheel_lines",
            Event::Wheel(Delta::Pixels { .. }) => "wheel_pixels",
            Event::Modifiers { .. } => "modifiers",
            Event::Append(_) => "append",
            Event::Resize { .. } => "resize",
            Event::ResizeBy { .. } => "resize_by",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, Delta, Event, Replay, Timed};

    #[test]
    fn replays_round_trip_through_json() {
        let replay = Replay {
            events: vec![
                Timed {
                    t_ms: 0.0,
                    event: Event::Move { x: 120.0, y: 80.0 },
                },
                Timed {
                    t_ms: 8.0,
                    event: Event::Press(Button::Left),
                },
                Timed {
                    t_ms: 16.0,
                    event: Event::Wheel(Delta::Lines { x: 0.0, y: -1.0 }),
                },
                Timed {
                    t_ms: 16.0,
                    event: Event::Wheel(Delta::Pixels { x: 0.0, y: -6.5 }),
                },
                Timed {
                    t_ms: 30.0,
                    event: Event::Modifiers {
                        ctrl: true,
                        shift: false,
                        alt: false,
                    },
                },
                Timed {
                    t_ms: 100.0,
                    event: Event::Append(1000),
                },
                Timed {
                    t_ms: 500.0,
                    event: Event::Resize {
                        width: 1600.0,
                        height: 1000.0,
                    },
                },
                Timed {
                    t_ms: 516.0,
                    event: Event::ResizeBy {
                        width: 0.85,
                        height: 0.85,
                    },
                },
                Timed {
                    t_ms: 600.0,
                    event: Event::Release(Button::Middle),
                },
            ],
        };
        let text = replay.to_jsonl();
        assert_eq!(Replay::from_jsonl(&text).unwrap(), replay);
        // The documented lines parse as written, missing modifiers false.
        let documented = concat!(
            r#"{"t_ms": 0.0, "event": {"move": {"x": 120.0, "y": 80.0}}}"#,
            "\n",
            r#"{"t_ms": 8.0, "event": {"press": "left"}}"#,
            "\n",
            r#"{"t_ms": 16.0, "event": {"wheel": {"lines": {"x": 0.0, "y": -1.0}}}}"#,
            "\n",
            r#"{"t_ms": 16.0, "event": {"wheel": {"pixels": {"x": 0.0, "y": -6.5}}}}"#,
            "\n",
            r#"{"t_ms": 30.0, "event": {"modifiers": {"ctrl": true}}}"#,
            "\n",
            r#"{"t_ms": 100.0, "event": {"append": 1000}}"#,
            "\n",
            r#"{"t_ms": 500.0, "event": {"resize": {"width": 1600.0, "height": 1000.0}}}"#,
            "\n",
            r#"{"t_ms": 516.0, "event": {"resize_by": {"width": 0.85, "height": 0.85}}}"#,
        );
        let parsed = Replay::from_jsonl(documented).unwrap();
        assert_eq!(parsed.events[..8], replay.events[..8]);
    }

    #[test]
    fn replays_reject_times_that_go_backwards() {
        let text = concat!(
            r#"{"t_ms": 5.0, "event": {"append": 1}}"#,
            "\n",
            r#"{"t_ms": 4.0, "event": {"append": 1}}"#,
        );
        assert!(Replay::from_jsonl(text).is_err());
        assert!(Replay::from_jsonl(r#"{"t_ms": 1.0, "event": {"jump": 1}}"#).is_err());
    }
}
