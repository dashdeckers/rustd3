# Phase 19 — Plot II: render pipeline

## Measurements

2026-10-27, aebb7b8 … 3b21383; `scripts/check.sh full` green at 3b21383
in 4 min 19 s (warm), no warnings. Phase 19 does no core work: the
harness stays at 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

**Debug-build render times** (vello_cpu at opt-level 0, as `cargo test`
builds it; `Settings::parallel()` on this 2-core container, one pool
thread). The scatter of `iced/tests/main/pipeline.rs` at 1600 × 1000
logical at 2 (3200 × 2000 device px), its plot-area cell about 5.7 million px,
rendered inline:

| Dots | Its work items | Plot-area cell |
| --- | --- | --- |
| 0 | 178 | 0.83 s |
| 2,000 | 2,186 | 2.4 s |
| 5,000 | 5,186 | 3.4 s |
| 20,000 | 20,186 | 7.4 s |
| 200,000 | 200,186 | 62.9 s |

The cost model gives the whole 2k-dot figure 20.6 ms (summed over its
cells' raster samples) and the 200k-dot one about 513 ms (the section's
row). A harness frame at 3200 × 2000 (draw and screenshot) adds about 4 s on
tiny-skia in a debug build. The probe's rates in the unit tests (the
2 × 2 grid at 2, 1,228,800 px, concurrent tests): about 1,500 px/ms
inline and 2,100 px/ms on the worker, against the model's 5e5 (release,
single-threaded): D19.3.

**The section's tests** (both backends, `crisp` on, `web-colors` on):

| Test | tiny-skia | wgpu (lavapipe) |
| --- | --- | --- |
| `worker_mode_matches_inline_mode` (2k dots, D19.1) | 0/255; first wake 1.5–2.3 s; current after 2 wakes | 0/255; first wake 2.3–3.2 s; current after 3 wakes |
| `cheap_refresh_renders_inline` (320 × 200 at 2) | model 2.0 ms, 9 cells inline, no wake | same |
| `expensive_refresh_uses_the_worker` | model 20.6 ms, first frame missing 9, `NextFrame` after the `Waken` | same |
| `newest_result_wins` | generation 2 everywhere, 1/255 against its PNG; 9 held generation-1 results dropped, no redraw | same, 1/255 |
| `previous_image_stays_until_the_new_one_arrives` | k = 1 and 2: the old frame pixel for pixel, previews 1; then cell 5 only (115 / 359 px), 1/255 against the new PNG | same |
| `worker_exits_on_drop` | 0 threads before, 2 with a set (the worker and its pool), 0 after | same |

Phase 18's two-panel figure models 7.98 / 8.19 / 8.72 ms at 1.25 / 1.5 /
2, against the first frame's 8.35 ms threshold, so its reference tests
render inline explicitly (D19.2 (13)).

**Renders viewed** (headless screenshots, scratch, not committed): the
legend figure at 1 and 2 on both backends mid-job (the old title "A",
every other pixel unchanged) and after the result (title "B"); the
2k-dot scatter at 1 and 2 after its first wake (the plot-area cell shown
with its axis lines, the gutters the theme's white background) and once
current.

## D19.1 — The section's 200k-dot harness tests in a debug build

**Date:** 2026-10-27 · **Context:** Phase 19, step 3, 685d6bd · **Status:** decided

### Issue

Steps 3: `worker_mode_matches_inline_mode` ("a 1600 × 1000 logical
figure at 2.0 with 200k dots … repeat `wait_for_wake(10 s)`") and
`expensive_refresh_uses_the_worker` ("the 200k-dot figure under
`Render::Auto`: model ≈ 513 ms"). The tests run in a debug build
(`check.sh` runs `cargo test` without `--release`, and `[profile.dev]`
leaves dependencies at opt-level 0). There vello_cpu renders the
figure's plot-area cell in 62.9 s (Measurements); the worker renders
plot-area cells first, so the first wake comes after about a minute and
the ten-second wait fails. The worker and the inline render take two
minutes per backend, and `check.sh full` runs the iced tests three
times.

### Investigation

Measured with a scratch test (Measurements); the model's 513 ms is a
release, single-threaded figure.

### Options

| | **A — 2k dots, as built** | B — 200k dots, longer waits | C — optimize dependencies in dev |
| --- | --- | --- | --- |
| Approach | The same 1600 × 1000 canvas at 2 with 2,000 dots (`DOTS`): the model gives 20.6 ms, so `Auto` still sends it to the worker (the pixels alone are 12.8 ms > 8.35); the 513 ms row is asserted on the model (`cost_model_matches_the_spot_rows`, `expensive_cells_are_predicted_for_the_worker`) | Waits of 120 s | `[profile.dev.package."*"] opt-level = 2` |
| Cost | — | About 5 more minutes per iced run, 15 per `check.sh full` | A manifest change (a decision), a cold rebuild of every dependency |
| Correctness | Every behaviour the tests name is exercised; the first wake after about 2.4 s | As written | As written, and every figure test faster |

### Decision

C, targeted (architect, 2026-10-27). The workspace's dev profile
optimizes the few dependency crates the debug render spends its time
in (vello_cpu and its pipeline crates; add tiny-skia or others only if
a measurement shows they dominate a test), with
`[profile.dev.package.<crate>] opt-level = 3` entries beside the
existing `"*"` entry. Done as the first commit before Phase 20's steps:
measure the 200k-dot plot-area cell in a debug build before and after,
record the cold-build and `check.sh full` time changes in this file's
Measurements, and restore the section's 200k-dot tests with its 10 s
wait if the cell renders well inside it; otherwise keep 2k dots and
record why. Results do not change: optimization levels leave IEEE
arithmetic as it is, so every pixel test keeps its bound.

### Why

The tests check the pipeline's behaviour, which the dot count does not
change once the model predicts the worker; the 513 ms prediction is a
property of the model, tested there.

### Followups

Phase 22's benchmark (release) renders the 200k-dot case.

## D19.2 — Details of Phase 19 as built

**Date:** 2026-10-27 · **Context:** Phase 19, aebb7b8 … b82e188 · **Status:** decided

### Issue

1. **Commit order.** Step 2 (budget, cost, `Render`) before step 3 (the
   worker) leaves `Budget` without a caller: `Auto` has nowhere to send
   an expensive refresh, and the workspace denies dead code.
2. **`Probe`'s path.** The layout puts `Probe` and `probe::{Sample,
   Stage, Target}` in `probe.rs`; `Plot::new(fig).probe(Probe::new(…))`
   names it at `plot::Probe`.
3. **Abandoned jobs.** "A refresh replaces the pending set" and "the
   worker … abandons the rest", with jobs planned for the changed cells
   only: a slot whose job was in the abandoned rest and whose content
   did not change again is never rendered, and stays missing or a
   preview.
4. **Results of an older partition.** "A different partition gets new
   ids and every slot is new", and a result is promoted when the slot
   has no image: a result of the old partition at the same index would
   land in a slot of another rect.
5. **The budget's instants.** "One `RedrawRequested` instant serves the
   up to three passes of a frame": recording each pass gives an interval
   of 0, clamped to 1 ms, and every refresh after the first pass goes to
   the worker.
6. **"The theme background shows"** for a slot without an image: the
   widget drew nothing there.
7. **`uploaded`** "accumulated in `update` as slots are promoted": two
   results for one slot in one drain would count an image no frame drew.
8. **A `Raster` sample's `frame`** for a worker result.
9. **Fields beyond the section's:** `Done` needs its cell's rect (the
   sample's pixels) and the bands' rows; the worker needs the probe's
   clock with `enqueued`.
10. **A thread the OS will not start** (`thread::Builder::spawn` errs).
11. **Thread counts.** vello_cpu's rayon pool threads, spawned from the
    worker, inherit its name on Linux, and rayon terminates them
    without joining.
12. **Concurrent tests** that start workers disturb each other's
    counts.
13. **Phase 18's reference tests** compare the first frame with the
    PNG; under `Auto` its figure (8.72 ms at 2) goes to the worker.
14. **Calibration** of a cell the model gives 0 ms (ratio not finite).
15. **The 144 Hz threshold.** 6.9 ms through `as_secs_f64() × 1e3 / 2`
    is 3.4499999999999997.
16. **Redraws without a refresh** (nothing rebuilt, resized or
    rescaled, Phase 18's fast path).

### Investigation

Compiler and lints (1); the iced sources at 3cbdacc (5,
`winit/src/lib.rs:777-830`); rayon-core 1.13.0
`thread_pool/mod.rs:399-403` (`Drop` terminates the registry) and
`/proc/self/task/*/comm` (11); the scratch measurements above (13).

### Options

| | **A — as built** |
| --- | --- |
| (1) | Five code commits: step 1 (aebb7b8), step 3 with `Render::{Inline, Worker}`, `Inline` the default (685d6bd), step 4 (5ee36c6), step 2 with `Auto` the default (a326c94), step 5 (b82e188) |
| (2) | `pub mod probe` with all four types, `pub use probe::Probe`: two paths, as D18.2 (5) for `Plot`; `plot::Render` (private module), `pub mod cost` |
| (3) | A worker refresh re-plans every slot still pending, at the new generation, plot-area cells first; an inline refresh that leaves no slot pending replaces the worker's set with an empty one |
| (4) | Results older than the generation that rebuilt the slots are dropped (and indices outside the slots) |
| (5) | `Budget::frame` records an instant only when it is later than the last |
| (6) | A quad of the theme background over each slot without an image |
| (7) | Each image is counted once, by the first `Frame` sample after it was promoted |
| (8) | The frame counter when the sample is sent: the `RedrawRequested` that rendered it inline, or the last one before the drain |
| (9) | `Done { rect, rows, … }`; `Job::enqueued: Option<job::Stamp { clock, at }>` |
| (10) | The cells render inline |
| (11) | `worker_exits_on_drop` asserts more named threads with a set than before, and polls (1 ms, at most 10 s) until the count is back after the drop |
| (12) | Tests that start a worker hold a test-local mutex |
| (13) | Those tests pass `.render(Render::Inline)` |
| (14) | Skipped |
| (15) | The threshold from `as_nanos() / 1e6 / 2`: exactly 3.45 |
| (16) | Kept: no resolve, but the `Frame` sample is sent |

### Decision

A for all sixteen (architect, 2026-10-27), as built.

### Why

(1), (9) and (15) follow the compiler, the lints and the arithmetic;
(3), (4), (5) and (7) are what "newest wins", "every slot is new", "the
budget is the interval" and "drawn for the first time" need to hold;
(6) does what the section says the frame shows; (2), (8), (10), (14)
and (16) are the closest readings; (11)–(13) keep the tests
deterministic.

### Followups

Phase 20: gestures add generations ("Phase 20 adds applied input"); a
gesture's re-plans go through (3).

## D19.3 — The cost model in a debug build

**Date:** 2026-10-27 · **Context:** Phase 19, "Decisions this phase may meet" · **Status:** decided

### Issue

The constants misjudge the machine the tests run on: a debug build
renders about 1,500–2,100 px/ms (Measurements), not 5e5, so without a
probe `Auto` predicts inline renders that take hundreds of
milliseconds there.

### Decision

The section's answer: the constants stay as the no-probe fallback; a
probe's calibration corrects them after 8 samples; Phase 22's benchmark
records the measured factor in release, and its verdict may move them.

### Followups

Phase 22: record the calibration factor of each scenario.

## Measurements after D19.1

2026-10-27, before Phase 20's steps. The dev profile optimizes four
dependency crates (`[profile.dev.package."<crate>@<version>"] opt-level
= 3`): `vello_cpu@0.3.0`, `vello_common@0.3.0`, `fearless_simd@0.7.0`
and `tiny-skia@0.12.0`. The specs name versions because hayro (typst's
PDF images) depends on `vello_cpu` 0.0.8, `vello_common` 0.0.8 and
`fearless_simd` 0.4.1 and 1.2.0: a bare name optimizes those too and
rebuilds typst-library for nothing. A spec that stops matching after a
lockfile change makes cargo warn.

**The plot-area cell** of `iced/tests/main/pipeline.rs`'s scatter
(1600 × 1000 logical at 2; about 5.7 million px), rendered inline
through the widget, `Raster` sample durations, debug build, tiny-skia
harness:

| Profile | 0 dots | 2,000 dots | 200,000 dots |
| --- | --- | --- | --- |
| Before (every dependency at opt-level 0) | 0.82 s | 2.2 s | 59.3 s |
| `vello_cpu` only | 0.65 s | 1.0 s | 34.1 s |
| `vello_cpu`, `vello_common`, `fearless_simd` | 33 ms | 73 ms | 0.94 s (wgpu harness: 0.98 s) |
| the same and `kurbo` (not kept) | 33 ms | 65 ms | 0.97 s |

`kurbo` gains at most 11% and would rebuild typst-library, usvg and
iced's renderers, so it is left at opt-level 0. A harness frame at
3200 × 2000 on tiny-skia took about 3.5 s (iced_tiny_skia draws through
tiny-skia at opt-level 0) and dominated the tiny-skia run of
`worker_mode_matches_inline_mode`; with `tiny-skia@0.12.0` optimized,
rustd3-iced's integration tests on tiny-skia take 11.8 s instead of
26.5 s. Optimization levels leave IEEE arithmetic as it is: every pixel
test keeps its bound.

**The section's 200k-dot tests are restored** with the 10 s wait
(`DOTS = 200_000`):

| Test | tiny-skia | wgpu (lavapipe) |
| --- | --- | --- |
| `worker_mode_matches_inline_mode` | 0/255; first wake 0.45 s; current after 2 wakes | 0/255; first wake 0.88 s; current after 3 wakes |
| `expensive_refresh_uses_the_worker` | model 515.6 ms (asserted within 1% of the section's 513: 12.8 ms of pixels, 500.5 of dots, the axes' labels) | same |
| `worker_exits_on_drop` | 0 threads before, 2 with a set, 0 after | same |

**Build times.** Compile time of the optimized crates on this 2-core
container, measured with `cargo build --timings` in a scratch target
directory (default features), opt-level 0 → 3: `fearless_simd` 3.9 →
4.4 s, `vello_common` 2.3 → 8.1 s, `vello_cpu` 5.0 → 17.3 s, `tiny-skia`
1.6 → 4.2 s; 21 s of compile time more in all. A cold build grows by
at most that (`vello_common` and `vello_cpu` lie on rustd3-figure's
critical path; typst builds alongside); the whole cold build was not
repeated. The switch itself rebuilds the dependents of those crates
once (rustd3-figure, rustd3-iced, iced's renderers, the dev crates).
