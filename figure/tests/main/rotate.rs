//! Rotated tick labels: their turned extents, the spacing rule, end
//! anchoring, and no overlap after the collision passes. Label sizes are
//! measured through the text cache, the boxes read from
//! `Resolved::label_boxes` (bounding rectangles of the turned ink boxes)
//! and turned back here to test them as oriented boxes.

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::kurbo::{Point, Rect, Size, Vec2};
use rustd3_figure::layout::{Grid, px};
use rustd3_figure::list::{Command, GlyphRun, List, part};
use rustd3_figure::text::{self, Key, Style, Text, anchor};
use rustd3_figure::{Figure, Held, Resolved, Target, axis};

use crate::figure::cache;

/// The tick-label size at notebook: 11 pt.
const SIZE: f64 = 44.0 / 3.0;
/// The tick length and the tick pad: 6 pt and 3.5 pt.
const GAP: f64 = (6.0 + 3.5) * 4.0 / 3.0;

/// `label`'s ink box at tick-label size: (advance width, ink height,
/// ascent).
fn ink(label: &str) -> (f64, f64, f64) {
    let style = Style {
        size: SIZE,
        bold: false,
        fill: Rgb::hex(0x262626),
        align: anchor::Horizontal::Middle,
    };
    let key = Key::new(&Text::from(label), &style);
    let mut cache = text::Cache::new(crate::text::fonts());
    cache.prepare([&key]);
    let shaped = cache.get(&key).expect("prepared").expect("shaped");
    (
        shaped.get_width(),
        shaped.get_ascent() + shaped.get_descent(),
        shaped.get_ascent(),
    )
}

/// `fig` resolved and drawn for vector output at `size`.
fn laid(fig: &Figure<'_>, size: Size) -> (Resolved, List) {
    let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    (r, list)
}

/// The glyph runs axis `axis` of panel `panel` draws in `list`: its kept
/// tick labels, then its title.
fn runs(list: &List, panel: u64, axis: u64) -> Vec<GlyphRun> {
    let id = part::Id::new(4 << 56 | panel << 32 | axis);
    let Some(part) = list.parts.iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    list.commands[part.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.clone())
            } else {
                None
            }
        })
        .collect()
}

/// The ink box of `label` turned counter-clockwise by `deg`, placed in
/// its bounding rectangle `aabb`: its four corners.
fn turned(label: &str, deg: f64, aabb: Rect) -> [Point; 4] {
    let (w, h, a) = ink(label);
    let (sin, cos) = deg.to_radians().sin_cos();
    let (u, n) = (Vec2::new(cos, -sin), Vec2::new(sin, cos));
    let corners = [(0.0, -a), (w, -a), (w, h - a), (0.0, h - a)].map(|(x, y)| u * x + n * y);
    let min = corners
        .iter()
        .fold(Vec2::new(f64::INFINITY, f64::INFINITY), |m, c| {
            Vec2::new(m.x.min(c.x), m.y.min(c.y))
        });
    corners.map(|c| aabb.origin() + (c - min))
}

/// Whether two convex quadrilaterals share interior, by the separating
/// axis test on their edge normals (touching is not overlap).
fn overlap(a: &[Point; 4], b: &[Point; 4]) -> bool {
    let normals = |q: &[Point; 4]| [q[1] - q[0], q[3] - q[0]].map(|e| Vec2::new(-e.y, e.x));
    let project = |q: &[Point; 4], axis: Vec2| {
        q.iter()
            .map(|p| p.to_vec2().dot(axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            })
    };
    normals(a).into_iter().chain(normals(b)).all(|axis| {
        let ((a0, a1), (b0, b1)) = (project(a, axis), project(b, axis));
        // A margin of 1e-9 px absorbs the rounding of the turned corners.
        a0 < b1 - 1e-9 && b0 < a1 - 1e-9
    })
}

/// One panel with a bottom axis on x over [0, 100] at `deg`, ticks every
/// 20, and a left axis on y over [0, 1], ticks every 0.5, at `deg`.
fn rotated(deg: f64) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .axis(axis::bottom(x).ticks(5).rotate(deg))
        .axis(axis::left(y).ticks(2).rotate(deg));
    fig
}

#[test]
fn rotated_labels_reach_by_their_turned_ink_boxes() {
    let xs = ["0", "20", "40", "60", "80", "100"];
    let ys = ["0.0", "0.5", "1.0"];
    for deg in [30.0, 45.0, 90.0, -30.0] {
        let (sin, cos) = f64::to_radians(deg).sin_cos();
        let reach = |labels: &[&str], across: bool| {
            labels
                .iter()
                .map(|l| {
                    let (w, h, _) = ink(l);
                    if across {
                        w * sin.abs() + h * cos.abs()
                    } else {
                        w * cos.abs() + h * sin.abs()
                    }
                })
                .fold(0.0, f64::max)
        };
        let (r, _) = laid(&rotated(deg), Size::new(640.0, 480.0));
        let bottom = r.get_plan(0, 0).expect("the bottom axis").thickness;
        let left = r.get_plan(0, 1).expect("the left axis").thickness;
        let (want_bottom, want_left) = (GAP + reach(&xs, true), GAP + reach(&ys, false));
        assert!(
            (bottom - want_bottom).abs() < 1e-9,
            "{deg}°: {bottom} {want_bottom}"
        );
        assert!(
            (left - want_left).abs() < 1e-9,
            "{deg}°: {left} {want_left}"
        );
    }
}

#[test]
fn positive_angles_end_bottom_labels_at_their_ticks_rising_left_to_right() {
    let (r, list) = laid(&rotated(30.0), Size::new(640.0, 480.0));
    let area = r.get_plot_areas()[0];
    let boxes = r.label_boxes();
    let bottom = &boxes[..6];
    for (i, b) in bottom.iter().enumerate() {
        let tick = area.x0 + area.width() * i as f64 / 5.0;
        assert!((b.x1 - tick).abs() < 1e-9, "label {i}: {b:?}, tick {tick}");
        assert!((b.y0 - (area.y1 + GAP)).abs() < 1e-9, "label {i}: {b:?}");
    }
    for run in runs(&list, 0, 0) {
        let [a, b, ..] = run.transform.as_coeffs();
        let (sin, cos) = 30f64.to_radians().sin_cos();
        assert!((a - cos).abs() < 1e-6 && (b + sin).abs() < 1e-6, "{a} {b}");
    }
    // Left labels end a gap short of the axis, centred on their ticks.
    for (i, b) in boxes[6..].iter().enumerate() {
        let tick = area.y1 - area.height() * i as f64 / 2.0;
        assert!((b.x1 - (area.x0 - GAP)).abs() < 1e-9, "{b:?}");
        assert!((b.center().y - tick).abs() < 1e-9, "{b:?}, tick {tick}");
    }
}

#[test]
fn negative_angles_start_bottom_labels_and_top_axes_mirror() {
    let (r, _) = laid(&rotated(-30.0), Size::new(640.0, 480.0));
    let area = r.get_plot_areas()[0];
    for (i, b) in r.label_boxes()[..6].iter().enumerate() {
        let tick = area.x0 + area.width() * i as f64 / 5.0;
        assert!((b.x0 - tick).abs() < 1e-9, "label {i}: {b:?}, tick {tick}");
    }
    for (deg, start) in [(30.0, true), (-30.0, false)] {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 100.0]));
        fig.panel(0, 0).axis(axis::top(x).ticks(5).rotate(deg));
        let (r, _) = laid(&fig, Size::new(640.0, 480.0));
        let area = r.get_plot_areas()[0];
        for (i, b) in r.label_boxes().iter().enumerate() {
            let tick = area.x0 + area.width() * i as f64 / 5.0;
            let edge = if start { b.x0 } else { b.x1 };
            assert!((edge - tick).abs() < 1e-9, "{deg}°, label {i}: {b:?}");
            assert!((b.y1 - (area.y0 - GAP)).abs() < 1e-9, "{deg}°: {b:?}");
        }
    }
}

/// The size, 300 px tall, at which panel 0 of `fig` is `width` px wide.
fn at_width(fig: &Figure<'_>, width: f64) -> Size {
    let mut size = Size::new(width + 60.0, 300.0);
    for _ in 0..20 {
        let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        let w = r.get_plot_areas()[0].width();
        if (w - width).abs() < 1e-9 {
            return size;
        }
        size.width += width - w;
    }
    panic!("no size gives a plot area {width} px wide");
}

/// Twenty speed bins "0–10" … "190–200".
fn bins() -> Vec<String> {
    (0..20)
        .map(|i| format!("{}–{}", 10 * i, 10 * i + 10))
        .collect()
}

#[test]
fn turned_boxes_keep_every_kth_label_with_a_smaller_stride() {
    // Flat, the row at 320 px keeps every fourth bin; turned, the
    // ink boxes (about 11 px tall) need d·|sin θ| ≥ h at a band step of
    // about 16 px: every second at 30°, every one at 45° and 90°.
    let mut kept = Vec::new();
    for deg in [0.0, 30.0, 45.0, 90.0] {
        let mut fig = Figure::new();
        let speed = fig.scale(scale::band(bins()).padding(0.1));
        fig.panel(0, 0).axis(axis::bottom(speed).rotate(deg));
        let (r, list) = laid(&fig, at_width(&fig, 320.0));
        let texts: Vec<String> = runs(&list, 0, 0)
            .iter()
            .map(|r| r.text.to_string())
            .collect();
        let boxes = r.label_boxes();
        assert_eq!(texts.len(), boxes.len());
        let quads: Vec<[Point; 4]> = texts
            .iter()
            .zip(&boxes)
            .map(|(t, b)| turned(t, deg, *b))
            .collect();
        for (i, a) in quads.iter().enumerate() {
            for b in &quads[i + 1..] {
                assert!(!overlap(a, b), "{deg}°: {a:?} {b:?}");
            }
        }
        kept.push(texts.len());
    }
    assert_eq!(kept, [5, 10, 20, 20]);
}

#[test]
fn no_turned_labels_overlap_across_axes_after_the_collision_passes() {
    // Two stacked panels whose rotated labels meet in the gap between
    // them: the larger panel's labels win.
    let mut fig = Figure::new();
    fig.layout(Grid::new().rows([1.0, 0.4]).row_gaps([px(-30.0)]));
    let x = fig.scale(scale::linear().domain([0.0, 100.0]));
    let y = fig.scale(scale::linear().domain([0.0, 100.0]));
    fig.panel(0, 0)
        .axis(axis::bottom(x).ticks(10).rotate(40.0))
        .axis(axis::left(y).rotate(40.0));
    fig.panel(1, 0)
        .axis(axis::top(x).ticks(10).rotate(40.0).labels(true))
        .axis(axis::left(y).rotate(40.0));
    let (r, list) = laid(&fig, Size::new(480.0, 400.0));
    let mut quads = Vec::new();
    let boxes = r.label_boxes();
    let mut at = 0;
    for (panel, axis) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        for run in runs(&list, panel, axis) {
            let text = run.text.to_string();
            if text.chars().all(|c| c.is_ascii_digit() || c == '.') {
                quads.push(turned(&text, 40.0, boxes[at]));
                at += 1;
            }
        }
    }
    assert_eq!(at, boxes.len());
    // The larger panel keeps all eleven; the smaller one loses some.
    assert_eq!(runs(&list, 0, 0).len(), 11);
    assert!(runs(&list, 1, 0).len() < 11);
    for (i, a) in quads.iter().enumerate() {
        for b in &quads[i + 1..] {
            assert!(!overlap(a, b), "{a:?} {b:?}");
        }
    }
}
