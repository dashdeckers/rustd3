//! Value labels on bars and pies, and size and color legends of scatter
//! channels.

use rustd3_core::color::Rgb;
use rustd3_core::format;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Band, Linear};
use rustd3_core::scheme;
use rustd3_figure::helpers::{labels, legend};
use rustd3_figure::kurbo::{Point, Rect, Shape};
use rustd3_figure::legend::Slot;
use rustd3_figure::list::{Command, GlyphRun, part};
use rustd3_figure::mark::{bar_y, dots};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held};

use crate::figure::resolve;
use crate::marks::{draw, marked};

/// DejaVu Sans's OS/2 typographic ascender and descender (1556 and 492 of
/// 2048 units) at the labels' 12 pt, 16 px: their line box.
const LINE: [f64; 2] = [1556.0 / 2048.0 * 16.0, 492.0 / 2048.0 * 16.0];

/// A figure with band keys "a", "b" and "c" (no padding) and y fixed to
/// [−2, 4].
fn figure() -> (
    Figure<'static>,
    ScaleRef<Band<&'static str>>,
    ScaleRef<Linear>,
) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b", "c"]));
    let y = fig.scale(scale::linear().domain([-2.0, 4.0]));
    (fig, x, y)
}

/// The bar rows of the section's spot row.
const ROWS: [(&str, f64); 3] = [("a", 3.0), ("b", -1.0), ("c", 2.5)];

/// The glyph runs among `commands`.
fn runs<'c>(commands: impl IntoIterator<Item = &'c Command>) -> Vec<&'c GlyphRun> {
    commands
        .into_iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect()
}

/// A run's text, its baseline's left end and its advance width, in the
/// coordinates of its group.
fn placed(run: &GlyphRun) -> (String, Point, f64) {
    let width: f32 = run.glyphs.iter().map(|g| g.advance).sum();
    let t = run.transform.translation();
    (run.text.to_string(), Point::new(t.x, t.y), f64::from(width))
}

/// `fig`'s plot area at 640 × 480.
fn area(fig: &Figure<'_>) -> Rect {
    resolve(fig, &Held::default()).plot_areas()[0]
}

/// Whether `a` is within `tolerance` of `b`.
fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance
}

#[test]
fn bar_labels_stand_three_points_beyond_each_bars_end_by_their_line_box() {
    let (mut fig, x, y) = figure();
    let spec = ".1f".parse().expect("a specifier");
    fig.panel(0, 0)
        .mark(bar_y(ROWS, x, y))
        .mark(labels::bars(&ROWS, x, y).format(spec));
    let (w, h) = (area(&fig).width(), area(&fig).height());
    let list = draw(&fig);
    let at = |v: f64| h - (v + 2.0) / 6.0 * h;
    let [ascender, descender] = LINE;
    let expected = [
        ("3.0", w / 6.0, at(3.0) - 4.0 - descender),
        ("\u{2212}1.0", w / 2.0, at(-1.0) + 4.0 + ascender),
        ("2.5", 5.0 * w / 6.0, at(2.5) - 4.0 - descender),
    ];
    let runs = runs(marked(&list));
    assert_eq!(runs.len(), 3);
    for (run, (text, centre, baseline)) in runs.into_iter().zip(expected) {
        let (got, origin, width) = placed(run);
        assert_eq!(got, text);
        assert!((run.size - 16.0).abs() < 1e-4, "12 pt: {}", run.size);
        assert!(
            near(origin.y, baseline, 1e-9),
            "{text}: {origin:?}, {baseline}"
        );
        assert!(
            near(origin.x + width / 2.0, centre, 1e-3),
            "{text}: {origin:?}"
        );
    }
}

#[test]
fn centred_bar_labels_sit_halfway_up_their_bars_and_follow_the_padding() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0)
        .mark(labels::bars(&ROWS, x, y).center())
        .mark(
            labels::bars(&ROWS[..1], x, y)
                .padding(6.0)
                .padding(f64::NAN),
        );
    let h = area(&fig).height();
    let list = draw(&fig);
    let at = |v: f64| h - (v + 2.0) / 6.0 * h;
    let [ascender, descender] = LINE;
    let runs = runs(marked(&list));
    let baselines: Vec<f64> = runs.iter().map(|r| placed(r).1.y).collect();
    for (i, v) in [3.0, -1.0, 2.5].into_iter().enumerate() {
        let middle = (at(0.0) + at(v)) / 2.0;
        let expected = middle + (ascender - descender) / 2.0;
        assert!(near(baselines[i], expected, 1e-9), "{v}: {}", baselines[i]);
    }
    // 6 pt is 8 px; a padding that is not finite leaves it as it was.
    assert!(near(baselines[3], at(3.0) - 8.0 - descender, 1e-9));
}

#[test]
fn bar_labels_write_d3s_default_format_in_the_figures_locale() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(labels::bars(&ROWS, x, y));
    let list = draw(&fig);
    let texts: Vec<String> = runs(marked(&list)).iter().map(|r| placed(r).0).collect();
    assert_eq!(texts, ["3", "\u{2212}1", "2.5"]);
    let german = Locale::from(format::Locale::DE_DE);
    let spec = ".2f".parse().expect("a specifier");
    fig.locale(german);
    fig.panel(0, 0)
        .mark(labels::bars(&ROWS[2..], x, y).format(spec));
    let list = draw(&fig);
    let texts: Vec<String> = runs(marked(&list)).iter().map(|r| placed(r).0).collect();
    assert_eq!(texts[3], "2,50");
}

#[test]
fn bar_labels_of_keys_outside_the_band_draw_nothing_and_are_reported() {
    let (mut fig, x, y) = figure();
    fig.panel(0, 0).mark(labels::bars(
        &[("a", 1.0), ("z", 2.0), ("b", f64::NAN)],
        x,
        y,
    ));
    let list = draw(&fig);
    assert_eq!(runs(marked(&list)).len(), 1);
    #[cfg(feature = "fonts")]
    {
        let issues = fig.check(rustd3_figure::kurbo::Size::new(640.0, 480.0));
        let outside = rustd3_figure::check::Issue::OutsideDomain {
            handle: x.index(),
            count: 1,
        };
        assert!(issues.contains(&outside), "{issues:#?}");
    }
}

/// The centres of the line boxes of the labels a figure of one panel with
/// `mark` draws, relative to its plot area's middle, and their texts.
fn centres(mark: labels::Pie<&'static str>) -> Vec<(String, Point)> {
    let mut fig = Figure::new();
    fig.panel(0, 0).mark(mark);
    let middle = area(&fig).size().to_vec2() / 2.0;
    let [ascender, descender] = LINE;
    let list = draw(&fig);
    runs(marked(&list))
        .into_iter()
        .map(|run| {
            let (text, origin, width) = placed(run);
            let y = origin.y - (ascender - descender) / 2.0;
            (text, Point::new(origin.x + width / 2.0, y) - middle)
        })
        .collect()
}

/// `shape::pie([3, 1, 2])` as keyed slices.
const SLICES: [(&str, f64); 3] = [("x", 3.0), ("y", 1.0), ("z", 2.0)];

/// Asserts that `got`'s points are `expected`'s (d3's centroids to six
/// decimals), within 1e-3 px across (f32 advances) and 1e-6 px up.
fn assert_centres(got: &[(String, Point)], expected: [(f64, f64); 3]) {
    assert_eq!(got.len(), 3);
    for ((text, p), (x, y)) in got.iter().zip(expected) {
        assert!(
            near(p.x, x, 1e-3) && near(p.y, y, 1e-6),
            "{text}: {p:?} against ({x}, {y})"
        );
    }
}

#[test]
fn pie_labels_centre_on_the_slices_arc_centroids() {
    let pie = centres(labels::pie(&SLICES, 0.0, 100.0));
    assert_eq!(
        pie.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
        ["x", "y", "z"]
    );
    assert_centres(&pie, [(50.0, 0.0), (-25.0, -43.301270), (-43.301270, 25.0)]);
    let donut = centres(labels::pie(&SLICES, 40.0, 80.0));
    assert_centres(
        &donut,
        [(60.0, 0.0), (-30.0, -51.961524), (-51.961524, 30.0)],
    );
}

#[test]
fn pie_labels_write_shares_and_move_to_another_ring_inside_the_pie() {
    let shares = centres(
        labels::pie(&SLICES, 40.0, 80.0)
            .percent(true)
            .radius(70.0)
            .radius(81.0),
    );
    let texts: Vec<&str> = shares.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(texts, ["50%", "17%", "33%"]);
    let k = 70.0 / 60.0;
    assert_centres(
        &shares,
        [
            (70.0, 0.0),
            (-30.0 * k, -51.961524 * k),
            (-51.961524 * k, 30.0 * k),
        ],
    );
    let empty = centres(labels::pie(
        &[("x", 3.0), ("none", 0.0), ("nan", f64::NAN)],
        0.0,
        100.0,
    ));
    assert_eq!(empty.len(), 1);
    assert!(
        near(empty[0].1.x, 0.0, 1e-3) && near(empty[0].1.y, 50.0, 1e-9),
        "{empty:?}"
    );
}

/// A legend's dots, each its diameter and paint, and its labels, each its
/// text and baseline.
type Drawn = (Vec<(f64, Brush)>, Vec<(String, f64)>);

/// The legend in figure slot 0 of `fig` drawn at 640 × 480.
fn legend_of(fig: &Figure<'_>) -> Drawn {
    let list = draw(fig);
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32);
    let part = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .expect("a slotted legend");
    let commands = &list.commands[part.commands.clone()];
    let dots = commands
        .iter()
        .filter_map(|c| {
            if let Command::Fill { path, paint } = c {
                Some((path.bounding_box().width(), paint.clone()))
            } else {
                None
            }
        })
        .collect();
    let labels = runs(commands)
        .into_iter()
        .map(|run| (run.text.to_string(), run.transform.translation().y))
        .collect();
    (dots, labels)
}

/// An opaque solid brush of `c`, its channels rounded to 8 bits.
fn solid(c: Rgb) -> Brush {
    let [r, g, b] = [c.r, c.g, c.b].map(|v| v.round() as u8);
    Brush::Solid(Color::from_rgba8(r, g, b, 255))
}

#[test]
fn a_size_legend_shows_d3s_ticks_inside_the_domain_as_dots_of_their_radii() {
    let mut fig = Figure::new();
    let radius = fig.scale(scale::linear().domain([1.0, 100.0]).range([1.0, 21.0]));
    fig.legend(Slot::Right, legend::sizes(radius));
    let (dots, labels) = legend_of(&fig);
    let texts: Vec<&str> = labels.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(texts, ["20", "40", "60", "80", "100"]);
    let ink = solid(Rgb::hex(0x262626));
    for ((d, paint), v) in dots.iter().zip([20.0, 40.0, 60.0, 80.0, 100.0]) {
        let r = 1.0 + (v - 1.0) / 99.0 * 20.0;
        assert!(near(*d, 2.0 * r, 1e-3), "{v}: {d}");
        assert_eq!(*paint, ink);
    }
    // Rows grow to hold their dots: the 42 px dot's row is taller than the
    // 0.5 em gap and the label's line box.
    let gaps: Vec<f64> = labels.windows(2).map(|w| w[1].1 - w[0].1).collect();
    assert!(gaps.windows(2).all(|g| g[1] > g[0]), "{gaps:?}");
    // Chosen values outside the domain are left out, whatever the count,
    // and so is a radius of 0.
    let mut fig = Figure::new();
    let radius = fig.scale(scale::linear().domain([0.0, 100.0]).range([0.0, 20.0]));
    let chosen = legend::sizes(radius)
        .values([0.0, 10.0, 50.0, 150.0])
        .count(2);
    fig.legend(Slot::Right, chosen);
    let (dots, labels) = legend_of(&fig);
    let texts: Vec<&str> = labels.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!((dots.len(), texts), (2, vec!["10", "50"]));
}

#[test]
fn a_color_legend_shows_d3s_ticks_as_dots_of_their_colors() {
    let mut fig = Figure::new();
    let heat = fig.scale(scale::sequential([1.0, 100.0], scheme::viridis));
    fig.legend(Slot::Right, legend::colors(heat));
    let (dots, labels) = legend_of(&fig);
    let texts: Vec<&str> = labels.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(texts, ["20", "40", "60", "80", "100"]);
    for ((d, paint), v) in dots.iter().zip([20.0, 40.0, 60.0, 80.0, 100.0]) {
        assert!(near(*d, 8.0, 1e-3), "the theme's 6 pt marker: {d}");
        assert_eq!(*paint, solid(scheme::viridis((v - 1.0) / 99.0)), "{v}");
    }
    let mut fig = Figure::new();
    let heat = fig.scale(scale::sequential([1.0, 100.0], scheme::viridis));
    fig.legend(Slot::Right, legend::colors(heat).count(2));
    assert_eq!(legend_of(&fig).1.len(), 2);
}

#[test]
fn scatter_legends_sit_in_slots_and_insets_beside_a_scatter_that_fits() {
    let mut fig = Figure::new();
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let heat = fig.scale(scale::sequential([1.0, 100.0], scheme::viridis));
    let radius = fig.scale(scale::sqrt().domain([1.0, 100.0]).range([2.0, 12.0]));
    fig.panel(0, 0)
        .mark(
            dots([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y)
                .fill_by(heat, [20.0, 45.0, 80.0])
                .size_by(radius, [10.0, 50.0, 100.0]),
        )
        .legend(
            rustd3_figure::legend::Inset::TopLeft,
            legend::sizes(radius).count(3),
        );
    fig.legend(Slot::Right, legend::colors(heat));
    let list = draw(&fig);
    let legends = list
        .parts
        .iter()
        .filter(|p| p.id >= part::Id::new(7 << 56) && p.id < part::Id::new(8 << 56));
    assert_eq!(legends.count(), 2);
    #[cfg(feature = "fonts")]
    {
        let issues = fig.check(rustd3_figure::kurbo::Size::new(640.0, 480.0));
        assert!(issues.is_empty(), "{issues:#?}");
    }
}
