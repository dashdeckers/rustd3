# rustd3 — session handoff 33

2026-10-28 · **Phase 33 — Figure handles breadth**, complete in one session (no cut) · next: **Phase 34 — Figure marks breadth** (Phase 31 still waits for D22.1's verdict)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**First, D32.3's addition:** 2558a96 `fix(examples): a right margin for
use case 1's x labels` (24 px; plots stay 860 px; window and simulator
924 × 430; checked with iced headless snapshots).

**Phase 33:**

- 2e299e5 `feat(figure)` — Step 1: band and point handles;
  `Issue::OutsideDomain { handle, count }` from dots, lines and rules,
  summed per handle by `fig.check`.
- 3fbdb3b `feat(core)` — D33.1: `Pad for Time` widens one instant by an
  hour per side.
- 71f090e `feat(figure)` — Step 2: time handles (`Scale`, `Kind` with
  `Span = SignedDuration`, `Position`, `Invert`, `Indexed<Timestamp>`,
  `Fittable`); windows and `Fit::Visible` memory in i128 ns; the time
  branch of `Draw::indices`; locale.rs's time test on a figure handle
  (D32.1).
- d21517b `test(iced)` — the time-handle wheel test.
- a10914f `feat(core)` — `Sequential::admits`, `Diverging::admits`,
  `Pad` for sequential, diverging and quantize scales.
- 09f236f `feat(figure)` — Step 3: color and binned handles
  (scale/color.rs, scale/binned.rs), the quantile doctest pair.
- f9207d3 `feat(figure)` — Step 4: `Axis::rotate`, `layout::Obb`, both
  collision passes on turned boxes.
- ba2b67e `test(figure)`, d0583ed `test(iced)` — Step 5: the E0271,
  E0277 (ordinal), E0308 and E0631 rows; the band row carries E0277.
- A `test(figure)` fix: `rule_x` imported inside the `fonts`-only test.
- The closing docs commit (`decisions/phase-33.md`, `DECISIONS.md`, this
  handoff).

Exit criteria, each checked this session:

1. `cargo test -p rustd3-figure --test main -- handles`: 19 pass (15 in
   handles.rs, one per kind and the time rows; the filter also matches
   older tests).
2. `cargo test -p rustd3-figure --test main -- rotate`: 6 pass (rotate.rs's
   5 and Phase 14's rotated left title).
3. `cargo test -p rustd3-figure --doc` and `cargo test -p rustd3-iced
   --features figure --doc` run the E0271, E0631 and quantile doctests and
   their twins.
4. `grep -rn --after-context=1 'compile_fail' figure/src iced/src`: six
   rows, each `compile_fail,E…` with a twin (axis.rs, mark/dots.rs ×3,
   scale/binned.rs, iced plot.rs).
5. `TZ=Asia/Kathmandu cargo test -p rustd3-figure` (in `check.sh full`);
   `check::the_examples_fit_at_960_by_760` (the trace and scatter
   examples check empty) unchanged and green.
6. `cargo run -p rustd3-harness -- --check`: unchanged, 16 suites, 6,722
   assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.
7. `scripts/check.sh quick` green on every commit; `full` green at
   d0583ed in 10 min 51 s (warm) and again at the closing docs commit,
   after a test fix for an import that warned without `fonts`.

Decisions: D33.1 and D33.2 decided (the section's answers); D33.3 open,
recommended A (thirteen details as built). D22.1 still waits for the user.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; Phase 22's verdict pending the user's run |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | only if D22.1's verdict asks |
| M4 Complete figure | 32–39 | 32, 33 done; next: 34 |
| M5 Release | 40 | pending |

## 5. What to do next

- D33.3 answered by the architect.
- Phase 34 (needs 22's code, 29, 33; two sessions, cut point in its
  section).

## 6. Notes for later phases

- Color handles: `Paint::paint` is `None` for values the scale does not
  admit (NaN, infinities, outside the transform), so Phase 34's `fill_by`
  skips them; `Kind::domain()` is `Some` for continuous color scales,
  `None` for binned ones. No data keeps a color handle's built domain.
- Windows and `Fit::Visible` are type-erased by `Box<dyn Any>`: a window's
  span and last value in their own types (`scale/window.rs::shown`), the
  visible memory as `visible::Domain`. A new domain type needs a branch
  there.
- Rotated labels: `Labels::oriented` gives the turned box, `Labels::bounds`
  its bounding rectangle (what layout reads); `Placed::boxes` hold
  `layout::Obb`s; `Resolved::label_boxes` returns rectangles. Phase 39's
  presets can call `.rotate(30.0)` for dates.
- Time label parity counts interval units from the floored epoch
  (`axis/draw.rs::intervals`).
- Scratch renders for this phase are in the session's scratchpad only.

## 7. Key files

`figure/src/scale/{time,color,binned,window,visible,indexed}.rs`,
`figure/src/layout/{obb,collide}.rs`, `figure/src/axis/{labels,placed,draw}.rs`,
`figure/src/check/find.rs`, `figure/src/mark.rs`,
`core/src/time/scale/pad.rs`, `core/src/scale/{pad,sequential,diverging,quantize}.rs`,
`figure/tests/main/{handles,rotate}.rs`, `iced/tests/main/gestures.rs`,
`.claude/decisions/phase-33.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --test main -- handles rotate
cargo test -p rustd3-figure --doc
cargo test -p rustd3-iced --features figure --doc
ICED_TEST_BACKEND=wgpu cargo test -p rustd3-iced --all-features -- wheel_zoom_on_a_time_handle
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at d0583ed
  and at the closing docs commit.
- D33.3 open with its recommendation; D22.1 waiting for the user.
- Conventional commits, no attribution.
