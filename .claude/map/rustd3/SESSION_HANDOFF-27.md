# rustd3 — session handoff 27

2026-10-27 · **Phase 26 — Number locales and `fmt!`** complete in one session · next: **Phase 27 — Time I: intervals and time scales** (27a, Steps 0–5)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 26** (core, harness, scripts, figure):

- 697500d `feat(core): number locales generated from d3-format's tables`
  — `dev/harness/gen/format_locales.mjs` writes
  `core/src/format/locales.rs`, `core/src/format/locales/{de_de,en_us,
  es_es,fr_fr,it_it,ja_jp,nl_nl,pl_pl,pt_br,ru_ru,zh_cn}.rs` (constants
  `format::Locale::DE_DE` … `ZH_CN`; `EN_US` moved out of `locale.rs`,
  equal field for field) and `dev/fixtures/format_locales.json` (d3's
  output for six probes per locale, en-IN and ar-001 included).
- 48e4b66 `feat(harness): d3-format's locale tables at translation time`
  — `locale("xx-YY")` reads the JSON when translating; the "locale data
  is valid" loop becomes a Rust loop over the 59 tables; 51 ids blessed.
- 936e299 `feat(core): fmt! and format::literal::is_valid`.
- cd79857 `build: generated-table freshness in check.sh full` — `fresh`
  runs first in `full`: the generated files equal HEAD before and after
  rerunning `scheme_tables.mjs` and `format_locales.mjs`.
- 9a862dd `fix(figure): the notation label in the figure locale` (the
  D17.1 (7) follow-up).
- 68fedf6 `test(core): degenerate values and Part I's rows under the
  tables`.
- The closing docs commit (`decisions/phase-26.md`, this handoff).

Exit criteria: (1) `scripts/check.sh quick` green at every commit. (2)
Rerunning both generators leaves `git diff --exit-code` clean; `full`'s
`fresh` checks it (verified failing on one-character edits to
`core/src/format/locales/de_de.rs` and `core/src/scheme/categorical.rs`,
committed or not); `bundled_locales_equal_their_json` guards in
`cargo test`. (3) `bundled_number_locales_match_d3` (and
`runtime_number_locales_match_d3` for en-IN and ar-001): `cargo test -p
rustd3-core --test main -- format::`. (4) `cargo run -p rustd3-harness
-- --check` green: `locale-test.js` 42 / 45, `arabicLocale-test.js`
27 / 27, `defaultLocale-test.js` 2 / 6 (D26.2, as D3.1). (5) `cargo test
-p rustd3-core --doc literal`: Part I's three examples, two
`compile_fail` doctests with twins. (6) `literal_check_agrees_with_parser`.
(7) `scripts/check.sh full` green at 68fedf6 in 9 min 16 s (warm).

Harness: 10 suites, 4,562 assertions, 3,580 passing, 982 skipped, 116
floors (was 3,529 / 1,033; floors `arabicLocale-test.js` 0 → 27,
`locale-test.js` 18 → 42). Translator +70 / −20 lines.

Decisions: D26.1 (open: the parser accepts a fill outside the BMP,
which d3's UTF-16 pattern rejects; recommended keep), D26.2 (open:
`defaultLocale-test.js` stays at D3.1's floor 2), D26.3 (open: eight
details as built). D24.1–D25.3 decided; D22.1 still waits for the
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
| M3 Core breadth | 23–30 | 23–26 done; next: 27 (two sessions, 27a/27b) |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 27a (Steps 0–5): read the front matter and "### Phase 27 —".
- The architect: answer D26.1, D26.2 and D26.3.

## 6. Notes for later phases

- Phase 28: append the time-locale generator to `fresh` and its output
  files to `GENERATED` in `scripts/check.sh`; `locale::Locale::{DE_DE,
  …}` pair `format::Locale::DE_DE` … with the time tables (until then
  `locale::Locale::from(format::Locale::DE_DE)` labels axes in German).
- Phase 40 (facade): `fmt!` is `#[macro_export]` at
  `rustd3_core::fmt`; re-export it (`pub use rustd3_core::fmt;`).
- `f` and `%` at 10²¹ and above keep every digit (Phase 3's rule; d3
  writes `1e+21`), asserted in `degenerate_values_under_the_tables_match_d3`.
- The translator's statement hook may now return a statement's Rust
  lines (it records its own assertions through the exported `idOf`).
- The figure's offset/multiplier label goes through
  `format::Locale::format` (`+.{n}r`, `d`), so any locale's decimal,
  minus and numerals apply.

## 7. Key files

`dev/harness/gen/format_locales.mjs`, `core/src/format/locales.rs`,
`core/src/format/locales/*.rs`, `core/src/format/literal.rs`,
`core/src/format/locale.rs`, `core/tests/main/{format,locale}.rs`,
`dev/fixtures/format_locales.json`, `dev/harness/translate/rules/format.mjs`,
`dev/harness/translate/expr.mjs`, `scripts/check.sh`,
`figure/src/axis/offset.rs`, `.claude/decisions/phase-26.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background; fresh, tests, docs, smoke)
node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/format_locales.mjs
cargo test -p rustd3-core --test main -- format:: locale::
cargo test -p rustd3-core --doc literal
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--bless-floors]]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  68fedf6.
- D26.1–D26.3 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
