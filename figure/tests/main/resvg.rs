//! Our SVG rendered back by resvg against vello_cpu's render of the list,
//! and the example figures' SVG files against their PNG files.

#[cfg(all(feature = "png", feature = "fonts"))]
use rustd3_figure::kurbo::Size;
use rustd3_figure::raster::Image;
use rustd3_figure::{lower, svg};

use crate::raster::vello::{diff, whole};
use crate::sink;
#[cfg(all(feature = "png", feature = "fonts"))]
use crate::{scatter, trace};

/// `svg` rendered by resvg at `scale`, as straight RGBA.
fn resvg(svg: &str, scale: f32) -> Image {
    let tree =
        resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).expect("our SVG parses");
    let size = tree
        .size()
        .to_int_size()
        .scale_by(scale)
        .expect("a positive size");
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(size.width(), size.height()).expect("a positive size");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Image {
        width: pixmap.width(),
        height: pixmap.height(),
        rgba: pixmap
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect(),
    }
}

#[test]
fn resvg_renders_our_svg_as_vello_cpu_renders_the_list() {
    let list = sink::with_sweep();
    let lowered = lower::lower(&list, svg::CAPS).expect("the sink lowers");
    let svg = svg::write(&lowered.list, &svg::Options::default()).expect("SVG needs no sweep");
    let d = diff(&resvg(&svg, 2.0), &whole(&list, 2.0));
    assert!(
        d.tile_score() <= 0.02,
        "tile score {:.4} at {:?}, max {}",
        d.tile_score(),
        d.worst_tile(),
        d.max()
    );
}

/// `fig`'s PNG at `scale`, written as the example mains write it, against
/// two SVGs rendered by resvg at `scale`: the one `save_svg` writes (vector
/// output, nothing snapped), and the same figure drawn for the PNG's target
/// (its plot areas and hairlines on device pixels): the worst tiles'
/// scores, (example files, same list).
#[cfg(all(feature = "png", feature = "fonts"))]
fn agreement(name: &str, fig: &rustd3_figure::Figure<'_>, size: Size, scale: f64) -> (f64, f64) {
    let base = std::env::temp_dir().join(format!("rustd3-figure-{}-{name}", std::process::id()));
    let (svg_path, png_path) = (base.with_extension("svg"), base.with_extension("png"));
    fig.save_svg(&svg_path, size).expect("the SVG is written");
    fig.save_png(&png_path, size, scale)
        .expect("the PNG is written");
    let svg = std::fs::read_to_string(&svg_path).expect("the SVG reads back");
    let png = std::fs::File::open(&png_path).expect("the PNG opens");
    let mut reader = png::Decoder::new(std::io::BufReader::new(png))
        .read_info()
        .expect("a PNG");
    let mut rgba = vec![0; reader.output_buffer_size().expect("a size")];
    let frame = reader.next_frame(&mut rgba).expect("one frame");
    rgba.truncate(frame.buffer_size());
    std::fs::remove_file(&svg_path).expect("the SVG is removed");
    std::fs::remove_file(&png_path).expect("the PNG is removed");
    let png = Image {
        width: frame.width,
        height: frame.height,
        rgba,
    };
    let target = rustd3_figure::Target::File { scale };
    let held = rustd3_figure::Held::default();
    let list = fig.draw_with(&mut crate::figure::cache(), size, target, &held);
    let lowered = lower::lower(&list, svg::CAPS).expect("the figure lowers");
    let same = svg::write(&lowered.list, &svg::Options::default()).expect("SVG needs no sweep");
    let files = diff(&resvg(&svg, scale as f32), &png).tile_score();
    let same = diff(&resvg(&same, scale as f32), &png).tile_score();
    (files, same)
}

#[cfg(all(feature = "png", feature = "fonts"))]
#[test]
fn resvg_renders_the_example_figures_as_their_pngs() {
    let rec = trace::recording(trace::SAMPLES);
    let (files, same) = agreement("trace", &trace::trace(&rec), trace::SIZE, trace::SCALE);
    // resvg and vello_cpu shade the trace's impacts (a stroke up and back
    // down within a quarter px, overlapping itself) 0.0258 apart on the
    // very same vector list; on the PNG's own list 0.0200.
    assert!(same <= 0.03, "trace: tile score {same:.4}");
    // The SVG file is not snapped: the PNG's plot areas sit up to half a
    // device px elsewhere, 0.0473 at 2x.
    assert!(files <= 0.05, "trace files: tile score {files:.4}");
    let fig = scatter::scatter(scatter::POINTS);
    let (files, same) = agreement("scatter", &fig, scatter::SIZE, scatter::SCALE);
    assert!(same <= 0.02, "scatter: tile score {same:.4}");
    assert!(files <= 0.05, "scatter files: tile score {files:.4}");
}
