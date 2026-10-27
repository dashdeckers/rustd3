# Phase 12 — Drawing list and raster outputs

## Measurements

2026-10-27, at e8b99de; `scripts/check.sh full` green at 80230ca in 42 s
(warm). Phase 12 does no core work beyond the two D11.3
commits that preceded it (e3589a8, 4b378c2), which change no d3 test:
the harness stays at 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors.

**Agreement on the kitchen sink** (`figure/tests/main/sink.rs`, with its
sweep disc; vello_cpu single-threaded at the baseline SIMD level):

| Comparison | Scale | Max | Channels over 16 | Tile score | Bound |
| --- | --- | --- | --- | --- | --- |
| tiny-skia vs vello_cpu | 1 | 52/255 | 780 | 0.0061 | max ≤ 64 |
| tiny-skia vs vello_cpu | 1.25 | 164/255 (46 without the nearest image) | 1,275 | 0.0296 | not asserted |
| tiny-skia vs vello_cpu | 1.5 | 54/255 | 1,857 | 0.0056 | max ≤ 64 |
| tiny-skia vs vello_cpu | 2 | 56/255 | 1,191 | 0.0053 | max ≤ 64 |
| resvg (our SVG, lowered for `svg::CAPS`) vs vello_cpu | 2 | 56/255 | — | 0.0053 | tile ≤ 0.02 |
| `lower` (glyphs, markers, sweep) vs its input, both vello_cpu | 1 | 1/255 | 0 | 0.0009 | max ≤ 16 |
| `lower` (glyphs, markers, sweep) vs its input, both vello_cpu | 2 | 0/255 | 0 | 0 | max ≤ 16 |

The worst tiles of tiny-skia and resvg against vello_cpu sit on the
rotated label's glyph edges. At 1.25 the 16-texel nearest checkerboard
spans 40 device px, so texel edges fall on pixel centres and the two
nearest samplers break those ties differently; both are correct
samplers, so the agreement test runs at 1, 1.5 and 2 (the spot check
asks for 2).

**Areas and shifts** (vello_cpu, scale 2): an area render, a shift of 8
list px (16 device px) and six bands rendered with one `Bounds` equal the
whole render within 1/255; a shift of 8.25 list px (half a device px)
differs by 108/255 (the section quotes 109).

**vello_cpu's size limit** (see D12.1 (1)): a full-size fill at width w
and height 1, and at width 64 and height h, `catch_unwind` per size:

| Side | Renders | Panics |
| --- | --- | --- |
| width | ≤ 65,408 | 65,409 … 65,535 (`vello_cpu-0.3.0/src/span.rs:84-90`) |
| height | ≤ 65,532 | 65,533 … 65,535 (`vello_common-0.3.0/src/util.rs:222-238`) |

The kitchen sink rendered over one band at 65,408 device px wide (scale
163.52, four bands) and at 65,532 tall (three columns) renders without a
panic.

## D12.1 — Statements of Phase 12 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 12, steps 1–7 (5a9d40a … e8b99de) · **Status:** open

### Issue

1. **The size bound.** Raster, area contract: "Above 65,535 device px on
   either side: `TooLarge`". vello_cpu 0.3.0 panics well below that for
   widths (from 65,409) and for heights from 65,533 (Measurements).
2. **`svg::CAPS` before `Caps`.** Step 3 writes `svg.rs` with `CAPS`;
   `Caps` is defined in `lower.rs`, Step 5.
3. **Where a paint sits for markers and glyph runs.** The section says
   nothing of the space a gradient (or image brush) is positioned in when
   it paints a markers command (a unit shape placed per point) or a glyph
   run (run transform, then each glyph's placement). Backends differ
   natively: vello_cpu and SVG `<use>` put it in the shape's or run's own
   space, `lower`'s fills in the group's.
4. **Image brushes.** `Brush::Image` as a fill, stroke, marker or glyph
   paint: the section says nothing for SVG and tiny-skia (Phase 37 makes
   it `Unsupported` for PDF, "the figure layer emits `Command::Image`,
   never image brushes").
5. **Stroke caps per end.** kurbo has `start_cap` and `end_cap`; SVG and
   tiny-skia have one cap.
6. **Stroke bounds.** "the bbox inflated by `width / 2 × miter_limit` for
   miter joins, `width / 2` otherwise": a square cap on a diagonal reaches
   `width / 2 × √2` past its end along x or y, so `width / 2` lets culling
   cut it.
7. **Bounds of commands that draw nothing.** Only Push and Pop are said
   to have `None`; an empty path's bbox is the zero rect at the origin,
   which a part's union would then reach.
8. **Number precision.** "Numbers: `{:.2}` with trailing zeros and point
   trimmed" cannot hold for every number: two places turn a rotation's
   0.9553 into 0.96, a unit circle's control point 0.2761 into 0.28, and a
   run's `scale(size / 1000)`, 0.011 at 11 px, into 0.01.
9. **Dump details** the example leaves open: alpha, gradient and image
   paints, path clips, different caps per end, and whether `part 0` is
   the part's index or id; the example prints the paint table as its
   length and indices only.
10. **`Settings::parallel()`.** The doc comment ("available parallelism −
    1, clamped to 1..=8") and Part I ("from 1 to 8 threads") disagree with
    "the value vello_cpu's own `RenderSettings::default` computes
    (`render.rs:183–187`)", which is 0 on one core (`saturating_sub(1)
    .min(8)`) and unwraps `available_parallelism`.
11. **`skia::render` "computes `Bounds::of`"**: a whole-list render culls
    nothing, so the value has no reader.
12. **`raster::Error`** has no variant for `png::encode` given pixels of
    the wrong length (`raster::Image` has public fields).
13. **tiny-skia's partial sweeps.** tiny-skia 0.12 applies a sweep's
    scale and bias only when both differ from the identity
    (`tiny-skia-0.12.0/src/shaders/sweep_gradient.rs:96-100`, `scale != 1.0
    && bias != 0.0`), so a sweep from angle 0 to less than a full turn
    renders as a full turn.
14. **Smaller details**: `check`'s "all coordinates finite" (do paints
    count?); `lower`'s warning indices (input or output list), a stroke or
    a marker painted with a sweep, the sweep image's resolution under a
    scaled group and its size limit, "middle stop" for an even count;
    `Bez` given a segment with no subpath (d3 writes `L` without `M`);
    where `Part` lives, since `part` must be public for `part::Id` and
    `Part`'s one path is `list::Part`; the kitchen sink's font in tests
    built without `fonts`.

### Investigation

- (1) Measured (Measurements); the overflows are `checked_*().unwrap()`
  in vello_cpu's tile and span arithmetic.
- (3) vello_cpu's paint transform composes after the path transform
  (`vello_common-0.3.0/src/transforms.rs:109-115`) and glifo captures the
  run's transform for its paint (`glifo-0.4.0/src/glyph.rs:743, 975`);
  tiny-skia post-concatenates the path transform to the shader's
  (`tiny-skia-0.12.0/src/painter.rs:302-313`, `shaders/mod.rs:96-108`).
  Group space makes `lower` exact without touching paints; a gradient
  carried into a shape's space by a run transform that is not a
  similarity (a skew) cannot be re-expressed as a radial gradient.
- (13) Read, not exercised: the kitchen sink's sweep is a full turn.

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | `TooLarge` above 65,408 device px on either side; images in a list up to 65,535 a side (vello's `u16` pixmaps); 70,000 × 10 at 1 and 40,000 × 10 at 2 are `TooLarge` as the spot checks say | 65,535: vello_cpu panics. Or per axis, 65,408 wide and 65,532 tall |
| (2) | `svg::CAPS` lands with `lower` (Step 5) | A `Caps` stub in Step 3 |
| (3) | A paint is positioned in the space of the command's group for every command (documented on `Command`): `lower` leaves paints untouched; vello_cpu and tiny-skia set the inverse of each point's or run's placement as the paint transform; SVG writes gradient markers and gradient runs as paths in the group | Each shape's or run's own space: `lower` transforms gradients, inexactly under a skewed run transform |
| (4) | vello_cpu draws image brushes (through its image cache); SVG and tiny-skia return `list::Error::Unsupported { what: "an image brush" }`, as Phase 37 does for PDF | — |
| (5) | SVG and tiny-skia take `start_cap` at both ends; vello_cpu and `Bounds` honour both | — |
| (6) | Reach = `width / 2 × max(miter_limit for miter joins, √2 for a square cap, 1)` | `width / 2` |
| (7) | `None` also for a command that draws nothing: an empty path, no points, no outlines, or clipped to zero area; the renderer skips it | The zero rect |
| (8) | px coordinates two places; matrix coefficients, unit marker shapes, stroke widths and dashes, opacities and offsets four; the glyph scale six; the dump uses the same formatters | — |
| (9) | `#rrggbb`, `#rrggbbaa` when not opaque; `linear(x0,y0,x1,y1)[o #rrggbb,…]`, `radial(…)`, `sweep(cx,cy,a0,a1)`, ` repeat`/` reflect`; `image(w×h)`; `push matrix(…) path(d)`; caps `butt/round` when they differ; the paint table as the example has it (length and indices); `part {index}` | — |
| (10) | `(parallelism − 1).clamp(1, 8)`, with no unwrap: vello_cpu's value on two or more cores, a one-thread pool on one | vello_cpu's 0 on one core |
| (11) | Not computed | Computed and unused |
| (12) | `Error::Length { width, height, found }`; an empty image is `Error::Empty`; `Error` also derives `Clone` and `PartialEq` | — |
| (13) | Left as tiny-skia draws it; documented here | A start angle nudged off 0 |
| (14) | `check` also rejects non-finite paint geometry, stops and colors and checks image brushes' lengths; warnings name the input command, one per kind per command; a sweep on a stroke clips to `kurbo::stroke`'s outline (tolerance 0.01), markers and runs painted with a sweep become fills first, the image is 2 px per list px through the group's transform, at most 4,096 px a side, and the middle stop is `stops[len / 2]`; `Bez` starts a subpath where a canvas would; `Part` is defined in `list.rs`, `list/part.rs` holds `part::Id`; the tests read `fonts/DejaVuSans.ttf` with `include_bytes!`, so no manifest change | — |

### Decision

Recommended: A for all fourteen, already built.

### Why

(1), (10), (12) and (13) follow the pinned crates where the text does
not hold; (3) is the one rule under which `lower` is exact and every
backend agrees (tested: gradient markers equal fills placed in the
group, and the agreement tests); (4) follows Phase 37's answer; the rest
are the closest readings of the section that compile and keep one public
path per item.

### Followups

- Phase 13: glyph runs from typst keep solid fills; nothing changes.
- Phase 18: cells and bands stay below 65,408 device px a side.
- Phase 37: krilla positions marker and run gradients in the group's
  space (D12.1 (3)); image brushes stay `Unsupported` there too.
- If tiny-skia fixes its sweep scale and bias, drop (13).
