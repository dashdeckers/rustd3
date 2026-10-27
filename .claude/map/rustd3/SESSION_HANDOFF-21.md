# rustd3 — session handoff 21

2026-10-27 · **Phase 19 — Plot II: render pipeline** complete, with D19.1 and D19.2 open (both recommended as built) · next: **Phase 20 — Plot III: gestures**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 19** (all in rustd3-iced):

- aebb7b8 `feat(iced): slots, hash skipping and the probe` — step 1:
  `plot/slot.rs` (Phase 18's `cell.rs` folded in), `plot/job.rs`,
  `plot/probe.rs`, `Plot::probe`; refreshes plan only cells whose hash
  changed; generations; `Frame` samples per `RedrawRequested`.
- 685d6bd `feat(iced): render changed cells on a worker thread` — step 3:
  `plot/worker.rs`, `plot/render.rs` (`Inline`, `Worker`), `Plot::render`,
  `Event::Waken` drains; re-planning of abandoned jobs; harness
  `wait_for_wake`.
- 5ee36c6 `test(iced): newest result wins and previews stay` — step 4;
  harness `rebuild`.
- a326c94 `feat(iced): frame budget and cost model decide inline or
  worker` — step 2: `plot/budget.rs`, `plot::cost`, `Render::Auto` the
  default; Phase 18's reference tests render inline explicitly.
- b82e188 `docs(iced): the render pipeline and the probe` — step 5.
- 3b21383 `test(iced): pipeline helpers only where the pipeline tests
  build` — no warnings in `check.sh full`'s feature sets.

Exit criteria (commands: `ICED_TEST_BACKEND=tiny-skia|wgpu
TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture`):
(1) `worker_mode_matches_inline_mode` 0/255 on both backends (2k dots,
D19.1). (2) `cheap_refresh_renders_inline` (model 2.0 ms) and
`expensive_refresh_uses_the_worker` (20.6 ms) pass. (3)
`unchanged_cells_are_skipped` and
`rebuild_of_an_equal_figure_renders_nothing` pass (`cargo test -p
rustd3-iced --all-features --lib`). (4) `newest_result_wins` and
`previous_image_stays_until_the_new_one_arrives` pass on both. (5)
`probe_reports_every_stage` passes, inline and worker. (6)
`worker_exits_on_drop` passes; `scripts/check.sh quick` on every
commit; `full` green at 3b21383 in 4 min 19 s (warm), no warnings.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-19.md`): D19.1 (open, A: 2k dots in
the harness tests, since a debug build renders 200k dots in about a
minute per cell), D19.2 (open, A: sixteen details as built), D19.3
(decided, the section's answer: the cost model stays the no-probe
fallback). Public API: `plot::{Render, Probe, cost}`,
`plot::probe::{Sample, Stage, Target}`, `Plot::{render, probe}`.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–19 done; 20 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

The architect answers D19.1 and D19.2; neither changes an API Phase 20
builds on (B for D19.1 changes only test parameters; B/C variants of
D19.2 are local). Then Phase 20: read `rustd3.md`'s front matter and
"### Phase 20 —". The refresh is `State::redraw` → `refresh` in
`iced/src/plot/state.rs`: one `begin`, `resolve`, `draw_resolved`,
`list::cells`, `plan` (generation), then inline jobs or
`worker.replace` (every pending slot re-planned, plot areas first).

## 6. Notes for later phases

- Phase 20: applied input bumps the generation ("Phase 20 adds applied
  input"); previews of plot-area cells map `Slot`'s shown bands by one
  affine per axis in `draw` (the slot keeps the generation and hash it
  shows; it would also need the scales it was rendered with). Gestures
  add messages and `capture_event`; the overlay still re-hovers after
  every refresh whose cells changed (areas and hits update at planning,
  not when images land).
- Phase 20/21: tests that start a worker in `iced/tests/main` hold
  `pipeline::workers()`'s lock, or `worker_exits_on_drop`'s count can
  see their threads; Phase 18's reference tests pass
  `.render(Render::Inline)`.
- Phase 21: `Plot::revision` and `Caching::Off` plug into `State::plan`.
- Phase 22: run the 200k-dot case in release and record the
  calibration factor (D19.3); report the trace at 2× (D18.3) and the
  1 px line (D15.3). In a debug build vello_cpu renders about 1,500–2,100
  px/ms, so `Auto` without a probe predicts inline renders of hundreds
  of milliseconds there.
- Phase 31: merging, tiles and a multi-thread pool sit on `Worker` and
  `Job`; vello_cpu's pool threads take the worker's name `rustd3-plot`.

## 7. Key files

`iced/src/plot.rs`, `iced/src/plot/{state,slot,job,worker,budget,cost,
render,probe,overlay,handle,scale_factor,fixture}.rs`,
`iced/tests/main/{pipeline,plot,headless,reference}.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
ICED_TEST_BACKEND=tiny-skia TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D19.1 and D19.2 open with recommendations; D19.3 decided.
- Conventional commits, no attribution.
