# Phase 23 — Color spaces, interpolation and color scales

## Measurements

2026-10-27, b65ed54 … 28b8f16, one session (no cut after Step 3).
`scripts/check.sh full` green at 28b8f16 in 9 min 50 s (warm; the benchmark rebuilt in release, its smoke plan about 4 min of it), no
warnings.

**Harness**, before (88b1357) and after (28b8f16): 9 suites, 3,314 →
3,822 assertions, 2,623 → 3,040 passing, 691 → 782 skipped, 80 → 97
floors. Every translated assertion passes; every passing one also
passes d3's semantics except two (D23.3 (11)).

| File | Before | After | Share | Skips after |
| --- | --- | --- | --- | --- |
| `d3-color/test/lab-test.js` | — | 49 / 62 | 79 % | js 11, types 2 |
| `d3-color/test/hcl-test.js` | — | 74 / 87 | 85 % | js 11, types 2 |
| `d3-color/test/cubehelix-test.js` | — | 0 / 2 | 0 % | js 2 (D23.2) |
| `d3-color/test/gray-test.js` | — | 2 / 4 | 50 % | js 2 (D23.2) |
| `d3-color/test/lch-test.js` | — | 5 / 7 | 71 % | js 2 |
| `d3-interpolate/test/rgb-test.js` | 11 / 22 | 19 / 22 | 86 % | js 3 |
| `d3-interpolate/test/hsl-test.js` | — | 19 / 21 | 90 % | js 2 |
| `d3-interpolate/test/hslLong-test.js` | — | 19 / 21 | 90 % | js 2 |
| `d3-interpolate/test/lab-test.js` | — | 11 / 13 | 85 % | js 2 |
| `d3-interpolate/test/hcl-test.js` | — | 37 / 39 | 95 % | js 2 |
| `d3-interpolate/test/hclLong-test.js` | — | 19 / 21 | 90 % | js 2 |
| `d3-interpolate/test/cubehelix-test.js` | — | 26 / 29 | 90 % | js 3 |
| `d3-interpolate/test/cubehelixLong-test.js` | — | 26 / 29 | 90 % | js 3 |
| `d3-interpolate/test/hue-test.js` | — | 15 / 21 | 71 % | js 6 |
| `d3-interpolate/test/quantize-test.js` | — | 2 / 2 | 100 % | — |
| `d3-interpolate/test/piecewise-test.js` | — | 14 / 21 | 67 % | types 7 (the table's 14) |
| `d3-interpolate/test/number-test.js`, `round-test.js` | 13, 14 | 13, 14 | 100 % | — |
| `d3-scale/test/log-test.js` | 126 / 171 | 130 / 171 | 76 % | js 23, types 7, api 11 |
| `d3-scale/test/sequential-test.js` | — | 31 / 54 | 57 % | api 12, js 8, types 3 (D23.2) |
| `d3-scale/test/diverging-test.js` | — | 56 / 75 | 75 % | api 10, js 6, types 3 |

The 8 `gap: phase 23` skips of `d3-interpolate/test/rgb-test.js` became
7 passes and one `js:` skip (`gamma({valueOf…})`); the 6 of
`log-test.js` became 4 passes and 2 `api:` skips (the `interpolate()`
getter, which those assertions read). No skip is a gap any more.

Floors raised: interpolate `rgb-test.js` 11 → 19, `log-test.js` 126 →
130; set for the 17 new files at their passing counts.

**Translator**: +167 / −43 lines in `expr.mjs`, `translate.mjs` and
`rules/{color,interpolate,scale}.mjs` (+123 / −35 without comments and
blank lines), within the budget of 250.

**Schemes**: `schemes_match_d3_at_11_samples` compares all 38 functions
at 11 samples (418 strings) with no tolerance: all equal. Every scheme
was rendered as a 550 px strip through `rustd3_figure::raster::render`
(a scratch example, not kept) with d3's eleven fixture swatches below
it and viewed: each runs smoothly through its swatches.

## D23.1 — The section's listed decisions

**Date:** 2026-10-27 · **Context:** Phase 23, "Decisions this phase may meet" · **Status:** decided

### Decision

The section's answers where the phase met the question:

- `piecewise` over fewer than two values: `interpolate::piecewise`
  takes `&[T; N]` and `const { assert!(N >= 2) }` fails when the
  function is instantiated, as `scale::piecewise` (780f75c, with a
  `compile_fail` doctest and its twin).
- A scheme at NaN: documented in `scheme`'s module doc and on the ramp
  functions: NaN channels for the spline and formula schemes, entry 0
  for the viridis family, where d3 returns undefined. No test needs
  d3's NaN output.
- `Interpolate for Rgb` against d3's `interpolateRgb`: no d3-interpolate
  test exposed a difference; the impl's body now calls
  `interpolate::rgb` (the same rule; no public item changed).
- `PartialEq` on `Sequential` or `Diverging`: not met (the figure's
  scale table is Phase 33's).

## D23.2 — Three files pass under 70 %

**Date:** 2026-10-27 · **Context:** Phase 23, steps 1 and 6 (b65ed54, b992a79) · **Status:** open

### Issue

Exit criteria 2: `cubehelix` (2), `gray` (4) and `sequential` (54) are
listed at 70 %. Every translated assertion of them passes:

| File | Passing | Skipped |
| --- | --- | --- |
| `d3-color/test/cubehelix-test.js` | 0 / 2 | js 2: `c instanceof cubehelix`, `c instanceof color` |
| `d3-color/test/gray-test.js` | 2 / 4 | js 2: `gray(120, null)`, `gray(120, undefined)` (opacity coercion) |
| `d3-scale/test/sequential-test.js` | 31 / 54 (57 %) | api 12: the `clamp()` (5), `unknown()` (2) and `interpolator()` (5) getters; js 8: strings (4) and a `Set` as the domain, `s(undefined)`, `s("N/A")`, `strictEqual(s.interpolator(i1), s)`; types 3: three-stop domains and ranges on a two-stop type |

### Investigation

- Phase 7 translated `instanceof` and null or undefined channels as
  `js:` skips in `rgb-test.js` and `hsl-test.js`; these files follow
  it. A Rust value's type is static, so an `instanceof` check would
  assert nothing; `gray(l)` with an absent opacity is the call
  `gray(120)` already makes.
- Part I's naming rule keeps only `get_domain` and `get_range` (D4.2,
  D7.1); the design's `Sequential` has no `clamp`, `unknown` or
  `interpolator` reader. Only those three getters would lift
  `sequential-test.js` over 70 % (43 of 54).

### Options

| | **A — record and bless at the passing counts** | B — reach 70 % |
| --- | --- | --- |
| Approach | Skip-table rows: `cubehelix-test.js`, "`instanceof` only (`js:`)"; `gray-test.js`, "null and undefined opacities (`js:`)"; `sequential-test.js`, "d3's getters (`api:`), strings, iterables and undefined (`js:`), three stops (`types:`)". Floors 0, 2 and 31 | Getters beyond `get_domain` and `get_range`; `instanceof` and null translated as tautologies |
| Correctness | Counts and reasons stay visible | Against Part I's naming rule; assertions that test nothing |

### Decision

Recommended: A. The floors are blessed at the passing counts (0, 2,
31), so the ratchet holds every passing assertion either way.

### Why

As D4.2 and D7.1: every shortfall follows from the getter rule, from
static types or from JavaScript's coercions.

### Followups

None.

## D23.3 — Details of Phase 23 as built

**Date:** 2026-10-27 · **Context:** Phase 23, b65ed54 … 28b8f16 · **Status:** open

### Issue

1. **Step 0 before Step 1.** "The rules are committed before Step 1":
   the generated Rust calls the API each rule targets, so rules
   committed first would not compile until that API exists, and every
   commit must pass `check.sh quick`.
2. **`impl Trait` captures in Rust 2024.** `basis(values: &[f64])`,
   `rgb_basis(colors: &[Rgb])` and `piecewise(values: &[T; N], …)`
   return closures that, under edition 2024's capture rules, borrow the
   slice argument (`rgb_basis(&[red, green, blue])` failed with E0716).
3. **The `.interpolator()` getter.** Translator rules: "`.interpolator()`,
   `.range()` and `.domain()` getters map to `get_*`"; the design's
   `Sequential` and `Diverging` have `get_domain` and `get_range` only.
4. **`assertLabEqual` and `assertHclEqual`** "become `check(Kind::
   InDelta(1e-6), [l, a, b], …)` plus a strict opacity check": d3's
   helpers match a NaN expected channel with a NaN actual one, which
   `assertInDelta`'s verdict fails.
5. **`stops.rs` in Step 4.** Its rows are crate-private; compiled
   before Step 5's functions read them, they are dead code.
6. **Public transform constructors.** `Sequential::transform(U)` needs
   built-in transform values outside core (users, the harness, the
   spot rows); D4.1 left their constructors crate-private.
7. **`From` between the spaces.** The design lists `From<Rgb>` for each
   space, each space for `Rgb`, Lab ↔ Hcl and `From<Hsl>` for each; d3
   also converts Hcl into Cubehelix (`interpolateCubehelix(…, hcl(…))`).
8. **`Interpolate for Timestamp`** "`a + round((b − a)·t)`" leaves the
   rounding of halves open.
9. **Positions of `Ticks`** "None for NaN": a value the transform maps
   to NaN (a log scale's other sign).
10. **Private fields beyond the listed ones**: a forward cache.
11. **d3's semantics.** `diverging-test.js:68` and `:76` (divergingLog
    at 2.8 and −2.8) pass the tolerant format and fail d3's exact one by
    one ulp: V8's `Math.log(3)` is 1.0986122886681096; the correctly
    rounded value (ln 3 = 1.09861228866810969…) is
    1.0986122886681098, which Rust's `ln` and Python's `math.log`
    return.
12. **Commits**: seven code commits and one docs commit, not eight.
13. **Translator additions** the section does not list: functions of one
    number as closures (`function(t) { return t * 2 + 1; }` as well as
    arrows), and a skipped assertion whose argument runs a setter
    (`assert.strictEqual(s.interpolator(i1), s)`) still runs it, so the
    following assertions see d3's state.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | Each step's commit carries its rules and the regenerated Rust; each step blesses against them | Rules first: a commit that does not compile, or stale generated files |
| (2) | `+ use<>` on `basis` and `rgb_basis`; `piecewise` names its interpolator type `F` and returns `+ use<T, G, U, F, N>` (precise capturing must list every type parameter, so an `impl Fn` argument cannot stay anonymous) | Closures tied to the borrowed slice |
| (3) | `api:` skips, as every other getter (no `get_interpolator`) | A public reader no phase asks for |
| (4) | Prelude predicates `lab_equal` and `hcl_equal`, as Phase 7's `hsl_equal` (`check(…, Kind::Strict, &pred(…), &true)`) | NaN channels failing d3's verdict |
| (5) | Generated and committed in Step 4, `mod stops;` from Step 5 | `#[allow(dead_code)]` for one commit |
| (6) | `transform::{Pow, Log, Symlog}::new` public (D4.1's followup); `Asinh` and `Func` stay as they are | — |
| (7) | Exactly the listed impls; the translator converts other pairs through `Rgb`, as d3 does | — |
| (8) | `f64::round` (halves away from zero), so `a.lerp(b, t)` and `b.lerp(a, 1 − t)` agree | — |
| (9) | `None` as well | `Some(NaN)` |
| (10) | `Sequential` and `Diverging` keep `forward: [f64; N]`, as `Continuous` does, so `apply` transforms once | Two or three transforms per `apply` |
| (11) | Kept: the port's value is the correctly rounded one (as D4.1 (3)) | — |
| (12) | b65ed54 (Lab, Hcl, Cubehelix), ba63fb3 (color interpolators), 780f75c (basis, piecewise, quantize, Timestamp), d224726 (tables), f3e73f9 (scheme functions, fixture, licence), b992a79 (Sequential, Ticks for color ranges), 28b8f16 (Diverging, examples), and the docs | — |
| (13) | `expr.mjs`: `closure` and `skip(reason, effect)` (+24 lines) | `gap:` skips |

### Decision

Recommended: A for all thirteen, as built.

### Why

(1), (2) and (5) follow the compiler and the commit rule; (3), (4),
(6), (7) and (10) keep to the design's public surface and Phase 7's
precedents, adding only what D4.1 left for this phase; (8), (9) and
(11) are the readings that agree with d3 wherever d3 is defined; (13)
translates d3's semantics more closely than a skip would.

### Followups

- Phase 33: `PartialEq` for the figure's scale table, if it asks
  (D23.1).
- Phase 26: the generator's rerun (`scheme_tables.mjs`) in
  `scripts/check.sh full`.
