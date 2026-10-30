//! The update pattern at fixed instants: `t0` is any instant and `ms(n)` is `n`
//! milliseconds after it; the values depend only on the offsets.

use std::time::{Duration, Instant};

use iced::Theme;
use rustd3_core::anim::Phase;
use rustd3_examples::update_pattern::{App, Bar, Message};

use crate::simulator;

/// The chart's window.
const SIZE: (f32, f32) = (600.0, 260.0);
/// The five rows' tops: d3's band over [0, 250], step 250 / 5.1, padding
/// 0.1.
const ROWS: [f64; 5] = [4.902, 53.922, 102.941, 151.961, 200.980];
/// The first data (seed 1) as widths in px, largest first (the LCG
/// replayed in Python).
const FIRST: [(&str, f64); 5] = [
    ("gnu", 387.381),
    ("dog", 352.442),
    ("hen", 278.094),
    ("cat", 252.121),
    ("fox", 184.759),
];
/// The second data.
const SECOND: [(&str, f64); 5] = [
    ("hen", 490.423),
    ("fox", 418.467),
    ("bee", 319.623),
    ("eel", 295.345),
    ("dog", 211.189),
];

/// Any base instant: values depend only on the offsets from it.
fn base() -> (Instant, impl Fn(u64) -> Instant) {
    let t0 = Instant::now();
    (t0, move |n| t0 + Duration::from_millis(n))
}

/// The bar named `name` and its phase.
fn bar(app: &App, name: &str) -> (Bar, Phase) {
    app.bars()
        .into_iter()
        .find(|b| b.0 == name)
        .map(|(_, bar, phase)| (bar, phase))
        .unwrap_or_else(|| panic!("no bar {name}"))
}

/// Whether `a` and `b` agree to the spot rows' three decimals.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 5e-4
}

/// Asserts that the app's bars are exactly `ranked`, at rest in rank
/// order: present, at their widths and rows, opaque.
fn assert_at_rest(app: &App, ranked: [(&str, f64); 5]) {
    assert_eq!(app.bars().len(), 5, "{:?}", app.bars());
    for ((name, width), row) in ranked.into_iter().zip(ROWS) {
        let (b, phase) = bar(app, name);
        assert_eq!(phase, Phase::Present, "{name}");
        assert!(near(b.width, width) && near(b.y, row), "{name}: {b:?}");
        assert_eq!(b.opacity, 1.0, "{name}");
    }
}

#[test]
fn first_update_grows_five_bars_from_zero_width() {
    let (t0, ms) = base();
    let mut app = App::new(t0);
    let names: Vec<&str> = app.bars().iter().map(|b| b.0).collect();
    assert_eq!(names, FIRST.map(|r| r.0));
    for ((name, _), row) in FIRST.into_iter().zip(ROWS) {
        let (b, phase) = bar(&app, name);
        assert_eq!(phase, Phase::Present);
        assert!(b.width == 0.0 && near(b.y, row), "{name}: {b:?}");
    }
    // The last of five bars starts 4 × 20 ms late: it settles at 830 ms.
    app.update(Message::Frame(ms(829)));
    assert!(app.is_animating());
    app.update(Message::Frame(ms(830)));
    assert!(!app.is_animating());
    assert_at_rest(&app, FIRST);
}

#[test]
fn second_update_exits_two_and_enters_two() {
    let (t0, ms) = base();
    let mut app = App::new(t0);
    app.update(Message::Frame(ms(1500)));
    let before = |name| bar(&app, name).0;
    let (hen, gnu, cat) = (before("hen"), before("gnu"), before("cat"));
    assert!(near(hen.y, 102.941));
    app.update(Message::NewData(ms(1500)));
    // gnu and cat leave from where they are.
    assert_eq!(bar(&app, "gnu"), (gnu, Phase::Exiting));
    assert_eq!(bar(&app, "cat"), (cat, Phase::Exiting));
    // bee and eel enter with zero width in their rows.
    for (name, row) in [("bee", ROWS[2]), ("eel", ROWS[3])] {
        let (b, phase) = bar(&app, name);
        assert_eq!(phase, Phase::Present);
        assert!(b.width == 0.0 && near(b.y, row), "{name}: {b:?}");
    }
    // hen heads from row 2 to row 0 without a jump.
    assert_eq!(bar(&app, "hen"), (hen, Phase::Present));
    app.update(Message::Frame(ms(1900)));
    let moving = bar(&app, "hen").0.y;
    assert!(ROWS[0] < moving && moving < hen.y, "hen at {moving}");
    // Exits end at 2250 ms, the last move at 1500 + 830 ms; the frame then
    // prunes the exits.
    app.update(Message::Frame(ms(2330)));
    assert!(!app.is_animating());
    assert_at_rest(&app, SECOND);
}

#[test]
fn chart_renders_on_each_backend() {
    let (t0, ms) = base();
    let mut app = App::new(t0);
    app.update(Message::Frame(ms(400)));
    let Some(mut ui) = simulator(SIZE, app.view()) else {
        return;
    };
    assert!(ui.snapshot(&Theme::Light).is_ok());
}
