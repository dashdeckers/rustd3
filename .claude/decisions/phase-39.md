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
