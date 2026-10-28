# Decisions

The format of rustd3's implementation decisions, and an index of the
files that hold them. The specification is
`.claude/map/rustd3/rustd3.md`; this log fills the gap between what it
says and what a phase had to decide because it did not. This file holds
no entries itself.

## Where entries go

Implementation decisions are written to `.claude/decisions/phase-NN.md`
(NN zero-padded, e.g. `phase-07.md`) as `D{N}.{k}`: N is the phase
number, k counts from 1 within the phase. One file per phase that meets
a decision; a phase that meets none has no file. The files are tracked
(`.gitignore` re-includes `.claude/`). Values a phase is told to record
(build measurements, the benchmark verdict, measured bounds) go to the
same files.

## When

- rustd3.md does not make a design call the phase needs.
- Two of its statements cannot both hold, or it is wrong about a fact.
- A later phase or the user overturns an earlier entry.

Write the entry, then stop and report. Where a phase section lists a
decision it may meet with a recommended answer, record that answer as
an entry and carry on: that is not a stop.

## Entry shape

Each entry starts with the heading `## D{N}.{k} — {short title}`, then:

```markdown
**Date:** YYYY-MM-DD · **Context:** Phase N, step S, commit abc1234 · **Status:** open

### Issue
What needs deciding: the statements of rustd3.md involved (section and
line), symptoms, compiler errors, observed values.

### Investigation
What was checked, with citations anchored to a pinned version or a
commit (`kurbo-0.13.1/src/bezpath.rs:120`, `iced@3cbdacc
renderer/src/lib.rs:48-55`) so they don't rot.

### Options

| | A — … | **B — …** | C — … |
| --- | --- | --- | --- |
| Approach | … | … | … |
| Cost | … | … | … |
| Correctness | … | … | … |
| Architectural fit | … | … | … |

### Decision
The recommendation while the status is `open`; the answer once it is
`decided`.

### Why
The reasoning, including what is deferred and to which phase.

### Followups
Revisit triggers, deferred alternatives, forward links to entries that
supersede this one.
```

**Status** is `open`, with a recommendation, until the user answers;
then `decided`.

## Conventions

- **Append-only.** Past entries are not rewritten. Overturning one is a
  new entry that links back, plus a forward link in the old entry's
  Followups.
- **Don't relitigate.** Refer to earlier entries by id (`see D4.2`)
  instead of re-arguing them; revisiting one is a new entry.
- **The options table** has the same shape when a decision is raised in
  a report and when it is persisted here.

## Index

- `decisions/phase-01.md` — Phase 1: build measurements; D1.1 d3-ease's licence; D1.2 the font licence texts
- `decisions/phase-03.md` — Phase 3: D3.1 two d3-format files under 70 %; D3.2 three statements that do not hold as written (both decided A)
- `decisions/phase-04.md` — Phase 4: D4.1 four statements of Phase 4 that do not hold as written (the log tick cap, the crate-visible readers, a spot check one ulp off Node, transform constructors); D4.2 three d3-scale files under 70 % (both decided A)
- `decisions/phase-05.md` — Phase 5: D5.1 six statements of Phase 5 that do not hold as written (asinh's rounding and its machine-dependent ±1e9 row, logit's linear branch, the thinned-symlog tag, asinh's fallback past the cap, bail-outs the fuzz needs); D5.2 `symlog-test.js` under 70 % (both decided A)
- `decisions/phase-06.md` — Phase 6: D6.1 five statements of Phase 6 that do not hold as written (`group_from(digits)` and `Continuous::forward` without a caller, symlog's decade minors under the one-step extension, extensions not past their end, `Ticks::minor` shadowed by the inherent `minor`; decided A); D6.2 `identity-test.js` under 70 % (decided A, the spec's answer); D6.3 a transform's default tick format must parse (decided A, the spec's answer)
- `decisions/phase-07.md` — Phase 7: harness counts before and after color ranges, translator size; D7.1 `point-test.js` under 70 % (decided A, the spec's answer); D7.2 `.unknown(undefined)` in `ordinal-test.js` (decided, the spec's answer); D7.3 four statements of Phase 7 that do not hold as written (point positions through `center`, `Hsl::from(&str)`, JavaScript's trim, private `BRIGHTER`/`DARKER`; decided A)
- `decisions/phase-08.md` — Phase 8: harness counts and translator size; D8.1 `bundle-test.js` under 70 % (decided A, the spec's answer); D8.2 seven statements of Phase 8 that do not hold as written (`Line::closes` per curve, Catmull–Rom's truthy alpha, `State::area`, the six-method reflecting context, no `null`-item rule, the extra `curve` argument, the assertion count; decided A)
- `decisions/phase-09.md` — Phase 9: measurements (prototype counts all hold; release timings); D9.1 no error variant for an append with the wrong number of channels (decided A, a `Channels` variant); D9.2 seven statements of Phase 9 that do not hold as written or leave a choice open (the redundant `where` clause, "every sample below four per column", ±1 steps in `locate`, crate-private tests, `len_bytes` outside tests, E0309 in a bound-free `mark`, unstated validation and query edges; decided A)
- `decisions/phase-10.md` — Phase 10: harness counts and the pad fixture; D10.1 `Pad` with no data on a logit scale (decided, the spec's answer); D10.2 twelve statements of Phase 10 that do not hold as written or leave a detail open (`get_domain` for `Pad`'s unbounded impl, exit criterion 1's command, V8's `Math.pow`, the sticky upper bound, the empty log row, the transform `Pad` uses, decreasing transforms, orientation for N > 2, the ctrl double click, the wheel anchor, d3's zero-translate shortcut, non-finite extents; decided A)
- `decisions/phase-11.md` — Phase 11: build measurements, placement and `BAND_BYTES` measurements on tiny-skia and wgpu (lavapipe) at factors 1–2, the bias sweep; D11.1 nine statements of Phase 11 that do not hold as written or leave a detail open (the harness's root and `Option`, its extra helpers, publishing and redrawing from a canvas program, one waiting press, Tracker details, the added placement tests, the image clip, the lockfile; decided A); D11.2 core contract review before M2: labels that follow the tick rule, `Continuous::with_range`, minors under thinned majors, the width bound, additive items, one path for `Transform` (decided A); D11.3 details of the D11.2 fixes (the tick gap at the ticks' precision, `func` labels, matplotlib's ten-decade minor cut, the `mapping` module; decided A, B, B, A)
- `decisions/phase-12.md` — Phase 12: agreement measurements (tiny-skia, resvg and `lower` against vello_cpu), vello_cpu's size limit; D12.1 fourteen statements of Phase 12 that do not hold as written or leave a detail open (the 65,408 px render bound, `svg::CAPS` with `lower`, paints in the group's space for markers and runs, image brushes, caps per end, square-cap bounds, bounds of empty commands, number precision, dump details, `Settings::parallel()`, no `Bounds` in `skia::render`, `Error::Length`, tiny-skia's partial sweeps, smaller details; decided A)
- `decisions/phase-13.md` — Phase 13: spot checks, the `<text>` SVG against outlines (0.06 % over 16/255), ink against boxes; D13.1 ten statements of Phase 13 that do not hold as written or leave a detail open (the baseline of labels that start with stacked math, read from a `box`; four commits; the key's fill; `Text::typst`; `;` after a span before markup; no `bundled()` without `fonts`; `Anchor`'s path; strokes as fills; `xml:space` and the math face's family in `<text>`; smaller details; decided A)
- `decisions/phase-14.md` — Phase 14: spot checks, the theme against seaborn 0.13.2's rc values; D14.1 nineteen statements of Phase 14 that do not hold as written or leave a detail open (`Continuous::admits` added to core; five code commits; root paths for the figure model with `cache::Stats`; the handle check in `ScaleRef::of`; `Entry` with fit functions; `Mark<'a>: 'a`; `Indexed` ranges; piecewise stops; categorical `admits`; theme getters; square-capped axis lines; layout details; label anchors; tick offset 0; plain data types; part roles; `stick`; helpers; decided A)
- `decisions/phase-15.md` — Phase 15: the trace spot checks (3,947 and 15,427 points against Performance's 3,949 and 15,429; fidelity with vello_cpu at 1× and 2×), example agreement, timings; D15.1 twelve statements of Phase 15 that do not hold as written or leave a detail open (`Points` readers by `Clone` and `Points::indexed` in core; `Indexed::thin` takes a public `thin::Frame`; the memo key's validity fingerprint; rule labels through `Fits`; lifetimes; commits; the examples compiled into figure tests; `fonts` for `draw`/`save_*`; defaults; curves and the guard band; the column metric's ink; counts; decided A); D15.2 the trace example's SVG and PNG agree within 0.03 (decided A); D15.3 a 1 px line thinned at one column per device px stripes at 2× (decided A)
- `decisions/phase-16.md` — Phase 16: spot checks, pass counts on grids up to 16 × 16 (2 or 3), the renders viewed; D16.1 seven details the section leaves open (`Resolved::plan` and `Resolved::label_boxes`, the absolute index from the domain and count, pass-loop details, cells, the outward rule, the slack's place, commits); D16.2 offset and multiplier notation only for ticks on d3's linear grid; D16.3 the stacked-rows spot row under Phase 14's overhangs (all decided A); D16.4 tick labels share a baseline, blanked ticks at minor size (decided A)
- `decisions/phase-17.md` — Phase 17: D16.4 measured, spot checks, example agreement after snapping, the renders viewed; D17.1 twelve details the section leaves open or that do not hold as written (D16.4's minor size in points and line box, `Legend<K>` without a lifetime, legend layout, hashes over open groups and font identity, cells, the notation label, snapping, hits, `fig.check` and `Points::is_collected`, `Fit::Visible` in data units, eleven commits); D17.2 the example SVG and PNG no longer share geometry (both decided A)
- `decisions/phase-18.md` — Phase 18: the widget against its PNG (1/255 on both backends at 1.25, 1.5, 2, translucent too), the overlay's pixels, one resolve per refresh, the renders viewed; D18.1 no public way to read or set a theme's ink and background (decided A: theme builders and `get_` readers); D18.2 nine details as built (steps 1 and 2 in one commit, `_boxed`, two resolves per refresh, `now` unstored, two paths for `Plot`, `figure` without `fonts`, the 256 × 125 row, opaque test dots, failed bands; decided A, B for the resolve: `Figure::draw_resolved`); D18.3 vello_cpu's output on dense strokes depends on the render's origin (decided A: record, measure before Phase 31)
- `decisions/phase-19.md` — Phase 19: debug-build render times, the pipeline tests on both backends, the renders viewed; D19.1 the section's 200k-dot harness tests take a minute per render in a debug build (decided C: optimized render crates in the dev profile; its measurements, the 200k-dot tests restored, appended after the entries); D19.2 sixteen details as built (commit order, `Probe`'s two paths, re-planning abandoned jobs, results of an older partition, the budget's repeated instants, the theme background of missing slots, `uploaded`, thread counts, Phase 18's tests inline; decided A); D19.3 the cost model in a debug build (decided, the section's answer)
- `decisions/phase-20.md` — Phase 20: the section's tests on both backends, spot rows, the frames viewed; D20.1 `Held::hold_erased` and D20.2 axis strips from the resolved layout and D20.3 previews from the gestures' transforms (decided, the section's answers); D20.4 two readers of `Resolved` the section does not name (`scale_at`, `axis`; decided A); D20.5 thirteen details as built (commits, extra fields, generations, flushes, wheel regions, stale anchors, zoom anchors, releases, double clicks, the fitted domain a reset publishes, keys on both axes, state-level tests, repeated `on_change`; decided A)
- `decisions/phase-21.md` — Phase 21: the section's tests on both backends, spot rows, the frames viewed; D21.1 the window's width and the layout depend on each other (decided A); D21.2 cells an append re-rendered beyond the touched ones (decided A); D21.3 the section's listed decisions (decided); D21.4 ten details as built (decided A)
- `decisions/phase-22.md` — Phase 22: the headless smoke plan on tiny-skia and wgpu (lavapipe), the also-measured items (D19.3, D18.3, D15.3, D21.1); D22.1 the benchmark verdict, the registered rule, the exact command and the three outcomes (open: waiting for the user's windowed run); D22.2 fifteen details as built (decided A)
- `decisions/phase-23.md` — Phase 23: harness counts before and after (3,314 → 3,822 assertions, 2,623 → 3,040 passing, 80 → 97 floors), translator size, the schemes against d3 and as strips; D23.1 the section's listed decisions (decided: `piecewise` over `&[T; N]`, schemes at NaN documented); D23.2 `cubehelix`, `gray` and `sequential` tests under 70 % (decided A); D23.3 thirteen details as built (decided A)
- `decisions/phase-24.md` — Phase 24: the `compile_fail` doctests and their errors; D24.1 the field bounds do not make `Self: Clone` hold (decided A); D24.2 six details as built (decided A)
- `decisions/phase-25.md` — Phase 25: harness counts before and after (3,822 → 4,562 assertions, 3,040 → 3,529 passing, 97 → 116 floors), translator size; D25.1 the section's listed decision (decided: not met, undefined observes as null); D25.2 fifteen files under 70 % (decided A); D25.3 twelve details as built (decided A)
- `decisions/phase-26.md` — Phase 26: harness counts before and after (3,529 → 3,580 passing, 1,033 → 982 skipped, floors 116, two raised), translator size, the `fmt!` errors and the agreement corpus; D26.1 the parser accepts a fill outside the BMP where d3's pattern does not (decided A); D26.2 `defaultLocale-test.js` under 70 % (decided A); D26.3 eight details as built (decided A)
- `decisions/phase-27.md` — Phase 27: 27a measurements (harness 4,562 → 5,698 assertions, 3,580 → 4,676 passing, 116 → 151 floors; the time suite's 35 files, translator size, `local`/`utc` against V8); 27b measurements (5,698 → 5,879 assertions, 4,676 → 4,794 passing, 151 → 153 floors; the `time_scale` suite, the release fuzz, spot rows against d3); D27.1 `tick_interval` takes a zone it does not use (decided B: no zone); D27.2 thirteen details of 27a as built (decided A); D27.3 `utcTime-test.js:283` expects `utcDay` ticks d3 7.9.0 no longer gives (decided B: an `upstream:` skip); D27.4 `time-test.js` and `utcTime-test.js` under 70 % (decided A); D27.5 eight details of 27b as built (decided A)
- `decisions/phase-28.md` — Phase 28: measurements (harness 5,879 → 6,411 assertions, 4,794 → 5,100 passing, 1 → 0 failing after D27.3's `upstream:` skip, 153 → 161 floors; translator, generator, the 56 tick rows against d3); D28.1 a runtime time table has no way into the figure locale (decided B: `locale::Locale::new(number, time)`); D28.2 twelve details as built (decided A)
- `decisions/phase-29.md` — Phase 29: measurements (harness 6,411 → 6,722 assertions, 5,100 → 5,292 passing, 161 → 179 floors; translator +245 / −6 of 250; the spot rows against d3; the renders viewed); D29.1 six d3-shape files under 70 % for api and js skips alone (decided A); D29.2 eleven details as built (decided A)
- `decisions/phase-30.md` — Phase 30: measurements (harness unchanged at 6,722 assertions, 5,292 passing, 179 floors; hand-transcribed brush and drag traces, the join rows from d3-selection in Node, the lifecycle trace); D30.1 thirteen details as built (the release that ends a gesture, presses during a gesture, d3's `empty`, the gesture's locals, unstored modifiers, `Animated`'s start and overflow rules, `Marks`' repeated keys); decided A
- `decisions/phase-32.md` — Phase 32: measurements (harness unchanged at 6,722 assertions, 5,292 passing, 179 floors; code lines 239 and 136 by the section's count against the prototypes' 202 and 131, Part I's 247 reproduced; the six example tests on both backends; the guard check; the frames viewed); D32.1 the time-axis locale test goes through core's axis until Phase 33's time handle (decided A); D32.2 iced's `time::every` needs its tokio or smol feature, so use case 2's data clock is a thread in the binary (decided A); D32.3 eight details as built (decided A)
