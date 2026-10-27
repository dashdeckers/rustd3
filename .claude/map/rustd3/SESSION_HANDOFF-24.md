# rustd3 — session handoff 24

2026-10-27 · **Phase 22 — Full-frame benchmark** complete; its verdict (D22.1) waits for the user's windowed run · next: **Phase 23 — Color spaces, interpolation and color scales**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**Phase 22** (`dev/bench`, a binary crate now; `lib.rs` removed):

- 477c550 `feat(bench): replayed input through the Plot widget, windowed
  and headless` — steps 1–3: `cli.rs` (hand-parsed; `--native-factor`
  added), `plan.rs`, `scenario.rs`, `replay.rs`, `workload.rs` +
  `workload/{trace,scatter,grid,label_churn}.rs`, `driver.rs`, `app.rs`,
  `headless.rs`, `record.rs`, `system.rs`, `recorder.rs`, `main.rs`.
- f8b6268 `feat(bench): statistics, the registered rule and the report`
  — step 4: `stats.rs`, `metrics.rs`, `rule.rs`, `report.rs`, the
  `report` command.
- 2c76768 `feat(bench): also measure what earlier phases left to the
  benchmark` — `also.rs`: D19.3, D18.3, D15.3, D21.1.
- ee94050 `fix(bench): attribute misses per offending window`.
- e533558 `build: the benchmark's headless smoke plan in check.sh full` —
  step 5, and `dev/bench/README.md` (the user's run sheet).
- The closing docs commit (`decisions/phase-22.md`, this handoff).

Exit criteria: (1) `cargo build --release -p rustd3-bench` succeeds;
`cargo test -p rustd3-bench` passes 25 tests. (2) `target/release/
rustd3-bench run --headless --plan smoke --backend tiny-skia --out
target/bench-smoke-tiny-skia`: 257 s, 46 runs + 4 n/a, every Outputs
file, a row per cell, no empty cell, no NaN, rule "report-only:
headless"; `--backend wgpu` (lavapipe) 261 s, the same. (3)
`scripts/check.sh full` builds the bench in release and runs the smoke
plan on tiny-skia, printing the budget table and the outcome; it fails
only when a run crashes or writes nothing. Green at e533558 in 7 min
36 s (warm; about 4 min 20 s of it the smoke plan), no warnings. (4) `decisions/phase-22.md`
holds D22.1, pending the user's run, with the exact command, the
readings, the plan's duration and the three outcomes.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions: D22.1 open (the verdict, waiting for the user's run); D22.2
open (fifteen details as built, recommended A).

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
| M3 Core breadth | 23–30 | next: 23 |
| Conditional | 31 | only if D22.1's verdict asks |
| M4–M5 | 32–40 | pending |

## 5. What to do next

- The user: give the display's refresh rate and scale factor, confirm
  the readings in D22.1, run `dev/bench/README.md` (optionally the
  5-minute windowed smoke first: the windowed path has never run, the
  container has no display), send back the run directory.
- When the results arrive: write the verdict into D22.1 from
  `report.md`/`summary.csv` (readings as written; inconclusive cells
  rerun with `--runs 10` first); Phase 31 is next if it says raster/upload
  bound. `rustd3-bench report <dir>` rebuilds the report from raw files.
- Meanwhile Phase 23 (Order of work): read the front matter and
  "### Phase 23 —".

## 6. Notes for later phases

- Phase 31: rerun with `--threads`/`--caching verify` (add them then);
  the headless smoke shows the case it targets: every pan frame's
  plot-area job is superseded before it lands, so nothing is sharp until
  the gesture ends (latency p95 ≈ the gesture). D18.3's dashes: 381 px
  over 2/255, max 237/255, cells against the whole trace at 2×.
- Phase 34: the wheel workload (`Workload` gains a variant;
  `rustd3-bench --headless --workload wheels` is the smoke on it) and the
  label-churn rerun with time formats; `format!("t+{v:.4} s")` churns
  only as ticks enter and on zooms (D22.2 (9)).
- D19.3: the windowed run's calibration (meta.json) may move the cost
  constants; on this container the 200k-dot cell renders at 1.24–1.32 ×
  the model in release.

## 7. Key files

`dev/bench/src/{main,cli,plan,scenario,replay,workload,driver,app,headless,
record,system,stats,metrics,rule,report,also,recorder}.rs`,
`dev/bench/README.md`, `scripts/check.sh`, `.claude/decisions/phase-22.md`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background; includes the smoke)
cargo test -p rustd3-bench
cargo build --release -p rustd3-bench
target/release/rustd3-bench run --headless --plan smoke --backend tiny-skia --out target/bench-smoke
target/release/rustd3-bench report <run dir>     # rebuild CSVs and report.md
target/release/rustd3-bench list                 # readings, plans, cells
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at the end
  of the phase.
- D22.1 open (waiting for the user), D22.2 open with a recommendation.
- Conventional commits, no attribution.
