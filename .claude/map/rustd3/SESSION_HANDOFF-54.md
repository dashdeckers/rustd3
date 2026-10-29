# rustd3 — session handoff 54

2026-10-29 · **Phase 31j is done** (D22.10: the benchmark for users of
the plotting library alone) · no decision open · next: the user runs
`cargo rustd3-bench --quick` on each machine (and
`cargo rustd3-bench-figure --quick` where only the library matters), then
the release

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31j on c49151a, eight commits, each green under `scripts/check.sh
quick`, the last under `scripts/check.sh full` (D22.10's Followups in
`.claude/decisions/phase-22.md` say the same). The session stopped once on
disk after 207d3aa (handoff e6a0ae9) and resumed when the architect freed
13 GB.

| Commit | What |
| --- | --- |
| 42b3968 `refactor(examples)` | the presets' figures in `rustd3_examples::presets` (`Data`, `NAMES`, `SIZE`, `figure`, `figures`, `rms`); the example writes the same 22 files byte for byte |
| 23e3d3d `feat(bench)` | the figure profile (`figure.rs`; `figure/{stage,subject,rss,report}.rs`), a process per figure (`figure-child`), `results.json` format 2, the level's report, `compare`, the README's schema, PRIVACY.md |
| 203930d `refactor(bench)` | the widget's part of a level behind `level::Widget`, in `level/widget.rs` (`Part`, `Profile`, `Machine`, `Limit`) |
| 5a957e4 `feat(bench)` | `--figure-only` at every level; `figure_only` in `results.json` |
| 207d3aa `feat(bench)` | the `widget` feature on rustd3-bench and rustd3-examples, the `rustd3-bench-figure` alias, `cli/widget.rs`, `commands.rs`, `workload/window.rs`, `check.sh`'s `no_widget` |
| e6a0ae9 `docs` | D22.10's Followups and this handoff at the disk stop |
| 5094eba `build(bench)` | rustd3-figure's `pdf` and `gif` declared in the bench's manifest (the architect's leave): the bench writes PDF and GIF itself; `helpers` stays with rustd3-examples, the bench using no item of it |
| 8cdd4d4 `docs(bench)` | D22.10 (4): `--help`'s first lines, both READMEs, the issue form (which command ran), `run.sh`, the changelog |

The figure profile: `build`, `resolve` (vector target, fresh cache),
`list`, `svg` and `pdf` from that list, `png-1x` and `png-2x` as
`save_png`'s whole work (a file target needs its own resolve and list),
`gif` (three frames) at `--full`; outputs in memory, held by tests to the
bytes the `save_*` calls write. `--quick`: trace, scatter100k, grid16,
labels (3 × 3 panels of typst math), stats (histogram and density of 1M
values), heatmap (1000 × 1000), 7 repetitions after a cold one;
`--default` adds lines of 10k–10M samples, scatters of 1k–1M markers, the
11 presets and the wheel figure, 10 repetitions; `--full` scatters to 10M
markers (skipped below about 23 GiB of memory) and GIFs, 15 repetitions.
Each figure in its own process: its peak RSS (VmHWM, reset after its
data; n/a off Linux) is its own, its first repetition truly cold.

Tests: 81 in the bench with the widget, 36 without it; clippy clean in
both builds.

## 3. Measured here (2 cores at 2.1 GHz, verification only)

| Run | Wall | Archive |
| --- | --- | --- |
| `cargo build --release -p rustd3-bench --no-default-features`, clean | 10 min 6 s, 298 crates compiled | binary 60.3 MB (85.1 MB with the widget) |
| `cargo build --release -p rustd3-bench` on top of it | 13 min 4 s more | — |
| `cargo rustd3-bench-figure --quick`, no `DISPLAY` | 66 s (65 s run) | 8 kB, 7 files |
| `rustd3-bench --quick --figure-only` (full build), no `DISPLAY` | 67 s | 8 kB, 7 files |
| `cargo rustd3-bench --quick --refresh-hz 60` under Xvfb | 7 min 24 s | 3.6 MB, 69 files: `figure` beside `native`, `nogpu` (`nogpu-x11` skipped) |

- The figure-only archive: `PRIVACY.md`, `report.md`, `results.json`,
  `run-figure.log`, `system.txt`, `figure/report.md`,
  `figure/summary.csv`; no host, home or scratch path in it or in the
  Xvfb level's. Without the widget the CPU reference reports the software
  composition as not measured; `run`, `--refresh-hz` and the other widget
  commands and flags are refused; `report` prints the stored report.
- `compare` of the two figure-only runs (no-widget build against the
  full build's `--figure-only`): the same figures and stages, B/A 0.70 to
  1.27 per stage, the CPU references within 12 %.
- Medians of the no-widget run, ms:

  | Figure | build | resolve | list | svg | pdf | png-1x | png-2x | peak RSS |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | trace | 0.02 | 1.58 | 22.7 | 32.8 | 44.3 | 154 | 385 | 141 MiB |
  | scatter100k | 0.02 | 1.98 | 4.06 | 45.4 | 240 | 270 | 490 | 226 MiB |
  | grid16 | 0.03 | 1.68 | 23.5 | 51.0 | 159 | 223 | 481 | 154 MiB |
  | labels | 0.43 | 2.36 | 0.99 | 5.41 | 4.40 | 33.3 | 91.1 | 58 MiB |
  | stats | 3823 | 7.61 | 0.23 | 1.10 | 1.18 | 28.6 | 71.4 | 80 MiB |
  | heatmap | 1.32 | 7.53 | 43.2 | 126 | 59.3 | 230 | 308 | 82 MiB |

- The default figure plan at 5 repetitions took 124 s (scatter1m 62 s);
  at its 10 the estimate is 3.8 min. GIF of three frames: trace 0.82 s,
  labels 0.46 s, a preset 0.09 s.
- `cargo tree -p rustd3-bench --no-default-features -e normal`: no
  iced, winit or wgpu; 291 crates against 466 with the widget.

### Found by the figure profile (not fixed here)

- **stats `build`: 3.6–3.9 s.** `density_x` computes its estimate over
  1M values exactly when the mark is built: 200 points × 1M Gaussian
  kernels (`figure/src/stats/kde.rs`'s `grid`), single-threaded. A
  binned estimate would make it milliseconds.
- **scatter1m: a 1.4 GiB peak.** vello_cpu's PNG at 2× (about 1.4 GiB)
  and krilla's PDF (about 0.55 GiB of it) hold the most; PDF 2.9 s, PNG
  2.7 s at 1× and 4.1 s at 2×. 10M markers would need about 14 GiB, so
  the figure profile skips them below about 23 GiB.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–j) | built; the user's runs pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user:** `cargo rustd3-bench --quick` on every machine (about 10
   minutes plus the first release build, about 23 minutes here from
   clean), and on a weak or display-less one `cargo rustd3-bench-figure
   --quick` (about a minute plus a 10-minute build here); send the
   archives each prints last.
2. Still open from handoffs 52 and 53: `dev/bench/repro-software.sh` in the
   COSMIC session for D22.8's upstream issue.
3. Release: the user follows `RELEASING.md`.
4. A later phase may take the two costs above (a binned density, the
   memory of large marker sets).

## 6. Unverified, per platform

- **Windows:** nothing built here. Specific to the figure profile:
  `figure-child` processes through `std::process` (stdout piped, killed
  past ten times the estimate plus 2 minutes), peak RSS "n/a", the memory
  guard reading `TotalPhysicalMemory` through PowerShell's CIM, the
  no-widget build (no platform code of its own), the cargo alias. The
  widget's part as in handoff 53.
- **macOS:** nothing built; peak RSS "n/a"; memory through `sysctl -n
  hw.memsize`.
- **Linux:** checked headless and under Xvfb, with and without the
  widget. `/proc/self/clear_refs` resets VmHWM here; where it is refused
  the peak is the process's whole life (data generation included). A
  Wayland session and a paced display as in handoff 53.

## 7. Notes for later phases (0.2)

- Unchanged from handoff 53, and: `report` without the widget prints the
  stored report and rebuilds nothing; the no-widget build allows dead
  code (a `cfg_attr` in `main.rs`), the default build checks for it.
- `results.json` format 2 adds `figure_only` and the figure profile's
  `figures`; format 1 reads without them.

## 8. Key files

- `dev/bench/src/figure.rs` (`Plan`, `child`), `figure/stage.rs`
  (`Stage`, `repetition`, `png`), `figure/subject.rs` (`Subject`, `Data`),
  `figure/rss.rs`, `figure/report.rs`; `level.rs` (`Level`, `Request`,
  `Widget`, `Outline`), `level/widget.rs` (`Part`); `cli.rs` (both
  `HELP`s), `cli/widget.rs`; `commands.rs`; `workload/window.rs`;
  `reference.rs` (its renders); `results.rs` (`Figure`, `Timing`, format
  2).
- `dev/examples/src/presets.rs`; `.cargo/config.toml`; `scripts/check.sh`
  (`no_widget`); `dev/bench/README.md`, `README.md`'s "Reporting a
  performance problem", `.github/ISSUE_TEMPLATE/performance.yml`.

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
  from 207d3aa); `scripts/check.sh full` at 8cdd4d4: green in 24 min 39 s
  (1,479 s), no warnings, its no-widget checks and the headless smoke
  plan included.
- Disk: 7.6 GB free at the start; a stop at 5.2 GB after 207d3aa; 13 GB
  after the architect's cleanup; 4.0 GB during `check.sh full`, so this
  session's verification outputs and the idle `target/debug/incremental`
  directories went (6.0 GB), the rest once it ended (7.1 GB free at the end).
- Manifests: the `widget` features, optional iced, iced_runtime and
  rustd3-iced, the examples' `required-features`, figure's `pdf` and `gif`
  in the bench's; no new crate, no `Cargo.lock` change, no `cargo
  update`, no install, no push, no AI attribution.
- Decisions: none open; D22.10 gained its Followups.
