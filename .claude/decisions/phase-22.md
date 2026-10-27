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

## D22.1 — The benchmark verdict: pending the user's run

**Date:** 2026-10-27 · **Context:** Phase 22, step 5 · **Status:** open — waiting for the user's windowed run

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

Pending the user's run. When the results arrive the agent writes the
verdict here from `report.md` and `summary.csv`, applying the readings as
written; an ambiguity found in real data is noted beside the verdict and
recorded as its own entry (the section's "Decisions this phase may
meet"). Inconclusive cells are rerun with `--runs 10` before the verdict.

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

**Date:** 2026-10-27 · **Context:** Phase 22, 477c550 … e533558 · **Status:** open

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

Recommended: A for all fifteen, as built.

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
