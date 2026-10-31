//! Notes: typst text at a point in data coordinates, by an anchor and an
//! offset in px, in the theme's ink unless filled.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::list::{self, Command, GlyphRun, List};
use rustd3_figure::mark::{dots, note};
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
