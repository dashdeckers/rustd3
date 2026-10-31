//! An iced canvas drawn with the core level: d3's scales, line and bottom
//! axis drawn into an iced canvas, with iced writing the axis labels
//! (`GUIDE.md`, "The core level in an iced canvas").

use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Stroke, Text};
use iced::{Fill, Point, Rectangle, Renderer, Theme, Vector, Widget, mouse};
use rustd3_core::{axis, scale, shape};

fn main() -> iced::Result {
    iced::run(Wave::update, Wave::view)
}

/// Ten seconds of a sine wave over a time axis.
#[derive(Default)]
struct Wave;

impl Wave {
    fn update(&mut self, _message: ()) {}

    fn view(&self) -> impl Widget<()> {
        canvas(Wave).width(Fill).height(Fill)
    }
}

impl canvas::Program<()> for Wave {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (width, height) = (f64::from(bounds.width), f64::from(bounds.height));
        // Scales are values: build them for this size, use them, drop them.
        let x = scale::linear()
            .domain([0.0, 10.0])
            .range([40.0, width - 40.0]);
        let y = scale::linear()
            .domain([-1.0, 1.0])
            .range([height - 60.0, 20.0]);
        let palette = theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let wave = rustd3_iced::path(|p| {
            let points = (0..=200).map(|i| {
                let t = f64::from(i) / 20.0;
                (x.apply(t), y.apply(t.sin()))
            });
            shape::line().draw(points, p);
        });
        let line = Stroke::default().with_width(2.0);
        frame.stroke(&wave, line.with_color(palette.primary.base.color));
        // The axis draws its line and ticks; iced writes its labels.
        let ink = palette.background.base.text;
        let bottom = axis::bottom(&x);
        frame.translate(Vector::new(0.0, (height - 40.0) as f32));
        let ticks = rustd3_iced::path(|p| bottom.draw(p));
        frame.stroke(&ticks, Stroke::default().with_color(ink));
        for label in bottom.labels() {
            frame.fill_text(Text {
                content: label.get_text().to_owned(),
                position: Point::new(label.get_x() as f32, label.get_y() as f32),
                color: ink,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Top,
                ..Text::default()
            });
        }
        vec![frame.into_geometry()]
    }
}
