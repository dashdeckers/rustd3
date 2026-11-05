//! The chart presets of `helpers::chart` over fixed data, each at 640 ×
//! 480. The `presets` example writes them as files, and the benchmark's
//! figure profile times them.

use std::f64::consts::PI;

use rustd3_core::series::{Clock, Recording};
use rustd3_core::{scale, scheme};
use rustd3_figure::helpers::chart::bar::Mode;
use rustd3_figure::helpers::chart::{self};
use rustd3_figure::kurbo::Size;
use rustd3_figure::mark::Grid;
use rustd3_figure::{Figure, text};

/// The presets' size in logical px: matplotlib's 6.4 × 4.8 in at 100 dpi.
pub const SIZE: Size = Size::new(640.0, 480.0);

/// Every preset's name, in the order [`figures`] gives them.
pub const NAMES: [&str; 11] = [
    "line",
    "line_symlog",
    "scatter",
    "bar_grouped",
    "bar_stacked",
    "bar_diverging",
    "area",
    "pie",
    "donut",
    "heatmap",
    "heatmap_keys",
];

/// The data the line presets borrow: 101 instants 0.1 s apart and a
/// recording of 2,000 samples at 100 Hz.
#[derive(Debug)]
pub struct Data {
    t: Vec<f64>,
    rec: Recording<f64>,
}

impl Data {
    /// The fixed data.
    pub fn new() -> Data {
        let samples: Vec<f64> = (0..2000)
            .map(|i| 50.0 * (f64::from(i) * 0.01).sin().powi(3))
            .collect();
        let clock = Clock::Regular {
            start: 0.0,
            period: 0.01,
        };
        Data {
            t: (0..=100).map(|i| f64::from(i) * 0.1).collect(),
            rec: Recording::new(clock, [samples]).expect("invariant: finite samples"),
        }
    }
}

impl Default for Data {
    fn default() -> Data {
        Data::new()
    }
}

/// A line preset: the recording's RMS on a symlog y with
/// minor ticks, titled with typst math.
pub fn rms(data: &Data) -> chart::Line<'_> {
    let channel = data
        .rec
        .series(0)
        .expect("invariant: the recording has a channel");
    chart::line(channel)
        .y_scale(scale::symlog().constant(1.0))
        .minor(true)
        .title(text::typst("RMS [$m slash s^2$]"))
}

/// The preset `name` (one of [`NAMES`]) over `data`; `None` for another
/// name.
pub fn figure<'a>(data: &'a Data, name: &str) -> Option<Figure<'a>> {
    let t = &data.t;
    let week = || {
        [
            ("mon", vec![3.0, 2.5, 1.0]),
            ("tue", vec![4.0, 1.0, 2.0]),
            ("wed", vec![2.0, 3.5, 0.5]),
            ("thu", vec![5.0, 2.0, 1.5]),
        ]
    };
    let regions = || vec!["north".to_string(), "south".into(), "east".into()];
    let spending = [
        ("rent", 1200.0),
        ("food", 450.0),
        ("travel", 300.0),
        ("other", 150.0),
    ];
    let bars = |mode: Mode| {
        chart::bar(week(), regions())
            .mode(mode)
            .title(name)
            .y_label("orders")
            .figure()
    };
    Some(match name {
        "line" => chart::line(t.iter().map(|&t| (t, t.sin())))
            .series(t.iter().map(|&t| (t, 0.5 * (0.7 * t).cos())), "cos")
            .series(t.iter().map(|&t| (t, 0.1 * t - 0.5)), "trend")
            .title("line")
            .x_label("t [s]")
            .y_label("value")
            .figure(),
        "line_symlog" => rms(data).figure(),
        "scatter" => {
            let points = (0..60).map(|i| {
                let k = f64::from(i);
                (
                    k * 0.37 % 10.0,
                    (k * 1.3).sin() * 1.5 + 0.03 * k + (k * 0.71).cos(),
                )
            });
            chart::scatter(points)
                .title("scatter")
                .x_label("x")
                .y_label("y")
                .figure()
        }
        "bar_grouped" => bars(Mode::Grouped),
        "bar_stacked" => bars(Mode::Stacked),
        "bar_diverging" => chart::bar(
            [
                ("k0", vec![3.0, -1.0, 2.0]),
                ("k1", vec![-2.0, 4.0, 1.0]),
                ("k2", vec![1.5, -2.5, -1.0]),
            ],
            vec!["a".into(), "b".into(), "c".into()],
        )
        .mode(Mode::Diverging)
        .title("bar_diverging")
        .figure(),
        "area" => {
            let hours: Vec<f64> = (0..24).map(f64::from).collect();
            let solar = hours
                .iter()
                .map(|h| (2.5 * (PI * (h - 6.0) / 12.0).sin()).max(0.0))
                .collect();
            let wind = hours.iter().map(|h| 1.5 + 0.5 * (h * 0.4).cos()).collect();
            chart::area(vec![solar, wind, vec![1.0; 24]], hours)
                .title("area")
                .x_label("hour")
                .y_label("GW")
                .figure()
        }
        "pie" => chart::pie(spending).title("pie").figure(),
        "donut" => chart::pie(spending).inner(0.5).title("donut").figure(),
        "heatmap" => {
            let field = (0..60)
                .map(|i| {
                    let (c, r) = (f64::from(i % 10), f64::from(i / 10));
                    (c * 0.6).sin() * (r * 0.5).cos() + 0.1 * r
                })
                .collect();
            chart::heatmap(Grid::new(10, 6, field).expect("invariant: 10 × 6 values"))
                .title("heatmap")
                .x_label("x")
                .y_label("y")
                .figure()
        }
        "heatmap_keys" => {
            let names: Vec<String> = ["speed", "load", "temp", "vibration"]
                .map(String::from)
                .to_vec();
            let correlation = vec![
                1.0, 0.8, -0.2, 0.3, 0.8, 1.0, 0.1, 0.5, -0.2, 0.1, 1.0, -0.6, 0.3, 0.5, -0.6, 1.0,
            ];
            chart::heatmap(Grid::new(4, 4, correlation).expect("invariant: 4 × 4 values"))
                .keys(names.clone(), names)
                .colors(scheme::magma)
                .title("correlation")
                .figure()
        }
        _ => return None,
    })
}

/// Every preset's figure over `data` with its name, in [`NAMES`]' order.
pub fn figures(data: &Data) -> Vec<(&'static str, Figure<'_>)> {
    NAMES
        .into_iter()
        .filter_map(|name| Some((name, figure(data, name)?)))
        .collect()
}
