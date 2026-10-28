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

## Measurements, session 31b

2026-10-28, session 31b (Steps 3–5: d12886b, b2d0cdd, 6b9af5e, ddfd934,
ded81af). Same machine (2 cores, release builds for every timing; single
runs on a shared container, ±30 %). Harness unchanged throughout: 16
suites, 6,722 assertions, 5,292 passing, 179 floors; nothing blessed.

**Exactness (Steps 4–5; every iced test on tiny-skia and wgpu).** Tiles
against the PNG at 1, 1.25 and 2: 1/255. `pan_renders_only_entering_tiles`
on the spot checks' 900 × 600 area (30 keys with the ring): +37 px renders
no visible tile, the 5 tiles of column −2 and 7 gutter cells; −37 px the
5 tiles of column 4; at 2× on a 1792 × 1324 area, column −2's 8 rows; each
frame against a fresh render of the snapped domain 1/255.
`thinned_lines_match_a_fresh_render_after_a_pan` (pans of 20, 57, −30 and
−300 device px at 1 and 2) and `dashed_lines_keep_their_phase_across_a_pan`
(0, 20, 57, −30): 1/255, 0 channels over 16. `drag_settles_on_the_exact_domain`
at 1.25: 10.3 logical px publishes the domain of 10.4 px mid-drag and of
10.3 at the release; 1/255 against the PNG mid-drag and settled.
`zoom_out_preview_uses_the_overscan_ring` (×1.2 out about the centre,
`Render::Worker`, ring resident): missing 0 on every frame; (missing,
previews) per frame on tiny-skia (0, 15), (0, 0) at 1 and (0, 24), (0, 0)
at 2, on wgpu (0, 15) … (0, 0) and (0, 24) … (0, 0) over 5 and 6 frames.
The references compose each plot area from the list drawn with the
overscan (D31.4).

**D18.3 on the trace.** `translated_tiles_match_re_rendered_ones_on_a_trace`:
a replica of the benchmark's trace (three channels of 400,000 samples at
10 kHz, 20 s in view, two panels; the iced tests cannot depend on the
bench crate), dragged by 1, 2, 4, 37 and −300 device px at 1 and 2,
`Caching::On` (tiles translated) against `Caching::Off` (all 36, resp. 64,
tiles re-rendered at the same keys each frame): **0/255, 0 px over 2**, on
both backends. A tile renders from its key's square, which moves with the
data, so vello_cpu's origin-dependent dashes move with it; the bound is
1/255 as for the other figures. The trace against its whole-canvas render
is unchanged (also: 381 px over 2/255, max 237/255 at 2×).

**D22.1's mechanism.** `a_pan_over_a_trace_keeps_the_plot_areas_current`
(the trace replica without bottom axes, the benchmark's `pan-x` input,
`Render::Worker`, each frame followed by the worker's renders, debug
build): after the first frame, **0 of 29 frames** show outdated plot-area
content at 1 and at 2 on both backends; 6.0 and 8.6 ring tiles render
after each frame (the leading ring column, every frame: D31.6).

The real-time share, headless (`target/release/rustd3-bench run --headless
--plan quick --scenario pan-x --workload trace --factor 1,2 --runs 1
--backend B`: full data, 1600 × 1000 logical, readback every frame; one
run each; plot-area share = frames whose previews exceed the gutter cells
and overlays rendered after them, from the raw records, an upper bound):

| Backend | Factor | Frames | Frame p50 | Preview share | Plot-area share | Latency p95 |
| --- | --- | --- | --- | --- | --- | --- |
| tiny-skia | 1 | 316 | 33.3 ms | 98.1 % | 11.7 % | 11.1 s |
| tiny-skia | 2 | 88 | 150.0 ms | 100 % | 43.2 % | 12.5 s |
| wgpu | 1 | 339 | 33.3 ms | 97.6 % | 13.6 % | 9.7 s |
| wgpu | 2 | 107 | 116.6 ms | 100 % | 42.1 % | 12.5 s |

The plot areas are current on 86–88 % of frames at 1 (D22.1: 6 %); the
preview share stays at 98 % because every frame re-renders 4–10 gutter
cells and overlays that show the panned handle (the x axes' strips and
the axis lines over the plot areas' bottom edges, about 4 ms each), which
`Render::Auto`'s model sends to the worker together, so every frame shows
an outdated axis strip, and the latency is the gesture's length. At 2 the
headless readback makes frames 4–5 times longer, the pan moves about 60
device px a frame, and one worker re-rendering the leading ring column
every frame falls behind (D31.6).

**D22.4's item 3** (`rustd3-bench also`, ded81af): a 1600 × 1000 device-px
plot area of tiles drawn per frame of a pan at 1, medians of 30 frames.
The widget's `draw` takes 0.02 ms on both. Composing the frame: tiny-skia
25.1 ms for 38 images against 20.8 ms for an empty figure's frame of the
same size (one image), wgpu 14.7 against 12.8 ms (both with the headless
read-back). The tiles add about 4.3 ms on tiny-skia, 0.11 ms an image:
iced's tiny-skia engine clears and refills a frame-sized clip mask before
every raster image (`tiny_skia/src/engine.rs:532`, `adjust_clip_mask` at
`:776`), whatever its clip; the tiles themselves blit with `Nearest` at
whole pixels. Fewer images (Step 6's merging) are what cut it.

**D22.4's item 4, again:** `scatter100k` resolve 0.54 ms, list 3.41 ms;
`scatter1m` 3.06 and 38.4 ms (31a: 3.6 and 39 ms).

**Viewed.** Headless screenshots of a curve-and-dots figure and the trace
replica at 1 and 2 on both backends, at rest, 150 px into a drag, after a
jump of 400 px (beyond the ring), settled, during a ×1.2 zoom-out and
during a zoom-in: tiles seamless at full resolution; the ring fills the
leading edge; only regions without data or beyond the ring stay blank
until their tiles land (a dot cut at that boundary for one frame); the
previews are the old tiles scaled, the ring filling the zoom-out's
margins; gutters keep their last image until theirs land.

## D31.4 — The plot areas' edges: thinned lines through the overscan against the file output

**Date:** 2026-10-28 · **Context:** Phase 31, step 4–5, b2d0cdd, ddfd934 · **Status:** decided

### Issue

Testing's agreement table (rustd3.md:1856) bounds iced against the PNG at
1/255 on tiny-skia (2/255 on wgpu). Phase 31 ("Translation and overscan")
draws the widget's plot-area layers with `Held::overscan`, "and thinning
returns one sample beyond the grown view". A thinned line that crosses a
plot area's left or right edge therefore continues through the edge in
the widget, where the file output (Phase 15) thins the plot area alone and
ends the line one sample beyond its edge: the last segment before the
edge differs, and its end shows within half a line width inside the area.
Measured on the tiles tests' figures: up to 32/255 on the edge columns,
nowhere else. Both statements cannot hold for thinned lines.

### Investigation

Everything inside the area agrees within 1/255 once the reference draws
the plot areas with the same overscan (tiles, pans to −300 px, dashes,
zooms). Dots, bars, unthinned lines and grid lines agree at the edges as
before: they are clipped, not ended.

### Options

| | **A — as built** | B — the files thin with a margin | C — the widget renders edge tiles without the overscan |
| --- | --- | --- | --- |
| Approach | The widget shows the overscan list; its reference (`iced/tests/main/reference.rs`) composes each plot area from the list drawn with the overscan, the rest from the PNG; the agreement table gains the edge's row | File outputs thin over the plot area grown by one column and half the line width, clipped to the area, so a line crosses the edge as on screen | A second list per refresh without the overscan for the tiles on the left and right edges |
| Cost | — | Every snapshot of a thinned line changes at its edges (re-bless) | The list stage doubles for thinned figures; an edge tile translated after a pan no longer equals a fresh render |
| Correctness | Screen and file differ within half a line width of an edge a thinned line crosses | Screen equals file | Screen equals file at rest; pans break exactness |

### Recommendation

A: the difference is an end cap against a continuation on one or two
pixel columns, and translation needs the continuation. B is the cleaner
long-term answer if the files should match the screen exactly; it is a
Phase 15 change outside this phase.

### Decision

A (architect, 2026-10-28): with overscan a thinned line continues
through the plot area's edge where the PNG ends it; the widget's
references compose plot areas from the overscanned list, and the edge
columns' difference against the PNG (up to 32/255) is documented.

### Followups

- Testing's agreement table: a row for thinned lines at a plot area's
  left and right edges (A), or the Phase 15 change (B).

## D31.5 — Details of Phase 31b as built

**Date:** 2026-10-28 · **Context:** Phase 31, d12886b … ded81af · **Status:** decided

### Issue

Steps 3–5 met eleven calls the section does not make.

1. **What a tile renders.** The section keys tiles per plot area but not
   which commands they draw, nor how chrome over the area (axis lines on
   its edge pixels, a border, an inset legend) stays put when tiles move.
2. **tiny-skia's image positions.** iced's tiny-skia backend truncates an
   image's device position toward zero (`tiny_skia/src/raster.rs:63`), so
   tiles left of or above the widget's origin landed a pixel off.
3. **The overscan's width.** One tile, as the ring.
4. **Grids at rest.** The section holds the column grid "during a
   gesture"; outside it, the left-edge anchor.
5. **The dash anchor.** "Phase 0 at the first sample of the column
   containing origin."
6. **A tile entering the area from the ring.** "A resident tile stays
   valid when its rendered rect, shifted by d, covers its new rect;
   otherwise it renders."
7. **A tile's work** for the cost model.
8. **Pending jobs of an earlier set.**
9. **The snap.**
10. **The tests' geometry and data.**
11. **What the probe counts as current.**

### Options

| | **A — as built** |
| --- | --- |
| (1) | The list splits by part id (`iced/src/plot/layer.rs`): an area's layer is the figure's background, the panel's background, grid lines and marks (one run of commands, the marks' group whole); the chrome is every other part; the chrome without the figure's background is drawn over each area as transparent overlays on disjoint rects covering where it reaches inside the area |
| (2) | Draw positions are nudged by half a device pixel away from zero (`slot::nudged`) |
| (3) | One tile and 8 px (`tile::MARGIN`): with exactly one tile, a thinned line's end (one sample beyond the grown view) fell on the ring's outer pixels and showed after pans of more than 256 px less half a line width |
| (4) | Every plot area's x lead handle holds a grid from the first refresh, at origin 0 until a drag moves it; the figure falls back to that grid whenever the area is grown (6b9af5e), so the first frame, before the widget knows its areas, draws the same |
| (5) | The column leads by a millionth of a column as thinning's do, and the phase reaches 0 a millionth of a px past the anchor (6b9af5e): an ulp before it the dash drew a join of the segment before, 17/255 at a tile seam |
| (6) | A tile whose image, moved, covers its part inside the area but not the rest still renders, after the visible tiles, cells and overlays with the ring; meanwhile it shows that part exact and counts as current (the architect's note: a frame whose plot area is covered by resident tiles is current). Since the ring is one tile wide, every tile entering the area is such a tile |
| (7) | Its layer's work times the share of the area's pixels it covers; ring tiles count pixels only |
| (8) | A refresh that plans a new set re-sends every pending job of an earlier one at the current generation (newest wins per slot) |
| (9) | Per track, in list px: `round(offset · k) / k`; the release applies the unsnapped transforms once (`Gesture::exact`), the layout staying held until then |
| (10) | The spot checks' 900 × 600 area by sizing the window (`tiles.rs`'s `sized`); a replica of the bench's trace in the iced tests; D22.1's assertion with a worker that keeps up (a debug build cannot pace the real thing), the real-time share from the bench's headless run (Measurements) |
| (11) | `Sample::Frame` keeps its fields; a visible tile is current when its image shows the current layer and, moved with the grid, covers its part inside the area |

### Recommendation

A for all eleven.

### Decision

A for all eleven (architect, 2026-10-28), as built.

### Followups

- (6) and (3) together are D31.6's cost.

## D31.6 — The leading ring column re-renders every frame of a pan

**Date:** 2026-10-28 · **Context:** Phase 31, step 5, ddfd934 · **Status:** decided

### Issue

The section renders a ring tile over its square cut by the grown rect
(spot check: key −2's square [−475, −219) ∩ grown = [−256, −219)) and
re-renders a tile whose moved image does not cover its new rect (the −37
px row: column 4's 5 tiles). During a continuous pan the leading ring
column's squares move into the grown rect by d px every frame, so that
whole column re-renders every frame: 6.0 tiles a frame at 1 and 8.6 at 2
in the replay (two panels), against one column per 256 px of pan if the
images reached their squares. In the headless real-time run one worker
keeps up at 1 (plot areas current on 86–88 % of frames) but not at 2 (57 %).

### Options

| | A — as specified | **B — ring tiles render their whole square** | C — grow the cut by a margin |
| --- | --- | --- | --- |
| Approach | As built | A ring tile renders its full square; the overscan grows to two tiles and the margin, so the list holds the data; a moved image then covers its rect until its square leaves | The cut is the grown rect plus 64 px, the overscan as much more |
| Ring work during a pan | A column every frame | A column every 256 px | A column every 64 px |
| Cost | — | The list's thinned columns grow by 2 × 256 px (+24 % at 1600 px); the first ring renders up to twice the pixels | Between A and B |
| Spot checks | As written | −37 px renders no tile; +37 px renders column −2's 5 tiles whole | Rows change |

### Recommendation

B, decided before Step 7 (the pool and its order assume the ring's
volume). With B, steady pans render a column of tiles per 256 px; the
remaining per-frame work is the gutter cells and overlays that show the
panned handle (4–10 renders of about 4 ms here), which D22.4's item 1
(the overhead term) and Step 6's merging address.

### Decision

B (architect, 2026-10-28): ring tiles render over their whole square,
with an overscan of two tiles plus the margin, so a continuous pan
renders one new column per 256 px instead of the leading ring column
every frame; the −37 px spot check changes accordingly and the entry
records the new numbers. First commit of session 31c.

### Followups

- Steps 6–7: the gutters and overlays of a panned handle render on the
  worker under `Render::Auto` today, so the benchmark's preview share
  (which counts an outdated axis strip) stays near 98 % until they render
  inline or merge.
- **Built (31c, the session's first commit).** An area's keys are those
  whose squares meet its rect grown by one tile, as before; every tile
  renders its whole square (`tile::Grid::cover`), and the layers are drawn
  with an overscan of two tiles and the margin (`tile::OVERSCAN`, 520
  device px). A moved image always covers its square, so `Area::change`
  keeps every current tile on a translation and the entering-from-the-ring
  case of D31.5 (6) no longer occurs. The spot checks, both backends at 1
  (`pan_renders_only_entering_tiles`, 900 × 600): +37 px renders column
  −2's 5 tiles over [−475, −219) and no visible tile; −37 px renders no
  tile (column 4 lies at [987, 1243), its square); at 2 on 1792 × 1324,
  +37 px renders column −2's 8 tiles; each frame against a fresh render of
  the snapped domain 1/255, 7 gutter cells and overlays alongside. The
  trace replay (`a_pan_over_a_trace_keeps_the_plot_areas_current`, 30
  frames): 0.2 ring tiles a frame at 1 and 0.6 at 2 (31b: 6.0 and 8.6),
  0 frames outdated. `tiles_cover_the_area_once` counts (columns + 2) ×
  (rows + 2) whole squares; the ×1.2 zoom-out still misses nothing.

## D31.7 — The two-point test of a handle change has no evaluator

**Date:** 2026-10-28 · **Context:** Phase 31, step 6 · **Status:** decided

### Issue

"Invalidation by dependency and merging", rule 1: "Compare the scale a
slot was rendered with to the new one at two domain points: a constant
integral device-pixel offset is a translation, anything else a change",
and "A `Fit::Visible` y handle that refits during an x pan is a change."
The widget holds every handle as rustd3-figure's object-safe
`scale::Scale` (`figure/src/scale.rs:53-75`: `as_any`, `boxed`,
`with_range`, `pan_px`, `zoom_px`), which maps no position, and the
figure's typed readers (`Resolved::scale::<S>`) need a type the widget
does not know. D20.3 met the same gap for previews and took the
gestures' views (each lead handle's map in range shares) instead; 31b's
`Area::translation` classifies from those views. A view changes only by
a gesture: a `Fit::Visible` y handle that refits while x pans, or any
handle the app changes during a drag, keeps its view, so the area counts
as translated. Step 6's style hash does not catch it either (positions
are what it leaves out). No test or workload uses `Fit::Visible`.

### Investigation

`Scale`'s implementors (`figure/src/scale/{continuous,color,ordinal,
binned,band,point}.rs`) are generic over transforms, key types and stop
counts, so the iced crate cannot downcast to an enumerable set. The
brush conversions (D36.1, `brush::Placed::place`) carry typed function
pointers per brush, built where the handle's type is known; nothing does
so for every handle. What the widget can compare without a scale: part
hashes of the axes showing a handle (absent when no axis shows it) and
the layer's positions (a pan changes them all).

### Options

| | A — views only (as built) | **B — a position map on `Scale`** | C — axis parts as a proxy |
| --- | --- | --- | --- |
| Approach | A handle change is what the gestures' views say; a non-gesture change during a drag goes undetected | Add an object-safe `Scale::remap(&self, from: &dyn Scale, range: [f64; 2], at: [f64; 2]) -> Option<[f64; 2]>` to rustd3-figure (each impl downcasts `from` to its own type: `self.position(from.invert(p))`; `None` across types or for non-positional scales); the widget keeps each lead handle's scale per grid and calls it at the area's two edges | A handle whose axis parts' hashes changed while its view did not is a change |
| Cost | — | One trait method in figure (about 60 lines over the impls), additive; one call per lead handle per refresh | A few lines; panels without a shown axis for the handle stay undetected |
| Correctness | Stale tiles shown as current while a `Fit::Visible` y refits during an x pan, until the drag ends and the layout settles | The section's rule exactly | Partial |
| API | none | A public method on figure's `Scale` (Phase 14's trait), which the section does not name | none |

### Recommendation

B: it is the section's rule and the only one that holds without an axis;
it adds a public method the section does not make, hence this entry.
Until it is decided the widget classifies by the views (A).

### Decision

B (architect, 2026-10-28): figure's object-safe `Scale` gains the
additive public `Scale::remap`, documented for widgets, and the widget
classifies handle changes by the section's two-point test, so a
`Fit::Visible` refit during a pan is caught at once. First commit of
session 31d.

### Followups

- **Built (31d, f6e611e and 0978b38).** `Scale::remap(&self, from,
  range, at) -> Option<[f64; 2]>`, a provided method (default `None`, so
  the trait stays additive), documented with a doctest; continuous and
  time scales go through `Invert`, band and point scales give `at` when
  both place every key alike. The widget runs the test after the resolve
  on each area the views translated (`State::confirm`, `Area::keeps`):
  the lead scales of the previous refresh against this one's, at the
  area's two edges, within a 256th of a device px of the grid's move; an
  area that fails restarts its grid where it lies (`Area::restart`).
  `visible_y_refit_during_a_pan_re_renders` (a 37 px drag over a rising
  curve, y `Fit::Visible` growing from [−2.48, 52.05] to [−2.48, 55.27])
  renders all 20 tiles at 1 and 35 at 2 in the drag's frame, 1/255
  against the refitted figure on tiny-skia and wgpu; with the test
  disabled, 0 tiles and 254/255. Details in D31.10 (1)–(4).

## D31.8 — The cost model cannot time renders without the probe's clock

**Date:** 2026-10-28 · **Context:** Phase 31, step 7 · **Status:** decided

### Issue

D22.4 (1): each widget's cost model is "updated from every render
whether or not a `Probe` is attached". Conventions item 5 (rustd3.md:2566)
and CLAUDE.md: "Library code never calls `TimeZone::system`,
`Zoned::now`, `Timestamp::now`, `SystemTime::now` or `Instant::now`";
"stage timings come only from an app-supplied clock (`Plot::probe`)";
clippy's `disallowed-methods` (`clippy.toml`) enforces it. A render's
duration needs two clock readings; the widget's only times are the
`RedrawRequested` instants, a frame apart whatever the renders took.
Both statements cannot hold.

### Investigation

`iced_core::time::Instant` is `std::time::Instant` on native targets, so
any reading is the disallowed call. The pool's threads and inline renders
are timed today through `job::Stamp` (the probe's `fn() -> Instant`). The
interval between `RedrawRequested` instants includes vsync waits and the
app's own work, so it cannot stand in for a render's time.

### Options

| | A — the probe's clock only (as built) | **B — the widget reads the monotonic clock for its cost model** | C — `Plot::clock(fn() -> Instant)` |
| --- | --- | --- | --- |
| Approach | The fit (`budget.rs`) folds every render timed by a probe's clock; without a probe it keeps its seed | `Instant::now` around each render, in one function of `job.rs` behind `#[expect(clippy::disallowed_methods, reason = "D31.8")]`; the probe's clock still times what the probe reports | An app passes a clock; without one the seed stays |
| D22.4 (1) | Holds with a probe only | Holds | Holds when the app opts in |
| Convention 5 | Holds | A fourth exception beside the `OnceLock` fonts, the id counters and typst's cache; only the inline/pool split depends on it, which already depends on frame timing | Holds |
| Cost | — | A few lines | A builder and a field |

### Recommendation

B: the machines D22.4 is for (two cores, no GPU) are where apps attach no
probe; reading the clock changes nothing a test can observe except which
renders run inline, which the frame interval already decides. Until it is
decided the widget calibrates from probe-timed renders (A).

### Decision

B (architect, 2026-10-28): the `Plot` widget may read the monotonic
clock (`Instant::now`) to time its own renders for the cost model, and
nowhere else: one function, a scoped `#[expect(clippy::disallowed_methods,
reason = "…D31.8")]`, and CLAUDE.md's rule names this exception. Output
never depends on it (only where a job runs); tests that assert where jobs
run keep forcing `Render::Inline` or `Render::Worker`. Session 31d.

### Followups

- **Built (31d, 0189f1b).** `job::timed` around the bands' renders in
  `Job::run`, under `#[expect(clippy::disallowed_methods, reason = "the
  Plot widget times its own renders for its cost model (D31.8)")]`;
  `State::land` folds every render's `elapsed` into the fit. Test
  `renders_calibrate_the_cost_model_without_a_probe`;
  `ten_hz_appends_stay_sharp` forces `Render::Inline`. Details in D31.10
  (5)–(6).

## Measurements, session 31c

2026-10-28, session 31c (D31.6 and Steps 6–8: e39be1b, e7b8ed0, ffdeb6d,
550a067, b3994cb, the splat's fast path and the benchmark's rows). Same
machine (2 cores, release builds for every timing; single runs on a
shared container, ±30 %). Harness unchanged throughout: 16 suites, 6,722
assertions, 5,292 passing, 179 floors; nothing blessed.

**Before the steps.** The architect's warm-up `scripts/check.sh full`
(rebuilding `target/debug`) was running when the session began; this
session's first cargo command waited for it.

**D31.6 (e39be1b).** Recorded under D31.6 ("Built").

**Step 6 (e7b8ed0).** `unchanged_panels_are_kept`: a 37 px drag over the
left of two side-by-side panels renders the left area's entering column
(5 tiles), no tile of the right area, and 5 of the 11 gutter runs and
overlays. `style_change_during_a_pan_re_renders`: 4 (at 1) and 5 (at 2)
tiles on the translated frame, then every tile (20 and 35) once the
figure comes back in another color, 1/255 against a fresh render of the
new color at the snapped domain. `gutters_merge_into_fewer_images`: the
two panels' 13 gutter cells show as 9 images at 1, 1.25 and 2, 1/255
against the PNG. Both backends.

**Step 7 (550a067).** `threads_sets_the_pool_size`: `threads(1, 2, 3, 0,
65)` give 1, 2, 3, 1 and 64 threads named `rustd3-plot`, the pool of
another size stopped first. `tiles_near_the_cursor_render_first` (a
wheel step 60 px right of and 50 px below the area's corner; the 12 tiles
in view, ranked by distance, in arrival order): 1 thread
[0, 1, …, 11]; 2 threads [1, 0, 3, 2, 4, 5, 7, 6, 8, …] on tiny-skia and
[0, 1, 3, 4, 5, 2, 7, 8, 6, …] on wgpu, none more than one place early.
`large_marker_sets_splat_on_screen`: 9,999 dots 1/255 against the PNG,
10,000 dots 29/255. `expensive_refresh_uses_the_worker`: 6 cheap jobs
inline (8.30 ms modelled), 98 in view on the pool, the model's total
514.8 ms against 514.1 expected.

**The gutters during a pan** (the architect's question; `rustd3-bench
also`'s new rows, release). Per frame of a `pan-x` replay over the
benchmark's `trace` at full size (1600 × 1000 logical, `Render::Auto`, the
default pool of one thread here, headless without drawing, the pool
drained until 300 ms of silence after each frame), means of 40 frames
after the press:

| Backend | Factor | Inline tiles | Inline cells | Inline ms | Pool tiles | Pool cells | Pool ms | Frames with every cell inline |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tiny-skia | 1 | 0.2 | 5.5–6.0 | 2.0–3.4 | 0.0 | 0.0–0.1 | 0.00 | 95–100 % |
| tiny-skia | 2 | 0.8–0.9 | 5.9–6.0 | 4.0–4.3 | 0.0–0.1 | 0.0 | 0.01–0.06 | 98–100 % |
| wgpu | 1 | 0.2 | 6.0 | 1.8–2.3 | 0.0 | 0.0 | 0.00 | 100 % |
| wgpu | 2 | 0.9 | 5.8–6.0 | 3.9–4.2 | 0.0 | 0.0–0.1 | 0.00–0.01 | 98–100 % |

(ranges over three runs of `also`). The cells are the gutter runs and the
overlays that show the panned handle; with merging they are about 6 a
frame, and the fitted cost model, which learns their per-render
overhead, renders them inline. One axis gutter render, single-threaded,
medians of 15 (the strip under the bottom plot area, 22 glyph runs, and
the overlay over the area's bottom edge, two device rows):

| Render | As drawn | Without glyphs | Without strokes | No command (fixed) |
| --- | --- | --- | --- | --- |
| Strip at 1 (1389 × 59 px) | 0.36–0.39 ms | 0.22–0.23 | 0.27 | 0.14 |
| Strip at 2 (2778 × 117 px) | 1.02–1.10 ms | 0.81–0.83 | 0.82–0.83 | 0.57–0.58 |
| Edge overlay at 1 (1389 × 2 px) | 0.10 | 0.09 | 0.02 | 0.01 |
| Edge overlay at 2 (2778 × 2 px) | 0.19–0.26 | 0.18 | 0.04 | 0.01 |

Glyphs cost about 0.15 ms of a strip at 1 and 0.2–0.3 ms at 2; the
fixed cost of a render of the strip's size (clearing, packing and
un-premultiplying its pixels) is the largest part at 2; strokes (the spine
and ticks) are what an overlay costs. Glyph rasterization does not
dominate, so no glyph decision is raised. 31b's "about 4 ms each" was
measured under the headless run's readback on two cores.

**The headless real-time run** (`target/release/rustd3-bench run
--headless --plan quick --scenario pan-x --workload trace --factor 1,2
--runs 1 --backend B`, as 31b; report-only):

| Backend | Factor | Frame p50/p95/p99 | Latency p50/p95 | Preview share | Inline |
| --- | --- | --- | --- | --- | --- |
| tiny-skia | 1 | 33.4/50.1/59.3 ms | 21/47 ms | 0.3 % (31b: 98.1 %) | 96 % |
| tiny-skia | 2 | 166.6/200.0/253.4 ms | 120/657 ms | 31.5 % (31b: 100 %) | 51 % |
| wgpu | 1 | 33.3/33.4/50.0 ms | 17/33 ms | 0.6 % (31b: 97.6 %) | 96 % |
| wgpu | 2 | 83.4/116.7/208.5 ms | 73/602 ms | 35.9 % (31b: 100 %) | 55 % |

At 1 a pan is current on all but a few frames, its latency a frame or
two (31b: the gesture's length, 9.7–12.5 s). At 2 the headless readback
of 3200 × 2000 px makes frames 83–167 ms long, so the replay's moves
between two frames add up to 50–100 logical px (100–200 device px): a
ring column (256 px) enters every one to three frames, the visible area
outruns the ring's renders on the pool, and those frames show previews.

**D31.2 in the widget** (`also`'s new row; the `scatter100k` plot area at
1600 × 1000, single-threaded, medians of 5): filled 214.6 ms, splatted
34.6 ms (6.20×) at 1, max 29/255; filled 336.9 ms, splatted 100.2 ms
(3.36×) at 2, max 30/255. The first build composited every covered pixel
with the general blend (3.96× and 1.80×); a fully covered pixel of an
opaque color now takes the color as it is, the same pixels.

**`scripts/check.sh full`** at 3543c0f: green in 18 min 47 s (warm, the
benchmark smoke included), no warnings. `quick` green on every commit.

**Viewed.** Headless screenshots (scratch, not committed) of the trace
replica at 800 × 500, factors 1 and 2, `threads(1)` and `threads(4)`,
both backends: after 8 drag frames of 37 px with 4 ms of results between
them, and right after a wheel zoom-out: tiles seamless, the axis strips'
labels on the ticks the data shows, no blank or outdated tile, the
zoom-out's margin beyond the data's end blank as the data ends there.

## D31.9 — Details of Phase 31c as built

**Date:** 2026-10-28 · **Context:** Phase 31, e39be1b … · **Status:** decided

### Issue

Steps 6–8 met calls the section does not make.

1. **The style hash's content.** "paints, stroke styles, dashes, marker
   shapes, glyph ids and fonts".
2. **Dependency sets** for merging.
3. **When runs are formed.**
4. **A merged run's probe target.**
5. **Overlays** (D31.5 (1)) beside merging.
6. **Part ids**, whose value rustd3-figure keeps private.
7. **The inline decision with a pool**, and "Phase 19's prediction divides
   the modelled time of the work sent to the pool by n".
8. **"The zoom anchor or cursor"** that orders the tiles.
9. **The fitted cost model** (D22.4 (1)): its form.
10. **The pool's restart** ("changing it restarts the pool").
11. **Which renders splat** (D31.2: "screen renders of markers commands
    with at least 10,000 points").
12. **Verify's checks**: where they run, what they report.
13. **Test placement.**

### Options

| | **A — as built** |
| --- | --- |
| (1) | Per command: its kind, paints (a solid color's components; a gradient's stops and interpolation, not its geometry; an image brush's pixels and sampler), a stroke's width, joins, caps and dash pattern (not its offset, which the column grid moves), a marker's shape, whether it is sized and its paint table (not the per-point arrays, which points entering the overscan change), a run's font, size and glyph ids, an image's pixels; equal consecutive commands folded once, so a line split into more runs by a NaN gap keeps its hash. Consequence: a change of the per-point sizes or paint indices alone during a drag is not seen until the drag ends |
| (2) | The handles of the axes whose ticks and labels (part role 4) reach into the cell, by `Resolved::axis`, and the legends (role 7) as themselves; spines, borders, titles, headings and the background change with the layout alone |
| (3) | When the partition is built (the rects change); during a gesture the layout holds, so they hold; a run whose cells change apart renders more pixels than it must |
| (4) | `Target::Cell(first cell)` with the pixels of the whole run (the probe docs say so); the plot tests read a run's cells back from its pixels |
| (5) | Not merged: they render from the chrome without the figure's background and draw over tiles |
| (6) | Decoded by a binary search over `part::Id::new(v) <= id` (64 comparisons) |
| (7) | Render::Auto renders inline, in the pool's order, each job whose predicted time still fits half the frame budget with those before it (the rest to the pool); nothing reads a prediction of the pool's work, so it is not computed |
| (8) | The position of the last gesture input (wheel, press, drag move), list px; each area's centre before one; the ring's tiles are ordered the same way |
| (9) | `t = o + p · px / 65,536 + w · work / 256`, exponentially weighted least squares (each render 1/16 of the fit), a prior holding each coefficient at its seed (o = 0, p and w from the constants) with the weight of a tenth of a render, coefficients clamped at 0; timed renders only (D31.8) |
| (10) | At the pool's next jobs: the old pool stops (its threads finish their job, every result lands), a pool of the new size starts |
| (11) | Per job: the tiles of a plot area whose layer holds a markers command of at least 10,000 points render with `Renderer::splat(true)` (every splattable command of that layer splats); gutters and overlays never; inline renders too |
| (12) | After the real jobs, inline under `Render::Inline`, on the pool otherwise; a check is a job for the image's own generation and content, landing as a comparison; checks report no `Raster` sample; a refresh whose plan is empty still checks; the benchmark names the variant (its flag is Step 9) |
| (13) | Step 6's tests in `iced/tests/main/classify.rs`; the pool's in `pipeline.rs`; `newest_result_wins_per_tile` and `verify_replaces_a_differing_image` as unit tests (`area.rs`, `state.rs`); `verify_finds_translated_tiles_exact` beside Step 6's |

### Recommendation

A for all thirteen.

### Decision

A for all thirteen (architect, 2026-10-28), as built.


## Measurements, session 31d

2026-10-28, session 31d (D31.7, D31.8, Steps 9–10: f6e611e, 0978b38,
acb8547, 0189f1b, e7c8c8c, then the docs). Same machine (2 cores, release
builds for every timing; single runs on a shared container, ±30 %).
Harness unchanged throughout: 16 suites, 6,722 assertions, 5,292 passing,
1,430 skipped, 179 floors; nothing blessed.

**Before the steps.** The architect's warm-up `scripts/check.sh full`
(a cold `target/debug`) ran 34 min 9 s, green; this session's first cargo
command waited for it. Free disk after it: 11–12 GB.

**D31.7 (f6e611e, 0978b38).** Recorded under D31.7 ("Built"). The
tolerance was measured before it was chosen: a linear domain near 1e9 at
16 px per unit panned by 5 px moves its edges by 5.0000019 and 4.99999999
px (2e-6 px of rounding), so a tolerance of a millionth would call such a pan a change; the
256th chosen is the visibility bound (D31.10 (2)). Every iced test passes on tiny-skia and
wgpu; the spot checks of 31b and 31c (`pan_renders_only_entering_tiles`,
the trace replay) are unchanged, so pure pans still translate.

**D31.8 (0189f1b).** Recorded under D31.8 ("Built"). One pre-existing
race surfaced in `check.sh quick` and was fixed first (acb8547):
`threads_sets_the_pool_size` counted the pool's threads by name right
after it started, and std names a thread from inside it, so it once saw
1 of 3.

**Step 9 (e7c8c8c).** Headless, release:

- The wheel workload's smoke cells (`--workload wheels`, tiny-skia): max
  generation per run, before (the warm-up's smoke) and after: `pan-y` 1
  → 47, `zoom-lines` 1 → 30, `zoom-pixels` 1 → 31, `4k` 1 → 8 (its zoom
  half), `pan-x` n/a (was 1), `resize` 5 → 5, `hover` and `cold` 1.
- A reduced `weak` plan (`pan-x`, `zoom-lines` on `trace`, 2 runs, data ÷
  100, tiny-skia, `--threads 1`) with `--caching verify`: the report opens
  "Weak profile: no GPU (tiny-skia), 2 cores", holds the cells to 2 T,
  150 ms, 250 ms and 25 %, ends with the weak outcome line; Flags: "every
  kept image matched a fresh render within 1/255 over 4 runs".
- **Verify is a diagnostic, not a timing mode.** The same `pan-x × trace
  × 2` with `--caching on` and `verify` (headless, 2 cores, one pool
  thread): latency p50/p95 119/641 against 4,003/9,125 ms, previews 35 %
  against 98 %, inline renders 54 % against 13 %, raster critical path
  5.7 against 202 ms, CPU 1.03 against 1.89 s/s. The checks (every kept
  tile and run, each refresh) keep the pool thread and the second core
  busy; the real jobs come first in each set but wait behind the check in
  progress, and the cost model, fed by every render (D31.8), learns the
  contended times and sends more of them to the pool.

Windowed under Xvfb (1280 × 1024 × 24, no window manager, wgpu on
lavapipe), what the user runs:

- `HZ=60 FACTOR=1 xvfb-run -a dev/bench/run.sh --smoke-only`: green in
  6 min 2 s; 53 runs (60 cells, 7 n/a), every window as asked (X without
  a window manager clamps none, `4k` included), window factor 1; the
  wheel cells move windowed too (`pan-y` 178, `zoom-lines` 98,
  `zoom-pixels` 111, `4k` 12).
- The script's whole control flow, with a temporary copy whose plans
  were cut to a few cells (not committed): smoke, default (`pan-x` at 1
  and 2, `4k` at 1), `weak-nogpu`, "Skipping weak-4cores: this machine has
  2 cores.", `weak-2cores` under `taskset -c 0-1` (its `meta.json` says 2
  cores); each into its own directory with its own log; the summary
  printed each report's outcome; 6 min 13 s. The factor cell at 2 opened
  800 × 500 logical px, 1600 × 1000 device px (D22.3).
- `4k`'s "n/a: display too small": a copy of the smoke run whose `4k ×
  trace` header records a 1920 × 1004 device px window, rebuilt with
  `rustd3-bench report`: the budget, metrics and summary rows read "n/a:
  display too small", Flags name the size, no budget of the cell is
  evaluated.
- `rustd3-bench list`: `weak` 5 cells, 25 runs, about 6 min a profile;
  smoke about 6, default about 46: about 75 minutes with the
  also-measured items after each plan.

**Viewed.** The reduced runs' reports (`report.md` of smoke, default and
both weak profiles) read as described above; nothing else was rendered
this session.

## D31.10 — Details of Phase 31d as built

**Date:** 2026-10-28 · **Context:** Phase 31, f6e611e … e7c8c8c · **Status:** decided

### Issue

D31.7, D31.8 and Step 9 met calls neither the section nor the decisions
make.

1. **`Scale::remap` on band and point scales**, which have no inverse,
   and on color scales.
2. **The translation's tolerance**: "a constant integral device-pixel
   offset" in floating point.
3. **Which scales are compared, and against what.** "The scale a slot
   was rendered with" against "the new one"; D31.7: "each lead handle's
   scale per grid".
4. **Where a failed test leaves the grid**: the list was already thinned
   on the moved grid's columns when the test runs, after the resolve.
5. **What D31.8's clock times**, and which renders feed the fit.
6. **The tests whose render path the fit now decides.**
7. **`--threads` and `--caching verify` in the benchmark**: range, record,
   report.
8. **Which cells are D22.3's "factor cells"**, and the wheel figure there.
9. **`4k`'s "n/a: display too small"**: from which size, and what the
   report does with the cell.
10. **The weak plan in the bench**: its budgets, its report section and
    outcome, and how the report knows the profile.
11. **The desktop outcome lines** now that Phase 31 is built.
12. **`run.sh`**: order, failures, and a machine without `taskset`.
13. **The wheel workload's gestures.** No panel of the wheel figure zooms
    x (its x is a band scale).

### Options

| | **A — as built** |
| --- | --- |
| (1) | A provided method returning `None`, so the trait stays additive for implementors outside the crate; continuous and time scales go through `Invert` (`scale::through`); band and point scales give `at` when the two, set to `range`, are equal, else `None` (they do not pan, so any difference is a change); color scales keep the default |
| (2) | Within a 256th of a device px of the grid's move, at both edges (`area::TRANSLATED`): an antialiased edge's coverage then moves by at most 1/255 (a half pixel: up to 109/255); the rounding through the domain is far below (2e-6 px measured near 1e9; a time scale's whole nanoseconds) |
| (3) | Each axis's lead handle (`region::Map::lead`: the first handle the panel zooms that way, else its first axis's), as the previous refresh resolved it, against this refresh's, each set to the area's range in list px (x left to right, y bottom to top) and compared at the area's edges; only on areas the views translated (a still area's content hash already re-renders a refit). A `Fit::Visible` y on no axis and not zoomed leads nothing and stays unseen during a drag, as before |
| (4) | The grid begins anew where it lies (`Area::restart`): its origin, density and the views it began at stay, so the thinned columns stay the grid's and the next drag step measures from them; every key new, the old tiles the preview layer, mapped by the views |
| (5) | `job::timed` around the bands' renders in `Job::run` (the splat switch and the probe's stamps outside it); every render feeds the fit, the pool's and `Caching::Verify`'s checks included (measured: under Verify the contended checks lower the inline share, Measurements 31d); the probe still reports its own clock's durations |
| (6) | `ten_hz_appends_stay_sharp` forces `Render::Inline` (the architect's note), the seed model's prediction for an append (≤ 8.35 ms) still asserted. No other test asserts where `Render::Auto` runs a job after the first frame, whose partition always uses the seed |
| (7) | `--threads N` takes 1 to 64 (the widget's clamp), is passed to every child, recorded in `meta.json` (`threads`, absent in older runs) and the report's first line; `--caching verify` records each `Sample::Mismatch` as a `mismatch` record (frame, cell or tile, max), which the report's Flags count |
| (8) | A cell whose factor is not the native one; the wheel figure keeps its own 960 × 760 at every factor, as before (it is in `smoke` and `full` only) |
| (9) | Windowed only: a `4k` cell any of whose runs recorded a device size (the window's reported logical size × the cell's factor) more than 1 px short of 3840 × 2160 reads "n/a: display too small" in the budget, metrics and summary rows; its budgets are not evaluated (so they cannot decide the outcome); Flags name the size |
| (10) | `Plan::Weak` (5 cells, 25 runs); `rule::Profile` (`Desktop`, `Weak`) gives each budget's limit, the plan's name picks it; a weak report opens a "Weak profile: …, N cores" section (no GPU for tiny-skia, else the backend and GPU; cores from `available_parallelism`, which follows `taskset`) with the registered budgets (`rule::WEAK`), and ends with the profile's outcome line: met; missed on raster/upload or on the UI thread → Phase 31 work while open, else a 0.2 item (never the GPU-layer question); or inconclusive |
| (11) | The desktop lines read as the rerun: "image path sufficient — Phase 31 built, no GPU data layer"; "still raster/upload bound after Phase 31 → GPU data layer recommended"; "UI-thread bound → move list building off the UI thread (0.2); no GPU data layer" |
| (12) | After the default plan, whatever its status: `weak-nogpu` (`--backend tiny-skia`), then `weak-4cores` and `weak-2cores` under `taskset -c 0-3` and `0-1`, each skipped with a message when `nproc --all` is smaller or `taskset` is missing; the script exits with a failure's status; its summary prints every report's outcome |
| (13) | Per-workload gesture anchors (`scenario::Anchors`, plain data): the wheel figure's wheel and drags act near the middles of its front-left and rear-left scatters, which zoom vibration and temperature along y; `pan-x` is n/a on it (`Workload::pans_x`) and `4k`'s pan half moves nothing there; replays are named by workload too |

### Recommendation

A for all thirteen.

### Decision

A for all thirteen (architect, 2026-10-28), as built. One addition:
`Caching::Verify`'s check renders do not feed the cost model's fit (they
run at background priority and are not the work the model predicts);
a one-line change in Phase 40's session. Verify stays a diagnostic mode
whose timings are not meaningful.

