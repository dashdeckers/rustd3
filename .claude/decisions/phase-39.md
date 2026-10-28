# Phase 39 — Helpers II: presets, labels and axis defaults

## Measurements, session 39a

2026-10-28, session 39a (D38.1's commit, steps 1–2: 508cb45, 149b8b9,
6510a55, 09f23e7). matplotlib 3.11.2, seaborn 0.13.2.

`scripts/check.sh quick` green on every commit. `full` at 6510a55: green
in 13 min 32 s (warm, the benchmark smoke included) but its featureless
build warned that the labels' locale and line readers are never used;
09f23e7 puts them behind `helpers`, and `full` at 09f23e7: green in 13 min 9 s with no warnings.
The architect's warm-up `full` built and passed the D38.1 edits before
the first commit.

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors; nothing blessed.

**Lines** (b3811e4..6510a55): library +1,062 / −155 in 38 files (D38.1's
renames and moves +143 / −74; steps 1–2 +919 / −81), integration tests
+561 in 3 files.

**D38.1.** One public path per item, read in the generated docs:
`mark/struct.Snap.html` (the private `mark/snap/` page is rustdoc's
redirect), `mark/guard/fn.band.html`, `mark/guard/fn.polygon.html`,
`list/fn.brush.html`, and `get_color`, `get_gray`, `get_line_width`,
`get_patch_width`, `get_density_width`, `get_marker` beside `get_ink` and
`get_background` on `Theme`.

**Step 1** (`helpers::labels` — 9 tests): the bar spot row at 16 px: "3.0"
and "2.5" with their line boxes' bottoms 4 px over their ends, "−1.0" its
top 4 px under its end (baselines to 1e-9, centres to 1e-3 across);
centred labels halfway up; d3's default format ("3", "−1", "2.5") and a
figure locale's (de-DE "2,50"); keys outside the band reported. Pie 0–100
and donut 40–80 label centres on (50, 0), (−25, −43.301270),
(−43.301270, 25) and (60, 0), (−30, −51.961524), (−51.961524, 30), checked
against d3 7.9.0 in Node through the resolve hook; "50%", "17%", "33%" on
a 70 px ring. Sizes over 1 … 100 at count 4 and colors at count 5 give 20,
40, 60, 80, 100 (d3's `ticks(1, 100, 4)` and `ticks(1, 100, 5)`, Node).
matplotlib's `bar_label` measured at 100 dpi: the text box's bottom 4.17
px (3 pt) over the end, its top 4.17 px under a negative end.

**Step 2** (`helpers::axes` — 6 tests): 20 linear minors 0.4 … 9.6 at
5.333 × 1.333 px with six labels; symlog [0, 5000]'s 28 decade minors
2–9, 20–90, 200–900, 2000–5000 (D6.1 (3)) at core's positions to 1e-9;
log [1, 1e6] draws every d3 tick and every core minor; none on time and
band axes or under whitegrid; `defaults` keeps "+1e9".

**Viewed** (vello_cpu PNGs at 2×, scratch, not committed), each beside
matplotlib 3.11.2 / seaborn 0.13.2: bar labels at the end and centred
(same places as `bar_label`); pie and donut labels on their slices; a
scatter with size and color legends in the right slot; axes with minor
ticks on linear, log (5 and 12 decades), symlog and time scales and an
offset axis under ticks and whitegrid, against `minorticks_on()`. Linear
minors match `AutoMinorLocator`'s; neither draws minors over 12 decades
or tick marks under whitegrid. matplotlib subdivides date axes and draws
no symlog minors over [0, 5000]; see D39.2 (9).

## D39.1 — Value labels and scatter legends need what the public API does not offer

**Date:** 2026-10-28 · **Context:** Phase 39, step 1, 149b8b9 · **Status:** decided

### Issue

The section: "Everything except `Axis::minor` is ordinary code over public
APIs"; Labels: "Labels use the figure's `Locale`" and sit at matplotlib's
`bar_label` offsets; Size and color legends: "Both implement
`legend::Guide`, so they go into a slot or an inset like any legend". The
architect's note for the phase: "Everything in `helpers` stays ordinary
code over the public API". As built through Phase 38 and D38.1:

1. **Text.** A custom mark cannot shape or draw text: `Fits::label` and
   `Ctx::label` are crate-private (Phases 15, 17). The one public way is
   `mark::note`, which needs x and y handles (pie labels have none: they
   sit px from the plot area's middle) and anchors by the ink box
   (`text::anchor::Vertical`), where tick labels sit by the face's line
   box (D16.4) and `bar_label` by its text box (measured: its bottom 4.17
   px over the bar's end at 100 dpi). Neither the line box (`text`'s
   fonts) nor the label size (`Theme::label_text`, `Theme::note`) is
   public.
2. **Locale.** Nothing public reads the figure's locale: `Figure::locale`
   is a setter, `Resolved::locale` crate-private, and `Fits` did not hold
   it. A helper that writes numbers cannot follow `fig.locale(…)`.
3. **Guides.** `legend::Guide` is sealed (D35.6 (1): `erase` returns
   `Erased`, unnameable outside the crate; implemented by `Legend` and
   `Colorbar`), and the key layout drew every dot at the theme's marker
   size, at most the 0.7-em box (D35.2 (8)): a size legend's dots of the
   scale's radii (24 px across for a radius of 12) could not be drawn
   through it.

### Investigation

`figure/src/figure/fits.rs`, `mark.rs` (`Ctx::label`),
`legend/guide.rs` and `legend/keys.rs` at b3811e4. No public composition
reaches the section's behaviour: notes give the wrong anchors and no
locale, and no public type implements `Guide`.

### Options

| | **A — publish a text kit, keep `Guide` sealed** | B — as built: in-crate | C — publish both |
| --- | --- | --- | --- |
| Approach | `Fits::label`, `Ctx::label`, `Fits::locale`, `Ctx::locale`, `Ctx::line(size)` (the line box) and a note-style reader on `Theme` become public with doc comments for custom marks; labels become ordinary code; the two legends stay in-crate | `helpers::{labels, legend}` use the crate-private items (`Fits::locale`, `Ctx::locale` and `Ctx::line` added crate-private; `legend::Entry` with an optional dot diameter, rows growing to hold it); the public surface is the section's exactly; `helpers`' docs name the two as the exception | A, plus a public way to implement `Guide` (a trait of entries with sizes, or a public `Entry`) |
| Users' own marks | Can label their geometry as the figure does | Cannot draw text but through `note` | As A, and can write legends |
| Public items | About six | None beyond the section | About ten, and legend layout becomes API (D35.6 kept it private) |
| Cost | A visibility change: the items exist as built | None | A design for open guides |

### Decision

A (architect, 2026-10-28): a small public text kit for custom marks,
documented as such: the label-shaping entry points the figure's own
marks use (`Fits::label` and `Ctx::label`) and the readers `Ctx::locale`
and `Ctx::line`, under their current names, so value labels are
ordinary code a user could write; `legend::Guide` stays sealed and the
legend entries' dot diameter stays crate-internal, since legend kinds
are a closed set. One visibility commit at the start of Phase 39b, with
`helpers::labels` switched to the public paths.

### Why

The section fixes the helpers' public surface and their behaviour; the
built API cannot give both through public items alone. A is D38.1's
answer applied to text, the smallest set that makes the labels ordinary
code; opening `Guide` would make legend layout API, which D35.6 (1) kept
private. B adds no public API the section does not name but leaves the
labels outside "ordinary code".

### Followups

- If A: one commit making the listed items public with doc comments.
- Step 4's pie preset uses `labels::pie` as built either way.

## D39.2 — Details of Phase 39a as built

**Date:** 2026-10-28 · **Context:** Phase 39, D38.1 and steps 1–2, 508cb45 … 6510a55 · **Status:** decided

### Issue

1. **D38.1's paths.** `mark::Snap` lived in `layout/snap.rs`; the decision
   names `mark::Snap`. Its `of`, `scale` and `point` and `guard::clip` are
   not named.
2. **`.format(spec)`**: the argument's type, and the format without it.
3. **Bar labels as drawn**: the box that sits 3 pt from the end, the
   padding's units, which way a label goes, its base, its size and color,
   the plot area's clip, fitting.
4. **Pie labels**: `Pie<K>` with keys only `Display`; the centre; slices
   that span no angle; the total; `.radius(r)` outside the pie.
5. **Legend entries**: values outside the domain, a radius of 0, the size
   dots' color, titles.
6. **Legend layout** for dots larger than the 0.7-em box.
7. **`defaults`' bound** (`S: Ticks`; `Axis` needs `S: Kind`) and the
   notation it "leaves on".
8. **Minor ticks as drawn**: styles without tick marks, grid lines, the
   colorbar's axis.
9. **Time axes and symlog** against matplotlib's `minorticks_on()`.
10. **Commits.**

### Options

| | **A — as built** |
| --- | --- |
| (1) | `layout/snap.rs` moves to `mark/snap.rs`, re-exported by `mark` (one public path); `rect`, `width`, `line` public as D38.1 (A)'s table lists, `of`, `scale`, `point` and `guard::clip` stay crate-private |
| (2) | A core `format::Specifier` (the parsed, validated type: no error path in the mark; `".1f".parse()?`); without it d3's `""` (up to 12 significant digits, trimmed), which writes what matplotlib's `%g` writes for values of six digits or fewer |
| (3) | The line box (the face's ascender above the first baseline, descender below the last, D16.4) as `bar_label`'s text box: its bottom `padding` over a rising bar's end, its top under a falling one's, centred halfway with `.center()`; padding in pt at 4/3 px, unscaled by the context (matplotlib's points, as D38.2 (10)'s caps); rising or falling read on screen against the base 0, placed by `mark::reach` (the guard band's end where 0 is not on the scale, as bars); 12 pt in ink (matplotlib's `font.size` at seaborn's notebook); clipped to the plot area like every mark and not fitted (matplotlib does not fit them either); keys outside a band report `OutsideDomain` |
| (4) | Keys written at build time, `K` kept as a type tag (`PhantomData`) for the section's `Pie<K>`; centred on the plot area's middle; slices with values that are not positive (no angle, d3) get no label; `.percent` divides by the positive total; `.radius(r)` outside [0, outer] leaves the ring |
| (5) | Entries (ticks or `.values`) outside the domain are left out; a radius that is not finite and positive has no entry (a 0 tick of a [0, 100] size scale); size dots in the theme's ink (matplotlib's `legend_elements` draws them black), color dots at the marker size; no titles (the section lists `.count` and `.values` only) |
| (6) | A dot larger than the 0.7-em box gets a square of its diameter centred where the box's centre is; a row or column grows to hold it, a column's swatches share the widest width; every other legend lays out as before |
| (7) | `defaults<'a, S: Kind>(axis)`, the axis's own bound; it calls `.minor(true)` and leaves `.offset` as set (on by default) |
| (8) | Minor ticks are tick marks: drawn only where the theme draws ticks (D35.2 (5)); no grid lines at them (matplotlib's `axes.grid.which = "major"`); colorbars draw none |
| (9) | Time axes draw none: core's `Time` keeps `Ticks::minor`'s empty default (Phase 27, "Not in this phase"), where matplotlib's `AutoMinorLocator` subdivides dates; symlog draws 2–9 per decade (Part I, D6.1 (3)) where matplotlib 3.11.2 draws none over [0, 5000] |
| (10) | 508cb45 D38.1; 149b8b9 step 1; 6510a55 step 2; 09f23e7 the readers only `helpers` uses behind it (a featureless-build warning `full` found); then the docs |

### Decision

A for all ten (architect, 2026-10-28), as built.

### Why

(1) follows the one-path rule and D38.1's table; (2) is OPAQUE's N1 (the
validated type) and d3's default; (3)–(6) follow matplotlib where the
section points to it and the existing layout where it does not; (7) is
what compiles; (8) and (9) follow D35.2, Part I and core as built.

### Followups

- (9): minor ticks for time axes need a rung rule in core (Phase 27's
  followup), not in v0.1.

## Measurements, session 39b

2026-10-28, session 39b (D39.1's commit, steps 3–4: fc3ae57, d8afd5a,
f34b302, 3d8a31f, 94cfd0c). matplotlib 3.11.2, seaborn 0.13.2. The architect's
warm-up `full` rebuilt `target/debug` before the first cargo command.

`scripts/check.sh quick` green on every commit (about 1 min 25 s warm);
`full` at 3d8a31f passed every test and stopped at `cargo doc -D
warnings` on an ambiguous link (`line`, also std's `line!`), fixed in
94cfd0c; `full` at 94cfd0c: green in 12 min 45 s (warm, the benchmark
smoke included), no warnings.

**Harness** unchanged (core gained one additive trait impl no d3 test
reaches): 16 suites, 6,722 assertions, 5,292 passing, 1,430 skipped, 0
failing, 179 floors; nothing blessed.

**Lines** (ff9851c..3d8a31f): figure library +998 / −17 in 13 files
(by commit: D39.1 +79 / −15, presets +922 / −5, doc examples
included), core +21 /
−2, the example +188, integration tests +596 / −3 in 3 files. The
section estimated 1,400–1,800 for the whole phase; 39a and 39b together
are about 2,000 library lines.

**D39.1** (fc3ae57). One public path per item, read in the generated
docs: `label`, `locale` and `line` on `mark/struct.Ctx.html`, `label` on
`struct.Fits.html`; `Fits::locale` stays crate-private (D39.3).

**Exit criteria.** (1) `cargo test -p rustd3-figure --features helpers
--test main -- presets labels axes`: 68 pass (presets 13, the rest 39a's
labels and axes and other modules' label tests the filter matches). (2) `cargo run -p rustd3-examples --example presets` writes 11 SVGs
and 11 PNGs (1280 × 960) and prints no issue; each preset test asserts
`fig.check` empty at 640 × 480. (3) `cargo test -p rustd3-figure
--features helpers,png --doc`: 103 pass (95 merged, 8 standalone), with
examples for `chart`, `line` (Part I's
sketch, saving a 1280 × 960 PNG), `Line::series`, `Line::y_scale`,
`scatter`, `bar`, `Bar::mode`, `area`, `pie`, `Pie::inner`, `heatmap`,
`Heatmap::colors`, `Heatmap::keys`, `Fits::label` and core's `Data for
Points`.

**Spot checks**, each asserted (`figure/tests/main/helpers/presets.rs`):
`shape::stack` of apples [3, 2, 1] and bananas [1, 1, 4] gives bananas
[[3, 4], [2, 3], [1, 5]] and the stacked preset draws exactly two span
marks with those spans; diverging a = [3, −2], b = [−1, 4], c = [2, 1]
gives a [0, 3] [−2, 0], b [−1, 0] [0, 4], c [3, 5] [4, 5], in core and
as drawn; the pie on a 600 × 400 plot area is 2 × 192.667 px across
(200 − em(0.5), em 14.667 px) about its middle; the line sketch draws
exactly its written-out composition (`fit` of `AnyContinuous` linear and
symlog, a stroked `line`, `axes::defaults` on both axes, the title),
has symlog ticks with minor ticks on y, and `save_png(path, 2.0)` writes
1280 × 960. The pie and donut centroid rows and the label, legend and
minor-tick rows are 39a's and still pass.

**Viewed** (vello_cpu PNGs at 2×, scratch, not committed), every preset
under ticks and whitegrid beside matplotlib 3.11.2 / seaborn 0.13.2
drawing the same data: three lines with a legend (`plot`), the symlog
sketch (`set_yscale("symlog", linthresh=1)`), a scatter, grouped bars
(`barplot` with `hue`), stacked and diverging bars (`bar` with
`bottom`), a stacked area (`stackplot`), a pie and a donut (`pie` with
labels inside and a wedge width), a heatmap (`pcolormesh` and a
colorbar) and a keyed one (seaborn's `heatmap` of a labelled frame); and
the presets example's eleven PNGs on one contact sheet. `fig.check` is
empty on every one. Agreement: line and dot placement, band positions
and widths (seaborn's 0.8), stacking and diverging spans, wedge angles,
labels at centroids, cell order (row 0 on top under keys) and colors,
colorbar ranges. Differences: span bars and ribbons hold nothing, so
stacked bars and areas keep a 5 % margin below 0 where matplotlib's
stand on the axis (D38.2 (3)); symlog ticks are d3's linear ticks on
[−50, 50] where matplotlib labels decades; seaborn's bars are
desaturated and edged (D35.1); matplotlib's pie leaves room for outside
labels; colorbars carry more ticks than matplotlib's (D35.6 (7)).

## D39.3 — The text kit and the presets still read crate-private items

**Date:** 2026-10-28 · **Context:** Phase 39b, fc3ae57 and steps 3–4 · **Status:** decided

### Issue

D39.1 (A) publishes `Fits::label`, `Ctx::label`, `Ctx::locale` and
`Ctx::line` "so value labels are ordinary code a user could write";
Part I's Helpers: "Each helper is ordinary code over public APIs"; the
section: "Everything except `Axis::minor` is ordinary code over public
APIs". The decided option's table listed `Fits::locale` and "a
note-style reader on `Theme`" as well; the decision's text and the
dispatch name the four. As built after fc3ae57, the helpers still call:

1. **Value labels**: `Fits::locale` and `Fits::theme` while fitting (a
   label must be reported with the key it is drawn with, so the locale
   and style that `Ctx` now gives at draw time are needed at fit time
   too: `Ctx::locale` alone cannot be matched), the label style
   `Theme::note` and size `Theme::label_text`, and `mark::reach`,
   `mark::within` and `Fits::across` (small helpers whose behaviour the
   labels' docs state).
2. **Presets**: the theme's palette (a new crate-private
   `Theme::get_palette`, behind `helpers`) for the ordinal scale over
   series keys ("colors come from the theme palette"), and `Theme::em`
   for the pie's margin ("outer radius = 0.5 × min(w, h) − em(0.5)").
3. **Scatter legends**: the legend layout (`Guide` stays sealed, D39.1).

### Options

| | A — as built | **B — publish the fit-time twins and the palette** | C — B and the text readers |
| --- | --- | --- | --- |
| Approach | The four of D39.1 public; the rest crate-private, named in `helpers`' and `chart`'s module docs | `Fits::locale` and `Fits::theme` public (twins of `Ctx::locale` and `Ctx::theme`), `Theme::get_palette()` public (D38.1's reader convention) | B, plus `Theme::get_note(align, fill) -> text::Style` and `Theme::get_em()` |
| Users' own marks | Can shape labels in their own style, but not in the figure's locale | Can label in the figure's locale and theme colors; pick their own text size | Can match the value labels and the pie exactly |
| Helpers | Labels and presets copyable after replacing 4–6 calls | Labels after replacing the style; presets after the em | Copyable verbatim but for `reach`/`within`/`across` |
| Public items | 4 (built) | +3 | +5 |

### Decision

B (architect, 2026-10-28): `Fits::locale`, `Fits::theme` and
`Theme::get_palette` become public, documented for custom marks, so the
labels and presets use only public items. One visibility commit at the
start of Phase 31's session.

### Why

The section and Part I ask for ordinary code; D39.1 chose a kit
narrower than its own table, and this session followed the decision's
text rather than widening the public API unasked.

### Followups

- If B or C: one visibility commit, the module docs' list of crate-private
  reads shortened.

## D39.4 — The keys of unnamed series

**Date:** 2026-10-28 · **Context:** Phase 39b, steps 3–4, f34b302, 3d8a31f · **Status:** decided

### Issue

"Colors come from the theme palette through an ordinal scale over the
series keys; a legend goes into `Slot::Right` when there is more than one
series." `chart::line(data)` takes its first series without a key (more
come with `.series(data, key)`), and `chart::area(columns, x)` takes
columns without names; the section names no key for either.

### Options

| | **A — keyed by index (as built)** | B — matplotlib's unlabelled series | C — builders |
| --- | --- | --- | --- |
| Approach | An unnamed series is keyed by its index as text: the line's first "0", the area's columns "0", "1", … (pandas' names for unnamed columns) | Unnamed series take the palette's colors in order but no legend entry; an area has no legend | `Line::key(k)` names the first series, `Area::keys(names)` the columns |
| Legend | Every series has an entry | Only named series; areas none, against the section's legend rule | Every entry named by the user |
| Public API | None added | None added | Two builders the section does not list |

### Decision

A (architect, 2026-10-28), as built: unnamed series are keyed "0", "1",
… as pandas names unnamed columns; `Line::key` and `Area::keys` builders
can come later, additively.

### Why

A keeps the section's legend rule for every preset and adds no API; B
contradicts it for areas; C is a public addition, which the dispatch
leaves to the architect.

### Followups

- If C: the builders replace the index keys where given.

## D39.5 — Heatmap keys

**Date:** 2026-10-28 · **Context:** Phase 39b, step 4, 3d8a31f · **Status:** decided

### Decision

The section's answer: `chart::Heatmap::keys(rows: Vec<String>, cols:
Vec<String>)`, band axes when set and linear axes over the grid extent
otherwise. With keys the rows run top to bottom (row 0 on top, seaborn's
`heatmap` and a table's order) and the grid's extent becomes one unit per
name; without them row 0 sits at the extent's first y, as `mark::Grid`
documents (at the bottom on an upward y).

## D39.6 — Details of Phase 39b as built

**Date:** 2026-10-28 · **Context:** Phase 39b, fc3ae57 … 94cfd0c · **Status:** decided

### Issue

1. **Paths.** The section's modules are `chart.rs` and
   `chart/{line,scatter,bar,area,pie,heatmap}.rs`, with `chart::bar(…)`,
   `chart::Bar<K>` and `chart::bar::Mode`.
2. **Keeping `impl Data`.** `line(data: impl Data<'a, f64, f64>)` with
   `.series(data, key)` must keep each series until `.figure()`, and
   `Points` (what `into_points` gives) was not itself `Data`.
3. **The finishing builders' signatures**: `.save_svg(path) ->
   Result<(), figure::Error>` where `Figure::save_svg` returns lowering's
   warnings; `.figure() -> Figure<'a>` on presets without a lifetime.
4. **What each preset draws** where the section is silent: the title's
   place, minor ticks' default, bar widths, missing values, pie legends
   and axes, wedge edges, colorbar settings, scatter series, the line's
   key type.
5. **Margins** of stacked bars and areas.
6. **Heatmap keys and the grid.** `mark::Grid` has no public readers.
7. **Test helpers** shared between `helpers/axes.rs` and
   `helpers/presets.rs`.
8. **Commits.**

### Options

| | **A — as built** |
| --- | --- |
| (1) | `chart/bar.rs` is public and holds `Mode`; `Bar` and `bar()` live in the private `chart/bar/preset.rs`, re-exported from `chart` (D38.2 (5)'s pattern), so each has one path; the shared builders are a macro in `chart.rs` over a private `chart/setup.rs` (`Setup`: title, labels, size, theme, minor) |
| (2) | Core gains `impl Data<'a, X, Y> for Points<'a, X, Y>` (the identity, d8afd5a), so kept data passes to `mark::line` still borrowed or indexed: slices are not copied (no `LargeOwned`), channels thin |
| (3) | As the section writes: `save_svg` drops the warnings (presets draw nothing lowering changes; `Figure::save_svg` still returns them); presets without a lifetime have `figure<'a>(self) -> Figure<'a>` (D34.4 (7)) |
| (4) | The title is the panel's (matplotlib's `ax.set_title`); minor ticks on by default (`axes::defaults`; band axes draw none); bars take 0.8 of each band step, half the gap outside the ends (`padding_inner(0.2)`, `padding_outer(0.1)`, seaborn's places); a row's missing values are NaN and draw no bar; the pie has no legend (labels inside, one series) and no axes, so `.x_label`, `.y_label` and `.minor` change nothing; wedges without edges (D35.1), colored in data order, labelled through `labels::pie` (reported while fitting, placed at the radii the plot area gives); `.inner` takes a share in [0, 1); heatmaps fit a sequential handle to the values and put `colorbar(handle)` with its defaults right; the scatter is one series without a legend (the section lists no `.series` for it); line keys are `impl Into<String>` and equal keys share a color |
| (5) | Span bars and ribbons hold nothing (the section's marks), so stacked bars and areas keep a 5 % y margin below 0 where matplotlib's stand on it (D38.2 (3)); x keeps matplotlib's 5 % margins |
| (6) | Keys set the image's extent to `[0, m] × [n, 0]` on linear handles fixed to `[0, m]` and `[0, n]`, under band handles of the names (rows reversed so row 0 is on top); the counts are documented as the grid's and not checked |
| (7) | `axis_part`, `Stroke` and `MINOR` in `tests/main/helpers/axes.rs` become `pub` for `presets.rs` |
| (8) | fc3ae57 D39.1; d8afd5a core's `Data for Points`; f34b302 step 3; 3d8a31f step 4 with the example; 94cfd0c a doc link `cargo doc -D warnings` found ambiguous (`line` and std's `line!`); then the docs |

### Decision

A for all eight (architect, 2026-10-28), as built, except the margins
below 0, which D39.7 replaces.

### Why

(1) is the one-path rule as D38.2 (5) applied it; (2) is the smallest
additive change that keeps `impl Data` meaning what it means for marks;
(3) and (5) follow the section's text; (4) and (6) follow matplotlib and
seaborn where the section points to them; (7) and (8) change no
behaviour.

### Followups

- (5): a `rect`, span or ribbon builder that holds a value (D38.2 (3)'s
  followup) would let stacked charts stand on the axis.
- (6): public `Grid` readers (`cols`, `rows`) would let the heatmap
  report mismatched keys through `fig.check`.

## D39.7 — Stacked bars and areas stand on the axis

**Date:** 2026-10-28 · **Context:** the architect's view of the presets contact sheet after Phase 39b (94cfd0c) · **Status:** decided

### Issue

The stacked bar, diverging bar and stacked area presets (and D38.2 (3)'s
stacked histograms) draw through span bars, `rect` and ribbons, which
stick nothing, so a value axis whose data starts at 0 gets the theme's
5 % margin below it and the bars float above the axis. matplotlib's
`bar` puts a sticky edge at its baseline and `fill_between` at its
lower edge, so stacked bars, `stackplot` and `hist(histtype='bar',
stacked=True)` stand on the axis.

### Decision

(architect, 2026-10-28) The span bars (`bar_y_spans`, `bar_x_spans`) and
the ribbon (`ribbon`, `ribbon_series`, and the area forms built on it)
stick each end of a span that is exactly 0 on the value axis, as
matplotlib's sticky edges do; `rect` stays as it is (it has no
baseline). A span that does not touch 0 sticks nothing, so diverging
bars keep both margins. One `fix(figure)` commit at the start of Phase
31's session, with the presets' and helpers' snapshots re-blessed after
reading the diffs and viewing the renders.

### Why

Floating bars read as a rendering error; this is the look every
matplotlib and seaborn user expects.
