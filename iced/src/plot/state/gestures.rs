//! Gestures through the widget's state: queued pointers applied at each
//! `RedrawRequested`, what they store, and what they publish.

use iced_core::Size;
use iced_core::shell::Waker;
use iced_core::time::{Duration, Instant};
use rustd3_core::interact::{Delta, Modifiers, Pointer};
use rustd3_core::scale::{self, Linear, Log};
use rustd3_figure::kurbo::{self, Point};
use rustd3_figure::mark::dots;
use rustd3_figure::scale::{Kind, Scale, ScaleRef};
use rustd3_figure::{Figure, Held, Target, axis};

use super::{Pipeline, State};
use crate::plot::Render;
use crate::plot::domains::Key;
use crate::plot::listener::Listener;

/// Dots on y (handle 0, linear [0, 10]) and x (handle 1, linear [0, 100]
/// or log [1, 100]), with left and bottom axes; the panel zooms both.
fn panel(log: bool) -> Figure<'static> {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let points = vec![(2.0, 1.0), (5.0, 5.0), (50.0, 9.0)];
    if log {
        let x = fig.scale(scale::log().domain([1.0, 100.0]));
        fig.panel(0, 0)
            .mark(dots(points, x, y))
            .axis(axis::left(y))
            .axis(axis::bottom(x))
            .zoom_x(x)
            .zoom_y(y);
    } else {
        let x = fig.scale(scale::linear().domain([0.0, 100.0]));
        fig.panel(0, 0)
            .mark(dots(points, x, y))
            .axis(axis::left(y))
            .axis(axis::bottom(x))
            .zoom_x(x)
            .zoom_y(y);
    }
    fig
}

/// The instant frames are counted from.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn epoch() -> Instant {
    Instant::now()
}

/// A state laid out at 480 × 320 logical px.
fn state() -> State {
    let mut state = State::default();
    state.resize(Size::new(480.0, 320.0));
    state
}

/// One inline `RedrawRequested(now)` at factor 1 with `listeners`: the
/// cells that changed and the messages.
fn redraw<Message>(
    state: &mut State,
    fig: &Figure<'_>,
    now: Instant,
    listeners: &[Listener<'_, Message>],
) -> (usize, Vec<Message>) {
    let pipeline = Pipeline {
        render: Render::Inline,
        probe: None,
        waker: &Waker::noop(),
    };
    state.redraw(fig, 1.0, now, &pipeline, listeners)
}

/// One wheel line up (zoom in) at `at`, list px.
fn wheel(at: Point) -> Pointer {
    Pointer::Wheel {
        x: at.x,
        y: at.y,
        delta: Delta::Lines { x: 0.0, y: -1.0 },
        modifiers: Modifiers::NONE,
    }
}

/// The domain of a scale of type `S`.
fn domain<S: Kind<Domain = f64>>(scale: Option<&dyn Scale>) -> [f64; 2] {
    let scale = scale.expect("a scale").as_any().downcast_ref::<S>();
    Kind::domain(scale.expect("of the type")).expect("continuous")
}

#[test]
fn owned_domains_survive_rebuilds() {
    let mut state = state();
    let t0 = epoch();
    redraw::<()>(&mut state, &panel(false), t0, &[]);
    let area = state.map.area(0).expect("one panel");
    let x = Key::of::<Linear>(1);
    assert!(state.queue(wheel(area.center())).capture);
    let (changed, messages) = redraw::<()>(&mut state, &panel(false), t0, &[]);
    assert!(changed > 0 && messages.is_empty());
    let owned = domain::<Linear>(state.domains.owned(x));
    assert!(owned[1] - owned[0] < 100.0, "zoomed in: {owned:?}");
    // The view is rebuilt every frame: the widget's domain stays.
    for t in [16, 32, 400] {
        state.rebuilt();
        redraw::<()>(
            &mut state,
            &panel(false),
            t0 + Duration::from_millis(t),
            &[],
        );
        assert_eq!(domain::<Linear>(state.domains.owned(x)), owned);
        let resolved = state.resolved.as_ref().expect("resolved");
        assert_eq!(domain::<Linear>(resolved.scale_at(1)), owned);
    }
}

#[test]
fn state_is_dropped_when_the_handle_type_changes() {
    let mut state = state();
    let t0 = epoch();
    redraw::<()>(&mut state, &panel(false), t0, &[]);
    let area = state.map.area(0).expect("one panel");
    state.queue(wheel(area.center()));
    redraw::<()>(&mut state, &panel(false), t0, &[]);
    let (x, y) = (Key::of::<Linear>(1), Key::of::<Linear>(0));
    assert!(state.domains.owned(x).is_some() && state.domains.owned(y).is_some());
    // Handle 1 becomes a log scale: what the widget kept for (1, linear)
    // goes; handle 0 is still linear and keeps its domain.
    state.rebuilt();
    redraw::<()>(
        &mut state,
        &panel(true),
        t0 + Duration::from_millis(16),
        &[],
    );
    assert!(state.domains.owned(x).is_none());
    assert!(state.domains.owned(y).is_some());
    let resolved = state.resolved.as_ref().expect("resolved");
    assert_eq!(domain::<Log>(resolved.scale_at(1)), [1.0, 100.0]);
}

/// Dots on y (handle 0, linear [0, 3]) and x (handle 1, linear [0, 10]),
/// with left and bottom axes; the panel zooms y. Also the y handle.
fn tall() -> (Figure<'static>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 3.0]));
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let points = vec![(1.0, 0.5), (5.0, 1.5), (9.0, 2.5)];
    fig.panel(0, 0)
        .mark(dots(points, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x))
        .zoom_y(y);
    (fig, y)
}

/// Eight lines up (zoom in by 2^0.4) at `at`, list px.
fn eight(at: Point) -> Pointer {
    Pointer::Wheel {
        x: at.x,
        y: at.y,
        delta: Delta::Lines { x: 0.0, y: -8.0 },
        modifiers: Modifiers::NONE,
    }
}

#[test]
fn layout_holds_during_a_wheel_gesture() {
    let mut state = state();
    let t0 = epoch();
    let y = Key::of::<Linear>(0);
    redraw::<()>(&mut state, &tall().0, t0, &[]);
    let start = state.resolved.as_ref().expect("resolved").plot_areas()[0];
    let at = Point::new(start.center().x, start.y1 - 2.0);
    // Eight wheel steps of eight lines near the bottom, 16 ms apart, the
    // view rebuilt after each: [0, 3] narrows to about a third, whose tick
    // labels take two decimals. Then redraws without input up to 288 ms
    // after the last.
    let (mut t, mut seen) = (t0, Vec::new());
    let mut look = |state: &State| {
        let resolved = state.resolved.as_ref().expect("resolved");
        seen.push((resolved.held(), resolved.plot_areas()[0]));
    };
    for _ in 0..8 {
        t += Duration::from_millis(16);
        state.queue(eight(at));
        redraw::<()>(&mut state, &tall().0, t, &[]);
        look(&state);
        state.rebuilt();
    }
    let last = t;
    for ms in (16..=288).step_by(16) {
        redraw::<()>(&mut state, &tall().0, last + Duration::from_millis(ms), &[]);
        look(&state);
    }
    assert!(
        seen.iter().all(|&(held, area)| held && area == start),
        "the plot area holds still: {seen:?}"
    );
    // The first redraw 300 ms after the last step ends the gesture and
    // lays out once, as a fresh resolve of the zoomed domain does.
    redraw::<()>(
        &mut state,
        &tall().0,
        last + Duration::from_millis(300),
        &[],
    );
    let settled = state.resolved.as_ref().expect("resolved");
    assert!(!settled.held());
    let zoomed = domain::<Linear>(state.domains.owned(y));
    let mut held = Held::default();
    held.hold_erased(0, Box::new(scale::linear().domain(zoomed)));
    let mut cache = super::cache();
    cache.begin();
    let size = kurbo::Size::new(480.0, 320.0);
    let fresh = tall()
        .0
        .resolve(size, &mut cache, &held, Target::Screen { scale: 1.0 });
    let moved = settled.plot_areas()[0];
    println!(
        "held layout: y {zoomed:?}; left edge {} held, {} once settled",
        start.x0, moved.x0
    );
    assert_eq!(moved, fresh.plot_areas()[0]);
    assert_ne!(
        moved, start,
        "a relayout per step would have moved the area"
    );
    // Nothing more lays out.
    assert_eq!(
        redraw::<()>(
            &mut state,
            &tall().0,
            last + Duration::from_millis(316),
            &[]
        )
        .0,
        0
    );
}

#[test]
fn published_domains_last_until_the_gesture_ends() {
    let mut state = state();
    let t0 = epoch();
    let (fig, handle) = tall();
    let y = Key::of::<Linear>(0);
    let listeners = [Listener::new(handle, |d: [f64; 2]| d)];
    redraw(&mut state, &fig, t0, &listeners);
    let area = state.map.area(0).expect("one panel");
    state.queue(eight(area.center()));
    let (changed, messages) = redraw(&mut state, &fig, t0 + Duration::from_millis(16), &listeners);
    // Published: the list waits for the app's rebuilt view.
    assert_eq!((changed, messages.len()), (0, 1));
    assert_eq!(domain::<Linear>(state.domains.published(y)), messages[0]);
    assert!(state.domains.owned(y).is_none());
    // The app applied nothing; until the gesture ends the widget shows
    // what it published.
    state.rebuilt();
    let (changed, messages) = redraw(&mut state, &fig, t0 + Duration::from_millis(32), &listeners);
    assert!(changed > 0 && messages.is_empty());
    let shown = domain::<Linear>(state.resolved.as_ref().and_then(|r| r.scale_at(0)));
    assert_eq!(Some(shown), state.domains.published(y).map(|_| shown));
    // The gesture ends: the app's domain rules.
    redraw(
        &mut state,
        &fig,
        t0 + Duration::from_millis(400),
        &listeners,
    );
    assert!(state.domains.published(y).is_none());
    let shown = domain::<Linear>(state.resolved.as_ref().and_then(|r| r.scale_at(0)));
    assert_eq!(shown, [0.0, 3.0]);
}
