//! Marks: what dots and rules fit and draw, their colors, and where they
//! sit in a figure's list; areas, ribbons, bars, rects, histograms,
//! densities, dodging and jitter, per-point channels, images, notes and
//! symbols in submodules.

mod area;
mod bar;
mod channel;
mod density;
mod dodge;
mod histogram;
mod image;
mod note;
mod symbol;

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::shape::Curve;
use rustd3_figure::kurbo::{Affine, BezPath, Cap, Join, PathEl, Point, Rect, Size};
use rustd3_figure::list::{self, Clip, Command, List, part};
use rustd3_figure::mark::{Place, Rule, dots, line, rule_x, rule_y};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::scale::ScaleRef;
use rustd3_figure::{Figure, Held, Target};

use crate::figure::{Values, cache};

/// `fig` drawn at 640 × 480 for vector output.
pub fn draw(fig: &Figure<'_>) -> List {
    fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    )
}

/// The role of a figure part: the top byte of its id.
pub fn role(id: part::Id) -> u64 {
    (0..=255)
        .find(|r| id < part::Id::new((r + 1) << 56))
        .unwrap_or(255)
}

/// The commands of the parts whose role is a mark (6).
pub fn marked(list: &List) -> Vec<&Command> {
    list.parts
        .iter()
        .filter(|p| role(p.id) == 6)
        .flat_map(|p| &list.commands[p.commands.clone()])
        .collect()
}

/// The points and paint of a markers command.
fn markers(command: &Command) -> (&[Point], &Brush) {
    let Command::Markers { at, paint, .. } = command else {
        panic!("not markers: {command:?}");
    };
    (at, paint)
}

/// A solid brush of `c`.
fn solid(c: Rgb) -> Brush {
    let [r, g, b] = [c.r, c.g, c.b].map(|v| v as u8);
    Brush::Solid(Color::from_rgba8(
        r,
        g,
        b,
        (c.opacity * 255.0).round() as u8,
    ))
}

/// A figure with fitted linear x and y handles.
fn fitted() -> (Figure<'static>, ScaleRef<Linear>, ScaleRef<Linear>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    (fig, x, y)
}

#[test]
fn three_dots_are_one_markers_command_in_one_part() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)], x, y));
    let list = draw(&fig);
    list::check(&list).expect("a valid list");
    let parts: Vec<_> = list.parts.iter().filter(|p| role(p.id) == 6).collect();
    assert_eq!(parts.len(), 1);
    let commands = &list.commands[parts[0].commands.clone()];
    assert_eq!(commands.len(), 1);
    let (at, paint) = markers(&commands[0]);
    assert_eq!(at.len(), 3);
    assert_eq!(*paint, solid(COLORBLIND[0]));
}

#[test]
fn a_dot_at_nan_is_dropped_and_y_fits_the_others() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0), (1.0, f64::NAN), (2.0, 1.0)], x, y));
    assert_eq!(
        crate::figure::resolve(&fig, &Held::default())
            .get_scale(y)
            .unwrap()
            .get_domain(),
        [-0.05, 1.05]
    );
    let list = draw(&fig);
    let (at, _) = markers(marked(&list)[0]);
    assert_eq!(at.len(), 2);
}

#[test]
fn a_rule_widens_a_fitted_count_domain() {
    let mut fig = Figure::new();
    let count = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(Values {
            on: count,
            values: vec![3.0, 7.0],
            base: Some(0.0),
        })
        .mark(rule_y(20.0, count));
    let domain = crate::figure::resolve(&fig, &Held::default())
        .get_scale(count)
        .unwrap()
        .get_domain();
    assert_eq!(domain, [0.0, 21.0]);
}

#[test]
fn marks_draw_inside_one_group_clipped_to_the_plot_area() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .border(Rgb::hex(0x262626))
        .mark(dots([(0.0, 0.0), (1.0, 2.0)], x, y))
        .mark(rule_y(1.0, y));
    let list = draw(&fig);
    let roles: Vec<u64> = list.parts.iter().map(|p| role(p.id)).collect();
    assert_eq!(roles, [0, 2, 6, 6, 3]);
    let area = crate::figure::resolve(&fig, &Held::default()).get_plot_areas()[0];
    let push = &list.commands[list.parts[2].commands.start - 1];
    assert_eq!(
        *push,
        Command::Push {
            transform: Affine::translate(area.origin().to_vec2()),
            clip: Some(Clip::Rect(Rect::from_origin_size(
                Point::ORIGIN,
                area.size()
            ))),
        }
    );
    assert_eq!(list.commands[list.parts[3].commands.end], Command::Pop);
    assert_eq!(list.parts[3].id, part::Id::new(6 << 56 | 1));
    let bounds = list.parts[2].bounds;
    assert!(
        area.inflate(4.0, 4.0).contains(bounds.origin()),
        "{bounds:?}"
    );
}

#[test]
fn a_panel_whose_marks_draw_nothing_gets_no_group() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(dots([(f64::NAN, 0.0)], x, y));
    let list = draw(&fig);
    assert!(
        !list
            .commands
            .iter()
            .any(|c| matches!(c, Command::Push { .. }))
    );
}

#[test]
fn dots_take_a_palette_color_and_the_theme_gray_for_unknown_keys() {
    let (mut fig, x, y) = fitted();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0)], x, y).fill(color, "b"))
        .mark(dots([(1.0, 1.0)], x, y).fill(color, "z"))
        .mark(
            dots([(2.0, 2.0)], x, y)
                .color(Rgb::hex(0x123456))
                .opacity(0.5),
        );
    let list = draw(&fig);
    let paints: Vec<&Brush> = marked(&list).into_iter().map(|c| markers(c).1).collect();
    assert_eq!(
        paints,
        [
            &solid(COLORBLIND[1]),
            &solid(Rgb::hex(0x949494)),
            &solid(Rgb::hex(0x123456).with_opacity(0.5)),
        ]
    );
}

#[test]
fn dots_are_circles_of_the_theme_marker_size_unless_given_a_radius() {
    use rustd3_figure::kurbo::Shape;
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(dots([(0.0, 0.0)], x, y))
        .mark(dots([(1.0, 1.0)], x, y).r(6.0).r(f64::NAN));
    let list = draw(&fig);
    let radii: Vec<f64> = marked(&list)
        .into_iter()
        .map(|c| {
            let Command::Markers { shape, .. } = c else {
                panic!("not markers: {c:?}");
            };
            let b = shape.bounding_box();
            assert!(b.center().to_vec2().hypot() < 1e-9, "{b:?}");
            (b.width() / 2.0 * 1e6).round() / 1e6
        })
        .collect();
    assert_eq!(radii, [4.0, 6.0]);
}

#[test]
fn dots_farther_than_a_radius_outside_the_plot_area_are_dropped() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0).mark(dots(
        [(5.0, 5.0), (-100.0, 5.0), (5.0, 1e12), (10.0, 0.0)],
        x,
        y,
    ));
    let list = draw(&fig);
    let (at, _) = markers(marked(&list)[0]);
    assert_eq!(at.len(), 2);
}

#[test]
fn dots_on_a_band_scale_sit_mid_band_and_keys_need_only_clone() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::band(["front".to_string(), "rear".to_string()]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(dots(
        [("rear".to_string(), 1.0), ("side".to_string(), 2.0)],
        x,
        y,
    ));
    let list = draw(&fig);
    let w = crate::figure::resolve(&fig, &Held::default()).get_plot_areas()[0].width();
    let (at, _) = markers(marked(&list)[0]);
    assert_eq!(at.len(), 1);
    assert!((at[0].x - 0.75 * w).abs() < 1e-9, "{} of {w}", at[0].x);
}

/// The stroked paths of `commands`.
fn strokes(commands: &[&Command]) -> Vec<(Vec<PathEl>, rustd3_figure::kurbo::Stroke)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { path, style, .. } = c {
                Some((path.elements().to_vec(), style.clone()))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn rules_cross_the_plot_area_at_their_value() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(rule_y(2.5, y).width(1.0).dash([6.0, 3.0]))
        .mark(rule_x(5.0, x))
        .mark(rule_x(50.0, x));
    let area = crate::figure::resolve(&fig, &Held::default()).get_plot_areas()[0];
    let list = draw(&fig);
    let strokes = strokes(&marked(&list));
    assert_eq!(strokes.len(), 2);
    let (w, h) = (area.width(), area.height());
    assert_eq!(
        strokes[0].0,
        [
            PathEl::MoveTo(Point::new(0.0, 0.75 * h)),
            PathEl::LineTo(Point::new(w, 0.75 * h)),
        ]
    );
    assert_eq!(strokes[0].1.width, 1.0);
    assert_eq!(strokes[0].1.dash_pattern.as_slice(), [6.0, 3.0]);
    assert_eq!(
        strokes[1].0,
        [
            PathEl::MoveTo(Point::new(0.5 * w, 0.0)),
            PathEl::LineTo(Point::new(0.5 * w, h)),
        ]
    );
    assert!((strokes[1].1.width - 2.0).abs() < 1e-12);
}

#[test]
fn a_rule_label_sits_above_its_right_end_in_the_rule_color() {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    let ink = Rgb::hex(0x262626);
    fig.panel(0, 0)
        .mark(rule_y(5.0, y).color(ink).label("min. count"));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let area = fig
        .resolve(size, &mut cache, &Held::default(), Target::Vector)
        .get_plot_areas()[0];
    let runs: Vec<_> = marked(&list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(runs.len(), 1);
    assert_eq!(&*runs[0].text, "min. count");
    assert_eq!(runs[0].fill, solid(ink));
    let bounds = list::Bounds::of(&list, &mut list::Outlines::default());
    let label = list
        .commands
        .iter()
        .position(|c| matches!(c, Command::Glyphs(run) if &*run.text == "min. count"))
        .and_then(|i| bounds.get(i))
        .expect("the label has ink");
    let line = area.y0 + area.height() / 2.0;
    assert!(
        label.y1 < line && label.y1 > line - 10.0,
        "{label:?} over {line}"
    );
    assert!(
        label.x1 < area.x1 && label.x1 > area.x1 - 10.0,
        "{label:?} in {area:?}"
    );
}

/// The ink of the label "limit" of the rule `rule` draws on a figure whose
/// x and y run over [0, 10], and the plot area, for vector output at 640 ×
/// 480.
fn rule_label(rule: impl Fn(ScaleRef<Linear>, ScaleRef<Linear>) -> Rule<Linear>) -> (Rect, Rect) {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0).mark(rule(x, y).label("limit"));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let area = fig
        .resolve(size, &mut cache, &Held::default(), Target::Vector)
        .get_plot_areas()[0];
    let bounds = list::Bounds::of(&list, &mut list::Outlines::default());
    let ink = list
        .commands
        .iter()
        .position(|c| matches!(c, Command::Glyphs(run) if &*run.text == "limit"))
        .and_then(|i| bounds.get(i))
        .expect("the label has ink");
    (ink, area)
}

#[test]
fn a_rule_label_sits_where_label_at_and_label_offset_put_it() {
    // A horizontal rule at 5 runs through the middle of the plot area.
    let (ink, area) = rule_label(|_, y| rule_y(5.0, y).label_at(Place::Start));
    let line = area.center().y;
    assert!(
        ink.x0 > area.x0 && ink.x0 < area.x0 + 10.0,
        "{ink:?} at the left of {area:?}"
    );
    assert!(ink.y1 < line && ink.y1 > line - 10.0, "{ink:?} over {line}");

    let (ink, area) = rule_label(|_, y| rule_y(5.0, y).label_at(Place::Middle));
    assert!(
        (ink.center().x - area.center().x).abs() < 1.0,
        "{ink:?} in the middle of {area:?}"
    );

    // Moved 20 px down, the label sits under the line.
    let (ink, area) = rule_label(|_, y| {
        rule_y(5.0, y)
            .label_at(Place::Start)
            .label_offset(0.0, 20.0)
    });
    assert!(ink.y0 > area.center().y, "{ink:?} under the line");

    // A vertical rule's label at its start sits at the bottom, right of
    // the line.
    let (ink, area) = rule_label(|x, _| rule_x(5.0, x).label_at(Place::Start));
    let line = area.center().x;
    assert!(
        ink.y1 < area.y1 && ink.y1 > area.y1 - 10.0,
        "{ink:?} at the bottom of {area:?}"
    );
    assert!(
        ink.x0 > line && ink.x0 < line + 10.0,
        "{ink:?} right of {line}"
    );

    // An offset that is not finite leaves the label where it was.
    let (kept, _) = rule_label(|_, y| {
        rule_y(5.0, y)
            .label_offset(4.0, 0.0)
            .label_offset(f64::NAN, 1.0)
    });
    let (moved, _) = rule_label(|_, y| rule_y(5.0, y).label_offset(4.0, 0.0));
    assert_eq!(kept, moved);
}

#[test]
fn a_rule_label_is_shaped_with_the_figure_labels_once() {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0).mark(rule_y(5.0, y).label("limit"));
    let mut cache = cache();
    let size = Size::new(640.0, 480.0);
    fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    let first = cache.get_stats().text;
    assert_eq!((first.compiles, first.misses), (1, 1));
    fig.draw_with(&mut cache, size, Target::Vector, &Held::default());
    assert_eq!(cache.get_stats().text.compiles, 1);
}

#[test]
fn a_rule_outside_the_plot_area_draws_nothing() {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(rule_y(11.0, y).label("off"))
        .mark(rule_y(f64::NAN, y));
    assert!(marked(&draw(&fig)).is_empty());
}

/// The single stroke a figure's marks drew, and the plot area.
fn stroked(fig: &Figure<'_>) -> (BezPath, rustd3_figure::kurbo::Stroke, Brush, Rect) {
    let list = draw(fig);
    let area = crate::figure::resolve(fig, &Held::default()).get_plot_areas()[0];
    let marked = marked(&list);
    assert_eq!(marked.len(), 1, "{marked:?}");
    let Command::Stroke { path, style, paint } = marked[0] else {
        panic!("not a stroke: {:?}", marked[0]);
    };
    (path.clone(), style.clone(), paint.clone(), area)
}

/// Every point of `path`'s elements.
fn vertices(path: &BezPath) -> Vec<Point> {
    path.elements()
        .iter()
        .flat_map(|e| match e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => vec![*p],
            PathEl::QuadTo(a, b) => vec![*a, *b],
            PathEl::CurveTo(a, b, c) => vec![*a, *b, *c],
            PathEl::ClosePath => vec![],
        })
        .collect()
}

#[test]
fn a_line_far_outside_its_domain_stays_within_three_panel_sizes() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).mark(line([(-1e9, 0.0), (1e9, 1.0)], x, y));
    let (path, ..) = stroked(&fig);
    let area = crate::figure::resolve(&fig, &Held::default()).get_plot_areas()[0];
    let (w, h) = (area.width(), area.height());
    let points = vertices(&path);
    assert_eq!(points.len(), 2);
    for p in points {
        assert!(p.is_finite(), "{p:?}");
        assert!(
            (-w..=2.0 * w).contains(&p.x) && (-h..=2.0 * h).contains(&p.y),
            "{p:?} outside the guard band of {w} × {h}"
        );
    }
}

#[test]
fn a_line_breaks_where_a_point_cannot_be_placed() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0).mark(line(
        [(0.0, 0.0), (1.0, f64::NAN), (2.0, 1.0), (3.0, 2.0)],
        x,
        y,
    ));
    let (path, ..) = stroked(&fig);
    let moves = path
        .elements()
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_)))
        .count();
    let lines = path
        .elements()
        .iter()
        .filter(|e| matches!(e, PathEl::LineTo(_)))
        .count();
    assert_eq!((moves, lines), (2, 1));
}

#[test]
fn a_line_joins_its_points_with_its_curve() {
    let (mut fig, x, y) = fitted();
    fig.panel(0, 0)
        .mark(line([(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)], x, y).curve(Curve::StepAfter));
    let (path, ..) = stroked(&fig);
    let p = vertices(&path);
    assert_eq!(p.len(), 5);
    assert_eq!((p[1].y, p[2].x), (p[0].y, p[1].x));
}

#[test]
fn a_line_takes_its_palette_color_width_and_dashes() {
    let (mut fig, x, y) = fitted();
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0).mark(
        line([(0.0, 0.0), (1.0, 2.0)], x, y)
            .stroke(color, "b")
            .width(3.0)
            .width(-1.0)
            .dash([2.0, 1.0])
            .dash([f64::NAN]),
    );
    let (_, style, paint, _) = stroked(&fig);
    assert_eq!(paint, solid(COLORBLIND[1]));
    assert_eq!(style.width, 3.0);
    assert_eq!(style.dash_pattern.as_slice(), [2.0, 1.0]);
    assert_eq!(style.dash_offset, 0.0);
    assert_eq!(
        (style.start_cap, style.end_cap, style.join),
        (Cap::Butt, Cap::Butt, Join::Round)
    );
}

#[test]
fn a_line_wholly_outside_the_guard_band_draws_nothing() {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, 1.0]));
    let y = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0).mark(line([(5.0, 0.0), (6.0, 1.0)], x, y));
    assert!(marked(&draw(&fig)).is_empty());
}
