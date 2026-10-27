# rustd3 — session handoff 16

2026-10-27 · **Phase 15 — Marks and file outputs** complete in one session (no cut; D15.1–D15.3 open) · next: **Phase 16 — Layout passes**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 15** (ten commits, `check.sh quick` green on each, `full` green
at the end):

- 507178d `feat(core): points read by clone, and their indexed view` —
  `Points::get`/`iter` take `Clone` (were `Copy`); bound-free
  `Points::indexed() -> Option<Points<'a, X, f64>>` (D15.1 (1)).
- 7546e45 `feat(figure): dots and rules` — `mark::{dots, Dots, rule_x,
  rule_y, Rule}`, crate-private `mark::Color`; marks drawn per panel
  between background and border in one `Push` clipped to the plot area,
  each a part of role 6; `Fits` gains crate-private `scale`, `theme`,
  `label`; `Theme` gains `line_width`, `marker`, `color`, `gray`,
  `mark_label`.
- b3fe98f `feat(figure): line mark` — `mark::{line, Line}`, the guard
  band (`mark/guard.rs`, Liang–Barsky).
- fca47de `feat(figure): thinning` — `thin::{Thin, Frame}`, crate-private
  `thin::Memo` in `figure::Cache`, `cache::Stats::m4_queries`;
  `Indexed::thin` gains `frame: &mut thin::Frame` (D15.1 (2)).
- 063aa35 `feat(examples): trace and scatter figures` —
  `rustd3_examples::{trace::{signal, recording, trace}, scatter::scatter}`.
- b1e365a `feat(figure): draw and Target::File` — `Target::File { scale }`,
  `Figure::draw`; snapshot `list_dump_of_scatter`; the trace spot checks.
- f8612d4 `fix(figure): unambiguous doc links in mark`.
- 206d10a `feat(figure): save_svg and save_png` — `rustd3_figure::Error`,
  `save_svg`, `save_png`; `tests/main/support.rs::figures_equal`;
  snapshot `svg_of_scatter`.
- 0656ce6 `test(figure): the trace thinned at 2x` (D15.3).
- a49c764 `feat(examples): trace and scatter mains` — the mains; resvg
  agreement of the example outputs (D15.2).

Exit criteria: (1) `cargo test -p rustd3-figure --all-features` passes;
every Spot-checks row is a named test: `marks::{three_dots_are_one_markers_command_in_one_part,
a_dot_at_nan_is_dropped_and_y_fits_the_others,
a_rule_widens_a_fitted_count_domain,
a_line_far_outside_its_domain_stays_within_three_panel_sizes}`,
`thin::{the_trace_on_screen_keeps_four_points_per_column_and_every_impact,
the_trace_for_vector_output_takes_four_columns_per_px,
the_trace_for_vector_output_is_within_5_percent_of_every_sample_stroked,
the_trace_on_screen_is_within_12_percent_and_its_columns_within_3_percent,
a_second_draw_of_an_unchanged_figure_runs_no_m4_query_and_no_typst_compile,
line_thin_off_on_a_million_samples_draws_every_sample}`,
`thin::frame::tests::two_channels_of_one_recording_compute_column_boundaries_once`
(unit), `draw::{save_png_at_2x_writes_a_1280_by_960_rgba_png,
a_scatter_of_5600_points_in_three_keys_is_three_markers_commands}`. (2)
`cargo run -p rustd3-examples --release --example trace -- "$DIR"` and
`--example scatter` write the four files (run, viewed);
`resvg::resvg_renders_the_example_svgs_as_their_pngs` (scatter 0.0055,
trace 0.0275 ≤ 0.03, D15.2). (3)
`draw::the_trace_figure_redraws_with_no_m4_query_and_no_typst_compile`.
(4) `draw::{list_dump_of_scatter, svg_of_scatter}`. (5) `scripts/check.sh
full` green.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-15.md`): measurements; **D15.1**
(twelve items, A recommended, all built), **D15.2** (the trace example's
resvg agreement 0.0275, bound 0.03), **D15.3** (1 px lines stripe at 2×
on screen). The public-API choices beyond the section: D15.1 (1) the core
additions, D15.1 (2) `Indexed::thin`'s frame and the public `thin::Frame`.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.
- The release build of `rustd3-examples` (iced included) exists in
  `target/release` (11 min 42 s cold); `target/` is about 17 GB, the disk
  has about 13 GB free.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–15 done; 16 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 16: read `rustd3.md`'s front matter and "### Phase 16 —". If the
architect answers D15.1 (1) or (2) with anything but A: (1) is
`core/src/series/points.rs` (and `Slices::get`); (2) is
`figure/src/scale/indexed.rs`, its continuous impl and
`figure/src/thin/frame.rs`. D15.2 and D15.3 change only test bounds.

## 6. Notes for later phases

- Phase 16: tick counts from length replace `.ticks(n)`/10 (the trace's
  time axis shows 11 ticks now); rule labels are shaped through
  `Fits::label` keys in resolve's batch, so a pass loop that re-resolves
  keeps them; marks draw inside the panel's group, so moving a plot area
  moves only the `Push`.
- Phase 17: `Mark::hits` for thinned lines calls `Indexed::thin` again
  through the draw's `thin::Frame` (a memo hit); parts of role 6 are the
  marks; `fig.check`'s `LargeOwned` reads `Points::len` of owned data
  (`Points::indexed()` is `None` for it); raster snapping must keep mark
  coordinates panel-relative.
- Phase 18: the widget's reference renders draw with `Target::Screen`;
  one column per device px stripes 1 px lines at 2× (D15.3).
- Phase 22: the benchmark's workloads are its own, but the trace module
  (`signal`, `recording`) is public in `rustd3-examples`.
- Phase 33: the time scale's `Indexed::thin` hashes its edges' bits and
  calls `thin::Frame::lines` (crate-private, generic over `Coordinate`).
- Phase 34: `thin_joint` takes the frame; `Dots`/`Line` builders take
  `.dodge` etc.; `mark::Color` has a `Lookup` trait for palette keys
  (`Paint::slot` can join it); a note reports its text through
  `Fits::label`.
- The figure tests compile `dev/examples/src/{scatter,trace}.rs` by
  `#[path]`: `check.sh quick` reruns them only when `figure/` changes.

## 7. Key files

`figure/src/mark.rs`, `figure/src/mark/{color,dots,guard,line,rule}.rs`,
`figure/src/thin.rs`, `figure/src/thin/{frame,memo}.rs`,
`figure/src/figure/{cache,decorate,draw,fits,resolve,save}.rs`,
`figure/src/target.rs`, `figure/src/theme.rs`,
`figure/src/scale/{indexed,continuous}.rs`, `core/src/series/points.rs`;
`figure/tests/main/{marks,thin,draw,support,resvg}.rs`,
`figure/tests/snapshots/{list_dump_of_scatter,svg_of_scatter}.txt`;
`dev/examples/src/{lib,trace,scatter}.rs`,
`dev/examples/examples/{trace,scatter}.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features --test main -- marks:: thin:: draw::
cargo run -p rustd3-examples --release --example trace -- "$DIR"
cargo run -p rustd3-examples --release --example scatter -- "$DIR"
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D15.1–D15.3 open with their recommendations.
- Conventional commits, no attribution.
