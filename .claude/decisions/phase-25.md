# Phase 25 — Binned scales and array statistics

## Measurements

2026-10-27, 604a750 … 855463a, the session that also did Phase 24.
`scripts/check.sh full` green at 855463a in 9 min 30 s (warm, the
benchmark smoke included), no warnings.

**Harness**, before (91592f4) and after (855463a): 9 → 10 suites (the
new `array` suite; the binned scales join `scale`), 3,822 → 4,562
assertions, 3,040 → 3,529 passing, 782 → 1,033 skipped, 97 → 116
floors. Every translated assertion of the new files passes, and passes
d3's semantics too (`cargo run -p rustd3-harness` reports "N pass d3's
semantics" equal to the passing count for each).

| File | Passing / assertions | Share | Skips |
| --- | --- | --- | --- |
| `d3-array/test/count-test.js` | 4 / 7 | 57 % | js 2, helpers 1 |
| `d3-array/test/min-test.js`, `max-test.js`, `extent-test.js` | 26 / 46 each | 57 % | js 20 each |
| `d3-array/test/sum-test.js` | 22 / 42 | 52 % | js 20 |
| `d3-array/test/mean-test.js` | 24 / 38 | 63 % | js 14 |
| `d3-array/test/variance-test.js` | 28 / 30 | 93 % | js 2 |
| `d3-array/test/deviation-test.js` | 29 / 29 | 100 % | — |
| `d3-array/test/median-test.js` | 29 / 53 | 55 % | js 18, api 6 |
| `d3-array/test/quantile-test.js` | 42 / 72 | 58 % | api 21, js 9 |
| `d3-array/test/range-test.js` | 77 / 77 | 100 % | — |
| `d3-array/test/bisect-test.js` | 62 / 77 | 81 % | js 14, api 1 |
| `d3-array/test/bin-test.js` | 15 / 36 | 42 % | api 11, js 8, gap 1, helpers 1 |
| `d3-array/test/threshold/{sturges,scott,freedmanDiaconic}-test.js` | 3 / 3, 4 / 4, 4 / 4 | 100 % | — |
| `d3-scale/test/quantize-test.js` | 28 / 51 | 55 % | js 16, api 3, types 2, gap 2 |
| `d3-scale/test/quantile-test.js` | 28 / 48 | 58 % | js 11, api 8, types 1 |
| `d3-scale/test/threshold-test.js` | 12 / 31 | 39 % | js 19 |

**Translator**: +231 / −3 lines without comments and blank lines
(`rules/stats.mjs` new, `rules/binned.mjs` new, `translate.mjs` and one
line of `expr.mjs`), within the budget of 250.

**Spot checks**: every row of the section, in
`core/tests/main/{array,binned}.rs` and the doc examples; the d3 values
were recomputed with d3 7.9.0 in Node through the resolve hook, with
d3's results for infinite, NaN-only and empty data under `bin()`.

## D25.1 — The section's listed decision

**Date:** 2026-10-27 · **Context:** Phase 25, "Decisions this phase may meet" · **Status:** decided

### Decision

Not met. d3's expected `undefined` is told from NaN through `Obs`: the
`Option` returns observe as null, and the translator writes d3's
`[undefined, undefined]` and `[undefined, 1/3]` as `[Option<f64>; N]`
(`extent`'s and `Threshold::invert_extent`'s results are compared as
such), so no assertion is skipped for it. `Bucket` keeps NaN edges as
designed.

## D25.2 — Fifteen files pass under 70 %

**Date:** 2026-10-27 · **Context:** Phase 25, steps 1–5 (5fb4dd7 … 855463a) · **Status:** decided

### Issue

Exit criteria 2 lists every file at 70 % (bisect at 50 %). Fifteen pass
less, each with every translated assertion passing (table above). The
skips, by reason:

- **Strings** as values or domains (`js:`, the section's "string-valued
  inputs"): min, max and extent 14 each (6 all-string, 8 mixed), sum 16,
  mean 8, median 12, count 1, bin 1, quantile's `p = "no"` 1; quantize 5
  (a string domain), threshold 6, the quantile scale 1.
- **Objects** (`js:`, "iterables of objects with valueOf"): `{valueOf}`
  objects, `{}` elements and `OneTimeNumber` in min, max and extent 4
  each, sum 2, mean and median 4 each; objects as arbitrary range values
  compared by identity in quantize 10, the quantile scale 6, threshold 10.
- **Iterables, generators and Dates** (`js:`, `helpers:`): `new Set` in
  count, quantile, quantize, the quantile scale (2) and threshold;
  generators in count and bin (`helpers:`); quantile's Date test (6,
  strings included).
- **Accessors with more than one argument** (`js:`, "accessor-function
  inputs"): the `(d, i, array)` and `this` tests, 2 per reduction file.
- **d3 API the design leaves out** (`api:`): `quantileIndex` 21 and
  `medianIndex` 6 (the section's "quantileIndex"); `unknown()` on the
  binned scales (quantize 3, quantile 8: `apply` returns `None`); bin's
  getters, value accessors and function domains or thresholds (11:
  `Thresholds` is a closed set); the `bisect` and `histogram` aliases.
- **Other `js:`**: sparse arrays at 2³⁰ (bisect 14), array holes as a
  range (`[,,,,,]`, 2), `null` coerced to 0 (quantile, threshold), no
  argument (threshold), a CSV read (bin), captured variables without a
  value in bin's function tests (6).
- **`gap:`, named**: bin's `for…of` loop (1) and quantize's `forEach`
  over a function (2), constructs outside the translator's budget. The
  loop would fail in our port from n = 10,000: `ticks(1, 2, n)` returns
  no ticks past Phase 3's 10,000-tick cap.

### Investigation

Raising any file over 70 % takes translating JavaScript coercion
(strings as numbers, `valueOf`, `+null`), object identity, d3's getters
and `unknown`, or `quantileIndex`, which the design leaves out (D4.2,
D7.1, D23.2 for the precedents). The `.map(box), unbox` accessor form,
objects declared in a test, null and undefined elements, `lo`/`hi`
bisection and nested arrays are translated, which is what lifted
deviation, variance, range and bisect to 81–100 %.

### Options

| | **A — record and bless at the passing counts** | B — reach 70 % |
| --- | --- | --- |
| Approach | Skip-table rows for the fifteen files with the reasons above; floors 4, 26, 26, 26, 22, 24, 29, 42, 15, 28, 28, 12 (and 28, 29, 62 for variance, deviation and bisect, above 70 %) | Coercion, identity, getters and `unknown` translated or added |
| Correctness | Counts and reasons stay visible; every passing assertion held by the ratchet | Against the naming rule and the typed API |

### Decision

A (architect, 2026-10-27). Phase 25's skip table gains the fifteen
files listed above with their floors at the passing counts; every
shortfall is JavaScript coercion or identity, d3 getters and index
functions outside v0.1, or a named `gap:`.

### Why

As D4.2, D7.1 and D23.2: every shortfall follows from JavaScript
coercion or identity, or from d3 API the design leaves out; no
translated assertion fails.

### Followups

None.

## D25.3 — Details of Phase 25 as built

**Date:** 2026-10-27 · **Context:** Phase 25, 604a750 … 855463a · **Status:** decided

### Issue

1. **Step 0.** "Commit before Step 1": the rules are committed first,
   wired into the scale suite with no file using them (the generated Rust
   unchanged); each step adds its files, regenerates and blesses (D23.3
   (1)'s order otherwise).
2. **`median` in Step 2**, not Step 1: it is `quantile(values, 0.5)`.
3. **Where `Bin` lives.** Layout: `array/bin.rs` holds "Bin, bin()";
   `array::bin` is also the public module of `Thresholds`, `Bucket` and
   the rules, so `Bin` there would have two public paths (`array::Bin`,
   `array::bin::Bin`), against "one public path per item".
4. **Private fields beyond the listed ones**: `Quantize` keeps its inner
   edges, as `Quantile` does (d3's `rescale`).
5. **`Quantize::invert_extent` with one range value**: d3 returns `[x0,
   undefined]` (its `i < 1` branch reads `domain[0]` of an empty array).
6. **`range` lengths of 2³¹ and more**: d3's `| 0` wraps them or throws;
   the section covers infinite and NaN lengths only.
7. **The ticks' step**: "the smallest gap" when values repeat (a
   quantile sample's equal quantiles) or there is one value.
8. **Labels of uneven edges**: `,f` to `precision_fixed(step)` gives
   `quantile(sample, [0, 1, 2, 3])`'s edges 3, 7.25, 9, 14.5, 20 (step
   1.75) the labels 3, 7, 9, 15, 20.
9. **The typed-array rule** (`Float64Array`/`Uint8Array` literals): no
   translated file uses one.
10. **Translator additions** beyond the listed rules: `[…].map(box)`
    with a file-level one-property accessor; an object declared in a test
    skips only the assertions that use it; nested number arrays; `Math.f(x)`
    as `f64::f(x)` (a literal argument needs a type); sparse arrays and
    file reads as named `js:` skips.
11. **Exit criterion 1's command** (`cargo test -p rustd3-core array::
    scale::quantize …`): cargo takes one test name before `--`.
12. **The bisect floor at 50 %**: `bisect-test.js` has no `bisector`
    tests (those are `bisector-test.js`, not translated).

### Options

| | **A — as built** |
| --- | --- |
| (1) | Rules (604a750), then each step's files with its implementation |
| (2) | Step 2 (c93c136) |
| (3) | `Bin` and `bin()` in the private module `array/histogram.rs` (d3's own name for the generator), re-exported as `array::{Bin, bin}`; `array/bin.rs` holds the rest, as in D11.2 (6) for `scale::Transform` |
| (4) | `thresholds: Vec<f64>` recomputed by `domain` and `range`; `thresholds()` returns a copy as specified |
| (5) | `[x0, x1]`: the bin is the whole domain |
| (6) | Empty, without allocating; the doc comment says so |
| (7) | The smallest positive finite gap; NaN for none, which formats at `,f`'s default precision |
| (8) | As written (d3's tick format at the step); Phase 35 may label colorbars of uneven edges otherwise |
| (9) | Not written: it would be dead code |
| (10) | In `rules/stats.mjs`, within the budget |
| (11) | `cargo test -p rustd3-core -- array:: binned:: scale::quantize scale::quantile scale::threshold` (the integration tests are `core/tests/main/binned.rs`): 1 unit, 25 integration and 4 doc tests pass |
| (12) | 62 / 77 (81 %), floor 62 |

### Decision

A for all twelve (architect, 2026-10-27), as built. Quantile and
threshold labels follow the section's rule for now; Phase 35, which
draws their legends and colorbars, decides a rule that keeps uneven
edges distinct (7.25 must not read "7").

### Why

(1), (2), (9) and (11) follow the compiler, cargo and the commit rule;
(3) keeps the specified public paths and the one-path rule; (4) is
D23.3 (10)'s precedent; (5) and (6) answer where d3 has no meaningful
result; (7) keeps a zero gap from setting six places; (8) and (12) are
the section's rules applied as written.

### Followups

- Phase 35: colorbar labels for binned scales with uneven edges (item 8).
