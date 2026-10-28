# Phase 33 — Figure handles breadth

## Measurements

2026-10-28, 2558a96 … b9242d2 and the closing docs commits, one session
(no cut after Step 3). `scripts/check.sh full` green at d0583ed in 10 min
51 s (warm, the benchmark smoke included), at 436ad32 (after a test fix,
9 min 57 s) and at b9242d2 (two doc comments, 10 min 6 s).

**Harness** unchanged (exit criterion 6; the flat-time decision changed
core's `Pad for Time`, which no d3 test reaches): 16 suites, 6,722
assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.

**Lines** (78b2f2b..d0583ed): library code +1,754 / −165 in 33 files
(figure, core, iced), tests +925 / −16 in 9 files; the section estimated
1,300–1,800.

**Spot checks.** Every row is asserted. Band "a" at 9.677 (d3's step
300/3.1), bandwidth 87.10, "z" drawing nothing and one `OutsideDomain`
issue; the twenty bins keep 5, 7, 10 at 320, 480, 960 px unrotated
(Phase 16's test, unchanged). Time: [22:00Z, 23:00Z next day] fits
[20:45Z, 00:15Z] exactly, t and t + 1 µs pad by 50 ns, no data gives
1970-01-01 … 1970-01-02, the Berlin day on an 800-px axis labels 10-25
03:00 … 21:00 10-26 with `Hour(3)` (also under DE_DE, D32.1's rewrite).
Color: the sequential, fitted sequential, diverging, quantize and threshold
rows, each printed by the vendored d3-scale 4.0.2 and d3-scale-chromatic in
Node through the resolve hook this session. Rotation: 60 × 15 px needs 60 /
30 / 21.21 / 15 px and reaches 15 / 42.99 / 53.03 / 60 px at 0 / 30 / 45 /
90°; +30° ends bottom labels at their ticks, rising left to right. The four
`compile_fail` rows each compiled once in a scratch test: E0271 "type
mismatch resolving `<Continuous<Identity> as Kind>::Domain == Timestamp`",
E0631 "type mismatch in function arguments", E0277 "`Quantile<Rgb>:
Fittable` is not satisfied", E0308, E0277 "`Ordinal<Source, Rgb>:
Position`", E0277 "`Bin` doesn't implement `Display`".

**Tests.** `cargo test -p rustd3-figure --test main -- handles` runs 19
(exit criterion 1: the 15 of handles.rs and four older `figure::` tests
the filter matches), `-- rotate` 6 (rotate.rs's 5 and Phase 14's rotated
left title), `layout::obb` 4 unit tests; the time wheel test passes on
tiny-skia and wgpu (lavapipe).

**Viewed** (vello_cpu PNGs at 1× and 2×, scratch, not committed): time x
axes over a day (Berlin, UTC), a month (UTC, Kathmandu), a decade (UTC,
Berlin) and one instant; bottom, top, left and right labels at 30°, 90° and
−30° on a month's time axis and a twenty-bin band axis; point and band axes;
two 8 × 6 heatmap panels drawn by a user mark through fitted sequential
(viridis) and diverging (RdBu) handles; use case 1's window at 924 px
(D32.3) with iced's headless snapshot. No labels overlap; `fig.check` is
empty on every one.

**"Decisions this phase may meet"**: the first two are met and recorded as
the section's answers (D33.1, D33.2). The third is not met: band and
threshold `fitted` already return the scale as built (D14.1 (9)), so
`fig.fit(band)` behaves as `fig.scale(band)` with no new method on `Kind`.
D23.1's `PartialEq` for the color scales is not needed: the table holds
trait objects and compares nothing.

## D33.1 — A flat time domain widens by an hour

**Date:** 2026-10-28 · **Context:** Phase 33, step 2, 3fbdb3b · **Status:** decided

### Decision

The section's answer: `Pad for Time` widens a single instant by an hour per
side instead of matplotlib's two years, so a one-sample axis reads in hours
and minutes (noon with 5% margins fits [10:54, 13:06]); the figure keeps
delegating its padding to core.

## D33.2 — Diverging data on one side of the middle

**Date:** 2026-10-28 · **Context:** Phase 33, step 3, 09f236f · **Status:** decided

### Decision

The section's answer: the middle stop is kept and never crossed, the
domain `[min(lo, mid), mid, max(hi, mid)]` ([2, 5] over 0 fits [0, 0, 5]),
reversed for a descending domain, so the ramp never reverses.

## D33.3 — Details of Phase 33 as built

**Date:** 2026-10-28 · **Context:** Phase 33, 2558a96 … d0583ed · **Status:** decided

### Issue

1. **The out-of-domain report.** "`fig.check` reports them once per
   handle with a count" names no issue: `Issue::UnknownKey { panel, mark }`
   is Phase 17's palette row.
2. **Color fitting through core.** "Fitting pads through core's
   `scale::Pad`" (Part I) and "a flat extent widens by Phase 14's
   `nonsingular`": core had no `Pad` for the sequential, diverging and
   quantize scales, and no `admits` on the first two.
3. **Color handles with no data.** Core's `Pad` gives [0, 1]; matplotlib's
   `Normalize.autoscale_None` leaves an unset norm alone (a diverging
   [−1, 0, 1] would fit [0, 0, 1]).
4. **`Paint::paint` at NaN.** d3 returns the unknown value or undefined;
   core's viridis family returns entry 0 there (D23.1).
5. **Which continuous color scales.** The section names `Linear<Rgb>`.
6. **Time windows' columns.** "Quantized to the device-column grid in i128
   ns": a span rarely divides into whole nanoseconds per column.
7. **Time label parity.** "The number of whole tick intervals …
   (`interval.count(interval.floor(epoch), tick)`)": `Interval::count`
   ignores the step (D27.2 (6)).
8. **`Fit::Visible` on a time handle**: D17.1 (11) fitted it without
   memory; handoff 18 asked for memory in nanoseconds.
9. **The same-axis rotated rule** uses "max w and h over the pair" of
   "adjacent kept labels"; rotated labels anchored at their ends are not
   centred on their ticks, so labels of different widths are not
   congruent boxes `d` apart.
10. **Rotated anchors.** D16.4 keeps D14.1's anchors for rotated labels,
    which align the ink box, not its turned bounding rectangle; "left and
    right axes rotate about the label's center".
11. **`Resolved::label_boxes`** returns `Rect`s; rotated labels' boxes are
    turned.
12. **The bridge test's twin** "shifted to epoch seconds": an `f64` of
    2026 epoch seconds resolves about 240 ns, so column edges within that
    of a sample thin differently.
13. **Commits**: ten code commits and the docs, not five.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `Issue::OutsideDomain { handle, count }` (`Issue` is `#[non_exhaustive]`): dots, lines and rules count the data whose x or y `Position::position` is `None` (only band and point scales return it), recording channels not read; `fig.check` sums one handle's counts into its first issue |
| (2) | Additive core items (a10914f): `Sequential::admits`, `Diverging::admits`, `Pad for Sequential<R, T>`, `Pad for Diverging<R, T>` (the end stops), `Pad for Quantize<R>`, through one crate-private `pad::ends` |
| (3) | No data keeps a color or quantize handle's domain as built |
| (4) | `None` for values the scale does not admit (NaN, infinities, what its transform does not take), as matplotlib masks them; `.unknown(c)` acts only on `apply` |
| (5) | `Continuous<T, Rgb, N>`: linear, piecewise and transformed color ranges, inner stops keeping their share |
| (6) | Columns of `span / w` rounded to whole nanoseconds, from the epoch: the window spans exactly `w` columns, within `w / 2` ns of `span`, and its edges repeat exactly; a span under half a nanosecond per column fits as `All` |
| (7) | Unit counts from the floored epoch, the step ignored, as written: `Hour(3)` ticks index by hours, so a stride of 2 keeps 00, 06, 12, 18 |
| (8) | `visible::Domain { Number([f64; 2]), Time([i128; 2]) }`; instants step by whole eighths rounded down to nanoseconds |
| (9) | Both passes test the turned boxes themselves with the four-axis SAT (`layout::Obb`), all pairs as D16.1 (2): exact for labels of any widths, the section's predicate for congruent ones (the spot rows) |
| (10) | A rotated label's box is its ink box turned about its first baseline, aligned by its bounding rectangle as matplotlib's default `rotation_mode` aligns: bottom axes top at the tick end plus pad, right edge at the tick for positive angles and left for negative; top axes mirrored; left and right axes ending or starting a gap from the axis, centred on the tick. Unrotated labels keep D16.4 |
| (11) | The bounding rectangles of rotated labels' turned boxes (documented); the turned boxes stay crate-private |
| (12) | Samples 37 µs off every column edge; the twin's x is seconds from a whole second, so both thin the same samples |
| (13) | 2558a96 D32.3; 2e299e5 step 1; 3fbdb3b D33.1; 71f090e and d21517b step 2; a10914f and 09f236f step 3; f9207d3 step 4; ba2b67e and d0583ed step 5; a test fix (c43a58b: an import only the `fonts` test uses, which warned in `check.sh full`'s featureless build); b9242d2 the docs of `Kind::domain` and `Invert` |

### Decision

A for all thirteen (architect, 2026-10-28), as built. `Paint::paint`
returns `None` for a value the scale does not admit and marks skip that
datum (matplotlib's default "bad" colour is transparent); colour handles
without data keep the domain they were built with; core gains the
additive `Sequential::admits`, `Diverging::admits` and the `Pad` impls.

### Why

(1), (2) and (5) are the smallest public additions that make the section's
rows checkable and keep fitting in core; (3), (4) and (10) follow
matplotlib where the section points to it; (6)–(8) keep every instant in
integer nanoseconds; (9) is exact where the section's predicate is an
approximation and equal to it on its rows; (11)–(13) change no behaviour
the section states.

### Followups

- Phase 34: `fill_by` over the color handles; `Paint::paint`'s `None`
  skips a point.
- Phase 35: colorbars read the color and binned handles' tick sets; the
  quantile label rule is D25.3's.
- Phase 36: brushes accept time handles through `Invert for Time`.
