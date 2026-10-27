//! The trace figure: three channels of a long vibration-like recording in
//! two panels that share their time axis, thinned at draw time.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::mark::line;
use rustd3_figure::{Figure, axis};

/// Samples per second of [`signal`].
const RATE: f64 = 10_000.0;

/// A vibration-like trace of `n` samples at 10 kHz: drift, a 7 Hz tone,
/// noise, 0.2 s bursts every 10 s and 50 one-sample impacts, from an LCG
/// seeded with `seed` (`proto/thin2/src/main.rs`, whose numbers seed 42
/// gives). Returns x in seconds, y, and the impacts' sample indices,
/// sorted.
pub fn signal(n: usize, seed: u64) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
    let mut state = seed;
    let mut u = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1_u64 << 53) as f64
    };
    let tau = std::f64::consts::TAU;
    let x: Vec<f64> = (0..n).map(|i| i as f64 / RATE).collect();
    let mut y: Vec<f64> = x
        .iter()
        .map(|t| {
            let burst = if t % 10.0 < 0.2 {
                0.8 * (tau * 120.0 * t).sin()
            } else {
                0.0
            };
            0.3 * (tau * 0.05 * t).sin() + 0.4 * (tau * 7.0 * t).sin() + 0.25 * (u() - 0.5) + burst
        })
        .collect();
    let mut impacts: Vec<usize> = (0..50).map(|_| (u() * n as f64) as usize).collect();
    impacts.sort_unstable();
    for (k, &i) in impacts.iter().enumerate() {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        y[i] = sign * (1.4 + 0.6 * u());
    }
    (x, y, impacts)
}

/// Three channels of `n` samples on one clock: [`signal`] with seeds 42,
/// 43 and 44, whose x all agree.
pub fn recording(n: usize) -> Recording<f64> {
    let (x, a, _) = signal(n, 42);
    let (_, b, _) = signal(n, 43);
    let (_, c, _) = signal(n, 44);
    Recording::new(Clock::Explicit(x), [a, b, c]).expect("invariant: the signal's x increase")
}

/// Channel 0 of `rec` in the top panel and channels 1 and 2 in the bottom
/// one, sharing x in seconds, each channel in its palette color, with left
/// and bottom axes and no legend. Lines over the recording thin at draw
/// time, so the figure costs what the screen or file shows, not what the
/// recording holds.
pub fn trace(rec: &Recording<f64>) -> Figure<'_> {
    let mut fig = Figure::new();
    let time = fig.fit(scale::linear());
    let top = fig.fit(scale::linear());
    let bottom = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal([0, 1, 2], COLORBLIND));
    let channel = |c: usize| {
        rec.series(c)
            .expect("invariant: the recording has three channels")
    };
    fig.title("Vibration, three channels");
    fig.panel(0, 0)
        .mark(line(channel(0), time, top).stroke(color, 0))
        .axis(axis::left(top).label("channel 0"))
        .axis(axis::bottom(time).labels(false));
    let lower = fig.panel(1, 0);
    for c in [1, 2] {
        lower.mark(line(channel(c), time, bottom).stroke(color, c));
    }
    lower
        .axis(axis::left(bottom).label("channels 1, 2"))
        .axis(axis::bottom(time).label("time [s]"));
    fig
}
