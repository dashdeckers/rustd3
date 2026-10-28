# Phase 32 — The use cases

## Measurements

2026-10-28, e5897c9 … bc93514 and the closing docs commit, one session.
`scripts/check.sh full` green at bc93514 in 9 min 56 s (warm, the
benchmark smoke included).

**Harness** unchanged, as exit criterion 5 asks (no core code changes):
16 suites, 6,722 assertions, 5,292 passing, 1,430 skipped, 0 failing,
179 floors.

**Lines.** `dev/examples/src` +593 (focus_context.rs 371,
update_pattern.rs 222, with doc comments), the two binaries 11 and 37,
`dev/examples/tests` +283, `figure/tests` +131. Code lines (non-blank,
not comments) after rustfmt, the prototypes formatted the same way
(`rustfmt --edition 2024` on copies of
`proto/rfc/glue/examples/{focus_context,update_pattern}.rs`):

| Count | Use case 1 | Prototype | Use case 2 | Prototype |
| --- | --- | --- | --- | --- |
| Part I's method for use case 1: tests excluded, imports, data and `main` included | 285 + 7 in the binary = 292 | **247** (Part I's figure, reproduced) | 165 + 27 in the binary = 192 | 160 |
| The section's method: also without imports, data generation and `main` | **239** | 202 | **136** | 131 (Part I: 133) |

Data generation is `App::default`'s trace loop (31 lines, as the
prototype's `new`) and `next_data` (18; the prototype's 15, rustfmt
breaks the LCG chain). Part I's 133 for use case 2 is 131 by this count;
the counting behind Part I's figure is not recorded, while its 247 for
use case 1 is reproduced exactly.

What differs. Use case 2: the derive removes the prototype's 11-line
`impl Interpolate for Bar`; the app gains `is_animating` and `bars`
(the tests' readers, 8 lines), `rows` (the band, used twice, so the
height is `bandwidth()` instead of 40) and `rgb`; the 1.5 s clock left
`subscription` for the binary (D32.2). Use case 1, +37: the Tracker in
each canvas's state and the `waiting` redraw request (D11.1 (4)), the
plot rectangle as the pointer origin, a `Show` that keeps a window
inside the data with its span, labels placed from core's `Label`
anchor and `dy` instead of hard-coded offsets, named constants, and
`new` delegating to `Default`.

**Tests** (exit criterion 2): `ICED_TEST_BACKEND=tiny-skia` and
`=wgpu` (lavapipe) `cargo test -p rustd3-examples --all-features` each
pass all six. Exit criterion 3: with the `if !brush.is_active()` guard
removed by hand, `brushing_the_overview_sets_the_window` fails with the
window [0 h, 719 h] (`Show(full)` last), and so does the band drag;
restored (1995eac's body). Exit criterion 4: `cargo test -p
rustd3-figure --test main -- locale` passes the six `locale::` tests
(the five of Step 4 and the format rows) and the three older
`figure::` locale tests.

**Spot checks.** Every row is asserted except one: the wheel window
(205 and 635 of 860 px, ±1 s; span 359.5 h ±1 s), the brush (260 and
480 px, ±1 min), the band moved by 86 px (71.9 h, span kept, ±1 s), the
two data sets' widths and the five rows to three decimals, the last
bar settling at 830 ms (still moving at 829), the de-DE labels and
formats; the guard row by hand (above); the core trace is Phase 30's
`anim::use_case_2_lifecycle`. The bandwidth 44.118 is used by the
chart but not asserted (Phase 7's band tests cover it). Every expected
de-DE label and format was printed by d3-format 3.1.2, d3-scale 4.0.2
and d3-time-format 4.1.0 in Node through the resolve hook.

**Viewed** (headless screenshots from a scratch test kept out of the
repository, `UserInterface` and `Headless::screenshot` through
`iced_test`'s re-exports, factors 1 and 2, both backends): use case 1
full, mid-brush and brushed, zoomed, the band moved, the focus panned;
the regression's frames with the guard removed (mid-gesture right, the
release back to all 30 days); use case 2 at 0, 300 and 830 ms, then
new data at 1500 ms seen at 1800, 2100 and 2330 ms.

**"Decisions this phase may meet"**: neither. `Ticks::labels` for a
time scale needs no zone from the axis (`time::Time` carries its own),
and no use-case test failed because of core behavior.

## D32.1 — The time-axis locale test has no figure time handle to use

**Date:** 2026-10-28 · **Context:** Phase 32, step 4 · **Status:** open

### Issue

Step 4 asks for `time_axes_keep_the_iso_ladder_under_de_de` in
`figure/tests/main/locale.rs` (rustd3.md:13995–13998), a file that "uses
the public figure API only" with domains set through `fig.scale(…)`
(13920–13924). A figure handle is a `Kind` (`Figure::scale<S: Kind>`,
`figure/src/figure.rs:109`), and at Phase 32 only linear-family,
ordinal, band and point scales implement it
(`figure/src/scale/{continuous,ordinal,band,point}.rs`): `time::Time`
becomes a handle in Phase 33 (rustd3.md:14066–14069, its module list
`figure/src/scale/time.rs` at 14121; D28.1's investigation says the same).
`fig.scale(time::time(zone)…)` does not compile (E0277) before then.

### Investigation

The claim the test checks has two halves: the time half of
`Locale::DE_DE` keeps the ISO ladder (core, Phase 28), and a figure
applies its locale to the time formats it creates (figure, Phase 33).
The first can be checked now through core's axis, which labels through
`Ticks::labels(&set, &locale)` exactly as a figure axis does; the
second cannot until a time axis can be drawn by a figure. Nothing else
in Phase 32 depends on the answer.

### Options

| | **A — core's axis now, the figure in Phase 33** | B — defer the test | C — the time handle now |
| --- | --- | --- | --- |
| Approach | The test, in `locale.rs` as placed, labels `time::time(Europe/Berlin)` through `rustd3_core::axis::bottom(&x).ticks(8).locale(…)` under `Locale::default()` and `Locale::DE_DE`; Phase 33 rewrites it on `fig.scale(time::time(…))` | Four tests now; Phase 33 adds the fifth | Pull `Kind`, `Position` and `Fittable` for `time::Time` forward from Phase 33 |
| Cost | none; one test to rewrite in Phase 33 | none | Phase 33's step, out of order |
| Correctness | checks the labels and the interval (`Hour(3)`), not the figure's plumbing | checks nothing about time now | full |
| Fit | exit criterion 4's command finds five de-DE tests (plus the format rows, below) | criterion 4 short by one | Phase 33's design taken early |

### Decision

Recommended: A, as built. Phase 33's time-handle step replaces the core
axis in `time_axes_keep_the_iso_ladder_under_de_de` with a figure panel
on `fig.scale(time::time(berlin).domain(day))` and `axis::bottom(x)
.ticks(8)`, reading the labels from the glyph runs like the other four.

### Why

It asserts now everything Phase 32 can assert about time labels under
DE_DE without building Phase 33's handle out of order, and leaves the
figure half to the phase that makes it possible.

### Followups

- Phase 33: rewrite the test on a figure time handle (above).

## D32.2 — iced's `time::every` is not available to the use case 2 app

**Date:** 2026-10-28 · **Context:** Phase 32, step 3 (35a324a) · **Status:** open

### Issue

Part I's use case 2 subscribes to `time::every(every).map(Message::NewData)`
(rustd3.md:2274–2285); Phase 32 lifts it and lists `App::subscription`
as "frames only while animating" (13895), with "only
`examples/update_pattern.rs` calls `Instant::now()`" (13779–13781). At
iced 3cbdacc, `iced::time::every` exists only with iced's `tokio` or
`smol` feature (`src/time.rs:4–9` re-exports
`iced_futures::backend::default::time::*`; the `thread-pool` backend,
the default (`Cargo.toml:25`), has an empty `time` module,
`futures/src/backend/native/thread_pool.rs`). The workspace's `iced`
dependency has the default features plus `advanced` and `canvas`; tokio
is in `Cargo.lock` only as a wasm dependency. Enabling `tokio` or `smol`
is a manifest change, which this phase may not make.

### Investigation

`window::frames()` is `event::listen_raw` over `RedrawRequested`
(`runtime/src/window.rs:196–201`): it yields only when the window
redraws, so it cannot be the 1.5 s clock without redrawing continuously.
`Subscription::run_with(data, fn(&D) -> Stream)`
(`futures/src/subscription.rs:219–229`) takes any stream, such as the
receiver of a `futures` channel fed by a thread.

### Options

| | **A — the clock in the binary** | B — iced's `smol` (or `tokio`) feature | C — the clock in the library |
| --- | --- | --- | --- |
| Approach | `App::subscription` is the frames while animating; `examples/update_pattern.rs` batches it with its own `every(period)`, a thread sending `Instant::now()` into a channel behind `Subscription::run_with` | `iced = { …, features = [ …, "smol"] }` in `dev/examples`, and Part I's code as written | A's `every` in `src/update_pattern.rs`, inside `App::subscription` as Part I shows it |
| Cost | 15 lines in the binary (the function and its comment) | a manifest decision; the lockfile gains smol's tree | the same 15 lines in the library |
| Correctness | the same ticks; the library reads no clock, tests unaffected | iced's own timer | the library calls `Instant::now()` |
| Fit | keeps 13779–13781 and 13895 literally; Part I's `subscription` body differs | Part I verbatim | Part I's shape, against 13779–13781 |

### Decision

Recommended: A, as built (35a324a).

### Why

It needs no dependency change, keeps the app library free of clocks as
the section asks, and keeps `App::subscription` what the section's
signature says. B is the better end state if the architect accepts a
dev-only feature for iced's own timer; the change would then move the
clock back into `App::subscription` as Part I writes it.

### Followups

- If B: one `build(examples)` commit adding the feature, Part I's
  subscription in `src/update_pattern.rs`, the thread removed.

## D32.3 — Details of Phase 32 as built

**Date:** 2026-10-28 · **Context:** Phase 32, steps 1–4 (e5897c9, 1995eac, 35a324a, bc93514) · **Status:** open

### Issue

1. **The canvases and their states.** Part I names the focus canvas's
   state `Focus { zoom, tracker }`; the section's module list has
   `Focus` and `Overview`; a `canvas::Program` needs `&App` in `self`
   and its own `State: Default`. `Zoom::default()` has no scale extent
   [1, 50], and `BrushX` has no `Default`.
2. **`update` "clamps to `full`".** The prototype intersects the window
   with `full`: a pan against an end shrinks the window, and a window
   wholly past an end inverts (`k < 0` at the next event); a zero-width
   `Brush` event (a press, then a move back to it) gives a window of no
   span (`k = ∞`).
3. **Redraw requests.** Part I: "while `tracker.wants_redraw()` is true
   the app requests that redraw"; the section's code returns `None` when
   the zoom or brush has nothing.
4. **Label placement.** The prototype offsets labels by hard-coded
   pixels; core's `Label` gives the anchor and d3's `dy` in em. The
   focus's top tick label sat half above the canvas.
5. **`App::new()`** without `Default` fails clippy's
   `new_without_default` under `-D warnings`.
6. **Readers for the tests.** `marks.is_animating(ms(830))` is asserted
   in Step 3, but `marks` is private and the API lists only `bars()`.
7. **The bar height** in `draw` needs a band; the app keeps none.
8. **Window sizes and test details.** Use case 2's window and the
   no-GPU skip are not given; the tests' base instant needs a clock.

### Options

| | **A — as built** |
| --- | --- |
| (1) | Private programs `Focus<'a>` and `Overview<'a>` holding `&App`, with states `(Option<Zoom>, Tracker)` and `(Option<BrushX>, Tracker)`: each behavior made at its canvas's first event (the zoom with `scale_extent([1, 50])`, the brush with the extent from the plot's range), no new types |
| (2) | `Show([a, b])` keeps the span (cut to the full span when longer) and moves the window inside `full`; a span that is not positive is ignored. Every spot row stays inside, so none changes |
| (3) | Both canvases return `Action::request_redraw()` when they publish nothing while a press waits (D11.1 (4)), so the focus's double click works |
| (4) | Labels centred at `y + (dy − 0.32) · 11` px (d3's `dy` for a vertically centred label is 0.32 em), aligned by the anchor; the y ranges keep 8 px above the top tick |
| (5) | `impl Default for App` builds the data; `new()` returns it |
| (6) | `App::is_animating()` (the same test `subscription` uses); `bars()` samples at the stored instant, so the tests move it with `Frame(ms(n))` |
| (7) | `rows(names)`, the band of `update`, over the present names in `draw` |
| (8) | 600 × 260 for use case 2; `simulator(size, view)` in `tests/main/main.rs` returns `None` with a message for wgpu without a device under `RUSTD3_ALLOW_NO_GPU=1`; test base instants from `Instant::now()` (the dev crates allow it); extra assertions: still animating at 829 ms, hen between its rows at 1900 ms |

### Decision

Recommended: A for all eight, as built.

### Why

(1), (5) and (7) are the smallest forms the pinned iced and the
workspace lints accept; (2) and (3) avoid an inverted or infinite
transform and a lost double click without changing a spot row; (4)
draws the labels where d3 would; (6) and (8) are test plumbing.

### Followups

- Phase 33: D32.1's rewrite of the time-axis test.
- Phase 36: the figure-level focus and context (a `Plot` with a brush
  panel) can compare against these apps' windows.
