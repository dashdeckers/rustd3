//! The root README's `Plot` example, compiled. A figure is built in `view`
//! and shown by the widget, which zooms and pans its x axis and sends the
//! app the new window. Below this header the code is the README's, with
//! `rustd3_core` and `rustd3_figure` in place of the facade's `rustd3` and
//! `rustd3::figure`. `tests/main/readme.rs` checks that the two match.

use iced::Element;
use rustd3_core::scale;
use rustd3_figure::mark::line;
use rustd3_figure::{Figure, axis};
use rustd3_iced::Plot;

fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

/// The x window the user chose by zooming or panning, once there is one.
#[derive(Default)]
struct App {
    window: Option<[f64; 2]>,
}

#[derive(Debug, Clone)]
enum Message {
    Window([f64; 2]),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Window(window) => self.window = Some(window),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let wave = (0..=2000).map(|i| {
            let t = f64::from(i) / 100.0;
            (t, (-t / 8.0).exp() * t.sin())
        });
        let mut fig = Figure::new();
        let x = match self.window {
            Some(window) => fig.scale(scale::linear().domain(window)),
            None => fig.fit(scale::linear()),
        };
        let y = fig.fit(scale::linear());
        fig.panel(0, 0)
            .mark(line(wave, x, y))
            .axis(axis::bottom(x).label("time [s]"))
            .axis(axis::left(y))
            .zoom_x(x);
        // Wheel to zoom, drag to pan: the widget sends the new x window.
        Plot::new(fig).on_change(x, Message::Window).into()
    }
}
