# rustd3 — session handoff 51

2026-10-29 · **Phase 31g is done** (D22.6 4d; D22.7 4b B and C, item 5 B;
software compositing; long strokes cut per tile; `rustd3-bench suite`) ·
no decision open · next: the user's run on 31g (`dev/bench/run.sh
--focus` on Linux, `rustd3-bench suite --focus` on Windows), and the
architect's Linux diagnostic of tiny-skia's gesture frames

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31g on b948663, one commit per item, each with a test that fails
without it (table, numbers and before/after in
`.claude/decisions/phase-22.md`, "Measurements, Phase 31g"):

- 3cbf0f0 `perf(iced)` D22.6 (4d): a drag's entering column renders
  inline when the cost model says it fits; wheel gestures and brushes
  keep their tiles on the pool.
- 0d9cef5 `perf(iced)` D22.7 (4b), B: during a gesture the gutters and
  overlays wait ahead of the tiles on the pool.
- 7b9a614 `perf(iced)` D22.7 (4b), C: the cost model fits tiles apart
  from gutters and overlays (`budget::Kind`); strips render inline.
- 35f4712 `feat(iced)` D22.7 item 5, B: `probe::Sample`'s variants are
  `#[non_exhaustive]`; the bench's patterns have `..`.
- e2e4c39 `feat(bench)` item 4's harness: `also` composes a real gesture
  frame on headless tiny-skia (`dev/bench/src/also/compose.rs`).
- 81b7301 `perf(iced)` item 4 (b): on a renderer that reports no scale
  (tiny-skia), an area that shows a preview layer is composed on the CPU
  and drawn as one image (`iced/src/plot/mosaic.rs`, `Area::draw_mosaic`).
- 8334798 `perf(figure)` item 5: a render cuts long stroked paths to the
  segments within the stroke's reach of the area (`figure/src/raster/cull.rs`);
  dashed runs are dashed from the path's phase there and stroked in one
  call; `also` profiles one `trace` tile.
- a5ad88c `feat(bench)` item 6: `rustd3-bench suite`; `run.sh` a wrapper;
  `run --cpu-limit`.
- 15762cb `test(figure)`: the cut test's module is gated on the `raster`
  feature (the first `check.sh full` failed to compile it without it).
- Docs: the measurements and D22.7's followups (this commit).

Not built, and why (no D22.8: nothing that was built measured worse):

- Candidate (a), nearest at 1:1, was already the state: tiles and gutters
  are drawn `Nearest`, and tiny-skia draws an integer-translate image
  nearest whatever its filter.
- (b) for pans: the harness shows nothing at factor 1 and 10–25 % at 2,
  for a whole-area conversion every frame; pans draw per tile as before.
- Item 5's binary search for x-monotone lines: the general
  segment-by-segment clip costs a few µs a tile against the milliseconds
  of stroke expansion it removes.

Harness unchanged (no core or harness change).

## 3. Measured here (not rule outcomes)

Windowed under Xvfb (2 cores, lavapipe, no vsync), 2 runs per cell,
`--threads 1` and `2`; full tables in phase-22.md. Outputs and scripts in
`/tmp/claude-0/rustd3-bench31g/` (`measure.sh`, `tab.py`, `ba.py`,
`axis.py`, `shots.sh`; `before/` and `after/` the brief's table; `B/`,
`BC/`, `mosaic/`, `mosaicZ/`, `cull/` the steps; `compose*.md` and
`geo-runs.txt` the `also` rows; `shots/*/keep/` a few frames).

- Pans: latency p95 55–163 → 20–30 ms, previews about 0.
- Zooms: frame p99 down a third to a half on both backends, latency p95
  1.0–1.1 s (one or two pool threads cannot finish a step's tiles in 33
  ms here). The trace strips' lag p50 300–426 → 0 ms.
- tiny-skia zoom frames: harness `trace` mid-burst 92 → 28 ms at 1 and 573
  → 99 ms at 2 (draw + first composition); UI frame p99 rises 13 → 24 ms
  (the mosaic is laid on the UI thread).
- One `trace` tile mid-zoom: 7–9 → 3.4–4.8 ms at 1, 11–14 → 3.7 ms at 2.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–g) | built; the user's run of 31g pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user's run** at a5ad88c or later: `dev/bench/run.sh --focus`
   (Linux), or on Windows `cargo build --release -p rustd3-bench` then
   `.\target\release\rustd3-bench.exe suite --focus` (PowerShell). It
   reads whether the strips now let `trace` and `label-churn` zooms finish
   a generation mid-burst on 8 workers, the weak profiles' pans, and the
   mosaic on tiny-skia. The Windows path (`start /AFFINITY`) has never
   run: if a weak profile fails there, its `run-weak-*.log` says why.
2. The architect's Linux diagnostic (CPU or display server in a tiny-skia
   gesture frame): the harness composes a pan frame of 1.6 Mpx in about
   20 ms here, so most of the user's 117–133 ms frames are outside the
   composition, or in its present.
3. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

- On tiny-skia every image costs a frame-sized clip mask and a
  high-precision pass (about 10 ns a px here); a pan at factor 2 still
  composes in about 120 ms here. Composing pans as mosaics saves 10–25 %
  at 2 only; a cheaper draw needs iced (a blit without a clip mask for an
  image inside its clip, or a low-precision nearest path).
- `drag-resize` (report-only) reads UI frame p99 higher on wgpu with one
  thread (25.6 → 41.0 ms) and lower elsewhere; two runs, noisy.
- `suite` measures the refresh rate from paced hover frames; a display
  that paces nothing at idle (Xvfb) needs `--refresh-hz`.

## 7. Key files

- `iced/src/plot/state.rs` (`render`: the drag/wheel split, `waiting`'s
  order, `draw`'s mosaic), `iced/src/plot/budget.rs` (`Kind`, the fits),
  `iced/src/plot/mosaic.rs`, `iced/src/plot/area.rs` (`draw_mosaic`),
  `iced/src/plot/slot.rs` (`Band`'s opacity, `lay_at`, `lay_mapped`).
- `figure/src/raster/cull.rs`, `figure/src/raster/vello.rs` (the stroke
  arm), `figure/src/raster.rs` (the cut test).
- `dev/bench/src/suite.rs`, `dev/bench/src/also/compose.rs`,
  `dev/bench/src/also.rs` (`tile_geometry`), `dev/bench/run.sh`,
  `dev/bench/README.md` (the run sheet per platform).

## 8. Commands

```bash
scripts/check.sh quick | full | release
xvfb-run -a -s "-screen 0 1920x1080x24" target/release/rustd3-bench run \
  --plan default --factor 1 --scenario pan-x,zoom-lines,drag-resize \
  --workload trace,scatter100k,label-churn --refresh-hz 60 --native-factor 1 \
  --runs 2 --threads 1 [--backend tiny-skia] --out /tmp/…   # diagnostics only
target/release/rustd3-bench also          # the also rows, the compose and tile ones included
dev/bench/run.sh [--smoke-only | --focus] # the user's machine (Linux)
rustd3-bench suite [--smoke-only | --focus] [--refresh-hz HZ]   # any platform
```

## 9. End-of-session checklist

- `scripts/check.sh quick` before every commit.
- `scripts/check.sh full` at 15762cb: green in 19 min 51 s (iced,
  examples and bench tests on tiny-skia and wgpu, the benchmark smoke with
  the new `also` rows). The first run, at a5ad88c, failed in
  `rustd3-figure --no-default-features` (the cut test; fixed by 15762cb).
- Disk: free space fell below 6 GB three times; `target/debug/incremental`
  was removed each time, never while cargo ran (2.4 → 7.1 GB; 5.3 → 5.6
  after deleting this session's own diagnostic binaries → 7.1; after the
  full check 3.1 GB → 7.0). The next session's warm-up rebuilds it.
- Decisions: none open; no AI attribution in commits.
