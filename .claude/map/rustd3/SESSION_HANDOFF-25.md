# rustd3 — session handoff 25

2026-10-27 · **Phase 23 — Color spaces, interpolation and color scales** complete in one session (no cut) · next: **Phase 24 — `#[derive(Interpolate)]`**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 23** (core only):

- b65ed54 `feat(core): Lab, Hcl and Cubehelix with gray and lch` —
  `color/{lab,hcl,cubehelix}.rs`; `K` beside `BRIGHTER`/`DARKER` in
  `color.rs`.
- ba63fb3 `feat(core): color interpolators and Interpolate for the color
  spaces` — `interpolate/{gamma,color,hue}.rs` (`gamma::Channel` is
  d3's color.js, crate-private); `Interpolate for Hsl, Lab, Hcl,
  Cubehelix`; `Rgb::lerp` calls `interpolate::rgb`.
- 780f75c `feat(core): basis, rgb_basis, piecewise, quantize and
  Timestamp lerp` — `interpolate/{basis,piecewise,quantize}.rs`,
  `Interpolate for jiff::Timestamp`.
- d224726 `feat(core): scheme tables generated from d3-scale-chromatic` —
  `dev/harness/gen/scheme_tables.mjs` → `core/src/scheme/{stops,viridis,
  magma,inferno,plasma,categorical}.rs`.
- f3e73f9 `feat(core): every d3-scale-chromatic scheme as a function of
  t` — `scheme/{sequential,diverging,cyclical}.rs`, the ColorBrewer
  notice in `core/LICENSE-d3`, `dev/fixtures/schemes.{mjs,json}`, test
  `schemes_match_d3_at_11_samples`.
- b992a79 `feat(core): Sequential with transforms and Ticks for color
  ranges` — `scale/sequential.rs`; `Ticks for Continuous<T, Rgb, N>`;
  `transform::{Pow, Log, Symlog}::new` public.
- 28b8f16 `feat(core): Diverging with transforms and Ticks` —
  `scale/diverging.rs`; test `examples_in_formats_time_and_color`
  (`core/tests/main/examples.rs`).
- The closing docs commit (`decisions/phase-23.md`, this handoff).

Exit criteria: (1) `scripts/check.sh quick` green at every commit. (2)
`cargo run -p rustd3-harness -- --check` green; floors per
`decisions/phase-23.md` (cubehelix 0/2, gray 2/4, sequential 31/54 under
70 %: D23.2; piecewise 14/21 as the table says). (3)
`cargo test -p rustd3-core --test main schemes_match_d3_at_11_samples`:
38 schemes × 11 samples equal. (4) `examples_in_formats_time_and_color`
passes. (5) `node --import ./dev/harness/translate/d3-resolve.mjs
dev/harness/gen/scheme_tables.mjs && git diff --exit-code core/src/scheme`
clean; every generated file carries the header. (6) `cargo tree -p
rustd3-core -e normal --depth 1`: jiff, thiserror. (7) `cargo doc` with
`-D warnings` and `scripts/check.sh full` green at 28b8f16 (9 min 50 s warm).

Harness: 9 suites, 3,822 assertions, 3,040 passing, 782 skipped, 97
floors (was 3,314 / 2,623 / 691 / 80). Two passing ids pass the tolerant
format only (V8's `Math.log(3)` is one ulp off; D23.3 (11)).

Decisions: D23.1 decided (the section's answers); D23.2 open
(recommended A: the three files' floors as blessed); D23.3 open
(thirteen details as built, recommended A). D22.1 still waits for the
user's windowed benchmark run.

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
| M3 Core breadth | 23–30 | 23 done; next: 24 |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 24 (`#[derive(Interpolate)]`): read the front matter and
  "### Phase 24 —"; it builds on Phase 4's `Interpolate`, which now has
  impls for `f64`, `f32`, `Rgb`, `Hsl`, `Lab`, `Hcl`, `Cubehelix` and
  `jiff::Timestamp`.
- The architect: answer D23.2 and D23.3.

## 6. Notes for later phases

- Phase 26: put `node --import ./dev/harness/translate/d3-resolve.mjs
  dev/harness/gen/scheme_tables.mjs` and `git diff --exit-code
  core/src/scheme` into `scripts/check.sh full`.
- Phases 33 and 35 (colorbars, color handles): `Ticks` for
  `Sequential`, `Diverging` and `Continuous<T, Rgb, N>` positions in
  [0, 1] (`extent()` is `[0, 1]`); `PartialEq` on the color scales is
  D23.1's open question if the figure's table needs it (`Arc::ptr_eq`
  on the interpolator).
- Translator: `rules/scale.mjs`'s `SEQ` builds d3's sequential and
  diverging constructors; `expr.mjs` translates functions of one number
  as closures, and `skip(reason, effect)` lets a skipped assertion still
  run a setter. Budget used here: +123 / −35 code lines.
- Rust 2024: a function taking a slice and returning `impl Fn` needs
  `+ use<…>` (precise capturing), or the closure borrows the slice.
- `rustd3_core` does not re-export jiff yet (the spec says it does);
  core's doctests name `jiff::` directly.

## 7. Key files

`core/src/color/{lab,hcl,cubehelix}.rs`,
`core/src/interpolate/{gamma,color,hue,basis,piecewise,quantize}.rs`,
`core/src/scheme/{sequential,diverging,cyclical}.rs` (hand-written) and
`core/src/scheme/{stops,viridis,magma,inferno,plasma,categorical}.rs`
(generated), `core/src/scale/{sequential,diverging}.rs`,
`core/src/scale/ticks/continuous.rs`, `dev/harness/gen/scheme_tables.mjs`,
`dev/fixtures/schemes.{mjs,json}`,
`dev/harness/translate/{expr.mjs,rules/color.mjs,rules/interpolate.mjs,rules/scale.mjs}`,
`.claude/decisions/phase-23.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background; includes the smoke)
cargo test -p rustd3-core --test main scheme     # the 11-sample fixture
node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/scheme_tables.mjs
node --import ./dev/harness/translate/d3-resolve.mjs dev/fixtures/schemes.mjs
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--bless-floors]]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the end
  of the phase.
- D23.2 and D23.3 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
