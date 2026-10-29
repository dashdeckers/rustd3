# Phase 22 — Full-frame benchmark

## Measurements

2026-10-27, 477c550 … e533558. Phase 22 does no core work: the harness
stays at 9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80
floors; nothing blessed.

**Build.** `cargo build --release -p rustd3-bench`: 35 s at the start of
the phase (iced, wgpu and typst were already built in `target/release` by
earlier phases; not a cold build), about 27 s incremental after a change
to the bench. `cargo test -p rustd3-bench`: 25 unit tests (stats, rule,
metrics, replay, scenario, plan, workload, parser, system), under a
second.

**`scripts/check.sh full`** at e533558: green in 7 min 36 s (warm),
against 4 min 5 s at the end of Phase 21; the release build of the bench
and the tiny-skia smoke plan (about 4 min 20 s) are the difference. No
warnings.

**Smoke plan, headless** (`rustd3-bench run --headless --plan smoke
--backend B --out target/bench-smoke-B`, release, 2-core container, 60 Hz
virtual frames, readback every frame; report-only numbers):

| Backend | Wall | Runs | Files | `report.md` |
| --- | --- | --- | --- | --- |
| tiny-skia | 257 s | 46 run, 4 n/a (`live`, `soak` × the scatters), 0 failed, all settled | every file under Outputs | a row per cell of 50, no empty cell, no NaN; rule section "report-only: headless" |
| wgpu (lavapipe) | 261 s | the same | the same | the same |

What the smoke shows on this container (report-only; every budget is
inconclusive at n = 1, the outcome line reads "no verdict yet"):

- Headless frames are paced by the readback of 3200 × 2000 device px:
  median 83 ms on tiny-skia, 63 ms on lavapipe; frame intervals 67–100 ms
  median.
- Every pan and zoom frame sends its plot-area cells to the worker (the
  model predicts more than half the clamped 16.7 ms budget at 2×), and
  each frame's job set replaces the last before it lands, so no frame is
  sharp until the gesture ends: pan latency p95 ≈ the pan's length (2.5 s
  of a 2.4 s smoke pan), preview share 92–100%. This is the pipeline as
  designed when a render outlasts a frame (Phase 19, newest wins); the
  windowed run measures whether it happens on real hardware.
- Calibration (median measured / modelled raster time over every render):
  8.3 on tiny-skia, 9.1 on wgpu: the worker shares 2 cores with the
  readback, vello_cpu's pool has one thread.
- Cold start (process start to the first frame with every slot current):
  median 253 ms (tiny-skia), 342 ms (wgpu), at data ÷ 100.
- The spot run of step 3 (`pan-x × trace`, wgpu): 47 frames, generation
  non-decreasing from 1 to 35, the last frame sharp; 19 frames with
  `uploaded` > 0, each after a `renderer.tick()`.

**Also measured** (the architect's items, in the run's own process,
release; the same on both smoke runs within ±10%):

| Item | Result |
| --- | --- |
| D19.3: the 200k-dot plot-area cell (1600 × 1000 at 2; 5,734,980 px, 200,186 items), vello_cpu's pool | 633–674 ms (median of 3); model 511.9 ms; calibration 1.24–1.32 |
| D18.3: the trace example (960 × 600 at 2, 3 × 1M samples), 15 cells stitched against the whole render, single-threaded baseline SIMD | 381 px over 2/255, 276 over 16/255, max 237/255 |
| D15.3: a 1 px line thinned at one column per device px at 2×, against every sample stroked | 24.3% of columns off by more than 1 px, 4.32% of pixels off by more than 16/255 (Phase 15's numbers exactly) |
| D21.1: resolving the windowed trace (1600 × 1000 at 2) | fresh cache 1.65–2.31 ms, 486 label lookups; steady layout after an append 0.30–0.48 ms, 159 lookups; 3 layout passes in the last round of each |

The resolve's rounds are not observable through public API: a lookup is
one label in one layout pass of one round, so the fresh resolve looks up
3.1 times the labels of the steady one; its time is 5–6 times.

## D22.1 — The benchmark verdict

**Date:** 2026-10-27, verdict 2026-10-28 · **Context:** Phase 22, step 5; the user's windowed run `bench-results/20261028T085234Z` at 8b6f1bf · **Status:** decided

### Issue

Whether the `Plot` widget's image path meets its frame budgets on real
hardware (Part I, "Open question: the full-frame benchmark"), which
decides whether Phase 31 (tiles, pan translation, the renderer pool,
envelope fills, marker splatting) is built. The rule was registered
before any run (below); headless numbers never decide it.

### Investigation

The headless smoke plan runs on both backends here (Measurements); its
numbers are report-only. Before the run the user gives the display's
refresh rate and scale factor (both are required arguments of a windowed
run; the benchmark never assumes them) and confirms the operational
readings.

**The exact command** (dev/bench/README.md is the run sheet), from the
repository root on Pop!_OS with the RTX 3090, on power, `system76-power
profile performance`, other GPU-heavy apps closed, the compositor as it
is (recorded):

```bash
cargo build --release -p rustd3-bench
# optional first check, about 5 minutes, numbers do not count:
target/release/rustd3-bench run --plan smoke --refresh-hz <HZ> --native-factor <F> --out ~/bench-out
# the verdict's run, about 46 minutes, mouse and keyboard untouched:
target/release/rustd3-bench run --plan default --refresh-hz <HZ> --native-factor <F> --out ~/bench-out
tar czf bench.tgz -C ~/bench-out <stamp>
```

**The plan:** `default`, 28 cells and 136 runs at a native factor among
1, 1.25, 1.5 and 2: (A) the nine scenarios other than `soak` on `trace`
at native; (B) `hover`, `pan-x`, `zoom-lines` on the four other workloads
at native; (C) `pan-x`, `zoom-lines` on `trace` at the three other
factors; (D) `soak` once. Five runs per cell, `soak` once; about 135 × 16 s
+ 10 min ≈ 46 minutes.

**Budgets:** p99 frame interval ≤ 1.5 × the display's frame period;
input-to-sharp p95 under 100 ms for pans and 150 ms for zooms; preview
share under 10% during pans. Each budget is met, missed or inconclusive by
its 95% interval; the 10-minute memory run is one run per configuration.

**Operational readings** (fixed now; never changed after seeing data):

1. With vsync, intervals are multiples of the period T, so p99 ≤ 1.5 T
   means the 99th-percentile frame was not a missed frame. A dropped
   frame is an interval > 1.5 T, counted as round(interval / T) − 1.
2. Pans are `pan-x`, `pan-y` and the pan half of `4k`; zooms are
   `zoom-lines`, `zoom-pixels` and the zoom half of `4k`.
3. A budget is missed when the 95% CI of the per-run statistic lies
   entirely above it, met when entirely below, inconclusive otherwise
   (rerun that cell with `--runs 10`).
4. Preview share = frames with `previews + missing > 0` / frames, from
   the first input to 300 ms after the last, per run. An outdated axis
   strip is a preview.
5. Attribution of a missed budget over its offending frames or latency
   windows: UI = `update_ms + draw_ms` (stage split from the probe);
   raster = Σ `Raster.duration` of the sharp frame's generation (the
   single worker's critical path; inline renders included and flagged);
   upload = Σ `Frame.uploaded` MB × the run's least-squares slope of frame
   interval on uploaded MB. The largest share names the bucket; the
   report shows all three.

### Options

| | A — image path sufficient | B — UI-thread bound | C — raster/upload bound |
| --- | --- | --- | --- |
| When | Every budget met | A budget missed and attributed to the UI thread; no pan or zoom budget missed on raster or upload | A pan or zoom budget missed and attributed to raster or upload |
| Consequence | Phase 31 skipped | Move list building off the UI thread; Phase 31 skipped | Build Phase 31 (tiles, translation, pool, envelope fills, splatting) and rerun this benchmark; a GPU data layer only if that rerun still misses |
| Report line | "image path sufficient — Phase 31 skipped" | "UI-thread bound → move list building off the UI thread; Phase 31 skipped" | "raster/upload bound → build Phase 31 … and rerun" |

### Decision

**C — raster/upload bound → build Phase 31 (tiles, translation, pool,
envelope fills, splatting) and rerun this benchmark** (architect,
2026-10-28), applying the readings as written to the user's run.

The run: `bench-results/20261028T085234Z/default` (the user's machine,
`~/Projects/rustd3`), plan `default`, wgpu, AutoVsync, 59.96 Hz
(T = 16.68 ms), native factor 1 (observed by the smoke run, 1.0); AMD
Ryzen 7 5800X (16 threads), NVIDIA RTX 3090 (595.104.02), COSMIC on
Wayland, one 1920 × 1080 display at 60 Hz; workspace 8b6f1bf. 56 checks:
18 met, 35 missed, 3 inconclusive.

Pan and zoom budgets missed and attributed to raster or upload, which
alone decide C:

| Cell | Budget | Mean [95% CI] | Bucket |
| --- | --- | --- | --- |
| pan-x × trace × 1 | preview share | 94.2% [92.8, 95.6] (≤ 10%) | raster |
| pan-x × trace × 1 | frame p99 | 33.3 ms [33.3, 33.4] (≤ 25.0) | raster |
| pan-x × scatter100k × 1 | pan latency p95 | 11,468 ms (≤ 100) | upload |
| pan-x × label-churn × 1 | preview share | 91.3% (≤ 10%) | raster |
| zoom-lines × trace × 1.25 and × 1.5 | zoom latency p95 | 969 and 976 ms (≤ 150) | upload |
| 4k × trace × 1 | zoom latency p95 | 974 ms (≤ 150) | upload |

Met: hover on every workload, `pan-y × trace` (31 ms, 91% inline), live,
resize, and `pan-x × trace × 2` (16 ms, a 960 × 502 window rendered
inline). The three inconclusive cells are frame-p99 cells; no outcome of
theirs changes C, so they were not rerun.

What the raw records show (`raw/pan-x__trace__f1__r1.jsonl`): during a
continuous pan the plot-area cell renders on the worker in 10–25 ms,
above the 8.35 ms inline threshold, a new generation starts every frame,
and every result lands one generation late; images upload every frame
(5–6 MB) yet no frame is current until the gesture ends, so latency is
the gesture's length. Translation (only entering tiles render) and the
pool are what Phase 31 builds for exactly this.

### Notes beside the verdict

- **The display is 1920 × 1080 at 60 Hz, not the 4K panel the plan
  assumed.** The `4k` cells ran in a 1920 × 1004 window and the factor
  cells (1.25, 1.5, 2) in windows the compositor clamped (1536 × 803,
  1280 × 669, 960 × 502); every such run is flagged. C stands on the
  native-factor `trace` cells alone. D22.3 sizes the rerun's factor cells
  to fit the display.
- **Attribution sums UI time over a latency window** (every frame's
  update and draw inside it) against one raster critical path, so the
  UI bucket wins most latency rows (e.g. pan-x × trace latency: UI 281,
  raster 11.8, upload 77 ms). The rule is applied as written; the
  preview-share and frame-p99 rows above attribute per frame and point
  at raster.
- **`scatter1m` is UI-thread bound as well**: list building takes a
  median 51 ms per frame (resolve 4 ms), so its frame p99 on pans is 165 ms. Marker splatting (Phase 31) cuts its raster cost, not
  its list; moving list building off the UI thread is recorded for 0.2
  (rule B's consequence, here secondary to C).
- **Cost model (D19.3):** median measured/modelled raster time 2.78
  over the run's renders (small cells' fixed cost), while the 200k-dot
  cell with vello_cpu's pool measured 0.179 of the model. Phase 31's
  pool recalibrates `cost` with a per-render overhead term.
- Memory: over the 10-minute soak RSS grew from 360 to 524 MB while the
  replay appended 1,000 samples every 100 ms (6 M samples per channel,
  three channels, about 160 MB with the index), so the growth is the data;
  GPU memory peaked 114 MiB above baseline.

### Why

The rule and readings are registered before any data, so the verdict
cannot be fitted to it; the container's headless numbers measure the
harness's readback, not a display.

### Followups

- The user: the display's refresh rate and scale factor, then the run.
- Phase 31, if built, appends its rerun here; Phase 34 appends the
  label-churn rerun with real time formats (the label-churn rows stay
  provisional until then); whichever of the two runs last writes the
  closing line.
- D19.3: the run's calibration (meta.json, report "Flags") may move
  `cost::PIXELS_PER_MS` and `ITEMS_PER_MS`.

## D22.2 — Details of Phase 22 as built

**Date:** 2026-10-27 · **Context:** Phase 22, 477c550 … e533558 · **Status:** decided

### Issue

1. **Commits.** Steps 1–3 cannot be separate commits: the CLI's flags
   and the workloads' figures have no reader before the child exists, and
   the workspace denies dead code.
2. **The crate.** Phase 1 left `dev/bench/src/lib.rs` (a doc comment
   naming the wheel figure); the section's layout has `main.rs` only.
3. **Modules beyond the layout:** per-run metrics, the also-measured
   items and the `record` command's app each need a home.
4. **Dependencies.** The bench has neither `thiserror` nor a jiff
   re-export (`rustd3_core::jiff` does not exist yet), and manifests do
   not change in this phase.
5. **The driver.** The section's `Driver` has `child`, `replay`,
   `records`, `cue`; joining the probe's `Frame` with the driver's
   update and draw times, and knowing when every slot is sharp, need the
   probe's receiver and the process's start instant.
6. **Record kinds.** The section lists `frame`, `input`, `stage`,
   `raster`, `sys`; the report needs the run's header (backend that ran,
   window obtained, window factor, duration) and the settle's end, and
   `harness_ms` is "recorded separately".
7. **The native factor.** Plan (A)–(C) need the display's factor before
   any window opens; the section names it as input from the user but no
   flag.
8. **The 4k window** "if the display cannot fit it, 1920 × 1080 at factor
   2, recorded": a window cannot render more device pixels than it has,
   and 1920 × 1080 at 2 is the same 3840 × 2160 device px.
9. **Label churn.** `format!("t+{v:.4} s")` labels tick values, which
   stay at nice numbers during a pan: new strings appear only as ticks
   enter, and on every zoom step.
10. **Cue latency.** An `Append` or `Resize` is applied by the app before
    the next pass, so its effect shows in the pass after the one it was
    injected in; a window resize is deferred by the runtime.
11. **Attribution of the frame and preview budgets.** Reading 5 defines
    the shares for latency windows; frame-interval and preview-share
    misses have offending frames, not windows.
12. **Non-verdicts.** n = 1 (smoke) has no interval; a selection may hold
    no budget (`cold`, `soak`); a share may be zero everywhere.
13. **Smoke timing.** "Durations ÷ 5" against the driver's 2 s settle.
14. **Positions and rhythms** the table leaves open.
15. **Size.** About 6,400 lines with tests and docs against the section's
    estimate of 1,500.

### Investigation

Compiler and lints (1, 4); the iced sources at 3cbdacc (5, 8:
`winit/src/lib.rs:777-830` reruns `RedrawRequested` after messages and
defers window actions); the smoke runs (Measurements).

### Options

| | **A — as built** |
| --- | --- |
| (1) | Five code commits: steps 1–3 (477c550), step 4 (f8b6268), the also-measured items (2c76768, the architect's list), a fix (ee94050), step 5 (e533558); then the docs |
| (2) | `lib.rs` removed: a binary crate whose modules are private, as `rustd3-harness` |
| (3) | `metrics.rs` (`Metrics`: one run parsed), `also.rs`, `recorder.rs` |
| (4) | Errors with hand-written `Display`; the run directory's UTC stamp from `SystemTime` (Hinnant's civil date) |
| (5) | `Driver::new(child, replay, records, cue, samples, epoch)`: the driver drains the probe after each forwarded event, keeps the last pass's frame record until its draw, and ends the run when the last `Frame` shows every slot current |
| (6) | Kinds `run` (header, first line), `harness` (readback ms), `end` (settled or timed out) beside the five; `frame.draw_ms` is absent for a pass that was not drawn |
| (7) | `--native-factor F`: required with `--refresh-hz` for a windowed run (a clear error otherwise), 2 and 60 headless; the window is opened at the plot's size × factor / native |
| (8) | The `4k` window is asked for at 3840 × 2160 device px; the run records the size it opened with, and the report flags a window that differs (and a window factor other than the native one) |
| (9) | As written; noted in the workload's docs and the report ("provisional until Phase 34") |
| (10) | A cue's input record takes the frame after the pass that applied it; `resize` latencies are reported, no budget reads them |
| (11) | Frame budget: per offending interval, UI = update + draw of the frame before it minus its inline renders, raster = those inline renders, upload = its MB × slope. Preview budget: UI and upload of the stale frames, raster of the generations they waited for, each once. Shares are means per offending window or frame (ee94050) |
| (12) | Verdict "inconclusive" without an interval; the attribution table also lists budgets a single run exceeded ("over, n = 1", no verdict); outcomes "no verdict yet" (inconclusive, none missed) and "no budget evaluated"; a zero share names no bucket |
| (13) | The settle stays 2 s (it waits for sharp frames); a run that is not sharp 30 s after its settle ends unsettled and flagged |
| (14) | Positions as fractions of the plot: the top panel at 27% of the height, the bottom one at 88%; sweeps 600 px or 60% of the window; hover a Lissajous at 0.37 and 0.23 Hz over 84% of each side; wheel bursts every 1.5 s; `soak`'s sweep a 2 s one-way drag every 10 s; steps 5 ± 3 px; estimates add 4 s per run |
| (15) | Kept: the size follows from the section's metrics, CSVs and report |

### Decision

A for all fifteen (architect, 2026-10-27), as built. A windowed run
requires `--refresh-hz` and `--native-factor`; the bench is a binary
crate with hand-written error `Display`s (no manifest change).

### Why

(1), (2) and (4) follow the lints and the frozen manifests; (5), (6),
(10) and (13) are what the section's join, cold start and "every slot
sharp" need; (7) and (8) follow the architect's note (both inputs
explicit, nothing assumed) and what a window can render; (11) and (12)
extend reading 5 and the outcome forms without changing a budget or the
rule; (3), (9), (14) and (15) are the closest readings.

### Followups

- The user's first windowed run is the first time the windowed path
  runs (the container has no display): the optional windowed smoke in
  D22.1's command checks it in 5 minutes.
- Phase 34: the wheel workload and the label-churn rerun.

## Label churn rerun with real time formats (Phase 34)

2026-10-28, ee179bf. Report-only, as every headless number is; D22.1
stays open for the user's windowed run.

**What changed.** `label-churn` no longer labels a linear x `t+{v:.4} s`
at 16 ticks (D22.2 (9)): `trace`'s three channels run on a 10 kHz clock
of instants centred on 2026-10-28T00:00Z under a `time::utc()` handle
labelled by its own ticks in the figure's locale (the neutral ISO ladder:
the date at midnight, seconds or milliseconds around it, the figure's
default tick count), so pans cross the day boundary and zooms change the
rung. The verdict's plan is unchanged: `default` keeps the 28 cells and
136 runs registered here (the new `wheels` workload runs in `smoke` and
`full` only; D34.4 (9)).

**The rerun.** `target/release/rustd3-bench run --headless --workload
label-churn --backend tiny-skia` (smoke: factor 2, 1600 × 1000, durations
÷ 5, data ÷ 100, n = 1 per cell), against the stand-in's rows in the
smoke report `scripts/check.sh full` wrote earlier the same day on the
same container (`target/bench-smoke/20261028T030003Z`). Medians:

| Cell | frame p50/p95/p99 ms | latency p50/p95 ms | previews | UI p99 frame ms | resolve/list ms |
| --- | --- | --- | --- | --- | --- |
| hover, stand-in → time | 83/117/126 → 100/117/126 | 52/124 → 50/118 | 12% → 12% | 6.5 → 7.7 | 5.5/4.0 → 6.1/4.8 |
| pan-x | 83/117/135 → 84/117/127 | 1554/2630 → 1521/2597 | 100% → 100% | 12.9 → 14.5 | 0.3/2.8 → 0.4/3.7 |
| pan-y | 100/117/143 → 100/117/135 | 1321/2397 → 1369/2446 | 92% → 93% | 52.7 → 48.5 | 0.3/2.3 → 0.3/3.2 |
| zoom-lines | 100/184/211 → 100/188/211 | 800/1228 → 834/1261 | 86% → 84% | 14.5 → 15.5 | 0.3/2.5 → 0.3/3.7 |
| zoom-pixels | 100/167/177 → 100/183/204 | 842/1305 → 742/1222 | 91% → 84% | 10.8 → 14.7 | 0.2/2.8 → 0.2/3.5 |
| live | 100/133/140 → 109/140/150 | 558/3072 → 1067/3722 | 100% → 98% | 9.6 → 13.2 | 0.5/2.7 → 0.6/3.4 |
| 4k | 117/232/244 → 117/250/250 | 1747/2395 → 1780/2428 | 95% → 100% | 14.4 → 17.2 | 0.6/2.8 → 0.7/3.4 |
| resize | 50/83/103 → 50/100/100 | 209/302 → 158/299 | 50% → 39% | 5.0 → 8.5 | 0.4/2.1 → 0.5/2.8 |
| cold | 83/100/113 → 100/115/117 | n/a | 17% → 17% | 8.1 → 9.6 | 5.0/3.6 → 5.2/4.6 |
| soak | 100/136/153 → 100/135/150 | 700/4362 → 4156/10495 | 73% → 68% | 18.5 → 20.1 | 0.5/5.6 → 0.6/6.4 |

Real time labels cost about 1 ms more list building per frame here
(stage medians 2.1–2.8 → 2.8–3.7 ms; soak 5.6 → 6.4) and 1–4 ms more UI
time at the frame p99 on most cells. Frame intervals, latencies and
preview shares move within what n = 1 shows as run-to-run noise: the
`trace` rows of the same stand-in report differ from its `label-churn`
rows by as much (its soak latency p95 10,228 ms against label-churn's
4,362 ms; the time axis's 10,495 ms is the same spread). The rule's
outcome line is unchanged: "no verdict yet: some budget is inconclusive
and none missed".

**Whether Phase 22's conclusion holds:** yes. Headless frames stay paced
by the readback, pans stay previews until the gesture ends (the pipeline
as designed when a render outlasts a frame), and real time formats add
about a millisecond of list building, far below what decides any budget.
The label-churn rows are no longer provisional; the report's flag now says
what the workload labels. The exit criterion's form, `cargo run -p
rustd3-bench -- --headless --workload label-churn` (debug, wgpu on
lavapipe), also runs all ten cells and prints its report.

**The wheel workload's first smoke** (same command with `--workload
wheels`; 960 × 760 at 2, 1920 × 1520 device px; 8 cells, live and soak
n/a): frame p50 50–67 ms (133 ms at 4k), latency p95 59–180 ms on pans and
zooms (228 ms at 4k), previews 6–20%, list stage about 1.5 ms, cold start
169 ms; outcome "no verdict yet".

**The closing line** is not written: the user's run has not given the
verdict, and Phase 31 is neither skipped nor done, so it is left for
Phase 31's part (built with its rerun appended, or recorded as skipped)
once the verdict arrives (Phase 34's section; Order of work).

## D22.3 — The rerun after Phase 31 fits the user's display

**Date:** 2026-10-28 · **Context:** D22.1's run on a 1920 × 1080 display · **Status:** decided

### Issue

The default plan assumed a 4K panel: its factor cells ask for a
1600 × 1000 logical window at factors 1.25, 1.5 and 2 (up to 3200 × 2000
device px), and `4k` for 3840 × 2160 device px. On the user's
1920 × 1080 display the compositor clamped every one of them (D22.1's
flags), so those cells measured smaller windows than they name.

### Options

| | **A — factor cells keep the device size** | B — as registered |
| --- | --- | --- |
| Approach | At factor k a factor cell's window is 1600 / k × 1000 / k logical px (1600 × 1000 device px at every factor), which fits a 1080p display; `4k` runs only on a display at least 3840 × 2160 and is reported `n/a` (display too small) otherwise | Windows the compositor clamps, flagged |
| Fit | The factor cells test scale handling at a fixed raster size, which is what they are for | Cells whose names do not match what ran |

### Decision

A (architect, 2026-10-28), for the rerun Phase 31 owes; the rule, the
readings, the budgets and the other cells stay as registered. Phase 31's
bench step makes the change and the README says it.

### Why

The verdict (D22.1) does not depend on these cells; the rerun should
measure what each cell names.

## D22.4 — Performance on weaker machines and without a GPU

**Date:** 2026-10-28 · **Context:** the user, after D22.1's run: "we also want good performance on less powerful machines, as well as machines without a gpu" · **Status:** decided

### Issue

D22.1's run measured a 16-thread Ryzen 7 5800X with an RTX 3090. The
widget must also hold up on machines with 2–4 cores, integrated GPUs, or
no GPU at all (iced's tiny-skia backend: software rasterizing and
compositing). Three facts from the run matter there: the cost model's
constants are machine-specific (median measured/modelled raster time
2.78 over the run's renders against 0.179 for one large cell with the
pool); the worker pool's speedup depends on the core count; and list
building for 1M dots already takes 51 ms on the UI thread of the fast
machine.

### Decision

(architect, 2026-10-28) Phase 31's scope gains four items, and the rerun
it owes gains two profiles, registered here before any data:

1. **The cost model calibrates itself.** Each widget keeps a running
   estimate (an exponentially weighted mean) of its measured raster
   time per pixel and per work item, plus a fixed per-render overhead,
   seeded by `cost::PIXELS_PER_MS` and `ITEMS_PER_MS`, updated from every
   render whether or not a `Probe` is attached, and used for the inline
   or worker decision. Per-widget state only (no global).
2. **Threads follow the machine.** `Plot::threads` defaults to the
   available parallelism minus one, 1 to 8 (which respects CPU
   affinity), and with one thread the pool is the single worker; tests
   run at 1 and 2 threads.
3. **Software compositing stays cheap.** Translated tiles draw at whole
   device pixels without resampling, so tiny-skia blits them; Phase 31
   measures a 1600 × 1000 plot area of tiles drawn by tiny-skia per frame
   and reports it.
4. **The list stage for large scatters is measured and cut.** Target:
   panning `scatter100k` keeps resolve plus list under 8 ms on the UI
   thread of the fast machine (it is 0.6 + 5.2 ms now; keep it there or
   better), and `scatter1m` drops what it can (culling to the plot area,
   no per-point allocation); what remains is recorded for 0.2 (list
   building off the UI thread).

**The rerun** runs, from `dev/bench/run.sh`, after the default plan
(the verdict's profile, rule unchanged, sizes per D22.3):

- **weak profile cells** (a new plan `weak`): `hover`, `pan-x` and
  `zoom-lines` on `trace`, and `pan-x` and `zoom-lines` on `scatter100k`,
  native factor, five runs each;
- **no GPU:** `weak` with `--backend tiny-skia`;
- **4 cores and 2 cores:** `weak` on wgpu under `taskset -c 0-3` and
  `taskset -c 0-1`.

Their budgets, registered now: p99 frame interval ≤ 2 T; pan latency p95
≤ 150 ms; zoom latency p95 ≤ 250 ms; preview share ≤ 25 % during pans.
Each profile is reported met, missed or inconclusive by the same
readings; a miss is attributed as in D22.1 and becomes work for Phase 31
if that phase is still open, or a recorded 0.2 item if not. These
profiles never reopen the GPU-layer question, which stays with the
desktop profile.

### Why

The fast desktop hides costs a laptop or a GPU-less machine pays. A cost
model fixed at compile time misjudges every machine but one, and a pool
sized for 16 threads oversubscribes 2; both are cheap to get right now,
and the weak profiles make "good performance" a measured claim instead
of a hope.

## The rerun after Phase 31 (waiting for the user's run)

2026-10-28. Phase 31 is built (31a–31d: tiles and pan translation, the
render pool, the 64-stamp splat, the fitted cost model; D31.1–D31.10), so
D22.1's rule asks for this benchmark again on the user's machine. The
rule, the readings and the default plan's cells and budgets are as
registered in D22.1, with D22.3's windows (factor cells at 1600 × 1000
device px; `4k` "n/a: display too small" below 3840 × 2160) and D22.4's
weak profiles (no GPU, 4 cores, 2 cores; their own budgets and report
sections) after it. The report's outcome line reads as the rerun:
"image path sufficient — Phase 31 built, no GPU data layer", "still
raster/upload bound after Phase 31 → GPU data layer recommended", or
"UI-thread bound → move list building off the UI thread (0.2)".

**The command**, from the repository root at the commit that holds this
section, on the machine of D22.1's run (on power, other GPU-heavy apps
closed, mouse and keyboard untouched for about 75 minutes):

```bash
dev/bench/run.sh
tar czf bench.tgz -C bench-results <the stamp the script prints>
```

It builds the release bench, detects the refresh rate and factor (`HZ=60
FACTOR=1 dev/bench/run.sh` sets them), and writes
`bench-results/<stamp>/{smoke,default,weak-nogpu,weak-4cores,weak-2cores}`
with a log per plan. Checked here under Xvfb with `--smoke-only` and with
a reduced copy of the whole flow (D31's Measurements, session 31d).

**When it comes back:** the default plan's outcome is appended to D22.1
here (image path sufficient, or GPU layer recommended), each weak
profile's outcome beside it (D22.4: a miss is Phase 31 work while it is
open, else a 0.2 item), and, since Phase 34 has appended its label-churn
rerun, the closing line is written then.

## The rerun after Phase 31 (appended to D22.1)

**Date:** 2026-10-29 · the user's run `bench-results/20261028T212847Z` at
`m4-complete` (3dc1b51), same machine and display as D22.1 (RTX 3090,
1920 × 1080 at 60 Hz, COSMIC on Wayland, native factor 1).

**Desktop (`default`, rule unchanged, D22.3's sizes):** 52 checks, 31
met, 21 missed, 0 inconclusive. **Every pan is met** on `trace`,
`grid16`, `label-churn` and `scatter100k` at every factor: latency p95
15.8–16.2 ms (D22.1: 2.8–11.4 s), preview share 0.0–0.3 % (D22.1: 91–97 %),
frame p99 18.3 ms. Missed: **every zoom** (latency p95 993–1,121 ms,
about the wheel burst's length; frame p99 50–76 ms with 310–470 dropped
frames per run, D22.1 had 18 ms), and `pan-x × scatter1m` (frame p99
132 ms; list 24.5 ms and the hit index rebuilt each frame on the UI
thread). The zoom latencies attribute to raster. **Outcome under the
rule: "still raster/upload bound after Phase 31 → GPU data layer
recommended"** (a 0.2 item; nothing in v0.1 builds it).

**Weak profiles (D22.4's budgets):**

| Profile | Pans (trace, scatter100k) | Zooms |
| --- | --- | --- |
| 4 cores, wgpu | met: 15.8 ms, previews ≤ 0.4 %, frame p99 18.4 ms | missed: 1,004–1,022 ms, frame p99 55–67 ms |
| 2 cores, wgpu | met: 16.1–16.2 ms, previews ≤ 0.7 %, frame p99 18.3 ms | missed: 1,018–1,021 ms, frame p99 67 ms |
| no GPU (tiny-skia) | latency met (122 ms) and previews met (0–2 %); **frame p99 134 ms missed** (p50 25.5 ms) | missed: 12.4–12.8 s, frames 1.3–1.4 s |

**What the records show about zooms and software compositing.** Each
wheel step (about 20 a second) invalidates every tile, and with D31.6's
two-tile overscan a step queues about 80 tiles where about 30 are
visible; newest-wins per tile then leaves no generation complete until
the burst ends. Part of the tiles render inline (4–16 %), on the UI
thread, which is where the dropped frames come from (UI p99 27–49 ms a
frame), and every pool worker runs vello_cpu's own thread pool inside it
(CPU 4.4 cores busy on the desktop). Under tiny-skia every drawn image
clears a frame-sized clip mask (D22.4 item 3's measurement), and the
ring's tiles are drawn too.

## D22.5 — The zoom path and software compositing before release

**Date:** 2026-10-29 · **Context:** the rerun above · **Status:** decided

### Decision

(architect, 2026-10-29) The rule's outcome stands (a GPU data layer is
recommended for 0.2). The rerun also shows CPU-side waste that a GPU
layer would not remove, so one more session (Phase 31e) fixes it before
release, then the user reruns the zoom and weak cells (report-only, the
rule's question being answered):

1. **During a gesture, render only visible tiles;** the ring renders
   after the gesture settles (and for pans as today, one column per 256
   px).
2. **Coalesce zoom steps:** one queued generation (the newest); a tile
   already rendering finishes and is shown through the per-tile preview
   mapping instead of being dropped, so a burst shows sharp-but-stale
   tiles every render time rather than none.
3. **No inline tile renders during a gesture:** the UI thread composes;
   inline renders resume when idle.
4. **No nested parallelism:** with two or more pool threads, each worker
   renders single-threaded; vello_cpu's own pool serves the
   single-worker case only.
5. **Progressive resolution during a zoom:** while the wheel is active,
   tiles render at half device resolution and are drawn scaled; full
   resolution follows on settle. The registered latency metric keeps
   its meaning (full resolution); the bench adds latency to current
   content at any resolution beside it, report-only.
6. **Software compositing:** draw only images that intersect the
   visible plot area, and none of the ring; skip per-image clipping
   where an image lies inside its clip; measure tiny-skia's frame time
   on a pan of `trace` and `scatter100k` before and after.
7. **`scatter1m`'s hit index** is rebuilt only when hover is enabled and
   not during a gesture (rebuilt on settle).

### Why

Pans now meet every budget on every profile, including two cores;
zooms and software compositing are the remaining first-impression
problems on exactly the machines the user named, and each item above
removes work, which helps every machine, GPU layer or not.

## D22.6 — Two D22.5 items not built as written, and what measuring them showed

**Date:** 2026-10-29 · **Context:** Phase 31e, after a9eb8d5 (items 1, 2, 3, 6, 7 of D22.5, D40.4 (1), `drag-resize` and the focus plan built) · **Status:** decided

### Issue

1. **Item 4 as written measures worse.** D22.5 (4): with two or more
   pool threads each worker renders single-threaded, and vello_cpu's
   own pool serves the single-worker case only. At a9eb8d5 the pool's
   threads already render single-threaded (`iced/src/plot/worker.rs`,
   `run`: `threads: 0`, since Phase 19); the one renderer on
   vello_cpu's pool is the UI thread's inline one
   (`iced/src/plot/state.rs`, `raster::Settings::parallel()`), whatever
   the pool's size. Built as written (A below), it was slower on the only
   machine here: 2 cores, so a pool of one thread by default, whose
   worker A moves onto vello_cpu's pool.
2. **Item 5 needs a public item.** Its report-only column, latency to
   current content at any resolution, needs each frame to say how many
   shown tiles are current at half resolution. `probe::Sample::Frame`
   has public fields (`images`, `missing`, `previews`, `uploaded`) and
   the enum is not `#[non_exhaustive]`, so a field breaks every struct
   literal and exhaustive pattern of it: a public API change, which the
   brief routes here. Without the column, half resolution shows only in
   the registered latency, which it raises: full resolution follows the
   gesture's end, 300 ms after the last step, so the last step's
   full-resolution latency becomes at least 300 ms plus a render.
3. **Item 6 cannot skip the clip mask from the widget's side.** iced's
   tiny-skia engine at 3cbdacc clears and refills a frame-sized clip
   mask for every raster image whether or not it lies inside its clip
   (`tiny_skia/src/engine.rs:533`, `adjust_clip_mask` at `:776`; quads,
   text and SVG skip it when inside, raster images never do), so no
   pre-cropped render or edge-only layer avoids it. What multiplied it
   was damage: each moved image adds its old and new bounds
   (`tiny_skia/src/layer.rs:289-294`), the rects are grouped
   (`graphics/src/damage.rs:47`) and each is redrawn apart with every
   image that meets it (`tiny_skia/src/lib.rs:72-160`). 01f47a6 draws
   the gutters and tiles in one layer whose bounds alternate by half a
   logical px from one gesture frame to the next, so the layer is
   damaged whole (`layer.rs:206-209`) and each image drawn once a frame;
   previews outside the plot area are no longer drawn (the ring's own
   tiles never were). Recorded here because it is not the item's
   letter.

### Investigation

Windowed under Xvfb (2 cores, Mesa lavapipe), `--plan weak --scenario
zoom-lines --native-factor 1 --refresh-hz 60`, 2 runs per cell, items
1–3 built in every column; latency p95 ms / UI frame p99 ms / preview
share:

| Cell | Items 1–3 | A, as written | C, single-threaded everywhere |
| --- | --- | --- | --- |
| wgpu `trace` | 1,167 / 45.9 / 0.43 | 1,917 / 32.8 / 0.70 | 1,210 / 19.3 / 0.50 |
| wgpu `scatter100k` | 1,074 / 40.4 / 0.46 | 10,283 / 36.0 / 1.00 | 1,077 / 20.4 / 0.51 |
| tiny-skia `trace` | 11,471 / 12.0 / 0.96 | 5,745 / 12.3 / 0.91 | 9,997 / 10.5 / 0.71 |
| tiny-skia `scatter100k` | 11,971 / 12.9 / 1.00 | 13,063 / 26.2 / 1.00 | 10,226 / 12.0 / 0.98 |

(The tiny-skia rows predate 01f47a6, when its frames took 0.3–1.8 s.)
The probe's `Raster` samples of the same wgpu runs, median ms a render:

| | Items 1–3 | A | C |
| --- | --- | --- | --- |
| `trace`: inline cells (axis strips, ~19–32 kpx) | 3.96 | 3.94 | 0.06 |
| `trace`: pool tiles (64 kpx) | 4.89 | 11.21 | 5.52 |
| `scatter100k`: inline cells | 2.90 | 3.93 | 0.06 |
| `scatter100k`: pool cells | 0.06 | 3.99 | 0.09 |
| `scatter100k`: pool tiles | 2.46 | 12.93 | 2.82 |

vello_cpu 0.3.0's pool, at `Settings::parallel()`'s one thread on two
cores, costs about 4 ms a render whatever its size: an axis strip takes
3.9 ms on it and 0.06 ms single-threaded. That is also why the axes lag
the data mid-burst (4b). With `--threads 2`, where A, B and C all render
single-threaded everywhere, against items 1–3 alone: wgpu `trace`
1,145 → 1,119 ms, UI p99 54.7 → 32.3 ms; `scatter100k` 1,046 → 1,043
ms, 32.0 → 24.3 ms.

4. Found while measuring, before and after Phase 31e alike:
   a. **A blank plot at the settle.** When the layout settles at a
      gesture's end and a plot area's rect or the partition changes,
      `State` builds new areas and gutter runs (`same` false in the
      refresh): every image goes, and the frames until they render show
      the theme background. After a zoom burst on wgpu here: `images 1,
      missing 37` for about 300 ms (screenshots: a nearly white window
      between the last mid-burst frame and the sharp one).
   b. **Axes lag the data mid-burst.** Gutter runs render inline one or
      two a frame (3–4 ms each on vello_cpu's pool here); the rest queue
      on the pool behind the visible tiles, and each step replaces them.
      At factor 1 the axes now move every few steps, up to about 20
      steps behind the tiles (before 31e they did not move until the
      burst ended); at factor 2 they still do not move until the end.
   c. **tiny-skia at factor 2 under Xvfb** (3200 × 2000): a burst's
      frames take 9.5 s (before: 7.8 and 17.6 s), against an 84 ms frame
      p99 at factor 1. The time is in iced's compositing and present,
      which the probe does not see; not investigated further. The focus
      plan's factor cells run on the native backend only.
   d. **Item 3's price on pans.** A drag's entering column now renders
      on the pool, a frame or more later. wgpu, `pan-x`, 3 runs, latency
      p95 / preview share / frame p99 / UI frame p99 / dropped frames,
      items 1 → 1–3 → all of 31e: `scatter100k` 29 ms / 2 % / 43.6 /
      24.6 / 180 → 71 ms / 10 % / 42.8 / 21.8 / 191 → 46 ms / 6 % /
      33.1 / 16.5 / 51; `trace` 356 (one outlier run) / 9 % / 54.6 /
      31.8 / 160 → 94 / 9 % / 37.1 / 20.3 / 120 → 75 / 8 % / 32.4 /
      16.4 / 83. Within the weak budgets (150 ms, 25 %); smoother frames
      for a little latency.
   e. **wgpu zoom frames on `trace`** (lavapipe, 2 runs) got slower
      with items 1–3: frame p95 64 → 83 ms, p99 84 → 123 ms, dropped
      frames 383 → 412, UI frame p99 46.9 → 42.0 ms; on `scatter100k`
      p99 107 → 95 ms. No measurement here separates the three items;
      likely lavapipe's rasterizer sharing the two cores with a worker
      that item 3 keeps busy through the burst. The user's focus run
      reads it on real hardware; if zoom frames are worse there, item 3
      is the first to revisit.

### Options

Item 4:

| | A — as written | B — the two-thread half | **C — single-threaded everywhere** |
| --- | --- | --- | --- |
| Approach | ≥ 2 pool threads: the inline renderer single-threaded too; one thread: the worker and the inline renderer on vello_cpu's pool | ≥ 2 threads: the inline renderer single-threaded; one thread: as at a9eb8d5 | The inline renderer single-threaded whatever the pool's size; nothing in the widget uses vello_cpu's pool (`Settings::parallel()` stays for the file outputs) |
| Cost | Built, then reverted (about 60 lines) | About 20 lines | About 10 lines and a test |
| Measured here | Zoom latency p95 up to 10× worse with one worker | Equal to C at ≥ 2 threads; unchanged with one | Latency unchanged, UI frame p99 about halved, inline renders 50× cheaper |
| Fit | D22.5's letter | Keeps the 4 ms floor on 2-core machines | One setting; the pool is the only parallelism, D22.5's aim |

Item 5:

| | A — a field | **B — a field, `#[non_exhaustive]`** | C — inferred by the bench |
| --- | --- | --- | --- |
| Approach | `Sample::Frame` gains `coarse: u32`: shown tiles current at half resolution, counted in `previews` too, so the registered latency keeps its meaning | A, and `Sample`'s variants `#[non_exhaustive]`, so later fields are additive | The bench guesses half-resolution tiles from `Raster` samples' pixels |
| Cost | One breaking change in the unpublished `rustd3-iced`; the bench reads it | A plus `..` in the bench's patterns | No library change; tied to the tile size; cannot see which tiles show |
| Correctness | Exact | Exact | Heuristic |

### Decision

(architect, 2026-10-29) **Item 4: C.** The widget's inline renderer is
single-threaded whatever the pool's size; nothing in the widget uses
vello_cpu's pool (`Settings::parallel()` stays for the file outputs).

**Item 5: B, made progressive.** `Sample::Frame` gains `coarse` and
`Sample`'s variants become `#[non_exhaustive]`. During a wheel gesture a
generation queues its visible tiles at half resolution first. The same
generation's full-resolution tiles follow at once, without waiting for
the settle, and newer steps coalesce both passes as item 2 does. Where
full resolution keeps up (the desktop's pool), the registered latency
does not rise. Where it cannot, the half pass shows current content. The
session builds it with its seam and placement tests at 1, 1.25, 1.5 and
2 on both backends, the `Verify` rule (full resolution only), the
any-resolution column in the focus report and `summary.csv`, and
mid-burst screenshots. If it measures worse here, it is not committed.

**3:** accepted as built (one layer whose bounds alternate while a
gesture lasts, each image keeping its own clip).

**4a:** as recommended. The old areas' imaged tiles carry into the new
areas' preview layers through the per-tile map, and each old gutter
image stays until its run renders. No frame shows the background where
content was.

**4b:** C, and gutter runs ahead of tiles on the pool while a gesture
lasts. A gutter run whose predicted cost is under 1 ms may also render
inline during a gesture: item 3 keeps tile-sized renders off the UI
thread, not 0.06 ms axis strips. The axes keep step with the data.

**4c:** investigated in the same session, time-boxed to about an hour.
Find where a 9.5 s frame at factor 2 on tiny-skia under Xvfb goes:
iced's compositing, the present, or Xvfb itself. The control is a plain
iced app drawing the same number of images at the same sizes. Fix it if
the cause is on the widget's side; otherwise record the finding here.

**4d:** as recommended: drags keep item 3 unless the user's focus run
shows a pan missing a budget.

**4e:** read from the user's focus run, which runs at 920ac26 while the
next session builds the above. A second focus run follows that session.

The original recommendation, for the record: C for item 4; B for item 5, built in a following
session with its seam and placement tests at 1, 1.25, 1.5 and 2 on both
backends, its `Verify` rule, the any-resolution column in the focus
report and `summary.csv`, and mid-burst screenshots; 4a and 4b in the
same session (4a: carry the old areas' imaged tiles into the new areas'
preview layers through the per-tile map, and keep each old gutter image
until its run renders; 4b: C, and gutter runs ahead of tiles on the
pool while a gesture lasts). 3 needs no answer; 4d: keep item 3 for
drags unless the user's focus run shows pans missing a budget, in which
case drags render inline again and only wheel gestures and brushes
leave tiles to the pool.

### Why

C removes the one measured cost item 4 aimed at (a pool inside a pool,
here a 4 ms dispatch floor per inline render) without the regression A
brings to the single worker; with two or more threads A, B and C agree.
Item 5's column is what makes half resolution judgeable; built without
it, the only number that moves is the registered latency, upward. 4a
and 4b are what a user sees at the end and in the middle of every zoom,
on every machine.

### Followups

- The user's `dev/bench/run.sh --focus` gives the numbers on the
  Ryzen; 4c is worth a tiny-skia cell at factor 2 there if the weak
  profiles' native factor is 1.
- Phase 31f: items 4 (bdfa439) and 4a (2e47bf1) built; 4b and item 5
  measured worse as written and were not committed (D22.7).
- 4a's price, Phase 31f, read back to back under Xvfb (wgpu, `--runs 1`,
  two rounds of bdfa439, 2e47bf1 and 7cba714 interleaved, since this
  container's wgpu frames drifted 30–40 % slower over the session, hover
  included): `zoom-lines` × `trace` and `scatter100k` unchanged (frame
  p50 15–18 ms without 4a, 12–21 ms with it); `zoom-lines` ×
  `scatter1m`, whose tiles take seconds, now draws 24 carried previews
  where 18–21 tiles were blank in 63–76 % of frames before (0–1 % now),
  and its frame interval p50 reads 28 and 31 ms before, 33–60 ms after
  (noisy; latency unchanged). On tiny-skia the same cell's frame p99
  reads 116 and 117 ms in the two builds without 4a (bdfa439 and 4b's),
  197 and 198 ms in the two with it (item 5's and 7cba714).
- 4c, found (Phase 31f, 7cba714): the time was iced's compositing, on
  the widget's side. gdb samples through the 10–11 s frame put the main
  thread in `iced_tiny_skia`'s `Compositor::present` → `Engine::draw_image`
  → `draw_pixmap` (bilinear), with Xvfb idle; a plain iced app replaying
  that frame's 87 image draws in a layer whose bounds alternate takes
  183 ms a frame, and 92 synthetic previews 180–190 ms. The widget's
  half-px toggle flipped on idle draws as well, so after an odd number of
  them the first draw of a gesture reused the idle layer's bounds;
  tiny-skia then damaged each moved preview apart and redrew every image
  in each damaged rect. Holding the toggle while idle: the factor-2
  tiny-skia burst's longest frame interval 1.2 and 0.8 s (two runs)
  against 11.2 and 10.7 s, the run ending after 6.0 and 5.7 s against
  14.1 and 13.9 s. While previews show, a tiny-skia frame at factor 2
  still costs about 0.2 s here (each scaled image filtered bilinearly
  through a frame-sized clip mask); 2e47bf1 keeps the alternating layer
  while carried previews or old gutter images show, for the same reason.

## D22.7 — Two D22.6 items measure worse as written: 4b and item 5

**Date:** 2026-10-29 · **Context:** Phase 31f: D22.6 (4b) built on bdfa439 (item 4, C), item 5 (B made progressive) built on 2e47bf1 (4a), each measured and not committed · **Status:** decided

Both patches apply to 7cba714 as they stand:
`/tmp/claude-0/rustd3-bench31f/patches/0001-perf-iced-during-a-gesture-the-axes-render-ahead-of-.patch`
(4b, with its tests) and `…/patches/item4-as-built.patch` (item 5:
widget, probe, bench and tests; 16 files, +951 −88). `/tmp` does not
outlive the container.

### Issue

**4b.** D22.6 (4b), as the brief wrote it: while a gesture lasts,
gutter runs go ahead of tiles on the pool, and "a gutter run whose predicted cost
(the D31.8 cost model) is under 1 ms may render inline during a
gesture. Item 3's no-inline rule stays for tiles and anything predicted
at 1 ms or more." Item 3 (D22.5 (3), d8ec675) kept tiles off the UI
thread, not gutters: before 4b any non-tile render that fit half the
frame budget rendered inline during a gesture. The 1 ms bound is
therefore a new restriction, and the fitted model predicts an axis strip
far above what it costs: the fit is dominated by the pool's contended
tile renders (50–80 µs per kpx here), so a 31.5 kpx strip predicts about
2 ms while it renders in 0.04–0.4 ms single-threaded since item 4, C.
Built as written, nearly every axis strip goes to the pool, during drags
too.

**Item 5.** Built as D22.6 decided it: during a wheel gesture a
generation queues its visible tiles at half device resolution, then the
same tiles at full; a newer step replaces both passes; a half tile is
doubled on the CPU (0.5–0.8 ms a tile) and drawn on its full square,
since tiny-skia's `draw_pixmap` truncates a scaled image's position to
whole texels and puts a GPU-scaled half tile on even device px only;
`Sample::Frame` gains `coarse`, `Sample`'s variants are
`#[non_exhaustive]`, `Verify` compares full-resolution tiles only, and
the bench reports the any-resolution latency in the focus report, the
default report's zoom rows and `summary.csv`. Its seam and placement test
(no background px in the plot area, the panel's edges on the area's
first and last device rows and columns, in every frame with half tiles)
passes at 1, 1.25, 1.5 and 2 on both backends and fails on both when the
half tiles are shifted 1 px. Screenshots mid-burst at 1 and 2 on both
backends showed no seam and no misplaced tile. It measures worse: half
resolution does not make these renders cheaper.

### Investigation

**4b.** Windowed under Xvfb (2 cores, Mesa lavapipe), `--plan weak` (and
its zoom cells at `--threads 2`), `--native-factor 1 --refresh-hz 60`, 2 runs
per cell: item 4, C alone (bdfa439); A as written; B the order alone (the
gutters and overlays ahead of the tiles while a gesture lasts, inline as
before 4b). The axis lag is, per generation planned in a zoom burst, the
time from its first frame to the first render of each axis strip (gutter
cells of 20 kpx or more) for it or a newer one, median over generations
and strips (`/tmp/claude-0/rustd3-bench31f/axis.py`):

| Cell | C alone | A as written | B order only |
| --- | --- | --- | --- |
| wgpu `zoom-lines` × `trace`: axis lag p50 ms | 387 | 52 | 38 |
| the same, `--threads 2` | 267 | 77 | 63 |
| tiny-skia `zoom-lines` × `trace`: axis lag p50 ms | 212 | 36 | 46 |
| the same, `--threads 2` | 142 | 46 | 84 |
| wgpu `zoom-lines` × `trace`: strip renders inline | 441 / 500 | 25 / 1,058 | 521 / 1,114 |
| wgpu `zoom-lines` × `trace`: frame p50 / p99, UI frame p99 ms | 9.9 / 108 / 17.3 | 12.9 / 109 / 19.0 | 10.2 / 109 / 17.1 |
| wgpu `pan-x` × `trace`: latency p95 ms, preview share | 49, 6 % | 137, 11 % | 52, 7 % |
| tiny-skia `pan-x` × `trace`: latency p95 ms, preview share | 43, 4 % | 131, 11 % | 51, 6 % |
| wgpu `pan-x` × `scatter100k`: latency p95 ms | 39 | 67 | 36 |

On `scatter100k` the strips already kept step (lag p50 0 ms in every
column). Registered zoom latency p95 stays about 1.1 s in every column.
A's pan latencies p95 are about 2.7 times C's on both backends (the
tiny-skia intervals do not overlap: 29–56 against 118–144 ms) and its
preview share doubles; its wgpu zoom frames take 3 ms more at the median
(more strip renders on the pool beside lavapipe). B's binary also held
D22.6 (4a) before its alternating layer (2e47bf1), which slowed the
settle frames of tiny-skia `scatter100k` zooms; its other rows are not
affected.

**Item 5.** The same setup, `--plan weak --scenario zoom-lines`, the two
binaries run back to back (2e47bf1 without item 5, then with it), 2 runs
per cell; `any` is the any-resolution latency, the coarse share the
share of shown tiles current at half resolution:

| Cell | Latency p95 ms (any) | Frame p50 ms | Preview share | Coarse share |
| --- | --- | --- | --- | --- |
| wgpu × `trace` | 1,216 → 2,511 (2,511) | 12.5 → 18.4 | 0.52 → 0.76 | 0.22 |
| wgpu × `scatter100k` | 1,096 → 1,147 (1,100) | 13.5 → 19.2 | 0.57 → 0.68 | 0.11 |
| wgpu × `trace`, `--threads 2` | 1,141 → 1,286 (1,286) | 13.2 → 12.9 | 0.40 → 0.53 | 0.11 |
| wgpu × `scatter100k`, `--threads 2` | 1,047 → 1,112 (1,089) | 12.4 → 12.2 | 0.43 → 0.48 | 0.06 |
| tiny-skia × `trace` | 1,177 → 1,783 (1,783) | 1.0 → 1.0 | 0.10 → 0.15 | 0.05 |
| tiny-skia × `scatter100k` | 1,102 → 1,144 (1,116) | 0.9 → 1.0 | 0.27 → 0.44 | 0.08 |
| tiny-skia × `trace`, `--threads 2` | 1,157 → 1,244 (1,244) | 1.0 → 1.0 | 0.08 → 0.11 | 0.02 |
| tiny-skia × `scatter100k`, `--threads 2` | 1,139 → 1,706 (1,697) | 0.9 → 1.0 | 0.31 → 0.31 | 0.03 |

Every latency rises; the any-resolution latency is within 5 % of the
registered one, so the half pass does not show current content sooner
either. The pool's raster records put a half tile of `trace` at a median
6.2 ms against 5.8 ms at full. Timed alone in a release build (one
256-px tile, median of 25, single-threaded), a 400k-point line costs
116.5 ms at full and 81.8 at half at factor 1, 55.5 and 62.0 at factor
2; 100k dots 4.9 and 4.6 ms, 3.4 and 4.2 ms. vello_cpu's time on these
lists goes to the geometry (flattening and stroking each segment, each
dot's path), not to the px, so a half pass adds a render per tile and
step instead of replacing a dearer one; on two cores it delays the full
pass. At factor 2 no frame of the mid-burst screenshots showed half
tiles: each step was replaced before its half pass landed.

### Options

4b:

| | A — as written | **B — the order alone** | C — a fit per kind of render |
| --- | --- | --- | --- |
| Approach | Gutters and overlays ahead of tiles on the pool while a gesture lasts; inline only those predicted under 1 ms | Gutters and overlays ahead of tiles on the pool while a gesture lasts; inline, as before 4b, any non-tile render that fits half the frame budget | A, with the cost model fitting gutters and tiles apart, so that a strip's prediction is its own |
| Cost | Built (about 40 lines and two tests), reverted | About 10 lines and a test of the order | A plus a second fit in `budget.rs` |
| Measured here | Axes in step; pans worse (p95 2.7×, previews 2×); wgpu zoom frames +3 ms | Axes in step as A; pans and frames as C alone | Not measured |
| Fit | The brief's letter | D22.6's aim ("the axes keep step with the data") with D22.5 (3)'s letter (tiles off the UI thread) | The bound means what it says |

Item 5:

| | A — as built | **B — no half pass; the probe made additive** | C — half resolution where px cost |
| --- | --- | --- | --- |
| Approach | The patch above | `Sample`'s variants `#[non_exhaustive]` and the bench's patterns `..`; no `coarse`, no half pass, no any-resolution column | A, with the half pass only for renders the cost model predicts px-bound (images, heatmaps, large fills), a fit per kind as 4b's C |
| Cost | Built (+951 −88), reverted | About 15 lines from the patch | A plus a second fit in `budget.rs`; no workload here to measure it on |
| Measured here | Latency p95 up 4–106 %, any-resolution no better; wgpu frames +6 ms on one thread | As 2e47bf1 | Not measured |
| Fit | D22.6's letter | Later progressive fields are additive | Half resolution where it saves |

### Decision

(architect, 2026-10-29) **4b: B**, the order alone. **Item 5: B**:
`Sample`'s variants become `#[non_exhaustive]`, with no half pass and no
any-resolution column. Half resolution is closed for 0.1. The user's run
of 31e on the Ryzen (`bench-results/20261029T071522Z`, the full plan)
shows that `trace`'s zoom latency is the cost of each tile's geometry
(raster critical path 134 ms on 8 workers, the cost model's calibration
12×). That cost is what the next session goes after, not pixels.

The original recommendation, for the record: for 4b, B. The axes' lag came from the pool's order, not
from inline renders: behind the tiles each step replaced, a strip that
did not fit the frame budget never rendered until the burst ended. Ahead
of them it renders within a frame or two, and the strips that fit stay
inline, which is what keeps drags as they were.

Recommendation for item 5: B, and close half resolution for 0.1. If the
user's focus run on the Ryzen shows zoom latency still short of what
the architect wants, C is the form to revisit, with a px-bound workload
in the plan to judge it by.

### Why

4b: A's bound trusts a prediction that is ten times a strip's real cost,
so it moves cheap renders off the UI thread and delays every drag's
frame until the pool returns them; B keeps D22.5 (3) as it was built and
only changes what the pool takes first.

Item 5: half resolution saves where a render's cost scales with its px.
The lines and dots of the bench's workloads cost per segment and per
dot, so a half pass is a second render of nearly the same cost; on a
machine whose pool keeps up the full pass lands as soon, and on one that
does not, the half pass is what delays it.
