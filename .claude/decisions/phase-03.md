# Phase 3 — First port: path, ticks, format, ease

## D3.1 — Two d3-format test files pass under 70% and are not in the skip table

**Date:** 2026-10-27 · **Context:** Phase 3, step 5 (feat(harness): grade path, ease, ticks and format) · **Status:** decided

### Issue

rustd3.md, Phase 3, Exit criteria 4: "every file passes at least 70 % of
its assertions except those in the skip table". Its skip table names
`locale-test.js`, `arabicLocale-test.js` and `formatSpecifier-test.js`.
"Decisions this phase may meet": a translated file not in the skip table
under 70 % is recorded with its counts and reason, not blessed silently.
Two files of the `format` suite are under 70 % after translation:

| File | Assertions | Translated / passing | Skipped | Share |
| --- | --- | --- | --- | --- |
| `d3-format/test/format-type-c-test.js` | 5 | 0 / 0 | types: 5 | 0 % |
| `d3-format/test/defaultLocale-test.js` | 6 | 2 / 2 | js: 4 | 33 % |

- `format-type-c-test.js`: every assertion formats a string
  (`format("c")("☃")`, `formatLocale({decimal: "/"}).format("c")(".")`):
  d3's `c` type writes its argument as text. `Format::format` takes an
  `f64` (Phase 3, Design, `format`), so these are `types:` skips
  ("Format::format takes a number, not a string").
- `defaultLocale-test.js`: the two assertions on the locale
  `formatDefaultLocale(frFr)` returns translate (`Locale::new` over the
  fr-FR definition) and pass. The other four check that the call rebinds
  the module's `format` and `formatPrefix`: two by function identity
  (`assert.strictEqual(format, locale.format)`, `js: function identity`)
  and two by calling the rebound `format`/`formatPrefix` (`js:
  formatDefaultLocale rebinds the module's exports`). rustd3 has no
  mutable default (Conventions 5; the free functions use
  `Locale::NEUTRAL`, and the harness grades under `Locale::EN_US`).

### Investigation

- The prototype's run had both files at 0 % (`proto/poc/harness/src/generated.rs`:
  format-type-c 0 of 5 translated, "formatted value is not a number";
  defaultLocale 0 of 6, "setup: unsupported identifier frFr"), so the
  skip table predates neither: they were missed.
- `formatSpecifier-test.js` is in the table "only if field writes
  (`types:`) dominate": it translates 27 of 51 (53 %), skipping `types:`
  13 (field writes) and `js:` 11 (10 `new FormatSpecifier({…})` coercions,
  one `instanceof`), so field writes are its largest category and the
  entry applies.
- Every other owned file passes at least 79 %: the lowest are
  `poly-test.js` (54 of 68), `linear-test.js` (11 of 13), the other
  ease files (34 of 40, `elastic-test.js` 62 of 70), `pathRound-test.js`
  (12 of 14), `precisionPrefix-test.js` (14 of 16) and `format-test.js`
  (12 of 13), then `path-test.js` (62 of 65); every other file passes
  all of its assertions.

### Options

| | **A — add both to the skip table** | B — reach 70 % | C — drop both files from the suite |
| --- | --- | --- | --- |
| Approach | Skip-table rows: format-type-c, "`c` formats strings; `Format::format` takes `f64` (`types:`)"; defaultLocale, "rebinding the module's default locale (`js:`)". Floors stay at 0 and 2 | format-type-c needs a string-formatting entry point (an API beyond the spec); defaultLocale needs a mutable default (forbidden), or a translation that rewrites `format` to `locale.format` after `formatDefaultLocale`, which reaches 4 of 6 (67 %, still under) and only repeats test 1 | Remove the two files from the `format` suite's list |
| Cost | Two table rows | An API addition or a global; still short of 70 % for defaultLocale | Contradicts "the 24 d3-format files" |
| Correctness | Counts and reasons stay visible in the report and floors | Inflates counts with assertions that test nothing Rust has | Hides 11 assertions |

### Decision

A (architect, 2026-10-27). Phase 3's skip table gains two rows, which
this entry carries in place of an edit to rustd3.md:
`d3-format/test/format-type-c-test.js`, "`c` formats strings;
`Format::format` takes `f64` (`types:`)"; and
`d3-format/test/defaultLocale-test.js`, "rebinding the module's
default locale (`js:`)". Their floors stay at 0 and 2.

Recommended: A. Floors are blessed at the current counts
(`format-type-c-test.js` 0, `defaultLocale-test.js` 2) so the ratchet
holds the two passing assertions either way; the skip table itself is
the architect's to amend.

### Why

Both shortfalls follow from choices the spec makes on purpose (an `f64`
formatter, no mutable default locale), not from the port; every
translated assertion passes.

### Followups

- Phase 26 (number locales) does not change either file: neither loads
  JSON tables.

## D3.2 — Three statements of Phase 3 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 3, steps 2–3 · **Status:** decided

### Issue

1. Phase 3, Design, `ticks`: `#[doc(alias = "ticks")] pub fn ticks`.
   rustc rejects it, "`#[doc(alias = "ticks"]` is the same as the item's
   name" (Rust 1.97.0, checked by adding it to `core/src/ticks.rs`).
2. Phase 3, Design, `format`: `pub(crate) fn group_from(self, digits:
   usize) -> Locale` on `Locale`. Its only caller is Phase 6's per-axis
   rule (`Format::format_all`); in Phase 3 it is dead code, which the
   workspace's `-D warnings` and the dispatch rules ("no dead code")
   reject.
3. Phase 3, Design, `ticks`: "the count is doubled once for 0.5 ≤ count
   < 2 when no tick fits". d3's tickSpec recurses while the condition
   holds, so a count in [0.5, 1) doubles twice; d3's own test
   `ticks(1, 364, 0.5)` → `[200]` needs both doublings (0.5 → 1 → 2).

### Investigation

See the Issue; the port follows d3 for 3 (`core/src/ticks.rs`, the
recursion in `Spec::new`), which all 92 assertions of
`d3-array/test/ticks-test.js` pass.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| Approach | No alias on `ticks` (the name is already the search term); `group_from(digits)` arrives with Phase 6's caller; tickSpec doubles as d3 does | Alias, a dead `group_from`, one doubling |
| Correctness | Compiles; d3's tests pass | Does not compile; fails `ticks-test.js:113` |

### Decision

A (architect, 2026-10-27): `ticks` carries no alias; tickSpec doubles
the count as d3 does, while 0.5 ≤ count < 2 and no tick fits; Phase 6
adds `Locale::group_from(digits)` together with its caller.

Recommended: A, already built; Phase 6 adds `Locale::group_from(digits)`
with `Format::format_all`.

### Why

Each written form is either rejected by the compiler or contradicts
d3's test the spec grades by.

### Followups

- Phase 6: add `pub(crate) fn group_from(self, digits: usize) -> Locale`.
