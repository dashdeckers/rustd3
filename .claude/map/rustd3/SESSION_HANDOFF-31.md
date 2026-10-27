# rustd3 — session handoff 31

2026-10-28 · **Phase 29 — Shapes breadth** and **Phase 30 — Behaviors and animation**, complete · M3 closed · next: **Phase 32 — The use cases** (Phase 31 waits for D22.1's verdict)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 29** (core, harness): 0a79c75 … b81f677, then 00e3a7e (its
decision file, written by the architect after the session was cut off).
Pie, arc, stack with its orders and offsets, the fill symbols, the
radial line and area, and `LineOf`; harness 6,411 → 6,722 assertions,
5,100 → 5,292 passing, 161 → 179 floors. D29.1 (six files under 70 %)
and D29.2 (eleven details) decided A. `decisions/phase-29.md` holds the
counts per file.

**Phase 30** (core only):

- 9df22ce `feat(core): join::join and join::Join, d3's keyed data join`
  — Step 1.
- d60488d `feat(core): interact::BrushX, BrushY and BrushEvent over d3's
  brush(dim)` — Steps 2 and 3 together (D30.1 (1)).
- 9b93952 `feat(core): interact::Drag and drag::Event, d3-drag for one
  pointer` — Step 4.
- 102be32 `feat(core): anim::Timing, Animated, Marks and Phase` — Step 5.
- The closing docs commit (`decisions/phase-30.md`, `DECISIONS.md`, this
  handoff).

Exit criteria, each checked this session:

1. `scripts/check.sh quick` green on every commit.
2. `cargo test -p rustd3-core --test main -- brush::brush_traces_match_d3`
   (T1–T4, T8, T11, T1Y–T4Y, every event and `is_active`).
3. `cargo test -p rustd3-core --test main -- drag::drag_traces
   join::join_matches_d3_data_join`.
4. `cargo test -p rustd3-core --test main -- anim::use_case_2_lifecycle`
   (the printed trace equals Part I's text).
5. No clock in library code (clippy `disallowed-methods` in `check.sh`);
   `anim` takes `Instant`s; the tests' base instant is `Instant::now()`
   under `#[allow(clippy::disallowed_methods, reason = "a base instant
   for offsets; values do not depend on it")]` (`core/tests/main/anim.rs`).
6. `cargo run -p rustd3-harness -- --check`: unchanged, 16 suites, 6,722
   assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.
7. `scripts/check.sh full` green at 102be32 in 10 min 40 s.

Decisions: D30.1 (open, recommended A: thirteen details as built; (3),
only the left button's release ends a brush or a drag, is the one
behavior that departs from d3, following the section's brush table and
`Zoom`). D22.1 still waits for the user's windowed benchmark run.

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
| M4–M5 | 32–40 | pending; next: 32 |

## 5. What to do next

- Phase 32 (needs 11, 17, 24, 28, 30): use cases 1 and 2 as apps on
  `BrushX`, `Zoom`, `anim::Marks` and `join`.
- D30.1 before Phase 32 builds on the brush; nothing in Phase 30 waits
  on it.
- Phase 31 as soon as D22.1's verdict asks for it.

## 6. Notes for later phases

- Use case 1 (Phase 32) syncs the brush only between gestures (`if
  !brush.is_active()`); `set_selection` mid-gesture replaces what the
  next move compares against (d3's `brush.js:479, 505`), and a
  selection cleared mid-gesture leaves the previous one to compare
  against (tested in `core/tests/main/brush.rs`).
- Use case 2 (Phase 32): `Marks::default()` builds it; `values(now)` in
  insertion order; gate the frame subscription on `is_animating(now)`,
  which holds during a delay too; `prune` drops only finished exits. A
  key repeated in one `update` keeps one mark (D30.1 (12)).
- Phase 36 (the brush overlay in `Plot`): `BrushX`/`BrushY` take the
  plot area's pixel coordinates and a fixed extent from `new`; handles
  reach `handle_size / 2` past the selection along the other axis too.
- `Drag::handle(input, subject)` calls `subject` only on a left press
  without ctrl.

## 7. Key files

`core/src/join.rs`, `core/src/interact/brush.rs`,
`core/src/interact/brush/{engine,event,x,y}.rs`, `core/src/interact.rs`
(`Drag`), `core/src/interact/drag.rs`, `core/src/interact/drag/event.rs`,
`core/src/anim.rs`, `core/src/anim/{animated,timing,marks}.rs`,
`core/tests/main/{join,brush,drag,anim}.rs`,
`.claude/decisions/phase-30.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main -- brush:: drag:: join:: anim::
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  102be32.
- D30.1 open with its recommendation; D22.1 waiting for the user.
- Conventional commits, no attribution.
