# Phase 5 — Continuous scales II

## D5.1 — Statements of Phase 5 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 5, steps 3 and 5 (d925391, 7c917f2, a6e278d) · **Status:** decided

### Issue

1. **Asinh's rounding.** Phase 5, Design, Transforms, Asinh: "`q =
   sign(x)·10^⌊log₁₀|x|⌋`". With log₁₀ three spot-check rows marked "=
   matplotlib" fail: `[-1000, 1000]` gives ±1000 (9 ticks, not 7), `[0,
   1e6]` gains 1e6, and `.linear_width(10.0)` over `[-1000, 1000]` gains
   ±1000. matplotlib divides natural logarithms, and the quotient falls
   just below the exponent at some exact powers: ln 10³ / ln 10 =
   2.9999999999999996, ln 10⁶ / ln 10 = 5.999999999999999, ln 10⁹ / ln 10
   = 8.999999999999998 (but ln 10⁴ / ln 10 = 4.0).
2. **The `[-1e9, 1e9]` asinh row depends on numpy's instruction set.**
   Spot checks: `[-1e9, -1e7, …, 1e7, 1e9]`, "= matplotlib". numpy 2.5.3
   on this AVX-512 machine computes `arcsinh(1e9)` = 21.416413017506358
   and `sinh` of it 1000000000.0000012 (floor 9: ±1e9 kept); glibc's
   `asinh` (Python's `math.asinh`, numpy with
   `NPY_ENABLE_CPU_FEATURES=X86_V2`, and Rust's `f64::asinh`) gives
   21.416413017506354 and 999999999.9999976 (floor 8: ±1e8). On 1,001
   sampled points numpy's dispatched `sinh`, `arcsinh` and `log10` differ
   from glibc's in 252, 33 and 21 of them. So matplotlib's own answer for
   this row is machine-dependent, and a fixture regenerated on another
   machine would not be byte-identical (Exit criteria 2).
3. **Logit's linear branch is not matplotlib's.** Design, Logit: "if
   `numideal < 2`: `ticks::linear`"; Fixtures: "Every Logit and Asinh
   case is `compared: true`". `LogitLocator` falls back to
   `MaxNLocator(nbins, steps=[1, 2, 5, 10])`, which differs from d3's
   ticks on 4 of 8 linear-branch domains probed at count 10:

   | Domain | matplotlib | d3 (built) |
   | --- | --- | --- |
   | [0.45, 0.55] | 0.46, 0.48, …, 0.54 (5) | 0.45, 0.46, …, 0.55 (11) |
   | [0.35, 0.65] | 0.35, 0.40, …, 0.60 (6) | 0.36, 0.38, …, 0.64 (15) |
   | [0.15, 0.85] | 0.2, 0.3, …, 0.8 (7) | 0.15, 0.20, …, 0.85 (15) |
   | [0.6, 0.8] | 0.6, 0.65, …, 0.8 (5) | 0.6, 0.62, …, 0.8 (11) |

   They agree on [0.2, 0.7], [0.1, 0.9], [0.12, 0.88] and [0.5, 0.9].
4. **"symlog-thinned" covers only part of the thinning.** Fixtures:
   "matplotlib's `SymmetricalLogLocator` emits more than 2 × count ticks
   where ours thins every k-th decade". Ours also thins where matplotlib
   stays under 2 × count: `[0, 1e12]` at count 10 gives 8 ticks here
   (every other decade) and 14 in matplotlib. Neither of the two allowed
   reasons fits such a case.
5. **Asinh's fallback exceeds the cap.** `numticks = min(count, 10_000) +
   1` equal steps of the ends give 10,001 values at counts of 10,000 and
   above, over Phase 3's cap (10,000), which the fuzz asserts.
6. **The fuzz properties need bail-outs the rules as written lack.**
   Fuzzing: "all finite; … inside the ends". Found on the fixed seed:
   - `ticks::linear` over `[∞, ∞]` returns `[∞]` (d3: `ticks(∞, ∞, n)`
     is `[start]`);
   - `ticks::transformed` under `pow 2` over `[-1e-320, -5e-324]` returns
     `[0]` (the squares underflow to 0) and over `[1e300, f64::MAX]`
     `[∞]`;
   - the default `minor` (`ticks::subdivide`) under `func(sqrt)` with
     `TickRule::Data` over `[-1, 0]` returns NaN (the square root of the
     negative majors);
   - `ticks::log` base 2 over `[-0.5, -f64::MAX]` returns −∞: log₂ of
     `f64::MAX` rounds up to 1024, and 2¹⁰²⁴ overflows.

### Investigation

- (1) matplotlib 3.11.2 `ticker.py`, `AsinhLocator.tick_values`:
  `pows = np.sign(xs) * self.base ** np.floor(np.log(abs(xs)) /
  math.log(self.base))`. Computed with this formula, every asinh spot
  row holds except (2).
- (2) `NPY_ENABLE_CPU_FEATURES=X86_V2 python3 -c 'import numpy;
  print(numpy.arcsinh(1e9))'` prints the glibc value; with the variable
  unset, the AVX-512 one. An unknown feature name also leaves numpy at
  its baseline.
- (3) matplotlib 3.11.2 `ticker.py`, `LogitLocator.tick_values`, last
  line: `return super().tick_values(vmin, vmax)` (`MaxNLocator`);
  d3 values from `d3-array` 3.2.4 `ticks` through the resolve hook.
- (5), (6) `core/tests/main/fuzz.rs` at a6e278d: 2,000 debug and 20,000
  release cases pass; two further seeds of 400,000 cases each passed
  locally (not committed).

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `⌊ln|x| / ln 10⌋`, matplotlib's arithmetic | `⌊log₁₀|x|⌋`: three spot rows fail |
| (2) | The row asserts ±1e8, glibc's arithmetic; `scales.py` sets `NPY_ENABLE_CPU_FEATURES=X86_V2` before importing numpy, so the fixture is the same on any x86-64 machine and its case is compared | ±1e9: unreachable with Rust's or glibc's `asinh`; the fixture would differ by machine |
| (3) | d3's linear ticks as the spec says; the fixture holds the linear-branch cases where both agree ([0.2, 0.7], [0.1, 0.9]); the table above records the others | Every fixture case compared, including ones that fail; or a third reason, which the spec rules out |
| (4) | The fixture holds no thinned case under 2 × count | — |
| (5) | The fallback takes `min(numticks, 10,000)` steps | 10,001 values at count ≥ 10,000 |
| (6) | `ticks::linear` gives none for a non-finite end; `transformed` leaves out values carried more than a relative 1e-9 past an end, and non-finite ones; `subdivide` leaves out non-finite steps; `log`'s decades branch leaves out non-finite powers; Asinh gives none for a non-finite end, Logit none when an index is not finite (matplotlib raises `OverflowError`) | Non-finite or out-of-domain ticks |

### Decision

A for all six (architect, 2026-10-27): asinh rounds as matplotlib does
(⌊ln|x| / ln 10⌋); the `[-1e9, 1e9]` row asserts the portable ±1e8 and
the fixture script pins numpy's CPU features; logit's linear branch uses
d3's ticks as the spec says, and the fixture keeps only the cases where
matplotlib agrees, with the two skip reasons the spec allows and no
third tag; the asinh fallback is capped at 10,000; the fuzz bail-outs
stay.

Recommended: A for all six, already built (7ceaacc, d925391, 7c917f2,
a6e278d). The changes to Phase 4's `ticks::linear`, `ticks::log` and
`ticks::subdivide` only add bail-outs at edges no d3 test reaches (the
harness stays at 0 failing). For (3) and (4), if the fixture should
show the differences, a third reason (`"logit-linear"`) and a case like
`[0, 1e12]` tagged `"symlog-thinned"` are a small change to
`scales.py`.

### Why

(1), (2) and (3) follow matplotlib where the spec says "= matplotlib",
and the spec's own rule where matplotlib was not what it described;
(5) and (6) are the bail-out rule of Phase 3 applied to the properties
this phase's fuzz asserts. Reproducible fixtures need a fixed numeric
path; numpy's baseline is glibc's, which is also what the port computes
on Linux.

### Followups

- Later (0.2), with matplotlib's logit minor rule: `MaxNLocator` for
  Logit's linear branch, if matching matplotlib there matters more than
  d3's linear ticks.

## D5.2 — `symlog-test.js` passes under 70 %

**Date:** 2026-10-27 · **Context:** Phase 5, step 6 (f16ea99) · **Status:** decided

### Issue

Phase 5, Exit criteria 5: `symlog-test.js` "passes at least 70 % of its
73 assertions". "Decisions this phase may meet" foresees a shortfall.
The file has 71 assertion calls (the translator and `grep -o
'assert[A-Za-z.]*('` agree), and 39 translate and pass (55 %); every
translated one also passes under d3's semantics.

| Category | Count | Assertions (lines of `symlog-test.js`) |
| --- | --- | --- |
| js | 23 | arrays changed after a scale copied them, `d.push(3)` (10: 60–70, 86–96); strings as range values (4: 27, 32, 75, 76); clamp flags `"true"`, `1`, `""`, `0` (4: 118–121); objects and arrays as range values (2: 80, 81); Dates, strings and `new Number` in a domain (3: 53–55) |
| api | 6 | `clamp()` getter (4: 9, 100, 159, 163); `constant()` getter (2: 10, 37) |
| types | 2 | `domain([])` and `domain([1, 2, 3])` against the two-stop range (47, 49) |
| gap | 1 | `range(["#000", "#fff"]).invert("#999")` (31, gap: phase 7) |

### Investigation

The same categories hold `linear-test.js` and `pow-test.js` under 70 %
(D4.2). No skip is a translator gap; Phase 7 lifts at most line 31,
whose `invert` of a color string stays NaN in d3 (60 %).

### Options

| | **A — add the file to Phase 5's skip table** | B — reach 70 % | C — drop the file |
| --- | --- | --- | --- |
| Approach | Row: `symlog-test.js`, "aliasing, coercion and truthiness (`js:`), the `clamp()` and `constant()` getters (`api:`)"; floor 39 | Getters, string coercion or mutable copies: APIs the design excludes | Contradicts Exit criteria 5 |
| Correctness | Counts stay visible in the report and the floor | Tests of behaviour Rust does not have | Hides 71 assertions |

### Decision

A (architect, 2026-10-27). Phase 5's skip table gains `symlog-test.js`
(71 assertions; coercion `js:`, the `clamp()` and `constant()` getters
`api:`), floor 39.

Recommended: A. The floor is blessed at 39 (f16ea99), so the ratchet
holds every passing assertion either way; the spec's count of 73 reads
71.

### Why

As D4.2: every shortfall follows from choices the spec makes on purpose
(`get_` getters only, typed ranges, copied arrays, two-stop `N`) or from
JavaScript coercion.

### Followups

- Phase 7 re-runs the scale suite; line 31 stays a skip unless color
  ranges translate `invert` of a string.
