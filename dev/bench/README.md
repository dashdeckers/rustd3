# rustd3-bench

The benchmark of rustd3: the plotting library and its `Plot` widget. For
the library, it times figures from data to SVG, PDF and PNG through the
library's own calls, headless, each in a process of its own (the figure
profile). For the widget, it drives the real widget with
replayed input, one cold process per run, and measures frame intervals,
the latency from input to a sharp frame, previews, the UI thread, CPU
and memory; on the software renderer it also composes every frame
offscreen and times it by part (the software profile). One command runs
it on any machine and packs one file to attach to an issue, so a
performance problem on your machine comes with the numbers that explain
it: open an issue and attach the archive.

## The two commands

From a checkout of rustd3:

```bash
cargo rustd3-bench --quick          # everything: the library and the widget
cargo rustd3-bench-figure --quick   # the library alone: no iced, no window
```

`cargo rustd3-bench` is an alias (`.cargo/config.toml`) for `cargo run
--release -p rustd3-bench --`, the same on every platform, with no shell
script. Its first release build compiles iced, wgpu and typst: several
minutes. It runs everything, the figure profile included, and is the
command for any machine with a display.

`cargo rustd3-bench-figure` is `cargo run --release -p rustd3-bench
--no-default-features --`: the bench without its default feature
`widget`, which builds no iced, winit or wgpu (291 crates against 466).
It runs the CPU reference and the figure profile alone, as
`cargo rustd3-bench --figure-only` does: the command for users of the
plotting library alone, and for servers and CI without a display. Its
`--help` shows only what it can run.

The flags:

| Flag | |
| --- | --- |
| `--quick` | about 16 minutes (the figure profile alone: about 1): what a performance issue needs |
| `--default` | about 95 minutes (the figure profile alone: about 4), the default: the figure profile's ladders, the software profile, the registered verdict run and the weak profiles |
| `--full` | hours (the estimate is printed first): every figure and every cell of every plan |
| `--figure-only` | the plotting library alone, at any level: the CPU reference and the figure profile, no display measurement and no window (implied by `cargo rustd3-bench-figure`) |
| `--out DIR` | where results go, `bench-results` by default |
| `--refresh-hz HZ` | the display's refresh rate, for a display that cannot be measured (not in the build without the widget) |
| `--native-factor F` | the display's scale factor, likewise |
| `--frames DIR [--every N]` | the software profile also writes its composed frames (every N-th) as PNGs under `DIR/<cell>__r<run>/`, to look at; the encoding takes a core, so those timings are not for comparing |

Before anything runs, the level prints its steps and an estimate of the
time: the figure profile's figures as timed on two cores at 2.1 GHz, and
each widget run's scenario plus 4 s to start and settle, summed over the
plans' cells and runs; real machines vary. Windows then open and close by
themselves; keep your hands off the mouse and keyboard until it prints
the archive's path. Real input is ignored, but focus changes can disturb
the compositor. A tiling window manager should float the windows (they
are not resizable); one that resizes them anyway is flagged.

### Experiments

A level also takes flags that narrow or vary it for one question, so
that a question about one case is one command and one archive on
the machine that shows it. They are recorded in `results.json`
(`experiment`) and in the level's report, whose "Experiment" section says
what it ran instead of the level's usual profiles.

| Flag | |
| --- | --- |
| `--only P,…` | only these profiles: `figure`, `software`, `native`, `nogpu`, `nogpu-x11`, `4cores`, `2cores`; when none of them opens a window, the display is not measured |
| `--scenario A,B`, `--workload A,B`, `--factor F,G` | only these cells of the widget's profiles; a profile left without a cell is skipped and says why |
| `--runs N` | N runs a cell |
| `--render auto\|inline\|worker`, `--threads N`, `--caching on\|off\|verify` | the `Plot` widget's settings, passed to the widget's profiles (as `run` takes them) |
| `--sweep KEY=V1,V2,…` | each selected widget profile once per value, as the profile `<profile>@KEY=V` (its own directory and log), into the one archive; `KEY` is `render`, `threads` or `caching`, and is not given on its own too |

The level's report sets a sweep's values side by side ("Sweep: …", each
cell's key metrics per value), and `compare` lines up profiles of the same
name, so two sweeps (or one with itself) compare value by value. For
example, how the software path scales with the render pool on `zoom-lines`
over `trace`, in about three minutes:

```bash
cargo rustd3-bench --quick --only software --sweep threads=1,2,8 \
  --scenario zoom-lines --workload trace
```

## The levels

Every level starts the same way:

1. **display**: one window for a few seconds (`run --plan display`, a
   3 s hover on `trace` at data ÷ 100) on the renderer iced chooses. Its
   idle frames give the refresh rate: the median frame interval, when at
   least half the intervals lie within 5 % of it and it lies within
   20–500 Hz. The window reports the scale factor, and its renderer says
   whether iced found a GPU. `--refresh-hz` and `--native-factor` win
   over what it measures.
2. **cpu**: the CPU reference, single-threaded and headless at factor 1,
   medians of a few repetitions, about 5 s: one 256 px tile of `trace`
   mid-zoom; the `scatter100k` plot area with its markers filled, then
   splatted; and a `trace` frame mid-zoom composed by iced's software
   renderer. They put a machine on a scale with others.
3. **figure**: the plotting library alone, headless: figures
   timed from data to SVG, PDF and PNG, each in a process of its own
   ([below](#the-figure-profile)).
4. **software**: the `Plot` widget on iced's software renderer, offscreen
   headless on any machine: every frame composed into a pixmap
   and timed by part ([below](#the-software-profile)).

With `--figure-only`, or built without the widget, a level skips the
display measurement and the software profile and is done after the figure
profile. Otherwise its widget profiles follow, each a `run` of a plan into
its own directory:

| Profile | `--quick` | `--default` | `--full` |
| --- | --- | --- | --- |
| `software`: tiny-skia offscreen, headless at 60 Hz, weak budgets, report-only | `hover`, `pan-x`, `zoom-lines`, `drag-resize` on `trace`, `scatter100k` and `grid16` at factors 1 and 2; 1 run | the same, 3 runs | the same, 5 runs |
| `native`: the renderer iced chooses (wgpu, or tiny-skia without a GPU), desktop budgets | `hover`, `pan-x`, `zoom-lines` on `trace` and `scatter100k`, `drag-resize` on `trace`; 3 runs | the default plan: the registered verdict run, its rule outcome | the full plan: every scenario × workload × factor, 5 runs |
| `nogpu`: tiny-skia, weak budgets | `pan-x`, `zoom-lines` on `trace`; 3 runs | the weak plan, 5 runs | the weak plan |
| `nogpu-x11`: tiny-skia on X11 through XWayland, a Linux Wayland session only, report-only | as `nogpu` | the weak plan | the weak plan |
| `4cores`, `2cores`: iced's renderer limited to 4 and 2 cores, weak budgets | — | the weak plan | the weak plan |

`--full` ends with every `also` measurement (`also.md`). The quick level
is report-only: its cells are held to the budgets to show where each
stands, and its verdicts decide nothing. The default level is the
registered verdict run unchanged, so its history stays comparable.

### The figure profile

What a user of the plotting library pays for a figure, stage by stage,
through the library's public API with its default settings. Report-only:
there are no budgets, and the CPU reference puts the numbers on a scale
across machines.

| Stage | What it times |
| --- | --- |
| `build` | the `Figure` built from data in memory: marks collect what they own (a histogram its values, a density its estimate) |
| `resolve` | `Figure::resolve` for vector output with a fresh cache, as `Figure::draw` starts: fitting, layout, typst text |
| `list` | `Figure::draw_resolved` of that resolve: the drawing list |
| `svg` | that list lowered for SVG and written with glyph outlines: what `save_svg` does after drawing |
| `pdf` | that list as PDF: what `save_pdf` does after drawing |
| `png-1x`, `png-2x` | `save_png`'s whole work at 1 and 2 device px per px, end to end: a PNG is drawn for a file target, which snaps and thins at its own scale, so it has its own resolve and list (a fresh cache, vello_cpu on its own pool, the encoder) |
| `gif` | three frames of the figure at 1× through `gif::Writer`, end to end (`--full` only) |

Outputs are written to memory, not to files, so no disk is timed; tests
hold their bytes to what `save_svg`, `save_pdf` and `save_png` write.

| Figure | Size | What | Levels |
| --- | --- | --- | --- |
| `trace` | 1600 × 1000 | the widget's `trace`: 2 panels, 3 channels × 1M samples on a time axis, a legend | all |
| `scatter100k` | 1600 × 1000 | 100k markers in 5 clusters | all |
| `grid16` | 1600 × 1000 | 4 × 4 panels, each a 100k-sample line and 2k markers | all |
| `labels` | 1600 × 1000 | 3 × 3 panels titled in typst math, math axis labels with minor ticks, a math note each, a legend | all |
| `stats` | 1600 × 1000 | a histogram and a density of 1M values | all |
| `heatmap` | 1600 × 1000 | a 1000 × 1000 heatmap with a colorbar | all |
| `line10k` … `line10m` | 1600 × 1000 | one line of 10k, 100k, 1M, 10M samples | default, full |
| `scatter1k` … `scatter1m` | 1600 × 1000 | 1k, 10k, 1M markers (and `scatter100k`) | default, full |
| `scatter10m` | 1600 × 1000 | 10M markers | full |
| `preset-…` | 640 × 480 | the 11 chart presets of `dev/examples` | default, full |
| `wheels` | 960 × 760 | the wheel figure of `dev/examples`: 4 × 4 panels, 52 marks | default, full |

`--quick` times each stage 7 times after a cold first repetition,
`--default` 10 times, `--full` 15. Metrics per figure and stage: the cold
time apart; the median, least and 90th percentile of the rest; the
output's bytes; and per figure the peak resident memory.

Each figure runs in a process of its own (`rustd3-bench figure-child`):
in one process the resident high-water mark carries what earlier figures
left with the allocator, so a figure's peak would depend on what ran
before it. Alone, its first repetition is as cold as a program's first
figure (fonts parsed, typst started), its peak memory is its own, and a
crash or a hang fails that figure only. The peak is `VmHWM` of
`/proc/self/status`, reset once the figure's data is generated (Linux);
other platforms report "n/a".

The profile writes `figure/summary.csv` (a row per figure and stage),
`figure/report.md` (every figure's stages: cold, median, least, p90,
bytes) and `run-figure.log`; the level's `report.md` shows the medians.

### The software profile

The `Plot` widget's software path as a user without a GPU gets it,
measured where nothing else intervenes: the real widget, driven
through the gestures with replayed input on iced's software renderer
(tiny-skia), headless. No window, display or compositor is involved, so
the profile runs on any machine, and what it measures is the widget's and
iced's work alone. It is `run --plan software --headless --backend
tiny-skia`, one cold process per run.

- **Cells.** `hover`, `pan-x`, `zoom-lines` and `drag-resize` on `trace`,
  `scatter100k` and `grid16`, at factors 1 and 2, each at 1600 × 1000
  logical px (1600 × 1000 and 3200 × 2000 device px): 24 cells, one run
  each at `--quick`, three at `--default`, five at `--full`.
- **Frames.** Virtual frames at 60 Hz in the replay's own clock, as the
  headless runs pace them: a frame that takes longer than its slot delays
  the next, and the replay's input arrives on time regardless. Every
  frame is composed into an offscreen pixmap and read back (iced's
  `screenshot`).
- **Each frame's cost, by part** (`parts.csv`, one row per composed
  frame): the widget's UI work (update and draw over the frame's passes,
  inline renders and the software mosaic included), iced's composition
  into the pixmap (the screenshot), and, after a frame that still shows
  previews or missing tiles, the wait on the pool until the next frame.
  Before its first frame, each run reads back an empty frame of its size:
  the screenshot's fixed cost (a frame-sized buffer and clip mask, the
  background, the conversion to RGBA), which a window's composition does
  not pay.
- **Latency** in the replay's clock, from an input to the first frame
  showing it sharp, and its split: the UI work and compositions of the
  frames before that frame, and the rest, waiting (on the pool, and for
  the next frame).
- **Budgets.** The weak ones, report-only, the frame budget read as
  the frame's cost: a frame must be produced within 2 T (33.3 ms).
  Attribution puts composition where a display would put upload.
- **Frames to look at.** `--frames DIR [--every N]` writes every N-th
  composed frame as a PNG under `DIR/<cell>__r<run>/`, on a thread of its
  own. The encoding takes a core, so such a run is for looking, and the
  report says so.

The profile's report has a "Frame cost by part" table; `summary.csv` and
`results.json` add, per cell, `cost_p50_ms`, `cost_p95_ms`, `cost_p99_ms`,
`ui_p50_ms`, `ui_p99_ms`, `compose_p50_ms`, `compose_p99_ms`,
`readback_ms`, the means per frame `ui_mean_ms`, `compose_mean_ms` and
`wait_mean_ms`, and the latency split `latency_ui_ms`,
`latency_compose_ms` and `latency_wait_ms` (every headless run that reads
each frame back has them).

### Any machine, no stops

- **No display** (SSH, CI, a console: on Linux and the BSDs neither
  `WAYLAND_DISPLAY` nor `DISPLAY` is set), or the display measurement's
  window never drew: every profile runs `--headless` on iced's headless
  renderers, virtual frames at 60 Hz, and the report says why. Headless
  numbers are report-only.
- **A display that paces no frames** (a VM, Xvfb, a remote desktop): the
  level runs at 60 Hz (or `--refresh-hz`), and the report marks every
  frame budget as assuming that rate.
- **No GPU adapter**: iced falls back to tiny-skia, so `native` is the
  software renderer, which the report says; `nogpu` would repeat it and
  is skipped. (An explicit `--backend wgpu` opens no window there; the
  levels use `--backend auto`, which leaves the choice to iced.)
- **Fewer cores than a profile**: it is skipped, and the report says so.
- **Less memory than a figure needs** (more than 60 % of it: 10M markers
  need about 14 GiB): the figure is skipped, and the report says why.
- **Limiting cores**: `taskset -c 0-(n-1)` on Linux when present; `cmd /C
  start "" /B /WAIT /AFFINITY <mask>` on Windows; otherwise (macOS, or
  Linux without `taskset`) nothing is pinned and the widget's render pool
  is set to n − 1 threads (`--threads`). Each weak report's heading says
  which.
- **A run that crashes or hangs**: it is killed two minutes past its
  scenario's length, recorded as failed (its cell shows the runs read of
  those planned), and the level goes on.
  A figure whose process crashes, or outlives ten times its estimate plus
  two minutes, is recorded as failed, and the profile goes on.
- **Ctrl-C**: the archive is written after the display measurement and
  after every profile, so an interrupted level leaves the last whole one
  (std cannot catch the signal portably); its `results.json` reads
  `"complete": false` and its report names the profiles that did not run.
- `system76-power`, where present, is set to `performance` while the
  level runs and restored after.

Linux is where the levels are checked (under Xvfb, and headless), with
and without the widget. Windows and macOS build from the same code with
no platform-specific code path beyond the core limits, the machine
description and the figures' peak memory ("n/a" there), and are
unverified.

## The archive

A level writes `bench-results/<UTC stamp>-<level>/` and packs it into
`bench-results/rustd3-bench-<level>-<UTC stamp>.tar.gz`, whose path it
prints last: the one file to attach (with `--figure-only`, or built
without the widget, both names read `figure-<level>` for `<level>`, and
the archive holds the top-level files and `figure/` alone). Inside:

| Path | Content |
| --- | --- |
| `report.md` | the machine and display, the CPU reference, flags, every profile's budgets met and missed with its key metrics, and at the default and full levels the rule's outcome |
| `results.json` | the same, machine-readable (below) |
| `PRIVACY.md` | what the archive holds |
| `system.txt` | how each profile ran (skips, core limits), then the session type and desktop, whether `WAYLAND_DISPLAY` and `DISPLAY` are set, `xrandr --current`, `cosmic-randr list`, GNOME's scaling factors, `nvidia-smi`, `uname -srm` and `/etc/os-release`, each where present |
| `run-<profile>.log` | each step's console |
| `figure/` | the figure profile's `summary.csv` and `report.md` |
| `software/` | the software profile's run directory, with `parts.csv` |
| `display/`, `<profile>/` | each run's directory (below) |

Nothing personal goes in. `system.txt` never takes `uname -a` (it prints
the host name) nor the display variables' values (`DISPLAY` can name a
host). Before packing, every text file is rewritten: the results
directory's own path goes, so paths are relative to it; the home
directory becomes `~`; the host name `<host>` (as written and in lower
case); the user name, where it is a path component, `<user>`. Archive
entries carry no owner. `PRIVACY.md` lists the rest.

### Reading a submission

```bash
rustd3-bench compare A B   # two directories or archives side by side
rustd3-bench report A      # rebuild and print a level's report
```

`compare` prints the two machines and their CPU references side by side,
then per profile and cell frame p50 and p99, latency p95, preview share,
drops, UI frame p99 and CPU s/s of A and B with B/A (for the figure
profile, per figure and stage the medians of A and B with B/A, and the
peak memory), and marks the cells, figures and profiles only one side
has. `report` rebuilds each widget profile's report from its raw records
(in place for a directory, in a temporary directory for an archive),
then `results.json` and `report.md`, and prints the report; built
without the widget it prints the stored report. Both read results from
either build. Both also read a suite's directory from before the levels (one
directory per plan: `default/` is read as `native`, `weak-nogpu/` as
`nogpu`, and so on), from its `summary.csv` files, without rebuilding;
`report` on one run's directory (`meta.json` at its top) rebuilds it as
before.

### `results.json`

Format 2 (format 1, before the figure profile, reads as a level without
it). Unknown fields are to be ignored; a new required field or a changed
meaning raises `format`. The software profile is a widget
profile whose `plan` is `software` and whose `frame p99` budget reads each
frame's cost, not its interval.

| Field | Content |
| --- | --- |
| `format` | 2 |
| `rustd3` | `version`, `commit` (`git rev-parse HEAD`, or "unknown"), `iced` (the pinned revision) |
| `level` | `quick`, `default` or `full` |
| `figure_only` | true for the plotting library alone (`--figure-only`, or the build without the widget): the CPU reference and the `figure` profile, `display` not measured |
| `experiment` | the experiment flags the level ran with, absent without: `only`, `scenarios`, `workloads`, `factors`, `runs`, `render`, `threads`, `caching`, and `sweep` (`key`: `render`, `threads` or `caching`; `values`) |
| `stamp` | the level's start, UTC, `YYYYmmddTHHMMSSZ` |
| `complete` | false in the archive of an interrupted level |
| `machine` | `os`, `release`, `kernel`, `arch`, `cpu`, `cores`, `ram_mb`, `gpu` (each "unknown" or 0 where no tool says), `session`, `desktop` (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, or "unset"), `wayland`, `x11` (whether `WAYLAND_DISPLAY`, `DISPLAY` are set) |
| `display` | `headless` (why, or null), `refresh_hz`, `refresh`, `native_factor`, `factor` (`refresh` and `factor`: `given`, `measured` or `assumed`), `backend` (the renderer iced chose: `wgpu` or `tiny-skia`, or null) |
| `cpu_reference` | per timing `name` (`trace-tile`, `scatter-filled`, `scatter-splatted`, `composition`), `what`, `ms` (the median, or null when not measured), `repetitions` |
| `profiles` | in order, those skipped and not run included; each below |

A profile: `name` (a sweep's variants `<profile>@KEY=V`), `what` (a
line), `plan`, `backend` (asked: `auto`, `tiny-skia`), `ran` (the
renderers its runs ran on), `budgets` (`desktop` or `weak`), `limit`
(how its cores were limited, or null), `xwayland`, `headless`, `status`
(`done`, `failed`, `skipped`, `pending`), `reason` (why skipped, or what
failed), `outcome` (its outcome line), `checks` (`met`, `missed`,
`inconclusive`), `cells` and `figures`. The `figure` profile has
`figures` and no cells (its `plan` is `figure`, `backend` and `ran`
`vello_cpu`, `budgets` `none`); the widget's profiles have cells and no
figures. A figure: `name`, `what`, `size` (logical px, `[w, h]`),
`rss_peak_mb` (its process's peak resident memory, MiB, or null where
the platform does not say), `failed` and `skipped` (why, or null) and
`stages`, each `stage` (`build`, `resolve`, `list`, `svg`, `pdf`,
`png-1x`, `png-2x`, `gif`), `cold_ms` (the first repetition),
`median_ms`, `min_ms` and `p90_ms` (of the repetitions after it),
`repetitions` (how many after it) and `bytes` (the output's size, or
null). A cell: `scenario`, `workload`, `factor`, `runs` (planned),
`read` (runs with records), `applies` (false: the workload has no such
cell), `metrics` (each per-run statistic's median across runs, named as
in `summary.csv`: `frame_p50_ms`, `frame_p95_ms`, `frame_p99_ms`,
`drops`, `latency_p50_ms`, `latency_p95_ms`, `preview_share`,
`ui_event_p99_ms`, `ui_frame_p99_ms`, `raster_critical_ms`,
`inline_share`, `cpu_s_per_s`, `upload_mb_per_s`, `resolve_ms`,
`list_ms`, `calibration`, and per scenario `latency_p95_pan_ms`,
`latency_p95_zoom_ms`, `preview_share_pan`, `cold_ms`,
`last_step_latency_p50_ms`, `last_step_latency_p95_ms`, `rss_max_mb`,
`gpu_max_mb`) and `budgets`: per budget `budget` (`frame p99`, `pan
latency p95`, `zoom latency p95`, `preview share`, `pan: ` or `zoom: `
before `4k`'s halves), `limit` (ms, or a share), `n`, `mean`, `ci` (the
95% interval, from two runs) and `verdict` (`met`, `missed`,
`inconclusive`).

## For developers

### The commands

`rustd3-bench --help` lists them under "Advanced (developers)":

- `run`: one plan into `--out DIR/<UTC stamp>/`, every flag of the
  benchmark (`--plan`, `--scenario`, `--workload`, `--factor`, `--runs`,
  `--backend wgpu|tiny-skia|auto`, `--budgets desktop|weak`,
  `--present-mode`, `--headless`, `--readback`, `--render`, `--caching`,
  `--threads`, `--replay`, `--data-scale`, `--cpu-limit`, `--xwayland`,
  `--no-also`, and headless `--frames DIR [--every N]`). Windowed, `--refresh-hz` and `--native-factor` are
  required; the benchmark never guesses them. Headless they default to 60
  and 2, and `--headless` without `--plan` runs `smoke`.
- `list`: the readings, scenarios, workloads, plans and a plan's cells.
- `raster`: the raster variants (envelope fills, marker splatting)
  against the cells as drawn, CPU only, into `raster.csv`.
- `also`: the measurements beside the scenarios (below).
- `replay --out DIR`, `record --out FILE`: the generated replays; your
  own input in the `trace` window as a replay (Escape stops).
- `report PATH`, `compare A B`: above.
- `repro-software`: a plain-iced reproduction (below).
- `child`: one scenario × workload × factor × run (internal).
- `figure-child --level L --subject NAME`: one figure of the figure
  profile timed in this process, printed as JSON (internal).

Built without the widget, the bench has the levels, `report`, `compare`
and `figure-child`; `scripts/check.sh` checks that build (`no_widget`:
no iced, winit or wgpu in its tree, clippy, its tests).

### Plans

- `default` (the verdict's): (A) the ten scenarios other than
  `soak` on `trace` at the native factor; (B) `hover`, `pan-x`,
  `zoom-lines` on `scatter100k`, `scatter1m`, `grid16` and `label-churn`
  at native; (C) `pan-x` and `zoom-lines` on `trace` at the other factors
  of 1, 1.25, 1.5 and 2; (D) `soak` once. Five runs per cell: 29 cells,
  141 runs. `drag-resize` (A) is report-only. The factor cells keep
  1600 × 1000 device px at every factor (a
  window of 1600 / k × 1000 / k logical px), which fits a 1080p display,
  and `4k` reads "n/a: display too small" below 3840 × 2160 device px.
- `weak`: `hover`, `pan-x` and `zoom-lines` on `trace`, `pan-x`
  and `zoom-lines` on `scatter100k`, at native, five runs each.
- `quick`: the quick level's cells, three runs each.
- `display`: one 3 s `hover` on `trace`, data ÷ 100.
- `focus`: `zoom-lines` and `zoom-pixels` on `trace`;
  `zoom-lines` on `trace` at the other three factors and on
  `scatter100k`, `scatter1m`, `grid16` and `label-churn`; `pan-x` on
  `trace` and `scatter1m`; `drag-resize` on `trace` and `grid16`.
  Report-only: `run --plan focus` only.
- `software`: `hover`, `pan-x`, `zoom-lines`, `drag-resize` on
  `trace`, `scatter100k` and `grid16` at factors 1 and 2, three runs each;
  run headless on tiny-skia with `--native-factor 2`, so both factors keep
  1600 × 1000 logical px. Report-only, the weak budgets, the frame budget
  read as each frame's cost.
- `smoke`: every scenario × workload once at factor 2, durations ÷ 5,
  data ÷ 100, `soak` 20 s; `scripts/check.sh full` runs it headless.
- `full`: every scenario × workload × factor, five runs.

Two scenarios resize the window: `resize` jumps between three fixed sizes
every 500 ms; `drag-resize` drags a corner as a user does (a step every
16 ms for 1.5 s, down to 70% of the window and back, a drag every 3 s),
and the report's "Window drags" section gives its frame intervals,
dropped frames and the latency from each drag's last step to the first
current frame.

### The registered rule

The rule and its operational readings were registered before any run
and never change after seeing data; `rustd3-bench list` prints
them.

**Budgets.** p99 frame interval ≤ 1.5 × the display's frame period;
input-to-sharp p95 under 100 ms for pans and 150 ms for zooms; preview
share under 10% during pans. Each budget is met, missed or inconclusive by
its 95% interval; the 10-minute memory run is one run per configuration.

**Rule.** A missed budget is attributed to the UI thread (resolve, text,
M4, list), raster (the worker's critical path) or upload. The widget's
image path is tiles that pans translate, rendered on a pool, with
envelope fills and marker splatting; if any pan or zoom budget is missed
and attributed to raster or upload on it, a GPU data layer is
recommended. UI-thread misses route to moving list building off the UI
thread, not to a GPU layer. If every budget is met, the image path
suffices.

**Operational readings.**

1. With vsync, intervals are multiples of the period T, so p99 ≤ 1.5 T
   means the 99th-percentile frame was not a missed frame. A dropped
   frame is an interval > 1.5 T, counted as round(interval / T) − 1.
2. Pans are `pan-x`, `pan-y` and the pan half of `4k`; zooms are
   `zoom-lines`, `zoom-pixels` and the zoom half of `4k`.
3. A budget is missed when the 95% CI of the per-run statistic lies
   entirely above it, met when entirely below, inconclusive otherwise
   (rerun that cell with `--runs 10`).
4. Preview share = frames with `previews + missing > 0` / frames, from the
   first input to 300 ms after the last, per run. An outdated axis strip
   is a preview.
5. Attribution of a missed budget over its offending frames or latency
   windows: UI = `update_ms + draw_ms` (stage split from the probe);
   raster = Σ `Raster.duration` of the sharp frame's generation (the
   single worker's critical path; inline renders included and flagged);
   upload = Σ `Frame.uploaded` MB × the run's least-squares slope of frame
   interval on uploaded MB. The largest share names the bucket; the
   report shows all three.

**Weak profile.** The `weak` plan (`hover`, `pan-x` and
`zoom-lines` on `trace`, `pan-x` and `zoom-lines` on `scatter100k`, at the
native factor, five runs each) runs without a GPU (`--backend tiny-skia`)
and on wgpu pinned to 4 and to 2 cores, each its own report. Budgets: p99
frame interval ≤ 2 T; pan latency p95 ≤ 150 ms; zoom latency p95 ≤ 250
ms; preview share ≤ 25% during pans. Each is met, missed or inconclusive
by the readings above; a miss is attributed as above and becomes an item
for a later release. These profiles never
reopen the GPU-layer question, which stays with the desktop profile.

The levels hold the quick level's native cells to the desktop budgets and
its software cells to the weak ones, report-only (`--budgets`).

### What a run writes

| File | Content |
| --- | --- |
| `meta.json` | machine, cores (as pinned), GPU and driver, session (and `xwayland`), present mode, backend asked, budgets, refresh, factors, render threads, caching, compiler, revisions, the command line, the cost model's constants and its measured calibration |
| `system.txt` | as in the archive |
| `replays/*.jsonl` | the generated input, one event per line |
| `raw/<scenario>__<workload>__f<factor>__r<k>.jsonl` | every frame, input, stage, raster, `sys` and (under `--caching verify`) mismatch record of one run; its first line names the renderer that ran |
| `frames.csv`, `inputs.csv`, `memory.csv` | per frame, per input (with its latency), RSS and GPU samples |
| `parts.csv` | headless, every frame read back: per composed frame its UI work, composition and wait on the pool |
| `summary.csv` | per cell and metric: n, median, mean, 95% CI, budget, verdict |
| `report.md` | readings, budget table, metrics, window drags, attribution, memory, flags, also measured (without `--no-also`), the outcome |

### Optional runs

- `--render inline`: every changed cell rendered on the UI thread.
- `--threads N`: the widget's render pool of N threads (1 to 64) instead
  of its default, the available parallelism less one, 1 to 8.
- `--caching verify`: every kept or translated image renders again and is
  compared; the report's Flags count those that differed by over 1/255.
- Your own touchpad: `rustd3-bench record --out zoom.jsonl`, then `run
  --scenario zoom-pixels --replay zoom.jsonl` with the display's flags.
- An inconclusive cell: rerun it with `--scenario`, `--workload`,
  `--factor` and `--runs 10`.
- The wheel figure (960 × 760, no recordings): `--workload wheels` with
  any plan; it is in `smoke` and `full`, not in `default`.

### The upstream reproduction

`rustd3-bench repro-software` is plain iced with nothing of rustd3
(`src/repro.rs`): 40 images of 256 × 256 px (`--single`: one
window-sized image) cover a 1600 × 1000 window and move 1 px a frame,
redrawn every frame. After a 2 s warm-up it measures for 10 s and prints
the frames drawn, the frame interval's p50, p90 and p99, the CPU seconds
per second and the session's display variables. It is an advanced command
for a question to iced upstream (why iced's software renderer presents
few frames on Wayland); the software profile measures the widget's
software path offscreen. Run it on each renderer, on Wayland and on X11
through XWayland: `ICED_BACKEND=tiny-skia target/release/rustd3-bench
repro-software [--out FILE]` (and `ICED_BACKEND=wgpu` as the control),
with `env -u WAYLAND_DISPLAY` for X11.

### Headless

Automated checks run the windowed benchmark only under Xvfb, and
`--headless` otherwise. Headless frame intervals are not display
intervals: every headless number is report-only. `scripts/check.sh full`
runs the smoke plan on tiny-skia and fails only on errors.

### Also measured

`run` measures in its own process, unless `--no-also`, and the report
lists under "Also measured" (`rustd3-bench also` prints the same):

- the 200k-dot plot-area cell (1600 × 1000 at 2) rendered with
  vello_cpu's pool, against the cost model;
- the trace example at 2×, its cells rendered at their own origins
  against the whole canvas, as pixels over 2/255 and 16/255;
- a 1 px line thinned at one column per device px at 2×, against
  every sample stroked;
- resolving the windowed trace from a fresh cache and in a steady
  layout after an append;
- a pan's UI-thread work on the scatters, and a plot area of tiles
  drawn per frame on iced's headless renderers;
- renders per frame of a pan, and one axis gutter's render;
- a gesture frame composed by iced's software renderer, as
  captured and replayed as each candidate would draw it, and one `trace`
  tile mid-zoom;
- the `scatter100k` plot area, markers filled and splatted;
- a 1400 × 800 area of opaque images composed by iced's software
  renderer as tiles clipped to the area and each to itself, as one image,
  and as as many tiny images, against no image: tiny-skia's cost per
  image and per pixel at factors 1 and 2.
