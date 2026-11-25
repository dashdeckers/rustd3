//! Layout by protrusions: tracks, padding, the figure title's reserve, and
//! attached panels sharing pixel ranges exactly.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_figure::axis::Axis;
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::layout::{Cell, Grid, em, px};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::text::{self, Key, Style, Text, anchor};
use rustd3_figure::{Figure, Held, Panel, Target, axis};

use crate::figure::{cache, line, resolve};

/// 0.8 em of padding at notebook: 0.8 × 11 pt × 4/3.
const PAD: f64 = 0.8 * 11.0 * 4.0 / 3.0;

/// `label` shaped in the theme's ink at `size` px: (width, ink ascent,
/// ink descent).
fn shape(label: &str, size: f64) -> (f64, f64, f64) {
    let style = Style {
        font: None,
        mono: None,
        size,
        bold: false,
        fill: Rgb::hex(0x262626),
        align: anchor::Horizontal::Middle,
    };
    let key = Key::new(&Text::from(label), &style);
    let mut cache = text::Cache::new(crate::text::fonts());
    cache.prepare([&key]);
    let shaped = cache.get(&key).expect("prepared").expect("shaped");
    (
        shaped.get_width(),
        shaped.get_ascent(),
        shaped.get_descent(),
    )
}

/// `label` measured in the theme's ink at `size` px: (width, ink height).
fn measure(label: &str, size: f64) -> (f64, f64) {
    let (width, ascent, descent) = shape(label, size);
    (width, ascent + descent)
}

/// The figure title, measured at `size` px: (width, ink height).
fn title_box(size: f64) -> (f64, f64) {
    measure("Hub vibration and bearing temperature", size)
}

/// `label`'s ink height at `size` px.
fn ink_height(label: &str, size: f64) -> f64 {
    measure(label, size).1
}

/// `label`'s ink above its baseline at `size` px.
fn ink_ascent(label: &str, size: f64) -> f64 {
    shape(label, size).1
}

/// DejaVu Sans's OS/2 typographic ascender and descender (1556 and 492 of
/// 2048 units) at `size` px: the line box a row of tick labels shares.
fn line_box(size: f64) -> [f64; 2] {
    [1556.0, 492.0].map(|units| units / 2048.0 * size)
}

/// Whether `a` and `b` are the same f64, bit for bit.
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
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
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
    let areas = resolve(&fig, &Held::default()).get_plot_areas().to_vec();
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
    let areas = resolve(&fig, &Held::default()).get_plot_areas().to_vec();
    let gap = areas[1].y0 - areas[0].y1;
    assert!((gap - 11.0 * 4.0 / 3.0).abs() < 1e-9, "{gap}");
}

#[test]
fn weights_and_gaps_that_are_not_finite_lengths_take_the_defaults() {
    let areas = |grid: Grid| {
        let mut fig = Figure::new();
        fig.layout(grid);
        fig.panel(0, 0);
        fig.panel(1, 1);
        resolve(&fig, &Held::default()).get_plot_areas().to_vec()
    };
    let plain = areas(Grid::new());
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        let weighed = areas(Grid::new().rows([bad, 1.0]).cols([1.0, bad]));
        assert_eq!(weighed, plain, "weight {bad}");
    }
    // Equal weights split equally, however small or large.
    for equal in [5e-324, f64::MAX] {
        let weighed = areas(Grid::new().rows([equal, equal]).cols([equal, equal]));
        assert_eq!(weighed, plain, "weights {equal}");
    }
    for bad in [f64::NAN, f64::INFINITY] {
        let gapped = areas(Grid::new().row_gaps([em(bad)]).col_gaps([px(bad)]));
        assert_eq!(gapped, plain, "gap {bad}");
    }
    // Finite in em, infinite in px.
    assert_eq!(areas(Grid::new().row_gaps([em(1e308)])), plain);
}

#[cfg(feature = "fonts")]
#[test]
fn a_size_that_is_not_finite_and_positive_leaves_no_room() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(rustd3_figure::mark::histogram([1.0, 2.0, 2.0], x, y))
        .axis(axis::bottom(x));
    for size in [
        Size::new(-640.0, 480.0),
        Size::new(640.0, f64::NAN),
        Size::new(f64::INFINITY, 480.0),
    ] {
        let resolved = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        let area = resolved.get_plot_areas()[0];
        assert!(
            [area.x0, area.y0, area.x1, area.y1]
                .iter()
                .all(|v| v.is_finite())
                && area.width() >= 0.0
                && area.height() >= 0.0,
            "{size:?}: {area:?}"
        );
        let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
        assert!(
            list.size.width >= 0.0 && list.size.height >= 0.0,
            "{size:?}"
        );
        let issues = fig.check(size);
        assert!(
            issues
                .iter()
                .any(|i| matches!(i, rustd3_figure::check::Issue::NoRoom { .. })),
            "{size:?}: {issues:?}"
        );
    }
}

#[test]
fn a_span_that_negative_gaps_pull_back_is_empty_not_inverted() {
    // Reaches that leave no room, and a negative gap that pulls the second
    // column back past the start of the first.
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.layout(Grid::new().col_gaps([px(-400.0)]));
    fig.panel(0, 0).axis(axis::left(y).outward(600.0));
    fig.panel_at(Cell::new(1, 0).span(1, 2));
    let areas = resolve(&fig, &Held::default()).get_plot_areas().to_vec();
    assert!(areas.iter().all(|a| a.width() >= 0.0), "{areas:?}");
}

#[test]
fn an_outward_offset_that_is_not_finite_is_ignored() {
    let areas = |outward: Option<f64>| {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 1.0]));
        let bottom = axis::bottom(x);
        fig.panel(0, 0).axis(match outward {
            Some(px) => bottom.outward(px),
            None => bottom,
        });
        resolve(&fig, &Held::default()).get_plot_areas().to_vec()
    };
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(areas(Some(bad)), areas(None), "{bad}");
    }
}

#[cfg(feature = "fonts")]
#[test]
fn a_cell_past_the_grids_tracks_is_reported_and_not_drawn() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).axis(axis::bottom(x));
    fig.panel(usize::MAX, 0).axis(axis::bottom(x));
    fig.panel_at(Cell::new(1, 1).span(usize::MAX, 1));
    fig.panel(4096, 0);
    fig.panel(4096, 0);
    let resolved = resolve(&fig, &Held::default());
    assert_eq!(resolved.get_plot_areas().len(), 1);
    let outside: Vec<_> = fig
        .check(Size::new(640.0, 480.0))
        .into_iter()
        .filter_map(|issue| {
            if let rustd3_figure::check::Issue::OutsideGrid { row, col } = issue {
                Some((row, col))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(outside, [(usize::MAX, 0), (1, 1), (4096, 0)]);
    let mut last = Figure::new();
    last.panel(4095, 4095);
    assert_eq!(resolve(&last, &Held::default()).get_plot_areas().len(), 1);
}

#[test]
fn weights_without_panels_still_take_their_share() {
    let mut fig = Figure::new();
    fig.layout(Grid::new().cols([1.0, 1.0]).col_gaps([em(0.0)]));
    fig.panel(0, 0);
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
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
    let areas = resolve(&fig, &Held::default()).get_plot_areas().to_vec();
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
    assert_eq!(r.get_scale(x).unwrap().get_domain(), [-0.5, 10.5]);
    assert_eq!(
        labels(r.get_scale(x).unwrap()),
        (0..=10).map(|i| i.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(
        r.get_scale(y)
            .unwrap()
            .get_domain()
            .map(|v| format!("{v:.2}")),
        ["-0.15", "3.15"]
    );
    assert_eq!(
        r.get_scale(y).unwrap().tick_set(10).get_values(),
        [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]
    );
    assert_eq!(labels(r.get_scale(y).unwrap())[0], "0.0");
    let area = r.get_plot_areas()[0];
    // left: 8 + 4.667 + 23.325 ("0.0"); bottom: 8 + 4.667 + 14.667 (the
    // line box, which holds the digits' ink).
    let left = area.x0 - PAD;
    let bottom = 480.0 - PAD - area.y1;
    assert_eq!(
        [left, bottom].map(|v| format!("{v:.3}")),
        ["35.992", "27.333"]
    );
    let expected = [47.725, 11.733, 580.541, 429.200];
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
    let bare = resolve(&bare, &Held::default()).get_plot_areas()[0];
    let titled = resolve(&titled, &Held::default()).get_plot_areas()[0];
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
    let bare = resolve(&bare, &Held::default()).get_plot_areas()[0];
    let titled = resolve(&titled, &Held::default()).get_plot_areas()[0];
    let added = titled.x0 - bare.x0;
    let ink = ink_height("vibration RMS [m/s²]", 16.0);
    assert!((added - (16.0 / 3.0 + ink)).abs() < 1e-9, "{added} {ink}");
}

#[test]
fn an_unlabelled_axis_reaches_by_its_tick_and_an_outward_one_by_its_offset() {
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).labels(false).label("ignored"));
    });
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
    assert!((area.x0 - (PAD + 8.0)).abs() < 1e-9, "{area:?}");
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).labels(false).outward(10.0));
    });
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
    assert!((area.x0 - (PAD + 18.0)).abs() < 1e-9, "{area:?}");
}

#[test]
fn a_panel_title_adds_the_title_pad_the_descender_and_its_ink_ascent_on_top() {
    let (fig, ..) = one_panel(|p, _, _| {
        p.title("Wheel: RL");
    });
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
    let [_, descender] = line_box(16.0);
    let ascent = ink_ascent("Wheel: RL", 16.0);
    let expected = PAD + 8.0 + descender + ascent;
    assert!((area.y0 - expected).abs() < 1e-9, "{area:?}");
}

/// The baselines of panel `panel`'s title in `list`: each run's origin.
fn title_baselines(list: &List, panel: u64) -> Vec<f64> {
    let id = part::Id::new(5 << 56 | panel << 32);
    let part = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .expect("a title part");
    list.commands[part.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.transform.translation().y)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn panel_titles_of_a_row_share_a_baseline_whatever_their_descenders_and_side_axes() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let titles = ["Wheel: FL", "gypsy", "Top", "plain"];
    for (i, title) in titles.into_iter().enumerate() {
        let panel = fig.panel(i / 2, i % 2);
        panel
            .mark(line(x, [0.0, 10.0]))
            .mark(line(y, [0.0, 3.0]))
            .title(title);
        // A labeled left axis overhangs the top by half its line box in
        // the first panel of each row; a top axis raises the second row.
        if i % 2 == 0 {
            panel.axis(axis::left(y));
        }
        if i == 2 {
            panel.axis(axis::top(x));
        }
    }
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    let areas = resolve_at(&fig, size).get_plot_areas().to_vec();
    let [_, descender] = line_box(16.0);
    let [tick_ascender, tick_descender] = line_box(44.0 / 3.0);
    // The top axis reaches its tick, pad and line box.
    let top = 8.0 + 14.0 / 3.0 + tick_ascender + tick_descender;
    for (panel, raise) in [0.0, 0.0, top, top].into_iter().enumerate() {
        let expected = areas[panel].y0 - 8.0 - descender - raise;
        let baselines = title_baselines(&list, panel as u64);
        assert!(!baselines.is_empty());
        for y in baselines {
            assert!(
                (y - expected).abs() < 1e-9,
                "panel {panel}: {y} against {expected}"
            );
        }
    }
}

#[test]
fn tick_labels_at_the_ends_widen_the_sides_they_cross() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 3.0]));
    fig.panel(0, 0).axis(axis::bottom(x)).axis(axis::right(y));
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
    // "0" overhangs the left end by half its advance, "3.0" the top by
    // half its line box, centered on the tick; the right axis's own reach is
    // wider than "10"'s half.
    let digit = 9.3314;
    assert!((area.x0 - PAD - digit / 2.0).abs() < 1e-3, "{area:?}");
    let [ascender, descender] = line_box(44.0 / 3.0);
    assert!(
        (area.y0 - PAD - (ascender + descender) / 2.0).abs() < 1e-9,
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
    let areas = resolve(&fig, &Held::default()).get_plot_areas().to_vec();
    assert_attached(&areas, &cells);
    assert!(areas[0].x0 < areas[1].x0 && areas[0].y0 < areas[2].y0);
}

/// A `rows` × `cols` grid of panels, each with a left and a bottom axis on
/// its own handles, fitted to 0 … 10 and 0 … 3.
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
                (2..=4).contains(&r.get_passes()),
                "{n}×{n} at {size:?}: {}",
                r.get_passes()
            );
        }
    }
    let (fig, ..) = one_panel(|p, x, y| {
        p.axis(axis::left(y).ticks(5))
            .axis(axis::bottom(x).ticks(5));
    });
    assert_eq!(
        resolve(&fig, &Held::default()).get_passes(),
        2,
        "fixed counts settle at once"
    );
}

/// An `n` × `n` grid of shared axes: x fitted and shared per column, y per
/// row, left and bottom axes on every panel.
fn shared(n: usize) -> Figure<'static> {
    let mut fig = Figure::new();
    let xs: Vec<_> = (0..n).map(|_| fig.fit(scale::linear())).collect();
    let ys: Vec<_> = (0..n).map(|_| fig.fit(scale::linear())).collect();
    for (r, &y) in ys.iter().enumerate() {
        for (c, &x) in xs.iter().enumerate() {
            fig.panel(r, c)
                .mark(line(x, [0.0, 10.0 * (c + 1) as f64]))
                .mark(line(y, [0.0, 3.0 * (r + 1) as f64]))
                .axis(axis::left(y))
                .axis(axis::bottom(x));
        }
    }
    fig
}

#[test]
fn grids_of_shared_axes_settle_within_four_passes_labelled_on_the_outer_panels() {
    for n in [4, 8, 16] {
        let fig = shared(n);
        let size = Size::new(1600.0, 1200.0);
        let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        assert!(r.get_passes() <= 4, "{n}×{n}: {}", r.get_passes());
        for (i, (row, col)) in (0..n).flat_map(|r| (0..n).map(move |c| (r, c))).enumerate() {
            let left = r.get_plan(i, 0).expect("a left axis");
            let bottom = r.get_plan(i, 1).expect("a bottom axis");
            assert!(left.shown && bottom.shown);
            assert_eq!(left.labeled, col == 0, "{n}×{n} panel {i}");
            assert_eq!(bottom.labeled, row == n - 1, "{n}×{n} panel {i}");
        }
    }
}

/// A 1 × 2 grid whose two panels each get the axes `axes` places, over a
/// y handle fitted to 0 … 3 and a temperature handle fitted to 30 … 65.
fn one_by_two(
    axes: impl Fn(&mut Panel<'static>, ScaleRef<Linear>, ScaleRef<Linear>),
) -> Figure<'static> {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    let temp = fig.fit(scale::linear());
    for c in 0..2 {
        let panel = fig.panel(0, c);
        panel
            .mark(line(y, [0.0, 3.0]))
            .mark(line(temp, [30.0, 65.0]));
        axes(panel, y, temp);
    }
    fig
}

#[test]
fn a_shared_axis_is_labelled_on_the_outer_panel_only() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y));
    });
    let r = resolve(&fig, &Held::default());
    let left = r.get_plan(0, 0).expect("the left panel's axis");
    let right = r.get_plan(1, 0).expect("the right panel's axis");
    assert!(left.shown && left.labeled, "{left:?}");
    assert!(right.shown && !right.labeled, "{right:?}");
    assert_eq!(right.thickness, 8.0, "ticks alone");
    assert_eq!(r.get_plan(1, 1), None);
}

#[test]
fn a_second_axis_on_one_side_stacks_8_px_past_the_first() {
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).label("vib"))
            .axis(axis::left(y).label("temp"));
    });
    let r = resolve(&fig, &Held::default());
    let vib = r.get_plan(0, 0).expect("vib");
    let temp = r.get_plan(0, 1).expect("temp");
    assert_eq!(vib.offset, 0.0);
    assert_eq!(temp.offset, vib.thickness + 8.0);
    let area = r.get_plot_areas()[0];
    assert!(
        (area.x0 - PAD - (temp.offset + temp.thickness)).abs() < 1e-9,
        "{area:?}"
    );
}

#[test]
fn a_stacked_axis_on_an_inner_panel_is_not_drawn() {
    let fig = one_by_two(|p, y, temp| {
        p.axis(axis::left(y).label("vib"))
            .axis(axis::left(temp).label("temp"));
    });
    let r = resolve(&fig, &Held::default());
    let plans: Vec<_> = (0..2)
        .map(|axis| r.get_plan(1, axis).expect("an axis"))
        .collect();
    assert!(plans[0].shown && !plans[0].labeled, "{plans:?}");
    assert!(!plans[1].shown, "{plans:?}");
    let outer = r.get_plan(0, 1).expect("the left panel's temp axis");
    assert!(outer.shown && outer.labeled, "{outer:?}");
    // The undrawn axis takes no room: the right panel reaches by vib's ticks.
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    let axes = |panel: u64| {
        let ids: Vec<part::Id> = (0..2)
            .map(|axis| part::Id::new(4 << 56 | panel << 32 | axis))
            .collect();
        list.parts.iter().filter(|p| ids.contains(&p.id)).count()
    };
    assert_eq!((axes(0), axes(1)), (2, 1));
}

#[test]
fn keep_label_keeps_the_title_of_an_inner_axis_without_its_tick_labels() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).label("amplitude").keep_label(true));
    });
    let r = resolve(&fig, &Held::default());
    let left = r.get_plan(0, 0).expect("the left panel's axis");
    let right = r.get_plan(1, 0).expect("the right panel's axis");
    assert!(right.shown && !right.labeled, "{right:?}");
    // Ticks, a tick pad, a label pad and the title: thinner than the
    // labeled axis, thicker than ticks alone.
    assert!(
        8.0 < right.thickness && right.thickness < left.thickness,
        "{right:?} {left:?}"
    );
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    let texts = |panel: u64| -> Vec<String> {
        let id = part::Id::new(4 << 56 | panel << 32);
        list.parts
            .iter()
            .filter(|p| p.id == id)
            .flat_map(|p| &list.commands[p.commands.clone()])
            .filter_map(|c| {
                if let Command::Glyphs(run) = c {
                    Some(run.text.to_string())
                } else {
                    None
                }
            })
            .collect()
    };
    assert_eq!(texts(1), ["amplitude"]);
    assert!(texts(0).len() > 2 && texts(0).contains(&"amplitude".to_owned()));
}

#[test]
fn labels_true_labels_an_inner_axis() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).labels(true));
    });
    let r = resolve(&fig, &Held::default());
    let inner = r.get_plan(1, 0).expect("the right panel's axis");
    assert!(inner.shown && inner.labeled, "{inner:?}");
}

#[test]
fn an_explicit_outward_axis_on_an_inner_panel_is_not_drawn_and_labels_false_draws_it() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).outward(10.0));
    });
    let r = resolve(&fig, &Held::default());
    assert!(!r.get_plan(1, 0).expect("an axis").shown);
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).outward(10.0).labels(false));
    });
    let r = resolve(&fig, &Held::default());
    let plan = r.get_plan(1, 0).expect("an axis");
    assert!(
        plan.shown && !plan.labeled && plan.offset == 10.0,
        "{plan:?}"
    );
}

/// Asserts the layout invariants of `fig` at `size` for `target`, its
/// panels in `cells`: at most four passes, no two kept tick labels' boxes
/// overlap, attached panels share pixel ranges bit for bit.
pub fn assert_layout(fig: &Figure<'_>, size: Size, target: Target, cells: &[(usize, usize)]) {
    let r = fig.resolve(size, &mut cache(), &Held::default(), target);
    assert!(r.get_passes() <= 4, "{} passes", r.get_passes());
    let boxes = r.label_boxes();
    for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            let overlap = a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1;
            assert!(!overlap, "{a:?} overlaps {b:?}");
        }
    }
    assert_attached(r.get_plot_areas(), cells);
}

/// The texts of the tick labels drawn by axis `axis` of panel `panel`.
fn drawn(fig: &Figure<'_>, size: Size, panel: u64, axis: u64) -> Vec<String> {
    drawn_with(fig, size, panel, axis, &mut cache())
}

/// [`drawn`] through `cache`.
fn drawn_with(
    fig: &Figure<'_>,
    size: Size,
    panel: u64,
    axis: u64,
    cache: &mut rustd3_figure::cache::Cache,
) -> Vec<String> {
    let list = fig.draw_with(cache, size, Target::Vector, &Held::default());
    texts(&list, panel, axis)
}

/// The texts of the glyph runs of axis `axis` of panel `panel` in `list`.
fn texts(list: &List, panel: u64, axis: u64) -> Vec<String> {
    let id = part::Id::new(4 << 56 | panel << 32 | axis);
    let Some(part) = list.parts.iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    list.commands[part.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.text.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// The baselines of the tick labels axis `axis` of panel `panel` draws in
/// `list`: each run's origin.
fn baselines(list: &List, panel: u64, axis: u64) -> Vec<f64> {
    let id = part::Id::new(4 << 56 | panel << 32 | axis);
    let part = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .expect("an axis part");
    list.commands[part.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.transform.translation().y)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn tick_labels_of_a_row_share_a_baseline_on_the_text_face_line_box() {
    let mut fig = Figure::new();
    let greek = fig.scale(scale::band(["alpha", "beta", "gamma"]));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = fig.scale(scale::band(months));
    fig.panel(0, 0)
        .axis(axis::bottom(greek))
        .axis(axis::top(greek));
    fig.panel(1, 0)
        .axis(axis::bottom(month))
        .axis(axis::top(month));
    let size = Size::new(960.0, 720.0);
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    let areas = resolve_at(&fig, size).get_plot_areas().to_vec();
    let [ascender, descender] = line_box(44.0 / 3.0);
    let gap = 8.0 + 14.0 / 3.0;
    for (panel, (area, count)) in areas.iter().zip([3, 12]).enumerate() {
        let bottom = baselines(&list, panel as u64, 0);
        let top = baselines(&list, panel as u64, 1);
        assert_eq!((bottom.len(), top.len()), (count, count));
        for y in bottom {
            assert!((y - (area.y1 + gap + ascender)).abs() < 1e-9, "{y}");
        }
        for y in top {
            assert!((y - (area.y0 - gap - descender)).abs() < 1e-9, "{y}");
        }
    }
}

#[test]
fn multi_line_labels_keep_their_first_baseline_below_and_their_last_above() {
    let mut fig = Figure::new();
    let keys = fig.scale(scale::band(["one", "two\nlines"]));
    fig.panel(0, 0)
        .axis(axis::bottom(keys))
        .axis(axis::top(keys));
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    // Runs: "one", then the two lines of "two\nlines".
    let bottom = baselines(&list, 0, 0);
    let top = baselines(&list, 0, 1);
    assert_eq!((bottom.len(), top.len()), (3, 3));
    assert!(bottom[2] > bottom[1] + 10.0, "{bottom:?}");
    assert!((bottom[0] - bottom[1]).abs() < 1e-9, "{bottom:?}");
    assert!((top[0] - top[2]).abs() < 1e-9, "{top:?}");
}

/// `fig` resolved at `size` for vector output.
fn resolve_at(fig: &Figure<'_>, size: Size) -> rustd3_figure::Resolved {
    fig.resolve(size, &mut cache(), &Held::default(), Target::Vector)
}

/// The size, 480 px tall, at which panel 0 of `fig` is `width` px wide.
fn at_width(fig: &Figure<'_>, width: f64) -> Size {
    let mut size = Size::new(width + 60.0, 480.0);
    for _ in 0..20 {
        let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        let w = r.get_plot_areas()[0].width();
        if (w - width).abs() < 1e-9 {
            return size;
        }
        size.width += width - w;
    }
    panic!("no size gives a plot area {width} px wide");
}

/// Twenty speed bins "0–10" … "190–200" on a band scale with padding 0.1.
fn bins() -> Vec<String> {
    (0..20)
        .map(|i| format!("{}–{}", 10 * i, 10 * i + 10))
        .collect()
}

#[test]
fn twenty_band_labels_keep_every_kth_at_four_widths() {
    let widths: Vec<String> = ["0–10", "10–20", "90–100", "100–110"]
        .iter()
        .map(|l| format!("{:.3}", measure(l, 44.0 / 3.0).0))
        .collect();
    assert_eq!(widths, ["35.328", "44.659", "53.990", "63.322"]);
    let mut fig = Figure::new();
    let speed = fig.scale(scale::band(bins()).padding(0.1));
    fig.panel(0, 0).axis(axis::bottom(speed));
    let rows = [
        (320.0, vec![0, 4, 8, 12, 16]),
        (480.0, vec![0, 3, 6, 9, 12, 15, 18]),
        (960.0, (0..20).step_by(2).collect()),
        (1400.0, (0..20).collect()),
    ];
    for (width, kept) in rows {
        let size = at_width(&fig, width);
        let expected: Vec<String> = kept.iter().map(|&i| bins()[i].clone()).collect();
        assert_eq!(drawn(&fig, size, 0, 0), expected, "at {width} px");
        assert_layout(&fig, size, Target::Vector, &[(0, 0)]);
    }
}

/// A bottom axis over `domain`, fixed, with 20 ticks asked.
fn every_five(domain: [f64; 2]) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain(domain));
    fig.panel(0, 0).axis(axis::bottom(x).ticks(20));
    fig
}

#[test]
fn the_stride_counts_ticks_from_zero_so_a_pan_keeps_the_same_labels() {
    let fig = every_five([0.0, 100.0]);
    let size = at_width(&fig, 200.0);
    let kept = ["0", "15", "30", "45", "60", "75", "90"];
    assert_eq!(drawn(&fig, size, 0, 0), kept);
    let panned = every_five([2.5, 102.5]);
    let size = at_width(&panned, 200.0);
    assert_eq!(drawn(&panned, size, 0, 0), kept[1..]);
}

#[test]
fn kept_labels_only_change_at_the_ends_while_an_axis_pans() {
    // 100 units over about 400 px, ticks every 5 (about 20 px): three-digit
    // labels (28 px) keep every second tick for any width from 280 to 560.
    let size = Size::new(440.0, 200.0);
    let mut cache = cache();
    let mut seed = 7_u32;
    let mut start = 100.0;
    let mut last: Option<Vec<f64>> = None;
    for _ in 0..200 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let px = f64::from(1 + (seed >> 8) % 50);
        let step = if seed >> 31 == 0 { px } else { -px } / 4.0;
        start = if (100.0..=850.0).contains(&(start + step)) {
            start + step
        } else {
            start - step
        };
        let fig = every_five([start, start + 100.0]);
        let width = fig
            .resolve(size, &mut cache, &Held::default(), Target::Vector)
            .get_plot_areas()[0]
            .width();
        assert!((300.0..=540.0).contains(&width), "{width}");
        let kept: Vec<f64> = drawn_with(&fig, size, 0, 0, &mut cache)
            .iter()
            .map(|t| t.parse().expect("a number"))
            .collect();
        assert!(kept.iter().all(|v| v % 10.0 == 0.0), "{start}: {kept:?}");
        if let Some(last) = &last {
            let within = |values: &[f64], of: &[f64]| -> Vec<f64> {
                let (low, high) = (of[0], of[of.len() - 1]);
                values
                    .iter()
                    .copied()
                    .filter(|v| (low..=high).contains(v))
                    .collect()
            };
            assert_eq!(within(last, &kept), within(&kept, last), "at {start}");
        }
        last = Some(kept);
    }
}

/// Two stacked panels, rows weighted 1.0 and 0.3 with `gap` between them,
/// y fixed to [0, 3] on top and [0, 100] below, both on left axes.
fn stacked_rows(gap: f64) -> Figure<'static> {
    let mut fig = Figure::new();
    fig.layout(Grid::new().rows([1.0, 0.3]).row_gaps([px(gap)]));
    let top = fig.scale(scale::linear().domain([0.0, 3.0]));
    let bottom = fig.scale(scale::linear().domain([0.0, 100.0]));
    fig.panel(0, 0).axis(axis::left(top));
    fig.panel(1, 0).axis(axis::left(bottom));
    fig
}

#[test]
fn end_labels_of_stacked_panels_keep_the_gap_their_overhangs_reserve() {
    let size = Size::new(640.0, 480.0);
    let fig = stacked_rows(4.0);
    let top = drawn(&fig, size, 0, 0);
    let bottom = drawn(&fig, size, 1, 0);
    assert_eq!(
        (top[0].as_str(), bottom.last().map(String::as_str)),
        ("0.0", Some("100"))
    );
    let r = resolve(&fig, &Held::default());
    let boxes = r.label_boxes();
    let zero = boxes[0];
    let hundred = boxes[boxes.len() - 1];
    assert!(
        (hundred.y0 - zero.y1 - 4.0).abs() < 1e-9,
        "{zero:?} {hundred:?}"
    );
    assert_layout(&fig, size, Target::Vector, &[(0, 0), (1, 0)]);
}

#[test]
fn where_end_labels_of_two_panels_overlap_the_larger_panel_keeps_its_own() {
    let size = Size::new(640.0, 480.0);
    let fig = stacked_rows(-12.0);
    let top = drawn(&fig, size, 0, 0);
    let bottom = drawn(&fig, size, 1, 0);
    assert_eq!(top[0], "0.0");
    assert_eq!(bottom, ["0", "50"], "the bottom panel's 100 gives way");
    assert_layout(&fig, size, Target::Vector, &[(0, 0), (1, 0)]);
}

#[test]
fn layout_invariants_hold_on_the_test_figures_and_the_examples() {
    let size = Size::new(640.0, 480.0);
    let (fig, ..) = one_panel(|p, x, y| {
        p.axis(axis::left(y)).axis(axis::bottom(x));
    });
    assert_layout(&fig, size, Target::Vector, &[(0, 0)]);
    let cells: Vec<(usize, usize)> = (0..4).flat_map(|r| (0..4).map(move |c| (r, c))).collect();
    for target in [Target::Vector, Target::Screen { scale: 2.0 }] {
        assert_layout(&grid(4, 4), size, target, &cells);
        assert_layout(&shared(4), size, target, &cells);
    }
    let scatter = crate::scatter::scatter(crate::scatter::POINTS);
    assert_layout(&scatter, crate::scatter::SIZE, Target::Vector, &[(0, 0)]);
    let rec = crate::trace::recording(crate::trace::SAMPLES);
    let trace = crate::trace::trace(&rec);
    assert_layout(
        &trace,
        crate::trace::SIZE,
        Target::Vector,
        &[(0, 0), (1, 0)],
    );
}

/// A 2 × 2 grid with panels in (0, 0) and (0, 1) and one across row 1, all
/// with a bottom axis on one x handle; the spanning panel has a left axis
/// with wide labels.
fn spanning() -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let wide = fig.fit(scale::linear());
    for c in 0..2 {
        fig.panel(0, c)
            .mark(line(x, [0.0, 10.0]))
            .mark(line(y, [0.0, 3.0]))
            .axis(axis::left(y))
            .axis(axis::bottom(x));
    }
    fig.panel_at(Cell::new(1, 0).span(1, 2))
        .mark(line(x, [0.0, 10.0]))
        .mark(line(wide, [0.0, 300_000.0]))
        .axis(axis::left(wide))
        .axis(axis::bottom(x));
    fig
}

#[test]
fn a_spanning_panel_runs_from_its_first_column_to_its_last() {
    let fig = spanning();
    let r = resolve(&fig, &Held::default());
    let areas = r.get_plot_areas();
    let (first, second, span) = (areas[0], areas[1], areas[2]);
    assert!(
        same(span.x0, first.x0) && same(span.x1, second.x1),
        "{areas:?}"
    );
    // Its wide labels count against column 0's left track: the panel
    // above starts where it does, beyond the narrow labels' reach.
    let narrow = r.get_plan(0, 0).expect("the top left panel's axis");
    let reach = r.get_plan(2, 0).expect("the spanning panel's axis");
    assert!(reach.thickness > narrow.thickness + 20.0, "{reach:?}");
    assert!((first.x0 - PAD - reach.thickness).abs() < 1e-9, "{first:?}");
    assert!(span.y0 > first.y1 && same(first.y0, second.y0));
}

#[test]
fn a_spanning_panel_below_makes_the_axes_above_it_inner() {
    let fig = spanning();
    let r = resolve(&fig, &Held::default());
    for panel in 0..2 {
        let bottom = r.get_plan(panel, 1).expect("a bottom axis");
        assert!(bottom.shown && !bottom.labeled, "{panel}: {bottom:?}");
    }
    let outer = r.get_plan(2, 1).expect("the spanning panel's bottom axis");
    assert!(outer.labeled);
}

#[test]
fn the_same_cell_and_span_give_the_same_panel() {
    let mut fig = spanning();
    fig.panel_at(Cell::new(1, 0).span(1, 2)).title("again");
    fig.panel(0, 1);
    assert_eq!(resolve(&fig, &Held::default()).get_plot_areas().len(), 3);
    fig.panel(1, 0);
    assert_eq!(resolve(&fig, &Held::default()).get_plot_areas().len(), 4);
}

/// One panel with a fixed `domain` on axis `side` asking 6 ticks, its
/// offset notation on or off.
fn notation(
    domain: [f64; 2],
    side: fn(ScaleRef<Linear>) -> Axis<'static, Linear>,
    on: bool,
) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain(domain));
    fig.panel(0, 0).axis(side(x).ticks(6).offset(on));
    fig
}

#[test]
fn far_from_zero_domains_label_ticks_from_an_offset_at_the_axis_end() {
    let size = Size::new(640.0, 480.0);
    let rows: [([f64; 2], &[&str]); 3] = [
        (
            [1e9, 1e9 + 1.0],
            &["0.0", "0.2", "0.4", "0.6", "0.8", "1.0", "+1e9"],
        ),
        (
            [12345.0, 12345.3],
            &[
                "0.00",
                "0.05",
                "0.10",
                "0.15",
                "0.20",
                "0.25",
                "0.30",
                "+1.2345e4",
            ],
        ),
        (
            [-1e9 - 1.0, -1e9],
            &["−1.0", "−0.8", "−0.6", "−0.4", "−0.2", "0.0", "−1e9"],
        ),
    ];
    for (domain, expected) in rows {
        let fig = notation(domain, axis::bottom, true);
        assert_eq!(drawn(&fig, size, 0, 0), expected, "{domain:?}");
        assert_layout(&fig, size, Target::Vector, &[(0, 0)]);
    }
    let fig = notation([1000.0, 1000.5], axis::bottom, true);
    let labels = drawn(&fig, size, 0, 0);
    assert_eq!(labels.first().map(String::as_str), Some("1000.0"));
    assert!(labels.iter().all(|l| !l.contains('e')), "{labels:?}");
}

#[test]
fn tiny_domains_label_ticks_in_units_of_a_power_of_ten() {
    let size = Size::new(640.0, 480.0);
    for side in [axis::bottom, axis::left] {
        let fig = notation([0.0, 3e-7], side, true);
        let expected = ["0.0", "0.5", "1.0", "1.5", "2.0", "2.5", "3.0", "1e−7"];
        assert_eq!(drawn(&fig, size, 0, 0), expected);
        assert_layout(&fig, size, Target::Vector, &[(0, 0)]);
    }
}

#[test]
fn offset_false_prints_full_labels() {
    let size = Size::new(640.0, 480.0);
    let fig = notation([1e9, 1e9 + 1.0], axis::bottom, false);
    let x = scale::linear().domain([1e9, 1e9 + 1.0]);
    let full = x.labels(&x.tick_set(6), &Locale::default());
    assert_eq!(full[1], "1\u{202f}000\u{202f}000\u{202f}000.2");
    // Labels this long collide: every second one is kept.
    let kept: Vec<String> = full.iter().step_by(2).cloned().collect();
    assert_eq!(drawn(&fig, size, 0, 0), kept);
}

#[test]
fn an_offset_label_takes_a_row_under_a_bottom_axis_and_the_top_above_a_left_one() {
    let size = Size::new(640.0, 480.0);
    let ink = ink_height("+1e9", 44.0 / 3.0);
    let plan = |fig: &Figure<'_>| {
        resolve(fig, &Held::default())
            .get_plan(0, 0)
            .expect("an axis")
    };
    let on = plan(&notation([1e9, 1e9 + 1.0], axis::bottom, true));
    let off = plan(&notation([0.0, 1.0], axis::bottom, true));
    assert!((on.thickness - off.thickness - (14.0 / 3.0 + ink)).abs() < 1e-9);
    let area = |fig: &Figure<'_>| resolve(fig, &Held::default()).get_plot_areas()[0];
    let on = area(&notation([1e9, 1e9 + 1.0], axis::left, true));
    let off = area(&notation([0.0, 1.0], axis::left, true));
    // "1.0" overhangs the top by half its line box; "+1e9" sits a tick pad
    // above.
    let [ascender, descender] = line_box(44.0 / 3.0);
    let half = (ascender + descender) / 2.0;
    assert!(
        (on.y0 - off.y0 - (14.0 / 3.0 + ink)).abs() < 1e-9,
        "{on:?} {off:?}"
    );
    assert!((off.y0 - PAD - half).abs() < 1e-9, "{off:?}");
    assert_layout(
        &notation([1e9, 1e9 + 1.0], axis::left, true),
        size,
        Target::Vector,
        &[(0, 0)],
    );
}

#[test]
fn log_axes_keep_their_labels_whatever_their_magnitude() {
    let size = Size::new(640.0, 480.0);
    let mut fig = Figure::new();
    let y = fig.scale(scale::log().domain([1.0, 1e7]));
    fig.panel(0, 0).axis(axis::left(y));
    let r = resolve(&fig, &Held::default());
    let count = r.get_plan(0, 0).expect("an axis").count;
    let log = scale::log().domain([1.0, 1e7]);
    let own = log.labels(&log.tick_set(count), &Locale::default());
    let kept = drawn(&fig, size, 0, 0);
    assert!(kept.iter().all(|l| own.contains(l)), "{kept:?} of {own:?}");
    assert!(kept.contains(&"10M".to_string()), "{kept:?}");
}

/// One 640 × 480 panel with a left axis over `y`, fixed to [0, 3].
fn held_left() -> (Figure<'static>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 3.0]));
    fig.panel(0, 0).axis(axis::left(y));
    (fig, y)
}

#[test]
fn held_areas_keep_the_plot_while_labels_grow_outward() {
    let size = Size::new(640.0, 480.0);
    let (fig, y) = held_left();
    let mut cache = cache();
    let start = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    assert!(!start.get_held());
    let mut held = Held::default();
    held.hold(y, scale::linear().domain([0.0, 30_000.0]));
    let areas = start.get_areas();
    let during = fig.resolve_held(size, &mut cache, &held, Target::Vector, &areas);
    assert!(during.get_held() && during.get_passes() == 1);
    let (a, b) = (start.get_plot_areas()[0], during.get_plot_areas()[0]);
    assert!(
        [(a.x0, b.x0), (a.y0, b.y0), (a.x1, b.x1), (a.y1, b.y1)]
            .iter()
            .all(|(p, q)| same(*p, *q)),
        "{a:?} {b:?}"
    );
    let grown = during.get_plan(0, 0).expect("an axis").thickness
        - start.get_plan(0, 0).expect("an axis").thickness;
    let widths = measure("30\u{202f}000", 44.0 / 3.0).0 - measure("3.0", 44.0 / 3.0).0;
    assert!((grown - widths).abs() < 1e-9, "{grown} {widths}");
    // Released, the layout is fresh and the plot area narrower by as much.
    let fresh = fig.resolve(size, &mut cache, &held, Target::Vector);
    assert!((fresh.get_plot_areas()[0].x0 - a.x0 - grown).abs() < 1e-9);
    let list = fig.draw_held(&mut cache, size, Target::Vector, &held, &areas);
    let labels = texts(&list, 0, 0);
    assert!(labels.contains(&"30\u{202f}000".to_string()), "{labels:?}");
}

#[test]
fn areas_for_another_size_or_panel_count_are_ignored() {
    fn data<T: Clone + std::fmt::Debug + PartialEq>(_: &T) {}
    let (fig, _) = held_left();
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let areas = fig
        .resolve(size, &mut cache, &Held::default(), Target::Vector)
        .get_areas();
    data(&areas);
    assert_eq!(areas.clone(), areas);
    let other = Size::new(800.0, 480.0);
    let r = fig.resolve_held(other, &mut cache, &Held::default(), Target::Vector, &areas);
    assert!(!r.get_held());
    let fresh = fig.resolve(other, &mut cache, &Held::default(), Target::Vector);
    assert_eq!(r.get_plot_areas(), fresh.get_plot_areas());
    let (mut more, _) = held_left();
    more.panel(0, 1);
    let r = more.resolve_held(size, &mut cache, &Held::default(), Target::Vector, &areas);
    assert!(!r.get_held());
}

#[test]
fn a_labelled_vertical_axis_on_screen_reserves_two_characters_of_slack() {
    let zero = measure("0", 44.0 / 3.0).0;
    assert!((zero - 9.3315).abs() < 1e-4, "{zero}");
    let size = Size::new(640.0, 480.0);
    let (fig, _) = held_left();
    let thickness = |target| {
        fig.resolve(size, &mut cache(), &Held::default(), target)
            .get_plan(0, 0)
            .expect("an axis")
            .thickness
    };
    let slack = thickness(Target::Screen { scale: 1.0 }) - thickness(Target::Vector);
    assert_eq!(format!("{slack:.3}"), "18.663");
    assert_eq!(
        thickness(Target::File { scale: 2.0 }),
        thickness(Target::Vector)
    );
    let (fig, ..) = one_panel(|p, x, _| {
        p.axis(axis::bottom(x));
    });
    let bottom = |target| {
        fig.resolve(size, &mut cache(), &Held::default(), target)
            .get_plan(0, 0)
            .expect("an axis")
            .thickness
    };
    assert_eq!(
        bottom(Target::Screen { scale: 1.0 }),
        bottom(Target::Vector)
    );
}

#[test]
fn the_axis_room_lays_every_target_out_alike() {
    let size = Size::new(640.0, 480.0);
    // Each target snaps to its own pixels: the left edges, within one.
    let lefts = |room: Option<f64>| -> Vec<f64> {
        let (mut fig, _) = held_left();
        if let Some(room) = room {
            fig.axis_room(room);
        }
        [
            Target::Screen { scale: 1.0 },
            Target::File { scale: 2.0 },
            Target::Vector,
        ]
        .into_iter()
        .map(|target| {
            let r = fig.resolve(size, &mut cache(), &Held::default(), target);
            r.get_plot_areas()[0].x0
        })
        .collect()
    };
    let near = |a: &[f64], b: f64| a.iter().all(|x| (x - b).abs() < 1.0);
    let by_target = lefts(None);
    assert!(by_target[0] - by_target[1] > 18.0, "{by_target:?}");
    assert!(near(&by_target[1..], by_target[2]), "{by_target:?}");
    assert!(near(&lefts(Some(2.0)), by_target[0]));
    assert!(near(&lefts(Some(0.0)), by_target[2]));
    assert_eq!(lefts(Some(f64::NAN)), by_target);
    assert_eq!(lefts(Some(-1.0)), by_target);
}

#[test]
fn labels_that_grow_past_held_reaches_give_way_to_a_larger_panels() {
    let size = Size::new(640.0, 480.0);
    let mut fig = Figure::new();
    fig.layout(Grid::new().cols([2.0, 1.0]).col_gaps([px(4.0)]));
    let left = fig.scale(scale::linear().domain([0.0, 3.0]));
    let right = fig.scale(scale::linear().domain([0.0, 100.0]));
    fig.panel(0, 0).axis(axis::bottom(left));
    fig.panel(0, 1).axis(axis::bottom(right));
    let mut cache = cache();
    let start = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let mut held = Held::default();
    held.hold(right, scale::linear().domain([-100_000.0, 0.0]));
    let areas = start.get_areas();
    let r = fig.resolve_held(size, &mut cache, &held, Target::Vector, &areas);
    let boxes = r.label_boxes();
    for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            assert!(!(a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1));
        }
    }
    let list = fig.draw_held(&mut cache, size, Target::Vector, &held, &areas);
    assert_eq!(texts(&list, 0, 0).last().map(String::as_str), Some("3.0"));
    let right = texts(&list, 1, 0);
    assert!(
        !right.is_empty() && !right.contains(&"−100\u{202f}000".to_string()),
        "{right:?}"
    );
}

/// A 2 × 2 grid of scales: x linear over [0, 12 000] per column on bottom
/// axes, y log over [1, 1e5] in the top row and symlog over [−100, 100] in
/// the bottom row on left axes, a band of months on top, right twin axes
/// with titles (one far from zero, one tiny), a figure title and panel
/// titles.
pub fn scales() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.title("Layout passes on a grid of scales");
    let xs: Vec<_> = (0..2)
        .map(|_| fig.scale(scale::linear().domain([0.0, 12_000.0])))
        .collect();
    let log = fig.scale(scale::log().domain([1.0, 1e5]));
    let symlog = fig.scale(scale::symlog().domain([-100.0, 100.0]));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = fig.scale(scale::band(months).padding(0.1));
    let epoch = fig.scale(scale::linear().domain([1e9, 1e9 + 5.0]));
    let current = fig.scale(scale::linear().domain([0.0, 2.5e-6]));
    let names = ["A", "B", "C", "D"];
    for r in 0..2 {
        for (c, &x) in xs.iter().enumerate() {
            let panel = fig.panel(r, c);
            panel
                .title(format!("Panel {}", names[2 * r + c]))
                .axis(axis::bottom(x).label("distance [m]"));
            if r == 0 {
                panel.axis(axis::left(log)).axis(axis::top(month));
            } else {
                panel.axis(axis::left(symlog));
            }
            if c == 1 && r == 0 {
                panel.axis(axis::right(epoch).label("time [s]"));
            }
            if c == 1 && r == 1 {
                panel.axis(axis::right(current).label("current [A]"));
            }
        }
    }
    fig
}

#[test]
fn list_dump_of_a_grid_of_scales() {
    let fig = scales();
    let size = Size::new(960.0, 720.0);
    let cells = [(0, 0), (0, 1), (1, 0), (1, 1)];
    for target in [
        Target::Vector,
        Target::File { scale: 2.0 },
        Target::Screen { scale: 2.0 },
    ] {
        assert_layout(&fig, size, target, &cells);
    }
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    rustd3_figure::list::check(&list).expect("a valid list");
    let dump = rustd3_figure::list::dump(
        &list,
        rustd3_figure::list::dump::Options::default().glyphs(false),
    );
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("list_dump_of_a_grid_of_scales.txt", &dump);
}

/// A 2 × 2 grid with linear and log axes, a right twin, borders and rules
/// at awkward values.
fn ruled() -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 12_000.0]));
    let y = fig.scale(scale::log().domain([1.0, 1e5]));
    let twin = fig.scale(scale::linear().domain([0.0, 2.5e-6]));
    fig.title("Snapped");
    for r in 0..2 {
        for c in 0..2 {
            let panel = fig.panel(r, c);
            panel
                .border(Rgb::hex(0x4d4d4d))
                .axis(axis::left(y))
                .axis(axis::bottom(x).label("distance [m]"))
                .mark(rustd3_figure::mark::rule_y(333.3, y).dash([4.0, 3.0]))
                .mark(rustd3_figure::mark::rule_x(4_321.0, x).width(1.0));
            if c == 1 {
                panel.axis(axis::right(twin).label("current [A]"));
            }
        }
    }
    fig
}

/// Whether `v` px is a whole number of device px at `scale`.
fn whole(v: f64, scale: f64) -> bool {
    (v * scale - (v * scale).round()).abs() < 1e-9
}

#[test]
fn raster_targets_put_plot_area_edges_and_hairlines_on_device_pixels() {
    let fig = ruled();
    let size = Size::new(960.0, 720.0);
    let vector = fig
        .resolve(size, &mut cache(), &Held::default(), Target::Vector)
        .get_plot_areas()
        .to_vec();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let target = Target::File { scale };
        let r = fig.resolve(size, &mut cache(), &Held::default(), target);
        let areas = r.get_plot_areas();
        for (a, v) in areas.iter().zip(&vector) {
            for (edge, raw) in [(a.x0, v.x0), (a.y0, v.y0), (a.x1, v.x1), (a.y1, v.y1)] {
                assert!(whole(edge, scale), "{edge} at {scale}");
                assert_eq!(edge, rustd3_figure::raster::snap::edge(raw, scale));
            }
        }
        assert_layout(&fig, size, target, &[(0, 0), (0, 1), (1, 0), (1, 1)]);
        let list = fig.draw_with(&mut cache(), size, target, &Held::default());
        let mut hairlines = 0;
        for part in &list.parts {
            // Borders, axes, their lines and the rules among the marks.
            let role = part.id >= part::Id::new(3 << 56) && part.id < part::Id::new(7 << 56)
                || part.id >= part::Id::new(8 << 56) && part.id < part::Id::new(9 << 56);
            if !role {
                continue;
            }
            for command in &list.commands[part.commands.clone()] {
                let Command::Stroke { path, style, .. } = command else {
                    continue;
                };
                assert!(whole(style.width, scale) && style.width * scale >= 1.0);
                let half = style.width / 2.0;
                let mut from = None;
                for e in path.elements() {
                    match *e {
                        rustd3_figure::kurbo::PathEl::MoveTo(p) => from = Some(p),
                        rustd3_figure::kurbo::PathEl::LineTo(p) => {
                            let a = from.expect("a segment starts with a move");
                            let across = if a.x == p.x {
                                Some(p.x)
                            } else if a.y == p.y {
                                Some(p.y)
                            } else {
                                None
                            };
                            let c = across.expect("hairlines are axis-aligned");
                            assert!(
                                whole(c - half, scale) && whole(c + half, scale),
                                "{c} ± {half} at {scale}"
                            );
                            hairlines += 1;
                            from = Some(p);
                        }
                        rustd3_figure::kurbo::PathEl::ClosePath => {}
                        rustd3_figure::kurbo::PathEl::QuadTo(..)
                        | rustd3_figure::kurbo::PathEl::CurveTo(..) => {
                            panic!("a curve among hairlines")
                        }
                    }
                }
            }
        }
        assert!(hairlines > 100, "{hairlines} hairline segments");
        if scale == 1.25 {
            // 1.25 pt = 1.667 px = 2.083 device px: 2.
            let axis = list
                .parts
                .iter()
                .find(|p| p.id == part::Id::new(8 << 56))
                .expect("an axis line");
            let Command::Stroke { style, .. } = &list.commands[axis.commands.start] else {
                panic!("the axis line");
            };
            assert!((style.width * 1.25 - 2.0).abs() < 1e-9, "{}", style.width);
        }
    }
}

#[test]
fn vector_output_keeps_plot_areas_unsnapped() {
    let fig = ruled();
    let size = Size::new(960.0, 720.0);
    let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    let edges: Vec<f64> = r
        .get_plot_areas()
        .iter()
        .flat_map(|a| [a.x0, a.y0, a.x1, a.y1])
        .collect();
    assert!(edges.iter().any(|e| !whole(*e, 1.0)), "{edges:?}");
}
