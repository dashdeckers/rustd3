# rustd3 — session handoff 15

2026-10-27 · **Phase 14 — Figure model and first layout** complete in one session (no cut; D14.1 open) · next: **Phase 15 — Marks and file outputs**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 14** (six commits, `check.sh quick` green on each, `full` green
at the end):

- 419aa3c `feat(core): Continuous::admits` — finite and finite under the
  transform for the current domain, ignoring clamp (D14.1 (1)).
- 14de529 `feat(figure): scale traits` — `scale::{Scale, Kind, Position,
  Paint, Fit, Fittable, Indexed, SeriesExtent, Invert}` for
  `Continuous<T, f64, N>`, `Band<K>`, `Point<K>`, `Ordinal<K, Rgb>`.
- 60a7afd `feat(figure): figure model, held scales and fitting` —
  `Figure<'a>`, `ScaleRef<S>`, `Held`, `Panel<'a>`, `mark::{Mark, Ctx}`,
  `Fits`, `Cache`/`cache::Stats`, `Target`, `Theme`, `layout::{Grid,
  Length, em, px}`, tracks, `resolve`.
- 4e31086 `feat(figure): axes and the protrusion layout` —
  `axis::{Axis, bottom, left, right, top}`, labels measured in one batch,
  reaches, overhangs; the E0277 doctest in `axis.rs`.
- da62a77 `feat(figure): decorations` — `draw_with`, parts, the axes'
  drawing; snapshot `decorations_of_one_panel.txt`.
- f7c71d1 `test(figure): a foreign handle fits unchecked in release builds`.

Exit criteria: (1) `cargo test -p rustd3-figure` passes; every Spot-checks
row is a named test (`figure/tests/main/{fit,figure,layout}.rs`, the track
row in `layout/tracks.rs`, `default_theme_sizes` in `theme.rs`). (2)
`a_handle_from_another_figure_panics_when_a_mark_reports_on_it` (and
`…_when_its_scale_is_read`), debug only; `cargo test -p rustd3-figure
--release fit` passes (16 + 1 tests). (3)
`attached_panels_with_axes_and_titles_share_pixel_ranges_exactly` and the
bare-grid twin, bit for bit. (4) `decorations_of_one_panel` passes. (5)
`cargo test -p rustd3-figure --doc`: the compile_fail in `axis.rs` (E0277
"`Bin` doesn't implement `Display`", checked by compiling it) and its
twin. (6) `scripts/check.sh full` green.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-14.md`): measurements, and **D14.1**
(open, A recommended, all built). The two that are public-API choices
beyond the section: (1) the core reader `Continuous::admits`; (3) the
figure model's types at the crate root (`rustd3_figure::{Figure, Cache,
Held, Fits, Resolved, Panel, Target, Theme}`), public modules `axis`,
`layout`, `mark`, `scale`, and `cache::Stats` through a `cache` module in
`lib.rs`; `figure::Id` crate-private.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11–14 done; 15 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 15: read `rustd3.md`'s front matter and "### Phase 15 —". If the
architect answers D14.1 (1) or (3) with anything but A, apply that first
(for (1) the figure's `Kind::admits` is one line in
`figure/src/scale/continuous.rs`; for (3) the re-exports in
`figure/src/lib.rs` and the `use` lines of the figure tests).

## 6. Notes for later phases

- Phase 15: marks draw inside each panel between its background part and
  its border part (`figure/src/figure/decorate.rs`); add `Role::Mark` (6)
  in `figure/part.rs`; `Theme` gains `line_width` (2 px), `marker` (8 px)
  and `palette` with their first reader (D14.1 (10)); `mark::Ctx` has no
  constructor yet: build it in `decorate` from `&Resolved`, `&Theme` and
  the plot area. `list::brush(Rgb)` makes a solid brush. `Fits::stick`
  does not widen: bars add their base too. Marks call `y.admits(&v)` as
  the `valid` filter; the continuous impl delegates to core's
  `Continuous::admits`.
- Phase 16: `Axis.labels` and `Axis.outward` are `Option`s already (for
  "explicit"); `axis::Labels` holds keys, ink and orient; overhangs are
  measured on a bare placement in `resolve` — fold that into the pass
  loop. Tick count is `.ticks(n)` or 10 in `Axis::set`.
- Phase 17: part hashes; snap tick positions (offset 0 now); `Role::Legend`.
- Phase 21: `Entry` keeps no fit mode (D14.1 (5)); store it with the
  window's span.
- Handoff 14's notes for Phases 16, 17, 37 stand.

## 7. Key files

`figure/src/{figure,panel,mark,scale,axis,layout,target,theme}.rs`,
`figure/src/figure/{cache,decorate,draw,fits,held,part,resolve,resolved}.rs`,
`figure/src/scale/{band,continuous,fit,fittable,handle,indexed,invert,kind,ordinal,paint,point,position}.rs`,
`figure/src/axis/{draw,labels}.rs`, `figure/src/layout/{protrusion,tracks}.rs`;
`figure/tests/main/{figure,fit,layout}.rs`,
`figure/tests/snapshots/decorations_of_one_panel.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features --test main -- figure:: fit:: layout::
TZ=Asia/Kathmandu cargo test -p rustd3-figure --release fit
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D14.1 open with its recommendation.
- Conventional commits, no attribution.
