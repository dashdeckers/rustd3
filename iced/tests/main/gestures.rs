//! Gestures through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: wheel zooms around the cursor, over a plot area or an axis, one
//! message per handle per frame, twin handles, steps the scale rejects;
//! drags and batched updates; the cursor; scaled previews of each plot
//! area's tiles; double clicks;
//! the overlay when the pointer leaves; and the pixels against the PNG of
//! the published domains.

use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Event, Point, Size, event, keyboard, mouse};
use rustd3_core::jiff::Timestamp;
use rustd3_core::scale::{self, Linear};
use rustd3_core::time;
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::mark::dots;
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Cache, Figure, Held, Target, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::Sample;
use rustd3_iced::plot::{Probe, Render};

use crate::headless::{self, Harness, Shot};

/// The figures' logical size.
pub const SIZE: Size = Size::new(480.0, 320.0);

/// One frame at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// What the test apps hear from the widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    X([f64; 2]),
    Y([f64; 2]),
    Vib([f64; 2]),
    Temp([f64; 2]),
}

/// An app whose `view` shows a `Plot` and whose `update` applies its
/// messages, as iced's runtime drives it.
pub trait App {
    /// Applies a message.
    fn update(&mut self, message: Message);
    /// The widget for the app's state.
    fn view(&self) -> Plot<'static, Message>;
}

/// One frame as iced's runtime runs it (`winit/src/lib.rs:788-830`):
/// `RedrawRequested(now)`; while the widget published messages, `app`
/// applies them, the interface is rebuilt from its view and
/// `RedrawRequested(now)` is sent again, at most three times in all; then
/// the screenshot. Returns it and the messages, in order.
pub fn run_frame(
    harness: &mut Harness<'static, Message>,
    app: &mut impl App,
    now: Instant,
) -> (Shot, Vec<Message>) {
    harness.tick();
    let mut all = Vec::new();
    for _ in 0..3 {
        let _ = harness.events(&[Event::Window(iced::window::Event::RedrawRequested(now))]);
        let messages = harness.messages();
        if messages.is_empty() {
            break;
        }
        for &message in &messages {
            app.update(message);
        }
        all.extend(messages);
        harness.rebuild(app.view());
    }
    (harness.shoot(), all)
}

/// Forty dots in [0, 100] × [0, 10] on x and y (fixed domains `x` and
/// `y`), with left and bottom axes; the panel zooms both. Also its
/// handles.
pub fn figure(x: [f64; 2], y: [f64; 2]) -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let hx = fig.scale(scale::linear().domain(x));
    let hy = fig.scale(scale::linear().domain(y));
    let points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let t = f64::from(i) / 40.0;
            (100.0 * t, 10.0 * (t * 9.0).sin().abs())
        })
        .collect();
    fig.panel(0, 0)
        .mark(dots(points, hx, hy))
        .axis(axis::left(hy))
        .axis(axis::bottom(hx))
        .zoom_x(hx)
        .zoom_y(hy);
    (fig, hx, hy)
}

/// The plot areas of `fig` at `SIZE` on the screen at `k`, list px.
pub fn areas(fig: &Figure<'_>, k: f32) -> Vec<Rect> {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let mut cache = Cache::default();
    cache.begin();
    let target = Target::Screen {
        scale: f64::from(k),
    };
    fig.resolve(size, &mut cache, &Held::default(), target)
        .plot_areas()
        .to_vec()
}

/// A wheel step of `lines` lines in iced's sign (positive scrolls up,
/// zooming in).
pub fn lines(lines: f32) -> Event {
    Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: lines },
    })
}

/// The value at `share` of the way through `domain`.
pub fn at(domain: [f64; 2], share: f64) -> f64 {
    domain[0] + share * (domain[1] - domain[0])
}

/// Whether `a` and `b` agree within `tolerance` relative to `b`.
pub fn close(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance * b.abs().max(f64::MIN_POSITIVE)
}

/// The cursor's shares of `area` across (from its left) and up (from its
/// bottom).
fn shares(area: Rect, cursor: Point) -> (f64, f64) {
    (
        (f64::from(cursor.x) - area.x0) / area.width(),
        (area.y1 - f64::from(cursor.y)) / area.height(),
    )
}

/// The x and y domains among `messages`, the last of each.
fn domains(messages: &[Message]) -> (Option<[f64; 2]>, Option<[f64; 2]>) {
    let mut found = (None, None);
    for message in messages {
        match *message {
            Message::X(d) => found.0 = Some(d),
            Message::Y(d) => found.1 = Some(d),
            Message::Vib(_) | Message::Temp(_) => {}
        }
    }
    found
}

/// A harness on [`figure`] with x [0, 100] and y [0, 10] at factor `k`,
/// both handles published, after its first frame at `t0`; `None` when the
/// backend is skipped.
fn published(k: f32, t0: Instant) -> Option<(Harness<'static, Message>, Rect)> {
    let (fig, x, y) = figure([0.0, 100.0], [0.0, 10.0]);
    let area = areas(&fig, k)[0];
    let plot = Plot::new(fig)
        .scale_factor(k)
        .render(Render::Inline)
        .on_change(x, Message::X)
        .on_change(y, Message::Y);
    let mut harness = Harness::new(plot, SIZE, k)?;
    let _ = harness.frame(t0);
    Some((harness, area))
}

#[test]
fn wheel_zoom_keeps_the_point_under_the_cursor() {
    for k in [1.0, 2.0] {
        let t0 = headless::epoch();
        let Some((mut harness, area)) = published(k, t0) else {
            return;
        };
        let backend = harness.backend();
        let cursor = Point::new(
            (area.x0 + 0.3 * area.width()).round() as f32 + 0.25,
            (area.y0 + 0.4 * area.height()).round() as f32,
        );
        let (sx, sy) = shares(area, cursor);
        harness.point_at(cursor);
        let statuses = harness.events(&[lines(3.0)]);
        assert_eq!(statuses, [event::Status::Captured], "{backend}");
        let _ = harness.frame(t0 + FRAME);
        let messages = harness.messages();
        assert_eq!(messages.len(), 2, "{backend}: {messages:?}");
        let (Some(dx), Some(dy)) = domains(&messages) else {
            panic!("{backend}: x and y published, {messages:?}");
        };
        // The data under the cursor, before and after.
        let x = (at([0.0, 100.0], sx), at(dx, sx));
        let y = (at([0.0, 10.0], sy), at(dy, sy));
        println!(
            "wheel zoom, {backend}, k = {k}: x {dx:?}, y {dy:?}; under the cursor x {:e} and y {:e} relative",
            (x.1 - x.0).abs() / x.0,
            (y.1 - y.0).abs() / y.0
        );
        assert!(close(x.1, x.0, 1e-12), "{backend}, k = {k}: x {x:?}");
        assert!(close(y.1, y.0, 1e-12), "{backend}, k = {k}: y {y:?}");
        // Three lines: k = 2^0.15.
        let k3 = 2f64.powf(0.15);
        assert!(close(dx[1] - dx[0], 100.0 / k3, 1e-12), "{backend}: {dx:?}");
        assert!(close(dy[1] - dy[0], 10.0 / k3, 1e-12), "{backend}: {dy:?}");
    }
}

#[test]
fn wheel_steps_follow_d3s_wheel_delta() {
    let k = 2.0;
    let t0 = headless::epoch();
    let Some((mut harness, area)) = published(k, t0) else {
        return;
    };
    let backend = harness.backend();
    let cursor = Point::new(area.center().x as f32, area.center().y as f32);
    harness.point_at(cursor);
    let span = |messages: &[Message]| {
        let (Some(x), _) = domains(messages) else {
            panic!("{backend}: x published, {messages:?}");
        };
        x[1] - x[0]
    };
    // iced's `Lines { y: 1 }` scrolls up; d3's delta is +0.05: 2^0.05.
    let _ = harness.events(&[lines(1.0)]);
    let _ = harness.frame(t0 + FRAME);
    let one = span(&harness.messages());
    assert!(
        close(one, 100.0 / 2f64.powf(0.05), 1e-12),
        "{backend}: {one}"
    );
    // 100 logical px of pixel delta, which iced reports in physical px:
    // 2^0.2 more, from the gesture's start.
    let pixels = Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels {
            x: 0.0,
            y: 100.0 * k,
        },
    });
    let _ = harness.events(&[pixels]);
    let _ = harness.frame(t0 + FRAME * 2);
    let more = span(&harness.messages());
    assert!(
        close(more, 100.0 / 2f64.powf(0.25), 1e-12),
        "{backend}: {more}"
    );
    // With ctrl, ten times the exponent: one line is 2^0.5.
    let ctrl = Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::CTRL));
    let _ = harness.events(&[ctrl, lines(1.0)]);
    let _ = harness.frame(t0 + FRAME * 3);
    let tenfold = span(&harness.messages());
    assert!(
        close(tenfold, 100.0 / 2f64.powf(0.75), 1e-12),
        "{backend}: {tenfold}"
    );
    println!("wheel delta, {backend}: spans {one}, {more}, {tenfold}");
}

#[test]
fn wheel_over_an_axis_zooms_only_that_axis() {
    let t0 = headless::epoch();
    let Some((mut harness, area)) = published(1.0, t0) else {
        return;
    };
    let backend = harness.backend();
    // Left of the plot area, over the y axis's tick labels.
    harness.point_at(Point::new(area.x0 as f32 - 12.0, area.center().y as f32));
    let statuses = harness.events(&[lines(2.0)]);
    assert_eq!(statuses, [event::Status::Captured], "{backend}");
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    assert!(
        matches!(messages[..], [Message::Y(_)]),
        "{backend}: {messages:?}"
    );
    // Below it, over the x axis, after the first gesture ended.
    let t = t0 + Duration::from_secs(1);
    let _ = harness.frame(t);
    harness.point_at(Point::new(area.center().x as f32, area.y1 as f32 + 12.0));
    let _ = harness.events(&[lines(2.0)]);
    let _ = harness.frame(t + FRAME);
    let messages = harness.messages();
    assert!(
        matches!(messages[..], [Message::X(_)]),
        "{backend}: {messages:?}"
    );
    // Far outside both: nothing is captured or published.
    harness.point_at(Point::new(2.0, 2.0));
    let statuses = harness.events(&[lines(2.0)]);
    assert_eq!(statuses, [event::Status::Ignored], "{backend}");
    let _ = harness.frame(t + FRAME * 2);
    assert!(harness.messages().is_empty(), "{backend}");
}

#[test]
fn five_wheel_events_publish_one_message() {
    let t0 = headless::epoch();
    let Some((mut harness, area)) = published(1.0, t0) else {
        return;
    };
    let backend = harness.backend();
    harness.point_at(Point::new(area.center().x as f32, area.center().y as f32));
    for _ in 0..5 {
        let _ = harness.events(&[lines(1.0)]);
    }
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    let [Message::X(x), Message::Y(y)] = messages[..] else {
        panic!("{backend}: one message per handle, {messages:?}");
    };
    let k5 = 2f64.powf(0.25);
    assert!(close(x[1] - x[0], 100.0 / k5, 1e-12), "{backend}: {x:?}");
    assert!(close(y[1] - y[0], 10.0 / k5, 1e-12), "{backend}: {y:?}");
    // The next frame has no new input and publishes nothing.
    let _ = harness.frame(t0 + FRAME * 2);
    assert!(harness.messages().is_empty(), "{backend}");
}

/// What [`wheel_zoom_on_a_time_handle_publishes_timestamps`]'s widget
/// publishes for its time handle.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Window([Timestamp; 2]);

#[test]
fn wheel_zoom_on_a_time_handle_publishes_timestamps() {
    let at = |s: &str| s.parse::<Timestamp>().expect("an instant");
    let day = [at("2026-10-24T22:00:00Z"), at("2026-10-25T23:00:00Z")];
    let mut fig = Figure::new();
    let t = fig.scale(time::utc().domain(day));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(dots([(day[0], 0.25), (day[1], 0.75)], t, y))
        .axis(axis::bottom(t))
        .zoom_x(t);
    let area = areas(&fig, 1.0)[0];
    let plot = Plot::new(fig).render(Render::Inline).on_change(t, Window);
    let t0 = headless::epoch();
    let Some(mut harness) = Harness::new(plot, SIZE, 1.0) else {
        return;
    };
    let backend = harness.backend();
    let _ = harness.frame(t0);
    let cursor = Point::new(
        (area.x0 + 0.3 * area.width()).round() as f32 + 0.25,
        area.center().y as f32,
    );
    harness.point_at(cursor);
    let _ = harness.events(&[lines(1.0)]);
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    let [Window([a, b])] = messages[..] else {
        panic!("{backend}: one domain of instants, {messages:?}");
    };
    // One line: the span shrinks by 2^0.05 about the instant under the
    // cursor, to the nanosecond.
    let span = day[1].duration_since(day[0]).as_nanos();
    let shrunk = b.duration_since(a).as_nanos();
    let expected = span as f64 / 2f64.powf(0.05);
    assert!(
        (shrunk as f64 - expected).abs() <= 1.0,
        "{backend}: {shrunk} ns"
    );
    let share = (f64::from(cursor.x) - area.x0) / area.width();
    let under = |[a, b]: [Timestamp; 2]| {
        let offset = (share * b.duration_since(a).as_nanos() as f64).round() as i128;
        a.as_nanosecond() + offset
    };
    let moved = under([a, b]) - under(day);
    assert!(
        moved.abs() <= 1,
        "{backend}: the cursor's instant moved {moved} ns"
    );
}

/// Twin y handles on panel 0 (vib left, temp right) over one x, and temp
/// again on panel 1; the app owns both twins.
struct Twins {
    vib: [f64; 2],
    temp: [f64; 2],
    k: f32,
}

impl Twins {
    fn figure(&self) -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 10.0]));
        let vib = fig.scale(scale::linear().domain(self.vib));
        let temp = fig.scale(scale::linear().domain(self.temp));
        let x1 = fig.scale(scale::linear().domain([0.0, 10.0]));
        let wave = |offset: f64, scale: f64| -> Vec<(f64, f64)> {
            (0..50)
                .map(|i| {
                    let t = f64::from(i) / 5.0;
                    (t, offset + scale * (1.0 + (t + offset).sin()))
                })
                .collect()
        };
        fig.panel(0, 0)
            .mark(dots(wave(0.0, 1.0), x, vib))
            .mark(dots(wave(20.0, 4.0), x, temp))
            .axis(axis::left(vib))
            .axis(axis::right(temp))
            .axis(axis::bottom(x))
            .zoom_y(vib)
            .zoom_y(temp);
        fig.panel(0, 1)
            .mark(dots(wave(20.0, 4.0), x1, temp))
            .axis(axis::left(temp))
            .axis(axis::bottom(x1));
        (fig, vib, temp)
    }
}

impl App for Twins {
    fn update(&mut self, message: Message) {
        match message {
            Message::Vib(d) => self.vib = d,
            Message::Temp(d) => self.temp = d,
            Message::X(_) | Message::Y(_) => {}
        }
    }

    fn view(&self) -> Plot<'static, Message> {
        let (fig, vib, temp) = self.figure();
        // No overlay: the frames are compared with the PNG.
        Plot::new(fig)
            .scale_factor(self.k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false)
            .on_change(vib, Message::Vib)
            .on_change(temp, Message::Temp)
    }
}

#[test]
fn twin_handles_zoom_together() {
    let k = 1.5;
    let mut app = Twins {
        vib: [0.0, 2.0],
        temp: [20.0, 30.0],
        k,
    };
    let area = areas(&app.figure().0, k)[0];
    let Some(mut harness) = Harness::new(app.view(), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = run_frame(&mut harness, &mut app, t0);
    let cursor = Point::new(
        area.center().x as f32,
        (area.y0 + 0.25 * area.height()) as f32,
    );
    let (_, up) = shares(area, cursor);
    harness.point_at(cursor);
    let _ = harness.events(&[lines(4.0)]);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME);
    let [Message::Vib(vib), Message::Temp(temp)] = messages[..] else {
        panic!("{backend}: both twins published once, {messages:?}");
    };
    // The same k and the same anchor: the same share of each span, and
    // the value under the cursor kept on both.
    let k4 = 2f64.powf(0.2);
    assert!(
        close(vib[1] - vib[0], 2.0 / k4, 1e-12),
        "{backend}: {vib:?}"
    );
    assert!(
        close(temp[1] - temp[0], 10.0 / k4, 1e-12),
        "{backend}: {temp:?}"
    );
    assert!(
        close(at(vib, up), at([0.0, 2.0], up), 1e-12),
        "{backend}: {vib:?}"
    );
    assert!(
        close(at(temp, up), at([20.0, 30.0], up), 1e-12),
        "{backend}: {temp:?}"
    );
    assert_eq!((app.vib, app.temp), (vib, temp));
    // Panel 1 shares temp and follows: once the gesture ended and the
    // layout settled, the frame is the PNG of the figure the app now
    // builds, both panels.
    let end = t0 + FRAME + Duration::from_millis(300);
    let (shot, messages) = run_frame(&mut harness, &mut app, end);
    assert!(messages.is_empty(), "{backend}: {messages:?}");
    let diff = against_png(&shot, &app.figure().0, k);
    println!(
        "twins, {backend}: vib {vib:?}, temp {temp:?}; against the app's PNG max {}/255",
        diff.max()
    );
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
}

#[test]
fn sub_precision_zoom_is_ignored() {
    let k = 1.0;
    let t0 = headless::epoch();
    let start = [1e9, 1e9 + 1e-3];
    let (fig, x, _) = figure(start, [0.0, 10.0]);
    let area = areas(&fig, k)[0];
    // Only x is published; y is the widget's.
    let plot = Plot::new(fig)
        .scale_factor(k)
        .render(Render::Inline)
        .on_change(x, Message::X);
    let Some(mut harness) = Harness::new(plot, SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let _ = harness.frame(t0);
    harness.point_at(Point::new(area.center().x as f32, area.center().y as f32));
    // The threshold: 2⁻⁴⁰ of the centre, about 9.09e-4.
    let threshold = 2f64.powi(-40) * 1e9;
    let mut spans = Vec::new();
    for (step, lines_up) in [1.0, 1.0, 1.0, -1.0].into_iter().enumerate() {
        let _ = harness.events(&[lines(lines_up)]);
        let _ = harness.frame(t0 + FRAME * (step as u32 + 1));
        let (x, _) = domains(&harness.messages());
        spans.push(x.map(|x| x[1] - x[0]));
    }
    println!("sub-precision zoom, {backend}: spans {spans:?}, threshold {threshold:e}");
    let step = 2f64.powf(0.05);
    let expect = |span: Option<f64>, k: f64| span.is_some_and(|s| close(s, 1e-3 / k, 1e-3));
    assert!(expect(spans[0], step), "{backend}: {spans:?}");
    assert!(expect(spans[1], step * step), "{backend}: {spans:?}");
    assert!(
        1e-3 / step.powi(3) < threshold,
        "the third step's span is below it"
    );
    assert_eq!(spans[2], None, "{backend}: the third step is rejected");
    // The zoom-out works from the last accepted transform: k back to
    // 2^0.05, far inside the [1e-6, 1e6] extent.
    assert!(expect(spans[3], step), "{backend}: {spans:?}");
}

/// An app showing [`figure`] on the domains it holds, publishing both
/// handles when `listens` (else the widget owns them), without the
/// overlay, so its frames compare with the PNG.
struct Controlled {
    x: [f64; 2],
    y: [f64; 2],
    k: f32,
    listens: bool,
}

impl Controlled {
    /// Domains x [0, 100], y [0, 10] at factor `k`.
    fn new(k: f32, listens: bool) -> Controlled {
        Controlled {
            x: [0.0, 100.0],
            y: [0.0, 10.0],
            k,
            listens,
        }
    }
}

impl App for Controlled {
    fn update(&mut self, message: Message) {
        match message {
            Message::X(d) => self.x = d,
            Message::Y(d) => self.y = d,
            Message::Vib(_) | Message::Temp(_) => {}
        }
    }

    fn view(&self) -> Plot<'static, Message> {
        let (fig, x, y) = figure(self.x, self.y);
        let plot = Plot::new(fig)
            .scale_factor(self.k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false);
        if self.listens {
            plot.on_change(x, Message::X).on_change(y, Message::Y)
        } else {
            plot
        }
    }
}

/// One pointer input of a drag, with where the cursor is for it.
#[derive(Debug, Clone, Copy)]
enum Step {
    Press(Point),
    Move(Point),
    Release,
}

impl Step {
    /// Sends the step's events to `harness`: the cursor placed and moved
    /// there, then the button pressed or released.
    fn send(self, harness: &mut Harness<'static, Message>) -> Vec<event::Status> {
        let moved = |at: Point| Event::Mouse(mouse::Event::CursorMoved { position: at });
        let left = mouse::Button::Left;
        match self {
            Step::Press(at) => {
                harness.point_at(at);
                harness.events(&[moved(at), Event::Mouse(mouse::Event::ButtonPressed(left))])
            }
            Step::Move(at) => {
                harness.point_at(at);
                harness.events(&[moved(at)])
            }
            Step::Release => harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(left))]),
        }
    }
}

/// A press at `from`, `moves` moves of `by` each, and the release.
fn drag(from: Point, by: (f32, f32), moves: u16) -> Vec<Step> {
    let mut steps = vec![Step::Press(from)];
    for i in 1..=moves {
        let i = f32::from(i);
        steps.push(Step::Move(Point::new(from.x + by.0 * i, from.y + by.1 * i)));
    }
    steps.push(Step::Release);
    steps
}

/// `steps` sent to a harness of `app`, a frame (as the runtime runs it)
/// after every `batch` of them and one more after the last, 16 ms apart
/// from `t0`; then a frame after the layout settled. The last frame and
/// the backend; `None` when the backend is skipped.
fn play(app: &mut Controlled, steps: &[Step], batch: usize, t0: Instant) -> Option<(Shot, String)> {
    let mut harness = Harness::new(app.view(), SIZE, app.k)?;
    let backend = harness.backend();
    let mut t = t0;
    let _ = run_frame(&mut harness, app, t);
    for chunk in steps.chunks(batch) {
        for step in chunk {
            let _ = step.send(&mut harness);
        }
        t += FRAME;
        let _ = run_frame(&mut harness, app, t);
    }
    let (shot, messages) = run_frame(&mut harness, app, t + Duration::from_secs(1));
    assert!(messages.is_empty(), "{backend}: {messages:?}");
    Some((shot, backend))
}

#[test]
fn drag_pans_every_handle_the_panel_zooms() {
    let k = 2.0;
    let mut app = Controlled::new(k, true);
    let area = areas(&figure(app.x, app.y).0, k)[0];
    let Some(mut harness) = Harness::new(app.view(), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = run_frame(&mut harness, &mut app, t0);
    let from = Point::new(area.center().x as f32, area.center().y as f32);
    let statuses = Step::Press(from).send(&mut harness);
    assert_eq!(
        statuses,
        [event::Status::Ignored, event::Status::Captured],
        "{backend}: the press is captured"
    );
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
    assert_eq!(
        harness.interaction(),
        mouse::Interaction::Grabbing,
        "{backend}"
    );
    let to = Point::new(from.x + 40.0, from.y + 25.0);
    let _ = Step::Move(to).send(&mut harness);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    assert_eq!(messages.len(), 2, "{backend}: {messages:?}");
    let _ = Step::Release.send(&mut harness);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    assert!(messages.is_empty(), "{backend}: the release moves nothing");
    // Content follows the pointer: 40 px right shows what was 40 px left,
    // 25 px down what was 25 px up.
    let dx = 40.0 / area.width() * 100.0;
    let dy = 25.0 / area.height() * 10.0;
    println!(
        "drag, {backend}: x {:?}, y {:?} (shifts {dx}, {dy})",
        app.x, app.y
    );
    for (got, want) in app.x.iter().zip([-dx, 100.0 - dx]) {
        assert!(close(*got, want, 1e-12), "{backend}: x {:?}", app.x);
    }
    for (got, want) in app.y.iter().zip([dy, 10.0 + dy]) {
        assert!(close(*got, want, 1e-12), "{backend}: y {:?}", app.y);
    }
    assert_ne!(
        harness.interaction(),
        mouse::Interaction::Grabbing,
        "{backend}"
    );
}

#[test]
fn pan_survives_batched_updates() {
    let k = 1.0;
    let area = areas(&figure([0.0, 100.0], [0.0, 10.0]).0, k)[0];
    let from = Point::new((area.x0 + 100.0) as f32, area.center().y as f32);
    // 120 px as 6 moves of 20 px: 8 inputs with the press and release.
    let steps = drag(from, (20.0, 0.0), 6);
    let t0 = headless::epoch();
    let mut finals = Vec::new();
    for batch in [1, 3, steps.len()] {
        let mut app = Controlled::new(k, true);
        let Some((shot, backend)) = play(&mut app, &steps, batch, t0) else {
            return;
        };
        println!(
            "pan, {backend}, an update per {batch} inputs: x {:?}",
            app.x
        );
        finals.push((app.x, shot, backend));
    }
    let shift = 120.0 / area.width() * 100.0;
    let (first, _, backend) = &finals[0];
    assert!(close(first[0], -shift, 1e-12), "{backend}: {first:?}");
    for (x, _, _) in &finals[1..] {
        for (a, b) in x.iter().zip(first) {
            assert!(close(*a, *b, 1e-12), "{backend}: {x:?} against {first:?}");
        }
    }
    let diff = against_png(&finals[0].1, &figure(*first, [0.0, 10.0]).0, k);
    assert!(
        diff.max() <= bound(backend),
        "{backend}: max {}/255",
        diff.max()
    );
}

#[test]
fn pan_survives_batched_updates_of_a_widget_owned_domain() {
    let k = 1.0;
    let area = areas(&figure([0.0, 100.0], [0.0, 10.0]).0, k)[0];
    let from = Point::new((area.x0 + 100.0) as f32, area.center().y as f32);
    let steps = drag(from, (20.0, 0.0), 6);
    let t0 = headless::epoch();
    // The app owns the domain and applies every message…
    let mut controlled = Controlled::new(k, true);
    let Some((reference, backend)) = play(&mut controlled, &steps, 1, t0) else {
        return;
    };
    // …or the widget does, with every input batch: the same final domain,
    // so the same pixels.
    for batch in [1, 3, steps.len()] {
        let mut owned = Controlled::new(k, false);
        let (shot, _) = play(&mut owned, &steps, batch, t0).expect("the backend found above");
        assert_eq!(owned.x, [0.0, 100.0], "the app hears nothing");
        let diff = rustd3_testkit::pixels::Diff::between(&shot.image(), &reference.image(), 16)
            .expect("invariant: both shots have the widget's size");
        println!(
            "widget-owned pan, {backend}, an update per {batch} inputs: max {}/255 against the controlled one",
            diff.max()
        );
        assert_eq!(diff.max(), 0, "{backend}, batch {batch}");
    }
}

#[test]
fn the_cursor_shows_what_a_gesture_would_do() {
    let k = 1.0;
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    let fixed = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .axis(axis::left(y))
        .axis(axis::bottom(x))
        .zoom_x(x)
        .zoom_y(y);
    fig.panel(0, 1).axis(axis::left(fixed));
    let rects = areas(&fig, k);
    let Some(mut harness) = Harness::new(Plot::<Message>::new(fig).scale_factor(k), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let _ = harness.frame(headless::epoch());
    let over = |harness: &mut Harness<'static, Message>, x: f64, y: f64| {
        let at = Point::new(x as f32, y as f32);
        harness.point_at(at);
        let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
        harness.interaction()
    };
    let [zoomed, other] = [rects[0], rects[1]];
    let cases = [
        (
            zoomed.center().x,
            zoomed.center().y,
            mouse::Interaction::Grab,
        ),
        (
            zoomed.x0 - 8.0,
            zoomed.center().y,
            mouse::Interaction::ResizingVertically,
        ),
        (
            zoomed.center().x,
            zoomed.y1 + 8.0,
            mouse::Interaction::ResizingHorizontally,
        ),
        (
            other.center().x,
            other.center().y,
            mouse::Interaction::Crosshair,
        ),
        (other.x0 - 8.0, other.center().y, mouse::Interaction::None),
        (2.0, 2.0, mouse::Interaction::None),
    ];
    for (x, y, expected) in cases {
        assert_eq!(
            over(&mut harness, x, y),
            expected,
            "{backend}: at ({x}, {y})"
        );
    }
}

/// The clock a test's probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn clock() -> Instant {
    Instant::now()
}

/// What the last `Frame` sample among `samples` reported: (missing,
/// previews).
fn last_frame(samples: &[Sample]) -> Option<(u32, u32)> {
    samples.iter().rev().find_map(|sample| match sample {
        Sample::Frame {
            missing, previews, ..
        } => Some((*missing, *previews)),
        Sample::Stage { .. } | Sample::Raster { .. } | Sample::Mismatch { .. } => None,
    })
}

/// Frames of `harness` after each wake, the `Waken` injected before each
/// and the frames `t` + 16 ms apart, until one shows every slot current.
fn settle(
    harness: &mut Harness<'static, Message>,
    probe: &mpsc::Receiver<Sample>,
    mut t: Instant,
) -> (Shot, Instant) {
    for _ in 0..1000 {
        assert!(
            harness.wait_for_wake(Duration::from_secs(10)),
            "no wake within ten seconds"
        );
        let _ = harness.events(&[Event::Waken]);
        t += FRAME;
        let shot = harness.frame(t);
        let samples: Vec<Sample> = probe.try_iter().collect();
        if last_frame(&samples) == Some((0, 0)) {
            return (shot, t);
        }
    }
    panic!("a thousand wakes without a current frame")
}

/// Pixels of the device rect `[x0, y0, x1, y1]` that differ between `a`
/// and `b`, and those outside it.
fn changed_in(a: &Shot, b: &Shot, [x0, y0, x1, y1]: [u32; 4]) -> (usize, usize) {
    let mut inside = (0, 0);
    for (i, (p, q)) in a
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.rgba.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if p != q {
            let (x, y) = (i as u32 % a.width, i as u32 / a.width);
            if (x0..x1).contains(&x) && (y0..y1).contains(&y) {
                inside.0 += 1;
            } else {
                inside.1 += 1;
            }
        }
    }
    inside
}

/// The mean absolute channel difference of `a` and `b` inside the device
/// rect `[x0, y0, x1, y1]`.
fn mean_in(a: &Shot, b: &Shot, [x0, y0, x1, y1]: [u32; 4]) -> f64 {
    let (mut sum, mut n) = (0.0, 0.0);
    for y in y0..y1 {
        for x in x0..x1 {
            let (p, q) = (a.pixel(x, y), b.pixel(x, y));
            for c in 0..3 {
                sum += f64::from(p[c].abs_diff(q[c]));
                n += 1.0;
            }
        }
    }
    sum / n
}

#[test]
fn zoom_shows_a_scaled_preview_at_once() {
    let _workers = crate::pipeline::workers();
    for k in [1.0, 2.0] {
        let t0 = headless::epoch();
        let (fig, _, _) = figure([0.0, 100.0], [0.0, 10.0]);
        let area = areas(&fig, k)[0];
        let device =
            [area.x0, area.y0, area.x1, area.y1].map(|e| (e * f64::from(k)).round() as u32);
        let (tx, rx) = mpsc::channel();
        let plot = Plot::<Message>::new(fig)
            .scale_factor(k)
            .render(Render::Worker)
            .crosshair(false)
            .highlight(false)
            .probe(Probe::new(clock, tx));
        let Some(mut harness) = Harness::new(plot, SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let _ = harness.frame(t0);
        let (before, t) = settle(&mut harness, &rx, t0);
        // One wheel step of two lines over the plot area.
        let cursor = Point::new(
            (area.x0 + 0.3 * area.width()) as f32,
            (area.y0 + 0.6 * area.height()) as f32,
        );
        harness.point_at(cursor);
        let _ = harness.events(&[lines(2.0)]);
        let preview = harness.frame(t + FRAME);
        let samples: Vec<Sample> = rx.try_iter().collect();
        let (missing, previews) = last_frame(&samples).expect("a frame sample");
        // Every tile of the plot area shows the preview layer of the grid
        // before the zoom.
        let side = |a: u32, b: u32| (b - a).div_ceil(256);
        let tiles = side(device[0], device[2]) * side(device[1], device[3]);
        assert_eq!(missing, 0, "{backend}, k = {k}");
        assert!(
            previews >= tiles,
            "{backend}, k = {k}: previews {previews} of {tiles} tiles"
        );
        let (sharp, _) = settle(&mut harness, &rx, t + FRAME);
        // The same input rendered inline.
        let (fig, _, _) = figure([0.0, 100.0], [0.0, 10.0]);
        let inline = Plot::<Message>::new(fig)
            .scale_factor(k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false);
        let mut reference = Harness::new(inline, SIZE, k).expect("the backend found above");
        let _ = reference.frame(t0);
        reference.point_at(cursor);
        let _ = reference.events(&[lines(2.0)]);
        let expected = reference.frame(t + FRAME);
        let diff = rustd3_testkit::pixels::Diff::between(&sharp.image(), &expected.image(), 16)
            .expect("invariant: both shots have the widget's size");
        // The preview moved the plot area's content and nothing else, and
        // is nearer the sharp frame there than the frame before the zoom.
        let (inside, outside) = changed_in(&before, &preview, device);
        let (near, far) = (
            mean_in(&preview, &sharp, device),
            mean_in(&before, &sharp, device),
        );
        println!(
            "zoom preview, {backend}, k = {k}: {previews} previews, {inside} px moved in the plot \
             area, {outside} outside; mean difference from the sharp frame {near:.2} (before: \
             {far:.2}); sharp against inline max {}/255",
            diff.max()
        );
        assert!(inside > 0 && outside == 0, "{backend}, k = {k}");
        assert!(near < far / 2.0, "{backend}, k = {k}: {near} against {far}");
        assert!(
            diff.max() <= 1,
            "{backend}, k = {k}: max {}/255",
            diff.max()
        );
    }
}

/// x fitted to forty dots in [2.5, 100] and published,
/// y fixed at [0, 10] and the widget's; the panel zooms both. The app
/// hears x but builds it fitted whatever it heard.
struct Fitted {
    heard: Vec<[f64; 2]>,
    k: f32,
}

impl Fitted {
    fn figure() -> (Figure<'static>, ScaleRef<Linear>) {
        let mut fig = Figure::new();
        let x = fig.fit(scale::linear());
        let y = fig.scale(scale::linear().domain([0.0, 10.0]));
        let points: Vec<(f64, f64)> = (0..40)
            .map(|i| {
                let t = f64::from(i) / 40.0;
                (100.0 * t + 2.5, 10.0 * (t * 9.0).sin().abs())
            })
            .collect();
        fig.panel(0, 0)
            .mark(dots(points, x, y))
            .axis(axis::left(y))
            .axis(axis::bottom(x))
            .zoom_x(x)
            .zoom_y(y);
        (fig, x)
    }
}

impl App for Fitted {
    fn update(&mut self, message: Message) {
        match message {
            Message::X(d) => self.heard.push(d),
            Message::Y(_) | Message::Vib(_) | Message::Temp(_) => {}
        }
    }

    fn view(&self) -> Plot<'static, Message> {
        let (fig, x) = Fitted::figure();
        Plot::new(fig)
            .scale_factor(self.k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false)
            .on_change(x, Message::X)
    }
}

/// A wheel zoom over `Fitted`'s plot area, its end, then two left clicks
/// `apart` and `away` px apart over it; the frame after the second click
/// and what the app heard from it, and the frame before the clicks.
fn clicks(apart: Duration, away: f32) -> Option<(Shot, Shot, Vec<Message>, Fitted, String)> {
    let k = 1.0;
    let mut app = Fitted {
        heard: Vec::new(),
        k,
    };
    let area = areas(&Fitted::figure().0, k)[0];
    let mut harness = Harness::new(app.view(), SIZE, k)?;
    let backend = harness.backend();
    let t0 = headless::epoch();
    let (fresh, _) = run_frame(&mut harness, &mut app, t0);
    let at = Point::new(area.center().x as f32, area.center().y as f32);
    harness.point_at(at);
    let _ = harness.events(&[lines(3.0)]);
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
    assert_eq!(app.heard.len(), 1, "{backend}: the zoom is published");
    // The wheel gesture ends; the app built x fitted all along, y is the
    // widget's zoomed domain.
    let t1 = t0 + Duration::from_secs(1);
    let (zoomed, _) = run_frame(&mut harness, &mut app, t1);
    assert_ne!(zoomed.rgba, fresh.rgba, "{backend}: y is zoomed");
    let click = |harness: &mut Harness<'static, Message>, at: Point| {
        let _ = Step::Press(at).send(harness);
        let _ = Step::Release.send(harness);
    };
    click(&mut harness, at);
    let _ = run_frame(&mut harness, &mut app, t1 + FRAME);
    click(&mut harness, Point::new(at.x + away, at.y));
    let (shot, messages) = run_frame(&mut harness, &mut app, t1 + FRAME + apart);
    Some((shot, zoomed, messages, app, backend))
}

#[test]
fn double_click_resets_touched_handles() {
    let Some((shot, zoomed, messages, app, backend)) = clicks(Duration::from_millis(200), 3.0)
    else {
        return;
    };
    // x, published, gets its fitted domain; y, the widget's, refits: the
    // frame is the figure as built.
    let [Message::X(x)] = messages[..] else {
        panic!("{backend}: the fitted x once, {messages:?}");
    };
    println!(
        "double click, {backend}: x {x:?} published; heard {:?}",
        app.heard
    );
    // The dots span [2.5, 100]; 5% of that span pads each side.
    let pad = 0.05 * 97.5;
    assert!(
        close(x[0], 2.5 - pad, 1e-12) && close(x[1], 100.0 + pad, 1e-12),
        "{backend}: {x:?}"
    );
    let diff = against_png(&shot, &Fitted::figure().0, 1.0);
    assert!(
        diff.max() <= bound(&backend),
        "{backend}: max {}/255",
        diff.max()
    );
    assert_ne!(shot.rgba, zoomed.rgba, "{backend}");
}

#[test]
fn slow_double_click_does_not_reset() {
    for (apart, away) in [
        (Duration::from_millis(400), 3.0),
        (Duration::from_millis(200), 8.0),
    ] {
        let Some((shot, zoomed, messages, _, backend)) = clicks(apart, away) else {
            return;
        };
        assert!(
            messages.is_empty(),
            "{backend}, {apart:?}, {away} px: {messages:?}"
        );
        assert_eq!(
            shot.rgba, zoomed.rgba,
            "{backend}, {apart:?}, {away} px: y stays zoomed"
        );
    }
}

#[test]
fn cursor_left_clears_the_overlay() {
    let k = 2.0;
    let (fig, _, _) = figure([0.0, 100.0], [0.0, 10.0]);
    let area = areas(&fig, k)[0];
    let plot = Plot::<Message>::new(fig)
        .scale_factor(k)
        .render(Render::Inline);
    let Some(mut harness) = Harness::new(plot, SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let bare = harness.frame(t0);
    let at = Point::new(area.center().x as f32, area.center().y as f32);
    harness.point_at(at);
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: at })]);
    let hovered = harness.frame(t0 + FRAME);
    assert_ne!(hovered.rgba, bare.rgba, "{backend}: the crosshair shows");
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorLeft)]);
    let left = harness.frame(t0 + FRAME * 2);
    assert!(left.rgba == bare.rgba, "{backend}: the overlay is gone");
}

#[test]
fn zoomed_pixels_match_a_figure_built_with_the_published_domain() {
    for k in [1.0, 2.0] {
        let mut app = Controlled::new(k, true);
        let (start, _, _) = figure(app.x, app.y);
        let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
        let mut cache = Cache::default();
        cache.begin();
        let target = Target::Screen {
            scale: f64::from(k),
        };
        let resolved = start.resolve(size, &mut cache, &Held::default(), target);
        let (area, held) = (resolved.plot_areas()[0], resolved.areas());
        let Some(mut harness) = Harness::new(app.view(), SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let t0 = headless::epoch();
        let _ = run_frame(&mut harness, &mut app, t0);
        harness.point_at(Point::new(
            (area.x0 + 0.7 * area.width()) as f32,
            (area.y0 + 0.2 * area.height()) as f32,
        ));
        let _ = harness.events(&[lines(5.0)]);
        // During the gesture: the published domains in the held layout.
        let (during, _) = run_frame(&mut harness, &mut app, t0 + FRAME);
        let zoomed = figure(app.x, app.y).0;
        let png =
            crate::reference::png_held(&zoomed, size, f64::from(k), iced::Color::WHITE, &held);
        let png = rustd3_testkit::pixels::Image::new(png.width, png.height, &png.rgba)
            .expect("invariant: a render holds width × height pixels");
        let held_diff = rustd3_testkit::pixels::Diff::between(&during.image(), &png, 16)
            .expect("invariant: the shot has the PNG's size");
        // Once it ended: the same domains laid out afresh.
        let (after, _) = run_frame(&mut harness, &mut app, t0 + Duration::from_secs(1));
        let diff = against_png(&after, &zoomed, k);
        println!(
            "zoomed pixels, {backend}, k = {k}: x {:?}, y {:?}; held max {}/255, settled max {}/255",
            app.x,
            app.y,
            held_diff.max(),
            diff.max()
        );
        assert!(
            held_diff.max() <= bound(&backend),
            "{backend}: max {}/255",
            held_diff.max()
        );
        assert!(
            diff.max() <= bound(&backend),
            "{backend}: max {}/255",
            diff.max()
        );
    }
}

/// The pixel bound of `backend` against the PNG: 1/255 on tiny-skia,
/// 2/255 on wgpu.
pub fn bound(backend: &str) -> u8 {
    if backend == "wgpu" { 2 } else { 1 }
}

/// The widget's frame against the PNG of `fig` at `k` on the screenshot's
/// white.
pub fn against_png(shot: &Shot, fig: &Figure<'_>, k: f32) -> rustd3_testkit::pixels::Diff {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let png = crate::reference::png_of(fig, size, f64::from(k), iced::Color::WHITE);
    let png = rustd3_testkit::pixels::Image::new(png.width, png.height, &png.rgba)
        .expect("invariant: a render holds width × height pixels");
    rustd3_testkit::pixels::Diff::between(&shot.image(), &png, 16)
        .expect("invariant: the shot has the PNG's size")
}

#[test]
fn a_listener_on_a_handle_of_another_figure_is_not_added() {
    let t0 = headless::epoch();
    let k = 1.0;
    let (fig, _, hy) = figure([0.0, 100.0], [0.0, 10.0]);
    // The same table position and type in another figure.
    let (other, ox, _) = figure([0.0, 1.0], [0.0, 1.0]);
    assert!(!fig.owns(ox));
    drop(other);
    let area = areas(&fig, k)[0];
    let plot = Plot::new(fig)
        .scale_factor(k)
        .render(Render::Inline)
        .on_change(ox, Message::X)
        .on_change(hy, Message::Y);
    let Some(mut harness) = Harness::new(plot, SIZE, k) else {
        return;
    };
    let _ = harness.frame(t0);
    harness.point_at(Point::new(
        (area.x0 + 0.5 * area.width()) as f32,
        (area.y0 + 0.5 * area.height()) as f32,
    ));
    let _ = harness.events(&[lines(3.0)]);
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    assert_eq!(domains(&messages).0, None, "{messages:?}");
    assert!(domains(&messages).1.is_some(), "{messages:?}");
}
