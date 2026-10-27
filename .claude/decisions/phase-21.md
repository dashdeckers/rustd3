# Phase 21 — Plot IV: live data

## Measurements

2026-10-27, a052abf … 80dd830; `scripts/check.sh full` green at 80dd830
in 4 min 5 s (warm), no warnings. Phase 21 does no core work: the harness
stays at 9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80
floors; nothing blessed in the harness. Three figure snapshots blessed
(1108c09, D21.2): only their part lines changed.

**`Fit::Window` spot rows** (`figure/tests/main/fit.rs`, a plot area of
1,000 px on the screen at 1×): last 123.456 s → [113.46, 123.46]; 1,000
samples at 10 kHz later → [113.56, 123.56], a move of 10 columns within
1e-9; one sample more (123.5561 s) → unchanged; no data → [0, 1]; the
column edges after 1,000 samples are the old ones shifted by 10 columns,
apart by less than 1e-9 of a column; a y handle fits as `All`; two
panels sharing a window count the wider one's columns. A fresh cache
fits the same windows as a warm one.

**Layout under a sliding window**: 20 appends of 0.1 s with ticks every
second passing the right end (640 × 300, left and bottom axes): the
plot area holds bit for bit with the reserve of D21.1, and moves when
the reserve is disabled.

**The section's tests** (`ICED_TEST_BACKEND=tiny-skia|wgpu`, iced's
`crisp` on, `web-colors` on; `iced/tests/main/plot.rs`; LIVE = 640 × 400
logical, two panels: a 1-channel recording above, a 2-channel one below):

| Test | tiny-skia | wgpu (lavapipe) |
| --- | --- | --- |
| `revision_change_re_renders_everything` (k = 2) | 15 cells; 0 at the same revision; 15 at the new one; 0 after | same |
| `caching_off_renders_every_cell` | a moved dot: `On` cell 4, `Off` all 9; an equal list none | same |
| `live_append_re_renders_only_touched_cells` (k = 2) | parts changed: the top line only; rendered [4] (panel 0's plot area); 1/255 against the PNG | same |
| `fit_window_follows_in_whole_device_columns`, k = 1 / 1.25 / 2 | moves of 5–6 / 6–7 / 10–11 columns of 540 / 676 / 1,081, integral within 2e-13; first append 12 cells (a tick leaves the left end), then [4, 7, 10, 11, 13, 14], then [4, 7, 10, 13]; parts changed: the three lines and both axes' ticks and labels; every frame 1/255 against the PNG | same |
| `ten_hz_appends_stay_sharp` (800 × 500 at 2, 1M samples per channel) | an append changes 5 parts and 6 of 15 cells: 1,386,708 device px and 716 items, 4.56 ms predicted (≤ 8.35); no wake after the first frame; max 2/255 against the PNG, no channel over 16 | same |
| `drop_front_re_renders_the_cell` (k = 2) | revision bumped; the top line only; rendered [4]; 1/255 against a fresh recording of the retained samples | same |
| `a_gesture_holds_a_windowed_handle` (k = 1) | after a 40 px drag an append renders no cell, frame unchanged; after a double click 1/255 against the windowed figure | same |

Cells of LIVE: rows top padding (0–2), panel 0 (3–5), the gap (6–8),
panel 1 (9–11), the bottom (12–14); 4 and 10 are the plot areas, 7
holds the upper panel's ticks, 13 the lower axis's ticks and labels.

**Frames viewed** (headless screenshots, scratch, not committed): the
windowed trace at factors 1 and 2 over ten appends (tiny-skia): lines
meet the plot-area edges and the closed spine corners without seams
(8× crops of the cell edges at 2), ticks and labels move with the data,
the label at the right end fits inside its reserve; before D21.2 the
same frames showed nothing wrong, but the y-axis strips re-rendered.

## D21.1 — The window's width and the layout depend on each other

**Date:** 2026-10-27 · **Context:** Phase 21, step 1, a052abf · **Status:** decided

### Issue

1. Design: the window uses "a resolved plot-area width of `w` device
   columns". Resolving fits every domain before layout (Phase 14), and
   layout measures the labels of the window it is laying out: `w`
   depends on the window and the window on `w`.
2. Layout, item 1: "tick labels that overhang the panel's ends add to
   the sides they cross". A sliding window moves its ticks through the
   right end; while one sits within half a label of it, the right reach
   grows by up to half a label and the plot area narrows, then widens
   again. `w` changes (the move is no longer a whole number of columns)
   and the partition's rects change (every cell re-renders), against the
   section's spot rows ("a 10-column move"; "the panels' plot-area cells
   plus the bottom x-axis gutter cells render").

### Investigation

`a_windowed_axis_holds_the_layout_while_ticks_slide` (fit.rs) fails
with the reserve disabled: the plot area changes while ticks pass the
right end.

### Options

| | **A — rounds and a reserve, as built** | B — rounds; overhangs as written | C — the widget holds the layout between appends | D — drop overhanging labels |
| --- | --- | --- | --- | --- |
| Approach | Resolve fits, settles windows and `Fit::Visible` handles and lays out, again while the column counts used are not the laid-out ones (at most 3 rounds; the cache remembers the last counts, so a steady layout resolves once). A horizontal axis of a windowed handle reserves the most any of its labels would overhang, at both ends | The rounds alone | Hold the plot areas as during a gesture, relayout on resize | A windowed axis's labels that would cross an end are not drawn |
| Cost | About half a label of plot width at the right end | Layout jitters as ticks pass; every cell re-renders then | The widget's frame no longer equals the figure's PNG; files and screen differ | Labels pop in and out at the ends |
| Correctness | Whole-column moves while label widths keep (they change at a new digit count) | Moves are whole only between relayouts | As A on screen | Stable, but loses labels |

### Decision

A (architect, 2026-10-27), as built: resolve repeats fit and layout
until the column count it fitted with equals the one it laid out (at
most 3 times), and horizontal axes on a windowed handle reserve room
for an end label, so the plot area holds still while ticks slide.

### Why

A is the smallest change under which the section's spot rows hold and
the widget still shows exactly what the figure's PNG shows; the
reserve is matplotlib-like room for an end label that would otherwise
appear and push the layout.

### Followups

Phase 33's time windows use the same rounds and reserve; Phase 22
measures the resolve's cost with windows (one round in a steady state,
two from a fresh cache).

## D21.2 — Cells an append re-rendered beyond the touched ones

**Date:** 2026-10-27 · **Context:** Phase 21, step 3, 60de9af and 1108c09 · **Status:** decided

### Issue

Part I (Live data) and the section: "y gutters are untouched unless a
`Fit::Visible` y handle moved". As Phase 17 built the cells, two things
listed unchanged pixels' cells under changed parts:

1. `list::cells` inflates every part's device bounds by a device px for
   antialiasing, also past the clip a mark is drawn in: a line reaching
   its plot area's edge (every windowed line, data at the domain's end)
   was listed in the y-axis strip beside it.
2. One part per axis held its line, ticks and labels: the line's square
   caps reach into the cells beside the plot area, so a new x domain
   changed those cells' hashes.

Measured on the live trace at k = 1: every windowed append re-rendered
12 of 15 cells, all but the top padding.

### Options

| | **A — fix both, as built** | B — record only | C — hash per command |
| --- | --- | --- | --- |
| Approach | (1) A part's inflated bounds are cut at the clip it is drawn in, taken out to whole device px (60de9af). (2) Each axis line is a part of its own, role 8 (`Spine`), emitted before its axis's part with the same commands (1108c09) | The y strips re-render on every windowed append (about 0.1 ms each at 2×) | Cells hash the commands that touch them, not parts |
| Cost | Three snapshots blessed (part lines only); tests that read an axis part's strokes or part ids updated | — | Phase 31's dependency classification |
| Correctness | Steady state: plot areas, the cell under the upper panel and the bottom row's middle; side cells only when a tick sits within a pixel of an end | As built, more than the section says | Finer still |

### Decision

A (architect, 2026-10-27), as built: a part drawn inside a clip is not
listed in cells beyond the clip, and each axis line is its own part
(role 8, `Spine`), so a windowed append re-renders the plot areas and
the cells whose ticks moved.

### Why

Both are the section's invariant made true at the granularity cells
already have, without new public API: a clipped part cannot ink past a
clip on device pixels, and an axis line does not move with its domain.

### Followups

Phase 31 (tiles, dependency classification) builds on parts that no
longer spill across clips.

## D21.3 — The section's listed decisions

**Date:** 2026-10-27 · **Context:** Phase 21, "Decisions this phase may meet" · **Status:** decided

### Decision

The section's answers: two panels of different widths sharing a window
count the widest one's columns; the narrower one's column edges do not
repeat between updates, its pixels are right (documented on
`Fit::Window`; `a_window_follows_the_widest_panel_that_places_it`). A
window's marks on recordings with different clocks share one `last`,
their newest x of all (documented on `Fit::Window`).

## D21.4 — Details of Phase 21 as built

**Date:** 2026-10-27 · **Context:** Phase 21, a052abf … 80dd830 · **Status:** decided

### Issue

1. **Commits:** six, not four.
2. **Which handle is x.** "Window on a y handle behaves as All": the
   figure model has no notion of x but the marks'.
3. **"The last admitted x"** across marks of several kinds.
4. **Spans and domains** that are not `f64` (time, Phase 33), invalid
   spans, window ends a scale does not admit.
5. **`Fit::Visible` memory** under several resolve rounds.
6. **Which axes reserve** room for end labels.
7. **`ten_hz_appends_stay_sharp`**: "no probe" and "every frame after the
   first reports `previews == 0`" (only a probe reports frames).
8. **The live figures**: a recording appends to every channel at once
   (D9.1), so "append 1,000 samples to channel 0" needs a recording of
   its own; fitted y moves with a new extreme; dense self-overlapping
   strokes render origin-dependently (D18.3).
9. **The harness**: a view borrows the recording an append mutates.
10. **Expected cells** of the window test.

### Options

| | **A — as built** |
| --- | --- |
| (1) | figure `Fit::Window` (a052abf), iced revision and caching (2c99eb7), the two figure fixes of D21.2 (60de9af, 1108c09), the live tests (0340667), docs (80dd830) |
| (2) | A handle is x where a figure mark places it across a plot area: line and dots through the crate-private `Fits::across`, `rule_x`; marks reporting through `Fits::add` alone do not, so a user mark's handle fits as `All` |
| (3) | The upper end of the handle's extent: every mark's admitted values, rules included |
| (4) | All fit as `All`; time windows are Phase 33's |
| (5) | Each round steps a copy of the memory; the final round's is kept, so one resolve steps once |
| (6) | Horizontal axes of a `Fit::Window` handle placed across, held by a gesture or not, so the layout does not move when a hold ends |
| (7) | No probe (nothing calibrates the model); no wake after the first frame (nothing went to the worker, so no slot showed a preview) and every frame within 16/255 of the PNG (2/255 at most, where two lines cross: the cell renders at its own origin, D18.3) |
| (8) | Two recordings (top 1 channel, bottom 2); smooth sines whose sampled peaks are exactly their amplitudes, so fitted y stays and no stroke overlaps itself; "x set wider than the data" is [0, 20] over 10 s |
| (9) | `Harness::park` and `Parked::resume` (`iced/tests/main/headless.rs`): the interface goes into iced's cache between frames, as the runtime does around `update` |
| (10) | The cells that list a changed part in the list before or after: a tick leaving an end takes its pixels in the gutter along |

### Decision

A for all ten (architect, 2026-10-27), as built. A mark that reports a
handle only through `Fits::add` does not place it across a plot area,
so a `Window` on that handle fits as `All`; the `Fit::Window` docs say
so.

### Why

(1) follows the scope-per-crate rule and D21.2; (2)–(6) are the
readings closest to the section that need no new public API; (7)–(10)
test what the section names with what the harness can observe.

### Followups

Phase 33: time spans through `Fits`' window functions; a public way for
user marks to place a handle across, if one is asked for.
