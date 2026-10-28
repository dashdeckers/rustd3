# Phase 31 — Plot V: tiles and pan translation

## Measurements, session 31a

2026-10-28, session 31a (the two pre-step commits, Steps 1–2 and D22.4's
item 4; e1355bc …). Intel Xeon at 2.1 GHz, 2 cores, release builds for
every timing below; single runs on a shared container (±30 %).

**Before the steps.** The architect's warm-up `scripts/check.sh full`
(rebuilding `target/debug`) ran 20 min 11 s; it compiled the working tree
as this session was writing Step 1's bench module, and its only failure was
that module's first splat test (since rewritten). Harness unchanged
throughout the session: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors; nothing blessed.

**Pre-step 1 (D39.3 B, e1355bc).** `Fits::locale`, `Fits::theme` and
`Theme::get_palette` are public with docs for custom marks; the value
labels and presets call the same methods, now public; the `helpers` and
`chart` module docs drop them from what code outside the crate cannot
reach. A custom mark in `figure/tests/main/locale.rs` labels 1234.5 as
"1.234,5" under de-DE in the palette's second color through the public
paths while fitting and drawing.

**Pre-step 2 (D39.7, 2fe8e79).** Span bars and ribbons hold 0 when an end
is exactly 0 (a recording channel's least or greatest value, from its
index). Spot rows asserted: stacked spans [0, 3] [3, 4] fit [0, 4.2];
diverging spans [0, 3] [−2, 0] keep both margins, [−2.25, 3.25];
`bar_x_spans([0, 2])` fits x [0, 2.1]; a ribbon from 0 fits [0, 4.2], one
reaching 0 from below [−2.1, 0]; `ribbon_series` whose lower channel's
least value is 0 fits [0, 11.55]. No snapshot changed (no list dump or SVG
snapshot draws span bars or ribbons), so nothing was re-blessed. Viewed (the
presets example's PNGs at 2×): the stacked bar and stacked area presets
stand on the axis; the diverging bars keep both margins.

**Step 1 (1dab54b): the raster variants.** `cargo run --release -p
rustd3-bench -- raster --plan default --out target/bench-raster` (the
default plan's workloads at full size; every plot-area cell of each at
1600 × 1000 at 1 and 2 and at 1920 × 1080 at 2 (4K); one vello_cpu thread,
SIMD detected; median of 7; 1 min 41 s). `ms` sums the workload's cells;
speedup, max diff and the share of channels off by more than 16/255 are
against the cells as drawn:

| Workload | Factor | Stroke ms | Envelope ms | Speedup | Max | % > 16 |
| --- | --- | --- | --- | --- | --- | --- |
| trace | 1 | 31.4 | 4.5 | 7.06 | 254 | 1.38 |
| trace | 2 | 87.8 | 16.1 | 5.47 | 254 | 0.77 |
| trace | 4K | 121.2 | 19.3 | 6.27 | 254 | 0.72 |
| label-churn | 1 | 38.3 | 4.6 | 8.27 | 254 | 1.38 |
| label-churn | 2 | 92.3 | 15.1 | 6.12 | 254 | 0.77 |
| label-churn | 4K | 108.3 | 19.8 | 5.47 | 254 | 0.72 |
| grid16 | 1 | 76.7 | 50.6 | 1.52 | 254 | 5.70 |
| grid16 | 2 | 146.3 | 84.1 | 1.74 | 254 | 2.99 |
| grid16 | 4K | 172.6 | 91.9 | 1.88 | 254 | 3.03 |

| Workload | Factor | Markers ms | Splat ms | Speedup | Max | % > 16 |
| --- | --- | --- | --- | --- | --- | --- |
| scatter100k | 1 | 197.7 | 30.1 | 6.56 | 57 | 0.29 |
| scatter100k | 2 | 335.1 | 96.7 | 3.47 | 55 | 0.16 |
| scatter100k | 4K | 338.6 | 114.3 | 2.96 | 61 | 0.16 |
| scatter1m | 1 | 2,160.3 | 290.7 | 7.43 | 56 | 0.24 |
| scatter1m | 2 | 3,584.3 | 724.8 | 4.95 | 54 | 0.13 |
| scatter1m | 4K | 3,602.9 | 747.1 | 4.82 | 55 | 0.13 |
| grid16 | 1 | 76.7 | 38.3 | 2.00 | 59 | 0.59 |
| grid16 | 2 | 146.3 | 99.1 | 1.48 | 58 | 0.34 |
| grid16 | 4K | 172.6 | 125.9 | 1.37 | 59 | 0.32 |

Splat's first render, which also makes its 16 stamps per shape with the
figure's renderer, matches the warm renders within the runs' noise (33,
95, 121 ms for `scatter100k` at 1, 2, 4K against 30, 97, 114 warm; 289,
878, 758 for `scatter1m` against 291, 725, 747): the stamps cost a few ms.

**Viewed** (scratch PNGs of the first plot-area cell of `trace` and
`scatter100k` at 1 and 2, as drawn, as the variant and a map of channels
off by more than 2, 16 and 32, and 8× crops around the worst pixel; not
committed):

- *Envelope.* Inside the dense band the two are the same pixels; every
  difference lies on the band's outline and at the impacts. The stroke's
  outline is the antialiased edge of a 2 px pen; the envelope's is a
  staircase on whole device columns with crisp vertical steps. An impact
  (one sample) strokes 2 px wide at the sample's x, while its column's band
  grown by half the width on each side is 3 px wide: column extremes keep
  no position inside the column, so the band is a column too wide wherever
  a column holds a lone spike. 14,522 px at 1 and 32,086 px at 2 differ by
  more than 32/255, the worst by 254 (a spike's extra column).
- *Splat.* Indistinguishable at 1× and 2× in the crops; the 1,258 (at 1)
  and 3,049 (at 2) px over 32/255 are edge pixels of dots whose centre the
  nearest quarter-pixel phase moves by up to 1/8 px along each axis: up to
  a quarter of an edge pixel's coverage, 64/255 for a dark dot on white.
  With centres on quarter pixels the two agree within 4/255 (the
  compositing's rounding; `splatted_dots_on_quarter_pixels_match_filled_ones`).

## D31.1 — The raster variants: neither is built

**Date:** 2026-10-28 · **Context:** Phase 31, steps 1–2, 1dab54b · **Status:** decided

### Issue

Design, "Raster variants": a variant is built into the figure crate only if
its speedup is at least 1.5× with a max diff of at most 32/255 on the
workloads the verdict named (D22.1: `trace` and `label-churn` for lines,
`scatter100k` for dots).

### Investigation

Measurements, Step 1, and the renders viewed. Envelope fills are 5.5–8.3×
faster on `trace` and `label-churn` but differ by up to 254/255 (0.7–1.4 %
of channels over 16/255); splatting is 3.0–6.6× faster on `scatter100k`
(4.8–7.4× on `scatter1m`) but differs by up to 55–61/255 (0.13–0.29 % over
16/255). Both differences follow from the variants' geometry, not from the
implementation: column extremes lose the sample's x inside the column, and
a quarter-pixel phase moves a dot's edge by up to an eighth of a pixel per
axis.

### Options

| | **A — the section's rule: neither built** |
| --- | --- |
| Envelope | Speedup met, max diff missed on `trace` and `label-churn` at every factor: not built |
| Splat | Speedup met, max diff missed on `scatter100k` at every factor: not built |
| Step 2 | This entry only; `Thin` and `raster::Renderer` unchanged |

### Decision

A, the section's rule applied as registered: neither `Thin::Envelope` nor
`Renderer::splat` is built; Step 2's commit is this file.

### Why

The rule asks for both a clear speedup and a bounded difference; each
variant has the first and misses the second on every workload and factor
the verdict named.

### Followups

- D31.2: a finer splat (64 stamps) meets the rule.
- Phase 31's rerun (step 9) measures the widget without either variant.

## D31.2 — Splatting at eighth-pixel phases meets the rule

**Date:** 2026-10-28 · **Context:** Phase 31, step 2 · **Status:** decided

### Issue

The section defines splatting as 16 coverage stamps at quarter-pixel
phases (4 × 4), which misses 32/255 (D31.1). The same splat with 64
stamps at eighth-pixel phases (8 × 8; the bench's `PHASES` set to 8 for one
run, not committed) halves the largest phase error, and the rule then holds
on the verdict's dot workload. Building it would change the variant the
section names, which the section does not decide.

### Investigation

`rustd3-bench raster --plan default --workload scatter100k,grid16` with
`PHASES = 8`, same container:

| Workload | Factor | Markers ms | Splat ms | Speedup | Max | % > 16 | Rule |
| --- | --- | --- | --- | --- | --- | --- | --- |
| scatter100k | 1 | 210.7 | 33.8 | 6.24 | 29 | 0.025 | met |
| scatter100k | 2 | 419.3 | 85.0 | 4.93 | 30 | 0.012 | met |
| scatter100k | 4K | 402.9 | 111.8 | 3.60 | 29 | 0.013 | met |
| grid16 | 1 | 72.9 | 37.5 | 1.94 | 29 | 0.053 | met |
| grid16 | 2 | 155.2 | 106.2 | 1.46 | 30 | 0.033 | missed (speed) |
| grid16 | 4K | 172.4 | 126.1 | 1.37 | 30 | 0.030 | missed (speed) |

64 stamps of a 6 pt dot at 2× are about 64 × 19 × 19 B = 23 KB per shape
and paint-free (coverage only); making them costs a few ms once per shape
and transform.

### Options

| | **A — as the section defines it: not built** | B — build `Renderer::splat` at 8 × 8 phases |
| --- | --- | --- |
| Approach | D31.1 stands; dot-heavy cells keep vello_cpu's path per point | `raster::Renderer::splat(&mut self, on: bool)` (default off) composites solid, unsized markers in rectangular clips from 64 coverage stamps made by the renderer itself; everything else as now; agreement tests ≤ 32/255 against markers in `figure/tests/main/raster.rs` |
| Cost | — | About 300 lines in `figure/src/raster/vello.rs` and a stamp cache in `Renderer`; the replay renders the commands between markers into the pixmap with `TargetInit::SrcOver` (vello_cpu 0.3) and composites the stamps there |
| Correctness | Exact markers | Within 30/255 of exact on the measured workloads; 0.01–0.05 % of channels over 16/255 |
| Use | — | Nothing turns it on yet: the section leaves open who sets the switch (the widget's pool renderers would, in Step 7) |
| Fit | The rule was registered with the 16-stamp variant | The rule's thresholds unchanged, the variant refined after seeing its first measurement |

### Decision

B (architect, 2026-10-28): the eighth-pixel splat (64 stamps), which
meets the section's unchanged 32/255 rule on `scatter100k` at every
factor, is built in figure as a default-off `Renderer::splat` switch.
The widget turns it on for screen renders of markers commands with at
least 10,000 points (the weak machines of D22.4 gain most); file
outputs stay exact. Tests comparing the widget with a figure's PNG keep
their figures under the threshold or name the switch. Built beside
Step 7's pool (31c).

### Why

Marker raster time is what dot-heavy plot areas spend (D22.1:
`scatter100k`'s pans; 335 ms per 100k-dot cell at 2× here); tiles divide
that work but do not cut it. The finer grid removes the only reason the
variant missed.

### Followups

- If B: one `feat(figure)` commit before Step 3 or beside Step 7, with
  the switch set by the pool's renderers, and the rerun (step 9) measures
  it.

## Measurements, D22.4's item 4 (the list stage of the scatters)

222e9cb (the measurement, `rustd3-bench also`'s new row) and 0d97e24 (the
cuts). What the widget does on the UI thread per frame of a pan on the
bench's scatters at full size, 1600 × 1000 at 1: resolve with both handles
held at the pan's domains and draw the list (the probe's resolve and list
stages), then the list's bounds, its cells and the hit index the widget
rebuilds after them; release, medians of 9 frames over a warm cache, the
median of 3 runs, this container:

| Stage, ms | scatter100k before | after | scatter1m before | after |
| --- | --- | --- | --- | --- |
| resolve | 0.61 | 0.55 | 3.17 | 3.05 |
| list (`draw_resolved`) | 8.49 | 3.60 | 93.7 | 39.4 |
| the widget's `Bounds::of` | 3.07 | 0.20 | 29.9 | 1.84 |
| `list::cells` | 0.01 | 0.01 | 0.02 | 0.01 |
| the hit index (`Figure::hits`) | 5.05 | 5.39 | 77.1 | 61.6 |

Inside `draw_resolved` (instrumented once, scratch, scatter1m): the dots
marks 40 → 33 ms, `Bounds::of` 30 → 3.2 ms, the part hashes 23 → 3.5 ms.
The cuts: the dots mark writes centres (and radii and colors only when a
channel sets them) straight into the command's vectors instead of a
72-byte record per point, and skips the jitter hash of unjittered dots
(culling to the plot area was already there); marker bounds add the
shape's extent once to the points' extremes (exactly the per-point union);
part hashes take a markers command's points, sizes and paint indices a
64-bit word per FNV-1a step. A one-off probe of the dots loop: the two
scale applications per point are about 14 ns of the remaining ~33 ns per
point here.

**Against the target.** The list stage fell to 0.42× on both scatters
here. Applied to the user's machine (D22.1: resolve 0.6 + list 5.2 ms on
`scatter100k`), resolve plus list is about 0.6 + 2.2 ms, under the 8 ms
target; `scatter1m`'s list is about 51 × 0.42 ≈ 21 ms there.

**What remains for 0.2** (list building off the UI thread): on
`scatter1m` here a pan's frame still spends about 3 + 39 + 2 ms on resolve,
list and bounds, and 62 ms rebuilding the hit index (the dots' placement
again per point, then a bucket grid), which only hover reads; building the
index lazily or on the worker is the larger cut. A counting-sort grid was
tried and measured within the noise (not kept).

**`scripts/check.sh full`** at 0d97e24: green in 13 min 50 s (warm, the
benchmark smoke included), no warnings; its smoke report's "Also measured"
carries the new row (`scatter100k` list 3.55 ms, `scatter1m` 42.7 ms).
`quick` green on every commit (25 s to 1 min 1 s).

## D31.3 — Details of Phase 31a as built

**Date:** 2026-10-28 · **Context:** Phase 31, e1355bc … 0d97e24 · **Status:** decided

### Issue

1. **The raster command's workloads.** "For each workload the verdict
   named" beside `--plan default`.
2. **Its rows.** The CSV has one `ms` per workload, factor and variant;
   workloads have several plot-area cells, and `grid16` draws lines and
   dots in each.
3. **Where the variants live.** The bench has no vello_cpu (manifests are
   frozen) and the variants may not enter the figure crate before they win.
4. **Which strokes are thinned**, for the envelope.
5. **"Grown by half the line width"**: vertically only, or on every side.
6. **The splat's phase**: `proto/dlist/src/bin/stamps.rs` floors the
   sub-pixel offset to a phase.
7. **The list stage's measurement** (D22.4 (4)) has no home in the
   section.
8. **The part hash** of long marker arrays.
9. **Commits.**

### Options

| | **A — as built** |
| --- | --- |
| (1) | `raster --plan P` runs the plan's workloads (`default`: trace, scatter100k, scatter1m, grid16, label-churn) at its data size, `--workload` restricting them; the rule is read on the verdict's (`trace`, `label-churn`, `scatter100k`), the table prints it per row |
| (2) | `ms` sums the median render of each plot-area cell; `stroke` and `markers` rows are the cells as drawn (one render, listed under both names for `grid16`); speedup, max diff and the share over 16/255 are against those renders over the cells the variant changes |
| (3) | Envelope: the bench replaces each thinned stroke by a fill of its outline and renders the list with the figure's renderer. Splat: the runs of commands between markers render with the figure's renderer over a disjoint cover of where they reach (an axis spine's strip, not the cell), and the stamps (made by the same renderer) are composited in straight RGBA. An in-figure splat would render into one pixmap with `TargetInit::SrcOver` instead, so the bench's splat times are an upper bound |
| (4) | A solid stroke whose path is a polyline with at least 90 % of its vertices in subpaths spanning at least 8 device columns with at least 2 vertices per column (M4 keeps up to 4; axis ticks are many 2-vertex subpaths) |
| (5) | On every side: the band at each x takes the extremes of the columns within half the width, as a pen reaches sideways from a vertical stroke by half its width; grown vertically only, the band would leave that reach out on every steep stretch |
| (6) | The nearest quarter phase (an offset of at most 1/8 px per axis), half the prototype's error |
| (7) | A row of `also.rs` (`rustd3-bench also` prints the section; every headless run writes it): the resolve and list stages and what the widget does after them |
| (8) | A markers command's points, sizes and paint indices are hashed a 64-bit word per FNV-1a step (`Part::hash`'s doc says so); one differing word still always changes the hash, as each step is a bijection |
| (9) | e1355bc D39.3; 2fe8e79 D39.7; 1dab54b step 1; 584c5f1 step 2 (this file); 222e9cb the measurement; 0d97e24 the cuts; then the docs |

### Decision

A for all nine (architect, 2026-10-28), as built.

### Why

(1), (2) and (7) are the readings that run what the section names and keep
the table one row per variant; (3) follows the frozen manifests and the
rule that a variant enters the figure crate only after it wins; (4)–(6) are
the closest readings of the section's variants (and (6) gives the
16-stamp splat its best chance); (8) keeps the hash's guarantee at an
eighth of the steps.

### Followups

- D31.2 decides whether the 64-stamp splat is built.
