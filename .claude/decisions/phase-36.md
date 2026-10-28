# Phase 36 — Plot interactions II: hover and 1-D brush

## Measurements

2026-10-28, 0235149 … acaab7b and the closing docs commit, one session.
`scripts/check.sh full` at the end: see "Measurements at the end of the
phase" below.

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors; nothing blessed.

**Lines** (ad501ef..acaab7b): library code +1,556 / −111 (figure +456:
the brush module with its unit tests, `Panel::brush`,
`Resolved::{panel_cell, brush}`; iced +1,100 / −107: hover, the brush
state with its listener and machine, the state's routing and sync,
`Plot::{on_hover, on_brush}` with docs); integration tests +1,150 (figure
41, `iced/tests/main/{hover,brush}.rs` 1,109). The section estimated
1,000–1,400 lines.

**The section's tests** (`ICED_TEST_BACKEND=tiny-skia|wgpu`, iced's
`crisp` on, `web-colors` on; `iced/tests/main/{hover,brush}.rs`):

| Test | tiny-skia | wgpu (lavapipe) |
| --- | --- | --- |
| `hovering_a_dot_reports_its_index` (k = 1, 2) | one `Some(Hover { panel: 3, mark: 0, index: 1, at })`, `at` the pointer in widget px; `panel_cell(3) == (1, 0)` (panel (1, 0) made fourth); line sample 10 and bar "b"'s top hovered too | same |
| `moving_within_one_dot_sends_nothing` (k = 2) | one message, then none after 1 px moves right and up | same |
| `leaving_the_widget_reports_none_once` | `CursorLeft`: one `None`, none the frame after; into a gutter: one `None` | same |
| `hover_does_not_rebuild_the_hit_index` | ten moves between two redraws: one `Frame` sample, no `Resolve` or `List`; one message (the last dot) | same |
| `brushing_the_context_sets_the_focus_window` (k = 1, 2 published; k = 1 owned) | 25–50% of the context: [179.75, 359.5] h exactly (bound ± 719 h / width); a message while brushing and at the end; owned: the focus panel 1/255 against the PNG of the figure built with the window | same, 1/255 |
| `events_before_update_keep_the_last_window` | press and two moves before an update: [179.75, 359.5] h; one more move: [179.75, 449.375] h | same |
| `zooming_the_focus_shrinks_the_band` | the band at device columns 204–345 (25–50%); 20 lines at the focus centre: window [224.6875, 314.5625] h, the band at 237–307 (expected 237.31–307.44 in the zoomed layout); a drag of the band's middle by 10% moves the halved window by 71.9 h | same |
| `an_empty_release_reports_none` | `[Window(full), Brushed(None)]` | same |
| `modifiers_do_not_change_a_selection` | alt + shift mid-drag: no message; the release publishes the selection as it was | same |
| `brush_overlay_stays_inside_its_panel` (2 × 2, brush in (0, 0)) | live and band: 20,475 px changed at k = 1, 82,501 at k = 2, all inside panel 0's plot area, 0 outside; the edges' columns changed, the columns past them not; every row down to the axis line | same counts |
| `brushing_alone_renders_no_cell` (k = 2, view rebuilt per message) | 3 messages, 6 stage samples, 0 rasters, every frame generation 1 | same |

Also `a_drag_brushes_where_the_wheel_zooms` (section's precedence: the
drag publishes only `on_brush`, the wheel only `on_change`) and figure's
`flat_panel_indices_map_back_to_their_cells`, `panels_report_their_brush`,
`a_brushed_handle_from_another_figure_panics_when_resolved`, the
`placed.rs` unit tests (window and place round trips on linear, log and
time handles; a descending target keeps its orientation).

**Exit criteria**: (1) `ICED_TEST_BACKEND=tiny-skia|wgpu cargo test -p
rustd3-iced --features figure --test main -- hover brush` — 13 tests on
each (the 4 hover, 8 brush, and Phase 18's
`crosshair_stays_inside_the_hovered_panel`, which the filter matches);
(2) `cargo test -p rustd3-figure --doc brush` — the E0271
`compile_fail` row (`brush::x(full_linear).sets(focus_time)`, compiled
once as a plain doctest: "type mismatch resolving `<Time as
Kind>::Domain == f64`") and its twin over two time handles, plus
`Panel::brush`'s example; (3) as the table; (4) every commit's
`check.sh quick` green.

**Viewed** (headless screenshots, scratch, not committed): the hover
figure (a line, bars, two dot panels; panel (1, 0) made fourth) hovered on
a dot, on line sample 10 and on bar "b"'s top at k = 1 and 2 on both
backends: crosshair in the hovered plot area only, ring on the dot, the
sample and the centre of the bar's value end. The figure-level focus and
context (a focus over the window above a 30-day context, rows 3:1) at
k = 1 and 2 on both backends: no band at the start (the focus shows all
30 days), the live rectangle mid-drag over 25–50% spanning the context's
plot area from its top to the axis line with a one-device-px outline, the
band after the release, and the band halved about its middle at the frame
after a ×2 wheel zoom of the focus. A y brush over a dot panel at k = 1
and 2: a band from 3 to 7 across the plot area's width, published as
[7, 3].

## D36.1 — How a widget reads a panel's brush

**Date:** 2026-10-28 · **Context:** Phase 36, step 1, 0235149 · **Status:** decided

### Issue

The section names `brush::Placed` (what `Panel::brush` takes) and has
the widget route presses to "a panel with a brush", publish selections
"with domain values from the resolved scale's invert", and set a target
"through the same path as `on_change`" between gestures and back
(`set_selection` from the target's current domain). rustd3-iced is
another crate: it needs a public way to read a panel's brush (its axis,
the handles it is over and sets) and, for `.sets(target)` without an
`on_change` for the target, typed conversions between a selection in
pixels and a scale of the target's type, which only the figure side
knows (`A` and `B` are erased once the brush is placed). The section
names no reader; `Panel`'s fields and `Figure::panels` are crate-private.

### Investigation

D20.4 met the same gap for zooms and axes (`Resolved::zooms(panel)`,
`Resolved::axis(panel, axis)`, `Resolved::scale_at(index)`), decided as
built. `Kind` has no domain setter; figure sets a window's domain with
`fitted(Some(&ends), &[], 0.0)` (`figure/src/figure/fits.rs:355-371`).
`on_brush<A: Invert>` is typed, so the published values need nothing
from figure.

### Options

| | **A — readers in D20.4's shape, as built** | B — the widget takes typed brushes itself | C — stop |
| --- | --- | --- | --- |
| Approach | `Resolved::brush(panel) -> Option<&brush::Placed>`; `brush::Dim { X, Y }`; `Placed::{dim, over, target}` (table positions) and `Placed::window(over, template, selection) -> Option<Box<dyn Scale>>` / `Placed::place(over, window) -> Option<[f64; 2]>`, fn pointers monomorphized at `From<Brush<A, B>>` | `Plot::brush_x(cell, over, sets)`, which the section rejects (Design, The API) | — |
| Cost | 1 reader, 1 enum, 5 methods | Contradicts the section | The phase stops after step 1 |
| Fit | Mirrors `zooms` and `axis`; the conversions are where the types are | — | — |

### Decision

A (architect, 2026-10-28), as built, on D20.4's precedent: the public
readers `Resolved::brush(panel)`, `brush::Dim` and
`brush::Placed::{dim, over, target, window, place}` are what the widget
outside the crate needs.

### Why

Any implementation of the section needs a reader; A adds the smallest
one in the shape of the existing readers and keeps the typed work in
figure, where `A` and `B` are known. Only rustd3-iced calls it.

### Followups

If renamed, only `iced/src/plot/{brush,state}.rs` change.

## D36.2 — Presses on an axis of a brushed panel

**Date:** 2026-10-28 · **Context:** Phase 36, step 3, "Decisions this phase may meet" · **Status:** decided

### Issue

The section: a press on a panel with a brush but outside its plot
rectangle "d3 ignores it, and so does the widget — record a decision only
if Phase 20 already routes axis presses elsewhere". Phase 20 does: a left
press over an axis strip starts a drag that pans that axis's handle when
the panel zooms it (D20.2, D20.5 (9)).

### Options

| | **A — the brush ignores it; Phase 20's axis drag stays** | B — such presses do nothing |
| --- | --- | --- |
| Approach | Brushes start only inside the plot area (`Region::Area`); over an axis strip the press is Phase 20's | Over an axis of a brushed panel, no drag |
| Fit | d3's brush ignores it, as the section says; an axis the panel zooms still pans | Removes a Phase 20 gesture from brushed panels |

### Decision

A (architect, 2026-10-28): a press on an axis strip stays Phase 20's
axis drag; the brush ignores it.

## D36.3 — Details of Phase 36 as built

**Date:** 2026-10-28 · **Context:** Phase 36, 0235149 … acaab7b · **Status:** decided

### Issue

1. **`.sets` with an empty selection.** The section says what `on_brush`
   gets (`None`), not the target.
2. **The band of a target that shows everything.** The sync rule sets the
   selection from the target's current domain; at the start a focus shows
   all of the context's domain, which would be a band over the whole plot
   area.
3. **Selections a target places outside the plot area** (a target wider
   than over, or partly outside).
4. **A brush without a target between gestures**: its selection is kept
   by the core brush in pixels; a zoom of over or a new layout moves the
   data under it.
5. **Setting the target's domain**: `Kind` has no setter; a selection of
   no width mid-drag.
6. **`on_brush`'s order and cadence**: "domain values ... at the
   selection's pixel ends"; several events per frame.
7. **The hit index "rebuilt only when the list is rebuilt"**, the hover
   message's timing ("CursorMoved (accumulated per frame)"), what "no hit
   testing while a gesture is active" shows, and which pointer a redraw
   re-hovers at.
8. **The hit in the panel under the pointer**: Phase 18 takes the nearest
   point of every panel, then keeps it only if it is in the hovered panel.
9. **Layout during a brush gesture, wheels during one, a press while one
   is active.**
10. **Double click on a brushed panel**: "resets touched handles (Phase
    20) and clears the brush selection".
11. **The cursor over a brushed plot area.**
12. **The overlay**: "a 1-px ink outline"; how fill and outline composite.
13. **The key of brush state**: "(panel cell, handle index, `TypeId`)".
14. **`State::redraw`'s listeners**, `Hover::at`.
15. **Commits**: four code commits and the docs, as the section.

### Options

| | **A — as built** |
| --- | --- |
| (1) | The target gets over's whole domain: d3's `event.selection \|\| x2.range()` and use case 1's `End(None) => Show(full)` |
| (2) | No band when the target's window covers over's whole extent (within 1e-6 px): use case 1's `(visible != full).then_some(shown)` |
| (3) | Synced selections are clipped to the plot area; none when nothing of them is inside |
| (4) | A brush without a target keeps its selection as a domain (a scale of over's type, `Placed::window` with over as template) and syncs from it like a target, so the band stays on its data; a selection covering the whole extent stays |
| (5) | `fitted(extent, &[], 0.0)` in the target's own orientation (exact on linear and time scales, within an ulp on log ones); a selection of no width sets nothing |
| (6) | `[invert(s0), invert(s1)]` in pixel order (x left to right; y top to bottom, so a y brush publishes high then low, as d3's `s.map(y.invert)`); one message per brush per frame, the latest; the end always publishes, after the target's `on_change` |
| (7) | Rebuilt with lists whose cells changed (unchanged hashes draw the same points); each move is hit-tested at once (the overlay redraws then) and the change reported once per frame at `RedrawRequested`; during a zoom, pan or brush gesture nothing is hit-tested, the ring hides and nothing is reported; the widget keeps the pointer from `CursorMoved`/`CursorLeft`, not the redraw's cursor |
| (8) | Kept as Phase 18 built it (no new public `hit::Index` query) |
| (9) | A brush gesture holds the plot areas like a zoom and settles at its end; wheel steps during it are not captured; a press ends one whose release was lost with a left release (its `End` publishes) |
| (10) | The clicked panel's brush loses its selection (`on_brush(None)` if it had one) and its target is reset with the panel's zoomed handles when touched |
| (11) | A crosshair (d3's overlay cursor) over brushed plot areas and during a brush gesture |
| (12) | One device px, as the crosshair; the outline's four sides and the fill are separate quads that do not overlap (one quad with a border composites differently on tiny-skia and wgpu); edges rounded to device px |
| (13) | The axis too: a brush turned from x to y on the same panel and handle starts afresh |
| (14) | `redraw` takes the `on_change` and `on_brush` listeners as a pair; `Hover::at` is the pointer minus the widget's origin, logical px |
| (15) | 0235149 step 1, f4ca2ef step 2, 4f6f958 step 3, acaab7b step 4, then the docs |

### Decision

A for all fifteen (architect, 2026-10-28), as built.

### Why

(1)–(3) follow use case 1, which Part I calls the design of focus and
context, and d3; (4) is the sync rule applied to the only domain a brush
without a target has; (5) is how the figure sets windows; (6)–(13) are
the readings closest to the section, to d3 and to Phases 18 and 20;
(14)–(15) change nothing the section states.

### Followups

- Phase 38–39 helpers that link panels may place brushes through
  `Panel::brush`.
- A per-panel nearest-point query on `hit::Index` would make (8) exact
  next to tightly packed panels.

## Measurements at the end of the phase

2026-10-28, at acaab7b: `scripts/check.sh full` green in 11 min 43 s
(warm; the iced tests on tiny-skia and wgpu through lavapipe, the
benchmark smoke report-only); no warnings. Exit criterion 1's command as
written (`--features figure`, not all features): 13 pass on tiny-skia
and 13 on wgpu; exit criterion 2: the `compile_fail` row and two passing
doctests.
