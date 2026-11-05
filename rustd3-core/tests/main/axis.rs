//! Core axes: d3-axis 3.0.0's formulas through a recording `PathContext`
//! for the four orients, the label positions and strings, and the two
//! d3-axis snapshots read with a small hand-written parser.

use std::path::Path;

use rustd3_core::axis::{self, Anchor, Label};
use rustd3_core::locale::Locale;
use rustd3_core::path::PathContext;
use rustd3_core::scale::{self, Ticks};

/// One call a `PathContext` received.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Call {
    Move(f64, f64),
    Line(f64, f64),
}

/// A test double that logs each move and line; axes draw nothing else.
#[derive(Debug, Default)]
struct Recorder(Vec<Call>);

impl PathContext for Recorder {
    fn move_to(&mut self, x: f64, y: f64) {
        self.0.push(Call::Move(x, y));
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.0.push(Call::Line(x, y));
    }

    fn quadratic_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64) {
        panic!("axes draw no curves");
    }

    fn bezier_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: f64) {
        panic!("axes draw no curves");
    }

    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) {
        panic!("axes draw no arcs");
    }

    fn close_path(&mut self) {
        panic!("axes close no path");
    }
}

/// The calls `axis` draws.
fn drawn<S: Ticks>(axis: &axis::Axis<'_, S>) -> Vec<Call> {
    let mut recorder = Recorder::default();
    axis.draw(&mut recorder);
    recorder.0
}

/// The domain path: the calls before the first tick's move.
fn domain(calls: &[Call]) -> &[Call] {
    let end = calls[1..]
        .iter()
        .position(|c| matches!(c, Call::Move(..)))
        .map_or(calls.len(), |i| i + 1);
    &calls[..end]
}

/// The tick marks after the domain path, as (from, to) pairs.
fn marks(calls: &[Call]) -> Vec<((f64, f64), (f64, f64))> {
    calls[domain(calls).len()..]
        .chunks(2)
        .map(|pair| match *pair {
            [Call::Move(x0, y0), Call::Line(x1, y1)] => ((x0, y0), (x1, y1)),
            _ => panic!("a tick is a move and a line: {pair:?}"),
        })
        .collect()
}

/// Whether two numbers agree to 1e-9.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

/// The 960-pixel scale of the spot checks.
fn pixels() -> scale::Linear {
    scale::linear().domain([0.0, 100.0]).range([0.0, 960.0])
}

#[test]
fn a_bottom_axis_draws_its_domain_ticks_and_labels_below() {
    let x = pixels();
    let a = axis::bottom(&x).ticks(8);
    let calls = drawn(&a);
    assert_eq!(
        domain(&calls),
        [
            Call::Move(0.5, 6.0),
            Call::Line(0.5, 0.5),
            Call::Line(960.5, 0.5),
            Call::Line(960.5, 6.0)
        ]
    );
    let ticks = marks(&calls);
    assert_eq!(ticks.len(), 11);
    assert_eq!(ticks[1], ((96.5, 0.0), (96.5, 6.0)));
    let labels = a.labels();
    assert_eq!(labels.len(), 11);
    assert_eq!(
        (labels[1].get_x(), labels[1].get_y(), labels[1].get_text()),
        (96.5, 9.0, "10")
    );
    assert_eq!(
        (labels[1].get_anchor(), labels[1].get_dy()),
        (Anchor::Middle, 0.71)
    );
}

#[test]
fn a_left_axis_of_the_default_scale_matches_d3s_snapshot() {
    let y = scale::linear();
    let a = axis::left(&y).locale(Locale::EN_US);
    let calls = drawn(&a);
    assert_eq!(
        domain(&calls),
        [
            Call::Move(-6.0, 0.5),
            Call::Line(0.5, 0.5),
            Call::Line(0.5, 1.5),
            Call::Line(-6.0, 1.5)
        ]
    );
    let ticks = marks(&calls);
    assert_eq!(ticks.len(), 11);
    for (k, ((x0, y0), (x1, y1))) in ticks.into_iter().enumerate() {
        let y = 0.5 + k as f64 / 10.0;
        assert!(near(y0, y) && near(y1, y), "tick {k} at {y0}, {y1}");
        assert_eq!((x0, x1), (0.0, -6.0));
    }
    let labels = a.labels();
    let texts: Vec<&str> = labels.iter().map(Label::get_text).collect();
    assert_eq!(
        texts,
        [
            "0.0", "0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7", "0.8", "0.9", "1.0"
        ]
    );
    for label in &labels {
        assert_eq!(
            (label.get_x(), label.get_anchor(), label.get_dy()),
            (-9.0, Anchor::End, 0.32)
        );
    }
}

#[test]
fn a_left_axis_over_five_hundred_pixels_matches_d3s_snapshot() {
    let y = scale::linear().range([0.0, 500.0]);
    let calls = drawn(&axis::left(&y));
    assert_eq!(
        domain(&calls),
        [
            Call::Move(-6.0, 0.5),
            Call::Line(0.5, 0.5),
            Call::Line(0.5, 500.5),
            Call::Line(-6.0, 500.5)
        ]
    );
    let at: Vec<f64> = marks(&calls).iter().map(|(from, _)| from.1).collect();
    let expected: Vec<f64> = (0..=10).map(|k| 0.5 + 50.0 * f64::from(k)).collect();
    assert!(
        at.iter().zip(&expected).all(|(a, b)| near(*a, *b)),
        "{at:?}"
    );
}

#[test]
fn an_outer_size_of_zero_draws_a_plain_line() {
    let x = pixels();
    let calls = drawn(&axis::bottom(&x).tick_size_outer(0.0));
    assert_eq!(
        domain(&calls),
        [Call::Move(0.5, 0.5), Call::Line(960.5, 0.5)]
    );
    let y = scale::linear();
    let calls = drawn(&axis::left(&y).tick_size_outer(0.0));
    assert_eq!(domain(&calls), [Call::Move(0.5, 0.5), Call::Line(0.5, 1.5)]);
}

#[test]
fn a_top_axis_draws_ticks_and_labels_upwards() {
    let x = pixels();
    let a = axis::top(&x);
    let calls = drawn(&a);
    assert_eq!(domain(&calls)[0], Call::Move(0.5, -6.0));
    assert!(
        marks(&calls)
            .iter()
            .all(|(from, to)| from.1 == 0.0 && to.1 == -6.0)
    );
    for label in a.labels() {
        assert_eq!(
            (label.get_y(), label.get_dy(), label.get_anchor()),
            (-9.0, 0.0, Anchor::Middle)
        );
    }
}

#[test]
fn a_right_axis_draws_ticks_and_labels_rightwards() {
    let y = scale::linear().range([100.0, 0.0]);
    let a = axis::right(&y).ticks(2);
    let calls = drawn(&a);
    assert_eq!(
        domain(&calls),
        [
            Call::Move(6.0, 100.5),
            Call::Line(0.5, 100.5),
            Call::Line(0.5, 0.5),
            Call::Line(6.0, 0.5)
        ]
    );
    assert_eq!(marks(&calls)[0], ((0.0, 100.5), (6.0, 100.5)));
    let label = &a.labels()[2];
    assert_eq!(
        (
            label.get_x(),
            label.get_y(),
            label.get_text(),
            label.get_anchor(),
            label.get_dy()
        ),
        (9.0, 0.5, "1.0", Anchor::Start, 0.32)
    );
}

#[test]
fn a_negative_inner_size_points_ticks_inwards_and_keeps_the_label_gap() {
    let x = pixels();
    let a = axis::bottom(&x).tick_size_inner(-4.0).ticks(2);
    let calls = drawn(&a);
    assert_eq!(domain(&calls)[0], Call::Move(0.5, 6.0));
    assert_eq!(marks(&calls)[0], ((0.5, 0.0), (0.5, -4.0)));
    assert_eq!(a.labels()[0].get_y(), 3.0);
    let b = axis::left(&x).tick_size(-4.0).tick_padding(5.0).ticks(2);
    assert_eq!(domain(&drawn(&b))[0], Call::Move(4.0, 0.5));
    assert_eq!(b.labels()[0].get_x(), -5.0);
}

#[test]
fn an_offset_of_zero_puts_lines_on_the_range() {
    let x = pixels();
    let a = axis::bottom(&x).offset(0.0).ticks(2);
    let calls = drawn(&a);
    assert_eq!(
        domain(&calls),
        [
            Call::Move(0.0, 6.0),
            Call::Line(0.0, 0.0),
            Call::Line(960.0, 0.0),
            Call::Line(960.0, 6.0)
        ]
    );
    assert_eq!(marks(&calls)[1], ((480.0, 0.0), (480.0, 6.0)));
    assert_eq!(a.labels()[2].get_x(), 960.0);
}

#[test]
fn minor_ticks_follow_the_majors_at_their_own_size() {
    let x = scale::linear().domain([0.0, 10.0]).range([0.0, 100.0]);
    let a = axis::bottom(&x).ticks(5).minor(true).tick_size_minor(2.0);
    let ticks = marks(&drawn(&a));
    assert_eq!(ticks.len(), 6 + 20);
    assert!(ticks[..6].iter().all(|(_, to)| to.1 == 6.0));
    assert!(ticks[6..].iter().all(|(_, to)| to.1 == 2.0));
    assert!(near(ticks[6].0.0, 4.5));
    assert_eq!(marks(&drawn(&axis::bottom(&x).ticks(5))).len(), 6);
}

#[test]
fn chosen_values_and_a_borrowing_format_replace_the_defaults() {
    let x = pixels();
    let unit = String::from(" %");
    let a = axis::bottom(&x)
        .values([25.0, 75.0])
        .format(|v| format!("{v}{unit}"));
    assert_eq!(
        marks(&drawn(&a)),
        [((240.5, 0.0), (240.5, 6.0)), ((720.5, 0.0), (720.5, 6.0))]
    );
    let texts: Vec<String> = a.labels().iter().map(|l| l.get_text().to_owned()).collect();
    assert_eq!(texts, ["25 %", "75 %"]);
}

#[test]
fn chosen_values_keep_the_precision_of_the_tick_count() {
    let y = scale::linear();
    let a = axis::left(&y).values([0.25, 0.5]);
    let texts: Vec<String> = a.labels().iter().map(|l| l.get_text().to_owned()).collect();
    assert_eq!(texts, ["0.3", "0.5"]);
}

#[test]
fn labels_follow_the_axis_locale() {
    let x = scale::linear().domain([0.0, 12000.0]);
    let neutral: Vec<String> = axis::bottom(&x)
        .ticks(5)
        .labels()
        .iter()
        .map(|l| l.get_text().to_owned())
        .collect();
    assert_eq!(neutral[6], "12\u{202f}000");
    let en = axis::bottom(&x).ticks(5).locale(Locale::EN_US).labels();
    assert_eq!(en[6].get_text(), "12,000");
}

#[test]
fn ticks_off_the_scale_are_left_out_of_marks_and_labels() {
    let x = scale::log().domain([1.0, 100.0]).range([0.0, 200.0]);
    let a = axis::bottom(&x).values([-1.0, 10.0, f64::NAN]);
    assert_eq!(marks(&drawn(&a)), [((100.5, 0.0), (100.5, 6.0))]);
    let labels = a.labels();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].get_text(), "10");
}

#[test]
fn an_identity_scale_draws_as_its_stops() {
    let s = scale::identity().domain([0.0, 300.0]);
    let a = axis::bottom(&s).ticks(3);
    assert_eq!(
        domain(&drawn(&a)),
        [
            Call::Move(0.5, 6.0),
            Call::Line(0.5, 0.5),
            Call::Line(300.5, 0.5),
            Call::Line(300.5, 6.0)
        ]
    );
    let texts: Vec<String> = a.labels().iter().map(|l| l.get_text().to_owned()).collect();
    assert_eq!(texts, ["0", "100", "200", "300"]);
}

#[test]
fn an_axis_prints_its_settings() {
    let x = scale::linear();
    let text = format!("{:?}", axis::left(&x).format(|v| v.to_string()));
    assert!(text.contains("orient: Left") && text.contains("format: Some(\"custom\")"));
}

/// What one d3-axis snapshot draws: the root's text anchor, the domain
/// path's points and each tick.
#[derive(Debug, Default)]
struct Snapshot {
    anchor: String,
    domain: Vec<(f64, f64)>,
    ticks: Vec<SnapshotTick>,
}

/// One `<g class="tick">`: its translation, its line's end and its text.
#[derive(Debug, Default)]
struct SnapshotTick {
    at: (f64, f64),
    line: (f64, f64),
    text: String,
    text_at: (f64, f64),
    dy: String,
}

/// The value of attribute `name` in the opening tag `tag`.
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let len = tag[start..].find('"')?;
    Some(&tag[start..start + len])
}

/// A number of the snapshot.
fn number(text: &str) -> f64 {
    text.parse()
        .unwrap_or_else(|_| panic!("a number in the snapshot: {text}"))
}

/// The points of path data made of `M`, `L`, `H` and `V` segments, each
/// `H`/`V` rewritten to the point it reaches.
fn points(d: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut at = (0.0, 0.0);
    let mut rest = d;
    while let Some(command) = rest.chars().next() {
        let end = rest[1..]
            .find(|c: char| c.is_ascii_alphabetic())
            .map_or(rest.len(), |i| i + 1);
        let args: Vec<f64> = rest[1..end].split(',').map(number).collect();
        at = match (command, args.as_slice()) {
            ('M' | 'L', [x, y]) => (*x, *y),
            ('H', [x]) => (*x, at.1),
            ('V', [y]) => (at.0, *y),
            _ => panic!("an unexpected segment {command} {args:?}"),
        };
        out.push(at);
        rest = &rest[end..];
    }
    out
}

/// The snapshot `name` under d3-axis's test directory, parsed.
fn snapshot(name: &str) -> Snapshot {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/d3/d3-axis/snapshots")
        .join(format!("{name}.html"));
    let html = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut snapshot = Snapshot::default();
    for piece in html.split('<').skip(1) {
        let (tag, content) = piece.split_once('>').expect("a closed tag");
        let tick = snapshot.ticks.last_mut();
        match (tag.split_whitespace().next(), tick) {
            (Some("g"), _) if attr(tag, "class") == Some("tick") => {
                let translate = attr(tag, "transform").expect("a tick's transform");
                let inner = translate
                    .strip_prefix("translate(")
                    .and_then(|t| t.strip_suffix(')'))
                    .expect("a translation");
                let (x, y) = inner.split_once(',').expect("two coordinates");
                snapshot.ticks.push(SnapshotTick {
                    at: (number(x), number(y)),
                    ..SnapshotTick::default()
                });
            }
            (Some("g"), _) => {
                if let Some(anchor) = attr(tag, "text-anchor") {
                    snapshot.anchor = anchor.to_owned();
                }
            }
            (Some("path"), _) => snapshot.domain = points(attr(tag, "d").expect("path data")),
            (Some("line"), Some(tick)) => {
                let end = |a| attr(tag, a).map_or(0.0, number);
                tick.line = (tick.at.0 + end("x2"), tick.at.1 + end("y2"));
            }
            (Some("text"), Some(tick)) => {
                let shift = |a| attr(tag, a).map_or(0.0, number);
                tick.text_at = (tick.at.0 + shift("x"), tick.at.1 + shift("y"));
                tick.dy = attr(tag, "dy").expect("a dy").to_owned();
                tick.text = content.to_owned();
            }
            _ => {}
        }
    }
    snapshot
}

/// Whether two points agree to 1e-9.
fn same(a: (f64, f64), b: (f64, f64)) -> bool {
    near(a.0, b.0) && near(a.1, b.1)
}

#[test]
fn axis_snapshots_of_d3_are_reproduced() {
    let unit = scale::linear();
    let tall = scale::linear().range([0.0, 500.0]);
    for (name, scale) in [
        ("axisLeftScaleLinear", &unit),
        ("axisLeftScaleLinearNonNumericRange", &tall),
    ] {
        let expected = snapshot(name);
        let a = axis::left(scale).locale(Locale::EN_US);
        let calls = drawn(&a);
        let ours: Vec<(f64, f64)> = domain(&calls)
            .iter()
            .map(|c| match *c {
                Call::Move(x, y) | Call::Line(x, y) => (x, y),
            })
            .collect();
        assert_eq!(ours.len(), expected.domain.len(), "{name}: domain");
        assert!(
            ours.iter().zip(&expected.domain).all(|(a, b)| same(*a, *b)),
            "{name}: domain {ours:?} against {:?}",
            expected.domain
        );
        let ticks = marks(&calls);
        let labels = a.labels();
        assert_eq!(ticks.len(), expected.ticks.len(), "{name}: ticks");
        assert_eq!(labels.len(), expected.ticks.len(), "{name}: labels");
        assert_eq!(expected.anchor, "end");
        for ((mark, label), tick) in ticks.iter().zip(&labels).zip(&expected.ticks) {
            assert!(
                same(mark.0, tick.at),
                "{name}: tick {mark:?} against {tick:?}"
            );
            assert!(
                same(mark.1, tick.line),
                "{name}: tick {mark:?} against {tick:?}"
            );
            assert!(
                same((label.get_x(), label.get_y()), tick.text_at),
                "{name}: {label:?}"
            );
            assert_eq!(label.get_text(), tick.text, "{name}");
            assert_eq!(format!("{}em", label.get_dy()), tick.dy, "{name}");
            assert_eq!(label.get_anchor(), Anchor::End, "{name}");
        }
    }
}
