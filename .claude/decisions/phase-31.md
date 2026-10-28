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
