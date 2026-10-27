# Phase 28 — Time II: formats, locales and time ticks

## Measurements

2026-10-27, bdcb912 … 54cb28a and the closing docs commit, one session:
the section's cut after Step 2 was not needed. `scripts/check.sh full`
green at 54cb28a in 10 min 15 s (warm, the benchmark smoke included).

**D27.3 and D27.4, done first.** bdcb912 adds the skip category
`upstream:` (documented in `expr.mjs` beside js, types, api, gap and
helpers; ids listed in its `UPSTREAM` with the versions) and makes
`utcTime-test.js:283` the first, after the vendored `utcTime-test.js`
run in Node 22 under a minimal `it` failed exactly that one test (45
tests, 1 failing, line 283). The harness has reported 0 failing since.
03fa49e raises the two floors D27.4 left (below).

**Harness**, before (e8f2904) and after: 12 → 13 suites, 5,879 → 6,411
assertions, 4,794 → 5,100 passing, 1,084 → 1,311 skipped, 1 → 0
failing, 153 → 161 floors. Every translated assertion passes, under d3's
semantics too; d3 itself passes every assertion of the five graded
d3-time-format files run in Node (format 33 tests, utcFormat 36,
isoFormat, isoParse, locale 1 each).

| File | Before | After | Share | Skips after |
| --- | --- | --- | --- | --- |
| `d3-time-format/test/format-test.js` | — | 122 / 137 | 89 % | js 7 (numbers as dates), helpers 7 (`multi()`), types 1 (year 10002) |
| `d3-time-format/test/utcFormat-test.js` | — | 132 / 140 | 94 % | helpers 7 (`multi()`), types 1 (year 10002) |
| `d3-time-format/test/isoFormat-test.js` | — | 2 / 2 | 100 % | — |
| `d3-time-format/test/isoParse-test.js` | — | 3 / 3 | 100 % | — |
| `d3-time-format/test/locale-test.js` | — | 1 / 1 | 100 % | — (the loop over the 35 tables) |
| `d3-time-format/test/defaultLocale-test.js` | — | 2 / 13 | listed | api 11 (`timeFormatDefaultLocale`, parsing) |
| `d3-time-format/test/parse-test.js` | — | 0 / 139 | listed | api 138 (parsing, Later), types 1 (D28.2 (9)) |
| `d3-time-format/test/utcParse-test.js` | — | 0 / 97 | listed | api 97 (parsing, Later) |
| `d3-scale/test/time-test.js` | 62 / 96 | 84 / 96 | 87.5 % | types 4, js 6, api 2 |
| `d3-scale/test/utcTime-test.js` | 56 / 85 | 78 / 85 | 91.8 % | types 4, api 2, upstream 1 |

Floors are each file's passing count (8 new, 2 raised). The section's
283 assertions (format 137, utcFormat 140, isoFormat 2, isoParse 3,
locale 1) hold as counted.

**Translator**: Phase 28 +123 / −5 lines of its 200
(`rules/time_format.mjs` 109; `rules/time.mjs` +4 / −3, exporting
`civil`, `date` and `zone` and skipping years outside jiff's range;
`translate.mjs` +10 / −2, the suite entries); D27.3's `expr.mjs` +11 / −2
besides.

**Generator**: `dev/harness/gen/time_locales.mjs`, 88 lines; it writes
240 lines of Rust (eleven constants and `locales.rs`) and
`dev/fixtures/time_locales.json` (11 locales × 28 probes: seven
specifiers on four Los Angeles instants). Every (d3) spot row of the
section equals d3-time-format 4.1.0's output in the fixture; a rerun
writes the same bytes, and `fresh` fails on a one-character edit of
`de_de.rs` left uncommitted (checked with `fresh` alone).

**Core**: `core/src` +1,296 / −16 lines (240 generated; doc examples
included), `core/tests` +559 / −1 before the closing commits.

**Against d3** (the architect's check): `Ticks for Time` beside d3's
`scaleTime().ticks(n)` and `tickFormat()` in Node (d3-scale 4.0.2,
d3-time 3.1.0, d3-time-format 4.1.0), each under `TZ=America/Los_Angeles`
and `TZ=Europe/Berlin`, with en-US and with de-DE as d3's default locale
(the port: `time::Locale::new` over de-DE.json with `Ladder::D3`): the
days of 25 October and 1 November 2026, three minutes and 90 ms across
midnight, two months, a year and a quarter and twelve years, at counts
10 and 5. All 56 rows give d3's instants and labels; five of them are
asserted in `spans_tick_and_label_as_d3s_scale_time`. The neutral
ladder's labels for the same rows follow the section's ISO rows.

**Zones**: the time tests pass under `TZ=Asia/Kathmandu` and, by hand,
`TZ=UTC`.

**"Decisions this phase may meet"**: not met. No d3 format assertion
depends on a gap or fold that jiff resolves differently from `Date`.

## D28.1 — A runtime time table has no way into the figure locale

**Date:** 2026-10-27 · **Context:** Phase 28, step 5 (899319e) · **Status:** open

### Issue

Part I, Formats, time and color: "A ladder is not a second knob beside
`fig.locale`: other tables come from `time::Locale::new(time::Definition
{ …, ladder: time::Ladder::D3 })`." Phase 28's Design gives
`locale::Locale` the private field `time`, the reader `time()` and the
twelve constants; D11.2 item 5 gave it `From<format::Locale>`, whose time
half is neutral. No public item builds a `locale::Locale` from a
`time::Locale`, so a runtime time table (the 24 of d3's 35 without a
constant, or a user's) formats through `time::Format` and
`Time::tick_label` but cannot reach an axis's labels through
`Ticks::labels` or `fig.locale`.

### Investigation

`core/src/locale.rs` at 899319e: `Default`, `From<format::Locale>`, the
constants, `number()`, `time()`; the struct's fields are private.
`Figure::locale(&mut self, l: Locale)` (`figure/src/figure.rs:238`) takes
the core type, so a constructor added to core reaches `fig.locale`
unchanged.
Nothing in Phase 28 depends on the answer; Phase 33 (time handles) is the
first to draw time axes.

### Options

| | A — as built | **B — `Locale::new(number, time)`** | C — `From<time::Locale>` |
| --- | --- | --- | --- |
| Approach | Twelve constants and `From<format::Locale>` only | `pub fn new(number: format::Locale, time: time::Locale) -> Locale` beside the `From` | The time half's twin of D11.2 item 5, the number half neutral |
| Cost | none | one function, one test | one impl, one test |
| Correctness | runtime time tables label nothing a figure draws | any pair of tables | a runtime time table with neutral numbers only |
| Fit | Part I's "other tables come from `time::Locale::new`" has no path to the figure | the constants' pairing as a function, `format::Locale::new`'s shape | symmetric with the existing `From`; cannot pair two runtime tables |

### Decision

Recommended: B. Not built: it is a public item the section does not list.

### Why

Part I promises runtime time tables to figures, and only B gives every
pair of tables the constants give; C covers half of it.

### Followups

- If B or C: one `feat(core)` commit with its test, before Phase 33.

## D28.2 — Details of Phase 28 as built

**Date:** 2026-10-27 · **Context:** Phase 28, steps 0–6 (bdcb912 … 54cb28a) · **Status:** open

### Issue

1. **Steps and commits.** The section names seven commits in Step
   order. With `Locale` holding its ladder from Step 1, the field would be
   unread (dead code under `-D warnings`) until Step 4's `tick_label`.
2. **NEUTRAL's composites.** "`%c %x %X` as `%F %T`, `%F`, `%T`": those
   are jiff directives, which d3's table (and so the port) writes as
   their letters.
3. **A composite inside its own expansion** (a runtime table whose `%c`
   pattern holds `%c`, or `%x` and `%X` that refer to each other): d3
   recurses without end (a `RangeError`).
4. **Week numbers and ISO years at jiff's edges**: `%U %W %V %g %G %j`
   floor and ceil through intervals, which return `None` past jiff's
   range (years −9999 and 9999 only).
5. **A tick whose floor leaves jiff's range** in `tick_label`'s rung
   choice.
6. **Generated layout.** The generator's arrays exceed rustfmt's widths,
   and `cargo fmt --check` covers `core/src`.
7. **The fixture's probes.** "`%c`, `%x`, `%X` and `%a %A %b %B %p` on
   four probe instants", while exit criterion 5 checks every (d3) spot row
   from the fixture, among them `%x %X`, `%a %d %b` and `%B %b`.
8. **Translator details.** `parse-test.js` has one assertion inside an
   `Object.prototype` write that skips as `types: a field write` rather
   than "parsing (Later)"; `defaultLocale-test.js` (listed: 13) translates
   its two format defaults, which pass; `Date.UTC(…)` translates (2
   lines) so that one does; `timeFormatLocale` over a vendored table uses
   `Ladder::D3`, d3's own.
9. **`iso_parse`.** "jiff's RFC 3339 parser": jiff 0.2.38 has no
    separate RFC 3339 parser; `Timestamp::from_str` (its Temporal parser)
    accepts RFC 3339 and requires an offset or `Z`.
10. **Readers.** The section lists no getters on `time::Locale`; the
    format reads its patterns, names and ladder through crate-private
    readers.
11. **`From<format::Locale>`** keeps the neutral time half (D11.2), so
    `Locale::from(format::Locale::EN_US)` no longer equals
    `Locale::EN_US`; Phase 17's test asserted they did.
12. **The test names** `berlin_example`, `sub_second_labels` and
    `time_locales_match_d3` are the section's, not sentences.

### Investigation

(1) D27.2 (2) merged steps for the same reason. (3) Node 22, d3-time-format
4.1.0: `timeFormatLocale({dateTime: "%c", …}).format("%c")(date)` throws
`RangeError: Maximum call stack size exceeded`. The section's `%Z`
"truncated to minutes" needs no entry: V8 gives `getTimezoneOffset()`
472 for local 1800-01-01 in Los Angeles (LMT −07:52:58) and d3 writes
`-0752`, as the port does
(`offsets_east_of_utc_and_in_minutes_are_signed_hours_and_minutes`).
(6) The harness's `#[rustfmt::skip] mod generated;` is the precedent. (8)
`parse-test.js:294–300`. (9) `jiff-0.2.38/src/timestamp.rs:2353–2360`.
(11) `a_number_locale_makes_a_figure_locale`, now comparing number
halves.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | Steps 1 and 4's core in one commit (da8817c: `Locale`, `Ladder`, `Format`, every directive with `%N`, `tick_label`); Step 2's ISO helpers alone (6d224b5); a core and a harness commit per step, D27.3 first, the freshness step as `build:` (9e63997), a style fix: 12 commits with the closing docs | Seven, with a dead field or a dead reader in Step 1 |
| (2) | `"%x %X"`, `"%Y-%m-%d"`, `"%H:%M:%S"`: the meanings in d3's directives | Patterns that print `F T` |
| (3) | The composite writes its letter (`%c` → `c`) | — |
| (4) | `NaN`, as d3 writes an Invalid Date's fields | — |
| (5) | Counts as aligned (the coarser rung) | — |
| (6) | `#[rustfmt::skip] mod locales;` in `time.rs`; the generator's layout is the source's | Generated rustfmt layout, or the generator calling rustfmt |
| (7) | The fixture also holds `%x %X`, `%a %d %b` and `%B %b`: seven specifiers, 28 probes per locale | Four specifiers; three spot rows asserted without d3's output beside them |
| (8) | As described | — |
| (9) | `s.parse::<Timestamp>()` | — |
| (10) | `pub(crate)` readers in `time/locale.rs` | Public getters the section does not list |
| (11) | The test compares number halves | — |
| (12) | The section's names | Sentences |

### Decision

Recommended: A for all twelve, as built.

### Why

(1), (6) and (10) keep the section's API and `-D warnings`; (2), (7),
(9) and (11) are the closest forms that hold with the section's other
statements; (3)–(5) choose total behaviour where d3 has none; (8) and
(12) change no API.

### Followups

- If B for (6): the generator pipes its output through `rustfmt` and
  `fresh` depends on the rustfmt version.
