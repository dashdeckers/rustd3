# Phase 29 — Shapes breadth

## Measurements

2026-10-27, 0a79c75 … b81f677, one session (Phase 30 follows in it).
`scripts/check.sh full` green at b81f677 in 16 min 34 s (the
architect's run after a container restart, so partly cold; the benchmark
smoke included); its first run, at af0dc35, failed in `cargo
doc` on ambiguous links to shape's modules (`line`, `area`, `pie` and
`stack` are functions too), which b81f677 fixes.

**Harness**, before (82a3ad4) and after: 13 → 16 suites, 6,411 → 6,722
assertions, 5,100 → 5,292 passing, 1,311 → 1,430 skipped, 0 failing, 161
→ 179 floors. Every translated assertion passes, all of them under d3's
semantics too (arc-test.js:58, the centroid `[-75, -0]`, after the
prelude's `js_round` kept the sign of a zero result as `Math.round` does,
ee0fb15).

| File | Passing / assertions | Share | Skips |
| --- | --- | --- | --- |
| `pie-test.js` | 15 / 38 | 39 % | api 14, js 8, types 1 (D29.1) |
| `arc-test.js` | 111 / 122 | 91 % | api 11 (accessors' `this` and arguments) |
| `stack-test.js` | 3 / 23 | 13 % | api 15, js 5 (D29.1) |
| `offset/diverging-test.js` | 6 / 7 | 86 % | types 1 |
| `offset/expand-test.js` | 3 / 4 | 75 % | types 1 |
| `offset/none-test.js` | 3 / 4 | 75 % | types 1 |
| `offset/silhouette-test.js` | 3 / 4 | 75 % | types 1 |
| `offset/wiggle-test.js` | 3 / 6 | 50 % | js 2, types 1 (D29.1) |
| `order/*-test.js` (6 files) | 10 / 10 | 100 % | — |
| `symbol-test.js` | 32 / 56 | 57 % | api 24: stroke symbols 12 (Later), getters and accessors 12 (D29.1) |
| `symbols-test.js` | 1 / 3 | the `symbolsFill` row | js 1 (`symbols === symbolsFill`), api 1 (`symbolsStroke`) |
| `lineRadial-test.js` | 1 / 6 | 17 % | api 5 (D29.1) |
| `areaRadial-test.js` | 1 / 28 | 4 % | api 27 (D29.1) |

The offsets and orders pass 28 of 35 together (80 %), as exit criterion
2 counts them. Floors are each file's passing count.

**Translator**: +245 / −6 lines of the phase's 250: `rules/generator.mjs`
43 (a generator's setter state at translation time, shared),
`rules/pie.mjs` 75, `rules/stack.mjs` 71, `rules/symbol.mjs` 41,
`rules/shape.mjs` +7 / −5 (the radial generators), `translate.mjs` +8 /
−1 (three suites, two files). The curve, line and area files translate
unchanged.

**Core**: `core/src` +1,379 / −79 lines (doc examples included; the
section estimates about 1,150), `core/tests` +628 / −1.

**Spot checks**: every row of the section is asserted
(`core/tests/main/shape/{pie,arc,stack,symbol,radial,line_of}.rs`),
recomputed in this session with d3 7.9.0 (d3-shape 3.2.0) in Node: the
pie rows exactly; the arc, symbol and radial paths at d3-shape's 3
digits; the stack rows exactly (Wiggle prints 0.8999999999999999 where
the section rounds to 0.9; InsideOut's `stacking_order` [2, 0, 1] is
d3's per-series index 1, 2, 0). Cross and Wye, a cardinal radial line, a
basis radial area and gaps in both were checked against d3 too.

**Renders viewed** (af0dc35's body): a padded, rounded pie, a donut, a
Wiggle / InsideOut streamgraph, the seven fill symbols and a radial
area, through rustd3-figure's vello_cpu raster at 2× from a scratch
example kept out of the repository.

**"Decisions this phase may meet"**: not met. No translated pie
assertion depends on where NaN lands (the NaN test is a js skip: its
data holds `undefined`); `Sort` keeps NaN last both ways, asserted in
`nan_values_sort_last_both_ways_and_span_no_angle`.

## D29.1 — Six d3-shape files under 70 % for api and js skips alone

**Date:** 2026-10-27 · **Context:** Phase 29, steps 1, 3, 4, 5 (0c21f9c, 900059e, dbf0835, 53e8077) · **Status:** decided

### Issue

Exit criterion 2: "`cargo run -p rustd3-harness -- --check` is green with
these floors (≥ 70% of a file's assertions unless stated)". Six files
cannot reach 70 % with the Rust API the section designs, because most of
their assertions test d3's accessors, getters and derived generators,
which Part I's option A leaves out on purpose:

| File | Passing / assertions | Skips by cause |
| --- | --- | --- |
| `pie-test.js` | 15 / 38 (39 %) | api 14: `value`, `sortValues`, `sort`, `startAngle`, `endAngle`, `padAngle` getters and accessor functions, and their `this` and arguments; js 8: values coerced through `valueOf` or from strings, NaN and `undefined` in the data; types 1: a `Set` as data |
| `stack-test.js` | 3 / 23 (13 %) | api 15: the `keys`, `value`, `order` and `offset` getters, `keys(function)` and `value(function)` accessors; js 5: tests whose accessors write variables declared without a value |
| `offset/wiggle-test.js` | 3 / 6 (50 %) | js 2: `isNaN(series[1][0][2])` and `[3]` read past a point (`undefined`); types 1: the series compared after a `"NaN"` was written into it |
| `symbol-test.js` | 32 / 56 (57 %) | api 12: the stroke symbols (Later); api 12: the `type`, `size` and `context` getters, a `(d, i)` size accessor and the accessors' `this` and arguments |
| `lineRadial-test.js` | 1 / 6 (17 %) | api 5: the `angle`, `radius`, `defined`, `curve` and `context` getters |
| `areaRadial-test.js` | 1 / 28 (4 %) | api 27: seven getters and the four derived lines (`lineStartAngle`, `lineEndAngle`, `lineInnerRadius`, `lineOuterRadius`) |

Each translated assertion passes, under d3's semantics too. The listed
skips of exit criterion 2 name some of these causes (accessors reading
`this` or `arguments`, object identity, the stroke symbols) but not the
getters, which dominate.

The criterion's own figures: `symbol-test.js` has 56 assertions, not
44, of which 12 (not 13) are on stroke symbols; of the other 44, 32
pass (73 %), and of the 37 that draw or measure a fill symbol, 32 (86 %).
`wiggle-test.js` belongs to the group "`offset/*.js` and `order/*.js`
(35 together)", which passes 28 (80 %). `symbols-test.js` passes its
`symbolsFill` row, as the criterion asks.

### Investigation

Counts from the harness report at 53e8077; skip reasons as the
translator writes them (`node dev/harness/translate/translate.mjs`).
Phase 8 had the same shape for `line-test.js` (3 of 32) and `area-test.js`
(7 of 68), which its skip table named; Phase 29's criterion does not.

### Options

| | **A — record and bless at the passing counts** | B — reach 70 % |
| --- | --- | --- |
| Approach | Floors 15, 3, 3, 32, 1, 1 (blessed in the steps' commits after reading the diffs); this entry stands in for skip-table rows | Getters and accessor builders on `Pie`, `Stack`, `Symbol`, `line::Radial`, `area::Radial`, and derived radial lines |
| Cost | none | An accessor API the section and Part I's option A rule out ("Not in this phase": accessors, `pie.value` with functions, a `Symbol` builder type) |
| Correctness | The ratchet holds every passing assertion; the counts stay visible | Tests JavaScript's getters, not the port |

### Decision

A (architect, 2026-10-27): this entry stands in for Phase 29's
skip-table rows for `pie-test.js` (15), `stack-test.js` (3),
`offset/wiggle-test.js` (3), `symbol-test.js` (32), `lineRadial-test.js`
(1) and `areaRadial-test.js` (1), floors at those counts. No getters or
accessor builders are added to reach 70 %; Part I's option A stands.

### Why

The shortfall is the accessor design Part I chooses (option A, with
`LineOf` the only accessor wrapper), the Later stroke symbols, and
JavaScript coercions; every assertion the Rust API can express passes.

### Followups

- If B for any file: a phase that adds accessors raises its floor.

## D29.2 — Details of Phase 29 as built

**Date:** 2026-10-27 · **Context:** Phase 29, steps 0–7 (0a79c75 … b81f677) · **Status:** decided

### Issue

1. **One public path per item.** The layout puts `Pie` in `shape/pie.rs`
   beside `pie::Sort`, `Stack` and `stack()` in `shape/stack.rs` beside
   `stack::Order` and `stack::Offset`, and makes `line` and `area` public
   modules while `shape::Line`, `Vertex`, `Area` and `Span` stay
   re-exported; each would then have two public paths (CLAUDE.md, D27.2
   (1)).
2. **Sorting signed zeros.** "`Descending` and `Ascending` sort stably
   with `f64::total_cmp`": `total_cmp` orders −0 before +0, which d3's
   comparators (`descending.js`, `a - b`) call equal.
3. **Exit criterion 4**: "`Symbol::Circle` emits one `move_to` and two
   `arc`s". `symbol/circle.js:7–8` calls `moveTo` and `arc` once each; the
   two arcs are `SvgPath`'s printing of a full turn as two `A` commands.
4. **`symbol-test.js`'s counts** (exit criterion 2): 56 assertions, 12 on
   stroke symbols (D29.1).
5. **Malformed input to `Offset::apply`.** d3 reads past a short series or
   an order index past the last one (a TypeError or NaN); the section does
   not say what the public `apply` does then.
6. **Paths at 3 digits.** The section's path rows are "printed by
   d3-shape's default 3-digit path"; `SvgPath::new()` prints full
   precision (`0.000000000000004898587196589413` for the donut's 0).
7. **Translator details.** Three suites (`pie` for pie and arc, `stack`,
   `symbol`) and the radial files in `shape`; a declaration of a function
   other than of one number is an api skip (`generator.mjs`); `new
   Array(n)` passed to an order is n empty layers (only its length is
   read); a `"NaN"` written into a series is not made and the series is not
   read again (`types:`), so the `isNaN` before it is graded; the prelude
   gains `slices` (d3's arc objects) and `Polygon` (d3's
   `polygonContext`, curves and arcs adding their end point, which d3's
   double lacks); `js_round` keeps the sign of a zero result.
8. **"`arc().innerRadius(() => 42)` constant-returning arrows become the
   builder"**: no pie or arc test passes one, so no rule translates them
   (D8.2 (5)).
9. **Commits.** The section names 8; built as Step 0, then a core and a
   harness commit per step (Phase 28's pattern), a translator style
   commit, the Part I tests and a rustdoc link fix: 15 commits and the
   closing docs.
10. **`LineOf`** has a `Debug` that shows its curve (the accessors are
    closures), and `shape::line_of()` builds it, as Part I calls it.
11. **Names.** The area loop over `(x0, y0, x1, y1)` items is the private
    `area::draw_edges`; d3's `curveRadial` point is the crate-private
    `curve::radial`, which both radial forms use.

### Investigation

(1) As in D25.3 (3) and D27.2 (1). (2) `descending.js`: `b < a ? -1 : b >
a ? 1 : b >= a ? 0 : NaN` gives 0 for ±0. (3) `circle.js` in the vendored
d3-shape 3.2.0; the counting doubles are
`core/tests/main/shape/symbol.rs`. (6) `withPath` in `src/path.js` sets
3 digits.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `Line`, `Vertex`, `Area`, `Span`, `Pie` and `Stack` are defined in `shape.rs`; their methods (and the tuple impls of `Vertex` and `Span`) live in the public modules `line`, `area`, `pie` and `stack`, beside `line::Radial`, `area::Radial`, `pie::Sort`, `stack::Order` and `stack::Offset`; every path the section and Phase 8 name is the item's only one | Two public paths for six items |
| (2) | NaN after every number, ±0 equal (`partial_cmp`), stable: d3 wherever d3 is defined | `total_cmp`: −0 before +0 |
| (3) | The double asserts one `move_to` and one `arc` call; `SvgPath`'s two `A`s are the Circle spot row | A count no context sees |
| (4) | Recorded (D29.1) | — |
| (5) | Rows are the shortest layer's length; order indices past the last layer are ignored; no input panics | — |
| (6) | The spot checks and doc examples print through `SvgPath::with_digits(3)` | — |
| (7) | As described | — |
| (8) | Not translated | A rule no test reaches |
| (9) | 15 commits | Eight |
| (10), (11) | As described | — |

### Decision

A for all eleven (architect, 2026-10-27), as built.

### Why

(1) keeps every specified path and the one-path rule, as the earlier
phases did; (2), (3) and (6) follow d3's sources; (5) makes a public
function total; (7)–(11) change no API the section specifies.

### Followups

None.
