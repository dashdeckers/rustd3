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

**27b** (Steps 6–10), 2026-10-27, 7d7022b … fc3d121, two sessions (the
first was interrupted after 9a84618 and wrote no measurements; the
second added the doc examples, ed421fb, and the 1 µs spot rows on the
section's t0, fc3d121). `scripts/check.sh full` green at 9a84618
(architect's run, 9 min 51 s) and at fc3d121 in 9 min 30 s (warm, the
benchmark smoke included).

**Harness**, before (744b4ea, as 27a left it) and after (fc3d121): 11 →
12 suites, 5,698 → 5,879 assertions, 4,676 → 4,794 passing, 1,022 →
1,084 skipped, 0 → 1 failing (`utcTime-test.js:283`, D27.3), 151 → 153
floors. The new suite `time_scale` translates d3-scale 4.0.2's
`time-test.js` (96 assertions: 62 translated, all passing, 64.6 %) and
`utcTime-test.js` (85: 57 translated, 56 passing, 65.9 %); floors 62 and
56 (D27.4). Its 62 skips: gap 44 (tick formats, Phase 28), types 8 (a
polylinear domain, a range of colors with `.interpolate`), js 6
(numbers as dates, an iterable domain, `instanceof`, a distinct copy),
api 4 (d3's `clamp()` getter). The d3-time files are unchanged.

**Translator**: 27b +55 / −2 lines (`rules/time.mjs` +54 / −2 for the
`scaleTime()`/`scaleUtc()` chains, the suite entry); the phase +199 / −3
of its 250.

**Core**: `time/scale.rs` and `time/scale/{ticks,rescale,pad}.rs`,
`series/clock/timestamp.rs`; 27b's `core/src` diff +680 / −65 lines (doc
examples included), its `core/tests` diff +874 / −21.

**Spot checks**: every 27b row is asserted (`core/tests/main/time/scale.rs`,
doc examples on `Time`, `Rescale for Time`, `Pad for Time`). The d3 rows
were recomputed in this session with the vendored d3-scale 4.0.2 and
d3-time 3.1.0 in Node: the Berlin rows under `TZ=Europe/Berlin`
(`apply` gives 448.00000000000006 there, asserted within 10⁻⁹), the
others under `TZ=America/Los_Angeles`; every value matches the table.
The matplotlib rows are the section's (3.11.2), not rerun.

**Fuzz**: `RUSTD3_FUZZ=1 cargo test -p rustd3-core --release --test main
-- time::fuzz`: 3,000 cases in 14.6 s, none over the 1 s per-case
timeout (the first session measured the slowest at 133 ms). The default
runs of `check.sh` are unoptimized, 150 cases with 15 s per case
(D27.5 (5)).

**Zones**: the time and `Recording<Timestamp>` tests (43) pass under
`TZ=Asia/Kathmandu` and, by hand, `TZ=UTC`.

**"Decisions this phase may meet"**: not met in 27b; every `time_scale`
assertion that d3 itself passes, the port passes.

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

## D27.3 — `utcTime-test.js:283` expects `utcDay` ticks that d3 7.9.0 no longer gives

**Date:** 2026-10-27 · **Context:** Phase 27, step 7 (212f5c9) · **Status:** decided

### Issue

d3-scale 4.0.2's test "scaleUtc.ticks(count) can generate 2-day ticks"
(`utcTime-test.js:281–289`) expects `[utc(2011, 0, 3), utc(2011, 0, 5),
utc(2011, 0, 7), utc(2011, 0, 9)]` for the domain 2011-01-02T16:28:27Z …
2011-01-09T21:34:12Z: days of the month ≡ 1 (mod 2), `utcDay.every(2)`.
The section's table and its spot row (`utc()`, ".ticks(4) | 01-02, 01-04,
01-06, 01-08") take d3-time 3.1.0's `unixDay` for the utc ticks, which
gives 4, 6, 8 January. The port gives 4, 6, 8, so the assertion fails.

### Investigation

- `d3-time/src/ticks.js:55` (3.1.0, vendored):
  `ticker(utcYear, utcMonth, utcSunday, unixDay, utcHour, utcMinute)`.
  d3-scale 4.0.2 accepts `"d3-time": "2.1.1 - 3"`
  (`d3-scale/package.json:39`); its test predates `unixDay`.
- d3 itself, the vendored d3-scale 4.0.2 over d3-time 3.1.0 in Node 22:
  `scaleUtc().domain([utc(2011, 0, 2, 16, 28, 27), utc(2011, 0, 9, 21,
  34, 12)]).ticks(4)` is 2011-01-04, 01-06, 01-08 at 00:00Z. The
  harness's own d3 run agrees: the suite reports 118 of 119 translated
  assertions passing d3's semantics.

### Options

| | **A — leave it failing** | B — skip it | C — `utcDay` for utc ticks |
| --- | --- | --- | --- |
| Approach | Translated, not in `passing.txt`; the floor is 56 of 85 | A skip category naming the d3-time version | `utc_tick_interval` takes `Day(k)` |
| Cost | none | a translator rule for one id | contradicts the section, its spot row and d3 7.9.0 |
| Correctness | the port equals d3 7.9.0 | hides a failing assertion | equals d3-scale's test, not d3 |
| Fit | the report shows it as failing, with d3 failing it too | against "a failing assertion is never skipped" | — |

### Decision

B (architect, 2026-10-27), with a category of its own. The harness
gains the skip prefix `upstream:` for an assertion that the vendored d3
itself fails when run in Node, and only for that; the reason names the
versions ("d3-scale 4.0.2's test expects utcDay; d3-time 3.1.0, which
d3 7.9.0 vendors, ticks by unixDay; d3 fails it"). `utcTime-test.js:283`
is the first. "0 failing" stays an invariant every phase's exit
criteria can rely on, and "a failing assertion is never skipped" keeps
its meaning for the port: an `upstream:` skip needs the Node run that
shows d3 failing, cited in the commit body. Done as the first commit of
Phase 28's session.

### Why

The port matches d3 as vendored and the section's spot row; the test is
stale against d3-time 3.1.0, which the harness's d3 run shows.

### Followups

- Phase 28 raises the floor of `utcTime-test.js` to 70 %: 56 + its 22
  tick formats is 78 of 85 (91.8 %), without this assertion.
- If d3-scale's test is updated upstream (d3-scale ≥ 4.0.3), revendoring
  makes it pass.

## D27.4 — `time-test.js` and `utcTime-test.js` under 70 %

**Date:** 2026-10-27 · **Context:** Phase 27, steps 6–7 (e8e797b, 212f5c9) · **Status:** decided

### Issue

Grading by d3's tests: a core phase's files pass ≥ 70 % unless its skip
table names them. Exit criterion 6 sets ≥ 60 % for these two, with the
rest "their `tickFormat` tests (Phase 28, which raises both floors to
70%) and listed skips (polylinear time domains, `domain([NaN…])`,
`.interpolate`, `.copy()` identity)".

| File | Assertions | Translated | Passing | Skipped |
| --- | --- | --- | --- | --- |
| `time-test.js` | 96 | 62 | 62 (64.6 %) | 34: gap 22 (tick formats), types 4 (polylinear 1; a range of colors with `.interpolate` 3), js 6 (`domain([-1e50, 1e50])` as NaN 3; an iterable domain 1; `instanceof` 1; a distinct copy 1), api 2 (`clamp()` getter) |
| `utcTime-test.js` | 85 | 57 | 56 (65.9 %) | 28: gap 22, types 4 (as above), api 2 (`clamp()` getter); 1 failing (D27.3) |

Beyond the listed skips: d3's `clamp()` getter (api, 2 per file; the
section's `Time` has the builder `clamp(bool)` and no getter, as
Phase 4's scales) and an iterable domain (js, 1; `domain` takes
`[Timestamp; 2]`).

### Investigation

`cargo run -p rustd3-harness` (the per-file report above);
`dev/harness/src/generated/time_scale.rs`. Each blessed id was read in
the diffs of e8e797b (40 ids) and 212f5c9 (78 ids): every one is d3's
value under America/Los_Angeles.

### Options

| | **A — floors at the passing counts** | B — raise to 70 % now |
| --- | --- | --- |
| Approach | Floors 62 and 56, as blessed; exit criterion 6's 60 % holds; Phase 28 adds the 22 tick formats per file | Needs tick formats (Phase 28) or skipping fewer constructs |
| Cost | none | Phase 28's work, or a `get_clamp` the section does not list |
| Fit | the section's own criterion | — |

### Decision

A (architect, 2026-10-27): Phase 27's skip table gains `time-test.js`
and `utcTime-test.js` with floors 62 and 56; Phase 28 raises them when
tick formats translate.

### Why

Every assertion missing below 70 % is a tick format (Phase 28), a listed
skip, a getter or input type the section's `Time` does not have, or
D27.3.

### Followups

- Phase 28: raise both floors once tick formats pass.

## D27.5 — Details of Phase 27b as built

**Date:** 2026-10-27 · **Context:** Phase 27, steps 6–10 (7d7022b … fc3d121) · **Status:** decided

### Issue

1. **Commits.** The section counts 11 commits for the phase; 27a made
   10 and 27b makes 11 (one for D27.1, a core and a harness commit for
   each of Steps 6 and 7, one each for Steps 8, 9 and 10's fuzz, the
   doc examples, the spot rows, the closing docs).
2. **`apply`'s blend.** Design: `apply = r0 + (r1 − r0)·(u / s)`. Built:
   d3's `interpolateNumber`, `r0·(1 − f) + r1·f`, through `Interpolate for
   f64`; it is exact at both ends and equal to d3 bit for bit. Every spot
   row gives the same value either way.
3. **`invert` beyond the formula.** A NaN `y` (d3: an Invalid Date)
   gives the domain's start; ±∞ saturate at `Timestamp::MIN`/`MAX`; a
   flat range gives the domain's middle, as d3's `normalize`.
4. **`Coordinate::is_valid`.** Design: `fn is_valid(self) -> bool { true
   } // every Timestamp is`. D9.2's followup and handoff 09: `is_valid`
   must reject what `at` cannot represent, because a regular clock checks
   only its last appended x. Built: every instant but `Timestamp::MAX`,
   where `at` saturates, so an explicit clock cannot hold jiff's last
   instant either.
5. **The fuzz timeout.** Exit criterion 10: "fuzzed under a 1 s timeout
   per case". Built: 1 s optimized (`RUSTD3_FUZZ=1`, 3,000 cases in
   release); 15 s per case unoptimized, as `check.sh quick` and `full`
   run it (150 cases), where 10,000 ticks of a stepped week take 1.4 s.
6. **Gestures at the edges.** `pan_px` on a flat range moves by an
   infinite fraction, which leaves jiff's range: `None`; `pan_px(0.0)`
   there reads the NaN fraction as 0 and returns the domain unchanged.
   `pad` saturates margins at jiff's first and last instants.
7. **Phase 9's suite "instantiated".** Phase 9's tests are `f64` test
   functions; built: their helpers (`xs`, `thinned_by_index`, `m4_scan`)
   are generic over `Coordinate`, and the `Timestamp` tests
   (`core/tests/main/series/timestamp.rs`) call them on the section's
   clocks.
8. **Doc examples** use POSIX zones (`TimeZone::posix`), so they need no
   time zone database; the tests use `testkit::zone`.

### Investigation

(2) `core/src/interpolate.rs:191–196`; d3-interpolate 3.0.1
`src/number.js:1–5`. (3) d3-scale 4.0.2 `src/continuous.js:12–16`
(`normalize`). (4)
`core/src/series/clock/timestamp.rs`; test
`instants_past_jiffs_range_and_empty_periods_are_rejected`. (5)
`core/tests/main/time/fuzz.rs`, `PER_CASE`; this session's release run
in the Measurements. (6) tests `degenerate_gestures_return_none`,
`pad_fits_dates_as_matplotlib`.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | 21 commits for the phase, each one logical change | 11, merging core and harness work |
| (2) | d3's blend | the Design's formula; the last bits differ from d3 for some ranges |
| (3) | Total and d3's where d3 has a value | — |
| (4) | `Timestamp::MAX` rejected | `true`: a regular clock past jiff's range would store saturated, repeated x |
| (5) | 1 s optimized, 15 s unoptimized | 1 s in every build: the debug runs of `check.sh` would fail on slow but correct cases |
| (6) | As described | — |
| (7) | Generic helpers, `Timestamp` tests | Phase 9's test functions made generic |
| (8) | POSIX zones in examples | — |

### Decision

A for all eight (architect, 2026-10-27), as built.

### Why

(2) and (3) follow d3 where the section is silent or writes the same
value another way; (4) is D9.2's decided followup; (5) keeps the 1 s
bound where it measures the code rather than the build; (1), (6)–(8)
change no API.

### Followups

- If B for (4): `is_valid` returns `true` and `Recording::append` needs
  another way to refuse a saturated x.
