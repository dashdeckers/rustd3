# rustd3 — session handoff 35

2026-10-28 · **Phase 34 — Figure marks breadth**, session 34b: **done** (D34.3's hit change and steps 5–7) · next: **Phase 35 — Themes, colorbars and hatching** (after D34.4 is answered)

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- 41c2370 `feat(figure)` — D34.3's change: bars (`bar_y`, `bar_x`, span
  forms) and histogram bars report one hit candidate each at the centre
  of the bar's value end, inside the plot area; areas, ribbons,
  densities, rects (and images) none. Tests in `tests/main/hit.rs`.
- e418bbd `feat(figure)` — Step 5: `Dots::fill_by`, `Dots::size_by`
  (`mark/channel.rs`, `mark/dots.rs`: a paint table and per-point radii in
  one `Markers` command, `Error::ChannelLength` kept and checked),
  `image` and `mark::Grid` (`mark/image.rs`, `Error::GridLength`), `note`
  (`mark/text.rs`, crate-private `Theme::note`). Tests in
  `tests/main/marks/{channel,image,note}.rs`.
- bfcdfda `feat(testkit)` — `fixture::Fixture`: JSON fixtures read by
  pointer into `Obs` (examples has no serde_json).
- 49967c9 `feat(examples)` — Step 6: `rustd3_examples::wheels`
  (`figure`, `figure_with`, `simulate`, `SIZE`, `PALETTE`),
  `examples/wheels.rs`, `tests/main/wheels.rs`, snapshots
  `tests/snapshots/wheels.{txt,svg}`, `dev/fixtures/wheels.{py,json}`.
- a56a95c `fix(examples)` — the wheel figure returns `Figure<'a>`
  (`Figure` is invariant; the bench needs a borrowed lifetime).
- ee179bf `feat(bench)` — Step 7: the `wheels` workload at 960 × 760
  (`Cell::window`), in `smoke` and `full`; `label-churn` on a UTC time
  axis around midnight; `plan::REGISTERED` keeps D22.1's `default` plan.
- The closing docs commit: `decisions/phase-34.md` (34b measurements,
  D34.4), `decisions/phase-22.md` (the label-churn rerun), this handoff.

Exit criteria, each checked this session:

1. `cargo test -p rustd3-figure --test main -- marks`: 71 tests (steps
   1–5 and the KDE fixture test); `cargo test -p rustd3-core --test main
   -- series` passes the joint test (34a, unchanged).
2. `cargo run -p rustd3-examples --example wheels` writes
   `target/wheels/{wheels,wheels_inset}.{svg,png}`; `cargo test -p
   rustd3-examples -- wheels`: 6 tests (counts and domains against
   `wheels.json`, `fig.check` empty at 960 × 760 and 1600 × 1200 with
   and without the inset, labels, lowering, snapshots).
3. `cargo test -p rustd3-figure --doc` runs the histogram E0277 doctest
   and its twin (34a, unchanged).
4. `cargo run -p rustd3-bench -- --headless --workload wheels` and
   `--workload label-churn` run and print their reports (debug, wgpu on
   lavapipe), as do the release runs on tiny-skia the numbers come from.
5. `decisions/phase-22.md` has the label-churn rerun appended; earlier
   entries untouched; no closing line (the verdict has not arrived).
6. `scripts/check.sh quick` green on every commit; `full` green at
   ee179bf in 11 min 10 s (warm; the smoke plan now 54 runs).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 0 failing, 179 floors.

Decisions: D34.4 open (ten details as built, recommended A): the
`size_by` bound (`Position<Domain = f64>`), channel, image and note
details, the wheel function's four forced departures from Figures' text
(`Source::AccelX | Source::AccelZ => vib`, `Figure<'a>`,
`#[rustfmt::skip]`, `trend` without unwrap), testkit's fixture reader,
the bench's window and plan. D22.1 still waits for the user.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md` (phase-34.md was already listed).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; Phase 22's verdict pending the user's run |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | only if D22.1's verdict asks |
| M4 Complete figure | 32–39 | 32, 33, 34 done; next 35 |
| M5 Release | 40 | pending |

## 5. What to do next

- D34.4 answered by the architect.
- Phase 35 (themes, colorbars, hatching): colorbars read `fill_by`'s and
  `image`'s color handles; the talk-context check of the wheel figure is
  Phase 35's (`rustd3_examples::wheels::figure` is ready for it).
- When the user's windowed run arrives: the verdict in D22.1, then Phase
  31's part (built and rerun, or recorded as skipped) writes the closing
  line.

## 6. Notes for later phases

- `Dots::placed` yields `Placed { index, at, fill, r }`: one place for
  draw and hits; `fill_by`/`size_by` resolve once per draw
  (`mark/channel.rs`, `Channel<T>` over a type-erased handle).
- Images crop to the cells reaching into the guard band and snap like
  rects; `Grid` has no accessors yet (none were needed).
- Notes are shaped in the figure's batch (`fits.label`) like rule labels;
  `Theme::note` is 12 pt.
- `rustd3_examples::wheels::SIZE` is 960 × 760; the bench's
  `workload::wheels::SIZE` converts it for iced.
- testkit's `fixture::Fixture::get("/a/b")` reads fixtures without
  serde_json in the reading crate.
- Scratch renders for this session are in the session's scratchpad only;
  temporary examples and tests were never committed.

## 7. Key files

`figure/src/mark/{channel,image,text,dots,bar,histogram,error}.rs`,
`figure/src/hit.rs`, `figure/tests/main/hit.rs`,
`figure/tests/main/marks/{channel,image,note}.rs`,
`dev/testkit/src/fixture.rs`, `dev/examples/src/wheels.rs`,
`dev/examples/examples/wheels.rs`, `dev/examples/tests/main/wheels.rs`,
`dev/examples/tests/snapshots/wheels.{txt,svg}`,
`dev/fixtures/wheels.{py,json}`,
`dev/bench/src/workload/{wheels,label_churn,trace}.rs`,
`dev/bench/src/{workload,plan}.rs`, `.claude/decisions/phase-34.md`,
`.claude/decisions/phase-22.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-figure --test main -- marks
cargo test -p rustd3-examples -- wheels
cargo run -p rustd3-examples --example wheels    # target/wheels/
python3 dev/fixtures/wheels.py                   # only when a phase says so
cargo run --release -p rustd3-bench -- --headless --workload wheels
cargo run -p rustd3-harness [-- --check]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at ee179bf.
- D34.4 open with its recommendation; D22.1 waiting for the user.
- Conventional commits, no attribution.
