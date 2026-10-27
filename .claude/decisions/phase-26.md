# Phase 26 — Number locales and `fmt!`

## Measurements

2026-10-27, 697500d … 68fedf6, one session. `scripts/check.sh full`
green at 68fedf6 in 9 min 16 s (warm, the benchmark smoke included).

**Harness**, before (8ba94eb) and after (48e4b66): 10 suites, 4,562
assertions (unchanged), 3,529 → 3,580 passing, 1,033 → 982 skipped,
116 floors (two raised). Every translated assertion passes, under d3's
semantics too. No `gap:` skip is left in the format suite.

| File | Before | After | Share | Skips after |
| --- | --- | --- | --- | --- |
| `d3-format/test/arabicLocale-test.js` | 0 / 27 | 27 / 27 | 100 % | — |
| `d3-format/test/locale-test.js` | 18 / 45 | 42 / 45 | 93 % | js 3 (`format(undefined)`) |
| `d3-format/test/defaultLocale-test.js` | 2 / 6 | 2 / 6 | 33 % | js 4 (D3.1, D26.2) |

Floors raised: `arabicLocale-test.js` 0 → 27, `locale-test.js` 18 → 42.
The 51 new ids are the 27 ar-* assertions, the 20 of the en-IN block and
the four of the "locale data is valid" loop.

**Translator**: +70 / −20 lines in `rules/format.mjs` and `expr.mjs`
(+60 / −17 without comments and blank lines), within the budget of 100.

**Generator**: `dev/harness/gen/format_locales.mjs`, 105 lines; it
writes 240 lines of Rust (eleven constants and `locales.rs`) and
`dev/fixtures/format_locales.json` (13 locales × 6 probes). Every
spot-check row of the section equals d3-format 3.1.2's output (the
fixture), en-IN and ar-001 included.

**`fmt!`**: the two `compile_fail` doctests, compiled once with rustc
1.97 against the built crate, fail with `error[E0080]: evaluation
panicked: invalid d3 format specifier: 10..f` and `…: 536870889`
(`{^10..f` too). `literal_check_agrees_with_parser` covers the 1,372
double-quoted string literals of d3-format's 24 test files (733 of them
valid specifiers under d3's pattern), 16 fill and width edges, and all
292,561 strings of length ≤ 4 over the section's 23-symbol alphabet.

## D26.1 — The parser accepts a fill outside the BMP; d3's pattern does not

**Date:** 2026-10-27 · **Context:** Phase 26, step 3 (936e299) · **Status:** open

### Issue

"Decisions this phase may meet": if the parser deviates from d3's regex,
changing it is a change to a Phase 3 item, recorded with every call site
updated. The agreement test finds `is_valid` and `Specifier::from_str`
equal everywhere, and both accept a fill of one character of any length,
as the section asks ("decode its length from the first byte …
`11110xxx` 4"). d3's pattern does not: JavaScript's `.` matches one
UTF-16 code unit, so a character outside the Basic Multilingual Plane
(a surrogate pair) cannot be the fill.

### Investigation

Node 22, d3-format 3.1.2 (`formatSpecifier.js:2`):
`formatSpecifier("😀>10")` throws `invalid format: 😀>10`;
`formatSpecifier("é^10")` and `("€>8")` parse. A lone high surrogate
followed by `>1` matches, which a Rust `&str` cannot hold. Rust's parser
(`core/src/format/specifier.rs`, Phase 3) reads `char`s, so `"😀>10"`
parses with fill U+1F600. No d3 test uses such a fill. The other
non-ASCII cases agree with d3: U+212A, U+0131 and U+017F are not type
letters (the pattern's `i` flag folds no non-ASCII character into
`[a-z]`), and the four line terminators are not fills.

### Options

| | **A — keep: a fill is any one `char`** | B — d3's code units |
| --- | --- | --- |
| Approach | As built: parser and `is_valid` accept a 4-byte fill | Parser and `is_valid` reject a fill above U+FFFF, `InvalidFormat` |
| Cost | None | A change to Phase 3's `FromStr` (no signature change) and the 4-byte branch of `is_valid` |
| Correctness | Pads with the character; widths count `char`s, as everywhere in the port | d3's verdict exactly, a JavaScript encoding artefact |
| Fit | The section's own grammar text | "The parser ports d3's regex" literally |

### Decision

Recommended: A. The section's grammar asks for 4-byte fills, padding
counts characters in Rust, and nothing depends on rejecting them.

### Why

d3's refusal follows from UTF-16, not from a rule about fills; d3
accepts every other single character but the line terminators.

### Followups

If B: one commit changing `Specifier::from_str` and `is_valid`, with the
agreement test's edge list asserting `"😀>10"` invalid.

## D26.2 — `defaultLocale-test.js` stays under 70 %

**Date:** 2026-10-27 · **Context:** Phase 26, step 2 (48e4b66) · **Status:** open

### Issue

Exit criteria 4: floors of 70 % on `locale-test.js` (45),
`arabicLocale-test.js` (27) and `defaultLocale-test.js` (6). The first
two pass 42 and 27. `defaultLocale-test.js` passes 2 of 6, as decided in
D3.1: its other four assertions check that `formatDefaultLocale`
rebinds the module's `format` and `formatPrefix` (two by function
identity, two by calling them), which needs the mutable default locale
Conventions 5 rules out. D3.1's followup already says Phase 26 does not
change the file: it loads no JSON table.

### Options

| | **A — D3.1's row stands, floor 2** | B — reach 70 % |
| --- | --- | --- |
| Approach | No change; the floor stays at the passing count | A mutable default, or translating `format` as `locale.format` after the call (4 of 6, still under) |
| Correctness | Counts visible in the report and the floor | Forbidden global, or assertions that repeat test 1 |

### Decision

Recommended: A, D3.1's answer; the exit criterion's "(6)" is read as
the file's assertion count, whose floor D3.1 fixed at 2.

### Why

The shortfall is the decided absence of a mutable default.

### Followups

None.

## D26.3 — Details of Phase 26 as built

**Date:** 2026-10-27 · **Context:** Phase 26, 697500d … 68fedf6 · **Status:** open

### Issue

1. **Doc lines of the generated constants.** The section's sample
   documents `DE_DE` with a `///` line holding "1.234,50 €" and also
   says every non-ASCII character is emitted as `\u{…}`; a `///` line
   cannot escape U+00A0.
2. **The runtime spot rows.** en-IN and ar-001 are "(harness, runtime
   JSON)" rows; the harness has only `locale-test.js:43` and ar-001's
   `$,.2f` of −1234.56 among them.
3. **The freshness step as written.** `node --import
   dev/harness/translate/d3-resolve.mjs …` fails in Node 22
   (`ERR_MODULE_NOT_FOUND`: a path without `./` resolves as a package
   name); the file list omits `core/src/format/locales.rs`, which the
   generator writes; and an uncommitted hand edit is overwritten by the
   rerun, after which `git diff --exit-code` is clean, so the step's
   verification ("a deliberate one-character edit … makes `full` fail")
   does not hold for an edit that is not committed.
4. **Translator.** The statement hook can only throw a skip; the loop
   over 59 tables needs to emit lines. "Bundled" needs a list of the
   eleven. "The loop … counts as its single assertion": d3's loop body
   holds four assertion calls.
5. **`MAX_WIDTH`** is private to `specifier.rs`; `is_valid` needs the
   same bound.
6. **The notation label** (the architect's follow-up of D17.1 (7)):
   how it is written in the locale from the figure crate, which sees
   only `format::Locale`'s public API.
7. **`f` at 10²¹ and above.** A degenerate row found `$,.0f` of 1e21
   under JA_JP writing "1,000,000,000,000,000,000,000円" where d3 writes
   "1e+21円" (JavaScript's `toFixed` switches to exponent notation).
8. **Commits**: the section names five.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | A `#[doc = "…"]` attribute whose string escapes them (`1.234,50\u{a0}\u{20ac}`), the values from d3's own output; `locales.rs`'s table (a private module) writes the escapes as text; the generated files say `Locale`, not `Self` | `///` with raw U+00A0, invisible in a diff |
| (2) | The fixture holds en-IN and ar-001 too (`constant: null`); `runtime_number_locales_match_d3` builds them from their JSON with serde_json, beside `bundled_number_locales_match_d3` for the eleven | Only the harness's assertions |
| (3) | `fresh`, first in `full`: `git diff --exit-code HEAD` on the generated files before and after `node --import ./dev/…` reruns both generators, and no untracked generated file; the list adds `locales.rs`. Verified with one-character edits to `de_de.rs` and `categorical.rs`, uncommitted, and to `de_de.rs` committed (then undone) | Fails to run; misses uncommitted edits |
| (4) | The hook may return a statement's Rust lines (`expr.mjs`, +6 / −3); `idOf` exported; a table is bundled when `core/src/format/locales/<xx_yy>.rs` exists (no d3 test uses one, so the constant branch is exercised only through `bundled_locales_equal_their_json`'s equality); the loop's four assertions are four ids, each recorded once per table | — |
| (5) | `pub(super) const MAX_WIDTH`, read by `literal.rs` | A second constant |
| (6) | `Notation::label(locale)`: the shortest mantissa through `+.{digits}r` and the exponent through `d`, with `Locale::format`; decimal mark, minus and numerals apply ("+1,2345e4" under DE_DE, "1e-7" under a hyphen minus, "+١٫٢٣٤٥e٤" with Arabic-Indic digits); neutral and en-US labels unchanged | — |
| (7) | Kept: Phase 3's "no emulation of JavaScript's exponent notation" (Not in this phase), asserted as the port's value in `degenerate_values_under_the_tables_match_d3`; the other 16 degenerate rows equal d3 | Port `toFixed`'s switch at 10²¹ (and for `%`) |
| (8) | Six code commits and one docs commit: 697500d (tables), 48e4b66 (harness), 936e299 (`fmt!`), cd79857 (freshness), 9a862dd (`fix(figure)`, the notation label), 68fedf6 (degenerate and Part I checks) | Five |

### Decision

Recommended: A for all eight.

### Why

(1), (3) and (5) are the forms that compile, run and keep the section's
intent; (2) checks the section's own spot rows; (4) adds the smallest
hook the loop needs and avoids a second list of bundled tables; (6)
uses only core's public API, as the figure crate must; (7) follows
Phase 3's stated rule and only records where d3 differs; (8) keeps the
architect's figure follow-up its own commit.

### Followups

- Phase 28: append the time-locale generator to `fresh` and its files to
  `GENERATED` in `scripts/check.sh`; `locale::Locale::{DE_DE, …}` pair
  these number constants with the time tables.
- Phase 40: the facade re-exports `rustd3_core::fmt` (a
  `#[macro_export]` macro lives at the crate root).
