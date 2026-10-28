# rustd3 — session handoff 38

2026-10-28 · **Phase 36 — Plot interactions II: hover and 1-D brush**: **Steps 1–4 done; Phase 36 complete**, with D36.1–D36.3 open (recommended as built) · next: **Phase 37 — Outputs II: PDF and GIF**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 0235149 `feat(figure)` — Step 1: `figure/src/brush.rs` (`brush::x`,
  `brush::y`, `Brush<A, B>`, `.sets(target)` bounded `B: Position<Domain
  = A::Domain>`), `brush/placed.rs` (`Placed`: axis, handle positions,
  `window`/`place` conversions as fn pointers made at `From<Brush<A,
  B>>`), `brush/dim.rs`; `Panel::brush` (one per panel; file outputs
  ignore it); `Resolved::panel_cell(panel)` and `Resolved::brush(panel)`
  (D36.1). The E0271 `compile_fail` doctest and its twin over two time
  handles.
- f4ca2ef `feat(iced)` — Step 2: `plot/hover.rs` (`Hover`, public at
  `plot::Hover`; `Hits`: the hit index, rebuilt with lists whose cells
  changed, and the hover last reported); `Plot::on_hover`; moves are
  hit-tested at once, reported once per frame at `RedrawRequested`,
  never during a gesture; the overlay takes the found hit.
- 4f6f958 `feat(iced)` — Step 3: `plot/brush.rs` (`Brush`, the widget's
  state per placed brush, kept by panel cell, over key and axis;
  `brush::Spot`, a placed brush as one refresh resolved it),
  `brush/machine.rs` (core's `BrushX`/`BrushY` as one enum),
  `brush/listener.rs` (`on_brush`'s erased closure); `Plot::on_brush`;
  routing in `State::apply` (`stroke`, `stroke_on`, `unbrush`,
  `publish`), the sync rule in `State::sync` at each refresh, held layout
  during brush gestures, double click, cursor.
- acaab7b `feat(iced)` — Step 4: `Brush::draw` (fill 10%, outline one
  device px at 40%, clipped to the brushed plot area), called from
  `State::draw` before the hover overlay.
- The closing docs commit: `decisions/phase-36.md`, the index, this
  handoff.

Exit criteria, all met:

1. `ICED_TEST_BACKEND=tiny-skia|wgpu cargo test -p rustd3-iced
   --features figure --test main -- hover brush`: 13 pass on each
   (4 hover, 8 brush, Phase 18's crosshair test), including the
   figure-level focus and context.
2. `cargo test -p rustd3-figure --doc brush`: the E0271 row, its twin,
   `Panel::brush`'s example.
3. `brush_overlay_stays_inside_its_panel`: 20,475 (k = 1) and 82,501
   (k = 2) px changed, all inside panel (0, 0)'s plot area, on both
   backends.
4. `scripts/check.sh quick` green on every commit; `full` green at
   acaab7b in 11 min 43 s.

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D36.1 (how rustd3-iced reads a panel's brush: the readers
above), D36.2 (axis presses on a brushed panel keep Phase 20's axis
drag), D36.3 (fifteen details), all open with recommendation A. D22.1
still waits for the user.

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
| M4 Complete figure | 32–39 | 32–36 done; 37 next |
| M5 Release | 40 | pending |

## 5. What to do next

Phase 37 (PDF and GIF), from its first step, once the architect has
answered D36.1–D36.3. Only rustd3-iced calls D36.1's readers; renaming
them touches `iced/src/plot/{brush,state}.rs` only.

## 6. Notes for later phases

- A brush's selection lives in core's machine in list px; between
  gestures `Brush::sync` re-derives it from the domain it selects (the
  target's current one, or for a brush without a target its own last
  selection kept as a scale of over's type), so layouts and zooms move
  the band with the data. During a gesture nothing syncs and selections
  invert through over's scale as at the gesture's start.
- A target's domain is set with `Kind::fitted(extent, &[], 0.0)`, as
  `Fit::Window` sets windows: exact on linear and time scales, an ulp on
  log ones.
- Hover's index is rebuilt only when a refresh's cells changed; a
  helper that changes hit candidates without changing drawn commands
  would need a revision bump.
- `State::redraw` takes `(on_change listeners, on_brush listeners)` as a
  pair; the state-level tests pass `(&[], &[])`.
- Screenshots and the scratch test that wrote them (`shots.rs`) are in
  the session's scratchpad only.

## 7. Key files

`figure/src/brush.rs`, `figure/src/brush/{placed,dim}.rs`,
`figure/src/panel.rs`, `figure/src/figure/resolved.rs`,
`iced/src/plot.rs`, `iced/src/plot/{hover,brush,overlay,state}.rs`,
`iced/src/plot/brush/{listener,machine}.rs`,
`iced/tests/main/{hover,brush}.rs`, `figure/tests/main/figure.rs`,
`.claude/decisions/phase-36.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-iced --features figure --test main -- hover brush
ICED_TEST_BACKEND=wgpu cargo test -p rustd3-iced --features figure --test main -- hover brush
cargo test -p rustd3-figure --doc brush
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  acaab7b.
- D36.1–D36.3 open with their recommendations; D22.1 waiting for the
  user.
- Conventional commits, no attribution.
