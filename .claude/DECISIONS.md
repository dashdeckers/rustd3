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
