# rustd3 — session handoff 14

2026-10-27 · **Phase 13 — Text with typst** complete in one session (D13.1 open) · next: **Phase 14 — Figure model and first layout**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 13** (four code commits, `check.sh quick` green on each, `full`
green at the end):

- 690db00 `feat(figure): Text and spans` — `text::{Text, Span, span,
  typst, Style, Anchor, anchor::{Horizontal, Vertical}}`; spans as typst
  string literals (escapes, `strong`, `text(fill: …)`), raw markup
  verbatim, `;` after a span that precedes markup.
- 7cca9f1 `feat(figure): typst shaping and the text cache` — steps 2–4
  and `Shaped::place` in one commit (dead code otherwise): `text::Fonts`
  (`bundled()` behind `fonts`, `with(extra)`), the private `World`,
  `text::{Key, Shaped, Cache, Stats, Error}`, batch shaping with the
  per-label fallback, frame → runs and fills.
- efd0912 `test(figure): placed labels` — snapshot
  `list_dump_of_labels.txt`; the vello ink-at-anchors test.
- d292c8f `feat(figure): SVG text elements` — `svg::Text::Elements`;
  snapshot `svg_text_elements_of_labels.txt`; resvg `<text>` vs outlines.
- 2678df4 `test(figure): batch timings without shared memoization`.

Exit criteria: (1) `cargo test -p rustd3-figure`, `--all-features` and
`--no-default-features` pass; every Spot-checks row is a named test in
`figure/tests/main/text.rs` (timing rows report only). (2)
`a_second_prepare_of_the_same_labels_compiles_nothing`,
`labels_unused_for_32_generations_are_dropped` (32 begins keep, the 33rd
evicts). (3) `a_rejected_label_does_not_stop_its_batch`. (4) both
snapshots pass; `resvg_draws_text_elements_close_to_the_outlines`: 0.06 %
of pixels over 16/255 (≤ 1 %). (5) `scripts/check.sh full` green.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-13.md`): measurements (spot checks,
release timings, `<text>` vs outlines, ink vs boxes), and **D13.1** (open,
A recommended, all built). The one later phases meet most: (1) the key's
snippet is `#align(…, box(text(…)[…]))` and `ascent` is the box frame's
baseline, typst's line baseline, because typst inlines line frames and a
label that starts with `$a/b$` or `$sqrt(x)$` has no text item on its
baseline; every spot check measures the same. Also: (3) `Key` holds the
style's fill beside the snippet and has `as_str()`; (4) `Text::typst(&self)`
returns the markup beside the free `text::typst`; (6) `Fonts::bundled()`
and `Default for Cache` exist only with `fonts`; (7) `Anchor` lives in
`text.rs`, `Anchor` and `text::Stats` are plain data with public
fields; (9) `<text>` carries `xml:space='preserve'`, the math face's
family is "NewComputerModern Math" (its `name` table).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11, 12, 13 done; 14 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 14: read `rustd3.md`'s front matter and "### Phase 14 —". If the
architect answers D13.1 with anything but A, apply that first (for (1):
drop `box(…)` from `Key::new`, read the first text item's baseline in
`convert.rs`, re-bless both label snapshots after viewing them, and
rewrite `stacked_math_keeps_the_baseline_of_its_line` and
`a_page_break_is_rejected_and_one_outside_the_box_is_two_pages`).

## 6. Notes for later phases

- Phase 14: `figure::Cache` wraps `text::Cache` (`Cache::new(fonts)`,
  `Default` only with `fonts`); call `begin()` once per figure draw, then
  `prepare` all of a draw's keys at once (one typst document); `get`
  returns `Result<&Shaped, &Error>`. Theme sizes in px through
  `Style::size` (4 decimals in the key). Tests that must run without
  `fonts` build `Fonts::with` from the three files
  (`figure/tests/main/text.rs`, `fonts()`).
- Phase 16: `Shaped::bounds(anchor)` is the box relative to the anchor
  before rotation; width is the advance width (ink can overhang it by a
  side bearing), top and bottom are the ink's.
- Phase 17: `Shaped::missing()`, `text::Error::{Markup, Pages}`; no
  warnings are kept (none measured).
- Phase 37: runs carry `text` and per-glyph byte ranges; fills are
  typst's rules and stroke outlines.
- Handoff 13's notes for Phases 15, 18, 19, 37 stand.

## 7. Key files

`figure/src/text.rs`, `figure/src/text/{anchor,batch,cache,convert,error,
fonts,key,shaped,span,style,world}.rs`, `figure/src/svg.rs`;
`figure/tests/main/text.rs`, `figure/tests/snapshots/{list_dump_of_labels,
svg_text_elements_of_labels}.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background, ~70 s warm)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features --test main text
TZ=Asia/Kathmandu cargo test --release -p rustd3-figure --test main -- one_document --nocapture
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the
  end of the phase.
- D13.1 open with its recommendation.
- Conventional commits, no attribution.
