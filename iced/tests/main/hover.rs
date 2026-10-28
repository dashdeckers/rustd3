//! Hover through the headless harness on the backend `ICED_TEST_BACKEND`
//! names: the hovered point's message, deduplicated, its panel mapped back
//! to its grid cell, the pointer leaving, and moves that resolve nothing.

use std::sync::mpsc;
use std::time::Duration;

use iced::time::Instant;
use iced::{Event, Point, Size, mouse};
use rustd3_core::scale;
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::mark::{bar_y, dots, line};
use rustd3_figure::{Cache, Figure, Held, Resolved, Target, axis};
use rustd3_iced::Plot;
use rustd3_iced::plot::probe::Sample;
use rustd3_iced::plot::{Hover, Probe, Render};

use crate::headless::{self, Harness};

/// The figure's logical size.
pub const SIZE: Size = Size::new(640.0, 480.0);

/// One frame at 60 Hz.
const FRAME: Duration = Duration::from_millis(16);

/// The dots of panel (1, 0), on [0, 10] × [0, 10].
pub const DOTS: [(f64, f64); 3] = [(2.0, 3.0), (5.0, 8.0), (8.0, 4.0)];

/// The flat index of panel (1, 0): panels are numbered in the order of
/// first use.
pub const DOTS_PANEL: usize = 3;

/// What the test app hears.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    Hovered(Option<Hover>),
}

/// A 2 × 2 figure on fixed scales, its panels made out of row order: a
/// line in (0, 0), bars in (0, 1), dots in (1, 1) and [`DOTS`] in (1, 0),
/// each with left and bottom axes.
pub fn grid() -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let keys = fig.scale(scale::band(["a", "b", "c"]).padding(0.2));
    let wave: Vec<(f64, f64)> = (0..=40)
        .map(|i| {
            let t = f64::from(i) / 4.0;
            (t, 5.0 + 3.0 * (t * 0.8).sin())
        })
        .collect();
    fig.panel(0, 0)
        .mark(line(wave, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig.panel(0, 1)
        .mark(bar_y([("a", 3.0), ("b", 7.0), ("c", 5.0)], keys, y))
        .axis(axis::left(y))
        .axis(axis::bottom(keys));
    fig.panel(1, 1)
        .mark(dots([(1.0, 1.0), (9.0, 9.0)], x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig.panel(1, 0)
        .mark(dots(DOTS, x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    fig
}

/// `fig` resolved at `SIZE` for the screen at `k`.
pub fn resolve(fig: &Figure<'_>, k: f32) -> Resolved {
    let size = kurbo::Size::new(f64::from(SIZE.width), f64::from(SIZE.height));
    let mut cache = Cache::default();
    cache.begin();
    let target = Target::Screen {
        scale: f64::from(k),
    };
    fig.resolve(size, &mut cache, &Held::default(), target)
}

/// Where `(x, y)` of [0, 10] × [0, 10] lies in `area`, logical px.
pub fn at(area: Rect, (x, y): (f64, f64)) -> Point {
    Point::new(
        (area.x0 + x / 10.0 * area.width()) as f32,
        (area.y1 - y / 10.0 * area.height()) as f32,
    )
}

/// Moves the cursor to `to`.
fn move_to(harness: &mut Harness<'static, Message>, to: Point) {
    harness.point_at(to);
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorMoved { position: to })]);
}

/// A harness on [`grid`] at factor `k` reporting hovers, after its first
/// frame at `t0`, and the figure resolved as it shows it; `None` when the
/// backend is skipped.
fn hovering(
    k: f32,
    t0: Instant,
    probe: Option<Probe>,
) -> Option<(Harness<'static, Message>, Resolved)> {
    let resolved = resolve(&grid(), k);
    let plot = Plot::new(grid())
        .scale_factor(k)
        .render(Render::Inline)
        .on_hover(Message::Hovered);
    let plot = match probe {
        Some(probe) => plot.probe(probe),
        None => plot,
    };
    let mut harness = Harness::new(plot, SIZE, k)?;
    let _ = harness.frame(t0);
    assert!(harness.messages().is_empty(), "nothing hovered yet");
    Some((harness, resolved))
}

#[test]
fn hovering_a_dot_reports_its_index() {
    for k in [1.0, 2.0] {
        let t0 = headless::epoch();
        let Some((mut harness, resolved)) = hovering(k, t0, None) else {
            return;
        };
        let backend = harness.backend();
        let areas = resolved.plot_areas();
        let dot = at(areas[DOTS_PANEL], DOTS[1]);
        move_to(&mut harness, dot);
        let _ = harness.frame(t0 + FRAME);
        let messages = harness.messages();
        let [Message::Hovered(Some(hover))] = messages[..] else {
            panic!("{backend}, k = {k}: one hover, {messages:?}");
        };
        println!("hover, {backend}, k = {k}: {hover:?}");
        assert_eq!(
            (hover.panel, hover.mark, hover.index),
            (DOTS_PANEL, 0, 1),
            "{backend}"
        );
        assert_eq!(resolved.panel_cell(hover.panel), (1, 0), "{backend}");
        assert_eq!(hover.at, dot, "{backend}: widget coordinates");
        // The line's samples and the bars' value ends are hovered too.
        let sample = at(areas[0], (2.5, 5.0 + 3.0 * 2.0f64.sin()));
        move_to(&mut harness, sample);
        let _ = harness.frame(t0 + FRAME * 2);
        let messages = harness.messages();
        assert!(
            matches!(
                messages[..],
                [Message::Hovered(Some(Hover {
                    panel: 0,
                    mark: 0,
                    index: 10,
                    ..
                }))]
            ),
            "{backend}, k = {k}: the line's sample 10, {messages:?}"
        );
        let bars = areas[1];
        let top = Point::new(
            bars.center().x as f32,
            (bars.y1 - 0.7 * bars.height()) as f32,
        );
        move_to(&mut harness, top);
        let _ = harness.frame(t0 + FRAME * 3);
        let messages = harness.messages();
        assert!(
            matches!(
                messages[..],
                [Message::Hovered(Some(Hover {
                    panel: 1,
                    mark: 0,
                    index: 1,
                    ..
                }))]
            ),
            "{backend}, k = {k}: bar b's top, {messages:?}"
        );
    }
}

#[test]
fn moving_within_one_dot_sends_nothing() {
    let k = 2.0;
    let t0 = headless::epoch();
    let Some((mut harness, resolved)) = hovering(k, t0, None) else {
        return;
    };
    let backend = harness.backend();
    let dot = at(resolved.plot_areas()[DOTS_PANEL], DOTS[0]);
    move_to(&mut harness, dot);
    let _ = harness.frame(t0 + FRAME);
    let messages = harness.messages();
    assert!(
        matches!(
            messages[..],
            [Message::Hovered(Some(Hover {
                mark: 0,
                index: 0,
                ..
            }))]
        ),
        "{backend}: {messages:?}"
    );
    // One px off, well inside the 12 px radius: the same dot.
    move_to(&mut harness, Point::new(dot.x + 1.0, dot.y));
    let _ = harness.frame(t0 + FRAME * 2);
    assert!(harness.messages().is_empty(), "{backend}");
    move_to(&mut harness, Point::new(dot.x, dot.y - 1.0));
    let _ = harness.frame(t0 + FRAME * 3);
    assert!(harness.messages().is_empty(), "{backend}");
}

#[test]
fn leaving_the_widget_reports_none_once() {
    let k = 1.0;
    let t0 = headless::epoch();
    let Some((mut harness, resolved)) = hovering(k, t0, None) else {
        return;
    };
    let backend = harness.backend();
    let areas = resolved.plot_areas();
    move_to(&mut harness, at(areas[DOTS_PANEL], DOTS[2]));
    let _ = harness.frame(t0 + FRAME);
    assert_eq!(harness.messages().len(), 1, "{backend}");
    let _ = harness.events(&[Event::Mouse(mouse::Event::CursorLeft)]);
    let _ = harness.frame(t0 + FRAME * 2);
    assert_eq!(
        harness.messages(),
        [Message::Hovered(None)],
        "{backend}: the pointer left"
    );
    let _ = harness.frame(t0 + FRAME * 3);
    assert!(harness.messages().is_empty(), "{backend}: once");
    // Back over a dot, then into a gutter: lost once more.
    move_to(&mut harness, at(areas[DOTS_PANEL], DOTS[2]));
    let _ = harness.frame(t0 + FRAME * 4);
    assert_eq!(harness.messages().len(), 1, "{backend}");
    move_to(&mut harness, Point::new(2.0, 2.0));
    let _ = harness.frame(t0 + FRAME * 5);
    assert_eq!(harness.messages(), [Message::Hovered(None)], "{backend}");
}

/// The clock the probe times stages with.
#[expect(clippy::disallowed_methods, reason = "test clock")]
fn clock() -> Instant {
    Instant::now()
}

#[test]
fn hover_does_not_rebuild_the_hit_index() {
    let k = 1.0;
    let t0 = headless::epoch();
    let (tx, rx) = mpsc::channel();
    let Some((mut harness, resolved)) = hovering(k, t0, Some(Probe::new(clock, tx))) else {
        return;
    };
    let backend = harness.backend();
    let first: Vec<Sample> = rx.try_iter().collect();
    let stages = |samples: &[Sample]| {
        samples
            .iter()
            .filter(|sample| matches!(sample, Sample::Stage { .. }))
            .count()
    };
    assert_eq!(stages(&first), 2, "{backend}: the first frame resolves");
    // Ten moves between two redraws, across every dot of the panel.
    let area = resolved.plot_areas()[DOTS_PANEL];
    for i in 0..10_u8 {
        let share = f64::from(i) / 9.0;
        move_to(
            &mut harness,
            at(area, (1.0 + 8.0 * share, 2.0 + 6.0 * share)),
        );
    }
    move_to(&mut harness, at(area, DOTS[2]));
    let _ = harness.frame(t0 + FRAME);
    let samples: Vec<Sample> = rx.try_iter().collect();
    println!(
        "hover between redraws, {backend}: {} samples, {} stages",
        samples.len(),
        stages(&samples)
    );
    assert_eq!(stages(&samples), 0, "{backend}: no Resolve or List sample");
    assert!(
        matches!(samples[..], [Sample::Frame { .. }]),
        "{backend}: {samples:?}"
    );
    // One message for the frame: the dot the pointer ended on.
    let messages = harness.messages();
    assert!(
        matches!(
            messages[..],
            [Message::Hovered(Some(Hover { index: 2, .. }))]
        ),
        "{backend}: {messages:?}"
    );
}
