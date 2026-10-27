# Phase 11 — rustd3-iced core level and image placement tests

## Measurements

2026-10-27, at 542c74c. Phase 11 does no core work: the harness stays at
9 suites, 3,314 assertions, 2,623 passing, 691 skipped, 80 floors.

**Build.** The iced checkout at 3cbdacc was already in `~/.cargo/git`;
`cargo tree` downloaded the crates. `Cargo.lock` grows from 327 to 613
packages (+286) and no locked version changes (see D11.1 (9)).

| Run | Wall | `target` after |
| --- | --- | --- |
| `cargo check -p rustd3-iced --all-features`, cold for iced (exit criterion 2) | 49 s | — |
| `scripts/check.sh quick`, all ten packages after the manifest change (clippy and test builds with iced, wgpu, winit) | 381 s | 8.5 GB (4.9 GB before) |
| `scripts/check.sh full`, warm, at the end of the phase | 46 s | 9.8 GB |

**Environment.** wgpu runs on Mesa lavapipe (llvmpipe, LLVM 20.1.2,
256 bits; Vulkan 1.4.318). The dev-dependency `iced` has its default
features: `crisp` on (`renderer::CRISP`), `web-colors` on
(`graphics::color::GAMMA_CORRECTION` false). Every number below is for
that state; the other states were not measured.

**Placement at `NUDGE = 0.25`** (`images_land_on_device_pixels_at_every_factor`:
four 300 × 200 checkerboards of 1-px cells in two mid-tone colors inside a
1-px red border, at device (125, 75), (438, 75), (125, 286), (438, 286);
nearest filtering; each image clipped to its own bounds). Max channel
difference against the source pixels; every red border on its expected
device columns, background just outside every edge:

| Backend | k = 1 | k = 1.25 | k = 1.5 | k = 2 |
| --- | --- | --- | --- | --- |
| tiny-skia | 0/255 | 0/255 | 0/255 | 0/255 |
| wgpu (lavapipe) | 0/255 | 0/255 | 0/255 | 0/255 |

These are the evidence behind the bounds Phases 18–21 assert for the
widget against the PNG (opaque ≤ 1/255 on tiny-skia, ≤ 2/255 on wgpu):
placement itself adds nothing. The canvas spot check (a 2-px d3 line at
scale 2) differs between the backends by 0/255.

**The bias.** 111 images of 4 × 4 distinct texels at device offsets
d = 7i + 1 (1 … 771) on both axes, at each factor, on both backends,
clipped once to their own bounds and once to the widget's (a scratch
test, not committed; texel (1, 1) located in the screenshot):

| Bias | tiny-skia | wgpu |
| --- | --- | --- |
| 0 | a pixel early (up and left) at 14/111 offsets at k = 1.25 (57, 99, 183, 218, …) and 36/111 at k = 1.5 (43, 50, 71, 92, …); exact at 1 and 2 | exact everywhere |
| 0.25 | exact everywhere, either clip | exact everywhere, either clip |
| 0.5 | the image lands right, but its own-bounds clip (a non-anti-aliased mask from d + 0.5, `tiny_skia/src/engine.rs:776-790`) drops its first row and column; exact with the widget's clip | a pixel late everywhere, either clip |

This is the section's analysis: truncation lands early when f32
round-off falls below the integer, and `nudge::round` (`wgpu/src/image/mod.rs:238-239`)
rounds d + 0.501 up. Lavapipe behaves as the section expects of a GPU:
wgpu's placement is decided on the CPU by `nudge::round`, and the quad
then covers whole device pixels that nearest sampling fills exactly. The
committed `every_device_offset_lands_exactly_at_every_factor` repeats
the sweep with own-bounds clips; with `NUDGE` set to 0 it fails on
tiny-skia, with 0.5 on both backends (checked, then restored). The 2 × 2
grid alone passes with a bias of 0.

**`BAND_BYTES`.** First frame that draws the image, at k = 1:

| Image | Bytes | tiny-skia | wgpu |
| --- | --- | --- | --- |
| 1024 × 511 (scratch) | 2,093,056 | shown | shown |
| 1023 × 512 (scratch) | 2,095,104 | shown | shown |
| 724 × 724 | 2,096,704 | shown | shown |
| 1024 × 512 | 2,097,152 = `BAND_BYTES` | shown | absent; shown after one tick and frame |
| 1000 × 600 | 2,400,000 | shown | absent; shown after one tick and frame |
| 2048 × 1024 (scratch) | 8,388,608 | shown | absent; shown after one tick and frame |

wgpu uploads synchronously only below 2 MiB (`image.len() < MAX_SYNC_SIZE`,
`wgpu/src/image/cache.rs:223-246`), so Phase 18's rule, bands for every
cell image at or above `BAND_BYTES`, is exact at the boundary. "After one
tick and frame" follows a 10 ms sleep: the worker uploads on its own
thread, and the test polls (sleep, tick, frame) up to 200 times.

## D11.1 — Statements of Phase 11 that do not hold as written, or leave a detail open

**Date:** 2026-10-27 · **Context:** Phase 11, steps 1–6 (a872bd6 … 542c74c) · **Status:** decided

### Issue

1. **The harness's root.** Design, headless harness:
   `Harness::new(root: impl Into<iced::Element<'a, Message>>, …)`. At
   3cbdacc no widget converts into an `Element` (E0277 for `Canvas` and
   `Container`): `Element::new` is `pub(super)`
   (`core/src/widget/element.rs:22`), widgets box through
   `Widget::_boxed`, and `UserInterface::build` takes `root: impl
   Widget<Message, Theme, Renderer> + 'a` (`runtime/src/user_interface.rs:99-104`).
   `Message: 'a` is needed too (E0309).
2. **`ui: Option<UserInterface<…>>`.** Nothing in this phase takes the
   interface out: `UserInterface::update` lays out again by itself after
   a layout invalidation (`user_interface.rs:344-358`), so the `Option`
   would always be `Some`.
3. **The harness's API** has more than the section lists:
   `with_backend(root, size, scale, backend)` (the canvas spot row's
   "backend diff ≤ 64/255" needs both renderers in one test),
   `Shot::image`, `Shot::pixel`, `features()` (the `crisp`/`web-colors`
   line every recorded number states) and `epoch()` (the one
   `Instant::now`, under `#[expect(clippy::disallowed_methods)]`, since
   the root `clippy.toml` covers `iced/tests`).
4. **Publishing and redrawing.** `wants_redraw`'s doc says "return
   `Action::request_redraw()` while this is true". `canvas::Action` at
   3cbdacc cannot both publish a message and request the next frame
   (`widget/src/action.rs`: `publish`, `request_redraw`,
   `request_redraw_at`, `and_capture`, `and_request_redraw_at`), and a
   press always yields `Pressed`.
5. **One waiting press.** The struct holds "a press awaiting its
   timestamp". A second press before the next `RedrawRequested` takes
   its place, so two presses within one frame never make a double click;
   iced's `Click` stamps `Instant::now()` as it processes each and would
   count them.
6. **Tracker details the table leaves open.** (a) `CursorLeft` while a
   button is held; a drag that ends outside the area. (b) A press of a
   second button while one is held. (c) A third quick press. (d) A
   scale factor that is not finite and positive. (e) The edges of iced's
   rule.
7. **Placement tests beyond the three listed.** The 2 × 2 grid passes
   with a bias of 0 on both backends, so alone it does not pin `NUDGE`;
   the section's tests do not reach an image of exactly `BAND_BYTES`,
   the boundary Phase 18's "at or above" depends on.
8. **The image clip.** "`draw_image(image, bounds, clip_bounds)`" and
   Part I's "clipped to its own rect" leave open whether the clip is the
   biased bounds or `device / k`.
9. **"The build adds the iced packages to `Cargo.lock` and nothing else
   changes in it."** No locked version changes, but three existing
   entries (`cc`, `half`, `num-traits`) gain dependency edges from
   features iced's graph enables (`jobserver`/`libc`, `num-traits`,
   `libm`), and 47 dependency lines gain a version suffix because 16
   crates now come in more than one version (thiserror 1 and 2,
   hashbrown 0.15, 0.16 and 0.17, …). Phase 1 wrote rustd3-iced's `figure` feature as
   `["dep:rustd3-figure"]` with `raster` on the dependency entry;
   the section names no form for the dev crates' `rustd3-iced` entries.

### Investigation

- (1), (2): compiler errors with the section's signature; iced sources
  at 3cbdacc as cited.
- (4): iced's runtime rebuilds the interface and redraws after any
  published message (`winit/src/lib.rs:777-830`).
- (5): two presses within one 60 Hz frame are under 17 ms apart; a
  person's double click takes 100–300 ms.
- (7), (8): the bias table in Measurements.

### Options

| | **A — as built** | B — as written |
| --- | --- | --- |
| (1) | `Harness::new(root: impl iced::Widget<Message> + 'a, …)` with `Message: 'a` | Does not compile at 3cbdacc |
| (2) | A plain `UserInterface` field; the phase that first rebuilds the interface from a new root (`into_cache`, then `build`) makes it an `Option` | An `Option` that is never `None` |
| (3) | The extra helpers, test code only | The cross-backend row cannot run |
| (4) | A program publishes each pointer and requests the redraw when it publishes nothing for an event while a press waits; the doc says so | — |
| (5) | One slot, as the struct's comment says; documented | A queue of presses stamped in order by the next redraw |
| (6) | (a) no `Left` while a button is held (drags continue, as d3's window listeners); a drag that ends outside reports `Left` at the next move, since the area never reported the pointer leaving; (b) `Pressed` for the second button, `Released` only for the first; (c) after `DoubleClicked` the stamped press is forgotten, so presses 2, 4, 6 … double-click, as iced's Single, Double, Triple, Double; (d) the previous factor is kept, as `Plot` skips such factors; (e) iced's `as_millis() <= 300` with a strictly later stamp and distance < 6 (`core/src/mouse/click.rs:73-84`) | — |
| (7) | Two more tests and a row: `every_device_offset_lands_exactly_at_every_factor`, `image_bounds_for_device_125_at_125_percent_are_100_2` (the spot row's arithmetic), and 1024 × 512 in `a_larger_image_needs_a_tick` | The three tests only |
| (8) | The clip is the image's own biased bounds, measured exact on both backends at 0.25 (as is the widget's whole rect); `device / k` should be exact too (tiny-skia's non-anti-aliased mask covers pixel centres from d, wgpu rounds d + 0.001) but was not measured | — |
| (9) | Recorded; `figure = ["dep:rustd3-figure", "rustd3-figure/raster"]` as the section writes it (same effect); `rustd3-iced = { path = "iced" }` in `[workspace.dependencies]`, used by the dev crates | — |

### Decision

A for all nine (architect, 2026-10-27), as built: the harness takes
`impl Widget<Message>` at the pinned revision; `ui` is a plain field
until a phase needs to take it; programs request a redraw only when
they publish nothing; the Tracker's details and the added tests stand.

Recommended: A for all nine, already built (a872bd6 … 542c74c).

### Why

(1), (2) and (4) follow the pinned iced API where the section's text
does not compile or would be dead code; (3) and (7) are test code that
makes the section's own spot rows and constants checkable; (5), (6) and
(8) are the readings closest to the section and to iced and d3; (9)
states what "nothing else changes" means for the lockfile.

### Followups

- Phase 18: `Plot` places with `NUDGE` and bands at or above
  `BAND_BYTES`; the harness gains an `Option` around the interface if
  its tests rebuild the view; the iced pixel bounds stay 1/255 and
  2/255 (placement measured 0/255).
- Phase 20: the widget's own pointer handling can publish and request a
  redraw through its `Shell`, so (4) concerns only `canvas::Program`
  apps (use cases, Phase 32).

## D11.2 — Core contract review before M2

**Date:** 2026-10-27 · **Context:** after Phase 11 (cb30109); an independent read-only review of `rustd3-core` at 978d068 · **Status:** decided

### Issue

Phases 1–10 form the core contract; after them, changing a public item
costs a decision and call-site updates across figure, iced and the dev
crates. A review of core before the figure layer builds on it found:

1. **Default labels ignore the tick rule.** `Ticks::labels` for
   `Continuous` (`scale/ticks/continuous.rs`) always formats through
   the transform's `tick_format`, d3's linear format or log's blanking,
   even when the ticks came from another rule. `Symlog` has no
   `tick_format` override, yet its ticks are decades:
   `symlog().constant(0.05).domain([0, 500])`, count 10, ticks
   `[0, 0.05, 0.5, 5, 50, 500]`, labels `"0","0","1","5","50","500"`;
   `constant(1e-6).domain([0, 1e-3])` labels `"0.0000"` three times;
   `sqrt().domain([0, 1]).tick_rule(TickRule::Transformed)`, count 5,
   ticks `[0, 0.04, 0.16, 0.36, 0.64, 1]`, labels
   `"0.0","0.0","0.2","0.4","0.6","1.0"`; `log().domain([1, 1000])
   .tick_rule(TickRule::Data)`, count 10, blanks six of ten labels.
2. **`range` and `range_round` clear `unknown` and the interpolator
   when the range type is unchanged.**
   `linear().unknown(-1.0).range_round([0, 100]).apply(NaN)` is NaN
   (d3: −1); `linear().range_round([0, 10]).range([0, 10]).apply(0.59)`
   is 5.9 (d3: 6). Phase 14's object-safe `Scale::with_range` can only
   call `range`, so every figure axis would drop a user's `unknown`,
   rounding and interpolator.
3. **Log and symlog minor ticks fill every decade even when the majors
   skip decades.** `log().domain([1, 1e12])`, `Ticks::minor` of
   `tick_set(5)`: majors 1, 100, 1e4, …; 96 minors at 2–9 of every
   decade and none at the skipped powers; likewise `[1e-4, 1e4]` at
   count 5 and symlog `[0, 1e9]` (through `between`), and the inherent
   `minor(count)`.
4. **An unbounded width aborts the process.** `format("1000000000000")`
   parses and then `format(1.0)` tries to allocate about 1 TB;
   `"99999999999999999999"` saturates the width to `usize::MAX` and
   `repeat` panics. d3 throws a `RangeError` (V8's string length limit).
5. **Additive gaps the next phases need:** no way from a runtime
   `format::Locale` to `locale::Locale` (so 48 of d3's 59 tables could
   never label an axis); no `Default` for `interact::Transform` (iced's
   `canvas::Program::State: Default`); no `Debug` or `Clone` for
   `series::Points` (Phase 15's marks hold it); aliases exist for
   `Linear`, `Pow` and `Log` only.
6. **Style:** `scale::Transform` has two public paths
   (`scale::transform::Transform` too), against "one public path per
   item"; the crate-private `LogKeep` is a composite name outside the
   kept list; the two parse errors have different shapes
   (`InvalidFormat(pub String)`, `ParseColorError` private with
   `input()`).
7. **Kept as they are:** `d`/`X` of ±∞ print `Infinity` (the spec's
   departure, Phase 3); band, point and ordinal keys compare with
   `PartialEq`, so a NaN key never matches (d3's SameValueZero finds
   it).

### Investigation

d3 values were computed with Node through the resolve hook; matplotlib
3.11.2 is installed. No decision of Phases 3–10 covers items 1–6;
Phase 5's "symlog tick format: the defaults (d3's symlog is
linearish)" assumed linear ticks, which symlog only has for narrow
domains.

### Options

| | **A — fix now, before Phase 12** | B — fix when a later phase meets it |
| --- | --- | --- |
| Approach | One session of core changes with call sites updated | Leave the contract as is |
| Cost | ~400 lines, one session | The same work later, plus figure and iced call sites |
| Correctness | Labels and minors right on every symlog, log and tick-rule axis; no abort from a parsed specifier | Wrong labels ship into the figure layer's snapshots |

### Decision

A (architect, 2026-10-27). The fix session does, one commit per item:

1. **Labels follow the ticks.** `Symlog::tick_format` follows the
   branch its `ticks` takes: decades label with `",.12~g"` as `Asinh`
   does; the linear branch keeps d3's linear format. When a scale's
   `tick_rule` is not the default, its default labels (`Ticks::labels`
   and `tick_format` without an explicit specifier) are `,f` with
   `format::precision_fixed` of the smallest positive gap between
   consecutive ticks, with no log blanking; fewer than two ticks fall
   back to the transform's format. Tests: the four rows of the Issue.
2. **`Continuous::with_range(self, range: [R; N]) -> Self`**, public:
   d3's `range(r)` for the same range type, keeping the interpolator,
   clamp and unknown. `range_round` is built on it, so it keeps
   `unknown` as d3 does. `range`'s doc says it always resets the
   interpolator and the unknown value (its type may change). Phase 14's
   `Scale::with_range` calls `with_range`. Tests: the two d3 rows.
3. **Minors under thinned majors follow matplotlib 3.11.2's
   `LogLocator`** (read the installed source, `ticker.py`, the minor
   case with `subs='auto'`): where it draws no minor ticks for a
   stride above one decade, draw none between those majors; where it
   draws them at the skipped decades, draw those. Same rule for symlog's
   decade branch and the inherent `minor(count)`. Tests: `[1, 1e12]`
   and `[1e-4, 1e4]` at count 5, symlog `[0, 1e9]` at count 5, compared
   with matplotlib's `LogLocator(subs='auto')` output (computed once,
   values in the test).
4. **Width bound:** `Specifier::from_str` returns `InvalidFormat` when
   the width exceeds V8's maximum string length, 2²⁹ − 24 = 536,870,888,
   d3's `RangeError` point. Phase 26's `const` checker applies the same
   bound.
5. **Additive items:** `impl From<format::Locale> for locale::Locale`
   (the time half Phase 28 adds defaults to the neutral one, so the
   impl stays valid); `impl Default for interact::Transform`
   (`IDENTITY`); `Clone for series::Points` where `X: Clone, Y: Clone`
   and a `Debug` that prints its kind and length (no bounds on the
   pairs beyond what it needs); aliases `scale::{Symlog, Logit, Asinh}`
   in the shape of `Linear`, `Pow` and `Log`.
6. **Style:** `Transform` is defined in a private module of `scale` and
   re-exported once as `scale::Transform`; `scale::transform` holds the
   transform types only. `LogKeep` becomes `Keep` (crate-private).
   `InvalidFormat` takes `ParseColorError`'s shape: a private field and
   `input()` returning the specifier as given.
7. Nothing changes for item 7; the band, point and ordinal docs say
   keys compare with `PartialEq`.

Each change to a public item of Phases 1–10 updates every call site
(core tests, harness rules and generated code, iced, examples) in the
same commit, with `scripts/check.sh quick` green; `full` at the end.

### Why

Items 1–3 put wrong output into every axis that uses symlog, a tick
rule, or a wide log domain, and the figure layer's snapshots would
bless it. Item 2 is a d3 divergence the figure layer cannot repair
from outside core. Item 4 turns a configuration typo into an abort.
Items 5–6 are cheapest before anything outside core names them.

### Followups

- Phase 14: `Scale::with_range` delegates to `Continuous::with_range`.
- Phase 26: the width bound in the `const` specifier checker.
- Phase 28: `locale::Locale`'s time half defaults to neutral in the
  `From<format::Locale>` impl.

## D11.3 — Details of the D11.2 fixes that the entry leaves open

**Date:** 2026-10-27 · **Context:** the D11.2 fix session (78ef1ca … 7d6f91b, 98d49c7) · **Status:** open

### Issue

1. **Float noise in the gap.** D11.2 (1): "`,f` with
   `format::precision_fixed` of the smallest positive gap between
   consecutive ticks". A gap is a difference of two doubles: d3's
   `precisionFixed(0.3 - 0.2)` is 2 (0.09999999999999998), so read
   literally, `linear().domain([0, 1]).tick_rule(TickRule::Data)` labels
   "0.00", "0.10", … where d3, and the scale without the rule, write
   "0.0", "0.1", …; far from zero the noise is larger: `[1e9, 1e9 + 1]`
   has the gap 1e9 + 0.3 − (1e9 + 0.2) = 0.09999990463256836.
2. **`func` with a rule other than `Data`.** D11.2 (1) covers a scale
   whose `tick_rule` is not the default. A `func` scale's rule belongs to
   its transform (`transform::Func`, Phase 5: "other methods default"),
   so the rule does not reach it, and it keeps the defect of the issue's
   third row: `func(f64::sqrt, |y| y * y, TickRule::Transformed).domain([0,
   1])`, count 5, ticks `[0, 0.04, 0.16, 0.36, 0.64, 1]`, labels "0.0",
   "0.0", "0.2", "0.4", "0.6", "1.0".
3. **matplotlib's second minor cut.** D11.2 (3) follows `LogLocator`
   where the major stride is above one decade. The same code
   (`ticker.py`, `tick_values`, `if n_avail >= 10 or b < 3: … return
   np.array([])` for `subs='auto'`) draws no minor ticks at all once ten
   or more decades are in view, at any stride.
   `log().domain([1, 1e12])` at count 10 has 13 majors a decade apart and
   96 minor ticks here; matplotlib draws none.
4. **The private module's name.** D11.2 (6) names no module for the
   trait.

### Investigation

- (1) Node 22 with the vendored d3: `precisionFixed(0.09999999999999998)`
  is 2. Ticks k/10 near 1e9 alternate gaps 0.10000002384185791 and
  0.09999990463256836.
- (2), (3) measured on the built code (throwaway test, not committed);
  matplotlib 3.11.2 `ticker.py` read in the installed source.

### Options

| | **A — as built** | B |
| --- | --- | --- |
| (1) | The gap is read to 12 significant digits of the largest tick (`format::precision_fixed_between`, crate-private) before `precision_fixed`; a gap below that resolution keeps the transform's format. Under a `Data` rule, linear labels equal d3's at [0, 1] and [1e9, 1e9 + 1] (tested) | The literal rule: one place too many wherever a gap falls just below a power of ten |
| (2) | Not built: `func` labels with d3's linear tick format whatever its rule | `Func::tick_format` uses the gap format (`TickFormat::spaced`) when no specifier is given; for d3's linear ticks (`Data`) that is the same format, so it can apply to every rule; about 15 lines and a test |
| (3) | Not built: 2–9 in every decade whenever majors are a decade apart | Log and symlog minors empty when the domain spans ten or more decades, as matplotlib |
| (4) | `scale::mapping` (private); the trait's only path is `scale::Transform` | — |

### Decision

Open. Recommended: A for (1) and (4), already built (78ef1ca, 98d49c7,
20e1f3a); B for (2), which completes D11.2's "labels follow the ticks"
for the one scale whose rule is its transform's; for (3), B if the
figure's minor ticks (Phase 39) should match matplotlib's density, A
to keep this session's scope.

### Why

(1) is the reading of D11.2 (1) that keeps d3's labels where the rule
gives d3's ticks; without it a `Data` rule would change labels it was
meant to leave alone. (2) and (3) change behaviour beyond what D11.2
decided, so they wait for an answer.

### Followups

- (2), if B: `transform::Func` overrides `tick_format`; test with the
  row above.
- (3), if B: `ticks::thinned` gains the decade count; Phase 39's minor
  ticks inherit it.
