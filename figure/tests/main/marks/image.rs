//! Images: a grid of values as one unsmoothed RGBA image at the device
//! rectangle of its extent, rows flipped so row 0 sits at its first y.

use rustd3_core::{scale, scheme};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::mark::{Error, Grid, image};
use rustd3_figure::peniko::{ImageAlphaType, ImageFormat};
use rustd3_figure::scale::Paint;
use rustd3_figure::{Figure, Held, Target};

use super::marked;
use crate::figure::{cache, resolve};

/// The image a list's marks drew: its rect, size and straight RGBA bytes.
fn drawn(list: &List) -> (Rect, [u32; 2], Vec<u8>, bool) {
    list::check(list).expect("a valid list");
    let marked = marked(list);
    assert_eq!(marked.len(), 1, "{marked:?}");
    let Command::Image {
        rect,
        image,
        smooth,
    } = marked[0]
    else {
        panic!("an image: {:?}", marked[0]);
    };
    assert_eq!(
        (image.format, image.alpha_type),
        (ImageFormat::Rgba8, ImageAlphaType::Alpha)
    );
    (
        *rect,
        [image.width, image.height],
        image.data.data().to_vec(),
        *smooth,
    )
}

/// A 2 × 2 grid of 0, 1 (row 0) and 2, 3 (row 1) over [1, 3] × [1, 3] on
/// x and y fixed to [0, 4], colored by viridis over [0, 3].
fn two_by_two() -> (Figure<'static>, Vec<[u8; 4]>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 4.0]));
    let y = fig.scale(scale::linear().domain([0.0, 4.0]));
    let heat = fig.scale(scale::sequential([0.0, 3.0], scheme::viridis));
    let grid = Grid::new(2, 2, vec![0.0, 1.0, 2.0, 3.0])
        .expect("four values")
        .extent([1.0, 3.0], [1.0, 3.0]);
    fig.panel(0, 0).mark(image(grid, x, y, heat));
    let r = resolve(&fig, &Held::default());
    let rgba = |v: f64| {
        let c = r.scale(heat).paint(&v).expect("a color");
        [c.r as u8, c.g as u8, c.b as u8, 255]
    };
    let colors = [0.0, 1.0, 2.0, 3.0].map(rgba).to_vec();
    (fig, colors)
}

#[test]
fn a_2_by_2_grid_becomes_a_2_by_2_rgba_image_at_the_device_rect() {
    let (fig, colors) = two_by_two();
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    let (rect, size, rgba, smooth) = drawn(&list);
    assert_eq!(size, [2, 2]);
    assert!(!smooth, "nearest sampling");
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let want = Rect::new(w / 4.0, h / 4.0, 3.0 * w / 4.0, 3.0 * h / 4.0);
    let d = [
        rect.x0 - want.x0,
        rect.y0 - want.y0,
        rect.x1 - want.x1,
        rect.y1 - want.y1,
    ];
    assert!(
        d.iter().all(|d| d.abs() < 1e-9),
        "{rect:?} against {want:?}"
    );
    // Row 1 (2, 3) on top: y runs up.
    let expected: Vec<u8> = [2, 3, 0, 1].iter().flat_map(|&i| colors[i]).collect();
    assert_eq!(rgba, expected);
}

#[test]
fn an_image_on_a_raster_target_has_its_edges_on_device_pixels() {
    let (fig, _) = two_by_two();
    let list = fig.draw_with(
        &mut cache(),
        Size::new(641.0, 479.0),
        Target::Screen { scale: 1.5 },
        &Held::default(),
    );
    let (rect, size, ..) = drawn(&list);
    assert_eq!(size, [2, 2]);
    for v in [rect.x0, rect.y0, rect.x1, rect.y1] {
        assert!((v * 1.5 - (v * 1.5).round()).abs() < 1e-9, "{rect:?}");
    }
}

#[test]
fn reversed_axes_flip_the_image_and_cells_without_color_are_transparent() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([4.0, 0.0]));
    let y = fig.scale(scale::linear().domain([4.0, 0.0]));
    let heat = fig.scale(scale::sequential([0.0, 3.0], scheme::viridis));
    let grid = Grid::new(3, 1, vec![0.0, f64::NAN, 3.0]).expect("three values");
    fig.panel(0, 0).mark(image(grid, x, y, heat));
    let r = resolve(&fig, &Held::default());
    let rgba = |v: f64| {
        let c = r.scale(heat).paint(&v).expect("a color");
        [c.r as u8, c.g as u8, c.b as u8, 255]
    };
    let (rect, size, data, _) = drawn(&super::draw(&fig));
    assert_eq!(size, [3, 1]);
    // x runs right to left: column 2 (3.0) is leftmost.
    let expected: Vec<u8> = [rgba(3.0), [0; 4], rgba(0.0)].concat();
    assert_eq!(data, expected);
    let area = r.plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    assert!(
        (rect.x0 - w / 4.0).abs() < 1e-9 && (rect.x1 - w).abs() < 1e-9,
        "{rect:?}"
    );
    // y runs down from 4 at the bottom: [0, 1] is the top quarter.
    assert!(
        rect.y0.abs() < 1e-9 && (rect.y1 - h / 4.0).abs() < 1e-9,
        "{rect:?}"
    );
}

#[test]
fn an_image_holds_its_extent_and_fits_its_values_to_a_fitted_color_handle() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let heat = fig.fit(scale::sequential([0.0, 1.0], scheme::viridis));
    let grid = Grid::new(2, 1, vec![-2.0, 5.0])
        .expect("two values")
        .extent([10.0, 20.0], [0.0, f64::NAN])
        .extent([10.0, 20.0], [-1.0, 1.0]);
    fig.panel(0, 0).mark(image(grid, x, y, heat));
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.scale(x).get_domain(), [10.0, 20.0], "no margin: held");
    assert_eq!(r.scale(y).get_domain(), [-1.0, 1.0]);
    assert_eq!(r.scale(heat).get_domain(), [-2.0, 5.0]);
}

#[test]
fn only_the_cells_within_the_guard_band_are_drawn() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    let heat = fig.scale(scale::sequential([0.0, 1.0], scheme::viridis));
    // 1,000 columns one plot width wide over [-100.5, 899.5]: the plot
    // area shows halves of columns 100 and 101.
    let grid = Grid::new(1000, 1, vec![0.5; 1000])
        .expect("1,000 values")
        .extent([-100.5, 899.5], [0.0, 1.0]);
    fig.panel(0, 0).mark(image(grid, x, y, heat));
    let (rect, size, ..) = drawn(&super::draw(&fig));
    let w = resolve(&fig, &Held::default()).plot_areas()[0].width();
    // The guard band spans [-w, 2w]: columns 99..103 reach into it.
    assert_eq!(size, [4, 1]);
    let (x0, x1) = (rect.x0 / w, rect.x1 / w);
    assert!(
        (x0 + 1.5).abs() < 1e-9 && (x1 - 2.5).abs() < 1e-9,
        "{rect:?}"
    );
}

#[test]
fn a_grid_whose_values_do_not_fill_its_cells_is_an_error_with_the_lengths() {
    assert_eq!(
        Grid::new(3, 2, vec![0.0; 5]),
        Err(Error::GridLength {
            cols: 3,
            rows: 2,
            len: 5
        })
    );
    assert_eq!(
        Grid::new(usize::MAX, 2, vec![]),
        Err(Error::GridLength {
            cols: usize::MAX,
            rows: 2,
            len: 0
        })
    );
    assert!(Grid::new(0, 0, vec![]).is_ok());
}
