//! Gestures through the widget's state: queued pointers applied at each
//! `RedrawRequested`, what they store, and what they publish.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};

use iced_core::shell::Waker;
use iced_core::time::{Duration, Instant};
use iced_core::{Rectangle, Size};
use rustd3_core::interact::{Delta, Modifiers, Pointer};
use rustd3_core::scale::{self, Linear, Log};
use rustd3_figure::kurbo::{self, Point};
use rustd3_figure::mark::dots;
use rustd3_figure::scale::{Kind, Scale, ScaleRef};
use rustd3_figure::{Figure, Held, Target, axis};

use super::{Pipeline, State};
use crate::plot::domains::Key;
use crate::plot::fixture::Recorder;
use crate::plot::listener::Listener;
use crate::plot::probe::{self, Sample};
use crate::plot::{Caching, Probe, Render};

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

/// Runs one inline `RedrawRequested(now)` at factor 1 with `listeners`,
/// and returns the cells that changed and the messages.
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
        hover: true,
    };
    state.redraw(fig, 1.0, now, &pipeline, (listeners, &[]))
}

/// Runs one inline `RedrawRequested(now)` at factor 1 reported to a probe,
/// and returns the tiles it rendered as (column, row) of plot area 0's
/// grid.
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
        hover: true,
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
/// without axes that zooms both. Its plot area lies in the same place
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
    // A wheel step zooms both handles. Every key is new, and only the four
    // that show render while the gesture lasts.
    let area = state.map.area(0).expect("one panel");
    let rect = state.areas[0].rect();
    assert!(state.queue(wheel(area.center())).capture);
    let step = rendered(&mut state, &fig, t0 + Duration::from_millis(16));
    let (shown, ring) = keys(&state);
    assert_eq!(step, shown, "during the gesture");
    assert!(rendered(&mut state, &fig, t0 + Duration::from_millis(160)).is_empty());
    // The first redraw 300 ms after the step ends the gesture. The plot
    // area stays, and its ring renders.
    let settled = rendered(&mut state, &fig, t0 + Duration::from_millis(316));
    assert_eq!(state.areas[0].rect(), rect);
    assert_eq!(settled, ring, "once the gesture ends");
    assert!(rendered(&mut state, &fig, t0 + Duration::from_millis(332)).is_empty());
}

#[test]
fn a_tile_in_flight_through_a_burst_of_zoom_steps_lands_in_the_preview() {
    // The pool takes a tile of the first step's grid, and three more steps
    // arrive before it finishes. After each step the queue holds only the
    // newest step's jobs. The finished tile, of a grid three steps old,
    // shows through the preview mapping instead of being dropped.
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
        hover: true,
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
            // A thread takes the first tile that shows, which is the job
            // the pool has.
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

#[test]
fn no_tile_renders_inline_while_a_gesture_lasts() {
    // Under `Render::Auto` the bare panel's tiles are cheap enough to
    // render inline, and do so when idle. During a wheel gesture they go to
    // the pool. Once it ends, inline renders resume for the ring.
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    let (waker, wakes) = counting();
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(epoch, tx);
    let pipeline = Pipeline {
        render: Render::Auto,
        caching: Caching::On,
        revision: 0,
        probe: Some(&probe),
        waker: &waker,
        threads: 1,
        hover: true,
    };
    // Tiles reported as rendered, and how many of them inline.
    let inline = |samples: Vec<Sample>| {
        samples
            .iter()
            .fold((0, 0), |(all, inline), sample| match sample {
                Sample::Raster {
                    target: probe::Target::Tile { .. },
                    inline: on,
                    ..
                } => (all + 1, inline + usize::from(*on)),
                Sample::Raster { .. }
                | Sample::Stage { .. }
                | Sample::Frame { .. }
                | Sample::Mismatch { .. } => (all, inline),
            })
    };
    let drained = |state: &mut State| {
        let mut seen = 0;
        while state.pool.is_some() && !state.waiting().is_empty() {
            let woke = (0..10_000).any(|_| {
                wakes.load(Ordering::SeqCst) > seen || {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    false
                }
            });
            assert!(woke, "no wake within ten seconds");
            seen = wakes.load(Ordering::SeqCst);
            state.drain(Some(&probe));
        }
    };
    state.redraw::<()>(&fig, 1.0, t0, &pipeline, (&[], &[]));
    let (all, first) = inline(rx.try_iter().collect());
    assert!(first > 0, "idle: {first} of {all} tiles inline");
    let area = state.map.area(0).expect("one panel");
    assert!(state.queue(wheel(area.center())).capture);
    let step = t0 + Duration::from_millis(16);
    state.redraw::<()>(&fig, 1.0, step, &pipeline, (&[], &[]));
    drained(&mut state);
    let (all, during) = inline(rx.try_iter().collect());
    assert_eq!((all, during), (4, 0), "the step's 4 tiles, on the pool");
    state.redraw::<()>(
        &fig,
        1.0,
        step + Duration::from_millis(300),
        &pipeline,
        (&[], &[]),
    );
    drained(&mut state);
    let (all, after) = inline(rx.try_iter().collect());
    assert!(
        all == 12 && after > 0,
        "the ring once the gesture ends: {after} of {all} inline"
    );
}

#[test]
fn a_drags_entering_tiles_render_inline_and_a_wheel_steps_on_the_pool() {
    // Under `Render::Auto` a drag's entering column renders inline when
    // the cost model says it fits, as idle renders do. On the pool it would
    // land a frame later, and a frame is long on the software renderer. A
    // wheel step's tiles still go to the pool.
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    let (waker, wakes) = counting();
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(epoch, tx);
    let pipeline = piped(Render::Auto, Some(&probe), &waker);
    // Tiles reported as rendered: (inline, on the pool).
    let tiles = |samples: Vec<Sample>| {
        samples
            .iter()
            .fold((0, 0), |(here, there), sample| match sample {
                Sample::Raster {
                    target: probe::Target::Tile { .. },
                    inline,
                    ..
                } => {
                    if *inline {
                        (here + 1, there)
                    } else {
                        (here, there + 1)
                    }
                }
                Sample::Raster { .. }
                | Sample::Stage { .. }
                | Sample::Frame { .. }
                | Sample::Mismatch { .. } => (here, there),
            })
    };
    let drained = |state: &mut State| {
        let mut seen = 0;
        while state.pool.is_some() && !state.waiting().is_empty() {
            let woke = (0..10_000).any(|_| {
                wakes.load(Ordering::SeqCst) > seen || {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    false
                }
            });
            assert!(woke, "no wake within ten seconds");
            seen = wakes.load(Ordering::SeqCst);
            state.drain(Some(&probe));
        }
    };
    state.redraw::<()>(&fig, 1.0, t0, &pipeline, (&[], &[]));
    drained(&mut state);
    let _ = rx.try_iter().count();
    // A press, then a move of 300 px right. Column −2 enters the area
    // without an image.
    let area = state.map.area(0).expect("one panel");
    let at = area.center();
    let press = Pointer::Pressed {
        x: at.x,
        y: at.y,
        button: rustd3_core::interact::Button::Left,
        modifiers: Modifiers::NONE,
    };
    assert!(state.queue(press).capture);
    let mut t = t0 + Duration::from_millis(16);
    state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
    assert!(
        state
            .queue(Pointer::Moved {
                x: at.x + 300.0,
                y: at.y
            })
            .redraw
    );
    t += Duration::from_millis(16);
    state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
    drained(&mut state);
    let (here, there) = tiles(rx.try_iter().collect());
    assert!(
        state.is_dragging() && here > 0 && there == 0,
        "the drag's entering tiles inline: {here} inline, {there} on the pool"
    );
    // The release ends the drag, and a wheel step's tiles go to the pool.
    state.queue(Pointer::Released {
        button: rustd3_core::interact::Button::Left,
    });
    t += Duration::from_millis(16);
    state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
    drained(&mut state);
    let _ = rx.try_iter().count();
    assert!(state.queue(wheel(area.center())).capture);
    t += Duration::from_millis(16);
    state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
    drained(&mut state);
    let (here, there) = tiles(rx.try_iter().collect());
    assert_eq!(
        (here, there),
        (0, 4),
        "the wheel step's 4 tiles, on the pool"
    );
}

#[test]
fn a_gesture_draws_in_one_layer_that_changes_and_no_preview_outside_the_area() {
    // While a gesture lasts, the widget's images go in one layer whose
    // bounds differ from one draw to the next. iced's software renderer
    // damages such a layer whole, so a frame damages one rect. When idle,
    // the layer keeps the widget's bounds. Of the previews only those that
    // meet the plot area are drawn. A step's 16 old tiles (the ring
    // included), which the pool has not replaced yet, draw as 4.
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    rendered(&mut state, &fig, t0);
    let bounds = Rectangle::new(iced_core::Point::ORIGIN, Size::new(480.0, 320.0));
    let theme = rustd3_figure::Theme::default();
    let draw = |state: &State| {
        let mut recorder = Recorder::gpu();
        state.draw(&mut recorder, bounds, &theme);
        recorder
    };
    let idle = [draw(&state), draw(&state)];
    for recorder in &idle {
        assert_eq!(recorder.layers(), [bounds], "idle");
        assert!(recorder.images_in_first_layer());
    }
    let pipeline = Pipeline {
        render: Render::Worker,
        caching: Caching::On,
        revision: 0,
        probe: None,
        waker: &Waker::noop(),
        threads: 1,
        hover: true,
    };
    let area = state.map.area(0).expect("one panel");
    assert!(state.queue(wheel(area.center())).capture);
    state.redraw::<()>(
        &fig,
        1.0,
        t0 + Duration::from_millis(16),
        &pipeline,
        (&[], &[]),
    );
    assert_eq!(state.areas[0].preview().len(), 16);
    let [a, b] = [draw(&state), draw(&state)];
    assert!(a.images_in_first_layer() && b.images_in_first_layer());
    assert_eq!((a.layers().len(), b.layers().len()), (1, 1));
    assert_ne!(
        a.layers(),
        b.layers(),
        "the layer changes from draw to draw"
    );
    assert!(a.layers().contains(&bounds) || b.layers().contains(&bounds));
    assert_eq!(a.images().1, 4, "the previews that meet the area");
}

#[test]
fn during_a_gesture_the_gutters_wait_ahead_of_the_tiles() {
    // A wheel step over a panel with axes changes its tiles, its axis
    // strips and the overlays over its edges. On the pool the gutters and
    // overlays render first, so the axes keep step with the data. When
    // idle, the tiles that show still go first.
    let mut state = state();
    let t0 = epoch();
    let fig = panel(false);
    rendered(&mut state, &fig, t0);
    let waker = Waker::noop();
    let area = state.map.area(0).expect("one panel");
    assert!(state.queue(wheel(area.center())).capture);
    let step = t0 + Duration::from_millis(16);
    let pipeline = piped(Render::Worker, None, &waker);
    state.redraw::<()>(&fig, 1.0, step, &pipeline, (&[], &[]));
    let kinds = |order: &[super::At]| -> String {
        order
            .iter()
            .map(|at| match at {
                super::At::Tile(..) => 't',
                super::At::Gutter(_) | super::At::Overlay(..) => 'g',
            })
            .collect()
    };
    let during = kinds(&state.waiting());
    let gutters = during.matches('g').count();
    assert!(
        gutters > 0 && during.matches('t').count() > 0,
        "a step changes both: {during}"
    );
    assert!(
        during.starts_with(&"g".repeat(gutters)),
        "the gutters first: {during}"
    );
    // The gesture ends, and what the settle renders puts the tiles first.
    let settled = step + Duration::from_millis(300);
    state.redraw::<()>(&fig, 1.0, settled, &pipeline, (&[], &[]));
    let after = kinds(&state.waiting());
    assert!(after.starts_with('t'), "idle: {after}");
}

#[test]
fn the_hit_index_waits_for_a_gesture_to_end_and_hover_stays_right() {
    // The index is built for a widget that hovers, but not during a
    // gesture (scatter1m spent 46–62 ms a pan frame on it). The refresh
    // after the gesture builds it. Meanwhile nothing is hit-tested, so no
    // outdated point is reported.
    let mut state = state();
    let t0 = epoch();
    let fig = bare();
    let x = Key::of::<Linear>(0);
    rendered(&mut state, &fig, t0);
    let area = state.map.area(0).expect("one panel");
    // Where the dot (50, 5) is drawn over x `domain`, in the widget's
    // logical px at factor 1.
    let dot = |domain: [f64; 2]| {
        let share = (50.0 - domain[0]) / (domain[1] - domain[0]);
        iced_core::Point::new(
            (area.x0 + share * area.width()) as f32,
            area.center().y as f32,
        )
    };
    let bounds = Rectangle::new(iced_core::Point::ORIGIN, Size::new(480.0, 320.0));
    let shown = crate::plot::overlay::Shown {
        crosshair: true,
        highlight: true,
    };
    let index = |state: &mut State| state.reported().map(|hover| hover.map(|h| h.index));
    let before = dot([0.0, 100.0]);
    state.moved(Some(before), bounds, shown);
    assert_eq!(index(&mut state), Some(Some(1)), "the dot at the centre");
    // Eight wheel lines at the area's left edge zoom x by 2^0.4, and the
    // dot moves right by about 70 px.
    assert!(
        state
            .queue(eight(Point::new(area.x0 + 1.0, area.center().y)))
            .capture
    );
    rendered(&mut state, &fig, t0 + Duration::from_millis(16));
    assert!(state.unhit, "no index during the gesture");
    state.moved(Some(before), bounds, shown);
    assert_eq!(index(&mut state), None, "nothing hit-tested during it");
    // The gesture ends, and the refresh builds the index of the zoomed
    // dots.
    rendered(&mut state, &fig, t0 + Duration::from_millis(316));
    assert!(!state.unhit);
    let zoomed = domain::<Linear>(state.domains.owned(x));
    let after = dot(zoomed);
    assert!(after.x - before.x > 40.0, "{before:?} → {after:?}");
    state.moved(Some(before), bounds, shown);
    assert_eq!(index(&mut state), Some(None), "no dot where it was");
    state.moved(Some(after), bounds, shown);
    assert_eq!(index(&mut state), Some(Some(1)), "the dot where it is");
    // A widget that does not hover builds no index.
    let mut idle = self::state();
    let pipeline = Pipeline {
        render: Render::Inline,
        caching: Caching::On,
        revision: 0,
        probe: None,
        waker: &Waker::noop(),
        threads: 1,
        hover: false,
    };
    idle.redraw::<()>(&fig, 1.0, t0, &pipeline, (&[], &[]));
    assert!(
        idle.unhit
            && idle
                .hits
                .nearest(
                    kurbo::Point::new(f64::from(before.x), f64::from(before.y)),
                    0
                )
                .is_none()
    );
}

/// A waker that counts its wakes.
fn counting() -> (Waker, Arc<AtomicUsize>) {
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = Waker::new({
        let wakes = Arc::clone(&wakes);
        move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        }
    });
    (waker, wakes)
}

/// One wheel line up (zoom in) at `at`, in list px.
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
    Kind::ends(scale.expect("of the type")).expect("continuous")
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
    // The view is rebuilt every frame, and the widget's domain stays.
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
        assert_eq!(domain::<Linear>(resolved.get_scale_at(1)), owned);
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
    // Handle 1 becomes a log scale, so what the widget kept for
    // (1, linear) is dropped. Handle 0 is still linear and keeps its
    // domain.
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
    assert_eq!(domain::<Log>(resolved.get_scale_at(1)), [1.0, 100.0]);
}

/// Dots on y (handle 0, linear [0, 3]) and x (handle 1, linear [0, 10]),
/// with left and bottom axes, in a panel that zooms y. Also returns the
/// y handle.
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

/// Eight lines up (zoom in by 2^0.4) at `at`, in list px.
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
    let start = state.resolved.as_ref().expect("resolved").get_plot_areas()[0];
    let at = Point::new(start.center().x, start.y1 - 2.0);
    // Eight wheel steps of eight lines near the bottom, 16 ms apart, with
    // the view rebuilt after each. [0, 3] narrows to about a third, whose
    // tick labels take two decimals. Then come redraws without input, up
    // to 288 ms after the last step.
    let left = |state: &State| {
        let resolved = state.resolved.as_ref().expect("resolved");
        let boxes = resolved.label_boxes();
        boxes.iter().map(|b| b.x0).fold(f64::INFINITY, f64::min)
    };
    let reach = left(&state);
    let (mut t, mut seen, mut grown) = (t0, Vec::new(), 0.0_f64);
    let mut look = |state: &State| {
        let resolved = state.resolved.as_ref().expect("resolved");
        seen.push((resolved.get_held(), resolved.get_plot_areas()[0]));
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
    assert!(!settled.get_held());
    let zoomed = domain::<Linear>(state.domains.owned(y));
    let mut held = Held::default();
    held.hold_erased(0, Box::new(scale::linear().domain(zoomed)));
    let mut cache = super::cache();
    cache.begin();
    let size = kurbo::Size::new(480.0, 320.0);
    let fresh = tall()
        .0
        .resolve(size, &mut cache, &held, Target::Screen { scale: 1.0 });
    let moved = settled.get_plot_areas()[0];
    println!(
        "held layout: y {zoomed:?}; left edge {} held, {} once settled; labels grew {grown:.2} px outward",
        start.x0, moved.x0
    );
    assert_eq!(moved, fresh.get_plot_areas()[0]);
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
fn the_first_draw_of_a_gesture_damages_the_widget_whole() {
    // iced's software renderer redraws a layer whole only when its bounds
    // differ from the last frame's. Otherwise it damages each moved image
    // separately. On the tiny-skia renderer before it, which redrew every
    // image once per damaged rect, that took a zoom burst's first frame
    // 10–11 s at factor 2 under Xvfb.
    // However many idle draws came before, the first draw of a gesture is
    // in a layer whose bounds differ from the last idle draw's.
    let bounds = Rectangle::new(iced_core::Point::ORIGIN, Size::new(480.0, 320.0));
    let theme = rustd3_figure::Theme::default();
    let layers = |state: &State| {
        let mut recorder = Recorder::default();
        state.draw(&mut recorder, bounds, &theme);
        recorder.layers()
    };
    for idle in 1..=4 {
        let mut state = state();
        let t0 = epoch();
        let fig = bare();
        rendered(&mut state, &fig, t0);
        let before: Vec<_> = (0..idle).map(|_| layers(&state)).collect();
        assert!(before.iter().all(|l| *l == [bounds]), "idle: {before:?}");
        let area = state.map.area(0).expect("one panel");
        assert!(state.queue(wheel(area.center())).capture);
        rendered(&mut state, &fig, t0 + Duration::from_millis(16));
        let during = [layers(&state), layers(&state), layers(&state)];
        assert_ne!(during[0], [bounds], "after {idle} idle draws: {during:?}");
        assert!(
            during[0] != during[1] && during[1] != during[2],
            "after {idle} idle draws: {during:?}"
        );
    }
}

/// A pipeline that renders as `render` asks, reports to `probe` and wakes
/// `waker`, with a pool of one thread.
fn piped<'a>(render: Render, probe: Option<&'a Probe>, waker: &'a Waker) -> Pipeline<'a> {
    Pipeline {
        render,
        caching: Caching::On,
        revision: 0,
        probe,
        waker,
        threads: 1,
        hover: true,
    }
}

/// The `Frame` samples among `samples`, as (missing, previews).
fn frames(samples: impl IntoIterator<Item = Sample>) -> Vec<(u32, u32)> {
    samples
        .into_iter()
        .filter_map(|sample| match sample {
            Sample::Frame {
                missing, previews, ..
            } => Some((missing, previews)),
            Sample::Stage { .. } | Sample::Raster { .. } | Sample::Mismatch { .. } => None,
        })
        .collect()
}

#[test]
fn the_layout_settling_after_a_zoom_leaves_no_slot_blank() {
    // Eight-line wheel steps near the bottom of `tall` narrow y until its
    // labels take two decimals. The layout then settles at the gesture's
    // end with the plot area narrower, which makes a new partition with
    // every run, tile and overlay new. With every render on the pool, the
    // old area's tiles show in the new area's preview layer, and the old
    // gutter images show under the new runs. So no frame shows a slot
    // missing, from the first step to the sharp frame after the settle.
    let mut state = state();
    let t0 = epoch();
    let (fig, _) = tall();
    rendered(&mut state, &fig, t0);
    let before = state.areas[0].rect();
    let start = state.resolved.as_ref().expect("resolved").get_plot_areas()[0];
    let at = Point::new(start.center().x, start.y1 - 2.0);
    let (waker, wakes) = counting();
    let (tx, rx) = mpsc::channel();
    let probe = Probe::new(epoch, tx);
    let pipeline = piped(Render::Worker, Some(&probe), &waker);
    let mut t = t0;
    let mut seen = Vec::new();
    for _ in 0..8 {
        t += Duration::from_millis(16);
        state.queue(eight(at));
        state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
        seen.extend(frames(rx.try_iter()));
    }
    // The first redraw 300 ms after the last step settles the layout,
    // before the pool's renders of the last step land.
    t += Duration::from_millis(300);
    state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
    let settle = frames(rx.try_iter());
    assert_ne!(
        state.areas[0].rect(),
        before,
        "the layout settled elsewhere"
    );
    assert!(
        settle.iter().all(|&(_, previews)| previews > 0),
        "nothing landed yet: {settle:?}"
    );
    seen.extend(settle);
    // Every image moved at the settle, as in a gesture. Until nothing of
    // the old layout shows, the widget draws in one layer whose bounds
    // change from draw to draw, so the software renderer redraws it once.
    let bounds = Rectangle::new(iced_core::Point::ORIGIN, Size::new(480.0, 320.0));
    let theme = rustd3_figure::Theme::default();
    let layers = |state: &State| {
        let mut recorder = Recorder::default();
        state.draw(&mut recorder, bounds, &theme);
        recorder.layers()
    };
    assert_ne!(layers(&state), layers(&state), "the old layout shows");
    // Then a frame after each batch of results, until one is sharp.
    let mut woken = 0;
    for _ in 0..1_000 {
        if seen.last() == Some(&(0, 0)) && state.waiting().is_empty() {
            break;
        }
        let woke = (0..10_000).any(|_| {
            wakes.load(Ordering::SeqCst) > woken || {
                std::thread::sleep(std::time::Duration::from_millis(1));
                false
            }
        });
        assert!(woke, "no wake within ten seconds");
        woken = wakes.load(Ordering::SeqCst);
        state.drain(Some(&probe));
        t += Duration::from_millis(16);
        state.redraw::<()>(&fig, 1.0, t, &pipeline, (&[], &[]));
        seen.extend(frames(rx.try_iter()));
    }
    assert_eq!(seen.last(), Some(&(0, 0)), "sharp at last: {seen:?}");
    assert!(
        seen.iter().all(|&(missing, _)| missing == 0),
        "no slot missing in any frame: {seen:?}"
    );
    assert_eq!(
        [layers(&state), layers(&state)],
        [[bounds], [bounds]],
        "idle"
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
    // Published, so the list waits for the app's rebuilt view.
    assert_eq!((changed, messages.len()), (0, 1));
    assert_eq!(domain::<Linear>(state.domains.published(y)), messages[0]);
    assert!(state.domains.owned(y).is_none());
    // The app applied nothing. Until the gesture ends, the widget shows
    // what it published.
    state.rebuilt();
    let (changed, messages) = redraw(&mut state, &fig, t0 + Duration::from_millis(32), &listeners);
    assert!(changed > 0 && messages.is_empty());
    let shown = domain::<Linear>(state.resolved.as_ref().and_then(|r| r.get_scale_at(0)));
    assert_eq!(Some(shown), state.domains.published(y).map(|_| shown));
    // The gesture ends, and the app's domain rules.
    redraw(
        &mut state,
        &fig,
        t0 + Duration::from_millis(400),
        &listeners,
    );
    assert!(state.domains.published(y).is_none());
    let shown = domain::<Linear>(state.resolved.as_ref().and_then(|r| r.get_scale_at(0)));
    assert_eq!(shown, [0.0, 3.0]);
}
