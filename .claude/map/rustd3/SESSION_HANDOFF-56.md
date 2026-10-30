# rustd3 — session handoff 56

2026-10-30 · **Phase 31l is done** (why every software zoom's latency
p95 is the wheel burst's length) · open: **D22.14** (recommended A: no
widget change; D22.13's upstream fix, and a continuous-input reading in
the report) and D22.13's upstream patch · next: the architect answers
D22.14; the pin moves when iced has D22.13's fix

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

| Commit | What |
| --- | --- |
| 24d4130 `fix(bench)` | an experiment's `--factor` reaches the widget profiles' runs: it narrowed a profile's cells (plan, estimate, description) but the run it started got no `--factor` and measured every factor; the experiment test now checks the run's arguments (fails without the fix) |
| docs | D22.14 in `phase-22.md`, `DECISIONS.md`'s index, this handoff |

No widget change: the cause is neither a behaviour bug (a) nor a probe
or metric bug (b) (D22.14).

## 3. The finding (D22.14)

On the Ryzen a software frame at factor 1 costs 18–28 ms (cost p50),
more than the 16.7 ms period, so paced frames come every 33.3 ms, which
is the `zoom-lines` wheel's interval (30 Hz). Every burst frame applies
a new step; wheel steps render their tiles on the pool only, so the frame
shows the previous step's tiles mapped (24 of 24 previews). The pool
keeps up there (5,050 of 5,050 generations fully drained before the next
frame); the results join the preview layer because the next step has
begun a new grid. No burst frame is sharp (0 of 1,186–1,195 per cell at
1), and the first frame without a new step is sharp in 40 of 40 bursts.
The native path draws at 60 Hz, a frame between steps, hence 22 ms.

Evidence measured here (2 cores at 2.1 GHz, factor 1, 2 runs; `before`
is the default build):

| Run | `trace` p50/p95 ms | `scatter100k` p50/p95 ms | cost p50 ms |
| --- | --- | --- | --- |
| default (`auto`) | 625/1,075 | 642/1,092 | 42 / 40 |
| `--threads 1` / `2` / `8` | 575–584/1,025 | 584–617/1,042–1,067 | 35–47 / 32–59 |
| `--render inline` | 42/92 | 59/109 | 74 / 95 |
| wheel at 10 Hz (temporary, not committed) | 134/292 | 359/875 | 31 / 30 |

At factor 2 (default build): `trace` 817/1,300, `scatter100k`
1,025/2,584 (the pool lags on 2 cores there, which the Ryzen's does
not). Mid-burst frames (`--frames --every 2`) show the previous step's
tiles crisp, without seams or holes; the frame after a burst is sharp.

Scripts and outputs: `/tmp/claude-0/sub/a31l/` (`burst.py` the
per-frame table from a raw `.jsonl`, `cadence.py` sharpness of burst
frames from `frames.csv` and `inputs.csv`, `keepup.py` the pool's
per-generation drain, `summ.py` the zoom summary; `out/before-*`,
`out/render`, `out/wheel10`).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–l) | built; D22.14 open; D22.13 upstream |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The architect:** D22.14. A needs nothing here; D adds a reading to
   the report (`metrics.rs`, `latency`): per input, the first frame
   showing its step or a later one, preview layer included, and the
   sharp share of burst frames. B (a wheel frame waits for its tiles on
   the pool, with a deadline) would be measured against A once the pin
   moves.
2. When iced has D22.13's fix: move the pin in its own commit, rerun the
   iced tests on both backends, and on the Ryzen `cargo rustd3-bench
   --quick --only software --scenario zoom-lines --factor 1`: zoom
   latency p95 below 50 ms where frame cost p50 falls below 16.7 ms.
3. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

- The software profile's description says "at factors 1 and 2" even
  when `--factor` selects one (static text in `level/widget.rs`).
- Each frame of a wheel step plans two generations (the widget's step,
  then the app's rebuild); only the second renders tiles. Harmless,
  noted while tracing.

## 7. End-of-session checklist

- `scripts/check.sh quick` green before each commit.
- `scripts/check.sh full` at 24d4130: see section 8.
- Disk: 9.9 GB free at the start, 6.2 GB after the full check (3.7 GB of
  it `target/debug/incremental`); frame PNGs deleted after viewing.
- Manifests unchanged; no install, no push, no AI attribution.
- Decisions: D22.14 open.

## 8. `check.sh full`

Green at 24d4130 in 26 min 8 s (1,568 s): the generated tables, clippy,
the no-widget checks, every package's tests with and without default
features, the iced, examples and bench tests on tiny-skia and wgpu, the
translator check, the docs and the headless smoke plan.
