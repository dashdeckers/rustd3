//! The figure model: handles, the scale table, held scales, decorations
//! and their parts, and the test mark the fitting and layout tests share.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{BezPath, PathEl, Rect, Size};
use rustd3_figure::list::{self, Command, GlyphRun, List, part};
use rustd3_figure::mark::{Ctx, Mark};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::scale::{Kind, ScaleRef};
use rustd3_figure::{Cache, Figure, Fits, Held, Resolved, Target, axis};

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
    assert_eq!((stats.entries, stats.misses, stats.hits), (1, 1, 1));
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
    let expected: Vec<f64> = ranged
        .tick_set(10)
        .values()
        .iter()
        .map(|v| area.x0 + Ticks::position(&ranged, v, 0.0).expect("placed"))
        .collect();
    let list = draw(&fig);
    let axis = &list.commands[list.parts[2].commands.clone()];
    let ticks: Vec<f64> = strokes(axis)[1]
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
    // background, panel, border, three axes, the panel title, the title.
    let expected = [
        id(0, 0, 0),
        id(2, 0, 0),
        id(3, 0, 0),
        id(4, 0, 0),
        id(4, 0, 1),
        id(4, 0, 2),
        id(5, 0, 0),
        id(1, 0, 0),
    ];
    assert_eq!(ids, expected);
    let mut end = 0;
    for part in &list.parts {
        assert_eq!(part.commands.start, end, "parts are contiguous and ordered");
        end = part.commands.end;
        assert_eq!(part.hash, 0);
    }
    assert_eq!(end, list.commands.len());
    assert_eq!(list.parts[0].bounds, Rect::new(0.0, 0.0, 640.0, 480.0));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert_eq!(list.parts[1].bounds, area);
    let title = list.parts[7].bounds;
    // The title's ink top on the padding, to the glyphs' f32 precision.
    assert!(
        (title.y0 - 0.8 * 44.0 / 3.0).abs() < 1e-4 && title.y1 < area.y0,
        "{title:?}"
    );
    let heading = list.parts[6].bounds;
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
    let axis = &list.commands[list.parts[2].commands.clone()];
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
    assert_eq!(paints.len(), 2 + 8 + 1, "line, ticks, 8 labels, the title");
    assert!(paints.iter().all(|p| **p == orange), "{paints:?}");
}
