# Phase 13 — Text with typst

## Measurements

2026-10-27, at 2678df4; `scripts/check.sh full` green at d292c8f in 68 s (warm) and again at the end of the phase.
Phase 13 does no core work: the harness stays at 9 suites, 3,314
assertions, 2,623 passing, 691 skipped, 80 floors.

**Spot checks** (typst 0.15.1, bundled fonts, 11 px, black, centred;
`figure/tests/main/text.rs`, each a named test, values to three places):
every row of the section holds as measured. "0123456789" 69.985 wide,
8.320 tall, one run; x 6.510 / 6.016 / 0, g 6.982 / 6.161 / 2.288;
"line one\nline two" runs at (0, 8.357) and (0.215, 24.021), 43.817 ×
24.176; "Wheel: " 41.642 and "RL" 13.771 in (196, 78, 82); "a $ # _ b"
49.425 as one literal run; 113.137, 32.629, 9.168; "RMS [$m slash
s^2$]" six runs in two faces, the superscript at 7.70, 61.510; "漢字"
ids [0, 0], `missing() == 2`; "$ x^ $" beside "ok" and "fine" is
`Error::Markup` and the other two shape (four compilations: the batch,
then each label); "Wheel: RL" ids 58, 75, 72, 72, 79, 29, 3, 53, 47;
`bounds` of g at Middle/Top (−3.491, 0, 3.491, 8.449). No spot-check
label draws a typst warning (`batch.rs`, unit test), so the section's
first listed decision is not met.

**Timings** (reported, not asserted;
`one_document_per_batch_costs_less_than_one_per_label`, `cargo test
--release`, 2-core container, four runs; after one warm-up label, each
arm shaping its own 40 strings so neither reuses what typst memoized for
the other): 40 new tick labels take 2.0–2.2 ms as one document (one run
4.8 ms) against 2.6–3.2 ms one document each (the section: 1.8 against
6.0); the per-label documents cost less than the section measured
because every batch's world shares the hashed library and font book.
12 labels in a new cache take 0.67–0.92 ms, and again in another new
cache, typst memoizing the compile, 150–210 µs including our
conversion of twelve pages (the section: 22 µs for the compile). The
release test build took 458 s cold for the test's dependencies.

**SVG `<text>` against outlines**, both rendered by resvg at 2× with
the three bundled fonts loaded: 370 of 614,400 pixels (0.06 %) differ
by more than 16/255 (bound 1 %), on the script-size math glyphs (typst
draws their `ssty` variants; resvg re-shaping "2" takes the default
glyph) and on anti-aliasing specks.

**Ink against boxes** (vello_cpu, scale 2, eleven labels at every
vertical anchor, one under a quarter turn): each label's ink lies in
its placed box and reaches its top and bottom within a device px; top,
middle and bottom anchors have that edge of the ink on the anchor
point; no ink outside the boxes.

## D13.1 — Statements of Phase 13 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 13, 690db00 … 2678df4 · **Status:** open

### Issue

1. **The baseline.** Conversion: "`ascent` = baseline y of the first
   text item (`Frame::baseline()` is unset and returns the height)";
   Part I, Text and math: "The frame reports no baseline either, so it
   is read from the first text item." typst inlines small soft frames
   into their parent, so the page frame of a label that starts with
   stacked math holds no item on the line's baseline: `$a/b$` has its
   numerator at 3.396 and typst's line baseline at 7.730; `$sqrt(x)$`
   has the radical sign at 0.880 and x on 8.806. Read as written, a
   baseline anchor puts such a label 4–8 px off its neighbours.
2. **Commits.** Steps 2–4 (fonts and world, batch shaping, the cache)
   are one commit (7cca9f1) and `Shaped::place` (step 5) landed in it:
   until the cache exists nothing of the fonts, world or batch is
   reachable from outside the crate, and the workspace denies dead code.
   Step 5's commit (efd0912) holds its tests. Four commits, not six.
3. **What a key holds.** "`pub struct Key(Arc<str>)`" and "`fill` from
   `Paint::Solid` (other paints: the style's fill)": the snippet alone
   does not give conversion the style's fill.
4. **`Text::typst`.** The API block has the free function
   `text::typst(markup) -> Text`; "Lift `Text`, `Span`, `span`,
   `Text::typst`" names the prototype's method that writes the markup,
   which the spot check "markup `#"Wheel: "#text(…)`" needs.
5. **A span before markup.** `#"a"` followed by raw markup that starts
   with `(`, `[` or `.` continues the span's expression (`#"a"[b]` calls
   the string).
6. **Without `fonts`.** "`Fonts::bundled()` … DejaVu Sans, DejaVu Sans
   Bold, New Computer Modern Math" cannot hold without the bundled
   fonts, and the fallback list "user families, "DejaVu Sans", "New
   Computer Modern Math"" names families typst then warns about
   ("unknown font family", `typst-library-0.15.1/src/text/mod.rs:1576-1587`).
7. **Paths and public fields.** `text::anchor` must be public for
   `anchor::{Horizontal, Vertical}`, so `Anchor` defined there would
   have two paths; `Anchor { pub h, pub v }` and `Stats { pub entries,
   … }` have public fields but are not in Conventions item 4's list.
8. **Shapes.** "lines as thin rectangles of the stroke thickness,
   `Rect`, `Curve`" leaves strokes of rects and curves, caps, dashes,
   the even-odd rule and group clips open.
9. **SVG `<text>`.** The template has no `xml:space`; without it SVG
   collapses a run's leading or trailing space (" plain", "Wheel: ") and
   every x after it lands one glyph off. The math face's `name` table
   gives "NewComputerModern Math"; typst's "New Computer Modern Math"
   comes from its exceptions table
   (`typst-library-0.15.1/src/text/font/exceptions.rs:185-190`).
10. **Smaller details.** `Error::Markup`'s `markup` (prepare takes keys,
    so the label's own markup is not at hand); hits for a key repeated
    within one `prepare`; the 8-bit color of runs and fills; the
    timing rows measured outside release.

### Investigation

- (1) `typst-library-0.15.1/src/layout/frame.rs:180-186, 227-230`
  (`push_frame` inlines a soft frame of at most five items);
  `typst-layout-0.15.1/src/inline/box.rs:40` (a box's frame is hard, so
  it stays a group, with its baseline set). Probed: with the snippet's
  text inside `box(…)`, every spot-check row measures the same (the
  section notes "plain `#box[…]` gives the same numbers"), and the box
  group's baseline is 7.7297 for `$a/b$`, 8.8055 for `$sqrt(x)$`, the
  first text item's for plain text, the first line's for two lines.
  Test: `stacked_math_keeps_the_baseline_of_its_line`.
- (9) resvg's render and the pixel count in Measurements.

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | The snippet is `#align(…, box(text(…)[…]))`; `ascent` is the baseline of the first frame that has one (the box), else the height. A `#pagebreak()` in raw markup is then typst's "pagebreaks are not allowed inside of containers", `Error::Markup`; `Error::Pages` stays for markup that closes the snippet's brackets (tested) | The first text item's baseline: Baseline anchors of labels that start with a fraction or radical are off by 4–8 px |
| (2) | Four commits; each passes `check.sh quick` | Six commits with transient `#[expect(dead_code)]` |
| (3) | `Key { snippet: Arc<str>, fill: [u8; 4] }`, private; Eq and Hash over both (equal snippets carry equal fills); `Key::as_str()` public | Paint other paints black, or parse the fill back from the snippet |
| (4) | `Text::typst(&self) -> String` beside the free `text::typst` | Name the method `markup` |
| (5) | A span followed by markup ends with `;` (consumed by typst); span after span unchanged, so the spot check's markup holds | Verbatim; such markup fails or misreads |
| (6) | `Fonts::bundled()` and `Default for Cache` only with `fonts`; `Fonts::with` then holds the user fonts alone; the family list is the families of the faces present, in fallback order, each once (with `fonts`: user families, "DejaVu Sans", "New Computer Modern Math") | — |
| (7) | `Anchor` is defined in `text.rs` (as `Part` in `list.rs`, D12.1 (14)); `Anchor` and `text::Stats` are plain data types with public fields | — |
| (8) | A shape's fill is a `Fill`; any stroke (lines too) is a `Fill` of `kurbo::stroke`'s outline with its thickness, cap, join, miter limit and dashes (a butt-capped line is the thin rectangle); the nonzero rule; group clips and `Image`, `Link`, `Tag` ignored | — |
| (9) | `xml:space='preserve'` on each `<text>`; the family as the `name` table spells it | Collapsed spaces; or typst's family names |
| (10) | `markup` is the key's whole snippet; a key repeated in one `prepare` is one miss, then hits; colors are typst's sRGB rounded to 8 bits (`Color::from_rgba8`), so a span's fill equals its `Rgb`; timings reported from a release test run (handoff) | — |

### Decision

Recommended: A for all ten, already built (690db00 … 2678df4).

### Why

(1) is the spec's own observation ("plain `#box[…]` gives the same
numbers") turned into the source of the baseline: typst's line baseline,
not a reconstruction, and no spot check moves. (2) follows the
workspace's lints; (3)–(10) are the readings closest to the section that
compile, keep one public path per item, and make the spot checks
checkable.

### Followups

- Phase 14: theme sizes go through `Style::size` in px; the second listed
  decision (4-decimal sizes splitting the cache) is not met here.
- Phase 16: layout may anchor labels on their baseline, including math
  that starts with a fraction (1).
- Phase 17: `fig.check` reads `Shaped::missing` and `text::Error`; no
  label of the spot checks draws a typst warning, so `Shaped` keeps none.
- Phase 37: krilla's text takes `GlyphRun::text` and the glyph ranges;
  the math face's family as its `name` table spells it (9).
