//! Gestures through the widget's state: queued pointers applied at each
//! `RedrawRequested`, what they store, and what they publish.

use std::sync::mpsc;

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
use crate::plot::domains::Key;
use crate::plot::listener::Listener;
use crate::plot::probe::{self, Probe, Sample};
use crate::plot::{Caching, Render};

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
        caching: Caching::On,
        revision: 0,
        probe: None,
        waker: &Waker::noop(),
        threads: 1,
    };
    state.redraw(fig, 1.0, now, &pipeline, (listeners, &[]))
}

/// One inline `RedrawRequested(now)` at factor 1 reported to a probe: the
/// tiles it rendered, as (column, row) of plot area 0's grid.
fn rendered(state: &mut State, fig: &Figure<'_>, now: Instant) -> Keys {
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(epoch, tx);
    let pipeline = Pipeline {
        render: Render::Inline,
        caching: Caching::On,
        revision: 0,
        probe: Some(&probe),
        waker: &Waker::noop(),
        threads: 1,
    };
    state.redraw::<()>(fig, 1.0, now, &pipeline, (&[], &[]));
    let mut tiles: Keys = rx
        .try_iter()
        .filter_map(|sample| match sample {
            Sample::Raster {
                target: probe::Target::Tile { i, j, .. },
                ..
            } => Some((i, j)),
            Sample::Raster { .. }
            | Sample::Stage { .. }
            | Sample::Frame { .. }
            | Sample::Mismatch { .. } => None,
        })
        .collect();
    tiles.sort_unstable();
    tiles
}

/// Tiles as (column, row) of a plot area's grid.
type Keys = Vec<(i32, i32)>;

/// Plot area 0's tiles, sorted: those that meet the area and those of its
/// ring.
fn keys(state: &State) -> (Keys, Keys) {
    let area = &state.areas[0];
    let (mut shown, mut ring) = (Keys::new(), Keys::new());
    for tile in area.tiles() {
        let key = (tile.key().i, tile.key().j);
        if area.shows(tile) {
            shown.push(key);
        } else {
            ring.push(key);
        }
    }
    shown.sort_unstable();
    ring.sort_unstable();
    (shown, ring)
}

/// Dots on x (handle 0, [0, 100]) and y (handle 1, [0, 10]) in a panel
/// without axes that zooms both: its plot area lies where it does
/// whatever the domains.
fn bare() -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let points = vec![(20.0, 1.0), (50.0, 5.0), (90.0, 9.0)];
    fig.panel(0, 0).mark(dots(points, x, y)).zoom_x(x).zoom_y(y);
    fig
}

#[test]
fn a_zoom_renders_the_tiles_that_show_and_the_ring_once_it_ends() {
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    let first = rendered(&mut state, &fig, t0);
    let (shown, ring) = keys(&state);
    assert_eq!((shown.len(), ring.len()), (4, 12));
    let all = {
        let mut all = [shown.clone(), ring.clone()].concat();
        all.sort_unstable();
        all
    };
    assert_eq!(first, all, "the first frame renders every tile");
    // A wheel step zooms both handles: every key is new, and only the four
    // that show render while the gesture lasts.
    let area = state.map.area(0).expect("one panel");
    let rect = state.areas[0].rect();
    assert!(state.queue(wheel(area.center())).capture);
    let step = rendered(&mut state, &fig, t0 + Duration::from_millis(16));
    let (shown, ring) = keys(&state);
    assert_eq!(step, shown, "during the gesture");
    assert!(rendered(&mut state, &fig, t0 + Duration::from_millis(160)).is_empty());
    // The first redraw 300 ms after the step ends the gesture: the plot
    // area stays, and its ring renders.
    let settled = rendered(&mut state, &fig, t0 + Duration::from_millis(316));
    assert_eq!(state.areas[0].rect(), rect);
    assert_eq!(settled, ring, "once the gesture ends");
    assert!(rendered(&mut state, &fig, t0 + Duration::from_millis(332)).is_empty());
}

#[test]
fn a_tile_in_flight_through_a_burst_of_zoom_steps_lands_in_the_preview() {
    // D22.5 (2): the pool takes a tile of the first step's grid; three more
    // steps arrive before it finishes. The queue holds the newest step's
    // jobs alone after each, and the finished tile, of a grid three steps
    // old, shows through the preview mapping instead of being dropped.
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    rendered(&mut state, &fig, t0);
    let pipeline = Pipeline {
        render: Render::Worker,
        caching: Caching::On,
        revision: 0,
        probe: None,
        waker: &Waker::noop(),
        threads: 1,
    };
    let area = state.map.area(0).expect("one panel");
    let mut in_flight = None;
    for step in 1..=4_u64 {
        assert!(state.queue(wheel(area.center())).capture);
        let now = t0 + Duration::from_millis(16 * step);
        state.redraw::<()>(&fig, 1.0, now, &pipeline, (&[], &[]));
        let waiting = state
            .pool
            .as_ref()
            .map(|pool| pool.waiting())
            .unwrap_or_default();
        assert!(
            waiting.iter().all(|&g| g == state.generation),
            "step {step}: the newest generation {} alone waits, {waiting:?}",
            state.generation
        );
        if step == 1 {
            // A thread takes the first tile that shows: the job the pool
            // has.
            let t = state.areas[0]
                .tiles()
                .iter()
                .position(|tile| state.areas[0].shows(tile))
                .expect("a tile that shows");
            in_flight = state.job(super::At::Tile(0, t), state.generation, None);
        }
    }
    let job = in_flight.expect("a job");
    let generation = job.generation;
    let done = job.run(&mut rustd3_figure::raster::Renderer::new(
        rustd3_figure::raster::Settings::default(),
    ));
    assert!(state.land(done, false, None), "the old grid's tile shows");
    let preview = state.areas[0].preview();
    assert_eq!(
        preview.last().and_then(|tile| tile.slot().generation()),
        Some(generation),
        "drawn last, over the older previews"
    );
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
    let left = |state: &State| {
        let resolved = state.resolved.as_ref().expect("resolved");
        let boxes = resolved.label_boxes();
        boxes.iter().map(|b| b.x0).fold(f64::INFINITY, f64::min)
    };
    let reach = left(&state);
    let (mut t, mut seen, mut grown) = (t0, Vec::new(), 0.0_f64);
    let mut look = |state: &State| {
        let resolved = state.resolved.as_ref().expect("resolved");
        seen.push((resolved.held(), resolved.plot_areas()[0]));
        grown = grown.max(reach - left(state));
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
    // The y labels grew outward, still inside the canvas.
    assert!(
        grown > 0.0 && reach - grown >= 0.0,
        "grown {grown} from {reach}"
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
        "held layout: y {zoomed:?}; left edge {} held, {} once settled; labels grew {grown:.2} px outward",
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
