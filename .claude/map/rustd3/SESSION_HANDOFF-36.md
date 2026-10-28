# rustd3 — session handoff 36

2026-10-28 · **Phase 35 — Themes, colorbars and hatching**, session 35a: **Step 2 done** (D34.5's fix, steps 1–2; the section's cut) · next: **Phase 35, Step 3 — colorbar strips** (session 35b: steps 3–5)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- e19632d `fix(figure)` — D34.5 (decided A): a panel title's last
  baseline sits the title pad and the text face's descender above the
  largest top-side axis reach of its grid row (`layout/passes.rs`
  `headings`, `Laid::heading`, `figure/decorate.rs`); its room is the
  larger of as before and what that baseline needs. Snapshots
  `wheels.{txt,svg}` and `list_dump_of_a_grid_of_scales.txt` re-blessed.
- 6e9de08 `feat(figure)` — Step 1: `theme` public module (`Style`,
  `Context::scale`, `Domains`, `palette` with core's five schemes and
  seaborn's muted, pastel, bright, dark); `Theme` in
  `theme/settings.rs`, re-exported at the root only, builders
  `seaborn`, `palette`, `font_scale`, `domains` (with D18.1's `ink`,
  `background`); styles draw grid lines (part role `Grid` = 9), axis
  lines and tick marks per style; `Kind::niced` (continuous: core's
  `nice`, time: the tick interval) and `Domains::Nice` in resolve
  (`layout::first_counts`). Tests: `theme/settings.rs` unit tests,
  `figure/tests/main/theme.rs` (11).
- a0e4b5d `feat(figure)` — Step 2: `legend::Swatch { Square, Dot, Line
  { dash } }`, `Legend::glyph`; `examples/wheels.rs --theme
  <style>:<context>`; tests `every_style_checks_clean_at_paper_and_notebook_on_960_by_760`
  and `wheels_fit_at_talk` in `dev/examples/tests/main/wheels.rs`, the
  glyph test in `figure/tests/main/legend.rs`.
- 7b40c1b `docs(figure)` — a doc link `cargo doc -D warnings` rejected
  in `theme.rs`, and a test comment.
- The closing docs commit: `decisions/phase-35.md`, the index, this
  handoff.

Exit criteria met so far (the rest are steps 3–5's):

- 1 (partly): `cargo test -p rustd3-figure --features raster --test main
  -- colorbar hatch theme` runs the 11 theme tests (no colorbar or hatch
  tests yet).
- 3 (partly): `cargo test -p rustd3-examples -- wheels` passes 9 tests,
  every style at paper and notebook on 960 × 760 clean and
  `wheels_fit_at_talk`; `--hatch` is step 5's.
- 5: `scripts/check.sh quick` green on every commit; `full` green at
  7b40c1b (its first run, at a0e4b5d, passed every test and stopped at
  `cargo doc`).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D35.1 decided (the section's answer: no bar edges by
default); D35.2 open (eleven details as built, recommended A). D22.1
still waits for the user.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md` (phase-35.md added).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; Phase 22's verdict pending the user's run |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | only if D22.1's verdict asks |
| M4 Complete figure | 32–39 | 32–34 done; 35 at Step 3 |
| M5 Release | 40 | pending |

## 5. What to do next

Phase 35, session 35b, from **Step 3** (colorbar strips), then Step 4
(ticks and ends) and Step 5 (hatching), as the section orders them.
Carried into those steps:

- `legend::Guide<'a>` over `Legend<K>` and `Colorbar<'a, S>`
  (handoff 18, D17.1 (3)): `Figure::legend` and `Panel::legend` take
  `impl legend::Guide<'a>`; today they store `Box<dyn legend::Draw>`
  (`legend.rs`), and `Content`/`Block` (`legend/draw.rs`) lay legends
  out — the colorbar's reach must enter `layout/slots.rs` the same way.
- Quantile and threshold colorbar labels must keep uneven edges
  distinct: 7.25 must not read "7" (D25.3 (8)).
- `PartialEq` for the color scales only if the figure's table needs it
  (D23.1); it did not in Phase 33.
- The colorbar outline is "the axis line (theme edge color)", but the
  dark styles have no edge (`Theme::edge()` is `None`): a rule is needed
  (D35.2's followup).
- The colorbar inset length (the section's second listed decision) is
  step 3's.
- Hatching (step 5) goes on `area`, `ribbon`, `bar_y`, `bar_x`, span
  bars, `histogram`, `density_y`, `density_x`, and swatches repeat it:
  `legend::Swatch::draw` (`legend/swatch.rs`) is where a hatched square
  is drawn.

## 6. Notes for later phases

- Theme getters are crate-private; `Theme::get_domains` reads the domain
  rule; `Theme::edge`/`grid` are `Option`s, `Theme::ticks` says whether
  tick marks are drawn (their room is kept regardless).
- Grid lines: part role 9 per panel, drawn right after the panel
  background, from the first shown horizontal and vertical axis.
- `Domains::Nice` rounds only `Fit::All` handles, with the first axis's
  first-pass count (`layout::first_counts`), d3's 10 without an axis.
- Phase 39's `Axis::minor` should use `Theme::minor_tick`/`minor_width`,
  which follow the context, and draw only where `Theme::ticks()`.
- Scratch renders and the Python references are in the session's
  scratchpad only; no scratch example was committed.

## 7. Key files

`figure/src/theme.rs`, `figure/src/theme/{settings,style,context,domains,palette}.rs`,
`figure/src/legend/{swatch,draw}.rs`, `figure/src/legend.rs`,
`figure/src/layout/passes.rs`, `figure/src/figure/{decorate,resolve,fits,part}.rs`,
`figure/src/axis/placed.rs`, `figure/src/scale/{kind,continuous,time}.rs`,
`figure/src/scale.rs` (`stuck`), `figure/tests/main/{theme,legend,layout}.rs`,
`dev/examples/examples/wheels.rs`, `dev/examples/tests/main/wheels.rs`,
`.claude/decisions/phase-35.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --features raster --test main -- colorbar hatch theme
cargo test -p rustd3-examples -- wheels
cargo run -p rustd3-examples --example wheels -- [dir] --theme whitegrid:paper
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at 7b40c1b.
- D35.2 open with its recommendation; D22.1 waiting for the user.
- Conventional commits, no attribution.
