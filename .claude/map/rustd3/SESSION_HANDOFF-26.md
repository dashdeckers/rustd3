# rustd3 — session handoff 26

2026-10-27 · **Phase 24 — `#[derive(Interpolate)]`** and **Phase 25 — Binned scales and array statistics** complete in one session · next: **Phase 26 — Number locales and `fmt!`**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 24** (derive, core):

- 4f669da `feat(derive): #[derive(Interpolate)] for structs with named
  fields` — `derive/src/lib.rs`: `expand` on proc-macro2, the hand-walked
  type-parameter test, field bounds plus `Self: Clone` on generic structs
  (D24.1), `crate = "path"`, spanned and combined errors; six unit tests.
- 03140d3 `feat(core): re-export #[derive(Interpolate)] beside the trait`
  — `core/src/interpolate.rs`, behind `derive`; five `compile_fail`
  doctests (E0277 twice, E0599, two derive errors) with six passing twins.
- e870a57 `test(core): derive spot checks and use case 2's Bar` —
  `core/tests/main/derive.rs`.
- 91592f4 docs (`decisions/phase-24.md`).

`scripts/check.sh full` green at e870a57 (about 9 min warm).

**Phase 25** (core, harness):

- 604a750 `feat(harness): translator rules for d3-array statistics and
  binned scales` — `rules/stats.mjs`, `rules/binned.mjs`, wired into the
  scale suite before any file used them.
- 5fb4dd7 reductions (`array::{count, min, max, extent, sum, mean,
  variance, deviation}`), the `array` suite, `Math.f(x)` as `f64::f(x)`.
- c93c136 `array::{quantile, quantile_sorted, median, range, bisect_left,
  bisect_right}`, crate-private `quickselect`.
- 204245b `array::{Bin, bin}` (in the private `array/histogram.rs`),
  `array::bin::{Thresholds, Bucket, sturges, scott, freedman_diaconis}`.
- 914971e `scale::{Quantize, quantize}` with `Ticks`;
  `tick_format::stepped` factored out (crate-private).
- 855463a `scale::{Quantile, quantile, Threshold, threshold}` with
  `Ticks`.
- The closing docs commit (`decisions/phase-25.md`, this handoff).

Exit criteria: (1) `scripts/check.sh quick` green at every commit;
`cargo test -p rustd3-core -- array:: binned:: scale::quantize
scale::quantile scale::threshold` passes (the literal command in the
section is rejected by cargo, D25.3 (11)). (2) `cargo run -p
rustd3-harness -- --check` green with the floors of
`decisions/phase-25.md`; fifteen files under 70 % (D25.2). (3) Spot
checks in `core/tests/main/{array,binned}.rs`. (4) Degenerate `range` and
`bin` inputs under a one-second timeout. (5) `scripts/check.sh full`
green at 855463a in 9 min 30 s (warm, the benchmark smoke included).

Harness: 10 suites, 4,562 assertions, 3,529 passing, 1,033 skipped, 116
floors (was 3,822 / 3,040 / 782 / 97). Translator +231 / −3 code lines.

Decisions: D24.1 (open, `Self: Clone` on generic structs), D24.2 (open,
six details); D25.1 decided (not met); D25.2 (open, fifteen files under
70 %), D25.3 (open, twelve details). D22.1 still waits for the user's
windowed benchmark run.

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
| M3 Core breadth | 23–30 | 23, 24, 25 done; next: 26 |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 26 (number locales and `fmt!`): read the front matter and
  "### Phase 26 —"; handoff 25's note applies there: put the scheme-table
  generator's rerun (`node --import ./dev/harness/translate/d3-resolve.mjs
  dev/harness/gen/scheme_tables.mjs` and `git diff --exit-code
  core/src/scheme`) into `scripts/check.sh full`.
- The architect: answer D24.1, D24.2, D25.2 and D25.3.

## 6. Notes for later phases

- Phase 30 (`anim::Marks`): the derive bounds generic structs by
  `Self: Clone` as well as the field types (D24.1), so `Nested<T> { m:
  Mark<T> }` and skipped `PhantomData<U>` derive.
- Phase 40 (facade): `#[interpolate(crate = "::rustd3")]` resolves to
  `::rustd3::interpolate::Interpolate`, so the facade re-exports the
  module, not only the trait (Phase 24's section).
- Phases 34 and 38 (histogram, KDE, box plots): `array::Bin::layout`
  returns indices into the input; `Thresholds::Count` goes through d3's
  ticks, so more than 10,000 bins give one bucket (Phase 3's tick cap).
- Phase 35 (colorbars of binned scales): `Ticks` positions are in [0, 1];
  `Threshold`'s placement is the documented default; labels use `,f` at
  `precision_fixed` of the smallest edge gap, which rounds uneven
  quantile edges (D25.3 (8)).
- Translator: `rules/stats.mjs` projects one-property accessors at
  translation time and declares test-local objects lazily (only the
  assertions that use them skip); `rules/binned.mjs` folds setters into
  the constructor before the first read. Both are in the scale suite.
- `rustd3_core` still does not re-export jiff (handoff 25).

## 7. Key files

`derive/src/lib.rs`, `core/src/interpolate.rs`,
`core/tests/main/derive.rs`; `core/src/array.rs`,
`core/src/array/{count,min,max,extent,sum,mean,variance,deviation,median,quantile,quickselect,range,bisect,histogram,bin}.rs`,
`core/src/array/bin/{thresholds,bucket}.rs`,
`core/src/scale/{quantize,quantile,threshold}.rs`,
`core/src/scale/ticks.rs` (`edges`, `edge_labels`),
`core/src/scale/tick_format.rs` (`stepped`),
`core/tests/main/{array,binned}.rs`,
`dev/harness/translate/rules/{stats,binned}.mjs`,
`.claude/decisions/phase-24.md`, `.claude/decisions/phase-25.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background; includes the smoke)
cargo test -p rustd3-core --features derive      # derive spot checks and doctests
cargo test -p rustd3-core -- array:: binned:: scale::quantize scale::quantile scale::threshold
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--bless-floors]]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the end
  of each phase (e870a57, 855463a).
- D24.1, D24.2, D25.2, D25.3 open with recommendations; D22.1 waiting for
  the user.
- Conventional commits, no attribution.
