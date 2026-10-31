//! Per-point channels: dots colored through a color handle into one
//! `Markers` command with a paint table, sized through a numeric handle,
//! and a channel of the wrong length kept as an error.

use rustd3_core::scale;
use rustd3_core::scheme::{self, COLORBLIND};
use rustd3_figure::kurbo::{Shape, Size};
use rustd3_figure::list::{self, Command, Paints};
use rustd3_figure::mark::dots;
use rustd3_figure::scale::Paint;
use rustd3_figure::{Figure, Held, Target};

use super::{draw, fitted, marked, solid};
use crate::figure::{cache, resolve};

/// The single markers command a figure's marks drew.
fn markers(fig: &Figure<'_>) -> Command {
    let list = draw(fig);
    list::check(&list).expect("a valid list");
    let marked = marked(&list);
    assert_eq!(marked.len(), 1, "{marked:?}");
    marked[0].clone()
}

#[test]
fn fill_by_colors_each_dot_into_one_markers_command_with_a_paint_table() {
    let (mut fig, x, y) = fitted();
    let heat = fig.scale(scale::sequential([0.0, 3.0], scheme::viridis));
    let values = [0.0, 3.0, 3.0, f64::NAN, 1.0];
    fig.panel(0, 0).mark(
        dots(
            [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0)],
            x,
            y,
        )
        .fill_by(heat, values)
        .opacity(0.5),
    );
    let Command::Markers {
        at, sizes, paints, ..
    } = markers(&fig)
    else {
        panic!("markers");
    };
    assert_eq!(at.len(), 4, "the dot at NaN is skipped with its position");
    assert_eq!(sizes, None);
    let r = resolve(&fig, &Held::default());
    let color = |v: f64| {
        let c = r.get_scale(heat).unwrap().paint(&v).expect("a color");
        solid(c.with_opacity(0.5))
    };
    assert_eq!(
        paints,
        Some(Paints {
            table: vec![color(0.0), color(3.0), color(1.0)],
            index: vec![0, 1, 1, 2],
        })
    );
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let resolved = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    let index = fig.hits(&resolved, &mut cache);
    assert_eq!(index.len(), 4, "hits skip the dot at NaN too");
}

#[test]
fn fill_by_fits_a_fitted_color_handle_to_its_values() {
    let (mut fig, x, y) = fitted();
    let heat = fig.fit(scale::sequential([0.0, 1.0], scheme::viridis));
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, 1.0)], x, y).fill_by(heat, [2.0, 8.0]));
    let domain = resolve(&fig, &Held::default())
        .get_scale(heat)
        .unwrap()
        .get_domain();
    assert_eq!(domain, [2.0, 8.0]);
}

#[test]
fn size_by_stamps_a_unit_circle_at_per_point_radii_and_skips_sizes_that_are_not_finite() {
    let (mut fig, x, y) = fitted();
    let radius = fig.scale(scale::linear().domain([0.0, 10.0]).range([2.0, 12.0]));
    fig.panel(0, 0).mark(
        dots([(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)], x, y)
            .size_by(radius, [0.0, 5.0, f64::NAN, 10.0])
            .r(30.0),
    );
    let Command::Markers {
        shape,
        at,
        sizes,
        paint,
        paints,
    } = markers(&fig)
    else {
        panic!("markers");
    };
    assert_eq!(at.len(), 3);
    assert_eq!(sizes, Some(vec![2.0, 7.0, 12.0]));
    let b = shape.bounding_box();
    assert!(
        (b.width() - 2.0).abs() < 1e-6 && b.center().to_vec2().hypot() < 1e-9,
        "{b:?}"
    );
    assert_eq!(paints, None);
    assert_eq!(paint, solid(COLORBLIND[0]));
}

#[test]
fn radii_are_drawn_at_most_three_times_the_plot_areas_larger_side() {
    let (mut fig, x, y) = fitted();
    let radius = fig.scale(scale::linear().domain([0.0, 1.0]).range([0.0, 1e300]));
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, 1.0)], x, y).size_by(radius, [1.0, -1.0]))
        .mark(dots([(0.0, 0.0)], x, y).r(f64::MAX));
    let list = draw(&fig);
    let area = resolve(&fig, &Held::default()).get_plot_areas()[0];
    let most = 3.0 * area.width().max(area.height());
    let markers: Vec<_> = marked(&list);
    let Command::Markers { sizes, .. } = &markers[0] else {
        panic!("markers");
    };
    assert_eq!(sizes.as_deref(), Some(&[most, -most][..]));
    let Command::Markers { shape, .. } = &markers[1] else {
        panic!("markers");
    };
    assert!((shape.bounding_box().width() - 2.0 * most).abs() < 1e-6 * most);
}

#[test]
fn a_dot_is_dropped_when_more_than_its_own_radius_outside_the_plot_area() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let radius = fig.scale(scale::linear().domain([0.0, 1.0]).range([0.0, 100.0]));
    fig.panel(0, 0)
        .mark(dots([(-1.0, 5.0), (-1.0, 5.0)], x, y).size_by(radius, [1.0, 0.01]));
    let Command::Markers { at, sizes, .. } = markers(&fig) else {
        panic!("markers");
    };
    assert_eq!(at.len(), 1);
    assert_eq!(sizes, Some(vec![100.0]));
    assert!(at[0].x < 0.0 && at[0].x > -100.0, "{:?}", at[0]);
}

#[test]
fn a_channel_of_another_length_draws_and_fits_nothing_until_replaced() {
    let (mut fig, x, y) = fitted();
    let heat = fig.scale(scale::sequential([0.0, 1.0], scheme::viridis));
    let radius = fig.scale(scale::linear().range([1.0, 5.0]));
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, 1.0)], x, y))
        .mark(dots([(0.0, 5.0), (1.0, 9.0), (2.0, 7.0)], x, y).fill_by(heat, [0.5, 0.5]))
        .mark(
            dots([(0.0, 0.0), (1.0, 1.0)], x, y)
                .size_by(radius, [0.5])
                .size_by(radius, [0.5, 1.0]),
        );
    let list = draw(&fig);
    let counts: Vec<usize> = marked(&list)
        .into_iter()
        .map(|c| {
            let Command::Markers { at, .. } = c else {
                panic!("not markers: {c:?}");
            };
            at.len()
        })
        .collect();
    assert_eq!(counts, [2, 2], "the mismatched mark draws nothing");
    let domain = resolve(&fig, &Held::default())
        .get_scale(y)
        .unwrap()
        .get_domain();
    assert_eq!(domain, [-0.05, 1.05], "nor does it fit");
    #[cfg(feature = "fonts")]
    {
        use rustd3_figure::check::Issue;
        use rustd3_figure::kurbo::Size;
        use rustd3_figure::mark::Error;
        assert_eq!(
            fig.check(Size::new(640.0, 480.0)),
            [Issue::Mark {
                panel: 0,
                mark: 1,
                error: Error::ChannelLength {
                    points: 3,
                    values: 2
                },
            }]
        );
    }
}
