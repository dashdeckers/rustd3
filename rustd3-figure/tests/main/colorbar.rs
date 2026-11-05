//! Colorbars: strips equal to their scales at every pixel, bars along
//! figure slots and in panel corners with the room they take from the grid,
//! ticks at the scales' own positions and labels in the figure's locale,
//! binned edges, end triangles and outlines.

use rustd3_core::color::Rgb;
use rustd3_core::locale::Locale;
#[cfg(feature = "raster")]
use rustd3_core::scale::Sequential;
use rustd3_core::scale::{self, Ticks, transform};
use rustd3_core::scheme;
use rustd3_figure::colorbar::{Extend, Ramp, colorbar};
use rustd3_figure::kurbo::{BezPath, PathEl, Rect, Shape, Size};
use rustd3_figure::legend::{Inset, Slot};
use rustd3_figure::list::{Command, GlyphRun, List, part};
use rustd3_figure::peniko::{Brush, ImageData};
use rustd3_figure::scale::{Paint, ScaleRef};
use rustd3_figure::theme::{Context, Style};
use rustd3_figure::{Figure, Held, Target, Theme, axis};

use crate::figure::{cache, line};

/// The tick-label size at notebook, px: one em.
const EM: f64 = 44.0 / 3.0;

/// The canvas padding, px.
const PAD: f64 = 0.8 * EM;

/// The tick length at notebook, px: 6 pt.
const TICK: f64 = 8.0;
/// The tick pad at notebook, px: 3.5 pt.
const TICK_PAD: f64 = 14.0 / 3.0;

/// The blues of d3's `schemeBlues[5]`.
fn blues5() -> [Rgb; 5] {
    ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"]
        .map(|c| c.parse::<Rgb>().expect("a color"))
}

/// A one-panel figure, x and y fitted to 0 … 10 with axes, and a color
/// handle of `color`.
fn panel<S: Paint<Domain = f64> + Ramp>(color: S) -> (Figure<'static>, ScaleRef<S>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let c = fig.scale(color);
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .mark(line(y, [0.0, 10.0]))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    (fig, c)
}

/// `fig` drawn at `size` for `target`, and its first panel's plot area.
fn drawn(fig: &Figure<'_>, size: Size, target: Target) -> (List, Rect) {
    let held = Held::default();
    let area = fig
        .resolve(size, &mut cache(), &held, target)
        .get_plot_areas()[0];
    (fig.draw_with(&mut cache(), size, target, &held), area)
}

/// `fig` drawn at 640 × 480 for vector output, and its first plot area.
fn vector(fig: &Figure<'_>) -> (List, Rect) {
    drawn(fig, Size::new(640.0, 480.0), Target::Vector)
}

/// The commands of the guide in figure slots added `index`th.
fn slotted(list: &List, index: u64) -> &[Command] {
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32 | index);
    let p = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .unwrap_or_else(|| panic!("no slot guide {index}"));
    &list.commands[p.commands.clone()]
}

/// The strips among `commands`: each `Image` command's rect and pixels.
fn strips(commands: &[Command]) -> Vec<(Rect, &ImageData)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Image {
                rect,
                image,
                smooth,
            } = c
            {
                assert!(!smooth, "strips are nearest-sampled");
                Some((*rect, image))
            } else {
                None
            }
        })
        .collect()
}

/// The one strip among `commands`.
fn strip(commands: &[Command]) -> Rect {
    let [(bar, _)] = strips(commands)[..] else {
        panic!("one strip");
    };
    bar
}

/// The glyph runs among `commands`.
fn runs(commands: &[Command]) -> Vec<&GlyphRun> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect()
}

/// The texts of the glyph runs among `commands`.
fn texts(commands: &[Command]) -> Vec<String> {
    runs(commands).iter().map(|r| r.text.to_string()).collect()
}

/// The strokes among `commands`: paths and widths.
fn strokes(commands: &[Command]) -> Vec<(&BezPath, f64)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { path, style, .. } = c {
                Some((path, style.width))
            } else {
                None
            }
        })
        .collect()
}

/// The fills among `commands`: paths and paints.
fn fills(commands: &[Command]) -> Vec<(&BezPath, &Brush)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Fill { path, paint } = c {
                Some((path, paint))
            } else {
                None
            }
        })
        .collect()
}

/// The segments of an open path of moves and lines: (from, to) pairs.
fn segments(path: &BezPath) -> Vec<[(f64, f64); 2]> {
    let mut out = Vec::new();
    let mut at = (0.0, 0.0);
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => at = (p.x, p.y),
            PathEl::LineTo(p) => {
                out.push([at, (p.x, p.y)]);
                at = (p.x, p.y);
            }
            PathEl::QuadTo(..) | PathEl::CurveTo(..) | PathEl::ClosePath => {
                panic!("a tick path has only moves and lines")
            }
        }
    }
    out
}

/// Whether `path` is closed: the outline, not ticks.
fn closed(path: &BezPath) -> bool {
    path.elements().contains(&PathEl::ClosePath)
}

/// The tick marks among `commands`: each tick's segment, majors first.
fn ticks(commands: &[Command]) -> Vec<[(f64, f64); 2]> {
    strokes(commands)
        .into_iter()
        .filter(|(path, _)| !closed(path))
        .flat_map(|(path, _)| segments(path))
        .collect()
}

/// Whether `a` and `b` agree to 1e-9 px.
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A run's left end and the right end of its advance, px.
fn span(run: &GlyphRun) -> [f64; 2] {
    let x = run.transform.translation().x;
    let end = run
        .glyphs
        .last()
        .map_or(0.0, |g| f64::from(g.x) + f64::from(g.advance));
    [x, x + end]
}

#[test]
fn a_right_colorbar_spans_the_plot_area_its_labels_outside_it() {
    let (bare, _) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    let (_, bare) = vector(&bare);
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.legend(Slot::Right, colorbar(heat));
    let (list, area) = vector(&fig);
    let cb = slotted(&list, 0);
    let bar = strip(cb);
    let [(_, image)] = strips(cb)[..] else {
        panic!("one strip");
    };
    assert_eq!((image.width, image.height), (1, 256));
    assert!(close(bar.y0, area.y0) && close(bar.y1, area.y1));
    assert!((bar.width() - EM).abs() < 1e-9);
    // Labels start a tick and a tick pad right of the bar; the widest ends
    // at the canvas padding, so the slot takes exactly what it holds.
    let ends: Vec<[f64; 2]> = runs(cb).iter().map(|r| span(r)).collect();
    assert!(
        ends.iter()
            .all(|[x0, _]| (x0 - (bar.x1 + TICK + TICK_PAD)).abs() < 1e-9)
    );
    let right = ends.iter().map(|[_, x1]| *x1).fold(0.0, f64::max);
    assert!((right - (640.0 - PAD)).abs() < 1e-6, "{right}");
    let reserve = bare.x1 - area.x1;
    assert!(
        (reserve - (0.5 * EM + right - bar.x0)).abs() < 1e-6,
        "{reserve}"
    );
}

#[test]
fn ticks_sit_at_the_scales_positions_and_point_out_of_the_bar() {
    let s = scale::sequential([0.0, 80.0], scheme::viridis);
    let (mut fig, heat) = panel(s.clone());
    fig.legend(Slot::Right, colorbar(heat));
    let (list, _) = vector(&fig);
    let cb = slotted(&list, 0);
    let bar = strip(cb);
    // As an axis as long: one tick per 35 px up.
    let count = (bar.height() / 35.0).round() as usize;
    let set = s.tick_set(count);
    assert_eq!(set.get_values().len(), 17);
    assert_eq!(texts(cb), s.labels(&set, &Locale::NEUTRAL));
    let marks = ticks(cb);
    assert_eq!(marks.len(), set.get_values().len());
    for ([(x0, y0), (x1, y1)], v) in marks.iter().zip(set.get_values()) {
        let y = bar.y1 - v / 80.0 * bar.height();
        assert!(close(*y0, y) && close(*y1, y), "{v}");
        assert!(close(*x0, bar.x1) && close(x1 - x0, TICK));
    }
}

#[test]
fn top_and_bottom_colorbars_lie_across_the_plot_areas() {
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::magma));
    fig.legend(Slot::Bottom, colorbar(heat))
        .legend(Slot::Top, colorbar(heat));
    let (list, area) = vector(&fig);
    let [bottom, top] = [0, 1].map(|k| slotted(&list, k));
    let [(low, across), (high, up)] = [strips(bottom)[0], strips(top)[0]];
    for image in [up, across] {
        assert_eq!((image.width, image.height), (256, 1));
    }
    for bar in [high, low] {
        assert!(close(bar.x0, area.x0) && close(bar.x1, area.x1));
    }
    assert!(high.y1 < area.y0 && area.y1 < low.y0);
    // Ticks point away from the grid: down below it, up above it.
    let [a, b] = ticks(bottom)[0];
    assert!((b.1 - a.1 - TICK).abs() < 1e-9 && (a.1 - low.y1).abs() < 1e-9);
    let [from, to] = ticks(top)[0];
    assert!((from.1 - to.1 - TICK).abs() < 1e-9 && (from.1 - high.y0).abs() < 1e-9);
}

#[test]
fn a_side_colorbar_spans_every_row_beside_it() {
    let mut fig = Figure::new();
    let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0);
    fig.panel(1, 0);
    fig.legend(Slot::Left, colorbar(heat).title("temp [°C]"));
    let size = Size::new(640.0, 480.0);
    let held = Held::default();
    let areas = fig
        .resolve(size, &mut cache(), &held, Target::Vector)
        .get_plot_areas()
        .to_vec();
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &held);
    let cb = slotted(&list, 0);
    let bar = strip(cb);
    assert!(close(bar.y0, areas[0].y0) && close(bar.y1, areas[1].y1));
    // Labels end a tick and a tick pad left of the bar; the title is turned
    // a quarter counter-clockwise past them, centered on the bar.
    let all = runs(cb);
    let (title, labels) = all.split_last().expect("labels and a title");
    assert_eq!(&*title.text, "temp [°C]");
    for run in labels {
        assert!((span(run)[1] - (bar.x0 - TICK - TICK_PAD)).abs() < 1e-6);
    }
    let t = title.transform.as_coeffs();
    assert!(t[1] < -0.99, "{t:?}");
    let leftmost = labels
        .iter()
        .map(|r| span(r)[0])
        .fold(f64::INFINITY, f64::min);
    assert!(t[4] < leftmost && t[4] > PAD);
}

#[test]
fn vertical_titles_read_bottom_to_top_on_either_side() {
    // A right axis's title and a right colorbar's, as matplotlib draws
    // every vertical label: a quarter turn counter-clockwise (the run's
    // x axis points up the page), past the labels it titles.
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0)
        .axis(axis::left(y).label("left"))
        .axis(axis::right(y).label("right"));
    fig.legend(Slot::Right, colorbar(heat).title("bar"));
    let (list, area) = vector(&fig);
    let all = runs(&list.commands);
    let title = |text: &str| {
        let run = all.iter().find(|r| &*r.text == text).expect("a title");
        run.transform.as_coeffs()
    };
    for text in ["left", "right", "bar"] {
        let t = title(text);
        assert!(t[1] < -0.99 && t[2] > 0.99, "{text}: {t:?}");
    }
    // The right axis's title lies right of its labels, its ascenders (its
    // ink above the baseline, left of the run's origin) toward them; the
    // left one's left of the plot area.
    let id = part::Id::new(4 << 56 | 1);
    let axis = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .expect("the right axis");
    let labels = runs(&list.commands[axis.commands.clone()]);
    let rightmost = labels
        .iter()
        .filter(|r| &*r.text != "right")
        .map(|r| span(r)[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let [right, left] = [title("right"), title("left")];
    let ascent = 0.6 * 16.0;
    assert!(right[4] - ascent > rightmost, "{right:?} {rightmost}");
    assert!(left[4] < area.x0, "{left:?}");
    let colorbar = runs(slotted(&list, 0));
    let bar = colorbar.last().expect("the bar's title");
    assert_eq!(&*bar.text, "bar");
    assert!(bar.transform.as_coeffs()[4] > right[4]);
}

#[test]
fn an_inset_colorbar_runs_along_the_longer_side_for_40_percent_of_it() {
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0).legend(Inset::TopRight, colorbar(heat));
    let (list, area) = vector(&fig);
    let bar = strip(&list.commands);
    // Wider than tall: a horizontal bar, 40% of the width, ticks below,
    // framed 0.4 em inside and 0.5 em from the corner.
    assert!((bar.width() - 0.4 * area.width()).abs() < 1e-9);
    assert!((bar.height() - EM).abs() < 1e-9);
    assert!((bar.y0 - (area.y0 + 0.9 * EM)).abs() < 1e-9);
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0).legend(Inset::BottomLeft, colorbar(heat));
    let (list, area) = drawn(&fig, Size::new(400.0, 640.0), Target::Vector);
    let [(bar, image)] = strips(&list.commands)[..] else {
        panic!("one strip");
    };
    assert_eq!((image.width, image.height), (1, 256));
    assert!((bar.height() - 0.4 * area.height()).abs() < 1e-9);
    assert!((bar.x0 - (area.x0 + 0.9 * EM)).abs() < 1e-9);
}

#[test]
fn binned_colorbars_draw_a_row_per_bin_and_label_their_edges() {
    let (mut fig, q) = panel(scale::quantize([0.0, 100.0], blues5()));
    let red = Rgb::new(255.0, 0.0, 0.0);
    let blue = Rgb::new(0.0, 0.0, 255.0);
    let white = Rgb::new(255.0, 255.0, 255.0);
    let t = fig.scale(scale::threshold([0.0, 1.0], [red, white, blue]));
    let sample = [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];
    let quantile = fig.scale(scale::quantile(sample, [red, white, blue, red]));
    fig.legend(Slot::Right, colorbar(q))
        .legend(Slot::Right, colorbar(t))
        .legend(Slot::Right, colorbar(quantile));
    let (list, _) = vector(&fig);
    let [quantize, threshold, quantiles] = [0, 1, 2].map(|k| slotted(&list, k));
    let [(near, rows)] = strips(quantize)[..] else {
        panic!("one strip");
    };
    assert_eq!((rows.width, rows.height), (1, 5));
    // The first row is the high end: blues5's darkest.
    assert_eq!(&rows.data.data()[..4], [0x08, 0x51, 0x9c, 255]);
    assert_eq!(texts(quantize), ["0", "20", "40", "60", "80", "100"]);
    // A threshold scale's three bins are equal; its thresholds sit at 1/3
    // and 2/3 of the bar.
    let [(far, rows)] = strips(threshold)[..] else {
        panic!("one strip");
    };
    assert_eq!((rows.width, rows.height), (1, 3));
    assert_eq!(&rows.data.data()[4..8], [255, 255, 255, 255]);
    assert_eq!(texts(threshold), ["0", "1"]);
    for ([(_, y), _], share) in ticks(threshold).iter().zip([1.0 / 3.0, 2.0 / 3.0]) {
        assert!((y - (far.y1 - share * far.height())).abs() < 1e-9);
    }
    assert!(far.x0 > near.x1 + 0.5 * EM);
    // Uneven edges keep their own digits: 7.25 does not read "7".
    assert_eq!(texts(quantiles), ["3.00", "7.25", "9.00", "14.50", "20.00"]);
}

#[test]
fn labels_follow_the_figure_locale_unless_formatted() {
    let (mut fig, q) = panel(scale::quantize(
        [0.0, 20_000.0],
        [Rgb::new(0.0, 0.0, 0.0); 2],
    ));
    let heat = fig.scale(scale::sequential([0.0, 20_000.0], scheme::viridis));
    fig.legend(Slot::Right, colorbar(q)).legend(
        Slot::Right,
        colorbar(heat).ticks(2).format(|v| format!("{v:.0} m")),
    );
    let (list, _) = vector(&fig);
    assert_eq!(
        texts(slotted(&list, 0)),
        ["0", "10\u{202f}000", "20\u{202f}000"]
    );
    assert_eq!(texts(slotted(&list, 1)), ["0 m", "10000 m", "20000 m"]);
    fig.locale(Locale::EN_US);
    let (list, _) = vector(&fig);
    assert_eq!(texts(slotted(&list, 0)), ["0", "10,000", "20,000"]);
}

#[test]
fn a_symlog_colorbar_ticks_at_its_decades() {
    let s = scale::sequential([0.0, 1e6], scheme::viridis).transform(transform::Symlog::new(1.0));
    assert_eq!(s.apply(1000.0).format_hex(), "#21918c");
    let (mut fig, heat) = panel(s.clone());
    fig.legend(
        Slot::Right,
        colorbar(heat).ticks(9).format(|v| format!("{v}")),
    );
    let (list, _) = vector(&fig);
    let cb = slotted(&list, 0);
    assert_eq!(
        texts(cb),
        ["0", "1", "10", "100", "1000", "10000", "100000", "1000000"]
    );
    // log1p(1000) / log1p(1e6) = 0.50007 of the bar.
    let bar = strip(cb);
    let [(_, y), _] = ticks(cb)[4];
    assert!((bar.y1 - y - 0.500_07 * bar.height()).abs() < 1e-4 * bar.height());
}

#[test]
fn extend_both_adds_triangles_of_5_percent_of_the_interior_in_the_end_colors() {
    let s = scale::sequential([0.0, 80.0], scheme::viridis);
    let (mut fig, heat) = panel(s);
    fig.legend(Slot::Right, colorbar(heat).extend(Extend::Both));
    let (list, area) = vector(&fig);
    let cb = slotted(&list, 0);
    let bar = strip(cb);
    assert!((bar.height() * 1.1 - area.height()).abs() < 1e-9);
    let triangles = fills(cb);
    let [(low, low_paint), (high, high_paint)] = triangles[..] else {
        panic!("two triangles");
    };
    let (low, high) = (low.bounding_box(), high.bounding_box());
    assert!((low.height() - 0.05 * bar.height()).abs() < 1e-9);
    assert!((high.height() - 0.05 * bar.height()).abs() < 1e-9);
    assert_eq!((low.y0, high.y1), (bar.y1, bar.y0));
    assert!(close(high.y0, area.y0) && close(low.y1, area.y1));
    assert_eq!(*low_paint, crate::sink::rgb(0x440154));
    assert_eq!(*high_paint, crate::sink::rgb(0xfde725));
    // The outline runs round the bar and both triangles.
    let outline: Vec<Rect> = strokes(cb)
        .iter()
        .filter(|(path, _)| closed(path))
        .map(|(path, _)| path.bounding_box())
        .collect();
    let [outline] = outline[..] else {
        panic!("one outline");
    };
    let expected = Rect::new(bar.x0, area.y0, bar.x1, area.y1);
    let pairs = [
        (outline.x0, expected.x0),
        (outline.y0, expected.y0),
        (outline.x1, expected.x1),
        (outline.y1, expected.y1),
    ];
    assert!(pairs.iter().all(|(a, b)| close(*a, *b)), "{outline:?}");
}

#[test]
fn the_outline_is_the_axis_line_and_the_dark_styles_have_none() {
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.legend(Slot::Right, colorbar(heat).extend(Extend::Min));
    let (list, _) = vector(&fig);
    let cb = slotted(&list, 0);
    let outlines: Vec<f64> = strokes(cb)
        .iter()
        .filter(|(path, _)| closed(path))
        .map(|(_, w)| *w)
        .collect();
    let [width] = outlines[..] else {
        panic!("one outline");
    };
    assert!(close(width, 1.25 * 4.0 / 3.0));
    for style in [Style::DarkGrid, Style::Dark] {
        fig.theme(Theme::seaborn(style, Context::Notebook));
        let (list, _) = vector(&fig);
        let cb = slotted(&list, 0);
        assert!(strokes(cb).iter().all(|(path, _)| !closed(path)));
        // Tick marks stay, as matplotlib's colorbars keep theirs.
        assert!(!ticks(cb).is_empty());
    }
}

/// `fig` sized so that its first plot area is `length` px long along
/// `slot` (tall beside left and right slots, wide beside top and bottom
/// ones) when drawn for a PNG at 1×: the canvas and that area.
fn sized(fig: &Figure<'_>, slot: Slot, length: f64) -> (Size, Rect) {
    let target = Target::File { scale: 1.0 };
    let vertical = matches!(slot, Slot::Left | Slot::Right);
    let mut size = Size::new(length + 200.0, length + 160.0);
    for _ in 0..4 {
        let area = fig
            .resolve(size, &mut cache(), &Held::default(), target)
            .get_plot_areas()[0];
        let along = if vertical {
            area.height()
        } else {
            area.width()
        };
        if along == length {
            return (size, area);
        }
        if vertical {
            size.height += length - along;
        } else {
            size.width += length - along;
        }
    }
    panic!("no canvas gives a plot area {length} px long");
}

#[test]
fn a_300_px_bar_with_both_triangles_takes_15_px_for_each() {
    let s = scale::sequential([0.0, 80.0], scheme::viridis);
    let (mut fig, heat) = panel(s);
    fig.legend(Slot::Right, colorbar(heat).extend(Extend::Both));
    let (size, area) = sized(&fig, Slot::Right, 330.0);
    let (list, _) = drawn(&fig, size, Target::File { scale: 1.0 });
    let cb = slotted(&list, 0);
    let bar = strip(cb);
    assert_eq!(bar.height(), 300.0);
    for (triangle, _) in fills(cb) {
        assert_eq!(triangle.bounding_box().height(), 15.0);
    }
    assert_eq!((bar.y0 - 15.0, bar.y1 + 15.0), (area.y0, area.y1));
}

/// Each pixel's RGBA along the middle of a colorbar of `color`, from its
/// low end (bottom or left). The bar is drawn in `slot` beside a panel
/// 300 px long along it and rendered at 1× by vello_cpu. The style draws
/// no outline, which would cover the strip's end pixels.
#[cfg(feature = "raster")]
fn rendered<S: Paint<Domain = f64> + Ramp>(color: S, slot: Slot) -> Vec<[u8; 4]> {
    let (mut fig, heat) = panel(color);
    fig.theme(Theme::seaborn(Style::Dark, Context::Notebook))
        .legend(slot, colorbar(heat));
    let (size, area) = sized(&fig, slot, 300.0);
    let (list, _) = drawn(&fig, size, Target::File { scale: 1.0 });
    let bar = strip(slotted(&list, 0));
    let vertical = matches!(slot, Slot::Left | Slot::Right);
    if vertical {
        assert_eq!((bar.y0, bar.y1), (area.y0, area.y1));
    } else {
        assert_eq!((bar.x0, bar.x1), (area.x0, area.x1));
    }
    let image = crate::raster::vello::whole(&list, 1.0);
    let middle = |a: f64, b: f64| ((a + b) / 2.0).floor() as u32;
    (0..300)
        .map(|j| {
            let (x, y) = if vertical {
                (middle(bar.x0, bar.x1), bar.y1 as u32 - 1 - j)
            } else {
                (bar.x0 as u32 + j, middle(bar.y0, bar.y1))
            };
            let at = ((y * image.width + x) * 4) as usize;
            [0, 1, 2, 3].map(|k| image.rgba[at + k])
        })
        .collect()
}

/// `c`'s channels as 8 bits, as a strip stores them.
#[cfg(feature = "raster")]
fn rgba(c: Rgb) -> [u8; 4] {
    let c = c.clamp();
    [
        c.r as u8,
        c.g as u8,
        c.b as u8,
        (c.opacity * 255.0).round() as u8,
    ]
}

#[cfg(feature = "raster")]
#[test]
fn a_table_strip_equals_its_scale_at_every_pixel_of_a_300_px_bar() {
    let tables = [
        ("viridis", scheme::viridis as fn(f64) -> Rgb),
        ("magma", scheme::magma),
    ];
    for (name, table) in tables {
        let s: Sequential<Rgb> = scale::sequential([0.0, 80.0], table);
        for slot in [Slot::Right, Slot::Bottom] {
            let pixels = rendered(s.clone(), slot);
            let wrong: Vec<usize> = (0..300)
                .filter(|&j| pixels[j] != rgba(s.apply((j as f64 + 0.5) / 300.0 * 80.0)))
                .collect();
            assert_eq!(wrong, [0_usize; 0], "{name} {slot:?}");
            if name == "viridis" {
                // Pixel 150 is 40.13: t = 0.50167, row 128.
                assert_eq!(pixels[150], [0x21, 0x91, 0x8c, 255]);
                assert_eq!(pixels[0], [0x44, 0x01, 0x54, 255]);
                assert_eq!(pixels[299], [0xfd, 0xe7, 0x25, 255]);
            }
        }
    }
}

#[cfg(feature = "raster")]
#[test]
fn an_interpolator_strip_shows_the_interpolator_at_each_row_centre() {
    for interpolator in [scheme::turbo as fn(f64) -> Rgb, scheme::rd_bu] {
        for slot in [Slot::Left, Slot::Top] {
            let pixels = rendered(scale::sequential([-3.0, 5.0], interpolator), slot);
            let wrong: Vec<usize> = (0..300)
                .filter(|&j| {
                    let row = ((j as f64 + 0.5) / 300.0 * 256.0).floor();
                    pixels[j] != rgba(interpolator((row + 0.5) / 256.0))
                })
                .collect();
            assert_eq!(wrong, [0_usize; 0], "{slot:?}");
        }
    }
}
