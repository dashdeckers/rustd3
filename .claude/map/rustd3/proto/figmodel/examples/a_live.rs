//! Focus and context under option A, through the widget
//! logic the iced glue would run. Once with the app owning
//! the window (controlled), once with the widget owning it.

use d3::axis::Orient::{Bottom, Left};
use d3::interact::Pointer;
use d3::scale;
use d3::time::{Time, time};
use figmodel::fig::{Figure, ScaleRef};
use figmodel::live::{Chart, State};
use jiff::{SignedDuration, Timestamp, tz::TimeZone};

const SIZE: [f64; 2] = [720.0, 400.0];

struct App {
    data: Vec<(Timestamp, f64)>,
    window: Option<[Timestamp; 2]>,
}

#[derive(Debug, Clone)]
enum Msg {
    Window([Timestamp; 2]),
}

impl App {
    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Window(w) => self.window = Some(w),
        }
    }

    fn figure(&self) -> (Figure, ScaleRef<Time>) {
        let mut fig = Figure::new();
        let full = fig.fit(time(TimeZone::UTC));
        let focus = match self.window {
            Some(w) => {
                fig.scale(time(TimeZone::UTC).domain(w))
            }
            None => fig.fit(time(TimeZone::UTC)),
        };
        let y = fig.fit(scale::linear());
        let data = || self.data.iter().copied();
        fig.panel(0, 0)
            .line(data(), focus, y)
            .axis(Bottom, focus)
            .axis(Left, y)
            .zoom_x(focus);
        fig.panel(1, 0)
            .line(data(), full, y)
            .axis(Bottom, full)
            .brush_x(full, focus);
        (fig, focus)
    }
}

fn day(t: Timestamp) -> String {
    t.strftime("%m-%d %H:%M").to_string()
}

fn show(label: &str, fig: &Figure, state: &State) {
    let chart: Chart<'_, Msg> = Chart::new(fig, SIZE);
    let r = chart.resolved(state);
    let d = r.scales[1]
        .domain()
        .map(|v| day(Timestamp::from_millisecond(v as i64).unwrap()));
    println!("{label:30} focus {} .. {}", d[0], d[1]);
}

/// Feeds events in batches; the app sees a batch's
/// messages only after the whole batch, as when iced
/// delivers several events before the next update.
fn run(
    app: &mut App,
    state: &mut State,
    events: &[Pointer],
    batch: usize,
    controlled: bool,
) {
    for chunk in events.chunks(batch) {
        let (fig, focus) = app.figure();
        let chart = Chart::new(&fig, SIZE);
        let chart = if controlled {
            chart.on_change(focus, Msg::Window)
        } else {
            chart
        };
        let msgs: Vec<Msg> = chunk
            .iter()
            .filter_map(|&e| chart.update(state, e))
            .collect();
        msgs.into_iter().for_each(|m| app.update(m));
    }
}

fn main() {
    let start: Timestamp = "2026-09-01T00:00:00Z".parse().unwrap();
    let data = (0..24 * 30)
        .map(|h| {
            let t = start + SignedDuration::from_hours(h);
            (t, (h as f64 / 24.0).sin() * 2.0 + 10.0)
        })
        .collect();
    let mut app = App { data, window: None };

    // Focus plot: y 12..172; context: y 212..372.
    let zoom = [Pointer::Wheel { x: 400.0, y: 90.0, lines: -20.0 }];
    let pan: Vec<Pointer> = [Pointer::Pressed { x: 400.0, y: 90.0 }]
        .into_iter()
        .chain((1..=6).map(|i| Pointer::Moved {
            x: 400.0 - 20.0 * i as f64,
            y: 90.0,
        }))
        .chain([Pointer::Released])
        .collect();
    let brush = [
        Pointer::Pressed { x: 200.0, y: 290.0 },
        Pointer::Moved { x: 300.0, y: 290.0 },
        Pointer::Moved { x: 420.0, y: 290.0 },
        Pointer::Released,
    ];

    for controlled in [true, false] {
        println!("-- controlled: {controlled}");
        app.window = None;
        let mut state = State::default();
        let mut step = |label: &str, events: &[Pointer], batch| {
            run(&mut app, &mut state, events, batch, controlled);
            let (fig, _) = app.figure();
            show(label, &fig, &state);
            let name = label.split(' ').next().unwrap();
            let chart: Chart<'_, Msg> = Chart::new(&fig, SIZE);
            let file = format!("out/live_{controlled}_{name}.svg");
            std::fs::write(file, chart.resolved(&state).svg(&fig)).unwrap();
        };
        step("start", &[], 1);
        step("wheel in focus", &zoom, 1);
        step("pan in focus, 1 event/update", &pan, 1);
        step("brush in context", &brush, 1);
        println!("app.window is set: {}", app.window.is_some());
    }

    // The same pan, with the app three events behind.
    for batch in [1, 3] {
        app.window = None;
        let mut state = State::default();
        run(&mut app, &mut state, &zoom, 1, true);
        run(&mut app, &mut state, &pan, batch, true);
        let w = app.window.unwrap().map(day);
        println!("pan, {batch} events per update: {} .. {}", w[0], w[1]);
    }
}
