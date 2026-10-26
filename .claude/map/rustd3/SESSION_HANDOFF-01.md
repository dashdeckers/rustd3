# rustd3 — session handoff 01

2026-10-26 · Phase 1 complete · next: **Phase 2 — Test kit and grading harness**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 1 is complete; `scripts/check.sh quick` and `full` pass at
3af8543.

- The workspace: ten packages, every library dependency but iced locked
  (327 packages in `Cargo.lock`; jiff 0.2.38, typst 0.15.1, kurbo
  0.13.1, peniko 0.6.1, vello_cpu 0.3.0, krilla 0.8.2, png 0.18.1,
  tiny-skia 0.12.0, resvg 0.48.1; `cargo update --dry-run` changes
  nothing).
- Lint tables, `clippy.toml` (no ambient clocks or zones in library
  crates, checked by hand) and `dev/clippy.toml`.
- `rustd3-fonts`: the three fonts as `Font` constants backed by one
  static each (a plain `const` of `include_bytes!` is copied into every
  crate that uses it; measured), four parse tests, and its licence
  files.
- Licences: `LICENSE-MIT`, `LICENSE-APACHE`, `core/LICENSE-d3` (D1.1),
  `fonts/LICENSE*` (D1.2).
- `scripts/check.sh quick|full`; `CLAUDE.md`; `.claude/` guides,
  `DECISIONS.md` and `decisions/phase-01.md` (measurements, D1.1 and
  D1.2, both decided).

Decisions:

- **D1.1** (decided B) — d3-ease 3.0.1 is BSD-3-Clause, so
  `core/LICENSE-d3` is the ISC block for the sixteen ISC modules, then
  d3-ease's LICENSE verbatim.
- **D1.2** (decided B) — the font licence texts come whole from
  typst-assets 0.15.1's `NOTICE` (lines 139–236 and 103–130), not from
  the incomplete copies in `proto/fonts/`.

Commits: a8ae77f (spec tree), 0728bd2 (skeleton), a63b952 (lints),
c16e92b (fonts), 395e21f (check script), ddfdcb4 (CLAUDE.md and
.claude), b3e763a (measurements), fa4b8b1 (D1.1, D1.2 open), e244172
(handoff), b3e249d (D1.1, D1.2 decided), 940b643 (`core/LICENSE-d3`),
3af8543 (font licences), and this handoff.

Measurements: cold `quick` over all ten packages 377 s, 3.3 GB
`target`; `full` 67 s after it, 3.8 GB; no-op check 0.3 s.

Tests: 4 (fonts). No harness yet, so no counts or floors.

## 3. Project management

- `goon.yaml` still has `repo: none`; no GitHub repository was created
  (pushing is not allowed in agent sessions).
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.
- Licence-style facts the spec gets wrong: write the decision, finish
  the independent parts of the phase, and stop hard only when a
  decision would change code other phases build on (architect,
  2026-10-26).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1 workspace · 2 harness · 3 first port | 1 done; 2–3 pending |
| M1–M5 | 4–40 | pending |

## 5. What to do next

Phase 2 — Test kit and grading harness: read `rustd3.md`'s front matter
and "### Phase 2 —", and `.claude/decisions/phase-01.md`.

## 6. Known risks and open items

- Phase 40: the root README's credits name d3-ease's BSD-3-Clause
  licence beside d3's ISC (D1.1 Followups).
- `cargo tree -d` shows duplicates that come from typst's own graph
  (vello_cpu 0.0.8 through typst-svg → hayro, syn 3 through
  bytemuck_derive, two skrifa/read-fonts); nothing to decide.
- The risks of handoff 00 stand (Phase 22's benchmark, Phases 9, 14,
  27–28, 34; the iced pin).

## 7. Key files

`CLAUDE.md`; `.claude/DECISIONS.md`; `.claude/decisions/phase-01.md`;
`rustd3.md` (front matter, then "### Phase 2 —"); `proto/README.md`.

## 8. Commands

```bash
scripts/check.sh quick          # every commit
scripts/check.sh full           # end of a phase (background)
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green.
- Decisions D1.1 and D1.2 written and decided.
- Conventional commits, no attribution.
