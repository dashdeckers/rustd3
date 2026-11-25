//! `Figure::facet`: figures built on their own, taken in as panels of one
//! grid, their scales shared or their own.

use rustd3_core::scale::{self, Linear, Ordinal};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::check::Issue;
use rustd3_figure::kurbo::Size;
use rustd3_figure::layout::Cell;
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::list::Command;
use rustd3_figure::mark::line;
use rustd3_figure::scale::{Fit, ScaleRef};
use rustd3_figure::{Figure, Held, Share, Target, axis};

use crate::figure::{cache, resolve};

const SIZE: Size = Size::new(640.0, 480.0);

/// The handles of one facet: x, y and its palette.
type Handles = (
    ScaleRef<Linear>,
    ScaleRef<Linear>,
    ScaleRef<Ordinal<&'static str, rustd3_core::color::Rgb>>,
);

/// A plot built as a figure of its own: a line up to `top` on fitted x and
/// y with both axes, titled, with a palette legend in a corner and below
/// the grid.
fn plot(top: f64) -> (Figure<'static>, Handles) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(["data"], COLORBLIND));
    fig.title(format!("top {top}"));
    fig.legend(Slot::Bottom, legend(color));
    fig.panel(0, 0)
        .mark(line([(0.0, 0.0), (1.0, top)], x, y).stroke(color, "data"))
        .axis(axis::bottom(x).label("x"))
        .axis(axis::left(y).label("y"))
        .legend(Inset::TopLeft, legend(color))
        .zoom_y(y);
    (fig, (x, y, color))
}

/// Two plots, up to 10 and up to 40, side by side.
fn row(share: Share) -> (Figure<'static>, [Handles; 2]) {
    let (left, a) = plot(10.0);
    let (right, b) = plot(40.0);
    let mut fig = Figure::new();
    fig.facet(Cell::new(0, 0), left, share)
        .facet(Cell::new(0, 1), right, share);
    (fig, [a, b])
}

/// The texts `list` draws, in order.
fn texts(fig: &Figure<'_>) -> Vec<String> {
    fig.draw_with(&mut cache(), SIZE, Target::Vector, &Held::default())
        .commands
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

#[test]
fn shared_scales_fit_every_facets_marks_and_label_their_outer_axes() {
    let (fig, [a, b]) = row(Share::Scales);
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.get_plot_areas().len(), 2);
    let top = |h| r.get_scale(h).map(|s: &Linear| s.get_domain()[1]);
    // One y over both facets' lines (40 and a 5% margin).
    assert_eq!(top(a.1), top(b.1));
    assert!(top(a.1).is_some_and(|t| t > 40.0), "{:?}", top(a.1));
    assert_eq!(fig.handle(a.1), fig.handle(b.1));
    // The right panel's y axis is inner: unlabeled.
    let left = r.get_plan(0, 1).expect("the left panel's y axis");
    let right = r.get_plan(1, 1).expect("the right panel's y axis");
    assert!(left.labeled && !right.labeled, "{left:?} {right:?}");
    assert_eq!(fig.check(SIZE), []);
}

#[test]
fn unshared_facets_keep_their_scales_and_their_labels() {
    let (fig, [a, b]) = row(Share::None);
    let r = resolve(&fig, &Held::default());
    let top = |h| r.get_scale(h).map(|s: &Linear| s.get_domain()[1]);
    assert!(top(a.1).is_some_and(|t| t < 40.0), "{:?}", top(a.1));
    assert!(top(b.1).is_some_and(|t| t > 40.0), "{:?}", top(b.1));
    assert_ne!(fig.handle(a.1), fig.handle(b.1));
    let right = r.get_plan(1, 1).expect("the right panel's y axis");
    assert!(right.labeled, "{right:?}");
    assert_eq!(fig.check(SIZE), []);
}

#[test]
fn facets_titles_become_panel_titles_and_their_slot_legends_are_kept_once() {
    let (fig, _) = row(Share::Scales);
    let drawn = texts(&fig);
    let count = |t: &str| drawn.iter().filter(|d| *d == t).count();
    assert_eq!((count("top 10"), count("top 40")), (1, 1));
    // Two inset legends and one bottom legend over the shared palette.
    assert_eq!(count("data"), 3);
    let (fig, _) = row(Share::None);
    let drawn = texts(&fig);
    assert_eq!(drawn.iter().filter(|d| *d == "data").count(), 4);
}

#[test]
fn a_facets_handles_are_the_figures_and_an_unrelated_handle_is_not() {
    let (fig, [a, _]) = row(Share::None);
    assert!(fig.owns(a.0) && fig.owns(a.2));
    let own = fig.handle(a.1).expect("a handle of a facet");
    assert!(fig.owns(own));
    let stranger = Figure::new().scale(scale::linear());
    assert!(!fig.owns(stranger) && fig.handle(stranger).is_none());
    // Holding the figure's own handle holds the facet's scale.
    let mut held = Held::default();
    held.hold(own, scale::linear().domain([0.0, 99.0]));
    let r = resolve(&fig, &held);
    assert_eq!(r.get_scale(a.1).map(|s| s.get_domain()), Some([0.0, 99.0]));
    assert_eq!(r.get_zooms(0).1, [own.get_index()]);
}

#[test]
fn facets_of_facets_and_their_noted_issues_carry_over() {
    let (inner, [a, _]) = row(Share::Scales);
    let mut odd = Figure::new();
    let band = odd.fit_with(scale::band(["p", "q"]), Fit::Window(()));
    odd.panel(0, 0).axis(axis::bottom(band));
    let mut fig = Figure::new();
    fig.facet(Cell::new(0, 0), inner, Share::Scales)
        .facet(Cell::new(1, 0), odd, Share::None);
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.get_plot_areas().len(), 3);
    assert!(r.get_scale(a.1).is_some());
    let at = fig.handle(band).expect("the band").get_index();
    assert!(
        fig.check(SIZE)
            .contains(&Issue::CategoricalWindow { handle: at }),
        "{:?}",
        fig.check(SIZE)
    );
}
