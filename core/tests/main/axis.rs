//! Core axes: d3-axis 3.0.0's formulas through a recording `PathContext`
//! for the four orients, and the label positions and strings.

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
        (labels[1].x(), labels[1].y(), labels[1].text()),
        (96.5, 9.0, "10")
    );
    assert_eq!((labels[1].anchor(), labels[1].dy()), (Anchor::Middle, 0.71));
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
    let texts: Vec<&str> = labels.iter().map(Label::text).collect();
    assert_eq!(
        texts,
        [
            "0.0", "0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7", "0.8", "0.9", "1.0"
        ]
    );
    for label in &labels {
        assert_eq!(
            (label.x(), label.anchor(), label.dy()),
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
            (label.y(), label.dy(), label.anchor()),
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
            label.x(),
            label.y(),
            label.text(),
            label.anchor(),
            label.dy()
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
    assert_eq!(a.labels()[0].y(), 3.0);
    let b = axis::left(&x).tick_size(-4.0).tick_padding(5.0).ticks(2);
    assert_eq!(domain(&drawn(&b))[0], Call::Move(4.0, 0.5));
    assert_eq!(b.labels()[0].x(), -5.0);
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
    assert_eq!(a.labels()[2].x(), 960.0);
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
    let texts: Vec<String> = a.labels().iter().map(|l| l.text().to_owned()).collect();
    assert_eq!(texts, ["25 %", "75 %"]);
}

#[test]
fn chosen_values_keep_the_precision_of_the_tick_count() {
    let y = scale::linear();
    let a = axis::left(&y).values([0.25, 0.5]);
    let texts: Vec<String> = a.labels().iter().map(|l| l.text().to_owned()).collect();
    assert_eq!(texts, ["0.3", "0.5"]);
}

#[test]
fn labels_follow_the_axis_locale() {
    let x = scale::linear().domain([0.0, 12000.0]);
    let neutral: Vec<String> = axis::bottom(&x)
        .ticks(5)
        .labels()
        .iter()
        .map(|l| l.text().to_owned())
        .collect();
    assert_eq!(neutral[6], "12\u{202f}000");
    let en = axis::bottom(&x).ticks(5).locale(Locale::EN_US).labels();
    assert_eq!(en[6].text(), "12,000");
}

#[test]
fn ticks_off_the_scale_are_left_out_of_marks_and_labels() {
    let x = scale::log().domain([1.0, 100.0]).range([0.0, 200.0]);
    let a = axis::bottom(&x).values([-1.0, 10.0, f64::NAN]);
    assert_eq!(marks(&drawn(&a)), [((100.5, 0.0), (100.5, 6.0))]);
    let labels = a.labels();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].text(), "10");
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
    let texts: Vec<String> = a.labels().iter().map(|l| l.text().to_owned()).collect();
    assert_eq!(texts, ["0", "100", "200", "300"]);
}

#[test]
fn an_axis_prints_its_settings() {
    let x = scale::linear();
    let text = format!("{:?}", axis::left(&x).format(|v| v.to_string()));
    assert!(text.contains("orient: Left") && text.contains("format: Some(\"custom\")"));
}
