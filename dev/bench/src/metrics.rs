//! [`Metrics`]: one run's records as the decision rule reads them: frame
//! intervals, input-to-sharp latency, preview share, UI busy time, the
//! raster critical path, CPU and upload rates, memory, and the shares of
//! UI, raster and upload in any window of frames.

use crate::record::{self, Record};
use crate::scenario::Scenario;
use crate::stats::{self, percentile};

/// The part of a run a budget reads: `4k` is a pan, then a zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    /// The whole run.
    Whole,
    /// `4k`'s first half.
    Pan,
    /// `4k`'s second half.
    Zoom,
}

/// One frame pass as recorded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// The widget's counter.
    pub frame: u64,
    /// The `RedrawRequested` instant, ms since process start.
    pub at_ms: f64,
    /// The newest generation planned.
    pub generation: u64,
    /// Slots without an image or showing outdated content.
    pub stale: u32,
    /// Bytes of new handles drawn.
    pub uploaded: u64,
    /// The child's update and draw, ms.
    pub ui_ms: f64,
}

/// One injected event and when it became sharp.
#[derive(Debug, Clone, PartialEq)]
pub struct Latency {
    /// Its kind.
    pub event: String,
    /// When it was scheduled, ms since process start.
    pub scheduled_ms: f64,
    /// Scheduled to the first sharp frame showing it (to the run's end
    /// when none did: a lower bound).
    pub ms: f64,
    /// Whether a sharp frame showed it.
    pub sharp: bool,
    /// The pass it was applied in and the sharp frame.
    pub frames: [u64; 2],
    /// The sharp frame's generation.
    pub generation: u64,
}

/// UI, raster and upload time over some frames (operational reading 5).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Shares {
    /// Update and draw of the frames, ms.
    pub ui_ms: f64,
    /// Raster durations of the generations they waited for, ms.
    pub raster_ms: f64,
    /// Uploaded MB times the run's slope of interval on MB, ms.
    pub upload_ms: f64,
    /// Inline renders among the raster samples.
    pub inline: u32,
}

/// One rendered cell.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Raster {
    frame: u64,
    generation: u64,
    pixels: u32,
    work: u64,
    ms: f64,
    inline: bool,
}

/// A `sys` record.
#[derive(Debug, Clone, PartialEq)]
struct Sys {
    mark: String,
    at_ms: f64,
    cpu_ticks: Option<u64>,
    rss_kb: Option<u64>,
    gpu_mb: Option<f64>,
}

/// One run, parsed.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// The header.
    pub run: record::Run,
    /// The scenario.
    pub scenario: Scenario,
    /// Every frame pass, in order.
    pub frames: Vec<Frame>,
    /// Every input with its latency.
    pub latencies: Vec<Latency>,
    /// The driver's update time of each injected event, ms.
    pub event_ms: Vec<f64>,
    rasters: Vec<Raster>,
    stages: Vec<(String, f64)>,
    sys: Vec<Sys>,
    /// When the run ended, ms since process start.
    pub end_ms: f64,
    /// Whether every slot was sharp at the end.
    pub settled: bool,
    /// The first frame's instant, the replay's zero.
    pub start_ms: f64,
}

/// Window extension past the last input, ms (reading 4).
const AFTER_LAST_MS: f64 = 300.0;

impl Metrics {
    /// Parses a run's records; `None` without a header or frames.
    pub fn of(records: &[Record]) -> Option<Metrics> {
        let mut run = None;
        let mut frames = Vec::new();
        let mut inputs = Vec::new();
        let mut rasters = Vec::new();
        let mut stages = Vec::new();
        let mut sys = Vec::new();
        let (mut end_ms, mut settled) = (None, false);
        for record in records {
            match record {
                Record::Run(header) => run = Some(header.clone()),
                Record::Frame {
                    frame,
                    at_ms,
                    generation,
                    missing,
                    previews,
                    uploaded,
                    update_ms,
                    draw_ms,
                    ..
                } => frames.push(Frame {
                    frame: *frame,
                    at_ms: *at_ms,
                    generation: *generation,
                    stale: missing + previews,
                    uploaded: *uploaded,
                    ui_ms: update_ms + draw_ms.unwrap_or(0.0),
                }),
                Record::Input {
                    event,
                    scheduled_ms,
                    frame,
                    update_ms,
                    ..
                } => inputs.push((event.clone(), *scheduled_ms, *frame, *update_ms)),
                Record::Stage { stage, ms, .. } => stages.push((stage.clone(), *ms)),
                Record::Raster {
                    frame,
                    generation,
                    pixels,
                    work,
                    ms,
                    inline,
                    ..
                } => rasters.push(Raster {
                    frame: *frame,
                    generation: *generation,
                    pixels: *pixels,
                    work: *work,
                    ms: *ms,
                    inline: *inline,
                }),
                Record::Sys {
                    at_ms,
                    mark,
                    cpu_ticks,
                    rss_kb,
                    gpu_mb,
                } => sys.push(Sys {
                    mark: mark.clone(),
                    at_ms: *at_ms,
                    cpu_ticks: *cpu_ticks,
                    rss_kb: *rss_kb,
                    gpu_mb: *gpu_mb,
                }),
                Record::Harness { .. } => {}
                Record::End {
                    at_ms,
                    settled: done,
                } => {
                    end_ms = Some(*at_ms);
                    settled = *done;
                }
            }
        }
        let run = run?;
        let scenario = Scenario::parse(&run.scenario)?;
        frames.sort_by_key(|f: &Frame| f.frame);
        let start_ms = frames.first()?.at_ms;
        let end_ms = end_ms.unwrap_or(frames.last()?.at_ms);
        let event_ms = inputs.iter().map(|input| input.3).collect();
        let latencies = inputs
            .into_iter()
            .map(|(event, scheduled_ms, frame, _)| {
                latency(&frames, event, scheduled_ms, frame, end_ms)
            })
            .collect();
        Some(Metrics {
            run,
            scenario,
            frames,
            latencies,
            event_ms,
            rasters,
            stages,
            sys,
            end_ms,
            settled,
            start_ms,
        })
    }

    /// The display's frame period, ms.
    pub fn period_ms(&self) -> f64 {
        1e3 / self.run.refresh_hz
    }

    /// The frames as drawn: the last pass of each `RedrawRequested`
    /// instant.
    pub fn shown(&self) -> Vec<Frame> {
        let mut shown: Vec<Frame> = Vec::new();
        for frame in &self.frames {
            match shown.last_mut() {
                Some(last) if last.at_ms == frame.at_ms => *last = *frame,
                Some(_) | None => shown.push(*frame),
            }
        }
        shown
    }

    /// Intervals between consecutive `RedrawRequested` instants, ms.
    pub fn intervals(&self) -> Vec<f64> {
        self.shown()
            .windows(2)
            .map(|pair| pair[1].at_ms - pair[0].at_ms)
            .collect()
    }

    /// The `p`th percentile of the frame intervals.
    pub fn interval(&self, p: f64) -> Option<f64> {
        percentile(&self.intervals(), p)
    }

    /// Dropped frames: Σ round(interval / T) − 1 over intervals > 1.5 T.
    pub fn drops(&self) -> u32 {
        let period = self.period_ms();
        self.intervals()
            .iter()
            .filter(|&&interval| interval > 1.5 * period)
            .map(|interval| (interval / period).round() as u32 - 1)
            .sum()
    }

    /// The half an input scheduled at `scheduled_ms` belongs to.
    pub fn half(&self, scheduled_ms: f64) -> Half {
        if self.scenario != Scenario::FourK {
            return Half::Whole;
        }
        let split = self.start_ms + self.run.duration_ms * Scenario::PAN_HALF;
        if scheduled_ms < split {
            Half::Pan
        } else {
            Half::Zoom
        }
    }

    /// The latencies of `half` (every input for `Whole`).
    pub fn latencies_of(&self, half: Half) -> Vec<&Latency> {
        self.latencies
            .iter()
            .filter(|l| half == Half::Whole || self.half(l.scheduled_ms) == half)
            .collect()
    }

    /// The `p`th percentile of `half`'s input-to-sharp latencies.
    pub fn latency(&self, half: Half, p: f64) -> Option<f64> {
        let ms: Vec<f64> = self.latencies_of(half).iter().map(|l| l.ms).collect();
        percentile(&ms, p)
    }

    /// From the first input of `half` to 300 ms after its last, ms since
    /// process start; for a run without input, start to end.
    pub fn window(&self, half: Half) -> [f64; 2] {
        let inputs = self.latencies_of(half);
        match (inputs.first(), inputs.last()) {
            (Some(first), Some(last)) => [first.scheduled_ms, last.scheduled_ms + AFTER_LAST_MS],
            (None, _) | (_, None) => [self.start_ms, self.end_ms],
        }
    }

    /// The shown frames inside `window`.
    fn shown_in(&self, [from, to]: [f64; 2]) -> Vec<Frame> {
        self.shown()
            .into_iter()
            .filter(|f| f.at_ms >= from && f.at_ms <= to)
            .collect()
    }

    /// Frames with a missing or outdated slot over frames, in `half`'s
    /// window (reading 4).
    pub fn preview_share(&self, half: Half) -> Option<f64> {
        let frames = self.shown_in(self.window(half));
        (!frames.is_empty())
            .then(|| frames.iter().filter(|f| f.stale > 0).count() as f64 / frames.len() as f64)
    }

    /// The `p`th percentile of the update time per injected event.
    pub fn event_busy(&self, p: f64) -> Option<f64> {
        percentile(&self.event_ms, p)
    }

    /// The `p`th percentile of update + draw per frame pass.
    pub fn frame_busy(&self, p: f64) -> Option<f64> {
        let ms: Vec<f64> = self.frames.iter().map(|f| f.ui_ms).collect();
        percentile(&ms, p)
    }

    /// Σ raster time of `generation`, ms, and how many renders were inline.
    fn raster_of(&self, generation: u64) -> (f64, u32) {
        self.rasters
            .iter()
            .filter(|r| r.generation == generation)
            .fold((0.0, 0), |(ms, inline), r| {
                (ms + r.ms, inline + u32::from(r.inline))
            })
    }

    /// The median over latency windows of the raster critical path: Σ
    /// raster time of the sharp frame's generation.
    pub fn raster_critical(&self) -> Option<f64> {
        let ms: Vec<f64> = self
            .latencies
            .iter()
            .filter(|l| l.sharp)
            .map(|l| self.raster_of(l.generation).0)
            .collect();
        percentile(&ms, 50.0)
    }

    /// The share of renders that ran inline.
    pub fn inline_share(&self) -> Option<f64> {
        (!self.rasters.is_empty()).then(|| {
            self.rasters.iter().filter(|r| r.inline).count() as f64 / self.rasters.len() as f64
        })
    }

    /// Process CPU seconds per second over the interaction window: from
    /// the first input to 300 ms after the last (start to done without
    /// input).
    pub fn cpu_per_s(&self) -> Option<f64> {
        let mark = |name: &str| {
            self.sys
                .iter()
                .find(|s| s.mark == name)
                .and_then(|s| Some((s.at_ms, s.cpu_ticks?)))
        };
        let (from, to) = match (mark("first_input"), mark("window_end")) {
            (Some(from), Some(to)) => (from, to),
            (None, _) | (_, None) => (mark("start")?, mark("done")?),
        };
        cpu_rate(from, to, self.run.clk_tck)
    }

    /// Uploaded MB per second over the whole window.
    pub fn upload_mb_per_s(&self) -> Option<f64> {
        let window = self.window(Half::Whole);
        let seconds = (window[1] - window[0]) / 1e3;
        let bytes: u64 = self.shown_in(window).iter().map(|f| f.uploaded).sum();
        (seconds > 0.0).then(|| bytes as f64 / 1e6 / seconds)
    }

    /// The median duration of the `stage` stage, ms.
    pub fn stage(&self, stage: &str) -> Option<f64> {
        let ms: Vec<f64> = self
            .stages
            .iter()
            .filter(|s| s.0 == stage)
            .map(|s| s.1)
            .collect();
        percentile(&ms, 50.0)
    }

    /// Process start to the first frame with every slot current, ms.
    pub fn cold_ms(&self) -> Option<f64> {
        self.frames.iter().find(|f| f.stale == 0).map(|f| f.at_ms)
    }

    /// RSS samples: (s since start, MB).
    pub fn rss(&self) -> Vec<(f64, f64)> {
        self.sys
            .iter()
            .filter_map(|s| Some(((s.at_ms - self.start_ms) / 1e3, s.rss_kb? as f64 / 1024.0)))
            .collect()
    }

    /// GPU memory samples: (s since start, MiB).
    pub fn gpu(&self) -> Vec<(f64, f64)> {
        self.sys
            .iter()
            .filter_map(|s| Some(((s.at_ms - self.start_ms) / 1e3, s.gpu_mb?)))
            .collect()
    }

    /// Measured over modelled time of every render with a model time.
    pub fn calibration(&self) -> Vec<f64> {
        self.rasters
            .iter()
            .filter_map(|r| {
                let model = rustd3_iced::plot::cost::model_ms(u64::from(r.pixels), r.work);
                (model > 0.0).then(|| r.ms / model)
            })
            .collect()
    }

    /// The run's least-squares slope of frame interval (ms) on the MB
    /// uploaded in the frame.
    pub fn slope(&self) -> Option<f64> {
        let shown = self.shown();
        let points: Vec<(f64, f64)> = shown
            .windows(2)
            .map(|pair| (pair[0].uploaded as f64 / 1e6, pair[1].at_ms - pair[0].at_ms))
            .collect();
        stats::slope(&points)
    }

    /// The upload cost of `bytes`, ms: never negative.
    fn upload_ms(&self, bytes: u64) -> f64 {
        (bytes as f64 / 1e6 * self.slope().unwrap_or(0.0)).max(0.0)
    }

    /// Shares over latency windows slower than `limit` in `half`.
    pub fn latency_shares(&self, half: Half, limit: f64) -> Shares {
        let mut shares = Shares::default();
        for l in self.latencies_of(half).into_iter().filter(|l| l.ms > limit) {
            let frames = self
                .frames
                .iter()
                .filter(|f| f.frame >= l.frames[0] && f.frame <= l.frames[1]);
            let (ui, bytes) = frames.fold((0.0, 0), |(ui, bytes), f| {
                (ui + f.ui_ms, bytes + f.uploaded)
            });
            let (raster, inline) = self.raster_of(l.generation);
            shares.ui_ms += ui;
            shares.raster_ms += raster;
            shares.inline += inline;
            shares.upload_ms += self.upload_ms(bytes);
        }
        shares
    }

    /// Shares over frames whose next interval exceeds 1.5 T: inline
    /// renders count as raster and leave the UI share.
    pub fn interval_shares(&self) -> Shares {
        let period = self.period_ms();
        let shown = self.shown();
        let mut shares = Shares::default();
        for pair in shown
            .windows(2)
            .filter(|p| p[1].at_ms - p[0].at_ms > 1.5 * period)
        {
            let frame = pair[0];
            let inline: Vec<&Raster> = self
                .rasters
                .iter()
                .filter(|r| r.inline && r.frame == frame.frame)
                .collect();
            let raster: f64 = inline.iter().map(|r| r.ms).sum();
            shares.ui_ms += (frame.ui_ms - raster).max(0.0);
            shares.raster_ms += raster;
            shares.inline += inline.len() as u32;
            shares.upload_ms += self.upload_ms(frame.uploaded);
        }
        shares
    }

    /// Shares over the stale frames of `half`'s window: their UI time, the
    /// renders of the generations they waited for (each once), uploads.
    pub fn preview_shares(&self, half: Half) -> Shares {
        let stale: Vec<Frame> = self
            .shown_in(self.window(half))
            .into_iter()
            .filter(|f| f.stale > 0)
            .collect();
        let mut generations: Vec<u64> = stale.iter().map(|f| f.generation).collect();
        generations.dedup();
        let mut shares = Shares::default();
        for frame in &stale {
            shares.ui_ms += frame.ui_ms;
            shares.upload_ms += self.upload_ms(frame.uploaded);
        }
        for generation in generations {
            let (raster, inline) = self.raster_of(generation);
            shares.raster_ms += raster;
            shares.inline += inline;
        }
        shares
    }
}

impl Shares {
    /// Adds `other`.
    pub fn add(&mut self, other: Shares) {
        self.ui_ms += other.ui_ms;
        self.raster_ms += other.raster_ms;
        self.upload_ms += other.upload_ms;
        self.inline += other.inline;
    }

    /// The bucket with the largest share: `ui`, `raster` or `upload`;
    /// `none` when all three are zero.
    pub fn bucket(&self) -> &'static str {
        if self.ui_ms + self.raster_ms + self.upload_ms <= 0.0 {
            "none"
        } else if self.raster_ms >= self.ui_ms && self.raster_ms >= self.upload_ms {
            "raster"
        } else if self.upload_ms > self.ui_ms {
            "upload"
        } else {
            "ui"
        }
    }
}

/// The latency of an input applied in pass `frame`: the first pass from
/// it on whose generation is at least the one after it and whose slots
/// are all current; to `end_ms` when none is.
fn latency(frames: &[Frame], event: String, scheduled_ms: f64, frame: u64, end_ms: f64) -> Latency {
    let applied = frames.iter().find(|f| f.frame >= frame);
    let generation = applied.map_or(0, |f| f.generation);
    let sharp = frames
        .iter()
        .find(|f| f.frame >= frame && f.generation >= generation && f.stale == 0);
    match sharp {
        Some(sharp) => Latency {
            event,
            scheduled_ms,
            ms: sharp.at_ms - scheduled_ms,
            sharp: true,
            frames: [frame, sharp.frame],
            generation: sharp.generation,
        },
        None => Latency {
            event,
            scheduled_ms,
            ms: end_ms - scheduled_ms,
            sharp: false,
            frames: [frame, frames.last().map_or(frame, |f| f.frame)],
            generation,
        },
    }
}

/// CPU seconds per second between two (ms, ticks) marks.
fn cpu_rate(from: (f64, u64), to: (f64, u64), clk_tck: u64) -> Option<f64> {
    let seconds = (to.0 - from.0) / 1e3;
    (seconds > 0.0 && clk_tck > 0)
        .then(|| to.1.saturating_sub(from.1) as f64 / clk_tck as f64 / seconds)
}

#[cfg(test)]
pub mod tests {
    use super::{Half, Metrics, Shares, cpu_rate};
    use crate::record::{self, Record};

    /// A header for `scenario` at 60 Hz, 1 s long.
    pub fn header(scenario: &str) -> Record {
        Record::Run(record::Run {
            scenario: scenario.to_owned(),
            workload: "trace".to_owned(),
            factor: 2.0,
            run: 1,
            backend: "wgpu".to_owned(),
            headless: false,
            refresh_hz: 60.0,
            window: [1600.0, 1000.0],
            device: [3200.0, 2000.0],
            window_factor: Some(2.0),
            duration_ms: 1_000.0,
            data_scale: 1.0,
            clk_tck: 100,
        })
    }

    /// A frame pass.
    pub fn frame(frame: u64, at_ms: f64, generation: u64, previews: u32) -> Record {
        Record::Frame {
            frame,
            at_ms,
            generation,
            images: 9,
            missing: 0,
            previews,
            uploaded: 0,
            update_ms: 1.0,
            draw_ms: Some(0.5),
        }
    }

    /// An input applied in pass `frame`.
    pub fn input(scheduled_ms: f64, frame: u64) -> Record {
        Record::Input {
            event: "move".to_owned(),
            scheduled_ms,
            injected_ms: scheduled_ms,
            frame,
            update_ms: 0.1,
        }
    }

    #[test]
    fn latency_runs_to_the_first_sharp_frame_of_the_generation() {
        // Scheduled at 1000 ms, applied in the frame at 1004 ms (gen 7);
        // 1011 ms still previews, 1018 ms is sharp.
        let records = [
            header("pan-x"),
            frame(1, 990.0, 6, 0),
            frame(2, 1004.0, 7, 3),
            frame(3, 1011.0, 7, 3),
            frame(4, 1018.0, 7, 0),
            input(1000.0, 2),
        ];
        let metrics = Metrics::of(&records).unwrap();
        assert_eq!(metrics.latencies[0].ms, 18.0);
        assert_eq!(metrics.latencies[0].frames, [2, 4]);
    }

    #[test]
    fn preview_share_counts_outdated_gutters() {
        // 40 frames of a pan, two of them with an outdated slot (an x-axis
        // gutter still showing the previous labels counts as a preview).
        let mut records = vec![header("pan-x")];
        for k in 0..40_u64 {
            let previews = u32::from(k == 10 || k == 20);
            records.push(frame(k + 1, 1000.0 + 16.0 * k as f64, k, previews));
        }
        records.push(input(1000.0, 1));
        records.push(input(1000.0 + 16.0 * 39.0 - 300.0, 1));
        let metrics = Metrics::of(&records).unwrap();
        assert_eq!(metrics.preview_share(Half::Whole), Some(2.0 / 40.0));
    }

    #[test]
    fn intervals_count_dropped_frames() {
        // At 144 Hz an interval of 20.8 ms is round(3.0) − 1 = 2 drops.
        let mut records = vec![header("pan-x")];
        if let Record::Run(run) = &mut records[0] {
            run.refresh_hz = 144.0;
        }
        records.extend([
            frame(1, 0.0, 1, 0),
            frame(2, 6.94, 1, 0),
            frame(3, 27.74, 1, 0),
            // A second pass of the same instant is not an interval.
            frame(4, 27.74, 2, 0),
        ]);
        let metrics = Metrics::of(&records).unwrap();
        assert_eq!(metrics.intervals().len(), 2);
        assert_eq!(metrics.drops(), 2);
    }

    #[test]
    fn cpu_seconds_per_second_from_clock_ticks() {
        // 150 ticks over 10 s at 100 ticks a second.
        assert_eq!(cpu_rate((0.0, 1_000), (10_000.0, 1_150), 100), Some(0.15));
    }

    #[test]
    fn attribution_names_the_largest_share() {
        // A 30 ms window: UI 6 ms, raster 18 ms, upload 2 MB × 1.5 ms/MB.
        let shares = Shares {
            ui_ms: 6.0,
            raster_ms: 18.0,
            upload_ms: 2.0 * 1.5,
            inline: 0,
        };
        assert_eq!(shares.bucket(), "raster");
        let ui = Shares {
            ui_ms: 20.0,
            ..shares
        };
        assert_eq!(ui.bucket(), "ui");
        let upload = Shares {
            upload_ms: 30.0,
            ..shares
        };
        assert_eq!(upload.bucket(), "upload");
        // Through a run: the window's raster samples of the sharp
        // generation outweigh its frames' UI time.
        let records = [
            header("pan-x"),
            frame(1, 1004.0, 7, 3),
            frame(2, 1034.0, 7, 0),
            input(1000.0, 1),
            Record::Raster {
                frame: 2,
                generation: 7,
                cell: 4,
                pixels: 1_000_000,
                work: 100,
                queued_ms: 1.0,
                ms: 18.0,
                inline: false,
            },
        ];
        let metrics = Metrics::of(&records).unwrap();
        let shares = metrics.latency_shares(Half::Whole, 20.0);
        assert_eq!((shares.ui_ms, shares.raster_ms), (3.0, 18.0));
        assert_eq!(shares.bucket(), "raster");
    }
}
