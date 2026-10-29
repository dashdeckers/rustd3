# rustd3-bench: run sheet

The full-frame benchmark drives the real `Plot` widget with replayed
input, one cold process per run, and writes raw records, CSVs and a
`report.md` that evaluates the decision rule registered in
`.claude/decisions/phase-22.md` (D22.1). Its first verdict built Phase 31
(tiles, pan translation, the renderer pool, marker splatting); this is
the rerun the rule asks for: its outcome says whether the image path now
suffices or a GPU data layer is recommended, and D22.4's weak profiles
say how the widget holds up on weaker machines and without a GPU.

## Before you run

Plug in power and close other GPU-heavy apps. Leave the compositor as it
is; `system.txt` records the session, the displays and the scaling.

The operational readings are fixed before any run; numbers never change
them. `target/release/rustd3-bench list` prints them, then the plans and
cells.

## The run

From the repository root (or anywhere: the script finds the root):

```bash
dev/bench/run.sh
```

It prints what it will do, checks for `cargo`, builds the release bench
(the first release build compiles iced, wgpu and typst: several minutes),
reads the refresh rate (`xrandr --current`'s current mode, else
`cosmic-randr list`'s) and the scale factor (GNOME's `scaling-factor` when
above 0, else 1), and switches `system76-power` to `performance`,
restoring your profile at the end. Then it runs, into
`bench-results/<UTC stamp>/` under the repository root:

1. `smoke/`, about 6 minutes: every scenario × workload once in windows at
   your factor, to see that windows open, take the replayed input and
   close. If iced reports another scale factor for those windows, the
   script says so and the plans after it use that one.
2. `default/`, about 47 minutes: the verdict's rerun.
3. `weak-nogpu/`, about 7 minutes: the `weak` plan on the software
   renderer (`--backend tiny-skia`), as on a machine without a GPU.
4. `weak-4cores/` and `weak-2cores/`, about 7 minutes each: the `weak`
   plan on wgpu with the bench pinned to 4 and to 2 cores (`taskset -c
   0-3`, `taskset -c 0-1`); the widget's render pool follows the cores it
   may use. A pinning is skipped, with a message, on a machine with fewer
   cores or without `taskset`.

About 76 minutes in all, plus the build. Each plan's console goes to
`run-<plan>.log` (`run-smoke.log`, `run-default.log`, `run-weak-nogpu.log`,
…) and the diagnostics to `system.txt`, beside them. The script ends with
the directory, each report's rule outcome and the `tar` command for
sending it.

- `HZ=144 FACTOR=1.25 dev/bench/run.sh` overrides detection; with no
  refresh rate found, the script stops and asks for `HZ`.
- `dev/bench/run.sh --smoke-only` stops after smoke.
- `dev/bench/run.sh --focus` runs D22.5's focus plan in place of smoke and
  the default plan, then the three weak profiles, into the same layout:
  `probe/`, about 2 minutes, one run of the focus plan's first cell at your
  factor, whose windows' factor corrects `FACTOR` as smoke's do; `focus/`,
  about 17 minutes; then `weak-nogpu/`, `weak-4cores/` and
  `weak-2cores/`. About 40 minutes in all, plus the build.

Smoke runs each cell once, so its report reads "inconclusive" everywhere
by design: verdicts need 5 runs per cell, and smoke only checks that
windows open, take the replayed input and close.

Windows open and close for about 70 minutes after smoke (the default
plan's 140 runs of about 16 s and one 10-minute soak, then 3 × 25 runs);
do not touch the mouse or keyboard.
Real input is ignored during a run, but focus changes and pointer warps
can still disturb the compositor. Bench windows are not resizable, so a
tiling window manager should float them at their size; one that resizes
a window anyway is flagged in the report ("window resized by the window
manager"), and the replayed positions follow the window's real size.

The default plan: (A) the ten scenarios other than `soak` on `trace` at
your native factor; (B) `hover`, `pan-x`, `zoom-lines` on `scatter100k`,
`scatter1m`, `grid16` and `label-churn` at native; (C) `pan-x` and
`zoom-lines` on `trace` at the other factors of 1, 1.25, 1.5 and 2; (D)
`soak` once. Five runs per cell: 29 cells, 141 runs. The rule, readings
and budgets are D22.1's, unchanged: `drag-resize` (A), added with D22.5,
is report-only, and no budget reads it.

Two scenarios resize the window. `resize` jumps between three fixed sizes
every 500 ms, one re-layout each; it is not a drag. `drag-resize` drags
the window by a corner as a user does: a step every 16 ms for 1.5 s, down
to 70% of the window's real size and back, a drag every 3 s; the report's
"Window drags" section gives its frame intervals, dropped frames and the
latency from each drag's last step to the first current frame. Since D22.3 the factor cells (C) keep
the standard window's 1600 × 1000 device px at every factor (a window of
1600 / k × 1000 / k logical px at factor k), which fits a 1080p display,
and `4k` reads "n/a: display too small" when the window the compositor
gave is below 3840 × 2160 device px.

The focus plan (D22.5) reruns what Phase 31e changed, five runs each at
the native factor: `zoom-lines` and `zoom-pixels` on `trace`;
`zoom-lines` on `trace` at the other three factors; `zoom-lines` on
`scatter100k`, `scatter1m`, `grid16` and `label-churn`; `pan-x` on
`trace` and `scatter1m`; `drag-resize` on `trace` and `grid16` (13 cells,
65 runs). Its report shows each cell against the desktop budgets, marked
report-only: it is not the rule's run and decides nothing.

The weak plan (D22.4): `hover`, `pan-x` and `zoom-lines` on `trace`,
`pan-x` and `zoom-lines` on `scatter100k`, at the native factor, five
runs each (5 cells, 25 runs). Its budgets, registered before any run: p99
frame interval ≤ 2 T, pan latency p95 ≤ 150 ms, zoom latency p95 ≤ 250 ms,
preview share ≤ 25% during pans; each profile's report has its own "Weak
profile" section and outcome line. They never decide the GPU layer, which
stays with the default plan.

### By hand

The same steps without the script, from the repository root:

```bash
cargo build --release -p rustd3-bench
system76-power profile performance
target/release/rustd3-bench run --plan smoke --refresh-hz <HZ> \
    --native-factor <F> --factor <F> --out ~/bench-out
target/release/rustd3-bench run --plan default --refresh-hz <HZ> \
    --native-factor <F> --out ~/bench-out
target/release/rustd3-bench run --plan weak --backend tiny-skia \
    --refresh-hz <HZ> --native-factor <F> --out ~/bench-out
taskset -c 0-3 target/release/rustd3-bench run --plan weak \
    --refresh-hz <HZ> --native-factor <F> --out ~/bench-out
taskset -c 0-1 target/release/rustd3-bench run --plan weak \
    --refresh-hz <HZ> --native-factor <F> --out ~/bench-out
```

`--refresh-hz` and `--native-factor` are required for a windowed run; the
benchmark never guesses them. `--factor <F>` runs smoke at your factor
instead of 2. The `window_factor` in the first line of each file under
`raw/` is the factor iced reported for that window.

## What it writes

Each plan's run directory (`bench-results/<stamp>/smoke/` and `default/`
from the script, `<--out>/<UTC stamp>/` by hand):

| File | Content |
| --- | --- |
| `meta.json` | machine, cores (as pinned), GPU and driver, session, present mode, backend, refresh, factors, render threads, caching, compiler, revisions, the cost model's constants and its measured calibration |
| `system.txt` | the session's variables, `xrandr --current`, `cosmic-randr list`, the GNOME scaling factors, `nvidia-smi`, `uname -a`, `/etc/os-release` (each if present, 5 s at most) |
| `replays/*.jsonl` | the generated input, one event per line |
| `raw/<scenario>__<workload>__f<factor>__r<k>.jsonl` | every frame, input, stage, raster, `sys` and (under `--caching verify`) mismatch record of one run |
| `frames.csv`, `inputs.csv`, `memory.csv` | per frame, per input (with its latency), RSS and GPU samples |
| `summary.csv` | per cell and metric: n, median, mean, 95% CI, budget, verdict |
| `report.md` | readings, budget table, metrics, attribution, memory, flags, also measured, the rule's outcome |

## What to send back

```bash
tar czf bench.tgz -C bench-results <stamp>    # by hand: -C ~/bench-out <stamp>
```

At least `report.md`, `summary.csv` and `meta.json`; the whole directory
if you can (the raw files let the agent rebuild the report with
`rustd3-bench report <dir>`).

## Optional runs

- `--backend tiny-skia`: the software renderer (the `weak` plan's no-GPU
  profile holds it to budgets; other plans report it).
- `--render inline`: every changed cell rendered on the UI thread, the
  pure synchronous path.
- `--threads N`: the widget's render pool of N threads (1 to 64) instead
  of its default, the available parallelism less one, 1 to 8.
- `--caching verify`: every image the widget keeps or translates renders
  again after the real renders and is compared; the report's Flags count
  the ones that differed by more than 1/255.
- Your own touchpad: `rustd3-bench record --out zoom.jsonl` opens the
  `trace` window and records your input until Escape; then
  `rustd3-bench run --plan default --refresh-hz <HZ> --native-factor <F>
  --scenario zoom-pixels --replay zoom.jsonl --out ~/bench-out`.
- A cell the report calls inconclusive: rerun it with `--scenario`,
  `--workload`, `--factor` and `--runs 10`.
- The wheel figure (Part I's worked example, 960 × 760, no recordings):
  `--workload wheels` with any plan; it is in `smoke` and `full`, not in
  the verdict's `default` plan, which stays as registered. Its gestures
  act on the scatters, which zoom vibration and temperature along y; its
  x is a band scale, so `pan-x` is n/a and `4k`'s pan half moves nothing.

## Headless

Agents run `--headless` only (`--plan smoke` by default: every scenario ×
workload once at factor 2, durations ÷ 5, data ÷ 100 (the wheel figure's
data is fixed), `soak` 20 s, about 5 minutes on a 2-core container). Headless frame intervals are not
display intervals: every headless number is report-only.
`scripts/check.sh full` runs the smoke plan on tiny-skia and fails only on
errors.

## Also measured

Beside the scenarios, `run` measures in its own process, and the report
lists under "Also measured":

- D19.3: the 200k-dot plot-area cell (1600 × 1000 at 2) rendered with
  vello_cpu's pool, against the cost model, and its calibration factor;
- D18.3: the trace example at 2×, its cells rendered at their own
  origins against the whole canvas (vello_cpu's origin-dependent dashes),
  as pixels over 2/255 and 16/255;
- D15.3: a 1 px line thinned at one column per device px at 2×, the share
  of columns whose top or bottom ink moves more than 1 px against every
  sample stroked;
- D21.1: resolving the windowed trace from a fresh cache and in a steady
  layout after an append, with the labels looked up.
- Phase 31g: a real gesture frame composed by iced's software renderer
  (tiny-skia, headless): the widget driven through a zoom burst and a
  drag over `trace` and `scatter100k` at 1 and 2, every frame composed;
  a mid-burst and a mid-drag frame's images, scaled images, new handles
  and composition time (the first, which converts the new handles, and
  warm), and the same images replayed with no image, as captured, every
  image nearest, and each plot area's images composed into one on the CPU.
