# rustd3 — session handoff 23

2026-10-27 · **Phase 21 — Plot IV: live data** complete, with D21.1, D21.2 and D21.4 open (all recommended as built) · next: **Phase 22 — Full-frame benchmark**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 21:**

- a052abf `feat(figure): Fit::Window follows live data in whole columns`
  — step 1: `scale/window.rs` (the quantization, `window::Memory` of
  column counts in the `Cache`), `Entry::window`, `Fits::across` and
  per-panel reports, resolve in rounds until the windows' column counts
  are the laid-out ones (at most 3), `Labels::reserve` for horizontal
  axes of windowed handles (D21.1); spot-row tests in `fit.rs`.
- 2c99eb7 `feat(iced): Plot::revision and Plot::caching` — step 2:
  `plot/caching.rs`, the pipeline carries both to `State::plan`.
- 60de9af `fix(figure): a clipped part touches only the cells inside its
  clip` and 1108c09 `fix(figure): an axis line is a part of its own`
  (role 8, `Spine`) — D21.2, so an append re-renders only what it
  touched; three snapshots blessed (part lines only).
- 0340667 `test(iced): live appends, windows and drop_front re-render
  touched cells` — step 3, with `Harness::park`/`Parked::resume`.
- 80dd830 `docs(iced): live data, revision and caching in Plot's
  documentation` — step 4.
- The closing docs commit (decisions, this handoff).

Exit criteria (commands: `cargo test -p rustd3-figure --all-features
fit`; `ICED_TEST_BACKEND=tiny-skia|wgpu TZ=Asia/Kathmandu cargo test -p
rustd3-iced --all-features -- --nocapture plot::`): (1) the `Fit::Window`
spot rows pass (`fit_window_moves_in_whole_columns`,
`fit_window_with_no_data_is_all`,
`fit_window_column_edges_shift_by_whole_columns`). (2)
`live_append_re_renders_only_touched_cells`,
`fit_window_follows_in_whole_device_columns` (factors 1, 1.25, 2) and
`ten_hz_appends_stay_sharp` pass on both backends. (3)
`revision_change_re_renders_everything`, `caching_off_renders_every_cell`,
`drop_front_re_renders_the_cell`, `a_gesture_holds_a_windowed_handle`
pass on both. (4) `scripts/check.sh quick` on every commit; `full` green
at 80dd830 in 4 min 5 s (warm), no warnings.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-21.md`): D21.1 open (A: resolve
rounds and the end-label reserve of windowed axes), D21.2 open (A: clip-
aware cell bucketing and axis lines as parts), D21.3 decided (the
section's answers), D21.4 open (A: ten details).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–21 done; 22 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

The architect answers D21.1, D21.2 and D21.4; none changes a public API.
Then Phase 22: read `rustd3.md`'s front matter and "### Phase 22 —".

## 6. Notes for later phases

- Phase 22: a live scenario with `Fit::Window` resolves once per frame
  in a steady layout (the cache remembers the column counts) and twice
  from a fresh cache; the cells an append re-renders are the plot areas,
  the cells under the panels holding x ticks and the bottom row's
  middle, plus the side cells when a tick sits within a pixel of a plot
  area's end. `ten_hz_appends_stay_sharp` predicts 4.56 ms per append at
  800 × 500 at 2 (1,386,708 px, 716 items). Report the trace at 2×
  (D18.3, origin-dependent dashes; the live tests use smooth sines to
  avoid them) and the 1 px line (D15.3).
- Phase 31: parts drawn in a clip no longer reach cells past it; axis
  lines are their own parts (role 8). Reusing the memo per column
  across a shift and shifting images instead of re-rendering stay there.
- Phase 33: `Fit::Window(SignedDuration)` goes through the `Fitter`'s
  `last`/`window` functions (today `f64` only, else `All`) and the same
  rounds and reserve; time windows' quantization in i128 nanoseconds.
- Live apps that mutate a recording between frames build their view
  after `update`; the test harness models that with `park`/`resume`.

## 7. Key files

`figure/src/{scale/window.rs, scale/fit.rs, figure/resolve.rs,
figure/fits.rs, figure/resolved.rs, layout/passes.rs, axis/labels.rs,
axis/placed.rs, figure/decorate.rs, figure/part.rs, list/parts.rs}`,
`figure/tests/main/{fit, parts}.rs`, `iced/src/plot.rs`,
`iced/src/plot/{caching, state}.rs`, `iced/tests/main/{plot,
headless}.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
ICED_TEST_BACKEND=tiny-skia TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture plot::
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -p rustd3-iced --all-features -- --nocapture plot::
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D21.3 decided; D21.1, D21.2 and D21.4 open with recommendations.
- Conventional commits, no attribution.
