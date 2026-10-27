# rustd3 — session handoff 29

2026-10-27 · **Phase 27 — Time I: intervals and time scales**, complete (27a and 27b) · next: **Phase 28 — Time II: formats, locales and time ticks**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 27b** (core, harness), over two sessions; the first was
interrupted after 9a84618 without a handoff:

- 7d7022b `refactor(core): time::tick_interval takes no zone` — D27.1 B.
- 939bab7 `feat(core): time::Time, d3's time scale with an explicit
  zone` — Step 6, `core/src/time/scale.rs`; `time::time(zone)`,
  `time::utc()` in `time.rs`; the range-checked `at` and `saturating`
  moved to `time.rs`.
- e8e797b `feat(harness): grade d3-scale's time and utcTime basics` —
  the suite `time_scale` (`scaleTime()`/`scaleUtc()` chains).
- 125b580 `feat(core): Time's ticks and nice in the scale's zone` —
  Step 7, `time/scale/ticks.rs`.
- 212f5c9 `feat(harness): grade the time scales' ticks and nice`.
- c594095 `feat(core): Rescale and Pad for time::Time in nanoseconds` —
  Step 8, `time/scale/{rescale,pad}.rs`, the precision test.
- 37056a0 `feat(core): Recording<Timestamp> through Coordinate for
  Timestamp` — Step 9, `series/clock/timestamp.rs`,
  `core/tests/main/series/timestamp.rs`.
- 9a84618 `test(core): fuzz time ticks and gestures under a per-case
  timeout` — Step 10's fuzz, `core/tests/main/time/fuzz.rs`.
- ed421fb `docs(core): examples for time's intervals, tick functions and
  scales` — Step 10's doc examples (this session).
- fc3d121 `test(core): the 1 us spot rows' zoom and pan on
  2026-10-25T12:00Z` (this session).
- The closing docs commit (`decisions/phase-27.md`, `DECISIONS.md`, this
  handoff).

Exit criteria (27b), each checked this session:

6. `cargo run -p rustd3-harness -- --check` green; `time-test.js` 62 / 96
   (64.6 %), `utcTime-test.js` 56 / 85 (65.9 %); the rest are 22 tick
   formats per file, listed skips, the `clamp()` getter, an iterable
   domain (D27.4) and `utcTime-test.js:283` (D27.3).
7. `cargo test -p rustd3-core --test main -- time::scale::berlin_ticks
   time::scale::rescaling_berlin` and every 27b spot row
   (`core/tests/main/time/scale.rs`; d3's rows recomputed in Node under
   Berlin and Los Angeles).
8. `cargo test -p rustd3-core --test main --
   time::scale::timestamps_2026_over_one_microsecond`.
9. `cargo test -p rustd3-core --test main -- series::timestamp`.
10. `time::scale::degenerate_gestures_return_none`,
    `time::ticks_are_capped`, `time::fuzz` (150 cases, 15 s per case
    unoptimized; `RUSTD3_FUZZ=1 … --release`: 3,000 cases in 14.6 s,
    1 s per case). `scripts/check.sh full` green at fc3d121 in 9 min 30 s
    (warm); also at 9a84618 (architect, 9 min 51 s).

The 43 time and `Recording<Timestamp>` tests pass under
`TZ=Asia/Kathmandu` and `TZ=UTC`.

Harness: 12 suites, 5,879 assertions, 4,794 passing, 1,084 skipped, 1
failing (D27.3), 153 floors (27a left 11 / 5,698 / 4,676 / 1,022 / 151).
Translator: 27b +55 / −2; the phase +199 / −3 of 250.

Decisions: D27.3 (open, recommended A: `utcTime-test.js:283` stays
failing; d3 7.9.0 fails it too), D27.4 (open, recommended A: floors 62
and 56 until Phase 28's tick formats), D27.5 (open, recommended A: eight
details of 27b as built, among them `is_valid` rejecting
`Timestamp::MAX` and the 15 s unoptimized fuzz timeout). D22.1 still
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
| M3 Core breadth | 23–30 | 23–27 done; next: 28 |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- Phase 28 (builds on 26 and 27), from Step 0. Its tick formats are the
  22 `gap: time tick formats (Phase 28)` skips per file in
  `dev/harness/src/generated/time_scale.rs`; the translator's
  `tickFormat` rule in `rules/time.mjs` is where they turn on, and both
  floors rise to 70 % (D27.4).
- `Ticks for time::Time` takes `Step = time::Interval`;
  `Time::tick_interval(count)` is its step.

## 6. Notes for later phases

- Phase 28: the time locales' generator joins `fresh` and `GENERATED`
  in `scripts/check.sh` (handoff 27); `%U %W %V …` use `Interval`'s
  floors and counts (`Week(Sunday, 1)`, `Week(Monday, 1)`, `Year(1)`).
- Phase 33 (time handles): `Pad for Time` fits dates as matplotlib;
  `Recording<Timestamp>`'s column edges come from `Time::invert` at
  device-pixel edges (`core/tests/main/series/timestamp.rs`, `edges`).
- Phase 40 (facade): `pub use rustd3_core::jiff` with the core
  re-exports.
- jiff-core 0.1.1's `Timestamp::from_nanosecond` does not check jiff's
  range (D27.2 (7)); new callers go through `time::at` or
  `time::saturating` (crate-private).

## 7. Key files

`core/src/time.rs`, `core/src/time/scale.rs`,
`core/src/time/scale/{ticks,rescale,pad}.rs`, `core/src/time/ticks.rs`,
`core/src/series/clock/timestamp.rs`, `core/tests/main/time/scale.rs`,
`core/tests/main/time/fuzz.rs`, `core/tests/main/series/timestamp.rs`,
`dev/harness/translate/rules/time.mjs`,
`dev/harness/src/generated/time_scale.rs`, `.claude/decisions/phase-27.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main -- time:: series::timestamp
TZ=UTC cargo test -p rustd3-core --test main -- time::   # zone independence, by hand
RUSTD3_FUZZ=1 cargo test -p rustd3-core --release --test main -- time::fuzz
cargo run -p rustd3-harness [-- --failing d3-scale/test/utcTime | --check]
TZ=America/Los_Angeles node --import ./dev/harness/translate/d3-resolve.mjs script.mjs
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  fc3d121.
- D27.3–D27.5 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
