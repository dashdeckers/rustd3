# rustd3 — session handoff 47

2026-10-28 · **Phase 40 — Facade, documentation and release preparation**: complete · **every v0.1 phase is built** · waiting: the user's benchmark rerun (Phase 31's exit criterion 6), whose outcome is appended to D22.1

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

- Phase 40's session (526f512 … 4cd593a) was cut off after its
  decision file; the architect ran `scripts/check.sh full` at 4cd593a
  (green, 27 min 30 s, warm), answered D40.1 and D40.2 (A), and wrote
  this handoff.
  - 526f512 `fix(iced)`: `Caching::Verify`'s checks do not feed the cost
    model (D31.10).
  - 71442c3 `build(rustd3)`: publishing metadata, anchored `include`
    lists (D40.1), licence links, `CHANGELOG.md`.
  - f0f3f0d `feat(rustd3)`: the facade re-exports core's modules, `fmt!`,
    `jiff`, the `interpolate` module, and behind `figure` rustd3-figure
    (`pdf`, `gif` under their features).
  - b214358 `docs(rustd3)`: the README (also the facade's crate doc),
    package READMEs, compiled README examples (`readme_canvas`,
    `readme_plot` in rustd3-examples, held equal to the README by
    `tests/main/readme.rs`).
  - 8ae8622, 7a03794 `build(rustd3)`: `scripts/check.sh release`, the
    dry run (`cargo publish --dry-run --workspace --exclude rustd3-iced`)
    with package-list and lockfile checks.
  - 26c908d `docs(rustd3)`: `RELEASING.md` and the MSRV note (1.97; the
    dependency floors are 1.92).
- Harness: 16 suites, 6,722 assertions, 5,292 passing, 1,430 skipped,
  0 failing, 179 floors.
- Decisions: all decided; D22.1 waits only for the rerun's appendix.
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
| M5 | 40 | done |

## 5. What to do next

1. When the user's rerun returns (`bench-results/<stamp>/` on the user's
   machine: smoke, default, weak-nogpu, weak-4cores, weak-2cores):
   append to D22.1 the default plan's outcome under the unchanged rule
   and each weak profile's outcome under D22.4's budgets, with the
   attribution of any miss; write the closing line. A miss in a weak
   profile becomes a recorded 0.2 item (Phase 31 is closed); a desktop
   pan or zoom miss still attributed to raster or upload is the "GPU
   data layer recommended" outcome, which changes the README's
   performance paragraph and the CHANGELOG's Later list.
2. Release: the user follows `RELEASING.md` (the agent never runs
   `cargo publish` without `--dry-run`).

## 6. Notes for later phases (0.2)

- The Later table in rustd3.md and CHANGELOG.md, plus: building the
  `Plot` widget's drawing list off the UI thread (D22.1, D22.4 item 4:
  `scatter1m`'s list and hit index), `Line::key`/`Area::keys` builders
  (D39.4), the two-point test for non-lead handles (handoff 46).

## 7. Key files

- `README.md`, `CHANGELOG.md`, `RELEASING.md` (repository root).
- `.claude/decisions/phase-22.md` (the verdict and the rerun's setup),
  `phase-31.md`, `phase-40.md`.
- `dev/bench/README.md`, `dev/bench/run.sh`.

## 8. Commands

```bash
scripts/check.sh quick | full | release
cargo run -p rustd3-harness -- --check
dev/bench/run.sh            # the user's machine only (windowed)
```

## 9. End-of-session checklist

- `scripts/check.sh full` green at 4cd593a; the tree clean after this
  commit.
- Decisions recorded; no AI attribution in commits.
