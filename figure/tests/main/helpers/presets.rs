//! Chart presets: the marks, legend and axes each draws, the stack
//! offsets' spot rows, a line over a recording, and an empty
//! `fig.check` for each at its default size.

use rustd3_core::scale::{self, AnyContinuous};
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::series::{Clock, Recording};
use rustd3_core::shape::{self, stack};
use rustd3_figure::helpers::axes;
use rustd3_figure::helpers::chart::{self, bar::Mode};
use rustd3_figure::kurbo::{Rect, Shape, Size};
use rustd3_figure::legend::{Slot, Swatch, legend};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::mark::{Grid, line};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::{Figure, Target, axis};

use super::axes::{MINOR, axis_part};
use crate::marks::{draw, role};

/// The commands of each mark part of `list`, in the marks' order.
fn mark_parts(list: &List) -> Vec<&[Command]> {
    list.parts
        .iter()
        .filter(|p| role(p.id) == 6)
        .map(|p| &list.commands[p.commands.clone()])
        .collect()
}

/// The texts of the glyph runs of the legend in figure slot 0, when there
/// is one.
fn legend_texts(list: &List) -> Option<Vec<String>> {
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32);
    let p = list.parts.iter().find(|p| p.id == id)?;
    let texts = list.commands[p.commands.clone()]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.text.to_string())
            } else {
                None
            }
        })
        .collect();
    Some(texts)
}

/// The bounding boxes of the filled paths among `commands`.
fn fills(commands: &[Command]) -> Vec<Rect> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Fill { path, .. } = c {
                Some(path.bounding_box())
            } else {
                None
            }
        })
        .collect()
}

/// The paint of the first stroke or fill among `commands`.
fn paint(commands: &[Command]) -> Option<&Brush> {
    commands.iter().find_map(|c| {
        if let Command::Stroke { paint, .. } | Command::Fill { paint, .. } = c {
            Some(paint)
        } else {
            None
        }
    })
}

/// The opaque brush of palette color `i`.
fn palette(i: usize) -> Brush {
    let c = COLORBLIND[i];
    let [r, g, b] = [c.r, c.g, c.b].map(|v| v.round() as u8);
    Brush::Solid(Color::from_rgba8(r, g, b, 255))
}

/// Asserts that `fig` has no issue at the presets' default size.
fn assert_checks(fig: &Figure<'_>) {
    #[cfg(feature = "fonts")]
    {
        let issues = fig.check(rustd3_figure::kurbo::Size::new(640.0, 480.0));
        assert!(issues.is_empty(), "{issues:#?}");
    }
    #[cfg(not(feature = "fonts"))]
    let _ = fig;
}

/// Whether `a` is within 1e-9 of `b`.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// The spans `[y0, y1]` of `rects` in data units, read through the linear
/// map that puts `zero` px at 0 and `unit` px at 1.
fn spans(rects: &[Rect], [zero, unit]: [f64; 2]) -> Vec<[f64; 2]> {
    let v = |px: f64| (px - zero) / (unit - zero);
    rects.iter().map(|r| [v(r.y1), v(r.y0)]).collect()
}

/// Asserts that `got` equals `expected` to 1e-9.
fn assert_spans(got: &[[f64; 2]], expected: &[[f64; 2]]) {
    assert_eq!(got.len(), expected.len(), "{got:?}");
    for (g, e) in got.iter().zip(expected) {
        assert!(
            near(g[0], e[0]) && near(g[1], e[1]),
            "{got:?} ≠ {expected:?}"
        );
    }
}

#[test]
fn a_line_preset_draws_a_line_per_series_with_a_legend_and_minor_ticks() {
    let t: Vec<f64> = (0..=100).map(|i| f64::from(i) * 0.1).collect();
    let fig = chart::line(t.iter().map(|&t| (t, t.sin())))
        .series(t.iter().map(|&t| (t, (0.7 * t).cos())), "cos")
        .series(t.iter().map(|&t| (t, 0.1 * t)), "trend")
        .title("line")
        .x_label("t [s]")
        .figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 3);
    for (i, commands) in marks.iter().enumerate() {
        let strokes = commands
            .iter()
            .filter(|c| matches!(c, Command::Stroke { .. }))
            .count();
        assert_eq!(strokes, 1);
        assert_eq!(paint(commands), Some(&palette(i)));
    }
    assert_eq!(
        legend_texts(&list),
        Some(vec!["0".into(), "cos".into(), "trend".into()])
    );
    for axis in [0, 1] {
        let (strokes, _) = axis_part(&list, axis);
        assert_eq!(strokes.len(), 2, "major and minor ticks");
        assert!(near(strokes[1].0, MINOR[1]));
    }
    assert_checks(&fig);
    let one = chart::line([(0.0, 1.0), (1.0, 2.0)]).minor(false).figure();
    let list = draw(&one);
    assert_eq!(mark_parts(&list).len(), 1);
    assert_eq!(legend_texts(&list), None, "one series: no legend");
    assert_eq!(axis_part(&list, 0).0.len(), 1, "no minor ticks");
    assert_checks(&one);
}

#[test]
fn the_line_sketch_of_helpers_is_its_composition_and_saves_1280_by_960() {
    let samples: Vec<f64> = (0..2000)
        .map(|i| 50.0 * (f64::from(i) * 0.01).sin().powi(3))
        .collect();
    let rec = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 0.01,
        },
        [&samples],
    )
    .expect("a recording");
    let sketch = || {
        chart::line(rec.series(0).expect("channel 0"))
            .y_scale(scale::symlog().constant(1.0))
            .minor(true)
            .title("RMS")
    };
    let fig = sketch().figure();
    // The same figure written out: the preset is ordinary code.
    let mut manual = Figure::new();
    let x = manual.fit(AnyContinuous::from(scale::linear()));
    let y = manual.fit(AnyContinuous::from(scale::symlog().constant(1.0)));
    let color = manual.scale(scale::ordinal(["0".to_string()], COLORBLIND));
    manual
        .panel(0, 0)
        .mark(line(rec.series(0).expect("channel 0"), x, y).stroke(color, "0".into()))
        .axis(axes::defaults(axis::bottom(x)))
        .axis(axes::defaults(axis::left(y)))
        .title("RMS");
    let list = draw(&fig);
    assert_eq!(list.commands, draw(&manual).commands);
    // Symlog ticks with minor ticks between them.
    let (strokes, texts) = axis_part(&list, 1);
    assert_eq!(strokes.len(), 2);
    assert!(!strokes[1].1.is_empty() && near(strokes[1].0, MINOR[1]));
    assert!(texts.contains(&"0".to_string()), "{texts:?}");
    assert_checks(&fig);
    #[cfg(all(feature = "png", feature = "fonts"))]
    {
        let path =
            std::env::temp_dir().join(format!("rustd3-figure-{}-rms.png", std::process::id()));
        sketch().save_png(&path, 2.0).expect("the file is written");
        let file = std::fs::File::open(&path).expect("the file opens");
        let reader = png::Decoder::new(std::io::BufReader::new(file))
            .read_info()
            .expect("a PNG");
        let (width, height) = (reader.info().width, reader.info().height);
        std::fs::remove_file(&path).expect("the file is removed");
        assert_eq!((width, height), (1280, 960));
    }
}

#[test]
fn a_scatter_preset_draws_one_markers_command_without_a_legend() {
    let points: Vec<(f64, f64)> = (0..40)
        .map(|i| (f64::from(i), (f64::from(i) * 0.4).sin()))
        .collect();
    let fig = chart::scatter(points.clone()).y_label("signal").figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 1);
    let [Command::Markers { at, paint, .. }] = marks[0] else {
        panic!("one markers command: {:?}", marks[0]);
    };
    assert_eq!((at.len(), paint), (40, &palette(0)));
    assert_eq!(legend_texts(&list), None);
    assert_eq!(axis_part(&list, 1).0.len(), 2, "minor ticks on y");
    assert_checks(&fig);
}

/// Three series over four keys.
fn grouped() -> Figure<'static> {
    chart::bar(
        [
            ("mon", vec![3.0, 2.5, 1.0]),
            ("tue", vec![4.0, 1.0, 2.0]),
            ("wed", vec![2.0, 3.5]),
            ("thu", vec![5.0, 2.0, 1.5]),
        ],
        vec!["north".into(), "south".into(), "east".into()],
    )
    .y_label("orders")
    .figure()
}

#[test]
fn grouped_bars_are_one_dodged_bar_mark_per_series_in_four_fifths_of_each_step() {
    let fig = grouped();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 3);
    let rects: Vec<Vec<Rect>> = marks.iter().map(|m| fills(m)).collect();
    // "wed" has no third value: no bar.
    assert_eq!(rects.iter().map(Vec::len).collect::<Vec<_>>(), [4, 4, 3]);
    for (i, m) in marks.iter().enumerate() {
        assert_eq!(paint(m), Some(&palette(i)));
    }
    // Side by side, each a third of 0.8 of the step, the step the plot
    // area's width over four keys (marks draw in the plot area's
    // coordinates).
    let area = crate::figure::resolve(&fig, &rustd3_figure::Held::default()).plot_areas()[0];
    let step = area.width() / 4.0;
    for k in 0..4 {
        let left = step * (0.1 + k as f64);
        for (i, series) in rects.iter().enumerate().take(2) {
            let r = series[k];
            let x0 = left + i as f64 * 0.8 * step / 3.0;
            assert!(near(r.x0, x0) && near(r.width(), 0.8 * step / 3.0), "{r:?}");
        }
    }
    assert_eq!(
        legend_texts(&list),
        Some(vec!["north".into(), "south".into(), "east".into()])
    );
    assert_eq!(axis_part(&list, 0).0.len(), 1, "no minor ticks on bands");
    assert_eq!(axis_part(&list, 1).0.len(), 2);
    assert_checks(&fig);
}

#[test]
fn stacked_bars_are_one_span_mark_per_series_from_shape_stack() {
    let (apples, bananas) = ([3.0, 2.0, 1.0], [1.0, 1.0, 4.0]);
    let layers = shape::stack(&[&apples, &bananas]);
    assert_eq!(layers[1], [[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]]);
    let rows = ["a", "b", "c"]
        .into_iter()
        .enumerate()
        .map(|(i, k)| (k, vec![apples[i], bananas[i]]));
    let fig = chart::bar(rows, vec!["apples".into(), "bananas".into()])
        .mode(Mode::Stacked)
        .figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 2, "two bar_y_spans marks");
    let rects: Vec<Vec<Rect>> = marks.iter().map(|m| fills(m)).collect();
    // apples' "a" spans [0, 3]: its bottom is 0 and its top 3.
    let a = rects[0][0];
    let unit = a.y1 + (a.y0 - a.y1) / 3.0;
    let scale = [a.y1, unit];
    assert_spans(&spans(&rects[0], scale), &layers[0]);
    assert_spans(
        &spans(&rects[1], scale),
        &[[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]],
    );
    assert_checks(&fig);
}

#[test]
fn diverging_bars_stack_positives_up_and_negatives_down_from_zero() {
    let rows = [("k0", vec![3.0, -1.0, 2.0]), ("k1", vec![-2.0, 4.0, 1.0])];
    let columns: [&[f64]; 3] = [&[3.0, -2.0], &[-1.0, 4.0], &[2.0, 1.0]];
    let layers = shape::Stack::default()
        .offset(stack::Offset::Diverging)
        .layout(&columns);
    let expected = [
        [[0.0, 3.0], [-2.0, 0.0]],
        [[-1.0, 0.0], [0.0, 4.0]],
        [[3.0, 5.0], [4.0, 5.0]],
    ];
    for (layer, e) in layers.iter().zip(&expected) {
        assert_eq!(layer, e);
    }
    let fig = chart::bar(rows, vec!["a".into(), "b".into(), "c".into()])
        .mode(Mode::Diverging)
        .figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 3);
    let rects: Vec<Vec<Rect>> = marks.iter().map(|m| fills(m)).collect();
    // a's "k0" spans [0, 3].
    let a = rects[0][0];
    let scale = [a.y1, a.y1 + (a.y0 - a.y1) / 3.0];
    for (got, e) in rects.iter().zip(&expected) {
        assert_spans(&spans(got, scale), e);
    }
    assert_checks(&fig);
}

#[test]
fn presets_keep_their_size_title_theme_and_labels() {
    let fig = chart::scatter([(0.0, 0.0), (1.0, 1.0)])
        .size(800.0, 600.0)
        .size(f64::NAN, 10.0)
        .title("t")
        .x_label("x")
        .y_label("y")
        .theme(rustd3_figure::Theme::default().palette([COLORBLIND[3]]))
        .figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    let [Command::Markers { paint, .. }] = marks[0] else {
        panic!("one markers command");
    };
    assert_eq!(paint, &palette(3));
    let heading = part::Id::new(5 << 56);
    assert!(list.parts.iter().any(|p| p.id == heading), "a panel title");
    assert_eq!(axis_part(&list, 0).1.last().map(String::as_str), Some("x"));
    assert_eq!(axis_part(&list, 1).1.last().map(String::as_str), Some("y"));
    #[cfg(feature = "fonts")]
    {
        let path =
            std::env::temp_dir().join(format!("rustd3-figure-{}-preset.svg", std::process::id()));
        chart::scatter([(0.0, 0.0), (1.0, 1.0)])
            .size(800.0, 600.0)
            .size(f64::NAN, 10.0)
            .save_svg(&path)
            .expect("the file is written");
        let svg = std::fs::read_to_string(&path).expect("the file reads back");
        std::fs::remove_file(&path).expect("the file is removed");
        assert!(svg.contains("width='800' height='600'"), "{}", &svg[..200]);
    }
}

#[test]
fn line_legends_show_line_swatches_as_a_written_out_legend_does() {
    let pairs = |k: f64| (0..10).map(move |i| (f64::from(i), k * f64::from(i)));
    let fig = chart::line(pairs(1.0))
        .series(pairs(2.0), "double")
        .figure();
    let mut manual = Figure::new();
    let color = manual.scale(scale::ordinal(
        ["0".to_string(), "double".to_string()],
        COLORBLIND,
    ));
    manual.legend(
        Slot::Right,
        legend(color).glyph(Swatch::Line { dash: None }),
    );
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32);
    let legend_of = |list: &List| {
        let p = list.parts.iter().find(|p| p.id == id).expect("a legend");
        list.commands[p.commands.clone()].to_vec()
    };
    let strokes = |commands: &[Command]| {
        commands
            .iter()
            .filter(|c| matches!(c, Command::Stroke { .. }))
            .count()
    };
    assert_eq!(strokes(&legend_of(&draw(&fig))), 2);
    assert_eq!(strokes(&legend_of(&draw(&manual))), 2);
}

#[test]
fn a_stacked_area_is_one_ribbon_per_column_on_shape_stacks_edges() {
    let fig = chart::area(
        vec![vec![1.0; 3], vec![2.0; 3], vec![3.0; 3]],
        vec![0.0, 1.0, 2.0],
    )
    .figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 3, "one ribbon per column");
    let boxes: Vec<Rect> = marks
        .iter()
        .map(|m| {
            let f = fills(m);
            assert_eq!(f.len(), 1, "one polygon");
            f[0]
        })
        .collect();
    for (i, m) in marks.iter().enumerate() {
        assert_eq!(paint(m), Some(&palette(i)));
    }
    // Column 0 spans [0, 1]: the other two stack on it.
    let scale = [boxes[0].y1, boxes[0].y0];
    assert_spans(&spans(&boxes, scale), &[[0.0, 1.0], [1.0, 3.0], [3.0, 6.0]]);
    assert_eq!(
        legend_texts(&list),
        Some(vec!["0".into(), "1".into(), "2".into()])
    );
    assert_eq!(axis_part(&list, 0).0.len(), 2, "minor ticks on x");
    assert_checks(&fig);
}

/// `fig` drawn for vector output at a size whose plot area is `w × h`
/// (a figure without axes, whose padding does not depend on its size).
fn drawn_with_area(fig: &Figure<'_>, w: f64, h: f64) -> (List, Rect) {
    let area = crate::figure::resolve(fig, &rustd3_figure::Held::default()).plot_areas()[0];
    let size = Size::new(w + 640.0 - area.width(), h + 480.0 - area.height());
    let mut cache = crate::figure::cache();
    let held = rustd3_figure::Held::default();
    let area = fig
        .resolve(size, &mut cache, &held, Target::Vector)
        .plot_areas()[0];
    let list = fig.draw_with(&mut cache, size, Target::Vector, &held);
    (list, area)
}

/// The union of the boxes of the filled paths among `commands`.
fn union(commands: &[Command]) -> Rect {
    fills(commands)
        .into_iter()
        .reduce(|a, b| a.union(b))
        .expect("a filled path")
}

#[test]
fn a_pie_fills_its_plot_area_less_half_an_em_with_labels_at_the_centroids() {
    let fig = chart::pie([("a", 3.0), ("b", 1.0), ("c", 2.0), ("none", 0.0)]).figure();
    let (list, area) = drawn_with_area(&fig, 600.0, 400.0);
    assert!(near(area.width(), 600.0) && near(area.height(), 400.0));
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 1);
    assert_eq!(fills(marks[0]).len(), 3, "no wedge without an angle");
    // 200 − em(0.5), em = 11 pt = 14.667 px: 192.667.
    let outer = 200.0 - 11.0 * 4.0 / 3.0 / 2.0;
    let pie = union(marks[0]);
    let tolerance = 0.1;
    assert!((pie.width() - 2.0 * outer).abs() < tolerance, "{pie:?}");
    assert!((pie.height() - 2.0 * outer).abs() < tolerance, "{pie:?}");
    let centre = pie.center();
    assert!((centre.x - 300.0).abs() < tolerance && (centre.y - 200.0).abs() < tolerance);
    // Wedges in data order, each in its key's palette color.
    let paints: Vec<&Brush> = marks[0]
        .iter()
        .filter_map(|c| {
            if let Command::Fill { paint, .. } = c {
                Some(paint)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(paints, [&palette(0), &palette(1), &palette(2)]);
    // One label per wedge, in data order (their centroids are the
    // labels' own spot rows).
    let texts: Vec<String> = marks[0]
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.text.to_string())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(texts, ["a", "b", "c"]);
    assert_eq!(legend_texts(&list), None, "labelled inside: no legend");
    assert_checks(&fig);
}

#[test]
fn a_donut_takes_its_inner_share_of_the_outer_radius() {
    let fig = chart::pie([("a", 1.0)]).inner(0.5).inner(1.5).figure();
    let (list, _) = drawn_with_area(&fig, 600.0, 400.0);
    let marks = mark_parts(&list);
    let [Command::Fill { path, .. }, ..] = marks[0] else {
        panic!("a wedge first: {:?}", marks[0]);
    };
    let outer = 200.0 - 11.0 * 4.0 / 3.0 / 2.0;
    // A full ring: the path holds both circles; its winding at a point
    // halfway out is 0 inside the hole and nonzero on the ring.
    let centre = path.bounding_box().center();
    let hole = centre + rustd3_figure::kurbo::Vec2::new(0.4 * outer, 0.0);
    let ring = centre + rustd3_figure::kurbo::Vec2::new(0.75 * outer, 0.0);
    assert_eq!(path.winding(hole), 0, "0.5 of the outer radius is empty");
    assert_ne!(path.winding(ring), 0);
    assert_checks(&fig);
}

/// The image command among `commands`: its rect, pixel size and pixels.
fn image_of(commands: &[Command]) -> (Rect, [u32; 2], Vec<u8>) {
    commands
        .iter()
        .find_map(|c| {
            if let Command::Image { rect, image, .. } = c {
                Some((
                    *rect,
                    [image.width, image.height],
                    image.data.data().to_vec(),
                ))
            } else {
                None
            }
        })
        .expect("an image")
}

/// Whether the legend in figure slot 0 of `list` holds an image (a
/// colorbar's strip).
fn has_colorbar(list: &List) -> bool {
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32);
    list.parts.iter().any(|p| {
        p.id == id
            && list.commands[p.commands.clone()]
                .iter()
                .any(|c| matches!(c, Command::Image { .. }))
    })
}

/// A 3 × 2 grid whose row 0 holds 0 and row 1 holds 1.
fn rows() -> Grid {
    Grid::new(3, 2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).expect("a grid")
}

#[test]
fn a_heatmap_fills_its_plot_area_with_its_grid_and_a_colorbar() {
    let fig = chart::heatmap(rows()).figure();
    let list = draw(&fig);
    let marks = mark_parts(&list);
    assert_eq!(marks.len(), 1);
    let area = crate::figure::resolve(&fig, &rustd3_figure::Held::default()).plot_areas()[0];
    let (rect, size, pixels) = image_of(marks[0]);
    assert_eq!(size, [3, 2]);
    assert!(near(rect.width(), area.width()) && near(rect.height(), area.height()));
    // Row 0 at the bottom: the image's last row.
    let (top, bottom) = (&pixels[..12], &pixels[12..]);
    let low = rustd3_core::scheme::viridis(0.0);
    let [r, g, b] = [low.r, low.g, low.b].map(|v| v.round() as u8);
    assert_eq!(&bottom[..4], [r, g, b, 255]);
    assert_ne!(top, bottom);
    assert!(has_colorbar(&list));
    assert_eq!(axis_part(&list, 0).0.len(), 2, "minor ticks on linear axes");
    assert_checks(&fig);
}

#[test]
fn heatmap_keys_give_band_axes_with_row_zero_on_top() {
    let names = |n: &[&str]| n.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let fig = chart::heatmap(rows())
        .keys(names(&["first", "second"]), names(&["a", "b", "c"]))
        .colors(rustd3_core::scheme::magma)
        .figure();
    let list = draw(&fig);
    let (rect, size, pixels) = image_of(mark_parts(&list)[0]);
    let area = crate::figure::resolve(&fig, &rustd3_figure::Held::default()).plot_areas()[0];
    assert_eq!(size, [3, 2]);
    assert!(near(rect.width(), area.width()) && near(rect.height(), area.height()));
    let low = rustd3_core::scheme::magma(0.0);
    let [r, g, b] = [low.r, low.g, low.b].map(|v| v.round() as u8);
    assert_eq!(&pixels[..4], [r, g, b, 255], "row 0 on top");
    let (strokes, texts) = axis_part(&list, 0);
    assert_eq!((strokes.len(), texts), (1, names(&["a", "b", "c"])));
    // A band axis lists its keys from the bottom up.
    assert_eq!(axis_part(&list, 1).1, names(&["second", "first"]));
    assert!(has_colorbar(&list));
    assert_checks(&fig);
}
