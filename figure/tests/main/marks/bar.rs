//! Bars, span bars and rects: held and free edges, placement across bands
//! and continuous keys, dodging, and one rectangle per row.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{Point, Rect, Shape, Size};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{bar_x, bar_x_spans, bar_y, bar_y_spans, dots, rect};
use rustd3_figure::{Figure, Held, Target};

use super::{draw, fitted, marked, role, solid};
use crate::figure::{cache, resolve};

/// `a` and `b` within 1e-9.
fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

/// The rectangles a list's marks fill, in order.
fn filled(list: &List) -> Vec<Rect> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Fill { path, .. } = c {
                Some(path.bounding_box())
            } else {
                None
            }
        })
        .collect()
}

/// A band x over "a" and "b" and a fitted linear y.
fn banded() -> (
    Figure<'static>,
    rustd3_figure::scale::ScaleRef<scale::Band<&'static str>>,
    rustd3_figure::scale::ScaleRef<scale::Linear>,
) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.fit(scale::linear());
    (fig, x, y)
}

#[test]
fn bars_from_a_base_are_held_at_zero_and_dots_are_not() {
    let (mut fig, x, y) = banded();
    fig.panel(0, 0).mark(bar_y([("a", 3.0)], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 3.15]), "{domain:?}");
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(dots([("a", 3.0)], x, y))
        .mark(dots([("b", -1.0)], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [-1.2, 3.2]), "{domain:?}");
}

#[test]
fn span_bars_away_from_zero_hold_neither_end_and_ignore_a_base() {
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(bar_y_spans([("a", [1.0, 3.0])], x, y).base(-5.0));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.9, 3.1]), "{domain:?}");
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(bar_y([("a", 3.0)], x, y).base(1.0).base(f64::INFINITY));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [1.0, 3.1]), "{domain:?}");
}

#[test]
fn span_bars_hold_an_end_at_zero_so_stacks_stand_on_the_axis() {
    // Stacked from 0: the y margin stops at 0, matplotlib's sticky edge.
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(bar_y_spans([("a", [0.0, 3.0]), ("b", [0.0, 1.0])], x, y))
        .mark(bar_y_spans([("a", [3.0, 4.0]), ("b", [1.0, 2.0])], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 4.2]), "{domain:?}");
    // Diverging: 0 lies inside the data, so both margins stay.
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(bar_y_spans([("a", [0.0, 3.0]), ("b", [-2.0, 0.0])], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(y)
        .unwrap()
        .get_domain();
    assert!(close(domain, [-2.25, 3.25]), "{domain:?}");
    // Horizontal spans hold 0 along x.
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.scale(scale::band(["a"]));
    fig.panel(0, 0).mark(bar_x_spans([([0.0, 2.0], "a")], x, y));
    let domain = resolve(&fig, &Held::default())
        .scale(x)
        .unwrap()
        .get_domain();
    assert!(close(domain, [0.0, 2.1]), "{domain:?}");
}

#[test]
fn a_rect_fits_all_four_edges_and_fills_one_command_per_row_in_one_part() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(rect(
        [([0.0, 1.0], [0.0, 2.0]), ([1.0, 3.0], [1.0, 4.0])],
        x,
        y,
    ));
    let resolved = resolve(&fig, &Held::default());
    assert!(close(
        resolved.scale(x).unwrap().get_domain(),
        [-0.15, 3.15]
    ));
    assert!(close(resolved.scale(y).unwrap().get_domain(), [-0.2, 4.2]));
    let list = draw(&fig);
    let parts: Vec<_> = list.parts.iter().filter(|p| role(p.id) == 6).collect();
    assert_eq!(parts.len(), 1);
    let commands = &list.commands[parts[0].commands.clone()];
    assert_eq!(commands.len(), 2);
    assert!(commands.iter().all(|c| matches!(c, Command::Fill { .. })));
    let area = resolved.plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let px = |a: f64, b: f64| Point::new((a + 0.15) / 3.3 * w, (4.2 - b) / 4.4 * h);
    let first = Rect::from_points(px(0.0, 0.0), px(1.0, 2.0));
    let got = filled(&list)[0];
    for (a, b) in [
        (got.x0, first.x0),
        (got.y0, first.y0),
        (got.x1, first.x1),
        (got.y1, first.y1),
    ] {
        assert!((a - b).abs() < 1e-9, "{got:?} against {first:?}");
    }
}

#[test]
fn a_stroked_rect_is_outlined_and_filled_only_when_asked() {
    let (mut fig, x, y) = fitted();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(rect([([0.0, 1.0], [0.0, 1.0])], x, y).stroke(color, "b"))
        .mark(
            rect([([1.0, 2.0], [0.0, 1.0])], x, y)
                .fill(color, "a")
                .stroke(color, "b"),
        );
    let list = draw(&fig);
    let kinds: Vec<&str> = marked(&list)
        .into_iter()
        .map(|c| {
            if let Command::Fill { paint, .. } = c {
                assert_eq!(*paint, solid(COLORBLIND[0]));
                "fill"
            } else if let Command::Stroke { paint, style, .. } = c {
                assert_eq!(*paint, solid(COLORBLIND[1]));
                assert!((style.width - 4.0 / 3.0).abs() < 1e-12);
                "stroke"
            } else {
                panic!("{c:?}")
            }
        })
        .collect();
    assert_eq!(kinds, ["stroke", "fill", "stroke"]);
}

#[test]
fn bars_fill_their_band_and_dodged_bars_their_key_slot() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band([0, 1, 2, 3]).padding(0.1));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let color = fig.scale(scale::ordinal(["p", "q", "r"], COLORBLIND));
    fig.panel(0, 0)
        .mark(bar_y([(1, 5.0)], x, y))
        .mark(bar_y([(1, 5.0)], x, y).fill(color, "r").dodge());
    let list = draw(&fig);
    let area = resolve(&fig, &Held::default()).plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let step = w / (4.0 - 0.1 + 0.2);
    let (start, bandwidth) = (step * 0.1 + step, step * 0.9);
    let rects = filled(&list);
    let expected = [
        Rect::new(start, h / 2.0, start + bandwidth, h),
        Rect::new(start + bandwidth * 2.0 / 3.0, h / 2.0, start + bandwidth, h),
    ];
    for (got, want) in rects.iter().zip(expected) {
        let d = [
            got.x0 - want.x0,
            got.y0 - want.y0,
            got.x1 - want.x1,
            got.y1 - want.y1,
        ];
        assert!(d.iter().all(|d| d.abs() < 1e-9), "{got:?} against {want:?}");
    }
    assert_eq!(rects.len(), 2);
}

#[test]
fn bars_on_a_continuous_axis_take_0_8_of_the_smallest_gap_unless_given_a_width() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(bar_y([(1.0, 2.0), (2.0, 4.0), (4.0, 1.0)], x, y))
        .mark(bar_y([(6.0, 2.0)], x, y).width(12.0).width(-1.0))
        .mark(bar_y([(8.0, 2.0)], x, y));
    let list = draw(&fig);
    let w = resolve(&fig, &Held::default()).plot_areas()[0].width();
    let widths: Vec<f64> = filled(&list).iter().map(|r| r.width()).collect();
    let gap = 0.8 * w / 10.0;
    let expected = [gap, gap, gap, 12.0, 0.8 * w];
    assert!(
        widths
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() < 1e-9),
        "{widths:?} against {expected:?}"
    );
    let first = filled(&list)[0];
    assert!((first.center().x - 0.1 * w).abs() < 1e-9);
}

#[test]
fn horizontal_bars_mirror_vertical_ones() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.scale(scale::band(["a", "b"]));
    fig.panel(0, 0)
        .mark(bar_x([(4.0, "a"), (2.0, "b")], x, y))
        .mark(bar_x_spans([([1.0, 3.0], "b")], x, y));
    let resolved = resolve(&fig, &Held::default());
    assert!(close(resolved.scale(x).unwrap().get_domain(), [0.0, 4.2]));
    let area = resolved.plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let px = |v: f64| v / 4.2 * w;
    let rects = filled(&draw(&fig));
    // A band scale over a reversed range puts its first key at the bottom.
    let expected = [
        Rect::new(0.0, h / 2.0, px(4.0), h),
        Rect::new(0.0, 0.0, px(2.0), h / 2.0),
        Rect::new(px(1.0), 0.0, px(3.0), h / 2.0),
    ];
    for (got, want) in rects.iter().zip(expected) {
        let d = [
            got.x0 - want.x0,
            got.y0 - want.y0,
            got.x1 - want.x1,
            got.y1 - want.y1,
        ];
        assert!(d.iter().all(|d| d.abs() < 1e-9), "{got:?} against {want:?}");
    }
}

#[test]
fn bars_on_raster_targets_put_their_edges_on_device_pixels() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(bar_y([(0.3, 1.7), (1.1, 2.9), (2.6, 0.4)], x, y));
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Screen { scale: 1.25 },
        &Held::default(),
    );
    let rects = filled(&list);
    assert_eq!(rects.len(), 3);
    for r in rects {
        for v in [r.x0, r.y0, r.x1, r.y1] {
            assert!((v * 1.25 - (v * 1.25).round()).abs() < 1e-9, "{r:?}");
        }
    }
}

#[test]
fn bars_with_no_area_and_keys_outside_the_band_draw_nothing() {
    let (mut fig, x, y) = banded();
    fig.panel(0, 0)
        .mark(bar_y([("a", 0.0), ("z", 2.0), ("b", f64::NAN)], x, y));
    assert!(filled(&draw(&fig)).is_empty());
    #[cfg(feature = "fonts")]
    {
        use rustd3_figure::check::Issue;
        use rustd3_figure::kurbo::Size;
        let issues = fig.check(Size::new(640.0, 480.0));
        assert_eq!(
            issues,
            [Issue::OutsideDomain {
                handle: x.index(),
                count: 1
            }]
        );
    }
}
