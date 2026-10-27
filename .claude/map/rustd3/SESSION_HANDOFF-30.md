# rustd3 — session handoff 30

2026-10-27 · **Phase 28 — Time II: formats, locales and time ticks**, complete (one session, no cut) · next: **Phase 29 — Shapes breadth**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 28** (core, harness), one session; the section's cut after Step
2 was not needed:

- bdcb912 `feat(harness): the upstream skip category, first for
  utcTime-test.js:283` — D27.3 B; the harness reports 0 failing again.
- 7be43bf `feat(harness): translator rules for d3-time-format and time
  tick formats` — Step 0, `rules/time_format.mjs`.
- da8817c `feat(core): time::Locale, Definition, Ladder and Format, with
  Time::tick_label` — Steps 1 and 4's core together (D28.2 (1)).
- ca8adf2 `feat(harness): grade d3-time-format's format, utcFormat and
  locale tests` — the suite `time_format`.
- 03fa49e `feat(harness): grade the time scales' tick formats` — floors
  84 and 78 (D27.4's followup).
- 6d224b5 `feat(core): time::iso_format and time::iso_parse` — Step 2.
- 9d34be6 `feat(harness): grade d3-time-format's isoFormat and isoParse
  tests`.
- 8ea1b43 `feat(core): time locales generated from d3-time-format's
  tables` — Step 3, `dev/harness/gen/time_locales.mjs`,
  `core/src/time/locales{,/*}.rs`, `dev/fixtures/time_locales.json`.
- 9e63997 `build: time-locale freshness in check.sh full`.
- 899319e `feat(core): Ticks for time::Time and the figure locale's time
  half` — Step 5; the 56-row check against d3 in its body.
- 54cb28a `style(core): name the indices in Directive's invariant
  message`.
- The closing docs commit (`decisions/phase-28.md`, `DECISIONS.md`, this
  handoff).

Exit criteria, each checked this session:

1. `scripts/check.sh quick` green on every commit.
2. `cargo run -p rustd3-harness -- --check`: format 122 / 137, utcFormat
   132 / 140, isoFormat 2 / 2, isoParse 3 / 3, locale 1 / 1 (the loop over
   the 35 tables), time-test.js 84 / 96, utcTime-test.js 78 / 85; listed:
   defaultLocale 2 / 13, parse 0 / 139, utcParse 0 / 97, the two year-10002
   assertions, the seven `multi()` per file.
3. `fresh` reruns `time_locales.mjs`; a rerun writes the same bytes, and a
   one-character uncommitted edit of `de_de.rs` fails it.
4. `cargo test -p rustd3-core --test main -- time::labels::berlin_example`.
5. `cargo test -p rustd3-core --test main -- time::locales::time_locales_match_d3`
   (and `bundled_time_locales_write_the_sections_rows`).
6. `cargo test -p rustd3-core --test main -- time::labels::sub_second_labels`.
7. `scripts/check.sh full` green at 54cb28a in 10 min 15 s (warm, the
   benchmark smoke included).

Harness: 13 suites, 6,411 assertions, 5,100 passing, 1,311 skipped, 0
failing, 161 floors (Phase 27b left 12 / 5,879 / 4,794 / 1,084 / 1
failing / 153). Translator: Phase 28 +123 / −5 of 200 (D27.3's
`expr.mjs` +11 / −2 besides).

Decisions: D28.1 (open, recommended B: `locale::Locale::new(number,
time)`, since no public item puts a runtime `time::Locale` into a figure
locale; nothing in Phase 28 depends on it), D28.2 (open, recommended A:
twelve details as built). D22.1 still waits for the user's windowed
benchmark run.

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
| M3 Core breadth | 23–30 | 23–28 done; next: 29 |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 29 (builds on 8), from Step 0.
- D28.1 before Phase 33 draws time axes: if B, one `feat(core)` commit.

## 6. Notes for later phases

- Phase 33 (time handles): `Ticks for Time` is in `time/scale/labels.rs`;
  labels read only `locale.time()`, each tick on its own; `minor` is
  empty (the section's rule). Time axes take the figure locale's time
  half; a runtime time table needs D28.1.
- `time::Locale`'s readers are crate-private (`date_time`, `date`,
  `time`, `period`, `weekday`, `month`, `ladder`); the figure crate sees
  only `format`, `new`, the constants and `Default`.
- The generated time tables are `#[rustfmt::skip]` (D28.2 (6)); `fresh`
  in `scripts/check.sh` reruns all three generators.
- `upstream:` skips need the Node run that shows d3 failing; the
  translator lists them in `expr.mjs`'s `UPSTREAM`.
- Phase 40 (facade): `rustd3::time` re-exports `Locale`, `Definition`,
  `Ladder`, `Format`, `format`, `iso_format`, `iso_parse` through the core
  path.

## 7. Key files

`core/src/time/{locale,definition,format}.rs`,
`core/src/time/locale/ladder.rs`,
`core/src/time/format/{directive,item,pad,iso}.rs`,
`core/src/time/locales.rs`, `core/src/time/scale/labels.rs`,
`core/src/locale.rs`, `core/tests/main/time/{format,labels,locales}.rs`,
`dev/harness/gen/time_locales.mjs`, `dev/fixtures/time_locales.json`,
`dev/harness/translate/rules/time_format.mjs`,
`dev/harness/src/generated/time_format.rs`,
`.claude/decisions/phase-28.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main -- time:: locale::
TZ=UTC cargo test -p rustd3-core --test main -- time::   # zone independence, by hand
TZ=America/Los_Angeles node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/time_locales.mjs
cargo run -p rustd3-harness [-- --failing d3-time-format | --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  54cb28a.
- D28.1 and D28.2 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
