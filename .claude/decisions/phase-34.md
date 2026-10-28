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
