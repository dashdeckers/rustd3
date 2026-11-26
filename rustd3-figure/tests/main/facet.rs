//! `Figure::facet`: figures built on their own, taken in as panels of one
//! grid, their scales shared or their own.

use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Linear, Ordinal};
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::check::{Element, Issue};
use rustd3_figure::kurbo::Size;
use rustd3_figure::layout::Cell;
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::list::Command;
use rustd3_figure::mark::line;
use rustd3_figure::scale::{Fit, ScaleRef};
use rustd3_figure::text::{Text, span};
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

#[test]
fn a_taken_title_goes_to_the_grid_and_leaves_the_facets_panel_untitled() {
    let mut grid = Figure::new();
    for (col, top) in [10.0, 40.0].into_iter().enumerate() {
        let (mut fig, _) = plot(top);
        let title = fig.take_title();
        assert_eq!(fig.get_title(), None);
        if grid.get_title().is_none()
            && let Some(title) = title
        {
            grid.title(title);
        }
        fig.panel(0, 0).title(format!("value {col}"));
        grid.facet(Cell::new(0, col), fig, Share::Scales);
    }
    let drawn = texts(&grid);
    let count = |t: &str| drawn.iter().filter(|d| *d == t).count();
    assert_eq!(
        [
            count("top 10"),
            count("top 40"),
            count("value 0"),
            count("value 1")
        ],
        [1, 0, 1, 1]
    );
}

#[test]
fn a_figure_reports_its_locale_and_the_rows_and_columns_its_panels_reach() {
    let mut fig = Figure::new();
    assert_eq!(fig.get_extent(), (0, 0));
    assert_eq!(*fig.get_locale(), Locale::default());
    fig.locale(Locale::DE_DE);
    assert_eq!(*fig.get_locale(), Locale::DE_DE);
    // Two facets of a column of two panels each, stepped by their extent.
    let pair = || {
        let (mut fig, (x, y, _)) = plot(1.0);
        fig.panel(1, 0).mark(line([(0.0, 0.0), (1.0, 2.0)], x, y));
        fig
    };
    let mut grid = Figure::new();
    let mut row = 0;
    for _ in 0..2 {
        let facet = pair();
        let (rows, _) = facet.get_extent();
        assert_eq!(facet.get_extent(), (2, 1));
        grid.facet(Cell::new(row, 0), facet, Share::None);
        row += rows;
    }
    assert_eq!(grid.get_extent(), (4, 1));
    assert_eq!(resolve(&grid, &Held::default()).get_plot_areas().len(), 4);
}

/// The labels of `fig`'s inset legend `index` of panel `panel`.
fn inset_labels(fig: &Figure<'_>, panel: u64, index: u64) -> Vec<String> {
    let list = fig.draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    let id = rustd3_figure::list::part::Id::new(7 << 56 | panel << 32 | index);
    list.parts
        .iter()
        .filter(|p| p.id == id)
        .flat_map(|p| &list.commands[p.commands.clone()])
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.text.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// A facet whose legend names its curves `keys`, with a count each, on
/// fixed x and y as `plot` draws them.
fn counted(keys: &[&'static str]) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    let color = fig.scale(scale::ordinal(keys.to_vec(), COLORBLIND));
    let panel = fig.panel(0, 0);
    for (i, key) in keys.iter().enumerate() {
        let v = i as f64 / 4.0;
        panel.mark(line([(0.0, v), (1.0, v)], x, y).stroke(color, *key));
    }
    panel
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .legend(Inset::TopLeft, legend(color));
    fig
}

#[test]
fn shared_facets_keep_their_own_legends_entries() {
    let facets: [&[&'static str]; 3] = [&["FL  12", "FR   3"], &[], &["RL 340"]];
    let mut fig = Figure::new();
    for (col, keys) in facets.into_iter().enumerate() {
        fig.facet(Cell::new(0, col), counted(keys), Share::Scales);
    }
    assert_eq!(inset_labels(&fig, 0, 0), ["FL  12", "FR   3"]);
    assert_eq!(inset_labels(&fig, 1, 0), Vec::<String>::new());
    assert_eq!(inset_labels(&fig, 2, 0), ["RL 340"]);
    // The position scales are still one.
    let r = resolve(&fig, &Held::default());
    assert!(!r.get_plan(1, 1).expect("the middle y axis").labeled);
}

#[test]
fn an_inset_legend_that_does_not_fit_is_drawn_and_reported() {
    let long = "a legend entry far wider than its small panel, counted 1234";
    let mut fig = Figure::new();
    for col in 0..4 {
        fig.facet(Cell::new(0, col), counted(&[long]), Share::Scales);
    }
    for panel in 0..4 {
        assert_eq!(inset_labels(&fig, panel, 0), [long], "panel {panel}");
    }
    let issues = fig.check(SIZE);
    assert!(
        issues.iter().any(|i| matches!(
            i,
            Issue::Overflow {
                what: Element::Legend,
                ..
            }
        )),
        "{issues:?}"
    );
}

#[test]
fn a_panel_title_set_after_facet_stays_in_its_own_font() {
    let (mut plot_fig, _) = plot(10.0);
    plot_fig.title("the plot");
    let mut grid = Figure::new();
    grid.facet(Cell::new(0, 0), plot_fig, Share::Scales);
    let mono = Text::from(span("wheel= FL").font("DejaVu Sans Mono"));
    grid.panel(0, 0).title(mono);
    let list = grid.draw_with(&mut cache(), SIZE, Target::Vector, &Held::default());
    let face = |text: &str| {
        list.commands.iter().find_map(|c| {
            if let Command::Glyphs(run) = c
                && &*run.text == text
            {
                Some(list.fonts[run.font].data.id())
            } else {
                None
            }
        })
    };
    assert_eq!(face("the plot"), None, "replaced");
    let (title, tick) = (face("wheel= FL"), face("0.0"));
    assert!(
        title.is_some() && tick.is_some() && title != tick,
        "{title:?} {tick:?}"
    );
}
