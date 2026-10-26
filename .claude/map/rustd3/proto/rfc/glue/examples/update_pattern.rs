//! Use case 2: d3's general update pattern (keyed join, staggered enter/update/exit) in iced.

use d3::anim::{Marks, Phase, Timing};
use d3::color::{Rgb, scheme};
use d3::interpolate::Interpolate;
use d3::scale;
use iced::time::{self, Duration, Instant};
use iced::widget::canvas::{self, Frame, Geometry, Text};
use iced::{Color, Fill, Point, Rectangle, Renderer, Size, Subscription, Theme, Widget, mouse, window};

const NAMES: [&str; 8] = ["ant", "bee", "cat", "dog", "eel", "fox", "gnu", "hen"];

/// Everything about one bar that animates. d3 animates DOM attributes; here they are fields.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bar {
    y: f64,
    width: f64,
    opacity: f64,
    value: f64,
}

impl Interpolate for Bar {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        let f = |a: f64, b: f64| a.lerp(&b, t);
        Bar {
            y: f(self.y, to.y),
            width: f(self.width, to.width),
            opacity: f(self.opacity, to.opacity),
            value: f(self.value, to.value),
        }
    }
}

struct App {
    seed: u32,
    marks: Marks<&'static str, Bar>,
    now: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    NewData(Instant),
    Frame(Instant),
}

impl App {
    fn new() -> Self {
        let mut app = Self { seed: 1, marks: Marks::default(), now: Instant::now() };
        app.update(Message::NewData(Instant::now()));
        app
    }

    /// A random top-5 out of eight names, largest first.
    fn next_data(&mut self) -> Vec<(&'static str, f64)> {
        let mut rows: Vec<(&'static str, f64)> = NAMES
            .iter()
            .map(|&n| {
                self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (n, (self.seed >> 8) as f64 / (1u32 << 24) as f64 * 100.0)
            })
            .collect();
        rows.sort_by(|a, b| b.1.total_cmp(&a.1));
        rows.truncate(5);
        rows
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NewData(now) => {
                self.now = now;
                let data = self.next_data();
                let x = scale::linear()
                    .domain([0.0, 100.0])
                    .range([0.0, 500.0]);
                let names = data
                    .iter()
                    .map(|d| d.0);
                let y = scale::band(names)
                    .range([0.0, 250.0])
                    .padding(0.1);
                let targets = data
                    .iter()
                    .map(|&(name, v)| {
                        let y = y
                            .apply(&name)
                            .expect("name is in the domain");
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
                    |b| Bar { width: 0.0, opacity: 0.0, ..*b },
                );
            }
            Message::Frame(now) => {
                self.now = now;
                self.marks.prune(now);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let every = Duration::from_millis(1500);
        let data = time::every(every)
            .map(Message::NewData);
        if self.marks.is_animating(self.now) {
            let frames = window::frames()
                .map(Message::Frame);
            Subscription::batch([data, frames])
        } else {
            data
        }
    }

    fn view(&self) -> impl Widget<Message> {
        canvas::Canvas::new(Chart { app: self }).width(Fill).height(Fill)
    }
}

struct Chart<'a> {
    app: &'a App,
}

impl canvas::Program<Message> for Chart<'_> {
    type State = ();

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let color = scale::ordinal(NAMES, scheme::CATEGORY10);
        let mut frame = Frame::new(renderer, bounds.size());
        for (name, bar, phase) in self.app.marks.values(self.app.now) {
            let c = color.apply(name).copied().unwrap_or(Rgb::hex(0x777777));
            let fill = Color::from_rgba8(c.r as u8, c.g as u8, c.b as u8, bar.opacity as f32);
            frame.fill_rectangle(Point::new(60.0, bar.y as f32), Size::new(bar.width as f32, 40.0), fill);
            if phase == Phase::Present {
                let label = format!("{name} {:.0}", bar.value);
                frame.fill_text(Text { content: label, position: Point::new(4.0, bar.y as f32 + 12.0), ..Text::default() });
            }
        }
        vec![frame.into_geometry()]
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).subscription(App::subscription).run()
}
