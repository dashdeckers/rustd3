# Phase 34 — Figure marks breadth

## Measurements

2026-10-28, session 34a (steps 1–4, 3e752bf … 1e34953; the cut after
step 4). `scripts/check.sh full` green at 1e34953 in 11 min 31 s (warm,
the benchmark smoke included).

**Harness** unchanged (core's `m4_joint` reaches no d3 test): 16 suites,
6,722 assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.

**Lines** (39bb6e7..1e34953): library code +2,360 / −69 in 26 files
(core 5, figure 21; unit tests inside them included), integration tests
and fixtures +1,819 / −13 (`kde.json` 452 lines of numbers).

**Spot checks**, each asserted: `m4_joint` gives `[[0, 1, 2], [4, 5, 7,
8, 9]]` and agrees with a raw scan of both channels on 40 random
recordings (with NaN in both, two validity tests); two recordings are
`Err(Recordings { left, right })`. Bars `[(a, 3)]` fit `[0, 3.15]`, dots
`[(a, 3)], [(b, −1)]` `[−1.2, 3.2]`, `bar_y_spans([(a, [1, 3])])` `[0.9,
3.1]`; `rect` fits x `[−0.15, 3.15]`, y `[−0.2, 4.2]` with 2 `Fill`
commands in 1 part. `histogram(X)` gives d3.bin's `[0,2) 1, [2,4) 5,
[4,6) 5, [6,8) 2, [8,10) 1, [10,12) 0, [12,14) 0, [14,16] 1` (d3-array
3.2.4 in Node) and `Count(10)` numpy's `2 3 4 3 0 1 1 0 0 1` with edges
1.2, 2.48, …, 14.0. `Kde::new(X)`: σ̂ 3.229654, n^(−1/5) 0.581811, h
1.879048; the seven densities; the grid `[−4.437143, 19.637143]`, step
0.120976, peak (4.031199, 0.138086); every value within 1e-9 relative of
`dev/fixtures/kde.json` (scipy 1.18.1's `gaussian_kde`, seaborn 0.13.2's
`KDE`, `python3 dev/fixtures/kde.py` run once; a rerun is byte-identical).
The dodged line over band keys 0..4, slot 2 of 3, padding 0.1 on 300 px:
vertices at 0.1·s + k·s + 2.5·(0.9·s)/3 with s = 300/4.1.

**Ribbons thinned jointly** (`marks::area`): at 1× and 2× (`Screen`) the
drawn polygon equals core's `m4_joint` over the inverted device-pixel
edges point for point, keeps lower ≤ upper at every vertex pair and
both edges' extremes in every column; a second draw runs no M4 query.
Against `Thin::Off` rendered by vello_cpu (scratch, 400,000 samples, 700
× 300): a smooth band with sample noise differs in 2.2 % of pixels by
more than 16/255 at 1× and 2.1 % at 2× (the noisy edges' fringes);
adversarially dense data (an oscillation of 0.37 rad per sample, 570
samples per px) in 8.3 % and 6.3 %.

**Viewed** (vello_cpu PNGs at 1×, scratch, not committed), each against
seaborn 0.13.2's look: areas, a ribbon with a line, a log area reaching
past the axis, dodged bars over five band keys, stacked-ready span bars,
rects with an outline, horizontal bars; a default numeric histogram
(against `histplot`), dodged numeric and key histograms; vertical and
horizontal densities beside a scatter (against `kdeplot(fill=True)`);
three sources jittered at 0.8 and dodged over eight speed bins with their
dodged dashed trend lines (against `stripplot(dodge=True)`).

**"Decisions this phase may meet"**: not met in steps 1–4 (the wheel
figure is step 6).

## D34.1 — `series::Error` gains `Recordings`

**Date:** 2026-10-28 · **Context:** Phase 34, step 1 · **Status:** decided

### Decision

The section's answer (Design, Core: `series::m4_joint`; Conventions,
item 7): Phase 9's public `series::Error<X>` gains `Recordings { left:
series::Id, right: series::Id }`, returned by `Series::m4_joint` for
channels of two recordings, checked before the columns. No crate of the
workspace matches on `series::Error` exhaustively (figure, iced, examples
and bench only name it in doctests' `Ok::<(), …>`), so no call site
changes. `m4::Lines::thin`, crate-private, now takes up to two channels'
extremes per column, which keeps `Series::m4` as it was.

## D34.2 — Histogram bin counts past core's tick cap

**Date:** 2026-10-28 · **Context:** Phase 34, step 2, 27499fb · **Status:** decided

### Issue

`mark::Bins::Nice(n)` is d3.bin's nice bins: thresholds from
`ticks(min, max, n)`. Core's `ticks` gives no ticks when there would be
more than `min(10,000, ⌈10·count⌉)` (Phase 3's bail-out,
`core/src/ticks.rs:124`), so `array::Bin::layout` then returns one
bucket over the whole domain (handoff 26): a histogram asking for many
bins would silently collapse into one bar. The section is silent.
`Bins::Count(n)` (numpy's rule) does not go through ticks, but an
unbounded `n` allocates without bound.

### Investigation

d3's tick step rounds `span / count` to 1, 2, 5 or 10 × 10ᵏ, which can
be smaller than the raw step by up to √10 / 2 = 1.58, so a count gives
at most about 1.58·count + 3 ticks over its niced domain. d3 itself has
no cap: `bin().thresholds(20000)` on the spot-check data gives 25,601
bins and `thresholds(6000)` 6,401 (d3-array 3.2.4 in Node through the
resolve hook). Test: `mark::histogram::tests::nice_bins_never_collapse_past_the_tick_cap`
and `marks::histogram::a_histogram_asking_for_more_nice_bins_than_ticks_allow_keeps_them_apart`.

### Options

| | **A — clamp below the cap (built)** | B — clamp at the cap | C — retry smaller counts | D — no cap for bins |
| --- | --- | --- | --- | --- |
| Approach | `Nice(n)` asks ticks for `clamp(n, 1, 6,000)` (≤ 9,500 bins, under the cap for any data); `Count(n)` makes `clamp(n, 1, 10,000)`; both on `Bins`' docs | `clamp(n, 1, 10,000)` for both | Halve the count until ticks fit | A capless tick variant in core for bins |
| Correctness | Never collapses; the most bins either rule gives stays under 10,000 | Counts between about 6,300 and 10,000 can still collapse, depending on the data's span | Never collapses; the bin count depends on retries | d3's result, but unbounded allocation (d3 allocates 25,601 bins for 20,000) |
| Cost | Two constants | One constant | A loop and its doc | A core change (Phase 3's behaviour) |
| Fit | Matches matplotlib, which has no such cap, for every practical count | The architect's default, not sufficient alone | Ad hoc | Changes a Phase 1–10 item's behaviour (Conventions, item 7) |

### Decision

A (architect, 2026-10-28), as built: `Bins::Nice(n)` asks for at most
6,000 bins and `Bins::Count(n)` makes at most 10,000, both documented
and tested, so a histogram never collapses into one bin.

### Why

A is the smallest rule under which a nice histogram never silently
collapses, and both caps sit far above what a plot can show (10,000 bins
are 10 per px at 1000 px).

### Followups

None; Phase 38's `Bins::Auto` and `Edges` follow the same caps.

## D34.3 — Details of Phase 34a as built

**Date:** 2026-10-28 · **Context:** Phase 34, steps 1–4, 3e752bf … 1e34953 · **Status:** decided

### Issue

1. **What `fig.check` reports.** "The mark then draws nothing and
   `fig.check` reports it" (KDE) and Step 5's "the mark keeps
   `mark::Error::ChannelLength` … and `fig.check` reports the error"
   name no issue; `ribbon_series` over two recordings ("Both views must
   belong to one recording") has no figure-level answer.
2. **Bars on scales without bands.** `.width(px)` "for non-band axes"
   has no default.
3. **`rect`'s stroke**: its width, and whether a stroked rect is filled.
4. **Pixel snapping** of bars and rects on raster targets, and bars or
   bins with no area.
5. **Values a scale cannot place**: a base of 0 on a log axis (d3's
   interpolation gives NaN there), negative bar values on log.
6. **Areas over a recording channel**: whether they thin (the section
   lists no `.thin` on `Area`).
7. **Polygons and the guard band.** Lines clip with Liang–Barsky; areas,
   ribbons and densities are polygons.
8. **Hits** of the new marks.
9. **`mark::Binned`'s method**: "implemented by the figure crate for
   those types only" names no method.
10. **Density details**: what enters fitting, how `.adjust` combines
    with `.bandwidth`, whether the outline includes the baseline, a
    density without an estimate.
11. **Dodge and jitter** on dots over a band y, and dodge without a
    palette key.
12. **The memo key** of a joint query; **commits**.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `check::Issue::Mark { panel, mark, error: mark::Error }` (`Issue` is `#[non_exhaustive]`); `mark::Error` (`Clone`, `PartialEq`) gains `Recordings { left, right }` (step 1) and `Density { values }` (step 3) beside the section's `GridLength` and `ChannelLength` (step 5). A mark that keeps an error neither fits nor draws |
| (2) | 0.8 of the smallest gap between the bars' distinct key positions (matplotlib's 0.8 bar width when keys are one unit apart), 0.8 of the axis length for one position; `.width(px)` overrides; band scales use their bandwidth and ignore it. Point-key histograms use the same rule |
| (3) | Observable Plot's: `.stroke` alone outlines without filling, `.fill` and `.stroke` do both, neither fills in the theme's first color; outlines 1 pt (seaborn's `patch.linewidth`, crate-private `Theme::patch_width`), mitred, on whole device pixels |
| (4) | Rectangles snap their edges to device pixels on raster targets (matplotlib's auto-snapping of rectilinear patches; no seams between neighbouring bins); a rectangle with no area after clipping and snapping draws nothing (empty bins, a value at its base) |
| (5) | A value the transform does not take lies at the guard band's end on the side of the domain it is beyond (matplotlib's `nonpositive="clip"`): bases, bar values, `rect` edges and density sizes. Points of an area's line still break there, as a line does |
| (6) | Areas thin like lines (one channel's M4, `Thin::Auto`, no builder): per column the polygon reaches the column's extremes, so the fill covers what the full polygon covers per column |
| (7) | Sutherland–Hodgman against the guard band, inside vertices kept bit for bit |
| (8) | None (`Mark::hits`' default); Phase 36's hover can add them |
| (9) | `fn bars(values, bins: Option<Bins>) -> Vec<([Self::Domain; 2], f64)>`: both ends the key on band and point scales, a bin's edges on continuous ones; no new public type. Keys are counted in the order they first appear (keys are only `PartialEq`); numbers bin in data units on any transform |
| (10) | The finite values enter fitting on the value axis, the tails do not (the wheel figure's temperature domain needs it); the size axis gets 0 and 1, held at 0. `.adjust(f)` multiplies Scott's or the explicit `.bandwidth(h)`; `Kde`'s rescaling is crate-private. The outline is the closed region (seaborn's `PolyCollection` edge). A density without an estimate neither fits nor draws |
| (11) | Dots dodge and jitter along x when x has bands, else along y; jitter's share is clamped to [0, 1]; without a palette key (`.fill(c, key)`) dodge keeps the whole band |
| (12) | `thin::memo::Key` gains `upper: Option<usize>`; joint and single queries share the frame's column boundaries. Five code commits for four steps: core (`m4_joint`, D34.1) apart from figure in step 1 |

### Decision

A for all twelve (architect, 2026-10-28), as built, with one change:
bars (`bar_y`, `bar_x`, their span forms) and histogram bars report one
hit candidate each, at the centre of the bar's value end (the top of a
`bar_y`, the right end of a `bar_x`), so hover finds the nearest bar;
areas, ribbons, densities, rects and images report none. Done in Phase
34b's session.

### Why

(1) gives every "draws nothing" case of the phase one structured report
that Step 5 extends; (2)–(5), (7) and (10) are the readings closest to
matplotlib, seaborn and Observable Plot where the section points to
them; (6) keeps a long area as cheap as its line; (8), (9), (11) and
(12) add no public API beyond what the section names, except (1)'s issue
variant and error variants.

### Followups

- Phase 34b (step 5): `ChannelLength` and `GridLength` join
  `mark::Error`; `fig.check` already reports `Issue::Mark`.
- Phase 36: hits for bars, areas and ribbons if hover needs them.
- Phase 38: shared bin edges for dodged and stacked numeric histograms.

## Measurements, session 34b

2026-10-28, session 34b (D34.3's hit change and steps 5–7, 41c2370 …
ee179bf). `scripts/check.sh full` green at ee179bf in 11 min 10 s
(warm; the benchmark smoke now 60 cells, 54 run).

**Harness** unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

**Lines** (7d0551d..ee179bf): figure library +840 / −73 in 10 files,
figure tests +596 / −2; examples +334 (the wheel module and its main) and
tests +199, snapshots 541 lines (the list dump; the SVG is one 378 KB
line); testkit +70 / −3; bench +317 / −92; fixtures `wheels.py` 146 and
`wheels.json` 178.

**Spot checks**, each asserted. A 2 × 2 grid is one 2 × 2 RGBA8 image,
unsmoothed, at the device rect of its extent (`[w/4, 3w/4] × [h/4,
3h/4]` for [1, 3]² on [0, 4]²; on device pixels at 1.5×), row 1 on top;
`Grid::new(3, 2, 5 values)` is `GridLength { cols: 3, rows: 2, len: 5 }`.
`fill_by` over viridis [0, 3] with values 0, 3, 3, NaN, 1 gives 4 points,
the table [c(0), c(3), c(1)] and indices [0, 1, 1, 2]; `size_by` through
[0, 10] → [2, 12] px gives radii 2, 7, 12 on a unit circle; three points
with two values keep `ChannelLength { points: 3, values: 2 }` and
`fig.check` reports it. The wheel figure: 5,600 samples, 52 marks, 12
markers commands of 600, 600, 200 points per wheel, 232 histogram bars;
domains vib [−0.080022, 3.053438], temp [31.701220, 65.650696], count [0,
90.3] (within 1e-4 of `dev/fixtures/wheels.json`, itself checked against
matplotlib 3.11.2's autoscaling; `python3 dev/fixtures/wheels.py` run
twice, byte-identical); `fig.check` empty at 960 × 760 and 1600 × 1200,
with and without the inset legend, in at most 4 layout passes. Measured
where the section says "record the actual numbers": 64 labels as 64
glyph runs (only "Wheel: RL" splits, "Wheel: FL" stays one) of 365 glyphs
in one font (Part I's sketch: 65 labels, 66 runs, 371 glyphs, two fonts);
the list has 428 commands in 111 parts, and lowering its markers gives
6,016 (+5,588 = 5,600 − 12; the sketch 468 → 6,056); the SVG is 378 KB
(the sketch 344 KB).

**Viewed** (vello_cpu PNGs, scratch, not committed): the wheel figure at
960 × 760 at 1× and 2× and at 1600 × 1200 with the inset legend, and
resvg's render of the snapshot SVG at 2×, against Part I's description
(shared speed axis 0–160 on the histograms only, the green temperature
axis stacked outside vibration on the outer panels, dodged and jittered
dots with dashed trend lines in their sub-bands, densities on both y
scales, dodged bars under the dashed "min. count" rule, RL in red, title
above and legend below, centred, or inset top left in FL); a 400-dot
spiral colored through viridis and sized through a sqrt handle with NaN
gaps and two notes; a 40 × 25 heatmap filling its plot area with crisp
cells, row 0 at the bottom and three transparent NaN cells.

**Benchmark** (headless, report-only): see `decisions/phase-22.md`, "Label
churn rerun with real time formats (Phase 34)".

**"Decisions this phase may meet"**: met, the wheel function's text; its
departures are D34.4 (7).

## D34.4 — Details of Phase 34b as built

**Date:** 2026-10-28 · **Context:** Phase 34, D34.3's change and steps 5–7, 41c2370 … ee179bf · **Status:** decided

### Issue

1. **`size_by`'s bound.** The section writes `S: Kind<Domain = f64,
   Output = f64>`, but `Kind` has no way to apply a scale: only
   `Position::position` and `Paint::paint` do.
2. **Channel details** the section leaves open: a channel set twice, two
   channels that both mismatch, `.fill` beside `fill_by`, `.opacity`,
   the `Markers` command's own `paint` and shape, which dots are dropped
   near the plot area, how a fitted size handle fits.
3. **Images**: whether the extent is held, what a fitted color handle
   sees, cells without a color, an extent far outside the plot area,
   snapping, a reversed x, hits.
4. **Notes**: fitting, size and color, the default anchor, the offset's
   sign, clipping, issues.
5. **`GridLength`** is listed as an error a mark keeps, but `image` takes
   a `Grid` that `Grid::new` already validated.
6. **Bar hits** (D34.3's change): which end of a span is the value end,
   bars ending outside the plot area, empty bins.
7. **The wheel function's text** (Figures, lines 1235–1383) cannot stay
   verbatim: `_ => vib` (line 1265) is a wildcard arm on a workspace enum
   (forbidden, and clippy's `wildcard_enum_match_arm` denies it);
   `Figure<'static>` (line 1239) cannot stand where the bench's
   `Data::figure(&self) -> Figure<'_>` returns borrowed figures, because
   `Figure` is invariant in its lifetime; rustfmt would reflow the
   vertical style; `trend` unwraps.
8. **Fixtures in examples.** "Tests read both JSON files through testkit's
   tolerant `Obs` compare", but rustd3-examples has no serde_json and its
   manifest is final (Phase 1's table, Phase 11's iced).
9. **The benchmark**: the window of a workload with a size of its own,
   whether `wheels` joins the verdict's `default` plan (D22.1 registered
   its cells), the label-churn time axis's data, zone and tick count.
10. **Commits**: seven code commits and the docs, not three.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `S: Position<Domain = f64>`: every `Kind<Output = f64>` of the crate (continuous, band, point, time) is a `Position`; the radius is the handle's resolved scale applied as built (`Ctx::scale`, its own range, not a panel's) |
| (2) | A later `fill_by`/`size_by` replaces the earlier channel and its error; `fill_by`'s error is reported first; `.fill(c, key)` still picks the dodge slot; `.opacity` multiplies channel colors; `paint` holds the dots' own color (unused under `paints`); sized dots stamp a unit circle followed to 1e-3 px at the largest radius; a dot is dropped when more than its own radius outside the plot area; a fitted size handle pads by the theme's margin like any numeric handle (`fig.scale` maps exactly); hits skip what draw skips |
| (3) | The extent's edges enter fitting and are held (matplotlib's `imshow` sticky edges: a heatmap fills its plot area); the values fit a fitted color handle; a cell whose value has no color is transparent; only cells reaching into the guard band are drawn (bounded coordinates, Conventions item 6; the outermost may extend up to one cell past it); edges on device pixels on raster targets (D34.3 (4)); columns flip on a reversed x as rows do on y up; no hits (D34.3); no `LargeOwned` issue for grids; an extent with an end that is not finite is ignored |
| (4) | The point enters fitting (as rules do, and Observable Plot's text; matplotlib's text does not); 12 pt (matplotlib's `font.size` at seaborn's notebook) in ink, crate-private `Theme::note`; default anchor start/baseline (matplotlib's `ha="left"`, `va="baseline"`); `.offset(dx, dy)` in px with dy down (Observable Plot's `dx`, `dy`); clipped to the plot area like every mark (matplotlib's text is not); drawn when its point is within the guard band; a band or point key outside its domain is `OutsideDomain`; no hits |
| (5) | `Grid::new` returns `GridLength`; no mark keeps it |
| (6) | The value end is a bar's second end: the value of a bar from a base, `y1`/`x1` of a span; reported only inside the plot area (edges included), as lines are; an empty bin reports at its base |
| (7) | `Source::AccelX \| Source::AccelZ => vib`; `figure<'a>`/`figure_with<'a>` return `Figure<'a>` (a `Figure<'static>` to any caller that keeps it); `#[rustfmt::skip]` with its reason keeps the vertical style line for line; `scale::ordinal(SOURCES, palette)` is `figure_with`'s parameter (proto/dlist); `trend` is private and returns no points without data |
| (8) | testkit gains `fixture::Fixture` (parse, `get(pointer)` → `Obs`, `fixture::Error`); manifests unchanged |
| (9) | `Cell::window`: the wheel figure at 960 × 760 wherever the scenario uses the standard window (the 4k scenario keeps its device px; resize starts there). `wheels` is in `smoke` and `full`, not in `quick`/`default` (`plan::REGISTERED` = the four D22.1 named), so the verdict's plan stays 28 cells and 136 runs; the README's optional runs name `--workload wheels`. Label churn: `trace` on a 10 kHz clock of instants centred on 2026-10-28T00:00Z (half the samples before midnight at any data scale), `time::utc()`, the figure's default tick count and locale (the stand-in's 16 ticks went with its formatter) |
| (10) | 41c2370 D34.3's hits; e418bbd step 5; bfcdfda testkit's fixtures; 49967c9 step 6; a56a95c the wheel figure's lifetime (found by step 7); ee179bf step 7; then the docs |

### Decision

A for all ten (architect, 2026-10-28), as built. `size_by` is bounded by
`Position<Domain = f64>`; the wheel function's departures follow the
compiler and the lints; `wheels` stays out of the default plan, so D22.1's
registered plan is unchanged.

### Why

(1), (5) and (8) are the smallest readings that compile against what
Phases 1–33 built and keep the manifests; (2)–(4) and (6) follow
matplotlib, seaborn or Observable Plot where the section points to them
and D34.3's rules elsewhere; (7) changes four lines of Figures' text,
each forced by a workspace rule or the compiler, and none of what the
figure draws; (9) keeps D22.1's registered plan intact while the wheel
figure runs everywhere else; (10) separates the crates.

### Followups

- Phase 35: colorbars over `fill_by`'s and `image`'s color handles; size
  legends are Phase 39's.
- Phase 36: hover over bars uses D34.3's candidates; notes and images
  stay without hits.
- Phase 39: value labels may build on `note`.
- If the user wants the wheel figure in the verdict's run, it is a new
  entry against D22.1 before the run.

## D34.5 — Panel titles in a row share a baseline

**Date:** 2026-10-28 · **Context:** the architect's view of the 2× wheel figure after Phase 34b (8414bd6) · **Status:** decided

### Issue

On the wheel figure, "Wheel: FL" sits 3.5 px (1×) above "Wheel: FR", and
"Wheel: RL" above "Wheel: RR". `decorate.rs` places a panel title at
`area.y0 − laid.top − title_pad`, where `laid.top` is the panel's whole
top reach, including the half-label overhang of its left axis's top tick
label; the outer panels have labelled y axes and the inner ones do not.
Titles are also anchored at their ink bottom (D16.4 left titles as they
were), so a title with a descender would sit higher than one without.
matplotlib places an axes title at the axes' top plus a pad, moved up
only by top-side x-axis decorations, with text metrics, not ink.

### Options

| | **A — one baseline per row** | B — as built |
| --- | --- | --- |
| Approach | A title's baseline sits at `y0 − title_pad − descender − top`, where `top` is the largest reach of top-side axes (ticks, labels, axis title) among the panels of its grid row; overhangs of left and right axes do not count. Reserved room stays as now (it already includes the title) | Each title over its own whole reach, ink-anchored |
| Cost | Small: the title's anchor in `decorate.rs` and its box in layout | — |
| Correctness | Titles of a row align, as in matplotlib and seaborn | Ragged titles on every grid with labelled outer axes |

### Decision

A (architect, 2026-10-28). Done as the first commit of Phase 35's
session: titles are anchored by the face's line metrics (as D16.4 does
for tick labels) and placed per grid row as above; the wheel figure's
snapshots are re-blessed after reading the diffs and viewing the render.

### Why

It is the look a reader of matplotlib or seaborn figures expects, and
cheap now, before Phases 35–39 add more figures that bless the current
placement.

### Followups

- Supersedes "Titles keep theirs" in D16.4 (1) for panel titles.
