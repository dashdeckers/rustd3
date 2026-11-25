//! Axis groups: a second level of labels, one per run of neighboring ticks
//! with the same group name, past the tick labels.

use rustd3_core::scale::{self, Band};
use rustd3_figure::axis::{self, Nested};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held, Target};

use crate::figure::{cache, resolve};
use crate::support::figures_equal;

/// The canvas every figure here is drawn on.
const SIZE: Size = Size::new(640.0, 480.0);

/// A quarter of a year.
type Quarter = Nested<&'static str, &'static str>;

/// A figure with a band of five quarters over two years.
fn quarters() -> (Figure<'static>, ScaleRef<Band<Quarter>>) {
    let keys = [
        ("2025", "Q3"),
        ("2025", "Q4"),
        ("2026", "Q1"),
        ("2026", "Q2"),
        ("2026", "Q3"),
    ];
    let mut fig = Figure::new();
    let quarter = fig.scale(scale::band(keys.map(|(y, q)| Nested(y, q))).padding(0.2));
    (fig, quarter)
}

/// `fig` drawn for vector output, and its plot area.
fn drawn(fig: &Figure<'_>) -> (List, Rect) {
    let list = fig.draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    list::check(&list).expect("a valid list");
    (list, resolve(fig, &Held::default()).get_plot_areas()[0])
}

/// The ink of each glyph run in `list` whose text is `text`, in order.
fn ink(list: &List, text: &str) -> Vec<Rect> {
    let bounds = list::Bounds::of(list, &mut list::Outlines::default());
    list.commands
        .iter()
        .enumerate()
        .filter_map(|(i, command)| {
            let Command::Glyphs(run) = command else {
                return None;
            };
            (&*run.text == text).then(|| bounds.get(i))?
        })
        .collect()
}

#[test]
fn a_nested_key_is_written_as_its_inner_level() {
    assert_eq!(Nested("2026", "Q1").to_string(), "Q1");
}

#[test]
fn each_run_of_ticks_with_one_group_name_is_labeled_once_under_its_ticks() {
    let (mut fig, quarter) = quarters();
    fig.panel(0, 0)
        .axis(axis::bottom(quarter).group(|key| key.0.to_owned()));
    let (list, _) = drawn(&fig);
    let (first, second) = (ink(&list, "2025"), ink(&list, "2026"));
    assert_eq!((first.len(), second.len()), (1, 1));
    // The quarters: Q3 twice, the others once, all above the years.
    let q3 = ink(&list, "Q3");
    assert_eq!(q3.len(), 2);
    assert!(first[0].y0 > q3[0].y1, "{:?} under {:?}", first[0], q3[0]);
    // 2025 is centered on Q3 and Q4, and 2026 on Q1 to Q3 (its middle Q2).
    let q4 = ink(&list, "Q4")[0];
    let middle = (q3[0].center().x + q4.center().x) / 2.0;
    assert!((first[0].center().x - middle).abs() < 1.0, "{first:?}");
    let q2 = ink(&list, "Q2")[0];
    assert!(
        (second[0].center().x - q2.center().x).abs() < 1.0,
        "{second:?}"
    );
}

#[test]
fn group_labels_take_their_row_from_the_plot_area_and_move_the_title_out() {
    let plain = {
        let (mut fig, quarter) = quarters();
        fig.panel(0, 0).axis(axis::bottom(quarter).label("quarter"));
        drawn(&fig)
    };
    let grouped = {
        let (mut fig, quarter) = quarters();
        let bottom = axis::bottom(quarter).label("quarter");
        fig.panel(0, 0).axis(bottom.group(|key| key.0.to_owned()));
        drawn(&fig)
    };
    // The row is a tick pad and the tick labels' line box: over 12 px.
    let row = plain.1.height() - grouped.1.height();
    assert!((12.0..30.0).contains(&row), "{row}");
    let year = ink(&grouped.0, "2026")[0];
    let title = ink(&grouped.0, "quarter")[0];
    assert!(title.y0 > year.y1, "{title:?} under {year:?}");
}

#[test]
fn ticks_with_an_empty_group_name_are_drawn_as_without_groups() {
    let (mut plain, quarter) = quarters();
    plain.panel(0, 0).axis(axis::bottom(quarter));
    let (mut empty, quarter) = quarters();
    empty
        .panel(0, 0)
        .axis(axis::bottom(quarter).group(|_| String::new()));
    figures_equal(&plain, &empty, SIZE).expect("the same figure");
}

#[test]
fn an_axis_without_labels_has_no_group_labels() {
    let (mut fig, quarter) = quarters();
    let bottom = axis::bottom(quarter).group(|key| key.0.to_owned());
    fig.panel(0, 0).axis(bottom.labels(false));
    let (list, _) = drawn(&fig);
    assert!(ink(&list, "2025").is_empty());
}

#[test]
fn groups_label_a_vertical_axis_beside_its_tick_labels() {
    let (mut fig, quarter) = quarters();
    fig.panel(0, 0)
        .axis(axis::left(quarter).group(|key| key.0.to_owned()));
    let (list, _) = drawn(&fig);
    let year = ink(&list, "2025");
    assert_eq!(year.len(), 1);
    let q4 = ink(&list, "Q4")[0];
    assert!(year[0].x1 < q4.x0, "{:?} left of {q4:?}", year[0]);
}
