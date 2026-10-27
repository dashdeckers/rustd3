//! Gestures through the widget's state: queued pointers applied at each
//! `RedrawRequested`, what they store, and what they publish.

use iced_core::Size;
use iced_core::shell::Waker;
use iced_core::time::{Duration, Instant};
use rustd3_core::interact::{Delta, Modifiers, Pointer};
use rustd3_core::scale::{self, Linear, Log};
use rustd3_figure::kurbo::Point;
use rustd3_figure::mark::dots;
use rustd3_figure::scale::{Kind, Scale};
use rustd3_figure::{Figure, axis};

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
