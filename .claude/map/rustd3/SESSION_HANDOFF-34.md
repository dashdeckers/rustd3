# rustd3 — session handoff 34

2026-10-28 · **Phase 34 — Figure marks breadth**, session 34a: **Step 4 done** (steps 1–4, cut after step 4) · next: **Phase 34b, Step 5 — channels, image, notes** (then steps 6 and 7)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 3e752bf `feat(core)` — Step 1, core: `Series::m4_joint`,
  `series::Error::Recordings` (D34.1), `m4::Lines::thin` over up to two
  channels' extremes, `runs::intersect`; the joint spot row, a 40-case
  raw-scan agreement and the error rows in `core/tests/main/series.rs`.
- 345c37b `feat(figure)` — Step 1, figure: `area`, `ribbon`,
  `ribbon_series` (`mark/area.rs`), `bar_y`, `bar_x`, `bar_y_spans`,
  `bar_x_spans` (`mark/bar.rs`), `rect` and the rectangle code
  (`mark/rect.rs`), `Indexed::thin_joint` (default `None`; continuous and
  time override), the memo key's `upper` channel, `mark::Error` and
  `check::Issue::Mark`, polygon clipping in `mark/guard.rs`.
- 27499fb `feat(figure)` — Step 2: `histogram`, `mark::Binned`,
  `mark::Bins::{Nice, Count}` (`mark/histogram.rs`), the E0277 doctest and
  twin; bin counts clamped below core's tick cap (D34.2).
- 3b2ea8a `feat(figure)` — Step 3: `stats::Kde` (`stats.rs`,
  `stats/kde.rs`), `density_y`, `density_x` (`mark/density.rs`),
  `dev/fixtures/kde.{py,json}`.
- 1e34953 `feat(figure)` — Step 4: `Dots::dodge`, `Dots::jitter`,
  `Line::dodge`.
- The closing docs commit: `decisions/phase-34.md` (measurements, D34.2,
  D34.3), `DECISIONS.md`'s index, this handoff.

Exit criteria for steps 1–4, each checked this session:

1. `cargo test -p rustd3-figure --test main -- marks`: 56 tests pass,
   the mark tests of steps 1–4 in `tests/main/marks/{area,bar,histogram,
   density,dodge}.rs` and the KDE fixture test
   (`marks::density::the_kde_matches_scipys_gaussian_kde_and_seaborns_grid`);
   `cargo test -p rustd3-core --test main -- series` passes
   `joint_m4_splits_where_either_channel_has_a_gap` and its two
   neighbours.
3. `cargo test -p rustd3-figure --doc` runs the histogram E0277 doctest
   and its twin (`mark/histogram.rs`).
6. `scripts/check.sh quick` green on every commit; `full` green at
   1e34953 in 11 min 31 s (warm).

Criteria 2, 4 and 5 (wheel example, benchmark workloads, the label-churn
rerun) belong to steps 6 and 7. Harness unchanged: 16 suites, 6,722
assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.

Decisions: D34.1 decided (the section's answer); D34.2 open (recommended
A: `Nice` asks for at most 6,000, `Count` makes at most 10,000); D34.3
open (twelve details as built, recommended A). D22.1 still waits for the
user.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; Phase 22's verdict pending the user's run |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | only if D22.1's verdict asks |
| M4 Complete figure | 32–39 | 32, 33 done; 34: steps 1–4 done, next step 5 |
| M5 Release | 40 | pending |

## 5. What to do next

- D34.2 and D34.3 answered by the architect.
- Phase 34b: **Step 5** (`mark/{channel,image,text}.rs`, `mark/dots.rs`:
  `fill_by`/`size_by` into `Command::Markers` with a paint table, the
  length check, the image placement test), **Step 6** (the wheel figure,
  `dev/fixtures/wheels.{py,json}`), **Step 7** (the benchmark's wheel
  workload, the label-churn rerun appended to `decisions/phase-22.md`).

## 6. Notes for step 5 and later

- `mark::Error` exists (`mark/error.rs`) with `Recordings` and `Density`;
  add `GridLength { cols, rows, len }` and `ChannelLength { points,
  values }` there. `fig.check` reports a kept error through
  `Mark::issues` as `Issue::Mark { panel, mark, error }` (see `Ribbon`
  and `Density::issues`); a mark that keeps an error neither fits nor
  draws.
- Dodging: `Color::slot(ctx)` gives a keyed color's `(k, n)`;
  `mark::dodge(band, slot)` and `mark::across(scale, v, slot, share)` place
  in a sub-band; `Dots::placed` is the one place dots are positioned (draw
  and hits), so `fill_by`/`size_by` filter there together with their
  positions.
- `mark::reach(scale, v, guard)` places an `f64` on a value axis, putting
  what the transform cannot take at the guard band's end; rectangles go
  through `rect::fill(list, ctx, rect, paint)` (guard clip, device-pixel
  snap, no empty rectangles); polygons through `mark::polygon`.
- `Theme` gained crate-private `patch_width` (1 pt) and `density_width`
  (0.75 pt).
- The wheel figure (step 6): `histogram(bins, speed, count).fill(color,
  s).dodge()` counts keys in first-appearance order; `density_y(v,
  y_of(s), width).fill(color, s)` fits only the values on y, as the
  temperature domain `[31.70122, 65.65070]` needs; `dots(...).dodge()
  .jitter(0.8)` and `line(...).dodge()` place by the palette's slot.
- Scratch renders for this session are in the session's scratchpad only;
  temporary examples were never committed.

## 7. Key files

`core/src/series/{view,m4,runs,error}.rs`, `core/tests/main/series.rs`,
`figure/src/mark.rs`, `figure/src/mark/{area,bar,rect,histogram,density,
dots,line,color,error,guard}.rs`, `figure/src/stats.rs`,
`figure/src/stats/kde.rs`, `figure/src/scale/{indexed,continuous,time}.rs`,
`figure/src/thin/{frame,memo}.rs`, `figure/src/check.rs`,
`figure/tests/main/marks.rs`, `figure/tests/main/marks/*.rs`,
`dev/fixtures/kde.{py,json}`, `.claude/decisions/phase-34.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --test main -- marks
cargo test -p rustd3-core --test main -- series
cargo test -p rustd3-figure --doc
python3 dev/fixtures/kde.py                      # only when a phase says so
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at 1e34953.
- D34.2 and D34.3 open with their recommendations; D22.1 waiting for the
  user.
- Conventional commits, no attribution.
