# Phase 35 — Themes, colorbars and hatching

## Measurements, session 35a

2026-10-28, session 35a (D34.5's fix and steps 1–2, e19632d … 7b40c1b;
the cut after step 2). `scripts/check.sh full` green at 7b40c1b in
12 min 12 s (warm, the benchmark smoke included); its first run, at
a0e4b5d, passed every test and stopped at `cargo doc -D warnings` on a
redundant link target in `theme.rs` (fixed in 7b40c1b with a test
comment).

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors.

**Lines** (be48aae..a0e4b5d): figure library +1,398 / −491 in 23 files
(git counts the 361 lines of `theme.rs` that moved into
`theme/settings.rs` as removed and added), figure tests +660 / −102 in
8 files (snapshots included), the wheel example +58 / −5.

**D34.5** (e19632d). On the wheel figure, FR's and RR's titles move up
3.62 px onto FL's and RL's baselines and FL's and RL's down 0.15 px (ink
to metric anchoring); each row's baseline is 8 px plus DejaVu Sans's
descender at 16 px (3.844 px) above its scatters, asserted. The layout
iteration moves the plot areas by at most 0.0004 px. On the grid of
scales, panel B's title had sat 15.8 px above panel A's (B's right axis
carries "+1e9"); both now share their row's baseline, raised by the
month axes on top.

**Spot checks** (steps 1–2), each asserted: `Theme::default()` equals
`seaborn(Ticks, Notebook).palette(COLORBLIND).domains(Margin(0.05))`;
talk tick labels 22 px (unit test and drawn runs); paper line width
1.2 pt and major tick 4.8 pt; poster text 24 pt and axis line 2.5 pt;
`font_scale(1.1)` tick labels 12.1 pt (drawn runs 16.133 px), axis lines
unchanged; the three rejected inputs equal `Theme::default()`; every size
of the table at the four contexts; `Domains::Nice` with `.ticks(5)` over
[0.3, 9.7] gives [0, 10] (d3 in Node: `nice(5)` and `nice(12)` both
[0, 10], [0, 7.35] → [0, 8]), a handle without an axis rounds with 10, bars
keep 0; a UTC handle over 09:17–16:41 with `.ticks(5)` floors and ceils
to the hour, [09:00, 17:00] (d3-scale's `scaleUtc().nice(5)` in Node).
The palettes equal seaborn 0.13.2's `color_palette(name).as_hex()`.

**CAM02-UCS distances** (the section's last spot row, Part I's Themes
table). colorspacious is not installed; a numpy reimplementation of its
pipeline (sRGB to linear, Machado 2009 at severity 1, sRGB primaries to
XYZ100, CIECAM02 at D65, Y_b 20, L_A 64/π/5, average surround, CAM02-UCS)
reproduces every cell of the table, deep 5.6 and colorblind 18.6 among
them, when the simulated linear RGB is clipped to [0, 1]; without the
clip the protanopia cells of colorblind, Category10, Observable10 and
Okabe-Ito differ (21.3, 3.4, 23.7, 23.8 for the first three colors).
Scratch script, not committed.

**Wheel figure presets** at 960 × 760 (vector, `resolve`), the same for
all five styles (styles change no size):

| Context | Plot-area share | Passes | `fig.check` |
| --- | --- | --- | --- |
| paper | 62.18% | 3 | empty |
| notebook | 54.68% | 3 | empty |
| talk | 37.44% | 3 | 6 overflows |
| poster | 24.19% | 3 | 6 overflows |

Part I's sketch measured 58.6%, 50.7% and 34.7% with estimated 10-px
text; typst's measured text and D16.4's line boxes give each context
about 3.5 points more. Ticks at notebook with `Domains::Nice`: 51.41%.
At talk the overflows are Part I's: "vibration RMS [m/s²]" needs 246.84
of 162.14 px (Part I: 247 of 151), "bearing temp [°C]" 217.22 of 162.14
and "count" 67.71 of 48.64 (Part I: 68 of 45), on the two outer columns.
`wheels_fit_at_talk` (1600 × 1200) is empty.

**Viewed** (PNGs at 2×, scratch, not committed): a one-panel line and
scatter at 640 × 480 under the five styles beside seaborn's own
rendering of the same data; the wheel figure under the five styles at
the four contexts (paper and notebook at 960 × 760, talk and poster at
1600 × 1200) beside seaborn's rendering of a comparable 4 × 2 grid of
the same samples at whitegrid, darkgrid and ticks (notebook) and ticks
(talk, paper); the wheel example's `--theme` output for five presets; a
scatter with dashed and solid lines and four legends of the three
glyphs in insets and slots. Grid colors and widths, the dark styles'
missing axis lines, tick marks only in ticks, label distances, and text
and line sizes at talk read as seaborn's; seaborn's bars have white
1-pt edges (D35.1) and its axes all four spines.

## D35.1 — Bars without edges under the grid styles

**Date:** 2026-10-28 · **Context:** Phase 35, step 2, a0e4b5d · **Status:** decided

### Decision

The section's answer: seaborn's styles set `patch.edgecolor = "w"` with
`patch.force_edgecolor`, so its bars have white 1-pt edges; rustd3's
bars have no edges in any style by default, and no `Theme` builder is
added until one is asked for. Viewed on the wheel figure's dodged
histograms under whitegrid and darkgrid: the bars read as distinct
without edges, as Observable Plot's do.

The section's second listed decision (the colorbar inset length) belongs
to step 3.

## D35.2 — Details of Phase 35a as built

**Date:** 2026-10-28 · **Context:** Phase 35, D34.5 and steps 1–2, e19632d … a0e4b5d · **Status:** decided

### Issue

1. **Theme's paths.** The section writes `theme::Style`,
   `theme::Context`, `theme::Domains` and `theme::palette`, and keeps
   `Theme` where Phase 14 put it (D14.1 (3): the crate root, as iced and
   the tests import it). A public `theme` module that defines `Theme`
   gives it two public paths, against CLAUDE.md's one-path rule.
2. **Getters.** The section's sketch lists "getters used by layout and
   marks: tick_text(), label_text(), …" inside `impl Theme`, and a spot
   row reads `Theme::seaborn(Ticks, Talk).tick_text()`; D14.1 (10) kept
   getters crate-private, and the tests that compare every setting are
   to sit in `figure/tests/main/theme.rs`, which sees public items only.
3. **Pads.** "Every size is multiplied by the context factor", and the
   prototype scales the tick pad, label pad and title pad; seaborn's
   `plotting_context` does not set `xtick.major.pad`, `axes.labelpad` or
   `axes.titlepad`, so seaborn's pads stay 3.5, 4 and 6 pt at every
   context.
4. **Palette and defaults of a preset.** `Theme::seaborn(style,
   context)` names no palette or domains (seaborn's `set_theme` default
   palette is deep); the section's builder validation names only the
   empty palette.
5. **What styles draw.** The table's "none (white)" axis line of the
   dark styles; whether hidden tick marks keep their room; where grid
   lines come from on panels with twin axes or none, over which ticks,
   in which part and draw order, clipped or not; tinted axes under each
   style; seaborn's four spines.
6. **`Domains::Nice` details.** Which axis's count a handle shared by
   several axes takes; handles no axis shows; `Fit::Window` and
   `Fit::Visible` handles; how "sticky values still hold" applies; held
   plot areas.
7. **`Kind::niced`'s default** as the section writes it leaves `count`
   unused, which warns.
8. **Legend glyphs.** The dot's size, the line's length and caps, an
   invalid dash.
9. **D34.5's room.** "Reserved room stays as now" leaves a title on a
   panel with nothing on top (no top axis, no overhang) short of its new
   baseline by the descender: it would reach 3.84 px into the padding.
10. **The wheel example's `--theme`**: its files and what it reports.
11. **Commits**: four code commits for D34.5 and the two steps, with
    tests in each.

### Investigation

- (3) `seaborn/rcmod.py` 0.13.2: `_context_keys` lists 20 keys, no pad.
- (5) matplotlib 3.11.2 under each `sns.set_theme(style=…)` (scratch):
  the x tick labels' top sits 13.194 px (100 dpi) below the axes in every
  style, `tick1On` false outside ticks; spines `.15`, `.8` or white,
  1.25 pt; grid `.8` or white, 1 pt.
- (9) `figure/tests/main/layout.rs`, the one-panel title row: as built
  the plot area starts at PAD + 8 + 3.84375 + the ink ascent.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `pub mod theme` (`Style`, `Context` with `scale()`, `Domains`, `pub mod palette`); `Theme` defined in the crate-visible `theme/settings.rs` and re-exported at the root only: `rustd3_figure::Theme`, one path |
| (2) | Getters stay crate-private (`get_domains` where the builder takes the name, D18.1); `theme.rs`'s unit tests compare every setting at the four contexts and the spot rows; `tests/main/theme.rs` holds the builder example, defaults, validation, nice domains, and reads sizes, widths and colors from drawn lists |
| (3) | Pads scale with the context, as the section and prototype say (at talk 5.25 pt where seaborn keeps 3.5) |
| (4) | Every preset starts with the colorblind palette and `Margin(0.05)` (the section's default; its example sets `palette::DEEP` explicitly); `.palette` also rejects a palette with a non-finite color, as `.ink` and `.background` reject such colors |
| (5) | Dark styles draw no axis lines (seaborn's white spines are invisible on its white figure); hidden tick marks keep their room, as matplotlib's do; grid lines at each panel's first shown horizontal and first shown vertical axis's ticks (matplotlib's one x and one y grid per axes; twins add none), blanked log ticks excluded, unclipped, one stroke in part role `Grid` (9) after the panel background and before the marks (`axes.axisbelow`); a tinted axis's line is drawn only where the style draws axis lines; no top or right spines (`.border` frames a panel) |
| (6) | The count layout's first pass gives the first axis that shows the handle in panel and insertion order, before the passes iterate; d3's 10 without an axis; only `Fit::All` handles round, windows and `Fit::Visible` handles fit with no margin; sticky values at or beyond the extent's ends stop the rounded ends (exact comparisons); with held areas the counts come from them |
| (7) | `let _ = count;` in the default body |
| (8) | Dots the theme's marker diameter (6 pt), at most the 0.7-em box; lines 2 em (`legend.handlelength`), butt caps, the theme's line width; a dash that `mark::dashes` rejects draws solid |
| (9) | The room is the larger of as built and what the new baseline needs (title pad, descender, ink ascent above the row's top-side axes); the wheel figure keeps its room (0.0004 px through the iteration) |
| (10) | `--theme <style>:<context>` suffixes the files `_<style>_<context>` and prints `fig.check`'s issues; a malformed preset is an error naming the expected form |
| (11) | e19632d D34.5; 6e9de08 step 1; a0e4b5d step 2; 7b40c1b a doc link `cargo doc` rejected and a test comment |

### Decision

A for all eleven (architect, 2026-10-28), as built. Theme getters stay
crate-private beside D18.1's public `get_ink` and `get_background`;
pads scale with the context as the spec and the prototype do; under
`Domains::Nice` only `Fit::All` handles are rounded.

### Why

(1) keeps D14.1's root path and the section's `theme::` paths under the
one-path rule (D25.3 (3) moved `Bin` to a private file for the same
reason); (2) adds no public API the section does not name; (3), (4) and
(8) follow the section where it speaks and matplotlib where it points to
it; (5) and (6) are the readings closest to matplotlib, seaborn and d3;
(7) is the compiler's; (9) keeps the architect's "room stays" for every
figure where it suffices and never lets a title leave its room.

### Followups

- Step 3: colorbars outline the bar with the axis line; under the dark
  styles the theme has no edge color (`Theme::edge` is `None`), so the
  bar needs a rule (none, or the ink).
- Phase 39's `Axis::minor` draws at `minor_tick`/`minor_width`, which
  now follow the context.

## Measurements, session 35b

2026-10-28, session 35b (steps 3–5, 5493663 … 05aaad4). `scripts/check.sh
quick` green on every commit; `full` green at 05aaad4 in 11 min 55 s
(warm, the benchmark smoke included).

**Harness** unchanged (core changed one doc comment): 16 suites, 6,722
assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.

**Lines** (267e777..05aaad4): library code +2,307 / −188 in 36 files
(unit tests included), figure tests +820 in 3 files, the wheel example,
its library and tests +108 / −14.

**Exit criteria.** (1) `cargo test -p rustd3-figure --features raster
--test main -- colorbar hatch theme`: 28 pass (13 colorbar, 4 hatch, 11
theme). (2) `colorbar::a_table_strip_equals_its_scale_at_every_pixel_of_a_300_px_bar`
(viridis and magma, right and bottom slots: every pixel equals
`apply(value at its centre)`) and
`an_interpolator_strip_shows_the_interpolator_at_each_row_centre` (turbo
and RdBu, left and top). (3) `cargo test -p rustd3-examples -- wheels`: 10
pass; `cargo run -p rustd3-examples --example wheels -- --hatch` writes
`wheels_hatched.{svg,png}` and `wheels_inset_hatched.{svg,png}`. (4)
`cargo test -p rustd3-figure --doc -- colorbar`: the E0277 `compile_fail`
row and its twin pass; compiled once in a scratch example, rustc reports
E0277 (`Ordinal<&str, Rgb>: Ramp`) and E0271 (`Domain == f64`) for
`colorbar(ordinal)`.

**Spot checks**, each asserted: pixel 150 of a 300-px viridis bar is
#21918c (40.13, row 128), pixels 0 and 299 #440154 and #fde725; viridis
rows equal `scheme::viridis(i/256)` for all 256; symlog [0, 1e6]
(constant 1): `apply(1000)` #21918c, ticks 0 1 10 … 1000000 (8), 1000 at
0.50007 of the bar; quantize blues5 [0, 100]: 5 rows, labels 0 20 40 60 80
100; threshold [0, 1]: 3 equal rows, ticks at 1/3 and 2/3, labels 0 1;
`Extend::Both` on a 300-px interior: two triangles of 15 px (a bar of 330
px with them), in #440154 and #fde725; hatch over [100, 200] × [300, 400]
at notebook: k = 36 … 53, 18 lines, one stroke 1.067 px wide; neighbouring
bars: every line at a whole k·8 px from the figure's origin, k 45 … 53
on both sides of the shared edge.

**Wheel figure hatched** (release build, vector, warm cache, best of 20):
the draw takes 1.62 ms plain and 1.75 ms hatched (+0.13 ms); 211 of the
232 histogram bars get a hatch group (21 bars are too small for a 45°
line 8 px apart to cross them), 442 lines, 3 legend squares; the list
grows from 428 to 1,070 commands; SVG 377,754 → 430,027 bytes (+51 KB).
Part I's sketch hatched 190 bars by a size filter: 0.39 ms, +91 KB.

**Viewed** (PNGs at 1× and 2×, scratch, not committed): sequential
viridis (right, both triangles, title) and symlog magma (bottom, max
triangle), diverging RdBu over two rows (left, min triangle), quantize,
quantile and threshold side by side (right), inset bars along a wide and
a tall panel, each under ticks and darkgrid; matplotlib 3.11.2 / seaborn
0.13.2 colorbars of the same scales drawn beside them (same quantile
labels, same tick marks in darkgrid, an outline only in ticks); hatched
dodged bars (45°, cross 0°, white −45°), a cross-hatched area, a hatched
density, legends in a slot and an inset, and the hatched wheel figure,
beside matplotlib's hatched bars. `fig.check` is empty on every figure.

## D35.3 — The colorbar inset length

**Date:** 2026-10-28 · **Context:** Phase 35, step 3, 5493663 · **Status:** decided

### Decision

The section's answer: an inset colorbar is 40% of its panel's long side.
Phase 17's inset legends are sized by their content (a framed column 0.5
em from the corner), which suggests no length for a bar, so the
section's 40% stands; the bar runs along the plot area's longer side and
is framed as an inset legend (D35.6 (6)).

## D35.4 — Strips on raster targets have a row per device pixel

**Date:** 2026-10-28 · **Context:** Phase 35, step 3, 5493663 · **Status:** decided

### Issue

The section's strip is "one `Command::Image`, `smooth: false`, `N × 1` (or
`1 × N`) pixels stretched to the bar", 256 rows for interpolators and
tables, and exit criterion 2 asks that "every pixel of the rendered strip
equals the scale's color at that pixel's center value, exactly". With 256
rows over 300 px, the centres of pixels 37, 112, 187 and 262 (from the low
end) fall exactly on row edges (t = 32/256, 96/256, …), where d3's
`table[⌊256 t⌋]` takes the upper entry. vello_cpu 0.3.0 sampled the lower
one there, on vertical and horizontal bars alike (its nearest painter
steps in `f32`, `vello_cpu-0.3.0/src/fine/lowp/image.rs:73`), so those four
pixels were one entry off.

### Options

| | **A — a row per device pixel on raster targets (as built)** | B — 256 rows, nudged | C — 256 rows as written |
| --- | --- | --- | --- |
| Approach | `Target::File` and `Screen`: one image row per device px of the snapped bar, each the sample under its centre (d3's floor); vector output keeps one row per sample (256), binned unequal bins 4 rows per px | Shift the image 1/1024 px toward the low end and clip it to the bar, so ties fall upward | Keep 256 rows; the criterion fails at ties |
| Correctness | Exact at every pixel whatever the renderer's ties; SVG and PDF keep the 256 samples | Exact unless a pixel centre lies within the nudge of an edge; an extra group per strip | 4 of 300 pixels one entry off |
| Cost | The image depends on the scale factor (600 rows at 2× for 300 px) | A push and pop per strip | — |

### Decision

A (architect, 2026-10-28): raster targets sample one colorbar image row
per device pixel, so every pixel shows the colour at its centre; vector
output keeps the 256-row strip.

### Why

It makes the criterion hold by construction instead of by a renderer's
arithmetic; vector outputs keep the section's 256 rows.

## D35.5 — Labels of binned colorbars

**Date:** 2026-10-28 · **Context:** Phase 35, step 4, 0228e0b · **Status:** decided

### Issue

D25.3 (8): core labels binned tick sets with d3's `,f` at the smallest
gap's precision, so the quantile edges 3, 7.25, 9, 14.5, 20 read 3, 7, 9,
15, 20. The architect asked for the smallest precision that keeps every
label distinct and exact to the edges' own digits, the rule chosen
recorded here. Edges such as sample quantiles can have unbounded digits,
so "exact" needs a bound.

### Investigation

matplotlib 3.11.2 labels a `BoundaryNorm` colorbar with
`ScalarFormatter._set_format`: from `3 − ⌊log₁₀ range⌋` places down, the
fewest at which every location lies within `1e-3 × 10^⌊log₁₀ range⌋` of its
rounding. It gives "3.00 7.25 9.00 14.50 20.00", "0 20 40 60 80 100",
"0.000 0.333 0.667 1.000" for thirds, "0.0010 0.0025 0.0040", and "1.0 1.0
1.5 2.0" for 1, 1.0004, 1.5, 2 (two labels alike; scratch script).

### Options

| | **A — matplotlib's precision, raised until distinct (as built)** | B — exact shortest digits | C — only where d3's labels collide |
| --- | --- | --- | --- |
| Approach | `,f` in the figure's locale at matplotlib's precision for the edge set, plus places while two labels read alike (at most 20); for all three binned scales | The most places any edge's shortest form needs | d3's rule, more places only when labels repeat |
| Correctness | Reads every edge to a thousandth of the range's decade; matches matplotlib's labels where they are distinct | 3.1415926535 for a quantile of real data | 7.25 still reads 7 (distinct from its neighbours) |

### Decision

A (architect, 2026-10-28): binned colorbars label with matplotlib's
ScalarFormatter precision, raised until every label is distinct
(quantile edges read 3.00, 7.25, 9.00, 14.50, 20.00). This closes D25.3's
followup.

### Why

It is the rule of the library the colorbars follow, with the
distinctness the architect asked for added.

## D35.6 — Details of Phase 35b as built

**Date:** 2026-10-28 · **Context:** Phase 35, steps 3–5, 5493663 … 05aaad4 · **Status:** decided

### Issue

1. **`Guide<'a>`'s shape.** The section leaves its body open ("measure
   and draw"); a public trait whose methods take crate-private layout
   types fails `private_interfaces`.
2. **`Ramp`'s bound and kinds.** The colorbar needs its scale's tick set
   and positions; the section implements `Ramp` for `Linear<Rgb>`, while
   D33.3 (5) made `Continuous<T, Rgb, N>` the color handle.
3. **`Samples::Bins { edges }`**: "the bin edges in domain units" — all
   edges, or the inner ones a threshold scale has.
4. **End colors.** "The scale's clamped end colors" for interpolators,
   and the triangles' place: inside the length the slot gives, or
   beyond it.
5. **What the dark styles draw.** No edge color for the outline (D35.2's
   followup); tick marks, which the styles turn off on axes.
6. **Where ticks and titles go.** Slots put ticks away from the grid; an
   inset bar's orientation, its frame; the title's turn on a right bar
   (matplotlib reads it bottom to top, rustd3's right axes top to
   bottom).
7. **Colorbar labels**: thinning by collisions, offset notation.
8. **How legends learn hatches.** "A legend whose marks are hatched draws
   the hatch in each square" names no API; marks are public trait
   objects.
9. **Hatch geometry details**: shapes no line crosses, the density's
   outline, a dense hatch, rejected builder input.
10. **The wheel figure's hatch**: `figure_with(samples, inset, palette)`
    has no hatch.
11. **Crate-private changes**: `Placed::draw` splits into `marks` and
    `texts`; `Labels: Clone`; `Snap::scale`; `layout::plan` and
    `Theme::pt` crate-visible.
12. **Commits**: three, one per step.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `pub trait Guide<'a> { #[doc(hidden)] fn erase(self) -> Erased<'a>; }`, `Erased` public but unnameable (a private module), holding the crate's `Stored::{Keys, Bar}`; implemented for `Legend<K>` and `Colorbar<'a, S>` only; `legend::Content` and `legend::Block` dispatch between `keys::{Content, Block}` (Phase 17's, moved to `legend/keys.rs`) and `colorbar::{Content, Block}` |
| (2) | `Ramp: Ticks<Domain = f64, Step = f64>`, so `colorbar`'s signature stays the section's; implemented for `Sequential<Rgb, T>`, `Diverging<Rgb, T>`, `Continuous<T, Rgb, N>` (`Linear<Rgb>`, piecewise, transformed), `Quantize<Rgb>`, `Quantile<Rgb>`, `Threshold<Rgb>`; interpolator rows through the scale with an identity transform over [0, 1] (exact), color ranges through `range(stop positions).invert(t)` |
| (3) | The inner edges, n − 1 for n bins (thresholds, quantiles); the bar's ends close the first and last bins; a threshold bin the range has no color for is transparent |
| (4) | `paint` at the domain's ends (`Kind::domain`), the first and last bins' colors for binned scales; the bar with its triangles keeps the length its place gives (matplotlib shrinks the interior), each triangle 5% of the interior |
| (5) | No outline where `Theme::edge` is `None` (matplotlib's white outline is invisible on a white figure); colorbar tick marks in every style (matplotlib's colorbar turns its long axis's ticks on; seaborn's grid styles keep them) |
| (6) | Right and left slots: vertical, ticks right and left; top and bottom: horizontal, ticks up and down; insets: vertical with ticks right when the plot area is at least as tall as wide, else horizontal with ticks below, framed as an inset legend (face and edge); titles as axis titles, so a right bar's reads top to bottom as rustd3's right axes do |
| (7) | Labels are not thinned and have no offset notation; blanked log labels draw minor ticks as on axes; counts follow the interior's length (one per 35 px up, 80 px across) and join layout's settling test |
| (8) | The figure's own marks report a hatched palette key through `Fits` (crate-private, as they report labels); `Resolved` keeps (palette, key as `Box<dyn Any>`, hatch); a legend's square takes the first hatch reported for its key, anchored to the figure; dots and lines are not hatched; user marks do not report |
| (9) | A shape no line crosses gets no group (21 of the wheel's 232 bars); a density's hatch lies between its fill and its outline; a family of more than 4,096 lines over one shape is left out; a non-finite angle hatches nothing; spacing, width and color rejected as the theme's builders reject |
| (10) | `figure_with(samples, inset, palette, hatch: Option<&Hatch>)`, `wheels::hatched(samples, inset)` (45°), the example's `--hatch` adds `_hatched` to the names |
| (11) | As listed; no public item changes |
| (12) | 5493663 step 3, 0228e0b step 4 (with core's `Ticks for Threshold::position` doc), 05aaad4 step 5 |

### Decision

A for all twelve (architect, 2026-10-28), as built.

### Why

(1) keeps the section's public surface (`Guide<'a>`, `impl
legend::Guide<'a>`) and the existing calls without exposing layout
types; (2)–(4) are the readings closest to the section and matplotlib;
(5) and (6) follow matplotlib under seaborn where the section points to
it and rustd3's own axes where it has a rule; (8) needs no public API the
section does not name and keys hatches per palette key; (9)–(12) change
no behaviour the section states.

### Followups

- Phase 38–39: helpers that build heatmaps get colorbars from the same
  handle; a `Legend::hatch` builder only if per-legend hatches are asked
  for.
- Phase 37: the PDF bound compares a list with itself (D17.2); colorbar
  strips there are 256-row images (vector target).
