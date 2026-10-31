//! The figure locale against d3's de-DE output: the labels a figure's own
//! formats write, read back from the glyph runs of the drawing list (each
//! keeps its source text). Domains come from `fig.scale` and tick counts
//! from `.ticks(n)`, so layout cannot change the tick set.

use rustd3_core::jiff::{Timestamp, tz::TimeZone};
use rustd3_core::locale::Locale;
use rustd3_core::{scale, time};
use rustd3_figure::kurbo::{Point, Size};
use rustd3_figure::list::{self, Command, List};
use rustd3_figure::mark::{Ctx, Mark};
use rustd3_figure::text::{Anchor, Key, Style, Text, anchor};
use rustd3_figure::{Figure, Fits, Held, Target, Theme, axis};

use crate::figure::cache;

/// A custom mark that labels a number: written in the figure's locale and
/// colored from its theme's palette, while fitting through [`Fits`] and
/// while drawing through [`Ctx`], so the key it draws is the one it
/// reported.
struct Total(f64);

impl Total {
    /// The label's key under `locale` and `theme`.
    fn key(&self, locale: &Locale, theme: &Theme) -> Key {
        let format = locale.get_number().format(",.1f").expect("a specifier");
        let style = Style {
            size: 12.0,
            bold: false,
            fill: theme.get_palette()[1],
            align: anchor::Horizontal::Middle,
        };
        Key::new(&Text::from(format.format(self.0)), &style)
    }
}

impl<'a> Mark<'a> for Total {
    fn fit(&self, fits: &mut Fits) {
        fits.label(self.key(fits.get_locale(), fits.get_theme()));
    }

    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        if let Some(shaped) = ctx.get_label(&self.key(ctx.get_locale(), ctx.get_theme())) {
            let anchor = Anchor {
                h: anchor::Horizontal::Middle,
                v: anchor::Vertical::Baseline,
            };
            shaped.place(list, Point::new(100.0, 50.0), anchor, 0.0);
        }
    }
}

/// The texts of the glyph runs `fig` draws at 640 × 480 for vector output,
/// in order. The figures here have one axis and no other text, so these
/// are its labels.
fn labels(fig: &Figure<'_>) -> Vec<String> {
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    list.commands
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

/// One panel with a left axis on a linear y over `domain`, `count` ticks,
/// its labels in `locale`, or by `format` when given.
fn left(
    domain: [f64; 2],
    count: usize,
    locale: Locale,
    format: Option<fn(&f64) -> String>,
) -> Figure<'static> {
    let mut fig = Figure::new();
    let y = fig.scale(scale::linear().domain(domain));
    let axis = axis::left(y).ticks(count);
    let axis = match format {
        Some(f) => axis.format(f),
        None => axis,
    };
    fig.locale(locale).panel(0, 0).axis(axis);
    fig
}

#[test]
fn default_tick_labels_follow_the_figure_locale() {
    let thousands = |sep: &str| -> Vec<String> {
        (0..=6)
            .map(|i| match i {
                0 => "0".to_string(),
                i => format!("{}{sep}000", 2 * i),
            })
            .collect()
    };
    let at = |locale| labels(&left([0.0, 12_000.0], 6, locale, None));
    // The neutral default groups with U+202F; d3's en-US and de-DE
    // `,.0f` ticks of [0, 12000] at count 6 (d3 in Node).
    assert_eq!(at(Locale::default()), thousands("\u{202f}"));
    assert_eq!(at(Locale::EN_US), thousands(","));
    assert_eq!(at(Locale::DE_DE), thousands("."));
}

#[test]
fn fractions_use_the_locale_decimal() {
    let de = labels(&left([0.0, 1.0], 5, Locale::DE_DE, None));
    assert_eq!(de, ["0,0", "0,2", "0,4", "0,6", "0,8", "1,0"]);
}

#[test]
fn negative_labels_use_the_unicode_minus() {
    let de = labels(&left([-1.0, 1.0], 5, Locale::DE_DE, None));
    assert_eq!(de, ["\u{2212}1,0", "\u{2212}0,5", "0,0", "0,5", "1,0"]);
}

#[test]
fn a_user_format_is_not_localized() {
    let own = labels(&left(
        [-1.0, 1.0],
        5,
        Locale::DE_DE,
        Some(|v| format!("{v:.1}")),
    ));
    assert_eq!(own, ["-1.0", "-0.5", "0.0", "0.5", "1.0"]);
}

/// A figure time handle labels through the time half of the figure
/// locale: DE_DE keeps the ISO ladder, as the neutral default does.
#[test]
fn time_axes_keep_the_iso_ladder_under_de_de() {
    let berlin = rustd3_testkit::zone("Europe/Berlin");
    let day: [Timestamp; 2] =
        ["2026-10-24T22:00:00Z", "2026-10-25T23:00:00Z"].map(|s| s.parse().expect("an instant"));
    let ladder = [
        "10-25", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00", "10-26",
    ];
    for locale in [Locale::default(), Locale::DE_DE] {
        let mut fig = Figure::new();
        let x = fig.scale(time::time(berlin.clone()).domain(day));
        fig.locale(locale)
            .panel(0, 0)
            .axis(axis::bottom(x).ticks(8));
        assert_eq!(labels(&fig), ladder);
    }
}

/// DE_DE with its time half on d3's ladder (`with_time`, `ladder`)
/// labels the evaluation's German axis in German days and months, with
/// no hand-written format.
#[test]
fn de_de_on_d3s_ladder_labels_german_days_and_months() {
    let berlin = rustd3_testkit::zone("Europe/Berlin");
    let span: [Timestamp; 2] =
        ["2026-09-20T22:00:00Z", "2026-10-05T22:00:00Z"].map(|s| s.parse().expect("an instant"));
    let de = Locale::DE_DE.with_time(time::Locale::DE_DE.ladder(time::Ladder::D3));
    let mut fig = Figure::new();
    let x = fig.scale(time::time(berlin).domain(span));
    fig.locale(de).panel(0, 0).axis(axis::bottom(x).ticks(5));
    let german = [
        "Mo 21", "Mi 23", "Fr 25", "Sep 27", "Di 29", "Oktober", "Sa 03", "Mo 05",
    ];
    assert_eq!(labels(&fig), german);
}

#[test]
fn a_custom_mark_labels_in_the_figure_locale_and_palette_while_fitting() {
    let mut fig = Figure::new();
    fig.locale(Locale::DE_DE).panel(0, 0).mark(Total(1234.5));
    assert_eq!(labels(&fig), ["1.234,5"]);
    let list = fig.draw_with(
        &mut cache(),
        Size::new(640.0, 480.0),
        Target::Vector,
        &Held::default(),
    );
    let fills: Vec<_> = list
        .commands
        .iter()
        .filter_map(|c| {
            if let Command::Glyphs(run) = c {
                Some(run.fill.clone())
            } else {
                None
            }
        })
        .collect();
    let second = Theme::default().get_palette()[1];
    assert_eq!(fills, [list::brush(second)]);
}

/// The rest of the section's de-DE rows, d3-format and d3-time-format
/// 4.1.0's output in Node: the tables the figure locale pairs.
#[test]
fn de_de_writes_numbers_and_dates_as_d3() {
    let number = |spec: &str, v: f64| {
        let f = Locale::DE_DE
            .get_number()
            .format(spec)
            .expect("a specifier");
        f.format(v)
    };
    assert_eq!(number(",.1f", 1_234_567.5), "1.234.567,5");
    assert_eq!(number("$,.2f", 1234.5), "1.234,50\u{a0}\u{20ac}");
    assert_eq!(number(",.1f", -0.5), "\u{2212}0,5");
    let t: Timestamp = "2026-10-25T09:05:00Z".parse().expect("an instant");
    let c = Locale::DE_DE
        .get_time()
        .format("%c")
        .format(t, &TimeZone::UTC);
    assert_eq!(c, "Sonntag, der 25. Oktober 2026, 09:05:00");
}
