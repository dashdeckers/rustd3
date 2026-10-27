# rustd3 — session handoff 09

2026-10-27 · Phase 9 complete in one session (D9.1, D9.2 open) · next: **Phase 10 — Pointer, zoom and pixel-space operations**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 9 is complete; `scripts/check.sh quick` passes on every commit and
`full` on the last (33 s warm; no cold build this session).

- `rustd3-core`, new module `series` (`core/src/series.rs` and
  `core/src/series/`):
  - `Coordinate` (impl for `f64`: `at`, `index` through `js::to_int`,
    `is_valid`, `is_valid_period`) and `Clock::{Regular, Explicit}`.
  - `Recording<X>`: `new`, `extend` (all channels or none, validated
    before storing), `drop_front` (whole chunks, rebuilds index and
    runs), `series`, `len`, `is_empty`, `channels`, `id`, `revision`,
    `offset`, `x`, `locate` (regular: gallop from `Coordinate::index`;
    explicit: binary search over chunks), `x_extent`, `columns`. `Id`
    from an `AtomicU64`; `Clone` is deep with a fresh id.
  - `Error<X = f64>`: the nine variants of the spec plus `Channels`
    (D9.1).
  - `Series<'a, X>` (`Copy`): `recording`, `channel`, `len`, `is_empty`,
    `get`, `iter`, `extent`, `extremes(samples, valid)`, `runs(samples)`
    (clipped), `m4(columns, valid)`.
  - `Columns` (`len`, `edges`, `samples`), `m4::Extremes` (`min`,
    `max`), `m4::Lines` (`len`, `is_empty`, `points`, `iter`).
  - `Data<'a, X, Y>` with the three impls, `Points<'a, X, Y>` (`len`,
    `is_empty`, `get`, `iter`, `series`), `Slices`, `slices()`.
  - Crate-private storage: `chunks::Chunks<T>` (65,536 per chunk, full
    capacity, `slice`, `pieces`, `partition_point`), `channel::Channel`,
    `index::Index` (levels of `min`, `max`, `at_min`, `at_max`; complete
    nodes only; the predicate-descending query), `runs::Runs`.
- Tests: core 45 unit (16 in `series`), 330 in `main` (36 in
  `series.rs`, `series/exact.rs`, `series/agree.rs`; one ignored timing
  report), 44 doctests (6 in `series`, one `compile_fail` E0271 with its
  twin). The raw-scan reference is `m4_scan` in `series.rs`.
- Prototype counts all hold (3,949 / 7,791 / 15,429; 1,981; 19 runs and
  3,124 points; 51/51 and 41/41 views). Release timings are in
  `.claude/decisions/phase-09.md`.

Harness: unchanged (Phase 9 owns no d3 tests): 8 suites, 3,303
assertions, 2,612 passing, 691 skipped; 79 floors.

Decisions (`.claude/decisions/phase-09.md`):

- **D9.1** (open, recommending what is built) — `extend` with the wrong
  number of channels returns `Error::Channels { expected, found }`.
- **D9.2** (open, recommending what is built) — (1) no `where X::Period:
  Debug` on `Error`; (2) M4 below four samples per column follows its
  algorithm (1,981 points); (3) regular `locate` gallops; (4) index-level,
  memory and capacity tests are unit tests; (5) `len_bytes` feeds
  `Index`'s `Debug` summary; (6) a generic `mark` names `Points<'a, X,
  Y>` in its signature, or spells out `X: 'a, Y: 'a` (E0309);
  (7) unstated validation (regular start, last appended x), `Stale`'s
  fields, clipped `runs`, clamped ranges, private `CHUNK`/`BLOCK`.

Commits: 9152a42 (storage, clock, recording), 3388a67 (index), cb105b8
(extremes, columns, m4), 175b64f (exactness), 79c530d (appends, drops,
clocks, property test), 9e2d499 (Data, Points, slices), the decisions
and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | 9 done (D9.1, D9.2 open); 10 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 10: read `rustd3.md`'s front matter and "### Phase 10 —" (it builds
on Phases 5 and 6), and section 6 below.

## 6. Notes for later phases

- Phase 15 (thinning in marks): `rec.columns(&edges)` once per recording
  and frame, then `series.m4(&columns, valid)` per channel; `Lines`
  yields sample indices per NaN-free run, outer neighbours included.
  `Points::series()` recovers the indexed view from mark data. A mark
  constructor `fn line<'a, X, Y>(data: impl Data<'a, X, Y>, …) -> Line<'a,
  X, Y>` gets `X: 'a, Y: 'a` implied by its return type; a helper that
  takes `impl Data` and returns something else must spell them out
  (D9.2 (6)).
- Phases 17 and 21 (`Fit::Visible`, `Fit::Window`): `Series::extremes`
  over `locate`d sample ranges, with the y scale's validity predicate;
  `Recording::x_extent` for the x domain.
- Phase 27 (`Recording<Timestamp>`): implement `Coordinate` in
  `series/clock/timestamp.rs`; `is_valid` must reject what `at` cannot
  represent, because a regular clock checks only the last appended x
  (D9.2 followups). Error indices are absolute (`offset` + position).
- Phase 34 (`m4_joint`, `Error::Recordings`): `Lines::thin(runs, edges,
  extremes)` is crate-private and takes the runs and an extremes
  closure, so a joint query can pass intersected runs.
- The property test `fuzzed_recordings_agree_with_a_raw_scan_of_their_samples`
  runs 300 cases under `RUSTD3_FUZZ=1` (picked up by `check.sh`'s
  release `fuzz` filter).

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-09.md`; `core/src/series.rs`,
`core/src/series/*.rs`; `core/tests/main/series.rs`,
`core/tests/main/series/{exact,agree}.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core series
cargo test -p rustd3-core --release --test main series -- --ignored --nocapture   # timings
RUSTD3_FUZZ=1 cargo test -p rustd3-core --release --test main fuzz
cargo run -p rustd3-harness -- --check
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (33 s).
- D9.1 and D9.2 open with their recommendations.
- Conventional commits, no attribution.
