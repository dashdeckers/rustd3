//! Gestures through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: wheel zooms around the cursor, over a plot area or an axis, one
//! message per handle per frame, twin handles, and steps the scale
//! rejects.

use std::time::Duration;

use iced::time::Instant;
use iced::{Event, Point, Size, event, keyboard, mouse};
use rustd3_core::scale::{self, Linear};
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::mark::dots;
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Cache, Figure, Held, Target, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::Render;

use crate::headless::{self, Harness, Shot};

/// The figures' logical size.
const SIZE: Size = Size::new(480.0, 320.0);

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

/// Dots on x and y (fixed domains `x` and `y`), with left and bottom axes;
/// the panel zooms both. Also its handles.
pub fn figure(x: [f64; 2], y: [f64; 2]) -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let hx = fig.scale(scale::linear().domain(x));
    let hy = fig.scale(scale::linear().domain(y));
    let points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let t = f64::from(i) / 40.0;
            (
                x[0] + t * (x[1] - x[0]),
                y[0] + (t * 9.0).sin().abs() * (y[1] - y[0]),
            )
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
