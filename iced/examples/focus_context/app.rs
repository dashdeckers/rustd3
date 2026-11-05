//! Focus and context at the core level: three sensor traces
//! over 30 days in a detail view (wheel to zoom, drag to pan) above an
//! overview (drag to brush a window, drag the window to move it). One
//! field, `visible`, is the window both views show; the zoom and the brush
//! publish the window they want and derive their own state from it.

use iced::advanced::text::Alignment;
use iced::alignment::Vertical;
use iced::widget::canvas::{Action, Event, Frame, Geometry, Stroke, Text};
use iced::widget::{canvas, column};
use iced::{Color, Fill, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};
use rustd3_core::axis::{self, Anchor};
use rustd3_core::color::Rgb;
use rustd3_core::interact::{self, BrushEvent, BrushX, Rescale, Zoom};
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_core::shape::{self, Curve};
use rustd3_core::time::{self, Time};
use rustd3_core::{array, scheme};
use rustd3_iced::pointer::Tracker;

/// The left margin of both views, where the focus's y labels go.
pub const MARGIN: f32 = 40.0;
/// The right margin of both views, where half of the rightmost x label
/// goes.
pub const RIGHT: f32 = 24.0;
/// The focus view's height in logical px.
pub const FOCUS_HEIGHT: f32 = 340.0;
/// The overview's height in logical px.
pub const OVERVIEW_HEIGHT: f32 = 90.0;

/// Room under each view's plot for its bottom axis.
const AXIS_ROOM: f32 = 24.0;
/// Room above each view's plot for half a tick label.
const TOP: f64 = 8.0;
/// The tick labels' size in px.
const LABEL_PX: f64 = 11.0;
/// d3's `dy` of a label centred on its tick (vertical axes), in em.
const MIDDLE_EM: f64 = 0.32;
/// The axes' lines.
const AXIS_INK: Color = Color::from_rgb(0.4, 0.4, 0.4);
/// The overview's band over the visible window.
const BAND: Color = Color::from_rgba(0.3, 0.5, 0.8, 0.2);

/// One sensor's hourly values.
pub struct Trace {
    /// The legend name, also the color key.
    pub name: &'static str,
    /// Hourly instants and values.
    pub points: Vec<(Timestamp, f64)>,
}

/// The app: the traces and the window both views show.
pub struct App {
    traces: Vec<Trace>,
    tz: TimeZone,
    full: [Timestamp; 2],
    /// The one source of truth: the brush and the zoom derive from it.
    visible: [Timestamp; 2],
}

/// What the views ask of the app.
#[derive(Debug, Clone)]
pub enum Message {
    /// Show this window.
    Show([Timestamp; 2]),
}

/// The detail view: the visible window; wheel to zoom, drag to pan.
struct Focus<'a> {
    app: &'a App,
}

/// The overview: all 30 days and the visible window as a band; drag to
/// brush a window, drag the band to move it.
struct Overview<'a> {
    app: &'a App,
}

impl Default for App {
    /// Three traces of 720 hourly points from 2026-09-01 in UTC, all
    /// visible.
    fn default() -> Self {
        let start: Timestamp = "2026-09-01T00:00:00Z"
            .parse()
            .expect("invariant: a valid RFC 3339 instant");
        let mut seed = 7_u32;
        let mut noise = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            f64::from(seed) / f64::from(u32::MAX) - 0.5
        };
        let traces = ["sensor a", "sensor b", "sensor c"]
            .into_iter()
            .zip(0_u32..)
            .map(|(name, s)| Trace {
                name,
                points: (0..720_i32)
                    .map(|h| {
                        let t = start + SignedDuration::from_hours(h.into());
                        let wave = 2.0 * (f64::from(h) / 24.0).sin();
                        (t, 10.0 + 3.0 * f64::from(s) + wave + noise())
                    })
                    .collect(),
            })
            .collect();
        let full = [start, start + SignedDuration::from_hours(719)];
        App {
            traces,
            tz: TimeZone::UTC,
            full,
            visible: full,
        }
    }
}

impl App {
    /// [`App::default`]: 3 traces, 720 hourly points.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the window a view asks for, kept inside the full span: moved
    /// in when it reaches past an end, cut to the full span when longer.
    /// A window of no span is ignored.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Show([a, b]) => {
                let [start, end] = self.full;
                let span = b.duration_since(a).min(end.duration_since(start));
                if span.is_positive() {
                    let first = a.clamp(start, end - span);
                    self.visible = [first, first + span];
                }
            }
        }
    }

    /// The focus above the overview.
    pub fn view(&self) -> impl iced::Widget<Message> + '_ {
        column![
            canvas(Focus { app: self }).width(Fill).height(FOCUS_HEIGHT),
            canvas(Overview { app: self })
                .width(Fill)
                .height(OVERVIEW_HEIGHT),
        ]
    }

    /// The window the views show.
    pub fn visible(&self) -> [Timestamp; 2] {
        self.visible
    }

    /// The span of the data, from the first hour to the last.
    pub fn full(&self) -> [Timestamp; 2] {
        self.full
    }

    /// x over `domain` onto a plot `width` px wide, in the app's zone.
    fn x(&self, domain: [Timestamp; 2], width: f32) -> Time {
        time::time(self.tz.clone())
            .domain(domain)
            .range([0.0, f64::from(width)])
    }

    /// y over every value, nice, onto a plot `height` px high below
    /// [`TOP`].
    fn y(&self, height: f32) -> Linear {
        let values = self
            .traces
            .iter()
            .flat_map(|s| s.points.iter().map(|p| p.1));
        scale::linear()
            .domain(array::extent(values).unwrap_or([0.0, 1.0]))
            .range([f64::from(height), TOP])
            .nice(5)
    }

    /// Each trace as a monotone line through `x` and `y`, `width` px wide,
    /// in its category color.
    fn draw_lines(&self, frame: &mut Frame, x: &Time, y: &Linear, width: f32) {
        let color = scale::ordinal(self.traces.iter().map(|s| s.name), scheme::CATEGORY10);
        for s in &self.traces {
            let line = rustd3_iced::path(|p| {
                let points = s.points.iter().map(|&(t, v)| (x.apply(t), y.apply(v)));
                shape::line().curve(Curve::MonotoneX).draw(points, p)
            });
            let c = color
                .apply(&s.name)
                .expect("invariant: the color domain is the trace names");
            frame.stroke(
                &line,
                Stroke::default().with_color(rgb(c)).with_width(width),
            );
        }
    }
}

impl canvas::Program<Message> for Focus<'_> {
    /// d3's zoom behavior, made at the first event, and the tracker that
    /// feeds it.
    type State = (Option<Zoom>, Tracker);

    /// A wheel step or a drag on the plot publishes the window the zoom
    /// makes of the visible one.
    fn update(
        &self,
        (zoom, tracker): &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        // Pointer x in the x scale's range.
        let plot = plot(bounds);
        let zoom = zoom.get_or_insert_with(|| Zoom::default().scale_extent([1.0, 50.0]));
        let Some(input) = tracker.translate(event, plot, cursor) else {
            return waiting(tracker);
        };
        let full = self.app.x(self.app.full, plot.width);
        let [v0, v1] = self.app.visible.map(|t| full.apply(t));
        let [r0, r1] = full.get_range();
        let k = (r1 - r0) / (v1 - v0);
        let now = interact::Transform {
            k,
            x: -v0 * k,
            y: 0.0,
        };
        let Some(next) = zoom.handle(input, now) else {
            return waiting(tracker);
        };
        let window = full.rescale_x(&next).get_domain();
        Some(Action::publish(Message::Show(window)).and_capture())
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (w, h) = (plot(bounds).width, bounds.height - AXIS_ROOM);
        let x = self.app.x(self.app.visible, w);
        let y = self.app.y(h);
        let mut frame = Frame::new(renderer, bounds.size());
        // The clip is in frame coordinates, and the clipped frame starts
        // untranslated.
        frame.with_clip(
            Rectangle::new(Point::new(MARGIN, 0.0), Size::new(w, h)),
            |f| {
                f.translate(Vector::new(MARGIN, 0.0));
                self.app.draw_lines(f, &x, &y, 1.5);
            },
        );
        // The software renderer clips no paths: mask the margins too.
        frame.fill_rectangle(Point::ORIGIN, Size::new(MARGIN, h), Color::WHITE);
        frame.fill_rectangle(
            Point::new(MARGIN + w, 0.0),
            Size::new(RIGHT, h),
            Color::WHITE,
        );
        frame.translate(Vector::new(MARGIN, 0.0));
        draw_axis(&mut frame, &axis::bottom(&x).ticks(8), Point::new(0.0, h));
        draw_axis(&mut frame, &axis::left(&y).ticks(5), Point::ORIGIN);
        vec![frame.into_geometry()]
    }
}

impl canvas::Program<Message> for Overview<'_> {
    /// d3's x brush, made at the first event over the plot, and the
    /// tracker that feeds it.
    type State = (Option<BrushX>, Tracker);

    /// Brushing or moving the band publishes its window; clearing it
    /// shows everything.
    fn update(
        &self,
        (brush, tracker): &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let plot = plot(bounds);
        let Some(input) = tracker.translate(event, plot, cursor) else {
            return waiting(tracker);
        };
        let x = self.app.x(self.app.full, plot.width);
        let [r0, r1] = x.get_range();
        let brush =
            brush.get_or_insert_with(|| BrushX::new([[r0, 0.0], [r1, f64::from(OVERVIEW_HEIGHT)]]));
        // Between gestures follow the app's window; during one, the view may
        // not have caught up with our own messages yet, and syncing then
        // would pull the brush back under the pointer.
        if !brush.is_active() {
            let shown = self.app.visible.map(|t| x.apply(t));
            brush.set_selection((self.app.visible != self.app.full).then_some(shown));
        }
        let Some(brushed) = brush.handle(input) else {
            return waiting(tracker);
        };
        match brushed {
            BrushEvent::Brush(Some([a, b])) | BrushEvent::End(Some([a, b])) => {
                let window = [x.invert(a), x.invert(b)];
                Some(Action::publish(Message::Show(window)).and_capture())
            }
            BrushEvent::End(None) => Some(Action::publish(Message::Show(self.app.full))),
            BrushEvent::Start(_) | BrushEvent::Brush(None) => Some(Action::capture()),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (w, h) = (plot(bounds).width, bounds.height - AXIS_ROOM);
        let x = self.app.x(self.app.full, w);
        let y = self.app.y(h);
        let mut frame = Frame::new(renderer, bounds.size());
        frame.translate(Vector::new(MARGIN, 0.0));
        self.app.draw_lines(&mut frame, &x, &y, 1.0);
        if self.app.visible != self.app.full {
            let [a, b] = self.app.visible.map(|t| x.apply(t) as f32);
            frame.fill_rectangle(Point::new(a, 0.0), Size::new(b - a, h), BAND);
        }
        draw_axis(&mut frame, &axis::bottom(&x).ticks(8), Point::new(0.0, h));
        vec![frame.into_geometry()]
    }
}

/// A canvas's plot area: between the margins, full height.
fn plot(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + MARGIN,
        width: bounds.width - MARGIN - RIGHT,
        ..bounds
    }
}

/// Nothing to publish; while a press waits for the next redraw to be
/// timed (double clicks), ask for that redraw.
fn waiting(tracker: &Tracker) -> Option<Action<Message>> {
    tracker.wants_redraw().then(Action::request_redraw)
}

/// Strokes `axis`'s lines and writes its labels from `origin`: at the core
/// level the host draws the text.
fn draw_axis<S: Ticks>(frame: &mut Frame, axis: &axis::Axis<'_, S>, origin: Point) {
    frame.with_save(|f| {
        f.translate(Vector::new(origin.x, origin.y));
        let lines = rustd3_iced::path(|p| axis.draw(p));
        f.stroke(&lines, Stroke::default().with_color(AXIS_INK));
        for label in axis.labels() {
            // d3 sets the baseline `dy` em below y; centre the text there.
            let middle = label.get_y() + (label.get_dy() - MIDDLE_EM) * LABEL_PX;
            f.fill_text(Text {
                content: label.get_text().to_owned(),
                position: Point::new(label.get_x() as f32, middle as f32),
                size: Pixels(LABEL_PX as f32),
                align_x: match label.get_anchor() {
                    Anchor::Start => Alignment::Left,
                    Anchor::Middle => Alignment::Center,
                    Anchor::End => Alignment::Right,
                },
                align_y: Vertical::Center,
                ..Text::default()
            });
        }
    });
}

/// A core color as iced's.
fn rgb(c: &Rgb) -> Color {
    let channel = |v: f64| (v / 255.0) as f32;
    Color::from_rgba(channel(c.r), channel(c.g), channel(c.b), c.opacity as f32)
}
