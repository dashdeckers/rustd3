# rustd3 — session handoff 13

2026-10-27 · D11.3 (2) and (3) built, then **Phase 12 — Drawing list and raster outputs** complete in one session (D12.1 open) · next: **Phase 13 — Text with typst**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

**D11.3, decided B, B** (one `fix(core)` commit each, ratchet holding):

- e3589a8: `transform::Func` labels with the gap format
  (`TickFormat::spaced`) when no specifier is given:
  `func(f64::sqrt, |y| y * y, TickRule::Transformed).domain([0, 1])`, count
  5, labels "0.00", "0.04", "0.16", "0.36", "0.64", "1.00"; under `Data`
  the labels equal the linear scale's (tested at three domains, three
  counts).
- 4b378c2: log minors, and symlog's decade-branch minors, are empty when
  ten or more decade ticks are in view (matplotlib's `n_avail >= 10`;
  `for_domain` records it as `crowded` on `transform::Log` and
  `transform::Symlog`, through `ticks::crowded`), and log minors for a
  base below 3. `log [1, 1e12]` at count 10 and `[1, 1e9]` at 9: none;
  `[1, 1e8]` at 8: 2–9 in every decade (64), as matplotlib. The old test
  of base-e subdivision now uses base π and asserts e has none.

**Phase 12** (steps 1–7, one commit each, `check.sh quick` green on each,
`full` green on the last code commit):

- 5a9d40a `feat(figure): list types` — `List`, `Command`
  (`#[non_exhaustive]`), `Clip`, `Paints`, `GlyphRun`, `Glyph`, `Part`
  (defined in `list.rs`), `part::Id`, `list::Error`, `list::check`,
  `list::stroke`; `kurbo` and `peniko` re-exported at the crate root.
- 821eaee `feat(figure): Bez, bounds, outlines, dump` — `path::{Bez,
  bez}`, `list::{Bounds, Outlines, dump, dump::Options}`; the kitchen
  sink (`figure/tests/main/sink.rs`, plus `with_sweep()`); snapshot
  `list_dump_of_the_kitchen_sink.txt`.
- d17b8b8 `feat(figure): SVG writer` — `svg::{write, Options, Text}`;
  snapshot `svg_of_the_kitchen_sink.txt` (blessed after viewing resvg's
  render).
- 729dda3 `feat(figure): vello raster and PNG` — `raster::{Image, Error,
  Renderer, Settings, Simd, render, png::encode, snap}`.
- 154440b `feat(figure): lower` — `lower::{lower, Caps, Lowered,
  Warning}`, `svg::CAPS`.
- def06c6 `feat(figure): tiny-skia fallback` — `raster::skia::render`.
- e8b99de `test(figure): SVG through resvg`.
- 80230ca `fix(figure): unambiguous doc links` (caught by `full`'s
  `cargo doc`).

Exit criteria: `cargo test -p rustd3-figure` (default), `--features
raster,png`, `--all-features` and `--no-default-features` pass under
`TZ=Asia/Kathmandu`; both snapshots pass against checked-in references;
agreement: tiny-skia vs vello_cpu 52/54/56 at 1/1.5/2 (≤ 64), resvg on
our SVG vs vello_cpu tile 0.0053 (≤ 0.02), `lower` vs input ≤ 1/255,
shifts, areas and bands ≤ 1/255; `TooLarge`, `Empty` and exact sizes;
`lower_leaves_path_clips_in_place_without_clip_paths`; `List` and
`Bounds` `Send + Sync` (compile-time, `list.rs`); `check.sh full` green.

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed.

Decisions (`.claude/decisions/phase-12.md`): measurements, and **D12.1**
(open, A recommended, all built): fourteen statements that do not hold
or leave a detail open. The ones later phases meet: (1) vello_cpu 0.3.0
panics on render widths from 65,409 and heights from 65,533, so
`TooLarge` is above **65,408** device px a side (not 65,535); (3) a
paint (gradient, image brush) is positioned in the **group's space for
every command**, markers and glyph runs included; (4) image brushes are
drawn by vello_cpu only, `Unsupported` in SVG and tiny-skia; (8) SVG and
dump numbers: px two places, coefficients four, the glyph scale six;
(10) `Settings::parallel()` is `(parallelism − 1).clamp(1, 8)`; (12)
`raster::Error::Length`.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 and D11.3 fixes done |
| M2 Figure layer and widget | 11–22 | 11, 12 done; 13 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 13 (one session): read `rustd3.md`'s front matter and "### Phase
13 —"; it builds on Phase 12 (`list::Outlines`, `svg::{write, Options,
Text}`, `raster::render`). If the architect answers D12.1 with anything
but A, apply that first.

## 6. Notes for later phases

- Phase 13: `svg::Text` gains `Elements`; `svg::write`'s `match
  options.text` is where it plugs in. Runs from typst keep solid fills
  (gradient runs are written as paths, D12.1 (3)). The kitchen sink's
  runs are shaped from DejaVu's cmap; `list_dump_of_the_kitchen_sink`
  does not change when typst arrives.
- Phases 15, 37: `raster::Settings::parallel()` for file outputs;
  `save_svg` lowers with `svg::CAPS` and returns the warnings.
- Phase 18: cells and bands below 65,408 device px a side; `Renderer`
  is `Send` (asserted), `Bounds` `Send + Sync`.
- Phase 19: one `Renderer` per worker keeps the pixmaps of the images
  its last render drew (cache keyed by blob id, evicted per render).
- Phase 37: krilla positions marker and run gradients in the group's
  space; image brushes `Unsupported`.
- Tests read `fonts/DejaVuSans.ttf` with `include_bytes!`
  (`tests/main/sink.rs`), so they run without the `fonts` feature.
- Helpers for raster tests: `tests/main/raster/vello.rs` (`renderer`,
  `render`, `whole`, `crop`, `diff`).
- Handoff 12's notes for Phases 14, 26, 28 and 39 stand (Phase 39: log
  minors now also follow `n_avail >= 10`).

## 7. Key files

`figure/src/{list,path,svg,lower,raster,pixels}.rs`,
`figure/src/list/*.rs`, `figure/src/svg/{number,paint,base64}.rs`,
`figure/src/lower/*.rs`, `figure/src/raster/{vello,png,skia,snap}.rs`;
`figure/tests/main/{sink,list,svg,lower,skia,resvg}.rs`,
`figure/tests/main/raster.rs`, `figure/tests/main/raster/vello.rs`,
`figure/tests/snapshots/*.txt`; core: `core/src/scale/transform/{func,
log,symlog}.rs`, `core/src/ticks/rules.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background, ~95 s warm)
TZ=Asia/Kathmandu cargo test -p rustd3-figure --all-features
RUSTD3_BLESS=1 TZ=Asia/Kathmandu cargo test -p rustd3-figure   # after reading the .new file
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green at
  80230ca.
- D12.1 open with its recommendation.
- Conventional commits, no attribution.
