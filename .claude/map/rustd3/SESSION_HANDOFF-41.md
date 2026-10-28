# rustd3 — session handoff 41

2026-10-28 · **Phase 39 — Helpers II: presets, labels and axis defaults**: D38.1's commit and **Steps 1–2 done (session 39a, cut after Step 2)**, with D39.1 and D39.2 open · next: **Phase 39, Step 3** (presets: line, scatter, bar), then Step 4

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 508cb45 `feat(figure)` — D38.1 (decided A): `Theme::get_color`,
  `get_gray`, `get_line_width`, `get_patch_width`, `get_density_width`,
  `get_marker` (renamed from the crate-private readers, every call site
  updated); `list::brush`; `mark::Ctx::snap` with `mark::Snap` (`rect`,
  `width`, `line`; the file moved from `layout/snap.rs` to
  `mark/snap.rs`); `mark::guard::{band, polygon}` (`clip` stays
  crate-private). Doc comments say what a custom mark uses each for.
- 149b8b9 `feat(figure)` — Step 1: `helpers::labels::{bars, Bars, pie,
  Pie}` and `helpers::legend::{sizes, Sizes, colors, Colors}`, with
  crate-private plumbing: `Fits::locale`, `Ctx::locale`, `Ctx::line`,
  `mark::{reach, within}` pub(crate), `legend::Entry` (an optional dot
  diameter; `legend::Draw::entries` takes the theme), key layout growing
  to hold large dots.
- 6510a55 `feat(figure)` — Step 2: `Axis::minor(bool)` (core's
  `Ticks::minor` at the theme's minor size and width, in styles with tick
  marks; `axis::Draw::minors`, `Placed::minors`), `helpers::axes::defaults`.
- 09f23e7 `fix(figure)` — the labels' locale and line readers behind
  `helpers` (`full`'s featureless build warned they were unused).
- The closing docs commit: `decisions/phase-39.md`, the index, this
  handoff.

Exit criteria so far (the phase's are met at its end):

1. `cargo test -p rustd3-figure --features helpers --test main -- labels
   axes`: 55 pass (the filter also matches other modules' label tests);
   `-- helpers::labels helpers::axes`: 15 (labels and legends 9, axes 6),
   the step 1–2 spot rows among them; `presets` comes with Step 3.
2. Presets and `--example presets`: Steps 3–4.
3. `cargo test -p rustd3-figure --features helpers,png --doc`: passes, with
   examples for `labels`, `labels::bars`, `labels::pie`, `legend`,
   `legend::sizes`, `legend::colors`, `axes::defaults`, `Axis::minor`,
   `mark::Snap` and `mark::guard`.
4. `scripts/check.sh quick` green on every commit; `full` green at 09f23e7
   (see §9).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D39.1 (value labels and scatter legends need crate-private
text, locale and legend internals; open, recommended A: publish a text
kit for custom marks — `Fits::label`, `Ctx::label`, `Fits::locale`,
`Ctx::locale`, `Ctx::line`, a note-style reader — keeping `Guide` sealed;
built in-crate, so A is a visibility change), D39.2 (ten details as built;
open, recommended A). D22.1 still waits for the user.

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
| M4 Complete figure | 32–39 | 32–38 done; 39 steps 1–2 done, 3–4 next |
| M5 Release | 40 | pending |

## 5. What to do next

If D39.1 is answered A: one commit making the listed items public with
doc comments (the labels then call them by the same names). Then Phase
39 Step 3 (`helpers/chart.rs`, `helpers/chart/{line,scatter,bar}.rs`,
`figure/tests/main/helpers/presets.rs`: one test per preset counting
marks and checking `fig.check`; the stack-offset spot rows; the line
sketch of Helpers as a doctest), then Step 4 (area, pie and donut,
heatmap; `dev/examples/examples/presets.rs`; the heatmap `keys` decision
the section lists, recommended answer to record), then `check.sh full`.

## 6. Notes for later phases

- Value labels shape their own text by the line box (as tick labels,
  D16.4), not through `note` (ink anchors): `helpers/labels.rs`'s `place`.
  The pie preset can use `labels::pie` with its own radii; labels are
  centred on the plot area's middle.
- `labels::bars(...).format(spec)` takes a `format::Specifier`
  (`".1f".parse()?`).
- A size legend's dot diameters reach the key layout through
  `legend::Entry::size`; a `Legend` of an ordinal palette passes `None`.
- `Axis::minor` draws nothing on time, band and point axes (core's empty
  default) and nothing under the grid styles (no tick marks).
- The scratch example that rendered the views is in the session's
  scratchpad only (`dev/examples/examples/scratch39.rs` is not committed).

## 7. Key files

`figure/src/helpers/{labels,legend,axes}.rs` and their submodules,
`figure/src/axis.rs`, `figure/src/axis/{draw,placed}.rs`,
`figure/src/legend/{entry,keys,swatch}.rs`, `figure/src/mark/snap.rs`,
`figure/src/mark/guard.rs`, `figure/tests/main/helpers/{labels,axes}.rs`,
`.claude/decisions/phase-39.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --features helpers --test main -- labels axes
cargo test -p rustd3-figure --features helpers,png --doc
cargo check -p rustd3-figure --no-default-features --features helpers
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at 6510a55
  in 13 min 32 s with featureless-build warnings, fixed by 09f23e7, and
  green at 09f23e7 in 13 min 9 s with no warnings (warm, the benchmark
  smoke included).
- D39.1 and D39.2 open with their recommendations; D22.1 waiting for the
  user.
- Conventional commits, no attribution.
