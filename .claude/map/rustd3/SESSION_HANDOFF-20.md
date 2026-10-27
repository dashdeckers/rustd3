# rustd3 — session handoff 20

2026-10-27 · **Phase 18 — Plot I: display** complete (resumed after D18.1–D18.3 were decided; handoff 19 records the stop) · next: **Phase 19 — Plot II: render pipeline**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 18**, before the stop (handoff 19):

- 06b1c00 `feat(iced): Plot shows a figure's cells` — steps 1 and 2.
- 4f471b3 `test(iced): Plot against the PNG of its figure`.

After the decisions (6eeb9a7):

- dfa0795 `feat(figure): theme ink and background` — D18.1 A:
  `Theme::{ink, background}` builders, `Figure::get_theme`,
  `Theme::{get_ink, get_background}` (replacing the crate-private
  `ink()`, `background()`, `Figure::style()`).
- f6f0dcf `feat(figure): Figure::draw_resolved` — D18.2 (3) B: the
  public emitter; `draw_with` and `draw_held` call it.
- f448849 `perf(iced): one resolve per refresh` — `begin`, `resolve`,
  `draw_resolved`; `a_refresh_resolves_the_figure_once`.
- 57d508b `test(iced): translucent background against the PNG`.
- a359879 `feat(iced): crosshair and highlight` — `plot/overlay.rs`
  (`Overlay`, `Shown`), `Plot::{crosshair, highlight}`, the hit index in
  `State`, hover on `CursorMoved`/`CursorLeft` (redraw only on change;
  re-hover after a refresh that renders cells), quads in a layer clipped
  to the hovered plot area.
- 38d2987 `docs(iced): Plot's module documentation` — the `no_run`
  `view` example.
- 3298386 `test(figure): no dead-code warning for scatter::SCALE without
  raster`.

Exit criteria: (1) `ICED_TEST_BACKEND=tiny-skia|wgpu cargo test -p
rustd3-iced --features figure` passes on both (lavapipe). (2)
`plot_matches_the_png_of_its_figure` max 1/255 on both;
`translucent_background_matches_the_png` 1/255 on both. (3)
`plot_places_cells_on_device_pixels_at_fractional_scales` 1/255 at 1.25
and 1.5. (4) `crosshair_stays_inside_the_hovered_panel`: 0 overlay px
outside, both backends, at 2 and 1.25. (5) The unit tests, with
`hover_renders_no_cell`. (6) `scripts/check.sh quick` on every commit;
`full` green at 3298386 in 3 min 53 s (warm), no warnings.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-18.md`): D18.1 A, D18.2 A with (3) B,
D18.3 A, all decided; measurements at the phase's end appended. Public
API beyond the section: figure's `Theme::{ink, background, get_ink,
get_background}`, `Figure::{get_theme, draw_resolved}` (all decided).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–18 done; 19 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 19: read `rustd3.md`'s front matter and "### Phase 19 —". The
widget's refresh is `State::refresh` in `iced/src/plot/state.rs`: one
`cache.begin()`, one `resolve`, one `draw_resolved`, `list::cells`, then
every cell rendered inline (`Cell::render`) when any cell's rect or hash
changed; hits and plot areas come from the same resolve.

## 6. Notes for later phases

- Phase 19: per-cell skipping replaces the all-or-nothing comparison in
  `State::refresh`; `now` is stored with the frame budget (D18.2 (4));
  the widget renders with `Settings::parallel()` (SIMD `Detect`, up to
  1/255 from the reference's `Baseline`).
- Phases 19–21: cells of translucent overlapping dots differ from the
  whole render by up to 2/255; dense thinned traces by much more where
  vello_cpu's dashes move with the origin (D18.3). Reference tests keep
  figures that do not self-overlap densely.
- Phase 20: the overlay re-hovers after every refresh that renders
  cells; gestures add their own messages and `capture_event`.
- Phase 22: report the trace at 2× (D18.3) and the 1 px line (D15.3).
- Phase 31: measure whole-pixel shifts on the trace first (D18.3).
- Phase 35: its builders sit beside `Theme::{ink, background}`.
- Phase 36: the brush overlay reads `get_theme().get_ink()` as the
  crosshair does, in `Overlay`'s layer.

## 7. Key files

`iced/src/plot.rs`, `iced/src/plot/{state,cell,overlay,handle,
scale_factor,fixture}.rs`, `iced/tests/main/{plot,reference,headless,
placement}.rs`, `figure/src/theme.rs`, `figure/src/figure.rs`,
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

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D18.1–D18.3 decided.
- Conventional commits, no attribution.
