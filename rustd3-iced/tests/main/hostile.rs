//! Random event sequences into the `Plot` widget through the headless
//! harness: pointer moves to NaN, infinite and far positions, presses and
//! releases, wheel steps with NaN, infinite and huge deltas, window
//! factors that are not positive or finite, resizes to zero and to large
//! sizes, and frames at odd instants, over figures with hostile domains.
//! Nothing panics in `layout`, `update` or `draw`.

use std::time::Duration;

use iced::time::Instant;
use iced::{Event, Point, Size, keyboard, mouse, window};
use rustd3_core::scale;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Figure, axis, brush};
use rustd3_iced::Plot;
use rustd3_iced::plot::{Caching, Render};
use rustd3_testkit::hostile::{self, Rng};

use crate::headless::{self, Harness};

const CASES: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Message {
    X([f64; 2]),
    Y([f64; 2]),
    Brushed(Option<[f64; 2]>),
    Hovered,
}

fn f32_hostile(rng: &mut Rng) -> f32 {
    let v = rng.f64();
    // Saturates; NaN stays NaN.
    v as f32
}

fn position(rng: &mut Rng, size: Size) -> Point {
    if rng.chance(0.2) {
        Point::new(f32_hostile(rng), f32_hostile(rng))
    } else {
        Point::new(
            (rng.unit() as f32) * size.width,
            (rng.unit() as f32) * size.height,
        )
    }
}

fn plot(rng: &mut Rng) -> Plot<'static, Message> {
    let mut fig = Figure::new();
    let x = if rng.chance(0.5) {
        fig.scale(scale::linear().domain([rng.f64(), rng.f64()]))
    } else {
        fig.fit(scale::linear())
    };
    let y = fig.fit(scale::linear());
    let data: Vec<(f64, f64)> = rng
        .values(30)
        .into_iter()
        .map(|v| {
            (
                v,
                if rng.chance(0.2) {
                    rng.f64()
                } else {
                    rng.plain()
                },
            )
        })
        .collect();
    let panel = fig.panel(0, 0);
    panel
        .mark(dots(data.clone(), x, y))
        .mark(line(data, x, y))
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .zoom_x(x)
        .zoom_y(y);
    if rng.chance(0.3) {
        panel.brush(brush::x(x));
    }
    let mut plot = Plot::new(fig)
        .render(*rng.pick(&[Render::Inline, Render::Inline, Render::Worker, Render::Auto]))
        .caching(*rng.pick(&[Caching::On, Caching::Off, Caching::Verify]))
        .crosshair(rng.chance(0.5))
        .highlight(rng.chance(0.5))
        .threads(rng.small(4))
        .on_change(x, Message::X)
        .on_change(y, Message::Y)
        .on_brush(x, Message::Brushed)
        .on_hover(|_| Message::Hovered);
    if rng.chance(0.3) {
        plot = plot.scale_factor(f32_hostile(rng));
    }
    plot
}

fn size(rng: &mut Rng) -> Size {
    match rng.below(6) {
        0 => Size::ZERO,
        1 => Size::new(1e6, 2.0),
        2 => Size::new(rng.unit() as f32 * 3.0, rng.unit() as f32 * 3.0),
        3 => Size::new(
            1.0 + rng.unit() as f32 * 2000.0,
            1.0 + rng.unit() as f32 * 40.0,
        ),
        _ => Size::new(
            100.0 + rng.unit() as f32 * 500.0,
            80.0 + rng.unit() as f32 * 400.0,
        ),
    }
}

fn event(rng: &mut Rng, size: Size, now: &mut Instant) -> Event {
    let button = *rng.pick(&[
        mouse::Button::Left,
        mouse::Button::Right,
        mouse::Button::Middle,
    ]);
    match rng.below(12) {
        0 | 1 => Event::Mouse(mouse::Event::CursorMoved {
            position: position(rng, size),
        }),
        2 => Event::Mouse(mouse::Event::ButtonPressed(button)),
        3 => Event::Mouse(mouse::Event::ButtonReleased(button)),
        4 => {
            let (x, y) = (f32_hostile(rng), f32_hostile(rng));
            let delta = if rng.chance(0.5) {
                mouse::ScrollDelta::Lines { x, y }
            } else {
                mouse::ScrollDelta::Pixels { x, y }
            };
            Event::Mouse(mouse::Event::WheelScrolled { delta })
        }
        5 => Event::Mouse(mouse::Event::CursorLeft),
        6 => Event::Window(window::Event::Rescaled(f32_hostile(rng))),
        7 => Event::Keyboard(keyboard::Event::ModifiersChanged(*rng.pick(&[
            keyboard::Modifiers::SHIFT,
            keyboard::Modifiers::CTRL,
            keyboard::Modifiers::empty(),
        ]))),
        8 => Event::Waken,
        _ => {
            let step = match rng.below(4) {
                0 => Duration::ZERO,
                1 => Duration::from_millis(400),
                _ => Duration::from_millis(16),
            };
            *now += step;
            Event::Window(window::Event::RedrawRequested(*now))
        }
    }
}

#[test]
fn random_event_sequences_never_panic_the_widget() {
    hostile::run("widget", 0x91_07, CASES, |rng| {
        let mut size = size(rng);
        let mut now = headless::epoch();
        let scale = *rng.pick(&[1.0, 1.25, 2.0]);
        let Some(mut harness) = Harness::new(plot(rng), size, scale) else {
            return;
        };
        for _ in 0..rng.below(40) {
            match rng.below(20) {
                0 => {
                    size = self::size(rng);
                    harness.resize(plot(rng), size);
                }
                1 => harness.rebuild(plot(rng)),
                2 => harness.draw(),
                _ => {
                    let e = event(rng, size, &mut now);
                    if let Event::Mouse(mouse::Event::CursorMoved { position }) = e {
                        harness.point_at(position);
                    }
                    let _ = harness.events(&[e]);
                    let _ = harness.messages();
                }
            }
        }
        now += Duration::from_millis(16);
        let _ = harness.events(&[Event::Window(window::Event::RedrawRequested(now))]);
        harness.draw();
    });
}
