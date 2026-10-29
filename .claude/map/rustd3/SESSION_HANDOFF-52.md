# rustd3 — session handoff 52

2026-10-29 · **Phase 31h is done** (D22.8: the X11 no-GPU profile and the
plain-iced reproduction) · no decision open · next: the user runs
`dev/bench/repro-software.sh` in the COSMIC session, then the suite

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 31h on 7c07551, two commits (D22.8's followups in
`.claude/decisions/phase-22.md` say the same):

- 7243284 `feat(bench)` `weak-nogpu-x11`: on Linux with `WAYLAND_DISPLAY`
  and `DISPLAY` both set, `suite` runs the weak plan on tiny-skia again
  right after `weak-nogpu`, through `run --xwayland`, which removes
  `WAYLAND_DISPLAY` from the runs' environment and records `xwayland` in
  `meta.json`. The weak report's heading reads "Weak profile
  (report-only): no GPU (tiny-skia) on X11 through XWayland" with D22.8's
  reason, and its outcome line starts "report-only, X11 through XWayland
  (D22.8)". It joins the suite's summary list and `system.txt`'s notes;
  elsewhere the suite prints one "Skipping weak-nogpu-x11" line, and
  `run --xwayland` refuses to start. README, `--help` and `run.sh` give
  83 minutes (47 with `--focus`) on Wayland.
- 14ea223 `feat(bench)` `rustd3-bench repro-software [--out FILE]`
  (`dev/bench/src/repro.rs`, 164 lines, iced only) and
  `dev/bench/repro-software.sh`. The images move inside a `pin` with
  `ContentFit::None`, so the edge ones are cut by their layout box, not
  scaled. The backend is `ICED_BACKEND`, as `run` hands `--backend` to its
  children.

Tests: `the_x11_profile_runs_only_in_a_linux_wayland_session_with_xwayland`
(the decision over OS and the two variables, empty as unset);
`cli_parses_every_documented_flag` and `rejects_unknown_flags` cover
`--xwayland` and `repro-software`. Checked by hand under Xvfb: with
`WAYLAND_DISPLAY=wayland-bogus`, a `run --xwayland` child opened its
window and finished, while the same child without the flag panics in
winit (no compositor); the report showed the X11 heading. Harness
unchanged (no core or harness change).

## 3. Measured here (not rule outcomes)

`repro-software` under Xvfb (2 cores, lavapipe, no vsync: only frames and
CPU count), three runs each (direct runs and `repro-software.sh`):

| Backend | Frames in 10 s | Frame p50 / p90 / p99 ms | CPU s/s |
| --- | --- | --- | --- |
| tiny-skia | 89–91 | 110–113 / 123–129 / 126–152 | 0.97–0.98 |
| wgpu (lavapipe) | 650–673 | 14.6–15.1 / 17.7–18.8 / 22.9–24.6 | 1.62–1.67 |

tiny-skia is CPU-bound here; where its 110 ms go is not measured.
`repro-software.sh` with no `WAYLAND_DISPLAY` skips the Wayland rows;
with a bogus one it marks them failed and runs the X11 rows.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0–M3 | 1–30 | done |
| Conditional | 31 (a–h) | built; the user's runs pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's and D40.4's fixes done |

## 5. What to do next

1. **The user, in the COSMIC session:** `dev/bench/repro-software.sh`
   (about a minute after the build). Its table and
   `bench-results/repro-<stamp>/` answer D22.8: tiny-skia far below X11
   on Wayland, with wgpu paced on both, is the iced issue to file (with
   `src/repro.rs` as the program). If tiny-skia is slow on X11 there
   too, a variant with fewer images would separate drawing from
   presentation first.
2. **The user's suite run:** `dev/bench/run.sh --focus` now adds
   `weak-nogpu-x11/` on Wayland (about 47 minutes in all).
3. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

- Unchanged from handoff 51 (tiny-skia's per-image clip mask; the
  mosaic; `drag-resize`'s noise; `--refresh-hz` under Xvfb).
- `repro-software` measures the frame interval from the instants
  `window::frames()` delivers (iced stamps `RedrawRequested` when it
  starts handling it), so it reads how often redraws reach the app, not
  when frames are presented.

## 7. Key files

- `dev/bench/src/suite.rs` (`xwayland`, `xwayland_on`, `NO_XWAYLAND`),
  `dev/bench/src/main.rs` (`run`'s `--xwayland`, `repro_software`),
  `dev/bench/src/report.rs` (`XWAYLAND`, `rule_line`),
  `dev/bench/src/system.rs` (`Meta::xwayland`).
- `dev/bench/src/repro.rs`, `dev/bench/repro-software.sh`,
  `dev/bench/README.md` (run sheet; "The upstream reproduction").

## 8. Commands

```bash
scripts/check.sh quick rustd3-bench
dev/bench/repro-software.sh                         # the user's machine (Linux)
ICED_BACKEND=tiny-skia [env -u WAYLAND_DISPLAY] target/release/rustd3-bench repro-software
xvfb-run -a -s "-screen 0 1920x1080x24" env ICED_BACKEND=tiny-skia \
  target/release/rustd3-bench repro-software --out /tmp/…   # here
dev/bench/run.sh [--smoke-only | --focus]
```

## 9. End-of-session checklist

- `scripts/check.sh quick rustd3-bench` green before each commit (49
  tests); `check.sh full` not run (dev-only session, as briefed).
- Disk: 6.9 GB free at the start; 5.6 GB after the release build, so
  `target/debug/incremental` was removed once (→ 6.6 GB).
- No manifest change; no new binary; no install, push or AI attribution.
- Decisions: none open; D22.8 gained its Followups.
