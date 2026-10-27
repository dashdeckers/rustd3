# rustd3 — session handoff 28

2026-10-27 · **Phase 27 — Time I: intervals and time scales**, 27a (Steps 0–5) complete · next: **Phase 27b, Step 6** (`Time` core)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 27a** (harness, core):

- 81367ba `feat(harness): translator rules for d3-time` — Step 0:
  `dev/harness/translate/rules/time.mjs` (126 lines) and two `expr.mjs`
  hooks (`number` for `+e`, `observe` for the Obs code of a rules' type).
- eee574c `feat(core): re-export jiff as rustd3_core::jiff` — the
  Workspace table's re-export (handoffs 25 and 26); core's doctest names
  `rustd3_core::jiff`.
- 7f9c44f `feat(core): time::Interval with fixed and wall-clock units` —
  Steps 1 and 2 in one commit (D27.2 (2)): `core/src/time.rs` (the enum,
  `time::Interval`), `time/interval.rs` (its methods; the public module
  `time::interval` with `Range`), `time/interval/{fine,clock,range}.rs`.
- a9767d1 `feat(harness): grade d3-time's fixed and wall-clock
  intervals` — the `time` suite; prelude `local`, `utc`, `millis`, `ts`,
  `tss`; `zone` and `TimeZone` re-exported in the prelude.
- d0a1694 `feat(core): calendar intervals: days, weeks, months and
  years` — Step 3, `time/interval/calendar.rs`.
- d8ecb45 `feat(harness): grade d3-time's calendar intervals`.
- 7a27d06 `test(core): d3-time's zone script, sub-millisecond steps and
  lazy ranges` — Step 4.
- 786b402 `feat(core): d3's time tick functions with an explicit zone` —
  Step 5, `time/ticks.rs`.
- 19604c8 `feat(harness): grade d3-time's ticks and utcTicks`.
- The closing docs commit (`decisions/phase-27.md`, this handoff).

Exit criteria (27a): (1) `scripts/check.sh quick` green at every commit
(tests under `TZ=Asia/Kathmandu`; the time tests also pass under
`TZ=UTC`). (2) `cargo run -p rustd3-harness -- --check` green; all 35
translated d3-time files pass ≥ 91 % (`day-test.js` 83 / 91 lowest),
`ticks-test.js` 25 / 26, `utcTicks-test.js` 25 / 25; `interval-test.js`
is not in the suite (custom intervals). (3)–(5) `cargo test -p
rustd3-core --test main -- time::time_zones_script_cases
time::sub_millisecond_intervals_step_exactly time::interval_range_is_lazy`.
`scripts/check.sh full` green at 19604c8 in 9 min 59.6 s (warm).

Harness: 11 suites, 5,698 assertions, 4,676 passing, 1,022 skipped, 151
floors (was 10 / 4,562 / 3,580 / 982 / 116). Translator +138 / −3 of
the phase's 250 lines.

Decisions: D27.1 (open: `tick_interval` takes a zone it does not use;
recommended keep), D27.2 (open: thirteen details as built, recommended
A). D26.1–D26.3 open from the last session unless answered; D22.1 still
waits for the user's windowed benchmark run.

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
| M3 Core breadth | 23–30 | 23–26 done; 27a done, next: 27b (Steps 6–10) |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 27b, from Step 6: `time/scale.rs` (`Time`, `time::time`,
  `time::utc`), then Steps 7–10 as the section lists. The architect may
  first answer D27.1 (a drop of `tick_interval`'s zone goes in before
  Step 7).
- Carried into 27b: `Coordinate for Timestamp` makes `is_valid` reject
  what `at` cannot represent (handoff 09) and range-checks before
  `Timestamp::from_nanosecond`, which does not check (D27.2 (7)); the
  time impls of `Rescale` and `Pad` work in i128 ns with Phase 10's
  orientation rule (D10.2 (8)) and a 1 ns minimum span (handoff 10);
  `nice(0)` leaves the domain as d3 does (D27.2 (8)).
- The translator has 112 lines left for the `scaleTime()`/`scaleUtc()`
  chains; `rules/time.mjs`'s types (`date`, `odate`, `dates`, `interval`)
  and the prelude's `ts`/`tss` are the pieces to reuse.

## 6. Notes for later phases

- Phase 28: the time locales' generator joins `fresh` and `GENERATED`
  in `scripts/check.sh` (handoff 27); `%U %W %V …` use `Interval`'s
  floors and counts (`Week(Sunday, 1)`, `Week(Monday, 1)`, `Year(1)`).
- Phase 40 (facade): `pub use rustd3_core::jiff` comes with the core
  re-exports (handoff 27's `fmt!` note still applies).
- jiff-core 0.1.1's `Timestamp::from_nanosecond` does not check jiff's
  range (D27.2 (7)); any new caller checks `MIN..=MAX` first.

## 7. Key files

`core/src/time.rs`, `core/src/time/interval.rs`,
`core/src/time/interval/{fine,clock,calendar,range}.rs`,
`core/src/time/ticks.rs`, `core/tests/main/time.rs`,
`dev/harness/translate/rules/time.mjs`, `dev/harness/src/prelude.rs`,
`dev/harness/src/generated/time.rs`, `.claude/decisions/phase-27.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main -- time::
TZ=UTC cargo test -p rustd3-core --test main -- time::   # zone independence, by hand
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing d3-time | --check | --bless [--bless-floors]]
TZ=America/Los_Angeles node --import ./dev/harness/translate/d3-resolve.mjs script.mjs
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  19604c8.
- D27.1 and D27.2 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
