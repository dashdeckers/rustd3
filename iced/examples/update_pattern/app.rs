//! The general update pattern at the core level: a top-five
//! bar chart that re-ranks on new data, bars entering from zero width,
//! moving to their new rows and shrinking out, staggered 20 ms apart and
//! interrupted by the next update mid-flight. Everything that animates is
//! a field of [`Bar`]; `anim::Marks` keeps one per name, through enter,
//! update and exit.

use std::time::Duration;

use iced::advanced::text::Alignment;
use iced::alignment::Vertical;
use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Text};
use iced::{Color, Fill, Pixels, Point, Rectangle, Renderer, Size, Subscription, Theme, mouse};
use rustd3_core::anim::{Instant, Marks, Phase, Timing};
use rustd3_core::color::Rgb;
use rustd3_core::interpolate::Interpolate;
use rustd3_core::scale::{self, Band};
use rustd3_core::scheme;

/// The names the data draws from.
const NAMES: [&str; 8] = ["ant", "bee", "cat", "dog", "eel", "fox", "gnu", "hen"];
/// Where the bars start, right of their labels.
const BARS_X: f32 = 60.0;
/// The labels' size in px.
const LABEL_PX: f32 = 12.0;

/// Everything about one bar that animates: d3 tweens DOM attributes; here
/// they are fields, blended by the derive.
#[derive(Debug, Clone, Copy, PartialEq, Interpolate)]
pub struct Bar {
    /// The top of its row, in px.
    pub y: f64,
    /// Its length, in px.
    pub width: f64,
    /// Its opacity, 0 to 1.
    pub opacity: f64,
    /// The value it shows.
    pub value: f64,
}

/// The app: the data's generator, the marks and the latest frame's time.
pub struct App {
    seed: u32,
    marks: Marks<&'static str, Bar>,
    /// `view` reads no clock: the latest instant a message brought.
    now: Instant,
}

/// What drives the app.
#[derive(Debug, Clone)]
pub enum Message {
    /// New data arrived at this instant.
    NewData(Instant),
    /// A frame is drawn at this instant.
    Frame(Instant),
}

/// The bars and their labels.
struct Chart<'a> {
    app: &'a App,
}

impl App {
    /// The app at `now` with its first data (seed 1): five bars entering.
    pub fn new(now: Instant) -> Self {
        let mut app = App {
            seed: 1,
            marks: Marks::default(),
            now,
        };
        app.update(Message::NewData(now));
        app
    }

    /// New data re-ranks the bars: entering ones grow from zero width in
    /// their row, the others move to their new row and length, and those
    /// out of the top five shrink and fade where they are. A frame stores
    /// its instant and drops the exits that have finished.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::NewData(now) => {
                self.now = now;
                let data = self.next_data();
                let x = scale::linear().domain([0.0, 100.0]).range([0.0, 500.0]);
                let y = rows(data.iter().map(|d| d.0));
                let targets = data.iter().map(|&(name, v)| {
                    let y = y.apply(&name).expect("invariant: name is in the domain");
                    let bar = Bar {
                        y,
                        width: x.apply(v),
                        opacity: 1.0,
                        value: v,
                    };
                    (name, bar)
                });
                let timing = Timing {
                    duration: Duration::from_millis(750),
                    stagger: Duration::from_millis(20),
                    ..Timing::default()
                };
                // enter: grow from zero width in its row;
                // exit: shrink and fade where it is.
                self.marks.update(
                    targets,
                    now,
                    timing,
                    |t| Bar { width: 0.0, ..*t },
                    |b| Bar {
                        width: 0.0,
                        opacity: 0.0,
                        ..*b
                    },
                );
            }
            Message::Frame(now) => {
                self.now = now;
                self.marks.prune(now);
            }
        }
    }

    /// Frames only while something moves; at rest the app takes none, so
    /// it does not redraw at the display's rate.
    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_animating() {
            iced::window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        }
    }

    /// The chart filling the window.
    pub fn view(&self) -> impl iced::Widget<Message> + '_ {
        canvas(Chart { app: self }).width(Fill).height(Fill)
    }

    /// Every bar's name, value and phase at the latest instant, in the
    /// order the names first entered.
    pub fn bars(&self) -> Vec<(&'static str, Bar, Phase)> {
        self.marks
            .values(self.now)
            .map(|(&name, bar, phase)| (name, bar, phase))
            .collect()
    }

    /// Some bar has not reached its target at the latest instant.
    pub fn is_animating(&self) -> bool {
        self.marks.is_animating(self.now)
    }

    /// A random top five of the eight names, largest first: each name's
    /// value is `(seed >> 8) / 2²⁴ · 100` after one step of the LCG.
    fn next_data(&mut self) -> Vec<(&'static str, f64)> {
        let mut rows: Vec<(&'static str, f64)> = NAMES
            .iter()
            .map(|&name| {
                self.seed = self
                    .seed
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                (
                    name,
                    f64::from(self.seed >> 8) / f64::from(1_u32 << 24) * 100.0,
                )
            })
            .collect();
        rows.sort_by(|a, b| b.1.total_cmp(&a.1));
        rows.truncate(5);
        rows
    }
}

impl canvas::Program<Message> for Chart<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let color = scale::ordinal(NAMES, scheme::CATEGORY10);
        let bars = self.app.bars();
        let present = bars.iter().filter(|b| b.2 == Phase::Present);
        let height = rows(present.map(|b| b.0)).bandwidth() as f32;
        let mut frame = Frame::new(renderer, bounds.size());
        for (name, bar, phase) in bars {
            let c = color
                .apply(&name)
                .expect("invariant: the color domain is every name");
            let top = bar.y as f32;
            let size = Size::new(bar.width as f32, height);
            frame.fill_rectangle(Point::new(BARS_X, top), size, rgb(c, bar.opacity));
            if phase == Phase::Present {
                frame.fill_text(Text {
                    content: format!("{name} {:.0}", bar.value),
                    position: Point::new(4.0, top + height / 2.0),
                    size: Pixels(LABEL_PX),
                    align_x: Alignment::Left,
                    align_y: Vertical::Center,
                    ..Text::default()
                });
            }
        }
        vec![frame.into_geometry()]
    }
}

/// The rows of `names`, top to bottom over 250 px: d3's band scale with
/// padding 0.1.
fn rows(names: impl IntoIterator<Item = &'static str>) -> Band<&'static str> {
    scale::band(names).range([0.0, 250.0]).padding(0.1)
}

/// A core color as iced's, at `opacity`.
fn rgb(c: &Rgb, opacity: f64) -> Color {
    let channel = |v: f64| (v / 255.0) as f32;
    Color::from_rgba(channel(c.r), channel(c.g), channel(c.b), opacity as f32)
}
