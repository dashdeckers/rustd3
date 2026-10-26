# rustd3 — session handoff 00

2026-10-18 · next: **Phase 1 — Workspace, tooling and fonts**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary (scales,
ticks, shapes, formats, color, zoom and brush) and adds the tier-1
pieces only matplotlib has: typst text and math, legends and
colorbars, layout from measured text, statistical charts. It renders
SVG, PNG, PDF and GIF from one drawing list, and integrates with iced
at two levels: a core level where the app draws through `PathContext`,
and a `Plot` widget that shows vello_cpu images of a `Figure`.

Everything is specified in one document, `rustd3.md`, next to this
file: Part I is the design; the Implementation part holds the front
matter (workspace, conventions, checks, grading, decisions, order of
work), Phases 1–40, Later (0.2), and Appendices A (dispatch preamble)
and B (resume).

## 2. Where things stand

- No code repository yet. Phase 1 creates it: the directory that holds
  `.claude/map/rustd3/` becomes the repository root.
- This directory holds `rustd3.md`, `goon.yaml`, this handoff,
  `guides/` (style, opaque types, decision-log format) and `proto/`
  (the research prototypes, d3 sources and tests at the measured
  versions, the offline translator and Node resolve hook, bundled
  fonts; index in `proto/README.md`).
- Tests: none yet. Rust 1.97 is the toolchain; the dependency set
  (minus iced) resolves.

## 3. Project management

- No GitHub repository yet (`goon.yaml` has `repo: none`). Once Phase 1
  is committed, create the repo and set `repo` and `default_branch`;
  milestones M0–M5 can become GitHub milestones.
- Decisions: `.claude/decisions/phase-NN.md` (`D{phase}.{k}`), one file
  per phase that meets one; `.claude/DECISIONS.md` holds the format and
  an index.
- Handoffs: `SESSION_HANDOFF-NN.md`, numbered sequentially. A phase
  that takes two sessions says in its handoff which step comes next.

## 4. The plan

Phases run in numeric order. Phase 31 is conditional on the benchmark
verdict in `decisions/phase-22.md`.

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1 workspace · 2 harness · 3 first port | pending |
| M1 Core contract | 4–10: continuous scales, ticks, locales and axes, categorical scales and color, curves, recordings and M4, pointer and zoom | pending |
| M2 Figure layer and widget | 11 iced core level · 12 drawing list · 13 typst text · 14 figure model · 15 marks · 16 layout · 17 legends and checks · 18 Plot display · 19 render pipeline · 20 gestures · 21 live data · 22 full-frame benchmark (you run it) | pending |
| M3 Core breadth | 23 color · 24 derive · 25 binned and array · 26 number locales · 27 time I · 28 time II · 29 shapes · 30 behaviors and animation | pending |
| Conditional | 31 tiles and pan translation, if the benchmark asks | pending |
| M4 Complete figure | 32 use cases · 33 handles · 34 marks · 35 themes, colorbars, hatching · 36 hover and brush · 37 PDF and GIF · 38–39 helpers | pending |
| M5 Release | 40 facade, docs, release preparation | pending |

## 5. What to build next: Phase 1

Read `rustd3.md`'s Implementation front matter ("How to use this part"
to "Order of work"), then "Phase 1 — Workspace, tooling and fonts". In
short: the cargo workspace with ten packages and exactly the
dependencies of the Workspace list; figure features `raster`, `png`,
`gif`, `pdf`, `tiny-skia`, `helpers` and the default `fonts`;
`scripts/check.sh quick|full`; clippy `disallowed-methods` for ambient
clocks; `.gitignore` re-including `.claude/`; `CLAUDE.md` pointing at
`rustd3.md`; RUST_STYLE and OPAQUE copied into `.claude/`;
`DECISIONS.md` with the log format; licences per package (fonts:
`license-file`); a complete `rustd3-fonts`; build measurements in
`decisions/phase-01.md`.

## 6. Known risks and open items

- **The full-frame benchmark (Phase 22)** decides whether Phase 31
  (tiles, translation, a render pool, envelope fills, splatting) is
  built. You run it on your machine; the agent needs your display's
  refresh rate and scale factor first. Agents continue with Phases
  23–30 while you run it.
- **Riskiest phases:** 9 (recordings and the M4 index: the one core
  algorithm without a prototype to lift), 14 and 34 (where the
  four-wheel figure meets the real layout, text and legend engines),
  27–28 (V8 `Date` emulation in the harness, i128-ns time).
- **iced is pinned** to master 3cbdacc; rustd3-iced is published only
  once iced 0.15 is on crates.io. iced's tiny-skia backend announces a
  move to vello_cpu; moving the pin will change software-renderer
  numbers.

## 7. Key files to read first

1. `rustd3.md`: the Implementation front matter, then the phase.
2. `guides/RUST_STYLE.md`, `guides/OPAQUE.md`, `guides/DECISIONS.md`.
3. `proto/README.md`.
4. `goon.yaml`.

## 8. Commands

Before Phase 1 nothing builds. After it:

```bash
scripts/check.sh quick          # every commit
scripts/check.sh full           # end of a phase (long; background)
cargo run -p rustd3-harness -- --check   # from Phase 2: ratchet and floors
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green at a phase's end.
- Harness counts and floors recorded (from Phase 2).
- Decisions written to `.claude/decisions/phase-NN.md`.
- Conventional commits, no AI attribution.
- The next handoff written (for a half-done phase: the next step).
