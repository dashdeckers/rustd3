# rustd3 — session handoff 39

2026-10-28 · **Phase 37 — Outputs II: PDF and GIF**: **Steps 1–3 done; Phase 37 complete**, with D37.1 and D37.2 open (recommended as built) · next: **Phase 38 — Helpers I: statistical charts**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- bb3c0d7 `feat(figure)` — Step 1: `figure/src/pdf.rs` (`pdf::CAPS`,
  `pdf::Options { outlined, title }` with builders, `pdf::write`,
  `pdf::Error`), `figure/src/pdf/image.rs` (`Rgba`, krilla's
  `CustomImage` for smooth images), `Figure::save_pdf`,
  `figure::Error::Pdf`; tests in `figure/tests/main/pdf.rs` (units,
  identical bytes, nearest and smooth images, selectable text and
  subset fonts, errors, clips of no area) over
  `figure/tests/main/poppler.rs` (`pdftoppm`, `pdftotext`, `pdffonts`
  on stdin).
- 60d6c78 `test(figure)` — Step 2: the poppler round trips, text and
  outlined, of the kitchen sink, the scatter and trace examples and a
  heatmap figure (colorbar, bold span, math label); `wheels_pdf` in
  `dev/examples/tests/main/wheels.rs` (the wheel figure, plain and
  hatched); `decisions/phase-37.md` with the measurements and D37.1.
- fe27802 `feat(figure)` — Step 3: `figure/src/gif.rs` (`gif::Repeat`,
  `gif::Writer` with `new`, `frame`, `finish`; `gif::Error`), tests in
  `figure/tests/main/gif.rs`; the GIF measurements.
- 85381d5 `fix(figure)` — the PDF tests' bound helper only with `raster`
  (dead code with `pdf` and `fonts` alone).
- The closing docs commit: D37.2, the index, this handoff.

Exit criteria:

1. `cargo test -p rustd3-figure --features pdf,gif,raster --test main --
   pdf gif`: 19 pass (13 PDF, 6 GIF), no warning.
2. Met as D37.1 recommends: outlined PDFs within the section's rule
   (measured 0.0124–0.0246; the wheel figure 0.0204, hatched 0.0224),
   PDFs with text within 1.25 × their measurement (0.031–0.076, poppler's
   glyph placement); `pdftotext` returns every run and the spot labels;
   `pdffonts` lists embedded subsets (`cargo test -p rustd3-examples --
   wheels_pdf`).
3. `for f in pdf gif; do cargo check -p rustd3-figure
   --no-default-features --features $f; done`: both build.
4. `scripts/check.sh quick` green on every commit; `full` green at
   85381d5 (see §9).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D37.1 (the PDF pixel bound under pdftoppm holds for
outlines, not for text: open, recommended A, as built), D37.2 (fourteen
details as built, among them `pdf::Error::List`, `gif::Error::Raster`
and `gif::Writer` behind `fonts`: open, recommended A). D22.1 still
waits for the user.

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
| M4 Complete figure | 32–39 | 32–37 done; 38 next |
| M5 Release | 40 | pending |

## 5. What to do next

Phase 38 (statistical helpers), from its first step, once the architect
has answered D37.1 and D37.2. If D37.1 goes another way, only the two
round-trip helpers change (`assert_round_trip` in
`figure/tests/main/pdf.rs`, `wheels_pdf`).

## 6. Notes for later phases

- PDF pixels: poppler's Splash floors glyph origins to whole device px
  in y, draws images one device px wider at right and bottom, and snaps
  axis-parallel edges of fills; compare outlined PDFs when pixels
  matter. `pdftocairo` places text exactly but smears sweep seams.
- `pdf::write` lowers nothing (`pdf::CAPS` is everything); markers keep
  point order; a group clipped to no area draws nothing.
- `gif::Writer` keeps its figure cache, a vello_cpu renderer and glyph
  outlines between frames; NeuQuant dominates a frame's cost (about
  70 ms of 87.5 at 960 × 760 in release on this container).
- The facade (Phase 40) re-exports `pdf` and `gif` under its features;
  `gif::Writer` needs `fonts`.
- Scratch renders and the scratch tests that wrote them are in the
  session's scratchpad only.

## 7. Key files

`figure/src/pdf.rs`, `figure/src/pdf/image.rs`, `figure/src/gif.rs`,
`figure/src/figure/save.rs`, `figure/tests/main/{pdf,gif,poppler}.rs`,
`dev/examples/tests/main/wheels.rs`, `.claude/decisions/phase-37.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --features pdf,gif,raster --test main -- pdf gif
cargo test -p rustd3-examples -- wheels_pdf
for f in pdf gif; do cargo check -p rustd3-figure --no-default-features --features $f; done
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  85381d5's tree in 12 min 47 s.
- D37.1 and D37.2 open with their recommendations; D22.1 waiting for
  the user.
- Conventional commits, no attribution.
