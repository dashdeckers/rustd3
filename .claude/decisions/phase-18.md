# Phase 18 — Plot I: display

## Measurements

2026-10-27, at 4f471b3 (steps 1–3; step 3 without
`translucent_background_matches_the_png`, see D18.1). Phase 18 does no
core work: the harness stays at 9 suites, 3,314 assertions, 2,623
passing, 691 skipped, 80 floors; nothing blessed.

**The widget against the PNG of its figure** (`iced/tests/main/plot.rs`;
the section's 480 × 320 two-panel figure: a line of 10k samples, 2k dots
at the theme's opacity, axes, a title; iced's `crisp` on, `web-colors`
on):

| Backend | k = 1.25 | k = 1.5 | k = 2 | channels over 16 |
| --- | --- | --- | --- | --- |
| tiny-skia | 1/255 | 1/255 | 1/255 | 0 |
| wgpu (lavapipe) | 1/255 | 1/255 | 1/255 | 0 |

Where the 1/255 comes from (vello_cpu renders of the same list, scratch):
SIMD `Detect` (the widget's `Settings::parallel()`) against `Baseline`
(the reference) up to 1/255; vello_cpu's pool against one thread 0/255;
the cells stitched against the whole canvas, same settings, 1/255; the
bands of a 960 × 640 cell against the cell whole 1/255
(`bands_match_the_whole_cell`). Placement adds nothing (Phase 11:
0/255).

With the dots at 0.7 opacity (the first draft of the test figure), cells
against the whole canvas reach 2/255 on 4–5 channels, inside the dots
panel where translucent dots overlap, not at cell edges: vello_cpu's u8
compositing depends on where its strip grid falls (D18.3). The test
figure keeps the theme's opacity, as the section writes "2k dots".

**Renders viewed** (headless screenshots on both backends next to the
PNG, scratch, not committed): the trace (960 × 600), the scatter
(640 × 480) and a 2 × 2 grid with a slot and an inset legend (800 × 560)
at 1, 1.25 and 2. Edges crisp; axis lines, ticks and labels on the PNG's
device pixels; no seam at cell edges or at the band edge of the trace's
banded plot-area cells (bands of 307 and 175 rows at 2×). Differences
against the PNG, the same on both backends:

| Figure | k = 1 | k = 1.25 | k = 2 |
| --- | --- | --- | --- |
| grid | 1/255, 24 px | 1/255, 68 px | 1/255, 124 px |
| scatter (dots at 0.7) | 2/255, 13,125 px | 2/255, 19,564 px | 2/255, 15,662 px |
| trace (3 × 1M samples, thinned) | 140/255, 445 px | 2/255, 57 px | 237/255, 561 px |

The trace's large differences are D18.3.

## D18.1 — No public way to read or set a theme's ink and background

**Date:** 2026-10-27 · **Context:** Phase 18, steps 3 and 4, after 4f471b3 · **Status:** decided

### Issue

Phase 18 needs, from rustd3-iced and its tests:

1. Overlay: "two `fill_quad`s … in the theme's ink at 40% alpha" (Design,
   Overlay). The widget must read the figure's theme ink. Phase 36's
   brush overlay needs it too ("the theme's ink at 10% alpha with a 1-px
   ink outline at 40%").
2. Step 4's `crosshair_stays_inside_the_hovered_panel`: "overlay ink pure
   red through the theme". The test must set a theme's ink.
3. Step 3's `translucent_background_matches_the_png` "(background at 50%
   alpha)", and Builds on: "the theme background is a colour with
   alpha". The test must set a theme's background. Phase 37's
   `gif::Error::Translucent` needs the same to be tested.
4. Not in this phase: "No change to `figure/` or to any manifest."

`Theme` has private fields, `Default` and crate-private getters
(Phase 14, "Theme (default only)": "Builders are Phase 35"); Phase 35's
builders are `seaborn`, `palette`, `font_scale` and `domains`, none of
which sets the ink or the background. `Figure::style()` is
`pub(crate)`. So (1)–(3) cannot be built without a public figure API,
which (4) forbids and no phase defines.

### Investigation

- `figure/src/theme.rs`: no public method; `ink()`, `background()`,
  `edge()` are `pub(crate)` and called at 7 places
  (`axis/labels.rs:48`, `figure/decorate.rs:41`, `layout/passes.rs:97`,
  the theme's own test).
- `figure/src/figure.rs:223`: `pub(crate) fn style(&self) -> &Theme`.
- Bare builder names (`.ink(c)`, as Phase 35's `.palette(colors)`) clash
  with the crate-private getters of the same name; core's d3 convention
  for a builder and its reader is `domain(d)` / `get_domain()`.
- iced passes every `draw` a `renderer::Style { text_color }`, the app
  theme's text color (`core/src/renderer.rs:128-131` at 3cbdacc).

### Options

| | **A — theme builders and readers in figure** | B — ink from iced's style | C — nothing now |
| --- | --- | --- | --- |
| Approach | One additive commit before step 4: `Theme::ink(self, Rgb) -> Self` and `Theme::background(self, Rgb) -> Self` (a color with a non-finite channel or opacity leaves the theme as it was, as Phase 35's builders); readers `Figure::get_theme(&self) -> &Theme`, `Theme::get_ink(&self) -> Rgb`, `Theme::get_background(&self) -> Rgb` in core's `get_` form, which replace the crate-private `ink()` and `background()` | The overlay takes `renderer::Style::text_color` at 40%; the crosshair test sets the harness style's color to red; figure gains only `Theme::background(self, Rgb)` for the translucent test | The overlay draws `#262626` (every figure's ink until a builder exists); the crosshair test finds overlay pixels by difference with the unhovered frame; the translucent test moves to the phase that adds a background builder |
| Cost | About 60 lines and a test in figure; 7 call sites renamed | About 20 lines in figure | None now; a constant to replace later |
| Correctness | The overlay follows the figure's theme, as the section and Phase 36 say; the straight-alpha path is tested | The overlay follows the app's theme, not the figure's; a dark app over a light figure draws a light crosshair on white | Right for every figure until a theme can change its ink; the translucent path stays untested in Phase 18 |
| Fit | Contradicts "no change to `figure/`"; adds what Phases 36 and 37 also assume | Contradicts it less; Phase 36 meets the same question | Keeps the section's file boundary |

### Decision

A (architect, 2026-10-27). Theme gains the builders `Theme::ink(self,
Rgb)` and `Theme::background(self, Rgb)` (any alpha; a translucent
background lets an app's own background show through the widget), and
the readers `Figure::get_theme(&self) -> &Theme`, `Theme::get_ink(&self)
-> Rgb` and `Theme::get_background(&self) -> Rgb` in core's `get_` form
(getters only where a builder takes the name). They replace the
crate-private `ink()` and `background()` at their call sites. Phase 35's
seaborn styles set both through the same fields; Phases 36 and 37 read
them through these readers.

### Why

Three phases (18, 36, 37) assume a theme whose ink the widget reads and
whose background can be translucent; only a public builder and reader
make that true, and the builders belong to `Theme`, where Phase 35 puts
its own. B draws the figure's overlay in the app's color; C leaves the
straight-alpha handle path, which the section singles out (up to 4/255),
untested until some later phase adds a builder.

### Followups

- If A: commit `feat(figure): theme ink and background` before step 4;
  then the translucent test (step 3), the overlay (step 4) and docs
  (step 5) in rustd3-iced; Phase 35 adds its builders beside these.
- Phase 36's brush overlay reads the ink as the crosshair does.

## D18.2 — Details of Phase 18 that the section leaves open, or that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 18, 06b1c00 … 4f471b3 · **Status:** decided

### Issue

1. **Commits.** Step 1 alone leaves the scale-factor chain and the
   state's fields without a caller (the workspace denies dead code).
2. **`From<Plot> for Element` "`Element::new`".** `Element::new` is
   `pub(super)` at the pin (`core/src/widget/element.rs:22`); widgets
   box themselves through `Widget::_boxed`. `Layout` has no lifetime
   parameter at the pin.
3. **Resolve and draw.** Refresh needs the list and the resolved edges
   and plot areas; `draw_with` returns only the list, the emitter is
   private, and figure may not change.
4. **"store `now`".** Nothing reads it in Phase 18 (the frame budget is
   Phase 19), so the field would be dead code.
5. **Two public paths.** `pub mod plot` and `pub use plot::Plot` give
   `rustd3_iced::Plot` and `rustd3_iced::plot::Plot`; CLAUDE.md's "one
   public path per item" is written for private submodules.
6. **Without `fonts`.** `figure` without rustd3-iced's `fonts` has no
   `Cache::default()`, and `Plot` has no way to receive fonts.
7. **The spot row "204.8 × 1.25 = 256.00000000000003".** In `f64` the
   product is exactly 256.0 (204.8 is 204.80000000000001137; the exact
   product rounds to 256). The render is 256 × 125 either way.
8. **"2k dots".** With translucent dots the cells reach 2/255 against
   the whole render (Measurements); the section's bound is 1/255.
9. **Render errors.** The section does not say what a band that fails to
   render shows.

### Investigation

Compiler errors and lints on the section's text (1, 2, 4); the iced
sources as cited; the measurements above (7, 8).

### Options

| | **A — as built** | B |
| --- | --- | --- |
| (1) | Steps 1 and 2 in one commit (06b1c00), step 3 in another (4f471b3) | Transient `#[expect(dead_code)]` |
| (2) | `plot._boxed()`, with a comment citing the pin | — |
| (3) | `fig.draw_with` for the list, then `fig.resolve` with the same cache for edges and plot areas: a second resolve per refresh (text cache hits, no M4 query; `Fit::Visible` memory is stable on a repeated resolve) | A public `Figure::draw_resolved(&self, &mut Cache, &Resolved) -> List` in figure |
| (4) | Not stored until Phase 19 reads it | — |
| (5) | Both paths, as the section writes | `Plot` only at `plot::Plot` |
| (6) | The state's cache uses `Fonts::with([])`: rustd3-figure's bundled fonts when unified on, else none (labels draw no glyphs); the integration tests run with `figure` and `fonts` | Gate `plot` on `fonts` too |
| (7) | The test asserts the 256 × 125 render | — |
| (8) | The test figure's dots at the theme's opacity (1), as written | Bound 2/255 with translucent dots |
| (9) | Nothing (the band is skipped); render errors are list-check failures or sides above 65,408 px | — |

### Decision

(architect, 2026-10-27) A for (1), (2) and (4)–(9), as built. B for
(3): figure gains `Figure::draw_resolved(&self, cache: &mut Cache,
resolved: &Resolved) -> List` (today's private emitter: the list of a
`Resolved` that this figure's `resolve` or `resolve_held` returned for
`cache` after the draw's `cache.begin()`), and `draw_with`/`draw_held`
call it. The widget refreshes with one `begin`, one resolve and one
`draw_resolved`, so each frame resolves once.

### Why

(1), (2) and (4) follow the compiler, the lints and the pinned iced; (3)
is the only reading under which figure stays unchanged; (5) follows the
section; (6) keeps `figure` building without `fonts`; (7)–(9) record
what holds.

### Followups

- Phase 19: stage timings show the second resolve; `now` is stored with
  the frame budget.

## D18.3 — vello_cpu's output on dense strokes depends on the render's origin

**Date:** 2026-10-27 · **Context:** Phase 18, the architect's screenshot check · **Status:** decided

### Issue

Testing ("Raster agreement bounds") states that whole-pixel shifts
render within 1/255, and Part I's performance section builds Phase 31's
tile translation on it ("Whole-pixel shifts render identically (within
1/255 …)"). On the trace example's list (three channels of 1M samples
thinned for the screen at 2×) vello_cpu draws faint 3–4 px dashes at the
ends of dense self-overlapping strokes, and where it draws them depends
on the render's origin. The widget renders cells at their own origins,
so it shows the trace with dashes at other places than the PNG: up to
237/255 on 561 px at 2× (Measurements). The section's own figure, whose
line is not thinned, stays within 1/255.

### Investigation

The trace's list at 2×, vello_cpu single-threaded, baseline SIMD, the
whole canvas rendered again from an origin shifted by (dx, dy) device
px, compared where both cover:

| Shift | (1, 0) | (2, 0) | (4, 0) | (8, 0) | (16, 0) | (256, 0) | (0, 1) | (0, 2) | (0, 4) | (0, 8) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| max | 231 | 254 | 12 | 12 | 43 | 170 | 232 | 237 | 0 | 1 |
| px over 2 | 601 | 637 | 12 | 12 | 15 | 99 | 391 | 673 | 0 | 0 |

The dependence follows vello_cpu's 4 × 4 tiles (rows repeat every 4) and
its 256-px wide tiles across. Where a one-row shift changes vello_cpu's
output by more than 64 (98 px), tiny-skia agrees with one of the two
vello_cpu renders at every pixel (19 with the unshifted, 79 with the
shifted): each vello_cpu render has dashes the other and tiny-skia do
not. The PNG has them too.

### Options

| | **A — record; measure before relying on shifts** | B — render cells on vello_cpu's grid | C — report upstream |
| --- | --- | --- | --- |
| Approach | The widget's bound against the PNG holds for figures without dense self-overlapping strokes; Phase 22 reports the trace's dashes beside its scenarios; Phase 31 measures shift invariance on the trace before tile translation relies on it | Render each cell from an area whose top is a multiple of 4 device rows and whose left is a multiple of 256 columns, then crop | A reduced list to vello_cpu's issue tracker |
| Cost | — | Up to 3 extra rows and 255 extra columns per cell; a 4 × 4 grid can nearly double the raster work | Outside the session's network rules |
| Correctness | Dashes stay, at other places than the PNG's | The widget equals the PNG; the dashes stay in both | Fixes both outputs if accepted |

### Decision

A (architect, 2026-10-27): recorded as a property of vello_cpu 0.3.0.
Phase 22 reports the trace's origin-dependent dashes beside its
scenarios; Phase 31 measures whole-pixel shifts on the trace before it
relies on them. The widget's reference tests keep figures that do not
self-overlap densely, or compare the trace at its own origin.

### Why

The dashes are vello_cpu's and in every output; B makes the widget copy
the PNG's artifacts at a raster cost the cost model has no room for. The
fact matters most to Phase 31, whose premise it qualifies.

### Followups

- Phase 22: report the trace at 2× beside the scenarios (as D15.3's 1 px
  line).
- Phase 31: measure whole-pixel shifts on the trace, not only on the
  wheel figure, before translating tiles.
