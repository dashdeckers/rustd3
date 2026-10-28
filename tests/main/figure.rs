//! `rustd3::figure` is rustd3-figure, with figure's own paths.

use rustd3::figure::{Figure, axis};
use rustd3::scale;

#[test]
fn rustd3_figure_axis_bottom_is_the_figure_axis_of_a_handle() {
    let mut fig: rustd3_figure::Figure<'_> = Figure::new();
    let x = fig.fit(scale::linear());
    let bottom: rustd3_figure::axis::Axis<'_, rustd3_core::scale::Linear> = axis::bottom(x);
    fig.panel(0, 0).axis(bottom);
}

#[cfg(feature = "fonts")]
#[test]
fn a_figure_draws_with_the_bundled_fonts() {
    use rustd3::figure::kurbo::Size;
    use rustd3::figure::mark::line;

    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(line([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y))
        .axis(axis::bottom(x))
        .axis(axis::left(y));
    let list = fig.draw(Size::new(320.0, 240.0));
    assert!(!list.commands.is_empty());
    assert!(!list.fonts.is_empty());
}

#[cfg(feature = "pdf")]
#[test]
fn rustd3_figure_pdf_writes_a_pdf() {
    use rustd3::figure::kurbo::Size;
    use rustd3::figure::list::List;
    use rustd3::figure::pdf;

    let list = List {
        size: Size::new(100.0, 50.0),
        ..List::default()
    };
    let bytes = pdf::write(&list, &pdf::Options::default()).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

#[cfg(all(feature = "gif", feature = "fonts"))]
#[test]
fn rustd3_figure_gif_writes_a_gif() {
    use std::time::Duration;

    use rustd3::figure::gif::{Repeat, Writer};
    use rustd3::figure::kurbo::Size;

    let mut gif = Writer::new(Vec::new(), Size::new(80.0, 60.0), 1.0, Repeat::Forever).unwrap();
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).axis(axis::bottom(x));
    gif.frame(&fig, Duration::from_millis(40)).unwrap();
    assert!(gif.finish().unwrap().starts_with(b"GIF89a"));
}
