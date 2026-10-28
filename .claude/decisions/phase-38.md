# Phase 38 — Helpers I: statistical charts

## Measurements

2026-10-28, 238f40b … 9369d33 and the closing docs commit, one session.
matplotlib 3.11.2, numpy 2.5.3, scipy 1.18.1, seaborn 0.13.2.

`scripts/check.sh quick` green on every commit; `full` green at 9369d33
in 13 min 15 s (warm, the benchmark smoke included), no warnings.

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors; nothing blessed.

**Lines** (8b6f1bf..9369d33): library +2,364 / −1 in 19 files (helpers
1,484 lines of code after rustfmt, 543 of doc comments, about 260 of unit
tests; `mark/error.rs` +22, `lib.rs` +2, `mark.rs` one visibility),
integration tests +1,288 in 8 files, fixtures +871 (`stats.py` 201,
`stats.json` 670). The section estimated about 1,000 (550 library).

**Fixture.** `python3 dev/fixtures/stats.py` run once and committed; a
rerun writes byte-identical JSON; `--check` exits 0, and 1 naming the
three keys after a mean was edited in the JSON (then restored). After
step 3 corrected two comments of the script, `--check` exits 0 and the
JSON is unchanged.

**Spot checks.** Every row of the section is asserted, most against
`stats.json` within the tolerant compare's 1e-9 and as the section
writes them: the three summaries (q1 3.25, median 4.4, q3 5.9, IQR 2.65,
mean 5.206667, whiskers 1.2 and 9.5, fliers [14.0], notch 3.325764 …
5.474236; percentiles 5–95: whiskers 2.3 and 9.5, fliers [1.2, 14.0];
one value: everything 3.0, no fliers; none and NaN: `None`); core's
quantiles 3.25 / 4.4 / 5.9; `Count(10)`, its density, `Auto` (6 bins),
`Count(5)` plain, cumulative and weighted, `Nice(5)` (d3.bin's
`[0,2) 1 … [14,16] 1`), the `Edges` error at index 2; the stacked spans
(a [0,1] [0,2] [0,1], b [1,1] [2,3] [1,3], two rect marks) and their
density; the violin grid over [1.2, 14] with peak 0.138084 at 4.044444,
mean and median; the KDE bandwidth 1.879048; the ECDF's steps (1.2,
0.066667) … (14.0, 1.0); `subplots(2, 3)`: six panels, five distinct
handles, x labels on the bottom row only and y labels on the left
column only.

**Viewed** (vello_cpu PNGs at 2×, scratch, not committed), each beside
the same chart drawn by matplotlib 3.11.2 / seaborn 0.13.2 from the same
data: plain, notched (with means) and dodged box plots of four groups
with outliers on both sides; violins with the inner box, dodged violins
with quartiles and cut 2; an auto density histogram with a weighted step
outline; two groups stacked over 12 bins; four ECDFs and a complementary
one; bars with ± sd error bars capped at 4 pt and dots with asymmetric
uncapped bars. Box, whisker, cap and flier placement, notch shapes,
violin outlines and inner boxes, every histogram and step height, the
ECDF steps and the error bars agree. Two differences, both recorded in
D38.2: the stacked histogram's y keeps a margin below 0 (its rects hold
nothing), and the violins share one width scale (the section's choice;
seaborn scales each to its own peak by default).

## D38.1 — The helpers draw through crate-private items

**Date:** 2026-10-28 · **Context:** Phase 38, steps 2–4, 6a84e0f … 9369d33 · **Status:** decided

### Issue

The section and Part I (Helpers) say each helper "is ordinary code over
the public figure API", written so users can copy and change it, and the
architect's note for the phase asks that a helper needing a crate-private
figure item be a decision rather than a silent visibility change. The
box plot, violin and error bar charts draw geometry no public mark makes
(boxes at half a band's width, caps, notches, mirrored densities), so
they implement `Mark` themselves, and to draw as the figure's own marks
draw they need:

- `Theme::color` (the palette's first color: marks without a key),
  `Theme::gray` (a key its palette lacks), `Theme::patch_width`,
  `Theme::line_width`, `Theme::density_width` and `Theme::marker`: the
  theme's sizes at its context, crate-private since D14.1 (10) and kept
  so by D35.2 (2);
- `list::brush` (a color as an 8-bit solid brush);
- `Ctx::snap` and `layout::Snap::{rect, width, line}`: pixel snapping on
  raster targets (`raster::snap`'s math is public, but a mark cannot tell
  vector output from a raster at 1×);
- `mark::guard::{band, polygon}`: the guard band and Sutherland–Hodgman
  clipping of the notched boxes and violin bodies (Conventions, item 6);
  the module was private to `mark`, now `pub(crate)`.

Everything else is public: `Ctx::{x, y, raw, rect, theme, panel, mark}`,
`Paint::{paint, slot}`, `Position`, `Fits::{add, stick}`, `List` and
`Command`, `check::Issue`, `Theme::{get_ink, get_background}`,
`stats::Kde`, and the `rect` and `line` marks, through which `hist`, its
step outline and `ecdf` draw without any crate-private item. `Fits::across`
(which panels place a `Fit::Window` handle) is not called; user marks
cannot call it either.

### Investigation

Composing the charts from public marks does not reach the section's
geometry: on band scales `bar_y_spans` takes the whole band (or a dodged
part) and ignores `.width`, `line` and `dots` sit at a band's or slot's
middle, and `rect` needs a numeric x. A helper's handles cannot make new
scales either (no figure at that point).

### Options

| | **A — publish the kit** | B — keep it crate-visible (as built) | C — publish the theme readers only |
| --- | --- | --- | --- |
| Approach | The six theme readers, `list::brush`, `Ctx::snap` with `Snap`'s `rect`, `width` and `line`, and `mark::guard` with `band`, `clip` and `polygon` become public, documented as what the figure's own marks draw with | Nothing changes but the helpers module docs, which would say that a copy outside the crate replaces those calls | The six readers and `list::brush` public; helpers drop snapping and carry their own polygon clipping |
| Helpers | Unchanged; copyable verbatim | Unchanged; copyable after replacing about ten calls | Box edges antialiased on raster targets where bars are crisp; about 40 lines of clipping duplicated |
| Users' own marks | Can match the figure's widths, colors, snapping and clipping | Cannot read the theme's sizes or its default color | Can match widths and colors |
| Cost | About 13 public items; `Snap` and `guard` become API to keep | None | Seven public items |

### Decision

A (architect, 2026-10-28): the items become public, so the helpers are
ordinary code over the public API and a user's own `Mark` can match the
built-in marks. Theme readers follow the `get_` convention of D18.1
(`Theme::get_color`, `get_gray`, `get_line_width`, `get_patch_width`,
`get_density_width`, `get_marker`); `list::brush`, `mark::Ctx::snap`
with `mark::Snap`, and `mark::guard::{band, polygon}` keep their names.
Each gets a doc comment saying what a custom mark uses it for. Done as
the first commit of Phase 39's session, with the helpers switched to the
public paths.

### Why

The phase's helpers and Phase 39's presets are the first code outside
`mark` that draws like a mark; the choice is how much of `mark`'s own
toolkit becomes the public seam for that.

### Followups

- If A: one commit making the items public, the helpers unchanged.
- If B: the helpers module docs name the calls a copy replaces.
- Phase 39 builds on whichever answer stands.

## D38.2 — Details of Phase 38 as built

**Date:** 2026-10-28 · **Context:** Phase 38, 238f40b … 9369d33 · **Status:** decided

### Issue

1. **numpy's `"auto"`.** The section describes it as the larger count of
   Sturges and Freedman–Diaconis, Sturges when the IQR is 0. numpy 2.5.3
   (`numpy/lib/_histograms_impl.py`, `_hist_bin_auto`) takes
   `min(max(fd, sqrt / 2), sturges)` with `sqrt = range / √n`: an IQR of 0
   or a long tail is bounded by half the square-root rule. The two agree
   on X (6 bins); on the fixture's flat data (IQR 0, n = 9) numpy makes 6
   bins where Sturges makes 5.
2. **Stacking order.** "Order: the color handle's key order": `add_to`
   holds only the handle, and neither it nor a panel can read a scale's
   domain before the figure resolves.
3. **Stacked bars hold nothing.** Part I makes stacked histograms
   ordinary `rect` marks that "fit both ends"; `rect` holds no value, so
   y keeps a 5% margin below 0, where matplotlib's stacked bars stand on
   the axis.
4. **What `fig.check` reports for a histogram.** "A `Hist` whose `table`
   fails draws nothing and `fig.check` reports the error" names no issue.
5. **Public paths.** `stats::boxplot` is both the statistic's public
   module (`Summary`, `summary`, `Whis`) and the chart function, so is
   `hist`, `violin` and `errorbar`; defining the function inside its
   module would give it two paths.
6. **Builders the section does not list**: coloring several histograms
   or ECDFs on one panel.
7. **Box plots as drawn**: slots on scales without bands; line widths and
   colors of medians, whiskers and caps; fliers; means; notch joins.
8. **Violins as drawn**: flat groups, the inner box's sizes, quartile
   lines, what fits.
9. **Histogram edge cases**: `Auto` with weights, the caps of D34.2 for
   `Edges`, density without values, `Stacked` with edges that fail.
10. **ECDF and error bars**: y limits, colors, caps' units, invalid
    errors.
11. **`subplots`' bad indices** ("panics in debug on a bad index").
12. **Tests that call `fig.check`** need `fonts`.
13. **Commits**: four, as the steps, and the docs.

### Options

| | **A — as built** |
| --- | --- |
| (1) | numpy 2.5.3's rule, as the fixture grades it; `Bins::Auto`'s doc and `stats.py`'s comments say so; at most 10,000 bins (D34.2) |
| (2) | Groups stack in their given order from the bottom (matplotlib's `hist(stacked=True)`); the spot row's a-then-b holds either way |
| (3) | As Part I says; recorded here. Holding 0 would take a `rect` builder that holds a value, a public addition for Phase 39 or later |
| (4) | `mark::Error` gains `Edge { index, value }` and `Weights { values, weights }` (additive, as D34.3 (1) added `Density`), with `From<hist::Error>`; `Issue::Mark` carries them. A `Stacked` whose edges fail adds one `hist` of the union, which draws nothing and reports the error |
| (5) | Each chart and its type live in a private `chart` module (`ecdf` in its own private module) re-exported from `stats`: `stats::{boxplot, BoxPlot, violin, Violin, hist, Hist, ecdf, Ecdf, errorbar, ErrorBar}`; the public modules hold the statistics and options (`boxplot::{Summary, summary, Whis}`, `violin::Inner`, `hist::{Bins, Table, table, Error, stacked, Stacked}`, `errorbar::Err`), D25.3 (3)'s pattern |
| (6) | `Hist::fill(palette, key)` (bars or the step outline) and `Ecdf::stroke(palette, key)`, both through the `rect` and `line` marks' own builders |
| (7) | Slots as bars sit (band, dodged part, else 0.8 of the smallest key gap); box edge, median, whiskers and caps in ink at the patch width (matplotlib's 1 pt box lines under seaborn's colors); fliers as hollow rings of the marker size (matplotlib's and seaborn's unfilled `o`); means as ink triangles; plain boxes snapped as rects are, notched outlines clipped by the guard band, round joins, not snapped; a group without finite values draws nothing |
| (8) | A group of one value or of equal values draws a line across the full slot in the fill color; seaborn's inner box (box line 4.5 × 1.25 pt, whiskers a third of it, the median a dash of 1/1.2 × that long and 1/5 thick in the theme's background); seaborn's quartile dashes; the bodies' grid ends fit y; `.points(n)` below 2 and a negative or non-finite `.cut` leave them as they were |
| (9) | `Auto` with weights bins by the values alone (numpy refuses weights with `"auto"`); caller `Edges` are not capped; density divides by the total counted (numpy), which gives NaN heights for no counted values; a failing `Stacked` as in (4) |
| (10) | ECDF: y holds 0 and 1, so the end lines sit on the frame as in matplotlib; theme color by default. Error bars: ink, the theme's line width; caps in pt at 4/3 px, not scaled by the context (matplotlib's `capsize`); a negative or non-finite error draws nothing (matplotlib raises) |
| (11) | `x(col)` and `y(row)` panic on a bad index in every build (there is no handle to return); `panel(row, col)` debug-asserts the cell |
| (12) | `figure/tests/main/helpers.rs` is behind `helpers` as the section says; its `fig.check` tests are also behind `fonts` |
| (13) | 238f40b fixture, 6a84e0f step 2, e252e26 step 3, 9369d33 step 4, then the docs |

### Decision

A for all thirteen (architect, 2026-10-28), as built, including numpy
2.5.3's `"auto"` bin rule as the fixture grades it.

### Why

(1) follows the library the section names and its fixture; (2), (3) and
(5) are what the built APIs allow; (4) follows D34.3 (1); (6)–(10) follow
matplotlib and seaborn where the section points to them; (11)–(13) follow
the compiler, the features and the commit rule.

### Followups

- (3): a `rect` (or helper) that holds 0, if stacked histograms should
  stand on the axis.
