# Phase 4 — Continuous scales I

## D4.1 — Statements of Phase 4 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 4, steps 2 and 4 (6a7fd9f, 1535248) · **Status:** open

### Issue

1. **The log tick cap.** Phase 4, Design, `ticks` rules: `log` "takes
   the decades branch when `(j − i + 1)(base − 1)` exceeds Phase 3's
   cap", `min(10_000, ⌈10 · max(count, 1)⌉)`. At small counts that cap
   is below the multiples d3 itself returns. d3's own test
   `log-test.js:220`, `scaleLog().domain([286.9252014,
   329.4978332]).ticks(1)` → `[300]`, spans two decades after floor and
   ceil (2 · 9 = 18 multiples > cap 10), so the rule as written returns
   the decades branch's `[]`. Likewise `ticks(2)` of `[0.5, 10]`: d3
   gives 15 values (0.5 … 10), the written rule `[1, 10]`.
2. **Crate-visible readers.** Phase 4, Design, `Continuous`: `pub(crate)
   fn transform(&self) -> &T` and `pub(crate) fn forward(&self) ->
   &[f64; N]` "serve later core phases (axes in 6, `Rescale` and `Pad`
   in 10, sequential scales in 23)". Nothing in Phase 4 calls them, so
   they are dead code, which `-D warnings` and the dispatch rules reject
   (the same case as D3.2's `group_from`).
3. **One spot check is one ulp off Node.** Phase 4, Spot checks:
   `log().domain([1, 10]).range([0, 1])`, `invert(0.5)` →
   `3.16227766016838`. Ours is `3.1622776601683795`: both compute
   `exp(1.151292546497023)` (ln 10 / 2, rounded); the exact value is
   3.16227766016837967…, so the platform's `exp` returns the nearer
   double and V8's `Math.exp` the one above.
4. **How the built-in transforms are constructed** is not stated.
   `transform::Pow { exponent }` and `transform::Log { base, negative }`
   are written with their fields; `negative` is state `for_domain`
   derives from the domain.

### Investigation

- (1) d3-scale 4.0.2 `log.js` 70–106 loops over every multiple without a
  cap; the vendored d3 gives `[300]` and the 15 values above (Node,
  through the resolve hook). The loop in d3 is bounded only by the
  base: an integer base of 10⁹ would iterate 10⁹ times per decade, which
  is what the cap is for. A cap of 10,000 multiples bounds the loop, and
  no d3 test comes near it (the largest is 11 decades · 9 = 99).
- (2) In Phase 4 nothing outside `continuous.rs` reads a scale's
  transform or forward cache; the first reader is Phase 6's axes.
- (3) Every other spot check holds exactly as Node prints it.
- (4) Phase 23 builds `Sequential<R, transform::Log>::base` "delegating
  to the transform value", inside core; Phase 5's `AnyContinuous`
  converts from `Continuous<transform::Log>`, inside core. No phase
  builds a `transform::Pow` or `transform::Log` from outside core.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | The decades branch when `(j − i + 1)(base − 1) > 10_000`; d3's results otherwise | Phase 3's count-scaled cap: fails `log-test.js:220` |
| (2) | The readers arrive with their first caller (Phase 6) | Dead code, rejected by `-D warnings` |
| (3) | The spot check asserts Node's value within `Tolerance::default()` and names the cause | An exact comparison that fails on a correctly rounded `exp` |
| (4) | Private fields; crate-private `Pow::new(exponent)` and `Log::new(base)`; the public paths are `scale::pow()`, `sqrt()`, `log()` and the `exponent` and `base` builders | Public fields, which let `negative` disagree with the domain until the next builder |

### Decision

Recommended: A for all four, already built.

### Why

(1) and (3) follow d3's tests and the arithmetic; (2) is D3.2's rule
for code without a caller; (4) keeps derived state private (OPAQUE N2)
and adds no public constructor that no phase asks for.

### Followups

- Phase 6 (axes) adds `pub(crate) fn transform(&self) -> &T` and
  `pub(crate) fn forward(&self) -> &[f64; N]` on `Continuous` with their
  first caller.
- Phase 23: if `Sequential::transform(U)` wants built-in transforms
  from outside core, add public constructors then (adding is free).

## D4.2 — Three d3-scale test files pass under 70 %

**Date:** 2026-10-27 · **Context:** Phase 4, step 5 (12b09ab) · **Status:** open

### Issue

Phase 4, Exit criteria 3: every translated file passes at least 70 % of
its assertions except `radial-test.js`. "Decisions this phase may meet"
expects the linear and pow files to fall short. Three do; every
translated assertion of them passes, under d3's semantics too:

| File | Assertions | Passing | Skipped | Share |
| --- | --- | --- | --- | --- |
| `d3-scale/test/linear-test.js` | 247 | 153 | types 31, js 34, gap 22, api 7 | 62 % |
| `d3-scale/test/pow-test.js` | 244 | 155 | types 31, js 27, gap 22, api 9 | 64 % |
| `d3-scale/test/sqrt-test.js` | 6 | 3 | api 2, js 1 | 50 % |

`log-test.js` passes 121 of 171 (71 %), `tickFormat-test.js` 25 of 25.

- **types (31 each):** 28 assertions use a domain and a range of
  different lengths, which d3 truncates and the type's `N` rules out
  (the 20-assertion "polylinear domain" ticks test among them: a
  four-stop domain over the default two-stop range); 3 use tick counts
  NaN, −1 and ∞, which `usize` rules out.
- **gap: phase 7 (22 each):** color ranges (`["red", "blue"]` and the
  like).
- **js (34 and 27):** arrays changed after a scale copied them (10),
  Dates, strings and `new Number` as domain or range values, `Set`
  iterables, non-boolean clamp flags, `null`/`undefined`/string
  arguments, object ranges, `f + ""`.
- **api (7 and 9; 2 of sqrt's 6):** d3's `clamp()`, `unknown()`,
  `exponent()` and `interpolate()` getters (Part I's naming rule keeps
  only `get_domain` and `get_range`).

### Investigation

- No skip is a translator gap: each carries one of the prefixes Exit
  criteria 2 allows, and the translator already inlines test-local
  functions and translates chained setters on variables and `copy()`
  isolation.
- Phase 7 lifts at most the 22 color assertions per file (linear to
  about 71 %, pow to about 72 %); Phase 5's `piecewise` lifts none,
  since every multi-stop case has unequal lengths or color ranges.

### Options

| | **A — add the three files to Phase 4's skip table** | B — reach 70 % | C — drop the files from the suite |
| --- | --- | --- | --- |
| Approach | Rows: linear and pow, "unequal domain and range lengths (`types:`), color ranges (`gap: phase 7`), Dates, strings and aliasing (`js:`)"; sqrt, "3 of 6 assertions are d3 getters (`api:`)". Floors stay at 153, 155 and 3 | Translate getters or truncate unequal lengths: APIs the design excludes | Contradicts Exit criteria 2 |
| Correctness | Counts and reasons stay visible in the report and floors | Tests of behaviour Rust does not have | Hides 497 assertions |

### Decision

Recommended: A. The floors are blessed at the passing counts (153,
155, 3), so the ratchet holds every passing assertion either way.

### Why

Every shortfall follows from choices the spec makes on purpose
(two-stop `N`, typed ranges, `usize` counts, `get_` getters only) or
from JavaScript coercion; every translated assertion passes.

### Followups

- Phase 7 (color ranges) re-runs the scale suite and raises the floors
  of linear and pow.
