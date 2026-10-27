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

**Date:** 2026-10-27 · **Context:** Phase 11, steps 1–6 (a872bd6 … 542c74c) · **Status:** open

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
| (8) | The clip is the image's own biased bounds, exact on both backends at 0.25; Phase 18 may use either (both measured exact) | — |
| (9) | Recorded; `figure = ["dep:rustd3-figure", "rustd3-figure/raster"]` as the section writes it (same effect); `rustd3-iced = { path = "iced" }` in `[workspace.dependencies]`, used by the dev crates | — |

### Decision

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
