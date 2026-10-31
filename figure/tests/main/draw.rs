//! Drawing and saving figures: the scatter's list and SVG, `draw` for
//! vector output, the trace figure redrawn from its caches, files, and
//! `figures_equal`.

#[cfg(feature = "raster")]
use rustd3_core::scale;
#[cfg(feature = "fonts")]
use rustd3_figure::Error;
#[cfg(feature = "raster")]
use rustd3_figure::Figure;
use rustd3_figure::kurbo::Size;
use rustd3_figure::list::{self, Command};
#[cfg(feature = "raster")]
use rustd3_figure::mark::dots;
use rustd3_figure::{Held, Target, lower, svg};

use crate::figure::cache;
use crate::marks::marked;
use crate::support::figures_equal;
use crate::{scatter, trace};

/// The size the scatter is drawn at: the example's.
const SIZE: Size = scatter::SIZE;

#[test]
fn a_scatter_of_5600_points_in_three_keys_is_three_markers_commands() {
    let list = scatter::scatter(scatter::POINTS).draw_with(
        &mut cache(),
        SIZE,
        Target::Vector,
        &Held::default(),
    );
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
    assert_eq!(counts.iter().sum::<usize>(), 5_600);
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
        &mut rustd3_figure::cache::Cache::default(),
        SIZE,
        Target::Vector,
        &Held::default(),
    );
    assert_eq!(fig.draw(SIZE), fresh);
}

#[test]
fn the_trace_figure_redraws_with_no_m4_query_and_no_typst_compile() {
    let rec = trace::recording(trace::SAMPLES);
    let fig = trace::trace(&rec);
    let mut cache = cache();
    let target = Target::File {
        scale: trace::SCALE,
    };
    let size = trace::SIZE;
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

#[test]
fn figures_equal_tells_a_figure_from_another() {
    let size = Size::new(320.0, 240.0);
    assert_eq!(
        figures_equal(&scatter::scatter(30), &scatter::scatter(30), size),
        Ok(())
    );
    let error = figures_equal(&scatter::scatter(30), &scatter::scatter(31), size);
    assert!(error.is_err_and(|e| e.contains("markers n11")));
}

/// A figure with fixed scales and dots at `at`.
#[cfg(feature = "raster")]
fn dotted(at: [(f64, f64); 3]) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 2.0]));
    let y = fig.scale(scale::linear().domain([0.0, 2.0]));
    fig.panel(0, 0).mark(dots(at, x, y));
    fig
}

#[cfg(feature = "raster")]
#[test]
fn figures_equal_renders_what_dumps_leave_out() {
    let size = Size::new(320.0, 240.0);
    let corners = |middle| [(0.5, 0.5), middle, (1.5, 1.5)];
    let (a, b) = (dotted(corners((1.0, 1.0))), dotted(corners((1.0, 0.75))));
    let error = figures_equal(&a, &b, size).expect_err("the dots moved");
    assert!(error.contains("renders at 1× differ"), "{error}");
}

#[test]
fn svg_of_scatter() {
    let list =
        scatter::scatter(300).draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    let lowered = lower::lower(&list, svg::CAPS).expect("the scatter lowers");
    assert!(lowered.warnings.is_empty());
    let svg = svg::write(&lowered.list, &svg::Options::default()).expect("SVG takes the list");
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("svg_of_scatter.txt", &svg);
}

/// A path under the system's temporary directory for test `name`.
#[cfg(feature = "fonts")]
fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rustd3-figure-{}-{name}", std::process::id()))
}

#[cfg(feature = "fonts")]
#[test]
fn save_svg_writes_the_lowered_vector_draw() {
    let fig = scatter::scatter(300);
    let path = scratch("scatter.svg");
    let warnings = fig.save_svg(&path, SIZE).expect("the file is written");
    let written = std::fs::read_to_string(&path).expect("the file reads back");
    std::fs::remove_file(&path).expect("the file is removed");
    assert!(warnings.is_empty());
    let lowered = lower::lower(&fig.draw(SIZE), svg::CAPS).expect("the scatter lowers");
    let svg = svg::write(&lowered.list, &svg::Options::default()).expect("SVG takes the list");
    assert_eq!(written, svg);
}

#[cfg(feature = "fonts")]
#[test]
fn save_svg_names_the_file_it_cannot_write() {
    let path = scratch("missing").join("scatter.svg");
    let error = scatter::scatter(3)
        .save_svg(&path, SIZE)
        .expect_err("no such directory");
    assert!(matches!(&error, Error::Io { path: p, .. } if *p == path));
    assert!(error.to_string().starts_with("cannot write "), "{error}");
}

#[cfg(all(feature = "png", feature = "fonts"))]
#[test]
fn save_png_at_2x_writes_a_1280_by_960_rgba_png() {
    let path = scratch("scatter.png");
    scatter::scatter(300)
        .save_png(&path, SIZE, 2.0)
        .expect("the file is written");
    let file = std::fs::File::open(&path).expect("the file opens");
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let reader = decoder.read_info().expect("a PNG");
    let info = reader.info();
    let (width, height, color, depth) = (info.width, info.height, info.color_type, info.bit_depth);
    std::fs::remove_file(&path).expect("the file is removed");
    assert_eq!((width, height), (1280, 960));
    assert_eq!((color, depth), (png::ColorType::Rgba, png::BitDepth::Eight));
}
