# rustd3 — session handoff 45

2026-10-28 · **Phase 31 — Plot V: tiles and pan translation**: session 31c **done** (D31.6 and Steps 6–8) · next: **Phase 31, Step 9** (bench: `--threads`, `--caching verify`, the rerun's setup), then Step 10 (docs); D31.7–D31.9 open

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- e39be1b `feat(iced)` — D31.6 (decided B): ring tiles render their whole
  square, the layers' overscan is two tiles and the margin
  (`tile::OVERSCAN`, 520 device px); a steady pan renders a column per
  256 px. Spot checks: +37 px renders column −2's 5 tiles whole, −37 px no
  tile. The numbers are under D31.6 ("Built").
- e7b8ed0 `feat(iced)` — Step 6: `style.rs` (the position-free style
  hash; a translation keeps tiles only while it is equal), `merge.rs`
  (gutter cells of a row band with the same handles merge into runs under
  `BAND_BYTES`; a run reports its first cell), the state keeping what it
  drew from (`Drawn`) and building jobs from it. Tests
  `unchanged_panels_are_kept`, `style_change_during_a_pan_re_renders`,
  `gutters_merge_into_fewer_images` (`iced/tests/main/classify.rs`).
- ffdeb6d `feat(figure)` — D31.2 (decided B): `raster::Renderer::splat`,
  eighth-pixel stamps (`figure/src/raster/splat.rs`), the replay split
  around splatted markers with `TargetInit::SrcOver`; agreement tests
  `figure/tests/main/raster/splat.rs` (23–28/255 against filled markers).
- 550a067 `feat(iced)` — Step 7: the pool (`worker.rs`'s `Pool`,
  `queue.rs`), `Plot::threads` (1..=64; default available parallelism − 1,
  1 to 8), jobs ordered by distance from the last gesture input, the
  per-job inline decision, D22.4 (1)'s fitted cost model (`budget.rs`),
  D22.4 (2), the widget's splat switch (layers with ≥ 10,000 markers).
  Tests `threads_sets_the_pool_size`, `tiles_near_the_cursor_render_first`
  (1 and 2 threads), `newest_result_wins_per_tile`,
  `large_marker_sets_splat_on_screen`.
- b3994cb `feat(iced)` — Step 8: `Caching::Verify`,
  `probe::Sample::Mismatch`; tests `verify_replaces_a_differing_image`,
  `verify_finds_translated_tiles_exact`.
- `perf(figure)`: the splat's fast path (6.2× at 1, 3.4× at 2 on
  `scatter100k`, within 30/255); `feat(bench)`: `also`'s rows for the
  pan's renders per frame, one gutter render's costs and the splat.
- The closing docs commit: `decisions/phase-31.md` (31c measurements,
  D31.7–D31.9), the index, this handoff.

Exit criteria (the section's numbering): 2 and 3 hold as in 31b with the
new spot checks; 4 — `zoom_out_preview_uses_the_overscan_ring` missing 0,
`tiles_near_the_cursor_render_first`, `newest_result_wins_per_tile` pass;
5 — `unchanged_panels_are_kept`, `style_change_during_a_pan_re_renders`,
`gutters_merge_into_fewer_images`, `verify_replaces_a_differing_image`,
`threads_sets_the_pool_size` pass, and every iced test passes on
tiny-skia and wgpu (`ICED_TEST_BACKEND=… TZ=Asia/Kathmandu cargo test -p
rustd3-iced --all-features`). Criterion 6 (the user's rerun) belongs to
Step 9.

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 179
floors; nothing blessed.

Decisions, all open with a recommendation: **D31.7** (B) the section's
two-point test of a handle change needs `Scale::remap` in rustd3-figure,
a public method it does not name (until then a `Fit::Visible` y refit
during an x pan is not seen until the drag ends); **D31.8** (B) D22.4 (1)
asks for calibration without a probe, which needs the widget to read the
monotonic clock against Conventions item 5 (until then only probe-timed
renders calibrate); **D31.9** (A) thirteen details of 31c as built.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; D22.1 decided C |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | in progress: 31a, 31b, 31c (D31.6, Steps 6–8) done; 31d is Steps 9–10 |
| M4 Complete figure | 32–39 | done |
| M5 Release | 40 | after Phase 31 |

## 5. What to do next

Phase 31, **Step 9** (rustd3.md, `### Phase 31 —`, Steps): `--threads N`
and `--caching verify` in `dev/bench/src/{cli,app}.rs` (the driver
records `Sample::Mismatch`, which it ignores today; `cli::caching_name`
already names `verify`), D22.3's factor cells at a fixed device size,
D22.4's `weak` plan and `dev/bench/run.sh`, the headless smoke, the
user's rerun; then Step 10 (docs, `check.sh full`). Read D31.7–D31.9
first; if D31.8 is decided B, the clock reading goes in `Job::run`.

## 6. Notes for later phases

- **Gutters during a pan are inline now.** Per frame of `trace`'s pan
  about 6 gutter runs and overlays change; the fitted model renders them
  inline (2–4 ms a frame here), the pool is nearly idle. One strip render
  is 0.4 ms at 1 and 1.1 ms at 2, its fixed cost the largest part at 2.
- **At factor 2 headless**, frames of 83–167 ms let the pan move 100–200
  device px between frames, so the ring falls behind and 32–36 % of
  frames preview; the windowed rerun (frames of 16.7 ms) decides.
- **The splat** is 6.2× at 1 and 3.4× at 2 on `scatter100k`'s plot area;
  the passes around the markers and the stamps' blend are what remain.
- `rustd3-bench raster` still measures its own splat module (16 or 64
  stamps); the rerun can point at `Renderer::splat` through `also`.
- The trace replica in `iced/tests/main/tiles.rs` and the `classify.rs`
  figures are the iced tests' stand-ins for the bench's workloads.

## 7. Key files

`iced/src/plot/{style,merge,queue,worker,budget,state,area,slot,job}.rs`,
`iced/tests/main/{classify,pipeline,tiles,plot}.rs`,
`figure/src/raster/{splat,vello}.rs`, `figure/tests/main/raster/splat.rs`,
`dev/bench/src/also.rs`, `.claude/decisions/phase-31.md`.

## 8. Commands

```bash
scripts/check.sh quick                     # every commit (runs the ratchet)
scripts/check.sh full                      # end of a session (background)
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -q -p rustd3-iced --all-features
cargo build --release -p rustd3-bench && target/release/rustd3-bench also   # the pan's renders, gutter costs, splat
target/release/rustd3-bench run --headless --plan quick --scenario pan-x --workload trace --factor 1,2 --runs 1 --backend tiny-skia --out target/bench-p31c-tiny-skia
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` at the closing
  commit: see the session's report.
- Disk: the headless pan runs under `target/bench-p31c-*` deleted.
- D31.7–D31.9 open with their recommendations; D31.2 and D31.6 built.
- Conventional commits, no attribution.
