# rustd3 — session handoff 04

2026-10-27 · Phase 4 complete (D4.1 and D4.2 open) · next: **Phase 5 — Continuous scales II**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 4 is complete; `scripts/check.sh quick` and `full` pass (full:
12 s warm; no cold build this session).

- `rustd3-core`:
  - `interpolate`: `Interpolate` (`lerp`) for `f64` and `f32`;
    `number`, `round` (closures, `Clone + Send + Sync`).
  - `scale`: `Transform` (required `forward`, `inverse`, `ticks`;
    defaulted `minor`, `for_domain`, `nice`, `tick_format`);
    `Continuous<T, R, N>` with `domain`, type-changing `range`,
    `range_round`, `interpolate`, `clamp`, `unknown`, `nice`, `apply`,
    `invert` (f64 ranges), `ticks`, `minor`, `tick_format`,
    `tick_format_in`, `get_domain`, `get_range`, `exponent` (Pow),
    `base` (Log); aliases `Linear`, `Pow`, `Log`; `continuous`,
    `linear`, `pow`, `sqrt`, `log`; `TickFormat` (`format`,
    `format_all` value by value, log blanking through a private
    `LogKeep`); `tick_format` (tickFormat.js); crate-private
    `nice_linear`.
  - `scale::transform::{Identity, Pow, Log}`: private fields,
    crate-private `Pow::new`, `Log::new`; `Log`'s crate-private `base`,
    `integer`, `logs`, `pows`.
  - `ticks::{linear, log, subdivide}` (`ticks/rules.rs`), crate-private
    `ticks::pow10`; `Format::locale`; `js::{max, min}` (Math.max/min).
- Tests: core 17 unit + 87 in `main` (`tests/main/{scale,
  interpolate}.rs` hold every spot check) + 9 doctests + 2
  `compile_fail` with twins (`linear().base`, `range([f32]).invert`).
- Harness: suites `scale` and `interpolate`; `rules/scale.mjs` (155
  lines) and `rules/interpolate.mjs` (15). `expr.mjs` gained string
  arrays (`strs`), literal arrays' `items`, `Math.round` as JavaScript
  rounds (`js_round` in the prelude), `Math.SQRT1_2`, test-local
  functions inlined at their call statements (assertions listed once),
  a value's own statement (`stmt`), and the property node in `field`;
  `array.mjs` reads `length` and indexes. Translator 996 lines in all.

Harness counts (every passing assertion also passes d3's semantics):

| Suite | Assertions | Passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js 3, types 2 |
| ease | 431 | 365 | js 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap 51, js 21, types 18 |
| scale | 730 | 457 | api 61, gap 58, js 85, types 69 |
| interpolate | 27 | 27 | — |
| total | 2,237 | 1,803 | 434 |

Floors: 48 (was 40), each at its file's passing count: linear 153,
pow 155, sqrt 3, log 121, tickFormat 25, radial 0, number 13, round 14.

Decisions (`.claude/decisions/phase-04.md`, both open, recommending
what is built):

- **D4.1** — (1) `ticks::log` takes the decades branch above 10,000
  multiples, not at Phase 3's count-scaled cap, which fails d3's
  `log-test.js:220`; (2) `Continuous`'s crate-visible `transform()` and
  `forward()` wait for their first caller (Phase 6); (3) the spot check
  `log().invert(0.5)` is one ulp below Node's (V8's `exp`), asserted
  within tolerance; (4) the built-in transforms keep private fields and
  crate-private constructors.
- **D4.2** — linear (62 %), pow (64 %) and sqrt (50 %) are under 70 %:
  unequal domain and range lengths (`types:`), color ranges (`gap: phase
  7`), JavaScript coercion and aliasing (`js:`), getters (`api:`).
  Recommended: add them to the skip table; floors blessed.

Commits: 331c283 (interpolate), 6a7fd9f (Transform and Continuous),
fc947c1 (Pow and sqrt), 1535248 (Log), 12b09ab (harness), 1a949aa
(spot-check tolerance), ae83b27 (D4.1, D4.2), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4 continuous scales I · 5–10 | 4 done (D4.1, D4.2 open); 5 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 5: read `rustd3.md`'s front matter and "### Phase 5 —",
`decisions/phase-04.md`, and section 6 below.

## 6. Notes for later phases

- Phase 5: `scale::piecewise` builds N > 2; polymap, `invert`, `nice`
  and `ticks` on the end stops are Phase 4's code
  (`scale/continuous.rs`, unit-tested on a three-stop scale built
  in-module). `AnyContinuous`'s `From` impls live inside core, so the
  crate-private `transform::Pow::new` / `Log::new` serve them.
- Phase 5's translator: a domain or range of N ≠ 2 is decided at the
  first step that reads the scale (`settled()` in `rules/scale.mjs`):
  unequal lengths `types:`, then the values' own reason (colors `gap:
  phase 7`, strings and Dates `js:`), then other than two stops `gap:
  phase 5`. No assertion reaches the last today.
- Phase 6 adds `Continuous::{transform, forward}` (pub(crate)),
  `Locale::group_from`, `Format::format_all`, and makes
  `TickFormat::format_all` delegate (D3.2, D4.1).
- Phase 7: color ranges (`gap: phase 7`: 22 assertions each in linear
  and pow, 14 in log, some of which also need Phases 5 and 23) become
  translatable; raise those floors.
- A chain of scale calls on a variable becomes `({ x = x.domain(…);
  x.clone() })` when read in the same expression; generated modules
  allow `unused_assignments` for setters whose readers were skipped.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-04.md`; `core/src/{interpolate,
scale}.rs`, `core/src/scale/*.rs`, `core/src/scale/transform/*.rs`,
`core/src/ticks/rules.rs`; `dev/harness/translate/{expr.mjs,
rules/scale.mjs, rules/interpolate.mjs}`; `dev/harness/passing.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--allow-drop] [--bless-floors]]
node --import ./dev/harness/translate/d3-resolve.mjs script.mjs
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green.
- D4.1 and D4.2 open, with recommendations.
- Conventional commits, no attribution.
