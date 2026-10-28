//! The `trace` workload: two stacked panels sharing a linear x in seconds,
//! each with its own y; three channels on one 10 kHz clock, channels 1–2
//! in the top panel and 3 in the bottom one; x zoomed by both panels, y
//! by its own; a left and a bottom axis per panel and a legend.

use rustd3_core::scale::{self, Ticks};
use rustd3_core::scheme::COLORBLIND;
use rustd3_core::series::{Clock, Coordinate, Recording};
use rustd3_figure::legend::{self, Slot};
use rustd3_figure::mark::line;
use rustd3_figure::scale::{Fit, Kind, Position, ScaleRef};
use rustd3_figure::{Figure, axis};

use crate::workload::Rng;

/// Samples per channel at full size: 100 s at 10 kHz.
pub const SAMPLES: usize = 1_000_000;

/// Samples per second.
pub const RATE: f64 = 10_000.0;

/// One-sample impacts per channel.
const IMPACTS: usize = 50;

/// The channels' names, in the legend and the palette.
const CHANNELS: [&str; 3] = ["channel 1", "channel 2", "channel 3"];

/// A vibration-like signal sample by sample: drift, a 7 Hz tone, noise,
/// a 0.2 s burst every 10 s and one-sample impacts at fixed indices of the
/// first `n` samples; it continues past `n` for appends.
#[derive(Debug)]
pub struct Signal {
    rng: Rng,
    next: usize,
    /// Sorted sample indices of the impacts.
    impacts: Vec<usize>,
}

/// Three channels of [`Signal`] on one regular clock of `C` (seconds, or
/// instants for `label-churn`), the generators kept for appends.
#[derive(Debug)]
pub struct Trace<C: Coordinate> {
    rec: Recording<C>,
    signals: [Signal; 3],
}

impl Signal {
    /// The signal seeded with `seed`, its impacts among the first `n`
    /// samples (none for fewer than [`IMPACTS`] samples).
    pub fn new(seed: u64, n: usize) -> Signal {
        let mut rng = Rng::new(seed);
        let mut impacts: Vec<usize> = if n >= IMPACTS {
            (0..IMPACTS)
                .map(|_| (rng.unit() * n as f64) as usize)
                .collect()
        } else {
            Vec::new()
        };
        impacts.sort_unstable();
        Signal {
            rng,
            next: 0,
            impacts,
        }
    }

    /// The next `count` samples.
    pub fn take(&mut self, count: usize) -> Vec<f64> {
        let tau = std::f64::consts::TAU;
        let range = self.next..self.next + count;
        self.next += count;
        range
            .map(|i| {
                let t = i as f64 / RATE;
                let burst = if t % 10.0 < 0.2 {
                    0.8 * (tau * 120.0 * t).sin()
                } else {
                    0.0
                };
                let noise = 0.25 * (self.rng.unit() - 0.5);
                let y = 0.3 * (tau * 0.05 * t).sin() + 0.4 * (tau * 7.0 * t).sin() + noise + burst;
                match self.impacts.binary_search(&i) {
                    Ok(k) => {
                        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                        sign * (1.4 + 0.6 * self.rng.unit())
                    }
                    Err(_) => y,
                }
            })
            .collect()
    }

    /// How many impacts it has.
    #[cfg(test)]
    pub fn impacts(&self) -> usize {
        self.impacts.len()
    }
}

impl Trace<f64> {
    /// `n` samples per channel from 0 s, seeds 1, 2 and 3.
    pub fn new(n: usize) -> Trace<f64> {
        Trace::on(
            Clock::Regular {
                start: 0.0,
                period: 1.0 / RATE,
            },
            n,
        )
    }

    /// The figure; `windowed` puts x on `Fit::Window(10.0)`.
    pub fn figure(&self, windowed: bool) -> Figure<'_> {
        let mut fig = Figure::new();
        let time = if windowed {
            fig.fit_with(scale::linear(), Fit::Window(10.0))
        } else {
            fig.fit(scale::linear())
        };
        self.panels(&mut fig, time, "time [s]");
        fig
    }
}

impl<C: Coordinate> Trace<C> {
    /// `n` samples per channel on `clock`, seeds 1, 2 and 3.
    pub fn on(clock: Clock<C>, n: usize) -> Trace<C> {
        let mut signals = [1, 2, 3].map(|seed| Signal::new(seed, n));
        let channels = signals.each_mut().map(|signal| signal.take(n));
        let rec = Recording::new(clock, channels).expect("invariant: finite samples");
        Trace { rec, signals }
    }

    /// Appends `n` samples to every channel, continuing each signal.
    pub fn append(&mut self, n: usize) {
        let channels = self.signals.each_mut().map(|signal| signal.take(n));
        self.rec
            .extend(&[], channels)
            .expect("invariant: finite samples on a regular clock");
    }

    /// The two panels of the channels on the x handle `time`, with their
    /// axes (the bottom one titled `label`), zooms and the legend, added to
    /// `fig`.
    pub fn panels<'a, X>(&'a self, fig: &mut Figure<'a>, time: ScaleRef<X>, label: &str)
    where
        X: Position<Domain = C> + Ticks<Domain = <X as Kind>::Domain>,
    {
        let top = fig.fit(scale::linear());
        let bottom = fig.fit(scale::linear());
        let color = fig.scale(scale::ordinal(CHANNELS, COLORBLIND));
        fig.legend(Slot::Right, legend::legend(color));
        let channel = |c: usize| {
            self.rec
                .series(c)
                .expect("invariant: the recording has three channels")
        };
        fig.panel(0, 0)
            .mark(line(channel(0), time, top).stroke(color, CHANNELS[0]))
            .mark(line(channel(1), time, top).stroke(color, CHANNELS[1]))
            .axis(axis::left(top).label("channels 1, 2"))
            .axis(axis::bottom(time))
            .zoom_x(time)
            .zoom_y(top);
        fig.panel(1, 0)
            .mark(line(channel(2), time, bottom).stroke(color, CHANNELS[2]))
            .axis(axis::left(bottom).label("channel 3"))
            .axis(axis::bottom(time).label(label))
            .zoom_x(time)
            .zoom_y(bottom);
    }

    /// The recording.
    #[cfg(test)]
    pub fn recording(&self) -> &Recording<C> {
        &self.rec
    }

    /// Impacts of channel `c`.
    #[cfg(test)]
    pub fn impacts(&self, c: usize) -> usize {
        self.signals[c].impacts()
    }

    /// Bursts in the recording: samples where a 10 s period opens.
    #[cfg(test)]
    pub fn bursts(&self) -> usize {
        (0..self.rec.len())
            .filter(|&i| i % (10 * RATE as usize) == 0)
            .count()
    }
}
