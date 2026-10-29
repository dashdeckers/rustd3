# rustd3 — session handoff 54

2026-10-29 · **Phase 31j, steps 1–3 done; stopped on disk** (D22.10: the
benchmark for users of the plotting library alone) · no decision open ·
next: free disk, then step 4 (docs) and the verification runs

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Five commits on c49151a, each green under `scripts/check.sh quick`
(D22.10's Followups in `.claude/decisions/phase-22.md` say the same):

| Commit | What |
| --- | --- |
| 42b3968 `refactor(examples)` | the presets' figures in `rustd3_examples::presets` (`Data`, `NAMES`, `SIZE`, `figure`, `figures`, `rms`); the example writes the same 22 files byte for byte |
| 23e3d3d `feat(bench)` | the figure profile (`figure.rs`; `figure/{stage,subject,rss,report}.rs`), a process per figure (`figure-child`), `results.json` format 2, the level's report, `compare`, the README's schema, PRIVACY.md |
| 203930d `refactor(bench)` | the widget's part of a level behind `level::Widget`, in `level/widget.rs` (`Part`, `Profile`, `Machine`, `Limit`) |
| 5a957e4 `feat(bench)` | `--figure-only` at every level; `figure_only` in `results.json` |
| 207d3aa `feat(bench)` | the `widget` feature on rustd3-bench and rustd3-examples, the `rustd3-bench-figure` alias, `cli/widget.rs`, `commands.rs`, `workload/window.rs`, `check.sh`'s `no_widget` |

The figure profile: `build`, `resolve` (vector target, fresh cache),
`list`, `svg` and `pdf` from that list, `png-1x` and `png-2x` as
`save_png`'s whole work (a file target needs its own resolve and list),
`gif` (three frames) at `--full`; outputs in memory. Figures: `--quick`
trace, scatter100k, grid16, labels (3 × 3 panels of typst math), stats
(histogram and density of 1M values), heatmap (1000 × 1000), 7
repetitions after a cold one; `--default` adds lines of 10k–10M samples,
scatters of 1k–1M markers, the 11 presets and the wheel figure, 10
repetitions; `--full` scatters to 10M markers (skipped below about 23 GiB
of memory) and GIFs, 15 repetitions. Each figure in its own process: its
peak RSS (VmHWM, reset after its data; n/a off Linux) is its own.

Tests: 81 in the bench with the widget, 36 without it (`cargo test -p
rustd3-bench --no-default-features`); clippy clean in both builds
(`cargo clippy -p rustd3-bench -p rustd3-examples --all-targets
--no-default-features -- -D warnings`).

## 3. Measured here (2 cores at 2.1 GHz, verification only)

- `--quick --figure-only` (widget build), no `DISPLAY`: 64 s, archive of
  7 files (`PRIVACY.md`, `figure/report.md`, `figure/summary.csv`,
  `report.md`, `results.json`, `run-figure.log`, `system.txt`), no host,
  home or scratch path in it. Medians, ms:

  | Figure | build | resolve | list | svg | pdf | png-1x | png-2x | peak RSS |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | trace | 0.02 | 1.17 | 17.5 | 30.1 | 36.0 | 140 | 345 | 153 MiB |
  | scatter100k | 0.02 | 1.97 | 3.48 | 34.5 | 269 | 246 | 451 | 237 MiB |
  | grid16 | 0.03 | 1.13 | 19.5 | 44.3 | 140 | 236 | 462 | 164 MiB |
  | labels | 0.39 | 2.03 | 0.96 | 4.99 | 4.14 | 32.0 | 87.5 | 63 MiB |
  | stats | 3628 | 7.52 | 0.22 | 1.10 | 1.21 | 28.4 | 73.1 | 86 MiB |
  | heatmap | 1.29 | 7.32 | 44.0 | 119 | 60.9 | 230 | 298 | 89 MiB |

- The default figure plan at 5 repetitions took 124 s (scatter1m 62 s:
  PDF 2.9 s, PNG 2.7 s and 4.1 s, peak 1.4 GiB); at its 10 the estimate
  is 3.8 min. GIF of three frames: trace 0.82 s, labels 0.46 s, a preset
  0.09 s.
- Library findings: `density_x` over 1M values takes 3.6 s at mark build
  (an exact KDE, 200 points × 1M kernels); 1M markers peak at 1.4 GiB
  (vello_cpu's PNG and krilla's PDF).
- `cargo tree -p rustd3-bench --no-default-features -e normal`: no
  iced, winit or wgpu; 291 crates against 466 with the widget.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–i); 31j steps 1–3 | built; 31j step 4 and its verification left |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **Disk.** The session began with 7.6 GB free and stopped at 5.2 GB
   (incremental removed). Old outputs in `/tmp/claude-0/rustd3-bench31e`
   (751 MB), `31f` (1.6 GB) and `31g` (339 MB) are earlier sessions' and
   were left alone; removing them, with the architect's leave, gives
   about 8 GB.
2. **Step 4 (D22.10 (4)).** `cli::HELP`'s first line (widget build:
   "the benchmark of rustd3, the plotting library and its Plot widget";
   the no-widget `HELP` is written), `dev/bench/README.md` (the figure
   profile's figures, stages and metrics; the two builds; `run.sh`), the
   root README's "Reporting a performance problem" (both commands),
   `.github/ISSUE_TEMPLATE/performance.yml` (which command ran), the
   CHANGELOG.
3. **Verify** (the brief's list): `cargo build --release -p rustd3-bench
   --no-default-features` (time, size); `cargo rustd3-bench-figure
   --quick` end to end with no `DISPLAY`; `cargo rustd3-bench --quick
   --figure-only` (done with the widget build, above); the Xvfb quick
   level with the figure profile beside the widget's; `compare` of two
   figure-only runs; the privacy check of the new files (done once,
   above); `scripts/check.sh full` in the background.

## 6. Unverified, per platform

- **Everywhere:** the no-widget release binary has not run end to end
  (its 36 tests ran, the figure stages among them); the full level with
  the figure profile in front of the widget's has not run.
- **Windows:** nothing built. Specific: `figure-child` processes via
  `std::process` (stdout piped, a kill past ten times the estimate plus 2
  minutes), peak RSS "n/a", the memory guard reading memory through
  PowerShell's CIM, `cargo rustd3-bench-figure`'s build.
- **macOS:** nothing built; peak RSS "n/a"; memory through `sysctl`.
- **Linux:** `/proc/self/clear_refs` resets VmHWM here; where it is
  refused the peak is the process's whole life (data generation
  included).

## 7. Notes for later phases (0.2)

- Unchanged from handoff 53, and: `report` without the widget prints the
  stored report and rebuilds nothing; the bench reaches rustd3-figure's
  `pdf`, `gif` and `helpers` through rustd3-examples' features (declaring
  them in the bench's manifest is a one-line change for the architect).
- `density_x`'s exact KDE (3.6 s for 1M values here) and the 1.4 GiB peak
  of 1M markers are library costs the figure profile now shows.

## 8. Key files

- `dev/bench/src/figure.rs` (`Plan`, `child`), `figure/stage.rs`
  (`Stage`, `repetition`, `png`), `figure/subject.rs` (`Subject`, `Data`),
  `figure/rss.rs`, `figure/report.rs`; `level.rs` (`Level`, `Request`,
  `Widget`, `Outline`), `level/widget.rs` (`Part`); `cli.rs` (both
  `HELP`s), `cli/widget.rs`; `commands.rs`; `workload/window.rs`;
  `reference.rs` (its renders); `results.rs` (`Figure`, `Timing`, format
  2).
- `dev/examples/src/presets.rs`; `.cargo/config.toml`; `scripts/check.sh`
  (`no_widget`).

## 9. Commands

```bash
cargo rustd3-bench --quick [--figure-only]        # with the widget
cargo rustd3-bench-figure --quick                 # without iced
target/release/rustd3-bench --default --figure-only --dry-run   # the plan, the estimate
target/release/rustd3-bench figure-child --level quick --subject labels   # one figure, JSON
target/release/rustd3-bench compare A B
scripts/check.sh quick rustd3-bench rustd3-examples   # includes no_widget
```

## 10. End-of-session checklist

- `scripts/check.sh quick` green before each commit (with `no_widget`
  from 207d3aa); `scripts/check.sh full` not run (disk).
- Disk: 7.6 GB free at the start (the warm-up stopped at the bench's tests
  on this session's work in progress); 6.0 GB → 9.3 GB after removing
  incremental once; 3.2 GB after the first no-widget checks, 5.2 GB after
  removing it again: stopped.
- Manifests: the `widget` features, optional iced, iced_runtime and
  rustd3-iced, the examples' `required-features`; no new crate, no
  `Cargo.lock` change, no `cargo update`, no install, no push, no AI
  attribution.
- Decisions: none open; D22.10 gained its Followups.
