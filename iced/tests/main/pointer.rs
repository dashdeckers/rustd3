//! `pointer::Tracker`: iced events in, `interact::Pointer` input out, one
//! row per event kind.

use std::time::Duration;

use iced_core::time::Instant;
use iced_core::{Event, Point, Rectangle, keyboard, mouse, window};
use rustd3_core::interact::{self, Button, Delta, Modifiers, Pointer, Zoom};
use rustd3_iced::pointer::Tracker;

use crate::headless::epoch;

/// The area the tests translate for: 600 × 300 at (100, 50).
const BOUNDS: Rectangle = Rectangle {
    x: 100.0,
    y: 50.0,
    width: 600.0,
    height: 300.0,
};

/// The cursor at (`x`, `y`) in the area's own coordinates.
fn at(x: f32, y: f32) -> mouse::Cursor {
    mouse::Cursor::Available(Point::new(BOUNDS.x + x, BOUNDS.y + y))
}

/// The cursor moving to (`x`, `y`) in the area's own coordinates.
fn moved(x: f32, y: f32) -> Event {
    Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(BOUNDS.x + x, BOUNDS.y + y),
    })
}

fn pressed(button: mouse::Button) -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(button))
}

fn released(button: mouse::Button) -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(button))
}

fn wheel(delta: mouse::ScrollDelta) -> Event {
    Event::Mouse(mouse::Event::WheelScrolled { delta })
}

fn redraw(now: Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(now))
}

fn modifiers_changed(held: keyboard::Modifiers) -> Event {
    Event::Keyboard(keyboard::Event::ModifiersChanged(held))
}

#[test]
fn a_line_step_down_inside_is_a_wheel_step_in_the_doms_sign() {
    let mut tracker = Tracker::new();
    let event = wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 });
    let pointer = tracker.translate(&event, BOUNDS, at(450.0, 150.0));
    let Some(step @ Pointer::Wheel { x, y, delta, .. }) = pointer else {
        panic!("expected a wheel step, got {pointer:?}");
    };
    assert_eq!((x, y), (450.0, 150.0));
    let Delta::Lines { x: dx, y: dy } = delta else {
        panic!("expected lines, got {delta:?}");
    };
    assert!(dx == 0.0 && dx.is_sign_negative(), "{dx}");
    assert_eq!(dy, -1.0);
    // d3's defaultWheelDelta: −deltaY · 0.05 = +0.05, a zoom in.
    let t = Zoom::default()
        .handle(step, interact::Transform::IDENTITY)
        .expect("a wheel step zooms");
    assert!((t.k - 2f64.powf(0.05)).abs() < 1e-12, "{t}");
}

#[test]
fn pixel_deltas_divide_by_the_scale_factor() {
    let mut tracker = Tracker::new().scale_factor(1.5);
    let event = wheel(mouse::ScrollDelta::Pixels { x: 0.0, y: 30.0 });
    match tracker.translate(&event, BOUNDS, at(10.0, 10.0)) {
        Some(Pointer::Wheel { delta, .. }) => {
            assert_eq!(delta, Delta::Pixels { x: 0.0, y: -20.0 });
        }
        other => panic!("expected a wheel step, got {other:?}"),
    }
}

#[test]
fn a_scale_factor_that_is_not_positive_and_finite_is_ignored() {
    let event = wheel(mouse::ScrollDelta::Pixels { x: 0.0, y: 30.0 });
    for factor in [0.0, -2.0, f32::NAN, f32::INFINITY] {
        let mut tracker = Tracker::new().scale_factor(1.5).scale_factor(factor);
        match tracker.translate(&event, BOUNDS, at(10.0, 10.0)) {
            Some(Pointer::Wheel { delta, .. }) => {
                assert_eq!(delta, Delta::Pixels { x: 0.0, y: -20.0 }, "{factor}");
            }
            other => panic!("expected a wheel step, got {other:?}"),
        }
    }
}

#[test]
fn presses_carry_the_modifiers_of_the_last_change() {
    let mut tracker = Tracker::new();
    let changed = modifiers_changed(keyboard::Modifiers::SHIFT);
    assert_eq!(
        tracker.translate(&changed, BOUNDS, mouse::Cursor::Unavailable),
        Some(Pointer::Modifiers(Modifiers::SHIFT))
    );
    assert_eq!(
        tracker.translate(&pressed(mouse::Button::Left), BOUNDS, at(10.0, 10.0)),
        Some(Pointer::Pressed {
            x: 10.0,
            y: 10.0,
            button: Button::Left,
            modifiers: Modifiers::SHIFT,
        })
    );
}

#[test]
fn every_modifier_maps_to_its_core_modifier() {
    let mut tracker = Tracker::new();
    let all = keyboard::Modifiers::SHIFT
        | keyboard::Modifiers::CTRL
        | keyboard::Modifiers::ALT
        | keyboard::Modifiers::LOGO;
    let rows = [
        (keyboard::Modifiers::empty(), Modifiers::NONE),
        (keyboard::Modifiers::CTRL, Modifiers::CTRL),
        (keyboard::Modifiers::ALT, Modifiers::ALT),
        (keyboard::Modifiers::LOGO, Modifiers::LOGO),
        (
            all,
            Modifiers::SHIFT
                .union(Modifiers::CTRL)
                .union(Modifiers::ALT)
                .union(Modifiers::LOGO),
        ),
    ];
    for (iced, core) in rows {
        let event = modifiers_changed(iced);
        assert_eq!(
            tracker.translate(&event, BOUNDS, at(-50.0, -50.0)),
            Some(Pointer::Modifiers(core)),
            "{iced:?}"
        );
    }
}

#[test]
fn every_iced_button_maps_to_its_core_button() {
    let rows = [
        (mouse::Button::Left, Button::Left),
        (mouse::Button::Right, Button::Right),
        (mouse::Button::Middle, Button::Middle),
        (mouse::Button::Back, Button::Back),
        (mouse::Button::Forward, Button::Forward),
        (mouse::Button::Other(7), Button::Other(7)),
    ];
    for (iced, core) in rows {
        let mut tracker = Tracker::new();
        match tracker.translate(&pressed(iced), BOUNDS, at(1.0, 2.0)) {
            Some(Pointer::Pressed { button, .. }) => assert_eq!(button, core),
            other => panic!("expected a press, got {other:?}"),
        }
        assert_eq!(
            tracker.translate(&released(iced), BOUNDS, at(1.0, 2.0)),
            Some(Pointer::Released { button: core })
        );
    }
}

#[test]
fn a_second_press_within_300_ms_and_6_px_is_a_double_click() {
    let t = epoch();
    let mut tracker = Tracker::new();
    let left = pressed(mouse::Button::Left);
    let up = released(mouse::Button::Left);
    assert!(tracker.translate(&left, BOUNDS, at(20.0, 20.0)).is_some());
    assert!(tracker.translate(&up, BOUNDS, at(20.0, 20.0)).is_some());
    assert_eq!(tracker.translate(&redraw(t), BOUNDS, at(20.0, 20.0)), None);
    assert!(tracker.translate(&left, BOUNDS, at(23.0, 20.0)).is_some());
    let second = redraw(t + Duration::from_millis(250));
    assert_eq!(
        tracker.translate(&second, BOUNDS, at(23.0, 20.0)),
        Some(Pointer::DoubleClicked { x: 23.0, y: 20.0 })
    );
}

#[test]
fn a_second_press_after_350_ms_is_no_double_click() {
    let t = epoch();
    let mut tracker = Tracker::new();
    let left = pressed(mouse::Button::Left);
    let up = released(mouse::Button::Left);
    assert!(tracker.translate(&left, BOUNDS, at(20.0, 20.0)).is_some());
    assert!(tracker.translate(&up, BOUNDS, at(20.0, 20.0)).is_some());
    assert_eq!(tracker.translate(&redraw(t), BOUNDS, at(20.0, 20.0)), None);
    assert!(tracker.translate(&left, BOUNDS, at(23.0, 20.0)).is_some());
    let second = redraw(t + Duration::from_millis(350));
    assert_eq!(tracker.translate(&second, BOUNDS, at(23.0, 20.0)), None);
}

#[test]
fn a_double_click_needs_the_same_button_and_under_6_px() {
    let t = epoch();
    let later = t + Duration::from_millis(100);
    let rows = [
        (mouse::Button::Right, (21.0, 20.0)),
        (mouse::Button::Left, (26.0, 20.0)),
    ];
    for (second, (x, y)) in rows {
        let mut tracker = Tracker::new();
        let left = pressed(mouse::Button::Left);
        assert!(tracker.translate(&left, BOUNDS, at(20.0, 20.0)).is_some());
        assert_eq!(tracker.translate(&redraw(t), BOUNDS, at(20.0, 20.0)), None);
        assert!(
            tracker
                .translate(&pressed(second), BOUNDS, at(x, y))
                .is_some()
        );
        assert_eq!(tracker.translate(&redraw(later), BOUNDS, at(x, y)), None);
    }
}

#[test]
fn a_third_quick_press_starts_a_new_click() {
    let t = epoch();
    let mut tracker = Tracker::new();
    let left = pressed(mouse::Button::Left);
    let mut doubles = Vec::new();
    for i in 0..4 {
        assert!(tracker.translate(&left, BOUNDS, at(20.0, 20.0)).is_some());
        let now = redraw(t + Duration::from_millis(100 * i));
        doubles.push(tracker.translate(&now, BOUNDS, at(20.0, 20.0)).is_some());
    }
    assert_eq!(doubles, [false, true, false, true]);
}

#[test]
fn a_press_waits_for_a_redraw_to_be_timed() {
    let mut tracker = Tracker::new();
    assert!(!tracker.wants_redraw());
    let left = pressed(mouse::Button::Left);
    assert!(tracker.translate(&left, BOUNDS, at(5.0, 5.0)).is_some());
    assert!(tracker.wants_redraw());
    assert_eq!(
        tracker.translate(&redraw(epoch()), BOUNDS, at(5.0, 5.0)),
        None
    );
    assert!(!tracker.wants_redraw());
}

#[test]
fn a_drag_continues_outside_and_releases_anywhere() {
    let mut tracker = Tracker::new();
    let left = pressed(mouse::Button::Left);
    assert!(tracker.translate(&left, BOUNDS, at(20.0, 20.0)).is_some());
    assert_eq!(
        tracker.translate(&moved(-40.0, 20.0), BOUNDS, at(-40.0, 20.0)),
        Some(Pointer::Moved { x: -40.0, y: 20.0 })
    );
    let up = released(mouse::Button::Left);
    assert_eq!(
        tracker.translate(&up, BOUNDS, at(-40.0, 20.0)),
        Some(Pointer::Released {
            button: Button::Left
        })
    );
    // The drag ended outside: the next move reports it.
    assert_eq!(
        tracker.translate(&moved(-41.0, 20.0), BOUNDS, at(-41.0, 20.0)),
        Some(Pointer::Left)
    );
}

#[test]
fn leaving_without_a_button_is_left_once_each_time() {
    let mut tracker = Tracker::new();
    assert_eq!(
        tracker.translate(&moved(10.0, 10.0), BOUNDS, at(10.0, 10.0)),
        Some(Pointer::Moved { x: 10.0, y: 10.0 })
    );
    assert_eq!(
        tracker.translate(&moved(700.0, 10.0), BOUNDS, at(700.0, 10.0)),
        Some(Pointer::Left)
    );
    assert_eq!(
        tracker.translate(&moved(710.0, 10.0), BOUNDS, at(710.0, 10.0)),
        None
    );
    let left_window = Event::Mouse(mouse::Event::CursorLeft);
    assert_eq!(
        tracker.translate(&left_window, BOUNDS, mouse::Cursor::Unavailable),
        None
    );
    assert_eq!(
        tracker.translate(&moved(10.0, 10.0), BOUNDS, at(10.0, 10.0)),
        Some(Pointer::Moved { x: 10.0, y: 10.0 })
    );
    assert_eq!(
        tracker.translate(&left_window, BOUNDS, mouse::Cursor::Unavailable),
        Some(Pointer::Left)
    );
    assert_eq!(
        tracker.translate(&left_window, BOUNDS, mouse::Cursor::Unavailable),
        None
    );
}

#[test]
fn input_outside_with_no_button_held_makes_nothing() {
    let mut tracker = Tracker::new();
    let outside = at(-5.0, 10.0);
    let events = [
        pressed(mouse::Button::Left),
        released(mouse::Button::Left),
        wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 }),
        moved(-5.0, 10.0),
        Event::Mouse(mouse::Event::CursorEntered),
        Event::Window(window::Event::Focused),
        Event::Waken,
    ];
    for event in events {
        assert_eq!(
            tracker.translate(&event, BOUNDS, outside),
            None,
            "{event:?}"
        );
    }
    assert!(!tracker.wants_redraw());
}

#[test]
fn only_the_held_button_releases() {
    let mut tracker = Tracker::new();
    let inside = at(10.0, 10.0);
    assert!(
        tracker
            .translate(&pressed(mouse::Button::Left), BOUNDS, inside)
            .is_some()
    );
    assert_eq!(
        tracker.translate(&released(mouse::Button::Right), BOUNDS, inside),
        None
    );
    assert_eq!(
        tracker.translate(&released(mouse::Button::Left), BOUNDS, inside),
        Some(Pointer::Released {
            button: Button::Left
        })
    );
    assert_eq!(
        tracker.translate(&released(mouse::Button::Left), BOUNDS, inside),
        None
    );
}
