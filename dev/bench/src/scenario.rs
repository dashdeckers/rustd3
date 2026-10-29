//! [`Scenario`]: what a run does to the plot, as generated input.
//!
//! Positions are logical px relative to the plot, as fractions of its
//! size ([`Anchors`]), so every window size gets the same gesture: for
//! the standard figures the "top panel" is at 27% of the height (the
//! trace's upper panel, the grid's second row), the "bottom panel" at 88%,
//! both at the middle; a workload whose panels lie elsewhere names its
//! own. Sweeps are 600 px or 60% of the window, whichever is shorter.

use iced::Size;

use crate::replay::{Button, Delta, Event, Replay, Timed};
use crate::workload;

/// One of the benchmark's scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scenario {
    /// A Lissajous cursor path over the plot areas, a move every 8 ms.
    Hover,
    /// A drag on the top panel (shared x): 600 px sweeps left and right.
    PanX,
    /// A vertical drag on the bottom panel (its own y).
    PanY,
    /// Wheel bursts of ±1 line at 30 Hz for 1 s, 0.5 s pauses.
    ZoomLines,
    /// High-resolution pixel deltas at 120 Hz in 1 s bursts.
    ZoomPixels,
    /// 1,000 samples per channel every 100 ms, x on `Fit::Window(10.0)`.
    Live,
    /// `pan-x` then `zoom-lines` in a 3840 × 2160 device px window.
    FourK,
    /// The window jumps between three fixed sizes every 500 ms, one
    /// re-layout each.
    Resize,
    /// The window dragged by a corner: a step every 16 ms for 1.5 s, down
    /// to 70% of its real size and back, a drag every 3 s (report-only).
    DragResize,
    /// No input: process start to the first sharp frame.
    Cold,
    /// `live` plus a `pan-x` sweep every 10 s, memory sampled.
    Soak,
}

/// The default window, logical px: the workloads' size.
pub const WINDOW: Size = Size::new(
    workload::WINDOW.width as f32,
    workload::WINDOW.height as f32,
);

/// The 4K window in device px.
pub const FOUR_K: Size = Size::new(3840.0, 2160.0);

/// A window drag's step, ms: a 60 Hz pointer.
const DRAG_STEP_MS: f64 = 16.0;

/// How long a window drag lasts, ms.
const DRAG_MS: f64 = 1_500.0;

/// How far a window drag shrinks the window at its middle, a share of
/// its real size.
const DRAG_SHRINK: f32 = 0.3;

/// A window drag starts every this many ms.
const DRAG_EVERY_MS: f64 = 3_000.0;

/// The resize scenario's cycle, logical px.
const RESIZES: [Size; 3] = [
    Size::new(1200.0, 800.0),
    Size::new(1600.0, 1000.0),
    Size::new(1000.0, 700.0),
];

/// The pointer's rate during drags and hovers: a 125 Hz mouse.
const MOUSE_MS: f64 = 8.0;

/// Where the gestures act, as fractions of the plot's size: the vertical
/// drag and the wheel at `x` across, the wheel and the horizontal drag on
/// the "top panel" at `top` down (the drag starting half its sweep right
/// of `x`), the vertical drag on the "bottom panel" at `bottom` down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchors {
    /// Across, from the left.
    pub x: f32,
    /// Down, from the top: the wheel and the horizontal drag.
    pub top: f32,
    /// Down, from the top: the vertical drag.
    pub bottom: f32,
}

impl Anchors {
    /// The standard figures': the middle across, the trace's upper panel
    /// at 27% of the height, its lower one at 88%.
    pub const STANDARD: Anchors = Anchors {
        x: 0.5,
        top: 0.27,
        bottom: 0.88,
    };
}

impl Scenario {
    /// Every scenario, in report order.
    pub const ALL: [Scenario; 11] = [
        Scenario::Hover,
        Scenario::PanX,
        Scenario::PanY,
        Scenario::ZoomLines,
        Scenario::ZoomPixels,
        Scenario::Live,
        Scenario::FourK,
        Scenario::Resize,
        Scenario::DragResize,
        Scenario::Cold,
        Scenario::Soak,
    ];

    /// The scenario's name on the command line and in file names.
    pub fn name(self) -> &'static str {
        match self {
            Scenario::Hover => "hover",
            Scenario::PanX => "pan-x",
            Scenario::PanY => "pan-y",
            Scenario::ZoomLines => "zoom-lines",
            Scenario::ZoomPixels => "zoom-pixels",
            Scenario::Live => "live",
            Scenario::FourK => "4k",
            Scenario::Resize => "resize",
            Scenario::DragResize => "drag-resize",
            Scenario::Cold => "cold",
            Scenario::Soak => "soak",
        }
    }

    /// The scenario named `name`.
    pub fn parse(name: &str) -> Option<Scenario> {
        Scenario::ALL.into_iter().find(|s| s.name() == name)
    }

    /// Its full duration, ms.
    pub fn duration_ms(self) -> f64 {
        match self {
            Scenario::Hover | Scenario::Resize => 10_000.0,
            Scenario::PanX
            | Scenario::PanY
            | Scenario::ZoomLines
            | Scenario::ZoomPixels
            | Scenario::FourK
            | Scenario::DragResize => 12_000.0,
            Scenario::Live => 30_000.0,
            Scenario::Cold => 0.0,
            Scenario::Soak => 600_000.0,
        }
    }

    /// Whether the scenario appends to recordings, so a workload without
    /// any has no such cell.
    pub fn needs_recordings(self) -> bool {
        matches!(self, Scenario::Live | Scenario::Soak)
    }

    /// The window's logical size at `factor`.
    pub fn window(self, factor: f32) -> Size {
        match self {
            Scenario::FourK => Size::new(FOUR_K.width / factor, FOUR_K.height / factor),
            Scenario::Hover
            | Scenario::PanX
            | Scenario::PanY
            | Scenario::ZoomLines
            | Scenario::ZoomPixels
            | Scenario::Live
            | Scenario::Resize
            | Scenario::DragResize
            | Scenario::Cold
            | Scenario::Soak => WINDOW,
        }
    }

    /// Whether the scenario resizes the window itself, which must then be
    /// resizable.
    pub fn resizes(self) -> bool {
        matches!(self, Scenario::Resize | Scenario::DragResize)
    }

    /// The time from the first input to the end of the pan half of `4k`
    /// as a share of its duration; the rest is the zoom half.
    pub const PAN_HALF: f64 = 0.5;

    /// The generated input for a plot of `size` whose gestures act at
    /// `anchors`, lasting `duration_ms`.
    pub fn replay(self, size: Size, anchors: Anchors, duration_ms: f64) -> Replay {
        let mut events = Vec::new();
        let at = Placed { size, anchors };
        match self {
            Scenario::Hover => hover(&mut events, size, duration_ms),
            Scenario::PanX => drag(&mut events, at, (0.0, duration_ms), Axis::X, true),
            Scenario::PanY => drag(&mut events, at, (0.0, duration_ms), Axis::Y, true),
            Scenario::ZoomLines => wheel_lines(&mut events, at, 0.0, duration_ms),
            Scenario::ZoomPixels => wheel_pixels(&mut events, at, duration_ms),
            Scenario::Live => appends(&mut events, duration_ms),
            Scenario::FourK => {
                let half = duration_ms * Scenario::PAN_HALF;
                drag(&mut events, at, (0.0, half), Axis::X, true);
                wheel_lines(&mut events, at, half, duration_ms);
            }
            Scenario::Resize => resizes(&mut events, duration_ms),
            Scenario::DragResize => drags(&mut events, duration_ms),
            Scenario::Cold => {}
            Scenario::Soak => {
                appends(&mut events, duration_ms);
                let mut start = 10_000.0;
                while start < duration_ms {
                    let end = (start + 2_000.0).min(duration_ms);
                    drag(&mut events, at, (start, end), Axis::X, false);
                    start += 10_000.0;
                }
            }
        }
        events.sort_by(|a, b| a.t_ms.total_cmp(&b.t_ms));
        Replay { events }
    }
}

/// The axis a drag sweeps along.
#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

/// A plot's size and where its gestures act.
#[derive(Debug, Clone, Copy)]
struct Placed {
    size: Size,
    anchors: Anchors,
}

/// Every 8 ms a move along x = 0.5 + 0.42·sin(2π·0.37·t),
/// y = 0.5 + 0.42·sin(2π·0.23·t + π/4), as fractions of `size`.
fn hover(events: &mut Vec<Timed>, size: Size, duration_ms: f64) {
    let tau = std::f64::consts::TAU;
    let mut k = 0_u32;
    loop {
        let t_ms = f64::from(k) * MOUSE_MS;
        if t_ms >= duration_ms {
            break;
        }
        let t = t_ms / 1e3;
        let x = 0.5 + 0.42 * (tau * 0.37 * t).sin();
        let y = 0.5 + 0.42 * (tau * 0.23 * t + tau / 8.0).sin();
        events.push(Timed {
            t_ms,
            event: Event::Move {
                x: size.width * x as f32,
                y: size.height * y as f32,
            },
        });
        k += 1;
    }
}

/// A left-button drag from `from_ms` to `to_ms`: press, moves of 2–8 px
/// every 8 ms along `axis`, sweeping 600 px (at most 60% of the window)
/// one way, then back if `both`, and so on; release at the end. The x
/// drag starts half its sweep right of the anchor on the top panel and
/// sweeps left first; the y drag starts on the bottom panel and sweeps up
/// first.
fn drag(
    events: &mut Vec<Timed>,
    Placed { size, anchors }: Placed,
    (from_ms, to_ms): (f64, f64),
    axis: Axis,
    both: bool,
) {
    let (length, mut at) = match axis {
        Axis::X => {
            let length = 600.0_f32.min(0.6 * size.width);
            let x = anchors.x * size.width + 0.5 * length;
            (length, [x, anchors.top * size.height])
        }
        Axis::Y => {
            let length = 600.0_f32.min(0.6 * size.height);
            let at = [anchors.x * size.width, anchors.bottom * size.height];
            (length, at)
        }
    };
    let along = match axis {
        Axis::X => 0,
        Axis::Y => 1,
    };
    events.push(Timed {
        t_ms: from_ms,
        event: Event::Move { x: at[0], y: at[1] },
    });
    events.push(Timed {
        t_ms: from_ms + MOUSE_MS,
        event: Event::Press(Button::Left),
    });
    let (mut direction, mut swept, mut k) = (-1.0_f32, 0.0_f32, 2_u32);
    loop {
        let t_ms = from_ms + f64::from(k) * MOUSE_MS;
        if t_ms >= to_ms - MOUSE_MS {
            break;
        }
        if swept >= length {
            if !both {
                break;
            }
            direction = -direction;
            swept = 0.0;
        }
        // 2–8 px, a deterministic wobble around 5.
        let step = (5.0 + 3.0 * (f64::from(k) * 0.7).sin()) as f32;
        let step = step.min(length - swept);
        at[along] += direction * step;
        swept += step;
        events.push(Timed {
            t_ms,
            event: Event::Move { x: at[0], y: at[1] },
        });
        k += 1;
    }
    events.push(Timed {
        t_ms: (from_ms + f64::from(k) * MOUSE_MS).min(to_ms),
        event: Event::Release(Button::Left),
    });
}

/// From `from_ms`: the pointer on the top panel, then bursts of 30 wheel
/// lines in 1 s (30 Hz), alternately +1 (in) and −1 (out), a burst every
/// 1.5 s, so each pause outlasts the 300 ms wheel end.
fn wheel_lines(events: &mut Vec<Timed>, at: Placed, from_ms: f64, to_ms: f64) {
    events.push(Timed {
        t_ms: from_ms,
        event: Event::Move {
            x: at.anchors.x * at.size.width,
            y: at.anchors.top * at.size.height,
        },
    });
    bursts(events, from_ms, to_ms, 30, |burst, _| {
        let y = if burst % 2 == 0 { 1.0 } else { -1.0 };
        Delta::Lines { x: 0.0, y }
    });
}

/// The pointer on the top panel, then bursts of 120 pixel deltas in 1 s
/// (120 Hz), 2–12 px under a sinusoidal envelope, alternately up and
/// down, a burst every 1.5 s.
fn wheel_pixels(events: &mut Vec<Timed>, at: Placed, duration_ms: f64) {
    events.push(Timed {
        t_ms: 0.0,
        event: Event::Move {
            x: at.anchors.x * at.size.width,
            y: at.anchors.top * at.size.height,
        },
    });
    bursts(events, 0.0, duration_ms, 120, |burst, k| {
        let envelope = (std::f64::consts::PI * f64::from(k) / 120.0).sin();
        let px = (2.0 + 10.0 * envelope) as f32;
        let y = if burst % 2 == 0 { px } else { -px };
        Delta::Pixels { x: 0.0, y }
    });
}

/// Bursts of `per_second` wheel events over 1 s every 1.5 s from
/// `from_ms` (the first 8 ms after it) until `to_ms`; `delta(burst, k)`
/// gives event `k` of burst `burst`.
fn bursts(
    events: &mut Vec<Timed>,
    from_ms: f64,
    to_ms: f64,
    per_second: u32,
    delta: impl Fn(u32, u32) -> Delta,
) {
    let mut burst = 0_u32;
    loop {
        let start = from_ms + MOUSE_MS + f64::from(burst) * 1_500.0;
        if start >= to_ms {
            break;
        }
        for k in 0..per_second {
            let t_ms = start + f64::from(k) * 1_000.0 / f64::from(per_second);
            if t_ms >= to_ms {
                break;
            }
            events.push(Timed {
                t_ms,
                event: Event::Wheel(delta(burst, k)),
            });
        }
        burst += 1;
    }
}

/// 1,000 samples per channel every 100 ms.
fn appends(events: &mut Vec<Timed>, duration_ms: f64) {
    let mut k = 1_u32;
    while f64::from(k) * 100.0 <= duration_ms {
        events.push(Timed {
            t_ms: f64::from(k) * 100.0,
            event: Event::Append(1_000),
        });
        k += 1;
    }
}

/// A resize every 500 ms through [`RESIZES`].
fn resizes(events: &mut Vec<Timed>, duration_ms: f64) {
    let mut k = 1_u32;
    while f64::from(k) * 500.0 <= duration_ms {
        let size = RESIZES[(k as usize - 1) % RESIZES.len()];
        events.push(Timed {
            t_ms: f64::from(k) * 500.0,
            event: Event::Resize {
                width: size.width,
                height: size.height,
            },
        });
        k += 1;
    }
}

/// Window drags by a corner from 500 ms, one every [`DRAG_EVERY_MS`]
/// while one fits: every [`DRAG_STEP_MS`] for [`DRAG_MS`], the window's
/// shares of its real size fall linearly to 1 − [`DRAG_SHRINK`] at the
/// drag's middle and rise back to 1 at its last step.
fn drags(events: &mut Vec<Timed>, duration_ms: f64) {
    let steps = (DRAG_MS / DRAG_STEP_MS).round() as u32;
    let mut start = 500.0;
    while start + DRAG_MS <= duration_ms {
        for k in 1..=steps {
            let t = f64::from(k) / f64::from(steps);
            let depth = 1.0 - (2.0 * t - 1.0).abs();
            let share = 1.0 - DRAG_SHRINK * depth as f32;
            events.push(Timed {
                t_ms: start + f64::from(k) * DRAG_STEP_MS,
                event: Event::ResizeBy {
                    width: share,
                    height: share,
                },
            });
        }
        start += DRAG_EVERY_MS;
    }
}

#[cfg(test)]
mod tests {
    use super::{Anchors, Scenario, WINDOW};
    use crate::replay::{Delta, Event};

    #[test]
    fn hover_moves_every_8_ms() {
        let replay = Scenario::Hover.replay(WINDOW, Anchors::STANDARD, 10_000.0);
        assert_eq!(replay.events.len(), 1_250);
        for pair in replay.events.windows(2) {
            assert!((pair[1].t_ms - pair[0].t_ms - 8.0).abs() < 1e-9);
            assert!(matches!(pair[1].event, Event::Move { .. }));
        }
        for timed in &replay.events {
            let Event::Move { x, y } = timed.event else {
                panic!("hover only moves");
            };
            assert!((0.0..=WINDOW.width).contains(&x) && (0.0..=WINDOW.height).contains(&y));
        }
    }

    #[test]
    fn zoom_lines_pauses_exceed_the_wheel_end() {
        let replay = Scenario::ZoomLines.replay(WINDOW, Anchors::STANDARD, 12_000.0);
        let wheels: Vec<(f64, f32)> = replay
            .events
            .iter()
            .filter_map(|timed| {
                if let Event::Wheel(Delta::Lines { y, .. }) = timed.event {
                    Some((timed.t_ms, y))
                } else {
                    None
                }
            })
            .collect();
        // 8 bursts of 30 lines in 12 s, alternating in and out.
        assert_eq!(wheels.len(), 8 * 30);
        let mut pauses = 0;
        for pair in wheels.windows(2) {
            let gap = pair[1].0 - pair[0].0;
            if pair[0].1 == pair[1].1 {
                assert!((gap - 1_000.0 / 30.0).abs() < 1e-9, "{gap}");
            } else {
                // The widget ends a wheel gesture 300 ms after its last step.
                assert!(gap > 300.0, "{gap}");
                pauses += 1;
            }
        }
        assert_eq!(pauses, 7);
    }

    #[test]
    fn pans_move_2_to_8_px_and_release() {
        for scenario in [Scenario::PanX, Scenario::PanY] {
            let replay = scenario.replay(WINDOW, Anchors::STANDARD, 12_000.0);
            let moves: Vec<[f32; 2]> = replay
                .events
                .iter()
                .filter_map(|timed| {
                    if let Event::Move { x, y } = timed.event {
                        Some([x, y])
                    } else {
                        None
                    }
                })
                .collect();
            for pair in moves.windows(2) {
                let step = (pair[1][0] - pair[0][0]).abs() + (pair[1][1] - pair[0][1]).abs();
                assert!(step <= 8.0 + 1e-3, "{scenario:?}: {step}");
            }
            assert!(matches!(
                replay.events.last().map(|t| t.event),
                Some(Event::Release(_))
            ));
        }
    }

    #[test]
    fn drag_resize_steps_every_16_ms_down_to_70_percent_and_back() {
        let replay = Scenario::DragResize.replay(WINDOW, Anchors::STANDARD, 12_000.0);
        let steps: Vec<(f64, f32, f32)> = replay
            .events
            .iter()
            .filter_map(|timed| {
                if let Event::ResizeBy { width, height } = timed.event {
                    Some((timed.t_ms, width, height))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(steps.len(), replay.events.len(), "only shares");
        // Four drags of 94 steps, at 500, 3500, 6500 and 9500 ms.
        assert_eq!(steps.len(), 4 * 94);
        for drag in steps.chunks(94) {
            for pair in drag.windows(2) {
                assert!((pair[1].0 - pair[0].0 - 16.0).abs() < 1e-9);
            }
            assert!((drag[93].0 - drag[0].0 - 93.0 * 16.0).abs() < 1e-9);
            let least = drag.iter().map(|s| s.1).fold(f32::INFINITY, f32::min);
            assert!((least - 0.7).abs() < 0.01, "{least}");
            assert!(drag.iter().all(|s| s.1 == s.2 && s.1 <= 1.0));
            assert_eq!(drag[93].1, 1.0, "the last step is the real size");
        }
        assert_eq!(steps[0].0, 516.0);
        assert!(steps[94].0 - steps[93].0 > 1_000.0);
        assert!(Scenario::DragResize.resizes() && Scenario::Resize.resizes());
    }

    #[test]
    fn live_appends_every_100_ms_and_resizes_every_500() {
        let live = Scenario::Live.replay(WINDOW, Anchors::STANDARD, 30_000.0);
        assert_eq!(live.events.len(), 300);
        assert!(
            live.events
                .iter()
                .all(|t| matches!(t.event, Event::Append(1_000)))
        );
        assert_eq!(
            Scenario::Resize
                .replay(WINDOW, Anchors::STANDARD, 10_000.0)
                .events
                .len(),
            20
        );
        assert!(
            Scenario::Cold
                .replay(WINDOW, Anchors::STANDARD, 0.0)
                .events
                .is_empty()
        );
    }
}
