# rustd3 — session handoff 18

2026-10-27 · **Phase 17 — Legends, parts and checks** complete in one session (no cut; D16.4 done first; D17.1 and D17.2 open) · next: **Phase 18 — Plot I: display**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**D16.4** (decided A by the architect), before the phase's steps:

- 60b696f `fix(figure): tick labels share a baseline` — unrotated tick
  labels anchor by the text face's line box (typst's OS/2 typographic
  ascender and descender, `Fonts::line`), `Labels::{baseline, bounds}`,
  `Shaped::last`; layout boxes are line box ∪ ink; empty labels have no
  box. Four snapshots blessed.
- 73b398c `fix(figure): blanked ticks draw at the minor size` — ticks
  whose label the scale's format leaves empty: `Theme::{minor_tick,
  minor_width}` (4 pt, 1 pt), a second stroke in the axis part.

**Phase 17** (eight commits):

- 89a1571 `feat(figure): part hashes` — `list/parts.rs` (`Fnv`,
  `parts::hash`), filled by `Figure::emit` after one `list::Bounds`.
- b479ff7 `feat(figure): cells` — `list::{cells, Cells, Cell}`,
  `Cells::cell_at`, `Resolved::edges`.
- 4362ebb `feat(figure): legends` — `legend.rs`, `legend/{slot,inset,
  draw}.rs` (`Content`, `Block`), `layout/slots.rs`; `Figure::legend`,
  `Panel::legend`; part role 7.
- d3ac530 `feat(figure): figure locale` — `Figure::locale`; `Resolved`
  carries it to `Axis::texts`.
- d06d788 `feat(figure): raster targets` — `layout/snap.rs` (`Snap`):
  areas snapped in every pass; axis lines, ticks, borders, rules on whole
  device px; the example agreement test compares like with like (D17.2).
- 9ffe3e7 `feat(core): Points::is_collected` (additive).
- dbf1020 `feat(figure): hit index and check` — `hit.rs`,
  `hit/{grid,sink}.rs`, `Figure::hits`; `check.rs`, `check/find.rs`,
  `Figure::check` (with `fonts`), `check::size`; `Mark::{hits, issues}`
  with defaults, `Ctx::{panel, mark}`; layout records `room`.
- 9736871 `feat(figure): Fit::Visible` — `scale/visible.rs` (`Memory`,
  the step rule), `Cache::visible`, a second fitting pass through
  `Fits::window`; `Entry::visible`.
- 39e814a `test(figure): one set of test font files per process`.

Exit criteria: (1) `cargo test -p rustd3-figure --all-features` passes
with every spot row named in `figure/tests/main/{legend,parts,hit,check,
visible,layout,figure}.rs` and the unit tests of `list/parts.rs`,
`hit/grid.rs`, `layout/snap.rs`, `scale/visible.rs`. (2)
`parts::{a_figure_drawn_twice_hashes_every_part_equally,
a_figure_drawn_twice_hashes_every_cell_equally,
changing_one_marks_data_changes_that_parts_hash_only,
changing_one_marks_data_changes_the_hashes_of_the_cells_it_touches_only,
a_panel_pushed_down_by_a_title_changes_its_parts_bounds_and_hashes}`. (3)
`check::{the_examples_fit_at_960_by_760,
an_axis_title_longer_than_its_panel_overflows}`. (4)
`layout::raster_targets_put_plot_area_edges_and_hairlines_on_device_pixels`.
(5) `scripts/check.sh full` green.

`scripts/check.sh full` green at 39e814a in 3 min 17 s (warm).

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-17.md`): measurements; **D17.1**
(twelve details, A recommended, as built: notably D16.4's minor size in
points, `Legend<K>` without a lifetime, hashes over open groups and font
identity, `fig.check` needing `fonts`, `Fit::Visible` steps in data
units), **D17.2** (the example SVG and PNG no longer share geometry: the
test compares the PNG with its own list as SVG; files ≤ 0.05). Public
API beyond the section: `Shaped::last`, `Index::{len, is_empty}`,
`Ctx::{panel, mark}`, core's `Points::is_collected`.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–17 done; 18 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 18: read `rustd3.md`'s front matter and "### Phase 18 —". If the
architect answers D17.2 with B, `figure/tests/main/resvg.rs` is the one
place to change; D17.1 (1) with B changes `Theme::{minor_tick,
minor_width}` and re-blesses `list_dump_of_a_grid_of_scales`; D17.1 (3)
with B adds a `PhantomData` lifetime to `Legend`.

## 6. Notes for later phases

- Phase 18–21: render cells from `list::cells(&list, &xs, &ys, scale)`
  with `Resolved::edges()` on the snapped list (`Target::Screen`); a
  plot-area cell is one whose rect lies inside `Resolved::plot_areas()`
  (snapped). Hover reads `Figure::hits(&resolved, &mut cache)` with the
  draw's cache (memo hit for thinned lines). `Fit::Visible`'s memory is
  in `figure::Cache`, so the widget's cache keeps it across frames; hold
  areas as before.
- Part hashes name faces by font blob id: a cache over a new `Fonts`
  value re-renders everything once (conservative).
- Phase 26: the notation label ("+1e9") is not localised (D17.1 (7)).
- Phase 33: time handles need `Fit::Visible` memory in nanoseconds;
  rotated labels keep D14.1's anchors.
- Phase 35: `legend::Guide<'a>` over `Legend<K>`; minor tick size and
  width as theme values; legend glyphs.
- Phase 37: compare PDF renders on the same list (D17.2's followup).

## 7. Key files

`figure/src/{legend.rs,legend/*,hit.rs,hit/*,check.rs,check/find.rs}`,
`figure/src/list/parts.rs`, `figure/src/layout/{passes,slots,snap}.rs`,
`figure/src/scale/visible.rs`, `figure/src/figure/{fits,resolve,
resolved,cache,decorate,draw}.rs`, `figure/src/axis/{labels,placed}.rs`,
`figure/src/mark.rs`, `figure/src/mark/{dots,line,rule}.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D17.1 and D17.2 open with their recommendations.
- Conventional commits, no attribution.
