# Decisions logging

Companion to the **Decisions** section in your global instructions file
(`~/.claude/CLAUDE.md` for Claude Code, `~/.codex/AGENTS.md` for Codex).
That file covers the always-on rule + a short example. This file
covers the persisted-entry format for `<project>/.claude/DECISIONS.md`.

## When to write a `D{N}` entry

Any design call made *during implementation* that wasn't already
nailed down in a plan/spec doc. Triggers:

- An agent or orchestrator picked between concrete alternatives.
- You're about to escalate (notify-me, ask the user) because confidence
  dropped — write the entry first, then escalate.
- A reviewer overturns a prior choice.

If the decision was already settled in a `MAP_PLAN.md` revision
history or `MAP_PHASE_N.md` doc, point at that instead. DECISIONS.md
fills the gap between "the plan said" and "what we actually picked
when the plan didn't anticipate this."

## Entry shape

```markdown
## D{N} — {short title}

**Date:** YYYY-MM-DD · **Context:** {phase / branch / commit} · **Status:** decided

### Issue
What broke or needed deciding. Specific symptoms — compile errors,
line numbers, observed behavior.

### Investigation
What was checked. External precedents (other crates, prior art) cited
with file paths + line numbers anchored to a specific commit so they
don't rot.

### Options

| | A — ... | B — ... | **C — ...** |
|---|---|---|---|
| Approach | ... | ... | ... |
| Cost | ... | ... | ... |
| Correctness | ... | ... | ... |
| Architectural fit | ... | ... | ... |

### Decision
Picked C.

### Why
The reasoning, including what's deferred and to whom.

### Followups
Revisit triggers, deferred alternatives, forward-links to entries
that supersede this one.
```

## Conventions

- **Append-only.** Don't rewrite past entries. If a decision is
  overturned, add a new `D{M}` entry that links back, and add a
  forward-link in the original entry's `Followups` section.
- **Don't relitigate.** Reference past decisions by id (`see D3`)
  rather than rerunning the analysis. If it needs revisiting, that's
  a new entry, not an edit.
- **One file per project.** `<project>/.claude/DECISIONS.md` spans
  all maps/phases. Don't fragment into per-map DECISIONS.md files.
- **Not git-tracked** by default — `.claude/` is a global ignore
  pattern in most setups. The log is for the working repo, not for
  shipping.

## Composes with

- The **options-not-flags** rule from your global instructions file — same
  table shape applies in both surfacing (conversation) and persisting
  (DECISIONS.md entries).
- Project-specific feedback memories may add stricter conventions
  (e.g. granita's `feedback_decisions_doc.md` requires investigation
  citations to be commit-anchored). Project rules win over this guide.
