//! Our SVG rendered back by resvg against vello_cpu's render of the list.

use rustd3_figure::lower;
use rustd3_figure::raster::Image;
use rustd3_figure::svg;

use crate::raster::vello::{diff, whole};
use crate::sink;

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
