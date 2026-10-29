# rustd3 — session handoff 53

2026-10-29 · **Phase 31i is done** (D22.9: the benchmark as a
first-class, user-facing tool) · no decision open · next: the user runs
`cargo rustd3-bench --quick` on the desktop (and a Windows or macOS
machine, if one is at hand), then the release

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31i on c9b2c9d, eight commits (D22.9's Followups in
`.claude/decisions/phase-22.md` say the same):

| Commit | What |
| --- | --- |
| cd3fb11 `feat(bench)` | `rustd3-bench [--quick \| --default \| --full] [--out DIR] [--refresh-hz HZ] [--native-factor F]`: bare flags and no command are a level, `--default` the default; the plan and an estimate first (hidden `--dry-run` stops there); the display measurement (`run --plan display`, a 3 s hover) in place of smoke; the profiles `native` (`--backend auto`), `nogpu`, `nogpu-x11`, `4cores`, `2cores`; `Plan::Quick` reshaped; `run --budgets`, `--no-also`; runs killed 2 min past their scenario; the run header names the renderer that ran; `suite` gone; `run.sh` a wrapper; `.cargo/config.toml`'s alias |
| 326581b `feat(bench)` | `results.json` (format 1), the level's `report.md`, `PRIVACY.md`; the archive (a ustar written here, gzipped by flate2: `dev/bench/Cargo.toml` only, 1.1.10 already locked, the lock gains one dependency line and no crate), rewritten after every step; the privacy scrub; the machine on macOS and Windows |
| a07f4df `feat(bench)` | `compare A B` and `report PATH` on directories and archives, and on the layout before the levels |
| 4f8a3fe `feat(bench)` | the CPU reference (its message says about 10 s; it is about 5 s here) |
| 008f131 `feat(bench)` | `also` at the end of `--full` |
| 8084cb6 `fix(bench)` | `repro-software` records the display variables by presence |
| efa8e99 `docs(bench)` | root README "Reporting a performance problem", the issue form, `dev/bench/README.md` rewritten (schema, privacy, the rule verbatim for developers), the changelog |
| 2f97f1f `fix(bench)` | an unambiguous doc link (`check.sh full`'s rustdoc read `unreachable` as std's macro too) |

Tests (69 in the bench): the levels' profiles and skips as decision
functions over a `Machine` (headless, no GPU, 1, 2 and 16 cores,
Wayland), the display's decision (given, measured, unpaced, software,
headless) and its raw reading, `run`'s arguments round-tripping, the
plans, the CLI, the archive (round trip, long names, `tar tzf`, refused
paths), the scrub (paths, tokens, Windows spellings), an archive built
under a fake home with the host name seeded holding neither, the report,
`results.json`'s round trip, `compare` on synthetic results, loading a
directory, its archive and the old layout. Harness unchanged (no core
or harness change).

## 3. Measured here (verification, not rule outcomes)

2 cores at 2.1 GHz, wgpu on Mesa lavapipe, Xvfb (no vsync):

| Run | Wall | Archive |
| --- | --- | --- |
| `--quick --refresh-hz 60` (final build) | 6 min 20 s | 3.6 MB, 66 files |
| `--quick` (unpaced: 60 Hz assumed, flagged) | 6 min 19 s | 3.6 MB, 66 files |
| `--quick`, no `DISPLAY` or `WAYLAND_DISPLAY` (headless) | 6 min 11 s | 2.6 MB, 56 files |
| `--default --dry-run`, `--full --dry-run` | estimates 60 and 422 min (`4cores`, `nogpu-x11` skipped) | — |

The CPU reference: about 5 s; trace tile 3.5 ms, `scatter100k` area
200–250 ms filled and 31–38 ms splatted, software composition 20 ms.
`also` alone takes about 2 minutes. Ctrl-C (SIGINT to the process group
40 s in) left the archive of the display measurement and the CPU
reference, `"complete": false`. Two quick runs compared agree within about
10 % on most metrics. `compare` and `report` read the user's three suite
runs from their `summary.csv` files.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–i) | built; the user's runs pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user:** `cargo rustd3-bench --quick` in the COSMIC session
   (about 10 minutes plus the build), which also runs `nogpu-x11`, and
   sends the archive it prints last. It is the first run with a display
   that paces frames and in a Wayland session.
2. On Windows or macOS, if available: the same command. Everything
   there is unverified (below).
3. Still open from handoff 52: `dev/bench/repro-software.sh` in the
   COSMIC session for D22.8's upstream issue.
4. Release: the user follows `RELEASING.md`.

## 6. Unverified, per platform

- **Windows:** nothing built here. The levels' code is shared; specific
  to Windows are `cmd /C start "" /B /WAIT /AFFINITY` for `4cores` and
  `2cores` (as in 31g), the machine through PowerShell's CIM
  (`Win32_Processor`, `Win32_ComputerSystem`, `Win32_VideoController`)
  and `cmd /C ver`, `USERPROFILE`/`USERNAME`/`COMPUTERNAME` in the scrub
  (its Windows spellings are unit-tested as text only), the archive's
  writing and renaming over itself, and `cargo rustd3-bench` (no
  `run.sh`).
- **macOS:** nothing built here. `sysctl` (CPU, memory), `sw_vers`,
  `system_profiler SPDisplaysDataType` (GPU), `--threads` in place of
  pinning, and whether iced's windows report a paced rate there.
- **Linux:** checked under Xvfb and headless only: a Wayland session
  (`nogpu-x11`), a real display's measured rate, `lspci` naming a GPU,
  and `taskset` pinning at the default level (the 2-core container skips
  `4cores`) are the user's to see.

## 7. Notes for later phases (0.2)

- Unchanged from handoff 52.
- `report` does not rebuild the layout before the levels (it reads its
  `summary.csv` files); its machine and display come from `meta.json`
  where present, else from the report's lines.
- A display measurement whose window opens but never draws (an occluded
  Wayland window) is killed after its two-minute margin and the level
  goes headless; not seen here.

## 8. Key files

- `dev/bench/src/level.rs` (`Level`, `Request`, `Profile`, `Machine`,
  `Limit`, `save`), `display.rs` (`Display`, `Source`, `unreachable`),
  `results.rs` (`Results`, the level's report), `archive.rs`,
  `privacy.rs` (`Identity`, `PRIVACY`), `load.rs`, `compare.rs`,
  `reference.rs`; `plan.rs` (`Plan::Display`, `Plan::Quick`), `cli.rs`
  (`HELP`), `main.rs` (`report`).
- `dev/bench/README.md`, `dev/bench/run.sh`, `.cargo/config.toml`,
  `.github/ISSUE_TEMPLATE/performance.yml`.

## 9. Commands

```bash
cargo rustd3-bench --quick                         # the user's machine
xvfb-run -a -s "-screen 0 1920x1080x24" target/release/rustd3-bench --quick [--refresh-hz 60] --out /tmp/…
env -u DISPLAY -u WAYLAND_DISPLAY target/release/rustd3-bench --quick --out /tmp/…   # headless
target/release/rustd3-bench --default --dry-run    # the plan and the estimate
target/release/rustd3-bench compare A B            # directories or archives
target/release/rustd3-bench report A
scripts/check.sh quick rustd3-bench
```

## 10. End-of-session checklist

- `scripts/check.sh quick rustd3-bench` green before each code commit;
  `scripts/check.sh quick rustd3` for the README's doctests;
  `scripts/check.sh full` at 2f97f1f: green in 22 min 26 s (1,346 s, with
  `target/debug/incremental` rebuilt after the cleanup below), no
  warnings; its first run, at efa8e99, stopped at rustdoc on an ambiguous
  link (fixed in 2f97f1f).
- Disk: 7.2 GB free after the warm-up (which stopped at the bench's tests
  on this session's work in progress). Below 6 GB twice (5.5 GB after
  the first release builds, 5.4 GB at the end of the first `check.sh
  full`, 5.3 GB at the end of the second): `target/debug/incremental` was
  removed each time (→ 8.9, 8.5 and 8.4 GB), and the earlier sessions'
  smoke outputs in `target/bench-smoke/` (367 MB). Each full check
  rebuilds about 3 GB of incremental state; the next session starts at
  8.4 GB.
- Manifest: `flate2` in `dev/bench/Cargo.toml` only; no `cargo update`, no
  install, no push, no AI attribution.
- Decisions: none open; D22.9 gained its Followups.
