# rustd3 — session handoff 17

2026-10-27 · **Phase 16 — Layout passes** complete in one session (no cut; D16.1–D16.3 open) · next: **Phase 17 — Legends, parts and checks**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 16** (seven commits, `check.sh quick` green on each, `full` green
at the end):

- 93e6c77 `feat(figure): tick counts from length` — `layout/passes.rs`
  (`Layout`, `Laid`): the first placement with axes at offset and tick,
  then passes with counts `max(2, round(len / 80))` across and
  `max(2, round(len / 35))` up until they repeat (≤ 4);
  `Resolved::passes`; `layout::Plan`. Snapshots `decorations_of_one_panel`,
  `list_dump_of_scatter`, `svg_of_scatter` blessed (counts; 0.01 px
  overhang).
- 3b61d92 `feat(figure): outer-only labels and stacking` —
  `layout/plan.rs` (`Role`, `roles`, `stack`, `inner`);
  `Resolved::plan(panel, axis)`; undrawn axes take no room or part.
- 11abfc5 `feat(figure): collisions` — `layout/collide.rs` (`grid`,
  parity stride on the absolute index, greedy pass by plot area);
  `axis/placed.rs` (`Placed`, boxes, drawing kept labels);
  `Draw::indices`; `Resolved::label_boxes`.
- c6a370e `feat(figure): spans` — `layout::Cell`, `Figure::panel_at`,
  spanning tracks and inner axes over overlapping ranges.
- b3cc704 `feat(figure): offset and multiplier notation` —
  `axis/offset.rs` (`Notation`), `Axis::offset(bool)`, the label's row
  under horizontal axes and above vertical ones (`Placed::above`).
- 6fc41cd `feat(figure): held layout` — `layout::Areas`,
  `Resolved::{areas, held}`, `Figure::{resolve_held, draw_held}`, the
  screen slack (`Target::interactive`).
- bf5aa9f `test(figure): a grid of scales` — snapshot
  `list_dump_of_a_grid_of_scales`.

Exit criteria: (1) `cargo test -p rustd3-figure --all-features layout`
passes; the spot rows are `layout::plan::tests::counts_follow_length_one_tick_per_80_px_across_and_35_px_up`,
`layout::{layout_settles_within_four_passes,
grids_of_shared_axes_settle_within_four_passes_labelled_on_the_outer_panels,
twenty_band_labels_keep_every_kth_at_four_widths,
the_stride_counts_ticks_from_zero_so_a_pan_keeps_the_same_labels,
end_labels_of_stacked_panels_keep_the_gap_their_overhangs_reserve,
where_end_labels_of_two_panels_overlap_the_larger_panel_keeps_its_own,
a_shared_axis_is_labelled_on_the_outer_panel_only,
a_second_axis_on_one_side_stacks_8_px_past_the_first,
a_stacked_axis_on_an_inner_panel_is_not_drawn,
labels_true_labels_an_inner_axis,
a_spanning_panel_runs_from_its_first_column_to_its_last,
far_from_zero_domains_label_ticks_from_an_offset_at_the_axis_end,
tiny_domains_label_ticks_in_units_of_a_power_of_ten,
offset_false_prints_full_labels,
held_areas_keep_the_plot_while_labels_grow_outward,
a_labelled_vertical_axis_on_screen_reserves_two_characters_of_slack}`,
plus `axis::offset::tests::*`. (2)
`layout::layout_invariants_hold_on_the_test_figures_and_the_examples`
and `assert_layout` in every layout test. (3)
`layout::kept_labels_only_change_at_the_ends_while_an_axis_pans`. (4)
`layout::areas_for_another_size_or_panel_count_are_ignored`. (5)
`scripts/check.sh full` green.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-16.md`): measurements; **D16.1**
(seven details, A built), **D16.2** (notation only for ticks on d3's
linear grid, so log and symlog decades keep d3's labels), **D16.3** (the
stacked-rows spot row keeps both labels under Phase 14's overhangs; the
cross-axis rule tested at a negative gap and in held layout). Public API
beyond the section: `Resolved::plan`, `Resolved::label_boxes` (D16.1 (1)).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–16 done; 17 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 17: read `rustd3.md`'s front matter and "### Phase 17 —". If the
architect answers D16.2 with B, `Axis::notation` in `figure/src/axis.rs`
is the one place to change; D16.3 with B changes `layout/passes.rs`'s
`reach` and Phase 14's layout tests; D16.1 (1) with B removes the two
`Resolved` readers and moves their tests into the crate.

## 6. Notes for later phases

- Phase 17: legends' slot reservation goes into `place` and the
  figure-title `reserve` in `layout/passes.rs`; `fig.check` reads
  `Laid::axes[*].{plan, boxes, keep}` crate-internally; the figure
  locale replaces `Locale::default()` in `Axis::texts` (notation labels
  included); raster snapping must keep `Placed::boxes` consistent with
  snapped positions. Top-axis labels anchor at their ink bottom (D14.1
  (13)), so month labels with descenders sit higher; D13.1's followup
  allows baseline anchors if Phase 17 or 33 wants them.
- Phase 18–20: the widget keeps `layout::Areas` in its gesture state and
  calls `draw_held`; `Target::Screen` reserves the slack.
- Phase 33: time axes need an index branch in `Draw::indices`; rotated
  boxes replace `collide`'s axis-aligned overlap test.
- Phase 39: notation follows D16.2.

## 7. Key files

`figure/src/layout.rs`, `figure/src/layout/{passes,plan,collide,cell,hold}.rs`,
`figure/src/axis.rs`, `figure/src/axis/{draw,labels,placed,offset}.rs`,
`figure/src/figure/{resolve,resolved,draw,decorate}.rs`,
`figure/src/target.rs`; `figure/tests/main/layout.rs`,
`figure/tests/snapshots/list_dump_of_a_grid_of_scales.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features layout
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D16.1–D16.3 open with their recommendations.
- Conventional commits, no attribution.
