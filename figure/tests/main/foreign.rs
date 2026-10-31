//! Handles and resolved figures of another figure: what they are given to
//! draws nothing, and `fig.check` reports them; a `Fit::Window` on a
//! categorical scale fits as `Fit::All` and is reported.

use rustd3_core::scale;
use rustd3_core::scheme::{self, COLORBLIND};
use rustd3_figure::brush;
use rustd3_figure::check::Issue;
use rustd3_figure::colorbar::colorbar;
use rustd3_figure::kurbo::Size;
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::list::Command;
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::Fit;
use rustd3_figure::{Figure, Held, Target, axis};

use crate::figure::cache;

const SIZE: Size = Size::new(480.0, 320.0);

/// The commands `fig` draws for vector output at [`SIZE`].
fn commands(fig: &Figure<'_>) -> Vec<Command> {
    fig.draw_with(&mut cache(), SIZE, Target::Vector, &Held::default())
        .commands
}

/// One panel of dots and a line on fixed x and y, with both axes and an
/// inset legend.
fn own() -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(dots([(1.0, 2.0), (5.0, 5.0)], x, y).fill(color, "a"))
        .mark(line([(0.0, 0.0), (10.0, 10.0)], x, y))
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .legend(Inset::TopLeft, legend(color));
    fig
}

#[test]
fn handles_of_another_figure_draw_nothing_and_are_reported() {
    let mut other = Figure::new();
    let ox = other.scale(scale::linear().domain([0.0, 1.0]));
    let oy = other.scale(scale::linear().domain([0.0, 1.0]));
    let palette = other.scale(scale::ordinal(["a"], COLORBLIND));
    let heat = other.scale(scale::sequential([0.0, 1.0], scheme::viridis));

    let plain = commands(&own());
    let mut fig = own();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .axis(axis::top(ox))
        .axis(axis::right(oy))
        .legend(Inset::BottomRight, legend(palette))
        .legend(Inset::TopRight, colorbar(heat))
        .zoom_x(ox)
        .zoom_y(oy)
        .brush(brush::x(ox))
        .mark(dots([(0.5, 0.5)], ox, oy))
        .mark(line([(0.0, 1.0), (1.0, 2.0)], ox, y));
    fig.legend(Slot::Right, colorbar(heat));
    assert_eq!(
        commands(&fig),
        plain,
        "nothing drawn for the foreign handles"
    );

    let resolved = fig.resolve(SIZE, &mut cache(), &Held::default(), Target::Vector);
    assert!(resolved.get_scale(ox).is_none());
    assert_eq!(resolved.get_zooms(0), (&[][..], &[][..]));
    assert!(resolved.get_brush(0).is_none());
    assert!(!fig.owns(ox) && fig.owns(y));

    let mut found: Vec<(Option<usize>, usize)> = fig
        .check(SIZE)
        .into_iter()
        .filter_map(|issue| {
            if let Issue::ForeignHandle { panel, handle } = issue {
                Some((panel, handle))
            } else {
                None
            }
        })
        .collect();
    found.sort_unstable();
    let mut expected = vec![
        (Some(0), ox.get_index()),
        (Some(0), oy.get_index()),
        (Some(0), palette.get_index()),
        (Some(0), heat.get_index()),
        (None, heat.get_index()),
    ];
    expected.sort_unstable();
    assert_eq!(found, expected);
}

#[test]
fn a_palette_of_another_figure_leaves_an_axis_untinted_and_is_reported() {
    let mut other = Figure::new();
    let palette = other.scale(scale::ordinal(["a"], COLORBLIND));
    let tinted = |palette| {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 1.0]));
        let bottom = axis::bottom(x);
        fig.panel(0, 0).axis(match palette {
            Some(p) => bottom.color(p, "a"),
            None => bottom,
        });
        fig
    };
    let fig = tinted(Some(palette));
    assert_eq!(commands(&fig), commands(&tinted(None)));
    assert!(fig.check(SIZE).contains(&Issue::ForeignHandle {
        panel: Some(0),
        handle: palette.get_index()
    }));
}

#[test]
fn a_resolved_figure_of_another_figure_is_resolved_afresh() {
    let fig = own();
    let mut other = Figure::new();
    other.panel(0, 0);
    let mut c = cache();
    c.begin();
    let theirs = other.resolve(SIZE, &mut c, &Held::default(), Target::Vector);
    let list = fig.draw_resolved(&mut c, &theirs);
    assert_eq!(list.commands, commands(&fig));
}

#[test]
fn a_window_on_a_categorical_scale_fits_all_and_is_reported() {
    let fitted = |fit: Option<Fit<()>>| {
        let mut fig = Figure::new();
        let band = scale::band(["mon"]);
        let day = match fit {
            Some(fit) => fig.fit_with(band, fit),
            None => fig.fit(band),
        };
        let y = fig.scale(scale::linear().domain([0.0, 1.0]));
        fig.panel(0, 0)
            .mark(dots([("mon", 0.5), ("tue", 0.7)], day, y));
        let resolved = fig.resolve(SIZE, &mut cache(), &Held::default(), Target::Vector);
        let domain = resolved.get_scale(day).unwrap().get_domain().to_vec();
        (domain, fig.check(SIZE), day.get_index())
    };
    let (windowed, issues, handle) = fitted(Some(Fit::Window(())));
    let (all, plain, _) = fitted(None);
    assert_eq!(windowed, all);
    assert!(issues.contains(&Issue::CategoricalWindow { handle }));
    assert!(!plain.contains(&Issue::CategoricalWindow { handle }));
}
