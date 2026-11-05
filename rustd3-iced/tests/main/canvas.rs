//! A `canvas::Program` through the headless harness: a d3 line drawn with
//! `path()` lands on its device rows on both backends, and `Tracker`
//! output reaches the app as messages.

use std::time::Duration;

use iced::widget::{canvas, container};
use iced::{Color, Length, Point, Rectangle, Size, mouse};
use iced_core::{Event, keyboard, window};
use rustd3_core::interact::{Button, Delta, Modifiers, Pointer};
use rustd3_core::shape;
use rustd3_iced::pointer::Tracker;
use rustd3_testkit::pixels;

use crate::headless::{self, Harness};

/// Strokes d3's line from (0, 120) to (320, 120), 2 px wide, black on
/// white, and publishes what its `Tracker` makes of each event.
struct Trace;

impl canvas::Program<Pointer> for Trace {
    type State = Tracker;

    fn update(
        &self,
        tracker: &mut Tracker,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Pointer>> {
        match tracker.translate(event, bounds, cursor) {
            Some(pointer) => Some(canvas::Action::publish(pointer)),
            None if tracker.wants_redraw() => Some(canvas::Action::request_redraw()),
            None => None,
        }
    }

    fn draw(
        &self,
        _tracker: &Tracker,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::WHITE);
        let line = rustd3_iced::path(|p| shape::line().draw([(0.0, 120.0), (320.0, 120.0)], p));
        frame.stroke(
            &line,
            canvas::Stroke::default()
                .with_width(2.0)
                .with_color(Color::BLACK),
        );
        vec![frame.into_geometry()]
    }
}

/// The canvas filling a 320 × 240 window at scale 2, on `backend`, or
/// on `ICED_TEST_BACKEND`'s for `None`.
fn trace(backend: Option<&str>) -> Option<Harness<'static, Pointer>> {
    let root = canvas(Trace).width(Length::Fill).height(Length::Fill);
    let size = Size::new(320.0, 240.0);
    match backend {
        Some(_) => Harness::with_backend(root, size, 2.0, backend),
        None => Harness::new(root, size, 2.0),
    }
}

/// Whether every pixel of row `y` satisfies `test`.
fn row(shot: &headless::Shot, y: u32, test: impl Fn([u8; 4]) -> bool) -> bool {
    (0..shot.width).all(|x| test(shot.pixel(x, y)))
}

#[test]
fn a_d3_line_on_a_canvas_lands_on_its_device_rows() {
    let Some(mut harness) = trace(None) else {
        return;
    };
    let shot = harness.frame(headless::epoch());
    println!("{}: {}", harness.backend(), headless::features());
    assert_eq!((shot.width, shot.height), (640, 480));
    // Logical y 119..121 is device rows 238..242.
    for y in 238..242 {
        assert!(
            row(&shot, y, |p| p[..3].iter().all(|&c| c <= 64)),
            "row {y}"
        );
    }
    for y in [200, 236, 243] {
        assert!(row(&shot, y, |p| p == [255; 4]), "row {y}");
    }
}

#[test]
fn the_two_backends_draw_the_canvas_line_alike() {
    let Some(mut software) = trace(Some("tiny-skia")) else {
        return;
    };
    let Some(mut gpu) = trace(Some("wgpu")) else {
        return;
    };
    let now = headless::epoch();
    let (a, b) = (software.frame(now), gpu.frame(now));
    let diff = pixels::Diff::between(&a.image(), &b.image(), 16).unwrap();
    println!(
        "tiny-skia against wgpu: max {}/255, {} channels over 16 ({})",
        diff.max(),
        diff.over(),
        headless::features()
    );
    assert!(diff.max() <= 64, "max {}", diff.max());
}

#[test]
fn tracker_output_reaches_the_app_as_messages() {
    // A 20 px padding puts the canvas at (20, 20): positions are relative.
    let root = container(canvas(Trace).width(Length::Fill).height(Length::Fill)).padding(20);
    let Some(mut harness) = Harness::new(root, Size::new(320.0, 240.0), 2.0) else {
        return;
    };
    let t = headless::epoch();
    harness.point_at(Point::new(120.0, 70.0));
    let _ = harness.events(&[
        Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard::Modifiers::CTRL)),
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(120.0, 70.0),
        }),
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        }),
    ]);
    assert_eq!(
        harness.messages(),
        [
            Pointer::Modifiers(Modifiers::CTRL),
            Pointer::Moved { x: 100.0, y: 50.0 },
            Pointer::Wheel {
                x: 100.0,
                y: 50.0,
                delta: Delta::Lines { x: -0.0, y: -1.0 },
                modifiers: Modifiers::CTRL,
            },
        ]
    );

    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
    let _ = harness.events(std::slice::from_ref(&press));
    // The press waits for a redraw to be timed; an event that makes no
    // pointer input asks for that redraw.
    let _ = harness.events(&[Event::Window(window::Event::Focused)]);
    assert_eq!(harness.redraw_requested(), window::RedrawRequest::NextFrame);
    let _ = harness.frame(t);
    let _ = harness.events(&[release, press]);
    let _ = harness.frame(t + Duration::from_millis(250));
    assert_eq!(harness.redraw_requested(), window::RedrawRequest::Wait);
    assert_eq!(
        harness.messages(),
        [
            Pointer::Pressed {
                x: 100.0,
                y: 50.0,
                button: Button::Left,
                modifiers: Modifiers::CTRL,
            },
            Pointer::Released {
                button: Button::Left
            },
            Pointer::Pressed {
                x: 100.0,
                y: 50.0,
                button: Button::Left,
                modifiers: Modifiers::CTRL,
            },
            Pointer::DoubleClicked { x: 100.0, y: 50.0 },
        ]
    );
}

#[test]
fn the_harness_waker_counts_its_wakes() {
    let Some(harness) = trace(None) else {
        return;
    };
    assert_eq!(harness.wakes(), 0);
    harness.waker().wake();
    harness.waker().wake();
    assert_eq!(harness.wakes(), 2);
}
