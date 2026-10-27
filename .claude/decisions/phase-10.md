# Phase 10 — Pointer, zoom and pixel-space operations

## Measurements

2026-10-27, at 9abb262. Harness before: 8 suites, 3,303 assertions,
2,612 passing, 691 skipped, 79 floors. After: 9 suites, 3,314
assertions, 2,623 passing, 691 skipped, 80 floors; the new suite `zoom`
translates and passes all 11 assertions of `d3-zoom/test/transform-test.js`
(floor 11) with `rules/zoom.mjs` at 36 lines (29 of code; budget 50).

`dev/fixtures/pad.json` holds 26 cases: 17 linear (7 with sticky
values), 5 non-flat log (1 with a sticky value) and 4 inverted axes
(3 linear, 1 log); `Pad` matches every one within 1e-12 relative, and a
rerun of `pad.py` is byte-identical.

## D10.1 — `Pad` with no data on a logit scale

**Date:** 2026-10-27 · **Context:** Phase 10, step 4 (071d71b) · **Status:** decided

### Issue

Phase 10, Design, Pad, step 1 tries [0, 1], then [1, 10], then
[inverse(0), inverse(1)]. Under logit neither of the first two is
finite at both ends, so a logit scale with no data would get [0.5,
0.7310585786300049]. "Decisions this phase may meet" foresees it.

### Investigation

matplotlib 3.11.2 `ticker.py`, `LogitLocator.nonsingular`: with no data
(`-inf, inf`) it starts at `initial_range = (1e-7, 1 - 1e-7)`.

### Options

| | **A — try [1e-7, 1 − 1e-7] before the last case** | B — as written |
| --- | --- | --- |
| Approach | `EMPTY = [[0, 1], [1, 10], [1e-7, 1 − 1e-7]]`, the first whose ends the transform maps to finite values | [0.5, 0.7310585786300049] |
| Correctness | matplotlib's start range for logit | A range no axis would choose |

### Decision

A, the spec's recommended answer (071d71b): `scale::logit().pad(None,
…)` is [1e-7, 1 − 1e-7].

### Why

It matches matplotlib, and as one more entry in the list it applies to
any transform that is finite only inside (0, 1).

### Followups

None.

## D10.2 — Statements of Phase 10 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 10, steps 2–5 (803ec51, 357fb14, 071d71b, 9abb262) · **Status:** decided

### Issue

1. **`Pad`'s impl cannot read the domain.** Design, Pad: `impl<T:
   scale::Transform, R, const N: usize> Pad for Continuous<T, R, N>`,
   whose step 5 reads `self.get_domain()`. `get_domain` (Phase 4) and
   the crate-private `transform()` were defined in `impl<T: Transform,
   R: Interpolate, const N: usize>`, so the impl as written does not
   compile (E0599).
2. **Exit criterion 1's command does not run.** `cargo test -p
   rustd3-core interact scale::pad` fails: `error: unexpected argument
   'scale::pad' found` (cargo takes one test-name filter). And the spot
   checks the section places in `core/tests/main/pad.rs` are module
   `pad`, which the filter `scale::pad` does not match (only the unit
   tests in `core/src/scale/pad.rs` do).
3. **One spot row is V8's rounding.** Wheel `Pixels { x: 0, y: −120 }`:
   k 1.1809926614295305 is V8's `Math.pow(2, 0.24)`; the exact value is
   1.18099266142953039…, and `f64::powf` (and Python's `math.pow`)
   returns the correctly rounded 1.1809926614295303.
4. **The sticky upper bound.** Design, Pad, step 3: "x₁bound = the
   smallest sticky > x₁ − tol". matplotlib's `stickies.searchsorted(x1 -
   tol)` (side left) gives the smallest sticky ≥ x₁ − tol.
5. **"pad `None`: log → [1, 10], matplotlib"** is
   `LogLocator.nonsingular`'s start range, not what an empty matplotlib
   axis shows: an empty log axis autoscales to [0.8912509381337455,
   11.220184543019636] (the start range plus its margin; logit:
   [1.995e-8, 0.99999998]); linear's [0, 1] is the initial view limit,
   which matplotlib never autoscales with no data.
6. **Which transform pads.** The algorithm says forward and inverse;
   the scale's transform depends on its domain (`for_domain`: log
   reflects for a negative domain). With the transform as set for the
   current domain, `scale::log()` padded to the extent [−1000, −1] maps
   both ends to NaN and fits [e^−0.055, e^0.055] around 1, and a log
   scale whose domain is negative gets [inverse(0), inverse(1)] = [−1,
   −0.368] with no data.
7. **Decreasing transforms.** "On the ordered pair" and steps 2–4
   assume u₀ < u₁ and x₀ < x₁; a user transform may decrease (the
   reciprocal in `scale::Transform`'s own doc example), so forward of
   the ordered extent is descending and the inverse of the widened pair
   is too.
8. **Orientation for N > 2.** Design, Rescale: "when the orientation
   of d₀′…d′_{N−1} differs from the old domain's" does not say how
   inner stops count.
9. **The ctrl double-click.** The `handle` table's `DoubleClicked` row
   does not filter; d3's `dblclicked` calls `defaultFilter`, which drops
   any non-wheel event with `ctrlKey` (`zoom.js:13–15, 303–304`).
10. **The wheel anchor's location** after a step without a horizontal
    part: the row says the location becomes t′.invert(p) after a
    horizontal pan, and is silent otherwise.
11. **`interact::Transform::translate`** is written as (k, x + k·x′,
    y + k·y′); d3's returns the transform itself when x′ = y′ = 0
    (`transform.js:13`), which differs for −0 and infinite k, and keeps
    `Zoom::default().constrain(t)` equal to `t` bit for bit as the spot
    row and the section's "it returns t" say.
12. **Non-finite extents.** Step 2 sends a non-finite forward value to
    [−0.05, 0.05] (matplotlib's `_nonsingular`); matplotlib's
    `handle_single_axis` drops non-finite data values first, so
    `Some([NaN, 5])` is ±0.055 here where matplotlib would pad the flat
    [5, 5]. The figure (Phase 14) merges only admitted values.

### Investigation

- (1) `core/src/scale/continuous.rs` at 357fb14; every `Continuous`
  that can be built has `R: Interpolate` (`with_stops` and `range`
  require it).
- (3) Node 22 with the vendored d3: `Math.pow(2, 120*0.002)` prints
  1.1809926614295305, `Math.exp(0.24*Math.LN2)` 1.1809926614295303;
  Python's `decimal` gives 1.1809926614295303922…. The other wheel rows
  agree with d3 bit for bit.
- (4) matplotlib 3.11.2 `axes/_base.py`, `autoscale_view`,
  `handle_single_axis`: `i1 = stickies.searchsorted(x1 - tol)`;
  `x1bound = stickies[i1]`.
- (5) `plt.subplots()` then `ax.set_xscale("log")`, `ax.get_xlim()`.
- (9) d3-zoom 3.0.0 `src/zoom.js:303–304`.
- (11) d3-zoom 3.0.0 `src/transform.js:13`.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `get_domain` and `transform()` move to the impl block without bounds (where `map_transform` and `forward()` are); the signature is unchanged and no call site changes | `R: Interpolate` on the `Pad` impl: a different public signature from the spec's |
| (2) | Run as `cargo test -p rustd3-core -- interact scale::pad pad::`; the spot checks stay in `tests/main/pad.rs` as the section places them | Move them to `tests/main/scale/pad.rs` so `-- interact scale::pad` reaches them |
| (3) | The row asserts k = 1.1809926614295303 and d3's three values within 1e-12 relative (as D4.1 (3)) | Exact: fails on a correctly rounded `powf` |
| (4) | ≥, matplotlib's | >: differs only at exact equality |
| (5) | Step 1 as written (no margin), so the spot rows hold; the row's source is `LogLocator.nonsingular` | Margins on the default domain, as an empty matplotlib log axis shows |
| (6) | The transform as it applies to the extent (`for_domain(extent)`), and to each candidate domain when there is no data: a negative log extent pads to [−1412.54, −0.708]; a log scale with no data gets [1, 10] whatever its current sign | The scale's transform for its current domain |
| (7) | The extent ascending; u ascending before `nonsingular` (which sorts, as matplotlib's does); each inverse pair sorted again; the reciprocal pads [1, 4] to [1/1.0375, 1/0.2125] | Margins and sticky bounds applied to the wrong ends under a decreasing transform |
| (8) | The new end stops compare as the old ones do, and no neighbouring pair goes against that direction (equal neighbours allowed) | — |
| (9) | `DoubleClicked` is ignored while the stored modifiers hold ctrl, as d3's filter drops it | A ctrl double click zooms |
| (10) | The location is kept through a vertical burst (d3 keeps `g.mouse[1]`), so a burst repeats d3's arithmetic exactly; t′.invert(p) only after a horizontal pan | Re-inverting after every step: drifts from d3 by rounding |
| (11) | d3's shortcut | Arithmetic on zero: −0 becomes +0, k = ∞ gives NaN |
| (12) | As written (step 2) | Drop non-finite ends first, as matplotlib |

### Decision

A for all twelve (architect, 2026-10-27), as built. `get_domain` and
the crate-private `transform()` live in `Continuous`'s unbounded impl
block, so `Pad` needs no `Interpolate` bound; `Pad` uses the transform
as fitted to the padded extent; an empty log scale pads to [1, 10] as
the spec's rule and spot rows say.

Recommended: A for all twelve, already built. (2) only concerns where a
test file lives; (6) and (7) refine the algorithm without changing any
spot row or fixture value.

### Why

(1) makes the specified signature compile without a public bound the
spec leaves out, and widens a getter only to types no one can build
today; (3), (4), (9) and (11) follow d3 and matplotlib where the section
cites them; (6) uses the transform the padded domain will have, which is
what `for_domain` exists for; (7) keeps "low" and "high" meaning the data
ends for every transform; (8), (10) and (12) are the readings closest to
the text and to d3.

### Followups

- Phase 14 (fitting): `Pad` receives only admitted, finite values, so
  (12) is not reached from the figure.
- Phase 27: time-scale `Rescale` and `Pad` follow the same rules in
  i128 nanoseconds (orientation as in (8)).
