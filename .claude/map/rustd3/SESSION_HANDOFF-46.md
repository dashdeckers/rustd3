# rustd3 — session handoff 46

2026-10-28 · **Phase 31 — Plot V: tiles and pan translation**: **complete except the user's rerun** (session 31d: D31.7, D31.8, Steps 9–10) · next: **the user's benchmark rerun** (command below), then Phase 40; D31.10 open

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- f6e611e `feat(figure)` — D31.7 (decided B): the object-safe `Scale`
  gains the additive `Scale::remap(from, range, at)` (a provided method,
  default `None`; continuous and time scales through `Invert`; band and
  point scales `at` when they place every key alike), with a doctest.
- 0978b38 `feat(iced)` — D31.7: after the resolve, each area the views
  translated is confirmed by its lead handles' scales at its two edges
  (`State::confirm`, `Area::keeps`, within a 256th of a device px); an
  area that fails restarts its grid where it lies (`Area::restart`).
  Test `visible_y_refit_during_a_pan_re_renders` (`iced/tests/main/
  classify.rs`): all 20/35 tiles at once, 1/255; without the test 0
  tiles, 254/255.
- acb8547 `fix(iced)` — `threads_sets_the_pool_size` waits for the pool's
  threads to be named (a pre-existing race: std names a thread from
  inside it).
- 0189f1b `feat(iced)` — D31.8 (decided B): `job::timed` times every
  render on the monotonic clock (scoped `expect`, reason naming D31.8);
  every render feeds the fit, probe or not. Test
  `renders_calibrate_the_cost_model_without_a_probe`;
  `ten_hz_appends_stay_sharp` forces `Render::Inline`.
- e7c8c8c `feat(bench)` — Step 9: `--threads N`, `--caching verify`
  (mismatch records, counted in Flags), D22.3's windows (factor cells
  1600 × 1000 device px; `4k` "n/a: display too small"), D22.4's `weak`
  plan with its budgets (`rule::Profile`) and report section, the
  rerun's outcome lines, `run.sh` running the weak profiles (no GPU, 4
  cores, 2 cores) after the default plan, the README's run sheet, and
  the wheel workload's gestures on its scatters (`scenario::Anchors`;
  `pan-x` n/a there).
- a96a5df `docs` — `decisions/phase-31.md` (D31.7/D31.8 built, 31d
  measurements, D31.10), `decisions/phase-22.md` (the rerun's setup and
  command), the index. Then this handoff with the full check's time.

Exit criteria (the section's numbering): 1–5 hold as in 31c, every iced
test on tiny-skia and wgpu (`ICED_TEST_BACKEND=… TZ=Asia/Kathmandu cargo
test -p rustd3-iced --all-features`), D31.7's new test included; 6 — the
user's `--plan default` rerun is **not yet appended** (it needs the
user's hardware); `scripts/check.sh full` passes (section 9).

Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
skipped, 179 floors; nothing blessed.

Decisions: D31.7 and D31.8 built; **D31.10** open (thirteen details of
31d as built, A recommended).

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | done; D22.1 decided C, its rerun pending |
| M3 Core breadth | 23–30 | done |
| Conditional | 31 | built (31a–31d); the user's rerun pending |
| M4 Complete figure | 32–39 | done |
| M5 Release | 40 | after the rerun's outcome |

## 5. What to do next

1. **The user's rerun** (Phase 31's exit criterion 6), on the machine of
   D22.1's run, from the repository root at this commit, on power, other
   GPU-heavy apps closed, mouse and keyboard untouched for about 75
   minutes:

   ```bash
   dev/bench/run.sh
   tar czf bench.tgz -C bench-results <the stamp the script prints>
   ```

   (`HZ=60 FACTOR=1 dev/bench/run.sh` if detection fails; D22.1's display
   was 59.96 Hz at factor 1.) It runs smoke, the default plan, then
   `weak-nogpu`, `weak-4cores`, `weak-2cores`.
2. When it comes back: append the default plan's outcome to D22.1 in
   `decisions/phase-22.md` (image path sufficient, or GPU layer
   recommended) and each weak profile's outcome (a miss: Phase 31 work if
   still open, else a 0.2 item); Phase 34's label-churn rerun is there,
   so write the closing line. Then Phase 40.
3. The architect answers D31.10.

## 6. Notes for later phases

- **`Caching::Verify` is a diagnostic, not a timing mode.** Headless at
  2 on 2 cores with one pool thread, a `pan-x` over `trace` goes from
  latency p95 0.64 s, previews 35 %, inline 54 % to 9.1 s, 98 %, 13 %:
  the checks keep the pool and the second core busy and, fed into the
  fit (D31.8), push more renders to the pool. Never compare its timings.
- **The two-point test covers lead handles only** (`Map::lead`): a
  `Fit::Visible` y that no axis shows and the panel does not zoom is
  still not seen during a drag (it was not before either).
- **Headless numbers stay report-only.** A reduced weak run here missed
  its pan budgets at 2 (2 cores, readback every frame); the user's
  machine decides.
- `rustd3-bench raster` still measures its own splat module; `also`
  measures `Renderer::splat`.

## 7. Key files

`figure/src/scale.rs` (`Scale::remap`, `through`), `figure/src/scale/
{continuous,time,band,point}.rs`, `iced/src/plot/{area,state,job,
budget}.rs`, `iced/tests/main/classify.rs`, `dev/bench/src/{cli,plan,
rule,report,scenario,workload,record,driver,metrics,main,app}.rs`,
`dev/bench/src/workload/wheels.rs`, `dev/bench/run.sh`,
`dev/bench/README.md`, `.claude/decisions/phase-{22,31}.md`.

## 8. Commands

```bash
scripts/check.sh quick                     # every commit (runs the ratchet)
scripts/check.sh full                      # end of a session (background)
ICED_TEST_BACKEND=wgpu TZ=Asia/Kathmandu cargo test -q -p rustd3-iced --all-features
cargo build --release -p rustd3-bench && target/release/rustd3-bench list --native-factor 1
HZ=60 FACTOR=1 xvfb-run -a dev/bench/run.sh --smoke-only   # the windowed path under Xvfb, ~6 min
target/release/rustd3-bench run --headless --plan weak --backend tiny-skia --out target/bench-weak
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` at a96a5df:
  green in 17 min 19 s (warm, the benchmark smoke included: 53 runs,
  `pan-x × wheels` n/a), no warnings. The architect's cold warm-up full
  before the session: 34 min 9 s. Free disk after: 8.9 GB.
- Disk: the session's bench outputs (`target/bench-p31d-*`, the Xvfb
  runs' `bench-results/` stamps) deleted.
- D31.10 open with its recommendation; D31.7 and D31.8 built.
- Conventional commits, no attribution.
