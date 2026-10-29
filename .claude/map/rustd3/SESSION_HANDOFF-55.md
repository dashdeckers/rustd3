# rustd3 — session handoff 55

2026-10-30 · **Phase 31k is done** (D22.12 built with D22.11 (1), (3)
and (4)) · open: **D22.13** (the software path is composition-bound
inside iced's tiny-skia; recommended B, upstream in iced) · next: the
user runs the three commands in section 5 on the Ryzen, then the
architect answers D22.13

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31k on be39fb0, five commits and this handoff, each green under
`scripts/check.sh quick` (with its no-widget check), the last code
commit under `scripts/check.sh full` (section 8).

| Commit | What |
| --- | --- |
| 45eb931 `feat(bench)` | the `software` plan and profile (D22.12 (1)): tiny-skia headless, every frame composed offscreen and timed by part (UI work, composition, wait on the pool), `parts.csv`, "Frame cost by part", the frame budget read as cost (`rule::Reading::Offscreen`), an empty frame's readback, `--frames DIR [--every N]`; first among the widget's profiles at every level |
| 35246ad `feat(bench)` | experiment flags (D22.11 (1) without `--repro-software`): `--only`, `--scenario`, `--workload`, `--factor`, `--runs`, `--render`, `--threads`, `--caching`, `--sweep KEY=V1,V2` (`experiment.rs`); `results.json`'s `experiment`, the report's Experiment and Sweep sections, `compare`'s Experiment row; `--help`'s "Experiments" group |
| c02d00e `feat(bench)` | D22.13: `also`'s per-image composition row (`also/compose.rs`, `per_images`) and the decision entry; D22.12's Followups |
| bfceb60 `perf(figure)` | the binned density (D22.11 (3)): `Kde::BINNED` (10,000), `Kde::exact`, `Kde::binned`; tests in `figure/src/stats/kde.rs`, `dev/examples/src/wheels.rs`, `dev/bench/src/figure/subject.rs` |
| 428575e `build(bench)` | `dev/bench/run.sh` and `dev/bench/repro-software.sh` removed (D22.11 (4)); `repro-software` stays an advanced command |

Design notes:

- **The software profile** is `run --plan software --headless --backend
  tiny-skia --budgets weak --refresh-hz 60 --native-factor 2`, so both
  factors keep 1600 × 1000 logical px (1600 × 1000 and 3200 × 2000
  device px). It reuses the headless loop (paced at 60 Hz in the
  replay's clock, readback every frame). A frame's parts come from the
  existing records joined by instant: UI = update + draw over the
  instant's passes; composition = the screenshot (`Harness` record);
  wait = after a stale frame, its end to the next frame. `Readback`
  (new record) is an empty frame's screenshot before the first frame.
  Latency is unchanged (to the sharp frame's instant) and gets a split.
- **Runs:** 1 a cell at `--quick`, 3 at `--default`, 5 at `--full`. The
  quick level's estimate is now about 15 minutes (16 with the X11
  profile), the default about 95.
- **Experiments:** `--only` drops unselected profiles from the level
  (the figure profile included); with no selected profile that opens a
  window the display is not measured (the level reads as headless,
  "no profile selected opens a window (--only)"). A sweep's variants are
  profiles named `<profile>@KEY=V`, each with its directory and log; a
  sweep and its own flag together are refused. `Command::Level` now
  boxes its `Request`.
- `results.json` stays format 2: `experiment` is optional, and the
  software profile is a new profile (its `frame p99` budget reads cost).

## 3. Measured here (2 cores at 2.1 GHz, Xeon, no display)

**The software profile's 24 cells** (`cargo rustd3-bench --quick --only
software`, 6 min, one run each), means per composed frame, ms:

| Factor | UI | composition (its readback) | pool wait | composition's share |
| --- | --- | --- | --- | --- |
| 1 | 0.2–15.0 | 19.6–33.1 (3.2–10.9) | 0–8.0 | 58–99 % |
| 2 | 0.4–38.6 | 78.3–151.6 (14.8–36.4) | 0.1–19.8 | 72–99 % |

Frame cost p50/p99: 20–41 / 30–73 ms at 1, 100–169 / 129–205 at 2. Pan
latency p95 43–58 ms at 1, 284–589 at 2; zoom latency p95 1.0–2.6 s (the
wheel bursts' length, as on the desktop). D22.13 has the per-image and
per-pixel numbers.

**The sweep** (`--quick --only software --sweep threads=1,2,8 --scenario
zoom-lines --workload trace`, 89 s, 40 files, 734 kB), cost p50 at 1 /
2: 31.5 / 112 ms (1 thread), 41.6 / 135 (2), 44.9 / 164 (8): on two
cores more threads only add contention; `compare` of the archive with
its directory lines up `software@threads=1`, `=2`, `=8` at B/A 1.00.

**The density:** the stats figure's build 3.53–3.59 s before, 36–37 ms
after (`figure-child --level quick --subject stats`, release). Largest
relative error against the exact estimate: 1.6e-5 (stats, 1M values),
≤ 5.8e-5 (the wheel figure's 12 densities), 2.7e-4 (a bimodal 100k
sample with a far value), 4e-5 at the threshold.

**End to end without a display:** `cargo rustd3-bench --quick --only
software` (6 min, 49 files, 2.4 MB); `cargo rustd3-bench-figure --quick`
(7 min 52 s with its no-widget release build, the level itself about
30 s, 7 files, 8 kB; stats build 36.4 ms).

**Frames viewed** (`--frames … --every 15/60`): trace mid-zoom at 1,
grid16 mid-zoom at 2 and scatter100k mid-drag-resize at 2 render as
expected; mid-drag frames at 2 show the x-axis gutter missing and, at a
drag's end, the outgoing layout's y-axis strip beside an area not yet
relaid (they settle sharp; noted in D22.13's Followups, not
investigated).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–k) | built; D22.13 open; the user's runs pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user, on the Ryzen** (from a checkout at 428575e or later; the
   first build after this session's changes takes a few minutes more):

   ```bash
   # Where a software frame's time goes on a fast machine, every D22.12
   # cell, and the library's figures (the stats figure's new build time):
   # about 7 minutes.
   cargo rustd3-bench --quick --only software,figure

   # Whether the render pool's size moves the software path (frame cost,
   # pool wait, latency) when sixteen threads are there: pans and zooms of
   # trace and grid16 at factors 1 and 2, about 7 minutes.
   cargo rustd3-bench --quick --only software --sweep threads=1,2,8 \
     --scenario pan-x,zoom-lines --workload trace,grid16

   # Whether rendering inline on the UI thread or always on the pool
   # changes a software frame's cost and latency at factor 2, where
   # composition is heaviest: about 3 minutes.
   cargo rustd3-bench --quick --only software --sweep render=auto,inline,worker \
     --scenario pan-x,zoom-lines,drag-resize --workload trace --factor 2
   ```

   Each prints one archive path last.
2. **The architect:** D22.13 (recommended B: an iced issue and patch, no
   clip mask for an image inside its clip and row copies for opaque 1:1
   nearest images; the pin moves only after it lands). If A (the mosaic
   on every software frame) is wanted as a stopgap, a sweep against the
   current build measures it.
3. Release: the user follows `RELEASING.md`.

## 6. Unverified, per platform

- **Windows and macOS:** nothing built here. New since handoff 54: the
  software profile (headless tiny-skia, no platform code of its own),
  PNG frames written from a thread, profile directories named
  `software@threads=2` (valid on both; `@` and `=` in a path), the
  experiment flags (parsing only).
- **Linux:** checked headless (no `DISPLAY`) with and without the
  widget. Not run this session: a windowed level (Xvfb), a Wayland
  session, and `--sweep` over a windowed profile (its variants' args
  are unit-tested); `--only` with `nogpu-x11` on a Wayland session.
- The software profile's timings with `--frames` are perturbed by the
  PNG encoding (flagged in the report).

## 7. Notes for later phases (0.2)

- D22.13 (open). Also there: the offscreen readback (RGBA conversion)
  costs 3–11 ms at 1 and 15–36 ms at 2 in every composition here; a
  window does not pay it, and the report shows it apart.
- The figure profile's stats estimate still assumes the exact density's
  3.6 s (`figure/subject.rs`, `estimate`), so `--quick`'s figure line
  overestimates by about half a minute.
- `metrics::Metrics::latency_shares` puts compositions in the upload
  share for any headless run that read every frame back (the smoke plan
  included); all headless numbers are report-only.

## 8. End-of-session checklist

- `scripts/check.sh quick` green before each commit (its no-widget check
  included); `scripts/check.sh full` at 428575e: green in 19 min 57 s
  (1,197 s), its no-widget checks and the headless smoke plan included
  (the smoke plan's report now has `parts.csv` and "Frame cost by
  part").
- Disk: about 7.7 GB free after the warm-up (not 14); 5.6 GB after the
  density commit, so `target/debug/incremental` and this session's
  verification outputs went (11 GB).
- Manifests unchanged; no `Cargo.lock` change, no `cargo update`, no
  install, no push, no AI attribution.
- Decisions: D22.13 open; D22.12 gained its Followups; DECISIONS.md's
  index updated.
