//! Use case 1: d3's "focus + context" (brush and zoom) as an iced app over the prototype API.

use d3::axis::{self, Orient};
use d3::color::{Rgb, scheme};
use d3::interact::{BrushEvent, BrushX, Rescale, Transform, Zoom};
use d3::scale::{self, Linear};
use d3::shape::{self, Curve};
use d3::time::{self, Time};
use glue::{path, pointer};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, Stroke, Text};
use iced::widget::{canvas as canvas_widget, column};
use iced::{Color, Fill, Point, Rectangle, Renderer, Size, Theme, Widget, mouse};
use jiff::{SignedDuration, Timestamp, tz::TimeZone};

const MARGIN: f32 = 40.0;

struct Trace {
    name: &'static str,
    points: Vec<(Timestamp, f64)>,
}

struct App {
    traces: Vec<Trace>,
    tz: TimeZone,
    full: [Timestamp; 2],
    /// The one source of truth: the brush and the zoom both derive from it.
    visible: [Timestamp; 2],
}

#[derive(Debug, Clone)]
enum Message {
    Show([Timestamp; 2]),
}

impl App {
    fn new() -> Self {
        let start: Timestamp = "2026-09-01T00:00:00Z".parse().expect("valid timestamp");
        let mut seed = 7u32;
        let mut noise = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            seed as f64 / u32::MAX as f64 - 0.5
        };
        let traces = ["sensor a", "sensor b", "sensor c"]
            .into_iter()
            .enumerate()
            .map(|(s, name)| Trace {
                name,
                points: (0..24 * 30)
                    .map(|h| {
                        let t = start + SignedDuration::from_hours(h);
                        (t, 10.0 + 3.0 * s as f64 + (h as f64 / 24.0).sin() * 2.0 + noise())
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        let full = [start, start + SignedDuration::from_hours(24 * 30 - 1)];
        Self { traces, tz: TimeZone::UTC, full, visible: full }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Show([a, b]) => self.visible = [a.max(self.full[0]), b.min(self.full[1])],
        }
    }

    fn view(&self) -> impl Widget<Message> {
        column![
            canvas_widget(Focus { app: self }).width(Fill).height(340),
            canvas_widget(Context { app: self }).width(Fill).height(90),
        ]
    }

    fn x(&self, domain: [Timestamp; 2], width: f32) -> Time {
        time::time(self.tz.clone()).domain(domain).range([0.0, width as f64])
    }

    fn y(&self, height: f32) -> Linear {
        let values = self.traces.iter().flat_map(|s| s.points.iter().map(|p| p.1));
        let (lo, hi) = values.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)));
        scale::linear().domain([lo, hi]).range([height as f64, 0.0]).nice(5)
    }

    fn color(&self) -> scale::Ordinal<&'static str, Rgb> {
        scale::ordinal(self.traces.iter().map(|s| s.name), scheme::CATEGORY10)
    }

    fn draw_lines(&self, frame: &mut Frame, x: &Time, y: &Linear, width: f32) {
        let color = self.color();
        for s in &self.traces {
            let line = path(|p| {
                let points = s.points
                    .iter()
                    .map(|&(t, v)| (x.apply(t), y.apply(v)));
                shape::line()
                    .curve(Curve::MonotoneX)
                    .draw(points, p)
            });
            let c = rgb(color.apply(&s.name));
            let stroke = Stroke::default()
                .with_color(c)
                .with_width(width);
            frame.stroke(&line, stroke);
        }
    }
}

/// The detailed view: wheel to zoom, drag to pan.
struct Focus<'a> {
    app: &'a App,
}

impl canvas::Program<Message> for Focus<'_> {
    type State = Zoom;

    fn update(
        &self,
        zoom: &mut Zoom,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let input = pointer(event, bounds, cursor)?;
        let width = bounds.width - MARGIN;
        let full = self.app.x(self.app.full, width);
        let [v0, v1] = self.app.visible
            .map(|t| full.apply(t));
        let [r0, r1] = full.get_range();
        let k = (r1 - r0) / (v1 - v0);
        let now = Transform {
            k,
            x: -v0 * k,
            y: 0.0,
        };
        let next = zoom.handle(input, now)?;
        let window = full
            .rescale_x(&next)
            .get_domain();
        let show = Message::Show(window);
        let action = Action::publish(show)
            .and_capture();
        Some(action)
    }

    fn draw(&self, _: &Zoom, renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let (w, h) = (bounds.width - MARGIN, bounds.height - 24.0);
        let x = self.app.x(self.app.visible, w);
        let y = self.app.y(h);
        let mut frame = Frame::new(renderer, bounds.size());
        // The clip region is in frame coordinates, and the clipped frame starts untranslated.
        frame.with_clip(Rectangle::new(Point::new(MARGIN, 0.0), Size::new(w, h)), |f| {
            f.translate(iced::Vector::new(MARGIN, 0.0));
            self.app.draw_lines(f, &x, &y, 1.5);
        });
        // tiny-skia ignored the clip in headless renders, so mask the left margin as well.
        frame.fill_rectangle(Point::ORIGIN, Size::new(MARGIN, h), Color::WHITE);
        frame.translate(iced::Vector::new(MARGIN, 0.0));
        draw_axis(&mut frame, &axis::axis(Orient::Bottom, &x, 8), Point::new(0.0, h));
        draw_axis(&mut frame, &axis::axis(Orient::Left, &y, 5), Point::ORIGIN);
        vec![frame.into_geometry()]
    }
}

/// The overview: drag to brush a window, drag the window to move it.
struct Context<'a> {
    app: &'a App,
}

impl canvas::Program<Message> for Context<'_> {
    type State = Option<BrushX>;

    fn update(&self, brush: &mut Option<BrushX>, event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Action<Message>> {
        let input = pointer(event, bounds, cursor)?;
        let x = self.app.x(self.app.full, bounds.width - MARGIN);
        let brush = brush.get_or_insert_with(|| BrushX::new(x.get_range()));
        // Between gestures, follow the window the app shows, so zooming and brushing never disagree.
        // During one, the view may not have caught up with our own messages yet: leave it alone.
        if !brush.is_active() {
            let shown = self.app.visible.map(|t| x.apply(t));
            brush.set_selection((self.app.visible != self.app.full).then_some(shown));
        }
        match brush.handle(input)? {
            BrushEvent::Brush(Some([a, b])) | BrushEvent::End(Some([a, b])) => {
                Some(Action::publish(Message::Show([x.invert(a), x.invert(b)])).and_capture())
            }
            BrushEvent::End(None) => Some(Action::publish(Message::Show(self.app.full))),
            _ => Some(Action::capture()),
        }
    }

    fn draw(&self, _: &Option<BrushX>, renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<Geometry> {
        let (w, h) = (bounds.width - MARGIN, bounds.height - 24.0);
        let x = self.app.x(self.app.full, w);
        let y = self.app.y(h);
        let mut frame = Frame::new(renderer, bounds.size());
        frame.translate(iced::Vector::new(MARGIN, 0.0));
        self.app.draw_lines(&mut frame, &x, &y, 1.0);
        if self.app.visible != self.app.full {
            let [a, b] = self.app.visible.map(|t| x.apply(t) as f32);
            frame.fill_rectangle(Point::new(a, 0.0), Size::new(b - a, h), Color::from_rgba(0.3, 0.5, 0.8, 0.2));
        }
        draw_axis(&mut frame, &axis::axis(Orient::Bottom, &x, 8), Point::new(0.0, h));
        vec![frame.into_geometry()]
    }
}

/// Strokes an axis's lines and writes its labels: the only text-aware part.
fn draw_axis(frame: &mut Frame, a: &axis::Axis, origin: Point) {
    frame.with_save(|f| {
        f.translate(iced::Vector::new(origin.x, origin.y));
        f.stroke(&path(|p| a.draw(p)), Stroke::default().with_color(Color::from_rgb(0.4, 0.4, 0.4)));
        for t in &a.ticks {
            let position = match a.orient {
                Orient::Bottom => Point::new(t.position as f32, 9.0),
                _ => Point::new(-34.0, t.position as f32 - 6.0),
            };
            f.fill_text(Text { content: t.label.clone(), position, size: 11.0.into(), ..Text::default() });
        }
    });
}

fn rgb(c: Option<&Rgb>) -> Color {
    let c = c.copied().unwrap_or(Rgb::hex(0x777777));
    Color::from_rgb8(c.r as u8, c.g as u8, c.b as u8)
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Event as IcedEvent, mouse::ScrollDelta};
    use iced_test::Simulator;

    const SIZE: (f32, f32) = (900.0, 430.0);
    const OUT: &str = "/tmp/claude-0/-home-claude/1c968e93-2b23-5461-9237-760a997174fd/scratchpad/shots";

    fn span(app: &App) -> i64 {
        app.visible[1].duration_since(app.visible[0]).as_secs()
    }

    #[test]
    fn wheel_zooms_the_focus_and_brush_follows() {
        let mut app = App::new();
        let mut ui = Simulator::with_size(Default::default(), SIZE, app.view());
        ui.snapshot(&Theme::Light).unwrap().matches_image(format!("{OUT}/focus_full")).unwrap();
        ui.point_at(Point::new(450.0, 150.0));
        ui.scroll(ScrollDelta::Lines { x: 0.0, y: 20.0 });
        let before = span(&app);
        for m in ui.into_messages() {
            app.update(m);
        }
        assert!(span(&app) < before , "zooming in shows a shorter window: {} vs {before}", span(&app));
        let mut ui = Simulator::with_size(Default::default(), SIZE, app.view());
        ui.snapshot(&Theme::Light).unwrap().matches_image(format!("{OUT}/focus_zoomed")).unwrap();
    }

    #[test]
    fn brushing_the_context_sets_the_window() {
        let mut app = App::new();
        let mut ui = Simulator::with_size(Default::default(), SIZE, app.view());
        let at = |ui: &mut Simulator<'_, Message>, x: f32, e: mouse::Event| {
            ui.point_at(Point::new(x, 380.0));
            ui.simulate([IcedEvent::Mouse(e)]);
        };
        at(&mut ui, 300.0, mouse::Event::ButtonPressed(mouse::Button::Left));
        at(&mut ui, 520.0, mouse::Event::CursorMoved { position: Point::new(520.0, 380.0) });
        at(&mut ui, 520.0, mouse::Event::ButtonReleased(mouse::Button::Left));
        for m in ui.into_messages() {
            app.update(m);
        }
        let days = span(&app) as f64 / 86_400.0;
        assert!((7.0..9.5).contains(&days), "220 px of an 860 px, 30-day axis is about 7.7 days, got {days:.2}");
        let mut ui = Simulator::with_size(Default::default(), SIZE, app.view());
        ui.snapshot(&Theme::Light).unwrap().matches_image(format!("{OUT}/focus_brushed")).unwrap();
    }
}
