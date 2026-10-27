# rustd3 — session handoff 05

2026-10-27 · Phase 5 complete (D5.1 and D5.2 open) · next: **Phase 6 — Ticks, locale, core axes and the identity scale**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 5 is complete; `scripts/check.sh quick` and `full` pass (full:
14 s warm; no cold build this session).

- `rustd3-core`:
  - `scale::TickRule` (`Transformed`, `Data`, `Custom`; `custom`,
    `ticks`) and `.tick_rule(…)` on every `Continuous` (a private
    `tick_rule` field that survives `range`).
  - `ticks::{transformed, decades}` join `linear`, `log`, `subdivide`;
    crate-private `ticks::decade(c, k)` (c·10ᵏ, split past 10³⁰⁸).
  - `scale::transform::{Symlog, Logit, Asinh, Func, Any}`; Symlog and
    Asinh have private fields and crate-private `new`, Func a
    hand-written `Debug`. Constructors `scale::{symlog, logit, asinh,
    func}`; builders `constant` (symlog) and `linear_width` (asinh).
  - `scale::piecewise(domain, range)` (N ≥ 2 by an inline `const`
    assert; E0308 `compile_fail` pair) on a crate-visible
    `Continuous::with_stops`; `scale::AnyContinuous<R, N>` =
    `Continuous<transform::Any, R, N>`, one `From` per built-in
    transform; both macros take their list from `builtin_transforms!`
    in `scale.rs`; crate-private `Continuous::map_transform`.
  - Bail-outs from fuzzing (D5.1 (6)): `ticks::linear` none for a
    non-finite end, `transformed` drops values escaping the ends,
    `subdivide` drops non-finite steps, `log` drops overflowing powers.
- `rustd3-testkit`: `Tolerance::relative(rel)` (no absolute floor).
- `dev/fixtures/scales.py` → `scales.json`: 32 cases (12 symlog, 9
  logit, 11 asinh); numpy pinned to its baseline instruction set
  (`NPY_ENABLE_CPU_FEATURES=X86_V2`) so reruns are byte-identical.
- Tests: core 21 unit + 134 in `main` (`tests/main/rules.rs` holds the
  Phase 5 spot checks, `fixtures.rs` 28 compared / 4 skipped, `fuzz.rs`
  2,000 debug cases, 20,000 in 0.85 s release with `RUSTD3_FUZZ=1`) +
  18 doctests + 3 `compile_fail` with twins.
- `scripts/check.sh quick` runs the release fuzz when `RUSTD3_FUZZ=1`.
- Harness: `rules/scale.mjs` 167 lines (+12): `scaleSymlog`,
  `.constant`, `piecewise` for linear chains reaching n > 2 equal stops
  (no vendored assertion reaches it), scale values carry `n`.

Harness counts (every passing assertion also passes d3's semantics):

| Suite | Assertions | Passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js 3, types 2 |
| ease | 431 | 365 | js 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap 51, js 21, types 18 |
| scale | 801 | 496 | api 67, gap 59, js 108, types 71 |
| interpolate | 27 | 27 | — |
| total | 2,308 | 1,842 | 466 |

Floors: 49 (was 48): `symlog-test.js` 39 added; the others unchanged.

Decisions (`.claude/decisions/phase-05.md`, both open, recommending
what is built):

- **D5.1** — (1) Asinh rounds by matplotlib's `⌊ln|x| / ln 10⌋`, not
  log₁₀, which three spot rows need; (2) the `[-1e9, 1e9]` asinh row
  is ±1e8 under glibc's arithmetic (Rust's too) and ±1e9 only under
  numpy's AVX-512 arcsinh; (3) Logit's linear branch is d3's ticks, and
  matplotlib's `MaxNLocator` differs on 4 of 8 probed domains (fixture
  keeps agreeing cases); (4) "symlog-thinned" misses thinning under 2 ×
  count (`[0, 1e12]`); (5) Asinh's fallback capped at 10,000; (6)
  bail-outs the fuzz properties need.
- **D5.2** — `symlog-test.js` passes 39 of 71 (the spec says 73): js
  23, api 6, types 2, gap 1. Recommended: skip-table row, floor 39.

Commits: a62e1d8 (TickRule, transformed, decades), 7ceaacc (Symlog,
Func), d925391 (Logit, Asinh), 93916b2 (piecewise, AnyContinuous),
b436ea6 (testkit), 7c917f2 (fuzz bail-outs), a6e278d (fixtures and
fuzz), f16ea99 (harness), d7c0671 (D5.1, D5.2), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–5 continuous scales · 6–10 | 5 done (D5.1, D5.2 open); 6 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 6: read `rustd3.md`'s front matter and "### Phase 6 —",
`decisions/phase-03.md` to `phase-05.md`, and section 6 below.

## 6. Notes for later phases

- Phase 6 adds `Continuous::{transform, forward}` (pub(crate)),
  `Locale::group_from`, `Format::format_all` (D3.2, D4.1). `Ticks` for
  `Continuous<T, f64, N>` covers `AnyContinuous` and `piecewise` scales
  (ticks on the end stops). Labels of logit and asinh come from their
  own `tick_format` (`.12~g`, `,.12~g`).
- A transform-specific builder exists only on its `Continuous<…>`
  (`exponent`, `base`, `constant`, `linear_width`); `AnyContinuous`
  takes them before `.into()`.
- Phase 7 (color ranges): `piecewise([…], [Rgb; N])` works through
  `Interpolate`; the translator's piecewise rule needs numeric stops
  today (`stops()` in `rules/scale.mjs`).
- Phase 10 (`Rescale`, `Pad`) and 23 (sequential symlog/logit): the
  forward cache is refreshed by `transformed`; `map_transform` keeps it.
- The fuzz (`core/tests/main/fuzz.rs`) builds every built-in transform
  through `AnyContinuous`; extend `transforms()` when a transform is
  added.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-05.md`; `core/src/scale.rs`,
`core/src/scale/{any,continuous,piecewise,tick_rule}.rs`,
`core/src/scale/transform/*.rs`, `core/src/ticks/rules.rs`;
`core/tests/main/{rules,fixtures,fuzz}.rs`; `dev/fixtures/scales.py`;
`dev/harness/translate/rules/scale.mjs`; `dev/harness/passing.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
RUSTD3_FUZZ=1 scripts/check.sh quick             # plus 20,000 fuzz cases in release
scripts/check.sh full                            # end of a phase (background)
python3 dev/fixtures/scales.py && git diff --exit-code dev/fixtures
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--allow-drop] [--bless-floors]]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green.
- D5.1 and D5.2 open, with recommendations.
- Conventional commits, no attribution.
