//! `interact::Drag` against traces transcribed from d3-drag 3.0.0's
//! `drag.js` for the mouse: `mousedowned` 50–63 with the filter 9–11,
//! `beforestart` 121–140 (the subject and its offset), the gesture's
//! events 141–165, `mousemoved` 65–72 (the click distance), `mouseupped`
//! 74–79.

use rustd3_core::interact::drag::Event;
use rustd3_core::interact::{Button, Delta, Drag, Modifiers, Pointer};

const RELEASE: Pointer = Pointer::Released {
    button: Button::Left,
};

/// A step: the input, what the subject function answers if asked, and
/// what d3 emits.
type Step = (Pointer, Option<(f64, f64)>, Option<Event>);

/// A left press at (`x`, `y`).
fn press(x: f64, y: f64) -> Pointer {
    Pointer::Pressed {
        x,
        y,
        button: Button::Left,
        modifiers: Modifiers::NONE,
    }
}

/// A move to (`x`, `y`).
fn to(x: f64, y: f64) -> Pointer {
    Pointer::Moved { x, y }
}

/// Feeds the steps to `drag` and asserts each event, and that the drag
/// is active from a `Start` to its `End`.
fn replay(name: &str, mut drag: Drag, steps: &[Step]) {
    let mut active = false;
    for (i, &(input, subject, expected)) in steps.iter().enumerate() {
        let event = drag.handle(input, |_, _| subject);
        assert_eq!(event, expected, "{name}, step {i}: {input:?}");
        active = match event {
            Some(Event::Start { .. }) => true,
            Some(Event::End { .. }) => false,
            Some(Event::Drag { .. }) | None => active,
        };
        assert_eq!(drag.is_active(), active, "{name}, step {i}: is_active");
    }
}

#[test]
fn drag_traces() {
    // G1 drag, subject offset. s − p0 = (−2, −3) is added to every
    // position (138–139, 157–158), dx and dy are p − p0 (159–160), and
    // moved latches at 8² + 12² > 0 (69).
    replay(
        "G1",
        Drag::default(),
        &[
            (
                press(12.0, 13.0),
                Some((10.0, 10.0)),
                Some(Event::Start { x: 10.0, y: 10.0 }),
            ),
            (
                to(20.0, 25.0),
                None,
                Some(Event::Drag {
                    x: 18.0,
                    y: 22.0,
                    dx: 8.0,
                    dy: 12.0,
                }),
            ),
            (
                RELEASE,
                None,
                Some(Event::End {
                    x: 18.0,
                    y: 22.0,
                    moved: true,
                }),
            ),
        ],
    );
    // G2 drag, click: no move, so mousemoving stays false (59).
    replay(
        "G2",
        Drag::default(),
        &[
            (
                press(5.0, 5.0),
                Some((5.0, 5.0)),
                Some(Event::Start { x: 5.0, y: 5.0 }),
            ),
            (
                RELEASE,
                None,
                Some(Event::End {
                    x: 5.0,
                    y: 5.0,
                    moved: false,
                }),
            ),
        ],
    );
    // G3 drag, click distance 5. 3² + 4² = 25 is not > 25 (69), but 3² + 5²
    // is. Each move emits.
    let g3 = Drag::default().click_distance(5.0);
    let three_four = (
        to(3.0, 4.0),
        None,
        Some(Event::Drag {
            x: 3.0,
            y: 4.0,
            dx: 3.0,
            dy: 4.0,
        }),
    );
    replay(
        "G3",
        g3.clone(),
        &[
            (
                press(0.0, 0.0),
                Some((0.0, 0.0)),
                Some(Event::Start { x: 0.0, y: 0.0 }),
            ),
            three_four,
            (
                to(3.0, 5.0),
                None,
                Some(Event::Drag {
                    x: 3.0,
                    y: 5.0,
                    dx: 0.0,
                    dy: 1.0,
                }),
            ),
            (
                RELEASE,
                None,
                Some(Event::End {
                    x: 3.0,
                    y: 5.0,
                    moved: true,
                }),
            ),
        ],
    );
    replay(
        "G3, released at (3, 4)",
        g3,
        &[
            (
                press(0.0, 0.0),
                Some((0.0, 0.0)),
                Some(Event::Start { x: 0.0, y: 0.0 }),
            ),
            three_four,
            (
                RELEASE,
                None,
                Some(Event::End {
                    x: 3.0,
                    y: 4.0,
                    moved: false,
                }),
            ),
        ],
    );
    // G4 drag, no subject: beforestart returns nothing (136), so
    // mousedowned starts no gesture (53).
    replay(
        "G4",
        Drag::default(),
        &[
            (press(5.0, 5.0), None, None),
            (to(9.0, 9.0), None, None),
            (RELEASE, None, None),
        ],
    );
}

#[test]
fn the_subject_is_asked_only_on_a_left_press_without_ctrl() {
    let mut drag = Drag::default();
    let mut asked = Vec::new();
    let inputs = [
        Pointer::Pressed {
            x: 1.0,
            y: 2.0,
            button: Button::Right,
            modifiers: Modifiers::NONE,
        },
        Pointer::Pressed {
            x: 3.0,
            y: 4.0,
            button: Button::Left,
            modifiers: Modifiers::CTRL,
        },
        Pointer::Pressed {
            x: 5.0,
            y: 6.0,
            button: Button::Left,
            modifiers: Modifiers::SHIFT,
        },
        to(7.0, 8.0),
    ];
    for input in inputs {
        drag.handle(input, |x, y| {
            asked.push((x, y));
            Some((x, y))
        });
    }
    assert_eq!(asked, [(5.0, 6.0)]);
}

#[test]
fn other_inputs_are_ignored_and_a_drag_outlasts_leaving() {
    let mut drag = Drag::default();
    assert_eq!(
        drag.handle(press(10.0, 10.0), |x, y| Some((x, y))),
        Some(Event::Start { x: 10.0, y: 10.0 })
    );
    let ignored = [
        Pointer::Left,
        Pointer::Modifiers(Modifiers::SHIFT),
        Pointer::Wheel {
            x: 10.0,
            y: 10.0,
            delta: Delta::Pixels { x: 0.0, y: 5.0 },
            modifiers: Modifiers::NONE,
        },
        Pointer::DoubleClicked { x: 10.0, y: 10.0 },
        Pointer::Released {
            button: Button::Middle,
        },
    ];
    for input in ignored {
        assert_eq!(drag.handle(input, |_, _| None), None, "{input:?}");
        assert!(drag.is_active(), "{input:?}");
    }
    assert_eq!(
        drag.handle(to(-30.0, 10.0), |_, _| None),
        Some(Event::Drag {
            x: -30.0,
            y: 10.0,
            dx: -40.0,
            dy: 0.0
        })
    );
    assert_eq!(
        drag.handle(RELEASE, |_, _| None),
        Some(Event::End {
            x: -30.0,
            y: 10.0,
            moved: true
        })
    );
    assert_eq!(drag.handle(RELEASE, |_, _| None), None);
}

#[test]
fn a_nan_offset_counts_as_none() {
    // `s.x - p[0] || 0` (138–139): NaN is falsy.
    let mut drag = Drag::default();
    assert_eq!(
        drag.handle(press(4.0, 6.0), |_, _| Some((f64::NAN, 9.0))),
        Some(Event::Start { x: 4.0, y: 9.0 })
    );
    assert_eq!(
        drag.handle(to(5.0, 7.0), |_, _| None),
        Some(Event::Drag {
            x: 5.0,
            y: 10.0,
            dx: 1.0,
            dy: 1.0
        })
    );
}

#[test]
fn a_press_during_a_drag_starts_anew_only_with_a_subject() {
    // mousedowned during a gesture: beforestart replaces gestures.mouse
    // when the subject is not null (136, 144), and returns before touching
    // the old gesture when it is (53).
    let mut drag = Drag::default().click_distance(3.0);
    drag.handle(press(0.0, 0.0), |x, y| Some((x, y)));
    drag.handle(to(10.0, 0.0), |_, _| None);
    assert_eq!(drag.handle(press(20.0, 0.0), |_, _| None), None);
    assert_eq!(
        drag.handle(RELEASE, |_, _| None),
        Some(Event::End {
            x: 10.0,
            y: 0.0,
            moved: true
        })
    );

    drag.handle(press(0.0, 0.0), |x, y| Some((x, y)));
    drag.handle(to(10.0, 0.0), |_, _| None);
    assert_eq!(
        drag.handle(press(20.0, 0.0), |_, _| Some((25.0, 0.0))),
        Some(Event::Start { x: 25.0, y: 0.0 })
    );
    assert_eq!(
        drag.handle(RELEASE, |_, _| None),
        Some(Event::End {
            x: 25.0,
            y: 0.0,
            moved: false
        })
    );
}
