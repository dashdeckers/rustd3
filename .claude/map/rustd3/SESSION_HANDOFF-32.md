# rustd3 — session handoff 32

2026-10-28 · **Phase 32 — The use cases**, complete · M4 started · next: **Phase 33 — Figure handles breadth** (Phase 31 still waits for D22.1's verdict)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 32** (dev/examples, figure tests; no core change):

- e5897c9 `feat(examples): use case 1, focus and context, as a
  core-level iced app` — Step 1.
- 1995eac `test(examples): use case 1 through iced_test's simulator` —
  Step 2, with the integration-test target in `dev/examples/Cargo.toml`
  and the hand check of the brush guard in its body.
- 35a324a `feat(examples): use case 2, the general update pattern, with
  its tests` — Step 3.
- bc93514 `test(figure): the figure locale against d3's de-DE output` —
  Step 4.
- The closing docs commit (`decisions/phase-32.md` with Step 5's line
  counts, `DECISIONS.md`, this handoff).

Exit criteria, each checked this session:

1. `cargo build -p rustd3-examples --examples` builds `focus_context`
   and `update_pattern` (not run: no display).
2. `ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-examples
   --all-features` and the same with `wgpu` (lavapipe): six tests pass
   on each.
3. The `if !brush.is_active()` guard removed by hand:
   `brushing_the_overview_sets_the_window` fails with [0 h, 719 h];
   stated in 1995eac's body.
4. `cargo test -p rustd3-figure --test main -- locale`: the six
   `locale::` tests pass (the time row through core's axis, D32.1).
5. `cargo run -p rustd3-harness -- --check`: unchanged, 16 suites, 6,722
   assertions, 5,292 passing, 1,430 skipped, 0 failing, 179 floors.
6. `decisions/phase-32.md` holds the measured line counts: 239 and 136
   code lines by the section's count (prototypes 202 and 131); Part I's
   247 for use case 1 is the prototype's count with imports, data and
   `main` (ours 292).
7. `scripts/check.sh quick` green on every commit; `full` green at
   bc93514 in 9 min 56 s (warm).

Decisions: D32.1 (open, recommended A: the time-axis locale test goes
through core's axis until Phase 33 makes `time::Time` a figure handle),
D32.2 (open, recommended A: iced's `time::every` needs its `tokio` or
`smol` feature, so use case 2's 1.5 s clock is a thread in the binary;
B would add the feature to `dev/examples`), D32.3 (open, recommended A:
eight details as built). D22.1 still waits for the user's windowed
benchmark run.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; Phase 22's verdict pending the user's run |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | only if D22.1's verdict asks |
| M4 Complete figure | 32–39 | 32 done; next: 33 |
| M5 Release | 40 | pending |

## 5. What to do next

- D32.1–D32.3 answered by the architect.
- Phase 33 (needs 17, 20, 21, 23, 25, 28; two sessions, cut after
  Step 3). With its time handle, rewrite
  `figure/tests/main/locale.rs::time_axes_keep_the_iso_ladder_under_de_de`
  on `fig.scale(time::time(berlin).domain(day))` and a figure axis
  (D32.1).
- If D32.2 is answered B: add iced's `smol` feature to `dev/examples`
  in its own commit and move the clock back into `App::subscription`.

## 6. Notes for later phases

- The use-case tests build `iced_test::Simulator` through
  `tests/main/main.rs::simulator(size, view)`, which skips wgpu without
  a device under `RUSTD3_ALLOW_NO_GPU=1`.
- Screenshots of an app at any factor: `iced_test` re-exports `runtime`
  and `renderer`, so a scratch test can build a `UserInterface` with
  `iced::Renderer::new` and `hint(Scale { window: k, .. })` and call
  `Headless::screenshot`, as rustd3-iced's harness does; nothing of the
  kind is committed for the examples.
- iced at 3cbdacc without `tokio`/`smol` has no `time::every`;
  `window::frames()` only observes redraws.
- Phase 36's figure-level focus and context can reuse use case 1's spot
  rows (205/635 and 260/480 of 860 px).

## 7. Key files

`dev/examples/src/{focus_context,update_pattern}.rs`,
`dev/examples/examples/{focus_context,update_pattern}.rs`,
`dev/examples/tests/main/{main,focus_context,update_pattern}.rs`,
`figure/tests/main/locale.rs`, `.claude/decisions/phase-32.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-examples --all-features
ICED_TEST_BACKEND=wgpu cargo test -p rustd3-examples --all-features
cargo test -p rustd3-figure --test main -- locale
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  bc93514.
- D32.1–D32.3 open with recommendations; D22.1 waiting for the user.
- Conventional commits, no attribution.
