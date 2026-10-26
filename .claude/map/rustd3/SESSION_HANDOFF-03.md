# rustd3 — session handoff 03

2026-10-27 · Phase 3 complete (D3.1 and D3.2 open) · next: **Phase 4 — Continuous scales I**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 3 is complete; `scripts/check.sh quick` and `full` pass at
35b1dac (full: 11 s warm; no cold build this session).

- `rustd3-core`: `path` (`PathContext`, `SvgPath` with `with_digits`,
  `try_arc`, `arc_to`, `rect`; `path::Error`; `path::svg`), `ticks`
  (`ticks`, `tick_increment`, `tick_step`, `nice` with the bail-outs and
  the 10,000 cap), `format` (`Specifier`, `Align`, `Sign`, `Symbol`,
  `Kind`, `InvalidFormat`, `Format::{new, prefixed, format}`,
  `format`, `format_prefix`, `Locale::{NEUTRAL, EN_US, new, format,
  format_prefix}`, `Definition`, `precision_{fixed,prefix,round}`),
  `ease` (`Ease`, `Mode`), crate-private `js` (`round`, `to_int`,
  `Number`, JavaScript's `String(x)` without exponent notation) and
  `format::decimal` (toFixed/toExponential/toPrecision with JavaScript's
  round-half-up on exact ties, detected from the float's bits).
- Tests: core 8 unit + 42 in the `main` binary (`core/tests/main/
  {path,ticks,format,ease}.rs`, every spot check) + 4 doctests; harness
  23 unit tests.
- Harness: suites `path`, `ease`, `ticks`, `format`; the stub is gone.
  Rules in `translate/rules/{path,ease,array,format}.mjs`; `expr.mjs`
  gained the hooks `call`, `field`, `apply`, `equal`, `statement`, the
  `opt` type, field-write `types:` skips, try/finally and per-value
  `expect` messages (its header documents them). Translator 770 lines.

Harness counts (all also pass d3's own semantics):

| Suite | Assertions | Translated = passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js: 3, types: 2 |
| ease | 431 | 365 | js: 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap: 51 (phase 26), js: 21, types: 18 |
| total | 1,480 | 1,319 | 161 |

Floors: 40 `#floor` lines, one per file, each at its passing count
(was: one, `linear-test.js` 11, which stays).

Decisions (`.claude/decisions/phase-03.md`, both open):

- **D3.1** — `format-type-c-test.js` (0/5: `c` formats strings,
  `Format::format` takes `f64`) and `defaultLocale-test.js` (2/6: the
  module's rebound exports) are under 70 % and not in the skip table.
  Recommended: add both to the skip table. Floors 0 and 2 are blessed.
- **D3.2** — three statements of Phase 3 that do not hold: the doc alias
  `ticks` on `ticks` (rustc rejects it), `Locale::group_from(digits)`
  (dead until Phase 6; deferred to Phase 6), "doubled once" (d3 doubles
  twice for counts in [0.5, 1); built as d3).

Commits: 9defd46 (js, path), f6424c5 (ticks), cacf2ad (format), 3258e3a
(ease), 35b1dac (harness), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1 workspace · 2 harness · 3 first port | done (D3.1, D3.2 open) |
| M1 Core contract | 4–10 | pending |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 4: read `rustd3.md`'s front matter and "### Phase 4 —",
`decisions/phase-03.md`, and section 6 below.

## 6. Notes for later phases

- Phase 6 adds `pub(crate) fn Locale::group_from(self, digits: usize)
  -> Locale` with `Format::format_all` (D3.2). `Locale`'s fields are
  `pub(super)` to `format`; the grouping rule lives in `Format::format`
  (`group_from <= 1` always groups, as d3).
- `Specifier::with_kind` is `pub(super)` (formatPrefix's `f`);
  d3-scale's tickFormat needs only `with_precision` and `with_trim`.
- Departures from d3 kept on purpose (spec): no JavaScript exponent
  notation in printed numbers (paths, `c`, `toFixed` at 1e21 and above);
  `d` and `X` of ±∞ print `Infinity` (d3: `∞`, `INFINITY`);
  `formatPrefix` with a reference value of 0, NaN or ±∞ formats NaN, as
  d3 does.
- A grid of 4,466 format outputs against the vendored d3-format matched
  except for those departures (not committed; the commit body of cacf2ad
  describes it).
- Translator: the `call(node, ctx)` hook sees the raw AST (object
  literals, `new`); `calleeName` ignores names shadowed by locals; a
  skip inside a helper keeps its own category; `rules.statement` names a
  loop's skip reason. Generated files stay `#[rustfmt::skip]`.
- `nice` is translated as `ticks::nice(…)` (an `[f64; 2]`, which `Obs`
  reads) without the spec's `.to_vec()`.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-03.md`; `core/src/{js,path,ticks,
format,ease}.rs` and `core/src/format/*.rs`; `dev/harness/translate/
expr.mjs`, `rules/*.mjs`; `dev/harness/src/prelude.rs`;
`dev/harness/passing.txt`.

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
- D3.1 and D3.2 open, with recommendations.
- Conventional commits, no attribution.
