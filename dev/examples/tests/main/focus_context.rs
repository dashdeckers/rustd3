//! Focus and context through the simulator: each test sends events to the view,
//! applies the messages it published to the app in order, and checks the
//! window the app shows.

use iced::mouse::{self, ScrollDelta};
use iced::{Event, Point};
use iced_test::Simulator;
use rustd3_core::jiff::Timestamp;
use rustd3_examples::focus_context::{App, Message};

use crate::simulator;

/// The window: the focus above, the overview from y = 340; plots 860 px
/// wide between the 40 px and 24 px margins.
const SIZE: (f32, f32) = (924.0, 430.0);
/// A row through the overview's plot.
const OVERVIEW_Y: f32 = 380.0;
/// The data's 719 hours over the plot's 860 px.
const HOURS_PER_PX: f64 = 719.0 / 860.0;

/// Applies `messages` to `app` in order; the last one.
fn apply(app: &mut App, messages: impl IntoIterator<Item = Message>) -> Option<Message> {
    let mut last = None;
    for message in messages {
        app.update(message.clone());
        last = Some(message);
    }
    last
}

/// Hours from the first data hour to `t`.
fn hours(app: &App, t: Timestamp) -> f64 {
    t.duration_since(app.full()[0]).as_secs_f64() / 3600.0
}

/// Asserts that `window` runs from `from` to `to` hours after the first
/// data hour, within `seconds`.
fn assert_window(app: &App, window: [Timestamp; 2], [from, to]: [f64; 2], seconds: f64) {
    let [a, b] = window.map(|t| hours(app, t));
    let close = |h: f64, want: f64| (h - want).abs() * 3600.0 <= seconds;
    assert!(
        close(a, from) && close(b, to),
        "window [{a:.3} h, {b:.3} h], expected [{from:.3} h, {to:.3} h]"
    );
}

/// Twenty wheel lines at canvas (450, 150), d3's sign: zoom in; `None`
/// when the simulator is skipped.
fn zoom_in(app: &mut App) -> Option<()> {
    let mut ui = simulator(SIZE, app.view())?;
    ui.point_at(Point::new(450.0, 150.0));
    let _ = ui.scroll(ScrollDelta::Lines { x: 0.0, y: 20.0 });
    let messages: Vec<Message> = ui.into_messages().collect();
    apply(app, messages);
    Some(())
}

/// The window's span in seconds.
fn span([a, b]: [Timestamp; 2]) -> f64 {
    b.duration_since(a).as_secs_f64()
}

/// A left-button drag across the overview from canvas x `from` to `to`,
/// one event per `simulate`, the app not updated in between.
fn drag(ui: &mut Simulator<'_, Message>, from: f32, to: f32) {
    ui.point_at(Point::new(from, OVERVIEW_Y));
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);
    let position = Point::new(to, OVERVIEW_Y);
    ui.point_at(position);
    let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position })]);
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
}

#[test]
fn wheel_over_the_focus_halves_the_window_around_the_cursor() {
    let mut app = App::new();
    if zoom_in(&mut app).is_none() {
        return;
    }
    // k = 2^(20 · 0.05) = 2 about range x = 410: px 205 and 635 of 860,
    // [171.390 h, 530.890 h].
    let window = [205.0, 635.0].map(|px| px * HOURS_PER_PX);
    assert_window(&app, app.visible(), window, 1.0);
    assert!((span(app.visible()) - 359.5 * 3600.0).abs() <= 1.0);
}

#[test]
fn brushing_the_overview_sets_the_window() {
    let mut app = App::new();
    let Some(mut ui) = simulator(SIZE, app.view()) else {
        return;
    };
    drag(&mut ui, 300.0, 520.0);
    let messages: Vec<Message> = ui.into_messages().collect();
    // The last message wins; without the brush's guard it is Show(full).
    let Some(Message::Show(last)) = apply(&mut app, messages) else {
        panic!("the brush publishes its window");
    };
    // Px 260 and 480 of 860: [217.372 h, 401.302 h], 183.930 h, about
    // 7.7 days.
    let window = [260.0, 480.0].map(|px| px * HOURS_PER_PX);
    assert_window(&app, last, window, 60.0);
    assert_eq!(app.visible(), last);
}

#[test]
fn after_a_zoom_dragging_the_band_moves_the_window() {
    let mut app = App::new();
    if zoom_in(&mut app).is_none() {
        return;
    }
    let before = app.visible();
    let Some(mut ui) = simulator(SIZE, app.view()) else {
        return;
    };
    // The band spans px 205 to 635: press inside it and move it 86 px.
    drag(&mut ui, 40.0 + 420.0, 40.0 + 420.0 + 86.0);
    let messages: Vec<Message> = ui.into_messages().collect();
    apply(&mut app, messages);
    let shift = 86.0 * HOURS_PER_PX; // 71.9 h
    let window = before.map(|t| hours(&app, t) + shift);
    assert_window(&app, app.visible(), window, 1.0);
    assert!((span(app.visible()) - span(before)).abs() <= 1.0);
}
