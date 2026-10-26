# proto/ — research prototypes behind rustd3's design

Evidence and lift-from sources for the rustd3 build (`rustd3.md`, next to this directory). Not part of the
workspace, not maintained, and full of `unwrap()`: copy ideas and
algorithms, never copy style. Each directory is its own cargo project
with relative paths between them. Inside the rustd3 repo the root
manifest has `exclude = [".claude"]`, so cargo treats these as
standalone packages and they build in place (except where noted).
Read them rather than build them where possible. Build outputs, nested
git repos and logs were stripped.

| Dir | What it is | Phase that lifts from it |
| --- | --- | --- |
| `poc/core` | Idiomatic port of d3-path, d3-array ticks, d3-format (with locales), d3-ease | 3 |
| `poc/obs` | `Obs` intermediate format and the tolerant comparison | 2 (into dev/testkit) |
| `poc/harness` | Harness main, prelude (`check`, `skip`, `Kind`) and `generated.rs` (4,900 lines, translated d3 tests) | 2 |
| `tr/translate.mjs` | acorn-based translator, d3 test JS → Rust `check(...)` calls (acorn vendored under `tr/vendor`; runs offline) | 2, then every core phase |
| `rfc/d3` | 1,242-line core sketch: scale (Linear, Band, Ordinal), shape (Line, Area, LineOf, Pie, Arc, Curve), time (Interval, Time), color (Rgb, Hsl, schemes), interact (Pointer, Zoom, BrushX, Transform, Rescale), anim (Animated, Timing, Marks), join, axis, interpolate, format | 4–10, 23–30 |
| `rfc/glue` | IcedPath, Pointer translation, use case 1 (`examples/focus_context.rs`) and 2 (`examples/update_pattern.rs`); built against iced master ab336f6 and does not build in place (its manifest pointed at a local iced checkout, and iced a8231fd9d "Remove `Into` generics in `push` methods" breaks it before 3cbdacc) — read it, don't build it | 11, 32 |
| `rfc/d3/examples` | `doc_snippets.rs`, `doc_outputs.rs`, `small.rs`: the design's code examples and their outputs; `fail_*.rs`: the counter-examples | spot checks in every core phase |
| `fscale` | `Transform` trait, `TickRule`, `ticks::{linear, transformed, log, decades, subdivide}`, Logit, Asinh, Func, the Reciprocal user transform (`src/bin/ext.rs`) | 4, 5 |
| `poly` | Const-generic piecewise domains vs a Vec, for f64 and colors | 5 |
| `symlog/ticks.py` | d3 vs matplotlib vs ours for symlog ticks | 5 (fixtures) |
| `thin2` | M4 index (`main.rs`: blocks of 64, power-of-two groups; `bin/gaps.rs`: NaN runs, Series) with exactness checks against a full scan | 9 |
| `cols` | Shared column boundaries across channels on one clock | 9 |
| `tloc` | `time::Locale` shape over jiff | 28 |
| `derive`, `autoskip` | `#[derive(Interpolate)]` (40 lines), and the rejected autoref-specialization variant | 24 |
| `figmodel` | Figure model: `plot/mod.rs` (Figure, ScaleRef, Panel), `plot/scales.rs` (Scale, Kind, Position, Paint, Ticks), `plot/arrange.rs` (protrusion layout, collisions), `plot/axis.rs`, `plot/legend.rs`, `plot/marks.rs`, `plot/theme.rs`, `plot/text.rs`, `plot/draw.rs` (list, with the old unshaped `Label` command), `examples/wheels_auto.rs` | 14–17, 33, 34 |
| `dlist` | Backends and passes over figmodel's list: `vcpu.rs` (vello_cpu), `skia.rs` (tiny-skia), `svg.rs` (outlined SVG), `pdf.rs` (krilla), `anyr.rs` (anyrender), `lower.rs` (+ i_overlay clip cutting), `shape.rs` (typst shaping into glyph runs), `measure.rs` (typst-measured widths with cache), `outline.rs` (glyph outlines via skrifa), `wheels.rs` (the wheel figure); binaries `partial` (panel and strip renders, shift equivalence), `hatch`, `paints`, `bench`, `extra`, `sprites`, `stamps`, `imgcost`, `glyphres`, `skiat`, `measure`, `titles`, `themes` | 12, 13, 31, 34, 35, 37 |
| `typstspike` | typst `World` over bundled fonts, label layout on a tight page, outlines into PathContext, rich text (`bin/rich.rs`), label benchmark (`bin/bench.rs`) | 13 |
| `icedspike` | iced master 3cbdacc checks: `bin/layerclip.rs` (`with_layer` clips on both renderers), `bin/xform.rs` (`with_transformation`), `bin/canvasroute.rs` (canvas route, tiny-skia `with_clip` bug), `bin/plotwidget.rs` + `prim.rs` (vello in a shader widget), `main.rs` (timings). Cargo.toml repointed from a local checkout to the git revision | 11, 18, 19 |
| `d3-tests/<module>` | d3 test directories, LICENSE and package.json at the versions measured (d3-format and d3-time-format include `locale/`; `d3-time/test/test-time-zones` is a bash script the translator does not see) | 2 (vendored into dev/harness/d3) |
| `d3-src/<module>` | d3 source (`src/`), LICENSE and package.json at the same versions, plus d3-drag, d3-selection and d3-transition for behavior reference, and internmap, d3-dispatch and d3-timer so the sources import: the spec for behavior the tests don't cover (brush, drag, viridis tables, curves, time-format parsing) | every core phase |
| `tr/d3-resolve.mjs` | Node resolve hook: `node --import proto/tr/d3-resolve.mjs script.mjs` runs d3 from `d3-src` with no node_modules | fixtures in every core phase |
| `tr/vendor/acorn` | acorn 8.19.0 (MIT), vendored so the translator runs offline: `node proto/tr/translate.mjs proto/d3-tests out.rs` reproduces `poc/harness/src/generated.rs` | 2 |
| `fonts` | DejaVuSans.ttf, DejaVuSans-Bold.ttf (DejaVu 2.37, Bitstream Vera licence), NewCMMath-Regular.otf (GUST Font License), with licence texts | 1 |

Measured numbers quoted in `rustd3.md` came from these on a 2-core
container (release builds, ±30% run to run).
