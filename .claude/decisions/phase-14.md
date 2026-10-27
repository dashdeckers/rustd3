# Phase 14 — Figure model and first layout

## Measurements

2026-10-27, at f7c71d1. Phase 14 adds one core method (`Continuous::admits`,
D14.1 (1)) and no harness suite: 9 suites, 3,314 assertions, 2,623
passing, 691 skipped, 80 floors, before and after; nothing blessed.

**Spot checks** (typst 0.15.1, bundled fonts; `figure/tests/main/{fit,
figure,layout}.rs`, `figure/src/layout/tracks.rs`, `figure/src/theme.rs`):
every row holds as written. Fitting: [4.725, 5.275], [−0.055, 0.055],
[−0.349, 10.189], [−0.15, 3.15], [−2.5, 8.5], [999999999.95,
1000000001.05], bars [0, 7.35], NaN [3.78, 4.22], no data [0, 1], log
[0.70795, 1412.53754] and [7.94, 1258.93], held [2, 4] then [−0.5, 10.5],
`Visible` and `Window(5.0)` as `All`, all within 1e-12 relative of the
arithmetic. Layout: the track row (fixed 123, free 821, x 102, 434.4,
537.5, 869.9, the 21 px gap) within 1e-9; `em(1.6)` 23.467; the
single-panel row: "0.0" 23.325 wide, digits 11.093 tall, left reach
35.992, bottom 23.760, rect 47.725, 11.733, 580.54 × 432.77 (the spec's
±0.01); the figure title 289.273 × 14.194 at 14.667 px in the regular
weight (bold would be 327.244 × 14.309), ×1.30909 at 19.2 px.

**The theme against seaborn 0.13.2** (read in the installed
`seaborn/rcmod.py` and matplotlib 3.11.2's defaults, not executed):
notebook `font.size` 12, `axes.labelsize` 12, `axes.titlesize` 12,
`xtick.labelsize` 11, `axes.linewidth` 1.25, `lines.linewidth` 1.5,
`xtick.major.width` 1.25, `xtick.major.size` 6; the ticks style's
`axes.edgecolor`, `text.color` and `xtick.color` ".15"; matplotlib's
`xtick.major.pad` 3.5, `axes.labelpad` 4, `axes.titlepad` 6 — the
section's values. matplotlib draws spines with projecting caps
(`matplotlib/spines.py:61`), which D14.1 (11) follows.

**Release:** `cargo test -p rustd3-figure --release fit` runs 16
integration tests and 1 unit test (18 s warm), the debug-only panic tests
compiled out and `a_foreign_handle_fits_unchecked_in_release_builds` in.

## D14.1 — Statements of Phase 14 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 14, 419aa3c … f7c71d1 · **Status:** decided

### Issue

1. **`Kind::admits` cannot be written from core's public API.** "Finite
   and valid under the transform (log: > 0)" needs the transform, which
   has no public reader (Phase 4: getters are `get_domain`/`get_range`
   only; `transform()` is crate-private), and `apply` clamps first, so a
   clamped log scale maps −5 to a finite value.
2. **Seven commits.** Steps 2–4 have no caller before `resolve` (Step 6)
   and the workspace denies dead code; `ScaleRef` (Step 1) carries
   `figure::Id` (Step 2); the E0277 doctest of Step 1 needs the axis of
   Step 5; the axes' drawing has no caller before Step 7.
3. **Public paths.** `rustd3_figure::Figure` (Phases 33 and 40) and
   `figure::Cache`, `figure::Id`, `cache::Stats`, `figure/cache.rs` holding
   both `Cache` and `Stats` cannot all hold with one public path per item.
4. **"Every handle use goes through one `fn entry(&self, h)`."** After
   fitting, handles are used through `Fits` and `Resolved`, not the
   figure; `Held` must not check figure ids (a gesture's state outlives the
   figure it started in, and every `view` builds a new one).
5. **`Entry<'a>` and the fit mode.** Nothing in an entry borrows; storing
   `Fit::Window`'s span or `Visible` would be dead code until Phases 17
   and 21.
6. **`Panel::mark(m: impl Mark<'a>)`** cannot box `m` as `Box<dyn Mark<'a>>`
   without `'a` on it (E0310-like); likewise the erased axis.
7. **`Indexed` details.** The explicit `<'a>` on `series_extent` and `thin`
   is clippy's `needless_lifetimes`; the section does not say whether the
   x range includes its ends, how a reversed range reads, or what an empty
   range returns (`SeriesExtent::x` needs two values).
8. **Piecewise scales** (`Continuous<T, f64, N>` for every N): the
   object-safe `with_range([f64; 2])` and `fitted` (which `Pad` gives two
   ends) do not say where inner stops go.
9. **`admits`, `merge` for categorical kinds** are not stated.
10. **Theme getters.** Line width (2 px), marker diameter (8 px) and the
    palette have no reader before Phase 15's marks (dead code); titles'
    weight is not stated.
11. **Axis line caps.** `list::stroke` is butt-capped; two axes leave a
    notch of half the line width at their shared corner (seen at 12×).
12. **Layout details the section leaves open:** which plot length tick
    overhangs are measured on in a single pass; where a panel title sits
    relative to the top reach; the number of tracks when weights and cells
    disagree; free space below 0; the figure title's anchor and centre;
    the anchor of top and side axis titles.
13. **Tick-label anchors and line alignment** per side.
14. **Tick positions:** core's `Ticks::position(v, offset)` takes an
    offset; d3's axes use 0.5 at a device pixel ratio of 1.
15. **`.labels(false)`** and an axis with no tick labels: their reach.
16. **Plain data types** this phase adds: `cache::Stats`,
    `scale::SeriesExtent` (named by Conventions item 4 already),
    `layout::Length`.
17. **Part roles' numbers**, and parts that draw nothing.
18. **`Fits::stick`** and the extent: the bars row needs the base inside
    the extent ([3, 7] with sticky 0 pads to [2.8, 7.2] otherwise).
19. **Crate-internal helpers:** `text::channels` becomes crate-visible and
    `list::brush(Rgb)` is added (no public item).

### Investigation

- (1) `core/src/scale/continuous.rs`: `transform()` is `pub(crate)`;
  `apply` clamps before `forward`. A per-value `clone().clamp(false)` probe
  would clone the scale (and its `Arc`s) for every admitted value of an
  owned mark.
- (3) rustd3.md lines 149, 14246–14281, 16302–16303 write
  `rustd3::figure::Figure`, `rustd3_figure::{Figure, mark::dots}`,
  `rustd3_figure::Figure`; 36 places write `figure::Cache`, the facade's
  `rustd3::figure::Cache`.
- (6) `Box<dyn Mark<'a>>` needs `impl Mark<'a> + 'a`; a supertrait
  `'a` gives it to every implementor, and `&dyn Draw<'_>` then needs the
  object lifetime spelled out (`&(dyn Draw<'a> + 'a)`), checked by the
  compiler.
- (10) `figure/tests/main/layout.rs`, the figure-title row: regular
  289.2734 × 14.1940, bold 327.2436 × 14.3086 at 14.6667 px.
- (11) `matplotlib/spines.py:61` `self.set_capstyle('projecting')`;
  rendered before and after at 12× (`corner.png`, scratch, not committed).

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | `pub fn Continuous::admits(&self, x) -> bool`: finite and finite under the transform as set for the current domain, ignoring clamp (419aa3c, additive; doctest and two tests) | `apply(v).is_finite()`: a clamped log scale admits −5. Or clone an unclamped copy per value |
| (2) | Five code commits: core `admits`; scale traits (Step 1 without the handle); the figure model, held scales and fitting (Steps 2–4 and Step 6's tracks, with the handle); axes and the protrusion layout (Step 5 without drawing, Step 6); decorations (Step 7 with the axes' drawing); plus a release test. The E0277 doctest lives in `axis.rs` | Seven with transient `#[expect(dead_code)]` |
| (3) | `Figure`, `Cache`, `Held`, `Fits`, `Resolved`, `Panel`, `Target`, `Theme` at the crate root (from private modules); `axis`, `layout`, `mark`, `scale` public modules; `cache::Stats` through a public `cache` module in `lib.rs` that re-exports `Stats` alone; `figure::Id` crate-private (no public signature names it). `figure::Cache` reads as the crate's `Cache`, as through the facade | A public `figure` module: `rustd3::figure::figure::Cache`, and `Figure` with two paths |
| (4) | `ScaleRef::of(figure)` holds the `debug_assert!` ("handle from another figure"); `Fits` and `Resolved::scale` (and so `mark::Ctx` and axes) call it; `Held` keys by index and `TypeId` | A `Figure::entry` with no caller |
| (5) | `Entry { scale, fitter: Option<Fitter> }`, the fitter two function pointers instantiated for the scale type in `fit_with`; `Fits` keeps a `Slot<S>` per fitted handle as `Box<dyn Any>`; the mode is not stored | Stored modes, unread |
| (6) | `pub trait Mark<'a>: 'a` (and the crate-private `Draw<'a>: 'a`); `Panel::mark(m: impl Mark<'a>)` as written | `impl Mark<'a> + 'a` in the signature |
| (7) | `&Points<'_, D, f64>`; `x` is inclusive at both ends (`locate(low)..locate(high.next_up())`), a reversed range is ordered first, and no sample in range gives `None` (documented) | — |
| (8) | Ends exact; inner stops keep their share of the old end-to-end span (in range space for `with_range`, data space for `fitted`), spread evenly when that span is 0 or not finite | Two stops only |
| (9) | `admits`: a key of the domain (ordinal: any key once an unknown color is set); `merge` a no-op; `fitted` returns the scale; `domain()` `None` | — |
| (10) | Getters only for what Phase 14 reads (text sizes, `em`, tick, pads, axis width, ink, edge, panel, background, margin); Phase 15 adds line width, marker diameter and the palette with their first reader. All titles regular weight (the figure-title row) | Unread getters |
| (11) | The axis line square-capped, as matplotlib's projecting spines; ticks butt, borders butt with miter joins (closed path) | A notch at every corner |
| (12) | Overhangs measured on the plot areas laid out without them, then laid out again (one measuring pass, two placements); the panel title above the top reach including overhangs; tracks = max(weights, cells); a missing gap `em(1.0)`; free space and lengths never below 0, zero weights zero lengths; the figure title anchored at its top centre at the padding, centred on the plot areas' extent (the canvas without panels); axis titles anchored at the middle of their bottom edge (the box's bottom faces the axis), except a bottom axis's (top edge); left −90°, right +90° as written | — |
| (13) | Bottom (middle, top), top (middle, bottom), left (end, middle), right (start, middle) at tick + tick pad; a label's lines align as its anchor's horizontal | — |
| (14) | Offset 0, non-finite positions dropped (snapping is Phase 17) | d3's 0.5 |
| (15) | Unlabelled: offset + tick; labelled with neither tick labels nor title: the same | — |
| (16) | Named here as plain data types | — |
| (17) | Background 0, figure title 1, panel background 2, border 3, axis 4, panel title 5 (mark and legend 6 and 7 from Phases 15 and 17); a part with no commands is not emitted; bounds `Rect::ZERO` when no command has bounds | — |
| (18) | `stick` only records; a bar mark adds its base and sticks it (documented on `stick`; the test mark does so) | `stick` also widens |
| (19) | As built | — |

### Decision

A for all nineteen (architect, 2026-10-27), as built. Core gains the
additive `Continuous::admits(x)` (finite, and finite under the
transform for the current domain, clamp ignored). The figure model's
types (`Figure`, `Cache`, `Held`, `Fits`, `Resolved`, `Panel`,
`Target`, `Theme`) sit at the crate root; `axis`, `layout`, `mark` and
`scale` are public modules; `cache::Stats` has one path.

### Why

(1) is the only exact, per-value O(1) test of the specified rule; (2),
(5), (10) follow D13.1 (2)'s rule against transient dead code; (3) keeps
the paths Phases 33 and 40 write and one path per item; (4), (6), (7) are
the smallest changes that compile and keep the stated signatures; (8),
(9), (12)–(18) are the readings closest to the section and to matplotlib
and d3; (11) is what matplotlib draws.

### Followups

- Phase 15: `Theme` gains `line_width` (2 px), `marker` (8 px) and
  `palette` (`scheme::COLORBLIND`, referenced); marks draw between the
  panel background and the border; `part::Role` gains `Mark` (6).
- Phase 16: tick counts from length replace `.ticks(n)`/10; `Axis` keeps
  `labels` and `outward` as `Option`s for "explicit" and stacking; the
  overhang placement becomes part of the pass loop.
- Phase 17: `Role::Legend` (7); part hashes; raster snapping of the tick
  positions (D14.1 (14)).
- Phase 21: `Fit::Window` needs the mode stored in `Entry` (D14.1 (5)).
