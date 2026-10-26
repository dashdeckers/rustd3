# rustd3 — session handoff 02

2026-10-26 · Phase 2 complete · next: **Phase 3 — First port: path, ticks, format, ease**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 2 is complete; `scripts/check.sh quick` and `full` pass at
822679d.

- `rustd3-testkit`: `obs::{Obs, ToObs, Tolerance, Token, tokenize,
  Error}` (from `proto/poc/obs`, with `()` and `Result` impls and the JSON
  conversion), `pixels::{Image, Diff, Error}` with the 32 px tile score,
  `snapshot::Store` (fails on a missing reference unless
  `RUSTD3_BLESS=1`, writes `name.new`), `zone(name)` over jiff's bundled
  database. 41 unit tests, every spot check among them.
- `dev/harness/d3/`: the 20 d3 packages (LICENSE, package.json, src), the
  14 tested modules' `test/`, the two `locale/` directories (4.8 MB,
  `diff -r` against `proto/` empty). `translate/vendor/acorn` (8.19.0),
  `translate/d3-resolve{,-hooks}.mjs` pointing at `../d3/`.
- Translator: `translate.mjs` (suites table, ids, output, `--check`,
  summary), `expr.mjs` (generic translation), `rules/ease.mjs`
  (`easeLinear` → `stub::ease_linear`); 514 lines of JavaScript.
- `rustd3-harness` is a binary: `main.rs`, `registry.rs`, `prelude.rs`,
  `record.rs`, `run.rs`, `ratchet.rs`, `stub.rs`, `generated.rs` +
  `generated/stub.rs`; `passing.txt`. 21 unit tests (the ratchet's five
  regressions among them).
- `scripts/check.sh quick` runs `rustd3-harness --check` (under
  `TZ=Asia/Kathmandu`) after the test loop; `full` runs
  `translate.mjs --check`.

Decisions: none met; no `decisions/phase-02.md`.

Commits: 80b94ef (obs), 3100710 (pixels, snapshots, zones), eab7f89
(vendored d3, acorn, hook), df7e165 (translator), f4ed05b (runner,
report, ratchet, `passing.txt`), 822679d (check.sh gate), and this
handoff.

Harness: one suite, `stub` — `d3-ease/test/linear-test.js`: 13
assertions, 11 translated, 2 skipped (`js:` 2), 0 not reached, 11 pass
d3's semantics, 11 passing, 0 failing. Floors: `linear-test.js` 11 (was:
no harness). The failure path was proven by hand (stub `t * 1.001`: 10
id regressions and the floor regression, `--bless` refused) and reverted;
see f4ed05b's body.

Tests: testkit 41, harness 21, fonts 4. `check.sh full` took 7.5 s warm.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1 workspace · 2 harness · 3 first port | 1–2 done; 3 pending |
| M1–M5 | 4–40 | pending |

## 5. What to do next

Phase 3: read `rustd3.md`'s front matter and "### Phase 3 —", this
handoff's section 6, then `dev/harness/translate/expr.mjs` (its header
documents the typed values and the rules interface).

## 6. Notes for Phase 3 (how Phase 2 left the harness)

- Rules interface (`expr.mjs` header): `identifiers` (name → typed
  value; a callable is `{c, t: "fn", arity, ret}`), `method(obj, prop,
  args, ctx)` returning a typed value or undefined, `display` (types
  observed through `.to_string()`). Rules take precedence over a file's
  own helpers, so `precision` in `pathRound-test.js` can map to the
  prelude helper. Field reads (`spec.fill`) have no hook yet.
- The proto's module-specific code (path methods, format fields,
  locales, vec `pop`/`reverse`) was left out of `expr.mjs`; it belongs in
  `rules/*.mjs`. `Math.PI` etc. emit `std::f64::consts::…`.
- Constructs the translator does not handle become `gap: <construct>`
  (e.g. `gap: ForOfStatement`, `gap: identifier easeQuad`), without a
  phase; Phase 3's exit criteria allow only `gap: phase 26`, so its rules
  must cover or recategorize them.
- Generic unwraps are `.expect("d3 test")` (a `result-<t>` value, and a
  statement returning a `Result`); Phase 3 names `.expect("d3 test
  arc")` for `p.arc(…)`.
- The generated header allows only `unused_mut, unused_parens,
  clippy::all`, so the translator prefixes unused locals and helper
  parameters with `_` and emits a helper only when committed code uses
  it.
- `Kind::PathEq` and `prelude::throws` carry `cfg_attr(not(test),
  expect(dead_code, …))`; once the path suite uses them the expectation
  fails clippy, so Phase 3 must delete both attributes. Delete
  `pub(crate) use crate::stub;` in `prelude.rs` with `stub.rs`.
- `check(id, Kind::Throws, …)` passes iff `actual` is the `Obs` of an
  `Err` (`Str("error: …")`); the translator emits `throws(id, || …)`.
- A record passing d3's semantics but not the tolerant format is an
  internal fault (exit 1); with `PathEq` that happens when a path number
  is off by more than 1e-9 and either side prints it as an integer, which
  leaves no last-place slack (e.g. `M1e-7,0` against `M0,0`).
- `mod generated` is `#[rustfmt::skip]` in `main.rs`, so `cargo fmt`
  never touches generated files.

## 7. Key files

`CLAUDE.md`; `.claude/DECISIONS.md`; `rustd3.md` (front matter, then
"### Phase 3 —"); `dev/harness/translate/expr.mjs`;
`dev/harness/src/prelude.rs`; `dev/harness/passing.txt`.

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
- No decisions met.
- Conventional commits, no attribution.
