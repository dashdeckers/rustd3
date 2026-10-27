# Phase 27 — Time I: intervals and time scales

## Measurements

**27a** (Steps 0–5), 2026-10-27, 81367ba … 19604c8, one session.
`scripts/check.sh full` green at 19604c8 in 9 min 59.6 s (warm, the
benchmark smoke included).

**Harness**, before (259dc56) and after (19604c8): 10 → 11 suites, 4,562
→ 5,698 assertions, 3,580 → 4,676 passing, 982 → 1,022 skipped, 116 →
151 floors. The new suite `time` translates 35 d3-time files (1,136
assertions; `interval-test.js`, 32 more, is not translated: custom
intervals): 1,096 translated, all passing under d3's semantics too.
Skips: api 26 (d3's plural aliases of `range` such as `timeDays`; `count`
and `every` read as values on `timeYear.every(n)`; the ambient clock in
`day-test.js`), js 13 (numbers and `new Date(NaN)` as dates, NaN and
fractional range steps, `timeTicks(NaN, NaN, 10)`), types 1 (a negative
range step). Every file is at or above 91 %:

| Files | Passing / assertions |
| --- | --- |
| `millisecond`, `utcMillisecond`, `second`, `utcSecond`, `minute`, `utcMinute`, `hour`, `utcHour`, `week`, `utcWeek`, `year`, `utcYear`, `utcTicks` | all (4, 1, 29, 28, 28, 27, 50, 49, 36, 36, 25, 25, 25) |
| `day` | 83 / 91 |
| `utcDay`, `unixDay` | 63 / 64 |
| `sunday`, `utcSunday` | 47 / 50 |
| `monday`, `utcMonday` | 20 / 21 |
| `tuesday` … `saturday` and their `utc` twins | 19 / 20 |
| `month`, `utcMonth` | 90 / 94, 65 / 68 |
| `multiYear`, `utcMultiYear` | 10 / 12 |
| `ticks` | 25 / 26 |

Floors are each file's passing count. The section counts `ticks-test.js`
at 27 and `utcTicks-test.js` at 26 assertions, and d3-time at 1,167; the
translator finds 26, 25 and 1,136 + 32 = 1,168.

**Translator**: +138 / −3 lines (`rules/time.mjs` 126, `expr.mjs` +6 /
−2 for the `number` and `observe` hooks, the suite entry), of the
phase's 250.

**Reference values**: the prelude's `local` and `utc` equal V8's
(`TZ=America/Los_Angeles`, Node 22) on all 838 distinct `local(…)` and
`utc(…)` calls in d3-time's tests, to the millisecond (a one-off check).
Every 27a spot row holds; the rows not taken from `test-time-zones` were
recomputed with d3-time 3.1.0 in Node under Berlin, São Paulo, Los
Angeles and UTC. The time tests pass under `TZ=Asia/Kathmandu` and
`TZ=UTC`.

**"Decisions this phase may meet"**: not met; jiff's compatible
disambiguation and V8 agree on every test date.

## D27.1 — `tick_interval` takes a zone it does not use

**Date:** 2026-10-27 · **Context:** Phase 27, step 5 (786b402) · **Status:** decided

### Issue

Design, free tick functions: `pub fn tick_interval(start: Timestamp,
stop: Timestamp, count: usize, zone: &TimeZone) -> Interval`, beside
`utc_tick_interval(start, stop, count)`; Part I, "Sub-millisecond time
works": the free functions are "each taking its zone". The interval
depends on the span alone: d3's `timeTickInterval(start, stop, count)`
has no zone, and the zoned and `utc` choices differ only in `Day(k)`
against `UnixDay(k)`, which is not a zone. The parameter has nothing to
do.

### Investigation

`d3-time/src/ticks.js:39–48` (tickInterval reads `stop − start` and
`count` only). In `time/ticks.rs` the parameter is `_zone`, documented
as the zone [`ticks`] lays the interval out in; 27b's
`Time::tick_interval` would pass the scale's zone.

### Options

| | **A — keep the parameter (as built)** | B — drop it |
| --- | --- | --- |
| Approach | `tick_interval(start, stop, count, _zone)` | `tick_interval(start, stop, count)`, as `utc_tick_interval` |
| Cost | none now | one signature, one translator line, the tests |
| Correctness | the same intervals | the same intervals |
| Fit | the section's signature and Part I's "each taking its zone"; a parameter that does nothing, in the docs as `_zone` | no dead parameter; Part I's sentence reads as "the tick functions" |

### Decision

B (architect, 2026-10-27): `time::tick_interval(start, stop, count)`
takes no zone, as `utc_tick_interval` and d3's `timeTickInterval`: the
interval is chosen from the span alone, and the zone applies where the
interval floors and steps. The 27b session changes the signature, its
tests and the translator line before Step 7 calls it.

### Why

The section states the signature, and keeping it costs one ignored
argument; whether a public parameter that does nothing is worth that
symmetry is the architect's call, and now is the cheapest moment for B.

### Followups

- If B: change `time/ticks.rs`, the `timeTickInterval` rule in
  `rules/time.mjs` and `ticks_follow_d3s_table_in_the_zone`, in one
  commit before 27b's Step 7.

## D27.2 — Details of Phase 27a as built

**Date:** 2026-10-27 · **Context:** Phase 27, steps 0–5 (81367ba … 19604c8) · **Status:** decided

### Issue

1. **One public path per item.** The layout puts `Interval` in
   `time/interval.rs` and `Range` at `interval::Range`, with `time.rs`
   re-exporting; `Interval` would then have two public paths
   (`time::Interval`, `time::interval::Interval`), against CLAUDE.md's
   rule.
2. **Commits.** Steps 1 and 2 as written: with Nanosecond to Second
   alone no unit reads the zone, so `floor`'s `tz` would be dead in the
   step 1 commit.
3. **Day's offset.** The unit table says "calendar days, then floor";
   d3's `timeDay.offset` does not floor (`day-test.js`, "does not round
   the passed-in date": `offset(local(2010, 11, 31, 23, 59, 59, 999), 1)`
   is `local(2011, 0, 1, 23, 59, 59, 999)`).
4. **Day's floor.** "`Zoned::start_of_day` (the first instant of the
   day)"; d3's `setHours(0, 0, 0, 0)` resolves local midnight as `Date`
   resolves any local time. They agree at São Paulo's midnight gap, and
   differ only where a gap straddles midnight (23:30 → 00:30:
   `start_of_day` gives 00:30, `Date` 01:00).
5. **Calendar months and years.** "offset(n): calendar months / years"
   leaves open what happens past a month's end: jiff's spans clamp (Dec
   30 + 2 months = Feb 28), `setMonth` and `setFullYear` carry, and
   `month-test.js` expects the carry (`offset(local(2010, 11, 30), 2)` =
   `local(2011, 1, 30)` = March 2; Feb 29 + 1 year = Mar 1).
6. **`count` of stepped intervals.** "Base units between floor(start)
   and floor(end), ignoring the step": d3's `timeMillisecond.every(k)`
   has a count, `(floor_k(end) − floor_k(start)) / k`, which a count that
   ignores the step does not reproduce (no d3 test calls it).
7. **jiff-core 0.1.1.** `Timestamp::from_nanosecond` checks only that
   the seconds fit `i64` (`jiff-core-0.1.1/src/timestamp.rs:171–185`,
   `new_unchecked`): outside `Timestamp::MIN..=MAX` it debug-asserts in a
   debug build and returns an invalid value in a release build, against
   its documented error.
8. **Count 0.** d3's `tickInterval(…, 0)` is `null` and its ticks are
   `[]`; `tick_interval` returns an `Interval`.
9. **jiff's limits.** `ceil(t)` reads `t − 1 ns`, and floors near
   −9999-01-01 fall before the range, so `Timestamp::MIN` has no ceil
   and a range or ticks from it are empty.
10. **Filtered offsets** move one unit at a time, `|steps|·k` unit moves
    (d3 too); `Second(15).offset(t, 10¹¹)` runs until it leaves jiff's
    range, minutes of work.
11. **Harness helpers.** The section's `ts(&Option<Timestamp>)`,
    `Timestamp::from_millisecond(ms)` for `new Date(ms)`, and
    `prelude::local(&la, …)`.
12. **Interval equality** in `assert.strictEqual(utcMillisecond,
    timeMillisecond)`: the pairs differ in zone.
13. **Weeks' `every` anchor.** d3's `count(0, d)` counts from the week
    that holds the epoch's local date, which depends on the zone (in Los
    Angeles 1969-12-31, a Wednesday).

### Investigation

(1) Precedents: D11.2 (6) (`scale::Transform` in the private
`scale::mapping`, the public `scale::transform` holding the transform
types) and `array::bin` (`Bucket`, `Thresholds`) beside `array::Bin`.
(3), (5) d3-time 3.1.0 `day.js:5`, `month.js:8`, `year.js:8`; Node 22
values in the Measurements. (4) ECMAScript's `UTC(t)` for a gap takes
the offset before the transition, which is jiff's `compatible`
(`jiff-0.2.38/src/tz/ambiguous.rs:483–490`). (7) Found by
`results_outside_jiffs_range_are_none_and_end_a_range` (Hour floor of
`Timestamp::MIN`). (13) Node: `timeThursday.every(3).range(new
Date(1969, 11, 1), new Date(1970, 1, 1))` under Los Angeles is Dec 4,
Dec 25, Jan 15, which the port gives.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | The enum is defined in `time.rs` (`time::Interval`); its methods are in `time/interval.rs`, the public module `time::interval`, beside `Range` and the private unit families `fine`, `clock`, `calendar` | Two public paths for `Interval` |
| (2) | One core commit for Steps 1 and 2 (7f9c44f) and one harness commit grading both; 27a is nine code commits and one docs commit | Step 1 with an unused zone parameter |
| (3) | Offsets keep the local time of day; the floor happens in `ceil` and `range`, as d3 | Fails `day-test.js` |
| (4) | Local midnight resolved with `compatible`, as `Date` | `start_of_day`; no d3 test reaches the difference |
| (5) | `setMonth`/`setFullYear`'s carry: the first of the target month plus `day − 1` days, at the same local time | jiff's clamp; fails `month-test.js` |
| (6) | The unit's own floors and length, the step ignored for every variant | Millisecond(k) as d3, the rest ignoring the step |
| (7) | `time::interval`'s `at` checks `MIN..=MAX` before `from_nanosecond` | — |
| (8) | The year step is infinite and saturates: `Year(u32::MAX)`; `ticks` returns none through its cap of `min(10·count, 10_000)` = 0. 27b's `nice(0)` should leave the domain as d3 does | `Option<Interval>`, a signature the section does not write |
| (9) | `None` and empty, as "None means the result left jiff's range" | — |
| (10) | As d3; the docs of `offset` say so. No closed forms | — |
| (11) | `ts(impl Into<Option<Timestamp>>)` takes a date and an interval's result alike, `tss(&[Timestamp])` the vector; `millis(ms)` wraps `from_millisecond`; `local(&la, &[fields])` and `utc(&[fields])` take a slice of 1 to 7 fields | — |
| (12) | Two intervals are equal when their `Interval` values are (the zone is an argument of each call), so the alias assertions pass | Skip them as aliases: `utcMillisecond-test.js` would pass 0 of 1 |
| (13) | d3's anchor, the week of the epoch's local date in the zone | — |

### Decision

A for all thirteen (architect, 2026-10-27), as built.

### Why

(1) gives the section's paths, `time::Interval` and
`time::interval::Range`, one each, as D11.2 (6) did; (2) avoids a dead
parameter in a commit; (3), (5) and (13) are what d3's tests and
`Date` do; (4) is `Date`'s rule and equals the section's at its row;
(6) is the section's sentence read literally; (7) works around a jiff
bug the section could not know; (8), (9) and (10) keep the section's
signatures and d3's behaviour; (11) and (12) are harness conveniences
that change no core API.

### Followups

- 27b: `Time::nice(0)` and `nice_with` leave the domain unchanged when
  the interval's floor or ceil is `None` (d3's `nice(0)`).
- 27b: `Coordinate for Timestamp` checks `MIN..=MAX` the same way
  before `from_nanosecond` (handoff 09's `is_valid` rule).
- If jiff fixes `from_nanosecond`'s range check, the guard in
  `time::interval`'s `at` can go.
