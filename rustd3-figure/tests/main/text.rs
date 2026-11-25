//! Figure text: markup from spans and raw typst, fonts, shaping through
//! typst against its measured metrics, and the text cache.

use std::sync::Arc;

use rustd3_core::color::Rgb;
use rustd3_figure::kurbo::{Affine, Point, Rect, Shape, Size};
use rustd3_figure::list::{self, Clip, Command, GlyphRun, List};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::svg;
use rustd3_figure::text::{
    self,
    Anchor,
    Cache,
    Error,
    Fonts,
    Key,
    Shaped,
    Style,
    Text,
    anchor,
    span,
};

use crate::sink::DEJAVU;

/// The bundled fonts. Without the `fonts` feature, the same three files
/// given as extra fonts and read once per process, as the bundled ones
/// are. That keeps their blobs, which part hashes name faces by, the same
/// in every cache.
pub fn fonts() -> Arc<Fonts> {
    #[cfg(feature = "fonts")]
    let fonts = Fonts::bundled();
    #[cfg(not(feature = "fonts"))]
    let fonts = {
        static FILES: std::sync::OnceLock<Arc<Fonts>> = std::sync::OnceLock::new();
        FILES
            .get_or_init(|| {
                Arc::new(
                    Fonts::with([
                        DEJAVU.to_vec(),
                        include_bytes!("../../../rustd3-fonts/DejaVuSans-Bold.ttf").to_vec(),
                        include_bytes!("../../../rustd3-fonts/NewCMMath-Regular.otf").to_vec(),
                    ])
                    .expect("the font files hold faces"),
                )
            })
            .clone()
    };
    fonts
}

/// The spot checks' style: 11 px, black, lines centered.
pub fn style() -> Style {
    Style {
        size: 11.0,
        bold: false,
        fill: Rgb::hex(0),
        align: anchor::Horizontal::Middle,
    }
}

/// The keys of `labels` in the spot checks' style.
fn keys(labels: &[Text]) -> Vec<Key> {
    labels.iter().map(|t| Key::new(t, &style())).collect()
}

/// `f` of `label` shaped alone in a fresh cache.
fn shaped<R>(label: impl Into<Text>, f: impl FnOnce(&Shaped) -> R) -> R {
    let key = Key::new(&label.into(), &style());
    let mut cache = Cache::new(fonts());
    cache.prepare([&key]);
    f(cache.get(&key).expect("prepared").expect("shaped"))
}

/// The anchor at the box's top-left corner.
const TOP_LEFT: Anchor = Anchor {
    h: anchor::Horizontal::Start,
    v: anchor::Vertical::Top,
};

/// The list `shaped` places with `anchor` at the origin.
fn placed(shaped: &Shaped, anchor: Anchor) -> List {
    let mut list = List::default();
    shaped.place(&mut list, Point::ORIGIN, anchor, 0.0);
    list
}

/// The runs of `shaped` with the box's top-left corner at the origin:
/// their translations are the baselines' left ends in the box.
fn runs(shaped: &Shaped) -> Vec<GlyphRun> {
    placed(shaped, TOP_LEFT)
        .commands
        .into_iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run)
            } else {
                None
            }
        })
        .collect()
}

/// The sum of a run's advances.
fn advance(run: &GlyphRun) -> f64 {
    run.glyphs.iter().map(|g| f64::from(g.advance)).sum()
}

/// `v` to three places, as the spot checks quote typst's values.
fn places(v: f64) -> String {
    format!("{v:.3}")
}

#[test]
fn a_plain_label_is_one_string_literal() {
    assert_eq!(Text::from("0123456789").typst(), r#"#"0123456789""#);
    assert_eq!(Text::from(String::from("x")).typst(), r#"#"x""#);
}

#[test]
fn markup_characters_in_spans_stay_literal() {
    assert_eq!(Text::from("a $ # _ b").typst(), r##"#"a $ # _ b""##);
}

#[test]
fn backslashes_quotes_and_control_characters_are_escaped() {
    let text = Text::from("a\\b \"q\"\tc\r\nd");
    assert_eq!(text.typst(), r#"#"a\\b \"q\"\tc\nd""#);
}

#[test]
fn a_filled_span_wraps_its_literal_in_text() {
    let title = Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)));
    assert_eq!(
        title.typst(),
        r##"#"Wheel: "#text(fill: rgb("#c44e52"), "RL")"##
    );
}

#[test]
fn a_translucent_fill_is_written_as_integer_channels() {
    let text = Text::from(span("x").fill(Rgb::new(10.4, 20.6, 300.0).with_opacity(0.5)));
    assert_eq!(text.typst(), r#"#text(fill: rgb(10, 21, 255, 128), "x")"#);
}

#[test]
fn a_bold_span_is_strong_inside_its_fill() {
    assert_eq!(Text::from(span("b").bold()).typst(), r#"#strong("b")"#);
    assert_eq!(
        Text::from(span("b").bold().fill(Rgb::hex(0x4c72b0))).typst(),
        r##"#text(fill: rgb("#4c72b0"), strong("b"))"##
    );
}

#[test]
fn raw_markup_is_inserted_verbatim() {
    assert_eq!(
        text::typst("RMS [$m slash s^2$]").typst(),
        "RMS [$m slash s^2$]"
    );
}

#[test]
fn a_span_before_markup_ends_its_expression() {
    let text = Text::from("a").push(text::typst("[b]")).push("c");
    assert_eq!(text.typst(), r##"#"a";[b]#"c""##);
}

#[test]
fn a_text_without_characters_is_empty() {
    assert!(Text::default().is_empty());
    assert!(Text::from("").push(text::typst("")).is_empty());
    assert!(!Text::from("").push(" ").is_empty());
}

#[cfg(feature = "fonts")]
#[test]
fn the_bundled_fonts_load_once() {
    assert!(Arc::ptr_eq(&Fonts::bundled(), &Fonts::bundled()));
}

#[test]
fn bytes_without_a_face_are_rejected_with_their_position() {
    assert_eq!(
        Fonts::with([DEJAVU.to_vec(), b"not a font".to_vec()]).map(|_| ()),
        Err(Error::Font { index: 1 })
    );
    assert_eq!(
        Fonts::with([Vec::new()]).map(|_| ()),
        Err(Error::Font { index: 0 })
    );
}

#[test]
fn digits_are_0_636_em_wide_and_one_run() {
    shaped("0123456789", |s| {
        assert_eq!(places(s.get_width()), "69.985");
        assert_eq!(places(s.get_ascent() + s.get_descent()), "8.320");
        assert_eq!(runs(s).len(), 1);
    });
}

#[test]
fn the_box_is_the_ink_from_the_first_baseline() {
    shaped("x", |s| {
        assert_eq!(places(s.get_width()), "6.510");
        assert_eq!(places(s.get_ascent()), "6.016");
        assert_eq!(places(s.get_descent()), "0.000");
    });
    shaped("g", |s| {
        assert_eq!(places(s.get_width()), "6.982");
        assert_eq!(places(s.get_ascent()), "6.161");
        assert_eq!(places(s.get_descent()), "2.288");
    });
}

#[test]
fn a_newline_breaks_the_label_into_centred_lines() {
    shaped("line one\nline two", |s| {
        let runs = runs(s);
        assert_eq!(runs.len(), 2);
        let [first, second] = [0, 1].map(|i| runs[i].transform.translation());
        assert_eq!(
            (places(first.x), places(first.y)),
            ("0.000".into(), "8.357".into())
        );
        assert_eq!(
            (places(second.x), places(second.y)),
            ("0.215".into(), "24.021".into())
        );
        assert_eq!(places(s.get_width()), "43.817");
        assert_eq!(places(s.get_ascent() + s.get_descent()), "24.176");
        assert_eq!(places(s.get_last()), places(second.y - first.y));
    });
}

#[test]
fn a_filled_span_is_its_own_run_in_its_color() {
    let title = Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)));
    assert_eq!(
        title.typst(),
        r##"#"Wheel: "#text(fill: rgb("#c44e52"), "RL")"##
    );
    shaped(title, |s| {
        let runs = runs(s);
        assert_eq!(runs.len(), 2);
        assert_eq!(places(advance(&runs[0])), "41.642");
        assert_eq!(places(advance(&runs[1])), "13.771");
        assert_eq!(runs[0].fill, Brush::Solid(Color::from_rgb8(0, 0, 0)));
        assert_eq!(runs[1].fill, Brush::Solid(Color::from_rgb8(196, 78, 82)));
    });
}

#[test]
fn markup_characters_in_a_label_are_drawn_as_text() {
    shaped("a $ # _ b", |s| {
        let runs = runs(s);
        assert_eq!(runs.len(), 1);
        assert_eq!(&*runs[0].text, "a $ # _ b");
        assert_eq!(places(s.get_width()), "49.425");
    });
}

#[test]
fn measured_widths_differ_from_a_per_character_estimate() {
    let width = |label: &str| shaped(label, Shaped::get_width);
    assert_eq!(places(width("vibration RMS [m/s²]")), "113.137");
    assert_eq!(places(width("WWW")), "32.629");
    assert_eq!(places(width("iii")), "9.168");
}

#[test]
fn math_sets_in_the_math_face_with_a_smaller_superscript() {
    shaped(text::typst("RMS [$m slash s^2$]"), |s| {
        let runs = runs(s);
        assert_eq!(runs.len(), 6);
        let list = placed(s, Anchor::default());
        assert_eq!(
            list.fonts.len(),
            2,
            "DejaVu Sans and New Computer Modern Math"
        );
        let mut faces: Vec<usize> = runs.iter().map(|r| r.font).collect();
        faces.dedup();
        assert_eq!(faces, [0, 1, 0]);
        let sizes: Vec<String> = runs.iter().map(|r| format!("{:.2}", r.size)).collect();
        assert_eq!(sizes, ["11.00", "11.00", "11.00", "11.00", "7.70", "11.00"]);
        assert_eq!(places(s.get_width()), "61.510");
        assert_eq!(s.get_last(), 0.0, "one line: the superscript is no line");
    });
}

#[test]
fn stacked_math_keeps_the_baseline_of_its_line() {
    shaped(text::typst("$a/b$"), |s| {
        assert_eq!(
            places(s.get_ascent()),
            "7.730",
            "the math axis, not the numerator's 3.396"
        );
        assert_eq!(places(s.get_ascent() + s.get_descent()), "11.602");
        let list = placed(s, TOP_LEFT);
        let rules: Vec<Rect> = list
            .commands
            .iter()
            .filter_map(|c| {
                if let Command::Fill { path, .. } = c {
                    Some(rustd3_figure::kurbo::Shape::bounding_box(path))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(rules.len(), 1, "the fraction rule");
        assert_eq!(
            [rules[0].x0, rules[0].y0, rules[0].x1, rules[0].y1].map(places),
            ["1.100", "4.760", "5.874", "5.200"],
            "0.44 px thick, centred on typst's line at y 4.980"
        );
    });
    shaped(text::typst("$sqrt(x)$"), |s| {
        let runs = runs(s);
        assert_eq!(&*runs[1].text, "𝑥");
        assert_eq!(
            runs[1].transform.translation().y,
            s.get_ascent(),
            "x sits on the baseline"
        );
    });
}

#[test]
fn a_clipping_box_draws_only_what_it_clips_to() {
    // A disc 10 px across in a box 5 px tall that clips it to its top half.
    shaped(
        text::typst("#box(height: 5pt, clip: true, circle(radius: 5pt, fill: black))"),
        |s| {
            let mut list = List {
                size: Size::new(20.0, 20.0),
                ..List::default()
            };
            s.place(&mut list, Point::new(5.0, 5.0), TOP_LEFT, 0.0);
            let [
                Command::Push {
                    transform,
                    clip: Some(Clip::Path(clip)),
                },
                Command::Fill { path, .. },
                Command::Pop,
            ] = &list.commands[..]
            else {
                panic!("not a clipped disc: {:?}", list.commands);
            };
            assert_eq!(*transform, Affine::IDENTITY);
            let (clip, disc) = (clip.bounding_box(), path.bounding_box());
            assert_eq!(
                [clip.width(), clip.height()].map(places),
                ["10.000", "5.000"]
            );
            assert_eq!(
                [disc.width(), disc.height()].map(places),
                ["10.000", "10.000"]
            );
            assert_eq!(
                [clip.x0, clip.y0].map(places),
                [disc.x0, disc.y0].map(places)
            );

            #[cfg(feature = "raster")]
            {
                let image = rustd3_figure::raster::render(&list, 1.0).expect("the disc renders");
                let inked = |y: u32| {
                    (0..image.width)
                        .any(|x| image.rgba[((y * image.width + x) * 4 + 3) as usize] > 0)
                };
                let rows: Vec<u32> = (0..image.height).filter(|&y| inked(y)).collect();
                assert_eq!(
                    (rows.first(), rows.last()),
                    (Some(&(clip.y0 as u32)), Some(&(clip.y1.ceil() as u32 - 1))),
                    "ink in the clip's rows only"
                );
            }
        },
    );
}

#[test]
fn glyphs_no_font_has_are_counted_missing() {
    shaped("漢字", |s| {
        let runs = runs(s);
        assert_eq!(runs.len(), 1);
        let ids: Vec<u32> = runs[0].glyphs.iter().map(|g| g.id).collect();
        assert_eq!(ids, [0, 0]);
        assert_eq!(s.get_missing(), 2);
    });
}

#[test]
fn a_rejected_label_does_not_stop_its_batch() {
    let labels = [text::typst("$ x^ $"), "ok".into(), "fine".into()];
    let keys = keys(&labels);
    let mut cache = Cache::new(fonts());
    cache.prepare(&keys);
    match cache.get(&keys[0]) {
        Some(Err(Error::Markup { markup, message })) => {
            assert_eq!(markup, keys[0].as_str());
            assert!(markup.contains("$ x^ $"));
            assert!(!message.is_empty());
        }
        other => panic!("expected a markup error, got {other:?}"),
    }
    for key in &keys[1..] {
        assert!(matches!(cache.get(key), Some(Ok(_))));
    }
    assert_eq!(
        cache.get_stats().compiles,
        4,
        "the batch, then each label alone"
    );
    cache.prepare(&keys);
    assert_eq!(cache.get_stats().compiles, 4, "the error is cached");
}

#[test]
fn a_page_break_is_rejected_and_one_outside_the_box_is_two_pages() {
    let inside = Key::new(&text::typst("a#pagebreak()b"), &style());
    let outside = Key::new(&text::typst("a]))#pagebreak()#box([b"), &style());
    let mut cache = Cache::new(fonts());
    cache.prepare([&inside, &outside, &Key::new(&"ok".into(), &style())]);
    assert!(
        matches!(cache.get(&inside), Some(Err(Error::Markup { message, .. })) if message.contains("pagebreak")),
        "{:?}",
        cache.get(&inside)
    );
    assert_eq!(
        cache
            .get(&outside)
            .map(|r| r.map(|_| ()).map_err(Error::clone)),
        Some(Err(Error::Pages {
            markup: outside.as_str().into(),
            pages: 2
        }))
    );
}

#[test]
fn glyph_ids_are_dejavu_sans() {
    shaped("Wheel: RL", |s| {
        let ids: Vec<u32> = runs(s)[0].glyphs.iter().map(|g| g.id).collect();
        assert_eq!(ids, [58, 75, 72, 72, 79, 29, 3, 53, 47]);
    });
}

#[test]
fn the_ink_box_sits_around_its_anchor() {
    shaped("g", |s| {
        let middle_top = Anchor {
            h: anchor::Horizontal::Middle,
            v: anchor::Vertical::Top,
        };
        let b = s.bounds(middle_top);
        assert_eq!(
            [b.x0, b.y0, b.x1, b.y1].map(places),
            ["-3.491", "0.000", "3.491", "8.449"]
        );
        let end_bottom = Anchor {
            h: anchor::Horizontal::End,
            v: anchor::Vertical::Bottom,
        };
        assert_eq!(
            s.bounds(end_bottom),
            Rect::new(-s.get_width(), -s.get_ascent() - s.get_descent(), 0.0, 0.0)
        );
        let middle = Anchor {
            h: anchor::Horizontal::Start,
            v: anchor::Vertical::Middle,
        };
        let m = s.bounds(middle);
        assert!((m.y0 + m.y1).abs() < 1e-12, "centred on the anchor: {m:?}");
        assert_eq!(s.bounds(Anchor::default()).y1, s.get_descent());
    });
}

/// The spot checks' labels.
fn spot_labels() -> Vec<Text> {
    vec![
        "0123456789".into(),
        "x".into(),
        "g".into(),
        "line one\nline two".into(),
        Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52))),
        "a $ # _ b".into(),
        "vibration RMS [m/s²]".into(),
        "WWW".into(),
        "iii".into(),
        text::typst("RMS [$m slash s^2$]"),
        "漢字".into(),
        "Wheel: RL".into(),
    ]
}

#[test]
fn a_second_prepare_of_the_same_labels_compiles_nothing() {
    let keys = keys(&spot_labels());
    let mut cache = Cache::new(fonts());
    cache.prepare(&keys);
    let first = cache.get_stats();
    assert_eq!((first.entries, first.compiles, first.misses), (12, 1, 12));
    cache.prepare(&keys);
    let second = cache.get_stats();
    assert_eq!(second.compiles, first.compiles);
    assert_eq!((second.hits, second.misses), (12, 12));
}

#[test]
fn a_key_twice_in_one_prepare_is_shaped_once() {
    let key = Key::new(&"twice".into(), &style());
    let mut cache = Cache::new(fonts());
    cache.prepare([&key, &key]);
    let stats = cache.get_stats();
    assert_eq!((stats.entries, stats.misses, stats.hits), (1, 1, 1));
}

#[test]
fn labels_unused_for_32_generations_are_dropped() {
    let [kept, dropped] = ["kept", "dropped"].map(|t| Key::new(&t.into(), &style()));
    let mut cache = Cache::new(fonts());
    cache.prepare([&kept, &dropped]);
    for _ in 0..32 {
        cache.begin();
        cache.prepare([&kept]);
    }
    assert!(cache.get(&dropped).is_some(), "32 generations unused");
    cache.begin();
    assert!(cache.get(&dropped).is_none(), "33 begins evict it");
    assert!(cache.get(&kept).is_some());
    assert_eq!(cache.get_stats().entries, 1);
    let compiles = cache.get_stats().compiles;
    cache.prepare([&dropped]);
    assert_eq!(cache.get_stats().compiles, compiles + 1, "shaped again");
}

#[test]
fn begin_runs_often_without_typst_failing() {
    let key = Key::new(&"evicted".into(), &style());
    let mut cache = Cache::new(fonts());
    for _ in 0..100 {
        cache.begin();
        cache.prepare([&key]);
    }
    assert!(matches!(cache.get(&key), Some(Ok(_))));
}

/// Reported, not asserted: a container's timings vary by ±30 %.
#[test]
#[expect(clippy::disallowed_methods, reason = "a test reports wall times")]
fn one_document_per_batch_costs_less_than_one_per_label() {
    use std::time::Instant;

    // Tick labels from `base` on; each arm shapes its own strings, so
    // neither reuses what typst memoized for the other.
    let ticks = |base: f64| {
        let labels: Vec<Text> = (0..40)
            .map(|i| format!("{:.1}", base + 37.0 * f64::from(i)).into())
            .collect();
        keys(&labels)
    };
    Cache::new(fonts()).prepare([&Key::new(&"warm".into(), &style())]);
    let (together, apart) = (ticks(1000.0), ticks(5000.0));
    let mut batched = Cache::new(fonts());
    let start = Instant::now();
    batched.prepare(&together);
    let one = start.elapsed();
    let mut single = Cache::new(fonts());
    let start = Instant::now();
    for key in &apart {
        single.prepare([key]);
    }
    let each = start.elapsed();
    assert_eq!(batched.get_stats().compiles, 1);
    assert_eq!(single.get_stats().compiles, 40);
    let twelve: Vec<Text> = (0..12).map(|i| format!("memo {i}").into()).collect();
    let twelve = keys(&twelve);
    let start = Instant::now();
    Cache::new(fonts()).prepare(&twelve);
    let fresh = start.elapsed();
    let start = Instant::now();
    Cache::new(fonts()).prepare(&twelve);
    let memoized = start.elapsed();
    println!(
        "40 new labels: one document {one:?}, one each {each:?}; \
         12 labels in a new cache {fresh:?}, again (typst memoizes) {memoized:?}"
    );
}

/// One label of [`labels`]: its text and style, where it goes, how it
/// is anchored and rotated.
pub struct Placed {
    /// The label.
    pub text: Text,
    /// Its style.
    pub style: Style,
    /// The anchor point.
    pub at: Point,
    /// Which point of the box is at `at`.
    pub anchor: Anchor,
    /// Clockwise, radians.
    pub angle: f64,
}

/// The labels of the snapshots: plain, multi-line, colored, bold, math
/// with rules, a missing glyph, a translucent span, rotated, at every
/// anchor.
pub fn placements() -> Vec<Placed> {
    use anchor::{Horizontal, Vertical};
    let at = |h, v| Anchor { h, v };
    let sized = |size: f64, align: Horizontal| Style {
        size,
        fill: Rgb::hex(0x262626),
        align,
        ..style()
    };
    let place = |text: Text, style: Style, (x, y): (f64, f64), anchor: Anchor, angle: f64| Placed {
        text,
        style,
        at: Point::new(x, y),
        anchor,
        angle,
    };
    vec![
        place(
            "0123456789".into(),
            sized(11.0, Horizontal::Start),
            (20.0, 30.0),
            at(Horizontal::Start, Vertical::Baseline),
            0.0,
        ),
        place(
            "line one\nline two".into(),
            sized(11.0, Horizontal::Middle),
            (240.0, 20.0),
            at(Horizontal::Middle, Vertical::Top),
            0.0,
        ),
        place(
            Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52))),
            sized(14.6667, Horizontal::Start),
            (460.0, 30.0),
            at(Horizontal::End, Vertical::Baseline),
            0.0,
        ),
        place(
            Text::from("bold ").push(span("RMS").bold()).push(" plain"),
            sized(12.0, Horizontal::Start),
            (20.0, 90.0),
            at(Horizontal::Start, Vertical::Middle),
            0.0,
        ),
        place(
            text::typst("RMS [$m slash s^2$]"),
            sized(16.0, Horizontal::Start),
            (240.0, 100.0),
            at(Horizontal::Middle, Vertical::Middle),
            0.0,
        ),
        place(
            text::typst("$x = (a + b)/2$"),
            sized(14.0, Horizontal::Start),
            (460.0, 130.0),
            at(Horizontal::End, Vertical::Bottom),
            0.0,
        ),
        place(
            text::typst("$sqrt(x^2 + 1)$"),
            sized(14.0, Horizontal::Start),
            (20.0, 160.0),
            at(Horizontal::Start, Vertical::Top),
            0.0,
        ),
        place(
            "漢字 missing".into(),
            sized(11.0, Horizontal::Start),
            (240.0, 210.0),
            at(Horizontal::Middle, Vertical::Baseline),
            0.0,
        ),
        place(
            Text::from(span("half").fill(Rgb::hex(0x55a868).with_opacity(0.5))),
            sized(12.0, Horizontal::Start),
            (20.0, 260.0),
            at(Horizontal::Start, Vertical::Bottom),
            0.0,
        ),
        place(
            "Bold title".into(),
            Style {
                bold: true,
                fill: Rgb::hex(0x4c72b0),
                ..sized(16.0, Horizontal::Middle)
            },
            (240.0, 290.0),
            at(Horizontal::Middle, Vertical::Baseline),
            0.0,
        ),
        place(
            "vibration RMS [m/s²]".into(),
            sized(12.0, Horizontal::Middle),
            (470.0, 300.0),
            at(Horizontal::Start, Vertical::Bottom),
            -std::f64::consts::FRAC_PI_2,
        ),
    ]
}

/// `placements` shaped through one cache and placed into a 480 × 320
/// list, and each label's box as placed, in list px.
pub fn labels(placements: &[Placed]) -> (List, Vec<Rect>) {
    let keys: Vec<Key> = placements
        .iter()
        .map(|p| Key::new(&p.text, &p.style))
        .collect();
    let mut cache = Cache::new(fonts());
    cache.begin();
    cache.prepare(&keys);
    assert_eq!(cache.get_stats().compiles, 1, "one document");
    let mut list = List {
        size: Size::new(480.0, 320.0),
        ..List::default()
    };
    let mut boxes = Vec::new();
    for (p, key) in placements.iter().zip(&keys) {
        let shaped = cache.get(key).expect("prepared").expect("shaped");
        shaped.place(&mut list, p.at, p.anchor, p.angle);
        let placement = Affine::translate(p.at.to_vec2()) * Affine::rotate(p.angle);
        boxes.push(placement.transform_rect_bbox(shaped.bounds(p.anchor)));
    }
    (list, boxes)
}

#[test]
fn list_dump_of_labels() {
    let (list, _) = labels(&placements());
    let dump = list::dump(&list, list::dump::Options::default().glyphs(true));
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check("list_dump_of_labels.txt", &dump);
}

/// Every label's ink lies in its box and reaches the box's top and bottom
/// (the ink edges under typst's `"bounds"` text edges) within a device px.
/// A top, middle or bottom anchor has that edge of the ink on the anchor
/// point, under a rotation too. Nothing is drawn outside the boxes.
#[cfg(feature = "raster")]
#[test]
fn a_vello_render_draws_each_label_in_its_box_at_its_anchor() {
    let placements = placements();
    let (list, boxes) = labels(&placements);
    let scale = 2.0;
    let image = rustd3_figure::raster::render(&list, scale).expect("the labels render");
    let inked = |x: u32, y: u32| image.rgba[((y * image.width + x) * 4 + 3) as usize] > 0;
    let mut claimed = vec![false; (image.width * image.height) as usize];
    for (p, b) in placements.iter().zip(&boxes) {
        let device = Rect::new(b.x0 * scale, b.y0 * scale, b.x1 * scale, b.y1 * scale);
        let search = device.inflate(3.0, 3.0);
        let mut ink = Rect::new(
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for y in search.y0.floor() as u32..search.y1.ceil() as u32 {
            for x in search.x0.floor() as u32..search.x1.ceil() as u32 {
                claimed[(y * image.width + x) as usize] = true;
                if inked(x, y) {
                    ink = ink.union(Rect::new(
                        f64::from(x),
                        f64::from(y),
                        f64::from(x + 1),
                        f64::from(y + 1),
                    ));
                }
            }
        }
        let text = p.text.typst();
        let inside = device.inflate(1.0, 1.0);
        assert!(
            inside.contains(ink.origin()) && inside.contains(Point::new(ink.x1, ink.y1)),
            "{text}: ink {ink:?} outside its box {device:?}"
        );
        // The text's top-to-bottom axis: y, or x under the quarter turn.
        let (ink_edges, box_edges, at) = if p.angle == 0.0 {
            ([ink.y0, ink.y1], [device.y0, device.y1], p.at.y * scale)
        } else {
            ([ink.x0, ink.x1], [device.x0, device.x1], p.at.x * scale)
        };
        for (i, b) in ink_edges.into_iter().zip(box_edges) {
            assert!((i - b).abs() <= 1.0, "{text}: ink edge {i} vs box edge {b}");
        }
        let [top, bottom] = ink_edges;
        let on_anchor = match p.anchor.v {
            anchor::Vertical::Top => Some(top),
            anchor::Vertical::Middle => Some((top + bottom) / 2.0),
            anchor::Vertical::Bottom => Some(bottom),
            anchor::Vertical::Baseline => None,
        };
        if let Some(edge) = on_anchor {
            assert!(
                (edge - at).abs() <= 1.0,
                "{text}: ink at {edge}, anchor at {at}"
            );
        }
    }
    let stray = (0..image.height)
        .flat_map(|y| (0..image.width).map(move |x| (x, y)))
        .filter(|&(x, y)| inked(x, y) && !claimed[(y * image.width + x) as usize])
        .count();
    assert_eq!(stray, 0, "ink outside every box");
}

/// `list` as SVG with runs written as `text`.
fn svg(list: &List, text: svg::Text) -> String {
    svg::write(list, &svg::Options::default().text(text)).expect("the labels need no lowering")
}

#[test]
fn svg_text_elements_of_labels() {
    let (list, _) = labels(&placements());
    let store = rustd3_testkit::snapshot::Store::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots"
    ));
    store.check(
        "svg_text_elements_of_labels.txt",
        &svg(&list, svg::Text::Elements),
    );
}

#[test]
fn a_text_element_carries_the_runs_family_weight_and_positions() {
    let (list, _) = labels(&placements());
    let svg = svg(&list, svg::Text::Elements);
    assert!(!svg.contains("<defs>"), "no glyph outlines");
    assert!(
        svg.contains(
            "<text transform='matrix(1 0 0 1 20 30)' x='0 7 14 21 27.99 34.99 41.99 48.99 55.99 62.99' \
             y='0' font-family='DejaVu Sans' font-size='11' fill='#262626' xml:space='preserve'>0123456789</text>"
        ),
        "{svg}"
    );
    assert!(
        svg.contains("font-family='DejaVu Sans' font-size='16' font-weight='bold' fill='#4c72b0'")
    );
    assert!(
        svg.contains("font-family='NewComputerModern Math' font-size='11.2'"),
        "the math face's typographic family, as its name table spells it"
    );
    assert!(
        svg.contains("xml:space='preserve'> plain</text>"),
        "a leading space kept"
    );
    assert!(svg.contains("fill='#55a868' fill-opacity='0.502'"));
}

/// resvg's render of `svg` at 2×, with the bundled fonts for `<text>`.
fn resvg_render(svg: &str) -> (u32, u32, Vec<u8>) {
    let mut options = resvg::usvg::Options::default();
    for font in [
        DEJAVU,
        include_bytes!("../../../rustd3-fonts/DejaVuSans-Bold.ttf"),
        include_bytes!("../../../rustd3-fonts/NewCMMath-Regular.otf"),
    ] {
        options.fontdb_mut().load_font_data(font.to_vec());
    }
    let tree = resvg::usvg::Tree::from_str(svg, &options).expect("our SVG parses");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(960, 640).expect("a positive size");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(2.0, 2.0),
        &mut pixmap.as_mut(),
    );
    (pixmap.width(), pixmap.height(), pixmap.data().to_vec())
}

#[test]
fn resvg_draws_text_elements_close_to_the_outlines() {
    let (list, _) = labels(&placements());
    let (w, h, outlined) = resvg_render(&svg(&list, svg::Text::Outlines));
    let (_, _, elements) = resvg_render(&svg(&list, svg::Text::Elements));
    let off = outlined
        .as_chunks::<4>()
        .0
        .iter()
        .zip(elements.as_chunks::<4>().0)
        .filter(|(a, b)| a.iter().zip(*b).any(|(c, d)| c.abs_diff(*d) > 16))
        .count();
    let share = off as f64 / f64::from(w * h);
    println!(
        "<text> vs outlines through resvg: {off} pixels ({:.3} %) more than 16/255 off",
        share * 100.0
    );
    assert!(off > 0, "resvg re-shapes the text, so some pixels differ");
    assert!(share <= 0.01, "{off} pixels, {:.3} %", share * 100.0);
}
