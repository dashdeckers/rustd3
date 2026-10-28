//! Brushes through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: d3's focus and context as a figure, the window it publishes,
//! inputs before the app's update, the band following a zoom of its
//! target, empty selections, modifier changes, precedence over the wheel,
//! and the overlay: inside the brushed panel only, and no cell rendered.

use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Event, Point, Size, keyboard, mouse};
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::{self, Linear};
use rustd3_core::time::{self, Time};
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::layout::Grid;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Cache, Figure, Held, Target, axis, brush};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::Sample;
use rustd3_iced::plot::{Probe, Render};

use crate::headless::{self, Harness, Shot};

/// The figures' logical size.
pub const SIZE: Size = Size::new(640.0, 480.0);

/// One frame at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// What the test apps hear from the widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    /// The focus handle's domain.
    Window([Timestamp; 2]),
    /// The context brush's selection.
    Brushed(Option<[Timestamp; 2]>),
}

/// The first instant of the 30 days.
pub fn start() -> Timestamp {
    "2026-09-01T00:00:00Z"
        .parse()
        .expect("invariant: a valid RFC 3339 instant")
}

/// All 30 days: 719 hours.
pub fn full() -> [Timestamp; 2] {
    [start(), start() + SignedDuration::from_hours(719)]
}

/// Hours after [`start`].
pub fn hours(t: Timestamp) -> f64 {
    t.duration_since(start()).as_secs_f64() / 3600.0
}

/// Use case 1 as a figure: a focus panel over `window` above a context
/// panel over all 30 days, which brushes the focus's window; the app owns
/// the window when it `listens`.
pub struct FocusContext {
    pub window: [Timestamp; 2],
    pub k: f32,
    pub listens: bool,
    /// Every message, in order.
    pub heard: Vec<Message>,
}

impl FocusContext {
    /// All 30 days in view, at factor `k`.
    pub fn new(k: f32, listens: bool) -> FocusContext {
        FocusContext {
            window: full(),
            k,
            listens,
            heard: Vec::new(),
        }
    }

    /// The figure on the app's window, and its focus and context handles.
    pub fn figure(&self) -> (Figure<'static>, ScaleRef<Time>, ScaleRef<Time>) {
        let mut fig = Figure::new();
        fig.layout(Grid::new().rows([3.0, 1.0]));
        let focus = fig.scale(time::utc().domain(self.window));
        let context = fig.scale(time::utc().domain(full()));
        let y = fig.scale(scale::linear().domain([0.0, 20.0]));
        let points: Vec<(Timestamp, f64)> = (0..720_i32)
            .map(|h| {
                let t = start() + SignedDuration::from_hours(h.into());
                (t, 10.0 + 4.0 * (f64::from(h) / 24.0).sin())
            })
            .collect();
        fig.panel(0, 0)
            .mark(line(points.clone(), focus, y))
            .axis(axis::left(y))
            .axis(axis::bottom(focus))
            .zoom_x(focus);
        fig.panel(1, 0)
            .mark(line(points, context, y))
            .axis(axis::bottom(context))
            .brush(brush::x(context).sets(focus));
        (fig, focus, context)
    }

    /// Applies a message.
    pub fn update(&mut self, message: Message) {
        self.heard.push(message);
        if let Message::Window(window) = message {
            self.window = window;
        }
    }

    /// The widget for the app's state, without the crosshair and ring.
    pub fn view(&self) -> Plot<'static, Message> {
        let (fig, focus, context) = self.figure();
        let plot = Plot::new(fig)
            .scale_factor(self.k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false)
            .on_brush(context, Message::Brushed);
        if self.listens {
            plot.on_change(focus, Message::Window)
        } else {
            plot
        }
    }

    /// The plot areas of the figure as built now, list px: focus, then
    /// context.
    pub fn areas(&self) -> Vec<Rect> {
        let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
        let mut cache = Cache::default();
        cache.begin();
        let target = Target::Screen {
            scale: f64::from(self.k),
        };
        self.figure()
            .0
            .resolve(size, &mut cache, &Held::default(), target)
            .plot_areas()
            .to_vec()
    }
}

/// One frame as iced's runtime runs it: `RedrawRequested(now)`; while the
/// widget published messages, `app` applies them, the interface is rebuilt
/// from its view and `RedrawRequested(now)` is sent again, at most three
/// times; then the screenshot. Returns it and the messages.
pub fn run_frame(
    harness: &mut Harness<'static, Message>,
    app: &mut FocusContext,
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

/// The point `share` of the way across `area`, at its middle height.
pub fn across(area: Rect, share: f64) -> Point {
    Point::new(
        (area.x0 + share * area.width()) as f32,
        area.center().y as f32,
    )
}

/// The cursor moved to `to` and the left button pressed there.
pub fn press(harness: &mut Harness<'static, Message>, at: Point) {
    harness.point_at(at);
    let _ = harness.events(&[
        Event::Mouse(mouse::Event::CursorMoved { position: at }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    ]);
}

/// The cursor moved to `to`.
pub fn move_to(harness: &mut Harness<'static, Message>, to: Point) {
    harness.point_at(to);
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: to })]);
}

/// The left button released.
pub fn release(harness: &mut Harness<'static, Message>) {
    let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
}

/// The last window among `messages`.
fn window(messages: &[Message]) -> Option<[Timestamp; 2]> {
    messages.iter().rev().find_map(|message| match message {
        Message::Window(window) => Some(*window),
        Message::Brushed(_) => None,
    })
}

/// The last selection among `messages`.
fn brushed(messages: &[Message]) -> Option<Option<[Timestamp; 2]>> {
    messages.iter().rev().find_map(|message| match message {
        Message::Brushed(selection) => Some(*selection),
        Message::Window(_) => None,
    })
}

/// Whether `window` is `[from, to]` hours after the start within one
/// pixel's time of a context `width` px wide.
fn spans(window: [Timestamp; 2], [from, to]: [f64; 2], width: f64) -> bool {
    let pixel = 719.0 / width;
    (hours(window[0]) - from).abs() <= pixel && (hours(window[1]) - to).abs() <= pixel
}

/// A harness of `app` after its first frame at `t0`, and the context's
/// plot area; `None` when the backend is skipped.
fn started(app: &mut FocusContext, t0: Instant) -> Option<(Harness<'static, Message>, Rect)> {
    let context = app.areas()[1];
    let mut harness = Harness::new(app.view(), SIZE, app.k)?;
    let (_, messages) = run_frame(&mut harness, app, t0);
    assert!(messages.is_empty(), "{messages:?}");
    Some((harness, context))
}

/// The first and last-plus-one columns, device px, where `shot` differs
/// from the PNG of `app`'s figure by more than 8/255 inside the plot area
/// `area` (list px): the band the overlay draws there.
fn band_columns(shot: &Shot, app: &FocusContext, area: Rect) -> Option<(u32, u32)> {
    let k = f64::from(app.k);
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let png = crate::reference::png_of(&app.figure().0, size, k, iced::Color::WHITE);
    let [x0, y0, x1, y1] = [area.x0, area.y0, area.x1, area.y1].map(|e| (e * k).round() as u32);
    let mut columns: Option<(u32, u32)> = None;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * png.width + x) * 4) as usize;
            let differs = shot
                .pixel(x, y)
                .into_iter()
                .zip(&png.rgba[i..i + 4])
                .any(|(a, b)| a.abs_diff(*b) > 8);
            if differs {
                columns = Some(columns.map_or((x, x + 1), |(l, r)| (l.min(x), r.max(x + 1))));
            }
        }
    }
    columns
}

#[test]
fn brushing_the_context_sets_the_focus_window() {
    for (k, listens) in [(1.0, true), (2.0, true), (1.0, false)] {
        let mut app = FocusContext::new(k, listens);
        let t0 = headless::epoch();
        let Some((mut harness, context)) = started(&mut app, t0) else {
            return;
        };
        let backend = harness.backend();
        press(&mut harness, across(context, 0.25));
        let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
        move_to(&mut harness, across(context, 0.375));
        let (_, during) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
        move_to(&mut harness, across(context, 0.5));
        let _ = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
        release(&mut harness);
        let (_, end) = run_frame(&mut harness, &mut app, t0 + FRAME * 4);
        let expected = [179.75, 359.5];
        let selection = brushed(&app.heard).flatten().expect("a selection");
        println!(
            "focus and context, {backend}, k = {k}, published {listens}: {:?} h; during {:?}",
            selection.map(hours),
            brushed(&during)
        );
        assert!(
            spans(selection, expected, context.width()),
            "{backend}, k = {k}: {:?}",
            selection.map(hours)
        );
        assert!(
            matches!(brushed(&during), Some(Some(_))),
            "{backend}: a message while brushing"
        );
        assert_eq!(
            brushed(&end),
            Some(Some(selection)),
            "{backend}: the end publishes"
        );
        if listens {
            assert!(
                spans(app.window, expected, context.width()),
                "{backend}, k = {k}: {:?}",
                app.window.map(hours)
            );
        } else {
            // The widget owns the window: the focus panel shows it, as a
            // figure built with it does.
            assert!(window(&app.heard).is_none(), "{backend}");
            let (shot, _) = run_frame(&mut harness, &mut app, t0 + Duration::from_secs(1));
            let shown = FocusContext {
                window: selection,
                ..FocusContext::new(k, true)
            };
            let focus = shown.areas()[0];
            let device =
                [focus.x0, focus.y0, focus.x1, focus.y1].map(|e| (e * f64::from(k)) as u32);
            let png = crate::reference::png_of(
                &shown.figure().0,
                kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height)),
                f64::from(k),
                iced::Color::WHITE,
            );
            let mut max = 0;
            for y in device[1]..device[3] {
                for x in device[0]..device[2] {
                    let i = ((y * png.width + x) * 4) as usize;
                    for (shown, reference) in shot.pixel(x, y).into_iter().zip(&png.rgba[i..i + 4])
                    {
                        max = max.max(shown.abs_diff(*reference));
                    }
                }
            }
            println!("owned window, {backend}: focus panel max {max}/255 against its PNG");
            assert!(max <= crate::gestures::bound(&backend), "{backend}: {max}");
        }
    }
}

#[test]
fn events_before_update_keep_the_last_window() {
    let mut app = FocusContext::new(1.0, true);
    let t0 = headless::epoch();
    let Some((mut harness, context)) = started(&mut app, t0) else {
        return;
    };
    let backend = harness.backend();
    // The press and both moves arrive before the app's update runs.
    press(&mut harness, across(context, 0.25));
    move_to(&mut harness, across(context, 0.375));
    move_to(&mut harness, across(context, 0.5));
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME);
    let last = window(&messages).expect("a window");
    assert!(
        spans(last, [179.75, 359.5], context.width()),
        "{backend}: {:?}",
        last.map(hours)
    );
    // One more move: the gesture goes on from its press, not from the
    // window the app's view shows.
    move_to(&mut harness, across(context, 0.625));
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    let last = window(&messages).expect("a window");
    assert!(
        spans(last, [179.75, 449.375], context.width()),
        "{backend}: {:?}",
        last.map(hours)
    );
    release(&mut harness);
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    assert!(spans(app.window, [179.75, 449.375], context.width()));
}

#[test]
fn zooming_the_focus_shrinks_the_band() {
    let mut app = FocusContext::new(1.0, true);
    let t0 = headless::epoch();
    let Some((mut harness, context)) = started(&mut app, t0) else {
        return;
    };
    let backend = harness.backend();
    press(&mut harness, across(context, 0.25));
    move_to(&mut harness, across(context, 0.5));
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
    release(&mut harness);
    let (shot, _) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    let laid = app.areas()[1];
    let before = band_columns(&shot, &app, laid);
    let (left, right) = before.expect("a band");
    let edges = [0.25, 0.5].map(|share| laid.x0 + share * laid.width());
    assert!(
        (f64::from(left) - edges[0]).abs() <= 1.0 && (f64::from(right) - edges[1]).abs() <= 1.0,
        "{backend}: {before:?}"
    );
    // Twenty lines at the focus's centre: k = 2, around its middle.
    let focus = app.areas()[0];
    harness.point_at(Point::new(focus.center().x as f32, focus.center().y as f32));
    let _ = harness.events(&[Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: 20.0 },
    })]);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    let zoomed = window(&messages).expect("the zoom publishes the window");
    // The window 179.75–359.5 h halved about its middle, 269.625 h.
    let shrunk = [269.625 - 44.9375, 269.625 + 44.9375];
    assert!(
        spans(zoomed, shrunk, context.width()),
        "{backend}: {:?}",
        zoomed.map(hours)
    );
    let t1 = t0 + Duration::from_secs(1);
    let (shot, _) = run_frame(&mut harness, &mut app, t1);
    // The band drawn: half its width, about its middle, in the layout of
    // the zoomed window (its labels may move the plot areas).
    let context = app.areas()[1];
    let columns = band_columns(&shot, &app, context);
    let expected = [0.3125, 0.4375].map(|share| context.x0 + share * context.width());
    println!(
        "band after a zoom, {backend}: columns {columns:?}, expected {expected:?}; before {before:?}"
    );
    let (left, right) = columns.expect("a band");
    assert!(
        (f64::from(left) - expected[0]).abs() <= 1.0
            && (f64::from(right) - expected[1]).abs() <= 1.0,
        "{backend}: {columns:?}"
    );
    // The band followed: dragging it from its middle moves a window of
    // the halved span, where a band left at 25–50% would move 25%.
    press(&mut harness, across(context, 0.375));
    move_to(&mut harness, across(context, 0.475));
    let (_, messages) = run_frame(&mut harness, &mut app, t1 + FRAME);
    let moved = window(&messages).expect("a window");
    let shift = 0.1 * 719.0;
    println!(
        "band after a zoom, {backend}: zoomed {:?} h, dragged {:?} h",
        zoomed.map(hours),
        moved.map(hours)
    );
    assert!(
        spans(
            moved,
            [shrunk[0] + shift, shrunk[1] + shift],
            context.width()
        ),
        "{backend}: {:?}",
        moved.map(hours)
    );
}

#[test]
fn an_empty_release_reports_none() {
    let mut app = FocusContext::new(1.0, true);
    let t0 = headless::epoch();
    let Some((mut harness, context)) = started(&mut app, t0) else {
        return;
    };
    let backend = harness.backend();
    press(&mut harness, across(context, 0.25));
    move_to(&mut harness, across(context, 0.5));
    release(&mut harness);
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME);
    assert_ne!(app.window, full(), "{backend}: brushed");
    // A press and a release at one point: no selection, and the focus
    // shows all 30 days again.
    press(&mut harness, across(context, 0.75));
    let _ = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    release(&mut harness);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    assert_eq!(
        messages,
        [Message::Window(full()), Message::Brushed(None)],
        "{backend}"
    );
    assert_eq!(app.window, full(), "{backend}");
}

#[test]
fn modifiers_do_not_change_a_selection() {
    let mut app = FocusContext::new(1.0, true);
    let t0 = headless::epoch();
    let Some((mut harness, context)) = started(&mut app, t0) else {
        return;
    };
    let backend = harness.backend();
    press(&mut harness, across(context, 0.25));
    move_to(&mut harness, across(context, 0.5));
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME);
    let before = brushed(&messages).flatten().expect("a selection");
    // Alt and shift mid-drag: d3's keyboard modes are not in v0.1.
    let held = keyboard::Modifiers::ALT | keyboard::Modifiers::SHIFT;
    let _ = harness.events(&[Event::Keyboard(keyboard::Event::ModifiersChanged(held))]);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 2);
    assert!(messages.is_empty(), "{backend}: {messages:?}");
    let _ = harness.events(&[Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::empty(),
    ))]);
    release(&mut harness);
    let (_, messages) = run_frame(&mut harness, &mut app, t0 + FRAME * 3);
    assert_eq!(
        brushed(&messages),
        Some(Some(before)),
        "{backend}: the selection as it was"
    );
}

/// What [`a_drag_brushes_where_the_wheel_zooms`]'s widget publishes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Heard {
    X([f64; 2]),
    Brushed(Option<[f64; 2]>),
}

#[test]
fn a_drag_brushes_where_the_wheel_zooms() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(line([(0.0, 0.2), (100.0, 0.8)], x, y))
        .axis(axis::bottom(x))
        .zoom_x(x)
        .brush(brush::x(x));
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let mut cache = Cache::default();
    cache.begin();
    let area = fig
        .resolve(
            size,
            &mut cache,
            &Held::default(),
            Target::Screen { scale: 1.0 },
        )
        .plot_areas()[0];
    let plot = Plot::new(fig)
        .render(Render::Inline)
        .scale_factor(1.0)
        .on_change(x, Heard::X)
        .on_brush(x, Heard::Brushed);
    let Some(mut harness) = Harness::new(plot, SIZE, 1.0) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let (from, to) = (across(area, 0.2), across(area, 0.6));
    harness.point_at(from);
    let _ = harness.events(&[
        Event::Mouse(mouse::Event::CursorMoved { position: from }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    ]);
    harness.point_at(to);
    let _ = harness.events(&[
        Event::Mouse(mouse::Event::CursorMoved { position: to }),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    let [Heard::Brushed(Some([a, b]))] = messages[..] else {
        panic!("{backend}: the drag brushes and pans nothing, {messages:?}");
    };
    let share = |v: f64| v / 100.0;
    assert!(
        (share(a) - 0.2).abs() < 1e-3 && (share(b) - 0.6).abs() < 1e-3,
        "{backend}: [{a}, {b}]"
    );
    let _ = harness.events(&[Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 },
    })]);
    let _ = harness.frame(t0 + FRAME * 2);
    let messages = harness.messages();
    assert!(
        matches!(messages[..], [Heard::X(_)]),
        "{backend}: the wheel zooms, {messages:?}"
    );
}

/// A 2 × 2 grid of dots on one x and one y, [0, 10] each, with left and
/// bottom axes; the panel in (0, 0) brushes x.
fn quad() -> (Figure<'static>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    for (row, col) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let panel = fig.panel(row, col);
        panel
            .mark(dots([(1.0, 2.0), (5.0, 5.0), (9.0, 8.0)], x, y))
            .axis(axis::left(y))
            .axis(axis::bottom(x));
        if (row, col) == (0, 0) {
            panel.brush(brush::x(x));
        }
    }
    (fig, x)
}

/// The plot areas of [`quad`] at `SIZE` on the screen at `k`, list px.
fn quad_areas(k: f32) -> Vec<Rect> {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let mut cache = Cache::default();
    cache.begin();
    let target = Target::Screen {
        scale: f64::from(k),
    };
    quad()
        .0
        .resolve(size, &mut cache, &Held::default(), target)
        .plot_areas()
        .to_vec()
}

/// Pixels of the device rect `[x0, y0, x1, y1]` that differ between `a`
/// and `b`, and those outside it.
fn changed_in(a: &Shot, b: &Shot, [x0, y0, x1, y1]: [u32; 4]) -> (usize, usize) {
    let mut counts = (0, 0);
    for (i, (p, q)) in a
        .rgba
        .chunks_exact(4)
        .zip(b.rgba.chunks_exact(4))
        .enumerate()
    {
        if p != q {
            let (x, y) = (i as u32 % a.width, i as u32 / a.width);
            if (x0..x1).contains(&x) && (y0..y1).contains(&y) {
                counts.0 += 1;
            } else {
                counts.1 += 1;
            }
        }
    }
    counts
}

#[test]
fn brush_overlay_stays_inside_its_panel() {
    for k in [1.0, 2.0] {
        let area = quad_areas(k)[0];
        let device =
            [area.x0, area.y0, area.x1, area.y1].map(|e| (e * f64::from(k)).round() as u32);
        let plot = Plot::<Heard>::new(quad().0)
            .scale_factor(k)
            .render(Render::Inline)
            .crosshair(false)
            .highlight(false);
        let Some(mut harness) = Harness::new(plot, SIZE, k) else {
            return;
        };
        let backend = harness.backend();
        let t0 = headless::epoch();
        let bare = harness.frame(t0);
        let (from, to) = (across(area, 0.2), across(area, 0.6));
        harness.point_at(from);
        let _ = harness.events(&[
            Event::Mouse(mouse::Event::CursorMoved { position: from }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        ]);
        harness.point_at(to);
        let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: to })]);
        let live = harness.frame(t0 + FRAME);
        let _ = harness.events(&[Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))]);
        let band = harness.frame(t0 + FRAME * 2);
        // The band's edges and the plot area's rows, device px.
        let edge = |p: Point| (f64::from(p.x) * f64::from(k)).round() as u32;
        let (left, right) = (edge(from), edge(to));
        let middle = (device[1] + device[3]) / 2;
        for (name, shot) in [("live", &live), ("band", &band)] {
            let (inside, outside) = changed_in(&bare, shot, device);
            println!(
                "brush overlay, {backend}, k = {k}, {name}: {inside} px changed inside panel 0's plot area, {outside} outside"
            );
            assert!(inside > 0 && outside == 0, "{backend}, k = {k}, {name}");
            let changed = |x: u32, y: u32| shot.pixel(x, y) != bare.pixel(x, y);
            assert!(
                changed(left, middle) && changed(right - 1, middle),
                "{backend}, k = {k}, {name}: the band's edges"
            );
            assert!(
                !changed(left - 1, middle) && !changed(right, middle),
                "{backend}, k = {k}, {name}: nothing past them"
            );
            // The full height of the plot area.
            // The full height of the plot area, down to the bottom axis's
            // line, which the outline's ink at 40% does not change.
            let column = (left + right) / 2;
            let line = (4.0 * k).ceil() as u32;
            let rows: Vec<u32> = (device[1]..device[3] - line)
                .filter(|&y| !changed(column, y))
                .collect();
            assert!(
                rows.is_empty(),
                "{backend}, k = {k}, {name}: rows {rows:?} of {device:?} unchanged"
            );
        }
        assert!(
            live.rgba == band.rgba,
            "{backend}: the band is the last live rectangle"
        );
    }
}

/// The clock the probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn clock() -> Instant {
    Instant::now()
}

#[test]
fn brushing_alone_renders_no_cell() {
    let k = 2.0;
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(clock, tx);
    let view = || {
        let (fig, x) = quad();
        Plot::new(fig)
            .scale_factor(k)
            .render(Render::Inline)
            .probe(probe.clone())
            .on_brush(x, Heard::Brushed)
    };
    let area = quad_areas(k)[0];
    let Some(mut harness) = Harness::new(view(), SIZE, k) else {
        return;
    };
    let backend = harness.backend();
    let t0 = headless::epoch();
    let _ = harness.frame(t0);
    let first: Vec<Sample> = rx.try_iter().collect();
    let rasters = |samples: &[Sample]| {
        samples
            .iter()
            .filter(|sample| matches!(sample, Sample::Raster { .. }))
            .count()
    };
    let generations = |samples: &[Sample]| -> Vec<u64> {
        samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Frame { generation, .. } => Some(*generation),
                Sample::Stage { .. } | Sample::Raster { .. } | Sample::Mismatch { .. } => None,
            })
            .collect()
    };
    assert!(rasters(&first) > 0, "{backend}: the first frame renders");
    let generation = generations(&first);
    // Press, two moves and the release, a frame after each; the view is
    // rebuilt after every message, as the runtime does.
    let mut heard = Vec::new();
    let points = [0.2, 0.4, 0.6].map(|share| across(area, share));
    let inputs: [Vec<Event>; 4] = [
        vec![
            Event::Mouse(mouse::Event::CursorMoved {
                position: points[0],
            }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        ],
        vec![Event::Mouse(mouse::Event::CursorMoved {
            position: points[1],
        })],
        vec![Event::Mouse(mouse::Event::CursorMoved {
            position: points[2],
        })],
        vec![Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))],
    ];
    for (i, events) in inputs.iter().enumerate() {
        if let Some(&Event::Mouse(mouse::Event::CursorMoved { position })) = events.first() {
            harness.point_at(position);
        }
        let _ = harness.events(events);
        let now = t0 + FRAME * (i as u32 + 1);
        harness.tick();
        for _ in 0..3 {
            let _ = harness.events(&[Event::Window(iced::window::Event::RedrawRequested(now))]);
            let messages = harness.messages();
            if messages.is_empty() {
                break;
            }
            heard.extend(messages);
            harness.rebuild(view());
        }
        let _ = harness.shoot();
    }
    let samples: Vec<Sample> = rx.try_iter().collect();
    let stages = samples
        .iter()
        .filter(|sample| matches!(sample, Sample::Stage { .. }))
        .count();
    println!(
        "brushing alone, {backend}: {} messages, {} samples, {stages} stage samples, {} rasters, generations {:?}",
        heard.len(),
        samples.len(),
        rasters(&samples),
        generations(&samples)
    );
    assert_eq!(rasters(&samples), 0, "{backend}: no cell renders");
    assert!(
        generations(&samples)
            .iter()
            .all(|g| generation.last() == Some(g)),
        "{backend}: the cells' content never changed"
    );
    assert!(
        matches!(heard[..], [.., Heard::Brushed(Some(_))]) && heard.len() >= 3,
        "{backend}: {heard:?}"
    );
}
