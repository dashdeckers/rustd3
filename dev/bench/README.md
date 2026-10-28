# rustd3-bench: run sheet

The full-frame benchmark drives the real `Plot` widget with replayed
input, one cold process per run, and writes raw records, CSVs and a
`report.md` that evaluates the decision rule registered in
`.claude/decisions/phase-22.md` (D22.1). Its verdict decides whether
Phase 31 (tiles, pan translation, the renderer pool, envelope fills,
marker splatting) is built.

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

1. `smoke/`, about 5 minutes: every scenario × workload once in windows at
   your factor, to see that windows open, take the replayed input and
   close. If iced reports another scale factor for those windows, the
   script says so and the default plan uses that one.
2. `default/`, about 46 minutes: the verdict's run.

Each plan's console goes to `run-smoke.log` and `run-default.log`, and
the diagnostics to `system.txt`, beside them. The script ends with the
directory, each report's rule outcome and the `tar` command for sending it.

- `HZ=144 FACTOR=1.25 dev/bench/run.sh` overrides detection; with no
  refresh rate found, the script stops and asks for `HZ`.
- `dev/bench/run.sh --smoke-only` stops after smoke.

Smoke runs each cell once, so its report reads "inconclusive" everywhere
by design: verdicts need the default plan's 5 runs per cell, and smoke only
checks that windows open, take the replayed input and close.

Windows open and close for about 46 minutes in the default plan (135 runs
of about 16 s and one 10-minute soak); do not touch the mouse or keyboard.
Real input is ignored during a run, but focus changes and pointer warps
can still disturb the compositor. Bench windows are not resizable, so a
tiling window manager should float them at their size; one that resizes
a window anyway is flagged in the report ("window resized by the window
manager"), and the replayed positions follow the window's real size.

The default plan: (A) the nine scenarios other than `soak` on `trace` at
your native factor; (B) `hover`, `pan-x`, `zoom-lines` on `scatter100k`,
`scatter1m`, `grid16` and `label-churn` at native; (C) `pan-x` and
`zoom-lines` on `trace` at the other factors of 1, 1.25, 1.5 and 2; (D)
`soak` once. Five runs per cell: 28 cells, 136 runs.

### By hand

The same steps without the script, from the repository root:

```bash
cargo build --release -p rustd3-bench
system76-power profile performance
target/release/rustd3-bench run --plan smoke --refresh-hz <HZ> \
    --native-factor <F> --factor <F> --out ~/bench-out
target/release/rustd3-bench run --plan default --refresh-hz <HZ> \
    --native-factor <F> --out ~/bench-out
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
| `meta.json` | machine, GPU and driver, session, present mode, backend, refresh, factors, compiler, revisions, the cost model's constants and its measured calibration |
| `system.txt` | the session's variables, `xrandr --current`, `cosmic-randr list`, the GNOME scaling factors, `nvidia-smi`, `uname -a`, `/etc/os-release` (each if present, 5 s at most) |
| `replays/*.jsonl` | the generated input, one event per line |
| `raw/<scenario>__<workload>__f<factor>__r<k>.jsonl` | every frame, input, stage, raster and `sys` record of one run |
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

- `--backend tiny-skia`: the software renderer, report-only.
- `--render inline`: every changed cell rendered on the UI thread, the
  pure synchronous path.
- Your own touchpad: `rustd3-bench record --out zoom.jsonl` opens the
  `trace` window and records your input until Escape; then
  `rustd3-bench run --plan default --refresh-hz <HZ> --native-factor <F>
  --scenario zoom-pixels --replay zoom.jsonl --out ~/bench-out`.
- A cell the report calls inconclusive: rerun it with `--scenario`,
  `--workload`, `--factor` and `--runs 10`.
- The wheel figure (Part I's worked example, 960 × 760, no recordings):
  `--workload wheels` with any plan; it is in `smoke` and `full`, not in
  the verdict's `default` plan, which stays as registered.

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
