# rustd3-bench

The benchmark of rustd3's `Plot` widget. It drives the real widget with
replayed input, one cold process per run, and measures frame intervals,
the latency from input to a sharp frame, previews, the UI thread, CPU and
memory. One command runs it on any machine and packs one file to attach
to an issue, so a performance problem on your machine comes with the
numbers that explain it (D22.9 in `.claude/decisions/phase-22.md`).

## The one command

From a checkout of rustd3:

```bash
cargo rustd3-bench --quick
```

`cargo rustd3-bench` is an alias (`.cargo/config.toml`) for `cargo run
--release -p rustd3-bench --`; `dev/bench/run.sh` does the same on Linux
and macOS. The first release build compiles iced, wgpu and typst: several
minutes. The flags:

| Flag | |
| --- | --- |
| `--quick` | about 10 minutes: what a performance issue needs |
| `--default` | about 75 minutes, the default: the registered verdict run and the weak profiles |
| `--full` | hours (the estimate is printed first): every cell of every plan |
| `--figure-only` | the plotting library alone, at any level: the CPU reference and the figure profile, no display measurement and no window |
| `--out DIR` | where results go, `bench-results` by default |
| `--refresh-hz HZ` | the display's refresh rate, for a display that cannot be measured |
| `--native-factor F` | the display's scale factor, likewise |

Before anything runs, the level prints its steps and an estimate of the
time: each run's scenario plus 4 s to start and settle, summed over the
plans' cells and runs; real machines vary. Windows then open and close by
themselves; keep your hands off the mouse and keyboard until it prints
the archive's path. Real input is ignored, but focus changes can disturb
the compositor. A tiling window manager should float the windows (they
are not resizable); one that resizes them anyway is flagged.

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
3. **figure**: the plotting library alone (D22.10), headless: figures
   timed from data to SVG, PDF and PNG, each in a process of its own
   (below).

Then its profiles, each a `run` of a plan into its own directory:

| Profile | `--quick` | `--default` | `--full` |
| --- | --- | --- | --- |
| `native`: the renderer iced chooses (wgpu, or tiny-skia without a GPU), desktop budgets | `hover`, `pan-x`, `zoom-lines` on `trace` and `scatter100k`, `drag-resize` on `trace`; 3 runs | the default plan: D22.1's verdict run, its rule outcome | the full plan: every scenario × workload × factor, 5 runs |
| `nogpu`: tiny-skia, weak budgets | `pan-x`, `zoom-lines` on `trace`; 3 runs | the weak plan, 5 runs | the weak plan |
| `nogpu-x11`: tiny-skia on X11 through XWayland, a Linux Wayland session only (D22.8), report-only | as `nogpu` | the weak plan | the weak plan |
| `4cores`, `2cores`: iced's renderer limited to 4 and 2 cores, weak budgets | — | the weak plan | the weak plan |

`--full` ends with every `also` measurement (`also.md`). The quick level
is report-only: its cells are held to the budgets to show where each
stands, and its verdicts decide nothing. The default level is the
registered verdict run unchanged, so its history stays comparable.

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
- **Limiting cores**: `taskset -c 0-(n-1)` on Linux when present; `cmd /C
  start "" /B /WAIT /AFFINITY <mask>` on Windows; otherwise (macOS, or
  Linux without `taskset`) nothing is pinned and the widget's render pool
  is set to n − 1 threads (`--threads`). Each weak report's heading says
  which.
- **A run that crashes or hangs**: it is killed two minutes past its
  scenario's length, recorded as failed (its cell shows the runs read of
  those planned), and the level goes on.
- **Ctrl-C**: the archive is written after the display measurement and
  after every profile, so an interrupted level leaves the last whole one
  (std cannot catch the signal portably); its `results.json` reads
  `"complete": false` and its report names the profiles that did not run.
- `system76-power`, where present, is set to `performance` while the
  level runs and restored after.

Linux is where the levels are checked (under Xvfb, and headless). Windows
and macOS build from the same code with no platform-specific code path
beyond the core limits and the machine description, and are unverified.

## The archive

A level writes `bench-results/<UTC stamp>-<level>/` and packs it into
`bench-results/rustd3-bench-<level>-<UTC stamp>.tar.gz`, whose path it
prints last: the one file to attach. Inside:

| Path | Content |
| --- | --- |
| `report.md` | the machine and display, the CPU reference, flags, every profile's budgets met and missed with its key metrics, and at the default and full levels the rule's outcome |
| `results.json` | the same, machine-readable (below) |
| `PRIVACY.md` | what the archive holds |
| `system.txt` | how each profile ran (skips, core limits), then the session type and desktop, whether `WAYLAND_DISPLAY` and `DISPLAY` are set, `xrandr --current`, `cosmic-randr list`, GNOME's scaling factors, `nvidia-smi`, `uname -srm` and `/etc/os-release`, each where present |
| `run-<profile>.log` | each step's console |
| `figure/` | the figure profile's `summary.csv` and `report.md` |
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
drops, UI frame p99 and CPU s/s of A and B with B/A, and marks the cells
and profiles only one side has. `report` rebuilds each profile's report
from its raw records (in place for a directory, in a temporary directory
for an archive), then `results.json` and `report.md`, and prints the
report. Both also read a suite's directory from before the levels (one
directory per plan: `default/` is read as `native`, `weak-nogpu/` as
`nogpu`, and so on), from its `summary.csv` files, without rebuilding;
`report` on one run's directory (`meta.json` at its top) rebuilds it as
before.

### `results.json`

Format 2 (format 1, before the figure profile, reads as a level without
it). Unknown fields are to be ignored; a new required field or a changed
meaning raises `format`.

| Field | Content |
| --- | --- |
| `format` | 2 |
| `rustd3` | `version`, `commit` (`git rev-parse HEAD`, or "unknown"), `iced` (the pinned revision) |
| `level` | `quick`, `default` or `full` |
| `figure_only` | true for the plotting library alone (`--figure-only`, or the build without the widget): the CPU reference and the `figure` profile, `display` not measured |
| `stamp` | the level's start, UTC, `YYYYmmddTHHMMSSZ` |
| `complete` | false in the archive of an interrupted level |
| `machine` | `os`, `release`, `kernel`, `arch`, `cpu`, `cores`, `ram_mb`, `gpu` (each "unknown" or 0 where no tool says), `session`, `desktop` (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, or "unset"), `wayland`, `x11` (whether `WAYLAND_DISPLAY`, `DISPLAY` are set) |
| `display` | `headless` (why, or null), `refresh_hz`, `refresh`, `native_factor`, `factor` (`refresh` and `factor`: `given`, `measured` or `assumed`), `backend` (the renderer iced chose: `wgpu` or `tiny-skia`, or null) |
| `cpu_reference` | per timing `name` (`trace-tile`, `scatter-filled`, `scatter-splatted`, `composition`), `what`, `ms` (the median, or null when not measured), `repetitions` |
| `profiles` | in order, those skipped and not run included; each below |

A profile: `name`, `what` (a line), `plan`, `backend` (asked: `auto`,
`tiny-skia`), `ran` (the renderers its runs ran on), `budgets` (`desktop`
or `weak`), `limit` (how its cores were limited, or null), `xwayland`,
`headless`, `status` (`done`, `failed`, `skipped`, `pending`), `reason`
(why skipped, or what failed), `outcome` (its outcome line), `checks`
(`met`, `missed`, `inconclusive`), `cells` and `figures`. The `figure`
profile has `figures` and no cells (its `plan` is `figure`, `backend` and
`ran` `vello_cpu`, `budgets` `none`); the widget's profiles have cells
and no figures. A figure: `name`, `what`, `size` (logical px, `[w, h]`),
`rss_peak_mb` (its process's peak resident memory, MiB, or null where the
platform does not say), `failed` and `skipped` (why, or null) and
`stages`, each `stage` (`build`, `resolve`, `list`, `svg`, `pdf`,
`png-1x`, `png-2x`, `gif`), `cold_ms` (the first repetition), `median_ms`,
`min_ms` and `p90_ms` (of the repetitions after it), `repetitions` (how
many after it) and `bytes` (the output's size, or null). A cell: `scenario`,
`workload`, `factor`, `runs` (planned), `read` (runs with records),
`applies` (false: the workload has no such cell), `metrics` (each per-run
statistic's median across runs, named as in `summary.csv`:
`frame_p50_ms`, `frame_p95_ms`, `frame_p99_ms`, `drops`,
`latency_p50_ms`, `latency_p95_ms`, `preview_share`, `ui_event_p99_ms`,
`ui_frame_p99_ms`, `raster_critical_ms`, `inline_share`, `cpu_s_per_s`,
`upload_mb_per_s`, `resolve_ms`, `list_ms`, `calibration`, and per
scenario `latency_p95_pan_ms`, `latency_p95_zoom_ms`,
`preview_share_pan`, `cold_ms`, `last_step_latency_p50_ms`,
`last_step_latency_p95_ms`, `rss_max_mb`, `gpu_max_mb`) and `budgets`: per
budget `budget` (`frame p99`, `pan latency p95`, `zoom latency p95`,
`preview share`, `pan: ` or `zoom: ` before `4k`'s halves), `limit` (ms,
or a share), `n`, `mean`, `ci` (the 95% interval, from two runs) and
`verdict` (`met`, `missed`, `inconclusive`).

## For developers

### The commands

`rustd3-bench --help` lists them under "Advanced (developers)":

- `run`: one plan into `--out DIR/<UTC stamp>/`, every flag of the
  benchmark (`--plan`, `--scenario`, `--workload`, `--factor`, `--runs`,
  `--backend wgpu|tiny-skia|auto`, `--budgets desktop|weak`,
  `--present-mode`, `--headless`, `--readback`, `--render`, `--caching`,
  `--threads`, `--replay`, `--data-scale`, `--cpu-limit`, `--xwayland`,
  `--no-also`). Windowed, `--refresh-hz` and `--native-factor` are
  required; the benchmark never guesses them. Headless they default to 60
  and 2, and `--headless` without `--plan` runs `smoke`.
- `list`: the readings, scenarios, workloads, plans and a plan's cells.
- `raster`: the raster variants (envelope fills, marker splatting)
  against the cells as drawn, CPU only, into `raster.csv`.
- `also`: the items earlier phases asked to be measured.
- `replay --out DIR`, `record --out FILE`: the generated replays; your
  own input in the `trace` window as a replay (Escape stops).
- `report PATH`, `compare A B`: above.
- `repro-software`: D22.8's plain-iced reproduction (below).
- `child`: one scenario × workload × factor × run (internal).

### Plans

- `default` (the verdict's, D22.1): (A) the ten scenarios other than
  `soak` on `trace` at the native factor; (B) `hover`, `pan-x`,
  `zoom-lines` on `scatter100k`, `scatter1m`, `grid16` and `label-churn`
  at native; (C) `pan-x` and `zoom-lines` on `trace` at the other factors
  of 1, 1.25, 1.5 and 2; (D) `soak` once. Five runs per cell: 29 cells,
  141 runs. `drag-resize` (A), added with D22.5, is report-only. Since
  D22.3 the factor cells keep 1600 × 1000 device px at every factor (a
  window of 1600 / k × 1000 / k logical px), which fits a 1080p display,
  and `4k` reads "n/a: display too small" below 3840 × 2160 device px.
- `weak` (D22.4): `hover`, `pan-x` and `zoom-lines` on `trace`, `pan-x`
  and `zoom-lines` on `scatter100k`, at native, five runs each.
- `quick` (D22.9): the quick level's cells, three runs each.
- `display` (D22.9): one 3 s `hover` on `trace`, data ÷ 100.
- `focus` (D22.5): `zoom-lines` and `zoom-pixels` on `trace`;
  `zoom-lines` on `trace` at the other three factors and on
  `scatter100k`, `scatter1m`, `grid16` and `label-churn`; `pan-x` on
  `trace` and `scatter1m`; `drag-resize` on `trace` and `grid16`.
  Report-only: `run --plan focus` only.
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
(D22.1) and never change after seeing data; `rustd3-bench list` prints
them.

**Budgets.** p99 frame interval ≤ 1.5 × the display's frame period;
input-to-sharp p95 under 100 ms for pans and 150 ms for zooms; preview
share under 10% during pans. Each budget is met, missed or inconclusive by
its 95% interval; the 10-minute memory run is one run per configuration.

**Rule.** A missed budget is attributed to the UI thread (resolve, text,
M4, list), raster (the worker's critical path) or upload. If any pan or
zoom budget is missed and attributed to raster or upload, Phase 31 is
built (tiles, translation, the pool, envelope fills, splatting) and this
benchmark is rerun; a GPU data layer is recommended only if that rerun
still misses. UI-thread misses route to moving list building off the UI
thread, not to a GPU layer. If every budget is met, Phase 31 is skipped.

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

**Weak profile (D22.4).** The `weak` plan (`hover`, `pan-x` and
`zoom-lines` on `trace`, `pan-x` and `zoom-lines` on `scatter100k`, at the
native factor, five runs each) runs without a GPU (`--backend tiny-skia`)
and on wgpu pinned to 4 and to 2 cores, each its own report. Budgets: p99
frame interval ≤ 2 T; pan latency p95 ≤ 150 ms; zoom latency p95 ≤ 250
ms; preview share ≤ 25% during pans. Each is met, missed or inconclusive
by the readings above; a miss is attributed as above and becomes work for
Phase 31 while it is open, else a recorded 0.2 item. These profiles never
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

### The upstream reproduction (D22.8)

`rustd3-bench repro-software` is plain iced with nothing of rustd3
(`src/repro.rs`): 40 images of 256 × 256 px (`--single`: one
window-sized image) cover a 1600 × 1000 window and move 1 px a frame,
redrawn every frame. After a 2 s warm-up it measures for 10 s and prints
the frames drawn, the frame interval's p50, p90 and p99, the CPU seconds
per second and the session's display variables. On Linux
`dev/bench/repro-software.sh` builds the bench and runs it on both
renderers, on Wayland and on X11 through XWayland, then prints a table.
By hand: `ICED_BACKEND=tiny-skia target/release/rustd3-bench
repro-software [--out FILE]`, with `env -u WAYLAND_DISPLAY` for X11.

### Headless

Agents run the windowed benchmark only under Xvfb for verification, and
`--headless` otherwise. Headless frame intervals are not display
intervals: every headless number is report-only. `scripts/check.sh full`
runs the smoke plan on tiny-skia and fails only on errors.

### Also measured

`run` measures in its own process, unless `--no-also`, and the report
lists under "Also measured" (`rustd3-bench also` prints the same):

- D19.3: the 200k-dot plot-area cell (1600 × 1000 at 2) rendered with
  vello_cpu's pool, against the cost model;
- D18.3: the trace example at 2×, its cells rendered at their own origins
  against the whole canvas, as pixels over 2/255 and 16/255;
- D15.3: a 1 px line thinned at one column per device px at 2×, against
  every sample stroked;
- D21.1: resolving the windowed trace from a fresh cache and in a steady
  layout after an append;
- D22.4: a pan's UI-thread work on the scatters, and a plot area of tiles
  drawn per frame on iced's headless renderers;
- Phase 31: renders per frame of a pan, and one axis gutter's render;
- Phase 31g: a gesture frame composed by iced's software renderer, as
  captured and replayed as each candidate would draw it, and one `trace`
  tile mid-zoom;
- D31.2: the `scatter100k` plot area, markers filled and splatted.
