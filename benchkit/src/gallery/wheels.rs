//! The wheel figure: four wheels, three sources and 20 speed bins. Each
//! wheel has a scatter of value over speed bin, with its distributions
//! attached on the right and a histogram of counts attached below, all on
//! shared scales. The result is a 4 × 4 grid of 52 marks over 5,600
//! samples from [`simulate`].

use std::fmt;

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::Figure;
use rustd3_figure::axis::{bottom, left};
use rustd3_figure::hatch::Hatch;
use rustd3_figure::layout::{Grid, em};
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::mark::{density_y, dots, histogram, line, rule_y};
use rustd3_figure::text::{Text, span};

/// The figure's size in logical px.
pub const SIZE: rustd3_figure::kurbo::Size = rustd3_figure::kurbo::Size::new(960.0, 760.0);

/// A wheel of the car.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Wheel {
    FL,
    FR,
    RL,
    RR,
}

/// What a sample measures.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Source {
    AccelX,
    AccelZ,
    Temp,
}

/// A 10 km/h speed bin: `Bin(4)` is 40–50 km/h.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bin(u8);

impl fmt::Display for Bin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lo = 10 * u32::from(self.0);
        write!(f, "{lo}–{}", lo + 10)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Source::AccelX => "accel x",
            Source::AccelZ => "accel z",
            Source::Temp => "bearing temp",
        })
    }
}

/// One measurement: a wheel's source in a speed bin.
pub struct Sample {
    wheel: Wheel,
    source: Source,
    bin: Bin,
    value: f64,
}

const SOURCES: [Source; 3] = [Source::AccelX, Source::AccelZ, Source::Temp];

/// seaborn's "deep", one color per source.
pub const PALETTE: [Rgb; 3] = [Rgb::hex(0x4c72b0), Rgb::hex(0xdd8452), Rgb::hex(0x55a868)];
const RED: Rgb = Rgb::hex(0xc44e52);
const EDGE: Rgb = Rgb::hex(0x4d4d4d);
const DAMAGED: [Wheel; 1] = [Wheel::RL];
/// Bins with fewer samples are not trusted.
const MIN_COUNT: f64 = 20.0;

/// The wheel figure of `samples` in [`PALETTE`], its legend below the
/// grid, or in a corner of FL's scatter with `inset`. Every mark owns its
/// data, so the figure borrows nothing. It can be a `Figure<'static>`, or
/// a figure of any shorter lifetime that sits beside figures that borrow
/// (`Figure` is invariant in its lifetime).
pub fn figure<'a>(samples: &[Sample], inset: bool) -> Figure<'a> {
    figure_with(samples, inset, PALETTE, None)
}

/// The wheel figure of `samples` with its histograms' bars hatched at 45°
/// (6 pt apart, 0.8 pt wide in the theme's ink). The legend's squares
/// repeat the hatch.
pub fn hatched<'a>(samples: &[Sample], inset: bool) -> Figure<'a> {
    figure_with(samples, inset, PALETTE, Some(&Hatch::lines(45.0)))
}

/// The wheel figure of `samples` with the sources in `palette`'s colors,
/// its histograms' bars hatched with `hatch` when given.
// The body is written in a vertical style that reads as the figure's
// outline, so rustfmt leaves it as written.
#[rustfmt::skip]
pub fn figure_with<'a>(
    samples: &[Sample],
    inset: bool,
    palette: [Rgb; 3],
    hatch: Option<&Hatch>,
) -> Figure<'a> {
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
        scale::ordinal(SOURCES, palette),
    );
    // Handles are Copy: route the twin here.
    let y_of = |s: Source| match s {
        Source::Temp => temp,
        Source::AccelX | Source::AccelZ => vib,
    };

    // Axes and legends are values: define once,
    // place as often as needed.
    let speed_axis = bottom(speed)
        .values((0..20).step_by(4).map(Bin))
        .format(|b| format!("{}", 10 * b.0))
        .label("speed [km/h]");
    let vib_axis = left(vib)
        .label("vibration RMS [m/s²]");
    // The twin sits left too, tinted like its
    // series; layout stacks it outside `vib`.
    let temp_axis = left(temp)
        .color(color, Source::Temp)
        .format(|t| format!("{t:.0}°"))
        .label("bearing temp [°C]");
    let count_axis = left(count)
        .label("count");
    let key = legend(color).title("source");

    // A 4×4 grid: per wheel, a scatter with its
    // distributions right and histogram below.
    // Gaps sit between decorations; margins and
    // label placement are measured.
    fig.layout(
        Grid::new()
            .cols([1.0, 0.25, 1.0, 0.25])
            .rows([1.0, 0.3, 1.0, 0.3])
            .col_gaps([em(0.4), em(1.6), em(0.4)])
            .row_gaps([em(0.4), em(1.6), em(0.4)]),
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
            .axis(&vib_axis)
            .axis(&temp_axis)
            .zoom_y(vib)
            .zoom_y(temp);
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
            .axis(&count_axis)
            .axis(&speed_axis);
        for s in SOURCES {
            let bins = of(wheel, s)
                .map(|d| d.bin);
            let bars = histogram(bins, speed, count)
                .fill(color, s)
                .dodge();
            below.mark(match hatch {
                Some(h) => bars.hatch(h.clone()),
                None => bars,
            });
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

/// Ordinary least squares of value on the bin index, evaluated at the
/// first and last bin with data. Returns two points, or none without data.
fn trend(pts: &[(Bin, f64)]) -> Vec<(Bin, f64)> {
    let (Some(lo), Some(hi)) = (
        pts.iter().map(|p| p.0.0).min(),
        pts.iter().map(|p| p.0.0).max(),
    ) else {
        return Vec::new();
    };
    let n = pts.len() as f64;
    let x = |b: Bin| f64::from(b.0);
    let mx = pts.iter().map(|p| x(p.0)).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for &(b, y) in pts {
        sxy += (x(b) - mx) * (y - my);
        sxx += (x(b) - mx).powi(2);
    }
    let slope = sxy / sxx;
    let at = |b: u8| (Bin(b), my + slope * (x(Bin(b)) - mx));
    vec![at(lo), at(hi)]
}

/// The samples: 600 drives per wheel, with both accelerations logged on
/// every drive and the bearing temperature on every third, 5,600 in all.
/// RL's bearing is worn above 100 km/h. The samples are deterministic,
/// drawn from an LCG seeded with 11 and Box–Muller normal deviates.
pub fn simulate() -> Vec<Sample> {
    let mut seed = 11_u32;
    let mut u = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(seed >> 8) / f64::from(1_u32 << 24)
    };
    let mut gauss = move || {
        let (a, b) = (u().max(1e-12), u());
        (-2.0 * a.ln()).sqrt() * (std::f64::consts::TAU * b).cos()
    };
    let mut out = Vec::new();
    for wheel in [Wheel::FL, Wheel::FR, Wheel::RL, Wheel::RR] {
        let worn = wheel == Wheel::RL;
        for i in 0..600 {
            let city = i % 2 == 0;
            let (m, sd) = if city { (50.0, 15.0) } else { (120.0, 25.0) };
            let v: f64 = (m + sd * gauss()).clamp(0.0, 199.9);
            // Truncation is the binning: v lies in [0, 199.9].
            let bin = Bin((v / 10.0) as u8);
            let fast = (v - 100.0).max(0.0);
            let x = 0.2 + 0.006 * v + 0.08 * gauss();
            let z = 0.3 + 0.009 * v + 0.1 * gauss() + if worn { 0.012 * fast } else { 0.0 };
            let mut push = |source, value| {
                out.push(Sample {
                    wheel,
                    source,
                    bin,
                    value,
                })
            };
            push(Source::AccelX, x);
            push(Source::AccelZ, z);
            if i % 3 == 0 {
                let t = 35.0 + 0.12 * v + 2.0 * gauss() + if worn { 0.15 * fast } else { 0.0 };
                push(Source::Temp, t);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use rustd3_figure::stats::Kde;

    use super::{SOURCES, Wheel, simulate};

    #[test]
    fn the_wheel_figures_densities_binned_are_within_a_thousandth_of_exact() {
        // Its densities hold 200 to 600 values, so they draw exactly.
        // Binned, each of the twelve stays within 1e-3 of that.
        let samples = simulate();
        for wheel in [Wheel::FL, Wheel::FR, Wheel::RL, Wheel::RR] {
            for source in SOURCES {
                let values = samples
                    .iter()
                    .filter(|d| d.wheel == wheel && d.source == source)
                    .map(|d| d.value);
                let kde = Kde::new(values).expect("values that vary");
                let (binned, exact) = (kde.binned(200, 3.0), kde.grid(200, 3.0));
                assert_eq!(exact, kde.exact(200, 3.0), "below the threshold");
                let worst = binned
                    .iter()
                    .zip(&exact)
                    .map(|(b, e)| ((b.1 - e.1) / e.1).abs())
                    .fold(0.0, f64::max);
                assert!(worst <= 1e-3, "{wheel:?} {source:?}: {worst}");
            }
        }
    }
}
