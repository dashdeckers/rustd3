//! Themes: the builder example of Part I's Themes, defaults and builder
//! validation, sizes and styles as a drawn figure shows them, and nice
//! domains.

use rustd3_core::color::Rgb;
use rustd3_core::jiff::Timestamp;
use rustd3_core::scale;
use rustd3_core::time;
use rustd3_figure::kurbo::{ParamCurve, Rect, Size};
use rustd3_figure::list::{Command, List, part};
use rustd3_figure::mark::{bar_y, line as line_mark};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::theme::{Context, Domains, Style, palette};
use rustd3_figure::{Figure, Held, Target, Theme, axis};

use crate::figure::{cache, line};

/// The roles of a figure's parts: panel backgrounds, axes, axis lines and
/// grid lines.
const PANEL: u64 = 2;
const AXIS: u64 = 4;
const SPINE: u64 = 8;
const GRID: u64 = 9;

#[test]
fn the_builder_example_of_themes_sets_the_figures_theme() {
    let mut fig = Figure::new();
    // seaborn's structure, our defaults
    fig.theme(Theme::default());
    assert_eq!(*fig.get_theme(), Theme::default());

    fig.theme(
        Theme::seaborn(Style::WhiteGrid, Context::Paper)
            .palette(palette::DEEP)
            .font_scale(1.1)
            .domains(Domains::Nice),
    );
    assert_ne!(*fig.get_theme(), Theme::default());
    let mine = vec![Rgb::hex(0x336699), Rgb::hex(0x993366)];
    let by_reference = Theme::default().palette(&mine);
    assert_eq!(by_reference, Theme::default().palette(mine.clone()));
}

#[test]
fn the_default_theme_is_ticks_at_notebook_with_colorblind_and_five_percent_margins() {
    let expected = Theme::seaborn(Style::Ticks, Context::Notebook)
        .palette(palette::COLORBLIND)
        .domains(Domains::Margin(0.05));
    assert_eq!(Theme::default(), expected);
    assert_eq!(Theme::seaborn(Style::Ticks, Context::Notebook), expected);
    assert_ne!(Theme::default().palette(palette::DEEP), expected);
    assert_eq!(palette::COLORBLIND, rustd3_core::scheme::COLORBLIND);
    assert_eq!(
        [0.8, 1.0, 1.5, 2.0],
        [
            Context::Paper,
            Context::Notebook,
            Context::Talk,
            Context::Poster
        ]
        .map(Context::scale)
    );
}

#[test]
fn theme_builders_leave_the_theme_as_it_was_on_input_they_reject() {
    let theme = Theme::default();
    assert_eq!(theme.clone().font_scale(-1.0), theme);
    assert_eq!(theme.clone().palette(Vec::<Rgb>::new()), theme);
    assert_eq!(theme.clone().domains(Domains::Margin(0.5)), theme);
    assert_eq!(theme.clone().font_scale(f64::NAN), theme);
    assert_eq!(theme.clone().domains(Domains::Margin(-0.1)), theme);
    assert_ne!(theme.clone().domains(Domains::Margin(0.0)), theme);
}

/// A panel with a line over x in [0, 10] and y in [0, 3], left and bottom
/// axes, drawn at 640 × 480 for vector output under `theme`.
fn drawn(theme: Theme) -> (List, Rect) {
    let mut fig = Figure::new();
    fig.theme(theme);
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    fig.panel(0, 0)
        .mark(line_mark([(0.0, 0.0), (10.0, 3.0)], x, y))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    let size = Size::new(640.0, 480.0);
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    let resolved = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    (list, resolved.plot_areas()[0])
}

/// The commands of `list`'s parts of role `role`.
fn commands(list: &List, role: u64) -> Vec<&Command> {
    list.parts
        .iter()
        .filter(|p| part::Id::new(role << 56) <= p.id && p.id < part::Id::new((role + 1) << 56))
        .flat_map(|p| &list.commands[p.commands.clone()])
        .collect()
}

/// The sizes and baselines (as `(x, y)` origins) of the glyph runs among
/// `commands`.
fn runs(commands: &[&Command]) -> Vec<(f32, (f64, f64))> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                let t = run.transform.translation();
                Some((run.size, (t.x, t.y)))
            } else {
                None
            }
        })
        .collect()
}

/// The widths and paints of the strokes among `commands`.
fn strokes(commands: &[&Command]) -> Vec<(f64, Brush)> {
    commands
        .iter()
        .filter_map(|c| {
            if let Command::Stroke { style, paint, .. } = c {
                Some((style.width, paint.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// An opaque solid brush of `hex`.
fn solid(hex: u32) -> Brush {
    let [_, r, g, b] = hex.to_be_bytes();
    Brush::Solid(Color::from_rgba8(r, g, b, 255))
}

#[test]
fn theme_contexts_and_the_font_scale_size_the_drawn_text_and_lines() {
    let (notebook, _) = drawn(Theme::default());
    let (talk, _) = drawn(Theme::seaborn(Style::Ticks, Context::Talk));
    let (scaled, _) = drawn(Theme::default().font_scale(1.1));
    // Tick labels: 11 pt at notebook, 11 × 1.5 = 16.5 pt = 22 px at talk,
    // 12.1 pt with a font scale of 1.1.
    for (list, px) in [
        (&notebook, 44.0 / 3.0),
        (&talk, 22.0),
        (&scaled, 12.1 * 4.0 / 3.0),
    ] {
        let sizes = runs(&commands(list, AXIS));
        assert!(!sizes.is_empty());
        for (size, _) in sizes {
            assert!((f64::from(size) - px).abs() < 1e-4, "{size} against {px}");
        }
    }
    // Axis lines: 1.25 pt, times the context, not the font scale.
    let widths = |list: &List| -> Vec<f64> {
        strokes(&commands(list, SPINE))
            .into_iter()
            .map(|(w, _)| w)
            .collect()
    };
    assert_eq!(widths(&notebook), [1.25 * (4.0 / 3.0); 2]);
    assert_eq!(widths(&scaled), widths(&notebook));
    assert_eq!(widths(&talk), [1.25 * (4.0 / 3.0) * 1.5; 2]);
}

#[test]
fn theme_styles_draw_their_panel_grid_axis_lines_and_tick_marks() {
    let ink = 0x262626;
    let rows = [
        (Style::DarkGrid, 0xeaeaf2, Some(0xffffff), None, false),
        (
            Style::WhiteGrid,
            0xffffff,
            Some(0xcccccc),
            Some(0xcccccc),
            false,
        ),
        (Style::Dark, 0xeaeaf2, None, None, false),
        (Style::White, 0xffffff, None, Some(ink), false),
        (Style::Ticks, 0xffffff, None, Some(ink), true),
    ];
    let (ticks, area) = drawn(Theme::default());
    let labels = runs(&commands(&ticks, AXIS));
    for (style, panel, grid, edge, marks) in rows {
        let (list, at) = drawn(Theme::seaborn(style, Context::Notebook));
        assert_eq!(
            at, area,
            "{style:?}: the layout does not depend on the style"
        );
        let face: Vec<&Brush> = commands(&list, PANEL)
            .into_iter()
            .filter_map(|c| {
                if let Command::Fill { paint, .. } = c {
                    Some(paint)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(face, [&solid(panel)], "{style:?}");
        let grids = strokes(&commands(&list, GRID));
        match grid {
            Some(color) => {
                // One stroke of grid lines 1 pt wide.
                assert_eq!(grids, [(4.0 / 3.0, solid(color))], "{style:?}");
            }
            None => assert!(grids.is_empty(), "{style:?}"),
        }
        let spines = strokes(&commands(&list, SPINE));
        match edge {
            Some(color) => assert_eq!(
                spines,
                vec![(1.25 * (4.0 / 3.0), solid(color)); 2],
                "{style:?}"
            ),
            None => assert!(spines.is_empty(), "{style:?}"),
        }
        let axes = commands(&list, AXIS);
        let tick_marks = strokes(&axes);
        assert_eq!(!tick_marks.is_empty(), marks, "{style:?}");
        assert!(tick_marks.iter().all(|(_, c)| *c == solid(ink)));
        // Tick labels sit where the ticks style puts them, marks or not.
        assert_eq!(runs(&axes), labels, "{style:?}");
    }
}

#[test]
fn whitegrid_draws_one_grid_line_per_major_tick_under_the_marks() {
    let (list, area) = drawn(Theme::seaborn(Style::WhiteGrid, Context::Notebook));
    let grid = commands(&list, GRID);
    let Some(Command::Stroke { path, .. }) = grid.first() else {
        panic!("no grid lines: {grid:?}");
    };
    // One vertical line per bottom tick, then one horizontal line per left
    // tick: as many as each axis has labels, at their positions.
    let lines: Vec<_> = path.segments().collect();
    let label = |axis: u64| {
        let id = part::Id::new(AXIS << 56 | axis);
        let part = list.parts.iter().find(|p| p.id == id).expect("an axis");
        runs(
            &list.commands[part.commands.clone()]
                .iter()
                .collect::<Vec<_>>(),
        )
    };
    let (left, bottom) = (label(0), label(1));
    assert_eq!(lines.len(), bottom.len() + left.len());
    for (line, (_, (x, _))) in lines.iter().zip(&bottom) {
        let (a, b) = (line.start(), line.end());
        assert_eq!((a.x, a.y, b.y), (b.x, area.y0, area.y1));
        // Labels are centred on their ticks.
        assert!(a.x > *x, "{a:?} {x}");
    }
    for line in &lines[bottom.len()..] {
        let (a, b) = (line.start(), line.end());
        assert_eq!((a.y, a.x, b.x), (b.y, area.x0, area.x1));
    }
    assert!(bottom.len() >= 6 && left.len() >= 7, "{bottom:?} {left:?}");
    // Grid lines come before the marks.
    let first = |role: u64| {
        list.parts.iter().position(|p| {
            part::Id::new(role << 56) <= p.id && p.id < part::Id::new((role + 1) << 56)
        })
    };
    assert!(first(GRID) < first(6), "{:?} {:?}", first(GRID), first(6));
}

#[test]
fn nice_domains_round_a_linear_handle_to_its_axis_ticks() {
    let mut fig = Figure::new();
    fig.theme(Theme::default().domains(Domains::Nice));
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let free = fig.fit(scale::linear());
    let bars = fig.fit(scale::linear());
    let keys = fig.scale(scale::band(["a", "b"]));
    fig.panel(0, 0)
        .mark(line(x, [0.3, 9.7]))
        .mark(line(y, [0.3, 9.7]))
        .mark(line(free, [0.3, 9.7]))
        .axis(axis::bottom(x).ticks(5))
        .axis(axis::left(y));
    fig.panel(0, 1)
        .mark(bar_y([("a", 3.0), ("b", 7.35)], keys, bars))
        .axis(axis::left(bars).ticks(5));
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    // d3's nice(5) over [0.3, 9.7]: step 2.
    assert_eq!(r.scale(x).get_domain(), [0.0, 10.0]);
    // The left axis's count from its length (432 px tall, 35 px a tick:
    // 12) rounds to step 1.
    assert_eq!(r.scale(y).get_domain(), [0.0, 10.0]);
    // No axis: d3's default 10.
    assert_eq!(r.scale(free).get_domain(), [0.0, 10.0]);
    // Bars stick at 0: [0, 7.35] rounds up only.
    assert_eq!(r.scale(bars).get_domain(), [0.0, 8.0]);
    // Under margins the same data pads by 5%.
    fig.theme(Theme::default());
    let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    let [low, high] = r.scale(x).get_domain();
    assert!((low + 0.17).abs() < 1e-12 && (high - 10.17).abs() < 1e-12);
}

#[test]
fn nice_domains_floor_and_ceil_a_time_handle_to_its_tick_interval() {
    let at = |s: &str| s.parse::<Timestamp>().expect("an instant");
    let mut fig = Figure::new();
    fig.theme(Theme::default().domains(Domains::Nice));
    let t = fig.fit(time::utc());
    let v = fig.fit(scale::linear());
    let data = [
        (at("2026-10-28T09:17:00Z"), 1.0),
        (at("2026-10-28T16:41:00Z"), 2.0),
    ];
    fig.panel(0, 0)
        .mark(line_mark(data, t, v))
        .axis(axis::bottom(t).ticks(5))
        .axis(axis::left(v));
    let size = Size::new(640.0, 480.0);
    let r = fig.resolve(size, &mut cache(), &Held::default(), Target::Vector);
    // d3's tick interval for 5 ticks over 7 h 24 min is the hour
    // (d3-scale 4.0.2's `scaleUtc().nice(5)` in Node).
    let expected = [at("2026-10-28T09:00:00Z"), at("2026-10-28T17:00:00Z")];
    assert_eq!(r.scale(t).get_domain(), expected);
}
