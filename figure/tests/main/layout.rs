//! Layout by protrusions: tracks, padding, the figure title's reserve, and
//! attached panels sharing pixel ranges exactly.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::layout::{Grid, em, px};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::text::{self, Key, Style, Text, anchor};
use rustd3_figure::{Figure, Held, Panel, Target, axis};

use crate::figure::{cache, line, resolve};

/// 0.8 em of padding at notebook: 0.8 × 11 pt × 4/3.
const PAD: f64 = 0.8 * 11.0 * 4.0 / 3.0;

/// `label` measured in the theme's ink at `size` px: (width, ink height).
fn measure(label: &str, size: f64) -> (f64, f64) {
    let style = Style {
        size,
        bold: false,
        fill: Rgb::hex(0x262626),
        align: anchor::Horizontal::Middle,
    };
    let key = Key::new(&Text::from(label), &style);
    let mut cache = text::Cache::new(crate::text::fonts());
    cache.prepare([&key]);
    let shaped = cache.get(&key).expect("prepared").expect("shaped");
    (shaped.width(), shaped.ascent() + shaped.descent())
}

/// The figure title, measured at `size` px: (width, ink height).
fn title_box(size: f64) -> (f64, f64) {
    measure("Hub vibration and bearing temperature", size)
}

/// `label`'s ink height at `size` px.
fn ink_height(label: &str, size: f64) -> f64 {
    measure(label, size).1
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

/// One 640 × 480 panel with `left(y)` and `bottom(x)`, x fitted to 0 … 10
/// and y to 0 … 3, each axis passed through `axes`.
fn one_panel(
    axes: impl Fn(&mut Panel<'static>, ScaleRef<Linear>, ScaleRef<Linear>),
) -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let panel = fig.panel(0, 0);
    panel.mark(line(x, [0.0, 10.0])).mark(line(y, [0.0, 3.0]));
    axes(panel, x, y);
    (fig, x, y)
}

/// The labels core gives `scale`'s ticks at d3's default count.
fn labels(scale: &Linear) -> Vec<String> {
    scale.labels(&scale.tick_set(10), &Locale::default())
}

#[test]
fn one_panel_reaches_by_its_axes_ticks_pads_and_widest_or_tallest_labels() {
    let (fig, x, y) = one_panel(|p, x, y| {
        p.axis(axis::left(y)).axis(axis::bottom(x));
    });
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.scale(x).get_domain(), [-0.5, 10.5]);
    assert_eq!(
        labels(r.scale(x)),
        (0..=10).map(|i| i.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(
        r.scale(y).get_domain().map(|v| format!("{v:.2}")),
        ["-0.15", "3.15"]
    );
    assert_eq!(
        r.scale(y).tick_set(10).values(),
        [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]
    );
    assert_eq!(labels(r.scale(y))[0], "0.0");
    let area = r.plot_areas()[0];
    // left: 8 + 4.667 + 23.325 ("0.0"); bottom: 8 + 4.667 + 11.093 (digits).
    let left = area.x0 - PAD;
    let bottom = 480.0 - PAD - area.y1;
    assert_eq!(
        [left, bottom].map(|v| format!("{v:.3}")),
        ["35.992", "23.760"]
    );
    let expected = [47.725, 11.733, 580.541, 432.773];
    let actual = [area.x0, area.y0, area.width(), area.height()];
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() < 0.01, "{actual:?}");
    }
}

#[test]
fn an_axis_title_adds_the_label_pad_and_its_ink_height() {
    let (bare, ..) = one_panel(|p, x, _| {
        p.axis(axis::bottom(x));
    });
    let (titled, ..) = one_panel(|p, x, _| {
        p.axis(axis::bottom(x).label("speed [km/h]"));
    });
    let bare = resolve(&bare, &Held::default()).plot_areas()[0];
    let titled = resolve(&titled, &Held::default()).plot_areas()[0];
    let added = bare.y1 - titled.y1;
    let ink = ink_height("speed [km/h]", 16.0);
    assert!((added - (16.0 / 3.0 + ink)).abs() < 1e-9, "{added} {ink}");
}

#[test]
fn a_rotated_left_title_adds_its_ink_height_too() {
    let (bare, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y));
    });
    let (titled, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).label("vibration RMS [m/s²]"));
    });
    let bare = resolve(&bare, &Held::default()).plot_areas()[0];
    let titled = resolve(&titled, &Held::default()).plot_areas()[0];
    let added = titled.x0 - bare.x0;
    let ink = ink_height("vibration RMS [m/s²]", 16.0);
    assert!((added - (16.0 / 3.0 + ink)).abs() < 1e-9, "{added} {ink}");
}

#[test]
fn an_unlabelled_axis_reaches_by_its_tick_and_an_outward_one_by_its_offset() {
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).labels(false).label("ignored"));
    });
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert!((area.x0 - (PAD + 8.0)).abs() < 1e-9, "{area:?}");
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).labels(false).outward(10.0));
    });
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    assert!((area.x0 - (PAD + 18.0)).abs() < 1e-9, "{area:?}");
}

#[test]
fn a_panel_title_adds_the_title_pad_and_its_ink_height_on_top() {
    let (fig, ..) = one_panel(|p, _, _| {
        p.title("Wheel: RL");
    });
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let ink = ink_height("Wheel: RL", 16.0);
    assert!((area.y0 - (PAD + 8.0 + ink)).abs() < 1e-9, "{area:?}");
}

#[test]
fn tick_labels_at_the_ends_widen_the_sides_they_cross() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 3.0]));
    fig.panel(0, 0).axis(axis::bottom(x)).axis(axis::right(y));
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    // "0" overhangs the left end by half its advance, "3.0" the top by
    // half its ink; the right axis's own reach is wider than "10"'s half.
    let digit = 9.3314;
    assert!((area.x0 - PAD - digit / 2.0).abs() < 1e-3, "{area:?}");
    assert!(
        (area.y0 - PAD - ink_height("3.0", 14.6667) / 2.0).abs() < 1e-9,
        "{area:?}"
    );
    let right = 640.0 - PAD - area.x1;
    assert!(
        (right - (8.0 + 14.0 / 3.0 + 23.3249)).abs() < 1e-3,
        "{right}"
    );
}

#[test]
fn attached_panels_with_axes_and_titles_share_pixel_ranges_exactly() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let wide = fig.fit(scale::linear());
    let cells = [(0, 0), (0, 1), (1, 0), (1, 1)];
    for (r, c) in cells {
        let panel = fig.panel(r, c);
        panel
            .mark(line(x, [0.0, 10.0]))
            .mark(line(y, [0.0, 3.0]))
            .mark(line(wide, [0.0, 30_000.0]))
            .axis(axis::left(y))
            .axis(axis::bottom(x));
        if c == 1 {
            panel.axis(axis::right(wide).label("count"));
        }
        if r == 0 {
            panel.title(format!("panel {c}"));
        }
    }
    let areas = resolve(&fig, &Held::default()).plot_areas().to_vec();
    assert_attached(&areas, &cells);
    assert!(areas[0].x0 < areas[1].x0 && areas[0].y0 < areas[2].y0);
}

/// A `rows` × `cols` grid of panels at `size`, each with a left and a
/// bottom axis on its own handles, fitted to 0 … 10 and 0 … 3.
fn grid(rows: usize, cols: usize) -> Figure<'static> {
    let mut fig = Figure::new();
    for r in 0..rows {
        for c in 0..cols {
            let x = fig.fit(scale::linear());
            let y = fig.fit(scale::linear());
            fig.panel(r, c)
                .mark(line(x, [0.0, 10.0]))
                .mark(line(y, [0.0, 3.0]))
                .axis(axis::left(y))
                .axis(axis::bottom(x));
        }
    }
    fig
}

#[test]
fn layout_settles_within_four_passes() {
    for n in [1, 2, 4, 8, 16] {
        for size in [Size::new(640.0, 480.0), Size::new(1600.0, 1200.0)] {
            let fig = grid(n, n);
            let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
            assert!(
                (2..=4).contains(&r.passes()),
                "{n}×{n} at {size:?}: {}",
                r.passes()
            );
        }
    }
    let (fig, ..) = one_panel(|p, x, y| {
        p.axis(axis::left(y).ticks(5))
            .axis(axis::bottom(x).ticks(5));
    });
    assert_eq!(
        resolve(&fig, &Held::default()).passes(),
        2,
        "fixed counts settle at once"
    );
}
