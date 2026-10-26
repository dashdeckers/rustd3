//! Four wheels, three sources, 20 speed bins: scatter
//! panels with an attached histogram below and attached
//! distributions to the right, on shared scales.

use d3::color::Rgb;
use d3::scale;
use figmodel::plot::{Figure, Grid, Inset, Slot};
use figmodel::plot::{Text, bottom, left, legend};
use figmodel::plot::{density_y, dots, histogram};
use figmodel::plot::{line, rule_y, span};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Wheel {
    FL,
    FR,
    RL,
    RR,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Source {
    AccelX,
    AccelZ,
    Temp,
}

/// A 10 km/h speed bin: `Bin(4)` is 40–50 km/h.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bin(u8);

struct Sample {
    wheel: Wheel,
    source: Source,
    bin: Bin,
    value: f64,
}

const SOURCES: [Source; 3] = [
    Source::AccelX,
    Source::AccelZ,
    Source::Temp,
];

/// seaborn's "deep", one color per source.
const PALETTE: [Rgb; 3] = [
    Rgb::hex(0x4c72b0),
    Rgb::hex(0xdd8452),
    Rgb::hex(0x55a868),
];
const RED: Rgb = Rgb::hex(0xc44e52);
const EDGE: Rgb = Rgb::hex(0x4d4d4d);
const DAMAGED: [Wheel; 1] = [Wheel::RL];
/// Bins with fewer samples are not trusted.
const MIN_COUNT: f64 = 20.0;

fn figure(
    samples: &[Sample],
    inset: bool,
) -> Figure {
    let of = |w: Wheel, s: Source| {
        samples.iter().filter(move |d| {
            d.wheel == w && d.source == s
        })
    };

    let mut fig = Figure::new();

    // One scale per quantity, shared by all.
    let speed = fig.scale(
        scale::band((0..20).map(Bin))
            .padding(0.1),
    );
    let vib = fig.fit(scale::linear());
    let temp = fig.fit(scale::linear());
    let count = fig.fit(scale::linear());
    let width = fig.scale(
        scale::linear().domain([0.0, 1.0]),
    );
    let color = fig.scale(
        scale::ordinal(SOURCES, PALETTE),
    );
    // Handles are Copy: route the twin here.
    let y_of = |s: Source| match s {
        Source::Temp => temp,
        _ => vib,
    };

    // Axes and legends are values: define once,
    // place as often as needed.
    let speed_axis = bottom(speed)
        .values((0..20).step_by(4).map(Bin))
        .format(|b| format!("{}", 10 * b.0))
        .label("speed [km/h]");
    let vib_axis = left(vib)
        .label("vibration RMS [m/s²]");
    // The twin sits left too, pushed outward and
    // tinted like its series.
    let temp_axis = left(temp)
        .outward(52.0)
        .color(color, Source::Temp)
        .format(|t| format!("{t:.0}°"))
        .label("bearing temp [°C]");
    let count_axis = left(count)
        .ticks(2)
        .label("count");
    let key = legend(color).title("source");

    // A 4×4 grid: per wheel, a scatter with its
    // distributions right and histogram below.
    // Attached plots sit 4 px off; wheels get
    // gutters.
    fig.layout(
        Grid::new()
            .cols([1.0, 0.25, 1.0, 0.25])
            .rows([1.0, 0.3, 1.0, 0.3])
            .col_gaps([4.0, 28.0, 4.0])
            .row_gaps([4.0, 40.0, 4.0])
            .margin([30.0, 12.0, 48.0, 116.0]),
    );
    fig.title(
        "Hub vibration and bearing temperature",
    );

    let blocks = [
        (Wheel::FL, 0, 0),
        (Wheel::FR, 0, 2),
        (Wheel::RL, 2, 0),
        (Wheel::RR, 2, 2),
    ];
    for (wheel, r, c) in blocks {
        let (west, south) = (c == 0, r == 2);
        let id = span(format!("{wheel:?}"));
        let id = if DAMAGED.contains(&wheel) {
            id.fill(RED)
        } else {
            id
        };

        let main = fig
            .panel(r, c)
            .title(Text::from("Wheel: ").push(id))
            .border(EDGE)
            .axis(vib_axis.labels(west))
            .zoom_y(vib)
            .zoom_y(temp);
        if west {
            main.axis(temp_axis.clone());
        }
        for s in SOURCES {
            let pts: Vec<_> = of(wheel, s)
                .map(|d| (d.bin, d.value))
                .collect();
            let fit = trend(&pts);
            main.mark(
                dots(pts, speed, y_of(s))
                    .fill(color, s)
                    .dodge()
                    .jitter(0.8),
            );
            main.mark(
                line(fit, speed, y_of(s))
                    .stroke(color, s)
                    .dash([6.0, 3.0])
                    .dodge(),
            );
        }

        let side = fig
            .panel(r, c + 1)
            .border(EDGE);
        for s in SOURCES {
            let v = of(wheel, s).map(|d| d.value);
            side.mark(
                density_y(v, y_of(s), width)
                    .fill(color, s),
            );
        }

        let below = fig
            .panel(r + 1, c)
            .border(EDGE)
            .axis(count_axis.labels(west))
            .axis(speed_axis.labels(south));
        for s in SOURCES {
            let bins = of(wheel, s)
                .map(|d| d.bin);
            below.mark(
                histogram(bins, speed, count)
                    .fill(color, s)
                    .dodge(),
            );
        }
        below.mark(
            rule_y(MIN_COUNT, count)
                .dash([4.0, 3.0])
                .color(EDGE)
                .label("min. count"),
        );
    }

    if inset {
        fig.panel(0, 0)
            .legend(Inset::TopLeft, key);
    } else {
        fig.legend(Slot::Bottom, key);
    }
    fig
}

/// Least squares of value on bin index, at the
/// first and last bin with data.
fn trend(pts: &[(Bin, f64)]) -> [(Bin, f64); 2] {
    let n = pts.len() as f64;
    let x = |b: Bin| b.0 as f64;
    let mean = |f: &dyn Fn(&(Bin, f64)) -> f64| {
        pts.iter().map(f).sum::<f64>() / n
    };
    let mx = mean(&|p| x(p.0));
    let my = mean(&|p| p.1);
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for &(b, y) in pts {
        sxy += (x(b) - mx) * (y - my);
        sxx += (x(b) - mx).powi(2);
    }
    let slope = sxy / sxx;
    let at = |b: u8| {
        let b = Bin(b);
        (b, my + slope * (x(b) - mx))
    };
    let bins = pts.iter().map(|p| p.0 .0);
    let lo = bins.clone().min().unwrap();
    let hi = bins.max().unwrap();
    [at(lo), at(hi)]
}

fn main() {
    let samples = simulate();
    let size = [960.0, 760.0];
    for (inset, name) in [(false, "wheels"), (true, "wheels_inset")] {
        let fig = figure(&samples, inset);
        let path = format!("out/{name}.svg");
        std::fs::write(path, fig.svg(size)).unwrap();
    }

    // Wheel over RL's scatter, as the iced glue would
    // route it. The widget keeps the views here; with
    // `on_change` the app would get messages instead.
    let fig = figure(&samples, false);
    let r = fig.resolve(size);
    let rl = fig.rect(&r, 2, 0).unwrap();
    let at = (rl.x + rl.w / 2.0, rl.y + 0.75 * rl.h);
    let views: HashMap<_, _> = fig
        .wheel(&r, at, -20.0)
        .into_iter()
        .collect();
    let mut v: Vec<_> = views.iter().collect();
    v.sort_by_key(|e| e.0);
    println!("zoomed: {v:.2?}");
    let zoomed = fig.svg_with(size, &views);
    std::fs::write("out/wheels_zoomed.svg", zoomed)
        .unwrap();

    let title = Text::from("Wheel: ")
        .push(span("RL").fill(RED));
    println!("typst: {}", title.typst());
}

impl fmt::Display for Bin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lo = self.0 as u32 * 10;
        write!(f, "{lo}–{}", lo + 10)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Source::AccelX => "accel x",
            Source::AccelZ => "accel z",
            Source::Temp => "bearing temp",
        })
    }
}

/// 600 drives per wheel; temperature is logged every
/// third. RL has a worn bearing above 100 km/h.
fn simulate() -> Vec<Sample> {
    let mut seed = 11u32;
    let mut u = move || {
        seed = seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (seed >> 8) as f64 / (1u32 << 24) as f64
    };
    let mut gauss = move || {
        let (a, b) = (u().max(1e-12), u());
        (-2.0 * a.ln()).sqrt()
            * (std::f64::consts::TAU * b).cos()
    };
    let mut out = Vec::new();
    for wheel in [Wheel::FL, Wheel::FR, Wheel::RL, Wheel::RR] {
        let worn = wheel == Wheel::RL;
        for i in 0..600 {
            let city = i % 2 == 0;
            let (m, sd) = if city { (50.0, 15.0) } else { (120.0, 25.0) };
            let v: f64 = (m + sd * gauss()).clamp(0.0, 199.9);
            let bin = Bin((v / 10.0) as u8);
            let fast = (v - 100.0).max(0.0);
            let x = 0.2 + 0.006 * v + 0.08 * gauss();
            let z = 0.3 + 0.009 * v + 0.1 * gauss()
                + if worn { 0.012 * fast } else { 0.0 };
            let mut push = |source, value| {
                out.push(Sample { wheel, source, bin, value })
            };
            push(Source::AccelX, x);
            push(Source::AccelZ, z);
            if i % 3 == 0 {
                let t = 35.0 + 0.12 * v + 2.0 * gauss()
                    + if worn { 0.15 * fast } else { 0.0 };
                push(Source::Temp, t);
            }
        }
    }
    out
}
