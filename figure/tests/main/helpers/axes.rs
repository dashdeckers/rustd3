//! Axis defaults and minor ticks: core's `Ticks::minor` values drawn at the
//! theme's minor tick size and width without labels, on linear, log and
//! symlog axes; none on time and band axes, none without tick marks.

use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::{self, Ticks};
use rustd3_core::time;
use rustd3_figure::helpers::axes;
use rustd3_figure::kurbo::{BezPath, PathEl, Point, Size};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::theme::{Context, Style};
use rustd3_figure::{Figure, Held, Target, Theme, axis};

use crate::figure::{cache, resolve};

/// `fig` drawn at 640 × 480 for vector output.
fn draw(fig: &Figure<'_>) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    )
}

/// A stroke as its width and its segments.
type Stroke = (f64, Vec<[Point; 2]>);

/// The strokes of axis `axis` of panel 0 in `list` (its tick marks: major
/// ticks first, then minor ones when it has any), and the texts of its
/// glyph runs.
fn axis_part(list: &List, axis: u64) -> (Vec<Stroke>, Vec<String>) {
    let id = part::Id::new(4 << 56 | axis);
    let p = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .expect("an axis part");
    let mut strokes = Vec::new();
    let mut texts = Vec::new();
    for c in &list.commands[p.commands.clone()] {
        if let Command::Stroke { path, style, .. } = c {
            strokes.push((style.width, segments(path)));
        } else if let Command::Glyphs(run) = c {
            texts.push(run.text.to_string());
        }
    }
    (strokes, texts)
}

/// The segments of a path of moves and lines.
fn segments(path: &BezPath) -> Vec<[Point; 2]> {
    let mut out = Vec::new();
    let mut from = Point::ZERO;
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => from = *p,
            PathEl::LineTo(p) => out.push([from, *p]),
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => {
                panic!("not a segment: {el:?}")
            }
        }
    }
    out
}

/// Whether `a` is within 1e-9 of `b`.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The minor tick size and width at notebook: 4 pt and 1 pt, px.
const MINOR: [f64; 2] = [16.0 / 3.0, 4.0 / 3.0];

#[test]
fn minor_ticks_fill_fifths_of_a_linear_step_at_the_minor_size_without_labels() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0).axis(axis::bottom(x).ticks(5).minor(true));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (strokes, texts) = axis_part(&draw(&fig), 0);
    assert_eq!(texts, ["0", "2", "4", "6", "8", "10"]);
    assert_eq!(strokes.len(), 2, "major and minor ticks");
    let (width, minor) = &strokes[1];
    assert!(near(*width, MINOR[1]), "{width}");
    // 0.4, 0.8, 1.2, 1.6, 2.4, …, 9.6: 20 minors.
    let expected: Vec<f64> = (0..25)
        .filter(|k| k % 5 != 0)
        .map(|k| f64::from(k) * 0.4)
        .collect();
    assert_eq!(minor.len(), expected.len());
    for ([a, b], v) in minor.iter().zip(expected) {
        let x = area.x0 + v / 10.0 * area.width();
        assert!(near(a.x, x) && near(b.x, x), "{v}: {a:?}");
        assert!(
            near(a.y, area.y1) && near(b.y - a.y, MINOR[0]),
            "{a:?} {b:?}"
        );
    }
    // Off by default.
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .axis(axis::bottom(x).ticks(5).minor(true).minor(false));
    assert_eq!(axis_part(&draw(&fig), 0).0.len(), 1);
}

#[test]
fn symlog_axes_draw_the_decade_minors_of_core_ticks() {
    let symlog = scale::symlog().constant(1.0).domain([0.0, 5000.0]);
    let set = Ticks::tick_set(&symlog, 10);
    assert_eq!(set.values(), [0.0, 1.0, 10.0, 100.0, 1000.0]);
    let decades: Vec<f64> = [1.0, 10.0, 100.0, 1000.0]
        .into_iter()
        .flat_map(|d| (2..=9).map(move |m| f64::from(m) * d))
        .filter(|v| *v <= 5000.0)
        .collect();
    assert_eq!(Ticks::minor(&symlog, &set), decades);
    let mut fig = Figure::new();
    let y = fig.scale(symlog.clone());
    fig.panel(0, 0)
        .axis(axes::defaults(axis::left(y).ticks(10)));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (strokes, _) = axis_part(&draw(&fig), 0);
    let minor = &strokes[1].1;
    assert_eq!(minor.len(), 28);
    let ranged = symlog.range([area.height(), 0.0]);
    for ([a, b], v) in minor.iter().zip(decades) {
        let y = area.y0 + ranged.apply(v);
        assert!(near(a.y, y) && near(b.y, y), "{v}: {a:?}");
        assert!(near(a.x - b.x, MINOR[0]), "{a:?} {b:?}");
    }
}

#[test]
fn log_axes_draw_decade_minors_beside_the_ticks_d3_blanks() {
    let log = scale::log().domain([1.0, 1e6]);
    let set = Ticks::tick_set(&log, 5);
    let minors = Ticks::minor(&log, &set);
    let mut fig = Figure::new();
    let y = fig.scale(log);
    fig.panel(0, 0).axis(axis::left(y).ticks(5).minor(true));
    let (strokes, texts) = axis_part(&draw(&fig), 0);
    let blanked = texts.len() < set.values().len();
    let minor = &strokes[1].1;
    let majors = strokes[0].1.len();
    assert_eq!(
        majors + minor.len(),
        set.values().len() + minors.len(),
        "every tick d3 gives and every core minor, majors blanked or not ({blanked})"
    );
    assert!(!minors.is_empty());
}

#[test]
fn time_and_band_axes_have_no_minor_ticks() {
    let t0 = Timestamp::from_second(1_700_000_000).expect("a valid instant");
    let mut fig = Figure::new();
    let x = fig.scale(time::utc().domain([t0, t0 + SignedDuration::from_hours(30)]));
    let keys = fig.scale(scale::band(["a", "b", "c"]));
    fig.panel(0, 0)
        .axis(axes::defaults(axis::bottom(x)))
        .axis(axes::defaults(axis::left(keys)));
    let list = draw(&fig);
    assert_eq!(axis_part(&list, 0).0.len(), 1);
    assert_eq!(axis_part(&list, 1).0.len(), 1);
}

#[test]
fn minor_ticks_are_tick_marks_so_styles_without_them_draw_none() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.theme(Theme::seaborn(Style::WhiteGrid, Context::Notebook));
    fig.panel(0, 0).axis(axis::bottom(x).ticks(5).minor(true));
    let (strokes, texts) = axis_part(&draw(&fig), 0);
    assert!(strokes.is_empty(), "{strokes:?}");
    assert_eq!(texts.len(), 6);
}

#[test]
fn defaults_keep_offset_notation_on_beside_the_minor_ticks() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([1e9, 1e9 + 1.0]));
    fig.panel(0, 0)
        .axis(axes::defaults(axis::bottom(x).ticks(5)));
    let (strokes, texts) = axis_part(&draw(&fig), 0);
    assert!(texts.iter().any(|t| t == "+1e9"), "{texts:?}");
    assert_eq!(strokes.len(), 2);
    assert!(near(strokes[1].0, MINOR[1]));
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([1e9, 1e9 + 1.0]));
    fig.panel(0, 0)
        .axis(axes::defaults(axis::bottom(x).ticks(5).offset(false)));
    let (_, texts) = axis_part(&draw(&fig), 0);
    assert!(!texts.iter().any(|t| t == "+1e9"), "{texts:?}");
}
