# rustd3 — session handoff 11

2026-10-27 · Phase 11 complete in one session (D11.1 open) · next: **Phase 12 — Drawing list and raster outputs**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 11 is complete; `scripts/check.sh quick` passes on every commit and
`full` on the last code commit and on this handoff (46 s warm).

- iced is pinned at `3cbdacc3b0841c5145c766c3a758657d36b41ffe`
  (`iced_core`, `iced_widget`, `iced`, `iced_test`, `iced_runtime` in
  `[workspace.dependencies]`, plus `rustd3-iced = { path = "iced" }`).
  `Cargo.lock`: 327 → 613 packages, no version changed. rustd3-iced's
  normal graph has no winit, iced_wgpu or iced_tiny_skia (`cargo tree -p
  rustd3-iced -e normal -i …` prints nothing). `dev/examples` and
  `dev/bench` have their iced dependencies and no iced code yet.
- `rustd3-iced` (`iced/src/`):
  - `IcedPath` (`path.rs`): `PathContext` over
    `canvas::path::Builder`, private `builder`, `current`, `start`; arcs
    cut into cubics of at most π/2 with d3's semantics (negative radius
    draws nothing; full circle back to the start); `path(draw) ->
    canvas::Path`.
  - `pointer::Tracker` (`pointer.rs`): iced events → `interact::Pointer`
    in the area's coordinates; `new`/`Default`, `scale_factor`,
    `translate(event, bounds, cursor)`, `wants_redraw`. Double clicks by
    iced's rule from presses stamped by the next `RedrawRequested`.
- Tests (`iced/tests/main/`): 32 integration tests and 4 doctests.
  `headless.rs` is the harness every later iced test uses: `Harness::new`
  (`ICED_TEST_BACKEND`, no fallback, `RUSTD3_ALLOW_NO_GPU=1` → `None`),
  `with_backend`, `point_at`, `events`, `tick`, `frame(now)`,
  `redraw_requested`, `waker`/`wakes`, `messages`, `backend`; `Shot`
  with `image()`/`pixel()`; `features()`; `epoch()`. `path.rs` (8),
  `pointer.rs` (15), `canvas.rs` (4), `placement.rs` (5, with
  `BAND_BYTES` and `NUDGE`).
- `scripts/check.sh full` runs the tests of rustd3-iced, rustd3-examples
  and rustd3-bench once per `ICED_TEST_BACKEND` in `tiny-skia wgpu`, with
  `--nocapture` so the placement lines print.

Measured (`.claude/decisions/phase-11.md`; lavapipe for wgpu; `crisp` and
`web-colors` on): placement max 0/255 on both backends at k = 1, 1.25,
1.5, 2; a bias of 0 lands a pixel early on tiny-skia at 14/111 offsets
(k = 1.25) and 36/111 (k = 1.5), 0.5 a pixel late on wgpu everywhere,
0.25 exact everywhere; images of 2,097,152 B and more miss their first
wgpu frame and show after one tick and frame, 2,096,704 B shows at once.
Build: cold iced check 49 s; the first `check.sh quick` after the
manifest change 381 s; `target` 4.9 → 9.8 GB.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors.

Decisions (`.claude/decisions/phase-11.md`):

- **D11.1** (open, recommending what is built) — nine items: (1) the
  harness takes `impl iced::Widget<Message> + 'a` (no `Into<Element>`
  at 3cbdacc); (2) `ui` is a plain field, not an `Option`; (3) extra
  harness helpers; (4) `canvas::Action` cannot publish and request a
  redraw together, so a program requests it when publishing nothing;
  (5) one waiting press; (6) Tracker details (Left during drags, second
  buttons, triple clicks, invalid factors, iced's rule edges); (7) the
  offset sweep, the 100.2 row and an exactly-`BAND_BYTES` row added;
  (8) images clipped to their own biased bounds; (9) what changed in
  `Cargo.lock`, the `figure` feature's form, the workspace entry.

Commits: a872bd6 (pin), 6e8c423 (IcedPath), d1279ae (pointer),
3dff65e (harness, canvas), 7b3d161 (Tracker redraw doc), 0c8f9fd
(placement), 542c74c (check.sh), 68b1c93 (decisions), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done |
| M2 Figure layer and widget | 11–22 | 11 done (D11.1 open); 12 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 12 (two sessions, cut after Step 4): read `rustd3.md`'s front
matter and "### Phase 12 —"; it builds on Phases 1 and 7 and starts
`rustd3-figure`. No iced work until Phase 18.

## 6. Notes for later phases

- Phase 18 (`Plot`): place images at `(device + NUDGE) / k` with
  `NUDGE = 0.25` and size `device / k`, clipped to their own rect (the
  biased bounds measured exact on both backends; `device / k` should be
  too, by tiny-skia's non-anti-aliased mask and wgpu's rounding, but was
  not measured); band
  at or above `BAND_BYTES` (exactly 2 MiB already misses its first wgpu
  frame). Both constants live in `iced/tests/main/placement.rs`; the
  widget declares its own. At 3cbdacc `Layout` has no lifetime and
  `Widget::layout` writes `tree.size` (see `placement.rs`'s `Images`).
  The harness may need an `Option` around its `UserInterface` once a
  test rebuilds the view (D11.1 (2)).
- Phases 18–21: the iced pixel bounds against the PNG stay 1/255
  (tiny-skia) and 2/255 (wgpu); placement measured 0/255 on lavapipe.
  Widgets publish and request redraws through their `Shell`, so D11.1
  (4) concerns only `canvas::Program` apps.
- The root `clippy.toml` covers `iced/tests`: a test that needs an
  `Instant` uses `headless::epoch()`.
- `wildcard_enum_match_arm` passes nested wildcards such as
  `Event::Keyboard(_)` after a specific `Event::Keyboard(…)` arm (see
  `Tracker::translate`).
- Phase 32 (use cases): `canvas::Program::State` holds a `Tracker`; a
  program feeds its `Pointer`s to `Zoom` and requests a redraw while
  `wants_redraw()` and it publishes nothing.

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-11.md`; `iced/src/{lib,path,pointer}.rs`;
`iced/tests/main/{headless,path,pointer,canvas,placement}.rs`;
`scripts/check.sh`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo tree -p rustd3-iced -e normal -i winit     # and -i iced_wgpu, -i iced_tiny_skia: nothing
cargo check -p rustd3-iced --all-features
ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-iced
ICED_TEST_BACKEND=wgpu cargo test -p rustd3-iced -- --nocapture placement
cargo run -p rustd3-harness -- --check
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (46 s).
- D11.1 open with its recommendation.
- Conventional commits, no attribution.
