# rustd3 — session handoff 37

2026-10-28 · **Phase 35 — Themes, colorbars and hatching**, session 35b: **Steps 3–5 done; Phase 35 complete** · next: **Phase 36 — Plot interactions II: hover and 1-D brush**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 5493663 `feat(figure)` — Step 3: `legend::Guide<'a>` (sealed by a
  hidden `erase` returning the unnameable `Erased`); `Figure::legend` and
  `Panel::legend` take `impl legend::Guide<'a>` (existing calls compile
  unchanged); the keys legend's layout moved to `legend/keys.rs`;
  `legend::{Content, Block}` dispatch between keys and colorbars.
  `colorbar::{Colorbar, colorbar, Ramp, Samples}`; `Ramp` for
  Sequential, Diverging, `Continuous<T, Rgb, N>`, Quantize, Quantile,
  Threshold; the strip (`colorbar/strip.rs`), length and placement
  (`layout/slots.rs` aligns a bar with the plot areas' extent; insets
  40% of the long side, framed). On raster targets a strip has one row
  per device pixel (D35.4).
- 0228e0b `feat(figure)` — Step 4: builders `title`, `extend`, `ticks`,
  `format`; `colorbar::Side` makes the bar's long axis an `axis::Draw`,
  so `axis::Labels` and `axis::Placed` measure, place and draw its
  ticks, labels and title (`Placed::draw` split into `marks` and
  `texts`; colorbars draw marks in every style); layout measures every
  colorbar per pass and its counts join the settling test; binned labels
  at matplotlib's precision raised until distinct (`colorbar/edges.rs`,
  D35.5); `colorbar::Extend` triangles and the outline
  (`colorbar/extend.rs`); the E0277 `compile_fail` doctest and twin;
  core's `Ticks for Threshold::position` doc names the colorbar geometry.
- 05aaad4 `feat(figure)` — Step 5: `hatch::Hatch` (`figure/src/hatch.rs`)
  and `.hatch(h)` on area, ribbon, bar_y/bar_x (and spans), histogram,
  density_y/density_x; marks report hatched palette keys through `Fits`,
  `Resolved::hatch` finds them, legend squares repeat them;
  `wheels::hatched`, `figure_with(…, hatch)`, the example's `--hatch`.
- The closing docs commit: `decisions/phase-35.md` (35b measurements,
  D35.3–D35.6), the index, this handoff.

Exit criteria, all met:

1. `cargo test -p rustd3-figure --features raster --test main -- colorbar
   hatch theme`: 28 pass.
2. The 300-px strip tests (`colorbar::a_table_strip_equals_…`,
   `…an_interpolator_strip_shows_…`): every pixel exact on vertical and
   horizontal bars.
3. `cargo test -p rustd3-examples -- wheels`: 10 pass (presets clean at
   paper and notebook, `wheels_fit_at_talk`, the hatched figure);
   `cargo run -p rustd3-examples --example wheels -- --hatch` writes
   `wheels_hatched.{svg,png}` and `wheels_inset_hatched.{svg,png}`.
4. `cargo test -p rustd3-figure --doc -- colorbar`: the E0277 row and its
   twin.
5. `scripts/check.sh quick` green on every commit; `full` green at
   05aaad4 in 11 min 55 s.

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D35.3 decided (the section's answer: 40% inset); D35.4 (strip
rows per device pixel), D35.5 (binned labels), D35.6 (twelve details)
open with recommendation A. D22.1 still waits for the user.

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
| M4 Complete figure | 32–39 | 32–35 done; 36 next |
| M5 Release | 40 | pending |

## 5. What to do next

Phase 36 (hover and 1-D brush), from its first step, once the architect
has answered D35.4–D35.6.

## 6. Notes for later phases

- A colorbar is `Stored::Bar(Box<dyn colorbar::Draw<'a> + 'a>)`; its
  per-resolve `colorbar::Content` is measured each layout pass
  (`measure_slot` / `measure_inset` in `legend/content.rs`), then laid
  out as a `colorbar::Block` whose `Placed` draws ticks and labels
  beside the interior. Hit testing and hover (Phase 36) do not see
  colorbars or legends.
- Strips: vector targets keep 256 rows (equal bins one row each,
  unequal bins 4 rows per px); raster targets one row per device px.
  PDF (Phase 37) gets the vector image.
- `Resolved` holds `(palette, key: Box<dyn Any>, Hatch)` records from
  `Fits::hatch`; only the figure's own marks report (user `Mark`s
  cannot). The first hatch reported for a key wins in the legend.
- Helpers (Phases 38–39): a heatmap preset gets its colorbar from the
  image mark's color handle; stacked bars can take `.hatch` per series.
- Scratch renders, the matplotlib/seaborn reference scripts and the
  hatch measurement example are in the session's scratchpad only.

## 7. Key files

`figure/src/colorbar.rs`, `figure/src/colorbar/{ramp,strip,block,side,edges,extend}.rs`,
`figure/src/legend/{guide,content,block,keys,swatch}.rs`, `figure/src/legend.rs`,
`figure/src/hatch.rs`, `figure/src/mark/{bar,histogram,area,density,color,rect}.rs`,
`figure/src/figure/{fits,resolve,resolved}.rs`, `figure/src/layout/{passes,slots}.rs`,
`figure/src/axis/{labels,placed}.rs`, `figure/tests/main/{colorbar,hatch}.rs`,
`dev/examples/src/wheels.rs`, `dev/examples/examples/wheels.rs`,
`dev/examples/tests/main/wheels.rs`, `.claude/decisions/phase-35.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --features raster --test main -- colorbar hatch theme
cargo test -p rustd3-figure --doc -- colorbar
cargo test -p rustd3-examples -- wheels
cargo run -p rustd3-examples --example wheels -- [dir] --hatch
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at 05aaad4.
- D35.4–D35.6 open with their recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
