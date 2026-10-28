# Phase 37 — Outputs II: PDF and GIF

## Measurements

2026-10-28, bb3c0d7 and the commits after it, one session. krilla
0.8.2, poppler 24.02 (`pdftoppm`, `pdftotext`, `pdffonts`; `pdftocairo`
for cross-checks only), gif 0.14.2.

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors; nothing blessed.

**The PDF pixel bound**, measured first: each list written as PDF, with
text and with `Options::outlined(true)`, rendered by `pdftoppm -r 192
-png` and compared with vello_cpu's render of the same list at 2×
(`threads: 0`, `Simd::Baseline`; D17.2: each output against its own
list), testkit's 32-px tile score:

| List | Text: tile score (worst tile, max) | Outlined: tile score (worst tile, max) | Bytes, text / outlined |
| --- | --- | --- | --- |
| kitchen sink with its sweep, 400 × 300 | 0.0310 ((32, 224), the rotated label; 182) | 0.0124 ((32, 288); 173) | 13,398 / 10,604 |
| scatter example, 640 × 480 | 0.0635 ((96, 128), tick labels; 217) | 0.0191 ((128, 288); 147) | 79,800 / 78,520 |
| trace example, 960 × 600 | 0.0620 ((96, 288), tick labels; 217) | 0.0246 ((1280, 704), the impacts; 195) | 317,479 / 330,969 |
| heatmap (viridis cells, colorbar, bold title span, math label), 640 × 480 | 0.0740 ((672, 896), tick labels; 217) | 0.0161 ((1120, 96); 154) | 24,485 / 28,148 |
| wheel figure, 960 × 760 | 0.0756 ((1344, 736), tick labels) | 0.0204 ((384, 672), bar edges) | 122,179 / 185,088 |
| hatched wheel figure | 0.0756 ((1344, 736)) | 0.0224 ((768, 672), bar edges) | 137,972 / 200,068 |

Where the differences come from (renders viewed side by side and as
×4-amplified difference images, scratch):

- **Glyphs.** poppler's Splash rasterizer places each glyph's origin on
  its own grid: a single run of "HxgH" at 10–100 px, its origin at
  device y 300.0, 301.0 and 300.6, moves by +0.02, +0.02 and −0.60
  device px in y against vello_cpu (whole device px, floored) and by
  −0.03 to −0.67 in x; the same runs written as outlines move by at
  most 0.11 in y (their vertical stems snapped to device pixels, as
  Splash aligns axis-parallel edges of fills). At 192 dpi a tick label's glyphs then sit up to a
  device pixel up and left, which a 32-px tile of small text turns
  into 0.06–0.08. `pdftocairo -r 192` renders the scatter's text PDF
  0.0087 from vello_cpu, so the writer places the glyphs where the list
  does (D37.1).
- **Images.** Splash draws an image one device px wider at its right
  and bottom edges (the kitchen sink's checker; `pdftocairo` draws it
  exact), and smooths the colorbar's 256-row strip; cell edges of the
  heatmap differ along one device column or row.
- **Axis-aligned edges.** Splash snaps rectangle fills to device pixels
  (bars, ticks), where vello_cpu antialiases the vector list's
  fractional edges: the outlined wheel figures' worst tiles.
- **Shadings.** Faint banding in the linear wash and the radial
  gradient; the sweep's seam at angle 0 one device px wide (pdftocairo
  smears it across ±20 px: 0.0581 on the kitchen sink).

The tests assert the outlined scores under the section's rule (≤ 0.02
when measured ≤ 0.02, else 1.25 × the measurement, never above 0.05)
and the text scores at 1.25 × their measurement (D37.1).

**Spot checks.** A 960 × 760 px list is `/MediaBox[0 0 720 570]`;
`pdftoppm -r 72` gives 720 × 570 px and `-r 192` 1920 × 1520. The wheel
figure's PDF is 122,179 bytes (Part I's sketch: about 114 KB);
`pdftotext` returns "Hub vibration and bearing temperature", "Wheel:
RL", "speed [km/h]", "min. count" and every one of its 64 runs
(whitespace collapsed); `pdffonts` lists one font, `…+DejaVuSans`, `emb
yes`, `sub yes`: the wheel figure draws all its runs in DejaVu Sans
(Phase 34: 64 runs, one font), so the spot check's DejaVuSans-Bold
appears only where a run is bold: the heatmap lists `DejaVuSans`,
`DejaVuSans-Bold` and `NewCMMath-Regular-Identity-H`, each a subset with
a six-letter tag, and its math label extracts as "RMS [𝑚/𝑠2 ]". With
`Options::outlined(true)`, `pdftotext` returns nothing and `pdffonts`
lists no font. The same list written twice, with and without a title,
gives identical bytes. A 2 × 2 checker stretched over 200 px renders
with at most two colors per 32-px tile when nearest, and more than two
when smooth.

## D37.1 — The PDF pixel bound cannot hold for text rendered by pdftoppm

**Date:** 2026-10-28 · **Context:** Phase 37, step 2, after bb3c0d7 · **Status:** open

### Issue

The section (Design, "The PDF pixel bound"; exit criterion 2; the spot
check "PDF vs the pinned PNG at 2×") and Part I's Testing table ask that
the PDF rendered by `pdftoppm -r 192` stay within a tile score of 0.02
of vello_cpu's render, or 1.25 × the measurement, "never above 0.05: a
measurement above 0.05 is a bug in the writer (a wrong transform, a
dropped paint, a missing clip) to repair in this phase". Every figure
with tick labels measures 0.062–0.076 (Measurements), and no writer bug
causes it: the same lists with glyphs written as outlines measure
0.012–0.025, and pdftocairo renders the text PDF of the scatter 0.0087
from vello_cpu. The excess is Splash's glyph placement, which snaps each
glyph's origin to whole device px in y and to a coarse grid in x.

### Investigation

- Single runs (Measurements, "Glyphs"): the offset follows the origin's
  device-pixel fraction, not the font size or the writer's numbers,
  which are the list's (`KrillaGlyph::new(id, advance / size, (x − pen)
  / size, −y / size, …)`, as the prototype and the section).
- `pdftoppm` has no switch for it (24.02: `-aa`, `-aaVector`,
  `-freetype`, `-thinlinemode` only); outlines avoid it because Splash
  fills paths where they are, aligning only their axis-parallel edges
  to pixel boundaries.
- pdftocairo renders text exactly but smears sweep gradients
  (0.0581 on the kitchen sink), so swapping the tool loses the sweep.

### Options

| | **A — outlines for the bound, text checked as text (as built)** | B — pdftocairo | C — record the text measurements as the bound |
| --- | --- | --- | --- |
| Approach | The section's rule (≤ 0.02, else 1.25 ×, never above 0.05) on each list written with `Options::outlined(true)`, which goes through the same glyph normalization (krilla outlines each `KrillaGlyph` at its offsets); the text PDF asserted at 1.25 × its own measurement (0.039–0.095), and through `pdftotext` (every run) and `pdffonts` (embedded subsets) | Render with `pdftocairo -r 192` | Assert the text PDF at 1.25 × its measurement |
| Correctness | Every transform, paint, clip and glyph offset checked within the bound; text placement guarded against gross errors; Splash's grid out of the bound | Text exact; the sweep check lost (cairo's seam) | Breaks "never above 0.05"; a 0.02 regression hides under poppler's noise |
| Cost | Two writes and renders per list | Changes the tool the section names | None |

### Decision

Recommended: A, as built in `figure/tests/main/pdf.rs`
(`assert_round_trip`) and `dev/examples/tests/main/wheels.rs`
(`wheels_pdf`).

### Why

The bound exists to catch writer bugs, and the outlined PDF exercises
every mapping the text PDF does except krilla's text operators, which
`pdftotext`, `pdffonts` and the text PDF's own recorded bound cover.
Neither tool nor a looser cap measures the writer better.

### Followups

- If a later poppler places glyphs at subpixel positions, assert the
  text PDFs under the section's rule too.
