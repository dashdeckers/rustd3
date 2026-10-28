# Phase 37 — Outputs II: PDF and GIF

## Measurements

2026-10-28, bb3c0d7 … 85381d5 and the closing docs commit, one session.
krilla 0.8.2, poppler 24.02 (`pdftoppm`, `pdftotext`, `pdffonts`;
`pdftocairo` for cross-checks only), gif 0.14.2. `scripts/check.sh
full` green on 85381d5's tree in 12 min 47 s (the architect's cold rewarm
before the session: 29 min 18 s).

**Lines** (71c1158..fe27802): library +822 (`pdf.rs` 565, `pdf/image.rs`
61, `gif.rs` 176, `save.rs` and `lib.rs` 20, docs included); tests +758
(`figure/tests/main/{pdf,gif,poppler}.rs` 644, `wheels_pdf` and the
helper's `#[path]` 104, `main.rs` 10). The section estimated about 800
(500 + 300).

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

**GIF** (`figure/tests/main/gif.rs`, gif 0.14.2's own decoder):

- **Agreement.** A frame of the scatter example (640 × 480 at 1×,
  `Frame::from_rgba_speed(…, 10)`: NeuQuant, its 5,600 translucent dots
  far over 256 colors) against the pixels `save_png` encodes (the same
  list, `Settings::parallel()`): max 60/255, tile score 0.0095 at
  (288, 224), 0.162% of channels over 16. Asserted at 1.5 ×: max ≤ 90,
  tile score ≤ 0.0143.
- **Spot checks.** Delays of 40, 16.7 and 5 ms decode as 4, 2 and 2 cs;
  a 40,000 px wide figure at 2× is `Error::Size { width: 80000, height:
  20 }` before anything is written; 30 frames of a zooming window with
  `Repeat::Forever` decode as 30 frames of 160 × 120 with
  `Repeat::Infinite`, `Repeat::Times(3)` as `Finite(3)`; a theme
  background at opacity 0.5 is `Error::Translucent`.
- **Cost per frame** (report only; release, 2-core container, the wheel
  figure at 1×, 960 × 760, 30 frames after a first): the list with its
  text 2.1 ms (5.9 ms for the first, typst memoizing the repeated
  labels after it), vello_cpu's render 15.8 ms (22.7 ms first; one
  worker thread on two cores), `Writer::frame` 87.5 ms in all, so
  NeuQuant and LZW about 70 ms; 2,520,789 bytes for the 30 frames.
  Part I's sketch: 12.5 ms raster, 4.8 ms list, 2.0 ms text.
- **Viewed**: a 30-frame window moving over a damped sine (480 × 320,
  40 ms), frames 0, 15 and 29 decoded: axes, labels and the line as in
  the PNGs, no palette banding visible.

## D37.1 — The PDF pixel bound cannot hold for text rendered by pdftoppm

**Date:** 2026-10-28 · **Context:** Phase 37, step 2, after bb3c0d7 · **Status:** decided

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

A (architect, 2026-10-28): the 0.05 tile-score rule applies to PDFs
written with glyphs as outlines, which share the text path's placement;
PDFs with real text are checked by `pdftotext` and `pdffonts` and held to
1.25× their own measured score, since poppler snaps glyph origins to
device pixels.

### Why

The bound exists to catch writer bugs, and the outlined PDF exercises
every mapping the text PDF does except krilla's text operators, which
`pdftotext`, `pdffonts` and the text PDF's own recorded bound cover.
Neither tool nor a looser cap measures the writer better.

### Followups

- If a later poppler places glyphs at subpixel positions, assert the
  text PDFs under the section's rule too.

## D37.2 — Details of Phase 37 that the section leaves open, or that do not hold as written

**Date:** 2026-10-28 · **Context:** Phase 37, bb3c0d7 … 85381d5 · **Status:** decided

### Issue

1. **`pdf::Error`** has no variant for a list that fails `list::check`,
   which every backend runs first (Part I, Drawing and output).
2. **`gif::Error`** has no variant for a frame vello_cpu cannot render:
   a list that fails its check, a side between 65,409 and 65,535 device
   px (vello_cpu's limit, D12.1 (1), which GIF's `u16` sides allow), a
   size below one device px.
3. **Features.** `Writer::new` makes the writer's `figure::Cache`, whose
   `Default` needs `fonts` (D13.1 (6)); exit criterion 3 builds `gif`
   without `fonts`.
4. **The writer's state.** The section lists the encoder, size, scale and
   cache; a renderer made per frame would start vello_cpu's pool per
   frame, and `Bounds::of` reads glyph outlines.
5. **"Points are grouped by paint index"**: grouped across the command,
   dots of two paints would be drawn in another order than every other
   backend draws them.
6. **Commands that draw nothing** in the other backends but not in PDF:
   a stroke of width 0 is a device hairline in PDF; markers of size 0,
   runs of size 0 or under a singular transform, images and paths of no
   area.
7. **Clips of no area.** Splash (pdftoppm) draws everything under a clip
   to a degenerate path: the first draft failed
   `a_clip_of_no_area_clips_everything`.
8. **Glyph text ranges.** krilla slices `text[range]` for each glyph and
   panics on a range outside the run's text or off a char boundary
   (`krilla-0.8.2/src/text/group.rs:114-115`, `src/content.rs:557`);
   `list::check` does not check the ranges.
9. **Errors mid-page.** krilla asserts on drop that every push was
   popped (`krilla-0.8.2/src/surface.rs:611-627`), so a command that
   fails inside a group would panic.
10. **krilla's own details**: which list font a `KrillaError::Font` names;
    gradients without stops; sweep angles (krilla's PostScript `atan` is
    in degrees, peniko's in radians); radial gradients with repeat or
    reflect (krilla pads them, `krilla-0.8.2/src/graphics/paint.rs:69-72`).
11. **Spot checks as written.** pdf-writer writes `/MediaBox[0 0 720
    570]`, without the space; "page 720 × 570 pt" is also read through
    `pdftoppm -r 72`. The wheel figure lists one font (no run is bold,
    Phase 34), not "DejaVuSans and DejaVuSans-Bold"; it is 122,179 bytes,
    not about 114 KB.
12. **"The test figure"** is not named.
13. **Sharing the poppler helper** between rustd3-figure's and
    rustd3-examples' tests without a manifest change.
14. **The GIF agreement** "against the PNG": exit criterion 1's features
    (`pdf,gif,raster`) have no `png`, so `save_png` is not there.

### Options

| | **A — as built** | B — as written or the alternative |
| --- | --- | --- |
| (1) | `Error::List(#[from] list::Error)` | Map check failures to `Unsupported` or `Krilla(String)` |
| (2) | `Error::Raster(#[from] raster::Error)` | `Encoding(String)`; or `Size` at 65,408 with GIF's message |
| (3) | `Writer` with `fonts`; `Repeat` and `Error` with `gif` alone (as `save_png`, D15.1 (8)); `save_pdf` with `fonts` | `Writer::new` taking a cache |
| (4) | Private `raster::Renderer` and `Outlines` kept between frames | — |
| (5) | The fill switches where the index changes along the points, in point order, as the vello_cpu replay and the SVG writer group them; per point for gradients, placed by the inverse of each point's placement (D12.1 (3)) | Sorted by index |
| (6) | Skipped: PDF then draws what the other backends draw | A hairline in PDF only |
| (7) | A group whose clip has a bounding box of no width or height skips its commands until its `Pop` | Leave it to the viewer |
| (8) | Such a glyph maps to no text (`0..0`) | Panic, or a new `list::check` rule |
| (9) | Open groups are popped before the surface drops, then the error returns | — |
| (10) | `Error::Font { index }` of the font krilla names (else `Krilla`); no stops: transparent; degrees; pad, documented on `pdf::write` | — |
| (11) | Tested as found: `/MediaBox[0 0 720 570]`, 720 × 570 px at 72 dpi, 1920 × 1520 at 192; one embedded subset font for the wheel figure, the bold and math faces on the heatmap; the size recorded | — |
| (12) | Phase 12's kitchen sink with its sweep (every command, paint and clip), the scatter and trace examples and a heatmap (images, colorbar, bold, math); the wheel figures in rustd3-examples | — |
| (13) | `figure/tests/main/poppler.rs`, compiled into rustd3-examples' tests by `#[path]` (as figure's tests compile the examples' figures); the tools read the PDF on stdin | A testkit module: testkit has no PNG decoder and its manifest would change |
| (14) | The pixels `save_png` encodes, rendered in the test through the same target and settings | `save_png` under `png` only |

### Decision

A for all fourteen (architect, 2026-10-28), as built, including
`pdf::Error::List`, `gif::Error::Raster`, and `gif::Writer` requiring
`fonts` as `save_png` does.

### Why

(1), (2), (8) and (9) keep every failure a typed error rather than a
panic or a string; (3), (4) and (13) follow earlier decisions and the
manifests as they are; (5)–(7) and (10) make PDF draw what the other
backends draw; (11), (12) and (14) are the closest readings of the
section that the crates and the figures allow.

### Followups

- Phase 40: the facade's `gif` and `pdf` features re-export these
  modules; its README can say that `gif::Writer` needs the bundled
  fonts.
