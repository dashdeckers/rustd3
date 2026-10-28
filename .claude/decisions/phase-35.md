# Phase 35 — Themes, colorbars and hatching

## Measurements, session 35a

2026-10-28, session 35a (D34.5's fix and steps 1–2, e19632d … 7b40c1b;
the cut after step 2). `scripts/check.sh full` green at 7b40c1b in
12 min 12 s (warm, the benchmark smoke included); its first run, at
a0e4b5d, passed every test and stopped at `cargo doc -D warnings` on a
redundant link target in `theme.rs` (fixed in 7b40c1b with a test
comment).

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors.

**Lines** (be48aae..a0e4b5d): figure library +1,398 / −491 in 23 files
(git counts the 361 lines of `theme.rs` that moved into
`theme/settings.rs` as removed and added), figure tests +660 / −102 in
8 files (snapshots included), the wheel example +58 / −5.

**D34.5** (e19632d). On the wheel figure, FR's and RR's titles move up
3.62 px onto FL's and RL's baselines and FL's and RL's down 0.15 px (ink
to metric anchoring); each row's baseline is 8 px plus DejaVu Sans's
descender at 16 px (3.844 px) above its scatters, asserted. The layout
iteration moves the plot areas by at most 0.0004 px. On the grid of
scales, panel B's title had sat 15.8 px above panel A's (B's right axis
carries "+1e9"); both now share their row's baseline, raised by the
month axes on top.

**Spot checks** (steps 1–2), each asserted: `Theme::default()` equals
`seaborn(Ticks, Notebook).palette(COLORBLIND).domains(Margin(0.05))`;
talk tick labels 22 px (unit test and drawn runs); paper line width
1.2 pt and major tick 4.8 pt; poster text 24 pt and axis line 2.5 pt;
`font_scale(1.1)` tick labels 12.1 pt (drawn runs 16.133 px), axis lines
unchanged; the three rejected inputs equal `Theme::default()`; every size
of the table at the four contexts; `Domains::Nice` with `.ticks(5)` over
[0.3, 9.7] gives [0, 10] (d3 in Node: `nice(5)` and `nice(12)` both
[0, 10], [0, 7.35] → [0, 8]), a handle without an axis rounds with 10, bars
keep 0; a UTC handle over 09:17–16:41 with `.ticks(5)` floors and ceils
to the hour, [09:00, 17:00] (d3-scale's `scaleUtc().nice(5)` in Node).
The palettes equal seaborn 0.13.2's `color_palette(name).as_hex()`.

**CAM02-UCS distances** (the section's last spot row, Part I's Themes
table). colorspacious is not installed; a numpy reimplementation of its
pipeline (sRGB to linear, Machado 2009 at severity 1, sRGB primaries to
XYZ100, CIECAM02 at D65, Y_b 20, L_A 64/π/5, average surround, CAM02-UCS)
reproduces every cell of the table, deep 5.6 and colorblind 18.6 among
them, when the simulated linear RGB is clipped to [0, 1]; without the
clip the protanopia cells of colorblind, Category10, Observable10 and
Okabe-Ito differ (21.3, 3.4, 23.7, 23.8 for the first three colors).
Scratch script, not committed.

**Wheel figure presets** at 960 × 760 (vector, `resolve`), the same for
all five styles (styles change no size):

| Context | Plot-area share | Passes | `fig.check` |
| --- | --- | --- | --- |
| paper | 62.18% | 3 | empty |
| notebook | 54.68% | 3 | empty |
| talk | 37.44% | 3 | 6 overflows |
| poster | 24.19% | 3 | 6 overflows |

Part I's sketch measured 58.6%, 50.7% and 34.7% with estimated 10-px
text; typst's measured text and D16.4's line boxes give each context
about 3.5 points more. Ticks at notebook with `Domains::Nice`: 51.41%.
At talk the overflows are Part I's: "vibration RMS [m/s²]" needs 246.84
of 162.14 px (Part I: 247 of 151), "bearing temp [°C]" 217.22 of 162.14
and "count" 67.71 of 48.64 (Part I: 68 of 45), on the two outer columns.
`wheels_fit_at_talk` (1600 × 1200) is empty.

**Viewed** (PNGs at 2×, scratch, not committed): a one-panel line and
scatter at 640 × 480 under the five styles beside seaborn's own
rendering of the same data; the wheel figure under the five styles at
the four contexts (paper and notebook at 960 × 760, talk and poster at
1600 × 1200) beside seaborn's rendering of a comparable 4 × 2 grid of
the same samples at whitegrid, darkgrid and ticks (notebook) and ticks
(talk, paper); the wheel example's `--theme` output for five presets; a
scatter with dashed and solid lines and four legends of the three
glyphs in insets and slots. Grid colors and widths, the dark styles'
missing axis lines, tick marks only in ticks, label distances, and text
and line sizes at talk read as seaborn's; seaborn's bars have white
1-pt edges (D35.1) and its axes all four spines.

## D35.1 — Bars without edges under the grid styles

**Date:** 2026-10-28 · **Context:** Phase 35, step 2, a0e4b5d · **Status:** decided

### Decision

The section's answer: seaborn's styles set `patch.edgecolor = "w"` with
`patch.force_edgecolor`, so its bars have white 1-pt edges; rustd3's
bars have no edges in any style by default, and no `Theme` builder is
added until one is asked for. Viewed on the wheel figure's dodged
histograms under whitegrid and darkgrid: the bars read as distinct
without edges, as Observable Plot's do.

The section's second listed decision (the colorbar inset length) belongs
to step 3.

## D35.2 — Details of Phase 35a as built

**Date:** 2026-10-28 · **Context:** Phase 35, D34.5 and steps 1–2, e19632d … a0e4b5d · **Status:** open

### Issue

1. **Theme's paths.** The section writes `theme::Style`,
   `theme::Context`, `theme::Domains` and `theme::palette`, and keeps
   `Theme` where Phase 14 put it (D14.1 (3): the crate root, as iced and
   the tests import it). A public `theme` module that defines `Theme`
   gives it two public paths, against CLAUDE.md's one-path rule.
2. **Getters.** The section's sketch lists "getters used by layout and
   marks: tick_text(), label_text(), …" inside `impl Theme`, and a spot
   row reads `Theme::seaborn(Ticks, Talk).tick_text()`; D14.1 (10) kept
   getters crate-private, and the tests that compare every setting are
   to sit in `figure/tests/main/theme.rs`, which sees public items only.
3. **Pads.** "Every size is multiplied by the context factor", and the
   prototype scales the tick pad, label pad and title pad; seaborn's
   `plotting_context` does not set `xtick.major.pad`, `axes.labelpad` or
   `axes.titlepad`, so seaborn's pads stay 3.5, 4 and 6 pt at every
   context.
4. **Palette and defaults of a preset.** `Theme::seaborn(style,
   context)` names no palette or domains (seaborn's `set_theme` default
   palette is deep); the section's builder validation names only the
   empty palette.
5. **What styles draw.** The table's "none (white)" axis line of the
   dark styles; whether hidden tick marks keep their room; where grid
   lines come from on panels with twin axes or none, over which ticks,
   in which part and draw order, clipped or not; tinted axes under each
   style; seaborn's four spines.
6. **`Domains::Nice` details.** Which axis's count a handle shared by
   several axes takes; handles no axis shows; `Fit::Window` and
   `Fit::Visible` handles; how "sticky values still hold" applies; held
   plot areas.
7. **`Kind::niced`'s default** as the section writes it leaves `count`
   unused, which warns.
8. **Legend glyphs.** The dot's size, the line's length and caps, an
   invalid dash.
9. **D34.5's room.** "Reserved room stays as now" leaves a title on a
   panel with nothing on top (no top axis, no overhang) short of its new
   baseline by the descender: it would reach 3.84 px into the padding.
10. **The wheel example's `--theme`**: its files and what it reports.
11. **Commits**: four code commits for D34.5 and the two steps, with
    tests in each.

### Investigation

- (3) `seaborn/rcmod.py` 0.13.2: `_context_keys` lists 20 keys, no pad.
- (5) matplotlib 3.11.2 under each `sns.set_theme(style=…)` (scratch):
  the x tick labels' top sits 13.194 px (100 dpi) below the axes in every
  style, `tick1On` false outside ticks; spines `.15`, `.8` or white,
  1.25 pt; grid `.8` or white, 1 pt.
- (9) `figure/tests/main/layout.rs`, the one-panel title row: as built
  the plot area starts at PAD + 8 + 3.84375 + the ink ascent.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `pub mod theme` (`Style`, `Context` with `scale()`, `Domains`, `pub mod palette`); `Theme` defined in the crate-visible `theme/settings.rs` and re-exported at the root only: `rustd3_figure::Theme`, one path |
| (2) | Getters stay crate-private (`get_domains` where the builder takes the name, D18.1); `theme.rs`'s unit tests compare every setting at the four contexts and the spot rows; `tests/main/theme.rs` holds the builder example, defaults, validation, nice domains, and reads sizes, widths and colors from drawn lists |
| (3) | Pads scale with the context, as the section and prototype say (at talk 5.25 pt where seaborn keeps 3.5) |
| (4) | Every preset starts with the colorblind palette and `Margin(0.05)` (the section's default; its example sets `palette::DEEP` explicitly); `.palette` also rejects a palette with a non-finite color, as `.ink` and `.background` reject such colors |
| (5) | Dark styles draw no axis lines (seaborn's white spines are invisible on its white figure); hidden tick marks keep their room, as matplotlib's do; grid lines at each panel's first shown horizontal and first shown vertical axis's ticks (matplotlib's one x and one y grid per axes; twins add none), blanked log ticks excluded, unclipped, one stroke in part role `Grid` (9) after the panel background and before the marks (`axes.axisbelow`); a tinted axis's line is drawn only where the style draws axis lines; no top or right spines (`.border` frames a panel) |
| (6) | The count layout's first pass gives the first axis that shows the handle in panel and insertion order, before the passes iterate; d3's 10 without an axis; only `Fit::All` handles round, windows and `Fit::Visible` handles fit with no margin; sticky values at or beyond the extent's ends stop the rounded ends (exact comparisons); with held areas the counts come from them |
| (7) | `let _ = count;` in the default body |
| (8) | Dots the theme's marker diameter (6 pt), at most the 0.7-em box; lines 2 em (`legend.handlelength`), butt caps, the theme's line width; a dash that `mark::dashes` rejects draws solid |
| (9) | The room is the larger of as built and what the new baseline needs (title pad, descender, ink ascent above the row's top-side axes); the wheel figure keeps its room (0.0004 px through the iteration) |
| (10) | `--theme <style>:<context>` suffixes the files `_<style>_<context>` and prints `fig.check`'s issues; a malformed preset is an error naming the expected form |
| (11) | e19632d D34.5; 6e9de08 step 1; a0e4b5d step 2; 7b40c1b a doc link `cargo doc` rejected and a test comment |

### Decision

Recommended: A for all eleven, as built.

### Why

(1) keeps D14.1's root path and the section's `theme::` paths under the
one-path rule (D25.3 (3) moved `Bin` to a private file for the same
reason); (2) adds no public API the section does not name; (3), (4) and
(8) follow the section where it speaks and matplotlib where it points to
it; (5) and (6) are the readings closest to matplotlib, seaborn and d3;
(7) is the compiler's; (9) keeps the architect's "room stays" for every
figure where it suffices and never lets a title leave its room.

### Followups

- Step 3: colorbars outline the bar with the axis line; under the dark
  styles the theme has no edge color (`Theme::edge` is `None`), so the
  bar needs a rule (none, or the ink).
- Phase 39's `Axis::minor` draws at `minor_tick`/`minor_width`, which
  now follow the context.
