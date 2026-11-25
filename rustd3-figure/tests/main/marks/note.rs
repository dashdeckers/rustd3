//! Notes: typst text at a point in data coordinates, by an anchor and an
//! offset in px, in the theme's ink unless filled.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::list::{self, Command, GlyphRun, List};
use rustd3_figure::mark::{Avoid, dots, note};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::text::{Anchor, anchor};
use rustd3_figure::{Figure, Held, Target};

use super::{role, solid};
use crate::figure::{cache, resolve};

/// A figure whose x and y are fixed to [0, 10].
fn tens() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    (fig, x, y)
}

/// `fig` drawn at 640 × 480 for vector output, and its plot area.
fn drawn(fig: &Figure<'_>) -> (List, Rect) {
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    list::check(&list).expect("a valid list");
    (list, resolve(fig, &Held::default()).get_plot_areas()[0])
}

/// The glyph runs of a list's marks and the union of their ink, list px.
fn runs(list: &List) -> (Vec<GlyphRun>, Rect) {
    let bounds = list::Bounds::of(list, &mut list::Outlines::default());
    let mut ink: Option<Rect> = None;
    let mut runs = Vec::new();
    let marks = list.parts.iter().filter(|p| role(p.id) == 6);
    for i in marks.flat_map(|p| p.commands.clone()) {
        if let Command::Glyphs(run) = &list.commands[i] {
            runs.push(run.clone());
            let b = bounds.get(i).expect("ink");
            ink = Some(ink.map_or(b, |r| r.union(b)));
        }
    }
    (runs, ink.unwrap_or_default())
}

#[test]
fn a_note_starts_its_baseline_at_its_point_in_ink_at_the_label_size() {
    let (mut fig, x, y) = tens();
    fig.panel(0, 0).mark(note((5.0, 5.0), "HI", x, y));
    let (list, area) = drawn(&fig);
    let (runs, ink) = runs(&list);
    assert_eq!(runs.len(), 1);
    assert_eq!(&*runs[0].text, "HI");
    assert_eq!(runs[0].fill, solid(Rgb::hex(0x262626)));
    assert!(
        (runs[0].size - 16.0).abs() < 1e-3,
        "12 pt: {}",
        runs[0].size
    );
    let point = area.center();
    assert!((ink.y1 - point.y).abs() < 0.5, "{ink:?} at {point:?}");
    assert!(
        ink.x0 >= point.x && ink.x0 < point.x + 2.0,
        "{ink:?} at {point:?}"
    );
}

#[test]
fn a_note_places_its_anchor_at_its_point_moved_by_its_offset() {
    let (mut fig, x, y) = tens();
    let above = Anchor {
        h: anchor::Horizontal::Middle,
        v: anchor::Vertical::Bottom,
    };
    let red = Rgb::hex(0xc44e52);
    fig.panel(0, 0).mark(
        note((2.5, 5.0), "HI", x, y)
            .anchor(above)
            .offset(3.0, -6.0)
            .offset(f64::NAN, 0.0)
            .color(red),
    );
    let (list, area) = drawn(&fig);
    let (runs, ink) = runs(&list);
    assert_eq!(runs[0].fill, solid(red));
    let (px, py) = (area.x0 + area.width() / 4.0 + 3.0, area.center().y - 6.0);
    assert!((ink.y1 - py).abs() < 0.5, "{ink:?} at {py}");
    assert!((ink.center().x - px).abs() < 1.5, "{ink:?} at {px}");
}

#[test]
fn a_notes_point_enters_fitting_and_the_text_does_not() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, 1.0)], x, y))
        .mark(note(
            (3.0, 2.0),
            "a long note that runs far to the right",
            x,
            y,
        ));
    let r = resolve(&fig, &Held::default());
    let close = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    assert!(close(r.get_scale(x).unwrap().get_domain(), [-0.15, 3.15]));
    assert!(close(r.get_scale(y).unwrap().get_domain(), [-0.1, 2.1]));
}

#[test]
fn a_note_beyond_the_guard_band_or_outside_its_band_draws_nothing() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["a", "b"]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let color = fig.scale(scale::ordinal(["k"], COLORBLIND));
    fig.panel(0, 0)
        .mark(note(("a", 100.0), "far", x, y))
        .mark(note(("z", 5.0), "lost", x, y))
        .mark(dots([("b", 5.0)], x, y).fill(color, "k"));
    let (list, _) = drawn(&fig);
    let (runs, _) = runs(&list);
    assert!(runs.is_empty(), "{runs:?}");
    #[cfg(feature = "fonts")]
    {
        use rustd3_figure::check::Issue;
        use rustd3_figure::kurbo::Size;
        assert_eq!(
            fig.check(Size::new(640.0, 480.0)),
            [Issue::OutsideDomain {
                handle: x.get_index(),
                count: 1
            }]
        );
    }
}

/// The ink of each of a list's mark glyph runs, in order.
fn inks(list: &List) -> Vec<Rect> {
    let bounds = list::Bounds::of(list, &mut list::Outlines::default());
    let marks = list.parts.iter().filter(|p| role(p.id) == 6);
    marks
        .flat_map(|p| p.commands.clone())
        .filter(|i| matches!(list.commands[*i], Command::Glyphs(_)))
        .filter_map(|i| bounds.get(i))
        .collect()
}

/// Two notes at one point, each with `avoid` when given, drawn.
fn pair(avoid: [Option<Avoid>; 2]) -> List {
    let (mut fig, x, y) = tens();
    for (text, avoid) in ["1.10", "1.12"].into_iter().zip(avoid) {
        let label = note((5.0, 5.0), text, x, y);
        fig.panel(0, 0).mark(match avoid {
            Some(avoid) => label.avoid(avoid),
            None => label,
        });
    }
    drawn(&fig).0
}

#[test]
fn notes_that_do_not_avoid_are_drawn_over_each_other() {
    let inks = inks(&pair([None, None]));
    assert_eq!(inks.len(), 2);
    assert!((inks[0].y0 - inks[1].y0).abs() < 0.5, "{inks:?}");
}

#[test]
fn a_stepping_note_moves_by_its_height_to_the_nearest_free_place() {
    let step = Some(Avoid::Step);
    let inks = inks(&pair([step, step]));
    assert_eq!(inks.len(), 2);
    // The second steps up, the first way tried, and clears the first.
    assert!(inks[1].y1 < inks[0].y0, "{inks:?}");
    assert!(inks[0].y0 - inks[1].y1 < 12.0, "{inks:?}");
    assert!((inks[0].x0 - inks[1].x0).abs() < 0.5, "{inks:?}");
}

#[test]
fn a_dropping_note_is_not_drawn_where_one_was_drawn_before_it() {
    let drop = Some(Avoid::Drop);
    assert_eq!(inks(&pair([drop, drop])).len(), 1);
}

#[test]
fn a_note_that_does_not_avoid_is_avoided_by_none() {
    let inks = inks(&pair([None, Some(Avoid::Drop)]));
    assert_eq!(inks.len(), 2);
}

#[test]
fn notes_apart_keep_their_places_whatever_they_avoid() {
    let (mut fig, x, y) = tens();
    fig.panel(0, 0)
        .mark(note((2.0, 2.0), "a", x, y).avoid(Avoid::Step))
        .mark(note((8.0, 8.0), "b", x, y).avoid(Avoid::Drop));
    let avoiding = inks(&drawn(&fig).0);
    let (mut fig, x, y) = tens();
    fig.panel(0, 0)
        .mark(note((2.0, 2.0), "a", x, y))
        .mark(note((8.0, 8.0), "b", x, y));
    assert_eq!(avoiding, inks(&drawn(&fig).0));
}

#[test]
fn a_stepping_note_with_no_free_place_stays_where_it_was_put() {
    // Nine notes at one point: four steps each way are taken by the
    // second to the ninth, so the tenth has nowhere to go.
    let (mut fig, x, y) = tens();
    for _ in 0..10 {
        fig.panel(0, 0)
            .mark(note((5.0, 5.0), "1.10", x, y).avoid(Avoid::Step));
    }
    let inks = inks(&drawn(&fig).0);
    assert_eq!(inks.len(), 10);
    assert!((inks[9].y0 - inks[0].y0).abs() < 0.5, "{inks:?}");
}
