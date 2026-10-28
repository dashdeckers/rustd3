# rustd3 — session handoff 43

2026-10-28 · **Phase 31 — Plot V: tiles and pan translation**: session 31a **done** (D39.3's and D39.7's commits, Steps 1–2, D22.4's item 4), stopped at the section's first cut point · next: **Phase 31, Step 3** (figure: held column grid, dash phase, overscan), with D31.2 and D31.3 open

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- e1355bc `feat(figure)` — D39.3 (B): `Fits::locale`, `Fits::theme` and
  `Theme::get_palette` public, documented for custom marks; the helpers'
  and chart module docs updated; a custom mark outside the crate labels in
  the figure's locale and palette (`figure/tests/main/locale.rs`).
- 2fe8e79 `fix(figure)` — D39.7: span bars and ribbons hold 0 when an end
  (a channel's least or greatest value) is exactly 0; stacked bars and the
  stacked area stand on the axis, diverging bars keep both margins; no
  snapshot changed (none draws them), nothing re-blessed.
- 1dab54b `feat(bench)` — Step 1: `rustd3-bench raster` (envelope fills
  and 16-stamp marker splatting against the cells as drawn, `raster.csv`),
  `envelope_covers_every_column_extreme`.
- 584c5f1 `docs` — Step 2: neither variant meets 32/255, so neither is
  built (D31.1); D31.2 (open).
- 222e9cb `feat(bench)` — `rustd3-bench also` and the D22.4 (4) row: a
  pan's UI-thread work on the scatters.
- 0d97e24 `perf(figure)` — D22.4 (4): the dots mark, marker bounds and
  part hashes cost less per point (list stage 0.42× on both scatters).
- The closing docs commit: `decisions/phase-31.md` (measurements, D31.1–
  D31.3, item 4's numbers), the index, this handoff.

Exit criterion 1: `cargo run --release -p rustd3-bench -- raster --plan
default --out target/bench-raster` writes `raster.csv` (36 rows, 1 min
41 s); `decisions/phase-31.md` records that neither variant is built.
Criteria 2–6 belong to Steps 3–10.

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors; nothing blessed.

Decisions: D31.1 (decided, the section's rule: envelope 5.5–8.3× faster on
`trace` but up to 254/255 off; splat 3.0–6.6× faster on `scatter100k` but
up to 61/255 off). **D31.2 (open, recommended B):** the same splat at
eighth-pixel phases (64 stamps) stays within 29–30/255 at 3.6–6.2× on
`scatter100k`; building it is a `feat(figure)` commit (`Renderer::splat`,
default off) whenever the architect says so. **D31.3 (open, recommended
A):** nine details of 31a as built.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; D22.1 decided C |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | in progress: 31a done (pre-steps, Steps 1–2, D22.4 (4)); 31b is Steps 3–5, 31c Steps 6–10 |
| M4 Complete figure | 32–39 | done |
| M5 Release | 40 | after Phase 31 |

## 5. What to do next

Phase 31, **Step 3** (rustd3.md, `### Phase 31 —`, Steps): `thin::Grid`,
`Held::grids`/`grids_for`/`overscan`, the dash phase anchored on the tile
grid; tests `anchored_columns_repeat_after_a_whole_pixel_pan`,
`dash_phase_is_zero_at_the_anchor`, `overscan_extends_the_view`. Then
Steps 4–5 to the second cut point. Read D22.4 (items 1–3 belong to the
pool, the cost model and compositing, Steps 4–7), D18.3 (measure
whole-pixel shifts on the trace before relying on them) and D15.3 first.
If the architect answers D31.2 B, that commit comes before Step 3 or
beside Step 7.

## 6. Notes for later phases

- `rustd3-bench raster` stays for the rerun; its splat renders the runs
  between markers through the figure's renderer, so its splat times are an
  upper bound for an in-figure splat (D31.3 (3)).
- The scatters' UI thread after the cuts (this container, `scatter1m`):
  resolve 3, list 39, bounds 2 and the hit index 62 ms per pan frame; the
  hit index is the largest remaining cost and only hover reads it (a 0.2
  item with list building off the UI thread, or Step 7's worker).
- Part hashes of markers commands are word-wise FNV-1a; their values
  changed (no test or snapshot stores one).

## 7. Key files

`dev/bench/src/raster.rs`, `dev/bench/src/raster/{envelope,splat}.rs`,
`dev/bench/src/also.rs`, `figure/src/mark/dots.rs`,
`figure/src/list/{bounds,parts}.rs`, `figure/src/mark/{bar,area}.rs`,
`figure/src/figure/fits.rs`, `.claude/decisions/phase-31.md`.

## 8. Commands

```bash
scripts/check.sh quick                     # every commit (runs the ratchet)
scripts/check.sh full                      # end of a session (background)
cargo run --release -p rustd3-bench -- raster --plan default --out target/bench-raster
cargo run --release -p rustd3-bench -- also    # the Also measured items, D22.4 (4) among them
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` at 0d97e24:
  green in 13 min 50 s (warm, the benchmark smoke included), no warnings.
- Disk: 9.1 GB free after the session (`target/` 20 GB).
- D31.2, D31.3 open with their recommendations; D31.1 decided.
- Conventional commits, no attribution.
