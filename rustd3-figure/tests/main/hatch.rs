//! Hatching: a clip group after each hatched shape's fill, lines anchored
//! to the figure so neighboring bars share them, path clips for areas,
//! and legend swatches that repeat their key's hatch.

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::hatch::Hatch;
use rustd3_figure::kurbo::{Affine, BezPath, PathEl, Rect, Shape, Size, Vec2};
use rustd3_figure::legend::{Slot, legend};
use rustd3_figure::list::{Clip, Command, List, part};
use rustd3_figure::mark::{area, bar_y, density_y, histogram};
use rustd3_figure::peniko::Brush;
use rustd3_figure::{Figure, Held, Target, axis};

use crate::figure::cache;

/// The spacing of a default hatch at notebook, px: 6 pt.
const SPACING: f64 = 8.0;
/// The line width of a default hatch at notebook, px: 0.8 pt.
const WIDTH: f64 = 0.8 * 4.0 / 3.0;

/// `fig` drawn at 640 × 480 for vector output.
fn draw(fig: &Figure<'_>) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    )
}

/// The commands of mark `mark` of panel `panel`.
fn mark(list: &List, panel: u64, mark: u64) -> &[Command] {
    let id = part::Id::new(6 << 56 | panel << 32 | mark);
    let p = list
        .parts
        .iter()
        .find(|p| p.id == id)
        .unwrap_or_else(|| panic!("no mark {panel}:{mark}"));
    &list.commands[p.commands.clone()]
}

/// The hatch groups among `commands`: each one's clip and its stroke's
/// path, width and paint, checking that a group holds one stroke under
/// an identity transform.
fn groups(commands: &[Command]) -> Vec<(&Clip, &BezPath, f64, &Brush)> {
    let mut out = Vec::new();
    for w in commands.windows(3) {
        if let [
            Command::Push {
                transform,
                clip: Some(clip),
            },
            Command::Stroke { path, style, paint },
            Command::Pop,
        ] = w
        {
            assert_eq!(*transform, Affine::IDENTITY);
            out.push((clip, path, style.width, paint));
        }
    }
    out
}

/// Each line's `k`: its normal offset in the figure's coordinates as a
/// multiple of the spacing, for a 45° family in a group at `origin`.
fn offsets(path: &BezPath, origin: Vec2) -> Vec<f64> {
    let n = Vec2::new(0.5_f64.sqrt(), 0.5_f64.sqrt());
    path.elements()
        .iter()
        .filter_map(|el| {
            if let PathEl::MoveTo(p) = el {
                Some((p.to_vec2() + origin).dot(n) / SPACING)
            } else {
                None
            }
        })
        .collect()
}

/// A figure of one panel, a band x without padding over three keys, a
/// count y, and the plot area it lays out.
fn bars(hatched: bool) -> (Figure<'static>, Rect) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b", "c"]));
    let y = fig.fit(scale::linear());
    let bars = bar_y([("a", 3.0), ("b", 1.5), ("c", 2.0)], x, y);
    let bars = if hatched {
        bars.hatch(Hatch::lines(45.0))
    } else {
        bars
    };
    fig.panel(0, 0)
        .mark(bars)
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    let area = crate::figure::resolve(&fig, &Held::default()).get_plot_areas()[0];
    (fig, area)
}

#[test]
fn a_hatched_bar_gets_a_clip_group_after_its_fill() {
    let (plain, _) = bars(false);
    let (fig, area) = bars(true);
    let list = draw(&fig);
    let commands = mark(&list, 0, 0);
    assert_eq!(commands.len(), mark(&draw(&plain), 0, 0).len() + 3 * 3);
    let groups = groups(commands);
    assert_eq!(groups.len(), 3);
    for (i, (clip, path, width, paint)) in groups.into_iter().enumerate() {
        let Clip::Rect(r) = clip else {
            panic!("a bar clips to its rectangle");
        };
        let Command::Fill { path: fill, .. } = &commands[4 * i] else {
            panic!("the fill comes first");
        };
        assert_eq!(fill.bounding_box(), *r);
        assert!((width - WIDTH).abs() < 1e-9);
        assert_eq!(*paint, crate::sink::rgb(0x262626));
        // Every line of the stroke is a whole number of spacings from the
        // figure's origin.
        for k in offsets(path, area.origin().to_vec2()) {
            assert!((k - k.round()).abs() < 1e-9, "{k}");
        }
    }
}

#[test]
fn neighbouring_bars_share_the_lines_that_cross_their_edge() {
    let (fig, area) = bars(true);
    let list = draw(&fig);
    let groups = groups(mark(&list, 0, 0));
    let origin = area.origin().to_vec2();
    let ks: Vec<Vec<i64>> = groups
        .iter()
        .map(|(_, path, _, _)| {
            offsets(path, origin)
                .into_iter()
                .map(|k| k.round() as i64)
                .collect()
        })
        .collect();
    // Bars a and b touch (no padding): lines whose k both cover are the
    // same lines, continuing across the shared edge.
    let shared: Vec<i64> = ks[0]
        .iter()
        .filter(|k| ks[1].contains(k))
        .copied()
        .collect();
    assert!(shared.len() > 3, "{ks:?}");
    let (Clip::Rect(a), Clip::Rect(b)) = (groups[0].0, groups[1].0) else {
        panic!("bars clip to rectangles");
    };
    assert_eq!(a.x1, b.x0);
}

#[test]
fn areas_and_densities_clip_their_hatch_to_their_outline() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let w = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(area([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y).hatch(Hatch::cross(30.0)))
        .mark(density_y([1.0, 1.5, 2.0, 2.2, 2.9], y, w).hatch(Hatch::lines(-45.0)));
    let list = draw(&fig);
    for m in [0, 1] {
        let commands = mark(&list, 0, m);
        let [(clip, _, _, _)] = groups(commands)[..] else {
            panic!("one hatch group");
        };
        let Command::Fill { path, .. } = &commands[0] else {
            panic!("the fill comes first");
        };
        assert_eq!(*clip, Clip::Path(path.clone()));
    }
    // The density's outline is drawn over its hatch.
    let density = mark(&list, 0, 1);
    assert!(matches!(density.last(), Some(Command::Stroke { .. })));
}

#[test]
fn legend_squares_repeat_the_hatch_of_their_key() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let count = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(["hatched", "plain"], COLORBLIND));
    fig.panel(0, 0)
        .mark(
            histogram([1.0, 2.0, 2.5, 3.0], x, count)
                .fill(color, "hatched")
                .hatch(Hatch::lines(45.0).color(Rgb::hex(0xff0000))),
        )
        .mark(histogram([1.5, 2.0, 2.2], x, count).fill(color, "plain"));
    fig.legend(Slot::Bottom, legend(color));
    let list = draw(&fig);
    let id = part::Id::new(7 << 56 | 0xff_ffff << 32);
    let p = list.parts.iter().find(|p| p.id == id).expect("the legend");
    let commands = &list.commands[p.commands.clone()];
    let groups = groups(commands);
    let [(Clip::Rect(square), path, _, paint)] = groups[..] else {
        panic!("one hatched swatch: {groups:?}");
    };
    assert_eq!(*paint, crate::sink::rgb(0xff0000));
    let Command::Fill { path: fill, .. } = &commands[0] else {
        panic!("the hatched key's square comes first");
    };
    assert_eq!(fill.bounding_box(), *square);
    // A swatch's lines are anchored to the figure too.
    for k in offsets(path, Vec2::ZERO) {
        assert!((k - k.round()).abs() < 1e-9, "{k}");
    }
}
