# rustd3 — session handoff 22

2026-10-27 · **Phase 20 — Plot III: gestures** complete, with D20.4 and D20.5 open (both recommended as built) · next: **Phase 21 — Plot IV: live data**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**D19.1 (decided C)** first:

- 9ecd547 `build: optimize vello_cpu's pipeline and tiny-skia in the dev
  profile` — `[profile.dev.package."<crate>@<version>"] opt-level = 3`
  for vello_cpu 0.3.0, vello_common 0.3.0, fearless_simd 0.7.0 and
  tiny-skia 0.12.0 (versioned so hayro's own copies, and typst-library,
  stay as they were). The 200k-dot plot-area cell renders in 0.94 s
  instead of 59.3 s in a debug build; the section's 200k-dot pipeline
  tests are back (`DOTS = 200_000`, 10 s waits). Measurements appended
  to `decisions/phase-19.md`.

**Phase 20:**

- c9e1d6a `feat(figure): zoomed handles and the readers a gesture widget
  needs` — `Panel::{zoom_x, zoom_y}`, `Resolved::zooms`,
  `Held::hold_erased` (D20.1), `Resolved::scale_at` and
  `Resolved::axis` (D20.4).
- 11c66e6 `feat(iced): wheel zoom with widget-owned and published
  domains` — steps 1 and 2: `plot/{listener, domains, input, region,
  gesture}.rs`, `Plot::on_change`, `State::{translate, queue, apply}`,
  refresh with overrides and held areas, the 300 ms wheel end with
  `request_redraw_at`, the cursor.
- cd6a1c9 `feat(iced): drag pan and gesture sync` — step 3; the
  harness's `shoot` and `interaction`; `run_frame` in the tests.
- 143321a `feat(iced): scaled previews of pending plot-area cells` —
  step 4: `plot/preview.rs` (`Affine1`, `View`, `Basis`; D20.3).
- 60ebd24 `feat(iced): double-click reset and the held layout's tests`
  — step 5.
- 0d6eef5 `docs(iced): gestures in Plot's documentation` — step 6.
- dc8f665 `test(iced): the cursor reader only where the gesture tests
  build`.

Exit criteria (commands: `ICED_TEST_BACKEND=tiny-skia|wgpu
TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture`;
state-level tests with `--lib gestures`): (1) every test named under
Steps passes on both backends (`iced/tests/main/gestures.rs`, 15 tests;
`iced/src/plot/state/gestures.rs`, 4; `preview.rs`, `slot.rs` units).
(2) `pan_survives_batched_updates`: the same final domain bit for bit
with an update per 1, 3 and 8 inputs. (3)
`layout_holds_during_a_wheel_gesture`: the rect holds over eight steps
and redraws to +288 ms; one relayout at +300 ms. (4)
`double_click_resets_touched_handles` (200 ms) and
`slow_double_click_does_not_reset` (400 ms, and 8 px) pass. (5)
`zoom_shows_a_scaled_preview_at_once` and `sub_precision_zoom_is_ignored`
pass on both. (6) `scripts/check.sh quick` on every commit; `full` green
at dc8f665 in 2 min 5 s (warm), no warnings.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-20.md`): D20.1–D20.3 decided (the
section's answers); D20.4 open (A: `Resolved::scale_at(index)` and
`Resolved::axis(panel, axis)`, additive readers the section's design
needs but does not name); D20.5 open (A: thirteen details; (10) is the
one to look at: a reset publishes "the handle resolved with no
override", which for an app that builds a published handle with its own
window is that window, not the data's extent).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–20 done; 21 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

The architect answers D20.4 and D20.5; neither changes what Phase 21
builds on unless D20.4 renames the readers (only rustd3-iced calls
them). Then Phase 21: read `rustd3.md`'s front matter and "### Phase 21
—". The refresh is `State::redraw` → `apply` → `refresh` → `repaint` in
`iced/src/plot/state.rs`; `Plot::revision` and `Caching::Off` plug into
`State::plan`.

## 6. Notes for later phases

- Phase 21: a gesture holds a `Fit::Window` handle through the widget's
  `owned` map (held via `Held::hold_erased`), until a double click
  resets it; a published windowed handle meets D20.5 (10). Tests that
  start a worker hold `pipeline::workers()` (now `pub` within the test
  crate).
- Phase 21/22: `apply` bumps the generation for accepted steps and
  resets, and the refresh's plan again when cells change; a publishing
  pass reports a `Frame` sample without building the list.
- Phase 22: run the 200k-dot case in release and record the
  calibration factor (D19.3); report the trace at 2× (D18.3) and the
  1 px line (D15.3). The previews' quality is visible in the benchmark's
  preview share; a pan preview exposes the theme background where the
  shifted image left, and the plot-area half of the spines moves with
  it.
- Phase 31: tile translation would replace the affine previews of pans;
  the per-handle `View` (range shares since an epoch) is where snapped
  drags could read their offset.
- Phase 33: the time-handle wheel test and the E0631 doctest of
  `on_change`; `Listener` already publishes `[S::Domain; 2]`.
- Phase 36: hover messages and brushing can use `region::Map` (plot
  areas and axis strips from D20.2) and the `Resolved` readers of D20.4.

## 7. Key files

`figure/src/{panel.rs, figure/resolved.rs, figure/held.rs}`,
`iced/src/plot.rs`, `iced/src/plot/{state, domains, listener, input,
region, gesture, preview, slot, job}.rs`, `iced/src/plot/state/gestures.rs`,
`iced/tests/main/{gestures, headless, reference, pipeline}.rs`.

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
- D20.1–D20.3 decided; D20.4 and D20.5 open with recommendations.
- Conventional commits, no attribution.
