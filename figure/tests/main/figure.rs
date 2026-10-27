//! The figure model: handles, the scale table, held scales, and the test
//! mark the fitting and layout tests share.

use rustd3_core::scale;
use rustd3_figure::kurbo::Size;
use rustd3_figure::list::List;
use rustd3_figure::mark::{Ctx, Mark};
use rustd3_figure::scale::{Kind, ScaleRef};
use rustd3_figure::{Cache, Figure, Fits, Held, Resolved, Target};

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
