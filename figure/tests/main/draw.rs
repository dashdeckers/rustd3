//! Drawing figures: the scatter's list, `draw` for vector output, and the
//! trace figure redrawn from its caches.

use rustd3_figure::kurbo::Size;
use rustd3_figure::list::{self, Command};
use rustd3_figure::{Held, Target};

use crate::figure::cache;
use crate::marks::marked;
use crate::{scatter, trace};

/// The size the scatter is drawn at.
const SIZE: Size = Size::new(640.0, 480.0);

#[test]
fn a_scatter_of_5600_points_in_three_keys_is_three_markers_commands() {
    let list =
        scatter::scatter(5600).draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    let counts: Vec<usize> = marked(&list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Markers { at, .. } = c {
                Some(at.len())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(counts.len(), 3);
    assert_eq!(counts.iter().sum::<usize>(), 5600);
}

#[test]
fn list_dump_of_scatter() {
    let list =
        scatter::scatter(300).draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    list::check(&list).expect("a valid list");
    let dump = list::dump(&list, list::dump::Options { glyphs: true });
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("list_dump_of_scatter.txt", &dump);
}

#[cfg(feature = "fonts")]
#[test]
fn draw_is_a_vector_draw_with_a_fresh_cache() {
    let fig = scatter::scatter(30);
    let fresh = fig.draw_with(
        &mut rustd3_figure::Cache::default(),
        SIZE,
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(fig.draw(SIZE), fresh);
}

#[test]
fn the_trace_figure_redraws_with_no_m4_query_and_no_typst_compile() {
    let rec = trace::recording(200_000);
    let fig = trace::trace(&rec);
    let mut cache = cache();
    let target = Target::File { scale: 2.0 };
    let size = Size::new(960.0, 600.0);
    let first = fig.draw_with(&mut cache, size, target, &Held::default());
    let before = cache.stats();
    assert_eq!(before.m4_queries, 3);
    let second = fig.draw_with(&mut cache, size, target, &Held::default());
    let after = cache.stats();
    assert_eq!(
        (after.m4_queries, after.text.compiles),
        (before.m4_queries, before.text.compiles)
    );
    assert_eq!(first, second);
}
