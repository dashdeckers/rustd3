# Phase 9 — Recordings and data

## Measurements

2026-10-27, at 9e2d499. Phase 9 adds no harness suite: 8 suites, 3,303
assertions, 2,612 passing, 691 skipped, 79 floors, before and after.

Every prototype count of Spot checks holds against the raw-scan
reference (`core/tests/main/series/exact.rs`): the signal trace's full
view is one polyline of 3,949 / 7,791 / 15,429 points at 1000 / 2000 /
4000 columns with all 50 impacts kept; the 2,000-sample view gives
1,981 points; the gap trace has 19 runs and its full view 19 polylines
of 3,124 points; the index agrees with the scan in 51/51 and 41/41
views. The index holds 31,243 nodes, 749,832 bytes, at 1,000,000
samples (9.37 %).

Release timings, report only (`cargo test -p rustd3-core --release
--test main series -- --ignored`), 2-core container, best of 3 builds
and of 20 queries; build includes copying x and y into chunks, the index
and the runs:

| | 1M samples | 10M samples |
| --- | --- | --- |
| `Recording::new`, explicit clock | 20.3 ms | 161.8 ms |
| `Recording::new`, regular clock | 5.4 ms | 72.9 ms |
| `columns`, 1001 edges, explicit clock | 95 µs | 175 µs |
| full-view M4, 1000 columns | 0.22 ms (3,949 points) | 0.28 ms (4,000 points) |

The prototype measured 3.8–5.1 ms and 40–56 ms for its index alone, and
0.54–1.16 ms for a full-view M4 at 1M.

The model-based property test (`series/agree.rs`) runs 12 cases in
about 1.5 s in debug, and 300 under `RUSTD3_FUZZ=1`: 37 s in debug,
11 s in release.

## D9.1 — An append with the wrong number of channels has no error variant

**Date:** 2026-10-27 · **Context:** Phase 9, step 1 (9152a42) · **Status:** open

### Issue

Phase 9, Design, Recording: `extend` "appends to every channel at once,
or to none" and validates everything before touching storage, and
Error lists `Length { channel, expected, found }` for a channel whose
sample count differs. Nothing covers an append that names fewer or more
channels than the recording has (`rec.extend(&[], [&a])` on a recording
of two channels). `Length` cannot carry it: a missing channel has no
length to report beyond 0 (and an empty append would then pass), and an
extra channel has no expected count.

### Investigation

`new` takes its channel count from the iterator, so only `extend` meets
the case. The rest of the phase does not depend on which error it
returns.

### Options

| | **A — a `Channels` variant** | B — `Length` with `found: 0` | C — panic |
| --- | --- | --- | --- |
| Approach | `Error::Channels { expected, found }`, "expected {expected} channels, got {found}", checked first | A missing channel reports `Length { channel, expected, found: 0 }`; an extra one `Length { channel, expected: 0, found }` | `assert_eq!` on the count |
| Correctness | Says what is wrong | An empty append with a missing channel passes; "expected 0" misleads | A library panics on input |
| Architectural fit | One variant beside `Length`; Phase 34 adds `Recordings` the same way | No new variant | Against the error conventions |

### Decision

Recommended: A, as built (`core/src/series/error.rs`,
`Recording::append`); tested in `appends_name_every_channel_and_regular_clocks_take_no_x`.

### Why

The error carries both counts, and no append with a wrong channel count
can succeed.

### Followups

If B or C is chosen, the change is one variant and one check in
`recording.rs`.

## D9.2 — Statements of Phase 9 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 9, steps 1–6 (9152a42 … 9e2d499) · **Status:** open

### Issue

1. **Error's `where X::Period: fmt::Debug`** ("the one the compiler asks
   for"): `Coordinate` already bounds `type Period: Copy + PartialEq +
   fmt::Debug`, and `error.rs` compiles without the clause under
   thiserror 2.0.21, with `derive(PartialEq)` asking for nothing either.
2. **"Below four samples per column every sample is emitted"**: M4 emits
   `{a, argmin, argmax, b − 1}`, so a column of three samples drops its
   middle one unless it is the column's minimum or maximum. The spot
   check says so too: the 2,000-sample view keeps 1,979 + 2 = 1,981
   points.
3. **Regular `locate` "steps ±1 until x(i − 1) < value ≤ x(i)"**: with a
   period far below the start's ulp (start 1e9, period 1e−12), about
   60,000 consecutive x are equal and the estimate is off by as many
   steps.
4. **"Tests live in `core/tests/main/series.rs`"**: index levels compared
   bitwise (Steps 2 and 5) and `index_memory_stays_under_ten_percent_of_data`
   read crate-private state, and `Capacity` needs more than `u32::MAX`
   samples.
5. **`Index::len_bytes()` "crate-private, compiled in every build"**: its
   only caller would be a test, which is dead code outside tests.
6. **A generic `fn mark<'a, X, Y>(d: impl Data<'a, X, Y>)`** (Exit
   criterion 4) that calls `into_points()` fails with E0309 (`X` and `Y`
   may not live long enough): `Points<'a, X, Y>` can hold `Slices`,
   whose `&'a [X]` and `&'a [Y]` imply `X: 'a, Y: 'a`, and nothing in
   that signature implies them.
7. **Unstated**: the validity of a regular clock's start and of its
   appended x; what `Error::Stale` reports for columns of another
   recording; whether `Series::runs(samples)` yields whole or clipped
   runs; what `extremes` and `runs` do with a range past the end; the
   visibility of `CHUNK` and `BLOCK`.

### Investigation

Each item was checked by compiling (1, 6: `rustc` against the built
crate shows E0271 alone once the signature names `Points`) or by the
exactness suite (2: index and raw scan agree on 1,981).

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | No `where` clause | A redundant clause |
| (2) | M4 as its algorithm states; the sentence is read as "thinning fades out" | Emit every sample below four per column, which contradicts the algorithm and the 1,981 count |
| (3) | Gallop: doubling steps out from the estimate, then bisection; the same `partition_point`, O(log d) probes for an estimate off by d | ±1 steps, O(d) |
| (4) | Those tests are unit tests (`index.rs`, `channel.rs`, `recording.rs`; `Capacity` by setting `len` before an append); everything reachable through the public API is in `core/tests/main/series.rs` and its modules `series/exact.rs` and `series/agree.rs` | Not expressible without a test hook in the public API |
| (5) | `Index`'s `Debug` prints levels, nodes and `len_bytes`; `Chunks` and `Runs` print summaries too, so a recording's `Debug` never lists its samples | Dead code |
| (6) | `mark` returns `Points<'a, X, Y>` (a mark type holding `Points` does the same), and `Data`'s docs say a function that does not must spell out `X: 'a, Y: 'a` | Does not compile |
| (7) | A regular clock rejects an invalid start (`Invalid { index: 0 }`) and an append whose last x is not valid (start 1e308, period 1e308 overflows at sample 1); error indices count from the first sample ever appended; `Stale` carries the columns' recording and revision and the queried recording's revision (documented on the variant); `runs` clips each run to `samples`; `extremes`, `runs` clamp to the channel; `CHUNK` and `BLOCK` are crate-private | — |

### Decision

Recommended: A for all seven, as built.

### Why

Each written form either does not compile, is dead code, or contradicts
the algorithm the phase states and the counts it checks; the unstated
choices are the narrowest that keep every append validated before
storage and every query total.

### Followups

- Phase 15: marks that hold `Points` and appear in their constructors'
  return types get `X: 'a, Y: 'a` implied; a helper that only consumes
  `impl Data` spells the bounds out.
- Phase 27: `Coordinate for Timestamp` must make `is_valid` reject what
  `at` cannot represent; the regular-clock check of the last appended x
  relies on it.
