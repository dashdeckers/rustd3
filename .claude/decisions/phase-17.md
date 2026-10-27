# Phase 17 — Legends, parts and checks

## Measurements

2026-10-27; `scripts/check.sh full` green at 39e814a in 3 min 17 s
(warm; the iced tests on tiny-skia and wgpu through lavapipe). Phase 17's
core change is additive (`Points::is_collected`, 9ffe3e7) and changes no
d3 test: 9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80
floors, before and after; nothing blessed in the harness.

**D16.4 first** (60b696f, 73b398c). Line box of the text face (typst's
reading of DejaVu Sans, OS/2 `sTypoAscender` 1556 and `sTypoDescender`
−492 of 2048 units, 1 em in all: 11.143 and 3.523 px at 14.667 px).
The one-panel row's bottom reach grows from 23.760 to 27.333 px (the
plot area from 432.773 to 429.200 px tall); a vertical axis's end label
overhangs by half the line box, 7.333 px. Band labels "alpha", "beta",
"gamma" and the twelve months on bottom and top axes share one baseline
each (read from the runs, to 1e-9). The grid of scales' log axes keep 6
major ticks and draw 40 minor ones at 5.333 by 1.333 px. Snapshots
blessed: `decorations_of_one_panel`, `list_dump_of_scatter`,
`svg_of_scatter`, `list_dump_of_a_grid_of_scales` (twice).

**Spot checks** (`figure/tests/main/{legend,parts,hit,check,visible,
layout,figure,resvg}.rs`, `figure/src/{list/parts,hit/grid,layout/snap,
scale/visible,check/find}.rs`): every row holds as written.

- Legend squares 10.267 px in `#0173b2`, `#de8f05`, `#029e73`, domain
  order; one value in `Slot::Bottom` and `Inset::TopLeft` draws twice,
  the row's advance box centred on the plot area to 1e-4; `Slot::Bottom`
  raises the bottom edge by 22.000 px (the row's line box 14.667 + 0.5 em;
  the ink of "l" and "b" tops the ascender by 3e-5 px).
- Locale: "0", "2 000" … "12 000" (U+202F) against "0", "2,000" …
  "12,000"; "2000" … "8000" ungrouped; `.format` untouched; offset
  notation's labels follow the locale too.
- Part and cell hashes equal for the scatter drawn twice (one cache or
  two, or rebuilt); one dots mark's data changed changes its part and
  exactly the one cell that lists it; a figure title pushing the panel
  down changes the panel's, the left axis's and the mark's bounds and
  hashes, not the unmoved bottom axis's.
- Work 1 + ⌊33/16⌋ + 5,600 = 5,603; `cell_at([100, 10])` = 1,
  `cell_at([200, 10])` = `None`; the wheel-like 4 × 4 grid at 960 × 760,
  scale 2: 10 × 10 edges, 81 cells.
- `Target::File` at 1, 1.25, 1.5 and 2: every plot-area edge × scale
  whole and equal to `snap::edge` of the vector layout's; every stroke of
  borders, axes and rules (over 100 segments) a whole number of device px
  wide with both sides on pixel boundaries; at 1.25 the 1.667 px axis
  line is 2 device px. `Target::Vector` keeps Phase 16's rects (the
  layout spot rows hold unchanged).
- `nearest((12, 11), 5)` → index 0 at √5; `nearest((100, 100), 5)` →
  `None`; a thinned line's hits after a draw run no M4 query.
- `Overflow { AxisTitle, needed: 164.56, available: 100 }`;
  `LargeOwned { points: 150000 }`; "漢字" `MissingGlyphs { count: 2 }`;
  the trace and scatter examples check empty at 960 × 760.
- `Fit::Visible` ramp: [−5, 105] → [36.25, 63.75] in one resolve, then
  still (three further resolves); back to [−5, 105] at once; the same on
  a recording channel through the index. No flicker: the section's
  listed decision is not met.

**Example agreement** (D17.2), resvg against the PNG at 2×: the PNG's own
list as SVG, trace 0.0200 and scatter 0.0056; the unsnapped SVG files,
trace 0.0473 and scatter 0.0395. At 1, 1.25 and 1.5 the trace's same-list
scores are 0.0328, 0.0279, 0.0300 (resvg and vello_cpu 0.0345, 0.0252,
0.0288 apart on the vector list), the snapping offset alone 0.0479,
0.0423, 0.0298.

**Renders** (vello_cpu PNGs viewed, scratch, not committed): the grid of
scales at 960 × 720 before and after D16.4 at 1× and 2× (month rows on
one baseline, log axes with minor ticks), at 1200 × 900 with a legend in
every slot and two insets at 1×, 1.25× and 2× (rows centred on the plot
areas, columns centred beside the grid, framed insets, lines at 1.25×
without fringes, corners closed); the trace and scatter at 1× and 2×.

## D17.1 — Details of Phase 17 that the section leaves open, or that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 17, 60b696f … 39e814a · **Status:** open

### Issue

1. **D16.4's minor size.** "the theme's minor tick size and width
   (seaborn notebook 4 px, 1.0 px)" against "6 by 1.25" for the major,
   which the theme reads as points (8 px, 1.667 px).
2. **D16.4's line box.** Which face's metrics, how the last baseline of
   a label is found, what an empty (blanked) label's box is, how a
   multi-line label sits beside a vertical axis.
3. **`Legend<'a, K>`** names a lifetime nothing in a legend borrows
   (E0392 without a `PhantomData`), as `Rule<'a, S>` did (D15.1 (5)); one
   value in a slot and an inset needs a copy.
4. **Legend layout details:** the gap between a row's title and its first
   swatch, where a swatch sits against its label, the column title's
   alignment, several legends in one slot, what side slots centre on,
   the legend part ids and draw order, the legend text sizes.
5. **Part hashes:** a part drawn under another clip but the same
   transform; a run's font as a table index changes with other parts'
   fonts; gradients' and images' enumerated settings.
6. **Cells:** parts with no bounds; edges outside the canvas.
7. **Locale:** whether the offset or multiplier label itself ("+1e9")
   follows the locale.
8. **Snapping:** in which passes edges snap, the axis line's ends, minor
   ticks, labels, tick lengths, rule labels.
9. **Hits:** which points of a line are hoverable; how user marks name
   their panel and mark in issues.
10. **`fig.check`:** features; the room of the figure title and of
    legends; text issues' order and text; `Error::Pages`; telling
    collected data from borrowed in generic marks; when there is no room.
11. **`Fit::Visible`:** the step's units on non-linear and non-numeric
    domains; nothing in view; reversed domains; how marks learn the x
    window.
12. **Commits:** eleven, not seven.

### Investigation

- (1) `seaborn/rcmod.py` 0.13.2 notebook: `xtick.major.size` 6,
  `xtick.minor.size` 4, `xtick.major.width` 1.25, `xtick.minor.width` 1,
  all in points; `Theme::tick` is `pt(6)`.
- (2) `typst-library-0.15.1/src/text/font/metrics.rs:40-43` (typst's
  ascender is `typographic_ascender().unwrap_or(ascender())`); DejaVu
  Sans 1556 / −492 (unit test). An empty label's metric box would be
  zero-width and full-height, and collide with neighbours in the
  cross-axis pass.
- (5) `peniko-0.6.1/src/gradient.rs:235-323` (several enums without
  `Hash`); `Shaped::place` appends a face to the list's table on first
  use, so a run's index depends on earlier parts.
- (10) `rustd3-core` `Points` keeps its variants private
  (`core/src/series/points.rs`); `Fonts::bundled` exists only with
  `fonts` (D13.1 (6)).
- (11) The ramp rows (`figure/tests/main/visible.rs`).

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | 4 pt and 1 pt at the theme's 4/3 px per pt: 5.333 px and 1.333 px (`Theme::minor_tick`, `minor_width`) | 4 px and 1 px, against the theme's units |
| (2) | The first face in fallback order (DejaVu Sans with the bundled fonts), typst's typographic metrics at the tick-label size; `Shaped::last()`, the lowest origin among the runs at the label's largest size (scripts do not count); empty labels have no key and no box; beside a vertical axis the box from the first line's ascender to the last line's descender is centred on the tick | Each label's own face (rows of mixed faces lose their baseline) |
| (3) | `Legend<K>` and `legend<K>`, `Clone`; `Figure::legend` and `Panel::legend` take `Legend<K>` (Phase 35's `Guide<'a>` can be implemented for it) | `PhantomData<&'a ()>` |
| (4) | Row title 0.8 em before its first swatch (`handletextpad`); swatches stand on the baseline, 0.7 em tall (matplotlib's handle box with `ydescent` 0); a column's title centred over its entries; legends in one slot stack outward 0.5 em apart; side slots centre on the plot areas' extent vertically, as top and bottom ones horizontally; inset legends are parts `(7, panel, j)` drawn right after the panel's marks, slot legends `(7, 0xffffff, k)` after the figure title; entries at 11 pt, title 12 pt (seaborn notebook's `legend.fontsize`, `legend.title_fontsize`); the frame's face is the panel color at 0.8 | — |
| (5) | The hash covers every group open at the part's start (transform and clip) and a run's face by font blob id and face index; gradients and image samplers and formats by their debug form; glyph advances and source text are not hashed (they draw nothing) | The composite transform alone; the table index |
| (6) | Parts whose bounds have no area touch no cell; edges are rounded, sorted and deduplicated, non-finite ones dropped and negative ones taken as 0 | — |
| (7) | Tick labels under notation follow the locale; the notation label keeps matplotlib's form (Phase 26 may localise its decimal mark) | Format it through the locale's `~e` (d3 writes "1e+9") |
| (8) | Plot areas snap in every pass, so counts follow the snapped lengths; the axis line's position and its square-capped ends snap as hairlines of its width (corners close), each tick's position as a hairline of its own width (minor ticks too) on the snapped line; tick lengths, labels and boxes keep their positions; a rule's label moves with its snapped line | Snap only the final areas |
| (9) | Lines report the points they pass through inside the plot area (dots: every point drawn); `Ctx::panel()` and `Ctx::mark()` name the mark for `Issue`s; `Index::{len, is_empty}`; `nearest` includes points at exactly the radius | — |
| (10) | `Figure::check` needs `fonts`, as `draw` (D15.1 (8)); the figure title's and slot legends' room is the canvas inside its padding, an inset's the plot area 0.5 em inside its edges (width first); `MissingGlyphs` and `Markup` come from the fresh cache's labels, ordered by markup, `text` the label's characters when it is one literal span and its markup otherwise; `Error::Pages` is a `Markup` issue with the error's message; core gains the additive `Points::is_collected` (9ffe3e7); `NoRoom` when the last placement's reaches and gaps take at least the tracks' length | A trait or `Debug` probe for collected data |
| (11) | Two fitting passes (others first, then Visible ones through `Fits::window`); steps in domain units for `f64` domains (log axes step in data units: their low end rarely shrinks); other domains (time, Phase 33) fit without memory; with nothing in view the last domain stays; reversed domains step alike; the memory lives in `figure::Cache`, keyed by index and `TypeId`, unpruned | Steps in transformed units, which need a `Position` bound the fitter does not have |
| (12) | `fix(figure)` × 2 for D16.4, the seven steps, `feat(core)` for (10), and `test(figure)` reading the test font files once per process (without `fonts`, two caches named one face by two blobs, so equal drawings hashed apart) | — |

### Decision

Recommended: A for all twelve, as built.

### Why

(1) and (2) keep D16.4's intent in the theme's units and with one
baseline per row; (3) follows D15.1 (5); (4)–(9) are the readings closest
to matplotlib and to the section, (5) and (6) strictly safer for skipping
renders; (10) and (11) are the smallest changes that make the section's
rows checkable; (12) follows the workspace's lints.

### Followups

- Phase 18–21: cells come from `list::cells(&list, &xs, &ys, scale)` with
  `Resolved::edges()`; hover reads `Figure::hits` with the draw's cache.
- Phase 26: the notation label and locales with other numerals or
  decimal marks (7).
- Phase 33: time handles get `Fit::Visible` memory in nanoseconds;
  rotated labels keep D14.1's anchors and need rotated boxes.
- Phase 35: `legend::Guide<'a>` over `Legend<K>`; the minor tick size and
  width become theme values.

## D17.2 — The example SVG and PNG no longer share geometry

**Date:** 2026-10-27 · **Context:** Phase 17, step 5 (d06d788) · **Status:** open

### Issue

Phase 15's exit criterion 2 (resvg of each example SVG against its PNG,
tile score ≤ 0.02, D15.2: 0.03 for the trace) compared two draws of one
list. Phase 17 snaps raster targets: the PNG's plot areas move by up to
half a device px from the SVG's, and the dense trace and the scatter's
translucent dots shift with them. `resvg_renders_the_example_svgs_as_their_pngs`
failed at 0.0473 (trace).

### Investigation

Measurements above: on the PNG's own list resvg scores 0.0200 (trace)
and 0.0056 (scatter), as before snapping; vello_cpu's vector render
against its file render, the snapping offset alone, is 0.0466 for the
trace at 2×.

### Options

| | **A — compare like with like** | B — loosen the bound | C — snap screens only |
| --- | --- | --- | --- |
| Approach | The test renders the PNG target's own list as SVG and asserts the renderers' bound against the PNG (trace ≤ 0.03, scatter ≤ 0.02); the example files are asserted ≤ 0.05 | Compare the files at ≤ 0.05 | `Target::File` unsnapped |
| Correctness | Measures what the bound is about: two renderers on one list | Hides renderer regressions under the offset | Contradicts the section and exit criterion 4 |

### Decision

Recommended: A, as built (`figure/tests/main/resvg.rs`,
`resvg_renders_the_example_figures_as_their_pngs`).

### Why

Snapping is specified for raster outputs only, so the files differ by
design; the agreement bound is about renderers.

### Followups

Phase 37's PDF bound (poppler against vello_cpu) should also compare a
list with itself.
