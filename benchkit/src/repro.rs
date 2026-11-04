//! `bench_gestures repro-software`: a reproduction for iced, plain
//! iced with nothing of rustd3. A grid of 40 distinct 256 × 256 images
//! (`widget::image`, handles made once) covers a 1600 × 1000 window and
//! moves 1 logical px a frame; with `--single`, one image larger than the
//! window moves instead, so that drawing is cheap and presentation shows. `window::frames()` makes each redraw a
//! message and iced redraws after every message, so every frame asks for
//! the next, as a drag does. After a 2 s warm-up it counts frames for 10
//! s, then prints and writes to `--out` the backend, the frame interval's
//! p50, p90 and p99, the process's CPU seconds per second
//! (`/proc/self/stat`) and the session's display variables.
//!
//! What it tests: with rustd3's widget, iced's software renderer drew
//! about 8 frames a second during a drag in a Wayland session, where X11
//! (XWayland) on the same machine drew hundreds, its main thread busy
//! either way, and wgpu kept the display's rate. Does plain iced do the
//! same? The backend is iced's `ICED_BACKEND`, which is then the backend
//! in use: each compositor refuses the other's name.
//!
//! ```text
//! ICED_BACKEND=tiny-skia bench_gestures repro-software [--single]               # Wayland
//! ICED_BACKEND=tiny-skia env -u WAYLAND_DISPLAY bench_gestures repro-software   # X11
//! ICED_BACKEND=wgpu bench_gestures repro-software [--out FILE]                  # control
//! ```

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::widget::{column, image, pin, row};
use iced::{ContentFit, Size, Task, Widget, window};

/// The images' side, logical px.
const TILE: u32 = 256;
/// Columns and rows of images: one more of each than covers the window,
/// as the grid moves.
const GRID: [usize; 2] = [8, 5];
/// Frames before the measurement.
const WARM_UP: Duration = Duration::from_secs(2);
/// The measurement.
const MEASURE: Duration = Duration::from_secs(10);

/// The app: its images and the frames so far.
struct Repro {
    /// The grid's images, or the one image of `--single`.
    tiles: Vec<image::Handle>,
    single: bool,
    out: PathBuf,
    clk_tck: u64,
    /// Every frame's instant, from the first.
    frames: Vec<Instant>,
    /// The first measured frame's index, and the CPU ticks then.
    begin: Option<(usize, Option<u64>)>,
}

/// Opens the window and runs until the measurement is written to `out`;
/// `clk_tck` is the system's clock ticks a second; `single` draws one
/// image larger than the window in place of the grid.
pub fn run(out: PathBuf, clk_tck: u64, single: bool) -> iced::Result {
    let boot = move || Repro::new(out.clone(), clk_tck, single);
    let settings = window::Settings {
        size: Size::new(1600.0, 1000.0),
        resizable: false,
        ..window::Settings::default()
    };
    iced::application(boot, Repro::update, Repro::view)
        .title("repro-software")
        .window(settings)
        .subscription(|_| window::frames())
        .run()
}

impl Repro {
    fn new(out: PathBuf, clk_tck: u64, single: bool) -> Repro {
        let image = |[w, h]: [u32; 2], shade: u8| {
            let pixels: Vec<u8> = (0..w * h)
                .flat_map(|i| [(i % w) as u8, (i / w) as u8, shade, 255])
                .collect();
            image::Handle::from_rgba(w, h, pixels)
        };
        let tiles = if single {
            let size = [GRID[0] as u32 * TILE, GRID[1] as u32 * TILE];
            vec![image(size, 0)]
        } else {
            (0..GRID[0] * GRID[1])
                .map(|k| image([TILE, TILE], (k * 6) as u8))
                .collect()
        };
        Repro {
            tiles,
            single,
            out,
            clk_tck,
            frames: Vec::new(),
            begin: None,
        }
    }

    fn update(&mut self, at: Instant) -> Task<Instant> {
        self.frames.push(at);
        let since = |k: usize| at.saturating_duration_since(self.frames[k]);
        if let Some((k, ticks)) = self.begin {
            if since(k) >= MEASURE {
                self.finish(k, ticks);
                return iced::exit();
            }
        } else if since(0) >= WARM_UP {
            self.begin = Some((self.frames.len() - 1, cpu_ticks()));
        }
        Task::none()
    }

    /// The grid, 1 px further right and down each frame, wrapping a tile.
    fn view(&self) -> impl Widget<Instant> {
        let shift = (self.frames.len() % TILE as usize) as f32 - TILE as f32;
        // Images at their own size, cut by the window's edge, never scaled.
        let rows = self.tiles.chunks(GRID[0]).map(|tiles| {
            row(tiles
                .iter()
                .map(|tile| image(tile.clone()).content_fit(ContentFit::None)))
        });
        pin(column(rows)).x(shift).y(shift)
    }

    /// Prints and writes the measurement from frame `k`, whose CPU ticks
    /// were `ticks`.
    fn finish(&self, k: usize, ticks: Option<u64>) {
        let frames = &self.frames[k..];
        let seconds = (frames[frames.len() - 1] - frames[0]).as_secs_f64();
        let mut intervals: Vec<f64> = frames
            .windows(2)
            .map(|w| (w[1] - w[0]).as_secs_f64() * 1e3)
            .collect();
        intervals.sort_by(f64::total_cmp);
        let p = |q: f64| intervals[(q * (intervals.len() - 1) as f64).round() as usize];
        let busy = ticks
            .zip(cpu_ticks())
            .map(|(b, e)| (e - b) as f64 / self.clk_tck as f64);
        let cpu = busy.map_or_else(|| "n/a".to_owned(), |busy| format!("{:.2}", busy / seconds));
        let var = |name: &str| std::env::var(name).unwrap_or_else(|_| "unset".to_owned());
        // Whether the display variables are set, not their values, which
        // can name a host (`DISPLAY=host:0`).
        let set = |name: &str| match std::env::var_os(name) {
            Some(value) if !value.is_empty() => "set",
            Some(_) | None => "unset",
        };
        let report = format!(
            "plain iced: {} moving 1 px a frame, window 1600 x 1000\n\
             backend (ICED_BACKEND): {}\n\
             frames: {} in {seconds:.1} s\n\
             frame interval p50: {:.1} ms\n\
             frame interval p90: {:.1} ms\n\
             frame interval p99: {:.1} ms\n\
             CPU s/s: {cpu}\n\
             WAYLAND_DISPLAY: {}\nDISPLAY: {}\nXDG_SESSION_TYPE: {}\n",
            if self.single {
                format!(
                    "one image of {} x {}",
                    GRID[0] as u32 * TILE,
                    GRID[1] as u32 * TILE
                )
            } else {
                format!("{} images of {TILE} x {TILE}", self.tiles.len())
            },
            var("ICED_BACKEND"),
            intervals.len(),
            p(0.5),
            p(0.9),
            p(0.99),
            set("WAYLAND_DISPLAY"),
            set("DISPLAY"),
            var("XDG_SESSION_TYPE"),
        );
        print!("{report}");
        if let Err(error) = fs::write(&self.out, &report) {
            eprintln!("repro-software: {}: {error}", self.out.display());
        }
    }
}

/// utime + stime of `/proc/self/stat`, in clock ticks; `None` without it.
fn cpu_ticks() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let fields: Vec<&str> = stat[stat.rfind(')')? + 1..].split_whitespace().collect();
    // After the name, fields count from 3 (state): utime is 14, stime 15.
    Some(fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?)
}
