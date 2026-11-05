//! `fig.check`: what does not fit or will not draw well, found without
//! drawing.

use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::check::{Direction, Element, Issue};
use rustd3_figure::kurbo::Size;
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held, Target, axis, text};

use crate::figure::cache;
use crate::{scatter, trace};

/// One panel, x and y fixed to 0 … 10.
fn panel() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0);
    (fig, x, y)
}

/// The size, 640 px wide, at which panel 0 of `fig` is `height` px tall.
fn at_height(fig: &Figure<'_>, height: f64) -> Size {
    let mut size = Size::new(640.0, height + 60.0);
    for _ in 0..20 {
        let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
        let h = r.get_plot_areas()[0].height();
        if (h - height).abs() < 1e-9 {
            return size;
        }
        size.height += height - h;
    }
    panic!("no size gives a plot area {height} px tall");
}

#[test]
fn the_examples_fit_at_960_by_760() {
    let rec = trace::recording(trace::SAMPLES);
    assert_eq!(trace::trace(&rec).check(Size::new(960.0, 760.0)), []);
    let scatter = scatter::scatter(scatter::POINTS);
    assert_eq!(scatter.check(Size::new(960.0, 760.0)), []);
}

#[test]
fn an_axis_title_longer_than_its_panel_overflows() {
    let (mut fig, _, y) = panel();
    fig.panel(0, 0)
        .axis(axis::left(y).label("vibration RMS [m/s²]"));
    let size = at_height(&fig, 100.0);
    let issues = fig.check(size);
    let [
        Issue::Overflow {
            what,
            panel,
            needed,
            available,
        },
    ] = issues[..]
    else {
        panic!("{issues:#?}");
    };
    assert_eq!((what, panel), (Element::AxisTitle, Some(0)));
    assert_eq!(format!("{needed:.2}"), "164.56");
    assert!((available - 100.0).abs() < 1e-9, "{available}");
}

#[test]
fn panel_and_figure_titles_and_legends_wider_than_their_room_overflow() {
    let (mut fig, ..) = panel();
    let color = fig.scale(scale::ordinal(
        ["a rather long first key", "another rather long key"],
        COLORBLIND,
    ));
    fig.title("A figure title far too long for a canvas this narrow")
        .legend(Slot::Bottom, legend(color))
        .panel(0, 0)
        .title("A panel title too long for its panel")
        .legend(Inset::TopLeft, legend(color));
    let issues = fig.check(Size::new(240.0, 400.0));
    let found: Vec<(Element, Option<usize>)> = issues
        .iter()
        .filter_map(|i| {
            if let Issue::Overflow { what, panel, .. } = i {
                Some((*what, *panel))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        found,
        [
            (Element::FigureTitle, None),
            (Element::PanelTitle, Some(0)),
            (Element::Legend, Some(0)),
            (Element::Legend, None),
        ],
        "{issues:#?}"
    );
}

#[test]
fn a_canvas_too_small_for_its_decorations_has_no_room() {
    let (mut fig, x, y) = panel();
    fig.panel(0, 0)
        .axis(axis::left(y).label("vibration RMS [m/s²]"))
        .axis(axis::bottom(x).label("speed [km/h]"));
    let issues = fig.check(Size::new(60.0, 40.0));
    let axes: Vec<Direction> = issues
        .iter()
        .filter_map(|i| {
            let Issue::NoRoom {
                axis,
                needed,
                available,
            } = i
            else {
                return None;
            };
            assert!(needed >= available, "{i:?}");
            Some(*axis)
        })
        .collect();
    assert_eq!(axes, [Direction::Horizontal, Direction::Vertical]);
}

#[test]
fn an_axis_left_with_one_tick_label_is_reported() {
    let (mut fig, x, y) = panel();
    fig.panel(0, 0)
        .axis(axis::left(y))
        .axis(axis::bottom(x).format(|v| format!("{v} metres per second")));
    let issues = fig.check(Size::new(240.0, 240.0));
    let one = Issue::FewLabels {
        panel: 0,
        axis: 1,
        kept: 1,
    };
    assert_eq!(issues, [one]);
    assert_eq!(fig.check(Size::new(1600.0, 240.0)), []);
}

#[test]
fn inset_legends_drawn_over_each_other_are_reported() {
    let (mut fig, ..) = panel();
    let color = fig.scale(scale::ordinal(["accel x", "accel z", "temp"], COLORBLIND));
    let key = legend(color);
    fig.panel(0, 0)
        .legend(Inset::TopLeft, key.clone())
        .legend(Inset::TopLeft, key.clone())
        .legend(Inset::BottomLeft, key);
    // Two stacked in the top corner and one in the bottom corner hold
    // apart in a tall plot area and overlap in a short one.
    assert_eq!(fig.check(Size::new(640.0, 480.0)), []);
    let issues = fig.check(Size::new(640.0, 220.0));
    let overlaps: Vec<Issue> = issues
        .into_iter()
        .filter(|i| matches!(i, Issue::LegendsOverlap { .. }))
        .collect();
    let pair = |legends| Issue::LegendsOverlap {
        panel: Some(0),
        legends,
    };
    assert_eq!(overlaps, [pair([1, 2])]);
}

#[test]
fn owned_data_over_100000_points_belongs_in_a_recording() {
    let (mut fig, x, y) = panel();
    let many: Vec<(f64, f64)> = (0..150_000).map(|i| (f64::from(i) * 1e-4, 1.0)).collect();
    let few: Vec<(f64, f64)> = many[..100_000].to_vec();
    fig.panel(0, 0).mark(dots(few, x, y)).mark(line(many, x, y));
    let issues = fig.check(Size::new(640.0, 480.0));
    assert_eq!(
        issues,
        [Issue::LargeOwned {
            panel: 0,
            mark: 1,
            points: 150_000
        }]
    );
}

#[test]
fn glyphs_the_fonts_lack_and_rejected_markup_are_reported() {
    let (mut fig, ..) = panel();
    fig.title("漢字").panel(0, 0).title(text::typst("$ x^ $"));
    let issues = fig.check(Size::new(640.0, 480.0));
    // By label: `#"漢字"` sorts before `$ x^ $`.
    let [
        Issue::MissingGlyphs {
            text: missing,
            count,
        },
        Issue::Markup { text, message },
    ] = &issues[..]
    else {
        panic!("{issues:#?}");
    };
    assert_eq!(text, "$ x^ $");
    assert!(!message.is_empty());
    assert_eq!((missing.as_str(), *count), ("漢字", 2));
}

#[test]
fn a_palette_key_outside_the_palette_is_reported() {
    let (mut fig, x, y) = panel();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(dots([(1.0, 1.0)], x, y).fill(color, "a"))
        .mark(line([(1.0, 1.0), (2.0, 2.0)], x, y).stroke(color, "z"));
    let issues = fig.check(Size::new(640.0, 480.0));
    assert_eq!(issues, [Issue::UnknownKey { panel: 0, mark: 1 }]);
}
