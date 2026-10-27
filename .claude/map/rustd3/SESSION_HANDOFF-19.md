# rustd3 — session handoff 19

2026-10-27 · **Phase 18 — Plot I: display** half done: steps 1–3 committed (without the translucent test); stopped at **D18.1** (open) · next: the architect's answer to D18.1, then steps 3 (translucent test), 4 and 5

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 18** (two code commits):

- 06b1c00 `feat(iced): Plot shows a figure's cells` — steps 1 and 2:
  `iced/src/plot.rs` (`Plot`, `new`, `width`, `height`,
  `scale_factor`, the `Widget` impl, `BAND_BYTES`, `NUDGE`,
  `From<Plot> for Element` via `_boxed`), `plot/state.rs` (`State`,
  `refresh` returning the cells rendered, `draw`, `is_over_area`),
  `plot/cell.rs` (`Cell`, `Band`, `Cell::{render, shows, draw, bands,
  bounds}`), `plot/handle.rs` (`from_image`), `plot/scale_factor.rs`
  (`resolve`), `plot/fixture.rs` (test figures). Unit tests: the
  scale-factor table, band and placement spot rows,
  `cells_partition_the_canvas`, `unchanged_figure_renders_no_cell`,
  `cell_images_have_their_device_size`, `bands_match_the_whole_cell`,
  `bands_stay_under_the_limit`, the handle moved without a copy.
- 4f471b3 `test(iced): Plot against the PNG of its figure` —
  `iced/tests/main/reference.rs` (`png_of`), `iced/tests/main/plot.rs`
  (`plot_matches_the_png_of_its_figure`,
  `plot_places_cells_on_device_pixels_at_fractional_scales`): max 1/255
  on tiny-skia and on wgpu (lavapipe) at 1.25, 1.5 and 2.

Not done: `translucent_background_matches_the_png` (step 3), the overlay
(step 4: `plot/overlay.rs`, `Plot::{crosshair, highlight}`, hover in
`update`, the hit index and `hover_renders_no_cell`,
`crosshair_stays_inside_the_hovered_panel`,
`highlight_marks_the_nearest_point`), docs with the `no_run` `view`
example (step 5), `check.sh full` at the phase's end, and the overlay
screenshots the architect asked for.

`scripts/check.sh quick` green at 4f471b3. Harness: unchanged, 9 suites,
3,314 assertions, 2,623 passing, 691 skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-18.md`), all open:

- **D18.1** — no public way to read or set a theme's ink and background:
  the overlay's color ("the theme's ink at 40%"), the crosshair test's
  red ink "through the theme" and the translucent-background test need
  figure API that "no change to `figure/`" forbids and no phase defines.
  Recommended A: `Theme::ink(self, Rgb)`, `Theme::background(self, Rgb)`
  builders and `Figure::get_theme`, `Theme::{get_ink, get_background}`
  readers (replacing the crate-private `ink()`, `background()`).
- **D18.2** — nine details as built (A recommended).
- **D18.3** — vello_cpu's dashes on dense self-overlapping strokes move
  with the render's origin (rows repeat every 4, columns at 256), so the
  widget shows the trace's dashes elsewhere than the PNG (237/255 on
  561 px at 2×); the section's figure is unaffected. Recommended A:
  record, Phase 22 reports, Phase 31 measures on the trace.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–17 done; 18 half done |
| M3–M5 | 23–40 | pending |

## 5. What to do next

1. Read D18.1's answer. With A: one commit `feat(figure): theme ink and
   background` (builders leave the theme unchanged on a non-finite
   channel; readers; call sites renamed; a theme test).
2. Step 3's `translucent_background_matches_the_png`: the same figure
   with `Theme::default().background(…50% alpha…)`, max ≤ 4/255 on both
   backends.
3. Step 4: `plot/overlay.rs` (`Overlay { cursor, panel, point }` in
   device px from the widget's device origin); `State` keeps
   `hit::Index` from `fig.hits(&resolved, &mut cache)` beside the plot
   areas (rebuilt only when cells re-render); `hover(cursor, …) -> bool`
   on `CursorMoved`/`CursorLeft`, `request_redraw` only when it changed;
   the nearest hit within 12 logical px, accepted only in the hovered
   panel; `draw` under `with_layer(plot-area bounds)` after the cells
   (tiny-skia draws a layer's quads before its images, so the overlay
   needs its own layer): two 1-device-px quads at 40% ink, a ring quad
   (`border::rounded(4)`, 1.5 px border). Tests: overlay pixels found by
   difference with the unhovered frame, 0 outside panel 0's plot area on
   both backends; ring centre within 1 device px; `hover_renders_no_cell`.
4. Step 5: module docs with a `no_run` `view` returning
   `Plot::new(figure(&self.data)).into()`; then `scripts/check.sh full`
   in the background; screenshots with the crosshair and highlight at
   1, 1.25 and 2 (scratch tests live in the session scratchpad, not the
   repository).

## 6. Notes for later phases

- Phase 19: `State::refresh` resolves twice (D18.2 (3)); the render
  settings are `Settings::parallel()` (SIMD `Detect`, up to 1/255 from
  the reference's `Baseline`); store `now` with the frame budget.
- Phases 19–21: cells of translucent overlapping dots differ from the
  whole render by up to 2/255; dense thinned traces by much more where
  vello_cpu's dashes move (D18.3). Bounds against the PNG hold for the
  section's opaque figure.
- Phase 22: report the trace at 2× (D18.3) and the 1 px line (D15.3).
- Phase 31: whole-pixel shifts are not render-invariant on the trace
  (D18.3); measure there before translating tiles.
- Phase 36: the brush overlay reads the theme's ink as D18.1 settles.

## 7. Key files

`iced/src/plot.rs`, `iced/src/plot/{state,cell,handle,scale_factor,
fixture}.rs`, `iced/tests/main/{plot,reference,headless,placement}.rs`,
`figure/src/theme.rs`, `figure/src/figure.rs`,
`figure/src/figure/{draw,resolve,resolved}.rs`, `figure/src/hit.rs`,
`figure/src/list/parts.rs`, `figure/src/raster.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
ICED_TEST_BACKEND=tiny-skia TZ=Asia/Kathmandu cargo test -p rustd3-iced --features figure -- --nocapture
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -p rustd3-iced --features figure -- --nocapture
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit.
- D18.1, D18.2, D18.3 open with their recommendations.
- Conventional commits, no attribution.
