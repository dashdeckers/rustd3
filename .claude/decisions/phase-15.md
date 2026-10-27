# Phase 15 — Marks and file outputs

## Measurements

2026-10-27; `scripts/check.sh full` green at a49c764 in 3 min 8 s
(warm; the figure tests with all features 98 s of it). Phase 15's core
change (`Points` readers and `indexed`,
D15.1 (1)) adds no harness suite and changes no d3 test: 9 suites, 3,314
assertions, 2,623 passing, 691 skipped, 80 floors, before and after;
nothing blessed.

**The trace spot checks** (`figure/tests/main/thin.rs`):
`signal(1_000_000, 42)` on an explicit clock in the prototype's view (x
over [0, 100] s, y over [−2.4, 2.4], a plot area of exactly 1000 × 240
px, a 1 px stroke in (30, 60, 140)). The drawn path equals, point for
point, core's M4 over the edges the x scale's inverse gives, and:

| Target | Columns | Points | Polylines | Impacts | Performance |
| --- | --- | --- | --- | --- | --- |
| `Screen { scale: 1.0 }` | 1,000 | 3,947 | 1 | 50 of 50 | 3,949 |
| `Vector` | 4,000 | 15,427 | 1 | — | 15,429 |

The two points come from the edges: Performance used `100 × c / n`; the
scale's inverse (`d0 × (1 − t) + d1 × t`, t = c / n) rounds 246 of the
1,001 edges at 1,000 columns an ulp away from those (984 of 4,001 at
4,000), and at 10 samples per px 127 of them (493) fall exactly on a
sample, whose column changes. Every bound of the spot check holds (≤ 4 per
column + 2 per run + 2 neighbours, all impacts).

**Fidelity**, vello_cpu (single-threaded, baseline SIMD) rendering the
plot area at 1× against the same figure with `Thin::Off` (1,000,000
points in one path):

| Target | Pixels off by > 16/255 | Bound | Columns whose top or bottom moves > 1 px | Bound |
| --- | --- | --- | --- | --- |
| `Vector` (4 columns per px) | 0.81% | 5% | 1.1% | — |
| `Screen { scale: 1.0 }` (1 per px) | 1.88% | 12% | 2.1% | 3% |

Performance's 3.8% and 10.3% were tiny-skia's: tiny-skia rendering both
sides here gives 4.0% and 10.9%. The column ends are taken where a pixel
is at least half covered (D15.1 (11)).

At 2× (`the_trace_at_2x_keeps_the_bounds_per_device_pixel`), with the
1 px line and with the theme's 2 px line (D15.3):

| Line | Target | Pixels off, vello / tiny-skia | Columns off, vello / tiny-skia |
| --- | --- | --- | --- |
| 1 px | `Screen { scale: 2.0 }` | 4.32% / 4.54% | 24.3% / 32.8% |
| 1 px | `File { scale: 2.0 }` | 0.59% / 2.45% | 9.8% / 31.7% |
| 2 px | `Screen { scale: 2.0 }` | 0.75% / 0.86% | 2.7% / 2.85% |
| 2 px | `File { scale: 2.0 }` | 0.32% / 0.32% | 0.6% / 1.55% |

The test asserts the 2 px rows against the 1× bounds (5%, 12%, 3%).

**Example agreement** (exit criterion 2): resvg of each example SVG at the
PNG's scale (2) against the PNG: scatter 0.0055, trace 0.0275 (D15.2).
The trace SVG is thinned at 4 columns per list px and the PNG at 4 per
device px; vello_cpu renders the two lists 0.0078 apart, resvg and
vello_cpu the same list 0.0258 apart.

**Timings** (2-core container, reported only): in debug the figure tests
take about 90 s with all features, most of it the three fidelity tests
(two million-point paths drawn and rendered once each, shared). In
release, `--example trace` (three channels of 1M samples, SVG and a
1920 × 1200 PNG) runs in 1.2 s and `--example scatter` in 0.5 s; the
release build of the examples took 11 min 42 s cold.

## D15.1 — Statements of Phase 15 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 15, 507178d … a49c764 · **Status:** open

### Issue

1. **Generic marks cannot read `Points`.** `dots`/`line` are generic over
   `<X as Kind>::Domain` (`Clone + 'static`), but `Points::get` and
   `Points::iter` (Phase 9) require `X: Copy, Y: Copy`, so a mark over
   band keys such as `String` could not read its data. And the bridge
   (`Indexed::series_extent`, `Indexed::thin`) takes `&Points<'_, D,
   f64>` while a generic mark holds `Points<'a, X::Domain, Y::Domain>`;
   the section's `series_extent(&points, None, &|v| y.admits(&v))` passes
   an `f64` where `admits` takes `&<Y as Kind>::Domain`.
2. **Frame and memo cannot both sit behind `Indexed::thin` as built.**
   Step 1 of Thinning builds `Columns` "inside the bridge", step 2 caches
   them per draw in `thin::Frame`, and the memo needs the recording, its
   revision and the channel; Phase 14's `Indexed::thin(points, edges,
   valid)` takes no frame, and generic mark code can neither build
   `Columns` nor read a recording id (both need `X: Coordinate`). `Mark::draw`
   takes `&mark::Ctx` (and Phase 17's `hits` and `issues` do too).
3. **The memo key.** `(recording id, revision, channel, grid)` gives a
   channel drawn on a linear y and on a log y in one figure (same x grid)
   the linear panel's M4 output: the log panel's minima then fail to place
   and break its line.
4. **Rule labels.** Part I and exit criterion 3 want every label shaped in
   resolve's batch; `Mark` has only `fit` and `draw`, so nothing reports a
   mark's label before the batch.
5. **Lifetimes.** `Rule<'a, S>`, `rule_x<'a, X>` and `mark::Color<'a>`
   name a lifetime nothing in them borrows (E0392 without a
   `PhantomData`).
6. **Commits.** Steps 1–3 leave colors and the guard band without a
   caller (the workspace denies dead code); the figure tests' snapshots
   and spot checks need the examples' figures before Step 7.
7. **Where the example figures live for tests.** `rustd3-examples` has no
   resvg (exit criterion 2 needs it) and `rustd3-figure` cannot depend on
   the examples; no manifest may change.
8. **Features.** `draw`, `save_svg` and `save_png` make a fresh
   `figure::Cache`, whose `Default` exists only with `fonts` (D13.1 (6));
   `figure::Error` needs a public path.
9. **Defaults the section leaves open**: a rule's width, color, label size,
   color and anchor, a rule outside the plot area; dots' opacity; invalid
   builder input; where a mark sits on a band scale (`Position::position`
   is the band's start); the group when no mark draws; the dots' circle.
10. **Curves and the guard band.** "Before building paths, segments are
    clipped": the curve then runs through the clipped vertices.
11. **The column metric.** "Columns whose top or bottom is off by more
    than 1 px" has no stated ink threshold, and `proto/thin2` keeps no
    code for it. With any visible ink (> 16/255) a column moves by a whole
    impact's length whenever a 1 px impact line's fringe crosses into it:
    5.6% of columns (worst 74 px) against the bound of 3%.
12. **Counts.** The spot check's parenthesised 3,949 and 15,429 points are
    3,947 and 15,427 here (Measurements).

### Investigation

- (1), (5): compiled; `core/src/series/points.rs` before 507178d.
- (2): the prototype has no thinning in its figure model (`proto/figmodel`);
  `proto/cols` measures shared boundaries only.
- (3): `a_channel_on_a_log_and_a_linear_y_is_thinned_for_each`.
- (11): measured with vello_cpu at thresholds 16, 64, 100, 112, 120, 128,
  160 (columns off 56, 33, 25, 21, 20, 20, 20 of 1,000), and with
  tiny-skia on both sides (40 at 16, 35 at 128, on a grid 0.27 px off the
  columns); zoomed renders show the fringe.

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | `Points::get`/`iter` take `Clone` (semver-compatible, no call site changes); core gains the bound-free `Points::indexed() -> Option<Points<'a, X, f64>>`; marks turn a channel's `f64` into the y domain through an `Any` downcast (no API) | `Copy` bounds on every mark: no `String` band keys |
| (2) | `Indexed::thin(points, edges, valid, frame: &mut thin::Frame)`, edges increasing or decreasing (a reversed x domain; the bridge reverses them). `thin::Frame` (public, opaque) holds the draw's columns, keyed by recording, revision and the edges' FNV-1a hash, and the memo, which `figure::Cache` lends it for the draw; `Ctx` holds the frame in a `RefCell` | Bridge methods for each piece (source, columns, M4): three methods instead of one parameter |
| (3) | The key adds the validity test's fingerprint: its answers on 18 probe values (±∞, ±1e300, ±1e10, ±1, ±0.5, ±1e−10, ±1e−300, ±0, 2, NaN), which tell apart every admission set the figure's scales have (all finite, positive, negative, the open unit interval) | The section's key: wrong lines for the same channel on two y scales |
| (4) | Crate-private `Fits::label(key)` (with `Fits::theme`), shaped in resolve's batch; `Ctx::label(&key)` reads it while drawing. User marks report no labels (no public API added) | A public `Mark::labels` with a default; or shaping at draw time, outside the batch |
| (5) | `Rule<S>`, `rule_x<X>`, `rule_y<Y>`; `mark::Color` crate-private, without a lifetime | `PhantomData<&'a ()>` |
| (6) | Ten commits: core; dots and rules (with colors and marks in `draw_with`); line (with the guard band); thinning; the examples' figures; draw and `Target::File` (snapshot, trace spot checks); a doc-link fix; save (snapshot, `figures_equal`); the 2× fidelity test; the mains (resvg agreement) | Seven with transient `#[expect(dead_code)]` |
| (7) | The figure tests compile `dev/examples/src/{scatter,trace}.rs` by `#[path]`; parameters (`SIZE`, `SCALE`, `POINTS`, `SAMPLES`) are constants there, shared by the mains and the tests. `check.sh quick` reruns the figure tests only when `figure/` changes; `full` covers an examples-only change | Copies of the generators in the figure tests |
| (8) | `draw`, `save_svg`, `save_png` need `fonts` (`save_png` also `png`); `figure::Error` is `rustd3_figure::Error`, as `Cache` (D14.1 (3)) | — |
| (9) | Rules: the theme's line width (1.5 pt, matplotlib's `axhline` takes `lines.linewidth`), the palette's first color (`C0`), a label at tick-label size in the rule's color at (w − tick pad, y − tick pad) anchored end/bottom (`rule_y`) or (x + tick pad, tick pad) start/top (`rule_x`), inside the clip; nothing when the value is not placed or lies more than half the width outside. Dots: opacity 1 (matplotlib's). Builders ignore input that is not finite, positive or a valid dash pattern, and clamp opacity, as `Theme`'s builders do (Themes). Marks sit at the middle of a band. No `Push`/`Pop` when no mark draws. Circles to 1e−3 px | — |
| (10) | Clip the vertex polyline, then run the curve; a curve bends differently only where a vertex lies more than the panel's size outside; a closed curve closes each clipped run | Clip the curve's Béziers (subdivision) |
| (11) | Ink = a pixel at least half covered (the red channel half-way from white to the line's 30): 2.1%, against the prototype's 2.2% | Any visible ink: 5.6%, the row fails |
| (12) | Asserted as measured, with the cause in the test | — |

### Decision

Recommended: A for all twelve, as built. The two public-API choices
beyond the section are (1) the core additions (`Clone` readers and
`Points::indexed`) and (2) `Indexed::thin`'s frame parameter with the
public `thin::Frame`.

### Why

(1) and (2) are the smallest changes under which the section's own
design compiles: generic marks reach the index only through the x scale,
and the frame has to be where `Columns` can be built. (3) keeps the memo
exact for every admission set the figure has. (4) keeps one text batch
without a public trait method. (5), (6), (8) follow the compiler and the
workspace's lints; (7) tests exactly what the examples draw; (9), (10)
are the readings closest to matplotlib and to the section; (11) measures
where a reader sees a band end.

### Followups

- Phase 17: `Mark::hits` for thinned lines calls `Indexed::thin` again
  through the draw's frame (a memo hit, no M4 query); `fig.check` may
  report labels typst rejects through `Fits::label`'s keys.
- Phase 33: the time scale's `Indexed::thin` hashes its edges' bits
  (`i128` nanoseconds) and calls `thin::Frame::lines`.
- Phase 34: `Indexed::thin_joint` takes the frame as `thin` does; the
  note mark reports its text through `Fits::label`.
- Phase 31: anchored grids change the frame's and the memo's grid key.

## D15.2 — The trace example's SVG and PNG agree within 0.03, not 0.02

**Date:** 2026-10-27 · **Context:** Phase 15, exit criterion 2 · **Status:** open

### Issue

Exit criterion 2: "resvg of each SVG vs its PNG has tile score ≤ 0.02".
The scatter's is 0.0055; the trace's 0.0275.

### Investigation

At the PNG's scale 2: vello_cpu renders the SVG's list (4 columns per
list px) and the PNG's list (4 per device px) 0.0078 apart; resvg and
vello_cpu render the very same list 0.0258 apart. The worst tile (list
px (432, 352), bottom panel) holds four impacts: each a stroke up and
back down within a quarter px, overlapping itself; the two renderers shade
the seam inside such a stroke differently (zoomed renders). Same list,
resvg against vello_cpu: 0.0346 at 1×, 0.0289 at 1.5×, 0.0258 at 2×,
0.0169 at 3×.

### Options

| | **A — 0.03 for the trace** | B — compare at 3× | C — keep 0.02 |
| --- | --- | --- | --- |
| Approach | `resvg_renders_the_example_svgs_as_their_pngs` asserts the scatter ≤ 0.02 and the trace ≤ 0.03 | The example's PNG at 3×: 0.0251 against the SVG, still over | Change what the trace draws |
| Correctness | The gap is the renderers' antialiasing of self-overlapping strokes, not our output | Does not pass either | Nothing in the trace is wrong |

### Decision

Recommended: A, as built.

### Why

The renderers disagree on the identical list by more than the bound; the
thinning densities add 0.0017.

### Followups

Phase 37's PDF bound (poppler against vello_cpu) will meet the same
impacts if it measures on the trace.

## D15.3 — A 1 px line thinned at one column per device px stripes at 2×

**Date:** 2026-10-27 · **Context:** Phase 15, the architect's 2× check · **Status:** open

### Issue

The spec's bounds are measured at 1×. At 2× on screen (`Target::Screen
{ scale: 2.0 }`, one column per device px, as the section fixes), the
trace's 1 px line (2 device px) shows light vertical stripes through its
dense band: 24.3% of columns have a half-covered end more than 1 px off,
against 3% at 1× (pixels off stay within 12%: 4.32%). tiny-skia draws
the same stripes (32.8%), so they are M4's zigzag, not a renderer's.
With the theme's 2 px line (4 device px) the stripes go: 2.7% of
columns, 0.75% of pixels. Four columns per device px (`Target::File`)
keeps the pixels within 5% either way (0.59%, 0.32%).

### Investigation

Renders at 2× of the first 500 device px of each variant, zoomed 8×
(scratch, not committed): the thinned 1 px line alternates full and
partly covered device columns inside the band; the unthinned line and
the 2 px variants are solid. Measured with vello_cpu and tiny-skia
(Measurements).

### Options

| | **A — the section's columns, recorded** | B — more columns on screen | C — columns per line width |
| --- | --- | --- | --- |
| Approach | Screen keeps one column per device px; the 2× test asserts the bounds with the theme's line width; the stripes of thin lines at high scale factors are a known limit | Two columns per device px on screen at scale ≥ 2 | Columns per device px from the line's width in device px |
| Cost | — | About twice the M4 points and stroking time on HiDPI screens | A rule the section does not have |
| Correctness | Default lines draw within bounds at 1× and 2×; 1 px lines at 2× stripe | Measured at 4 per px: 9.8% of columns with 1 px lines, so it narrows the stripes without removing them | Untested |

### Decision

Recommended: A, as built; revisit with the full-frame benchmark (Phase 22),
which measures what more columns cost on screen.

### Why

The section fixes the screen's columns, and the default line meets every
bound at both scales; a change of `k` is a decision the section reserves
for a measured fidelity gap, and this one shows only for lines thinner
than the theme's at scale factors of 2 and more.

### Followups

Phase 22: report the striping case (a 1 px line on a 2× display) beside
the benchmark's scenarios; Phase 31's anchored grids keep the same
column density.
