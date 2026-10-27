# Phase 16 — Layout passes

## Measurements

2026-10-27, at bf5aa9f; `scripts/check.sh full` green there in 3 min 33 s (warm). Phase 16 does no core work: the harness stays at
9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80 floors;
nothing blessed.

**Spot checks** (typst 0.15.1, bundled fonts, notebook tick labels at
14.667 px; `figure/tests/main/layout.rs`, `figure/src/layout/*.rs`,
`figure/src/axis/offset.rs`): every row holds as written except the
stacked-rows row (D16.3) and the width of "0" (9.33140, the section
writes 9.3315; twice it is 18.663 as written).

- Auto counts 4 / 2 / 9 (`layout::plan::tests::counts_follow_length_one_tick_per_80_px_across_and_35_px_up`).
- The band row: label widths 35.328, 44.659, 53.990, 63.322 as written;
  strides 4, 3, 2, 1 keep 5 (0, 4, 8, 12, 16), 7, 10 and 20 labels at
  320, 480, 960, 1400 px. The section's first listed decision (other
  counts) is not met.
- Parity: [0, 100] by 5 at 200 px keeps 0, 15, …, 90; panned to
  [2.5, 102.5] it keeps 15, …, 90. A 200-step pan (1–50 px steps, 400 px
  for 100 units) keeps the same labels wherever two views overlap.
- Plans: the 1 × 2 rows, the stacked `temp` offset (vib thickness + 8),
  the inner stacked axis not shown, `.labels(true)` labelled.
- Spans: the spanning panel's x0 and x1 equal column 0's x0 and column
  1's x1 bit for bit.
- Notation: `+1e9` with 0.0 … 1.0 (6 ticks), `+1.2345e4` with
  0.00 … 0.30, `−1e9` with −1.0 … 0.0, none for [1000, 1000.5],
  `1e−7` with 0.0 … 3.0 on bottom and left axes; `.offset(false)` prints
  the full labels (collisions keep every second of the six).
- Held: y [0, 3] → [0, 30 000] keeps the rect bit for bit and grows the
  left axis by width("30 000") − width("3.0"); areas for another size or
  panel count give `held() == false` and the fresh layout.
- Slack: one labelled left axis on `Target::Screen` is 18.663 px thicker.

**Passes** (`Resolved::passes`, debug, shared axes as `subplots` makes
them, and a grid of independent handles):

| Grid | 640 × 480 | 960 × 720 | 1600 × 1200 |
| --- | --- | --- | --- |
| 2 × 2 shared / own | 3 / 3 | 3 / 3 | 3 / 3 |
| 4 × 4 shared / own | 2 / 3 | 2 / 2 | 2 / 3 |
| 8 × 8 shared / own | 2 / 2 | 2 / 2 | 2 / 3 |
| 16 × 16 shared / own | 2 / 2 | 2 / 2 | 2 / 2 |

The grid of scales (`list_dump_of_a_grid_of_scales`) and the scatter
take 3. Part I measured 2 or 3.

**Renders** (vello_cpu PNGs viewed, scratch, not committed): the grid of
scales at 960 × 720, 1× and 2×, and at 560 × 420, 2×; the trace and
scatter examples at 1× and 2×. No two labels overlap; shared axes are
labelled on the outer panels only; `+1e9` and `1e−6` sit above the right
axes' top ends, above the month labels; at 560 × 420 parity keeps every
second month, 0 / 5 000 / 10 000 and the symlog ends and zero.

**Multiplier limit.** The section's second listed decision (multipliers
at m ≥ 6 reading worse than grouped numbers) is not met: under the
neutral locale [0, 3e6] labels "0" … "3 000 000", up to nine characters
that the collision pass thins on narrow axes, where the multiplier keeps
"0.0" … "3.0" and "1e6". matplotlib's limits stand.

## D16.1 — Details of Phase 16 that the section leaves open, or that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 16, 93e6c77 … bf5aa9f · **Status:** open

### Issue

1. **Readers.** `layout::Plan` is public with public fields, and the spot
   rows and exit criterion 2 read plans and kept label boxes from the
   integration tests, but the section names no reader on `Resolved`.
2. **The absolute index.** "round(v / step) when the set's Step is a
   positive f64": the axis's generic code cannot read `Ticks::Step` as
   `f64` (core bounds it `Clone` only, not `'static`); how "uniform" is
   tested; what happens when no stride leaves a label (custom values
   whose indices share no small divisor).
3. **The pass loop's details:** what the first placement holds; where
   overhangs are measured in a pass; that a pass always follows the
   first (no previous counts).
4. **Cells.** `Cell { row, col, rows, cols }`: how `panel_at` matches
   panels, overlapping cells, a span of 0.
5. **Stacking.** The gap is past the last drawn axis on the side; the
   section does not say whether `.outward(px)` alone makes an inner axis
   undrawn.
6. **Slack.** Where the 18.663 px sit inside the thickness; which axes
   take it.
7. **Commits.** Seven: the six steps and the grid-of-scales snapshot.

### Investigation

- (1) `figure/tests/main/layout.rs` needs plans for six spot rows and
  boxes for exit criterion 2.
- (2) `core/src/scale/ticks/continuous.rs:16-18` (`step =
  ticks::tick_step(d0, dN, count)`), `core/src/ticks.rs:149-166`;
  d3's ticks are `i / −increment` below 1 and `i × increment` above, so
  `v × −increment` is the integer index exactly; dividing by a step
  derived from two ticks is off by up to 300 at [1e9, 1e9 + 1].
- (5) `proto/figmodel/src/plot/arrange.rs:77-78`: `outward =
  a.outward().is_some() || stacked; shown = explicit || !(inner &&
  outward)`.
- (6) Part I, Layout: "labels grow outward … reserves two characters of
  slack per labeled vertical axis".

### Options

| | **A — as built** | B |
| --- | --- | --- |
| (1) | `Resolved::plan(panel, axis) -> Option<layout::Plan>` and `Resolved::label_boxes() -> Vec<Rect>` (every kept tick label's box, list px); `Plan` is a plain data type | Tests through the drawn list only; `Plan` public with no reader |
| (2) | Recomputed from the domain's ends and the count: `ticks::tick_increment`, each value times −increment (or over increment) within 1e-6 + 8ε·|r| of an integer, the integers evenly spaced; else band and point keys by domain position (chosen `.values` matched by position among the scale's own ticks), else set position. A stride must leave at least one label; when none up to the index span + 1 does, set positions are used. Pairwise overlap is tested over all pairs | Add `'static` to the axis bounds (a Phase 14 public signature) and read `Set::step` |
| (3) | The first placement has drawn axes at their offset and tick, no titles; each pass measures overhangs at the tick positions of the previous placement (Phase 14 measured them on a bare placement: the scatter's right edge moves 0.01 px, blessed); every resolve runs at least two passes, held ones one | — |
| (4) | Private `rows`/`cols` ranges; `panel_at` finds the panel with an equal cell, overlapping cells are drawn over each other; a span of 0 is 1 | Match on the first cell and update the span |
| (5) | Proto's rule: an explicit `.outward(px)` counts as moved outward, so an inner axis with it is not drawn unless `.labels(b)` is set | Only stacking hides inner axes |
| (6) | Past every other decoration of a drawn labelled vertical axis (title included), so labels growing during a gesture push the title into it; drawn positions do not include it | Between labels and title |
| (7) | Seven commits | Six, the snapshot in step 6 |

### Decision

Recommended: A for all seven, already built.

### Why

(1) is the smallest reader that makes the public `Plan` reachable and the
exit criteria checkable; (2) gives the exact d3 index without changing a
Phase 14 signature; (3)–(6) are the readings closest to the section and
the prototype; (7) keeps the snapshot a separate, reviewable change.

### Followups

- Phase 17: `fig.check` reads plans and kept flags crate-internally.
- Phase 33: the time scale's index (intervals from the epoch) needs a
  branch in `Draw::indices` (`figure/src/axis/draw.rs`), which reaches
  `Ticks::Step` only by value; rotated boxes replace `collide`'s overlap
  test.

## D16.2 — Which axes take offset and multiplier notation

**Date:** 2026-10-27 · **Context:** Phase 16, step 5 (b3cc704) · **Status:** open

### Issue

Phase 16, Offset and multiplier notation: "For a continuous axis with
domain [a, b] … unless `.offset(false)`", labels "core's `Ticks::labels`
of t − o (a translated linear set with the same step)". Applied to
every continuous axis, a log axis over [1, 1e6] (m = 6) would get a
multiplier, and its decade labels "1", "10", …, "1M" would become linear
labels of 1e−6 … 1 ("0.0", …), losing d3's labels and blanking, which
D3/D4 keep for log axes. The section names no rule for transforms.

### Investigation

Built and rendered: a log axis over [1, 1e7] keeps d3's labels
(`log_axes_keep_their_labels_whatever_their_magnitude`); the grid of
scales' log [1, 1e5] and symlog [−100, 100] axes keep theirs. A rule by
transform would need the axis to know the scale's transform, which its
generic code reaches only through a new trait method or `TypeId`
checks of a closed list (missing piecewise scales and user transforms).

### Options

| | **A — ticks on d3's linear grid** | B — by transform | C — every continuous axis |
| --- | --- | --- | --- |
| Approach | Notation when the axis's ticks lie evenly on d3's tick grid for its domain and count (the absolute index's test, D16.1 (2)) and no `.format` is set | A `Position` method or `TypeId` list: identity and pow only | As written |
| Log, symlog decades | Keep d3's labels and blanking | Keep | Multiplier over decades: wrong labels |
| Log within a decade ([1e6, 5e6], ticks 1e6 … 5e6) | "1.0" … "5.0" with "1e6" instead of "1M" … "5M" | "1M" … "5M" | As A |
| Linear, pow, sqrt, symlog's linear branch, custom values on the grid | Notation | Linear and pow only | Notation |
| Cost | No API | A public trait method or a type list | — |

### Decision

Recommended: A, built: notation is a property of linear ticks, which is
what "a translated linear set with the same step" presupposes.

### Why

It keeps d3's log and symlog labels without new API, applies to every
scale whose ticks the translated linear set describes, and changes
d3's output only for log axes inside one decade at a magnitude of 10⁶
or more (or offsets far from zero), where matplotlib's log formatter
writes neither.

### Followups

Phase 39's `helpers::axes::defaults` leaves notation on; its spot rows
inherit this rule.

## D16.3 — The stacked-rows spot row cannot drop "100" under Phase 14's overhangs

**Date:** 2026-10-27 · **Context:** Phase 16, step 3 (11abfc5) · **Status:** open

### Issue

Spot row: "rows [1.0, 0.3], gap px(4); top panel y fixed [0, 3], bottom
panel y fixed [0, 100], both left → the bottom panel's "100" is dropped,
the top's "0.0" kept". Phase 14's layout (and Part I, Layout, item 1)
counts tick labels that overhang a panel's ends in its reach: "0.0"
reaches half its ink height below the top panel and "100" half its ink
height above the bottom one, both rows reserve that, and the gap is the
whitespace between the decorated boxes. The two labels end exactly 4 px
apart and both are kept. The row comes from the prototype, which had no
overhangs (`proto/figmodel/src/plot/arrange.rs:105-128`).

### Investigation

`end_labels_of_stacked_panels_keep_the_gap_their_overhangs_reserve`
measures the boxes 4 px apart (to 1e-9). With overhangs reserved,
labels of two panels collide in an unheld layout only with a gap below
zero, an explicit `.outward(px)` inside another axis's labels, or held
areas whose labels grow: `where_end_labels_of_two_panels_overlap_the_larger_panel_keeps_its_own`
(gap px(−12): "100" gives way to "0.0") and
`labels_that_grow_past_held_reaches_give_way_to_a_larger_panels` (a
held right panel's "−100 000" gives way to the larger left panel's
"3.0") assert the cross-axis rule.

### Options

| | **A — keep Phase 14's overhangs** | B — overhangs only at the canvas edge |
| --- | --- | --- |
| Approach | The row asserted as built (both kept, 4 px apart); the cross-axis precedence tested at gap px(−12) and in held layout | Tracks reserve overhangs only on the grid's outer sides |
| Correctness | No label is dropped where the gap could hold it; gaps stay the whitespace a reader sees, as Part I defines them | The spot row holds, and end labels of neighbouring panels collide and lose labels in ordinary layouts |
| Fit | Phase 14 as decided (D14.1 (12)) | Changes Phase 14's layout and its spot rows |

### Decision

Recommended: A, built.

### Why

Part I defines gaps as whitespace between decorated boxes and counts
overhangs in the reach; the prototype's collision existed because its
layout let labels run into the gap. The cross-axis pass still decides
wherever labels exceed their reaches.

### Followups

Phase 33's band check reads the band counts above (unchanged); Phase 20's
gestures meet the held case this entry tests.
