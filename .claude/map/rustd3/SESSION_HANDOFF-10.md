# rustd3 — session handoff 10

2026-10-27 · Phase 10 complete in one session (D10.2 open) · M1 closed · next: **Phase 11 — rustd3-iced core level and image placement tests**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 10 is complete; `scripts/check.sh quick` passes on every commit and
`full` on the last code commit (24.8 s warm; no cold build this
session). M1 (the core contract, Phases 4–10) is closed: from here on,
changing or removing a public item of Phases 1–10 is a decision whose
commit updates every call site.

- `rustd3-core`, new module `interact` (`core/src/interact.rs` and
  `core/src/interact/`):
  - `Pointer` (`#[non_exhaustive]`, public fields: `Pressed`, `Moved`,
    `Released`, `Wheel`, `DoubleClicked`, `Modifiers`, `Left`), `Delta`
    (`Lines`/`Pixels`, DOM sign convention), `Button`, `Modifiers`
    (private flags; `NONE`, `SHIFT`, `CTRL`, `ALT`, `LOGO`, `union`,
    readers).
  - `interact::Transform` (d3's `ZoomTransform`; public `k`, `x`, `y`;
    `IDENTITY`, `scale`, `translate` with d3's zero shortcut, `apply`,
    `invert`, per-axis forms, `Display` as d3's `toString`).
  - `Zoom` (d3's defaults; `scale_extent`, `translate_extent`, `extent`
    builders; `handle`, `is_active`, `scale_by`, `scale_to`,
    `translate_by`, `translate_to`, `constrain`). Private `Gesture`
    (grab location, wheel anchor, last modifiers).
  - `Rescale` (`rescale_x`, `rescale_y`, `pan_px`, `zoom_px`) for every
    `Continuous<T, f64, N>` and `scale::Identity`.
- `scale::Pad` (`core/src/scale/pad.rs`) for every `Continuous<T, R, N>`
  and `scale::Identity`; `dev/fixtures/pad.py` and `pad.json` (26
  matplotlib cases, byte-identical on rerun).
- `Continuous`: crate-private `forward()` (D4.1, D6.1) beside
  `transform()` and `get_domain()`, all three now in the impl block
  without bounds (D10.2 (1)); `scale::polymap` is crate-visible.
- Tests: core 58 unit (8 in `interact::zoom`, 3 in `interact::rescale`,
  2 in `scale::pad`), 374 in `main` (33 in `interact.rs`, 11 in
  `pad.rs`), 49 doctests.

Harness: 9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80
floors (before: 8 suites, 3,303, 2,612, 691, 79). New suite `zoom`:
`d3-zoom/test/transform-test.js` 11 of 11, floor 11; `rules/zoom.mjs` 36
lines (budget 50). The three DOM files of d3-zoom are not translated
(Phase 10's skip table); `interact/zoom.rs` ports `zoom-test.js`'s
sequences.

Decisions (`.claude/decisions/phase-10.md`):

- **D10.1** (decided, the spec's answer) — no data on a logit scale pads
  to [1e-7, 1 − 1e-7].
- **D10.2** (open, recommending what is built) — twelve details: (1)
  `get_domain`/`transform()` moved so `Pad`'s unbounded impl compiles;
  (2) exit criterion 1 runs as `cargo test -p rustd3-core -- interact
  scale::pad pad::`; (3) V8's `Math.pow` one ulp off in the pixel wheel
  row; (4) sticky upper bound ≥, as matplotlib; (5) the empty-log row is
  `LogLocator.nonsingular`'s start, not matplotlib's autoscaled view;
  (6) `Pad` uses `for_domain(extent)`; (7) decreasing transforms;
  (8) orientation for N > 2; (9) ctrl double click filtered as d3;
  (10) the wheel anchor's location kept through a vertical burst;
  (11) d3's zero-translate shortcut; (12) non-finite extents as written.

Commits: 878fc88 (pointer input, zoom transform), 803ec51 (zoom
behavior), 357fb14 (rescale, pixel gestures), 071d71b (pad, fixture),
9abb262 (harness suite), 7b10013 (doc link fix), 689643e (decisions),
and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done (D10.2 open) |
| M2 Figure layer and widget | 11–22 | 11 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 11: read `rustd3.md`'s front matter and "### Phase 11 —" (it
builds on Phases 3 and 10; it adds iced to the manifests and pins the
revision), and section 6 below. Run `cargo check -p rustd3-iced
--all-features` before starting.

## 6. Notes for later phases

- Phase 11 (`pointer::Tracker`): `Pointer` is `#[non_exhaustive]`, and
  clippy's `wildcard_enum_match_arm` (workspace lint) flags a `_` arm
  that also covers known variants; matches in other crates list every
  variant and then `_` (see `core/tests/main/interact.rs`). `Zoom` reads
  shift and ctrl for double clicks only from `Pointer::Modifiers`, so the
  tracker emits one on every `ModifiersChanged`.
- Phases 14, 17, 20: `Pad::pad(extent, margin, sticky)` takes the merged
  extent of admitted values; it pads under the transform as it applies
  to the extent (`for_domain`). `pan_px`/`zoom_px` return `None` on any
  degenerate step; the widget keeps the gesture's transform then.
  `Zoom` keeps d3's `[0, ∞]` scale extent; the widget sets `[1e-6, 1e6]`.
- Phase 27 (time scales): `Rescale` and `Pad` impls in i128 ns, with
  the same orientation rule (D10.2 (8)) and a 1 ns minimum span.
- Phase 30 (brushes, drag): `Pointer`, `Button` and `Modifiers` are
  ready; d3's ctrl-press filter applies as in `Zoom::handle`.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-10.md`; `core/src/interact.rs`,
`core/src/interact/{pointer,modifiers,transform,zoom,rescale}.rs`;
`core/src/scale/pad.rs`; `core/tests/main/{interact,pad}.rs`;
`dev/fixtures/pad.py`; `dev/harness/translate/rules/zoom.mjs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core -- interact scale::pad pad::
python3 dev/fixtures/pad.py && git diff --exit-code dev/fixtures/pad.json
cargo run -p rustd3-harness -- --check
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (24.8 s).
- D10.1 decided; D10.2 open with its recommendation.
- Conventional commits, no attribution.
