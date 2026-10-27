# Phase 20 — Plot III: gestures

## Measurements

2026-10-27, c9e1d6a … dc8f665; `scripts/check.sh full` green at dc8f665
in 2 min 5 s (warm), no warnings. Phase 20 does no core work: the harness
stays at 9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80
floors; nothing blessed.

**The section's tests** (`ICED_TEST_BACKEND=tiny-skia|wgpu`, iced's
`crisp` on, `web-colors` on; integration tests in
`iced/tests/main/gestures.rs`, state-level ones in
`iced/src/plot/state/gestures.rs`):

| Test | tiny-skia | wgpu (lavapipe) |
| --- | --- | --- |
| `wheel_zoom_keeps_the_point_under_the_cursor` (3 lines, k = 1 and 2) | value under the cursor off by 1.2e-16 / 2.4e-16 (x), 1.5e-16 (y) relative | same |
| `wheel_steps_follow_d3s_wheel_delta` | spans 100 / 2^0.05, / 2^0.25 after 100 logical px, / 2^0.75 after a ctrl line (1e-12) | same |
| `wheel_over_an_axis_zooms_only_that_axis` | left strip: `Y` only; bottom strip: `X` only; far outside: ignored | same |
| `five_wheel_events_publish_one_message` | one `X`, one `Y` (k = 2^0.25); the next frame none | same |
| `twin_handles_zoom_together` | vib and temp the same k and anchor (1e-12); panel 1 follows: 1/255 against the app's PNG | 1/255 |
| `sub_precision_zoom_is_ignored` | spans 9.6595e-4, 9.3305e-4, rejected (9.01e-4 < 9.0949e-4), then 9.6607e-4 on the zoom-out | same |
| `drag_pans_every_handle_the_panel_zooms` (+40, +25 px at 2) | x [−10.178, 89.822], y [0.954, 10.954] as computed (1e-12); grabbing during the drag | same |
| `pan_survives_batched_updates` (6 × 20 px; an update per 1, 3, 8 inputs) | x [−30.6122…, 69.3877…] bit for bit in all three; 1/255 against the PNG | same |
| … widget-owned | 0/255 against the controlled frames, all three batchings | 0/255 |
| `the_cursor_shows_what_a_gesture_would_do` | grab, resize vertical / horizontal, crosshair, none, none | same |
| `zoom_shows_a_scaled_preview_at_once` (worker, k = 1 / 2) | 9 previews, 0 missing; 5,061 / 16,775 px moved, all in the plot area; mean difference from the sharp frame 2.10 / 1.39 (before the zoom 4.25 / 4.33); sharp vs inline 0/255 | 9 previews; 5,129 / 16,785 px; 1.82 / 1.26; 0/255 |
| `double_click_resets_touched_handles` (200 ms, 3 px) | published x [−2.375, 104.875] (fitted), widget's y refits; 1/255 against the PNG | 1/255 |
| `slow_double_click_does_not_reset` (400 ms / 8 px) | nothing published, frame unchanged | same |
| `cursor_left_clears_the_overlay` | the frame after `CursorLeft` equals the unhovered one | same |
| `zoomed_pixels_match_a_figure_built_with_the_published_domain` (k = 1, 2) | 1/255 held (against `draw_held` in the start areas), 1/255 settled | 1/255, 1/255 |

State-level: `layout_holds_during_a_wheel_gesture` — eight steps of
eight lines 16 ms apart on y [0, 3] (to [0.0204, 0.3469]) and redraws to
+288 ms keep the plot area bit for bit and `Resolved::held()`; the y
labels grow 9.33 px outward, inside the canvas; the redraw at +300 ms
lays out once, as a fresh resolve of the zoomed domain (left edge 66 →
76 px); the next redraw renders nothing. `published_domains_last_until_the_gesture_ends`,
`owned_domains_survive_rebuilds`, `state_is_dropped_when_the_handle_type_changes`
(handle 1 linear → log drops (1, linear), keeps (0, linear)).

**Spot rows** asserted in unit tests: zoom ×2 at 100 px of [0, 100] over
400 px gives [12.5, 62.5] with 25 at 100 px; a +40 px drag gives
[−10, 90]; `Affine1 { 2, −200 }`, cell [0, 256) → [−200, 312); the
invariant new(start⁻¹(p)) = p · k + offset for linear and log scales
over both range directions (1e-9 relative).

**Frames viewed** (headless screenshots, scratch, not committed): a drag
at factors 1 and 2 on tiny-skia (mid-drag the plot area holds and the
content and labels follow the pointer; the settled frame is laid out
afresh); zoom and pan previews on the worker at factors 1 and 2 on both
backends (the zoom preview puts every dot where the sharp frame has it,
slightly larger and soft, under the old labels; the pan preview shifts
the old image, exposing the theme background where it left, with the
plot-area half of the spines moving along; sharp frames crisp); a wheel
burst on a y axis whose labels widen (held: labels grow toward the
canvas edge; settled: the area moves right by their growth).

## D20.1 — `Held` takes typed scales only

**Date:** 2026-10-27 · **Context:** Phase 20, step 1, c9e1d6a · **Status:** decided

### Issue

The widget keeps overrides erased (`Box<dyn figure::Scale>` keyed by
index and `TypeId`); `Held::hold` takes a typed handle and scale.

### Decision

The section's answer ("Decisions this phase may meet"): an additive
`Held::hold_erased(&mut self, index: usize, scale: Box<dyn Scale>)` in
`figure/src/figure/held.rs`, keyed by the boxed scale's own type
(`as_any().type_id()`); resolving uses it where the figure's scale at
`index` has that type, as `hold` does.

### Followups

None.

## D20.2 — Axis boxes

**Date:** 2026-10-27 · **Context:** Phase 20, step 2, 11c66e6 · **Status:** decided

### Issue

Phase 17's hit test returns no axis boxes; a wheel over an axis must
know which handle it zooms.

### Decision

The section's answer: `region.rs` derives each shown axis's strip from
its panel's plot area, its side and its plan: from `offset` to `offset +
thickness` px outside the area's edge, along the edge (`Resolved::plan`
gives offset and thickness, `Resolved::axis` the side and handle,
D20.4). Plot areas take precedence over strips; an axis whose plan is
not shown has none.

### Followups

The reader of the side and handle is D20.4.

## D20.3 — Previews from the gestures' transforms

**Date:** 2026-10-27 · **Context:** Phase 20, step 4, 143321a · **Status:** decided

### Issue

The object-safe figure `Scale` cannot map a position, so `draw` cannot
form `current.apply(rendered.invert(p))` at a cell's edges.

### Decision

The section's answer, as built: each handle keeps a preview `View`
(epoch, and the affine map in shares of its range — 0 at the range's
start, 1 at its end — that its gestures composed since the epoch began;
an epoch starts at first use and after a reset). Each job of a
plot-area cell carries, per axis, the `Basis` (lead handle and its view)
at planning time; the slot keeps it with the image. While a plot-area
slot shows outdated content and its render is pending, `draw` maps it by
`T_c ∘ T_r⁻¹` (the current view after the inverse of the planned one),
converted to device px through the cell's two edges and the panel's
plot area. A panel's lead handle along an axis is the first it zooms
that way, else the first of its axes that way; a basis of another
handle or epoch maps by the identity, and two identity maps draw the
slot as before (nearest filtering, on device pixels).

### Why

`T_c ∘ T_r⁻¹` equals `current.apply(rendered.invert(p))` by the
invariant of `apply` step 4; shares of the range make one view serve
every panel that shows the handle, whatever its pixel size.

### Followups

A domain changed by other means than gestures (an app setting a new
window) while a render is pending is previewed with the gestures' map
until the sharp image lands.

## D20.4 — Two readers of `Resolved` the section does not name

**Date:** 2026-10-27 · **Context:** Phase 20, step 1, c9e1d6a · **Status:** decided

### Issue

The widget knows handles only by position and scale type. The design
needs: the `TypeId` of each index `Resolved::zooms` lists (to form a
`Key`), "the resolved one" a gesture's track starts from, the type check
that drops entries of a retyped handle, and the fitted domain a reset
publishes — all an erased read of a resolved scale by position, which
`Resolved` (typed `scale<S>(h)` only) lacks. D20.2's answer reads each
axis's side and handle, which nothing public exposes (`Panel::axes` and
`axis::Draw` are crate-private; `layout::Plan` has offset and thickness
only).

### Investigation

`figure/src/figure/resolved.rs` at 4ab0e96: `scales: Vec<Box<dyn
Scale>>` private, `scale<S: Kind>(h)` the only reader; `layout::Plan`
(`figure/src/layout/plan.rs:27-38`); the prototype read `r.scales[i]`
and `fig.panels[p].gesture` directly (`proto/figmodel/src/live.rs`).

### Options

| | **A — two readers, as built** | B — fields on `Plan`, one reader | C — a widget-facing summary |
| --- | --- | --- | --- |
| Approach | `Resolved::scale_at(index) -> Option<&dyn Scale>`; `Resolved::axis(panel, axis) -> Option<(Orient, usize)>`, beside `plan(panel, axis)` | `scale_at` as A; `layout::Plan` gains `orient` and `handle` | One `Resolved::gestures()` returning per panel its zoom keys and axis strips |
| Cost | 2 methods, 2 private fields filled at resolve (`bind`) | Changes a Phase 16 plain data type and its literals | A new public type for one consumer |
| Fit | Mirrors `scale<S>` and `plan`; additive | `Plan` is how layout placed an axis, not its identity | Moves widget logic into figure |

### Decision

A (architect, 2026-10-27): figure's public readers
`Resolved::scale_at(index)` and `Resolved::axis(panel, axis)` stay as
built; they are what a gesture widget outside the crate needs.

### Why

The section's design presumes both reads ("else the resolved one", "the
TypeId of each zoomed index", D20.2's "side and measured reach in the
resolved layout"); A adds the smallest readers in the shape of the
existing ones.

### Followups

Phase 36 (hover and brush over axes) can use the same readers.

## D20.5 — Details of Phase 20 as built

**Date:** 2026-10-27 · **Context:** Phase 20, c9e1d6a … dc8f665 · **Status:** decided

### Issue

1. **Commits.** Step 1 alone (domains and listeners) has no producer of
   domains, and the workspace denies dead code; scope = crate puts the
   figure part in its own commit.
2. **Fields beyond the section's:** `Domains::{views, epochs}` (D20.3),
   `Track::base`, `Gesture::area` (the panel's plot area at the start,
   which pointers are made relative to), `gesture::Step`; `Plot` loses
   its `PhantomData<Message>`.
3. **"Bump `generation` if anything changed"** (step 5) and the refresh
   bumping it again when cells change.
4. **One message per key per apply** when a gesture ends inside an apply
   (a press after a wheel step, a release and a new press in one frame):
   what it published must go out before `pending` is cleared.
5. **A wheel over another region during a wheel gesture**: the section
   says only that a wheel with no gesture starts one.
6. **Stale wheel anchors.** Core's `Zoom` keeps a burst's anchor until a
   non-wheel event; the widget feeds each track's zoom only during its
   gestures, so a new wheel gesture at the same point would reuse an
   anchor from another gesture's frame.
7. **Applying `(k, offset)` through `zoom_px`/`pan_px`**: the section
   names the operations, not the anchor.
8. **`Released`** is queued always (the table), whether or not a drag
   is active or queued.
9. **The rest of a double click's queue**, and the panel of a double
   click over an axis strip.
10. **The fitted domain a reset publishes** ("the handle resolved with
    no override"): the next resolve has no override for it, so it is the
    handle as the app built it. An app that builds a published handle
    with its own window gets that window back; only a handle the app
    builds fitted returns to its data's extent. Part I says "an
    app-owned handle receives its fitted domain".
11. **A key in both the x and y lists of one panel.**
12. **Tests the integration harness cannot observe** (the widget's
    `Resolved`, owned domains) and harness additions.
13. **`on_change` twice for one handle.**

### Investigation

Compiler and lints (1, 2); core `zoom.rs` (`handle` ends a burst on any
non-wheel event, 6); the frames and tests in Measurements.

### Options

| | **A — as built** |
| --- | --- |
| (1) | The D19.1 build commit (9ecd547), then seven for the phase: figure (c9e1d6a), steps 1 and 2 (11c66e6), 3 (cd6a1c9), 4 (143321a), 5 (60ebd24), 6 (0d6eef5), and dc8f665 (the harness's cursor reader gated so `check.sh full`'s featureless build has no warning); then the closing docs |
| (2) | As listed |
| (3) | `apply` bumps the generation and marks the list stale when a step was accepted or a reset happened; the refresh's plan bumps it again when cells change |
| (4) | Messages are flushed when a gesture ends and at the end of the apply: one per key per gesture segment, normally one per frame |
| (5) | It ends the wheel gesture and starts one on the new region |
| (6) | Starting a gesture feeds each track's zoom `Modifiers(current)`, which ends an earlier burst and changes nothing else; queued `Modifiers` feed the active tracks |
| (7) | `zoom_px(range, anchor, k)` around the event's position along the axis, then `pan_px` by `offset − anchor·(1 − k)` (skipped when 0); k = 1 pans only; k = 1 and offset 0 is the start |
| (8) | Queued, redraws, ignored by `apply` without a drag |
| (9) | Dropped (modifier changes kept); a gesture in progress ends first; the region's panel, a plot area's or an axis strip's |
| (10) | As written; tested with a published handle the app builds fitted |
| (11) | One track, the first (x) |
| (12) | State-level unit tests in `iced/src/plot/state/gestures.rs`; the harness gains `shoot` and `interaction`, `reference::png_held`, and `pipeline::workers` is shared with the preview test |
| (13) | The later replaces the earlier |

### Decision

A for all thirteen (architect, 2026-10-27), as built. For (10): a
double-click reset publishes the domain the handle resolves to with no
gesture override. For a fitted handle that is its fitted domain, as
Part I says; for a handle the app built with its own domain or window,
it is that domain or window, which is what the figure shows without
the gesture. Phase 21's windowed handles follow the same rule.

### Why

(1), (2) follow the compiler and the lints; (3), (4), (8), (10) and (13)
are the readings closest to the text; (5), (6), (7), (9) and (11) keep
each gesture in its own frame and the zoom about the point under the
pointer; (12) reads what the section's tests name where the harness
cannot.

### Followups

Phase 21 (live data, `Fit::Window`) meets (10) for windowed handles;
Phase 33 adds the time-handle wheel test and the E0631 doctest.
