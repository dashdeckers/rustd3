# Phase 30 — Behaviors and animation

## Measurements

2026-10-28, 9df22ce … 102be32, one session after Phase 29's.
`scripts/check.sh full` green at 102be32 in 10 min 40 s (warm, the
benchmark smoke included).

**Harness** unchanged, as exit criterion 6 asks: 16 suites, 6,722
assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors (no d3
test file is translated in this phase).

**Core**: `core/src` +1,065 / −5 lines with docs (587 lines of code in
the twelve new files; the section estimates about 900 lines of Rust),
`core/tests` +1,076.

**Grading.** The brush and drag traces are transcribed by hand from the
vendored d3-brush 3.0.0 `src/brush.js` and d3-drag 3.0.0 `src/drag.js`
(both DOM-bound, so not run in Node); every trace step of the section is
asserted (`core/tests/main/brush.rs`, `drag.rs`), each trace annotated
with the source lines it follows. With the hit test's handle order
swapped, T11 fails with [12, 29], the section's "had w won". The join
rows (the section's two and four more) were printed by d3-selection 3.0.0
in Node over stand-in nodes (`bindKey`, `selection/data.js:31–73`).
`use_case_2_lifecycle` compares the printed trace with Part I's text; a
second test checks the section's arithmetic rows within 1e-12.

**"Decisions this phase may meet"**: no trace tie arose beyond those the
section names; the details below are where the text and d3's source or
the workspace rules part.

## D30.1 — Details of Phase 30 as built

**Date:** 2026-10-28 · **Context:** Phase 30, steps 1–5 (9df22ce, d60488d, 9b93952, 102be32) · **Status:** open

### Issue

1. **Steps 2 and 3 in one commit.** The engine's `Dim::Y` would be dead
   code (`-D warnings`) in a commit with `BrushX` alone.
2. **"The last modifiers".** Design, Brushes: the engine "holds … the last
   modifiers", and the table stores `Modifiers(m)`. Nothing in v0.1 reads
   them (keyboard modes are Later): a stored field would be dead state,
   and would make two brushes that differ only in the last modifier event
   compare unequal.
3. **Which release ends a gesture.** d3-brush and d3-drag end on any
   button's `mouseup` (`brush.js:400–402`, `drag.js:54–56`: the window
   listener does not check the button). The brush table ends only on
   `Released { button: Left }`; Drag's text names no button.
4. **A press during a gesture.** Not in the section. In d3 the group's
   `pointer-events` are `none` during a brush gesture (391) and only the
   overlay sets its own (`all`, 170), and `started` makes a new emitter
   (365), so a left press restarts on the overlay, the old gesture never
   ending; d3-drag's `beforestart` replaces the mouse gesture when the
   subject answers and leaves it when it does not (53, 136, 144).
5. **`empty`.** "an empty selection (zero width; zero height for Y)":
   d3's `empty` (131–134) tests both, for either brush. They differ only
   for an extent of zero height (X) or width (Y).
6. **The gesture's locals.** The section lists d3's `type`, `mode`,
   `signX`, `signY`, `w0 … s1`, `dx dy`, `point0`, `point`. In one
   dimension without keyboard modes the other axis always spans the
   extent (overlay presses, `dim.input` and moves all keep it), the
   handle type only picks the edge and the cursor, and `dx`, `dy`,
   `point`, `w1`, `e1` are read only within one move or by keyboard modes.
7. **Hit-test rects.** "edges inclusive": a point covers a rect at origin
   o of size s when o ≤ p ≤ o + s, from `redraw`'s attributes. SVG draws
   no rect of negative or zero size; a zero-size rect here covers its
   edge.
8. **Paths.** `interact.rs` re-exports `Drag` while `interact::drag` is
   public for `drag::Event`: `Drag` would have two public paths.
9. **Line citations.** The section cites the filter as "(9–11)" for the
   brush too; in `brush.js` it is 108–110 (9–11 is `drag.js`'s).
10. **`join`'s bounds.** `K: Eq + Hash + Clone` as written; the keys move
    into the sides, so `join` itself needs no clone.
11. **`Animated` details.** The prototype returns the target at once for
    a zero duration even during a delay, lerps at `ease(0)` before the
    start, adds `timing.delay + delay` and `start + duration` (both panic
    on overflow).
12. **`Marks` details.** The section lists no constructor; a key given
    twice in one `update` is not covered; derives are not listed.
13. **The base instant's attribute.** Exit criterion 5 says
    `#[allow(clippy::disallowed_methods)]`; other test files use
    `#[expect]`.

### Investigation

d3-brush 3.0.0 and d3-drag 3.0.0 as vendored under `dev/harness/d3/`;
d3-selection 3.0.0 `src/selection/data.js`. (2) `brush.js`: `altKey`,
`metaKey`, `shiftKey` are read in `started` (340, 353) and the key
handlers (510–577) only. (4) SVG 1.1 §16.6, `pointer-events` is
inherited. (6) by induction over `started` (367–383), `move` (442–454:
the inactive sign is null) and `dim.input` (28, 35).

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | Four code commits (join, brushes, drag, anim) and the docs | Five, with dead code in the second |
| (2) | `Modifiers(_)` gives `None` and stores nothing | A field no code reads |
| (3) | Only the left button's release ends a brush or a drag, as the brush table, Part I ("the held button ends a gesture") and `Zoom` (Phase 10) do | d3: any button's release |
| (4) | Brush: a left press during a gesture hits only the overlay and starts anew (a new `Start`, no `End` for the old); drag: starts anew when `subject` answers, else the drag goes on | — |
| (5) | d3's `empty`, both axes | Width only for X, height only for Y |
| (6) | The gesture holds `mode` (`Drag` or `Handle(edge)`, the flip swapping the edge), `edges` (w0, e0), `point0` along the axis and d3's local `selection`; selections are built from the extent; same arithmetic, JavaScript's `min` and `max` included | Both signs, all eight edges and the handle type |
| (7) | As described | — |
| (8) | `Drag` is defined in `interact.rs`, its methods in `interact/drag.rs` beside `drag::Event` (D29.2 (1)'s pattern); `drag::Gesture` is crate-private. The brush module is private, so `interact::BrushX`, `BrushY` and `BrushEvent` have one path each | Two paths for `Drag` |
| (9) | The tests cite `brush.js:108–110` | — |
| (10) | The bound kept as written | — |
| (11) | Before its start a value is where it came from (d3 runs no tween before a transition starts), a zero duration jumps at the start; delays add saturating, a start past what `Instant` holds is `now` (the section's rule), and `value`/`is_animating` measure with `checked_duration_since`, so nothing panics; a stagger is `stagger.saturating_mul(position)` | The prototype's |
| (12) | `Marks: Default` is the constructor; a repeated key keeps one mark, which heads for the later target at the later position (the prototype's lookup; Part I: one animated value per key); `Animated` and `Marks` derive `Debug`, `Clone`, `PartialEq`; `values`, `prune` and `is_animating` need only `T: Interpolate` | — |
| (13) | `#[allow(…, reason = "a base instant for offsets; values do not depend on it")]`, the section's words | `#[expect]` |

### Decision

Recommended: A for all thirteen. (3) departs from d3 on purpose and is
the one behavior choice here; it follows the section's own table and
Phase 10's zoom.

### Why

(1), (8), (9) and (13) are commits, paths and citations; (2) and (10)
follow the rule against dead code and the section's signature; (4), (5),
(6), (7) and (11) follow d3's source where the section is silent or
shorter; (12) follows the prototype and Part I.

### Followups

- Phase 32 (use cases 1 and 2 as apps) and Phase 36 (the brush overlay
  in `Plot`) use these APIs as specified.
- Keyboard modes (Later) would store modifiers again, with the T5–T10
  traces.
