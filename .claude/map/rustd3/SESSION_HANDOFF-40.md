# rustd3 — session handoff 40

2026-10-28 · **Phase 38 — Helpers I: statistical charts**: **Steps 1–4 done; Phase 38 complete**, with D38.1 and D38.2 open (recommended A) · next: **Phase 39 — Helpers II: presets, labels and axis defaults**, once D38.1 is answered

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 238f40b `test(figure)` — Step 1: `dev/fixtures/stats.py` and
  `stats.json` (matplotlib 3.11.2, numpy 2.5.3, scipy 1.18.1): box plot
  stats, quantiles, `hist` tables (count, density, cumulative, weights,
  auto, nice), stacked spans, `violin_stats`, `Axes.ecdf`; `--check` at
  relative 1e-9.
- 6a84e0f `feat(figure)` — Step 2: the `helpers` module (`lib.rs`
  `#[cfg(feature = "helpers")] pub mod helpers;`), `helpers::{Subplots,
  subplots}`, `helpers::stats::boxplot::{Summary, summary, Whis}`,
  `helpers::stats::{BoxPlot, boxplot}`; private `stats/fill.rs` and
  `stats/slots.rs`; `mark::guard` becomes `pub(crate)`.
- e252e26 `feat(figure)` — Step 3: `stats::{Violin, violin}` with
  `violin::Inner`; `stats::hist::{Bins, Table, table, Error, Stacked,
  stacked}`, `stats::{Hist, hist}`; `mark::Error::{Edge, Weights}`;
  private `stats/keyed.rs`.
- 9369d33 `feat(figure)` — Step 4: `stats::{Ecdf, ecdf}`,
  `stats::errorbar::Err`, `stats::{ErrorBar, errorbar}`.
- The closing docs commit: `decisions/phase-38.md`, the index, this
  handoff.

Exit criteria:

1. `python3 dev/fixtures/stats.py --check` exits 0.
2. `cargo test -p rustd3-figure --features helpers --test main --
   helpers`: 34 pass (box plots 9, histograms 8, violins 6, ECDF 4, error
   bars 4, subplots 3); the fixture's rows are asserted within them.
3. `cargo test -p rustd3-figure --features helpers --doc`: an example
   for `helpers`, `subplots`, `boxplot`, `boxplot::summary`, `violin`,
   `hist`, `hist::table`, `hist::stacked`, `ecdf`, `errorbar` passes.
4. `cargo check -p rustd3-figure --no-default-features --features
   helpers` builds.
5. `scripts/check.sh quick` green on every commit; `full` green (see §9).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D38.1 (the helpers draw through crate-private items: the
theme's six mark readers, `list::brush`, `Ctx::snap`/`Snap`,
`mark::guard`; open, recommended A, publish them; the helpers call them
by their current names, so A is a visibility change only and B nothing),
D38.2 (thirteen details as built; open, recommended A). D22.1 still waits
for the user.

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
| M4 Complete figure | 32–39 | 32–38 done; 39 next |
| M5 Release | 40 | pending |

## 5. What to do next

If D38.1 is answered A: one commit making the listed items public (doc
comments for the theme readers, `Snap`'s methods and `mark::guard`), with
`check.sh quick`; the helpers do not change. Then Phase 39 from its first
step; its presets build on the same readers.

## 6. Notes for later phases

- Charts live in private `chart` modules re-exported from
  `helpers::stats`; the public modules `boxplot`, `violin`, `hist` and
  `errorbar` hold statistics and options. Intra-doc links to a chart
  function need `()` (the module of the same name makes them
  ambiguous).
- `hist` and `ecdf` draw through `mark::rect` and `mark::line` built per
  draw (`stats/keyed.rs` erases a palette key for them); box plots,
  violins and error bars draw their own commands.
- numpy 2's `"auto"` is `min(max(FD, sqrt/2), Sturges)`, not the
  section's older rule (D38.2 (1)).
- Stacked histograms are plain `rect` marks and do not hold 0 (D38.2 (3)).
- Scratch renders and the scratch test that wrote them are in the
  session's scratchpad only.

## 7. Key files

`figure/src/helpers.rs`, `figure/src/helpers/`, `figure/src/mark/error.rs`,
`figure/tests/main/helpers.rs`, `figure/tests/main/helpers/`,
`dev/fixtures/stats.{py,json}`, `.claude/decisions/phase-38.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
python3 dev/fixtures/stats.py --check            # the fixture against matplotlib, numpy, scipy
cargo test -p rustd3-figure --features helpers --test main -- helpers
cargo test -p rustd3-figure --features helpers --doc
cargo check -p rustd3-figure --no-default-features --features helpers
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  9369d33 in 13 min 15 s (warm, the benchmark smoke included), no
  warnings.
- D38.1 and D38.2 open with their recommendations; D22.1 waiting for
  the user.
- Conventional commits, no attribution.
