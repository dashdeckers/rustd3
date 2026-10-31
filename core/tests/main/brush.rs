//! `interact::BrushX` and `interact::BrushY` against traces transcribed
//! from d3-brush 3.0.0's `brush.js` for one pointer (d3-brush's only test
//! is an export list): `started` 334–411, `moved` 413–430, `move`
//! 432–491, `ended` 493–508, the filter 108–110, `redraw`'s rects 256–284.
//! Extent [[0, 0], [100, 50]], handle size 6; X traces at pointer y = 10,
//! the Y mirrors at pointer x = 10.

use rustd3_core::interact::{BrushEvent, BrushX, BrushY, Button, Delta, Modifiers, Pointer};

const EXTENT: [[f64; 2]; 2] = [[0.0, 0.0], [100.0, 50.0]];

const RELEASE: Pointer = Pointer::Released {
    button: Button::Left,
};

/// A trace: the selection before it, and each input with what d3 emits.
struct Trace {
    name: &'static str,
    before: Option<[f64; 2]>,
    steps: Vec<(Pointer, Option<BrushEvent>)>,
}

/// The point at `v` along x, at y = 10.
fn x(v: f64) -> (f64, f64) {
    (v, 10.0)
}

/// The point at `v` along y, at x = 10.
fn y(v: f64) -> (f64, f64) {
    (10.0, v)
}

/// A left press at `p` with `modifiers` held.
fn press_with(p: (f64, f64), modifiers: Modifiers) -> Pointer {
    Pointer::Pressed {
        x: p.0,
        y: p.1,
        button: Button::Left,
        modifiers,
    }
}

/// A left press at `p`.
fn press(p: (f64, f64)) -> Pointer {
    press_with(p, Modifiers::NONE)
}

/// A move to `p`.
fn to(p: (f64, f64)) -> Pointer {
    Pointer::Moved { x: p.0, y: p.1 }
}

/// d3's `start` with the selection [a, b].
fn started(a: f64, b: f64) -> Option<BrushEvent> {
    Some(BrushEvent::Start(Some([a, b])))
}

/// d3's `brush` with the selection [a, b].
fn brushed(a: f64, b: f64) -> Option<BrushEvent> {
    Some(BrushEvent::Brush(Some([a, b])))
}

/// d3's `end` with the selection [a, b].
fn ended(a: f64, b: f64) -> Option<BrushEvent> {
    Some(BrushEvent::End(Some([a, b])))
}

/// Feeds a trace's inputs and asserts each event, and that the brush is
/// active from a `Start` to its `End`.
fn replay(trace: &Trace, mut feed: impl FnMut(Pointer) -> (Option<BrushEvent>, bool)) {
    let mut active = false;
    for (i, &(input, expected)) in trace.steps.iter().enumerate() {
        let (event, is_active) = feed(input);
        assert_eq!(event, expected, "{}, step {i}: {input:?}", trace.name);
        active = match event {
            Some(BrushEvent::Start(_)) => true,
            Some(BrushEvent::End(_)) => false,
            Some(BrushEvent::Brush(_)) | None => active,
        };
        assert_eq!(is_active, active, "{}, step {i}: is_active", trace.name);
    }
}

/// T1–T4, T8 and T11 along x.
fn x_traces() -> Vec<Trace> {
    let right = Pointer::Pressed {
        x: 50.0,
        y: 10.0,
        button: Button::Right,
        modifiers: Modifiers::NONE,
    };
    vec![
        // T1 create and flip: a press on the overlay (367–377) is handle
        // mode on e (signsX.overlay = +1, 340–341); dx clamps to
        // [W − e0, E − e0] (452); e1 < w1 flips (465–470).
        Trace {
            name: "T1",
            before: None,
            steps: vec![
                (press(x(20.0)), started(20.0, 20.0)),
                (to(x(60.0)), brushed(20.0, 60.0)),
                (to(x(10.0)), brushed(10.0, 20.0)),
                (RELEASE, ended(10.0, 20.0)),
            ],
        },
        // T2 click clears: a zero-width selection is empty (131–134) and
        // ends as null (506).
        Trace {
            name: "T2",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(x(50.0)), started(50.0, 50.0)),
                (RELEASE, Some(BrushEvent::End(None))),
            ],
        },
        // T3 drag, clamped: a press on the selection is MODE_DRAG (340);
        // dx 80 within [W − w0, E − e0] = [−10, 80] (442).
        Trace {
            name: "T3",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(x(15.0)), started(10.0, 20.0)),
                (to(x(95.0)), brushed(90.0, 100.0)),
                (RELEASE, ended(90.0, 100.0)),
            ],
        },
        // T4 handle: e's rect is [17, 23] (270, 272); dx 20 (452).
        Trace {
            name: "T4",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(x(21.0)), started(10.0, 20.0)),
                (to(x(41.0)), brushed(10.0, 40.0)),
                (RELEASE, ended(10.0, 40.0)),
            ],
        },
        // T8 ctrl, right button: the filter drops both (108–110).
        Trace {
            name: "T8",
            before: None,
            steps: vec![
                (press_with(x(50.0), Modifiers::CTRL), None),
                (right, None),
                (to(x(60.0)), None),
                (RELEASE, None),
            ],
        },
        // T11 overlapping handles: w's rect [7, 13] and e's [9, 15] both
        // cover 11; e is later in the DOM (27) and wins, w0 = 10, e0 = 12;
        // dx 19, e1 = 31 (452). Had w won: [12, 29] after the flip.
        Trace {
            name: "T11",
            before: Some([10.0, 12.0]),
            steps: vec![
                (press(x(11.0)), started(10.0, 12.0)),
                (to(x(30.0)), brushed(10.0, 31.0)),
                (RELEASE, ended(10.0, 31.0)),
            ],
        },
    ]
}

/// The Y mirrors T1Y–T4Y: handles n and s (34), so the selection is
/// clamped to [0, 50].
fn y_traces() -> Vec<Trace> {
    vec![
        // T1Y: dy 40 clamps to S − s0 = 30 (454), then flips (472–477).
        Trace {
            name: "T1Y",
            before: None,
            steps: vec![
                (press(y(20.0)), started(20.0, 20.0)),
                (to(y(60.0)), brushed(20.0, 50.0)),
                (to(y(10.0)), brushed(10.0, 20.0)),
                (RELEASE, ended(10.0, 20.0)),
            ],
        },
        Trace {
            name: "T2Y",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(y(40.0)), started(40.0, 40.0)),
                (RELEASE, Some(BrushEvent::End(None))),
            ],
        },
        // T3Y: dy 80 within [N − n0, S − s0] = [−10, 30]: 30 (443).
        Trace {
            name: "T3Y",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(y(15.0)), started(10.0, 20.0)),
                (to(y(95.0)), brushed(40.0, 50.0)),
                (RELEASE, ended(40.0, 50.0)),
            ],
        },
        // T4Y: s's rect is [17, 23] along y (271, 273).
        Trace {
            name: "T4Y",
            before: Some([10.0, 20.0]),
            steps: vec![
                (press(y(21.0)), started(10.0, 20.0)),
                (to(y(41.0)), brushed(10.0, 40.0)),
                (RELEASE, ended(10.0, 40.0)),
            ],
        },
    ]
}

#[test]
fn brush_traces_match_d3() {
    for trace in x_traces() {
        let mut brush = BrushX::new(EXTENT);
        brush.set_selection(trace.before);
        replay(&trace, |input| (brush.handle(input), brush.is_active()));
    }
    for trace in y_traces() {
        let mut brush = BrushY::new(EXTENT);
        brush.set_selection(trace.before);
        replay(&trace, |input| (brush.handle(input), brush.is_active()));
    }
}

#[test]
fn a_move_that_changes_nothing_emits_nothing() {
    // 483–490: d3 emits brush only when an edge changed.
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([90.0, 100.0]));
    assert_eq!(brush.handle(press(x(95.0))), started(90.0, 100.0));
    assert_eq!(brush.handle(to(x(99.0))), None);
    assert_eq!(brush.handle(to(x(95.0))), None);
    assert_eq!(brush.handle(to(x(85.0))), brushed(80.0, 90.0));
    assert_eq!(brush.handle(RELEASE), ended(80.0, 90.0));
}

#[test]
fn a_selection_set_mid_gesture_is_what_the_next_move_compares_against() {
    // 479: `if (state.selection) selection = state.selection;`.
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press(x(21.0))), started(10.0, 20.0));
    assert_eq!(brush.handle(to(x(41.0))), brushed(10.0, 40.0));
    brush.set_selection(Some([0.0, 5.0]));
    assert!(brush.is_active());
    assert_eq!(brush.handle(to(x(41.0))), brushed(10.0, 40.0));
    assert_eq!(brush.handle(to(x(41.0))), None);
    assert_eq!(brush.handle(RELEASE), ended(10.0, 40.0));
}

#[test]
fn a_selection_cleared_mid_gesture_leaves_the_last_one_to_compare_against() {
    // 479 skips a null state.selection, so a move compares against the
    // selection the previous move saw; 505–507 end with null unless
    // something set it again.
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press(x(21.0))), started(10.0, 20.0));
    assert_eq!(brush.handle(to(x(41.0))), brushed(10.0, 40.0));
    assert_eq!(brush.handle(to(x(51.0))), brushed(10.0, 50.0));
    brush.set_selection(None);
    assert_eq!(brush.handle(to(x(41.0))), None);
    assert_eq!(brush.get_selection(), None);
    assert_eq!(brush.handle(RELEASE), Some(BrushEvent::End(None)));

    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press(x(21.0))), started(10.0, 20.0));
    brush.set_selection(None);
    assert_eq!(brush.handle(to(x(31.0))), brushed(10.0, 30.0));
    assert_eq!(brush.handle(RELEASE), ended(10.0, 30.0));
}

#[test]
fn other_inputs_are_ignored_and_a_gesture_outlasts_leaving() {
    let mut brush = BrushX::new(EXTENT);
    assert_eq!(
        brush.handle(press_with(x(20.0), Modifiers::ALT.union(Modifiers::SHIFT))),
        started(20.0, 20.0)
    );
    let ignored = [
        Pointer::Left,
        Pointer::Modifiers(Modifiers::ALT),
        Pointer::Wheel {
            x: 30.0,
            y: 10.0,
            delta: Delta::Lines { x: 0.0, y: 1.0 },
            modifiers: Modifiers::NONE,
        },
        Pointer::DoubleClicked { x: 30.0, y: 10.0 },
        Pointer::Released {
            button: Button::Right,
        },
    ];
    for input in ignored {
        assert_eq!(brush.handle(input), None, "{input:?}");
        assert!(brush.is_active(), "{input:?}");
    }
    // Outside the extent, as d3 listens on the window: clamped at E.
    assert_eq!(brush.handle(to((140.0, 80.0))), brushed(20.0, 100.0));
    assert_eq!(brush.handle(RELEASE), ended(20.0, 100.0));
    assert_eq!(brush.handle(RELEASE), None);
    assert_eq!(brush.handle(to(x(50.0))), None);
}

#[test]
fn a_press_off_the_extent_and_the_selection_starts_nothing() {
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press(x(-1.0))), None);
    assert_eq!(brush.handle(press((50.0, 60.0))), None);
    assert!(!brush.is_active());
    assert_eq!(brush.get_selection(), Some([10.0, 20.0]));
}

#[test]
fn handles_reach_half_their_size_past_the_selection() {
    // 270–273: e's rect spans y from n − 3 to s + 3, past the extent.
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press((20.0, 52.0))), started(10.0, 20.0));
    assert_eq!(brush.handle(to((30.0, 52.0))), brushed(10.0, 30.0));
    assert_eq!(brush.handle(RELEASE), ended(10.0, 30.0));

    // A wider handle takes a press the default one leaves to the overlay.
    let mut wide = BrushX::new(EXTENT).handle_size(10.0);
    wide.set_selection(Some([10.0, 20.0]));
    assert_eq!(wide.handle(press(x(24.0))), started(10.0, 20.0));
    assert_eq!(wide.handle(to(x(44.0))), brushed(10.0, 40.0));
    let mut narrow = BrushX::new(EXTENT);
    narrow.set_selection(Some([10.0, 20.0]));
    assert_eq!(narrow.handle(press(x(24.0))), started(24.0, 24.0));
}

#[test]
fn a_press_during_a_gesture_reaches_only_the_overlay() {
    // 391: the group's pointer-events are none during a gesture; only
    // the overlay sets its own (170), so a press inside e's rect starts a
    // new selection there.
    let mut brush = BrushX::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    assert_eq!(brush.handle(press(x(15.0))), started(10.0, 20.0));
    assert_eq!(brush.handle(press(x(21.0))), started(21.0, 21.0));
    assert_eq!(brush.handle(to(x(41.0))), brushed(21.0, 41.0));
    assert_eq!(brush.handle(RELEASE), ended(21.0, 41.0));
}

#[test]
fn a_y_brush_spans_the_extent_across_and_flips_its_handles() {
    let mut brush = BrushY::new(EXTENT);
    brush.set_selection(Some([10.0, 20.0]));
    // n's rect is [7, 13] along y and [−3, 103] along x.
    assert_eq!(brush.handle(press((102.0, 9.0))), started(10.0, 20.0));
    assert_eq!(brush.handle(to((102.0, 39.0))), brushed(20.0, 40.0));
    assert_eq!(brush.handle(RELEASE), ended(20.0, 40.0));
}
