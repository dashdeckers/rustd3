# rustd3 — session handoff 42

2026-10-28 · **Phase 39 — Helpers II: presets, labels and axis defaults**: **done** (session 39b: D39.1's commit, Steps 3–4), with D39.3, D39.4 and D39.6 open · next: **Phase 31** (D22.1's verdict, decided C, asks for it), then Phase 40

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- fc3ae57 `feat(figure)` — D39.1 (decided A): `Fits::label`,
  `Ctx::label`, `Ctx::locale`, `Ctx::line` public with doc comments for
  custom marks (`Fits::label` carries a custom-mark example);
  `Fits::locale` stays crate-private (D39.3).
- d8afd5a `feat(core)` — `impl Data for Points` (the identity), so a
  preset keeps `impl Data` as `Points` and hands it to a mark still
  borrowed or indexed (D39.6 (2)).
- f34b302 `feat(figure)` — Step 3: `helpers::chart::{line, Line, scatter,
  Scatter, bar, Bar}`, `chart::bar::Mode`; the shared finishing builders
  (`title`, `x_label`, `y_label`, `size`, `theme`, `minor`, `save_svg`,
  `save_png`) as a macro over `chart/setup.rs`; crate-private
  `Theme::get_palette`.
- 3d8a31f `feat(figure)` — Step 4: `chart::{area, Area, pie, Pie,
  heatmap, Heatmap}` (`Pie::inner`, `Heatmap::colors`, `Heatmap::keys`,
  D39.5), `dev/examples/examples/presets.rs`.
- 94cfd0c `fix(figure)` — a doc link `cargo doc -D warnings` found
  ambiguous (`line`, also std's `line!`).
- The closing docs commit: `decisions/phase-39.md` (session 39b's
  measurements, D39.3–D39.6), the index, this handoff.

Exit criteria:

1. `cargo test -p rustd3-figure --features helpers --test main -- presets
   labels axes`: 68 pass; `-- presets` runs 13.
2. `cargo run -p rustd3-examples --example presets` writes 11 SVGs and 11
   PNGs into `target/presets` (or the directory given) and prints no
   issue; each preset test asserts `fig.check` empty at 640 × 480.
3. `cargo test -p rustd3-figure --features helpers,png --doc`: 103 pass;
   examples for every preset function and builder with behaviour of its
   own, Part I's line sketch among them (`chart::line`).
4. `scripts/check.sh quick` green on every commit; `full` green at
   94cfd0c (§9).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D39.3 (open, recommended B: publish `Fits::locale`,
`Fits::theme` and `Theme::get_palette`, so `Ctx::locale` can be matched
while fitting and keyed presets read the palette publicly), D39.4 (open,
recommended A: unnamed series keyed by index, "0", "1", …), D39.5
(decided, the section's heatmap keys), D39.6 (eight details as built;
open, recommended A). None of them changes behaviour that Phase 31 or 40
builds on; each answer other than A is additive.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; D22.1 decided C (raster/upload bound) |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | next: D22.1 asks for it (tiles and pan translation), then the rerun D22.3 describes |
| M4 Complete figure | 32–39 | done |
| M5 Release | 40 | after Phase 31 |

## 5. What to do next

Phase 31 (rustd3.md, `### Phase 31 —`), read with `decisions/phase-22.md`
(D22.1's verdict and registered rule, D22.3's factor cells). If the
architect answers D39.3 B or C, or D39.4 C, that is one small visibility
or builder commit first, with the module docs' notes of crate-private
reads updated.

## 6. Notes for later phases

- Presets are consuming builders; `.figure()` returns a `Figure` that can
  still be changed (more marks, `fig.locale`), and `save_svg`/`save_png`
  write it at the preset's `.size`.
- Stacked bars and areas keep a 5 % y margin below 0 (span marks and
  ribbons hold nothing); a holding builder would fix both (D39.6 (5),
  D38.2 (3)).
- The heatmap with keys draws its image on hidden linear handles fixed
  to the key counts under band axes; key counts are not checked against
  the grid (`mark::Grid` has no public readers).
- The pie is a mark of its own (`Wedges`, private) that sizes itself from
  the plot area at draw time and delegates its labels to `labels::pie`.
- The scratch example that rendered the comparisons and the matplotlib
  script are in the session's scratchpad only; the contact sheet of the
  example's PNGs is `presets_contact.png` there.

## 7. Key files

`figure/src/helpers/chart.rs`, `figure/src/helpers/chart/{setup,line,
scatter,bar,area,pie,heatmap}.rs`, `figure/src/helpers/chart/bar/preset.rs`,
`figure/src/figure/fits.rs`, `figure/src/mark.rs`,
`core/src/series/data.rs`, `figure/tests/main/helpers/presets.rs`,
`dev/examples/examples/presets.rs`, `.claude/decisions/phase-39.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --features helpers --test main -- presets labels axes
cargo test -p rustd3-figure --features helpers,png --doc
cargo run -p rustd3-examples --example presets [dir]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` at 3d8a31f
  passed every test and stopped at `cargo doc` (the ambiguous link),
  fixed by 94cfd0c; `full` green at 94cfd0c in 12 min 45 s, no warnings
  (warm, the benchmark smoke included).
- Disk: 7.6 GB free after the session (`target/` 22 GB).
- D39.3, D39.4, D39.6 open with their recommendations; D39.5 decided.
- Conventional commits, no attribution.
