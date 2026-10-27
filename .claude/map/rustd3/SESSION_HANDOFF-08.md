# rustd3 — session handoff 08

2026-10-27 · Phase 8 complete (D8.2 open; D8.1 decided) · next: **Phase 9 — Recordings and data**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 8 is complete; `scripts/check.sh quick` passes on every commit and
`full` on the last code commit (22 s warm; no cold build this session).

- `rustd3-core`, new module `shape`:
  - `shape::line()` → `Line { curve }` with `curve(Curve)` and
    `draw(points, ctx)` over `impl IntoIterator<Item = V: Vertex>`
    (`(f64, f64)` or `Option<(f64, f64)>`; `None` breaks the line);
    `shape::area()` → `Area { curve, baseline }` with `curve`, `y0(f64)`
    and `draw(items, ctx)` over `S: Span` (`(x, y0, y1)`, `(x, y1)` over
    the baseline, and their `Option`s). Both are d3-shape 3.2.0's loops.
  - `shape::Curve`: all 20 d3 curves, the parameterized ones as
    `Bundle { beta }`, `Cardinal*{ tension }`, `CatmullRom*{ alpha }`
    (d3's defaults 0.85, 0, 0.5 are the translator's, not `Default`s;
    `Curve::default()` is `Linear`).
  - Crate-private `shape::curve::State`: one variant per family
    (`linear`, `step`, `basis`, `bump`, `bundle`, `cardinal`,
    `catmull_rom`, `monotone`, `natural` under `core/src/shape/curve/`),
    dispatched by `match` (the `each!` macro for `line_start`, `line_end`,
    `point`; explicit matches for `area_start`/`area_end`, where closed
    curves and Bundle do nothing). `State::new(curve)` for lines,
    `State::area(curve)` for areas (Bundle draws as Basis). The private
    three-state `Line` flag is d3's `_line` (`closes(lone)`, `toggle`,
    `start`). MonotoneY draws through a reflecting `PathContext`.
  - `js::truthy` (crate-private): JavaScript's truthiness of a number.
- Tests: core 29 unit (+ `js::truthy`, monotone's `Reflect`) + 294 in
  `main` (new: `shape.rs`, 50 tests: every Spot checks row, the counting
  `PathContext` double, d3's monotone edge cases, gaps, baselines) + 39
  doctests.
- Harness: new suite `shape` (`rules/shape.mjs`, 109 lines, 89 of code,
  budget 150). `expr.mjs` gained an `array` hook, `nullable` types and a
  `statement` hook that sees declarations too. Prelude: `drawn`, `shape`,
  `Curve`.

Harness counts (translated = passing; every passing assertion also
passes d3's semantics):

| Suite | Assertions | Passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js 3, types 2 |
| ease | 431 | 365 | js 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap 51, js 21, types 18 |
| scale | 1,104 | 723 | api 106, gap 6, js 174, types 95 |
| interpolate | 49 | 38 | gap 8, js 3 |
| color | 333 | 298 | js 29, types 6 |
| shape | 337 | 234 | api 85, js 18 |
| total | 3,303 | 2,612 | 691 |

Floors: 79 (was 57): 20 curve files (every one at its full translated
count; all ≥ 70 % but `bundle-test.js` 2 of 3, D8.1), `line-test.js` 3,
`area-test.js` 7 (both in Phase 8's skip table).

Decisions (`.claude/decisions/phase-08.md`):

- **D8.1** (decided, the spec's answer) — `bundle-test.js` 2 of 3, its
  one skip `curveBundle.beta("0.5")`; floor 2.
- **D8.2** (open, recommending what is built) — (1) `Line::closes(lone:
  bool)`, each curve passing its own single-point test; (2) Catmull–Rom
  takes the cardinal state for any alpha that is not truthy (0 or NaN);
  (3) `State::new` and `State::area`; (4) `Reflect` implements all six
  `PathContext` methods; (5) no `null`-item translator rule (no test has
  one); (6) `.curve(c, 0.5)` translated as d3 reads it, `.curve(c)`;
  (7) the curve files hold 237 assertions, not about 280.

Commits: 45b5633 (line and area, linear curves), 384cd01 (step, basis,
bump, bundle), 3c23047 (cardinal, Catmull–Rom), 8dca618 (monotone,
natural), 79e86da (harness), 4b8374d (D8.1, D8.2), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | 8 done (D8.2 open); 9 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 9: read `rustd3.md`'s front matter and "### Phase 9 —" (it builds
on Phase 3 only; it names a cut point), and section 6 below.

## 6. Notes for later phases

- Phase 15 (figure marks): lines and areas draw through
  `shape::line()`/`shape::area()` into any `PathContext`; thinning and
  guard-band clipping happen before the iterator reaches `draw`.
- Phase 29 (radial lines and areas, `line_of`): reuse
  `curve::State` (`pub(crate)`): `State::new` for lines, `State::area`
  for areas; `LineOf::draw` maps through its accessors into
  `Line::draw`. d3's `curveBumpRadial` and `curveRadial` are not in
  `Curve`. The closed-family `area_start`/`area_end` no-op group in
  `State` holds `Bundle` too.
- Phase 34 (density on y): a vertical area draws through a
  coordinate-swapping `PathContext`, like monotone's private `Reflect`
  (which could move up to `shape` if a second user appears).
- Harness: `rules/shape.mjs` applies a file's `reflect`-style helper to
  literal points while translating (`mapped`); points are `{t:
  "points", pairs}`. A rules file can type array literals (`array`) and
  list `nullable` types; the `statement` hook now runs before
  declarations, so a declaration can skip its whole test.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-08.md`; `core/src/shape.rs`,
`core/src/shape/{line,area,curve}.rs`, `core/src/shape/curve/*.rs`,
`core/src/js.rs`; `core/tests/main/shape.rs`;
`dev/harness/translate/rules/shape.mjs`, `dev/harness/translate/expr.mjs`,
`dev/harness/src/prelude.rs`, `dev/harness/passing.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core shape
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--allow-drop] [--bless-floors]]
node --import ./dev/harness/translate/d3-resolve.mjs script.mjs   # d3 in Node
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (22 s).
- D8.2 open with its recommendation; D8.1 decided.
- Conventional commits, no attribution.
