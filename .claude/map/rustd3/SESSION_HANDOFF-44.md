# rustd3 — session handoff 44

2026-10-28 · **Phase 31 — Plot V: tiles and pan translation**: session 31b **done** (Steps 3–5), stopped at the section's second cut point · next: **Phase 31, Step 6** (iced: dependency classification, style hash, merging), with D31.4–D31.6 open

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- d12886b `feat(figure)` — Step 3: `thin::Grid`, `Held::grids` /
  `grids_for` / `overscan`, lines thinned on a held grid with a leading
  column edge (`thin::LEAD`), the dash phase anchored on it, marks drawing
  the overscanned view; `anchored_columns_repeat_after_a_whole_pixel_pan`,
  `dash_phase_is_zero_at_the_anchor`, `overscan_extends_the_view`.
- b2d0cdd `feat(iced)` — Step 4: tiles of 256 device px per plot area
  (`tile.rs`, `area.rs`), previews per area, `probe::Target::Tile`;
  `tiles_match_the_whole_area`, `tiles_cover_the_area_once`, Phase 20's
  preview tests on tiles.
- 6b9af5e `fix(figure)` — dashes anchor on the overscan's grid when no grid
  is held, the anchor column leads like thinning's, phase 0 a millionth of
  a px past the anchor (Step 5's tests found all three).
- ddfd934 `feat(iced)` — Step 5: snapped drags (`Gesture::exact` at the
  release), grids that translate by whole pixels, the overscan ring, the
  list split into plot-area layers and chrome (`layer.rs`) with overlays,
  tiles exact inside the area render with the ring; the five Step 5 tests
  plus the trace replica's two (D18.3, D22.1).
- ded81af `feat(bench)` — D22.4's item 3 in `rustd3-bench also`.
- The closing docs commit: `decisions/phase-31.md` (31b measurements,
  D31.4–D31.6), the index, this handoff.

Exit criteria (the section's numbering): 2 —
`pan_renders_only_entering_tiles` on the spot checks' 900 × 600 area, 1/255;
3 — `thinned_lines_match_a_fresh_render_after_a_pan`,
`dashed_lines_keep_their_phase_across_a_pan`,
`drag_settles_on_the_exact_domain` at 1/255; 4 (first half) —
`zoom_out_preview_uses_the_overscan_ring` missing 0 on every frame. All
iced tests pass on tiny-skia and wgpu (`ICED_TEST_BACKEND=…
TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features`). Criteria 4
(second half) to 6 belong to Steps 7–10.

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 179
floors; nothing blessed.

Decisions, all open with a recommendation: **D31.4** (A) thinned lines
at a plot area's left and right edges differ from the PNG by up to 32/255
because the widget's list is overscanned; the iced references compose
plot areas from the overscanned list. **D31.5** (A) eleven details of 31b
as built. **D31.6** (B) the leading ring column re-renders every frame of a
continuous pan as the section specifies; rendering ring tiles over their
whole square (overscan of two tiles) makes it one column per 256 px.
Decide D31.6 before Step 7.

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
| Conditional | 31 | in progress: 31a (pre-steps, Steps 1–2, D22.4 (4)) and 31b (Steps 3–5) done; 31c is Steps 6–10 |
| M4 Complete figure | 32–39 | done |
| M5 Release | 40 | after Phase 31 |

## 5. What to do next

Phase 31, **Step 6** (rustd3.md, `### Phase 31 —`, Steps and "Invalidation
by dependency and merging"): `style.rs` (the position-free style hash),
dependency classification per slot, greedy merging of gutter cells,
`state.rs`; tests `unchanged_panels_are_kept`,
`style_change_during_a_pan_re_renders`, `gutters_merge_into_fewer_images`.
Then Steps 7–10. Read D31.4–D31.6 and the 31b measurements first.

## 6. Notes for later phases

- **Gutters during a pan.** Every pan frame re-renders 4–10 gutter cells
  and overlays that show the panned handle (axis strips; the axis lines
  over the plot areas' bottom edges), about 4 ms each here; `Render::Auto`
  sends them to the worker together, so the benchmark's preview share
  stays at 98 % headless although the plot areas are current on 86–88 %
  of frames at 1. Step 6's merging and D22.4's item 1 (the cost model's
  per-render overhead) are the levers; Step 7's order (visible tiles,
  gutters, ring) should keep them ahead of the ring.
- **Two-point classification (Step 6).** The widget already detects a
  whole-pixel translation per area from the views (`Area::translation`)
  and checks layer hashes; the style hash slots in where `Area::change`
  compares the layer's hash after a translation.
- **Overlays.** The chrome over a plot area is drawn as transparent
  overlays on disjoint rects; a theme with grid lines draws them in the
  layer (they move), and its overlays are the chrome's reach only.
- **D31.2 (decided B: the 64-stamp splat)** is built in figure as a
  default-off `Renderer::splat` switch, which the widget turns on for
  screen renders of markers commands with at least 10,000 points; see the
  decision for where it goes.
- **tiny-skia compositing.** iced's tiny-skia engine clears a frame-sized
  clip mask per raster image (`tiny_skia/src/engine.rs:776`): tiles cost
  0.11 ms an image at 1600 × 1000; fewer, larger images help.
- The `iced/tests/main/tiles.rs` trace replica (400,000 samples × 3) is
  the iced tests' stand-in for the bench's `trace`; the D22.1 test drains
  the worker after each frame (300 ms of silence ends a drain).

## 7. Key files

`iced/src/plot/{tile,area,layer,state,slot,gesture,job,region}.rs`,
`iced/tests/main/{tiles,reference}.rs`, `figure/src/thin.rs`,
`figure/src/thin/grid.rs`, `figure/src/figure/held.rs`,
`figure/src/mark.rs`, `figure/src/mark/line.rs`, `dev/bench/src/also.rs`,
`.claude/decisions/phase-31.md`.

## 8. Commands

```bash
scripts/check.sh quick                     # every commit (runs the ratchet)
scripts/check.sh full                      # end of a session (background)
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -q -p rustd3-iced --all-features --test main -- tiles::
cargo run --release -p rustd3-bench -- also    # D22.4 (3) and (4) among the items
target/release/rustd3-bench run --headless --plan quick --scenario pan-x --workload trace --factor 1,2 --runs 1 --backend tiny-skia --out target/bench-p31b-tiny-skia   # the real-time share
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` at ded81af:
  green in 16 min 36 s (warm, the benchmark smoke included), no
  warnings.
- Disk: 7.0 GB free after the session (the headless pan runs
  under `target/bench-p31b-*` deleted).
- D31.4–D31.6 open with their recommendations; D31.2 and D31.3 decided.
- Conventional commits, no attribution.
