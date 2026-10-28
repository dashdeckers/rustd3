# rustd3-bench: run sheet

The full-frame benchmark drives the real `Plot` widget with replayed
input, one cold process per run, and writes raw records, CSVs and a
`report.md` that evaluates the decision rule registered in
`.claude/decisions/phase-22.md` (D22.1). Its verdict decides whether
Phase 31 (tiles, pan translation, the renderer pool, envelope fills,
marker splatting) is built.

## Before you run

1. Tell the agent your display's refresh rate and scale factor, and read
   the operational readings (`target/release/rustd3-bench list` prints
   them, then the plans and cells). They are fixed; numbers never change
   them.
2. Build: `cargo build --release -p rustd3-bench` from the repository
   root (the first release build compiles iced, wgpu and typst: several
   minutes).
3. Plug in power, `system76-power profile performance`, close other
   GPU-heavy apps. Leave the compositor as it is; the report records
   `XDG_SESSION_TYPE` and `XDG_CURRENT_DESKTOP`.

## The run

First, about 5 minutes: the smoke plan in windows, to see that windows
open, run and close on your desktop (its numbers do not count):

```bash
target/release/rustd3-bench run --plan smoke --refresh-hz <HZ> \
    --native-factor <F> --out ~/bench-out
```

Smoke runs each cell once, so its report reads "inconclusive" everywhere
by design: verdicts need the default plan's 5 runs per cell, and smoke only
checks that windows open, take the replayed input and close.

Then the verdict's run:

```bash
target/release/rustd3-bench run --plan default --refresh-hz <HZ> \
    --native-factor <F> --out ~/bench-out
```

`--refresh-hz` and `--native-factor` are required for a windowed run; the
benchmark never guesses them. Windows open and close for about 46 minutes
(135 runs of about 16 s and one 10-minute soak); do not touch the mouse or
keyboard. Real input is ignored during a run, but focus changes and
pointer warps can still disturb the compositor.

The default plan: (A) the nine scenarios other than `soak` on `trace` at
your native factor; (B) `hover`, `pan-x`, `zoom-lines` on `scatter100k`,
`scatter1m`, `grid16` and `label-churn` at native; (C) `pan-x` and
`zoom-lines` on `trace` at the other factors of 1, 1.25, 1.5 and 2; (D)
`soak` once. Five runs per cell: 28 cells, 136 runs.

## What it writes

`~/bench-out/<UTC stamp>/`:

| File | Content |
| --- | --- |
| `meta.json` | machine, GPU and driver, session, present mode, backend, refresh, factors, compiler, revisions, the cost model's constants and its measured calibration |
| `replays/*.jsonl` | the generated input, one event per line |
| `raw/<scenario>__<workload>__f<factor>__r<k>.jsonl` | every frame, input, stage, raster and `sys` record of one run |
| `frames.csv`, `inputs.csv`, `memory.csv` | per frame, per input (with its latency), RSS and GPU samples |
| `summary.csv` | per cell and metric: n, median, mean, 95% CI, budget, verdict |
| `report.md` | readings, budget table, metrics, attribution, memory, flags, also measured, the rule's outcome |

## What to send back

```bash
tar czf bench.tgz -C ~/bench-out <stamp>
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
