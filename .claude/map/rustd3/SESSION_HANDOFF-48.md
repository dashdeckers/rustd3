# rustd3 — session handoff 48

2026-10-29 · **The D40.3 fixes are done** (a customer evaluation's eight items, before release) · every v0.1 phase is built · open: D40.4 (one recommendation to answer) · waiting: the user's benchmark rerun (Phase 31's exit criterion 6), whose outcome is appended to D22.1

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- D40.3 (`.claude/decisions/phase-40.md`), one commit per item, each with
  a test that fails without it:
  - 26a1586 `fix(figure)` item 1: collisions drop labels, not a time
    axis's whole row (a tick closer than its interval's step to the one
    before gives way alone; a stride never leaves k labels in a row out);
    `Issue::FewLabels`.
  - f318598 `feat(figure)` item 2: legend swatches from the keys' marks
    (`fits::Reported` replaces `Hatched`), insets stacked per corner,
    `Issue::LegendsOverlap`.
  - 0e4e360 `feat(figure)` item 3: ribbons are bands (an opaque color at
    0.25); `Area` and `Ribbon` gain `.color` and `.opacity`.
  - a6b461e `docs(rustd3)` item 4: the README (facade crate doc), figure's
    and core's crate docs map the component crates' names.
  - 48f4aff `feat(core)` item 5: `time::Locale::ladder`,
    `locale::Locale::with_time`.
  - ea7d91f `docs(core)` item 7: `fmt!` writes under `Locale::NEUTRAL`,
    no currency; the locale's `format` writes one.
  - d29bdc7 `fix(figure)` item 8: vertical titles read bottom to top on
    both sides.
  - de9d3e8 `fix(iced)` item 6: windowed tiny-skia renders at the
    window's factor (Xvfb screenshots at 1, 1.25, 1.5, 2); only
    `iced_test` capture tells the widget no factor, now documented on
    `Plot::scale_factor` and in the README; the seam came from `NUDGE`
    doubling to half a device pixel, and `NUDGE` is now 0.0625.
  - 67b54c6 `docs`: D40.4.
- Harness unchanged: 16 suites, 6,722 assertions, 5,292 passing, 1,430
  skipped, 0 failing, 179 floors. No snapshot changed.
- `scripts/check.sh full` green at 67b54c6 (21 min 15 s, warm).
- Tags: `m2-complete`, `m3-complete`, `m4-complete` (Phase 31 built).

## 3. Project management

No GitHub repository yet (`repository` unset; RELEASING.md item 2 sets
it and the README's git lines). The user's copy is
`~/Projects/rustd3` (a clone from `rustd3.bundle` by `setup.sh`).

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 | 1–3 | done |
| M1 | 4–10 | done (core contract; D11.2 review fixes) |
| M2 | 11–22 | done (benchmark verdict D22.1: build Phase 31) |
| M3 | 23–30 | done |
| Conditional | 31 | built; rerun pending |
| M4 | 32–39 | done |
| M5 | 40 | done; D40.3's fixes done |

## 5. What to do next

1. Answer D40.4. Its recommendation for (1) is C: of two time labels a
   month boundary (or a clock change) puts closer than the step, keep
   the one on the coarser rung ("2026-11", "November") and drop the
   month's last stepped day; as built (A) the later tick drops, which
   loses the axis's only year or month name. If C: one `fix(figure)`
   commit in `figure/src/axis/draw.rs` (`restarts` says which tick of the
   pair is on its parent unit's start) and `figure/src/layout/collide.rs`
   (`restart`), the fortnight test expecting 10-19 … 10-29 and "2026-11".
2. When the user's rerun returns: append to D22.1 as handoff 47 §5 says.
3. Release: the user follows `RELEASING.md`.

## 6. Notes for later phases (0.2)

- The Later table in rustd3.md and CHANGELOG.md, plus handoff 47's
  items, plus: a legend square that repeats a translucent mark's opacity
  (D40.4 (3)), and `Format` re-localizing builders if asked (D40.4 (4)).

## 7. Key files

- `.claude/decisions/phase-40.md` (D40.3, D40.4).
- `figure/src/layout/collide.rs`, `figure/src/axis/draw.rs`,
  `figure/src/legend.rs`, `figure/src/mark/area.rs`,
  `iced/src/plot.rs` (`NUDGE`), `iced/src/plot/slot.rs`.
- `iced/tests/main/plot.rs`
  (`spines_have_no_seam_whatever_factor_the_widget_assumes`),
  `iced/tests/main/placement.rs` (the pinned `NUDGE`).

## 8. Commands

```bash
scripts/check.sh quick | full | release
cargo run -p rustd3-harness -- --check
xvfb-run -a -s "-screen 0 2200x1700x24" …   # a windowed app, then
import -window root shot.png                # ICED_BACKEND=tiny-skia|wgpu, WINIT_X11_SCALE_FACTOR=k
```

## 9. End-of-session checklist

- `scripts/check.sh quick` before every commit.
- `scripts/check.sh full` at 67b54c6: green in 21 min 15 s (warm; iced tests on tiny-skia and wgpu, the benchmark smoke).
- Decisions recorded (D40.4 open); no AI attribution in commits.
