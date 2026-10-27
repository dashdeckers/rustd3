//! Layout by protrusions: tracks, padding, the figure title's reserve, and
//! attached panels sharing pixel ranges exactly.

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::kurbo::Rect;
use rustd3_figure::layout::{Grid, em, px};
use rustd3_figure::text::{self, Key, Style, Text, anchor};
use rustd3_figure::{Figure, Held};

use crate::figure::{line, resolve};

/// 0.8 em of padding at notebook: 0.8 × 11 pt × 4/3.
const PAD: f64 = 0.8 * 11.0 * 4.0 / 3.0;

/// The figure title, measured in the theme's style at `size` px:
/// (width, ink height).
fn title_box(size: f64) -> (f64, f64) {
    let style = Style {
        size,
        bold: false,
        fill: Rgb::hex(0x262626),
        align: anchor::Horizontal::Middle,
    };
    let key = Key::new(&Text::from("Hub vibration and bearing temperature"), &style);
    let mut cache = text::Cache::new(crate::text::fonts());
    cache.prepare([&key]);
    let shaped = cache.get(&key).expect("prepared").expect("shaped");
    (shaped.width(), shaped.ascent() + shaped.descent())
}

/// `a` and `b` are the same f64, bit for bit.
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

/// Asserts that panels of one column share `x` and `w` bit for bit, and
/// panels of one row `y` and `h`.
fn assert_attached(areas: &[Rect], cells: &[(usize, usize)]) {
    for (a, ca) in areas.iter().zip(cells) {
        for (b, cb) in areas.iter().zip(cells) {
            if ca.1 == cb.1 {
                assert!(
                    same(a.x0, b.x0) && same(a.width(), b.width()),
                    "{a:?} {b:?}"
                );
            }
            if ca.0 == cb.0 {
                assert!(
                    same(a.y0, b.y0) && same(a.height(), b.height()),
                    "{a:?} {b:?}"
                );
            }
        }
    }
}

#[test]
fn the_figure_title_reserves_its_ink_height_and_eight_px() {
    let (width, height) = title_box(14.6667);
    assert_eq!(
        (format!("{width:.3}"), format!("{height:.3}")),
        ("289.273".into(), "14.194".into())
    );
    let (large_width, large_height) = title_box(19.2);
    let ratio = 19.2 / 14.6667;
    assert!((large_width / width - ratio).abs() < 1e-6, "{large_width}");
    assert!(
        (large_height / height - ratio).abs() < 1e-6,
        "{large_height}"
    );
    let mut fig = Figure::new();
    fig.title("Hub vibration and bearing temperature");
    fig.panel(0, 0);
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert!(
        (area.y0 - (PAD + large_height + 8.0)).abs() < 1e-9,
        "{area:?}"
    );
    assert!((area.y1 - (480.0 - PAD)).abs() < 1e-9, "{area:?}");
}

#[test]
fn bare_panels_fill_the_padded_canvas_by_weight() {
    let mut fig = Figure::new();
    fig.layout(Grid::new().cols([1.0, 3.0]).col_gaps([px(10.0)]));
    fig.panel(0, 0);
    fig.panel(0, 1);
    let areas = resolve(&fig, &Held::default()).plot_areas().to_vec();
    let free = 640.0 - 2.0 * PAD - 10.0;
    assert!((areas[0].x0 - PAD).abs() < 1e-9);
    assert!((areas[0].width() - free / 4.0).abs() < 1e-9);
    assert!((areas[1].x0 - (PAD + free / 4.0 + 10.0)).abs() < 1e-9);
    assert!((areas[1].x1 - (640.0 - PAD)).abs() < 1e-9);
}

#[test]
fn a_missing_gap_is_one_em() {
    let mut fig = Figure::new();
    fig.panel(0, 0);
    fig.panel(1, 0);
    let areas = resolve(&fig, &Held::default()).plot_areas().to_vec();
    let gap = areas[1].y0 - areas[0].y1;
    assert!((gap - 11.0 * 4.0 / 3.0).abs() < 1e-9, "{gap}");
}

#[test]
fn weights_without_panels_still_take_their_share() {
    let mut fig = Figure::new();
    fig.layout(Grid::new().cols([1.0, 1.0]).col_gaps([em(0.0)]));
    fig.panel(0, 0);
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert!((area.width() - (640.0 - 2.0 * PAD) / 2.0).abs() < 1e-9);
}

#[test]
fn attached_panels_of_a_bare_grid_share_pixel_ranges_exactly() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.layout(
        Grid::new()
            .cols([1.0, 0.3])
            .rows([0.7, 1.0])
            .col_gaps([em(0.4)])
            .row_gaps([em(1.6)]),
    );
    let cells = [(0, 0), (0, 1), (1, 0), (1, 1)];
    for (r, c) in cells {
        fig.panel(r, c)
            .mark(line(x, [0.1, 9.7]))
            .mark(line(y, [0.0, 3.0]));
    }
    let areas = resolve(&fig, &Held::default()).plot_areas().to_vec();
    assert_attached(&areas, &cells);
}
