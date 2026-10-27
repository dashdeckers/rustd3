//! The figure model: handles, the scale table, held scales, decorations
//! and their parts, and the test mark the fitting and layout tests share.

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::format;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{BezPath, PathEl, Rect, Size};
use rustd3_figure::list::{self, Command, GlyphRun, List, part};
use rustd3_figure::mark::{Ctx, Mark};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::scale::{Kind, ScaleRef};
use rustd3_figure::{Cache, Figure, Fits, Held, Resolved, Target, Theme, axis};

/// A mark that only reports values on one handle; with a base, it also
/// sticks there, as bars do.
pub struct Values<S: Kind> {
    pub on: ScaleRef<S>,
    pub values: Vec<S::Domain>,
    pub base: Option<S::Domain>,
}

impl<S: Kind> Mark<'static> for Values<S> {
    fn fit(&self, fits: &mut Fits) {
        for v in &self.values {
            fits.add(self.on, v);
        }
        if let Some(base) = &self.base {
            fits.add(self.on, base);
            fits.stick(self.on, base.clone());
        }
    }

    fn draw(&self, _ctx: &Ctx<'_>, _list: &mut List) {}
}

/// A line's values on `on`: not sticky.
pub fn line<S: Kind>(on: ScaleRef<S>, values: impl IntoIterator<Item = S::Domain>) -> Values<S> {
    Values {
        on,
        values: values.into_iter().collect(),
        base: None,
    }
}

/// A cache over the bundled fonts (or the same files without `fonts`).
pub fn cache() -> Cache {
    Cache::new(crate::text::fonts())
}

/// `fig` resolved at 640 × 480 for vector output with `held`.
pub fn resolve(fig: &Figure<'_>, held: &Held) -> Resolved {
    fig.resolve(Size::new(640.0, 480.0), &mut cache(), held, Target::Vector)
}

#[test]
fn handles_index_the_table_in_order_and_compare_by_figure() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 2.0]));
    let y = fig.fit(scale::log());
    assert_eq!((x.index(), y.index()), (0, 1));
    let copy = x;
    assert_eq!(copy, x);
    let mut other = Figure::new();
    let z = other.scale(scale::linear().domain([0.0, 2.0]));
    assert_eq!(z.index(), x.index());
    assert_ne!(format!("{z:?}"), "");
}

#[test]
fn unfitted_scales_keep_the_domain_they_were_built_with() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([3.0, 7.0]));
    fig.panel(0, 0).mark(line(x, [0.0, 100.0]));
    assert_eq!(
        resolve(&fig, &Held::default()).scale(x).get_domain(),
        [3.0, 7.0]
    );
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "handle from another figure")]
fn a_handle_from_another_figure_panics_when_a_mark_reports_on_it() {
    let mut first = Figure::new();
    let foreign = first.fit(scale::linear());
    let mut fig = Figure::new();
    fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(foreign, [1.0]));
    resolve(&fig, &Held::default());
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "handle from another figure")]
fn a_handle_from_another_figure_panics_when_its_scale_is_read() {
    let mut first = Figure::new();
    let foreign = first.fit(scale::linear());
    let mut fig = Figure::new();
    fig.fit(scale::linear());
    resolve(&fig, &Held::default()).scale(foreign);
}

#[test]
fn held_scales_are_typed_by_their_handle() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let mut held = Held::default();
    held.hold(x, scale::linear().domain([2.0, 4.0]));
    assert_eq!(held.get(x).map(|s| s.get_domain()), Some([2.0, 4.0]));
    let mut other = Figure::new();
    let log = other.fit(scale::log());
    assert!(held.get(log).is_none(), "same index, another type");
    held.release(log);
    assert!(held.get(x).is_some(), "a release of another type keeps it");
    held.release(x);
    assert!(held.get(x).is_none());
}

#[test]
fn prune_drops_what_the_figure_no_longer_has() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let mut held = Held::default();
    held.hold(x, scale::linear().domain([2.0, 4.0]));
    held.hold(y, scale::linear().domain([5.0, 6.0]));
    let mut rebuilt = Figure::new();
    let x2 = rebuilt.fit(scale::linear());
    held.prune(&rebuilt);
    assert_eq!(held.get(x2).map(|s| s.get_domain()), Some([2.0, 4.0]));
    assert!(held.get(y).is_none(), "index 1 is gone");
    let mut retyped = Figure::new();
    retyped.fit(scale::log());
    held.prune(&retyped);
    assert!(held.get(x2).is_none(), "index 0 is a log scale now");
}

#[test]
fn a_held_scale_of_another_type_is_ignored() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(x, [0.0, 10.0]));
    let mut other = Figure::new();
    let log = other.fit(scale::log());
    let mut held = Held::default();
    held.hold(log, scale::log().domain([1.0, 100.0]));
    assert_eq!(resolve(&fig, &held).scale(x).get_domain(), [-0.5, 10.5]);
}

#[test]
fn an_erased_hold_holds_like_a_typed_one() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line(x, [0.0, 10.0]));
    let mut held = Held::default();
    held.hold_erased(x.index(), Box::new(scale::linear().domain([2.0, 4.0])));
    assert_eq!(held.get(x).map(|s| s.get_domain()), Some([2.0, 4.0]));
    assert_eq!(resolve(&fig, &held).scale(x).get_domain(), [2.0, 4.0]);
    // Keyed by the boxed scale's own type: a log scale at x's position
    // is ignored.
    held.hold_erased(x.index(), Box::new(scale::log().domain([1.0, 100.0])));
    assert!(held.get(x).is_none());
    assert_eq!(resolve(&fig, &held).scale(x).get_domain(), [-0.5, 10.5]);
}

#[test]
fn the_resolved_scale_at_a_position_is_the_handles_without_its_type() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::log());
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .mark(line(y, [1.0, 1e3]));
    let r = resolve(&fig, &Held::default());
    let erased = r.scale_at(x.index()).expect("position 0");
    let linear = erased
        .as_any()
        .downcast_ref::<Linear>()
        .expect("a linear scale");
    assert_eq!(linear.get_domain(), r.scale(x).get_domain());
    let log = r.scale_at(y.index()).expect("position 1");
    assert_eq!(log.as_any().type_id(), std::any::TypeId::of::<scale::Log>());
    assert!(r.scale_at(2).is_none());
}

#[test]
fn panels_list_the_handles_they_zoom_once_each() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let vib = fig.scale(scale::linear().domain([0.0, 1.0]));
    let temp = fig.fit(scale::log());
    fig.panel(0, 0)
        .zoom_x(x)
        .zoom_y(vib)
        .zoom_y(temp)
        .zoom_y(vib);
    fig.panel(0, 1).zoom_y(temp);
    fig.panel(1, 0);
    let r = resolve(&fig, &Held::default());
    let none: &[usize] = &[];
    assert_eq!(r.zooms(0), (&[0][..], &[1, 2][..]));
    assert_eq!(r.zooms(1), (none, &[2][..]));
    assert_eq!(r.zooms(2), (none, none));
    assert_eq!(r.zooms(3), (none, none), "no such panel");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "handle from another figure")]
fn a_zoomed_handle_from_another_figure_panics_when_resolved() {
    let mut first = Figure::new();
    let foreign = first.fit(scale::linear());
    let mut fig = Figure::new();
    fig.fit(scale::linear());
    fig.panel(0, 0).zoom_x(foreign);
    resolve(&fig, &Held::default());
}

#[test]
fn axes_report_their_side_and_handle() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).axis(axis::left(y)).axis(axis::bottom(x));
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.axis(0, 0), Some((Orient::Left, y.index())));
    assert_eq!(r.axis(0, 1), Some((Orient::Bottom, x.index())));
    assert_eq!(r.axis(0, 2), None);
    assert_eq!(r.axis(1, 0), None);
}

#[test]
fn categorical_handles_keep_their_keys() {
    let mut fig = Figure::new();
    let b = fig.fit(scale::band(["a", "b"]));
    fig.panel(0, 0).mark(line(b, ["c"]));
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.scale(b).get_domain(), ["a", "b"]);
    assert_eq!(Kind::domain(r.scale(b)), None);
}

#[test]
fn the_cache_counts_the_figure_title_once_per_new_text() {
    let mut fig = Figure::new();
    fig.title("Hub vibration and bearing temperature");
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    cache.begin();
    fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let stats = cache.stats().text;
    // Each resolve runs two layout passes, each with the title in its
    // batch: one miss, then hits.
    assert_eq!((stats.entries, stats.misses, stats.hits), (1, 1, 3));
}

/// The single-panel row: 640 × 480, `left(y)` and `bottom(x)`, x fitted
/// to 0 … 10 and y to 0 … 3, no titles.
fn one_panel() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .mark(line(y, [0.0, 3.0]))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    (fig, x, y)
}

/// `fig` drawn at 640 × 480 for vector output.
fn draw(fig: &Figure<'_>) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    )
}

/// The paths of the strokes among `commands`.
fn strokes(commands: &[Command]) -> Vec<&BezPath> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { path, .. } = c {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

/// The glyph runs among `commands`.
fn runs(commands: &[Command]) -> Vec<&GlyphRun> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect()
}

/// The id of element `index` of role `role` in panel `panel`.
fn id(role: u64, panel: u64, index: u64) -> part::Id {
    part::Id::new(role << 56 | panel << 32 | index)
}

#[test]
fn decorations_of_one_panel() {
    let (fig, ..) = one_panel();
    let list = draw(&fig);
    list::check(&list).expect("a valid list");
    let dump = list::dump(&list, list::dump::Options { glyphs: false });
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("decorations_of_one_panel.txt", &dump);
}

#[test]
fn figure_axis_ticks_follow_core_axis_positions() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .axis(axis::bottom(x));
    let resolved = resolve(&fig, &Held::default());
    let area = resolved.plot_areas()[0];
    let ranged = resolved.scale(x).clone().with_range([0.0, area.width()]);
    // One tick per 80 px: 8 over about 610 px.
    let count = (area.width() / 80.0).round() as usize;
    assert_eq!(count, 8, "{area:?}");
    let expected: Vec<f64> = ranged
        .tick_set(count)
        .values()
        .iter()
        .map(|v| area.x0 + Ticks::position(&ranged, v, 0.0).expect("placed"))
        .collect();
    let list = draw(&fig);
    let part = list
        .parts
        .iter()
        .find(|p| p.id == id(4, 0, 0))
        .expect("an axis part");
    let axis = &list.commands[part.commands.clone()];
    let ticks: Vec<f64> = strokes(axis)[0]
        .elements()
        .iter()
        .filter_map(|e| {
            if let PathEl::MoveTo(p) = e {
                Some(p.x)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(ticks, expected);
    let labels: Vec<&str> = runs(axis).iter().map(|run| &*run.text).collect();
    let digits: Vec<String> = (0..=10).map(|i| i.to_string()).collect();
    assert_eq!(labels, digits);
}

/// The strokes of the axis part of panel 0, axis `axis`, in `list` (its
/// ticks; the line is a part of its own): each path's tick lengths (its
/// segments') and its width.
fn tick_strokes(list: &List, axis: u64) -> Vec<(Vec<f64>, f64)> {
    let part = list
        .parts
        .iter()
        .find(|p| p.id == id(4, 0, axis))
        .expect("an axis part");
    list.commands[part.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { path, style, .. } = c {
                let mut lengths = Vec::new();
                let mut from = None;
                for e in path.elements() {
                    match e {
                        PathEl::MoveTo(p) => from = Some(*p),
                        PathEl::LineTo(p) => {
                            lengths.push(from.map_or(0.0, |f| f.distance(*p)));
                        }
                        PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => {}
                    }
                }
                Some((lengths, style.width))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn ticks_the_scale_blanks_draw_at_the_minor_size_and_width() {
    let mut fig = Figure::new();
    let y = fig.scale(scale::log().domain([1.0, 1e5]));
    fig.panel(0, 0).axis(axis::left(y).ticks(10));
    let list = draw(&fig);
    let log = scale::log().domain([1.0, 1e5]);
    let labels = log.labels(&log.tick_set(10), &rustd3_core::locale::Locale::default());
    let blank = labels.iter().filter(|l| l.is_empty()).count();
    // d3 labels mantissas 1 and 2 of each decade at 10 ticks, and 1e5.
    assert_eq!((labels.len(), blank), (46, 35));
    let strokes = tick_strokes(&list, 0);
    assert_eq!(strokes.len(), 2, "major and minor ticks");
    let (major, width) = &strokes[0];
    assert_eq!(major.len(), 11);
    assert!(major.iter().all(|l| (l - 8.0).abs() < 1e-9), "{major:?}");
    assert!((width - 5.0 / 3.0).abs() < 1e-9);
    let (minor, width) = &strokes[1];
    assert_eq!(minor.len(), 35);
    assert!(
        minor.iter().all(|l| (l - 16.0 / 3.0).abs() < 1e-9),
        "{minor:?}"
    );
    assert!((width - 4.0 / 3.0).abs() < 1e-9);
}

#[test]
fn ticks_whose_labels_collide_or_a_format_blanks_keep_the_major_size() {
    // Labels this long collide on a 640 px axis: some are dropped.
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([1e9, 1e9 + 1.0]));
    fig.panel(0, 0)
        .axis(axis::bottom(x).ticks(10).offset(false))
        .axis(axis::top(x).ticks(10).format(|_| String::new()));
    let list = draw(&fig);
    let bottom = list
        .parts
        .iter()
        .find(|p| p.id == id(4, 0, 0))
        .expect("an axis");
    let labelled = runs(&list.commands[bottom.commands.clone()]).len();
    assert!((1..11).contains(&labelled), "{labelled} labels kept");
    for axis in [0, 1] {
        let strokes = tick_strokes(&list, axis);
        assert_eq!(strokes.len(), 1, "major ticks");
        assert_eq!(strokes[0].0.len(), 11);
        assert!(strokes[0].0.iter().all(|l| (l - 8.0).abs() < 1e-9));
    }
}

#[test]
fn decorations_are_parts_in_drawing_order_bounded_by_their_commands() {
    let (mut fig, _, y) = one_panel();
    fig.title("Hub vibration and bearing temperature");
    fig.panel(0, 0)
        .title("Wheel: RL")
        .border(Rgb::hex(0x4d4d4d))
        .axis(axis::right(y).label("temp"));
    let list = draw(&fig);
    list::check(&list).expect("a valid list");
    let ids: Vec<part::Id> = list.parts.iter().map(|p| p.id).collect();
    // background, panel, border, three axes (each its line, then its
    // ticks and labels), the panel title, the title.
    let expected = [
        id(0, 0, 0),
        id(2, 0, 0),
        id(3, 0, 0),
        id(8, 0, 0),
        id(4, 0, 0),
        id(8, 0, 1),
        id(4, 0, 1),
        id(8, 0, 2),
        id(4, 0, 2),
        id(5, 0, 0),
        id(1, 0, 0),
    ];
    assert_eq!(ids, expected);
    let mut end = 0;
    for part in &list.parts {
        assert_eq!(part.commands.start, end, "parts are contiguous and ordered");
        end = part.commands.end;
        assert_ne!(part.hash, 0);
    }
    assert_eq!(end, list.commands.len());
    assert_eq!(list.parts[0].bounds, Rect::new(0.0, 0.0, 640.0, 480.0));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert_eq!(list.parts[1].bounds, area);
    let title = list.parts[10].bounds;
    // The title's ink top on the padding, to the glyphs' f32 precision.
    assert!(
        (title.y0 - 0.8 * 44.0 / 3.0).abs() < 1e-4 && title.y1 < area.y0,
        "{title:?}"
    );
    let heading = list.parts[9].bounds;
    assert!(
        heading.y1 <= area.y0 - 8.0 + 1e-4 && heading.y0 > title.y1,
        "{heading:?}"
    );
}

#[test]
fn a_tinted_axis_takes_its_palette_color() {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(["x", "t"], COLORBLIND));
    fig.panel(0, 0)
        .mark(line(y, [30.0, 65.0]))
        .axis(axis::left(y).color(color, "t").label("temp"));
    let list = draw(&fig);
    // The axis line's part, then its ticks and labels'.
    let (line, rest) = (&list.parts[2], &list.parts[3]);
    assert_eq!((line.id, rest.id), (id(8, 0, 0), id(4, 0, 0)));
    let axis = &list.commands[line.commands.start..rest.commands.end];
    // "t" is the second key: colorblind's orange.
    let orange = Brush::Solid(Color::from_rgba8(0xde, 0x8f, 0x05, 0xff));
    let strokes = axis.iter().filter_map(|c| {
        if let Command::Stroke { paint, .. } = c {
            Some(paint)
        } else {
            None
        }
    });
    let fills = runs(axis).into_iter().map(|run| &run.fill);
    let paints: Vec<&Brush> = strokes.chain(fills).collect();
    // [28.25, 66.75] over about 433 px: 12 ticks asked, every 2 from 30.
    assert_eq!(
        paints.len(),
        2 + 19 + 1,
        "line, ticks, 19 labels, the title"
    );
    assert!(paints.iter().all(|p| **p == orange), "{paints:?}");
}

/// The texts of the glyph runs of axis 0 of panel 0 in `fig`, drawn at
/// 960 × 480 for vector output.
fn axis_texts(fig: &Figure<'_>) -> Vec<String> {
    let list = fig.draw_with(
        &mut cache(),
        Size::new(960.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    let axis = list
        .parts
        .iter()
        .find(|p| p.id == id(4, 0, 0))
        .expect("an axis");
    runs(&list.commands[axis.commands.clone()])
        .iter()
        .map(|run| run.text.to_string())
        .collect()
}

/// A bottom axis over `domain` asking for `count` ticks, its labels in
/// `locale`.
fn in_locale(domain: [f64; 2], count: usize, locale: Locale) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain(domain));
    fig.locale(locale)
        .panel(0, 0)
        .axis(axis::bottom(x).ticks(count));
    fig
}

#[test]
fn default_tick_labels_are_written_in_the_figure_locale() {
    let thousands = |sep: &str| -> Vec<String> {
        (0..=6)
            .map(|i| match i {
                0 => "0".to_string(),
                i => format!("{}{sep}000", 2 * i),
            })
            .collect()
    };
    let neutral = axis_texts(&in_locale([0.0, 12_000.0], 6, Locale::default()));
    assert_eq!(neutral, thousands("\u{202f}"));
    let us = axis_texts(&in_locale([0.0, 12_000.0], 6, Locale::EN_US));
    assert_eq!(us, thousands(","));
    // The neutral locale groups from five integer digits.
    let short = axis_texts(&in_locale([0.0, 8_000.0], 4, Locale::default()));
    assert_eq!(short, ["0", "2000", "4000", "6000", "8000"]);
    // Offset notation's labels too: [1e9, 1e9 + 12 000] reads 0 … 12 000.
    let offset = axis_texts(&in_locale([1e9, 1e9 + 12_000.0], 6, Locale::EN_US));
    assert_eq!(offset[..7], us[..]);
    assert_eq!(offset[7], "+1e9");
}

#[test]
fn the_notation_label_is_written_in_the_figure_locale() {
    let de = Locale::from(format::Locale::DE_DE);
    let offset = axis_texts(&in_locale([12_345.0, 12_345.3], 6, de.clone()));
    assert_eq!(offset[..2], ["0,00", "0,05"]);
    assert_eq!(offset.last().map(String::as_str), Some("+1,2345e4"));
    let hyphen = Locale::from(format::Locale::new(format::Definition {
        minus: "-".into(),
        ..format::Definition::default()
    }));
    let tiny = axis_texts(&in_locale([-3e-7, 0.0], 6, hyphen));
    assert_eq!(tiny[0], "-3.0");
    assert_eq!(tiny.last().map(String::as_str), Some("1e-7"));
    let tiny = axis_texts(&in_locale([0.0, 3e-7], 6, de));
    assert_eq!(tiny[1..3], ["0,5", "1,0"]);
    assert_eq!(tiny.last().map(String::as_str), Some("1e\u{2212}7"));
}

#[test]
fn a_format_closure_writes_its_own_labels_whatever_the_locale() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 12_000.0]));
    fig.locale(Locale::EN_US)
        .panel(0, 0)
        .axis(axis::bottom(x).ticks(6).format(|v| format!("{v:.0}°")));
    let labels = axis_texts(&fig);
    assert_eq!(labels[..3], ["0°", "2000°", "4000°"]);
    assert_eq!(labels.len(), 7);
}

#[test]
fn a_themes_ink_and_background_are_drawn_and_read_back() {
    let (mut fig, _, _) = one_panel();
    let red = Rgb::hex(0xff0000);
    let half = Rgb::hex(0x336699).with_opacity(0.5);
    fig.title("Ink");
    fig.theme(Theme::default().ink(red).background(half));
    assert_eq!(fig.get_theme().get_ink(), red);
    assert_eq!(fig.get_theme().get_background(), half);
    let list = draw(&fig);
    let Command::Fill { paint, .. } = &list.commands[list.parts[0].commands.start] else {
        panic!("the figure background is a fill");
    };
    assert_eq!(
        *paint,
        Brush::Solid(Color::from_rgba8(0x33, 0x66, 0x99, 128))
    );
    let ink = Brush::Solid(Color::from_rgba8(0xff, 0, 0, 0xff));
    // Both axes (lines, ticks, labels) and the title.
    let decorations = &list.commands[list.parts[2].commands.start..];
    let strokes = decorations.iter().filter_map(|c| {
        if let Command::Stroke { paint, .. } = c {
            Some(paint)
        } else {
            None
        }
    });
    let fills = runs(decorations).into_iter().map(|run| &run.fill);
    let paints: Vec<&Brush> = strokes.chain(fills).collect();
    assert!(paints.len() > 10, "{}", paints.len());
    assert!(paints.iter().all(|p| **p == ink), "{paints:?}");
}
