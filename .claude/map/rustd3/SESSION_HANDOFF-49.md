# rustd3 — session handoff 49

2026-10-29 · **Phase 31e is done** (D22.5's zoom path and software compositing, D40.4 (1), the `drag-resize` scenario and the focus plan) · open: D22.6 (D22.5's items 4 and 5, not built as written; recommendations to answer) · next: the user runs `dev/bench/run.sh --focus`

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31e, one commit per item, each with a test that fails without it:

- e0e4e19 `fix(figure)` D40.4 (1): of two time labels a month (day, clock
  change) boundary crowds, the finer one gives way, so "2026-11" and
  "November" stay (`collide::Restart`; tests
  `of_a_close_pair_the_label_on_the_finer_rung_gives_way`,
  `a_fitted_fortnight_keeps_every_label_that_fits`). No snapshot changed.
- b78c690 `feat(iced)` D22.5 (1): during a gesture only the tiles that
  show render; the ring waits for its end
  (`a_zoom_renders_the_tiles_that_show_and_the_ring_once_it_ends`: 4
  tiles at the step, the 12 of the ring after it).
- eee8b7d `feat(iced)` (2): a tile in flight through a zoom step lands in
  the preview layer; the queue holds the newest generation alone
  (`a_tile_in_flight_through_a_burst_of_zoom_steps_lands_in_the_preview`,
  `previews_that_newer_ones_cover_go`).
- d8ec675 `feat(iced)` (3): no tile renders inline while a gesture lasts
  under `Render::Auto` (`no_tile_renders_inline_while_a_gesture_lasts`).
- 01f47a6 `perf(iced)` (6): one damaged layer a gesture frame on
  tiny-skia; no preview outside the plot area
  (`a_gesture_draws_in_one_layer_that_changes_and_no_preview_outside_the_area`,
  with `fixture::Recorder`). iced's per-image clip mask cannot be skipped
  from the widget (D22.6 (3)).
- 84a775c `perf(iced)` (7): the hit index waits for a gesture to end and
  for a widget that hovers
  (`the_hit_index_waits_for_a_gesture_to_end_and_hover_stays_right`).
- 02022f8 `feat(bench)` `drag-resize`, report-only, its "Window drags"
  section and `summary.csv` rows; `resize` labelled as jumps between
  fixed sizes (`drag_resize_steps_every_16_ms_down_to_70_percent_and_back`,
  `a_drags_last_step_is_the_input_a_pause_follows`).
- 24a1e6d `feat(bench)` the focus plan (13 cells, 65 runs, report-only
  against the desktop budgets) and `run.sh --focus`
  (`focus_plan_has_d22_5s_cells`).
- a9eb8d5 `docs(iced)`: `Plot`, `Render` and `Caching` say how gestures
  render (visible tiles, the waiting ring, coalescing, no inline tiles).
- 6772c36, 67045a8 `docs`: D22.6.
- 589feb0 `test(iced)`: `newest_result_wins` counts the renders a quiet
  pool still sends. It failed once in the first full check (15 renders
  of generation 2 against 16 slots): an overlay, whose content a new
  background leaves as it was, showed current content from its
  generation-1 render before its requeued render arrived. A race in the
  test since Phase 31, not in the widget.

Not built: D22.5 (4) (measured worse as written) and (5) (needs a field
on `probe::Sample::Frame`), both in D22.6 with options; so the focus
report has no any-resolution latency column yet.

- Harness unchanged: 6,722 assertions, 5,292 passing, 179 floors.
- `scripts/check.sh full` at 67045a8: see §9.
- Tags: `m2-complete`, `m3-complete`, `m4-complete`.

## 3. Measured here (not rule outcomes)

Windowed under Xvfb on this container (2 cores, Mesa lavapipe), `--plan
weak` cells and `pan-x`/`zoom-lines` × `scatter1m`, `--native-factor 1
--refresh-hz 60`, 2 runs each, before (ec8dec2, Phase 31e's base) →
after (a9eb8d5). Latency is the registered one (full resolution); the
any-resolution latency waits on D22.6.

| Backend | Cell | Frame p50 ms | Frame p99 ms | Latency p50 / p95 ms | Inline share | CPU s/s |
| --- | --- | --- | --- | --- | --- | --- |
| wgpu | `pan-x` × `trace` | 19.6 → 17.8 | 42 → 39 | 12 / 49 → 11 / 82 | 0.95 → 0.92 | 1.38 → 1.39 |
| wgpu | `zoom-lines` × `trace` | 12.7 → 10.8 | 84 → 123 | 673 / 1,122 → 709 / 1,190 | 0.12 → 0.13 | 1.77 → 1.78 |
| wgpu | `pan-x` × `scatter100k` | 23.0 → 16.7 | 45 → 51 | 13 / 30 → 15 / 151 | 0.87 → 0.74 | 1.32 → 1.20 |
| wgpu | `zoom-lines` × `scatter100k` | 11.3 → 10.4 | 107 → 95 | 656 / 1,108 → 638 / 1,089 | 0.08 → 0.13 | 1.72 → 1.74 |
| wgpu | `pan-x` × `scatter1m` | 22.3 → 50.9 | 244 → 113 | 7,014 / 12,410 → 6,996 / 12,393 | 0.17 → 0.26 | 1.92 → 1.90 |
| wgpu | `zoom-lines` × `scatter1m` | 16.2 → 18.4 | 191 → 139 | 7,285 / 12,385 → 7,313 / 12,413 | 0.18 → 0.23 | 1.88 → 1.88 |
| tiny-skia | `pan-x` × `trace` | 1.0 → 1.0 | 159 → 35 | 75 / 151 → 15 / 56 | 0.85 → 0.93 | 1.02 → 1.03 |
| tiny-skia | `zoom-lines` × `trace` | 608 → 1.0 | 1,830 → 84 | 7,402 / 13,002 → 692 / 1,162 | 0.11 → 0.12 | 1.15 → 1.55 |
| tiny-skia | `pan-x` × `scatter100k` | 1.0 → 0.9 | 164 → 29 | 78 / 157 → 13 / 53 | 0.67 → 0.83 | 1.01 → 1.03 |
| tiny-skia | `zoom-lines` × `scatter100k` | 1.0 → 0.9 | 1,004 → 94 | 5,522 / 11,122 → 640 / 1,082 | 0.08 → 0.13 | 1.13 → 1.47 |
| tiny-skia | `pan-x` × `scatter1m` | 0.9 → 1.0 | 288 → 82 | 219 / 3,027 → 6,877 / 12,274 | 0.39 → 0.35 | 1.29 → 1.91 |
| tiny-skia | `zoom-lines` × `scatter1m` | 0.7 → 0.8 | 129 → 114 | 7,292 / 12,392 → 7,127 / 12,227 | 0.18 → 0.20 | 1.83 → 1.88 |

Readings: tiny-skia frames 5–20× faster (item 6), so its zooms now
animate and its latency falls to wgpu's; `scatter1m`'s frames halve
(item 7). Two-run wgpu pan latencies are noisy: with 3 runs (D22.6
(4d)) `scatter100k` reads 30 → 46 ms p95, previews 2 → 6 %, dropped
frames 230 → 51. tiny-skia `scatter1m` pans now never catch up, as wgpu
did before: faster frames, more generations for an expensive tile. wgpu
zoom frames on `trace` got slower on lavapipe (D22.6 (4e)). Zoom latency
p95 stays about 1.1 s: no generation completes during a burst at full
resolution, which is item 5's work. `drag-resize` (trace, 1 run): frame
p50/p99 9.3/53 ms, 155 drops, last step to current 198/232 ms.

Screenshots (`/tmp/claude-0/rustd3-bench31e/shots/`, a 2 s burst of 60
wheel lines on `trace`, both backends, factors 1 and 2, before and
after): before, wgpu showed the old tiles scaled and blurry with the
axes frozen at 0–100 until the burst ended, and tiny-skia did not move
at all for 3 s. After, at factor 1 both backends show sharp tiles a few
steps stale mid-burst and the axes move every few steps, though up to
about 20 steps behind the data (D22.6 (4b)); no seam or misplaced tile
at tile joins in the frames viewed full size. Unchanged by 31e: a nearly
blank plot for about 300 ms when the layout settles after the last step
(D22.6 (4a)); at factor 2 wgpu's axes stay frozen until the end, and
tiny-skia's burst frames take 9.5 s under Xvfb (7.8 and 17.6 s before;
D22.6 (4c)).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–e) | built; 31e's focus rerun pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user's focus run**, on the machine of the rerun (Ryzen 5800X,
   RTX 3090, COSMIC), from the repository root at this commit, on power,
   other GPU-heavy apps closed, mouse and keyboard untouched for about 40
   minutes:

   ```bash
   dev/bench/run.sh --focus
   tar czf bench.tgz -C bench-results <the stamp the script prints>
   ```

   (`HZ=59.96 FACTOR=1 dev/bench/run.sh --focus` if detection fails, the
   rerun's values.) It runs one probe run of the first focus cell, which
   corrects the factor, the focus plan (report-only, about 17 min), then
   `weak-nogpu`, `weak-4cores`, `weak-2cores`.
2. The architect answers D22.6: item 4 (recommended C, single-threaded
   renders everywhere), item 5 (recommended B, a `coarse` field on
   `Sample::Frame` with `#[non_exhaustive]`), and with them the blank
   settle frame (4a) and the axes' lag (4b), which a following session
   builds with item 5's seam tests and screenshots.
3. When the focus run returns: read 4d and 4e (item 3's price on pans
   and on wgpu zoom frames) on real hardware; append its numbers to
   D22.5 in `decisions/phase-22.md`.
4. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

The Later table in rustd3.md and CHANGELOG.md, handoff 47's and 48's
items, and D22.6's 4c (tiny-skia at factor 2) if the user's run shows it.

## 7. Key files

- `.claude/decisions/phase-22.md` (D22.5, D22.6).
- `iced/src/plot/state.rs` (`is_active`, `deferred`, `grown`, `unhit`,
  `render`, `draw`), `iced/src/plot/area.rs` (`land`, `layer`,
  `retire`), `iced/src/plot/fixture.rs` (`Recorder`),
  `iced/src/plot/state/gestures.rs` (the items' tests).
- `figure/src/axis/draw.rs` (`restarts`, `parent`),
  `figure/src/layout/collide.rs` (`Restart`).
- `dev/bench/src/plan.rs` (`Plan::Focus`), `dev/bench/src/report.rs`,
  `dev/bench/src/scenario.rs` (`DragResize`), `dev/bench/run.sh`,
  `dev/bench/README.md`.

## 8. Commands

```bash
scripts/check.sh quick | full | release
xvfb-run -a -s "-screen 0 1920x1080x24" target/release/rustd3-bench run \
  --plan weak --scenario zoom-lines --refresh-hz 60 --native-factor 1 --runs 2 \
  [--backend tiny-skia] --out /tmp/…      # diagnostics only, never in the repo
dev/bench/run.sh --focus                  # the user's machine (windowed)
```

## 9. End-of-session checklist

- `scripts/check.sh quick` before every commit.
- `scripts/check.sh full` at 67045a8: green in 19 min 33 s (`target/debug/incremental` removed
  once for disk; iced tests on tiny-skia and wgpu, the benchmark smoke). The first run, at
  a9eb8d5's tests, stopped after 4 min on `newest_result_wins` (589feb0).
- Decisions recorded (D22.6 open); no AI attribution in commits.
