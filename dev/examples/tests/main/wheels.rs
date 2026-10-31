//! The wheel figure: its samples, marks, bars and
//! fitted domains against `dev/fixtures/wheels.json` (written by
//! `wheels.py`), its checks at two sizes, its labels and lowering, text
//! snapshots of its list and SVG, and its PDF through poppler.

use rustd3_examples::wheels;
use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::{Shape, Size, Vec2};
use rustd3_figure::list::{self, Clip, Command, List, part};
use rustd3_figure::lower::{self, Caps};
use rustd3_figure::raster::{self, Image, Settings, Simd};
use rustd3_figure::theme::{Context, Style};
use rustd3_figure::{Figure, Held, Resolved, Target, Theme, pdf, svg};
use rustd3_testkit::fixture::Fixture;
use rustd3_testkit::obs::{Obs, ToObs, Tolerance};
use rustd3_testkit::pixels::{self, Diff};
use rustd3_testkit::snapshot::Store;

use crate::poppler;

/// The fixture, read at compile time so the test needs no working directory.
const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fixtures/wheels.json"
));

/// The role a figure gives its marks' parts.
const MARK: u64 = 6;

/// The role a figure gives its panels' titles.
const HEADING: u64 = 5;

/// The role a figure gives its legends.
const LEGEND: u64 = 7;

/// The fixture's value at `pointer`.
fn fixture(pointer: &str) -> Obs {
    Fixture::parse(FIXTURE)
        .and_then(|f| f.get(pointer))
        .expect("the fixture's value")
}

/// The wheel figure, its legend below the grid or inset in FL's scatter.
fn wheel_figure(inset: bool) -> Figure<'static> {
    wheels::figure(&wheels::simulate(), inset)
}

/// `fig` resolved and drawn at the example's size for vector output.
fn drawn(fig: &Figure<'_>) -> (Resolved, List) {
    let mut cache = Cache::default();
    let held = Held::default();
    let resolved = fig.resolve(wheels::SIZE, &mut cache, &held, Target::Vector);
    let list = fig.draw_resolved(&mut cache, &resolved);
    list::check(&list).expect("a valid list");
    (resolved, list)
}

/// The role (top byte) and panel (next three bytes) of a part's id.
fn role_and_panel(id: part::Id) -> (u64, u64) {
    let below = |v: u64| part::Id::new(v) <= id;
    let role = (0..256).rev().find(|r| below(r << 56)).expect("a role");
    let panel = (0..256)
        .rev()
        .find(|p| below(role << 56 | p << 32))
        .expect("a panel");
    (role, panel)
}

/// The commands of the parts of `list`'s marks, each with its panel.
fn marked(list: &List) -> Vec<(u64, &Command)> {
    list.parts
        .iter()
        .filter_map(|p| match role_and_panel(p.id) {
            (MARK, panel) => Some(
                list.commands[p.commands.clone()]
                    .iter()
                    .map(move |c| (panel, c)),
            ),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn the_hatched_wheel_figure_hatches_its_232_bars_and_legend_squares() {
    let (resolved, list) = drawn(&wheels::hatched(&wheels::simulate(), false));
    let marked = marked(&list);
    let histograms: Vec<(u64, &Command)> = marked
        .iter()
        .filter(|(panel, _)| panel % 3 == 2)
        .copied()
        .collect();
    // After each bar's fill: a group clipped to the bar, its 45° lines 8 px
    // apart from the figure's origin, the group's end; a bar no such line
    // crosses has none.
    let n = Vec2::new(0.5_f64.sqrt(), 0.5_f64.sqrt());
    let (mut bars, mut hatched) = (0, 0);
    for (i, (panel, c)) in histograms.iter().enumerate() {
        let Command::Fill { path, .. } = c else {
            continue;
        };
        bars += 1;
        let rect = path.bounding_box();
        let next: Vec<&Command> = histograms[i + 1..]
            .iter()
            .take(3)
            .map(|(_, c)| *c)
            .collect();
        if let [
            Command::Push {
                clip: Some(Clip::Rect(r)),
                ..
            },
            Command::Stroke { .. },
            Command::Pop,
        ] = next[..]
        {
            assert_eq!(*r, rect);
            hatched += 1;
        } else {
            let origin = resolved.plot_areas()[*panel as usize].origin().to_vec2();
            let corners = [(rect.x0, rect.y0), (rect.x1, rect.y1)]
                .map(|(x, y)| (Vec2::new(x, y) + origin).dot(n) / 8.0);
            assert!(corners[0].ceil() > corners[1].floor(), "{rect:?}");
        }
    }
    assert_eq!(bars.to_obs(), fixture("/bars"));
    assert_eq!(hatched, 211);
    // The scatters and densities are not hatched.
    let others = marked
        .iter()
        .filter(|(panel, c)| panel % 3 != 2 && matches!(c, Command::Push { .. }))
        .count();
    assert_eq!(others, 0);
    let legend = list
        .parts
        .iter()
        .find(|p| role_and_panel(p.id).0 == LEGEND)
        .expect("the legend");
    let clips = list.commands[legend.commands.clone()]
        .iter()
        .filter(|c| {
            matches!(
                c,
                Command::Push {
                    clip: Some(Clip::Rect(_)),
                    ..
                }
            )
        })
        .count();
    assert_eq!(clips, 3);
}

#[test]
fn the_wheel_figure_draws_5600_dots_in_12_markers_commands_and_232_bars() {
    assert_eq!(wheels::simulate().len().to_obs(), fixture("/samples"));
    let (_, list) = drawn(&wheel_figure(false));
    let marks = list
        .parts
        .iter()
        .filter(|p| role_and_panel(p.id).0 == MARK)
        .count();
    assert_eq!(marks.to_obs(), fixture("/marks"));
    let marked = marked(&list);
    let dots: Vec<usize> = marked
        .iter()
        .filter_map(|(_, c)| {
            if let Command::Markers { at, .. } = c {
                Some(at.len())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(dots.len().to_obs(), fixture("/markers"));
    // Per wheel: 600 accel x, 600 accel z and 200 temperature samples, in
    // the order the figure adds their dots.
    let per_wheel: Vec<Obs> = ["accel x", "accel z", "bearing temp"]
        .map(|s| fixture(&format!("/per_wheel/{s}")))
        .to_vec();
    let expected: Vec<Obs> = (0..4).flat_map(|_| per_wheel.clone()).collect();
    assert_eq!(dots.to_obs(), Obs::List(expected));
    // Histogram panels are each wheel's third (main, side, below).
    let bars = marked
        .iter()
        .filter(|(panel, c)| panel % 3 == 2 && matches!(c, Command::Fill { .. }))
        .count();
    assert_eq!(bars.to_obs(), fixture("/bars"));
}

#[test]
fn the_fitted_domains_match_the_fixture() {
    let samples = wheels::simulate();
    for inset in [false, true] {
        let fig = wheels::figure(&samples, inset);
        let (resolved, _) = drawn(&fig);
        // vib, temp and count are the second to fourth handles.
        for (index, name) in [(1, "vib"), (2, "temp"), (3, "count")] {
            let scale = resolved.scale_at(index).expect("a handle");
            let linear = scale
                .as_any()
                .downcast_ref::<rustd3_core::scale::Linear>()
                .expect("a linear scale");
            let domain = linear.get_domain().to_obs();
            let expected = fixture(&format!("/domains/{name}"));
            assert!(
                domain.approx_eq(&expected, &Tolerance::at_least(1e-4)),
                "{name}: {domain:?} against {expected:?}"
            );
        }
    }
}

#[test]
fn the_wheel_figure_checks_clean_in_at_most_four_layout_passes() {
    for inset in [false, true] {
        let fig = wheel_figure(inset);
        for (w, h) in [(960.0, 760.0), (1600.0, 1200.0)] {
            assert_eq!(fig.check(Size::new(w, h)), [], "{w} × {h}, inset {inset}");
            let resolved = fig.resolve(
                Size::new(w, h),
                &mut Cache::default(),
                &Held::default(),
                Target::Vector,
            );
            assert!(resolved.passes() <= 4, "{} passes", resolved.passes());
        }
    }
}

/// seaborn's five styles.
const STYLES: [Style; 5] = [
    Style::DarkGrid,
    Style::WhiteGrid,
    Style::Dark,
    Style::White,
    Style::Ticks,
];

#[test]
fn every_style_checks_clean_at_paper_and_notebook_on_960_by_760() {
    let samples = wheels::simulate();
    for inset in [false, true] {
        for style in STYLES {
            for context in [Context::Paper, Context::Notebook] {
                let mut fig = wheels::figure(&samples, inset);
                fig.theme(Theme::seaborn(style, context));
                let issues = fig.check(Size::new(960.0, 760.0));
                assert!(issues.is_empty(), "{style:?} {context:?}: {issues:#?}");
            }
        }
    }
}

// The talk context is too big for 960 × 760, and fits 1600 × 1200.
#[test]
fn wheels_fit_at_talk() {
    let samples = wheels::simulate();
    let mut fig = wheels::figure(&samples, false);
    fig.theme(Theme::seaborn(Style::Ticks, Context::Talk));
    let issues = fig.check(Size::new(1600.0, 1200.0));
    assert!(issues.is_empty(), "{issues:#?}");
}

#[test]
fn the_wheel_figures_64_labels_are_64_runs_of_365_glyphs_in_one_font() {
    let (_, list) = drawn(&wheel_figure(false));
    let runs: Vec<_> = list
        .commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect();
    let glyphs: usize = runs.iter().map(|r| r.glyphs.len()).sum();
    // One title, four panel titles ("Wheel: RL" two runs), 50 tick
    // labels, 8 axis titles, 4 rule labels and the legend's 4.
    assert_eq!((runs.len(), glyphs, list.fonts.len()), (64, 365, 1));
}

#[test]
fn the_wheel_figures_four_titles_share_two_baselines_one_per_row() {
    let (resolved, list) = drawn(&wheel_figure(false));
    // Per title part, its panel and its runs' baselines ("Wheel: RL" is
    // two runs).
    let titles: Vec<(u64, Vec<f64>)> = list
        .parts
        .iter()
        .filter_map(|p| match role_and_panel(p.id) {
            (HEADING, panel) => {
                let baselines = list.commands[p.commands.clone()]
                    .iter()
                    .filter_map(|c| {
                        if let Command::Glyphs(run) = c {
                            Some(run.transform.translation().y)
                        } else {
                            None
                        }
                    })
                    .collect();
                Some((panel, baselines))
            }
            _ => None,
        })
        .collect();
    // FL, FR, RL, RR: each wheel's scatter, the first of its three panels.
    let panels: Vec<u64> = titles.iter().map(|(panel, _)| *panel).collect();
    assert_eq!(panels, [0, 3, 6, 9]);
    let rows: Vec<f64> = titles
        .iter()
        .map(|(_, baselines)| {
            assert!(baselines.windows(2).all(|w| w[0] == w[1]), "{baselines:?}");
            baselines[0]
        })
        .collect();
    assert_eq!((rows[0], rows[2]), (rows[1], rows[3]), "{rows:?}");
    // Each row's baseline sits the title pad (8 px) and DejaVu Sans's
    // descender at 16 px above its scatters, which have no top axes.
    let descender = 492.0 / 2048.0 * 16.0;
    let areas = resolved.plot_areas();
    for (row, panel) in [(rows[0], 0), (rows[2], 6)] {
        let expected = areas[panel].y0 - 8.0 - descender;
        assert!((row - expected).abs() < 1e-9, "{row} against {expected}");
    }
}

#[test]
fn lowering_the_markers_grows_the_list_by_5588_commands() {
    let (_, list) = drawn(&wheel_figure(false));
    let caps = Caps {
        markers: false,
        ..Caps::ALL
    };
    let lowered = lower::lower(&list, caps).expect("a valid list");
    assert!(lowered.warnings.is_empty());
    assert_eq!(
        (list.commands.len(), lowered.list.commands.len()),
        (428, 428 + 5_600 - 12)
    );
}

#[test]
fn the_wheel_list_and_svg_match_their_snapshots() {
    let store = Store::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots"));
    let (_, list) = drawn(&wheel_figure(false));
    let dump = list::dump(&list, list::dump::Options::default().glyphs(true));
    store.check("wheels.txt", &dump);
    let lowered = lower::lower(&list, svg::CAPS).expect("a valid list");
    let svg = svg::write(&lowered.list, &svg::Options::default()).expect("a valid list");
    store.check("wheels.svg", &svg);
}

/// `list` at 2× on one thread at the baseline SIMD level: the pinned
/// reference a PDF's poppler render is compared with.
fn pinned(list: &List) -> Image {
    let bounds = list::Bounds::of(list, &mut list::Outlines::default());
    let area = rustd3_figure::kurbo::Rect::from_origin_size((0.0, 0.0), list.size);
    raster::Renderer::new(Settings::default().simd(Simd::Baseline))
        .render(list, &bounds, area, 2.0)
        .expect("the list renders")
}

/// `pdf` rendered by `pdftoppm -r 192` against `reference`.
fn round_trip(pdf: &[u8], reference: &Image) -> Diff {
    let back = poppler::render(pdf, 192.0);
    let a = pixels::Image::new(back.width, back.height, &back.rgba).expect("whole pixels");
    let b = pixels::Image::new(reference.width, reference.height, &reference.rgba)
        .expect("whole pixels");
    Diff::between(&a, &b, 16).expect("one size")
}

/// Whitespace runs as one space, ends trimmed.
fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn wheels_pdf() {
    // Tile scores measured with text and with outlined glyphs; the text's
    // are bound by poppler's glyph placement.
    for (name, fig, [text_score, outlined_score]) in [
        ("wheels", wheel_figure(false), [0.0756, 0.0204]),
        (
            "hatched",
            wheels::hatched(&wheels::simulate(), false),
            [0.0756, 0.0224],
        ),
    ] {
        let (_, list) = drawn(&fig);
        let reference = pinned(&list);
        let text = pdf::write(&list, &pdf::Options::default()).expect("the figure writes");
        let outlined =
            pdf::write(&list, &pdf::Options::default().outlined(true)).expect("the figure writes");
        let (t, o) = (
            round_trip(&text, &reference),
            round_trip(&outlined, &reference),
        );
        eprintln!(
            "{name} PDF: {} bytes (outlined {}); tile scores {:.4} at {:?}, outlined {:.4} at {:?}",
            text.len(),
            outlined.len(),
            t.tile_score(),
            t.worst_tile(),
            o.tile_score(),
            o.worst_tile()
        );
        assert!(t.tile_score() <= 1.25 * text_score, "{name}: text");
        assert!(o.tile_score() <= 1.25 * outlined_score, "{name}: outlines");
        assert!(outlined_score <= 0.05);

        let extracted = words(&poppler::text(&text));
        for label in [
            "Hub vibration and bearing temperature",
            "Wheel: RL",
            "speed [km/h]",
            "min. count",
        ] {
            assert!(extracted.contains(label), "{name}: {label:?}");
        }
        for command in &list.commands {
            if let Command::Glyphs(run) = command {
                assert!(
                    extracted.contains(&words(&run.text)),
                    "{name}: {:?}",
                    run.text
                );
            }
        }
        // One font: the 64 runs are all DejaVu Sans.
        let fonts = poppler::fonts(&text);
        assert_eq!(fonts.len(), 1, "{name}: {fonts:?}");
        assert!(
            fonts[0].name.ends_with("+DejaVuSans") && fonts[0].embedded && fonts[0].subset,
            "{name}: {fonts:?}"
        );
        assert_eq!(words(&poppler::text(&outlined)), "");
        assert_eq!(poppler::fonts(&outlined), []);
    }
}
