//! Layout by protrusions: tracks, padding, the figure title's reserve, and
//! attached panels sharing pixel ranges exactly.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Linear, Ticks};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::layout::{Cell, Grid, em, px};
use rustd3_figure::list::{Command, part};
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
        assert!(r.passes() <= 4, "{n}×{n}: {}", r.passes());
        for (i, (row, col)) in (0..n).flat_map(|r| (0..n).map(move |c| (r, c))).enumerate() {
            let left = r.plan(i, 0).expect("a left axis");
            let bottom = r.plan(i, 1).expect("a bottom axis");
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
    let left = r.plan(0, 0).expect("the left panel's axis");
    let right = r.plan(1, 0).expect("the right panel's axis");
    assert!(left.shown && left.labeled, "{left:?}");
    assert!(right.shown && !right.labeled, "{right:?}");
    assert_eq!(right.thickness, 8.0, "ticks alone");
    assert_eq!(r.plan(1, 1), None);
}

#[test]
fn a_second_axis_on_one_side_stacks_8_px_past_the_first() {
    let (fig, ..) = one_panel(|p, _, y| {
        p.axis(axis::left(y).label("vib"))
            .axis(axis::left(y).label("temp"));
    });
    let r = resolve(&fig, &Held::default());
    let vib = r.plan(0, 0).expect("vib");
    let temp = r.plan(0, 1).expect("temp");
    assert_eq!(vib.offset, 0.0);
    assert_eq!(temp.offset, vib.thickness + 8.0);
    let area = r.plot_areas()[0];
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
        .map(|axis| r.plan(1, axis).expect("an axis"))
        .collect();
    assert!(plans[0].shown && !plans[0].labeled, "{plans:?}");
    assert!(!plans[1].shown, "{plans:?}");
    let outer = r.plan(0, 1).expect("the left panel's temp axis");
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
fn labels_true_labels_an_inner_axis() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).labels(true));
    });
    let r = resolve(&fig, &Held::default());
    let inner = r.plan(1, 0).expect("the right panel's axis");
    assert!(inner.shown && inner.labeled, "{inner:?}");
}

#[test]
fn an_explicit_outward_axis_on_an_inner_panel_is_not_drawn_and_labels_false_draws_it() {
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).outward(10.0));
    });
    let r = resolve(&fig, &Held::default());
    assert!(!r.plan(1, 0).expect("an axis").shown);
    let fig = one_by_two(|p, y, _| {
        p.axis(axis::left(y).outward(10.0).labels(false));
    });
    let r = resolve(&fig, &Held::default());
    let plan = r.plan(1, 0).expect("an axis");
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
    assert!(r.passes() <= 4, "{} passes", r.passes());
    let boxes = r.label_boxes();
    for (i, a) in boxes.iter().enumerate() {
        for b in &boxes[i + 1..] {
            let overlap = a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1;
            assert!(!overlap, "{a:?} overlaps {b:?}");
        }
    }
    assert_attached(r.plot_areas(), cells);
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
    cache: &mut rustd3_figure::Cache,
) -> Vec<String> {
    let list = fig.draw_with(cache, size, Target::Vector, &Held::default());
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

/// The size, 480 px tall, at which panel 0 of `fig` is `width` px wide.
fn at_width(fig: &Figure<'_>, width: f64) -> Size {
    let mut size = Size::new(width + 60.0, 480.0);
    for _ in 0..20 {
        let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        let w = r.plot_areas()[0].width();
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
            .plot_areas()[0]
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
    let areas = r.plot_areas();
    let (first, second, span) = (areas[0], areas[1], areas[2]);
    assert!(
        same(span.x0, first.x0) && same(span.x1, second.x1),
        "{areas:?}"
    );
    // Its wide labels count against column 0's left track: the panel
    // above starts where it does, beyond the narrow labels' reach.
    let narrow = r.plan(0, 0).expect("the top left panel's axis");
    let reach = r.plan(2, 0).expect("the spanning panel's axis");
    assert!(reach.thickness > narrow.thickness + 20.0, "{reach:?}");
    assert!((first.x0 - PAD - reach.thickness).abs() < 1e-9, "{first:?}");
    assert!(span.y0 > first.y1 && same(first.y0, second.y0));
}

#[test]
fn a_spanning_panel_below_makes_the_axes_above_it_inner() {
    let fig = spanning();
    let r = resolve(&fig, &Held::default());
    for panel in 0..2 {
        let bottom = r.plan(panel, 1).expect("a bottom axis");
        assert!(bottom.shown && !bottom.labeled, "{panel}: {bottom:?}");
    }
    let outer = r.plan(2, 1).expect("the spanning panel's bottom axis");
    assert!(outer.labeled);
}

#[test]
fn the_same_cell_and_span_give_the_same_panel() {
    let mut fig = spanning();
    fig.panel_at(Cell::new(1, 0).span(1, 2)).title("again");
    fig.panel(0, 1);
    assert_eq!(resolve(&fig, &Held::default()).plot_areas().len(), 3);
    fig.panel(1, 0);
    assert_eq!(resolve(&fig, &Held::default()).plot_areas().len(), 4);
}
