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
- `decisions/phase-11.md` — Phase 11: build measurements, placement and `BAND_BYTES` measurements on tiny-skia and wgpu (lavapipe) at factors 1–2, the bias sweep; D11.1 nine statements of Phase 11 that do not hold as written or leave a detail open (the harness's root and `Option`, its extra helpers, publishing and redrawing from a canvas program, one waiting press, Tracker details, the added placement tests, the image clip, the lockfile; decided A); D11.2 core contract review before M2: labels that follow the tick rule, `Continuous::with_range`, minors under thinned majors, the width bound, additive items, one path for `Transform` (decided A); D11.3 details of the D11.2 fixes left open (float noise in the tick gap, built; `func` labels and matplotlib's ten-decade minor cut, not built; the trait's private module; open)
