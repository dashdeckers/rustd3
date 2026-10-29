# rustd3 — session handoff 50

2026-10-29 · **Phase 31f is done** (D22.6 as decided: item 4 C, 4a, 4c; 4b and item 5 measured worse as written and not committed) · open: D22.7 (4b and item 5; recommendations to answer) · next: the user's focus run, then the architect answers D22.7

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31f, on 24e4b98, one commit per item, each with a test that fails
without it:

- bdfa439 `perf(iced)` D22.6 item 4, C: every render the widget makes,
  inline or on its pool, is single-threaded; `Settings::parallel()` stays
  for the file outputs (`inline_renders_start_no_thread_whatever_the_pool_size`,
  pools of 1 and 4).
- 2e47bf1 `fix(iced)` 4a: when the layout settles, the old areas' imaged
  tiles and previews join the new areas' preview layers through
  `Affine1::device` (old area onto new), old gutter images stay until
  their runs render, and the alternating layer holds while either shows
  (`the_layout_settling_after_a_zoom_leaves_no_slot_blank`,
  `a_new_layout_maps_the_old_area_onto_the_new_one`).
- 7cba714 `fix(iced)` 4c: the layer's half-px toggle holds while idle, so
  the first draw of a gesture always differs from the idle one
  (`the_first_draw_of_a_gesture_damages_the_widget_whole`). Cause and
  numbers in D22.6's followups.
- 374f713 `docs`: D22.7, D22.6's followups.

Not committed (D22.7, open): 4b as written (1 ms bound on inline gutter
runs: pans 2.7× slower) and item 5 (half-resolution pass: every zoom
latency up 4–106 %, any-resolution no better). Patches that apply to
7cba714: `/tmp/claude-0/rustd3-bench31f/patches/0001-perf-iced-during-a-gesture-the-axes-render-ahead-of-.patch`
and `…/patches/item4-as-built.patch` (`/tmp` does not outlive the
container). So the focus report still has no any-resolution column, and
`probe::Sample` is unchanged.

- Harness unchanged.
- `scripts/check.sh full` at 374f713: see §9.

## 3. Measured here (not rule outcomes)

Windowed under Xvfb (2 cores, Mesa lavapipe), the 31e cells and the
zoom cells at `--threads 2`, `--native-factor 1 --refresh-hz 60`, 2 runs
each, 24e4b98 → 7cba714; outputs in `/tmp/claude-0/rustd3-bench31f/`
(`before/`, `final/`). This container's wgpu frames drifted 30–40 %
slower during the session (hover/`trace` frame p50 8.1 → 11.3 ms with no
change that touches it), so wgpu frame times below are not comparable
across rows' columns; the interleaved control (`ab/`) shows the weak zoom
cells unchanged or faster and `scatter1m` zooms slower (4a's price,
D22.6's followups).

| Backend | Cell | Frame p50 ms | Frame p99 ms | UI frame p99 ms | Latency p95 ms | Inline | Preview |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wgpu | `pan-x` × `trace` | 15.3 → 20.9 | 40 → 40 | 26.6 → 13.9 | 78 → 80 | 0.93 → 0.93 | 0.08 → 0.09 |
| wgpu | `zoom-lines` × `trace` | 8.8 → 15.9 | 116 → 122 | 38.1 → 21.1 | 1,198 → 1,235 | 0.13 → 0.23 | 0.44 → 0.59 |
| wgpu | `pan-x` × `scatter100k` | 12.6 → 17.8 | 27 → 38 | 13.8 → 10.8 | 38 → 60 | 0.88 → 0.86 | 0.05 → 0.07 |
| wgpu | `zoom-lines` × `scatter100k` | 9.4 → 16.5 | 96 → 99 | 37.3 → 20.3 | 1,091 → 1,092 | 0.12 → 0.30 | 0.48 → 0.59 |
| wgpu | `pan-x` × `scatter1m` | 54.0 → 67.6 | 99 → 111 | 76.9 → 83.8 | 12,298 → 12,588 | 0.40 → 0.70 | 1.00 → 1.00 |
| wgpu | `zoom-lines` × `scatter1m` | 17.2 → 43.6 | 148 → 376 | 125 → 323 | 12,399 → 12,642 | 0.21 → 0.62 | 1.00 → 1.00 |
| wgpu | `zoom-lines` × `trace`, 2 threads | 9.3 → 14.1 | 123 → 149 | 55.6 → 35.4 | 1,128 → 1,169 | 0.09 → 0.13 | 0.33 → 0.44 |
| wgpu | `zoom-lines` × `scatter100k`, 2 threads | 8.5 → 14.1 | 86 → 104 | 31.8 → 22.6 | 1,031 → 1,072 | 0.17 → 0.21 | 0.36 → 0.48 |
| tiny-skia | `pan-x` × `trace` | 0.9 → 1.0 | 35 → 38 | 11.5 → 11.8 | 59 → 67 | 0.93 → 0.93 | 0.07 → 0.07 |
| tiny-skia | `zoom-lines` × `trace` | 0.9 → 1.0 | 81 → 77 | 21.2 → 15.2 | 1,175 → 1,160 | 0.12 → 0.18 | 0.09 → 0.12 |
| tiny-skia | `pan-x` × `scatter100k` | 0.9 → 1.0 | 30 → 34 | 8.2 → 8.6 | 53 → 74 | 0.84 → 0.82 | 0.08 → 0.11 |
| tiny-skia | `zoom-lines` × `scatter100k` | 0.9 → 1.0 | 89 → 91 | 24.7 → 9.2 | 1,087 → 1,089 | 0.13 → 0.25 | 0.49 → 0.41 |
| tiny-skia | `pan-x` × `scatter1m` | 0.9 → 1.0 | 78 → 89 | 65.0 → 73.7 | 12,243 → 12,384 | 0.29 → 0.69 | 1.00 → 1.00 |
| tiny-skia | `zoom-lines` × `scatter1m` | 0.8 → 1.0 | 116 → 197 | 64.1 → 97.1 | 12,264 → 12,378 | 0.21 → 0.55 | 1.00 → 1.00 |
| tiny-skia | `zoom-lines` × `trace`, 2 threads | 0.9 → 1.0 | 82 → 114 | 17.8 → 19.1 | 1,133 → 1,168 | 0.11 → 0.11 | 0.09 → 0.12 |
| tiny-skia | `zoom-lines` × `scatter100k`, 2 threads | 0.9 → 1.0 | 82 → 95 | 19.5 → 13.3 | 1,058 → 1,083 | 0.16 → 0.20 | 0.41 → 0.47 |

Readings: item 4 nearly halves the UI frame p99 of zooms and lets more
gutters render inline; zoom latency p95 stays about 1.1 s (no
generation completes during a burst at full resolution; item 5's half
pass did not change that here). On `scatter1m`, frames that showed 18–21
blank tiles for most of a run now show carried previews (the preview
column counts frames with a missing or outdated slot, so it reads 1.00
either way). Pan latency
p95 on `scatter100k` reads higher on both backends; the two-run pan
cells were noisy in 31e too (D22.6 (4d)).

Screenshots and recordings (`/tmp/claude-0/rustd3-bench31f/shots/`):
the settle after a 60-line burst on `trace` at factor 1 on both
backends shows the old content mapped onto the moved area, then sharp;
no blank frame (31e: about 300 ms of background). Item 5's mid-burst
frames at factors 1 and 2 on both backends: no seam, no misplaced tile;
at 2 no frame caught a half tile. 4c: the factor-2 tiny-skia burst's
longest frame 10.7–11.2 s → 0.8–1.2 s.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–f) | built; the focus rerun pending; D22.7 open |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user's focus run** (handoff 49 §5 step 1), at 7cba714 or later;
   it reads 4d and 4e on real hardware, and 4a's price on `scatter1m`.
2. The architect answers D22.7: 4b (recommended B, the order alone:
   gutters and overlays ahead of tiles on the pool while a gesture lasts,
   inline as before) and item 5 (recommended B: no half pass; `Sample`'s
   variants `#[non_exhaustive]`).
3. A following session builds what D22.7 decides, from the patches if
   they still exist, else from D22.7's description.
4. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

The Later table in rustd3.md and CHANGELOG.md, handoff 47's and 48's
items; while previews show, a tiny-skia frame at factor 2 still costs
about 0.2 s here (iced filters each scaled image through a frame-sized
clip mask).

## 7. Key files

- `.claude/decisions/phase-22.md` (D22.6's followups, D22.7).
- `iced/src/plot/state.rs` (`plan`'s `!same` branch, `former`, `prune`,
  `report`, `draw`'s layer), `iced/src/plot/area.rs` (`inherit`,
  `maps_to`, `has_preview`, `shown`), `iced/src/plot/preview.rs`
  (`Affine1::device`), `iced/src/plot/slot.rs` (`fill`, `draw`),
  `iced/src/plot/state/gestures.rs` (the items' tests, `piped`).
- `/tmp/claude-0/rustd3-bench31f/`: `measure.sh`, `ab.sh`, `tab.py`,
  `axis.py`, `c4/` (gdb sampling, 4c), `patches/`.

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
- `scripts/check.sh full` at 374f713: green in 22 min 45 s (`target/debug/incremental`
  removed once for disk; iced tests on tiny-skia and wgpu, the benchmark smoke).
- Decisions recorded (D22.7 open); no AI attribution in commits.
