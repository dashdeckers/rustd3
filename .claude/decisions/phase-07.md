# Phase 7 — Categorical scales and basic color

## Measurements

2026-10-27, at c218bd9. Harness counts, translated = passing (every
translated assertion also passes under d3's semantics):

| File | Before | After | Share | Skips after |
| --- | --- | --- | --- | --- |
| `d3-scale/test/linear-test.js` | 153 / 247 | 171 / 247 | 69 % | js 34, types 32, api 10 |
| `d3-scale/test/pow-test.js` | 155 / 244 | 161 / 244 | 66 % | js 27, types 32, api 24 |
| `d3-scale/test/log-test.js` | 121 / 171 | 126 / 171 | 74 % | js 23, types 7, api 9, gap 6 |
| `d3-scale/test/symlog-test.js` | 39 / 71 | 39 / 71 | 55 % | js 23, types 3, api 6 |
| `d3-scale/test/band-test.js` | — | 77 / 104 | 74 % | js 17, api 10 |
| `d3-scale/test/point-test.js` | — | 14 / 22 | 64 % | api 6, types 2 (D7.1) |
| `d3-scale/test/ordinal-test.js` | — | 63 / 87 | 72 % | js 15, types 7, api 2 |
| `d3-color/test/color-test.js` | — | 79 / 79 | 100 % | — |
| `d3-color/test/rgb-test.js` | — | 97 / 116 | 84 % | js 16, types 3 |
| `d3-color/test/hsl-test.js` | — | 122 / 138 | 88 % | js 13, types 3 |
| `d3-interpolate/test/rgb-test.js` | — | 11 / 22 | 50 % | gap 8 (`.gamma`, phase 23), js 3 |

D4.2 expected color ranges to lift linear to about 71 % and pow to about
72 %. Of the 22 `gap: phase 7` skips in each: linear gains 18 (the six
color ranges and the twelve assertions of its three-stop color test,
whose variable is shadowed when its range turns numeric); pow gains the
six color ranges only, since its three-stop test is a polypow (`api:`,
piecewise is linear only); the copy-and-interpolator tests start with
the `interpolate()` getter (`api:`); `invert` of a color range is
`types:` (E0599, the compile_fail of Phase 7). log gains 5; its six
`interpolateHsl` assertions are `gap: phase 23`. symlog's line 31 is
`types:` (invert). linear and pow stay in Phase 4's skip table (D4.2),
with floors 171 and 161; log's floor is 126.

`--bless --bless-floors` added 492 ids and removed none; floors raised
for linear, pow and log and set for the seven new files.

Translator: `rules/{band,ordinal,color}.mjs` hold 189 lines of code
(228 with comments and blank lines) against the budget of 200. Generic
hooks went to `expr.mjs` (+50, −10: `combine`, test-file assertion
helpers, the `finish` and `assign` hooks, `strictEqual(x, null)` as an
`Err` check, a string literal's `raw` text) and color ranges to
`rules/scale.mjs` (+34, −12).

## D7.1 — `point-test.js` passes under 70 %

**Date:** 2026-10-27 · **Context:** Phase 7, step 5 (c218bd9) · **Status:** decided

### Issue

Phase 7, Harness: after `--bless --bless-floors` each of `band-test.js`,
`point-test.js`, `ordinal-test.js`, `color-test.js`, `rgb-test.js` and
`hsl-test.js` passes at least 70 %. `point-test.js` passes 14 of 22
(64 %); every translated assertion passes.

| Category | Count | Assertions (lines of `point-test.js`) |
| --- | --- | --- |
| api | 6 | the `round()`, `padding()` and `align()` getters, twice (10–12, 43–45) |
| types | 2 | `s.paddingInner` and `s.paddingOuter` are `undefined`: `Point` has no such builders, which the compiler checks (17, 18) |

### Options

| | **A — record and bless at 14** | B — reach 70 % |
| --- | --- | --- |
| Approach | Skip-table row: `point-test.js`, "d3's getters (`api:`), the absent inner and outer padding (`types:`)"; floor 14 | Getters beyond `get_domain` and `get_range` |
| Correctness | Counts stay visible | Against Part I's naming rule |

### Decision

A, the spec's recommended answer ("bless what passes and group the rest
by cause"): floor 14, blessed in c218bd9.

### Why

As D4.2: every shortfall follows from the getter rule or from the type.

### Followups

None.

## D7.2 — `ordinal-test.js` sets `.unknown(undefined)`

**Date:** 2026-10-27 · **Context:** Phase 7, step 5 (c218bd9) · **Status:** decided

### Issue

Two tests of `ordinal-test.js` (lines 14 and 184) call
`.unknown(undefined)`: an explicit, non-implicit mode with no value.

### Decision

The spec's recommended answer: no `.unknown` call is emitted, and every
`s(x)` on that scale maps to `apply`, whose `None` grades as d3's
`undefined` (`rules/ordinal.mjs`). Lines 15, 16, 18, 21, 23, 185 and
186 translate and pass; 19 (`s("0")`, a key of another type), 20
(`s([0])`) and 22 (`new Number(1)`) are skips.

### Why

`Ordinal::apply` never grows the domain, which is what d3's explicit
mode means; `None` is the value d3 returns.

### Followups

- `Ordinal::apply_or_insert` appends an unseen key even when an unknown
  value is set, as Phase 7's signature says ("an unseen key is appended
  first"); d3 returns the unknown value then and appends nothing. The
  translator calls `apply` whenever `.unknown(…)` was set, so no d3 test
  reaches the difference.

## D7.3 — Statements of Phase 7 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 7, steps 1 and 3 (db97b1c, 4b939e2, bf31f49) · **Status:** decided

### Issue

1. **Point's tick position.** Design, `Ticks` impls: `Point<K>`'s
   `position(v, o)` is `apply(v)` (bw = 0). d3-axis applies `center` to
   any scale with a `bandwidth`, point scales included:
   `max(0, bw − 2o)/2` added, which is 0 for every offset of at least 0
   and `−o` for a negative one.
2. **`Hsl::from("#808080")`** (Spot checks): the design has no
   `From<&str>` for `Hsl`, only `FromStr`.
3. **"trim"** (Parser): d3 trims with JavaScript's `trim()` and matches
   `\s` inside the parentheses; both take U+FEFF and not U+0085, where
   Rust's `str::trim` does the opposite.
4. **`BRIGHTER` and `DARKER`** (Modules, `color.rs`) carry no stated
   visibility.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `Point::position` delegates to the band's `center` with zero bandwidth: the point itself for offsets ≥ 0, d3's value for negative ones | `apply(v)` for every offset |
| (2) | The spot check is `"#808080".parse::<Hsl>()` | A `From<&str>` that would have to panic on bad input |
| (3) | JavaScript's whitespace set (`is_space` in `color/parse.rs`) | `str::trim` |
| (4) | Private constants in `color.rs`, used by `Rgb` and `Hsl` (bf31f49); a later phase may make them public | Public constants |

### Decision

A for all four (architect, 2026-10-27): `Point`'s tick position goes
through d3's `center`; the HSL spot row parses through `FromStr`; CSS
parsing trims JavaScript's whitespace set, as d3 does; `BRIGHTER` and
`DARKER` stay private until a phase asks for them.

Recommended: A for all four, already built.

### Why

(1) and (3) follow d3 where the wording is silent or narrower and agree
with it on every spot check; (2) is the design's own `FromStr`; (4)
adds no public item that no phase asks for, and adding one later is
free while removing one is a decision.

### Followups

- Phase 23: if `Lab`, `Hcl` or `Cubehelix` want `BRIGHTER` and `DARKER`,
  they reach them inside `color` as they are.
