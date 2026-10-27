# Phase 8 — Lines, areas and curves

## Measurements

2026-10-27, at 79e86da. The new suite `shape` (d3-shape 3.2.0), translated
= passing, every passing assertion also under d3's semantics:

| File | Passing / assertions | Share | Skips |
| --- | --- | --- | --- |
| `curve/basis-test.js` | 8 / 8 | 100 % | — |
| `curve/basisClosed-test.js` | 6 / 6 | 100 % | — |
| `curve/basisOpen-test.js` | 12 / 12 | 100 % | — |
| `curve/bumpX-test.js` | 8 / 8 | 100 % | — |
| `curve/bumpY-test.js` | 8 / 8 | 100 % | — |
| `curve/bundle-test.js` | 2 / 3 | 67 % | js 1 (D8.1) |
| `curve/cardinal-test.js` | 14 / 16 | 88 % | js 2 |
| `curve/cardinalClosed-test.js` | 14 / 16 | 88 % | js 2 |
| `curve/cardinalOpen-test.js` | 14 / 16 | 88 % | js 2 |
| `curve/catmullRom-test.js` | 17 / 19 | 89 % | js 2 |
| `curve/catmullRomClosed-test.js` | 16 / 18 | 89 % | js 2 |
| `curve/catmullRomOpen-test.js` | 17 / 19 | 89 % | js 2 |
| `curve/linear-test.js` | 8 / 8 | 100 % | — |
| `curve/linearClosed-test.js` | 4 / 4 | 100 % | — |
| `curve/monotoneX-test.js` | 21 / 21 | 100 % | — |
| `curve/monotoneY-test.js` | 21 / 21 | 100 % | — |
| `curve/natural-test.js` | 10 / 10 | 100 % | — |
| `curve/step-test.js` | 8 / 8 | 100 % | — |
| `curve/stepAfter-test.js` | 8 / 8 | 100 % | — |
| `curve/stepBefore-test.js` | 8 / 8 | 100 % | — |
| `line-test.js` | 3 / 32 | 9 % | api 29 (skip table) |
| `area-test.js` | 7 / 68 | 10 % | api 56, js 5 (skip table) |

Every js skip of a curve file is a parameter passed as a string
(`curveCardinal.tension("0.5")`, likewise `alpha` and `beta`). The api
skips are d3's accessors (`x`, `y`, `x0`, …, `defined`), their getters,
`context()`, `lineX0()`-style derived lines and `line.digits` (13, one
test).

Totals: before 7 suites, 2,966 assertions, 2,378 passing, 588 skipped,
57 floors; after 8 suites, 3,303 assertions, 2,612 passing, 691 skipped,
79 floors. `--bless --bless-floors` added 234 ids, removed none, and set
22 floors.

Translator: `rules/shape.mjs` is 109 lines (89 of code) against the
budget of 150. `expr.mjs` +11 −4: an `array` hook (rules may type array
literals), `nullable` types (`strictEqual(x, null)` checks them for
`None`), and the `statement` hook called for every statement, declarations
included (path's and format's hooks look only at loops; the existing
suites translate unchanged).

## D8.1 — `bundle-test.js` passes under 70 %

**Date:** 2026-10-27 · **Context:** Phase 8, step 5 (79e86da) · **Status:** decided

### Issue

Phase 8, Harness: after `--bless --bless-floors` every one of the 20 files
under `d3-shape/test/curve/` passes at least 70 %. `bundle-test.js` has
three assertions and passes 2 (67 %); every translated one passes.

| Category | Count | Assertion |
| --- | --- | --- |
| js | 1 | `curveBundle.beta("0.5")`, string coercion (the test at line 15) |

### Options

| | **A — record and bless at 2** | B — reach 70 % |
| --- | --- | --- |
| Approach | Floor 2; the file joins the skip table with "string coercion of `beta` (`js:`)" | Translate a string parameter as its number, which Rust's `f64` field rules out |
| Correctness | The count stays visible | Tests JavaScript's coercion, not the port |

### Decision

A, the spec's recommended answer ("bless what passes and group the rest
by cause"): floor 2, blessed in 79e86da.

### Why

The file is three assertions long; its one skip is the coercion the spec
names as a js skip, which every cardinal and Catmull–Rom file also has.

### Followups

None.

## D8.2 — Statements of Phase 8 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 8, steps 1–5 (45b5633, 384cd01, 3c23047, 8dca618, 79e86da) · **Status:** open

### Issue

1. **`Line::closes(self, point: u8)`** (Design, Curve state machines)
   tests `point == 1`. d3's open curves close on `_point === 3`
   (`basisOpen.js`, `cardinalOpen.js`, `catmullRomOpen.js`, `lineEnd`) and
   curveNatural on `n === 1` (`natural.js`, `lineEnd`).
2. **"`CatmullRom*` with `alpha == 0` construct the `Cardinal*` state"**:
   d3's factories test truthiness, `alpha ? new CatmullRom(…) : new
   Cardinal(context, 0)` (`catmullRom.js`, `custom`), so NaN takes the
   cardinal state too.
3. **"keep it `pub(crate)` with a constructor from `Curve`"**: an area
   needs the Bundle-as-Basis rule, so `State` has two, `State::new(curve)`
   for lines and `State::area(curve)` for areas.
4. **"a reflecting context that swaps x and y in every call, as d3's
   `ReflectContext`"**: d3's has four methods (`moveTo`, `closePath`,
   `lineTo`, `bezierCurveTo`); a `PathContext` needs six.
5. **Harness: "`null` items → `None`"**: no d3-shape test passes a `null`
   item (d3's default accessors would throw on one), so the rule would
   translate nothing.
6. **Harness: `area().curve(curveCatmullRomOpen, 0.5)`**
   (`catmullRomOpen-test.js`, lines 34 and 43; 6 assertions): an extra
   argument, which no rule names.
7. **"about 280 assertions"** in the 20 curve files: there are 237
   (`grep -c assert` counts the two import lines of each file).

### Investigation

d3-shape 3.2.0 sources as vendored (`dev/harness/d3/d3-shape/src/curve/`);
d3's `area.curve = function(_) { return arguments.length ? (curve = _, …)
: curve; }` (`area.js`) reads only its first argument. Every spot check
of Phase 8 holds as built (`core/tests/main/shape.rs`), and the harness
passes all 234 translated assertions.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `closes(self, lone: bool)`: each family passes its own test (`point == 1`, `point == 3`, `n == 1`) | `point == 1` for every curve: open curves and Natural close the wrong runs |
| (2) | `js::truthy(alpha)` (new crate-private helper beside `js::min`) picks the Catmull–Rom state | `alpha == 0.0`: a NaN alpha makes every chord NaN, so the spline degenerates to straight segments where d3 draws the cardinal spline |
| (3) | `State::new` and `State::area`; Phase 29's radial area calls `State::area` | One constructor, the Bundle rule repeated in each area generator |
| (4) | `Reflect` also mirrors `quadratic_curve_to` and `arc` (angle θ → π/2 − θ, the sweep reversed), unit-tested; no curve draws either | The trait cannot be implemented with four methods |
| (5) | Not written | Translator code no test reaches |
| (6) | Translated as `.curve(Curve::CatmullRomOpen { alpha: 0.5 })`, what d3 does: the file passes 17 of 19 | A js skip: 11 of 19 (58 %) |
| (7) | Recorded | — |

### Decision

Recommended: A for all seven, already built.

### Why

(1), (2) and (6) follow d3's sources where the wording is narrower; (3)
keeps the area rule in one place for Phase 29; (4) is what the trait
requires, done so it is correct rather than a panic; (5) is the no-dead-code
rule applied to the translator.

### Followups

- Phase 29 (radial lines and areas): draw radial areas through
  `curve::State::area`; `curve::State` stays `pub(crate)`.
