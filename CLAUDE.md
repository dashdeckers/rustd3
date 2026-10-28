# rustd3

## What rustd3 is

rustd3 is a Rust plotting library that speaks d3's vocabulary (scales,
ticks, shapes, number and time formats, color, zoom and brush) and adds
the tier-1 pieces only matplotlib has: typst text and math, legends and
colorbars, layout from measured text, statistical charts. One drawing
list renders SVG, PNG, PDF and GIF; rustd3-iced integrates with iced at
the core level (`IcedPath`, pointer input) and through the `Plot` widget.

The specification is `.claude/map/rustd3/rustd3.md`. Part I is the
design. The Implementation part holds the front matter (from "How to
use this part" to "Order of work"), one section per phase
(`### Phase N —`), Later (0.2), Appendix A (the dispatch preamble:
reading order, allowed and forbidden commands, the pre-commit
checklist, forbidden patterns) and Appendix B (resume).
`.claude/map/rustd3/proto/` is evidence and algorithms, never style.

## Layout

```text
.               rustd3           facade: re-exports by feature (src/lib.rs)
core/           rustd3-core      the d3 port; graded by d3's own tests
derive/         rustd3-derive    #[derive(Interpolate)], behind core's `derive`
figure/         rustd3-figure    figure model, layout, typst text, drawing list, outputs
fonts/          rustd3-fonts     DejaVu Sans (2 weights), New Computer Modern Math
iced/           rustd3-iced      IcedPath, pointer::Tracker, Plot (publish = false)
dev/testkit/    rustd3-testkit   Obs, pixel statistics, snapshots, zones (a leaf)
dev/harness/    rustd3-harness   d3's tests translated to Rust, ratchet and floors
dev/examples/   rustd3-examples  trace, scatter, wheels, use cases, presets
dev/bench/      rustd3-bench     the full-frame benchmark (windowed, --headless)
dev/fixtures/   (no package)     reference values: d3 in Node, matplotlib in Python
scripts/        check.sh         quick | full
.claude/        RUST_STYLE.md, OPAQUE.md, DECISIONS.md, decisions/, map/rustd3/
```

## Commands

```bash
scripts/check.sh quick [pkg…]   # every commit: fmt, clippy -D warnings, touched tests
scripts/check.sh full           # end of every phase; long, run in the background
cargo run -p rustd3-harness -- --check | --bless [--bless-floors] [--allow-drop]
node dev/harness/translate/translate.mjs [--check]
node --import ./dev/harness/translate/d3-resolve.mjs script.mjs   # d3 in Node
```

The harness and translator exist from Phase 2. Fixture scripts under
`dev/fixtures/` run only when a phase section says so. Tests run under
`TZ=Asia/Kathmandu`.

## Rules

- Style: `.claude/RUST_STYLE.md`.
- Opaque types: `.claude/OPAQUE.md`, with three exemptions: `PathContext`
  keeps d3's canvas argument lists (N4); field-less transforms are
  type-level strategies (N5); the plain data types with public fields
  keep them (N2).
- Kept composite names: `PathContext`, `SvgPath`, `LineOf`,
  `AnyContinuous`, `ScaleRef`, `TickRule`, `TickFormat`, `GlyphRun`,
  `BrushX`, `BrushY`, `BrushEvent`, `InvalidFormat`, `ParseColorError`,
  `IcedPath`, and no others.
- No ambient clock or zone in library code (clippy `disallowed-methods`),
  except the `Plot` widget timing its own renders for its cost model
  (D31.8), in one function with a scoped `expect` and its reason.
- No mutable global state beyond `OnceLock` statics of parsed fonts, the
  id and eviction counters, and the bounded typst cache.
- `f64` geometry; `f32` only at backend boundaries and in `GlyphRun`/`Glyph`.
- Dependencies exactly as rustd3.md lists them; any other manifest change
  is a decision.
- The iced pin moves only in its own commit, with the iced tests rerun on
  both backends.
- No AI attribution in commits, comments or docs.
- Nothing from Appendix A's forbidden list.
- Private submodules re-exported by their parent: one public path per item.
- `rustd3-harness` is a binary crate whose modules are private, so
  `missing_docs` never fires on its generated code; no `allow(missing_docs)`
  is added there.

## Decisions

A design call rustd3.md does not make, or two of its statements that
cannot both hold, is written as `D{phase}.{k}` in
`.claude/decisions/phase-NN.md` (format in `.claude/DECISIONS.md`), and
the session stops and reports.

## Commits

Conventional, `type(scope): summary`, scope = crate, one logical change
each. Every commit passes `scripts/check.sh quick`; a phase's last commit
passes `scripts/check.sh full`.

## Execution

One agent, phases in rustd3.md's order; Phase 31 only when the benchmark
verdict in `.claude/decisions/phase-22.md` asks for it. Phases after 10
add to core freely; changing or removing a public item built in Phases
1–10 is a decision whose commit also updates every call site in figure,
iced and the dev crates, with `check.sh quick` green. Handoffs are
`SESSION_HANDOFF-NN.md`, numbered sequentially, next to rustd3.md.

## Reading order

1. This file.
2. `.claude/RUST_STYLE.md` and `.claude/OPAQUE.md`.
3. `.claude/DECISIONS.md` and the decision files of the phases the
   current one builds on.
4. rustd3.md's Implementation front matter, the phase's section and the
   Part I sections it names.
5. The `proto/` files the phase cites.
6. The newest `SESSION_HANDOFF-NN.md` if the phase is half done.
