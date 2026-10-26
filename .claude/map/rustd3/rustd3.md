# rustd3: an idiomatic Rust API for d3

Design and implementation · 2026-10-18 · @Travis Hammond

This is the one document for rustd3. Part I is the API design: what
the library is, how its pieces fit, and why each decision went the way
it did, with the measurements and compiler errors behind them. The
Implementation part goes from there into the build: the workspace,
the conventions every commit follows, how the core is graded by d3's
own tests, the order of work, and forty phases, each specified down to
signatures, algorithms, tests and expected values. An agent
implementing a phase reads the Implementation front matter, its
phase's section and the Part I sections that section names.

## Summary

This document proposes a Rust plotting library that speaks d3's vocabulary and adds the tier-1 pieces only matplotlib has. It is standalone first: it renders SVG, PNG and PDF (and GIF for sequences), typesets text and math with typst, and handles time with jiff. iced integration comes second, as one backend among several, with a first attempt at interaction and animation. The document carries its own implementation plan as well: the part named Implementation turns this design into forty phases, each sized for one agent session or, where it names a cut point, two.

Four ideas carry it. Scales and generators are plain values. Data reaches generators as iterators of geometry, with a thin accessor layer for d3 familiarity. Generators draw into `PathContext`, and a figure layer turns that geometry, text and images into a styled drawing list that every backend renders. Interaction and animation are small state machines and time-sampled values.

A prototype with a 1,242-line core crate implements the proposal. Examples show the API this document specifies; most are taken from that compiled code or from the figure-layer sketches (figmodel, dlist; see Sources), lightly trimmed, and sketches are marked. The small ones run, and their output is shown; where the specified API differs from the prototype's, the outputs are d3's own, computed in Node. Counter-examples quote the real compiler errors. Both use cases build against iced master (0.15.0-dev, 2026-10-13), and use case 1 was also driven and rendered headlessly through `iced_test`.

Spikes back the later decisions (see Sources); two early ones set the direction. typst lays out a math label in 0.4–1.5 ms, and its glyph outlines, drawn through `PathContext`, match typst's own SVG. And a check of colorgrad against d3 showed that its presets are d3's own color stops run through a basis spline in `f32`, so the core ports d3's scheme code and is exact in `f64`.

Scales, shapes, formats, color, zoom and brush translate well, often better than in d3, because types rule out mistakes d3 only catches at runtime. d3's selection-and-transition model strains: keyed, staggered transitions work, but need a typed struct per mark and explicit lifecycle state, roughly four times the code of the d3 version. Every design question but one is decided (see Settled questions). The open one is the full-frame benchmark: Phase 22 measures it, and its verdict decides whether Phase 31 is built (see Implementation).

## Motivation

d3's value is its vocabulary and its math: scales, nice ticks, shape generators, number and time formats, color spaces, and well-tuned zoom and brush behavior. Its API, though, is shaped by JavaScript: chained getter-setters, functions passed where constants also work, objects that mutate when called, and a DOM selection model for everything visual. A literal port would carry those shapes into Rust and lose what Rust adds.

**Goals**

- A standalone plotting library: d3's tier-1 vocabulary and math, plus the tier-1 pieces only matplotlib has (text and math, legends, colorbars, statistical charts, more scale types).
- SVG, PNG and PDF output, GIF for sequences, and fast enough to pan without dropped frames.
- iced integration second: a Plot widget for figures and a canvas path for core-level apps, with d3's tier-3 behaviors (zoom, brush, transitions) as a first attempt at interaction and animation.
- Idiomatic Rust: `std` first, values over objects, types instead of runtime checks, no DOM concepts.
- A renderer-agnostic core that d3's own tests can grade.
- Names a d3 user recognizes: domain, range, ticks, bandwidth, curve.

**Non-goals**

- API compatibility with d3's JavaScript surface: no getter-setter overloading, no string-typed attributes.
- A grammar-of-graphics layer in the style of Observable Plot or Vega-Lite. It could sit on top later.
- Tier 2 layouts (hierarchy, force, geo) in this version.
- Pixel parity with matplotlib or d3. Our own raster outputs are compared with each other, within a tolerance (see Testing).

## Design principles

1. **Values, not objects.** Scales and generators are cheap `Clone` values, built in `view` from application state and dropped afterwards. Calling one never mutates it. d3 objects are long-lived and mutable; iced rebuilds the view, so there is nothing to keep.
2. **Geometry in, paths out.** Generators take iterators of already-scaled points. Reading fields from data is ordinary Rust (`map`, `filter`), not accessor closures stored inside the generator.
3. **One drawing seam.** `PathContext`, d3's canvas path interface, is the only trait between d3's generators and drawing. A figure layer adds style, text and images on top and emits one drawing list, which every backend renders: SVG, PNG, PDF, GIF, iced, or a test recorder.
4. **Types replace runtime checks.** `invert` exists only for numeric ranges, unknown keys return `Option`, parsing returns `Result`, and closed sets (curves, intervals, axis orientation) are enums.
5. **Explicit over ambient.** The time zone, the locale and the current `Instant` are parameters. Library code never reads the clock or the system zone; the only process-wide state is immutable parsed fonts, counters that hand out figure and recording ids and pace the text cache's eviction, and typst's memo cache, which the figure layer bounds.
6. **Behavior as state machines.** Zoom, brush and drag take pointer input and return new state. The application owns the single source of truth.
7. **d3's words, Rust's shapes.** d3 vocabulary in snake\_case under module paths (`scale::Linear`, `shape::Line`, `Curve::MonotoneX`), with no composite names such as `LinearScale`.

## Architecture

The library is three layers. The d3 core stays a pure computation crate, close to d3's own modules. A figure layer adds what d3 leaves to the browser: figures and axes, layout, text and style. Backends turn its output into files or pixels.

**It can be used at two levels.** At the core level an application draws itself, as with d3 in a browser; at the figure level it describes a figure and a backend draws it. The two use cases are core level; the wheel figure is figure level.

|  | Core level | Figure level |
| --- | --- | --- |
| Crates | `rustd3-core`, and `IcedPath` and `pointer::Tracker` from `rustd3-iced` | figure layer and backends, including an iced widget |
| The application writes | its own `canvas::Program`, layout and margins | a `Figure`: marks, axes, grid |
| Text | the host's (iced's `fill_text`) | typst only |
| Output | whatever its `PathContext` is | the drawing list: SVG, PNG, PDF, GIF or iced |

```rust
// Core level: the app draws.
let trace = path(|p| {
    shape::line()
        .curve(Curve::MonotoneX)
        .draw(points, p)
});

// Figure level: the app describes.
let accel_z = rec.series(2).expect("channel 2");
panel.mark(
    line(accel_z, time, vib)
        .stroke(color, src),
);
```

The figure layer sits between the core and the backends. Every file output is drawn from the same drawing list, so they cannot drift apart. iced shows images that vello\_cpu renders from the list, with interaction feedback drawn on top, and sends pointer events back to the core's zoom, brush and drag behaviors.

- **Dependencies point down.** The core depends on jiff and thiserror only, and on rustd3-derive behind its `derive` feature. typst enters at the figure layer, vello\_cpu, krilla and gif in their backends, iced only in its own crate.
- **Each backend is a cargo feature.** A headless report generator never compiles iced, and an iced app never compiles krilla. An iced app with figures does compile vello\_cpu, which renders its images.
- **The orphan rule question is settled.** `IcedPath` lives in rustd3-iced as a wrapper type, so the core never names iced types.

**The library ships as `rustd3`, a facade over five library crates.** The plain `d3`, `d3-core` and `d3-derive` names on crates.io belong to an unrelated server framework; every `rustd3` name below was free on 2026-10-16. Four library crates and the facade go to crates.io. `rustd3-iced` depends on iced at a pinned git revision, and crates.io rejects git dependencies, so it stays `publish = false` until iced 0.15 is on crates.io; until then apps take it from git.

```text
rustd3/                  workspace; the root package is the facade
├─ src/                  rustd3: re-exports by feature
├─ core/                 rustd3-core: the d3 port (core level)
├─ derive/               rustd3-derive: #[derive(Interpolate)]
├─ figure/               rustd3-figure: figure layer + outputs
├─ fonts/                rustd3-fonts: bundled fonts
├─ iced/                 rustd3-iced: IcedPath, pointer::Tracker, Plot
└─ dev/                  publish = false
   ├─ testkit/           rustd3-testkit: Obs, pixel statistics,
   │                     snapshots, zones (depends on no workspace crate)
   ├─ harness/           rustd3-harness: d3's tests → Obs format
   ├─ fixtures/          reference values: d3 via Node, matplotlib via Python
   ├─ examples/          rustd3-examples: wheel figure, use cases, presets
   └─ bench/             rustd3-bench: full-frame benchmark app
                         (uses examples' wheel figure)
```

| Crate | Contains | Depends on | Features |
| --- | --- | --- | --- |
| `rustd3-core` | `scale`, `ticks`, `axis`, `shape`, `path` (`PathContext`, `SvgPath`), `format`, `time`, `locale` (`Locale`), `color`, `scheme`, `interpolate`, `ease`, `array`, `series` (`Recording`, `Series`, `Data`), `anim`, `interact` (`Zoom`, `BrushX`, `BrushY`, `Drag`, `Pointer`, `Rescale`), `join` | jiff (re-exported as `rustd3_core::jiff`), thiserror | `derive` |
| `rustd3-derive` | `#[derive(Interpolate)]` | syn, quote, proc-macro2 | none |
| `rustd3-figure` | `Figure`, `Panel`, marks, figure axes, legends and colorbars, layout, `Theme`, typst text and its measure cache, `fig.check`, the drawing list (re-exporting kurbo and peniko), `lower`, the SVG writer; re-exports core's `Locale` | core, typst, typst-layout, ttf-parser, kurbo, peniko, png, thiserror; fonts (default) | `fonts` (default), `raster`, `png`, `gif`, `pdf`, `tiny-skia`, `helpers` |
| `rustd3-fonts` | DejaVu Sans, DejaVu Sans Bold and New Computer Modern Math through `include_bytes!`, in place of typst-assets' 9.6 MB font set, which lacks DejaVu Sans | none | none |
| `rustd3-iced` | `IcedPath` and `pointer::Tracker`, which turns iced events into `Pointer` input (core level); the `Plot` widget with its cell images, overlay and worker-thread renders (figure level) | core, iced\_core and iced\_widget at a pinned revision; figure with `raster` behind its `figure` feature | `figure`, `fonts` (default) |
| `rustd3` | re-exports, so users add one dependency | core; figure, optional | `figure`, `derive`, and the figure features |

| Figure feature | Adds | Crates beyond typst's 268 |
| --- | --- | --- |
| none | typst text with fonts the app supplies, layout, the SVG writer | 0 |
| `fonts` (default) | the three bundled fonts | +1 |
| `raster` | vello\_cpu rendering the list into pixels | +7 |
| `png`, `gif` | `raster` and an encoder | small |
| `pdf` | krilla | +5 |
| `tiny-skia` | the fallback rasterizer | +1 |
| `helpers` | ready-made charts such as `chart::line` and `subplots` | 0 |

```toml
# core-level iced app: no typst, no vello_cpu.
# rustd3-iced comes from git until iced 0.15 is on
# crates.io; take rustd3 from the same repository, so
# cargo builds one rustd3-core and their types match.
rustd3 = { git = "https://…/rustd3" }
rustd3-iced = { git = "https://…/rustd3" }

# headless reports
rustd3 = { version = "0.1", features = ["figure", "png", "pdf"] }

# iced app with figures: typst and vello_cpu, no krilla
rustd3 = { git = "https://…/rustd3", features = ["figure"] }
rustd3-iced = { git = "https://…/rustd3", features = ["figure"] }
```

A crates.io `rustd3` beside a git `rustd3-iced` would compile two copies of rustd3-core, one per source, and a `Pointer` from one would not be a `Pointer` of the other; hence one source for both. When iced 0.15 is on crates.io, rustd3-iced is published and the facade gains an `iced` feature, and each iced app needs one line: `rustd3 = { version = "0.1", features = ["iced"] }`, or `features = ["figure", "iced"]` with figures.

Backends are features of one figure crate rather than crates of their own, because they all read the same list types and only `pdf` brings a heavy dependency; they can split later if krilla's release cadence demands it. Fonts are a crate, on by default, so a figure renders the same on every machine without reading `/usr/share/fonts`, and users with their own fonts can turn it off. `lower` turns glyphs into outlines and markers into fills for a backend that lacks them; a path clip that a backend cannot draw stays in place and `lower` returns an `UnsupportedClip` warning. Cutting clips in geometry is not in v0.1, because no built-in backend needs it (see Later). The facade keeps the module paths of the naming rule: `rustd3::scale::Linear`, `rustd3::axis::bottom`, `rustd3::locale::Locale`, `rustd3::figure::Figure`.

## Dependencies

The library reuses established crates wherever they match d3's behavior, and ports d3 wherever the behavior is the product. Where a crate claims to replace a d3 module, it was checked against d3 7.9.0 output first. Library crates depend on exactly the crates below; any other dependency is a recorded decision (see Implementation).

| Crate | Role | Evidence or caveat |
| --- | --- | --- |
| typst 0.15.1 + typst-layout | All text and math | Spike output matches typst's own SVG; +46 MB stripped binary in the spike, 9.6 MB of it typst-assets' full font set, which rustd3 does not use: rustd3-fonts ships DejaVu Sans in two weights and New Computer Modern Math (see Architecture) |
| jiff 0.2.38 | Time, zones, nanoseconds | Already in the prototype; re-exported as `rustd3_core::jiff` |
| thiserror 2 | Error types that carry the rejected value | Every library crate |
| vello\_cpu 0.3.0 | PNG and GIF frames and the widget's images from the drawing list | 3× faster than tiny-skia and 5× faster than resvg on the wheel figure; +7 crates beyond typst and a young 0.x API, so the tiny-skia backend stays as a fallback (+1 crate). Built with `default-features = false` and `std`, `text`, `u8_pipeline`, `multithreading` |
| gif 0.14 | GIF sequences | Encodes raster frames |
| krilla 0.8.2 | PDF with real text from the drawing list | typst's own PDF export uses krilla 0.8.2; +5 crates against svg2pdf's 20 |
| kurbo 0.13.1 + peniko 0.6.1 | Geometry, strokes, paint and font data of the drawing list | Re-exported; typst 0.15.1 already depends on both, so they add no crates |
| ttf-parser 0.25 | Glyph outlines from the bundled and user fonts | For SVG outlines and for `lower` |
| png 0.18 | Images inside SVG; the `png` feature's encoder | Always in the figure crate |
| tiny-skia 0.12 | Fallback raster backend, and the raster oracle in tests | +1 crate; 61–66 ms for a 2× PNG of the wheel figure, against vello\_cpu's 18–20 ms |
| iced\_core + iced\_widget at rev 3cbdacc (0.15.0-dev), with `advanced`, `canvas`, `image` | GUI backend, in rustd3-iced only | No winit and no renderer in the library's graph; the app's own `iced` brings those |
| rustd3-derive (syn 2, quote 1, proc-macro2 1) | `#[derive(Interpolate)]`, behind core's `derive` feature | +2.2 s clean build alone, about +0.3 s next to typst, which already compiles syn |

Dev crates add the same iced revision (`iced`, `iced_test`, `iced_runtime`) for headless tests, resvg 0.48.1 to rasterize our SVG in tests, jiff's bundled time-zone database for tests, serde\_json for test fixtures, and serde only in the benchmark, for its replays and records. No library crate depends on serde.

Evaluated and not adopted: kasuari and taffy for layout (see Layout), minmaxlttb for thinning (see Performance). fast\_image\_resize is a candidate for raster heatmaps after v0.1.

**The core ports d3's color math and every scheme exactly, in `f64`.** Two color crates were checked against d3 and are not used. colorgrad 0.9's presets equal d3's at 11 samples for Blues, RdBu, Turbo, Cividis and Sinebow, because they are d3's own stops run through a basis spline, but in `f32`: at 257 samples they are exact for 27 schemes and within 1/255 for Blues (3 samples off), Reds (3), RdPu (3), PuRd (2), Purples, YlOrBr and CubehelixDefault (1 each), and its 9–11-stop viridis family differs from d3's 256-entry tables by up to 15/255 at 11 samples and 16/255 at 257 (inferno 13, magma 7, plasma 5). palette 0.7's Lab is within 1/255 of d3 as a color, but gives (74.968804, −3.399015, −10.696497) for rgb(170, 187, 204) against d3's (74.968800, −3.398999, −10.696507), 1.7e-5 off where d3's tests allow 1e-6, because d3 uses 7-digit rounded Bradford constants (`Xn = 0.96422`, `Zn = 0.82521`); and its HCL midpoint between white and steelblue differs completely (198, 182, 211 against d3's 138, 194, 248), because it has no undefined hue for grays. So the core ports d3-color's Lab, Hcl and Cubehelix formulas with d3's NaN rules, and d3-scale-chromatic's code:

- ColorBrewer sequential and diverging schemes are d3's basis spline over the scheme's 9- or 11-stop row, generated from d3's sources;
- `turbo` and `cividis` are d3's polynomials, `sinebow` d3's sin² formula and `rainbow` d3's cubehelix formula;
- `warm`, `cool` and the default cubehelix go through `interpolate::cubehelix_long`, as in d3;
- the viridis family keeps d3's 256-entry tables.

Every scheme matches d3 at t = 0, 0.1, …, 1 with no tolerance.

**Kept as our own code:** scales, ticks, shapes and curves, d3-format (the proof-of-concept port passes d3's own tests), d3-time-format's directive table, time intervals, color spaces and schemes, and easing. Their exact behavior is what d3's tests grade.

| Rejected or later | Why |
| --- | --- |
| iced\_aksel 0.3 | No general path primitive, and its filled area has no holes, so glyphs and full donut rings cannot be drawn. A second axis model competes with d3's scales. Only lockfile commits since 2026-05-20. 11,135 lines of code (9,721 plus 1,414 in its scale core) to save an estimated 1,200–1,500 |
| format\_num 0.1 | A 2020 d3-format port without locales |
| plotters 0.3 | Its own coordinate model; a reference for backend traits, not a dependency |
| colorgrad 0.9 | d3's stops through a basis spline in `f32`; porting d3's code is exact (above) |
| palette 0.7 | Lab outside d3's 1e-6 tolerance, and no undefined hue (above) |
| anyrender 0.14 | A 63-line adapter replaying the list onto vello on the GPU, vello\_hybrid and Skia (+2 crates); no v0.1 consumer (see Later) |
| i\_overlay 9.1 | Cuts non-rectangular clips in geometry for custom backends without clip paths (within 38/255 of a masked render, 5.2–7.8 ms for a 2,038-shape polar panel); no v0.1 consumer, since no built-in backend needs it (see Later) |
| mitex 0.2.4 | Converts LaTeX to typst, but its output calls the mitex typst package, and the crates.io release lags it. An optional later addition |

## Accessors

**Decision: generators take iterators of geometry (option A). A thin wrapper layer offers d3-style accessors (option B) that borrow for a lifetime `'a`, never a `'static` box.** In d3, every generator stores functions that pull values out of arbitrary objects:

```javascript
const line = d3.line()
    .x(d => x(d.t))
    .y(d => y(d.value))
    .defined(d => d.value != null);
path.attr("d", line(data));
```

**Option A, iterator-first (proposed).** Mapping data to points is ordinary Rust, and a missing value is `None`, which breaks the line as `defined` does:

```rust
let d = svg(|p| {
    let points = data
        .iter()
        .map(|r| {
            let x = tx.apply(r.t);
            r.value.map(|v| (x, ty.apply(v)))
        });
    shape::line()
        .draw(points, p)
});
// → M0,75L100,25M300,50L400,0
```

**Option B, accessor builder.** The same output, closer to d3. The `'a` lifetime on `LineOf<'a, D>` is what lets the closures borrow `tx` and `ty` from the enclosing `view`:

```rust
let d = svg(|p| {
    shape::line_of()
        .x(|r: &Reading| tx.apply(r.t))
        .y(|r: &Reading| {
            ty.apply(r.value.unwrap_or(0.0))
        })
        .defined(|r: &Reading| r.value.is_some())
        .draw(&data, p)
});
// → M0,75L100,25M300,50L400,0
```

Three pitfalls showed up when compiling alternatives:

1. **Stored as `Box<dyn Fn(&D) -> f64>`, accessors cannot borrow a scale.** The compiler explains why: `error[E0597]: x does not live long enough`, with the note that, due to object lifetime defaults, the box actually means `dyn Fn(&f64) -> f64 + 'static`. Users would have to clone each scale into a `move` closure.
2. **Closures have one arity.** d3 passes `(d, i, data)` and users take what they need. In Rust, `.x(|d: &f64, i: usize| …)` fails with `error[E0593]: closure is expected to take 1 argument, but it takes 2 arguments`. Indices come from `enumerate()` instead, which Option A gets for free.
3. **"Constant or function" works, but costs inference.** `impl From<f64>` and `impl<F: Fn(&D) -> f64> From<F>` on one `Accessor` enum compile, because the `Fn` traits are `#[fundamental]` and `f64` can never implement them. But `.y1(|r| r.v)` then fails with `error[E0282]: type annotations needed`, so every closure needs `|r: &Row|`.

| Option | Call site | Cost | Verdict |
| --- | --- | --- | --- |
| A: iterators of geometry | `.draw(data.iter().map(…), ctx)` | None; also handles gaps, indices and multiple series | Default |
| B: accessors with `'a` | `.x(\|r: &Reading\| …)` | Lifetime parameter, annotated closures, a box per accessor | Ships as a thin wrapper over A |
| B with `'static` boxes | Same | Every scale cloned into `move` closures | Reject for view-time generators |
| C: `impl Into<Accessor>` | `.y0(300.0).y1(\|r: &Row\| r.v)` | Annotations on every closure | Only where constants dominate, such as an area's baseline |

**The wrapper adds no second code path.** In the prototype, `LineOf::draw` maps the slice through its accessors and hands the resulting iterator to the option A generator, so curves, gaps and tests are shared:

```rust
pub fn draw(
    &self,
    data: &[D],
    ctx: &mut impl PathContext,
) {
    let points = data.iter().map(|d| {
        (self.defined)(d)
            .then(|| ((self.x)(d), (self.y)(d)))
    });
    line()
        .curve(self.curve)
        .draw(points, ctx);
}
```

## Scales

Scales are generic value types whose type parameters encode what d3 checks at runtime. Constructors mirror d3 (`scale::linear()` for `d3.scaleLinear()`), builders consume and return the value, and calling is `apply`, because user types cannot implement `Fn` on stable Rust.

```rust
let x = scale::linear()
    .domain([0.0, 100.0])
    .range([0.0, 960.0]);
x.apply(25.0);    // 240
x.invert(480.0);  // 50
```

**The range decides the output type.** `range` is a type-changing builder, so a color range yields `Linear<Rgb>`, interpolated through an `Interpolate` trait instead of d3's runtime type sniffing:

```rust
let steel: Rgb = "steelblue".parse()?;
let white = Rgb::new(255.0, 255.0, 255.0);
let heat = scale::linear()
    .domain([0.0, 1.0])
    .range([white, steel]);
heat.apply(0.5);  // rgb(163, 193, 218)
```

`invert` is implemented only for scales with an `f64` range, so `heat.invert(0.5)` does not compile: `error[E0599]: no method named invert found`, with a note pointing to the impl for `f64` ranges. d3 returns NaN here at runtime.

**Nice domains, ticks and tick formats** follow d3 exactly:

```rust
let y = scale::linear()
    .domain([0.13, 9.71])
    .range([300.0, 0.0])
    .nice(5);
let f = y.tick_format(5, None)?;
y.get_domain();  // [0.0, 10.0]
y.ticks(5)
    .iter()
    .map(|v| f.format(*v));
// → "0" "2" "4" "6" "8" "10"
```

`tick_format` formats under the neutral locale; `tick_format_in(&locale, count, specifier)` takes one explicitly. Both return a `scale::TickFormat`, d3's `tickFormat` as a value: a number format and, for log scales, which ticks keep a label. Besides `format(v)` it has `format_all(&[f64])`, which applies the per-axis grouping rule over all values at once (see Formats, time and color) and then the log scale's blanking, and `labels(&ticks::Set, &Locale)`, which the axis trait below calls.

**Categorical lookups return `Option`.** d3 returns `undefined` for an unknown band:

```rust
let bands = scale::band(["a", "b", "c"])
    .range([0.0, 300.0])
    .padding(0.1);
bands.apply(&"a");  // Some(9.677…)
bands.apply(&"z");  // None
bands.bandwidth();  // 87.10
```

**Ordinal scales get an explicit domain.** d3's `scaleOrdinal` silently appends unseen keys when called, a mutation hidden in a read. The proposal builds the domain from data up front, which suits views rebuilt from state, and keeps growth as an opt-in `&mut` method. It returns `Option<&V>`, because a scale with an empty range has nothing to return:

```rust
let color = scale::ordinal(
    ["cpu", "mem"],
    scheme::CATEGORY10,
);
color.apply(&"cpu");
// → Some(rgb(31, 119, 180))
color.apply(&"disk");  // None

let mut grow = scale::ordinal(
    Vec::<&str>::new(),
    scheme::CATEGORY10,
);
let disk = grow.apply_or_insert("disk").copied();
// → Some(rgb(31, 119, 180))
let net = grow.apply_or_insert("net").copied();
// → Some(rgb(255, 127, 14))
```

The borrow checker shows why growth should stay opt-in: `println!("{:?} {:?}", grow.apply_or_insert("disk"), grow.apply_or_insert("net"))` fails with `error[E0499]: cannot borrow grow as mutable more than once at a time`.

**Sequential scales keep their interpolator behind an `Arc`** so the type stays nameable as `Sequential<Rgb>` in a struct field and clones cheaply. A generic `F` would leak an unnameable closure type into every struct that holds the scale:

```rust
let seq = scale::sequential(
    [0.0, 50.0],
    scheme::blues,
);
seq.apply(25.0);  // rgb(109, 174, 213)
```

**Continuous scales are generic over a public transform.** d3 builds its log, pow and symlog scales on an internal `transformer()` that d3-scale does not export, which is why a logit or asinh scale is hard to add in d3. Here every continuous scale is `Continuous<T, R>`: `T` implements a public `scale::Transform` trait, and `R` is the range type, as in `Linear<Rgb>` above. d3's scales become aliases, and logit and asinh join them. ggplot2's `new_transform` and matplotlib's scale classes pair a transform with its own tick rule the same way.

```rust
pub trait Transform: Clone {
    fn forward(&self, x: f64) -> f64;
    fn inverse(&self, y: f64) -> f64;
    // Required: every transform states
    // its rule, usually from `ticks::`.
    fn ticks(
        &self,
        domain: [f64; 2],
        count: usize,
    ) -> Vec<f64>;
    // Default: equal steps in transformed
    // space; log-like transforms provide
    // 2–9 per decade.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        ticks::subdivide(self, major, 5)
    }
    // Defaults follow d3's linear scale;
    // Log provides its own for all three.
    fn for_domain(&self, domain: [f64; 2]) -> Self {
        self.clone()
    }
    fn nice(
        &self,
        domain: [f64; 2],
        count: usize,
    ) -> [f64; 2] {
        /* d3-scale's linear nice */
    }
    fn tick_format(
        &self,
        locale: &format::Locale,
        domain: [f64; 2],
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        /* d3-scale's tickFormat.js */
    }
}

// N stops; two unless built by
// scale::piecewise (see Settled
// questions, polylinear domains).
pub struct Continuous<
    T,
    R = f64,
    const N: usize = 2,
> {
    transform: T,
    domain: [f64; N],
    range: [R; N],
    // forward cache, interpolator, clamp,
    // unknown value, tick rule
}
pub type Linear<R = f64, const N: usize = 2> =
    Continuous<transform::Identity, R, N>;
```

**Log needs the three defaulted methods.** d3 picks `ln x` or `−ln(−x)` from the sign of the domain (`log.js` `rescale`), nices to powers of the base, and blanks tick labels so that a decade shows 1, 2, …, 5 and empties rather than nine crowded numbers. None of that fits `forward`, `inverse`, `ticks` and `minor`, and inherent methods on `Continuous<transform::Log>` would clash with the generic ones (E0592). The defaults keep a user transform at three required methods plus `Clone`, which the `Reciprocal` example below derives. A log scale over a domain that straddles zero yields NaN for values of the other sign, as d3 does, and its ticks and nice bail out (`[]`, the domain unchanged).

**Two `Identity`s exist, and code always writes the module path.** `scale::transform::Identity` is the transform under `Linear`. `scale::Identity` is d3's `scaleIdentity`, a scale whose domain and range are one array and whose `apply` and `invert` return their input. The same rule holds for the trait `scale::Transform` and the zoom value `interact::Transform` (see Interaction and animation).

Builders that belong to one transform exist only on that type, as d3 names them. User transforms are ordinary types with the same standing as built-in ones, and closures work through `func`:

```rust
let y = scale::log()
    .base(2.0)
    .domain([1.0, 1024.0]);

let p = scale::logit()
    .domain([0.001, 0.999]);
// ticks: 0.001 0.01 0.1 0.5 0.9 0.99 0.999

let v = scale::symlog()
    .constant(1.0)
    .domain([0.0, 1e6]);
// ticks: 0 1 10 100 … 1000000

// The rule is an argument: the choice is
// required by the signature.
let s = scale::func(
    |x| x.sqrt(),
    |y| y * y,
    TickRule::Data,
)
.domain([0.0, 100.0]);
s.ticks(5); // d3's meaning, unchanged
```

**Why a type parameter and not an enum of transforms.** Both designs were built and measured. The enum keeps one nameable type, but turns transform-specific builders into runtime checks (`linear().base(2.0)` compiled and panicked), closes the set of built-ins, and reaches user transforms only through `dyn`. The type parameter makes the same call a compile error (`E0599: no method named base found for Continuous<Identity>`). Transforms are an open set and the operations on them (apply, invert, ticks, nice, format, rescale) a small fixed one, which favors the type parameter. The enum was also 5–15% slower per point, which is negligible after thinning.

**Choosing a scale at runtime goes through an erased transform.** A log toggle or scales stored by id cannot name one type, so `AnyContinuous` is `Continuous<transform::Any>`, where `transform::Any` is an enum of the built-in transforms whose seven `Transform` methods a macro forwards. Every builder and reader of `Continuous` works on it unchanged; transform-specific builders apply before converting with `.into()`. A blanket `From<Continuous<T>>` would overlap the reflexive impl at `T = Any` (E0119), so there is one `From` per built-in transform. The figure's own table holds every scale kind behind a trait object (see Figures).

**Each transform brings its tick rule.** Nice ticks in transformed space, mapped back, gave an asinh axis −201.71, −27.29 and −3.63; matplotlib's `FuncScale` ticks linearly in data space instead, which suits sqrt and fails for logit. So `Transform::ticks` is required, built-in transforms ship their own rules, and the rules are public building blocks in a `ticks` module (`linear`, `transformed`, `log`, `decades`, `subdivide`). `func` takes its rule as an argument: `TickRule::Transformed`, `TickRule::Data` or `TickRule::custom(…)`. Any continuous scale accepts `.tick_rule(…)` to replace its transform's rule; it is not `.ticks(…)`, because scales already have d3's `ticks(count)` and Rust has no overloading (E0592).

Every scale and tick rule is in the core; helpers ship none.

| Scale | Default tick rule |
| --- | --- |
| `linear`, `identity` | d3's linear ticks |
| `pow`, `sqrt` | d3's linear ticks in data space |
| `log` | d3's: 1–9 per decade, or decades only when that exceeds the count |
| `symlog` | linear under two decades past the constant, thinned decades above |
| `logit` | 0.5 and decades towards 0 and 1 |
| `asinh` | decades, as matplotlib's AsinhScale |
| `func(f, g, rule)` | the caller's rule |
| `time`, `utc` | d3's time intervals, labeled by the locale's ladder: ISO by default, d3's under `EN_US` |
| `sequential`, `diverging` and their log, pow and symlog variants | as their transform; the ticks label colorbars |
| `band`, `point`, `ordinal` | the domain keys |
| `quantize`, `quantile`, `threshold` | the thresholds |

**symlog ticks by decades, not linearly as in d3.** None of the 23 tests in d3-scale's symlog-test.js checks ticks. Measured as the largest share of the axis without a tick:

| Domain | d3 | matplotlib 3.11 | Ours |
| --- | --- | --- | --- |
| \[−1000, 1000\] | 11 ticks, 38% | 9, 17% | 9, 17% |
| \[0, 10⁶\] | 11 ticks, 83% | 8, 17% | 8, 17% |
| \[−5, 50\] | 12, 31% | 4 (−1, 0, 1, 10) | 12, 31% |
| \[−3, 3\] | 13, 15% | 3 (−1, 0, 1) | 13, 15% |
| \[−10⁹, 10⁹\] | 11, 46% | 21, 6% | 11, 11% |

**What an axis needs is a trait.** d3 duck-types `scale.ticks`, `scale.tickFormat` and `scale.bandwidth`. Here `scale::Ticks` names it, and axes at both levels are generic over it:

```rust
pub trait Ticks {
    /// What ticks mark: f64, a band key,
    /// a Timestamp.
    type Domain: Clone;
    /// Chosen by the scale: f64 (continuous,
    /// binned), time::Interval (time),
    /// () (band, point, ordinal).
    type Step: Clone;

    fn tick_set(
        &self,
        count: usize,
    ) -> ticks::Set<Self::Domain, Self::Step>;
    /// Along the range; None outside the
    /// domain; in [0, 1] for a color range.
    fn position(
        &self,
        value: &Self::Domain,
        offset: f64,
    ) -> Option<f64>;
    /// One label per value, as one batch.
    fn labels(
        &self,
        ticks: &ticks::Set<Self::Domain, Self::Step>,
        locale: &Locale,
    ) -> Vec<String>;
    fn minor(
        &self,
        _ticks: &ticks::Set<Self::Domain, Self::Step>,
    ) -> Vec<Self::Domain> {
        Vec::new()
    }
    /// First and last range value;
    /// [0, 1] for a color range.
    fn extent(&self) -> [f64; 2];
}
```

A `ticks::Set` holds the tick values, the step they were chosen with and the requested count; `with_values` replaces the values and keeps the rest, as d3's `tickValues` still formats with `tickFormat(tickArguments)`. Three things shape the trait:

- **Labels are a batch over the whole set.** The neutral locale groups an axis all or nothing, so it needs every label at once:

  ```rust
  let a = scale::linear().domain([0.0, 8000.0]);
  a.labels(&a.tick_set(5), &Locale::NEUTRAL);
  // → "0" "2000" "4000" "6000" "8000"
  let b = scale::linear().domain([0.0, 12000.0]);
  b.labels(&b.tick_set(5), &Locale::NEUTRAL);
  // → "0" "2 000" "4 000" … "12 000"
  ```

- **The step and count travel with the values.** Log and `.values` ticks are not uniform, so a label format cannot be derived from neighbouring values, and d3's log `tickFormat(count)` keeps a label only for `i ≤ k` with `k = max(1, base · count / n)`. Over `log().domain([1, 100])` at count 10 the labels are 1, 2, 3, 4, 5, four empties, 10, 20, 30, 40, 50, four empties, 100.
- **The scale chooses the step type, not the domain type.** A step keyed by the domain type would need an impl for every band key, `impl<K: Display> … for K`, and that overlaps the impls for `f64` and `Timestamp`, which are `Display` too. An associated type on the scale has no such overlap.

The method is `tick_set`, not `ticks`, because `Continuous::ticks(count) -> Vec<f64>` is d3's and an inherent method shadows a trait method. `Ticks` is implemented for every `Continuous<T, f64, N>` and `scale::Identity` (`Step = f64`), `Band<K>`, `Point<K>` and `Ordinal<K, f64>` with `K: Clone + PartialEq + Display` (`Step = ()`), the binned scales (`Step = f64`), `time::Time` (`Step = time::Interval`), and `Sequential`, `Diverging` and `Linear<Rgb>`, whose positions and extent are normalized to [0, 1] so a colorbar maps them to pixels.

**Users extend ticks at three depths.** Tick positions alone: `.values(…)` on a figure axis, or `.tick_rule(TickRule::custom(f))` on any continuous scale. A new transform: implement `scale::Transform`, usually calling a `ticks::` rule, and get apply, invert, nice, rescale, `pan_px` and `zoom_px`, padding, axes and the figure table with it. A new kind of scale, such as a broken axis: implement `Ticks` for axes, `Position` for marks and the object-safe `Scale` for the figure's table, as band and time scales do.

```rust
/// Frequency laid out by period.
#[derive(Clone, Copy)]
struct Reciprocal;

impl scale::Transform for Reciprocal {
    fn forward(&self, x: f64) -> f64 {
        1.0 / x
    }
    fn inverse(&self, y: f64) -> f64 {
        1.0 / y
    }
    fn ticks(
        &self,
        d: [f64; 2],
        n: usize,
    ) -> Vec<f64> {
        ticks::log(10.0, d, n)
    }
}

let f = scale::continuous(Reciprocal)
    .domain([1.0, 1e6]);
// ticks: 1 10 100 … 1000000
```

`minor` can be replaced because the default does not suit log-like scales: subdividing symlog's decades in transformed space gave minor ticks at 0.149, 0.32, 0.516 and 4.562, so log and symlog provide 2–9 per decade. Logit and asinh use the default subdivision in v0.1 (matplotlib's rules for them: see Later). Axes draw minor ticks with `.minor(true)`.

## Shapes and PathContext

Every generator writes into one trait, d3's canvas path interface, so one line or arc can become SVG, an iced path, or anything else:

```rust
pub trait PathContext {
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn quadratic_curve_to(
        &mut self,
        x1: f64, y1: f64,
        x: f64, y: f64,
    );
    fn bezier_curve_to(
        &mut self,
        x1: f64, y1: f64,
        x2: f64, y2: f64,
        x: f64, y: f64,
    );
    /// A negative radius draws nothing.
    fn arc(
        &mut self,
        x: f64, y: f64, r: f64,
        start: f64, end: f64,
        anticlockwise: bool,
    );
    fn close_path(&mut self);
    /// d3's rect: move, three lines, close.
    fn rect(
        &mut self,
        x: f64, y: f64, w: f64, h: f64,
    ) { /* provided */ }
}
```

`SvgPath` implements it and prints what `d3.path()` would, including `M{x},{y}h{w}v{h}h{−w}Z` for `rect`, and `SvgPath::with_digits(n)` is d3's `pathRound`. d3 throws on a negative arc radius; a trait method returning `()` cannot, and the canvas API rejects the call too, so `arc` draws nothing, and `SvgPath::try_arc` returns `Err(path::Error::NegativeRadius(r))` for callers that want d3's error. The `rect` method is provided, so every context gets d3's `symbolSquare` and other users of `rect` without writing it.

For iced, a wrapper draws into the canvas path builder with no intermediate allocation:

```rust
pub struct IcedPath<'a> {
    /* the builder, the current point,
       and the start of the subpath */
}
impl<'a> IcedPath<'a> {
    pub fn new(
        builder: &'a mut path::Builder,
    ) -> Self;
}
impl PathContext for IcedPath<'_> {
    // f64 to f32 at the builder; arcs as
    // cubics of at most π/2 each
}

let line = path(|p| {
    shape::line()
        .curve(Curve::MonotoneX)
        .draw(points, p)
});
let stroke = Stroke::default()
    .with_color(c)
    .with_width(1.5);
frame.stroke(&line, stroke);
```

`path` builds a `canvas::Path` from anything that draws into a `PathContext`. A wrapper is required: implementing `PathContext` directly for iced's `Builder` from a third crate fails with `error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate`. So it stays in the iced crate, which keeps iced types out of the core. Its fields are private and it segments arcs itself, because iced's `Builder::arc` calls `ellipse`, which starts with a `move_to` (`graphics/src/geometry/path/builder.rs:104–118`): a d3 pie slice (move, arc, line, arc, close) would split into subpaths and fill wrong, and the builder exposes no current point. `IcedPath` tracks the current point and the subpath start (for `close_path`) and emits each arc as d3 defines it, in cubics with control distance `4/3 · tan(θ/4) · r`; a donut slice is exactly one subpath.

**Curves are an enum**, because d3's curve factories are a closed, well-known set. `Curve::Step` reproduces d3's output exactly:

```rust
let pts = [
    (0.0, 0.0),
    (1.0, 2.0),
    (2.0, 1.0),
    (3.0, 3.0),
];
svg(|p| {
    shape::line()
        .curve(Curve::Step)
        .draw(pts, p)
});
// → M0,0L0.5,0L0.5,2L1.5,2
//   L1.5,1L2.5,1L2.5,3L3,3
```

**Areas take `(x, y0, y1)` items**, with `None` breaking the area. The output matches `d3.area()`:

```rust
let rows = [
    (0.0, 10.0, 4.0),
    (1.0, 10.0, 2.0),
    (2.0, 10.0, 5.0),
];
svg(|p| {
    shape::area()
        .draw(rows.map(Some), p)
});
// → M0,4L1,2L2,5L2,10L1,10L0,10Z
```

**Pie slices carry indices, not a reference to the datum.** d3's arc datum embeds `data` and an `index` that is the slice's drawing order. A `shape::Slice` keeps both as positions: `datum` is where the value sits in the input, and `index` is d3's drawing order (descending by value by default). Indices avoid tying the slices' lifetime to the data and work for hit-testing. `layout` returns slices in input order, so `slices[i].datum == i`:

```rust
let slices = shape::pie()
    .layout([3.0, 1.0, 2.0]);
let donut = shape::arc()
    .inner_radius(40.0)
    .outer_radius(80.0);
// slices[0]: datum 0, index 0, angles 0..π
// slices[1]: datum 1, index 2, angles 5π/3..2π
// donut.centroid(&slices[0]) == (60.0, 0.0)
```

**Stacks take columns.** d3's `stack().keys([…])` reads dynamic keys from row objects, which has no typed equivalent. Columns are what Polars and most Rust data already look like:

```rust
let apples = [3.0, 2.0, 1.0];
let bananas = [1.0, 1.0, 4.0];
let layers = shape::stack(&[&apples, &bananas]);
// layers[1], bananas on top of apples:
// [[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]]
```

`shape::Stack` adds d3's orders and offsets; `shape::Symbol` draws d3's seven fill symbols. d3's link generators and its stroke symbols are not in v0.1 (see Later).

**Any `PathContext` works**, including test doubles. A counter shows `Curve::Basis` emits 6 commands for 4 points, which is a cheap way to unit-test curve logic without strings.

## Data and identity

The core never needs to know the user's data type. In d3, data is bound to DOM nodes and accessors read it back; with iterator-first generators, data stays in the application's own typed structs and only `(x, y)` geometry crosses into core. Generic code appears only where identity matters: keys for joins, categories for band and ordinal scales.

**`selection.data(data, key)` becomes a keyed diff.** Since iced rebuilds the view from state every time, the DOM's job of remembering what is on screen falls away. Only animation needs to know which marks are new, kept or leaving:

```rust
join::join(["a", "b", "c"], ["b", "c", "d"]);
// → Join {
//       enter: ["d"],
//       update: ["b", "c"],
//       exit: ["a"],
//   }
```

Duplicate keys follow d3's keyed data join: the first occurrence in the new keys pairs with the first unmatched one in the old; further copies enter or exit. `anim::Marks<K, T>` builds on it to hold one animated value per key through enter, update and exit (use case 2).

**Results point back with indices.** Pie slices, hit tests and stack layers return positions into the caller's slice rather than references. That keeps generator output free of lifetimes and lets it be stored in application state between frames.

**Wide data is columns.** Where d3 reads `d[key]` from row objects with string keys, the Rust API takes one slice per series, which matches Polars columns and avoids string lookups in hot loops.

**Long data is a recording.** A sensor log is many channels sampled on one clock, often millions of samples, often still growing. `series::Recording<X>` holds one `series::Clock` and any number of `f64` channels; NaN is a gap. The clock is `Clock::Regular { start, period }` or `Clock::Explicit(Vec<X>)`, and `X` is `f64` or a jiff `Timestamp` (with a `SignedDuration` period). Channels are stored in chunks of 65,536 samples, each allocated once at full capacity:

```rust
// 10 Hz, two channels on one clock.
let mut rec = Recording::new(
    Clock::Regular { start: 0.0, period: 0.1 },
    [&speed, &torque],
)?;
// Every channel or none; a regular
// clock takes no x values.
rec.extend(&[], [&more_speed, &more_torque])?;

// With 200,000 samples, drop what lies
// before sample 150,000, in whole chunks:
rec.drop_front(150_000);  // → 131,072
rec.len();                // → 68,928
rec.x(0);                 // → Some(13107.2)

let s = rec.series(0).expect("channel 0");
```

`extend` validates lengths, order and the clock before it touches storage, so a failed append leaves the recording as it was, and errors carry the rejected value: an explicit x of `[0, 1, 0.5]` gives `Unordered { index: 2, previous: 1.0, next: 0.5 }`. A regular clock keeps its numbering across `drop_front`, so sample 0 above sits at 131,072 · 0.1. Each recording has an id and a revision that every append and drop bumps; caches key their output by them. Behind each channel sits an exact M4 index (blocks of 64 samples, power-of-two levels) that answers per-column first, minimum, maximum and last within each NaN-free run, with an optional validity predicate such as `y > 0` under a log scale (see Performance).

`rec.series(i)` returns a `Series<'_, X>`: a `Copy` view of one channel, `None` for a channel that does not exist. Marks and helpers take any `series::Data`, which lives in the core with three impls:

```rust
pub trait Data<'a, X, Y> {
    fn into_points(self) -> Points<'a, X, Y>;
}

// Collected: small, computed or filtered data.
impl<'a, X, Y, I> Data<'a, X, Y> for I
where
    I: IntoIterator<Item = (X, Y)>,
{ /* owned */ }

// Indexed: one channel, the view by value.
impl<'a, X: Coordinate> Data<'a, X, f64>
    for Series<'a, X>
{ /* borrowed, index included */ }

// Borrowed columns without an index.
impl<'a, X, Y> Data<'a, X, Y>
    for series::Slices<'a, X, Y>
{ /* series::slices(&x, &y) */ }
```

`Points` is opaque; it yields its pairs and, for a recording, the indexed view that thinning and `Fit::Visible` read. Why this shape:

- **The trait must live beside `Series`.** With `Data` in the figure crate and `Series` in the core, the blanket iterator impl and an impl for `&Series` fail with E0119: `&` is fundamental only for the crate that owns `Series`, so from the figure crate the compiler must assume that upstream crates may add an `IntoIterator` impl for `&Series`. In the core, `Series` and `Slices` are local types without `IntoIterator`, so the three impls do not overlap.
- **A view passed by value works from `view`.** iced's `view(&self)` borrows the application state; `line(self.rec.series(0).expect("channel 0"), t, v)` borrows the recording for the figure's lifetime and copies nothing, so a rebuild costs O(panels), not O(points).
- **x can be a `Timestamp`.** Time computes in i128 nanoseconds; an `f64` epoch would lose precision (see Formats, time and color).
- **Channels share x.** One clock serves every channel, so column boundaries are computed once per frame for all of them (38 µs at 1M samples, 112 µs at 10M).
- **Appends never copy.** A growing `Vec` copies every sample when it doubles and briefly holds both; full chunks never move.
- **Ring buffers are whole-chunk drops.** `drop_front` removes the chunks before a sample and rebuilds the index from what remains.

One cost remains: `line(&pairs)` fails with E0271, because a borrowed `Vec` yields references; write `pairs.iter().copied()`, or `series::slices(&x, &y)` for two columns. An impl for `&[(X, Y)]` would overlap the blanket impl.

## Formats, time and color

**Formats parse at runtime or compile time.** A runtime specifier returns a `Result`; a literal can be checked by the compiler through a `const fn` grammar check inside a small macro:

```rust
let money = fmt!("$,.2f");
money.format(-1234.5);  // −1234.50
// −$1,234.50 under format::Locale::EN_US

fmt!(".3s")
    .format(0.000_042);  // 42.0µ

format::format(".1%")?
    .format(0.123);  // 12.3%

format::format("10..f");
// → Err(InvalidFormat("10..f"))
```

`fmt!("10..f")` fails to compile with `error[E0080]: evaluation panicked: invalid d3 format specifier: 10..f`. It is one of two public macros, with `#[derive(Interpolate)]`.

**Numbers default to SI style, not to d3's en-US.** d3's default locale writes 10000 as "10,000", which reads as ten in German. The neutral default follows the SI rule (CGPM 2003, Resolution 10): a point as decimal marker, digits grouped in threes by a narrow no-break space (U+202F, 0.20 em in DejaVu Sans against 0.32 em for a comma, and never split by a line break), no currency symbol, and − (U+2212) for minus, as d3 already uses. SI leaves four-digit numbers unsplit; applied label by label that mixes styles on one axis (8000 beside 10 000), so an axis groups all its labels when any has five or more integer digits and none otherwise, while single values such as tooltips group from five digits. Digits after the point are not grouped. d3's en-US is a named locale like any other.

| Locale | `,.1f` of 1234567.5 | ticks 0–8000 | ticks 0–12000 | `$,.2f` of 1234.5 |
| --- | --- | --- | --- | --- |
| Neutral (default) | 1 234 567.5 | 2000 … 8000 | 2 000 … 12 000 | 1234.50 |
| en-US (d3's default) | 1,234,567.5 | 2,000 … 8,000 | 2,000 … 12,000 | $1,234.50 |
| de-DE | 1.234.567,5 | 2.000 … 8.000 | 2.000 … 12.000 | 1.234,50 € |
| fr-FR | 1 234 567,5 | 2 000 … 8 000 | 2 000 … 12 000 | 1 234,50 € |

**One `Locale` holds both halves, in the core.** `locale::Locale { number, time }` pairs `format::Locale`, d3's formatLocale (decimal, thousands, grouping, currency, numerals, percent, minus), with `time::Locale`, its timeFormatLocale plus a tick-label ladder. It lives in the core because core's `Ticks::labels` formats with both halves; the figure crate re-exports it. Its constants are `NEUTRAL` (the default), `EN_US` and the ten languages of the time tables; number tables for those are converted from d3-format's 59. A figure applies its locale to the formats it creates (default tick formats, legends, tooltips); a user's own `.format(|v| …)` stays the user's.

```rust
fig.locale(Locale::DE_DE);

// core level: always explicit
let f = format::Locale::DE_DE
    .format(",.1f")?;
```

**Time takes an explicit zone** and builds on jiff. Ticks across the end of daylight saving time in Berlin pick d3's 3-hour interval, labeled by the ISO ladder of the neutral locale:

```rust
let berlin = TimeZone::get("Europe/Berlin")?;
let day: [Timestamp; 2] = [
    "2026-10-24T22:00:00Z".parse()?,
    "2026-10-25T23:00:00Z".parse()?,
];
let st = time::time(berlin)
    .domain(day)
    .range([0.0, 800.0]);
st.tick_interval(8);  // Hour(3)
st.labels(&st.tick_set(8), &Locale::default());
// → "10-25" "03:00" "06:00" "09:00"
//   "12:00" "15:00" "18:00" "21:00"
//   "10-26"
// EN_US, d3's ladder:
// → "Oct 25" "03 AM" "06 AM" "09 AM"
//   "12 PM" "03 PM" "06 PM" "09 PM"
//   "Mon 26"
```

**The ladder belongs to the time locale and is chosen per tick, as d3's `multi`.** Each tick, in the scale's zone, is labeled at the finest unit it is not aligned to: a tick off the whole second gets a sub-second rung (nanosecond, microsecond or millisecond, by that tick's own fraction), a tick on a whole second but not a whole minute gets the second rung, and so on through minute, hour, day or week, month and year. The ISO ladder is `%Y`, `%Y-%m`, `%m-%d`, `%H:%M`, `%H:%M:%S`, `%S.%L`, `%S.%f`, `%S.%N`; d3's is `%Y`, `%B`, `%b %d`, `%a %d`, `%I %p`, `%I:%M`, `:%S`, `.%L`, `.%f`, `.%N`. `time::Locale::NEUTRAL` (English names, a 24-hour clock, `%c %x %X` as `%F %T`, `%F` and `%T`) and the ten generated tables use the ISO ladder; `time::Locale::EN_US`, d3's default definition, uses d3's, so d3's default-locale time tests run under it. A ladder is not a second knob beside `fig.locale`: other tables come from `time::Locale::new(time::Definition { …, ladder: time::Ladder::D3 })`. `time::Locale` is `Clone` but not `Copy`, since its names are `Cow<'static, str>`.

jiff also enforces a distinction d3 blurs. Prototype code that wrote `Timestamp + 1.day()` panicked at runtime, because a day is a calendar unit and is not always 24 hours. Timestamps take absolute units (`24.hours()`); calendar arithmetic goes through `Zoned`, which is what d3's `timeDay` does implicitly with local `Date`s.

**d3-time-format's directive table is ported over jiff; jiff does the calendar math.** jiff's `strftime` differs from d3 in more than the four directives that obviously clash, `%L` (d3: milliseconds), `%f` (d3: microseconds; jiff: a variable fraction), `%Q` (d3: epoch milliseconds; jiff: IANA zone name) and `%Z` (d3: numeric offset; jiff: zone abbreviation, `PST` against d3's `-0800`). Measured with jiff 0.2.38, `%y` of year −2 prints `-2` (d3 `-02`) and `%Y` prints `-002` (d3 `-0002`, after taking the year mod 10000). So the core ports d3's table, and jiff supplies zone conversion and civil fields, once per formatted value. `%U %W %V %g %G %u %w %j` use d3's formulas on our own intervals, so d3's edge cases (`%V` is 53 on 2016-01-01) match by construction. `%N` prints nanoseconds in nine digits, jiff's meaning; d3 has none. Unknown directives print as d3 prints them. Parsing d3 format strings is not in v0.1 (see Later): `time::iso_parse` is jiff's RFC 3339 parser and returns `Result<Timestamp, jiff::Error>`, and other layouts parse with jiff's `strtime`.

**Sub-millisecond time works, unlike in d3.** JavaScript `Date` stops at 1 ms; jiff's `Timestamp` holds nanoseconds, and its zones come from the IANA database. Beside d3's calendar intervals, the interval enum has `Millisecond(k)`, `Microsecond(k)` and `Nanosecond(k)`, with tick formats to match, which kHz sensor recordings need. matplotlib resolves 1 µs, so the port goes past both. Time scales compute in i128 nanoseconds from the domain start, so no `Timestamp` passes through an `f64` epoch value, which in 2026 resolves only 256 ns. d3's `timeTicks`, `timeTickInterval`, `utcTicks` and `utcTickInterval` are the free functions `time::{ticks, ticks_with, tick_interval, utc_ticks, utc_tick_interval}`, each taking its zone.

**Colors are value types** with conversions through `From`, parsing through `FromStr`, and interpolation through the same `Interpolate` trait the scales use:

```rust
let hsl = Hsl {
    h: 200.0,
    s: 0.6,
    l: 0.5,
    opacity: 1.0,
};
Rgb::from(hsl);       // rgb(51, 153, 204)
steel.brighter(1.0);  // rgb(100, 186, 255)

"#zzz".parse::<Rgb>();
// → Err(ParseColorError("#zzz"))

Rgb::hex(0x000000)
    .lerp(&Rgb::hex(0xffffff), 0.5);
// → rgb(128, 128, 128)
```

`Lab`, `Hcl` and `Cubehelix` join `Rgb` and `Hsl` with d3's own conversions, and the interpolators in `interpolate` (`rgb`, `rgb_gamma`, `hsl`, `lab`, `hcl`, `cubehelix` and their long-hue forms, `rgb_basis`, `piecewise`) take the target space's type, which replaces d3's string dispatch. d3's rules for undefined hue stay: `interpolate::hcl(white, steelblue)(0.5)` is rgb(138, 194, 248), because white has no hue.

Categorical schemes are `const` arrays (`scheme::CATEGORY10`, built with the `const fn Rgb::hex`), and sequential, diverging and cyclical schemes are plain `fn(f64) -> Rgb` interpolators, named as d3's camel case lowered to snake case (`scheme::rd_bu`, `scheme::yl_or_rd`). They are d3's own code, exact in `f64` (see Dependencies): the ColorBrewer schemes run d3's basis spline over the 9- or 11-stop rows generated from d3's sources, `turbo` and `cividis` are d3's polynomials, `sinebow` and `rainbow` d3's formulas, `warm` and `cool` go through `cubehelix_long`, and the viridis family keeps d3's 256-entry tables, so `scheme::viridis(0.5)` is #21918c.

## Interaction and animation

d3's behaviors attach listeners to DOM nodes, keep state on them and dispatch named events. Here each behavior is a state machine in core that takes renderer-agnostic pointer input, keeps only the gesture state (such as where a drag started) and returns the new transform or selection; the iced crate translates iced events, and the application decides what the new state means.

```rust
#[non_exhaustive]
pub enum Pointer {
    Pressed {
        x: f64, y: f64,
        button: Button,
        modifiers: Modifiers,
    },
    Moved { x: f64, y: f64 },
    Released { button: Button },
    Wheel {
        x: f64, y: f64,
        delta: Delta,
        modifiers: Modifiers,
    },
    DoubleClicked { x: f64, y: f64 },
    Modifiers(Modifiers),
    Left,
}

/// The DOM's convention: positive y
/// scrolls down, positive x right.
pub enum Delta {
    Lines { x: f64, y: f64 },
    Pixels { x: f64, y: f64 },
}
```

Each variant carries what iced reports and d3's behaviors read:

| Input | Why it has this shape |
| --- | --- |
| `Pressed { button, modifiers }` | d3's filters drop secondary buttons and ctrl-presses (`d3-zoom/src/zoom.js:13–15`; d3-drag and d3-brush alike) |
| `Released { button }` | the held button ends a gesture wherever the pointer is, as d3 listens on the window; iced's release has no position |
| `Wheel { delta, modifiers }` | iced's `ScrollDelta` is lines or pixels, on both axes; d3-zoom's `defaultWheelDelta` scales a line by 0.05 and a pixel by 0.002, ten times that with ctrl (`zoom.js:34–36`) |
| `Modifiers` | iced reports `ModifiersChanged` as its own event, and `DoubleClicked` carries none, while d3-zoom's double-click reads shift to zoom out (`zoom.js:303–314`) |
| `Left` | iced's `CursorLeft`; it ends a wheel anchor, while a drag continues until release |

d3 ignores a wheel's horizontal delta; here it pans, by 25 px per line (d3's own line-to-pixel ratio, 0.05 / 0.002) and 1 px per pixel, so a trackpad's sideways swipe moves the chart. `Pointer` is `#[non_exhaustive]`, so a keyboard event can be added later without breaking matches; matches in other crates carry the wildcard arm the compiler asks for.

**rustd3-iced turns iced events into `Pointer` with a stateful `pointer::Tracker`**, kept in the app's `canvas::Program::State`: `tracker.translate(&event, bounds, cursor)`. Positions are relative to the bounds, and a drag continues outside them; leaving without a button held gives one `Left`. Wheel deltas are negated (iced and winit scroll up for positive y, d3's `deltaY` is positive down) and pixel deltas divided by the scale factor, since iced passes physical pixels. Modifiers are remembered for the next press or wheel. A double click needs time, and library code reads no clock, so a press waits for the next `RedrawRequested(Instant)`; the tracker then applies iced's own rule (same button, at most 300 ms and under 6 logical px apart, `core/src/mouse/click.rs:73–84`), and the app requests that redraw while `tracker.wants_redraw()` is true.

**Zoom** splits into a value and a behavior. `interact::Transform { k, x, y }` is d3's `ZoomTransform`; `Zoom` remembers only the gesture in progress and its limits:

```rust
let mut zoom = Zoom::default()
    .scale_extent([1.0, 50.0]);
let wheel = Pointer::Wheel {
    x: 450.0,
    y: 150.0,
    delta: Delta::Lines { x: 0.0, y: -20.0 },
    modifiers: Modifiers::NONE,
};
if let Some(t) = zoom.handle(wheel, current) {
    // k = 2^(20 · 0.05) = 2, around the pointer.
    // d3's transform.rescaleX(x):
    let visible = full
        .rescale_x(&t)
        .get_domain();
}
```

`Zoom` ports d3-zoom's wheel (anchored at the pointer), drag-to-pan, double-click (×2, ×0.5 with shift; the app animates if it wants, d3's 250 ms transition is not built in) and `defaultConstrain`, with d3's default scale extent `[0, ∞]`; the `Plot` widget gives each handle's zoom `[1e-6, 1e6]` and resets touched handles on double-click instead (see iced integration). One difference from d3 follows from reading no clock: the wheel anchor ends at the next non-wheel event or a moved pointer, not after 150 ms idle.

**`Rescale` names what d3 gets for free from `invert` and `copy`:**

```rust
pub trait Rescale: Sized {
    /// d3's transform.rescaleX(scale).
    fn rescale_x(&self, t: &interact::Transform) -> Self;
    fn rescale_y(&self, t: &interact::Transform) -> Self;
    /// Content follows the pointer by dx
    /// range units (pixels).
    fn pan_px(&self, dx: f64) -> Option<Self>;
    /// Scales by factor around the range
    /// position anchor; > 1 zooms in.
    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self>;
}
```

It is implemented for every `Continuous<T, f64, N>`, `scale::Identity` and `time::Time`. `rescale_x` and `rescale_y` are d3's, through `invert`. The pixel-space operations are what gestures use: they work in the scale's transformed space and in its own precision, so a time scale pans and zooms in i128 nanoseconds. An `f64` span loses `Timestamp` precision, about 240 ns in 2026, where epoch nanoseconds near 1.79·10¹⁸ sit 256 ns apart. A step is rejected with `None`, and the gesture keeps its transform, when a new stop is not finite, the orientation flips, the factor is not finite and positive, or the span falls below 2⁻⁴⁰ · |center| (time: 1 ns):

```rust
let x = scale::linear()
    .domain([0.0, 100.0])
    .range([0.0, 960.0]);
x.zoom_px(480.0, 2.0);  // Some: domain [25, 75]
x.pan_px(96.0);         // Some: domain [−10, 90]

scale::linear()
    .range([0.0, 960.0])
    .zoom_px(480.0, 1e13);
// → None: a span of 1e−13 is below
//   2⁻⁴⁰ · 0.5 = 4.55e−13
```

**`scale::Pad` fits domains in each scale's own precision.** `pad(extent, margin, sticky)` takes the merged data extent, a margin as a fraction of the span per side and sticky values the margin must not cross, and returns the padded domain. It is matplotlib's `autoscale_view` per axis: margins in transformed space, `nonsingular` widening of a flat or empty extent (expander 0.05; no data gives [0, 1], or [1, 10] on a log scale), sticky edges, and the domain's orientation kept. `linear` pads `Some([0, 10])` at 0.05 to [−0.5, 10.5] and `Some([0, 4])` with sticky `[0]` to [0, 4.2], so bars keep their baseline; `log` pads `Some([1, 1000])` to [0.7079457843841379, 1412.5375446227545]. Time scales pad in i128 nanoseconds. The figure's scale table delegates its gestures and fitting to `Rescale` and `Pad` (see Figures).

**Brush** is the same pattern. `BrushX::handle` returns d3's `start`, `brush` and `end` as an enum, and `set_selection` is `brush.move`:

```rust
match brush.handle(input)? {
    BrushEvent::Brush(Some([a, b]))
    | BrushEvent::End(Some([a, b])) => {
        Message::Show([
            x.invert(a),
            x.invert(b),
        ])
    }
    BrushEvent::End(None) => Message::Show(full),
    _ => return None,
}
```

`BrushX` and `BrushY` port d3-brush's one-dimensional brushes: a press on the overlay starts a selection, on the selection drags it, on a handle resizes it with flips; a selection of zero width ends as `None`; `set_selection` emits no events, since the caller already knows; `is_active` tells an app to sync the brush only between gestures (use case 1). `Modifiers` events are accepted and the ctrl-press filter applies, but d3-brush's keyboard modes (shift locks an axis, alt centers, meta starts anew) and the two-dimensional brush are not in v0.1 (see Later); `BrushEvent<S = [f64; 2]>` keeps a type parameter so the 2-D form can be added. d3-brush has no behavior tests, so traces transcribed from its source (`brush.js:432–580`) grade the port.

**Drag** is d3-drag: `drag.handle(input, |x, y| subject_at(x, y))` asks for the subject on a press (`None` ignores it), adds the subject's offset to every position, and reports `drag::Event::Start`, `Drag { dx, dy }` and `End { moved }`, where `moved` latches once the pointer travels beyond `click_distance`.

**Transitions are values sampled at an `Instant`.** `Animated<T: Interpolate>` stores where it came from, where it is going, when and how. Retargeting starts from the current value, which is exactly how an interrupted d3 transition behaves. `Timing` mirrors `transition.duration().delay().ease()`, plus a per-item `stagger` for d3's `delay((d, i) => i * 20)`:

```rust
let timing = Timing {
    duration: Duration::from_millis(750),
    stagger: Duration::from_millis(20),
    // no delay, cubic in-out: d3's defaults
    ..Timing::default()
};
```

The host drives the clock: the application subscribes to `window::frames()` only while something is animating, stores the latest `Instant`, and samples values in `view`. iced master has its own `Animation<T>` (built on lilt), but it requires `Copy` float-like states; core keeps its own so it stays iced-free and can animate any `Interpolate` type.

## Errors, numerics and naming

| Situation | d3 | Proposal |
| --- | --- | --- |
| Invalid number in a numeric pipeline | NaN propagates | NaN propagates, as d3 does; `ticks(NaN, 1, 10)` is empty |
| Unknown category | `undefined` | `Option` |
| Malformed specifier, color or date string | throws, or returns null | `Result`, or a compile error for literals |
| Negative arc radius | throws | `arc` draws nothing; `SvgPath::try_arc` returns `Err` |
| Impossible operation (invert a color scale) | NaN at runtime | Method does not exist for that type |
| Broken internal invariant | – | `expect("invariant: …")`, never `unwrap` in library code |

Each subsystem has its own `thiserror` error, and every error carries the rejected value: `InvalidFormat("10..f")`, `ParseColorError("#zzz")`, `path::Error::NegativeRadius(r)`, `series::Error::Length { channel, expected, found }`.

**Precision.** Geometry is `f64` through the core and the figure layer, and the figure layer clips it in `f64` to a guard band before any conversion. `f32` appears only at backend boundaries (iced, vello\_cpu, tiny-skia, krilla) and in glyph runs; at the core level the conversion happens once, in `IcedPath`, since iced's geometry is `f32`. Time computes in i128 nanoseconds.

**Constructors and builders.** Free functions mirror d3's factories (`scale::linear()`, `shape::line()`, `axis::bottom(&x)` for d3's `axisBottom`). Builder methods are named after d3's setters (`domain`, `range`, `padding`, `curve`, `ticks`).

```rust
axis::bottom(&x)
    .ticks(8)
    .draw(&mut path);
```

`axis::Orient` stays as the enum underneath, for code that picks the side at runtime. A core axis draws its domain path and ticks into any `PathContext` and returns its labels from `.labels()`, each with a position, an anchor and d3's `dy`, for the host's own text; the core never measures text.

**Three kinds of types, one rule each.** Values have no `&mut` setters: rebuild the value instead. Containers that hand out handles and are filled in loops take `&mut`, as `std::process::Command` does. Behaviors keep gesture state between events.

| Kind | Examples | Rule |
| --- | --- | --- |
| Values | scales, generators, axes, marks, legends, `Theme`, `Text` | consuming builders, `fn x(self, …) -> Self` |
| Containers | `Figure`, `Panel` | filled with `&mut self -> &mut Self`, read with `&self` (`resolve`, `draw`, `check`) |
| State machines | `Zoom`, `BrushX`, `BrushY`, `Drag`, `anim::Marks` | `&mut self` per event, returning a value the app stores |

A `series::Recording` is plain owned data: `extend` and `drop_front` take `&mut self` and report through their return values.

A consuming `Figure` was sketched and rejected: every handle comes back in a tuple and the figure is threaded through every loop.

```rust
let (fig, speed) = fig.scale(band(…));
let mut fig = fig;
for (w, r, c) in blocks {
    fig = fig.panel(r, c, |p| {
        p.mark(dots(…))
    });
}
```

Values placed into a container stay values: `bottom(speed).label(…)` is built once and placed on many panels by reference. Making scales mutable instead would let code change a scale already handed out, d3's `x.copy()` pitfall.

**Getters collide with builders.** Rust has no overloading, so `domain()` cannot be both the d3-style setter and the getter. The prototype uses `get_domain()`, which goes against the Rust API guidelines' no-`get_` convention; the alternative is `with_domain(…)` for the builder. Decided: d3's names for builders, and getters only where needed, as `get_<name>`, the exception `std::process::Command` makes (see Settled questions).

**Names follow the module path.** `scale::Linear`, `scale::Band<K>`, `shape::Curve::MonotoneX`, `axis::Orient::Bottom`, `interact::BrushX`, never `LinearScale` or `D3Color`. Where a name exists twice on purpose (`scale::Transform` and `interact::Transform`, `scale::Identity` and `scale::transform::Identity`), code writes the path at every use.

## Figures

**Decision: the figure owns a table of scales, and marks, axes, legends and gestures refer to them through typed handles.** `fig.scale(s)` keeps the domain `s` was built with; `fig.fit(s)` spans the data of every mark that uses the scale, in any panel. Both return `ScaleRef<S>`, a `Copy` index typed by its scale. Two panels share a scale by using the same handle, so there is no share or link API. Crossing groups (x shared per column, y per row), one scale in two roles (a scatter matrix variable is x in its column and y in its row), twin axes (a second scale in one panel) and one palette across several heatmaps all follow from that. It is d3's own idiom, create `x` once and pass it to every axis and mark that should agree, with an index where d3 shares an object. Vega names scales at the top level the same way, and Vega-Lite compiles its resolve rules down to them.

**A shared scale shares its domain, not its range.** Panels differ in pixel size. The resolve pass fits domains to data, applies the domains a gesture holds, lays out the panels, and then gives each panel its own concrete scale: the shared domain with that panel's range.

**Fitting has three modes, typed by the scale's span.** `fig.fit(s)` is `fig.fit_with(s, Fit::All)`: the data of every mark that uses the scale. `Fit::Window(span)` keeps an x handle on the newest `span` of a live recording. Its right end is the last sample rounded up to a whole device column and its left end lies `span` before it, with no margin, so each append moves the view by whole columns, the thinned line's column boundaries shift by whole columns with it, and the line does not shimmer as data streams in; the columns are those of the widest panel that shares the handle. `Fit::Visible` fits a y handle to the data whose x lies in the panel's current x domain, read from a recording's M4 index in O(log n); it grows at once and shrinks in steps of one eighth of its span, so panning does not make it flicker. `Window` on a y handle and `Visible` on an x handle behave as `All`. The span's type is the scale's own: `Kind::Span` is `f64` for numeric scales, `jiff::SignedDuration` for time scales and `()` for categorical ones, so a time window is a duration and a number there is a type error.

```rust
// The newest minute of a live recording, with y
// fitted to what that minute shows.
let t = fig.fit_with(
    time::utc(),
    Fit::Window(SignedDuration::from_secs(60)),
);
let v = fig.fit_with(
    scale::linear(),
    Fit::Visible,
);
```

`fit` and `fit_with` take any `Fittable` scale, and every kind is one except `Quantile`. A quantile scale's domain is the data sample itself, while fitting merges extents and sticky values and never collects samples, so users build it from their data with `fig.scale(scale::quantile(sample, colors))`, and `fig.fit(scale::quantile(sample, colors))` does not compile. Continuous position scales get the theme's margin (see Themes). Color scales fit to the data's extent without one, as matplotlib's `Normalize` does, because a padded color domain wastes ramp. Band, point, ordinal and threshold scales keep the domain they were built with, since their keys or thresholds are the domain.

**The table holds scales behind a trait object; typed code uses small capability traits.** An object-safe `Scale` lets one table hold every kind, including band scales over user key types and continuous scales with user transforms. It carries no numeric span. Its pixel operations hand a copy of the scale, set to the panel's range, to core's `Rescale::pan_px(dx)` and `zoom_px(anchor, factor)`, and fitting pads through core's `scale::Pad`, so each scale works in its own precision. A time scale pans, zooms and pads in i128 nanoseconds: fitted to two timestamps 1 µs apart, it pads each end by exactly 50 ns, where an `f64` epoch in seconds resolves only about 240 ns in 2026. A zoom step core rejects (non-finite, reversed, or a span below 2⁻⁴⁰ of its center; 1 ns for time) returns `None` and leaves the gesture where it was. Band, point and color scales return `None` from both pixel operations, so they ignore gestures. Marks and axes use `Kind` (domain, output and span types), `Position`, `Paint` and core's `Ticks`, and a handle's type recovers the concrete scale at draw time. `AnyContinuous` (see Scales) stays the type a user names when the transform is chosen at runtime.

```rust
/// One table holds every kind.
pub trait Scale: Any {
    fn with_range(
        &self,
        range: [f64; 2],
    ) -> Box<dyn Scale>;
    /// Core's `Rescale::pan_px` on a copy set to
    /// `range`; `None` where gestures do not apply.
    fn pan_px(
        &self,
        range: [f64; 2],
        dx: f64,
    ) -> Option<Box<dyn Scale>>;
    fn zoom_px(
        &self,
        range: [f64; 2],
        anchor: f64,
        factor: f64,
    ) -> Option<Box<dyn Scale>>;
    // as_any, boxed
}

pub trait Kind: Scale + Clone {
    type Domain: Clone + 'static;
    type Output;
    /// `f64`, `jiff::SignedDuration` or `()`.
    type Span: Copy + 'static;
    // admits, merge, fitted (through `scale::Pad`),
    // domain
}

/// Every kind except `Quantile`.
pub trait Fittable: Kind {}

pub enum Fit<Span> {
    All,
    Window(Span),
    Visible,
}
```

**Marks, axes and legends are values.** Marks come from functions, as in Observable Plot: `dots`, `line`, `area`, `ribbon` (rows `(x, y0, y1)`), `ribbon_series` (two channels of one recording, thinned together), `bar_y` and `bar_x`, `bar_y_spans` and `bar_x_spans` (rows `(x, [y0, y1])`), `rect` (rows `([x0, x1], [y0, y1])`, Observable Plot's `rect`), `histogram`, `density_y` and `density_x`, `rule_x` and `rule_y`, `image` for raster heatmaps and `note` for text at a data point. They take options such as `.fill(palette, key)`, `.stroke(palette, key)`, `.dash(…)`, `.dodge()`, `.jitter(…)` and `.thin(Thin::Off)`, and are added with `panel.mark(m)`. An axis is built once per quantity (`bottom`, `left`, `right` or `top`, with `.values`, `.format`, `.ticks`, `.label`, `.outward`, `.rotate(deg)` and `.color(palette, key)`) and placed on every panel that shows it; `.labels(false)` gives the inner-panel version. `.rotate(deg)` turns tick labels counter-clockwise, matplotlib's sign, anchored at their end for positive angles on a bottom axis as `autofmt_xdate`'s 30° does; layout measures and collides the rotated boxes (see Layout). A legend, `legend(palette).title(…)`, goes into a figure-level slot or a corner of a panel.

**Per-point color and size are channels.** `dots(pts, x, y).fill_by(heat, values)` colors each point through a continuous color handle, and `.size_by(radius, values)` sizes it through a numeric handle whose output is a radius in px. Both resolve at draw time into one `Markers` command with a paint table and per-point indices and sizes (see Drawing and output), so a colored scatter stays one command however many points it has, and its colorbar comes from the same handle. A channel whose length differs from the points is an error when the mark is built, never truncated; a point whose value maps to no color or a non-finite size is skipped together with its position.

**Stacking is computed by helpers, not a mark option.** A mark fits and draws on its own, and the figure has no panel-level pass that could see a group of marks, order them and offset each by the ones below. So helpers compute `[y0, y1]` with core's `shape::stack` and emit marks with explicit baselines: one `ribbon` per series for stacked areas, `bar_y_spans` for stacked and diverging bars, and `rect` rows over shared bin edges for stacked histograms (see Helpers). A stacked chart is then ordinary marks that fit both ends and hit-test like any other, and users stack their own data the same way.

Figure axes are built on the core ones. `bottom(speed)` takes a figure handle and is placed on panels; `axis::bottom(&x)` takes the scale itself and draws into a `PathContext`. The shared name follows the module-path rule, and the argument types keep the two from being mixed up.

**Colorbars are the legend of continuous and binned color scales.** `colorbar(handle)` takes a handle whose scale maps numbers to colors: sequential and diverging scales with any transform, quantize, quantile and threshold. An ordinal handle is a compile error, because categories get `legend`. A colorbar goes into a figure slot or a panel corner like any legend, is `em(1)` thick and as long as the panels beside it, and labels itself with its scale's own ticks, tick rule and the figure's locale, so a symlog color scale gets decade ticks. The ramp is one image strip drawn with nearest sampling: one sample per color for d3's 256-entry schemes, 256 samples of an interpolator otherwise, one per bin for binned scales. Checked at every pixel of a 300 px bar, the strip equals the scale exactly for viridis, magma, turbo and RdBu; a 256-stop linear gradient was off by 1.2–3.5/255, and rectangles per bin leave anti-aliased seams between neighbours in some PDF viewers. `.extend(Extend::Both)` adds matplotlib's end triangles for values the scale clamps.

```rust
let heat = fig.scale(
    scale::sequential(
        [0.0, 80.0],
        scheme::viridis,
    ),
);
fig.legend(
    Slot::Right,
    colorbar(heat).title("temp [°C]"),
);
```

**Hatching is clipped line geometry the figure layer builds.** Area marks (bars, histograms, areas, ribbons, densities) take `.hatch(Hatch::lines(45.0))` or `Hatch::cross(45.0)`, 6 pt apart and 0.8 pt wide in the theme's ink by default, and legend swatches repeat it. Each hatched shape becomes a clip group (a rectangle clip for bars, a path clip for areas and densities) around one stroked path of parallel lines anchored to the figure, so the lines continue across neighbouring bars as in matplotlib, and every backend draws plain vector geometry rather than a pattern paint. Hatching 190 of the wheel figure's 232 histogram bars took 0.39 ms to build, grew its SVG by 91 KB and changed raster time within run-to-run noise. Dot and star patterns are out of scope.

**Rich text is a list of spans, and typst renders it.** `Text::from("Wheel: ").push(span("RL").fill(RED))` colors only the wheel name. The figure layer translates it to `#"Wheel: "#text(fill: rgb("#c44e52"), "RL")`, and typst 0.15.1 compiled that with the fill on exactly the two "RL" glyph runs. Each span becomes a string literal, so labels need no escaping beyond `\` and `"`. Raw typst markup stays the entry point for math.

### Worked example: four wheels

This is the case the API must handle. Four wheels, three sources and 20 speed bins: per wheel, a scatter of value over speed bin, its distributions attached on the right and a histogram of counts attached below, giving a 4×4 grid with unequal track sizes, small gaps inside each wheel's block and gutters between blocks.

- Every panel shares the categorical speed scale and its custom ticks.
- Vibration and bearing temperature are twin y scales, shared by the scatters and the distribution plots. The temperature axis sits on the left, pushed outward and tinted like its series.
- The histograms share a count scale and carry a dashed minimum-count rule.
- One palette colors the dots, the dashed regression lines, the distributions, the bars and the legend, in the same order.
- The damaged wheel's name is red in its panel title; the figure title sits above and the legend below, both centered.

The types. Band ticks and legend entries are labeled through `Display`, so the speed bin and the source implement it:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Wheel {
    FL,
    FR,
    RL,
    RR,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Source {
    AccelX,
    AccelZ,
    Temp,
}

/// A 10 km/h speed bin: `Bin(4)` is 40–50 km/h.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bin(u8);

impl fmt::Display for Bin {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let lo = 10 * u32::from(self.0);
        write!(f, "{lo}–{}", lo + 10)
    }
}

impl fmt::Display for Source {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.write_str(match self {
            Source::AccelX => "accel x",
            Source::AccelZ => "accel z",
            Source::Temp => "bearing temp",
        })
    }
}

struct Sample {
    wheel: Wheel,
    source: Source,
    bin: Bin,
    value: f64,
}

const SOURCES: [Source; 3] = [
    Source::AccelX,
    Source::AccelZ,
    Source::Temp,
];

/// seaborn's "deep", one color per source.
const PALETTE: [Rgb; 3] = [
    Rgb::hex(0x4c72b0),
    Rgb::hex(0xdd8452),
    Rgb::hex(0x55a868),
];
const RED: Rgb = Rgb::hex(0xc44e52);
const EDGE: Rgb = Rgb::hex(0x4d4d4d);
const DAMAGED: [Wheel; 1] = [Wheel::RL];
/// Bins with fewer samples are not trusted.
const MIN_COUNT: f64 = 20.0;
```

The figure:

```rust
fn figure(
    samples: &[Sample],
    inset: bool,
) -> Figure<'static> {
    let of = |w: Wheel, s: Source| {
        samples.iter().filter(move |d| {
            d.wheel == w && d.source == s
        })
    };

    let mut fig = Figure::new();

    // One scale per quantity, shared by all.
    let speed = fig.scale(
        scale::band((0..20).map(Bin))
            .padding(0.1),
    );
    let vib = fig.fit(scale::linear());
    let temp = fig.fit(scale::linear());
    let count = fig.fit(scale::linear());
    let width = fig.scale(
        scale::linear().domain([0.0, 1.0]),
    );
    let color = fig.scale(
        scale::ordinal(SOURCES, PALETTE),
    );
    // Handles are Copy: route the twin here.
    let y_of = |s: Source| match s {
        Source::Temp => temp,
        _ => vib,
    };

    // Axes and legends are values: define once,
    // place as often as needed.
    let speed_axis = bottom(speed)
        .values((0..20).step_by(4).map(Bin))
        .format(|b| format!("{}", 10 * b.0))
        .label("speed [km/h]");
    let vib_axis = left(vib)
        .label("vibration RMS [m/s²]");
    // The twin sits left too, tinted like its
    // series; layout stacks it outside `vib`.
    let temp_axis = left(temp)
        .color(color, Source::Temp)
        .format(|t| format!("{t:.0}°"))
        .label("bearing temp [°C]");
    let count_axis = left(count)
        .label("count");
    let key = legend(color).title("source");

    // A 4×4 grid: per wheel, a scatter with its
    // distributions right and histogram below.
    // Gaps sit between decorations; margins and
    // label placement are measured.
    fig.layout(
        Grid::new()
            .cols([1.0, 0.25, 1.0, 0.25])
            .rows([1.0, 0.3, 1.0, 0.3])
            .col_gaps([em(0.4), em(1.6), em(0.4)])
            .row_gaps([em(0.4), em(1.6), em(0.4)]),
    );
    fig.title(
        "Hub vibration and bearing temperature",
    );

    let blocks = [
        (Wheel::FL, 0, 0),
        (Wheel::FR, 0, 2),
        (Wheel::RL, 2, 0),
        (Wheel::RR, 2, 2),
    ];
    for (wheel, r, c) in blocks {
        let id = span(format!("{wheel:?}"));
        let id = if DAMAGED.contains(&wheel) {
            id.fill(RED)
        } else {
            id
        };

        let main = fig
            .panel(r, c)
            .title(Text::from("Wheel: ").push(id))
            .border(EDGE)
            .axis(&vib_axis)
            .axis(&temp_axis)
            .zoom_y(vib)
            .zoom_y(temp);
        for s in SOURCES {
            let pts: Vec<_> = of(wheel, s)
                .map(|d| (d.bin, d.value))
                .collect();
            let fit = trend(&pts);
            main.mark(
                dots(pts, speed, y_of(s))
                    .fill(color, s)
                    .dodge()
                    .jitter(0.8),
            );
            main.mark(
                line(fit, speed, y_of(s))
                    .stroke(color, s)
                    .dash([6.0, 3.0])
                    .dodge(),
            );
        }

        let side = fig
            .panel(r, c + 1)
            .border(EDGE);
        for s in SOURCES {
            let v = of(wheel, s).map(|d| d.value);
            side.mark(
                density_y(v, y_of(s), width)
                    .fill(color, s),
            );
        }

        let below = fig
            .panel(r + 1, c)
            .border(EDGE)
            .axis(&count_axis)
            .axis(&speed_axis);
        for s in SOURCES {
            let bins = of(wheel, s)
                .map(|d| d.bin);
            below.mark(
                histogram(bins, speed, count)
                    .fill(color, s)
                    .dodge(),
            );
        }
        below.mark(
            rule_y(MIN_COUNT, count)
                .dash([4.0, 3.0])
                .color(EDGE)
                .label("min. count"),
        );
    }

    if inset {
        fig.panel(0, 0)
            .legend(Inset::TopLeft, key);
    } else {
        fig.legend(Slot::Bottom, key);
    }
    fig
}
```

`trend` is ordinary least squares of value on the bin index, evaluated at the first and last bin with data, returned as two `(Bin, f64)` points. The examples crate feeds the figure from `simulate()`: 600 drives per wheel, temperature logged every third drive, 5,600 samples in all, and RL's bearing worn above 100 km/h. With `inset = true`, the same legend value goes into a corner of FL's panel instead of the bottom slot. The function is 128 lines of code in this vertical style, blank lines and comments aside. It sets no margins, no per-panel label flags and no axis offsets: margins, which panels get labels, the stacked temperature axis and the tick counts come from the layout pass (see Layout). Every mark owns its data, since each takes filtered or computed values (`density_y` and `histogram` collect their iterators when built), so the result is a `Figure<'static>` that can be returned, stored or drawn to several outputs.

One zoom over RL's scatter rescaled vibration from \[0, 3\] to \[0.37, 1.87\] and temperature from \[30, 65\] to \[34.37, 51.88\], and all four scatters and all four distribution plots followed. Those numbers were measured with fitted domains rounded to nice values, as `Domains::Nice` gives; with the default 5% margins the fitted domains start at \[−0.080, 3.053\] for vibration (dots and lines are not held at zero), \[31.70, 65.65\] for temperature (a trend endpoint at 33.24 lies below the lowest sample) and \[0, 90.3\] for counts (bars are held at zero). Where a histogram meets its scatter, the histogram's top count label would sit on the scatter's lowest label; the layout's collision pass drops it.

**Types catch the common mistakes.**

| Mistake | Compiler error |
| --- | --- |
| Timestamps on a linear handle | E0271, `<Linear as Kind>::Domain == Timestamp` |
| `.fill(color, Wheel::FL)` with a palette over `Source` | E0308, expected `Source`, found `Wheel` |
| `histogram(bins, speed, color)` | E0277, `Ordinal<Source, Rgb>: Position` is not satisfied |
| `on_change(x_time, Msg::Range)` where `Range([f64; 2])` | E0631, expected `fn([jiff::Timestamp; 2]) -> _` |
| An axis on a band scale whose key has no `Display` | E0277, `Bin` doesn't implement `Display` |
| `fig.fit(scale::quantile(sample, colors))` | E0277, `Quantile<Rgb>: Fittable` is not satisfied |

**Gestures write scales, and the iced widget decides who owns them.** `.zoom_y(vib).zoom_y(temp)` makes the mouse wheel rescale both twins together around the cursor, in every panel that uses them; matplotlib's `twinx` only zooms the top axes. In iced, `Plot::new(fig).on_change(vib, Msg::Vib)` turns changes to `vib` into messages that carry the domain in the scale's own type, so the app owns that domain. Without it, the widget keeps the domain in its own state. Either way every panel follows, because they read the same scale. iced can deliver several events before `update` runs, so during a gesture the widget continues from the domain it last published and drops it when the gesture ends; without this, a 120 px pan delivered as three events per update lost half its distance in the sketch. A gesture holds its handles: fitting skips them, a `Fit::Window` handle included, until a double-click resets the touched handles to their fitted domains, so new data does not pull the view away from a stretch the user is looking at. Widget state is keyed by the handle's index and its scale's `TypeId`, which survive the figure's rebuild in every `view`; a rebuilt figure that puts another scale type at that index drops the state rather than misreading it. The widget itself is described in iced integration.

**Marks copy small data and borrow large data.** In the Elm loop the figure is rebuilt on every change. Copying data into marks, as the figure sketch (`proto/figmodel`) does, took 17 ms to build its marks and 14 ms to fit for two panels of 1M points, over the 16.7 ms frame budget, against 0.07 ms and 0.15 ms for 10,000 points. So mark data enters through core's `Data` trait (see Data and identity), whose impls split by size. Any iterator of points is collected into the mark: small, computed or filtered data. A `Series<'a, X>`, the `Copy` view of one channel of a recording, is passed by value and borrowed together with the recording's cached extent and the M4 index that thinning and `Fit::Visible` read (see Performance); fitting it reads the index, not the samples. `series::slices(&x, &y)` borrows two columns without an index and is drawn as given. Two marks of 1M points took 11.3 ms to build copied and 0.0003 ms borrowed, so a rebuild costs O(panels), not O(points).

```rust
fn view(&self) -> Element<'_, Message> {
    let mut fig = Figure::new();
    let secs = fig.fit(scale::linear());
    let accel = fig.fit(scale::linear());
    // A Copy view of channel 0, borrowed with
    // its index: nothing is copied.
    let ch = self.rec.series(0).expect("channel 0");
    fig.panel(0, 0).mark(line(ch, secs, accel));
    Plot::new(fig)
        .on_change(secs, Message::Window)
        .into()
}
```

A figure of owned data is `Figure<'static>` and can be returned or stored. One that borrows a recording is built in iced's `view(&self)`, which already returns `Element<'_>`, and is moved into the widget: `Plot::new(fig)` takes the figure by value, because it is a local of `view` and a widget borrowing it could not be returned (E0515). Borrowing everything was ported onto the wheel figure and rejected: every one of its 52 marks takes filtered or computed data, so borrowing saves no copy, and the figure could then neither be built in one loop (E0502: pushing derived data into storage the figure already borrows) nor be returned from a function (E0515). Borrowed slices get no index, because building one every frame would spend what borrowing saves; long data belongs in a recording, and `fig.check` reports owned marks over 100,000 points. One cost remains: `line(&pairs)` fails with E0271, because a borrowed `Vec` yields references; write `pairs.iter().copied()`. An impl for `&[(X, Y)]` would overlap the blanket impl, while `Series` and `Slices` are core's own types without `IntoIterator`, so their impls coexist with it.

**Settled details**

- Legends go into figure-level slots (top, bottom, left, right) or a corner of a panel, and one value works in both. A top or bottom legend is centered on the grid's plot extent, not the canvas.
- Outer-only labels are automatic: layout sees which panels share a handle on the same edge and labels the outer ones, and `.labels(bool)` sets it explicitly. A second axis stacked on the same side is left off inner panels, where it would land in the neighbor (see Layout).
- `.dodge()` places a mark in its fill or stroke key's slot among the palette's keys, so dots, bars and lines line up in legend order.
- `.jitter(share)` spreads dots across that share of their slot by a hash of the point index, so the same data jitters the same way in every render and on every platform.
- `density_y` draws a Gaussian KDE with Scott's bandwidth, as scipy and seaborn do, over the data rather than the view, peak-normalized on its width scale; the wheel figure's side panels share `width = [0, 1]`.
- Draw order is insertion order, like d3's DOM order; there is no z-index.
- Legend swatches are squares by default; a `.glyph(…)` option shows a dot or a dashed line instead.
- Each figure has an id that handles carry, so a handle from another figure panics in debug builds. Lifetime branding would catch it at compile time but makes handles awkward to store.
- `subplots(rows, cols)` is a helper returning x handles per column and y handles per row, matplotlib's common case (see Helpers).
- Fitted domains get a 5% margin, held at zero where bars and areas start; nice domains are a theme option (see Themes).

Two alternatives were built and rejected: sharing resolved by the layout tree, as in Vega-Lite, and axes that own their scales and are linked afterwards, as in matplotlib and Makie. See Alternatives considered.

## Layout

**Decision: tracks are sized by protrusions, in one pass of maxima and sums.** Makie's GridLayoutBase and ggplot2's patchwork work the same way. The wheel-figure numbers in this list come from the sketch, with 10 px text and estimated widths; typst measurement and the theme's sizes change the pixels, not the method.

1. Each panel measures how far its axes, labels and title reach past its plot area on each side. FL's scatter reaches 94 px to the left (vibration axis 43, a gap of 8, the stacked temperature axis 43) and 18 px up for its title. An axis reaches by its tick, the tick padding, its widest label (its tallest, along a horizontal axis) and, with a title, a label padding and the title's ink height; tick labels that overhang the panel's ends add to the sides they cross.
2. Each column takes the largest reach of its panels on each side, and each row likewise: the first column needs 94 px, the bottom row 37 px for the speed axis.
3. Reaches, gaps and padding are fixed; what remains is split by weight. In the wheel figure, 821 of 944 px go to plot areas, 328.4 px per unit of weight.
4. Positions are running sums, so every panel in a column starts at the same x, and attached panels share their scale's pixel range exactly.

With typst text at notebook size the parts add up as follows on a 640 × 480 figure with one panel, x fitted to 0 … 10 and y to 0 … 3. The left axis reaches 8 px of tick, 4.667 px of padding and 23.325 px for "0.0", its widest label: 35.99 px. The bottom axis reaches 8 + 4.667 + 11.093 px (the digits' ink height) = 23.76 px. With 0.8 em of outer padding the plot area is 580.5 × 432.8 px at (47.7, 11.7).

**Gaps are whitespace between decorated boxes, measured in em.** The number in `col_gaps` and `row_gaps` is the whitespace a reader sees, and the distance between plot areas follows from the reaches: in the sketch, 21 px between FL's side plot and FR's scatter and 39 px between FL's histogram and RL's scatter. `em(1.6)` is 1.6 times the theme's tick-label size, 23.467 px at notebook, so gaps scale with seaborn-style contexts; `px(…)` is there for exact pixels. matplotlib's constrained layout behaves the same, keeping 15.0 px of whitespace while labels grew from 31.1 to 68.0 px; its plain `subplots_adjust` holds the plot distance instead, and the same labels ran 29.5 px into the neighbor.

```rust
Grid::new()
    .cols([1.0, 0.25, 1.0, 0.25])
    .col_gaps([em(0.4), em(1.6), em(0.4)])
```

**Shared axes are labeled once, and axes on one side stack.** An axis is inner when a panel further out in the same row or column shows the same scale on the same side. Inner axes keep their ticks and lose labels and title, and a stacked second axis on an inner panel is dropped. Axes on one side stack outward in insertion order with an 8 px gap. `.labels(bool)` and `.outward(px)` set both explicitly.

**Tick counts follow axis length.** Without `.ticks(n)`, an axis asks for about one tick per 80 px across and per 35 px up, Observable Plot 0.6.17's spacing. Counts change label widths and widths change lengths, so layout repeats until the counts settle: 3 passes for the wheel figure, 2 or 3 for grids up to 16×16, capped at 4. An axis whose tick rule returns no ticks has no label extent.

**Labels that would still collide are dropped, in two passes.** Within one axis, layout keeps every k-th label for the smallest k without overlap, Vega's parity strategy, so spacing stays regular. With typst-measured labels at notebook size, the 20 speed-bin labels keep 5, 7 and 10 at 320, 480 and 960 px, and all 20 at 1,400 px; with the sketch's estimated text, where parity also kept 5 at 320 px, a greedy pass kept 7 there, 2 to 4 bins apart. The stride counts from each tick's absolute index (value over step for uniform ticks, the key's position for band and point scales), not from the first visible tick, so panning does not make labels flicker: a \[0, 100\] axis with ticks every 5 and a stride of 3 keeps 0, 15, 30, …, 90; panned to \[2.5, 102.5\] it keeps 15, 30, …, 90, where a stride counted from the first visible tick would switch to 5, 20, 35, …. Across axes, labels of larger panels are placed first, and later ones that overlap are dropped: where FL's histogram meets its scatter, the histogram's "100" gives way to the scatter's "0.0". Collisions are checked on the drawn boxes, not by tick rules; matplotlib's `MaxNLocator(prune=…)` missed the same case because the tick it pruned lay outside the view.

**Rotated labels collide as rotated boxes.** With `.rotate(deg)`, a label of ink size w × h at angle θ reaches w·|sin θ| + h·|cos θ| below a bottom axis, and two labels of one axis d apart are disjoint when d·|cos θ| ≥ w or d·|sin θ| ≥ h, the separating-axis test on their own two axes; the cross-axis pass tests oriented boxes on all four axes. A 60 × 15 px label needs 60, 30, 21.2 and 15 px of spacing at 0°, 30°, 45° and 90°, and reaches 15, 43.0, 53.0 and 60 px below the axis. The run is shaped once; the rotation lives in the glyph run's transform.

**Far-from-zero domains get offset notation.** When a continuous axis's domain sits more than 10⁴ spans from zero, its labels show the distance from an offset with the most trailing zeros, and the offset ("+1e9", "+1.2345e4", "−1e9") sits at the axis end and adds to its reach; otherwise a domain whose largest magnitude is below 10⁻⁴ or at least 10⁶ gets a multiplier ("1e−7") instead, matplotlib's `axes.formatter.limits`. \[1e9, 1e9 + 1\] labels 0.0 … 1.0 with "+1e9" and \[0, 3e−7\] labels 0.0 … 3.0 with "1e−7", as matplotlib does; \[1000, 1000.5\] gets no offset, where matplotlib shows "+1e3". `.offset(false)` turns it off.

**During a gesture the layout holds still.** Labels change width while zooming. Laying out on every event moved one panel's left edge from 45 to 51 and 57 px over eight wheel steps (45, 39, 51 and 57 px with a fixed count of 5), the same jump matplotlib's constrained layout makes as labels grow (22.9 to 61.2 px). So the widget keeps the plot areas from the start of the gesture, lets labels grow outward (by at most 12 px in that run), reserves two characters of slack per labeled vertical axis in interactive figures (twice the width of "0" at tick-label size, 18.7 px at notebook), and lays out once when the gesture ends; a wheel zoom ends after 300 ms without input. The figure offers this as `resolve_held` and `draw_held`, which take the plot areas of the gesture's start (`layout::Areas`) and ignore areas made for another size or panel count.

**Cost.** The sketch's times include fitting domains, every pass and the collision pass, with text widths estimated; matplotlib's include measuring text. With typst-measured text a warm layout of the wheel figure took 0.19 ms against 0.18 ms estimated (see Text and math).

| Grid | Panels | Protrusion layout | matplotlib constrained layout |
| --- | --- | --- | --- |
| 2×2 | 4 | 0.07 ms | 60 ms |
| 4×4 | 16 | 0.14 ms | 285 ms |
| 8×8 | 64 | 0.31 ms | 1.2 s |
| 16×16 | 256 | 1.04 ms | 5.7 s |

**Spans.** A panel that spans several tracks counts its reach against the outer track on each side, as in Makie, and its plot area runs from its first track's start to its last track's end: `fig.panel_at(Cell::new(1, 0).span(1, 2))`. Nested grids, which pass their reaches up as CSS's subgrid does, are not in v0.1 (see Later). Fixed, content-sized and aspect-ratio tracks are out of scope.

A constraint solver and CSS grid layout were built and rejected; see Alternatives considered.

## Helpers

A `helpers` cargo feature ships ready-made compositions of the core and the figure layer, so matplotlib's one-call conveniences exist without growing either API. Each helper is ordinary code over public APIs, so users can copy and change it.

- **Grids.** `subplots(rows, cols)`: a figure with one panel per cell, x fitted and shared per column and y per row, built with the figure API (see Figures).
- **Statistical charts.** Box plots with 1.5×IQR whiskers, outliers and notches, as matplotlib's `boxplot_stats`. Violins from a Gaussian KDE with Scott's bandwidth, widths shared within one violin mark so areas compare. Histograms with weights, density, cumulative counts and stacking: stacked histograms count every group over one set of edges, stack the counts with `shape::stack` and draw one `rect` mark per group. ECDF, and error bars, without caps unless asked, matplotlib's default.
- **Chart presets.** Line, scatter, bar (grouped, stacked, diverging), stacked area, pie and donut, heatmap, each with axes, legend and theme defaults. Grouped bars dodge; stacked bars are `bar_y_spans` marks from `shape::stack`, and diverging ones use d3's `stackOffsetDiverging` (positives up from zero, negatives down); stacked areas are one `ribbon` per series; a heatmap is an `image` mark with a colorbar.
- **Labels.** Bar value labels (matplotlib's `bar_label`: the value 3 pt beyond the bar's end, or centered inside), pie labels at `arc.centroid` inside the slices, so the pie reserves no room for them, and legend entries for scatter sizes and colors, chosen by d3's ticks.
- **Axes.** Defaults only: minor ticks on; offset and multiplier notation (“+1e6” at the axis end) is already every figure axis's default and stays on. The features live where their hooks are: `Transform::minor` and the figure axis's `.minor(true)`, the figure axis's offset notation (see Layout), and the time module's label ladder, whose ISO formats already label dates concisely.

Not in v0.1 (see Later): hexbin, stem and event plots, the streamgraph preset, polar axes, inset zoom, and the concise-date offset label.

No scale is a helper: logit, asinh, symlog and function scales come from the core's generic continuous scale (see Scales). Helpers return figures built with the figure API (see Figures). Presets are consuming builders that end in a figure (`.figure()`) or a file (`.save_svg`, `.save_png`), at 640 × 480 px unless told otherwise, matplotlib's 6.4 × 4.8 in at 100 dpi. The line preset over one channel of a recording:

```rust
let rms = helpers::chart::line(
    rec.series(0).expect("channel 0"),
)
.y_scale(scale::symlog().constant(1.0))
.minor(true)
.title(text::typst(
    "RMS [$m slash s^2$]",
));
// 1280 × 960 px.
rms.save_png("rms.png", 2.0)?;
```

## Text and math

**At the figure level, typst is the only text engine.** Tick labels, titles, legends, annotations and math all go through it, so the engine that sizes a label is the one that draws it, and bundled fonts make every machine render the same. That covers the gaps in the matplotlib comparison: shaping, right-to-left text, font fallback, explicit line breaks, rich text (10³, m/s², bold, color) and math. Chart-level text layout, such as label collisions and margins, stays ours whatever the engine. The core has no text engine, as d3 leaves text to the browser: core-level apps draw labels with the host's own text.

**Layout measures text by shaping it, once.** Margins, collision boxes and legend slots take their widths from typst through a text cache keyed by each label's typst snippet (its text with size, weight, color and alignment), which stores the shaped runs; the drawing list then places those same runs, so no label is shaped twice and no measured width differs from what is drawn. Before the first layout pass the figure collects its labels and measures the new ones as one batched document. If typst rejects one label's markup, the batch is compiled label by label: the others still shape, and the bad one is cached as its error, so it is not retried every frame. Estimating 0.6 em per character instead is off by −6% on digits (DejaVu's are 0.636 em, so number labels come out too narrow for the collision pass), +17% on "vibration RMS \[m/s²\]", −39% on "WWW" and +116% on "iii". On the wheel figure measuring moved no panel by more than 3.5 px; it matters on dense axes, where collision decisions are close. A warm layout took 0.19 ms with typst against 0.18 ms with the estimate, which stays only for tests.

**The text cache belongs to the caller and is bounded.** It lives in `figure::Cache`, beside the M4 memo and the `Fit::Visible` memory: the iced widget keeps one in its tree state, `fig.draw_with(&mut cache, …)` takes one, and `fig.draw` makes a temporary one. Each figure draw starts a generation, and labels unused for 32 generations are dropped, so a long pan does not grow the cache without bound. typst's own memoization, comemo, is a process-global cache that also grows with every distinct label, so once per 32 figure draws in the process the cache calls `typst::comemo::evict(32)`, which ages every comemo entry by one and drops those unused for 32 evictions. The cadence counts every draw in the process, so two `Plot` widgets, or a widget beside file output, evict more often; an eviction only ages entries, so the cost is recompiling labels no figure has used for that long. The counter is an `AtomicU64`, one of the figure crate's few pieces of process-wide state.

**Text never wraps by itself, and fitting is checked on request.** A title or label is one line unless its text contains an explicit `\n`, which typst renders and layout makes room for. Text longer than the space it labels is drawn as it is, overflowing centered, as matplotlib and ggplot2 do. Measured on the wheel figure at 960 × 760 px, every axis title fits at paper and notebook, while at talk "vibration RMS \[m/s²\]" needs 247 px of a 151 px axis and "count" 68 of 45. Whoever wants to know before plotting calls `fig.check(size)`, which runs layout without drawing and returns a list of issues (what, where, by how much), so a unit test can assert that a figure fits. Besides overflowing text and a canvas too small for its plot areas, it reports glyphs the fonts lack (a CJK label with the bundled fonts draws .notdef boxes), markup typst rejects, palette keys outside their palette, and owned marks over 100,000 points, which belong in a recording; it is the place for further quality checks. Automatic wrapping and Vega's truncation with an ellipsis were rejected: wrapping surprises, and truncation drops units.

```rust
#[test]
fn wheels_fit_at_talk() {
    let samples = wheels::simulate();
    let mut fig = wheels::figure(&samples, false);
    fig.theme(Theme::seaborn(
        Style::Ticks,
        Context::Talk,
    ));
    let issues = fig.check(size(1600.0, 1200.0));
    assert!(issues.is_empty(), "{issues:#?}");
}
```

**The spike** uses typst 0.15.1 as a library. A 40-line `World` holds the bundled fonts, each label is laid out on a tight page, and ttf-parser hands the glyph outlines to `PathContext`. Four labels drawn that way as plain filled paths (a sum identity, an axis label with units, a power spectral density definition and a standard deviation) matched typst's own SVG output side by side.

| Measured (release, 2-core container) | Time |
| --- | --- |
| First label, cold | 2.3–6.7 ms |
| Each further math label | 0.4–1.5 ms |
| The same label again (typst's memoization) | 55–85 µs |
| Glyph outlines into SVG path strings, per label | 0.25–0.7 ms |
| Loading typst-assets' 17 font faces | 3.3–5.6 ms |

Decisions and findings from the spike:

- **Labels are plain text by default.** They go into typst as string literals, so a `$`, `#` or `_` in user data stays literal. Markup and math are opt-in, through `text::typst(…)`.
- **Size the box to the ink.** typst's default text box ends at the baseline, which clipped descenders until `top-edge` and `bottom-edge` were set to `"bounds"`. The frame reports no baseline either, so it is read from the first text item.
- **Fonts.** typst-assets bundles 9.6 MB of fonts. Shipping DejaVu Sans in two weights plus New Computer Modern Math cuts that, and loading the chart's text font into typst keeps math and tick labels in one typeface. User fonts go first in the fallback order. The bundled faces have no CJK glyphs; `fig.check` reports labels that need them.
- **LaTeX input** through mitex, which converts LaTeX math to typst, is out of scope for v0.1 (see Dependencies for its caveats).
- **Cost.** The binary grows by 46 MB stripped, and per-frame layout cost needs caching; see Performance.

## Drawing and output

**The figure layer emits one drawing list, and each backend renders it.** `PathContext` stays the geometry vocabulary of d3's generators; the list adds what bare geometry lacks. Its geometry is kurbo's and its paint is peniko's, re-exported so users name the same versions. typst 0.15.1 already depends on kurbo 0.13.1 and peniko 0.6.1 (through typst-svg), so neither adds a crate, and vello\_cpu takes them without conversion. Seven commands cover the wheel figure and every output:

- **Fill and Stroke:** a `BezPath` with a peniko `Brush` (solid, gradient or image), and for strokes a kurbo `Stroke`. Caps and joins are always set explicitly: kurbo's `Stroke::new` defaults to round, while SVG, tiny-skia and PDF default to butt and miter, and the first vello\_cpu render differed by up to 170/255 at axis corners until they were set.
- **Markers:** one shape stamped at many points, with optional per-point sizes and per-point indices into a paint table, so a `.fill_by` or `.size_by` scatter stays one command. The wheel figure's 5,600 dots are 12 commands. Expanded, the list grows from 468 to 6,056 commands (468 − 12 + 5,600) and the SVG from 344 KB to 1,330 KB, while tiny-skia's raster time barely moves (61 to 62 ms).
- **Glyphs:** shaped text, below. Unshaped labels exist only inside the figure layer: layout shapes text with typst to measure it, and the list places those runs.
- **Image:** pixels and a placement, for raster heatmaps and colorbar strips.
- **Push and Pop:** a group with a transform and an optional clip. `Clip::Rect` and `Clip::Path` are distinct because rectangular clips are cheap in every backend while only path clips can be missing from one, and the figure knows which clips are plot areas; detecting rectangles from path data was fragile in the sketch.

**The list is divided into parts.** Each figure element (a background, a mark, an axis, a title, a legend) is a `Part`: its command range, its bounds in list px and a hash of what it drew. The `Plot` widget invalidates by them. It partitions the canvas into cells at every plot-area edge, maps part bounds to device pixels to find the parts each cell touches, and re-renders a cell only when their hashes change (see iced integration). A hash is FNV-1a 64 over the commands the part emitted, so it sees the output of user closures rather than their identity: a changed mark changes its own part's hash only, and moving a panel changes bounds, not hashes. A list is `Send + Sync`, so the render worker can take it as an `Arc<List>`.

```rust
pub struct List {
    pub size: Size,
    pub fonts: Vec<FontData>,
    pub commands: Vec<Command>,
    pub parts: Vec<Part>,
}

#[non_exhaustive]
pub enum Command {
    Fill {
        path: BezPath,
        paint: Brush,
    },
    Stroke {
        path: BezPath,
        style: Stroke,
        paint: Brush,
    },
    Markers {
        /// Centred on the origin at size 1.
        shape: BezPath,
        at: Vec<Point>,
        paint: Brush,
        /// Per-point scale of `shape`.
        sizes: Option<Vec<f64>>,
        /// Per-point paints, in place of `paint`.
        paints: Option<Paints>,
    },
    Glyphs(GlyphRun),
    Image {
        rect: Rect,
        image: ImageData,
        smooth: bool,
    },
    Push {
        transform: Affine,
        clip: Option<Clip>,
    },
    Pop,
}

pub enum Clip {
    Rect(Rect),
    Path(BezPath),
}

/// A paint table and one index per point.
pub struct Paints {
    pub table: Vec<Brush>,
    pub index: Vec<u32>,
}

pub struct Part {
    pub id: part::Id,
    /// Push and Pop balanced inside.
    pub commands: Range<usize>,
    /// Union of the commands' bounds, list px.
    pub bounds: Rect,
    /// FNV-1a 64 of the emitted commands.
    pub hash: u64,
}
```

Every backend validates a list before drawing it (`list::check`: balanced groups, parts in order, font and paint indices in range, channel lengths equal to the point count, finite coordinates), and the error names the command and the rejected value.

**Glyph runs index the list's font table and keep their source text.** A font entry is peniko's `FontData`: typst's font bytes, shared without a copy, and a face index. PDF and SVG need each font's glyph set for subsetting anyway, and an index keeps the list serializable for snapshot tests. A run splits wherever the style changes, so the title "Wheel: RL" is two runs.

```rust
pub struct GlyphRun {
    /// Index into `List::fonts`.
    pub font: usize,
    pub size: f32,
    /// Origin, rotation and anchor.
    pub transform: Affine,
    pub fill: Brush,
    pub glyphs: Vec<Glyph>,
    /// Source text: PDF text, SVG <text>.
    pub text: Arc<str>,
}

pub struct Glyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    /// Bytes of `text` this glyph shows.
    pub text: Range<u32>,
}
```

The wheel figure's 65 labels became 66 runs of 371 glyphs in two fonts, shaped as one typst document in 5.5 ms (0.8 ms on repeat). Positions are typst's, so what is drawn is what layout measured, and the source text gives krilla selectable PDF text and SVG its optional `<text>`.

**A figure draws for a target.** `fig.draw(size)` builds the list for vector output with a temporary cache. `fig.draw_with(&mut cache, size, target, &held)` is what the widget and repeated file outputs call. `Target::Vector` (SVG, PDF) thins long lines to four columns per px and snaps nothing. The raster targets snap plot-area edges and hairlines to device pixels: `Target::File { scale }` (PNG, GIF) thins to four columns per device pixel, and `Target::Screen { scale }` (iced) to one, and reserves label slack for gestures (see Layout). `save_svg`, `save_png` and `save_pdf` draw and write in one call.

**Command bounds are computed once per list.** `list::Bounds::of(&list, &mut outlines)` gives every command's bounds in list px: glyphs from their outlines, strokes inflated for their joins, markers point by point, and everything inside a clip intersected with it. Every render of that list takes the value: `raster::Renderer::render(&mut self, &list, &bounds, area, scale)` skips commands that miss the area, markers point by point, and returns exactly round(area × scale) device px. The widget renders one list as many cells and bands, so it computes bounds once and hands them to the worker with the list. A cache inside `List` was rejected: the list's fields are public and mutable, so a hidden cache could silently disagree with its commands.

**A backend declares what it draws natively, and `lower` rewrites the rest** into commands it does draw, so backends stay thin:

```rust
pub struct Caps {
    pub glyphs: bool,
    pub markers: bool,
    pub clip_paths: bool,
    pub sweep: bool,
}

pub struct Lowered {
    pub list: List,
    pub warnings: Vec<lower::Warning>,
}

// SVG has no sweep gradients.
let list = fig.draw(size);
let Lowered { list, warnings } =
    lower(&list, svg::CAPS)?;
```

**Warnings are values.** `lower` returns what it could not draw faithfully, and `save_svg` hands those warnings to its caller: `SweepRasterized` (a sweep gradient drawn as an embedded image), `SweepFlattened` (without the `raster` feature, the sweep drawn in its middle stop's color) and `UnsupportedClip` (a path clip left in place for a backend that cannot draw it). The library depends on no logging crate and prints nothing, and a test can assert that a figure lowers without warnings.

Of the built-in backends, SVG lowers sweep gradients, rendered with vello\_cpu into embedded images, and the tiny-skia fallback lowers glyph runs to outlines; vello\_cpu and krilla draw every command natively. iced shows vello\_cpu images (see iced integration) and needs no lowering. The geometry lowering serves backends that lack glyphs or markers: glyph runs become outlines and markers become fills. Lowering the wheel figure's glyphs and markers (its clips are rectangles) takes 1.2 ms (468 to 6,056 commands) and renders within 7/255 of the original, with no pixel off by more than 16. A path clip on a backend without clip paths stays in place, drawn unclipped, and is reported; no built-in backend needs that, since tiny-skia clips through masks. Cutting shapes against a path clip in geometry is Later. The sketch measured it with i\_overlay: a polar panel, a circle clip over 2,038 shapes, stayed within 38/255 of the masked render (0.004% of pixels off by more than 16). Locating each shape's bounding box against the clip first (if no clip edge crosses it, one point-in-polygon test keeps the shape whole or drops it, and only the rest go to i\_overlay) is exact, cut 118 of the 2,038 shapes (1,086 kept, 834 dropped), and took lowering from 21–42 ms to 5.2–7.8 ms with output within 13/255 of the uncut version. matplotlib instead gives its renderer default methods that fall back to paths. That works for glyphs and markers but not for clips, which need the clip stack across commands, and a list-in, list-out pass can be tested by pixel diff.

**Outputs**, in order of importance:

1. **SVG**, from our own writer. Text goes out as glyph outlines by default (344 KB for the wheel figure), so the file needs no fonts and draws exactly the glyphs typst measured. `<text>` is opt-in for editing (306 KB), at the price of the viewer re-shaping it: resvg's re-shaped render differed by up to 255/255 on 0.3% of pixels. Ids are assigned in command order, so the same list writes the same file.
2. **PNG**, rasterized from the list by vello\_cpu at any scale: 18–20 ms at 2× (1920 × 1520) against 61–66 ms for tiny-skia and 93 ms for resvg on the SVG. A render of any area equals the same pixels of the full render within 1/255, and file outputs use vello\_cpu's own thread pool (`raster::Settings::parallel()`: the available parallelism minus one, from 1 to 8 threads). The tiny-skia backend stays as a fallback; it and resvg agree within 64/255.
3. **GIF**, from raster frames through the gif crate: 12.5 ms a frame at 1× with vello\_cpu, plus 4.8 ms to draw the list and 2.0 ms to shape its text (less than the 5.5 ms cold figure because typst memoizes labels repeated between frames). This is what an animation becomes outside a GUI.
4. **PDF**, written from the list by krilla, which typst's own PDF export uses: 114 KB with selectable text and subset fonts in 18 ms, against 687 KB of outlines in 62 ms through svg2pdf.

iced renders the list as images rendered by vello\_cpu, without `lower`; see iced integration. A replay of the list into anyrender, which reaches vello on the GPU, vello\_hybrid and Skia, took 63 lines in the sketch; it is Later, since no v0.1 output needs it.

## Themes

**A theme is seaborn's split, as a typed struct:** a style (colors, and which of grid, axis lines and tick marks exist), a context that scales every size at once (paper 0.8, notebook 1, talk 1.5, poster 2), a font scale and a palette. The values are seaborn 0.13.2's, read from `axes_style` and `plotting_context`: at notebook, text is 12 pt and tick labels 11 pt; the figure title is matplotlib's `large`, 1.2 times the text size. Two defaults differ from seaborn's:

- **The `ticks` style, not `darkgrid`.** darkgrid's #EAEAF2 panel lowers every series color's contrast by 17% (deep's blue goes from 4.8:1 against white to 4.0:1), and ticks is what d3, Observable Plot and plain matplotlib draw. Grids are opt-in through `whitegrid`; darkgrid stays a preset.
- **The `colorblind` palette, not `deep`.** Under simulated color-vision deficiency (Machado 2009, full severity), deep's orange and green, two of the wheel figure's three colors, end up 5.6 CAM02-UCS units apart for protanopes; colorblind keeps seaborn's hue order and 18.6 units.

| Palette | Normal, first 3 | Protanopia, first 3 | Deuteranopia, first 3 | Protanopia, first 5 |
| --- | --- | --- | --- | --- |
| seaborn deep | 37.2 | 5.6 | 10.2 | 2.4 |
| seaborn colorblind | 34.2 | 18.6 | 27.4 | 10.6 |
| tab10 = d3 Category10 | 46.8 | 1.6 | 17.3 | 1.6 |
| d3 Observable10 | 28.5 | 22.3 | 13.3 | 8.8 |
| Okabe-Ito | 31.5 | 20.7 | 30.5 | 20.0 |

The table gives the smallest distance between any two of a palette's first colors, in CAM02-UCS units. Okabe-Ito scores best but includes black and a yellow at 1.4:1 against white. `theme::palette` re-exports these five from core's schemes and adds seaborn's muted, pastel, bright and dark. The theme palette colors marks without a key and the color scales helpers build, while an explicit `scale::ordinal(keys, colors)` keeps its own colors.

**Sizes are points, 1 pt = 4/3 px,** the CSS ratio, and figure sizes are pixels. `em` is the tick-label size, so the gaps from Layout scale with the context. Text comes out about 4% smaller than matplotlib's at the same pixel size, because matplotlib renders at 100 dpi. At 960 × 760 px the wheel figure's plot area is 58.6% of the canvas at paper, 50.7% at notebook and 34.7% at talk; layout re-measured each in three passes. Talk is too big for that canvas, and there axis titles longer than their panel overflowed it; the wheel figure's own test checks talk on 1600 × 1200 px instead (see Text and math).

**Fitted domains get a 5% margin on each side, held at zero** where bars and areas start, as matplotlib's sticky edges do; `Domains::Nice` rounds outward to ticks instead, with the axis's requested tick count or, without one, the count the first layout pass derives from its length (time scales floor and ceil to the chosen interval). Over 100,000 random data ranges, d3's `nice(5)` wasted a median 18% of the axis, 30% at the 90th percentile and 44% at worst, against 9.1% for the margins. Vega-Lite rounds by default; matplotlib, seaborn and ggplot2 pad by 5%; Observable Plot does neither.

```rust
// seaborn's structure, our defaults
fig.theme(Theme::default());

fig.theme(
    Theme::seaborn(
        Style::WhiteGrid,
        Context::Paper,
    )
    .palette(palette::DEEP)
    .font_scale(1.1)
    .domains(Domains::Nice),
);
```

**`Theme` has builders only.** `Theme::default()` is the ticks style at notebook with the colorblind palette and 5% margins, and `Theme::seaborn(style, context)` picks any of the five styles (`DarkGrid`, `WhiteGrid`, `Dark`, `White`, `Ticks`) and four contexts. `.palette(colors)` takes anything that is `AsRef<[Rgb]>`, so a palette constant by value and a `Vec` by reference both work, and copies it; `.font_scale(k)` scales text only; `.domains(d)` chooses margins or nice domains. The fields are private, so settings can be added without breaking callers. A builder given invalid input (a font scale that is not finite and positive, an empty palette, a margin outside \[0, 0.5)) leaves the theme as it was, so there is no theme error type. The sketch's struct needed about twenty settings to drive the whole wheel figure; matplotlib's rcParams have 339 keys, of which seaborn sets 27 for style and 20 for context. Theme files in TOML are Later. The bundled text face is DejaVu Sans, matplotlib's own default.

## Performance

**With caching, text costs about 0.3 ms per pan step, so the frame budget goes to data, not labels.** Uncached, 40 typst labels cost 5–6 ms a frame: fine at 60 Hz, tight at 120–144 Hz.

**What users notice.** An added delay becomes noticeable at about 55 ms when dragging with a mouse or touchpad, at 11 ms on a touchscreen, and at 69–96 ms for taps (Deber et al., 2015). Devices already spend 50–200 ms end to end before our code runs. So the practical target is to finish each frame inside the display's budget: 16.7 ms at 60 Hz, 8.3 ms at 120 Hz, 6.9 ms at 144 Hz. A missed frame shows as judder while panning and adds a frame of delay.

**Measured:** 40 tick labels ("1000.0" …) at 11 pt, release build, 2-core container, about ±30% between runs:

| Step | Time |
| --- | --- |
| typst, 40 new labels in one document | 1.7–2.8 ms |
| typst, one document per label | 4.6–5.2 ms |
| typst, the same 40 labels again | 0.1–0.2 ms |
| typst, panned by one tick | 1.8–2.8 ms |
| Plain shaping (rustybuzz or cosmic-text) | 0.25–0.6 ms |
| Glyph outlines into `PathContext` (240 glyphs) | 0.16 ms |
| The same as SVG path strings | 3.8 ms |
| Triangulating the outlines every frame (lyon) | 3.3 ms |
| Rasterizing the wheel figure's glyph runs to cached images (vello\_cpu), 1× / 2× | 1.8 / 2.4 ms |
| The same runs as SVG through usvg and resvg, as iced's draw\_svg does, 1× / 2× | 5.7 / 7.4 ms |
| CPU rasterization (tiny-skia) | 2.0 ms |

**Levers, in order:**

1. **Cache laid-out labels** in the figure layer, keyed by text and style. typst's memoization does not help when panning, because every pan is a new document; our cache does, and a pan step lays out only the one new label. The cache is bounded: `figure::Cache` owns a `text::Cache` that drops entries unused for 32 draws, and typst's process-wide memo cache is aged with `typst::comemo::evict(32)` once per 32 figure draws per process.
2. **Batch each frame's new labels** into one typst document, which is 2–3× cheaper than one document per label.
3. **Redraw only what changed.** The `Plot` widget cuts the canvas into cells at every plot-area edge and keeps one image per cell, split into bands under 2 MiB, with one content hash per cell. Hover redraws only the overlay; a change in some panels re-renders those panels' cells (4.4 ms for one wheel panel at 2×, against 16–25 ms for the figure). Changed cells render inline when a two-constant cost model predicts at most half the frame budget, and otherwise on one worker thread while the previous image stays on screen. See iced integration.
4. **Thin long series** to at most four points per device-pixel column (M4, below), so the cost follows screen width, not recording length.
5. **Preview while the sharp image renders.** During a zoom or pan whose cells go to the worker, the widget draws each plot-area cell's previous image mapped by one affine per axis, so the view follows the pointer at once and sharpens when the new image lands. Shifting cached tiles by whole device pixels on pans and rendering only the entering strip (under 0.4 ms per panel at 2×) is the follow-up the full-frame benchmark decides (Phase 31).
6. **A plain-text fast path** through rustybuzz with typst's fonts is about 5× cheaper than typst layout, but adds a second code path that must render alike. v0.1 does not include it; it waits for a measurement that demands it.

**Thinning keeps M4 per device-pixel column, read from an index that grows with each recording.** M4 (Jugel et al., 2014) keeps each column's first, last, minimum and maximum sample, so every extreme survives; uPlot draws the same four points per column (`paths/linear.js:4-15, 83-104`). Measured on a vibration-like trace of 1M samples (10 kHz for 100 s: drift, a 7 Hz tone, noise, 0.2 s bursts every 10 s and 50 one-sample impacts) in a 1000 px panel, against all samples stroked by tiny-skia:

| Method | Points | Impacts kept | Columns whose top or bottom is off by >1 px | Pixels off by >16/255 |
| --- | --- | --- | --- | --- |
| min/max | 2,000 | 50/50 | 3.9% | 14.2% |
| M4 | 3,949 | 50/50 | 2.2% | 10.3% |
| M4, 4 columns per px | 15,429 | 50/50 | 1.9% | 3.8% |
| LTTB, 2,000 / 4,000 points | 2,000 / 4,000 | 49 / 50 | 86% / 74% | 14.1% / 12.9% |
| MinMaxLTTB, 2,000 / 4,000 points | 2,000 / 4,000 | 49 / 50 | 60% / 53% | 13.7% / 12.4% |

The LTTB family picks visually representative points rather than extremes: it shrinks the noise band and the bursts, and at 2,000 points it drops an impact, the feature bearing data must not lose. No method matches the full trace pixel for pixel. Inside a dense band, thousands of overlapping anti-aliased segments render solid while M4's zigzag leaves lighter stripes, which is most of its 10.3%; the band's top and bottom, which a reader uses, are right. Four columns per pixel brings that to 3.8%, so file outputs use four and the screen uses one.

**The index** stores, for blocks of 64 samples and for every power-of-two group of blocks, which sample is the minimum and which the maximum. A column over any sample range is answered from about log₂ n of those entries plus a raw scan of its two partial blocks, and its output matched a full scan in 51 of 51 views. Nothing is cached per zoom level, so nothing is invalidated, and below about four samples per column M4 returns every sample, so thinning fades out by itself:

|  | 1M samples | 10M samples |
| --- | --- | --- |
| M4 by scanning, full view | 2.5 ms | 22.4 ms |
| LTTB / MinMaxLTTB, 2,000 points | 4.0 / 3.2 ms | 37.7 / 50.9 ms |
| Index build, once per series | 3.8 ms, +3.3% memory | 40 ms, +5.2% memory |
| M4 from the index, full view | 0.54 ms | 0.86 ms |
| M4 from the index, 5% view | 0.22 ms | 0.51 ms |

Those memory figures are for `proto/thin2`'s index, which stores positions only. The library's index also stores each node's extreme values, so queries and the validity predicate below skip reads into the channel: about 0.75 bytes per sample, 9.4% of an `f64` channel, and Phase 9 asserts it stays under 10% at 1M samples. Only complete nodes are stored, so `extend` only pushes: each completed block pushes a leaf and each completed pair its parent, and the index grows with the recording; `drop_front` rebuilds it. A fixed-level min/max pyramid, also sketched, took 89 ms to build at 1M with +200% memory and rounded column edges to its levels. Thinning matters for files too: stroking all 1M points took 465 ms, M4's 3,949 took 4.9 ms.

```rust
// One clock, many channels; NaN is a gap. The M4
// index grows with every extend.
let rec = Recording::new(
    Clock::Regular { start: 0.0, period: 1e-4 },
    [&accel_x, &accel_z],
)?;

panel.mark(
    line(rec.series(1).expect("channel 1"), time, vib)
        .stroke(color, Source::AccelZ),
);
// Opt out: .thin(Thin::Off)
```

`series::Recording` and its M4 query live in the core (see Data and identity), since they are pure computation with no dependencies, so core-level apps thin with the same code that figure marks call at draw time:

```rust
let s = rec.series(0).expect("channel 0");
// Sample index per column edge, from data-space
// edges (the x scale's invert at pixel edges).
let columns = rec.columns(&edges)?;
// Admit what the y scale can draw (log: y > 0).
for line in s.m4(&columns, |y| y > 0.0)?.iter() {
    // sample indices of one
    // NaN-free run
}
```

**NaN breaks the line.** The index skips NaN, and M4 runs per NaN-free run, one polyline each. With 24 gaps of 1 to 50,000 samples in 1M, building the index with run detection took 8.7 ms. In 41 views the index matched a full scan, emitted no NaN point and drew no segment across a gap; the full view was 19 polylines of 3,124 points in 0.44 ms. A gap narrower than a pixel column stays invisible at that zoom, as it does when every sample is drawn.

**Each polyline reaches past the view inside its own run.** Besides the columns' points, M4 returns the adjacent sample beyond each side of the view when it lies in the same NaN-free run, so the line leaves the panel at its true slope instead of stopping at the last visible sample. A run that ends inside the view contributes no neighbour, and a gap is never crossed.

**The query takes a validity predicate.** A log y scale cannot draw y ≤ 0, so the mark passes the y scale's admission test, and an indexed extremum that fails it is not used: the query descends to that node's children and scans the leaf raw with the predicate. NaN never qualifies.

**Thinning runs in line and area marks at draw time,** once layout knows the panel width and the scales know the visible domain. Columns are counted in device pixels: one per device pixel on screen (`Target::Screen`), four per device pixel for PNG and GIF (`Target::File`) and four per list pixel for SVG and PDF (`Target::Vector`). Column boundaries come from the x scale's `invert` at device-pixel edges, so log and time axes get the right columns, and the recording turns them into sample indices by search, so irregular timestamps work. Boundaries are cached per draw, keyed by the recording's id and revision and the edges' bits, so channels recorded on one clock compute them once: the search costs 38 µs at 1M samples and 112 µs at 10M, 7–13% of a channel's M4 query. Each channel keeps its own NaN runs, intersected with the shared columns. M4's output is memoized in `figure::Cache` by recording id, revision, channel and column grid, so a redraw of an unchanged view queries nothing; an append bumps the revision and the query reruns from the index. Thinned lines draw linear segments; curves apply only to unthinned data. Thinning applies to `Series` views over a continuous x; owned points and `series::slices` are drawn as given, and `fig.check` reports owned marks over 100,000 points.

**Marks clip in `f64` before anything becomes `f32`.** Segments are clipped (Liang–Barsky) to the plot area grown by its own width and height on every side, so no coordinate handed to a backend exceeds three times the panel's size, whatever the zoom; a segment wholly outside is dropped and breaks the polyline.

Whether the widget meets its frame budgets end to end on real hardware, through hover, pans, zooms, live data and a 4K-sized plot, is the question the full-frame benchmark answers (see Open question: the full-frame benchmark).

## Testing

**d3's own tests grade the core, through a tolerant intermediate format.** The port can then use idiomatic `std` Rust without chasing last-digit differences, while real bugs still fail.

- **The format.** Both sides of every assertion become an `Obs` value: `Null | Bool | Num(f64) | Str | List`. Numbers are equal within max(1e-9, 1e-9 × magnitude). Strings split into text and number tokens, so `4.2e1` matches `4.2e+1` but `inf` still fails against `Infinity`. A decimal inside a string may differ by one unit in its last printed place, and d3's `assertInDelta` keeps its own tolerance.
- **Locales and zones.** d3's tests assume its en-US default, and 158 of d3-format's 759 assertions depend on its separators, so the harness runs d3's default-locale tests under `format::Locale::EN_US` and `time::Locale::EN_US`; their expected values are unchanged. d3's time tests run under America/Los\_Angeles, taken from `testkit::zone`, which reads jiff's bundled time zone database rather than the host's. Every test runs under `TZ=Asia/Kathmandu`, an offset no expected value assumes, so code that reached for the system zone would show.
- **The translation.** A 329-line acorn-based translator (acorn vendored, so it runs offline) turns d3's test files into Rust calls to `check(id, kind, actual, expected)`, each with an id `module/test/file.js:line:col` relative to the vendored d3 root. `assert.throws` becomes `throws(id, || expr)` over a `Result`, and file-level helpers are translated when they are expressions the translator handles. JavaScript-only constructs (`valueOf`, `instanceof`, function aliases) become recorded skips with a category. The generated Rust is checked in, and each core phase extends the translator within a budget of 100–300 lines of JavaScript.
- **The ratchet.** `passing.txt` lists every passing id, and `--check` fails when a listed id fails, becomes a skip or disappears. Its header carries a floor per file (`#floor d3-ease/test/linear-test.js 11`), and `--check` also fails when a file passes fewer assertions than its floor. A core phase is done when every d3 test file it owns passes at least 70% of its assertions, unless the phase's skip table names the file and the reason. Newly passing ids are blessed only after reading the diff.
- **The proof of concept.** Across d3-array's ticks, d3-ease, d3-path and d3-format, 1,300 of 1,480 assertions translated, and all 1,300 pass against an idiomatic `std` port. 23 pass only because of the tolerance, all exponent notation. The tests also caught three real porting bugs: a NaN-to-integer cast, `f64::max` dropping NaN, and `inf` against `Infinity`.
- **Reach.** Nearly all of tier 1's 7,752 d3 assertions are reusable. Tier 3 offers only about 150 of 1,238, because d3 tests its DOM plumbing; zoom, brush and drag get new tests against the core state machines.

Beyond d3's suite:

- **d3 itself is the oracle where its tests stop.** Behavior d3 does not test (brush, drag, the viridis tables, curve edge cases) is checked against d3 run in Node from the vendored sources through the resolve hook (`dev/harness/translate/d3-resolve.mjs`, lifted from `proto/tr/`), or against traces derived from d3's source, as for the brush (`d3-brush/src/brush.js:432–580`). Core ports d3's color formulas and every scheme exactly in `f64`, so color is graded the same way, by d3's tests and by d3 in Node.
- **matplotlib grades what d3 does not have.** Values from matplotlib 3.11.2, seaborn 0.13.2 and scipy (symlog tick positions for spans of two decades or more where ours are not thinned, padded fitted domains, kernel density estimates, box-plot statistics, histogram edges) are generated by Python scripts checked in beside their JSON and compared through the same tolerant format, never byte for byte.
- **This document's examples are tests,** and every compiler error it quotes is a `compile_fail` doctest with a passing twin, since stable rustdoc does not check error codes.
- **Degenerate inputs and precision.** Tick rules bail out early, cap their output and are fuzzed (2,000 cases, more with `RUSTD3_FUZZ=1`); figure fitting ignores non-finite values and widens flat domains; zoom rejects non-finite, reversed and sub-precision spans; time computes in i128 nanoseconds and never passes through `f64` epoch values.
- **Recordings.** M4 is checked against full scans, split appends against one build, regular clocks against explicit ones, and a recording after `drop_front` against a fresh one of the retained samples.
- **Snapshots are text only:** the drawing list's dump and the SVG, which are deterministic because fonts are bundled and text goes out as outlines. A missing reference fails unless `RUSTD3_BLESS=1`. No PNG is stored, because binary references differ between machines; raster outputs are compared between our own renderers at test time, within these bounds (the tile score is the worst 32-px tile's mean absolute channel difference):

| Comparison | Bound |
| --- | --- |
| vello\_cpu against the tiny-skia fallback | max ≤ 64/255 |
| `lower`'s output against its input (glyphs, markers, sweeps) | no channel off by more than 16 |
| An area render against the same pixels of the whole render | ≤ 1/255 |
| A render shifted by whole device pixels against a fresh one | ≤ 1/255 (half a pixel: up to 109/255) |
| A cell rendered as bands against the whole cell | ≤ 1/255 |
| Our SVG through resvg against vello\_cpu | tile score ≤ 0.02 |
| PDF through poppler (`pdftoppm -r 192`) against vello\_cpu at 2× | measured, recorded in `decisions/phase-37.md`, then asserted: ≤ 0.02 if the measurement meets it, else 1.25 × the measurement, never above 0.05; `pdftotext` returns every label |
| GIF frames against the PNG | 1.5 × the measured difference, recorded likewise |
| iced against the PNG, opaque background | ≤ 1/255 on tiny-skia, ≤ 2/255 on wgpu |
| iced against the PNG, translucent background | ≤ 4/255 (the 8-bit premultiply round trip, `tiny_skia/src/raster.rs:113`) |

- **Layout invariants.** No two labels overlap, attached panels share their pixel ranges, layout takes at most four passes, and `fig.check` is empty on every example. `fig.check` gives users the same checks in their own tests.
- **iced is tested headlessly, pixels included.** rustd3-iced's tests drive real widgets through a small harness over `iced_runtime::UserInterface`: build, `update` with `RedrawRequested(now)`, `draw`, then `Headless::screenshot`, which returns RGBA for the statistics above. The harness calls `renderer.tick()` once per virtual frame, because wgpu drains its asynchronous image uploads only there (`wgpu/src/lib.rs:709-712`) or in a frame that draws a raster image, and in an app only winit's runtime calls it. Worker results arrive by injecting `Event::Waken`, and every test runs once per backend (`ICED_TEST_BACKEND=tiny-skia` and `wgpu`). `iced_test`'s `matches_image` and `matches_hash` are not used: both compare exactly and write a missing reference instead of failing (`test/src/simulator.rs:306-371`), and its snapshots expose no pixels. The use-case apps are driven through `iced_test`'s simulator for their messages, without comparing images; in use case 1 that test caught a real bug.
- **Scale factors.** Layout, snapping and raster are checked at 1, 1.25, 1.5 and 2, and the widget's image placement at the same four on both renderers.
- **Benchmarks report rather than gate.** The label benchmark and a headless smoke run of the full-frame benchmark (every scenario × every workload, under 8 minutes) print their numbers in `scripts/check.sh full` and fail only on errors, never on numbers: container timings vary by ±30% between runs. Budgets apply to the full-frame benchmark on real hardware (see Open question: the full-frame benchmark).

## Coverage

**Every tier-1 capability in the matplotlib and d3 comparison has a home in this design.** None needs a subsystem the design lacks; path effects are the one row left open, and are marked. Rows marked Later have their home but are built after v0.1 (see Later).

| matplotlib has it, portable d3 does not | Covered by |
| --- | --- |
| Text, rich labels and math | typst |
| Margins sized from measured text; tick count from axis length | Figure layer layout (see Layout) |
| Legends and colorbars | Figure layer and helpers; colorbars as image strips (see Figures) |
| Shared and twin axes, attached panels | Figure layer (see Figures) |
| Raster heatmaps | Image commands; iced `draw_image`; PNG embedded in SVG |
| Dashes, caps and joins | Stroke styles in the drawing list |
| Hatching and path effects | Hatching as clipped line geometry the figure layer builds (see Figures); path effects open |
| PNG, SVG, PDF and animation files | SVG writer; vello\_cpu for PNG and GIF frames, krilla for PDF; gif |
| Arbitrary time zones and sub-millisecond time | jiff; time scales in i128 nanoseconds |
| Logit, asinh and function scales | Generic continuous scales (see Scales) |
| Symlog and minor ticks, offset notation | Core scales and axes (see Scales); helpers set defaults |
| Box, violin, ECDF, error bars, histogram options | Helpers |
| Stem, hexbin and event plots | Later |
| Bar and pie labels | Helpers |
| Polar axes, inset zoom | Later |
| Lines with millions of points | Thinning (M4 per device pixel, see Performance) |

The d3 side of the comparison (curves, stack offsets, band scales, rounded arcs, perceptual color, easing) is the core port itself. Label collision avoidance appears in neither library and is ours to build. Outside tier 1 remain 3D, vector fields, spectral plots, maps, hierarchies and force layouts.

## iced integration

**The iced crate serves both levels, behind a feature.** rustd3-iced builds on `iced_core` and `iced_widget` alone, pinned to iced revision 3cbdacc. crates.io does not accept git dependencies, so the crate is not published until iced 0.15 is, and the facade gains its `iced` feature then; apps take rustd3-iced from git meanwhile. At the core level it provides `IcedPath` and `pointer::Tracker`, which turns iced events into the core's `Pointer` input; both use cases below are built that way, with iced's own text for labels. At the figure level, behind its `figure` feature, a `Plot` widget shows a `Figure` as images rendered from its drawing list by vello\_cpu, with interaction feedback drawn on top. Transitions are driven from `window::frames()` at either level. The full-frame benchmark measures an interactive `Figure` on real hardware (see Open question: the full-frame benchmark).

**The `Plot` widget draws images from the PNG rasterizer.** vello\_cpu renders the drawing list as it does for the PNG output, so iced shows exactly what a file shows: typst text, clip paths of any shape, every paint, markers. Crosshair, highlighted points, brush rectangle and selection band are drawn on top as quads, clipped per panel with `with_layer`; tooltips are iced widgets. It is a custom widget, not a `canvas::Program`: only a widget's `draw` can move cached content with `with_transformation` and clip with layers on both renderers (checked by `proto/icedspike`, binaries `layerclip` and `xform`). The same code runs on iced's GPU and software renderers and headlessly in tests, with no `lower` pass for iced.

### The widget

```rust
fn view(&self) -> Element<'_, Message> {
    // Borrows the recording; the figure is a local.
    let (fig, time) = trace_figure(&self.rec);
    Plot::new(fig)
        .on_change(time, Message::Window)
        .into()
}
```

`Plot::new(fig)` takes the `Figure<'a>` by value: a figure built in `view` is a local, so a borrowed one fails with E0515, and the widget keeps it for relayouts. The builders are `.width` and `.height`; `.scale_factor(f32)`; `.crosshair(bool)` and `.highlight(bool)`, both on by default as matplotlib's and uPlot's cursors are; `.render(Render)`; `.probe(Probe)`; `.on_change(handle, f)`; `.caching(Caching)`; and `.revision(u64)`. Phase 36 adds `.on_hover(f)` and `.on_brush(handle, f)` for hover messages and the one-dimensional brush.

**Everything happens once per `RedrawRequested`.** Pointer input is queued between frames. On each `RedrawRequested` the widget applies the queued input, resolves the figure and builds its drawing list through a caller-owned `figure::Cache` (laid-out labels, M4 output, `Fit::Visible` memory), cuts the list into cells and renders the cells whose content changed. `Figure<'a>` is neither `'static` nor `Send`, so resolving and building the list stay on the UI thread; the list is `Send + Sync` and is what crosses to the worker. A closure in a figure cannot be hashed, but its output can: every part of the list carries an FNV-1a hash of the commands it emitted, and those hashes decide what re-renders. `Plot::revision(u64)` covers what hashing cannot see, such as a paint image's pixels replaced in place or a swapped font: a new value re-renders every cell once. `Plot::caching(Caching::Off)` re-renders every cell whenever the list changes; it is a diagnostic for the benchmark.

### Cells, bands and placement

**Images are cells of a partition of the canvas by all plot-area edges.** Lines through every plot area's left, right, top and bottom edges cut the canvas into rectangles, and each cell renders every command that touches it, culled by the list's command bounds. Images per panel and per axis would overlap, since labels and ticks reach past their panel's box into gutters other panels also draw in, so re-rendering one image would leave pixels another image shows out of date; partition cells cannot overlap. On raster targets layout snaps plot-area edges and hairlines to device pixels, so every cell edge is a whole device pixel. A 4×4 grid gives 81 cells: 16 plot areas and 65 gutter cells. Each cell carries its device rect, the parts it shows, one hash over them and its raster work: one item per fill, stroke or image, one per 16 path elements, one per marker point and one per glyph. Hover renders no cell, and a change confined to some panels re-renders those panels' cells only.

**A cell whose image would reach 2 MiB is drawn as bands.** With `BAND_BYTES = 2 * 1024 * 1024`, a cell whose RGBA reaches it (`width × height × 4 ≥ BAND_BYTES`) is rendered and drawn as horizontal bands of whole device rows, `BAND_BYTES / (width × 4) − 1` rows each and the last one shorter, so every band stays strictly under the limit. wgpu uploads an image under 2 MiB synchronously inside the frame that first draws it (`wgpu/src/image/cache.rs:223-246`); a larger one goes to an upload worker whose results are drained only by `Renderer::tick` or by a later frame that draws a raster image, so it misses its first frame. Every render makes a new handle, and a new handle is a new texture (`core/src/image.rs:134`), so this holds for every re-render, not only the first. With every image under the limit, the widget needs no `allocate_image` and no residency bookkeeping. A 960 × 640 cell (2,457,600 bytes, 3,840 per row) becomes bands of 545 and 95 rows, the first 2,092,800 bytes; stitched back, the bands match the whole cell within 1/255.

**Images land on device pixels.** The widget's device origin is its layout position times the scale factor k, rounded. Each image is drawn with nearest filtering at logical bounds `(device + NUDGE) / k` for x and y and `device size / k` for width and height, with `NUDGE = 0.25` device px, and clipped to its own rect. The two renderers place images differently: wgpu adds 0.001 and rounds (`wgpu/src/nudge.rs:43,72-74`), tiny-skia truncates (`tiny_skia/src/raster.rs:63-64`). Bounds of exactly `device / k` can multiply back to a hair below the integer in `f32` and truncate a pixel early on tiny-skia; a quarter pixel lands exactly on both renderers, and half a pixel would land a pixel late on wgpu. A cell at device x 125 at factor 1.25 is drawn at logical x 100.2; a widget at logical x 10.3 at factor 1.5 has its device origin at round(15.45) = 15. Placement tests pin both constants on both renderers at factors 1, 1.25, 1.5 and 2.

**Handles carry the rasterizer's bytes.** vello\_cpu's output is straight-alpha RGBA, which is what iced handles carry (tiny-skia premultiplies on load, `tiny_skia/src/raster.rs:113`), so each image moves into `Handle::from_rgba` without a copy. With an opaque theme background, straight and premultiplied are the same bytes.

### Rendering inline or on one worker

**Changed cells render inline when cheap and on one worker thread otherwise, and the previous image stays until the new one arrives.**

```rust
/// Where the widget rasterizes changed cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Render {
    /// Inline when the predicted cost is at most half
    /// the frame budget, else on the worker thread.
    #[default]
    Auto,
    /// Always on the UI thread, inside RedrawRequested.
    Inline,
    /// Always on the worker (tests, the benchmark).
    Worker,
}
```

The worker thread starts with the first job and owns one `raster::Renderer` built with `raster::Settings::parallel()`: vello\_cpu's own thread pool, the available parallelism minus one, from 1 to 8 threads, is the only parallelism, and the widget adds none. Jobs are whole cells with all their bands, sharing one `Arc<List>` and one `Arc<list::Bounds>` computed once per list; plot-area cells go first, then gutters. At most one job set is pending: a refresh replaces it, and the worker checks after each job whether it was replaced and abandons the rest, so the newest state wins at the source. Results return over a channel and wake the UI through the waker the widget keeps from `shell.waker()` (`core/src/shell.rs:80`); a flag makes many finished cells cost one `Event::Waken`, on which the widget drains the channel. A result is shown only if its generation is newer than what its slot shows, and dropped otherwise; until then the slot keeps its previous image, so a cell never blinks to the background once it has one. Dropping the widget closes the queue and joins the thread.

**`Auto` predicts each refresh's raster cost against the frame budget.** The budget is the interval between the last two `RedrawRequested` instants, clamped to 1 ms ..= 16.7 ms, and 16.7 ms until two exist: an idle app's long gap must not make a 100 ms render look cheap, and a 144 Hz display gives 6.9 ms. The prediction sums a two-constant model over the changed cells:

```rust
pub mod cost {
    /// Device pixels rasterized per millisecond.
    pub const PIXELS_PER_MS: f64 = 5.0e5;
    /// Work items (commands, marker points, glyphs) per millisecond.
    pub const ITEMS_PER_MS: f64 = 400.0;
    /// pixels / PIXELS_PER_MS + work / ITEMS_PER_MS
    pub fn model_ms(pixels: u64, work: u64) -> f64;
}
```

The constants are fitted to two measurements of the wheel figure at 2× (1920 × 1520, 2,918,400 device px) on a 2-core container: 16–25 ms whole and 6.7 ms without its 5,600 dots. The dots cost about 14 ms, about 400 points per millisecond; the rest, 2,918,400 px and about 760 items, cost about 6.7 ms, of which about 1.9 ms is items, which leaves 4.8 ms for the pixels, about 6.1e5 px per millisecond, rounded down to 5e5 to stay conservative. Checked back, the model gives 5.84 + 15.9 ≈ 21.7 ms for the whole figure (measured 16–25) and 5.84 + 1.9 ≈ 7.7 ms without the dots (measured 6.7), so the figure without its dots renders inline at 60 Hz (threshold 8.35 ms) and the scatter-heavy one goes to the worker. On a machine with more cores for vello\_cpu's pool the model is conservative. With a probe attached, after 8 `Raster` samples an exponentially weighted mean (α = 1/16) of measured over modelled time scales the prediction. A cell with no image yet counts like any other: a cheap first frame renders inline and shows no gap, an expensive one shows the theme background for a frame.

**`Plot::probe` reports what each frame cost and what it showed.**

```rust
let (tx, rx) = mpsc::channel();
Plot::new(fig).probe(Probe::new(Instant::now, tx))
```

The app supplies the clock, a `fn() -> Instant`. Samples are `Stage` (resolve, list) per `RedrawRequested`; `Raster` per rendered cell, with its pixels, work, queue wait, duration and whether it ran inline; and `Frame`, which `update` sends at the end of each `RedrawRequested` because `draw` takes `&self` and changes nothing (`core/src/widget.rs:70-79`). A `Frame` describes what the following `draw` shows: its generation, its images, `missing` (slots with no image), `previews` (slots whose shown content hash differs from their current hash, gutters included) and the bytes of handles drawn for the first time. Without a probe nothing is sent and no clock is read. The probe is the full-frame benchmark's source for attributing time.

### Gestures

**Each handle gets one core `Zoom`, and input is applied once per frame.** A wheel over a plot area zooms every handle the panel zooms (`.zoom_x`, `.zoom_y`) around the cursor, so `.zoom_y(vib).zoom_y(temp)` zooms twin axes together; a wheel over an axis zooms that axis's handle only; a left-button drag pans, and so do horizontal wheel deltas. Each handle's `Zoom` gets the scale extent `[1e-6, 1e6]` (core keeps d3's `[0, ∞]`), so k stays finite across a long wheel burst. iced can deliver several events before `update` runs and the view is rebuilt, so the widget queues pointer input and applies it once per `RedrawRequested`, accumulates each gesture's transform from the gesture's start rather than step to step, and continues from the domain it last published until the gesture ends. Without that, a 120 px pan delivered as three events per update lost half its distance in the figure sketch (`proto/figmodel/examples/a_live.rs`); the widget's test delivers one 120 px drag with an update after every event, after every third event and once at the end, and all three publish the same domain within 1e-12. `on_change(handle, f)` publishes one message per handle per frame, with the domain in the scale's own type, and the app owns that domain; without it the widget keeps the domain, keyed by handle index and scale type (see Figures). When applying input publishes a message, the widget skips building the list in that pass: the runtime runs the app's `update`, rebuilds the view and sends `RedrawRequested` again, up to three times (`winit/src/lib.rs:777-830`), and the list is built once, from the app's new figure. Gestures go through each scale's pixel-space `pan_px` and `zoom_px`, so timestamps never pass through `f64` epoch values, and a step the scale rejects (non-finite, reversed, or a span below 2⁻⁴⁰ of its center; 1 ns for time) leaves the gesture's transform where it was and publishes nothing.

**The layout holds still during a gesture.** Laying out on every event moved a panel's left edge from 45 to 51 and 57 px over eight wheel steps. The widget keeps the plot areas from the gesture's start, lets labels grow outward and lays out once when the gesture ends (see Layout). A drag ends at its release; a wheel gesture ends at the first `RedrawRequested` at least 300 ms after its last wheel event, which the widget schedules with `request_redraw_at`. A double-click resets the handles the clicked panel zooms and a gesture touched to their fitted domains; an app-owned handle receives its fitted domain as a message. Core's `Zoom` keeps d3's double-click zoom-in, which the widget never forwards.

**Previews show while a sharp render is pending.** While a plot-area cell shows outdated content and its new image is on the worker, `draw` shows the cell's current bands at mapped bounds: one affine map per axis, from the scales at the cell's two edges (`current.apply(rendered.invert(p))`). Two points determine the map for every continuous transform, because figure scales apply pixel-space operations affinely in output space; band and point axes map by identity. Previews are drawn with linear filtering and clipped to the cell, and look soft until the sharp bands land. Gutter cells keep their last image unmapped: an axis strip stretched along its axis would show wrong labels. Drags are not snapped to device pixels: each drag frame re-renders the moved plot-area cells and the gutters that show the panned handle, inline when the prediction allows and on the worker with a preview otherwise. Snapping serves only tile translation (Phase 31). The cursor shows a grab hand over a zoomable plot area, a grabbing hand during a drag, a resize arrow over a zoomable axis and a crosshair over other plot areas.

### Live data

**`Fit::Window(span)` follows the newest data in whole device columns.** For an x handle whose widest panel has a plot area of `w` device columns:

```text
col   = span / w
right = ceil(last / col) · col      last: the newest admitted x
left  = right − w · col
```

The window is the view, so x gets no 5% margin; on a y handle `Window` behaves as `All`, and with no data the handle fits as `All`. Appends move the domain by whole columns, so the thinning column boundaries of one update are the previous ones shifted by a whole number of columns, and a streaming line does not shimmer. With `Fit::Window(10.0)` on 1,000 device px and the last sample at 123.456 s, the view is [113.46, 123.46]; after 1,000 more samples at 10 kHz it is [113.56, 123.56], a 10-column move; one sample more leaves it in place. A panel narrower than the widest one sharing the handle gets columns that are not whole device pixels; its pixels are right, only its column edges do not repeat between updates. A gesture holds a windowed handle until a double-click releases it.

**An append re-renders only what it touched.** `extend` bumps the recording's revision, so the M4 memo misses and M4 runs over the view from the index, about 0.5 ms for a full view of 1M samples. The line's part hash changes, so its cell re-renders; with `Fit::Window` the x-axis strips re-render too, and y gutters stay unless a `Fit::Visible` y handle moved. The widget keeps no recording state: the hashes are the dependency tracker. `drop_front` rebuilds the index, bumps the revision and re-renders the same way.

### Time and scale factor

**Library code never reads a clock.** The widget takes time only from `RedrawRequested(Instant)`: queued input is stamped by the redraw it requested, at most one frame late, and the frame budget comes from those instants. Stage timings come only from the clock an app passes to `Plot::probe`. iced's own `mouse::Click` calls `Instant::now` (`core/src/mouse/click.rs:44`), so `pointer::Tracker` applies iced's double-click rule (same button, under 6 px apart, at most 300 ms; `click.rs:73-84`) to presses stamped by the next `RedrawRequested`, and its `wants_redraw()` tells the caller to request that redraw. clippy's `disallowed-methods` keeps `Instant::now`, `SystemTime::now` and the system time zone out of every library crate.

**The scale factor is an input.** `Plot::scale_factor(f32)` sets device pixels per logical pixel. Without it the widget uses `renderer.scale().map(Scale::total)`, then the window's factor from `Opened` or `Rescaled`, then 1.0, skipping non-finite and non-positive values. tiny-skia's renderer reports no scale (`scale()` is `None`), window events carry only the window's own factor, and `iced_test` snapshots at 2.0 without hinting the renderer, so an app with its own `scale_factor` on the software renderer, and every pixel test, passes the factor.

### What it costs

Measured on the wheel figure (2-core container, ±30% between runs):

| Step | Time |
| --- | --- |
| Whole figure, 1× / 2× / 4× (about 4K) | 11.5 / 16–25 / 55.7 ms |
| Whole figure without its 5,600 dots, 2× | 6.7 ms |
| One panel, culled, 2× | 4.4 ms |
| Pan strip of 8 / 32 px, 2× | 0.33 / 0.38 ms |
| Drawing list rebuild, warm | about 5 ms |

Estimated for the v0.1 widget from those steps and the cost model, not measured end to end; "line figure" is the wheel figure without its dots:

| Use | Without caching | v0.1 (estimate) |
| --- | --- | --- |
| Hover, crosshair, tooltip | a full re-render per mouse move, 21–30 ms | the overlay only; no cell renders |
| Live data at 1–10 Hz, x advancing | 21–30 ms per update | the cells on the moving x handle and its axis strips: at most a whole render, about 7 ms inline for a line figure, 16–25 ms on the worker for this one |
| Drag-pan on a shared axis at 60 Hz | 21–30 ms a frame, about 35–45 fps | line figure: about 5 ms of list and 7 ms of raster inline, sharp at 60 fps; this figure: 60 fps of shifted previews, sharp after each 16–25 ms render |
| Pan on one panel's own axis | 21–30 ms | that panel's cell and axis strip, about 4.4 ms inline |
| Continuous zoom | the UI blocks 21–30 ms per step | a scaled preview at once, the sharp frame 20–30 ms later |
| A 4K-sized plot | about 56 ms per full render | previews at once for pans and zooms, 15–18 sharp frames a second |

**Pans on scatter-heavy or 4K figures are the case the benchmark checks.** Line figures, own-axis pans and hover stay inline at 60 Hz. A pan on a shared axis of a dot-heavy figure, or any pan at 4K, re-renders whole plot-area cells every frame, so the sharp image trails the pointer by a render and the share of preview frames rises. The prepared answer is Phase 31: plot areas split into 256-px tiles anchored in data space to the thinning column grid, translated by whole device pixels on pans so that only entering tiles render (the strip costs above), with an overscan ring, a pool of renderers set by `Plot::threads`, envelope fills and marker splatting. Whole-pixel shifts render identically (within 1/255; a half-pixel shift differs by up to 109/255), so that phase snaps drags to device pixels and keeps thinning columns and dash phase anchored during a gesture. It is built only if the full-frame benchmark attributes a missed pan or zoom budget to raster or upload.

Continuous zoom and figure-level animation stay CPU-bound, and zoom previews look soft until the new image lands. Line-based figures are cheaper than this scatter-heavy one: without its dots the figure renders in 6.7 ms at 2×. Tried and rejected: stamping dots from pre-rendered images (17.8 against 22.3 ms warm, 37.7 ms cold, off by up to 54/255; Phase 31 measures a finer variant, 16 coverage stamps at quarter-pixel phases, and builds it only at 1.5× the speed within 32/255), keeping vello\_cpu's glyph caches between renders (no change), and lower-resolution previews (1× is only about 35% cheaper than 2×, so the scaled previous image is the better preview).

### Two measured alternatives

Both were measured at iced revision 3cbdacc through `iced_test` snapshots, on software Vulkan:

- **vello in iced's shader widget.** iced 3cbdacc and vello 0.11 share wgpu 30.0.1 and peniko 0.6.1, so vello can draw the list on iced's own device: a 2× snapshot matched vello\_cpu within 54/255 on 0.038% of channels, and encoding the scene took 0.5 ms. It is not the v0.1 path: it is GPU-only (the software renderer drops shader widgets with a warning), adds 23 crates, needs `vello::Renderer` behind a `Mutex` because iced requires `Sync`, and ties our wgpu version to iced's. It is the candidate GPU data layer, behind a feature, if the full-frame benchmark still asks for one after Phase 31; the drawing list stays the interface.
- **iced geometry for everything.** 11.1 ms of tessellation per rebuild (5,928 fills, mostly dots), independent of pixel density, but it needs `lower` with label images, paint images and clips cut in geometry, draws edges with 4× multisampling (0.51% of channels off by more than 16 from the PNG), and on the software renderer re-rasterizes on every redraw: 41 ms at 1×, 55 ms at 2×.

### What works at the pinned revision

- **Paths:** `IcedPath` implements `PathContext` over iced's path builder. iced's `Builder::arc` starts a new subpath (`graphics/src/geometry/path/builder.rs:104-118`), which would split a d3 pie slice, so `IcedPath` tracks the current point and the subpath's start and segments arcs into cubics of at most a quarter turn itself; its fields are private (`IcedPath::new(&mut builder)`).
- **Pointer input:** `pointer::Tracker` turns iced events into `Pointer` values: wheel deltas negated to d3's sign and pixel deltas divided by the scale factor, moves that continue outside the bounds while a button is held, `Left` once, and `DoubleClicked` from stamped presses.
- **Text:** at the figure level labels are part of the vello\_cpu image, drawn from typst's glyph runs exactly as in the PNG. At the core level apps use iced's own text. Rejected for figures: iced's canvas text, shaped by cosmic-text rather than typst, so it sizes labels differently from layout and has no math; per-glyph meshes, which only the GPU renderer draws (iced\_tiny\_skia ignores meshes); and typst SVG through `draw_svg`, 2–4× slower per label and 61 ms for the whole figure (write 3 ms, parse 29 ms, rasterize 29 ms).
- **Measurement:** iced's `Paragraph` measures text headlessly ("1,000,000.5" at 11 px is 54.87 × 15.12 px), though typst makes it unnecessary.
- **Images:** the renderer's image primitive shows the `Plot` widget's cells; at the core level `Frame::draw_image` draws raster heatmaps and `draw_svg` embedded SVG.
- **Tests:** a headless harness drives the real widgets and reads back their pixels on both renderers (see Testing).

| iced limit (3cbdacc) | Workaround |
| --- | --- |
| `Frame::with_clip` clips no paths or text on the software renderer, headless or not: `paste` drops the clipped frame's bounds (`tiny_skia/src/geometry.rs:237`). A test left 443,622 inked pixels outside the clip; the GPU renderer left 0 | Clip each panel with `with_layer` in a custom widget, checked on both renderers; report upstream |
| Clips are rectangles only | `Plot`: vello\_cpu clips to any path. Core level: mask the margins, as use case 1 does; cutting geometry against a clip is Later |
| Fills are solid or linear gradients with at most 8 stops; further stops are dropped silently (`graphics/src/gradient.rs:72`) | `Plot`: vello\_cpu paints everything |
| Canvas meshes are anti-aliased with 4× multisampling: 0.51% of channels off by more than 16 from the PNG | `Plot`: images are the PNG's pixels |
| A `canvas::Program` cannot move cached geometry: `draw` gets a shared renderer, and the cache redraws when its bounds change | A custom widget's `with_transformation` moves and scales cached geometry, checked on both renderers |
| The software renderer re-rasterizes canvas geometry on every redraw: 41 ms at 1×, 55 ms at 2× for the wheel figure | `Plot` shows cached images |
| One geometry cache drawn twice in a frame shows only once on the GPU renderer | Draw each cache, and each image handle, once per frame |
| Meshes and the shader widget are GPU-only; the software renderer drops shader widgets with a warning | Not used by the `Plot` widget |

These limits do not reach the figure itself, which arrives as images; they apply to the `Plot` widget's overlay, which uses quads only, and to core-level apps that draw with iced's canvas themselves.

## Use case 1: focus and context (core level)

**This is where the design works best.** d3's brush-and-zoom chart, three sensor series over 30 days with a detail view above an overview, becomes a 247-line iced app (rustfmt defaults, tests excluded), and one piece of state replaces d3's event cross-talk. Its tests drive it headlessly through `iced_test`'s simulator.

In d3, zoom and brush each update the other, and each handler needs a guard against the update it caused:

```javascript
function brushed(event) {
  if (event.sourceEvent?.type === "zoom") return;
  const s = event.selection || x2.range();
  x.domain(s.map(x2.invert, x2));
  focus
      .selectAll(".line")
      .attr("d", line);
  const t = d3.zoomIdentity
      .scale(width / (s[1] - s[0]))
      .translate(-s[0], 0);
  svg
      .select(".zoom")
      .call(zoom.transform, t);
}

function zoomed(event) {
  if (event.sourceEvent?.type === "brush") return;
  const t = event.transform;
  const domain = t
      .rescaleX(x2)
      .domain();
  x.domain(domain);
  focus
      .selectAll(".line")
      .attr("d", line);
  const s = x
      .range()
      .map(t.invertX, t);
  context
      .select(".brush")
      .call(brush.move, s);
}
```

In the Rust version both behaviors publish the same message, and both views derive from one field:

```rust
struct App {
    traces: Vec<Trace>,
    tz: TimeZone,
    full: [Timestamp; 2],
    // the one source of truth
    visible: [Timestamp; 2],
}

enum Message {
    Show([Timestamp; 2]),
}

// The focus canvas's state: d3's zoom behavior
// and the iced-to-Pointer translation.
struct Focus {
    zoom: Zoom,
    tracker: pointer::Tracker,
}

// Focus canvas: wheel and drag set a window.
fn update(
    &self,
    focus: &mut Focus,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<Action<Message>> {
    // Pointer x in the x scale's range.
    let plot = Rectangle {
        x: bounds.x + MARGIN,
        width: bounds.width - MARGIN,
        ..bounds
    };
    let input = focus
        .tracker
        .translate(event, plot, cursor)?;
    let full = self.app.x(self.app.full, plot.width);
    let [v0, v1] = self.app.visible
        .map(|t| full.apply(t));
    let [r0, r1] = full.get_range();
    let k = (r1 - r0) / (v1 - v0);
    let now = interact::Transform {
        k,
        x: -v0 * k,
        y: 0.0,
    };
    let next = focus.zoom.handle(input, now)?;
    let window = full
        .rescale_x(&next)
        .get_domain();
    let show = Message::Show(window);
    let action = Action::publish(show)
        .and_capture();
    Some(action)
}

// Drawing: scales are rebuilt from state, and
// lines go straight into iced's path builder.
let x = self.app.x(self.app.visible, w);
let y = self.app.y(h);
let line = path(|p| {
    let points = s.points
        .iter()
        .map(|&(t, v)| (x.apply(t), y.apply(v)));
    shape::line()
        .curve(Curve::MonotoneX)
        .draw(points, p)
});
```

The tracker reports positions relative to the rectangle it is given, here the plot area, so `x` is in the scale's range, and it times a press by the next `RedrawRequested` rather than a clock, which is how it detects double clicks; while `tracker.wants_redraw()` is true the app requests that redraw.

**Verified headlessly.** Tests drive the real widgets through `iced_test`'s simulator and check the messages the app receives. Twenty wheel lines at the focus halve the window around the cursor, and the brush follows. Dragging the overview from x = 300 to 520 px sets a window of about 7.7 days, the expected share of a 30-day, 860-px axis. After a zoom, dragging the brushed band by 86 px moves the window by 86/860 of its 719 hours and keeps its span, which shows that the brush picked its selection up from `visible`.

**The test caught a real bug.** iced can deliver several pointer events to the same view before rebuilding it, as the simulator does, so the brush cannot copy its selection from `visible` on every event: mid-drag, that overwrote the gesture with out-of-date state and the window snapped back to all 30 days. The brush therefore syncs only between gestures (`if !brush.is_active()`); without that guard the test fails.

Two iced-specific notes. On iced's software renderer, `Frame::with_clip` clips no paths or text (see iced integration), so the example masks the left margin. And at the pinned revision `view` returns `impl Widget<Message>` and `Column` is generic over its children, which touches only rustd3-iced code, not core.

## Use case 2: the general update pattern (core level)

**This is where the design strains.** A top-five bar chart that re-ranks every 1.5 seconds, with bars entering from zero width, moving to new rows, and shrinking out, staggered 20 ms apart and interrupted by the next update mid-flight. It works, but takes roughly four times the code of d3, because d3's untyped attribute transitions become typed state the application owns.

The d3 version is compact because the DOM holds both the marks and their in-flight transitions:

```javascript
// data: the top five, sorted
function update(data) {
  const names = data.map(d => d.name);
  const y = d3.scaleBand(names, [0, 250])
      .padding(0.1);
  const t = svg
      .transition()
      .duration(750);
  svg
      .selectAll("rect")
      .data(data, d => d.name)
      .join(
        enter => enter
            .append("rect")
            .attr("fill", d => color(d.name))
            .attr("y", d => y(d.name))
            .attr("height", y.bandwidth())
            .attr("width", 0),
        update => update,
        exit => exit.call(exit => exit
            .transition(t)
            .attr("width", 0)
            .style("opacity", 0)
            .remove()))
      .call(bars => bars
          .transition(t)
          .delay((d, i) => i * 20)
          .attr("y", d => y(d.name))
          .attr("width", d => x(d.value)));
}

d3.interval(() => update(nextData()), 1500);
```

In Rust, everything that animates is a field, and the application keeps the marks:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bar {
    y: f64,
    width: f64,
    opacity: f64,
    value: f64,
}

impl Interpolate for Bar {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        let f = |a: f64, b: f64| a.lerp(&b, t);
        Bar {
            y: f(self.y, to.y),
            width: f(self.width, to.width),
            opacity: f(self.opacity, to.opacity),
            value: f(self.value, to.value),
        }
    }
}

// In update, on new data:
let names = data
    .iter()
    .map(|d| d.0);
let y = scale::band(names)
    .range([0.0, 250.0])
    .padding(0.1);
let targets = data
    .iter()
    .map(|&(name, v)| {
        let y = y
            .apply(&name)
            .expect("name is in the domain");
        let bar = Bar {
            y,
            width: x.apply(v),
            opacity: 1.0,
            value: v,
        };
        (name, bar)
    });
let timing = Timing {
    duration: Duration::from_millis(750),
    stagger: Duration::from_millis(20),
    ..Timing::default()
};
// enter: grow from zero width in its row;
// exit: shrink and fade where it is.
self.marks.update(
    targets,
    now,
    timing,
    |t| Bar { width: 0.0, ..*t },
    |b| Bar { width: 0.0, opacity: 0.0, ..*b },
);

// The frame clock runs only while
// something moves:
fn subscription(&self) -> Subscription<Message> {
    let every = Duration::from_millis(1500);
    let data = time::every(every)
        .map(Message::NewData);
    if self.marks.is_animating(self.now) {
        let frames = window::frames()
            .map(Message::Frame);
        Subscription::batch([data, frames])
    } else {
        data
    }
}
```

The lifecycle logic is checked with fixed instants (100 ms transitions, 20 ms stagger). New data arrives at 60 ms, mid-flight: `a` leaves from where it is, `b` retargets without a jump, and `c` enters one stagger step later:

```plain
t=  0ms  a=0.0  b=0.0
t= 50ms  a=5.0  b=2.2
t= 60ms  a=7.4(exit)  b=5.1  c=0.0
t=110ms  a=3.7(exit)  b=22.6  c=3.2
t=180ms  a=0.0(exit)  b=40.0  c=30.0
after prune at 180 ms:
b=40.0  c=30.0, nothing animating
```

**What strains:**

1. **A struct and an `Interpolate` impl per kind of mark.** d3 tweens any attribute by name. `#[derive(Interpolate)]`, from rustd3-derive behind core's `derive` feature, removes the impl above; the lifecycle code remains.
2. **The application owns the lifecycle.** Exiting marks stay in `Marks` until they finish, a frame message prunes them, and the frame subscription must be gated on `is_animating` or it burns CPU at the display rate.
3. **`now` lives in state.** `view` cannot read a clock, so the latest frame `Instant` is stored and threaded through.
4. **Some d3 features have no direct equivalent:** chained transitions (`.transition().transition()` for move-then-grow sequences), named concurrent transitions on one element, `attrTween` on arbitrary strings, and `on("end")` callbacks. Each needs explicit state or a sequencing API, and `Marks` covers the common case without them.

Without imports, data generation and `main`, and formatted with rustfmt and Prettier defaults, the Rust example is 133 lines of code, counting the hand-written impl, and the d3 core above is 30, though it leaves out labels and the SVG setup. Phase 32 builds both use cases on the real crates, counts their lines again and records the counts in `decisions/phase-32.md`.

## Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Port d3's JS surface literally: overloaded getter-setters via `Option` arguments, `serde_json::Value` data, string attributes | Gives up the types that make the Rust version safer, and reads badly in Rust |
| Accessor-first generators with `'static` boxed closures | Every scale would be cloned into `move` closures (E0597, see Accessors). Fine for long-lived objects such as forcefield's simulation; wrong for generators built per view |
| Fully monomorphized pipelines | d3\_geo\_rs took this route and is 5.6× d3-geo's size; generic types leak into every signature |
| A retained scene graph with keyed diffing, a "virtual DOM" for marks | Considered for use case 2. iced already rebuilds views, so a small keyed store (`Marks`) covers transitions without a second tree |
| A chart DSL macro (`chart! { line(x: d.t, y: d.v) }`) | Worse errors and IDE support; the macros kept are the compile-time format check and `#[derive(Interpolate)]` |
| iced's `Animation<T>` inside core | Ties core to iced and needs `Copy` float-like states; the iced crate can still use it for widget-level effects |
| iced\_aksel as the chart layer | No general paths and no holes in filled areas, a second axis model, and no functional commits since May 2026 (see Dependencies) |
| The host's text (iced, cosmic-text) instead of typst | No math, and measurement would depend on each backend; typst is one engine for all of them |
| Figures drawn by vello in iced's shader widget | Matched the PNG within 54/255 on 0.038% of channels and encodes in 0.5 ms, but is GPU-only (the software renderer drops it), adds 23 crates and ties our wgpu version to iced's. It is the candidate GPU data layer, behind a feature, if the full-frame benchmark asks for one |
| Figures drawn as iced canvas geometry | 11.1 ms of tessellation per rebuild, needs label images, paint images and clip cutting, 0.51% of channels off from the PNG, and 41–55 ms of re-rasterization per redraw on the software renderer |
| Widget images per panel and per axis | Their boxes overlap, since labels and ticks reach into gutters other panels draw in, so re-rendering one image leaves pixels another shows out of date; cells of a partition by all plot-area edges cannot overlap |
| A pool of single-threaded renderers in the widget | vello\_cpu already spreads one render over its own thread pool, and a pool of renderers beside it would oversubscribe the cores. A pool pays off when many small jobs queue at once, which is the case of translated tiles, so it comes with them (Phase 31) |
| `allocate_image` residency for large widget images | Bands keep every image under 2 MiB, which wgpu uploads inside the frame that first draws it (`wgpu/src/image/cache.rs:223-246`); a larger image waits for `Renderer::tick` or another raster frame, and holding allocations adds bookkeeping per handle |
| 256-px tiles translated on pans in the v0.1 widget | They pay off only for pans on scatter-heavy or 4K figures, which the full-frame benchmark measures before they are built; a line figure renders whole in about 7 ms at 2× and pans inline. With 125 Hz input at 60 Hz and entering tiles rendered off the UI thread, nearly every pan frame shows a preview unless an overscan ring is resident, and tiles cut per partition cell would split a plot area that a spanning panel crosses. Phase 31 builds them per plot area, with an overscan ring, if the benchmark asks |
| A plain-text fast path beside typst | Two code paths that must render alike; held in reserve until a benchmark demands it |
| Sharing resolved by the layout tree, as Vega-Lite's resolve | What is shared must follow the tree. Vega-Lite 6.5.0 cannot share x per column and y per row at once (its specs compiled to 4 x and 2 y scales, 1 and 2, or 2 and 4, never 2 and 2), and a brush that sets another view's scale needs named parameters, which are handles by another name |
| Axes that own their scales and are linked afterwards, as in matplotlib and Makie | Conflicts surface late. matplotlib 3.11.2 lets the last set\_xlim win silently, shares an axis only once, cannot link a variable's x with its y (seaborn's pair grid), and one shared Normalize clipped a second image to the first one's 0 to 3 range; a handle has exactly one domain |
| A constraint solver (kasuari, Cassowary) for layout | For a grid it reproduces the protrusion layout exactly (FL's scatter at 102 px, 328.4 px wide) at 1.19 ms to build and solve 16 columns. When rules conflict it keeps required ones and breaks strong ones silently: requiring FR's scatter to be at least 400 px shrank FL's to 280.7 px and dropped "FR as wide as FL" without a word |
| CSS grid layout (taffy) | Panels must be split into decoration and plot tracks to align, and nested grids misalign without subgrid, which taffy 0.15 lacks: two wheel blocks with 43 and 94 px of decorations put their plot areas at 43 and 94 px |
| Our own paint, stroke and font types in the drawing list | Every backend would convert, vello\_cpu, tiny-skia and krilla included; kurbo and peniko add no crates because typst already depends on them |
| anyrender's recorded Scene as the drawing list | Its glyph runs carry no source text, so no PDF or SVG text; it has no markers and no SVG or PDF backend, and its 0.x types would become our public API. A replay of the list into anyrender took 63 lines in the sketch and is Later |
| SVG as the interchange, with PNG and GIF through resvg | 93 ms for a 2× PNG against 19 ms with vello\_cpu from the list, 70 ms against 12.5 ms per GIF frame, and +13 crates |
| PDF through svg2pdf | 687 KB without text against krilla's 114 KB with selectable text, and +20 crates, including older resvg and tiny-skia |
| Fallbacks as default methods on a renderer trait, as in matplotlib | Works for glyphs and markers, not for clips, which need the clip stack across commands; a list-in, list-out pass is also testable by pixel diff |
| colorgrad and palette for color spaces and schemes | colorgrad's ColorBrewer presets are d3's own stops through a basis spline in `f32`, and palette 0.7's Lab misses d3's 1e-6 tolerance. Core ports d3's color formulas, interpolators and every scheme exactly in `f64` (basis splines over the 9- and 11-stop tables, the turbo and cividis polynomials, the viridis-family tables), graded by d3's own tests |
| LTTB or MinMaxLTTB for thinning, as plotly-resampler uses | Representative rather than extreme points: the band edge is off by >1 px in 53–86% of columns, and a 2,000-point budget dropped an impact |
| Min/max only per column | Half M4's points, but without first and last samples the joins between columns land wrong: 3.9% of columns off against 2.2% |
| A fixed-level min/max pyramid on a recording | 89 ms and +200% memory at 1M against 3.8 ms and +3.3% for the position index (9.4% with stored extremes), and column edges rounded to its levels |
| seaborn's own defaults: darkgrid and deep | darkgrid costs 17% contrast; deep's orange and green are 5.6 CAM02-UCS units apart for protanopes. Both stay as presets |
| A string-keyed parameter map, as matplotlib's rcParams | 339 stringly keys where about twenty typed settings drove the wheel figure; typos fail at run time |
| Nice domains by default, as Vega-Lite | A median 18% of the axis wasted over random ranges, 44% at worst, against 9.1% for 5% margins |

## Prior art in Rust

No existing crate ports d3's charting API idiomatically; the closest borrow d3's vocabulary for one renderer or port one module.

| Crate | Approach | What it teaches |
| --- | --- | --- |
| [gpui-d3rs](https://crates.io/crates/gpui-d3rs) | d3-inspired builders for gpui; a `Scale<Domain, Range>` trait with `scale`, `invert` and `ticks` | Putting `invert` on every scale forces it to return `Option`; type-level availability is cleaner |
| [iced\_aksel](https://crates.io/crates/iced_aksel) | Chart widget with axes in shared `State`, data via a `PlotData` trait, mesh rendering | A tick renderer that returns label priorities, and per-shape hit areas, are worth copying; as a dependency it was rejected (see Dependencies) |
| [forcefield](https://crates.io/crates/forcefield) | d3-force port: `Accessor` enum with `From<f64>` and `From<Vec<f64>>`; closures through an explicit `Accessor::func` | Sidesteps the inference cost of option C by making closures explicit |
| [d3-hierarchy](https://crates.io/crates/d3-hierarchy) | Arena of nodes with `usize` links; `serde_json::Value` data | Indices instead of references keep layouts lifetime-free |
| [d3\_geo\_rs](https://crates.io/crates/d3_geo_rs) | Generic stream pipeline: 382 generic impl blocks, 3 uses of `dyn` | The cost of monomorphizing a pipeline |
| [plotters](https://crates.io/crates/plotters) | Coordinate systems and series drawn through a `DrawingBackend` trait (bitmap, SVG, canvas) | A backend trait works; `PathContext` sits one level lower, at d3's level |
| [egui\_plot](https://crates.io/crates/egui_plot) | Immediate-mode plot with built-in pan and zoom | Built-in behaviors are convenient; keeping behavior state in core instead lets the application own it, as use case 1 needs |

## Settled questions

- **Arbitrary function scales.** A public `scale::Transform` trait and generic continuous scales, so logit, asinh and user transforms get ticks and `invert` in the same value-and-builder style as the built-in scales; runtime choice goes through `AnyContinuous` and the figure layer. See Scales.
- **Figure and axes model.** The figure owns a table of scales, and marks, axes, legends and gestures refer to them through typed `ScaleRef` handles, so panels share a scale by using the same handle. See Figures.
- **Layout and label collision.** Tracks sized by measured protrusions, gaps between decorated boxes in em, two collision passes, and a layout that holds still during gestures. See Layout.
- **The drawing list.** Seven commands on kurbo and peniko types, glyph runs that index a font table and keep their source text, parts with hashes, and a `lower` pass driven by declared capabilities that returns warnings as values; SVG comes from our own writer, PNG and GIF from vello\_cpu, PDF from krilla. See Drawing and output.
- **Thinning large series.** M4 per device-pixel column (one per pixel on screen, four for files), read from an exact index that grows with each recording, run in the marks at draw time; NaN breaks the line. See Performance.
- **Theme defaults.** seaborn's style, context, font scale and palette as a typed `Theme` with builders, defaulting to the ticks style, notebook context and colorblind palette, with sizes in points and 5% domain margins held at zero. See Themes.
- **Drawing figures in iced.** A `Plot` custom widget showing images that vello\_cpu renders from the drawing list, cut into cells at plot-area edges, rendered inline when cheap and on one worker otherwise, with scaled previews during gestures and an overlay of iced quads. It matches the PNG, behaves the same on both iced renderers, and needs no lowering for iced. See iced integration.
- **Builder versus getter names.** d3's names for builders, and a getter only where needed, as `get_<name>`, the exception `std::process::Command` makes (`arg` and `get_args`). Only domain and range clash; builders are called about 100 times across the sketches, getters 14.
- **Polylinear domains.** A const generic defaulting to two (`Continuous<T, R, const N: usize = 2>`), so two-stop types read as usual; three or more stops come only from `scale::piecewise(domain, range)`, which takes both arrays together so the compiler checks their lengths (a 3-stop domain with 2 colors is E0308, where d3 truncates silently). Nice and ticks act on the end stops, as in d3; mapping costs 1.4–6 ns a point.
- **A common `Scale` trait.** Small capability traits (`Kind`, `Position`, `Paint`, `Ticks`) plus the figure's object-safe `Scale` for its table; continuous scales already share one generic type. See Figures.
- **`#[derive(Interpolate)]`.** A small proc-macro crate, rustd3-derive (about 110 lines), re-exported by core behind core's optional `derive` feature (the facade's `derive` turns it on) (+2.2 s clean build alone, about +0.3 s next to typst, which already compiles syn). Every named field interpolates by its type or fails with E0277; `#[interpolate(skip)]` fields jump at t = 1. Skipping unsupported types automatically through autoref specialization let a mistyped `f32` field jump silently, so it is not used.
- **Time locales.** A locale-neutral default (ISO numerics, English names, a 24-hour clock: jiff's own output except `%c`, `%x` and `%X`, written as `%F %T`, `%F` and `%T`) with an ISO time-axis ladder; `time::Locale`, shaped like d3's `timeFormatLocale`, carries names, composites and the label ladder, with `time::Locale::EN_US` and ten tables from d3-time-format 4.1.0 bundled. d3's directive table is ported over jiff (`%N` formats nanoseconds). See Formats, time and color.
- **Number locales.** A neutral SI default (point decimal, U+202F groups of three, no currency, − for minus), grouped per axis when any label has five or more integer digits; `locale::Locale { number, time }` lives in core and a figure takes it with `fig.locale(…)`. See Formats, time and color.
- **Where `IcedPath` lives and how layout measures labels.** `IcedPath` is in rustd3-iced; typst measures every label layout uses.

## Open question: the full-frame benchmark

**Whether the image path is enough is measured, not assumed.** Phase 22 builds `rustd3-bench`, which drives the real `Plot` widget through a wrapper widget that replays time-stamped input (moves, presses, wheel lines and pixels, modifiers, appends, resizes), in a window or headlessly, with every run in a cold process. Its metrics, scenarios, budgets and decision rule are fixed here, before any run:

- **Workloads:** `trace`, two stacked panels sharing a linear x with three channels of 1M samples on one 10 kHz clock over 100 s (the vibration trace of Performance); scatters of 100k and 1M dots; `grid16`, 4 × 4 panels with x shared per column and y per row, each with a 100k-sample line and 2k dots; and `label-churn`, the trace with an x formatter that brings new label strings on every pan step. The wheel figure joins in Phase 34.
- **Scenarios:** hover; pan on a shared x; pan on a panel's own y; continuous zoom (wheel bursts, and high-resolution pixel deltas); live data at 10 Hz with `Fit::Window`; a 4K-sized window; resize; cold start; scale factors 1, 1.25, 1.5 and 2; and a 10-minute run of live data plus pans for memory.
- **Metrics per scenario:** frame intervals (p50, p95 and p99, and dropped frames, with the present mode and compositor recorded); input-to-sharp latency; preview share, the frames in which any slot, gutters included, shows outdated content or no image; UI-thread busy time per event (p99); the raster critical path, the sum of the probe's `Raster` durations for the generation that became sharp; process CPU-seconds per second of interaction; upload bytes per second; and RSS and GPU memory over the 10-minute run. N ≥ 5 runs per scenario, with 95% confidence intervals.
- **Budgets:** p99 frame interval at most 1.5 × the display's frame period (with vsync, the 99th-percentile frame was not a missed one); input-to-sharp p95 under 100 ms for pans and under 150 ms for zooms; preview share under 10% during pans. A budget is met when its 95% interval lies entirely below it, missed when entirely above, and inconclusive otherwise, in which case that cell reruns with ten runs.
- **Rule:** a missed budget is attributed, by the largest share over its offending frames, to the UI thread (resolve, text, M4, list), raster (the critical path) or upload (uploaded bytes times the run's fitted cost per byte). A raster- or upload-bound miss on a pan or a zoom builds Phase 31 (tiles per plot area translated on pans, an overscan ring, a pool of renderers with `Plot::threads`, `Caching::Verify`, cell merging, and envelope fills and marker splatting where they are at least 1.5× faster within 32/255) and reruns the benchmark. A GPU data layer, vello in iced's shader widget behind a feature, is recommended only if Phase 31's rerun still misses. A UI-thread miss routes to moving list building off the UI thread, not to a GPU layer. If every budget is met, Phase 31 is not built.

**The verdict lives in `decisions/phase-22.md`.** Phase 22 opens it as pending the user's run, with the exact command and the rule above, and runs only a headless smoke plan itself, whose numbers are report-only. Before the run, the display's refresh rate and scale factor are recorded; the default plan takes about 50 minutes on real hardware. The user's run adds the verdict: Phase 31 needed or not. Phase 31, if built, appends its rerun, and Phase 34 appends the label-churn rerun with real time formats; whichever of the two runs later writes the closing line.

## Sources

- Prototypes: `proto/` (index `proto/README.md`). The core sketch `proto/rfc/d3` (1,242 lines of code) and `proto/rfc/glue` (`IcedPath`, pointer translation and both use cases), built with Rust 1.97.0 against iced master at commit ab336f6 (2026-10-13) and jiff 0.2.38. The compiler errors quoted in this document come from compiling the counter-examples (`proto/rfc/d3/examples/fail_*.rs`). Measured numbers come from a 2-core container, release builds, ±30% run to run.
- Spikes: the typst label spike and benchmark (`proto/typstspike`; typst 0.15.1, rustybuzz 0.20.1, lyon 1.0, tiny-skia 0.12), the iced text measurement example, the colorgrad and palette checks against d3 7.9.0 from npm, and the scale-transform comparison (`scalecmp`), whose errors and timings the Scales section quotes.
- d3 APIs and behaviors: the `d3/d3-*` repositories on [GitHub](https://github.com/d3), with sources and tests at the measured versions in `proto/d3-src` and `proto/d3-tests`; the focus-and-context and general-update-pattern code above is written in their idiom.
- iced: the [iced repository](https://github.com/iced-rs/iced) at revision 3cbdacc3b0841c5145c766c3a758657d36b41ffe: `core/src/image.rs`, `core/src/renderer.rs`, `core/src/shell.rs`, `core/src/widget.rs`, `core/src/mouse/click.rs`, `wgpu/src/image/cache.rs`, `wgpu/src/nudge.rs`, `wgpu/src/lib.rs`, `tiny_skia/src/raster.rs`, `tiny_skia/src/geometry.rs`, `graphics/src/gradient.rs`, `graphics/src/geometry/path/builder.rs`, `runtime/src/user_interface.rs`, `winit/src/lib.rs` and `test/src/simulator.rs`.
- vello\_cpu 0.3.0's source (`src/render.rs`: `RenderSettings`, its thread pool and `num_threads`).
- uPlot's `paths/linear.js` (lines 4–15 and 83–104), which draws M4's four points per column.
- jiff strftime directives: the jiff 0.2.38 crate source (`fmt::strtime`).
- Scale transforms: ggplot2's [new\_transform](https://scales.r-lib.org/reference/new_transform.html) and matplotlib 3.11.2's `scale.py` (`FuncScale`, `LogitScale`, `AsinhScale`).
- Crates: [typst](https://crates.io/crates/typst), [resvg](https://crates.io/crates/resvg), [gif](https://crates.io/crates/gif), [svg2pdf](https://crates.io/crates/svg2pdf), [colorgrad](https://crates.io/crates/colorgrad), [palette](https://crates.io/crates/palette), [kurbo](https://crates.io/crates/kurbo), [kasuari](https://crates.io/crates/kasuari), [taffy](https://crates.io/crates/taffy), [minmaxlttb](https://crates.io/crates/minmaxlttb), [i\_overlay](https://crates.io/crates/i_overlay), [mitex](https://crates.io/crates/mitex), and the [iced\_aksel repository](https://github.com/QuistHQ/iced_aksel). Versions as of 2026-10-14.
- Rust crates in the prior-art table: their crates.io pages, linked there.
- Latency perception: [Deber et al., CHI 2015](https://unpaywall.org/10.1145%2F2702123.2702300), with its thresholds as summarized in [arXiv 2408.02525](https://arxiv.org/pdf/2408.02525); [Annett et al., CHI 2014](https://webdocs.cs.ualberta.ca/~wfb/publications/C-2014-SIGCHI-Latency.pdf).
- The tolerant testing figures come from the proof of concept (`proto/poc`, `proto/tr`) on Rust 1.97.0 and Node 22.22.0.
- Figure model: the `proto/figmodel` sketch (the option comparison, the four-wheel example and the zoom and pan simulations, including `examples/a_live.rs`); Vega-Lite 6.5.0 specs compiled with `vl.compile`; matplotlib 3.11.2 and seaborn 0.13.2 runs of axis sharing; [Vega scales](https://vega.github.io/vega/docs/scales/) and [Observable Plot marks](https://observablehq.com/plot/features/marks) as references.
- Layout: the `proto/figmodel` examples `wheels_auto`, `layout_cost`, `layout_motion` and `band_labels`; kasuari 0.4.12 and taffy 0.15 runs of the wheel columns; matplotlib 3.11.2 runs of constrained layout, `subplots_adjust` and `MaxNLocator(prune=…)`; Observable Plot 0.6.17's axis source for its tick spacing.
- Drawing list: the `proto/dlist` sketch (binaries `bench` and `extra`) on `figmodel`'s list, with typst 0.15.1 shaping, tiny-skia 0.12, vello\_cpu 0.3.0, resvg 0.48.1, krilla 0.8.2, svg2pdf 0.13.0, anyrender 0.14.0 and i\_overlay 9.1; crate counts from `cargo tree` against typst 0.15.1's 268; PDF text checked with pdftotext and pdffonts.
- iced rendering: the `proto/icedspike` sketch (binaries `plotwidget`, `canvasroute`, `layerclip` and `xform`, snapshotted with `iced_test`) against iced at 3cbdacc (2026-10-15), vello 0.11.0 and wgpu 30.0.1 on Mesa 25.2.8's software Vulkan; image-path costs from the `proto/dlist` binaries `imgcost`, `partial`, `stamps`, `sprites`, `glyphres` and `skiat`.
- Colorbars and hatching: a per-pixel comparison of image strips and 256-stop gradients against matplotlib 3.11's 256-entry colormaps, and `proto/dlist`'s `hatch` binary on the wheel figure. Number locales: the core's d3-format port (`proto/poc/core/examples/loc.rs`) and d3-format's locale tables; the SI rule from CGPM 2003, Resolution 10. Crate names checked with `cargo search` on 2026-10-16.
- Thinning: the `proto/thin2` sketch (`main.rs` and the `gaps` binary) with minmaxlttb 0.1.4 and tiny-skia 0.12, and `proto/cols` (shared column boundaries); Jugel et al., "M4: A Visualization-Oriented Time Series Data Aggregation", VLDB 2014.
- Themes: `proto/figmodel`'s `theme.rs` and `proto/dlist`'s `themes` binary; seaborn 0.13.2's `axes_style`, `plotting_context` and `color_palette`; color-vision simulation and CAM02-UCS distances with colorspacious (Machado, Oliveira and Fernandes, 2009).
- Further sketches: the `proto/poly` (multi-stop domains), `proto/derive` and `proto/autoskip` (`#[derive(Interpolate)]`), `proto/tloc` and `isot` (time locales, jiff 0.2.38) and `proto/cols` sketches, and `proto/dlist`'s `measure`, `titles` and `extra` binaries (typst-measured layout, title fit, clip pre-check); d3-time-format 4.1.0 and d3-format locale tables; Vega's `labelLimit` and `titleLimit`.

# Implementation

## How to use this part

Part I says what rustd3 is and why. This part says how to build it.
Each phase's section is a self-contained specification for one agent
session (two or more where the section names cut points): its goal, what it
builds on, the design with real signatures, the steps as commits, exit
criteria that a command or a named test can check, spot checks with
expected values, and what it leaves to later phases.

An agent implementing Phase N reads, in this order:

1. `CLAUDE.md` at the repository root (Phase 1 writes it).
2. `.claude/RUST_STYLE.md` and `.claude/OPAQUE.md`.
3. `.claude/DECISIONS.md` and the decision files of the phases Phase N
   builds on (a phase that met no decision has no file; read those that
   exist).
4. This front matter, from "How to use this part" to "Order of work".
5. The section "Phase N" and the Part I sections it names.
6. The `proto/` files it cites, and the newest `SESSION_HANDOFF-NN.md`
   if the phase is half done.

The phase section is the most specific text and governs the details of
its phase; Part I gives the reasons. Where a phase section is silent,
Part I and this front matter apply. Where two parts of this document
cannot both hold, or a design call is needed that none of them makes,
the agent records a decision and stops (see Decisions and handoffs).
Each phase's Design includes **Decisions this phase may meet**: the questions
that may come up, each with a recommended answer to record.

Appendix A is the prompt that dispatches a phase to an agent; Appendix
B is how a fresh session resumes the project.

## Workspace

Phase 1 creates the repository: the directory that holds `.claude/`
becomes its root.

```text
rustd3/                    [workspace] + facade package `rustd3`
├─ Cargo.toml, Cargo.lock  lockfile committed; exclude = [".claude"]
├─ src/lib.rs              facade: re-exports by feature
├─ core/                   rustd3-core: the d3 port
├─ derive/                 rustd3-derive: #[derive(Interpolate)]
├─ figure/                 rustd3-figure: figure layer and outputs
├─ fonts/                  rustd3-fonts: bundled fonts
├─ iced/                   rustd3-iced: IcedPath, pointer::Tracker, Plot (publish = false
│                          until iced 0.15 is on crates.io)
├─ dev/                    publish = false
│  ├─ testkit/             rustd3-testkit: Obs, pixel statistics, snapshots,
│  │                       zones (a leaf: depends on no workspace crate)
│  ├─ harness/             rustd3-harness: d3's tests → Obs, ratchet, floors
│  ├─ fixtures/            reference values: d3 in Node, matplotlib in Python
│  ├─ examples/            rustd3-examples: trace, scatter, wheels, use cases,
│  │                       presets
│  └─ bench/               rustd3-bench: the full-frame benchmark
├─ scripts/check.sh        quick | full | release
├─ CLAUDE.md, README.md, CHANGELOG.md, RELEASING.md
└─ .claude/
   ├─ RUST_STYLE.md, OPAQUE.md, DECISIONS.md
   ├─ decisions/phase-NN.md
   └─ map/rustd3/          rustd3.md (this document), proto/, guides/,
                           goon.yaml, SESSION_HANDOFF-NN.md
```

Dependencies point down: core ← derive (behind core's `derive`
feature); figure → core, fonts, typst; iced → core, iced_core and
iced_widget, and → figure behind its `figure` feature; the facade →
all, optional. `rustd3-testkit` depends on no workspace crate, so every
crate can dev-depend on it. `rustd3-bench` depends on `rustd3-examples`
for the wheel figure.

### Dependencies

Library crates use exactly these, with caret requirements;
`Cargo.lock` holds the measured versions. Phase 1 declares all but iced;
Phase 11 adds iced. Any other change to a manifest's dependencies is a
decision.

| Crate | Version and features | Used by |
| --- | --- | --- |
| jiff | 0.2.38 | core (re-exported as `rustd3_core::jiff`; figure and iced name jiff types through it; testkit depends on jiff directly for its bundled time-zone database) |
| thiserror | 2 | all library crates |
| syn 2, quote 1, proc-macro2 1 | | derive |
| typst, typst-layout | 0.15.1 | figure (`PagedDocument` is in typst-layout) |
| ttf-parser | 0.25 | figure (glyph outlines) |
| kurbo 0.13.1, peniko 0.6.1 | | figure (re-exported) |
| png | 0.18 | figure, always (images embedded in SVG) |
| vello_cpu | 0.3.0, `default-features = false`, `std`, `text`, `u8_pipeline`, `multithreading` | figure `raster` |
| gif | 0.14 | figure `gif` |
| krilla | 0.8.2 | figure `pdf` |
| tiny-skia | 0.12 | figure `tiny-skia` |
| iced_core, iced_widget | git rev `3cbdacc3b0841c5145c766c3a758657d36b41ffe`; iced_widget with `advanced`, `canvas`, `image` | rustd3-iced (no winit, no backend, no `web-colors`: the app's `iced` picks those) |

Dev-only, same iced revision: `iced` (default features plus `advanced`
and `canvas`), `iced_test` and `iced_runtime` for rustd3-iced's tests,
`dev/examples` and `dev/bench` (`iced` does not re-export
`iced_runtime`); `serde` with `derive` as a normal dependency of
`dev/bench` only; `serde_json` in the dev crates and as a
dev-dependency; png and ttf-parser as test readers; resvg 0.48.1 (to
render our SVG back); jiff with `tzdb-bundle-always`. Base64 for SVG
data URIs is hand-written (about 40 lines). Fetching iced's own git
dependencies is allowed.

### Features

| Crate | Features |
| --- | --- |
| rustd3-core | `derive` |
| rustd3-figure | `fonts` (default), `raster`, `png` (= `raster` + encoder), `gif` (= `raster` + gif), `pdf`, `tiny-skia`, `helpers` |
| rustd3-iced | `figure` (enables rustd3-figure with `raster`); `fonts` (default, weak: the bundled fonts whenever `figure` is on) |
| rustd3 (facade) | `derive`, `figure`, and each figure feature under the same name; `fonts` default (weak: it does not turn `figure` on) |

The facade gains an `iced` feature once iced 0.15 is on crates.io;
until then applications take `rustd3-iced` from git (see
Architecture).

## Conventions

1. **This document is the specification.** A phase that finds it
   wrong or silent on a design call records a decision and stops.
2. **Style** (`.claude/RUST_STYLE.md`): no `mod.rs`; one semantic type
   per module; module paths instead of composite names; no aliased
   imports; `thiserror` errors per subsystem carrying the rejected
   value; no `unwrap()` outside tests, `expect("invariant: …")` only for
   true invariants; edition 2024; doc comments on public items; tests
   named as sentences. The facade's re-exports (`pub use rustd3_core::{
   scale, axis, … }`, `pub use rustd3_figure as figure`) establish
   canonical paths and are not call-site aliasing.
3. **Names Part I fixes are kept**, including ones that look composite:
   `PathContext`, `SvgPath`, `LineOf`, `AnyContinuous`, `ScaleRef`,
   `TickRule`, `TickFormat`, `GlyphRun`, `BrushX`, `BrushY`,
   `BrushEvent`, `InvalidFormat`, `ParseColorError`, `IcedPath`. No other
   composite names. Behaviors live in `interact`. Two `Transform`s and
   two `Identity`s exist on purpose and are always written with their
   paths: `scale::Transform` (trait) and `interact::Transform`
   (struct); `scale::transform::Identity` (the transform) and
   `scale::Identity` (d3's `scaleIdentity`).
4. **Opaque types** (`.claude/OPAQUE.md`), with three recorded
   exceptions: `PathContext` keeps d3's canvas argument lists;
   field-less transforms are type-level strategies, not namespaces; the
   plain data types that carry no invariant keep public fields,
   including `List`, `Command`, `Part`, `Paints`, `Lowered`, `GlyphRun`,
   `Glyph`, `Caps`, `Rgb`, `Hsl`, `Lab`, `Hcl`, `Cubehelix`,
   `interact::Transform`, `Timing`, `Pointer`, `shape::Slice`,
   `join::Join`, `array::bin::Bucket`, `raster::Image`,
   `raster::Settings`, `list::Cells`, `list::Cell`, `hit::Hit`,
   `text::Style`, `boxplot::Summary`, `hist::Table`, `SeriesExtent` and
   `plot::Hover`; a phase that adds such a type names it as one.
   `Command`, `Pointer`, `lower::Warning` and `check::Issue` are
   `#[non_exhaustive]`; matches on them from other crates, including
   each crate's own integration tests, carry the wildcard arm the
   compiler requires. No other wildcard arms on this workspace's
   enums.
5. **Explicit over ambient.** No mutable global state of ours, except:
   immutable lazily built data in `OnceLock` statics (parsed fonts);
   `AtomicU64` counters for figure ids, recording ids and the text
   cache's eviction cadence; typst's comemo cache, which is
   process-global and bounded by `comemo::evict(32)` once per 32 figure
   draws. Library code never calls `TimeZone::system`, `Zoned::now`,
   `Timestamp::now`, `SystemTime::now` or `Instant::now`; clippy's
   `disallowed-methods` enforces it (`allow-invalid = true` on the jiff
   entries; dev crates opt out in their own `clippy.toml`). The widget
   takes time from `RedrawRequested(Instant)`; stage timings come only
   from an app-supplied clock (`Plot::probe`).
6. **Precision.** Geometry is `f64` through core and the figure layer,
   clipped in `f64` to a guard band before anything converts it to
   `f32`. `f32` appears in `GlyphRun` and `Glyph` and at the backend
   boundaries (iced, vello_cpu, tiny-skia, krilla). Time computes in
   i128 nanoseconds; no `Timestamp` passes through `f64` epoch values.
7. **Changing what exists.** Phases add to the crates freely. Changing
   or removing a public item built in Phases 1–10 is a decision whose
   commit also updates every call site in figure, iced and the dev
   crates, with `check.sh quick` green.
8. **iced is pinned.** Moving the revision is its own commit with the
   iced tests rerun on both backends. rustd3-iced alone does not build
   in release mode (`iced_renderer` without a backend hits
   `compile_error!`, `renderer/src/lib.rs:48-55`); `check.sh` builds it
   in debug or through a dev crate that enables a backend.
9. **Nothing from Later** (see Later (0.2)) is built in v0.1.
10. **No AI attribution** in commits, comments or docs. Commits are
    conventional (`type(scope): summary`, scope = crate), one logical
    change each.
11. **`.claude/` is tracked**, re-included in `.gitignore` (`!.claude/`).

## Checks and tests

**`scripts/check.sh quick`** runs on every commit and takes minutes:
`cargo fmt --check`; workspace clippy with `-D warnings` and all
features; the tests of the crates the commit touched, under
`TZ=Asia/Kathmandu` (an offset of +05:45 exposes code that reads the
host zone); the harness ratchet and floors.

**`scripts/check.sh full`** runs at the end of every phase, in the
background: workspace clippy with all features once; per package,
tests with `--no-default-features` and with `--all-features`; plain
`cargo test` (default features) for rustd3-figure and the facade;
`cargo doc` with `RUSTDOCFLAGS="-D warnings"`; the iced tests once per
`ICED_TEST_BACKEND` (`tiny-skia`, `wgpu`; with it set, iced does not
fall back, and a missing Vulkan device fails unless
`RUSTD3_ALLOW_NO_GPU=1` skips wgpu with a message); the
generated-table freshness check (from Phase 26); the benchmark's
headless smoke report (from Phase 22, report only). Phase 1 records
cold and warm times and the target-directory size.

**`scripts/check.sh release`** arrives in Phase 40.

**Layout of tests.** Each crate has one integration-test binary:
`autotests = false` (from Phase 1) and `[[test]] name = "main", path =
"tests/main/main.rs"`, added by the phase that writes the crate's first
integration test; further files are modules beside it; feature-gated
suites sit behind `#[cfg(feature = …)] mod …;`. Unit tests live beside
the code. Compile-time errors are `compile_fail` doctests, each with a
passing twin (stable rustdoc does not check error codes).

**Snapshots** are text only: list dumps and SVG, compared exactly,
failing on a missing reference unless `RUSTD3_BLESS=1`. No PNG is
stored; raster checks compare our own renderers at test time with
testkit's statistics (max difference, count over a threshold, a
32-px tile score).

**Raster agreement bounds.** vello_cpu against tiny-skia ≤ 64/255;
`lower`'s output against its input, no channel off by more than 16;
whole-pixel shifts ≤ 1/255; bands against the whole cell ≤ 1/255; our
SVG rendered by resvg against vello_cpu, tile score ≤ 0.02; PDF rendered
by poppler (`pdftoppm`) within the tile-score bound Phase 37 measures
and records (≤ 0.02 when met, never above 0.05); the iced widget against
the PNG of the same figure, opaque ≤ 1/255 on tiny-skia and ≤ 2/255 on
wgpu, translucent background ≤ 4/255.

**iced** is tested headlessly through a small harness in rustd3-iced's
tests: build an `iced_runtime::UserInterface`, update it with
`RedrawRequested`, draw, call `renderer.tick()` once per virtual frame,
and read pixels with `Headless::screenshot`. `iced_test`'s
`matches_image` and `matches_hash` are not used: they compare exactly
and write a missing reference instead of failing.

**Scale factors.** Layout, snapping and raster at 1, 1.25, 1.5 and 2
at the figure level; widget placement at the same four.

**Zones** come from `testkit::zone(name)` over jiff's bundled database
(asserted non-empty), never from host tzdata; d3's time tests run under
America/Los_Angeles.

**Performance** is reported, never gated, in `check.sh` (container
timings vary ±30%); the full-frame benchmark decides (Phase 22).

## Grading by d3's tests

The core is graded by d3's own tests. Phase 2 vendors the test and
source files of the d3 modules at the measured versions (with
internmap, d3-dispatch and d3-timer, so the sources import) under
`dev/harness/d3/`, and an offline translator (acorn-based, under
`dev/harness/translate/`) turns their assertions into Rust calls to
`check(id, kind, actual, expected)`. Both sides become `Obs` values
(`Null | Bool | Num | Str | List`) compared tolerantly: numbers within
max(1e-9, 1e-9 × magnitude); strings split into text and number
tokens, so `4.2e1` matches `4.2e+1` but `inf` does not match
`Infinity`; d3's `assertInDelta` keeps its delta. `throws` assertions
become `throws(id, || expr)` over a `Result`. The generated Rust is
checked in and never edited by hand.

- **Ids** are `module/test/file.js:line:col`, relative to the vendored
  d3 root.
- **Ratchet.** `dev/harness/passing.txt` lists every passing id;
  `cargo run -p rustd3-harness -- --check` fails if a listed id fails,
  becomes a skip, or disappears.
- **Floors.** `passing.txt` also carries `#floor <module>/test/<file>.js
  <n>` lines; `--check` fails when a file passes fewer assertions than
  its floor, and prints translated, passing and skipped counts per
  file. A core phase's exit criterion is that every test file it owns
  passes at least 70% of its assertions, unless the phase's skip table
  names the file with a reason.
- **Translator budget.** Each core phase extends the translator within
  the budget its section states (typically 100–300 lines of
  JavaScript). Constructs outside the budget become recorded skips in
  named categories.
- **Blessing.** `--bless` (and `--bless-floors`) only after reading
  the diff, with the reason the new output is right in the commit body.
- **Behavior d3 does not test** (brush and drag, viridis tables, curve
  edge cases) is checked against d3 itself, run in Node from the
  vendored sources through the resolve hook (`node --import
  dev/harness/translate/d3-resolve.mjs script.mjs`), or against traces
  derived from d3's source where the behavior is DOM-bound.
- **matplotlib reference values** come from checked-in Python scripts
  and JSON under `dev/fixtures/`, compared through `Obs`, never byte for
  byte.

## Decisions and handoffs

**Decisions.** A design call this document does not make, or a place
where two of its statements cannot both hold, is written to
`.claude/decisions/phase-NN.md` (NN zero-padded) as `D{N}.{k}`: issue,
investigation with citations anchored to a version or commit, an
options table, a recommendation. The session then stops and reports.
Entries are append-only; a later entry that overturns one links back.
`.claude/DECISIONS.md` holds the format and an index of the per-phase
files. Measured values a phase is told to record (the benchmark
verdict, the PDF bound, the placement differences) go to the same
files.

**Handoffs.** At the end of every session the agent writes
`SESSION_HANDOFF-NN.md` next to this document (numbered sequentially):
what was done, harness counts and floors, decisions written, and, for
a phase that takes two sessions, the next step. `goon.yaml` beside it
describes the project for the resume workflow (Appendix B).

## Order of work

One agent works through the phases in their numeric order. Phase 31 is
conditional: it is built only when the benchmark verdict in
`decisions/phase-22.md` asks for it, and otherwise recorded as
skipped. The windowed benchmark of Phase 22 runs on the user's
hardware; agents build and smoke-test it headlessly and continue with
Phases 23–30 while the user runs it. If the user's result is not back
when Phase 30 ends, work continues with Phases 32–39 (none of them
needs Phase 31), and Phase 31 runs as soon as the verdict asks for it.
Phase 40 waits for the verdict and, when it asks, for Phase 31. The
verdict's closing line is written by whichever of Phase 31 and Phase
34 finishes its part last.

| Milestone | Phases | What exists at its end |
| --- | --- | --- |
| M0 Foundation | 1–3 | Workspace, test kit, grading harness; d3's path, ticks, format and ease ported and graded |
| M1 Core contract | 4–10 | Continuous, categorical and identity scales; ticks, locales and core axes; curves; recordings and M4; pointer input, zoom and pixel-space operations |
| M2 Figure layer and widget | 11–22 | rustd3-iced's core level; the drawing list and its outputs; typst text; the figure model, layout, marks and legends; the `Plot` widget with gestures and live data; the full-frame benchmark |
| M3 Core breadth | 23–30 | Color spaces and schemes, the derive, binned scales and array statistics, number and time locales and formats, time scales, shapes, brushes, drag and animation |
| Conditional | 31 | Tiles and pan translation in the widget, if the benchmark asks |
| M4 Complete figure | 32–39 | The two use cases; time, color and binned handles; the wheel figure; themes, colorbars, hatching; hover and brushing; PDF and GIF; statistical helpers and presets |
| M5 Release | 40 | The facade, documentation, and a release ready to publish |

**Dependencies.** A phase needs the phases listed (and, transitively,
theirs). Phases 8, 24, 25, 26 and 29 need only early phases and may be
pulled forward when a session has room.

| Phase | Needs | | Phase | Needs |
| --- | --- | --- | --- | --- |
| 2 | 1 | | 22 | 21 |
| 3 | 2 | | 23 | 7 |
| 4 | 3 | | 24 | 4 |
| 5 | 4 | | 25 | 6 |
| 6 | 5 | | 26 | 6 |
| 7 | 6 | | 27 | 9, 10 |
| 8 | 3 | | 28 | 26, 27 |
| 9 | 3 | | 29 | 8 |
| 10 | 5, 6 | | 30 | 10 |
| 11 | 3, 10 | | 31 | 22 and the user's benchmark run |
| 12 | 1, 7 | | 32 | 11, 17, 24, 28, 30 |
| 13 | 12 | | 33 | 17, 20, 21, 23, 25, 28 |
| 14 | 6, 7, 9, 10, 13 | | 34 | 22 (its code), 29, 33 |
| 15 | 8, 14 | | 35 | 23, 34 |
| 16 | 15 | | 36 | 20, 21, 30, 33 |
| 17 | 16 | | 37 | 12, 17, 34 |
| 18 | 11, 17 | | 38 | 25, 33, 34, 35 |
| 19 | 18 | | 39 | 29, 35, 38 |
| 20 | 19 | | 40 | every v0.1 phase (31 if built) |
| 21 | 20 | | | |

**Two-session phases.** Phases 9, 12, 14, 17, 23, 27 (27a/27b), 28,
33, 34, 35, 39 and, if built, 31 name a cut point; Phase 24 runs in the
session of Phase 23's last step or together with Phase 26.

**Before a phase starts.**

- `scripts/check.sh quick` is green; a red check is fixed first.
- The decision files of the phases it builds on are read.
- For phases with iced: `cargo check -p rustd3-iced --all-features`.
- `cargo update --dry-run` shows no forced changes.
- Before Phase 22's windowed run goes to the user: their display's
  refresh rate and scale factor.
- Before Phase 31: `decisions/phase-22.md` holds the user's result and
  the verdict; if it says the image path suffices, the handoff marks
  Phase 31 skipped.
- Before Phase 40: `cargo search rustd3`.

## M0 — Foundation

Workspace, test kit and grading harness, and the first port graded by d3's tests.

### Phase 1 — Workspace, tooling and fonts

*M0 Foundation · builds on nothing · about 650 lines (manifests, scripts, docs; about 120 lines of Rust), 6 commits*

#### Goal

The `rustd3` repository is a cargo workspace with all ten packages, every
library dependency except iced declared and locked, the lint and clippy
configuration that enforces the style rules and the no-ambient-clock
rule, `scripts/check.sh quick|full`, the project's `CLAUDE.md`, the
tracked `.claude/` directory (style guides, the decision-log format with
its index, the per-phase decision directory), licences, and a working
`rustd3-fonts` crate. Build times, the lock's package count and the
target-directory size are recorded in `.claude/decisions/phase-01.md`.
There is no library code beyond fonts.

#### Builds on

- Nothing in the workspace. The repository root is the directory that
  holds `.claude/map/rustd3/` (this document, `proto/`, `guides/`,
  `goon.yaml` and the session handoffs).
- Inputs read: `guides/RUST_STYLE.md`, `guides/OPAQUE.md`,
  `guides/DECISIONS.md`; `proto/fonts/` (`DejaVuSans.ttf` 759,720 B,
  `DejaVuSans-Bold.ttf` 708,920 B, `NewCMMath-Regular.otf` 1,306,268 B,
  `LICENSE-DejaVu` with the Bitstream Vera and Arev terms,
  `LICENSE-NewCMMath.txt` with the GUST Font License 1.0, i.e. LPPL 1.3c
  plus a renaming request); `proto/d3-src/*/LICENSE` (ISC); the Apache
  2.0 text from the registry (`kurbo-0.13.1/LICENSE-APACHE`, full text
  with appendix).
- The crate, feature and facade tables of Part I, Architecture and
  Dependencies.

#### Design

##### Layout

```text
Cargo.toml  Cargo.lock  clippy.toml  rustfmt.toml  .gitignore
LICENSE-MIT  LICENSE-APACHE  CLAUDE.md
src/lib.rs                                   facade
core/{Cargo.toml, src/lib.rs, LICENSE-d3}
derive/{Cargo.toml, src/lib.rs}
figure/{Cargo.toml, src/lib.rs}
fonts/{Cargo.toml, src/lib.rs, tests/main/main.rs, LICENSE, LICENSE-DejaVu,
       LICENSE-NewCMMath.txt, DejaVuSans.ttf, DejaVuSans-Bold.ttf, NewCMMath-Regular.otf}
iced/{Cargo.toml, src/lib.rs}
dev/{testkit,harness,examples,bench}/{Cargo.toml, src/lib.rs}
dev/clippy.toml  dev/fixtures/README.md
scripts/check.sh
.claude/{RUST_STYLE.md, OPAQUE.md, DECISIONS.md, decisions/phase-01.md}
```

Every `src/lib.rs` is a one-paragraph crate doc comment and nothing
else. The harness (Phase 2) and the bench (Phase 22) gain `src/main.rs`
in their phases.

##### Root `Cargo.toml`

```toml
[package]
name = "rustd3"
description = "d3's vocabulary and math for Rust, with figures and an iced widget"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
autotests = false

[features]
default = ["fonts"]
figure = ["dep:rustd3-figure"]
fonts = ["rustd3-figure?/fonts"]
raster = ["figure", "rustd3-figure/raster"]
png = ["figure", "rustd3-figure/png"]
gif = ["figure", "rustd3-figure/gif"]
pdf = ["figure", "rustd3-figure/pdf"]
tiny-skia = ["figure", "rustd3-figure/tiny-skia"]
helpers = ["figure", "rustd3-figure/helpers"]
derive = ["rustd3-core/derive"]
# No `iced` feature until iced 0.15 is on crates.io.

[dependencies]
rustd3-core.workspace = true
rustd3-figure = { workspace = true, optional = true }

[lints]
workspace = true

[workspace]
resolver = "3"
members = ["core", "derive", "figure", "fonts", "iced",
           "dev/testkit", "dev/harness", "dev/examples", "dev/bench"]
exclude = [".claude"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.97"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
rustd3-core = { path = "core", version = "0.1.0" }
rustd3-derive = { path = "derive", version = "0.1.0" }
rustd3-figure = { path = "figure", version = "0.1.0", default-features = false }
rustd3-fonts = { path = "fonts", version = "0.1.0" }
rustd3-testkit = { path = "dev/testkit" }
jiff = "0.2.38"
thiserror = "2"
syn = "2"
quote = "1"
proc-macro2 = "1"
typst = "0.15.1"
typst-layout = "0.15.1"
ttf-parser = "0.25"
kurbo = "0.13.1"
peniko = "0.6.1"
png = "0.18"
vello_cpu = { version = "0.3.0", default-features = false, features = ["std", "text", "u8_pipeline", "multithreading"] }
gif = "0.14"
krilla = "0.8.2"
tiny-skia = "0.12"
serde_json = "1"
resvg = "0.48.1"

[workspace.lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "warn"
allow_attributes_without_reason = "warn"
mod_module_files = "warn"
wildcard_enum_match_arm = "warn"
dbg_macro = "warn"
todo = "warn"
unimplemented = "warn"

[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = "line-tables-only"
```

`exclude = [".claude"]` keeps the prototypes under
`.claude/map/rustd3/proto/` out of the workspace. `serde_json` is a
workspace entry for dev use only (testkit and dev-dependencies); `serde`
is not a workspace entry: `dev/bench` declares it itself.

`wildcard_enum_match_arm` also fires on the wildcard arm that a foreign
`#[non_exhaustive]` enum requires (checked with `std::io::ErrorKind`).
That arm, the only sanctioned wildcard (Appendix A), carries
`#[expect(clippy::wildcard_enum_match_arm, reason = "non_exhaustive enum from <crate>")]`.

##### Member manifests

Every member except `rustd3-fonts` sets `version`, `edition`,
`rust-version` and `license` with `.workspace = true`; every member sets
`autotests = false`, `[lints] workspace = true` and a `description`.
`rustd3-fonts` omits `license` and sets `license-file = "LICENSE"`. The
phase that creates a crate's `tests/main/main.rs` adds `[[test]] name =
"main"`, `path = "tests/main/main.rs"` (one integration-test binary per
crate, further files as modules beside it; here only fonts has one).
These dependencies are final except iced, which Phase 11 adds:

| Package | `[dependencies]` | `[dev-dependencies]` | Features |
| --- | --- | --- | --- |
| rustd3-core | jiff, thiserror, rustd3-derive (optional) | rustd3-testkit, serde_json | `derive = ["dep:rustd3-derive"]` |
| rustd3-derive (`[lib] proc-macro = true`) | syn, quote, proc-macro2 | — | — |
| rustd3-figure | rustd3-core, rustd3-fonts (optional), typst, typst-layout, ttf-parser, kurbo, peniko, png, thiserror; optional vello_cpu, gif, krilla, tiny-skia | rustd3-testkit, resvg, serde_json | `default = ["fonts"]`, `fonts = ["dep:rustd3-fonts"]`, `raster = ["dep:vello_cpu"]`, `png = ["raster"]`, `gif = ["raster", "dep:gif"]`, `pdf = ["dep:krilla"]`, `tiny-skia = ["dep:tiny-skia"]`, `helpers = []` |
| rustd3-fonts (`license-file = "LICENSE"`) | — | ttf-parser | — |
| rustd3-iced (`publish = false`) | rustd3-core, thiserror, rustd3-figure (optional, `features = ["raster"]`) | rustd3-testkit | `default = ["fonts"]`, `fonts = ["rustd3-figure?/fonts"]` (weak, as in the facade: figures shown by the widget get the bundled fonts unless the app turns them off), `figure = ["dep:rustd3-figure"]` |
| rustd3-testkit (`publish = false`) | jiff (`features = ["tzdb-bundle-always"]`), serde_json, thiserror | — | — |
| rustd3-harness (`publish = false`) | rustd3-core, rustd3-testkit | — | — |
| rustd3-examples (`publish = false`) | rustd3-core (`features = ["derive"]`), rustd3-figure (`features = ["fonts", "png", "pdf", "gif", "helpers"]`) | rustd3-testkit | — |
| rustd3-bench (`publish = false`) | rustd3-core, rustd3-figure (`features = ["fonts", "png"]`), rustd3-examples (`path = "../examples"`), serde (`version = "1"`, `features = ["derive"]`), serde_json | rustd3-testkit | — |

- `png` is a normal dependency of rustd3-figure in every feature set
  (SVG embeds images as PNG); the `png` feature adds only the raster
  path. vello_cpu reaches rustd3-examples through rustd3-figure's
  `raster`; examples declares no renderer of its own.
- rustd3-bench depends on rustd3-examples so Phase 34's `wheels`
  workload reuses `rustd3_examples::wheels::figure` and `simulate`.
  There is no cycle: examples depends on core and figure (Phase 11 adds
  iced and rustd3-iced to both). `serde` with `derive` and `serde_json`
  are bench's own normal dependencies (replays and records, Phase 22),
  and bench is the only package in the workspace that depends on
  `serde`.
- `fonts` is a default figure feature because fonts are on by default
  and users with their own fonts turn them off (Part I, Architecture).
  The workspace entry for rustd3-figure sets `default-features = false`
  (edition 2024 rejects a member's `default-features = false` over an
  inheriting entry that keeps defaults; checked), so each dependent
  names `fonts` itself and the facade forwards it as a weak default
  feature, `rustd3-figure?/fonts`.
- `cargo tree -p rustd3-core -e normal --depth 1` shows jiff and
  thiserror only.

##### Lint configuration

```toml
# clippy.toml — library crates never read clocks or the system zone.
# Dev crates opt out in dev/clippy.toml.
allow-unwrap-in-tests = true
disallowed-methods = [
  { path = "std::time::Instant::now", reason = "time is a parameter; library code reads no clock" },
  { path = "std::time::SystemTime::now", reason = "time is a parameter; library code reads no clock" },
  { path = "jiff::Timestamp::now", reason = "time is a parameter; library code reads no clock", allow-invalid = true },
  { path = "jiff::Zoned::now", reason = "time is a parameter; library code reads no clock", allow-invalid = true },
  { path = "jiff::tz::TimeZone::system", reason = "zones are parameters; library code reads no system zone", allow-invalid = true },
]
```

clippy 1.97 honours `allow-invalid = true` (the jiff paths do not exist
in crates without jiff). `dev/clippy.toml` holds `allow-unwrap-in-tests
= true` and `disallowed-methods = []`: the nearest `clippy.toml` wins and
files are not merged, so it replaces the root file for every crate below
`dev/` (checked). Every `tests/main/main.rs` starts with
`#![allow(clippy::unwrap_used, reason = "test helpers outside #[test] fns")]`.

`rustfmt.toml`: `style_edition = "2024"`, `use_field_init_shorthand =
true`, `use_try_shorthand = true`, `newline_style = "Unix"`.

##### `scripts/check.sh`

```bash
#!/usr/bin/env bash
# quick: every commit's gate. full: end of every phase (run in the background).
set -euo pipefail
cd "$(dirname "$0")/.."
PACKAGES=(rustd3 rustd3-core rustd3-derive rustd3-figure rustd3-fonts rustd3-iced
          rustd3-testkit rustd3-harness rustd3-examples rustd3-bench)

touched() {  # packages with uncommitted changes or changed in HEAD
  { git diff --name-only HEAD; git ls-files --others --exclude-standard;
    git diff --name-only HEAD~1 HEAD 2>/dev/null || true; } | while read -r f; do
    case "$f" in
      core/*) echo rustd3-core ;;        derive/*) echo rustd3-derive ;;
      figure/*) echo rustd3-figure ;;    fonts/*) echo rustd3-fonts ;;
      iced/*) echo rustd3-iced ;;        dev/testkit/*) echo rustd3-testkit ;;
      dev/harness/*) echo rustd3-harness ;; dev/examples/*) echo rustd3-examples ;;
      dev/bench/*) echo rustd3-bench ;;  src/*) echo rustd3 ;;
      Cargo.toml|Cargo.lock|clippy.toml|dev/clippy.toml) printf '%s\n' "${PACKAGES[@]}" ;;
    esac
  done | sort -u
}

quick() {
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  local pkgs=("$@")
  if [ ${#pkgs[@]} -eq 0 ]; then mapfile -t pkgs < <(touched); fi
  for p in "${pkgs[@]}"; do TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features; done
}

full() {
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  for p in "${PACKAGES[@]}"; do
    TZ=Asia/Kathmandu cargo test -q -p "$p" --no-default-features
    TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features
  done
  TZ=Asia/Kathmandu cargo test -q -p rustd3-figure   # default features: fonts
  TZ=Asia/Kathmandu cargo test -q -p rustd3
  RUSTDOCFLAGS="-D warnings" cargo doc -q --workspace --no-deps --all-features
}

case "${1:-quick}" in
  quick) shift || true; quick "$@" ;;
  full) full ;;
  *) echo "usage: scripts/check.sh quick [package…] | full" >&2; exit 2 ;;
esac
```

Clippy runs once over the workspace with all features; tests run per
package with no features and with all features, plus the default set
where it differs (figure and the facade, whose default is `fonts`).
Later phases extend the script, each in its own commit: Phase 2 the
harness ratchet (`quick`) and the translator freshness check (`full`);
Phase 5 the opt-in release fuzz run (`quick`, `RUSTD3_FUZZ=1`); Phase 11
the iced tests once per `ICED_TEST_BACKEND`, with `RUSTD3_ALLOW_NO_GPU`
(`full`); Phase 22 the report-only benchmark smoke run; Phase 26 the
generated-table freshness check (`full`); Phase 40 the `release` mode.
None of those steps is added here.

##### `rustd3-fonts`

```rust
//! Bundled fonts, so a figure renders the same on every machine.

/// One bundled font file.
#[derive(Debug, Clone, Copy)]
pub struct Font { family: &'static str, bold: bool, data: &'static [u8] }

impl Font {
    pub const fn family(&self) -> &'static str;   // the `name` table's family (id 1)
    pub const fn bold(&self) -> bool;
    pub const fn data(&self) -> &'static [u8];    // TrueType or OpenType bytes
}

/// DejaVu Sans 2.37, regular (`include_bytes!("../DejaVuSans.ttf")`).
pub const DEJAVU_SANS: Font = Font { family: "DejaVu Sans", bold: false, data: … };
pub const DEJAVU_SANS_BOLD: Font = …;  // "../DejaVuSans-Bold.ttf", "DejaVu Sans", bold
pub const NEW_CM_MATH: Font = …;       // "../NewCMMath-Regular.otf", "NewComputerModernMath"
pub const ALL: [Font; 3] = [DEJAVU_SANS, DEJAVU_SANS_BOLD, NEW_CM_MATH];
```

Every public item has a doc comment (elided above). The font files are
copied from `proto/fonts/` byte for byte (compare with `sha256sum`), as
are `LICENSE-DejaVu` and `LICENSE-NewCMMath.txt`. `fonts/LICENSE` is
one summary paragraph (the crate's code is MIT OR Apache-2.0; the DejaVu
fonts are under the Bitstream Vera licence with the Arev and
public-domain changes, full text in `LICENSE-DejaVu`; New Computer
Modern Math is under the GUST Font License, full text in
`LICENSE-NewCMMath.txt`), followed by the full text of `LICENSE-DejaVu`
and then the full text of `LICENSE-NewCMMath.txt`, concatenated. Cargo's
`license-file` points at that one file; an SPDX expression for the
fonts crate is a 0.2 item (see Later).

##### Licences

`LICENSE-MIT` (copyright 2026 Travis Hammond) and `LICENSE-APACHE`
(copied from the kurbo 0.13.1 crate) at the root. `core/LICENSE-d3`:
the ISC text once, preceded by the copyright line of each module core
ports, copied from its `proto/d3-src/<module>/LICENSE`: d3-array
2010-2023, d3-axis 2010-2021, d3-brush 2010-2021, d3-color 2010-2022,
d3-dispatch 2010-2021, d3-drag 2010-2021, d3-ease 2010-2021, d3-format
2010-2026, d3-interpolate 2010-2021, d3-path 2015-2022, d3-scale
2010-2021, d3-shape 2010-2022, d3-time 2010-2024, d3-time-format
2010-2021, d3-timer 2010-2021, d3-zoom 2010-2021, internmap 2021 (all
"Mike Bostock"). Phase 23 appends d3-scale-chromatic's copyright line
and its ColorBrewer notice (Apache-2.0, Cynthia Brewer) with the
schemes. Per-package copies of the MIT and Apache texts are Phase 40's
`include` work. `dev/harness` gets its notices with the vendored files
in Phase 2.

##### `.gitignore`

```text
/target
**/target/
*.new
!.claude/
```

`**/target/` covers prototypes built in place under `proto/`; `*.new`
is the snapshot-mismatch output of Phase 2's `snapshot::Store`;
`!.claude/` keeps the directory tracked against a global ignore.

##### `CLAUDE.md` (at most 120 lines)

1. **What rustd3 is**, in one paragraph. The spec is
   `.claude/map/rustd3/rustd3.md`: Part I is the design; Implementation
   holds the front matter (from "How to use this part" to "Order of
   work"), one section per phase, Later (0.2), Appendix A (the dispatch
   preamble: reading order, allowed and forbidden commands, the
   pre-commit checklist, forbidden patterns) and Appendix B (resume). `proto/` is evidence and
   algorithms, never style.
2. **Layout**: the tree above, one line per package.
3. **Commands**: `scripts/check.sh quick [pkg…]` and `full`; from
   Phase 2 `cargo run -p rustd3-harness -- --check | --bless
   [--bless-floors] [--allow-drop]`, `node
   dev/harness/translate/translate.mjs [--check]`, d3 in Node through
   `node --import ./dev/harness/translate/d3-resolve.mjs`; fixture
   scripts only when a phase section says so; tests run under
   `TZ=Asia/Kathmandu`.
4. **Rules**, one line each: `.claude/RUST_STYLE.md`;
   `.claude/OPAQUE.md` with three exemptions (`PathContext` keeps d3's
   canvas argument lists, N4; field-less transforms are type-level
   strategies, N5; the plain data types with public fields keep them,
   N2); the kept composite names (`PathContext`, `SvgPath`, `LineOf`,
   `AnyContinuous`, `ScaleRef`, `TickRule`, `TickFormat`, `GlyphRun`,
   `BrushX`, `BrushY`, `BrushEvent`, `InvalidFormat`,
   `ParseColorError`, `IcedPath`) and no others; no ambient clock or
   zone in library code (clippy-enforced); no mutable global state
   beyond `OnceLock` statics of parsed fonts, the id and eviction
   counters, and the bounded typst cache; `f64` geometry, `f32` only at
   backend boundaries and in `GlyphRun`/`Glyph`; dependencies exactly
   as rustd3.md lists them, any other manifest change is a decision;
   the iced pin moves only in its own commit with the iced tests rerun
   on both backends; no AI attribution; Appendix A's forbidden list;
   private submodules re-exported by their parent (one public path per
   item); `rustd3-harness` is a binary crate whose modules are private,
   so `missing_docs` never fires on its generated code and no
   `allow(missing_docs)` is added there.
5. **Decisions**: a design call rustd3.md does not make is written as
   `D{phase}.{k}` in `.claude/decisions/phase-NN.md` (format in
   `.claude/DECISIONS.md`), and the session stops and reports.
6. **Commits**: conventional, `type(scope): summary`, scope = crate;
   each commit passes `check.sh quick`, the phase's last passes `full`.
7. **Execution**: one agent, phases in rustd3.md's order (Phase 31 only
   when the benchmark verdict in `decisions/phase-22.md` asks for it).
   Phases after 10 add to core freely; changing or removing a public
   item built in Phases 1–10 is a decision whose commit also updates
   every call site in figure, iced and the dev crates, with `check.sh
   quick` green. Handoffs are `SESSION_HANDOFF-NN.md`, numbered
   sequentially, next to rustd3.md.
8. **Reading order** for a session: this file;
   `.claude/RUST_STYLE.md` and `.claude/OPAQUE.md`;
   `.claude/DECISIONS.md` and the decision files of the phases the
   current one builds on; rustd3.md's Implementation front matter, the
   phase's section and the Part I sections it names; the `proto/` files
   it cites; the newest handoff if the phase is half done.

##### `.claude/DECISIONS.md`

Copied from `guides/DECISIONS.md` and adapted; it contains no decision
entries. Its header states:

- **Where entries go.** Implementation decisions are written to
  `.claude/decisions/phase-NN.md` (NN zero-padded) as `D{N}.{k}`, N the
  phase number and k counting from 1 within the phase. One file per
  phase; the files are tracked (`.gitignore` re-includes `.claude/`).
- **When.** A design call that rustd3.md does not make, or a place
  where it is wrong: write the entry, then stop and report.
- **Entry shape**, from the guide: `## D{N}.{k} — title`, a line with
  date, context (phase, step, commit) and status (`open` with a
  recommendation until the user answers, then `decided`), then Issue,
  Investigation (citations anchored to a pinned version or commit),
  Options (the table), Decision, Why, Followups.
- **Conventions.** Append-only; reference earlier entries by id instead
  of re-arguing them; overturning one is a new entry that links back,
  plus a forward link in the old entry's Followups.

Then `## Index`: one line per per-phase file, appended by the phase that
creates it. This phase writes the first: ``- `decisions/phase-01.md` —
Phase 1: build measurements``.

##### Measurements

Recorded in `.claude/decisions/phase-01.md` with `nproc` and the date:
cold `check.sh quick` after `cargo clean`, warm `quick` with no changes,
`full`, `du -sh target`, and the lock's package count (`grep -c '^name
= ' Cargo.lock`). Reference from a 2-core container: a cold debug build
of the whole dependency set (all features, `line-tables-only`) took
about 6 minutes wall and left about 1.8 GB in `target`; a no-op rebuild
took 0.6 s. Record the numbers; there is no threshold.

##### Decisions this phase may meet

- The lock resolves a newer patch release than exit criterion 2 lists
  (published after the versions were measured) — keep the resolved
  version if `check.sh full` passes and note the version in the entry;
  record it as a decision.

#### Steps

1. **chore: workspace skeleton.** Root and member manifests, crate doc
   comments, `rustfmt.toml`, `.gitignore`, `LICENSE-MIT`,
   `LICENSE-APACHE`, `core/LICENSE-d3`; `git init -b main` if the root
   is not yet a repository, with `.claude/map/rustd3/` committed as it
   stands; `cargo generate-lockfile`; `cargo check --workspace
   --all-features`; verify exit criteria 1 and 2 and the dependency
   checks of criterion 7.
2. **chore: lints and clippy configuration.** `clippy.toml`,
   `dev/clippy.toml`, the workspace lint tables. Verify criterion 5 with
   a scratch edit in a library crate and in `dev/examples`, then revert
   both.
3. **feat(fonts): bundled fonts.** The three font files, `LICENSE`,
   `LICENSE-DejaVu`, `LICENSE-NewCMMath.txt`, `src/lib.rs`, `[[test]]`
   in `fonts/Cargo.toml`, and `tests/main/main.rs` with the spot checks
   (ttf-parser `Face::parse`, `names()` filtered to `name_id::FAMILY`,
   `is_bold`, `tables().math`, `number_of_glyphs`, `glyph_index`).
4. **chore: check script.** `scripts/check.sh` (executable); run both
   modes.
5. **docs: CLAUDE.md and .claude.** `.claude/RUST_STYLE.md` and
   `.claude/OPAQUE.md` copied from `guides/`; `.claude/DECISIONS.md`;
   `CLAUDE.md`; `dev/fixtures/README.md` (what fixtures are and how each
   kind is regenerated: d3 through the resolve hook, matplotlib 3.11.2
   through the checked-in Python scripts).
6. **docs: measurements.** `.claude/decisions/phase-01.md` with the
   Measurements; the index line in `.claude/DECISIONS.md`.

#### Exit criteria

1. `cargo metadata --no-deps --format-version 1` lists exactly the ten
   packages of the member table; `Cargo.lock` is committed and `grep -c
   '^name = "iced' Cargo.lock` prints 0 (iced arrives in Phase 11).
2. `cargo update --dry-run` reports no changes; the lock holds jiff
   0.2.38, typst 0.15.1, typst-layout 0.15.1, kurbo 0.13.1, peniko
   0.6.1, vello_cpu 0.3.0, krilla 0.8.2, png 0.18.1, tiny-skia 0.12.0
   and resvg 0.48.1 (`grep -A1 'name = "<crate>"' Cargo.lock`).
3. `scripts/check.sh quick` and `scripts/check.sh full` exit 0.
4. `cargo test -p rustd3-fonts` passes the three parse tests (spot
   checks).
5. A library crate calling `std::time::Instant::now()` fails `cargo
   clippy -- -D warnings` with `disallowed_methods`; the same call in
   `dev/examples` passes (checked once by hand, not committed).
6. `git check-ignore -v .claude/DECISIONS.md` prints nothing (tracked);
   `git ls-files .claude/map/rustd3/rustd3.md` prints the path;
   `.claude/decisions/phase-01.md` holds the measurements;
   `.claude/DECISIONS.md` has the format header and one index line, and
   `grep -c '^## D' .claude/DECISIONS.md` prints 0.
7. `cargo tree -p rustd3-core -e normal --depth 1` shows jiff and
   thiserror only; `grep -ln '^serde ' Cargo.toml */Cargo.toml
   dev/*/Cargo.toml` prints only `dev/bench/Cargo.toml`; `cargo
   metadata --no-deps --format-version 1` reports `license: null` and
   `license_file: "LICENSE"` for rustd3-fonts and `MIT OR Apache-2.0`
   for the nine others.
8. `CLAUDE.md` has at most 120 lines (`wc -l`) and names
   `.claude/map/rustd3/rustd3.md`.

#### Spot checks

Parse tests in `fonts/tests/main/main.rs`; values measured with
ttf-parser 0.25.1 on the files in `proto/fonts/` (and cross-checked
with fontTools).

| Font | family (`name` id 1) | `is_bold` | MATH table | glyphs | U+202F, U+2212, µ, € |
| --- | --- | --- | --- | --- | --- |
| `DEJAVU_SANS` | DejaVu Sans | false | present | 6,253 | all present |
| `DEJAVU_SANS_BOLD` | DejaVu Sans | true | absent | 6,196 | all present |
| `NEW_CM_MATH` | NewComputerModernMath | false | present | 8,603 | all present |

`Font::family()` equals the parsed family and `Font::bold()` the parsed
`is_bold` for each of `ALL`. U+202F (narrow no-break space) matters: it
is the neutral locale's group separator (Phase 3); U+2212 is its minus.

#### Not in this phase

- No core, figure or iced code; no `tests/main/main.rs` outside fonts;
  no harness, translator or vendored d3 (Phase 2).
- No iced dependency (Phase 11).
- No facade re-exports, per-package licence copies or `include` lists
  (Phase 40); no `iced` facade feature until iced 0.15 is on crates.io.
- Nothing under `proto/` is built or modified.

### Phase 2 — Test kit and grading harness

*M0 Foundation · builds on phase 1 · about 1,120 lines of Rust and 550 of JavaScript, plus about 5 MB of vendored d3, 6 commits*

#### Goal

`rustd3-testkit` gives every crate the tolerant `Obs` comparison, pixel
statistics with the 32 px tile score, text snapshots that fail on a
missing reference unless `RUSTD3_BLESS=1`, and `zone(name)` over jiff's
bundled database. `dev/harness` holds d3's sources and tests, the
offline translator with location ids, the Node resolve hook, the
checked-in generated Rust, and the `rustd3-harness` binary: a report
with per-file counts, and a `--check` ratchet that fails on any listed
id that stops passing and on any file that falls below its floor. The
whole pipeline is proven on a stub before core has any code: one d3
test file is translated, run, blessed and checked.

#### Builds on

- Phase 1: the workspace; the `rustd3-testkit` manifest (jiff with
  `tzdb-bundle-always`, serde_json, thiserror) and the `rustd3-harness`
  manifest (rustd3-core, rustd3-testkit); `dev/clippy.toml`;
  `scripts/check.sh`; `.gitignore`'s `*.new`.

#### Design

##### rustd3-testkit layout

```text
dev/testkit/src/lib.rs            pub mod obs; pub mod pixels; pub mod snapshot; mod zone; pub use zone::zone;
dev/testkit/src/obs.rs            Obs, ToObs (+ private mods tolerance, token, json)
dev/testkit/src/obs/tolerance.rs  Tolerance
dev/testkit/src/obs/token.rs      Token, tokenize
dev/testkit/src/obs/json.rs       TryFrom<&serde_json::Value> for Obs, obs::Error
dev/testkit/src/pixels.rs         Image, Diff (+ private mod error: pixels::Error)
dev/testkit/src/snapshot.rs       Store
dev/testkit/src/zone.rs           zone
```

Submodules are private and their parent re-exports their items, so each
item has one public path (`obs::Tolerance`, `pixels::Error`). This is
the workspace convention from here on.

##### `obs`

Lifted from `proto/poc/obs/src/lib.rs` (269 lines: `Obs`, `ToObs`,
`Tolerance`, `Token`, `tokenize`, `approx_eq`, `strict_eq` and five
tests), with the free functions turned into methods and the modules
split as above.

```rust
/// A value observed in a test, independent of how either language prints it.
#[derive(Debug, Clone, PartialEq)]
pub enum Obs { Null, Bool(bool), Num(f64), Str(String), List(Vec<Obs>) }

/// Converts a Rust value into an [`Obs`].
pub trait ToObs { fn to_obs(&self) -> Obs; }

impl Obs {
    /// Equal within `tolerance`: numbers within max(abs, rel × magnitude);
    /// strings token by token; a decimal in a string may differ by one
    /// unit in its last printed place.
    pub fn approx_eq(&self, other: &Obs, tolerance: &Tolerance) -> bool;
    /// JavaScript's `Object.is`: NaN equals NaN, 0 differs from −0.
    pub fn strict_eq(&self, other: &Obs) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Tolerance { rel: f64, abs: f64, last_place: bool }
impl Default for Tolerance { /* rel 1e-9, abs 1e-9, last_place true */ }
impl Tolerance {
    /// The default with the absolute bound raised to `abs` (d3's assertInDelta).
    pub fn at_least(abs: f64) -> Self;
    /// Exact strings: no last-place slack.
    pub fn without_last_place(self) -> Self;
}

pub enum Token { Text(String), Num { value: f64, places: u32 } }
/// Splits text and numbers; U+2212 reads as `-`.
pub fn tokenize(s: &str) -> Vec<Token>;
```

`ToObs` impls: `Obs`, `()` (→ `Null`), `f64`, `f32`, `bool`, `i32`,
`i64`, `u32`, `usize`, `str`, `String`, `[T]`, `Vec<T>`, `[T; N]`, `&T`,
`Option<T>` (`None` → `Null`), `(A, B)` (→ a two-element `List`), and
`impl<T: ToObs, E: fmt::Display> ToObs for Result<T, E>`: `Ok(v)` →
`v.to_obs()`, `Err(e)` → `Obs::Str(format!("error: {e}"))`, so a
`Result` graded with `Strict` fails loudly, with the error's text, when
it is `Err`. `Tolerance` has private fields; nothing outside testkit
sets them except through the two builders.

`impl TryFrom<&serde_json::Value> for Obs` maps null, bool, number,
string and array; an object is `Err(obs::Error::Object(text))` carrying
the object's JSON text. Fixture tests navigate objects with serde_json
and convert the leaves.

##### `pixels`

```rust
/// A borrowed RGBA8 image, row-major, 4 bytes per pixel.
#[derive(Debug, Clone, Copy)]
pub struct Image<'a> { width: u32, height: u32, rgba: &'a [u8] }
impl<'a> Image<'a> {
    pub fn new(width: u32, height: u32, rgba: &'a [u8]) -> Result<Self, pixels::Error>;
    pub fn width(&self) -> u32;
    pub fn height(&self) -> u32;
}

/// How two images of the same size differ, channel by channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diff { max: u8, over: u64, channels: u64, tile: f64, worst: (u32, u32) }
impl Diff {
    pub fn between(a: &Image<'_>, b: &Image<'_>, threshold: u8) -> Result<Diff, pixels::Error>;
    /// Largest absolute channel difference, 0–255.
    pub fn max(&self) -> u8;
    /// Channels whose difference exceeds the threshold.
    pub fn over(&self) -> u64;
    /// `over` as a share of all channels.
    pub fn share_over(&self) -> f64;
    /// Worst 32 px tile: mean absolute channel difference / 255.
    pub fn tile_score(&self) -> f64;
    /// Top-left pixel of the worst tile, for failure messages.
    pub fn worst_tile(&self) -> (u32, u32);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{width}×{height} needs {expected} bytes, got {actual}")]
    Length { width: u32, height: u32, expected: usize, actual: usize },
    #[error("sizes differ: {a:?} against {b:?}")]
    Size { a: (u32, u32), b: (u32, u32) },
}
```

**The tile score.** Tiles are 32 × 32 px anchored at (0, 0); right and
bottom tiles may be smaller and are averaged over their own channel
count. A tile's score is its mean absolute channel difference divided
by 255; the image's score is the **maximum** over tiles, not the mean,
because a localized defect (a missing glyph, a shifted marker) must not
average away over a large image. Inputs are compared as given (callers
pass the same alpha convention on both sides). An empty image scores 0.

##### `snapshot`

```rust
/// Text references in one directory.
#[derive(Debug, Clone)]
pub struct Store { dir: PathBuf, bless: bool }
impl Store {
    /// References under `dir` (callers pass
    /// `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots")`).
    /// Reads `RUSTD3_BLESS` once: only the value `1` blesses.
    pub fn new(dir: impl Into<PathBuf>) -> Self;
    /// Panics unless `dir/name` equals `actual` byte for byte. A missing
    /// or different reference is written (and the call passes) only when
    /// blessing; otherwise the panic names the path, the first differing
    /// line (1-based, both versions) and both line counts, and writes
    /// `actual` to `dir/name.new` for inspection.
    pub fn check(&self, name: &str, actual: &str);
}
```

`name` must match `[A-Za-z0-9_][A-Za-z0-9_.-]*` (it may carry an
extension such as `trace.svg`); anything else panics. Stored references
are text only; raster checks compare the workspace's own renderers at
test time.

##### `zone`

```rust
/// A time zone from jiff's bundled IANA database, never the host's.
pub fn zone(name: &str) -> jiff::tz::TimeZone;
```

A `OnceLock<TimeZoneDatabase>` holds `TimeZoneDatabase::bundled()`;
`zone` asserts `!db.is_definitively_empty()` (the message names the
`tzdb-bundle-always` feature) and panics on an unknown name with jiff's
error. Tests never call `TimeZone::get`, which reads the host's tzdata
first.

##### Vendored d3

```text
dev/harness/d3/<module>/{LICENSE, package.json, src/**}   from proto/d3-src/<module>
dev/harness/d3/<module>/test/**                          from proto/d3-tests/<module>
dev/harness/d3/{d3-format,d3-time-format}/locale/*.json  from proto/d3-tests/<module>/locale
dev/harness/translate/{translate.mjs, expr.mjs, rules/*.mjs}
dev/harness/translate/{d3-resolve.mjs, d3-resolve-hooks.mjs}
dev/harness/translate/vendor/acorn/{acorn.mjs, LICENSE}
```

`proto/d3-src/` has 20 packages (the 14 tested modules plus d3-drag,
d3-selection, d3-transition, d3-dispatch, d3-timer and internmap);
`proto/d3-tests/` has the 14 tested modules, with `locale/` for
d3-format and d3-time-format; `package.json` and `LICENSE` are identical
in both trees. Copy with `cp -r` and never edit
(`d3-time/test/test-time-zones` comes along; the translator ignores it).
The per-module `LICENSE` files and acorn's (acorn 8.19.0, MIT) are the
harness's notices. The hook, from `proto/tr/{d3-resolve.mjs,
d3-resolve-hooks.mjs}`, resolves `/^(d3-[a-z-]+|internmap)$/` to `new
URL("../d3/<spec>/src/index.js", import.meta.url)`; every other
specifier falls through, so d3 runs from `dev/harness/d3` with no
`node_modules`.

##### Location ids

An assertion's id is `<package>/test/<file>:<line>:<column>`, the test
file's path relative to the vendored d3 root `dev/harness/d3/` — for
example `d3-ease/test/linear-test.js:6:3`. Line and column are acorn's
`loc.start` of the assertion's `CallExpression` (line 1-based, column
converted to 1-based). Ids are stable under edits elsewhere in the file
and unique even for `a(), b()` sequences. An assertion inside a loop is
one id with several records; it passes iff every record passes and at
least one exists.

##### Translator

`proto/tr/translate.mjs` (346 lines) is the starting point: an
expression translator with typed values (`{c, t}`), `Skip`, the
assertion kinds (`strictEqual`, `deepStrictEqual`, `equal`, `isNaN`,
`assertInDelta`, `assertPathEqual`, `throws`, `assert`) and setup skips;
it reproduces `proto/poc/harness/src/generated.rs` byte for byte
(checked). It is split into:

- `translate.mjs` — CLI and driver: the suites table `{ suite, package,
  files, rules }`, file walking in listed order, ids, output, `--check`,
  the summary. Paths come from `import.meta.url`, so it runs from any
  directory.
- `expr.mjs` — the generic JavaScript-to-Rust translation (typed values,
  literals, arithmetic, arrays, `Math.*`, variables, statements,
  sequence expressions, the assertion kinds, setup skips), with
  module-specific names delegated to the suite's rules.
- `rules/<d3 module>.mjs` — identifier and method mappings per module.
  This phase has `rules/ease.mjs` for the stub only; Phase 3 and later
  core phases add one file per module.

`assert.throws(() => expr)` translates to `throws("<id>", || expr')`
over an expression that returns `Result` (for example `|| p.try_arc(…)`
in Phase 3).

**File-level helpers.** A top-level `const` or `function` of a test
file is emitted as a Rust `let` inside each test that uses it, or as an
`fn` in the generated module, when it is a pure expression the
translator already handles; otherwise every assertion that uses it is a
skip with the prefix `helpers:`.

**Skip reasons** carry a prefix the report groups by: `js:` (JavaScript
semantics: coercion, identity, `instanceof`, `undefined`), `types:`
(ruled out by the Rust types), `api:` (left out of the Rust API by
design: this document or a recorded decision), `gap:` (not ported yet;
names the phase, `gap: phase 26`), `helpers:` (a file-level helper the
translator cannot express). A setup skip gives every assertion of the
test its own id and the setup statement's reason.

**Output**, regenerated in full on each run: `src/generated.rs` declares
one private module per suite and `pub const SUITES: &[registry::Suite]`;
each `src/generated/<suite>.rs` starts with a "Generated by
translate.mjs from d3's own test files. Do not edit." line and
`#![allow(unused_mut, unused_parens, clippy::all, reason = "generated
from JavaScript")]`, then one function per d3 test (its name as a
comment) and the two tables:

```rust
fn t0() {   // linear-test.js: easeLinear(t) returns the expected results
    check("d3-ease/test/linear-test.js:6:3", Kind::Strict, &stub::ease_linear(0.0), &0.0);
    check("d3-ease/test/linear-test.js:7:3", Kind::InDelta(1e-6), &stub::ease_linear(0.1), &0.1);
    // … through line 16
}
fn t1() {   // linear-test.js: easeLinear(t) coerces t to a number
    skip("d3-ease/test/linear-test.js:20:3", "js: argument is not a number (coercion)");
    skip("d3-ease/test/linear-test.js:21:3", "js: object argument (valueOf coercion)");
}
pub const TESTS: &[Test] = &[Test { name: "…", run: t0 }, Test { name: "…", run: t1 }];
pub const ASSERTS: &[Assertion] = &[
    Assertion { id: "d3-ease/test/linear-test.js:6:3", test: "…", skip: None }, /* … */
];
```

`--check` regenerates in memory and exits 1 listing any file under
`src/generated/` that differs, is missing or is extra.

##### Harness binary

`rustd3-harness` is a binary crate: `src/main.rs` replaces Phase 1's
`src/lib.rs` (the crate doc moves to `main.rs`). Its modules are
private (`mod generated;`), so no item is exported and `missing_docs`
never fires on generated code; nothing adds `allow(missing_docs)`.
Modules under `dev/harness/src/`: `main.rs` (CLI: no flag reports;
`--check`; `--bless [--allow-drop] [--bless-floors]`; `--failing [id
prefix]`), `registry.rs` (the generated tables' types), `prelude.rs`
(`check`, `skip`, `throws`, `Kind`, and d3's test helpers as phases need
them), `record.rs` (thread-local recorder), `run.rs` (every `Test` under
`catch_unwind`, records folded into outcomes), `ratchet.rs`, `stub.rs`
(deleted in Phase 3), `generated.rs` with `generated/*.rs`; and
`dev/harness/passing.txt`. The run loop, the two verdicts, `Kind` and
`d3_normalize` (d3-shape's `normalizePath`) come from
`proto/poc/harness/src/{main.rs,prelude.rs}`; the prototype's numeric
ids and printing style do not.

```rust
// registry.rs
pub struct Suite { pub name: &'static str, pub tests: &'static [Test], pub asserts: &'static [Assertion] }
pub struct Test { pub name: &'static str, pub run: fn() }
pub struct Assertion { pub id: &'static str, pub test: &'static str, pub skip: Option<&'static str> }

// prelude.rs
pub enum Kind {
    Strict,        // strictEqual, deepStrictEqual, equal: exact in d3
    InDelta(f64),  // assertInDelta: absolute tolerance
    PathEq,        // assertPathEqual: normalizePath (integers within 1e-6, else 3 decimals)
    Throws,        // assert.throws: the call must return Err
}
pub fn check<A: ToObs + ?Sized, B: ToObs + ?Sized>(id: &'static str, kind: Kind, actual: &A, expected: &B);
pub fn skip(id: &'static str, reason: &'static str);
/// `assert.throws`: records a `Kind::Throws` record that passes under both
/// verdicts iff `f()` returns `Err`; an `Ok(v)` fails and reports `v`.
pub fn throws<T: ToObs, E: fmt::Debug>(id: &'static str, f: impl FnOnce() -> Result<T, E>);

// run.rs
pub enum Outcome { Pass, Fail { actual: Obs, expected: Obs }, Skip(&'static str), NotReached }

// ratchet.rs
pub struct Ratchet { listed: BTreeSet<String>, floors: BTreeMap<String, usize> }
impl Ratchet {
    pub fn load(path: &Path) -> std::io::Result<Self>;
    pub fn check(&self, outcomes: &BTreeMap<&'static str, Outcome>) -> Verdict;
    /// Rewrites `path` with every passing id. Floor lines are kept verbatim,
    /// unless `floors`: then every translated file's floor becomes its
    /// current tolerant-passing count, new files included.
    pub fn bless(path: &Path, outcomes: &BTreeMap<&'static str, Outcome>, floors: bool)
        -> std::io::Result<Change>;
}
pub struct Verdict { pub regressions: Vec<Regression>, pub new: Vec<&'static str> }
pub enum Regression {
    Fails(String),
    Skipped(String, &'static str),
    NotReached(String),
    Missing(String),
    Floor { file: String, passing: usize, floor: usize },
}
pub struct Change { pub added: Vec<String>, pub removed: Vec<String>, pub floors: Vec<(String, usize)> }
```

**Verdicts.** `check` records both verdicts, as the prototype did: d3's
own semantics (`Strict` → `strict_eq`; `InDelta(d)` → within d;
`PathEq` → normalized strings equal; `Throws` → `Err`) and the tolerant
one (`approx_eq` with `Tolerance::default()`, or `at_least(d)` for
`InDelta`). The ratchet uses the tolerant verdict. A record that passes
d3's semantics but fails the tolerant format is an internal error
(exit 1). At start the runner asserts that the ids in `ASSERTS` are
unique and that every recorded id is listed there.

**`passing.txt`.** Line 1 is a `#` comment; then one line per translated
file, `#floor <package>/test/<file>.js <n>` (n = the tolerant passes the
file must keep), sorted by file; then the ids, one per line, sorted by
(package, file, line, column) numerically, so `:6:` precedes `:10:`.

**Semantics.** `--check` exits 1 if any listed id fails, is a skip, was
not reached (its test panicked first; the panic message is printed) or
is missing from `ASSERTS`, or if a file's tolerant passes fall below its
floor. It prints ids that pass but are not listed as "N newly passing:
read the diff, then --bless" and still exits 0. `--bless` rewrites the
ids (floors verbatim), prints the added and removed ids, and refuses
(exit 1, file untouched) to remove a listed id unless `--allow-drop` is
given; `--bless --bless-floors` also sets every floor to the current
passing count and adds floor lines for new files. Exit codes: 0 ok, 1
regression or internal error, 2 usage.

**The 70 % rule.** Each core phase's exit criteria require every file it
translates to pass at least 70 % of its assertions, unless that phase's
skip table names the file with a reason. The phase agent checks this
when it blesses floors; the tool enforces only the recorded floors. The
stub file passes 11 of 13, 85 %.

**Report.** Per suite and per file: first the line `<file>: translated
T / passing P / skipped S`, then assertions, translated, skips by
prefix, not reached, d3-semantics passes, tolerant passes, failing; then
the failing ids with actual and expected (all with `--failing`, else the
first 20).

##### The stub

`src/stub.rs` holds `pub fn ease_linear(t: f64) -> f64 { t }` with a doc
comment saying it stands in for d3-ease's `easeLinear` so the harness
can prove translation, grading and the ratchet before core has code,
and that Phase 3 deletes it. `rules/ease.mjs` maps the identifier
`easeLinear` to the callable `stub::ease_linear`. The suites table has
one suite, `stub`, with `d3-ease/test/linear-test.js`.

##### Decisions this phase may meet

- None expected. A design question this section does not answer is
  recorded in `decisions/phase-02.md` and the phase stops.

#### Steps

1. **feat(testkit): Obs and tolerant comparison.** `obs.rs` and its three
   submodules from `proto/poc/obs`, the impls (with `()` and `Result`)
   and the JSON conversion; the five prototype tests and the `obs` spot
   checks.
2. **feat(testkit): pixel statistics, snapshots and zones.**
   `pixels.rs`, `snapshot.rs` (tests write into a directory under
   `std::env::temp_dir()` named after the test), `zone.rs`; spot checks.
3. **chore(harness): vendor d3, acorn and the resolve hook.** Copy per
   the layout, point the hook at `../d3/`, verify exit criteria 6–7.
4. **feat(harness): translator with location ids.** Split the
   translator as designed, prefixes on every skip reason, `throws`,
   file-level helpers, the stub suite; generate `src/generated.rs` and
   `src/generated/stub.rs`; verify `--check`.
5. **feat(harness): runner, report and ratchet.** `main.rs` in place of
   `lib.rs`, `registry.rs`, `prelude.rs`, `record.rs`, `run.rs`,
   `ratchet.rs` with unit tests; run, read the report, `--bless
   --bless-floors`; commit `passing.txt`. Prove the failure path once by
   hand: make the stub return `t * 1.001`; `--check` must report 10 id
   regressions (every id but line 6, where 0 × 1.001 = 0) and a floor
   regression for `d3-ease/test/linear-test.js` (1 passing against 11);
   revert, and say so in the commit body.
6. **chore: gate on the ratchet.** `scripts/check.sh quick` gains `cargo
   run -q -p rustd3-harness -- --check` after the test loop; `full`
   gains `node dev/harness/translate/translate.mjs --check`.

#### Exit criteria

1. `cargo test -p rustd3-testkit` passes, including every spot check
   below.
2. `node dev/harness/translate/translate.mjs --check` exits 0.
3. `cargo run -p rustd3-harness` reports the stub suite as 13
   assertions, 11 translated, 2 skipped (`js:`), 11 passing, 0 failing,
   0 not reached; the file line reads `translated 11 / passing 11 /
   skipped 2`.
4. `dev/harness/passing.txt` holds the comment line, `#floor
   d3-ease/test/linear-test.js 11`, then exactly the 11 ids
   `d3-ease/test/linear-test.js:6:3` … `d3-ease/test/linear-test.js:16:3`.
5. `cargo run -p rustd3-harness -- --check` exits 0; `cargo test -p
   rustd3-harness` runs the ratchet unit tests, in which a listed id
   that fails, becomes a skip, disappears or is not reached, and a file
   whose floor exceeds its passing count, each make `check` report a
   regression.
6. `diff -r` of `dev/harness/d3/<m>/src` against
   `.claude/map/rustd3/proto/d3-src/<m>/src` (and of `test/` against
   `proto/d3-tests/<m>/test`) is empty for every module.
7. `node --import ./dev/harness/translate/d3-resolve.mjs --input-type=module
   -e "import {scaleLinear} from 'd3-scale';
   console.log(scaleLinear().domain([0,100]).range([0,960])(25))"`
   prints 240.
8. `scripts/check.sh quick` runs the ratchet and `full` runs the
   translator `--check`; both exit 0.

#### Spot checks

| Input | Expected | Rule |
| --- | --- | --- |
| `"M1e-7,0"` vs `"M0.0000001,0"`, `approx_eq` | true | numbers compared as values |
| `"1e+21"` vs `"1000000000000000000000"` | true | same |
| `"0.13"` vs `"0.12"` / `"0.14"` vs `"0.12"` | true / false | one unit in the last place |
| `"2010-01-01"` vs `"2010-01-02"` | false | integers get no slack |
| `"-0"` vs `"0"` approx / `Num(-0.0)` vs `Num(0.0)` strict | true / false | `Object.is` only in `strict_eq` |
| `"\u{2212}1,234.50"` vs `"-1,234.50"` | true | U+2212 reads as `-` |
| `Num(0.1 + 0.2)` vs `Num(0.3)` / `NaN` vs `NaN` / `inf` vs `-inf` | true / true / false | rel 1e-9 |
| `"inf"` vs `"Infinity"` | false | text differs |
| JSON `{"a": 1}` → `Obs` | `Err(obs::Error::Object("{\"a\":1}"))` | objects rejected |
| `Ok::<f64, String>(2.0).to_obs()` / `Err::<f64, &str>("bad").to_obs()` | `Num(2.0)` / `Str("error: bad")` | `ToObs for Result` |
| two equal 64 × 64 images | max 0, over 0, tile 0 | |
| 64 × 64, one channel at pixel (5, 5) differs by 255, threshold 0 | max 255, over 1, share 1/16,384, tile 1/4,096 ≈ 0.000244, worst (0, 0) | 255 / (32·32·4) / 255 |
| a whole 32 × 32 tile off by 5 in every channel | tile 5/255 ≈ 0.0196 (≤ 0.02); off by 6 → 0.0235 | mean / 255 |
| 33 × 33 image, pixel (32, 32) off by 255 in all 4 channels | tile 1.0 (a 1 × 1 edge tile), worst (32, 32) | edge tiles averaged over their own channels |
| `Image::new(2, 2, &[0; 15])` | `Err(Length { expected: 16, actual: 15, .. })` | 2·2·4 |
| `zone("America/Los_Angeles")` at 2026-03-08T09:59:59Z / 10:00:00Z | −08 / −07 | jiff 0.2.38 bundled tzdb |
| `zone("Asia/Kathmandu")` at 2026-01-01T00:00:00Z | +05:45 | |
| `zone("Europe/Berlin")` at 2026-10-25T00:59:59Z / 01:00:00Z | +02 / +01 | |
| `zone("Mars/Olympus")` | panics with jiff's error | |
| `Store::check` on a missing reference, `RUSTD3_BLESS` unset | panics naming the path; writes `name.new` | |
| same with `RUSTD3_BLESS=1` | writes the reference, passes | |
| resolve hook, `scaleLinear().domain([0,100]).range([0,960])(25)` | 240 | d3 in Node |
| stub suite | 13 assertions, 11 pass, 2 `js:` skips; floor 11 | |

#### Not in this phase

- No core code (the stub goes in Phase 3); no translation of path, ease,
  ticks, format or any other module; no d3 test helpers beyond `check`,
  `skip`, `throws` and `Kind`.
- No list dumps or `figures_equal` (Phases 12, 15); no PNG decoding in
  testkit (references are text only).
- The vendored d3 files and acorn are never edited.

### Phase 3 — First port: path, ticks, format, ease

*M0 Foundation · builds on phase 2 · about 1,600 lines of Rust (port about 950, tests about 450, harness about 200) and 300 of translator rules, 5 commits*

#### Goal

`rustd3-core` has its first four modules in their final shape: `path`
(`PathContext`, `SvgPath`, `svg`), `ticks` (d3-array's `ticks`,
`tick_increment`, `tick_step` and `nice`, with bail-outs on degenerate
input), `format` (d3-format with the neutral SI default and `EN_US`) and
`ease` (d3-ease as one enum), plus the crate-private `js` helpers. The
harness grades them with d3's own tests through the real API instead of
the stub: at least the prototype's 1,300 assertions pass, every
translated file has a floor, and the ratchet holds them.

#### Builds on

- Phase 2: `rustd3-testkit` (`obs::{Obs, ToObs, Tolerance}`); the
  harness (`registry`, `prelude::{check, skip, throws, Kind}`, `run`,
  `ratchet` with floors, the stub suite); the translator split into
  `translate.mjs`, `expr.mjs` and `rules/*.mjs` with location ids and
  prefixed skip reasons; `passing.txt` with the stub's floor and 11 ids.

#### Design

##### Modules

```text
core/src/lib.rs               pub mod ease; pub mod format; pub mod path; pub mod ticks; mod js;
core/src/js.rs                crate-private: round (JavaScript Math.round), to_int
core/src/path.rs              PathContext, svg; re-exports SvgPath, Error
core/src/path/svg.rs          SvgPath
core/src/path/error.rs        path::Error
core/src/ticks.rs             ticks, tick_increment, tick_step, nice
core/src/format.rs            Format, format, format_prefix; re-exports below
core/src/format/locale.rs     Locale, Definition
core/src/format/specifier.rs  Specifier, Align, Sign, Symbol, Kind, InvalidFormat
core/src/format/precision.rs  precision_fixed, precision_prefix, precision_round
core/src/format/decimal.rs    private digit helpers
core/src/ease.rs              Ease, Mode
```

Submodules are private and re-exported by their parent (Phase 2's
convention), so paths are `path::SvgPath`, `format::Locale`,
`format::Specifier`. Algorithms are lifted from `proto/poc/core/src/`:
`path.rs` (arc, arcTo, rect), `ticks.rs` (`tick_spec` and the four
functions), `format.rs` (parsing, grouping, `decimal_parts`,
`to_precision`, `format_rounded`, `format_prefix_auto`, `format_trim`,
the precision functions) and `ease.rs` (formulas); the shapes change as
below. d3 sources: `d3-path/src/path.js`, `d3-array/src/{ticks,nice}.js`,
`d3-format/src/*.js`, `d3-ease/src/*.js` under `dev/harness/d3/`.

##### `js`

```rust
/// JavaScript's Math.round: ties toward +∞ (f64::round differs at −2.5).
pub(crate) fn round(x: f64) -> f64;   // r = x.floor(); if x - r >= 0.5 { r + 1.0 } else { r }
/// `round(x)` as an integer: Some iff it is finite and |round(x)| ≤ 2⁵³.
pub(crate) fn to_int(x: f64) -> Option<i64>;
```

**No wrapping casts.** In `core/src`, `f64 → integer` goes through
`to_int` (format's `b o x X d c` kinds, the index arithmetic in `ticks`)
or through a `usize` produced after the cap check in `ticks` (`i2 − i1
+ 1` is compared as `f64` against the cap before any `as usize`);
`integer → f64` is `f64::from` or `as f64`. A wrapping `as u128` on a
non-finite or huge value is the NaN-to-integer bug class the prototype
had. Before committing, `grep -nE ' as (u|i)(size|8|16|32|64|128)'
core/src` and justify every hit in the commit body. `path/svg.rs`,
`format` and later phases call `crate::js::round`.

##### `path`

```rust
/// d3's canvas path interface: the one drawing seam (Part I, Shapes and PathContext).
pub trait PathContext {
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64);
    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64);
    /// Radians, clockwise from +x in screen coordinates; `r` must be ≥ 0.
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool);
    fn close_path(&mut self);
}

/// SVG path data, as `d3.path()` prints it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SvgPath { data: String, start: Option<(f64, f64)>, current: Option<(f64, f64)>, scale: Option<f64> }

impl SvgPath {
    pub fn new() -> Self;                        // d3.path()
    /// d3.pathRound(digits): numbers rounded to `digits` places; above 15, unrounded.
    pub fn with_digits(digits: u32) -> Self;
    /// `arc` that reports a negative radius (d3 throws) instead of drawing nothing.
    pub fn try_arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) -> Result<(), path::Error>;
    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Result<(), path::Error>;
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64);
}
impl PathContext for SvgPath { /* arc: try_arc; a negative radius draws nothing */ }
impl fmt::Display for SvgPath { /* the path data */ }

/// Draws with `draw` into a fresh `SvgPath` and returns its data.
pub fn svg(draw: impl FnOnce(&mut SvgPath)) -> String;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("negative radius: {0}")]
    NegativeRadius(f64),
}
```

**Negative radii.** `PathContext::arc` returns `()`: the trait is the
one drawing seam and `IcedPath` (Phase 11) implements it too. A negative
radius is a contract violation the canvas API also rejects; `SvgPath`'s
trait method draws nothing for it, and `try_arc` exposes d3's error to
callers and to the harness. Phase 29 adds the provided `rect` to the
trait.

**Numbers** print with Rust's shortest round-trip `Display`, which
equals JavaScript's except for exponent notation (Rust writes
`0.0000001` and `1000000000000000000000` where JavaScript writes `1e-7`
and `1e+21`: valid SVG, and the tolerant format accepts it; the
prototype counted 23 such assertions). `-0` prints as `0` (JavaScript's
`String(-0)`). Rounding under `with_digits` uses `js::round`. `arc` and
`arc_to` keep d3's epsilon 1e-6 and its τ − ε full-circle rule.

**`with_digits` takes `u32`.** d3's `pathRound(NaN)` and
`pathRound(-1)` cannot be written, so their two assertions are `types:`
skips; `pathRound()` maps to `with_digits(3)` and `pathRound(null)` to
`with_digits(0)`.

##### `ticks`

```rust
/// d3.ticks: about `count` nicely rounded values in [start, stop], in the
/// order of start → stop. `count` is f64 as in d3 (NaN, 0.5 and negative
/// counts are meaningful there and tested).
#[doc(alias = "ticks")]
pub fn ticks(start: f64, stop: f64, count: f64) -> Vec<f64>;
#[doc(alias = "tickIncrement")]
pub fn tick_increment(start: f64, stop: f64, count: f64) -> f64;
#[doc(alias = "tickStep")]
pub fn tick_step(start: f64, stop: f64, count: f64) -> f64;
/// d3.nice: [start, stop] extended to tick boundaries.
pub fn nice(start: f64, stop: f64, count: f64) -> [f64; 2];
```

These are d3-array's names, which Phases 6, 25 and 27 call
(`ticks::tick_step`). The algorithm is `tick_spec`: thresholds e10 =
√50, e5 = √10, e2 = √2 on the error `step / 10^power`; a negative power
gives an integer inverse increment; the count is doubled once for 0.5 ≤
count < 2 when no tick fits; NaN counts propagate (no `f64::max`).
Bail-outs, each returning `vec![]` before allocating:

- `!(count > 0)` → empty (d3); `start == stop` → `[start]` (d3).
- The increment or its inverse is not finite: `ticks(-1e308, 1e308,
  10)` makes d3 return `[NaN]`, and subnormal spans such as
  `ticks(1e-320, 1e-319, 10)` make d3 throw a `RangeError`.
- `i1` or `i2` is not finite, or `i2 − i1 + 1` exceeds the cap
  `min(10_000, ceil(10 · max(count, 1)))`: d3 allocates `ticks(0, 1,
  1e6)` as 1,000,001 values. The cap is ten times the requested count,
  so every honest request fits, with an absolute ceiling for huge
  counts.
- `nice` stops after 10 iterations, returning the current [start,
  stop] (d3-array loops until the step repeats; d3-scale's linear nice
  already caps at 10).

No `as usize` cast happens before the cap check.

##### `format`

```rust
/// A parsed `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.
#[derive(Debug, Clone, PartialEq, Default)]   // Default = parse("")
pub struct Specifier { fill: char, align: Align, sign: Sign, symbol: Option<Symbol>, zero: bool,
                       width: Option<usize>, comma: bool, precision: Option<usize>, trim: bool,
                       kind: Option<Kind> }
impl FromStr for Specifier { type Err = InvalidFormat; }
impl fmt::Display for Specifier { /* d3's FormatSpecifier.toString */ }
impl Specifier {
    pub fn fill(&self) -> char;  pub fn align(&self) -> Align;  pub fn sign(&self) -> Sign;
    pub fn symbol(&self) -> Option<Symbol>;  pub fn zero(&self) -> bool;
    pub fn width(&self) -> Option<usize>;  pub fn comma(&self) -> bool;
    pub fn precision(&self) -> Option<usize>;  pub fn trim(&self) -> bool;
    pub fn kind(&self) -> Option<Kind>;
    /// d3-scale's tickFormat sets these on a parsed specifier.
    pub fn with_precision(self, precision: usize) -> Self;
    pub fn with_trim(self, trim: bool) -> Self;
}
pub enum Align { Left, Right, Center, AfterSign }           // < > ^ =
pub enum Sign { Minus, Plus, Parens, Space }                 // - + ( space
pub enum Symbol { Currency, Base }                           // $ #
pub enum Kind { Exponent, Fixed, General, Rounded, Si, Percent, PercentRounded,
                Binary, Octal, Decimal, Hex, HexUpper, Char, Number, Other(char) }
// Each enum: Debug, Clone, Copy, PartialEq, Eq, and Display printing d3's character.

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid format: {0}")]
pub struct InvalidFormat(pub String);     // kept name; InvalidFormat("10..f")
```

d3's type regex is case-insensitive; letters other than d3's types
parse as `Kind::Other(c)` and format as `~g` with precision 12, as d3's
`formatTypes` fallback does.

```rust
/// A number formatter: a specifier bound to a locale.
#[derive(Debug, Clone, PartialEq)]
pub struct Format { /* locale, specifier, normalized kind, precision, prefix, suffix,
                       optional fixed SI prefix (scale, symbol) */ }
impl Format {
    pub fn new(locale: &Locale, specifier: Specifier) -> Format;
    /// d3's formatPrefix: fixed-point, scaled to `value`'s SI prefix.
    pub fn prefixed(locale: &Locale, specifier: Specifier, value: f64) -> Format;
    pub fn format(&self, value: f64) -> String;
}
impl fmt::Display for Format { /* its specifier, d3's format.toString() */ }

/// Formats under the neutral default locale.
pub fn format(specifier: &str) -> Result<Format, InvalidFormat>;
pub fn format_prefix(specifier: &str, value: f64) -> Result<Format, InvalidFormat>;

pub fn precision_fixed(step: f64) -> Option<usize>;
pub fn precision_prefix(step: f64, value: f64) -> Option<usize>;
pub fn precision_round(step: f64, max: f64) -> Option<usize>;
```

One `Format` type serves both d3's format and formatPrefix, because
d3-scale's tickFormat returns either. `precision_*` return `None` where
d3 returns NaN (zero, NaN, ±∞ steps). The kind is normalized in `new`
into a private enum, so `format` matches exhaustively (no
`unreachable!`). Integer kinds (`b`, `o`, `x`, `X`, `d`) round with
`js::to_int` on finite values only; non-finite values print the
locale's `nan` or `Infinity` before any integer conversion.

```rust
/// d3's formatLocale: separators, grouping, currency, numerals.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale { decimal: Cow<'static, str>, thousands: Cow<'static, str>,
    grouping: Cow<'static, [usize]>, currency: [Cow<'static, str>; 2],
    numerals: Option<[Cow<'static, str>; 10]>, percent: Cow<'static, str>,
    minus: Cow<'static, str>, nan: Cow<'static, str>, group_from: usize }

/// What a locale table states; `Default` is d3's fallbacks (locale.js 12–21):
/// ".", no grouping (empty), ["", ""], no numerals, "%", "\u{2212}", "NaN", group_from 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition { pub decimal: Cow<'static, str>, pub thousands: Cow<'static, str>,
    pub grouping: Cow<'static, [usize]>, pub currency: [Cow<'static, str>; 2],
    pub numerals: Option<[Cow<'static, str>; 10]>, pub percent: Cow<'static, str>,
    pub minus: Cow<'static, str>, pub nan: Cow<'static, str>, pub group_from: usize }

impl Locale {
    /// SI style (CGPM 2003, Res. 10): ".", U+202F groups of three from five
    /// integer digits, no currency, U+2212. `Default` returns it.
    pub const NEUTRAL: Locale;
    /// d3's default: ",", groups of three, ["$", ""].
    pub const EN_US: Locale;
    pub fn new(definition: Definition) -> Locale;
    pub fn format(&self, specifier: &str) -> Result<Format, InvalidFormat>;
    pub fn format_prefix(&self, specifier: &str, value: f64) -> Result<Format, InvalidFormat>;
    /// Group only values with at least `digits` integer digits (1: always).
    pub(crate) fn group_from(self, digits: usize) -> Locale;
}
```

`Locale::new` cannot be `const` (dropping a `Definition` with `Cow`
fields in a const fn is E0493; checked). Constants are built by a
`pub(crate) const fn from_table(Table) -> Locale` over a `Copy` struct
of `&'static` fields (`decimal`, `thousands`, `grouping: &'static
[usize]`, `currency: [&'static str; 2]`, `numerals: Option<&'static
[&'static str; 10]>`, `percent`, `minus`, `nan`, `group_from`); checked
to compile, numerals included. Phase 26's generated locale tables use
`from_table`. Users set `group_from` through `Definition`.

**The grouping rule.** With `,`, a value is grouped when its integer
part (as printed, after rounding and SI scaling, without sign or
currency) has at least `group_from` digits; otherwise it formats as
without `,`, including the zero-fill path. d3's tables and `EN_US` use
1 (d3's behavior); `NEUTRAL` uses 5, so a four-digit value is not
grouped (`−1234.50`) and a five-digit one is (`−12 345.68`), as SI
style prescribes. Fractions are never grouped (d3 never groups them
either). Phase 6 adds the per-axis rule on top (`Format::format_all`);
single values follow this rule.

##### `ease`

```rust
/// d3-ease's curves as one closed set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    Linear,
    Quad(Mode), Cubic(Mode), Sin(Mode), Exp(Mode), Circle(Mode), Bounce(Mode),
    Poly { mode: Mode, exponent: f64 },                 // d3 default 3
    Back { mode: Mode, overshoot: f64 },                // 1.70158
    Elastic { mode: Mode, amplitude: f64, period: f64 },// 1, 0.3; amplitude < 1 acts as 1
}
pub enum Mode { In, Out, InOut }       // Debug, Clone, Copy, PartialEq, Eq
impl Ease {
    pub fn apply(&self, t: f64) -> f64;
    pub const fn poly(mode: Mode) -> Ease;      // d3's default parameters
    pub const fn back(mode: Mode) -> Ease;
    pub const fn elastic(mode: Mode) -> Ease;
}
impl Default for Ease { /* Cubic(InOut), d3's transition default */ }
```

Formulas are d3's (`tpmt`, the bounce constants b1…b0, the elastic `s =
asin(1/a)·p/τ`); the amplitude clamp lives in `apply` because variant
fields are public. d3's aliases map as d3 does: `easeQuad` =
`Quad(InOut)`, `easeBounce` = `Bounce(Out)`, `easeElastic` = `Elastic {
Out, .. }`, `easeBack` = `Back { InOut, .. }`.

##### Harness

- Delete `src/stub.rs`, the stub mapping in `rules/ease.mjs` and the
  `stub` suite. Suites (file lists from the `files` table of
  `proto/tr/translate.mjs`, lines 313–317): `path` (d3-path
  `path-test.js`, `pathRound-test.js`), `ease` (the ten d3-ease files),
  `ticks` (d3-array `ticks`, `tickIncrement`, `tickStep`, `nice`),
  `format` (the 24 d3-format files).
- `rules/path.mjs`: `path()` → `SvgPath::new()`, `pathRound(d)` as
  above; methods per the prototype's `PATH_METHODS` table; a statement
  `p.arc(…)` → `p.try_arc(…).expect("d3 test arc")` and `p.arcTo(…)` →
  `p.arc_to(…).expect("d3 test arcTo")` (test code); `assert.throws(()
  => p.arc(…))` → `throws(id, || p.try_arc(…))`, likewise `arcTo` →
  `throws(id, || p.arc_to(…))`. The prelude imports
  `rustd3_core::path::{self, PathContext, SvgPath}`.
- `rules/ease.mjs`: identifiers → `Ease` values (struct-literal variants
  for `.exponent()`, `.overshoot()`, `.amplitude()`, `.period()`
  chains); calls → `.apply(t)`; `strictEqual(easeA, easeB)` on two ease
  values → `check(id, Kind::Strict, &(a == b), &true)`.
- `rules/array.mjs`: `ticks::ticks`, `ticks::tick_increment`,
  `ticks::tick_step`, `ticks::nice(…).to_vec()`.
- `rules/format.mjs`: `format(s)` → `format::Locale::EN_US.format(s)`;
  `formatPrefix` likewise; `formatSpecifier(s)` →
  `s.parse::<format::Specifier>()`; field reads → getters
  (`.fill().to_string()`, `.align().to_string()`, `.symbol().map(|s|
  s.to_string()).unwrap_or_default()`, `.kind()…`); field writes →
  `types:` skips; `new FormatSpecifier({})` → `Specifier::default()`;
  `formatLocale({…})` → `format::Locale::new(format::Definition { …,
  ..Default::default() })` (grouping only when `thousands` is given too,
  as d3); `precision*` → `Option<usize>` values, with `isNaN(x)` →
  `x.is_none()`; `locale(...)` JSON loads → `gap: phase 26`.
- Prelude helpers from `proto/poc/harness/src/prelude.rs`:
  `precision(&str, f64)`, `out(Ease)`, `in_out(Ease)` → closures over
  `Ease::apply`.

d3's tests run under `EN_US` (Part I, Testing); the neutral default is
graded by the spot checks. Translator budget for this phase: 300 lines
of JavaScript.

**Skip table** (files allowed under 70 %):

| File | Reason |
| --- | --- |
| `d3-format/test/locale-test.js` | JSON locale loads, `gap: phase 26` |
| `d3-format/test/arabicLocale-test.js` | JSON locale loads, `gap: phase 26` |
| `d3-format/test/formatSpecifier-test.js` | only if field writes (`types:`) dominate |

##### Decisions this phase may meet

- A translated file not in the skip table passes under 70 % — record it
  with the file's counts and the reason, and stop; do not bless it
  silently.

#### Steps

1. **feat(core): js helpers and path.** `core/src/js.rs`;
   `core/src/{lib,path}.rs`, `path/{svg,error}.rs`; `[[test]] name =
   "main"`, `path = "tests/main/main.rs"` in `core/Cargo.toml`;
   `tests/main/main.rs` (module declarations) and `tests/main/path.rs`.
2. **feat(core): ticks with bail-outs.** `core/src/ticks.rs`;
   `tests/main/ticks.rs`.
3. **feat(core): format.** `core/src/format.rs`,
   `format/{locale,specifier,precision,decimal}.rs`;
   `tests/main/format.rs` (neutral, `EN_US`, and de-DE and fr-FR through
   `Locale::new`).
4. **feat(core): ease.** `core/src/ease.rs`; `tests/main/ease.rs`.
5. **feat(harness): grade path, ease, ticks and format.**
   `rules/{path,ease,array,format}.mjs`, `translate.mjs` (suites),
   `expr.mjs`, `prelude.rs`; delete `src/stub.rs` and
   `src/generated/stub.rs`; regenerate `src/generated/{path,ease,ticks,
   format}.rs`; run, read every failure, change core (not the rules) unless
   the difference is a design choice stated here; `--bless
   --bless-floors` after reading the diff; per-suite and per-file counts
   in the commit body; `scripts/check.sh full`.

#### Exit criteria

1. `cargo test -p rustd3-core` passes, every spot check a named test in
   the `main` test binary (`core/tests/main/{path,ticks,format,ease}.rs`).
2. `cargo run -p rustd3-harness` shows 0 failing and 0 not reached in
   suites `path`, `ease`, `ticks` and `format`, and **at least 1,300
   passing**. Reference from `proto/poc` (path 75, ease 359, ticks 241,
   format 625 translated, all passing): the port differs only by
   `pathRound(NaN)` and `pathRound(-1)` (two `types:` skips) and by `new
   FormatSpecifier({})`, which translates (+10).
3. `cargo run -p rustd3-harness -- --check` passes; `passing.txt` lists
   the 11 `d3-ease/test/linear-test.js:6:3` … `:16:3` ids, graded
   against `Ease::Linear`; `src/stub.rs` and the `stub` suite are gone.
4. Every translated file has a `#floor` line, and every file passes at
   least 70 % of its assertions except those in the skip table; skips
   carry only `js:`, `types:` and `gap: phase 26`.
5. `node dev/harness/translate/translate.mjs --check` exits 0.
6. `cargo clippy -p rustd3-core --all-targets -- -D warnings` is clean;
   library code has no `unwrap`, no `unreachable!` and no cast that can
   wrap (the `grep` above, every hit justified in a commit body).

#### Spot checks

d3 values from d3 7.9's modules run through the resolve hook (vendored
versions: d3-path 3.1.0, d3-array 3.2.4, d3-format 3.1.2, d3-ease
3.0.1); neutral values follow the grouping rule.

| Input | Expected | Source |
| --- | --- | --- |
| `move_to(150, 50); line_to(200, 100)` | `M150,50L200,100` | d3 |
| `move_to(0, 0); arc(0, 0, 50, 0, π/2, false)` | `M0,0L50,0A50,50,0,0,1,3.061616997868383e-15,50` (Rust prints the last x as `0.000000000000003061616997868383`; tolerant-equal) | d3 |
| `arc(100, 100, 50, 0, 2π, false)` on an empty path | `M150,100A50,50,0,1,1,50,100A50,50,0,1,1,150,100` | d3 |
| `move_to(270, 182); arc_to(270, 39, 363, 39, 53)` | `M270,182L270,92A53,53,0,0,1,323,39` | d3 |
| `rect(200, 100, 70, 40)` | `M200,100h70v40h-70Z` | d3 |
| `with_digits(1)`, `move_to(123.456, 789.012)` | `M123.5,789` | d3 `pathRound(1)` |
| `with_digits(3)`, `move_to(π, e)` | `M3.142,2.718` | d3 `pathRound()` |
| `try_arc(0, 0, -1, 0, 1, false)` | `Err(NegativeRadius(-1.0))`; `arc` leaves the path empty | negative radii |
| `ticks(0, 1, 10)` | `[0, 0.1, 0.2, …, 0.9, 1]` (11) | d3 |
| `ticks(-10, 10, 10)` / `ticks(1, 0, 5)` | `[-10, -8, …, 10]` / `[1, 0.8, 0.6, 0.4, 0.2, 0]` | d3 |
| `ticks(0, 1, NaN)`, `ticks(NaN, 1, 10)`, `ticks(0, ∞, 10)`, `ticks(0, 1, 0)` | `[]` | d3 too |
| `ticks(1, 1, 10)` / `ticks(0, 1, 0.5)` / `ticks(0, 10, 1.5)` | `[1]` / `[0]` / `[0, 5, 10]` | d3 |
| `ticks(1e9 + 1, 1e9 + 2, 10)` | `[1000000001, 1000000001.1, …, 1000000002]` (11) | d3 |
| `ticks(0, f64::MAX, 10)` | `[0, 2e307, …, 1.6e308]` (9) | d3 too |
| `ticks(-1e308, 1e308, 10)` | `[]` | bail-out; d3: `[NaN]` |
| `ticks(1e-320, 1e-319, 10)` | `[]` | bail-out; d3 throws `RangeError` |
| `ticks(0, 1, 1e6)` / `ticks(0, 1, 1000)` | `[]` / 1,001 values | cap 10,000; d3: 1,000,001 values / 1,001 |
| `tick_increment(0, 1, 10)` / `tick_increment(0, 10, 10)` | `-10` / `1` | d3 |
| `tick_step(0, 1, 10)` / `tick_step(1, 0, 10)` | `0.1` / `-0.1` | d3 |
| `nice(0.13, 9.71, 5)` / `nice(1.1, 10.9, 10)` / `nice(132, 876, 5)` | `[0, 10]` / `[1, 11]` / `[0, 1000]` | d3 |
| `EN_US` `,.2f` / `$,.2f` of −1234.5 | `−1,234.50` / `−$1,234.50` | d3; Part I, Formats, time and color |
| `EN_US` `,.1f` 1234567.5 / `,d` 10000 / `~s` 1500 / `.2e` 12345 | `1,234,567.5` / `10,000` / `1.5k` / `1.23e+4` | d3 |
| `EN_US` `08.2f` −3.14159 / `+.0f` 0 / `(.2f` −1.5 / `.1f` −0.04 | `−0003.14` / `+0` / `(1.50)` / `0.0` | d3 |
| `EN_US` `` (empty) 0.1 + 0.2 / `#x` 255 / `b` 5 | `0.3` / `0xff` / `101` | d3 |
| `format(".3s")` 0.000042 / `format(".1%")` 0.123 (neutral) | `42.0µ` / `12.3%` | Part I, Formats, time and color |
| `format("10..f")` | `Err(InvalidFormat("10..f"))` | Part I, Formats, time and color |
| neutral `,.1f` 1234567.5 / `,.2f` −12345.678 | `1 234 567.5` / `−12 345.68` (U+202F) | grouping rule |
| neutral `,d` 8000 / 12000 | `8000` / `12 000` | four vs five digits |
| neutral `$,.2f` −1234.5 / 1234.5 | `−1234.50` / `1234.50` | four digits: not grouped |
| `Locale::new` de-DE (`,` `.` [3] ["", "\u{a0}€"]) `,.1f` 1234567.5 / `$,.2f` 1234.5 | `1.234.567,5` / `1.234,50 €` | d3 `formatLocale` |
| fr-FR (`,` U+00A0 [3] ["", "\u{a0}€"]) `,.1f` 1234567.5 / `,d` 12000 | `1 234 567,5` / `12 000` (U+00A0) | d3 `formatLocale` |
| `EN_US` `format_prefix(",.0", 1e3)` 1e3 / `format_prefix(".1", 1e6)` 1234567 | `1k` / `1.2M` | d3 |
| `precision_fixed(0.01)` / `(1.0)` / `(0.0)` | `Some(2)` / `Some(0)` / `None` | d3: 2 / 0 / NaN |
| `".2s".parse::<Specifier>()?.to_string()` / `Specifier::default().to_string()` | ` >-.2s` / ` >-` | d3 |
| `Ease::default().apply(0.25)` (cubic in-out) / `Cubic(In)` 0.25 | 0.0625 / 0.015625 | d3 |
| `Elastic(Out)` 0.3 / `Bounce(Out)` 0.5 / `Back(In)` 0.5 | 0.8758553274682307 / 0.765625 / −0.0876975 | d3 |
| `Poly { In, exponent: 2 }` 0.5 / `Sin(InOut)` 0.25 / `Exp(Out)` 0.5 / `Circle(In)` 0.5 | 0.25 / 0.1464466094067262 / 0.9696969696969697 / 0.1339745962155614 | d3 |
| harness after this phase | 0 failing, ≥ 1,300 passing; skips only `js:`, `types:`, `gap: phase 26` | |

#### Not in this phase

- No scales or tick formats (Phase 4); no `locale::Locale` or
  `Format::format_all` (Phase 6).
- No `fmt!` or other number-locale tables (Phase 26); no time formats
  (Phase 28); no shapes (Phase 8) or provided `rect` (Phase 29); no
  `IcedPath` (Phase 11).
- No emulation of JavaScript's exponent notation in printed numbers.

## M1 — Core contract

The scales, ticks, locales, axes, curves, recordings and pointer operations that the figure layer and the widget build on.

### Phase 4 — Continuous scales I

*M1 Core contract · builds on phase 3 · about 1,300 lines of Rust and 300 of translator rules, 5 commits*

#### Goal

Core has continuous scales as Part I's Scales section designs them: a
public `scale::Transform` trait; `scale::Continuous<T, R, const N:
usize = 2>` with d3's builders (`domain`, type-changing `range`,
`range_round`, `clamp`, `interpolate`, `unknown`, `nice`) and readers
(`apply`, `invert` for `f64` ranges, `ticks`, `minor`, `tick_format`);
the transforms `scale::transform::{Identity, Pow, Log}` with their own
tick rules, nice and tick formats; `scale::TickFormat` with
`format_all`; and `interpolate::Interpolate` for numbers. The harness
grades them with d3-scale's linear, pow, sqrt, log and tickFormat tests
and d3-interpolate's number and round tests, translated through chained
setters and `.copy()`.

#### Builds on

- Phase 3: `ticks::{ticks, tick_increment, tick_step, nice}` with the
  cap `min(10_000, ceil(10 · max(count, 1)))`; `format::{Locale,
  Format, Specifier, InvalidFormat, precision_fixed, precision_prefix,
  precision_round}` (`Format::new`, `Format::prefixed`,
  `Specifier::with_precision`, `with_trim`, `Locale::NEUTRAL`,
  `EN_US`); `crate::js::{round, to_int}`.
- Phase 2/3: the harness (`throws`, floors, `--bless-floors`) and
  `rules/*.mjs`.

#### Design

##### Modules

```text
core/src/interpolate.rs                         Interpolate, number, round
core/src/scale.rs                               continuous(), linear(), pow(), sqrt(), log(); aliases
core/src/scale/continuous.rs                    Continuous
core/src/scale/tick_format.rs                   TickFormat, tick_format (d3-scale's tickFormat.js)
core/src/scale/transform.rs                     Transform (re-exported as scale::Transform), pub mods below
core/src/scale/transform/{identity,pow,log}.rs  transform::{Identity, Pow, Log}
core/src/ticks/rules.rs                         ticks::{linear, log, subdivide} (private module, re-exported),
                                                crate-private pow10
core/src/format.rs                              + Format::locale
```

`scale::transform` is a public module (paths such as
`scale::transform::Log`, which Phase 23's sequential variants use); the
trait's canonical path is `scale::Transform`. `scale::transform::Identity`
is the transform; `scale::Identity`, d3's `scaleIdentity`, is a
different type that Phase 6 builds. Call sites always write the module
path; neither is imported by its bare name at file scope.

Sources: `proto/fscale/src/bin/ext.rs` (`Transform` with required
`ticks` and default `minor`, `ticks::{linear, log, subdivide}`; its
`ticks::log` is simplified, so port d3's); `proto/rfc/d3/src/scale.rs`
(type-changing `range`, `clamp`, `nice`, `tick_format`, `invert` only
for `f64`) and `interpolate.rs`; d3's
`dev/harness/d3/d3-scale/src/{continuous,linear,tickFormat,pow,log,nice}.js`
and `d3-interpolate/src/{number,round}.js`.

##### `interpolate`

```rust
/// A value that blends with another of its type (d3's interpolate, typed).
pub trait Interpolate: Clone {
    /// The value a fraction `t` of the way from `self` to `to`.
    fn lerp(&self, to: &Self, t: f64) -> Self;
}
impl Interpolate for f64 { /* self * (1.0 - t) + to * t: exact at t = 0 and 1 (d3) */ }
impl Interpolate for f32 { /* through f64 */ }

/// d3.interpolateNumber.
pub fn number(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync;
/// d3.interpolateRound: js::round (ties toward +∞) of the blend.
pub fn round(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync;
```

The closure-returning shape matches Phase 23's color interpolators.

##### `Transform`

```rust
pub trait Transform: Clone {
    fn forward(&self, x: f64) -> f64;
    fn inverse(&self, y: f64) -> f64;
    /// Required: every transform states its tick rule (usually a `ticks::` block).
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64>;
    /// Minor ticks between `major`; log-like transforms supply their own.
    fn minor(&self, major: &[f64]) -> Vec<f64> { ticks::subdivide(self, major, 5) }
    /// This transform as it applies to `domain` (the end stops). Log
    /// reflects for negative domains; the default returns a clone.
    fn for_domain(&self, domain: [f64; 2]) -> Self { self.clone() }
    /// d3's nice for this transform; default: d3-scale's linear nice.
    fn nice(&self, domain: [f64; 2], count: usize) -> [f64; 2] { scale::nice_linear(domain, count) }
    /// d3's tickFormat for this transform; default: d3-scale's tickFormat.js.
    fn tick_format(&self, locale: &format::Locale, domain: [f64; 2], count: usize,
                   specifier: Option<&str>) -> Result<scale::TickFormat, format::InvalidFormat> {
        scale::tick_format(locale, domain, count, specifier).map(scale::TickFormat::from)
    }
}
```

Log cannot be built on `forward`, `inverse`, `ticks` and `minor` alone:
d3 picks ln or −ln(−x) from the domain's sign (`log.js` `rescale`),
nices to powers of the base and blanks tick labels, and inherent
methods on `Continuous<Log>` would clash with the generic ones (E0592).
Three defaulted methods carry that, and every user transform in Part I
(the `Reciprocal` example derives `Clone`) still compiles with only the
four. `scale::nice_linear` is crate-private.

##### `Continuous`

```rust
pub struct Continuous<T, R = f64, const N: usize = 2> {
    transform: T,
    domain: [f64; N],
    forward: [f64; N],            // transform.forward of each stop, kept current by builders
    range: [R; N],
    interpolate: Interpolator<R>, // private: Lerp | Custom(Arc<dyn Fn(&R, &R, f64) -> R + Send + Sync>)
    clamp: bool,
    unknown: Option<R>,
}
// Clone; Debug by hand (the custom interpolator prints as "custom"). No PartialEq.

pub type Linear<R = f64, const N: usize = 2> = Continuous<transform::Identity, R, N>;
pub type Pow<R = f64, const N: usize = 2> = Continuous<transform::Pow, R, N>;
pub type Log<R = f64, const N: usize = 2> = Continuous<transform::Log, R, N>;

pub fn continuous<T: Transform>(transform: T) -> Continuous<T>; // domain [0, 1], range [0.0, 1.0]
pub fn linear() -> Linear;
pub fn pow() -> Pow;    // exponent 1
pub fn sqrt() -> Pow;   // exponent 0.5
pub fn log() -> Log;    // base 10, domain [1, 10]

impl<T: Transform, R: Interpolate, const N: usize> Continuous<T, R, N> {
    pub fn domain(self, domain: [f64; N]) -> Self;
    pub fn range<S: Interpolate>(self, range: [S; N]) -> Continuous<T, S, N>; // resets interpolate, unknown
    pub fn interpolate(self, f: impl Fn(&R, &R, f64) -> R + Send + Sync + 'static) -> Self;
    pub fn clamp(self, clamp: bool) -> Self;
    pub fn unknown(self, value: R) -> Self;
    pub fn nice(self, count: usize) -> Self;
    pub fn apply(&self, x: f64) -> R;
    pub fn ticks(&self, count: usize) -> Vec<f64>;
    pub fn minor(&self, count: usize) -> Vec<f64>;
    /// Under the neutral default locale (Part I, Scales).
    pub fn tick_format(&self, count: usize, specifier: Option<&str>) -> Result<TickFormat, InvalidFormat>;
    pub fn tick_format_in(&self, locale: &format::Locale, count: usize, specifier: Option<&str>)
        -> Result<TickFormat, InvalidFormat>;
    pub fn get_domain(&self) -> [f64; N];
    pub fn get_range(&self) -> &[R; N];
}
impl<T: Transform, const N: usize> Continuous<T, f64, N> {
    pub fn invert(&self, y: f64) -> f64;
    pub fn range_round(self, range: [f64; N]) -> Self;  // range + interpolate::round
}
impl<R: Interpolate, const N: usize> Continuous<transform::Pow, R, N> { pub fn exponent(self, e: f64) -> Self; }
impl<R: Interpolate, const N: usize> Continuous<transform::Log, R, N> { pub fn base(self, b: f64) -> Self; }
```

**The interpolator** is `Lerp` or a shared `Arc<dyn Fn(&R, &R, f64) -> R
+ Send + Sync>`; `.interpolate(f)` takes d3's `(a, b) → value at t` in
one call. A fn pointer would not do: Phase 23's gamma interpolators
capture state.

**Locales and getters.** `tick_format` uses `Locale::NEUTRAL`, as Part
I's example does; `tick_format_in` takes the locale explicitly (std's
`_in` naming). Getters are `get_domain`/`get_range` only, as Part I's
naming rule says: d3's `clamp()`, `base()`, `exponent()`, `unknown()`
and `interpolate()` readers are `api:` skips.

Crate-visible readers `pub(crate) fn transform(&self) -> &T` and
`pub(crate) fn forward(&self) -> &[f64; N]` serve later core phases
(axes in 6, `Rescale` and `Pad` in 10, sequential scales in 23). N is
generic from the start, but only N = 2 can be constructed until Phase
5's `scale::piecewise`; `domain` and `range` keep N. Ticks, nice and
formats act on the end stops `[d[0], d[N−1]]`, as in d3.

**`apply(x)`** (d3 `continuous.js`): if `x` is NaN and `unknown` is
set, return it; with `clamp`, clamp `x` to the domain's min/max (by
`max`/`min`, never `f64::clamp`, which panics on NaN bounds); `u =
forward(x)`; for N = 2 (bimap) with `forward[1] < forward[0]`, swap
both the stops and the range ends; for N > 2 (polymap) reverse both when
the last stop is below the first, then `i = partition_point` over
`forward[1..N-1]` of `≤ u`; `t = normalize(a, b, u)`, where normalize
is `(u − a)/(b − a)`, NaN when `b − a` is NaN, and 0.5 when it is 0;
return the interpolator of `range[i], range[i+1]` at `t`. Without
`unknown`, NaN flows through as d3's arithmetic does.

**`invert(y)`**: the same piecewise map from the range (as numbers,
`f64::lerp`) to `forward`, then `transform.inverse`, then the clamp.
**`nice(count)`**: `transform.nice([d0, dN−1], count)` written into the
end stops, then `for_domain` and `forward` refreshed. Every builder
that changes the domain, the exponent or the base refreshes `forward`.

**Linear nice** (`nice_linear`, d3-scale `linear.js`): swap if
descending; up to 10 iterations of `step = tick_increment(start, stop,
count)`; when `step == prestep`, write back and stop; `step > 0` →
floor/ceil by step, `step < 0` → by its inverse, otherwise stop without
writing (d3-scale writes back only on convergence; d3-array's `nice`
differs here).

##### Transforms

- **`transform::Identity`**: forward and inverse return x. Ticks
  `ticks::linear`. All defaults.
- **`transform::Pow { exponent }`** (d3 `pow.js`): exponent 1 →
  identity; 0.5 → sqrt with sign (`x < 0 ? −√−x : √x`), inverse square
  with sign; otherwise `x < 0 ? −(−x)^e : x^e` and the inverse with
  1/e. Ticks linear in data space (`ticks::linear`); nice and format
  default.
- **`transform::Log { base, negative }`**: forward `ln x`, or `−ln(−x)`
  when `negative` (set by `for_domain`: `domain[0] < 0`); inverse `exp`
  or `−exp(−y)`. The base affects only ticks, nice, format and minor.
  Ticks `ticks::log(base, domain, count)`. Nice: `pows(floor(logs(x0)))`,
  `pows(ceil(logs(x1)))` on the ordered ends (d3 `nice.js`), count
  ignored. Minor: m·baseᵏ for integer bases, m = 2…base−1, between the
  first and last major, majors excluded, stopping after 10,000 values
  (every built-in minor rule caps its output there); a non-integer base
  uses the default. `logs`: base e → `ln`, 10 → `log10`, 2 → `log2`, else `ln x
  / ln b`; `pows`: base 10 with an integral exponent → the exact
  crate-private `ticks::pow10(k)` (`"1e{k}"` parsed, d3's exact
  `pow10`), e → `exp`, else `powf`; both reflected when `negative`.
- **A log domain straddling zero.** `for_domain` sets `negative =
  domain[0] < 0`; `forward` of an `x` with the other sign is `ln` of a
  negative number, NaN, as in d3 (`Math.log(-x)`), and `apply`
  propagates it (core keeps d3's semantics). `ticks::log` bails out to
  `[]` because `logs` of an end is not finite, and `nice` returns the
  domain unchanged when either `logs` is not finite (d3 writes NaN into
  the domain).

**Log tick format** (d3 `log.js` 105–118): the specifier defaults to
`"s"` for base 10 and `","` otherwise; for an integer base with no
precision, `trim` is set; `k = max(1, base · count /
self.ticks(domain, 10).len())`; a value `d` keeps its label when `i ≤
k`, with `i = d / pows(round(logs(d)))` and `i *= base` if `i · base <
base − 0.5`; otherwise its label is `""`.

##### `ticks` rules

```rust
/// d3's linear ticks in data space: `ticks(d[0], d[1], count as f64)`.
pub fn linear(domain: [f64; 2], count: usize) -> Vec<f64>;
/// d3's log ticks (log.js `ticks`): 1…base−1 per decade when fewer than
/// `count` decades (integer bases), else decades through `ticks(i, j, min(j−i, count))`;
/// falls back to linear ticks when that yields fewer than count/2.
pub fn log(base: f64, domain: [f64; 2], count: usize) -> Vec<f64>;
/// k equal steps in transformed space between neighbouring majors,
/// stopping after 10,000 values.
pub fn subdivide<T: Transform>(transform: &T, major: &[f64], k: usize) -> Vec<f64>;
```

`subdivide` takes `T: Transform` without `?Sized`: `Transform: Clone`
already implies `Sized`. All three return values in domain order (a
reversed domain gives reversed ticks). `log` bails out to `[]` when
`logs` of an end is not finite (`[0, 10]`, `[1, ∞]`, base ≤ 0 or 1), and
takes the decades branch when `(j − i + 1)(base − 1)` exceeds Phase 3's
cap.

##### `TickFormat`

```rust
/// Labels for one scale's ticks: a format, and for log scales which ticks get one.
#[derive(Debug, Clone, PartialEq)]
pub struct TickFormat { format: format::Format, keep: Option<LogKeep> } // LogKeep private
impl TickFormat {
    /// One label; "" for a tick a log scale leaves unlabelled.
    pub fn format(&self, value: f64) -> String;
    /// Labels for a whole tick set: the batch grouping rule over all values,
    /// then log blanking.
    pub fn format_all(&self, values: &[f64]) -> Vec<String>;
}
impl From<format::Format> for TickFormat { /* labels every tick */ }

/// d3-scale's tickFormat(start, stop, count, specifier).
pub fn tick_format(locale: &format::Locale, domain: [f64; 2], count: usize,
                   specifier: Option<&str>) -> Result<format::Format, format::InvalidFormat>;

// core/src/format.rs
impl Format { pub fn locale(&self) -> &format::Locale; }
```

`TickFormat` is the d3 name (`tickFormat`), one of the kept composite
names. `LogKeep` holds the base, `k` and the reflection, and applies
the keep rule above. In this phase `format_all` formats value by value
(`Format::format` per value, then blanking); Phase 6 writes
`Format::format_all` (the per-axis grouping rule), makes `format_all`
delegate to it, and adds `TickFormat::labels(&self, &ticks::Set<f64,
f64>, &Locale)`, which compares locales through `Format::locale`.

`tick_format` ports `tickFormat.js`: `step = tick_step(d0, d1, count)`;
default specifier `",f"`; type `s` → precision from
`precision_prefix(step, max(|d0|, |d1|))` and `Format::prefixed`; types
none, `e`, `g`, `p`, `r` → `precision_round(step, max(|d0|, |d1|))`
(minus 1 for `e`); `f`, `%` → `precision_fixed(step)` (minus 2 for
`%`); each only when the specifier has no precision. The neutral
grouping rule applies per value here.

##### Translator (`rules/scale.mjs`, `rules/interpolate.mjs`)

Budget: 300 lines of JavaScript across the two files. Values carry `{t:
"scale", tr, r, n}` (transform, range kind, stops).

| d3 | Rust |
| --- | --- |
| `scaleLinear()`, `scalePow()`, `scaleSqrt()`, `scaleLog()` | `scale::linear()` … `scale::log()` |
| `scaleLinear(range)` / `(domain, range)` | `.range(r)` / `.domain(d).range(r)` |
| `s.domain(d)` as a statement (and chains) | `s = s.domain(d);` (`let mut` already) |
| `s.domain()` / `s.range()` | `s.get_domain()` / `s.get_range()` |
| `s(x)`, `s.invert(y)` | `s.apply(x)`, `s.invert(y)` (numeric ranges only) |
| `s.copy()` | `s.clone()` |
| `s.nice()`, `s.ticks()`, `s.tickFormat()` | count 10 |
| `s.tickFormat(n, spec)` | `s.tick_format_in(&format::Locale::EN_US, n, Some(spec)).unwrap()`; calling it → `.format(x)`; `ticks.map(f)` → `.iter().map(\|v\| f.format(*v)).collect::<Vec<_>>()` |
| `s.rangeRound(r)`, `s.clamp(b)`, `s.unknown(v)`, `.base(b)`, `.exponent(e)` | same names |
| `.interpolate(interpolateRound)` | `.interpolate(\|a: &f64, b: &f64, t: f64\| interpolate::round(*a, *b)(t))` |
| `tickFormat(a, b, n, spec)` | `scale::tick_format(&format::Locale::EN_US, [a, b], n, spec).unwrap()` |
| `interpolateNumber(a, b)`, `interpolateRound(a, b)` | `interpolate::number(a, b)`, `interpolate::round(a, b)` (callable) |
| `assert.throws(() => …)` | `throws(id, \|\| …)` |

Skips: domains or ranges of other than 2 stops → `gap: phase 5` (equal
lengths) or `types:` (unequal: the type sets the count); color ranges
→ `gap: phase 7`, string ranges → `js:`; Dates, strings, `new Number` →
`js:`; `s(NaN)` against `undefined` → `js:`; non-integer tick counts →
`types:`; other interpolators → `gap: phase 23`; getters → `api:`.

**Skip table** (files allowed under 70 %):

| File | Reason |
| --- | --- |
| `d3-scale/test/radial-test.js` | every assertion `api:` (d3's `scaleRadial` is not part of the design) |

##### Decisions this phase may meet

- A translated file other than `radial-test.js` passes under 70 % (the
  linear and pow files carry many Date, string-range and
  `copy()`-isolation cases) — record it with its per-file counts and
  stop; do not bless it silently.

#### Steps

1. **feat(core): interpolate.** `core/src/interpolate.rs`, `lib.rs`;
   `tests/main/interpolate.rs`.
2. **feat(core): Transform and Continuous.** `scale.rs`,
   `scale/continuous.rs`, `scale/transform.rs`,
   `scale/transform/identity.rs`, `ticks.rs` (`mod rules;`),
   `ticks/rules.rs` (`linear`, `subdivide`), `scale/tick_format.rs`,
   `Format::locale` in `format.rs`; the two `compile_fail` doctests and
   their twins; `tests/main/scale.rs`.
3. **feat(core): Pow and sqrt.** `scale/transform/pow.rs`; tests.
4. **feat(core): Log** with `ticks::log`, `pow10`, nice, minor and the
   blanking tick format; `scale/transform/log.rs`; tests including the
   bail-outs and the straddling domain.
5. **feat(harness): grade d3-scale continuous tests.**
   `rules/{scale,interpolate}.mjs`, suites `scale` and `interpolate` in
   `translate.mjs`, regenerated `src/generated/{scale,interpolate}.rs`;
   change core until 0 failing; `--bless --bless-floors` after reading the
   diff; counts per file in the commit body; `scripts/check.sh full`.

#### Exit criteria

1. `cargo test -p rustd3-core` passes; every spot check is a named test
   in `core/tests/main/scale.rs` or `core/tests/main/interpolate.rs`.
2. `cargo run -p rustd3-harness` shows suites `scale` (d3-scale
   `linear-test.js`, `pow-test.js`, `sqrt-test.js`, `log-test.js`,
   `tickFormat-test.js`; `radial-test.js` as `api:` skips) and
   `interpolate` (d3-interpolate `number-test.js`, `round-test.js`) with
   0 failing and 0 not reached; every skip carries a prefix (`gap: phase
   5`, `gap: phase 7`, `gap: phase 23`, `api:`, `types:`, `js:`).
3. `cargo run -p rustd3-harness -- --check` passes; no earlier id
   regresses; every translated file has a `#floor` line and passes at
   least 70 % of its assertions except `radial-test.js`.
4. `cargo test -p rustd3-core --doc` runs two `compile_fail` doctests
   with passing twins: `scale::linear().base(2.0)` (E0599; twin
   `scale::log().base(2.0)`) and `scale::linear().range([0.0f32,
   1.0]).invert(0.5)` (E0599; twin with `f64`).

#### Spot checks

From d3-scale 4.0.2 run in Node through the resolve hook (`EN_US`),
except rows marked neutral or ours.

| Input | Expected | Source |
| --- | --- | --- |
| `linear().domain([0, 100]).range([0, 960])`: `apply(25)` / `invert(480)` | 240 / 50 | Part I, Scales |
| `linear().domain([0.13, 9.71]).range([300, 0]).nice(5)`: domain; `ticks(5)` through `tick_format(5, None)` (neutral) | `[0, 10]`; "0" "2" "4" "6" "8" "10" | Part I, Scales |
| `linear().domain([0.13, 9.71]).nice(10)` / domain `[9.71, 0.13]` nice | `[0, 10]` / `[10, 0]` | d3 |
| `linear().nice` on `[0, 0.96]` / `[1.1, 10.9]` (count 10) | `[0, 1]` / `[1, 11]` | d3 |
| `linear()` ticks(10) labels | "0.0" "0.1" … "1.0" | d3 |
| `linear().tick_format_in(EN_US, 10, Some("+%"))` of 0.3 | "+30%" | d3 |
| domain `[0, 1e6]`, ticks(5) labels: `EN_US` / neutral | "0" "200,000" … "1,000,000" / "0" "200 000" … "1 000 000" | d3 / grouping rule |
| domain `[0, 1e6]`, `tick_format_in(EN_US, 5, Some("s"))` / `"$s"` of 500000 | "0.5M" / "$0.5M" | d3 |
| domain `[1e9+1, 1e9+2]`: ticks(3); `EN_US` label of 1e9+1.5 | `[1000000001, 1000000001.5, 1000000002]`; "1,000,000,001.5" | d3 |
| `[0, 10] → [0, 100]`, clamp: `apply(20)` / `invert(200)` | 100 / 10 | d3 |
| `[0, 0] → [1, 2]`: `apply(0)`; `[1, 2] → [0, 0]`: `invert(0)` | 1.5 / 1.5 | normalize → 0.5 |
| `range_round([0, 10]).apply(0.59)` | 6 | d3 |
| `pow().exponent(2)`, `[0, 10] → [0, 100]`: `apply(5)` / `invert(25)` | 25 / 5 | d3 |
| `pow().exponent(0.5).apply(-4)` / `sqrt()`, `[0, 100] → [0, 10]`, `apply(25)` | −2 / 5 | d3 |
| `pow().exponent(2).domain([0, 10]).ticks(10)` / nice of `[0.13, 9.71]` | `[0, 1, …, 10]` (11) / `[0, 10]` | d3 (linear ticks in data space) |
| `log().base(2.0).domain([1, 1024])`: ticks(10); labels `EN_US` / neutral | `[1, 2, 4, …, 1024]`; last "1,024" / "1024" | d3 / grouping rule |
| `log().domain([1, 1e6])`: ticks(10) / labels | 55 values (1–9 per decade); "1", 8 × "", "10", …, "1k", "10k", "100k", "1M" | d3 |
| `log().domain([1, 1e6]).ticks(5)` | `[1, 10, 100, 1000, 10000, 100000, 1000000]` | d3 |
| `log().domain([1, 100])` labels at count 10 / 5 | "1" "2" "3" "4" "5" "" "" "" "" "10" "20" "30" "40" "50" "" "" "" "" "100" / "1" "2" 7 × "" "10" "20" 7 × "" "100" | d3 (k = 5.26 / 2.63) |
| `log()` nice of `[0.15, 68]` / `[-68, -0.15]` | `[0.1, 100]` / `[-100, -0.1]` | d3 |
| `log().domain([1, 10]).range([0, 1])`: `apply(5)` / `invert(0.5)` | 0.6989700043360187 / 3.16227766016838 | d3 |
| `log().domain([-100, -1]).ticks(10)` | `[-100, -90, …, -10, -9, …, -1]` (19) | d3 |
| `log().domain([0.5, 1.5]).ticks(10)` | `[0.5, 0.6, 0.7, 0.8, 0.9, 1]` | d3 |
| `log().base(e).domain([1, 100])`: ticks / `EN_US` labels | `[1, e, e², e³, e⁴]`; "1" "2.71828182846" "7.38905609893" "20.0855369232" "54.5981500331" | d3 |
| `log().domain([1, 1000])`, `tick_format_in(EN_US, 5, Some("+,"))` | "+1", 8 × "", "+10", 8 × "", "+100", 8 × "", "+1,000" | d3 |
| `log()` ticks of `[1, 1e100]` / `[1e-300, 1e300]` | 11 / 13 values | d3 |
| `log()` ticks of `[0, 10]` / `[1, ∞]` | `[]` / `[]` | d3 too |
| `log().domain([-1.0, 10.0])`: `apply(5.0)` / `ticks(10)` / `.nice(10).get_domain()` | NaN / `[]` / `[-1, 10]` | d3: NaN / `[]` / `[-1, NaN]` |
| `log().domain([1, 1000]).minor(10)` | empty (majors already 1–9 per decade) | ours |
| `log().domain([1, 1e6]).minor(5)` | 2…9 × 10ᵏ for k = 0…5 (48 values) | ours |
| `scale::tick_format(EN_US, [0, 1], 10, None)` / count 20, of 0.2 | "0.2" / "0.20" | d3 |
| `tick_format(EN_US, [-100, 100], 10, None)` of −20 | "−20" | d3 |
| `tick_format(EN_US, [0.19, 0.21], 10, Some("+%"))` of 0.2 | "+20.0%" | d3 |
| `tick_format(EN_US, [0, 9], 10, Some("e"))` 2.1 / `[0, 100], 10, "r"` 2.11e6 / `[0, 0.9], 10, "p"` 0.21 | "2e+0" / "2000000" / "20%" | d3 (`precision_round(10, 100)` = 1) |
| `interpolate::number(10, 20)(0.25)` / `round(10, 42)(0.3)` / `round(2.6, 3.6)(0.6)` | 12.5 / 20 / 3 | d3 |
| `interpolate::number(2e42, 335)(1)` | 335 exactly | d3 |

#### Not in this phase

- No `Symlog`, `Logit`, `Asinh`, `Func`, `TickRule`, `.tick_rule`,
  `scale::piecewise`, `AnyContinuous`, `ticks::transformed` or
  `ticks::decades`, matplotlib fixtures or tick fuzzing (Phase 5).
- No `scale::Identity` (d3's `scaleIdentity`), `scale::Ticks`,
  `ticks::Set`, `locale::Locale`, `Format::format_all` or
  `TickFormat::labels` (Phase 6); no color ranges (Phase 7); no
  `Rescale`, `pan_px`, `zoom_px` or `scale::Pad` (Phase 10); no time
  scales (Phase 27).
- d3's `scaleRadial` is not built; `radial-test.js` is translated as
  `api:` skips.

### Phase 5 — Continuous scales II

*M1 Core contract · builds on phase 4 · about 1,300 lines of Rust (transforms and rules about 750, tests and fuzzing about 450, `AnyContinuous` about 100), a 90-line fixture script and 120 lines of translator rules, 6 commits*

#### Goal

The continuous family is complete: `Symlog` (decades rule), `Logit` and
`Asinh` (matplotlib's major locators), `Func` with a required
`TickRule`, `.tick_rule(…)` on every continuous scale, the remaining
`ticks::` blocks (`transformed`, `decades`), polylinear scales through
`scale::piecewise`, and `AnyContinuous` for transforms chosen at run
time. matplotlib 3.11.2 reference values are checked in and graded;
every built-in tick rule is fuzzed under a timeout. d3's
`symlog-test.js` and the polylinear cases of the linear and pow tests
join the harness.

#### Builds on

- Phase 4: `scale::Transform` (`Clone`; `forward`, `inverse`, `ticks`,
  `minor`, `for_domain`, `nice`, `tick_format`);
  `scale::Continuous<T, R, N>` with the private `forward` cache,
  `Interpolator`, polymap `apply`/`invert` for any N;
  `scale::TickFormat`, `scale::tick_format`; `ticks::{linear, log,
  subdivide}` and the crate-private `ticks::pow10`;
  `interpolate::Interpolate`; `rules/scale.mjs`.
- Phase 3: the tick cap `min(10_000, ceil(10 · max(count, 1)))`.

#### Design

##### Modules

```text
core/src/scale/tick_rule.rs                         scale::TickRule
core/src/scale/piecewise.rs                         scale::piecewise
core/src/scale/any.rs                               scale::AnyContinuous and its From impls
core/src/scale/transform/{symlog,logit,asinh,func,any}.rs
                                                    transform::{Symlog, Logit, Asinh, Func, Any}
core/src/ticks/rules.rs                             + ticks::{transformed, decades}
core/src/scale/continuous.rs                        + tick_rule field and builder
```

Sources: `proto/fscale/src/bin/ext.rs` (`ticks::{transformed, decades,
subdivide}`, `Symlog` and its rule, `TickRule`, `Func`, `.tick_rule`,
the `Reciprocal` user transform; its outputs are the spot checks marked
fscale; its `decades` loops forever on an infinite domain and assumes
ascending ends, so port it with the rules below);
`proto/fscale/src/main.rs` (`Logit`, `Asinh`; its Asinh forward lacks
matplotlib's factor w: use `w·asinh(x/w)`);
`proto/poly/src/{main.rs,bin/mismatch.rs}` (const-generic stops,
bisection over inner stops, the E0308 case); `proto/symlog/ticks.py`
(the d3/matplotlib/ours comparison); matplotlib 3.11.2 `ticker.py`
(`SymmetricalLogLocator`, `LogitLocator.tick_values` and `nonsingular`,
`AsinhLocator.__call__`'s `symthresh` 0.2 and `tick_values`); d3's
`d3-scale/src/symlog.js` and `continuous.js` polymap.

##### `TickRule` and `.tick_rule`

```rust
/// Where ticks go, independent of the transform's own rule.
#[derive(Clone)]
pub enum TickRule {
    /// d3's linear ticks in transformed space, mapped back.
    Transformed,
    /// d3's linear ticks in data space (d3's meaning for every scale).
    Data,
    /// The caller's rule: end stops (domain order) and count to tick values.
    Custom(Arc<dyn Fn([f64; 2], usize) -> Vec<f64> + Send + Sync>),
}
impl fmt::Debug for TickRule { /* Custom prints as "Custom(..)" */ }
impl TickRule {
    pub fn custom(rule: impl Fn([f64; 2], usize) -> Vec<f64> + Send + Sync + 'static) -> Self;
    pub fn ticks<T: Transform>(&self, transform: &T, domain: [f64; 2], count: usize) -> Vec<f64>;
}

impl<T: Transform, R: Interpolate, const N: usize> Continuous<T, R, N> {
    /// Replaces the transform's tick rule (not `.ticks(…)`: E0592).
    pub fn tick_rule(self, rule: TickRule) -> Self;
}
```

`Continuous` gains a private `tick_rule: Option<TickRule>`; `ticks`
uses it when set and the transform's rule otherwise; `minor` still calls
`transform.minor` on whichever majors result. Built-in rules honour the
cap; a `Custom` rule's output is returned as the caller wrote it.

##### `ticks` blocks

With `linear`, `log` and `subdivide` from Phase 4 these are the five
public blocks Part I's Scales section lists.

```rust
/// d3's linear ticks of forward(domain), mapped back through inverse.
pub fn transformed<T: Transform>(transform: &T, domain: [f64; 2], count: usize) -> Vec<f64>;
/// 0 and ±c·10ᵏ inside the domain, every k-th decade to stay near `count`.
pub fn decades(domain: [f64; 2], constant: f64, count: usize) -> Vec<f64>;
```

`decades` on the ordered ends `[a, b]`: `m = max(|a|, |b|)`; bail out
to `[]` unless `constant > 0` and both ends are finite; decades `k = 0,
1, …` while `c·10ᵏ ≤ m` (exact powers through `pow10`; at most 400 decades);
`sides` = 2 when `a < 0 < b`, else 1; `step = max(1, ceil(len(k) ·
sides / max(count, 1)))`; keep `k % step == 0`; emit `±c·10ᵏ` inside
`[a, b]`, and 0 when `a ≤ 0 ≤ b`; sort in domain order.

##### Transforms

```rust
pub struct Symlog { constant: f64 }           // scale::symlog(): 1; .constant(c)
pub struct Logit;                              // field-less strategy (OPAQUE N5 exemption)
pub struct Asinh { linear_width: f64 }        // scale::asinh(): 1; .linear_width(w)
pub struct Func { forward: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
                  inverse: Arc<dyn Fn(f64) -> f64 + Send + Sync>, rule: TickRule }

pub fn symlog() -> Continuous<transform::Symlog>;
pub fn logit() -> Continuous<transform::Logit>;   // set a domain inside (0, 1)
pub fn asinh() -> Continuous<transform::Asinh>;
pub fn func(forward: impl Fn(f64) -> f64 + Send + Sync + 'static,
            inverse: impl Fn(f64) -> f64 + Send + Sync + 'static,
            rule: TickRule) -> Continuous<transform::Func>;
```

`constant` and `linear_width` are builders on
`Continuous<transform::Symlog, R, N>` and `Continuous<transform::Asinh,
R, N>` that refresh the forward cache. The transforms derive `Clone,
Copy, Debug, PartialEq` where their fields allow; `Func` has a
hand-written `Debug`.

- **Symlog** (d3 `symlog.js`): forward `sign(x)·ln1p(|x/c|)`, inverse
  `sign(y)·expm1(|y|)·c`. Ticks: `ticks::linear` when `max(|a|, |b|)/c
  < 100` (under two decades past the constant), else
  `ticks::decades(domain, c, count)`, falling back to linear when the
  decades give fewer than two ticks (`[1000, 2000]` has only 1000 inside
  it). Minor: when every nonzero major is `±c·10ᵏ`, m·10ᵏ·c for m = 2…9
  between neighbouring same-sign majors (none between 0 and ±c),
  stopping after 10,000 values;
  otherwise data-space subdivision by 5 (`ticks::subdivide(&
  transform::Identity, …)`). Subdividing decades in transformed space
  would put minors at 0.149, 0.32, 0.516 (Part I, Scales). Nice and tick
  format: the defaults (d3's symlog is linearish).
- **Logit**: forward `ln(p/(1 − p))`, inverse `1/(1 + e^−y)`. Ticks:
  matplotlib's `LogitLocator` with `nbins = max(count, 1)`: order the
  ends; `nonsingular` (non-finite, `vmax ≤ 0` or `vmin ≥ 1` → (1e-7, 1
  − 1e-7); `vmin ≤ 0` → 1e-7; `vmax ≥ 1` → 1 − 1e-7; equal → (0.1v, 1
  − 0.1v)); `ideal(b)` = 10ᵇ for b < 0, 0.5 for b = 0, 1 − 10⁻ᵇ for b >
  0; `binf` = ⌊log₁₀ vmin⌋ if vmin < 0.5, 0 if vmin < 0.9, else
  −⌈log₁₀(1 − vmin)⌉; `bsup` = ⌈log₁₀ vmax⌉ if vmax ≤ 0.5, 1 if vmax ≤
  0.9, else −⌊log₁₀(1 − vmax)⌋; `numideal = bsup − binf − 1`; if
  `numideal ≥ 2`: every `ideal(b)` for b in binf…bsup, or, with `f =
  ⌈numideal / nbins⌉ > 1`, those with `b.rem_euclid(f) == 0`; if
  `numideal < 2`: `ticks::linear`. Values outside the domain are
  dropped (matplotlib leaves that to the axis); domain order. Minor:
  the default (`Transform::minor`, subdivision by 5 in transformed
  space). Nice: the domain unchanged (no nice toward 0 or 1, whose
  transforms are infinite). Tick format: the specifier defaults to
  `".12~g"`; a given specifier is used as is.
- **Asinh**: forward `w·asinh(x/w)`, inverse `w·sinh(y/w)`. Ticks:
  matplotlib's `AsinhLocator` with `numticks = min(count, 10_000) + 1`
  and base 10: if the ends straddle 0 with `|1 + vmax/vmin| < 0.2`, use
  ±max(|vmin|, |vmax|); `ys` = linspace of the transformed ends over
  `numticks` points; when they straddle 0, keep the `ys` with `|y/(ymax
  − ymin)| > 0.5 / numticks` and add 0; `xs = inverse(ys)`; `q =
  sign(x)·10^⌊log₁₀|x|⌋` (0 for x = 0); sorted and unique; fewer than
  two → linspace of the ends; drop values outside the domain; domain
  order. Minor: the default. Nice: the default. Tick format: the
  specifier defaults to `",.12~g"`.
- **Func**: forward and inverse call the closures; ticks
  `self.rule.ticks(self, domain, count)`; other methods default.

**Counts and clipping.** Logit and Asinh port matplotlib's locators
with `nbins = count` and `numticks = count + 1`: count 10 gives
`AsinhLocator`'s default of 11 and the fixture's `nbins=10`, where an
axis-bound matplotlib locator picks nbins from the axis size instead.
Both clip their output to the domain. matplotlib's dense Logit minors
and Asinh's (2, 5) subs are not built in v0.1 (see Later); `.minor`
subdivides in transformed space by 5 for both.

##### `piecewise`

```rust
/// A linear scale with N ≥ 2 stops; domain and range lengths are checked
/// by the compiler (d3 truncates silently). `piecewise([0.0], [1.0])`
/// fails to compile at monomorphization.
pub fn piecewise<R: Interpolate, const N: usize>(domain: [f64; N], range: [R; N])
    -> Continuous<transform::Identity, R, N>;
```

`const { assert!(N >= 2, "piecewise needs at least two stops") }`
rejects N < 2 when the function is instantiated (a monomorphization-time
error, not a `compile_fail` doctest). Everything else (polymap,
`invert`, `nice` and `ticks` on the end stops, `domain`/`range` keeping
N) is Phase 4's code. Polylinear log, pow or symlog scales are not
offered: only `piecewise` (Part I, Settled questions: polylinear
domains).

##### `AnyContinuous`

```rust
/// A built-in transform chosen at run time.
#[derive(Clone, Debug)]
pub enum Any { Identity(Identity), Pow(Pow), Log(Log), Symlog(Symlog), Logit(Logit), Asinh(Asinh), Func(Func) }
impl Transform for Any { /* every method forwarded by a macro_rules! over the variants */ }

pub type AnyContinuous<R = f64, const N: usize = 2> = Continuous<transform::Any, R, N>;
// From<Continuous<transform::X, R, N>> for AnyContinuous<R, N>, one per X, by the same macro
```

Erasing the transform instead of the scale keeps every builder and
reader of `Continuous` on `AnyContinuous` for free; only the seven
transform methods are forwarded. A blanket `From<Continuous<T, …>>`
would overlap the reflexive impl at `T = Any` (E0119), hence one impl
per built-in transform. Transform-specific builders (`base`,
`exponent`, `constant`) are applied before converting:

```rust
let y: AnyContinuous = if log_axis {
    scale::log().domain([1.0, 1e3]).into()
} else {
    scale::linear().domain([0.0, 1e3]).into()
};
```

A crate-private `Continuous::map_transform` moves the fields across.

##### Fuzzing

`core/tests/main/fuzz.rs`: an xorshift64 generator with a fixed seed;
2,000 cases by default (debug, inside the normal suite) and 20,000 when
`RUSTD3_FUZZ=1` (run in release), over every built-in transform
(Identity; Pow 2 and 0.5; Log 10, 2 and e; Symlog 1 and 0.01; Logit;
Asinh 1 and 10; Func sqrt with `Data` and `Transformed`) and both
non-custom rules. Domains from {0, ±5e-324, ±1e-320, ±1e-300, ±0.5, ±1,
1e9 + small, ±1e300, ±f64::MAX, ±∞, NaN, uniform in ±1e6}, both orders;
counts {0, 1, 2, 5, 10, 37, 1000, 10⁶, usize::MAX}. Properties: `ticks`
length ≤ `min(10_000, 10 · max(count, 1))` (saturating); `minor` length
≤ 10,000 (minors fill between majors, so a log domain over 600
decades legitimately has about 4,800 of them); for both: all finite; monotone in domain order; inside the ends
(relative 1e-9; minor values may lie outside, the axis filters them).
The loop runs on a spawned thread; the test waits with
`recv_timeout(60 s)` and fails on timeout. `scripts/check.sh quick`
gains, after the test loop: `if [ "${RUSTD3_FUZZ:-0}" = 1 ]; then
TZ=Asia/Kathmandu cargo test -q -p rustd3-core --release --test main
fuzz; fi`.

##### Fixtures (`dev/fixtures/scales.py` → `scales.json`)

Axis-free locator calls, so results do not depend on figure size:
`SymmetricalLogLocator(linthresh=c, base=10).tick_values(a, b)`;
`LogitLocator(nbins=10).tick_values(a, b)`; `AsinhLocator(w,
numticks=11, base=10).tick_values` after the script applies
`__call__`'s `symthresh` rule. Values are filtered to `[a, b]` (what an
axis shows). Each case is tagged `"compared": true` or `"compared":
false` with a `"reason"` from exactly two strings:

- `"symlog-thinned"`: matplotlib's `SymmetricalLogLocator` emits more
  than 2 × count ticks where ours thins every k-th decade (`[-1e9,
  1e9]`: matplotlib 21, ours 11).
- `"symlog-linear"`: ours ticks linearly, below 100·c or through the
  fewer-than-two-decades fallback (`[-3, 3]`, `[-5, 50]`, `[1000,
  2000]`).

Every Logit and Asinh case is `"compared": true`. JSON keys are sorted
and numbers are `repr` of floats rounded to 12 significant digits, so
reruns are byte-identical. The script asserts `matplotlib.__version__
== "3.11.2"`.

##### Translator

Budget: 120 lines of JavaScript in `rules/scale.mjs`. `scaleSymlog()` →
`scale::symlog()`, `.constant(c)`; a chain whose `domain` and `range`
both have n > 2 numeric stops (in either order, or as the constructor's
two arguments) → `scale::piecewise([…], […])` followed by the remaining
calls; later `domain`/`range` calls of the same length keep N; unequal
lengths stay `types:`; polylinear log, pow or symlog stay `api:`
(`piecewise` is linear only). The scale value carries `n`. No file is
in this phase's skip table.

##### Decisions this phase may meet

- `d3-scale/test/symlog-test.js` passes under 70 % of its 73 assertions
  after blessing — record it with the counts and the reason, and stop.

#### Steps

1. **feat(core): TickRule, transformed and decades**, `.tick_rule`;
   `scale/tick_rule.rs`, `ticks/rules.rs`, `scale/continuous.rs`;
   `tests/main/rules.rs` (fscale rows).
2. **feat(core): Symlog and Func.** `scale/transform/{symlog,func}.rs`,
   constructors in `scale.rs`; tests including Part I's examples.
3. **feat(core): Logit and Asinh.** `scale/transform/{logit,asinh}.rs`;
   tests.
4. **feat(core): piecewise and AnyContinuous.** `scale/piecewise.rs`,
   `scale/any.rs`, `scale/transform/any.rs`; the E0308 doctest pair.
5. **test(core): matplotlib fixtures and tick fuzzing.**
   `dev/fixtures/scales.py`, `scales.json`,
   `tests/main/{fixtures,fuzz}.rs` (registered in `tests/main/main.rs`),
   the `RUSTD3_FUZZ` line in `scripts/check.sh`.
6. **feat(harness): symlog and polylinear d3 tests.** `rules/scale.mjs`,
   `translate.mjs`, regenerated `src/generated/scale.rs`; 0 failing;
   `--bless --bless-floors` after reading the diff; counts in the commit
   body; `scripts/check.sh full`.

#### Exit criteria

1. `cargo test -p rustd3-core` passes, including the 2,000-case debug
   fuzz run; spot checks are named tests in
   `core/tests/main/{rules,fixtures,fuzz}.rs`.
2. `python3 dev/fixtures/scales.py && git diff --exit-code dev/fixtures`
   succeeds (byte-identical regeneration under matplotlib 3.11.2).
3. `cargo test -p rustd3-core --test main fixtures` matches every
   `"compared": true` case (tolerant `Obs`, rel 1e-9) and prints how
   many cases were skipped per tag.
4. `RUSTD3_FUZZ=1 TZ=Asia/Kathmandu cargo test -q -p rustd3-core
   --release --test main fuzz` runs 20,000 cases within its 60 s budget
   with no property violation.
5. `cargo run -p rustd3-harness`: suite `scale` gains d3-scale
   `symlog-test.js`, and Phase 4's `gap: phase 5` polylinear skips are
   translated; 0 failing; `--check` passes; `symlog-test.js` has a
   `#floor` line and passes at least 70 % of its 73 assertions.
6. A `compile_fail` doctest with a passing twin:
   `scale::piecewise([0.0, 50.0, 100.0], [1.0, 2.0])` (E0308; twin with
   three range stops).

#### Spot checks

d3 rows from d3-scale 4.0.2 in Node; matplotlib rows from 3.11.2's
locators as in Fixtures (recomputed); fscale rows from `proto/fscale`.

| Input | Expected | Source |
| --- | --- | --- |
| `symlog().domain([0, 1e6]).ticks(10)` | `[0, 1, 10, 100, 1e3, 1e4, 1e5, 1e6]` | Part I, Scales; = matplotlib |
| same with `.tick_rule(TickRule::Data)` | `[0, 1e5, 2e5, …, 1e6]` (11) | d3's ticks |
| `symlog()` ticks(10) of `[-1000, 1000]` | `[-1000, -100, -10, -1, 0, 1, 10, 100, 1000]` | = matplotlib |
| `symlog()` ticks(10) of `[-1e9, 1e9]` | `[-1e8, -1e6, -1e4, -100, -1, 0, 1, 100, 1e4, 1e6, 1e8]` (11) | decades: 10 × 2 sides / 10 → step 2; matplotlib 21 |
| `symlog()` ticks(10) of `[-3, 3]` / `[-5, 50]` | `[-3, -2.5, …, 3]` (13) / `[-5, 0, 5, …, 50]` (12) | linear under 100·c; Part I's symlog table |
| `symlog()` ticks(10) of `[0, 100]` / `[1, 1e4]` / `[-1e4, 10]` | `[0, 1, 10, 100]` / `[1, 10, 100, 1e3, 1e4]` / `[-1e4, -1e3, -100, -10, -1, 0, 1, 10]` | = matplotlib |
| `symlog()` ticks(10) of `[1000, 2000]` | `[1000, 1100, …, 2000]` | linear fallback (one decade tick) |
| `symlog()` minor of `[-1e3, 1e3]`, values in (0, 12) | 2, 3, …, 9 | ours; not 0.149, 0.32, 0.516 |
| `symlog().domain([-10, 10]).range([0, 1])`: `apply(2)`; `.constant(2.0)`: `apply(2)` | 0.729078454995663; 0.6934264036172708 | d3 |
| `symlog().domain([-10, 10]).range([0, 1]).invert(0.8)` | 3.215369133091901 | d3 |
| `continuous(Logit).domain([0.001, 0.999]).ticks(10)` | `[0.001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999]` | Part I, Scales; = matplotlib |
| same, neutral labels | "0.001" "0.01" "0.1" "0.5" "0.9" "0.99" "0.999" | `".12~g"` |
| same `.range([0, 400])`: `apply(0.5)` / `invert(200)` | 200 / 0.5 | logit symmetry |
| Logit ticks(10) of `[1e-6, 1 − 1e-6]` | `[1e-6, 1e-4, 0.01, 0.5, 0.99, 0.9999, 0.999999]` | numideal 11, f = 2; = matplotlib |
| Logit ticks(10) of `[1e-9, 0.5]` / `[0.01, 0.99]` | `[1e-9, 1e-8, …, 0.1, 0.5]` (10) / `[0.01, 0.1, 0.5, 0.9, 0.99]` | = matplotlib |
| Logit ticks(10) of `[0.2, 0.7]` / `[0.002, 0.998]` | `[0.2, 0.25, …, 0.7]` (11) / `[0.01, 0.1, 0.5, 0.9, 0.99]` | linear when numideal < 2; = matplotlib |
| `asinh()` ticks(10) of `[-1000, 1000]` | `[-100, -10, -1, 0, 1, 10, 100]` | = matplotlib |
| `asinh()` ticks(10) of `[0, 1e6]` / `[-3, 3]` / `[0, 100]` | `[0, 1, 10, …, 1e5]` / `[-1, -0.1, 0, 0.1, 1]` / `[0, 0.1, 1, 10, 100]` | = matplotlib |
| `asinh()` ticks(10) of `[-1e9, 1e9]` | `[-1e9, -1e7, -1e5, -1e3, -10, 0, 10, 1e3, 1e5, 1e7, 1e9]` | = matplotlib |
| `asinh()` ticks(10) of `[-50, 5]` / `[1, 1e4]` / `[-50, 45]` | `[-10, -1, -0.1, 0, 1]` / `[1, 10, 100, 1000]` / `[-10, -1, 0, 1, 10]` | = matplotlib (`[-50, 45]`: `symthresh`) |
| `asinh().linear_width(10.0)` ticks(10) of `[-1000, 1000]` | `[-100, -10, 0, 10, 100]` | = matplotlib |
| `asinh().domain([-1000, 1000]).tick_rule(TickRule::Transformed).ticks(10)` | −201.71, −27.29, −3.63, 0, 3.63, 27.29, 201.71 | Part I, Scales |
| `func(f64::sqrt, \|y\| y * y, TickRule::Data).domain([0, 100]).ticks(5)` | `[0, 20, 40, 60, 80, 100]` | Part I, Scales; fscale |
| same with `TickRule::Transformed` | `[0, 4, 16, 36, 64, 100]` | fscale |
| `linear().domain([0, 12]).tick_rule(TickRule::custom(every 3))` | `[0, 3, 6, 9, 12]` | fscale |
| `continuous(Reciprocal)` (Part I's example) ticks(10) of `[1, 100]` / ticks(5) of `[1, 1e6]` | `[1, 2, …, 9, 10, 20, …, 90, 100]` (19) / `[1, 10, …, 1e6]` | `ticks::log` |
| `piecewise([4, 2, 1], [1, 2, 4])`: `apply(1.5)` / `invert(1.5)` | 3 / 3 | d3 |
| `piecewise([-10, 0, 100], [0, 50, 100])`: `apply(-5)` / `apply(50)` / `ticks(5)` | 25 / 75 / `[0, 20, 40, 60, 80, 100]` | d3 |
| `piecewise([-10.3, 0, 99.1], [0, 1, 2]).nice(10)` domain | `[-20, 0, 100]` | d3 |
| `AnyContinuous::from(log().domain([1, 1e3]))` vs the concrete scale | equal `apply`, `invert`, `ticks(10)`, labels | forwarding |
| fuzz: 2,000 cases (debug) / 20,000 (`RUSTD3_FUZZ=1`, release) | no timeout, no property violation | |

#### Not in this phase

- No `scale::Ticks`, `ticks::Set`, axes or batch labels (Phase 6); no
  `Rescale`, `pan_px`, `zoom_px` or `scale::Pad` (Phase 10); no
  sequential or diverging variants of these transforms (Phase 23).
- No public `ticks::logit` or `ticks::asinh` blocks (the five blocks
  are `linear`, `transformed`, `log`, `decades`, `subdivide`); no
  polylinear non-linear scales; no serialization of transforms
  (closures cannot be serialized).
- matplotlib's Logit and Asinh minor-tick rules are a 0.2 item (see
  Later).

### Phase 6 — Ticks, locale, core axes and the identity scale

*M1 · builds on phase 5 · about 1,300 lines, 5 commits*

#### Goal

After this phase core can draw a d3 axis for any continuous scale.
`locale::Locale` holds the number half of the figure locale; the
`scale::Ticks` trait, with its associated `Domain` and `Step` types, and
the `ticks::Set` it returns are final; labels are produced in one batch,
so the neutral locale's per-axis grouping rule holds; and
`axis::bottom|top|left|right(&scale)` draw the domain path and tick marks
into any `PathContext` and return label positions and strings for the
host's own text, with minor ticks behind `.minor(true)`. `scale::Identity`
(d3's `scaleIdentity`) exists with its `Ticks` impl and is graded by
d3-scale's `identity-test.js`. `scale::TickFormat` gains `labels`, which
`Ticks::labels` calls.

#### Builds on

- Phase 3: `path::{PathContext, SvgPath, svg}`; `ticks::{ticks,
  tick_increment, tick_step, nice}`; `format::{Locale, Format, Specifier,
  InvalidFormat}` with the consts `format::Locale::NEUTRAL` and `EN_US`
  (built by the private `const fn from_table`) and the crate-private
  `group_from` field; `format::precision_fixed`; `js::round`.
- Phase 4: `scale::Transform` (`forward`, `inverse`, `ticks`, `minor`,
  `for_domain`, `nice`, `tick_format`); `scale::Continuous<T, R, N>` with
  `apply`, `ticks`, `minor`, `get_domain`, `get_range` and the
  crate-private `transform()`; `scale::TickFormat { format, keep }` with
  `format` and a value-by-value `format_all`; `scale::tick_format`;
  crate-private `scale::nice_linear`; `ticks::{linear, subdivide}`;
  `Format::locale()`; `dev/harness/translate/rules/scale.mjs`.
- Phase 5: `.tick_rule(TickRule)`, `scale::piecewise`, the Symlog, Logit
  and Asinh transforms with their tick formats, `AnyContinuous`.
- Phase 2: the translator, the ratchet and `#floor` lines.
- `.claude/decisions/phase-03.md` to `phase-05.md`.

#### Design

##### Modules

```text
core/src/locale.rs                   locale::Locale
core/src/format.rs                   + Format::format_all (the batch rule)
core/src/scale/tick_format.rs        + TickFormat::labels; format_all delegates
core/src/ticks/set.rs                ticks::Set (declared in ticks.rs)
core/src/scale/ticks.rs              scale::Ticks
core/src/scale/ticks/continuous.rs   impl Ticks for Continuous<T, f64, N>
core/src/scale/ticks/identity.rs     impl Ticks for scale::Identity
core/src/scale/identity.rs           scale::Identity, scale::identity()
core/src/axis.rs                     axis::{Axis, new, bottom, top, left, right}
core/src/axis/orient.rs              axis::Orient
core/src/axis/label.rs               axis::{Label, Anchor}
```

`ticks::Set` lives under the top-level `ticks` module so the path reads
`ticks::Set`; the trait lives with the scales. If phases 3–5 laid out
`ticks/` children differently, keep their layout and add `set.rs` beside
them. `core/src/lib.rs` gains `pub mod locale; pub mod axis;`.

##### `locale::Locale`

```rust
/// The figure locale: number formats; time formats join in Phase 28.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    number: format::Locale,
}

impl Locale {
    /// SI style: point, U+202F groups of three from five digits, U+2212.
    pub const NEUTRAL: Self = Self { number: format::Locale::NEUTRAL };
    /// d3's default locale.
    pub const EN_US: Self = Self { number: format::Locale::EN_US };
    /// The number half, d3's `formatLocale`.
    pub fn number(&self) -> &format::Locale;
}

impl Default for Locale { /* NEUTRAL */ }
```

Fields stay private, so Phase 28 adds the time half and Phase 26 the
named number tables without breaking a caller. The consts compile
because Phase 3 builds `format::Locale`'s tables through a `const fn`.

##### Batch labels and the grouping rule

```rust
impl format::Format {
    /// Formats a set of values as one axis: with `,` under a locale that
    /// groups from N integer digits (NEUTRAL: 5), either every value is
    /// grouped (some value has ≥ N integer digits) or none is.
    pub fn format_all(&self, values: &[f64]) -> Vec<String>;
}

impl scale::TickFormat {
    /// Labels a whole tick set as one axis under `locale`.
    pub fn labels(&self, set: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String>;
}
```

Integer digits are counted on the value as it will print: after rounding
to the specifier's precision and SI-prefix scaling, without sign,
currency or grouping. Locales whose `group_from` is 1 (every d3 table,
`EN_US`) give the same strings as `format` per value. Fractions are
never grouped. Single values (tooltips) keep Phase 3's per-value rule.

Phase 4 builds `TickFormat::format_all` value by value; this phase makes
it call `Format::format_all` over the whole set first and then apply the
log blanking (`keep`), so blanked labels are `""` while the grouping
decision still sees every value. `TickFormat::labels` is
`self.format_all(set.values())` after re-binding the format to
`locale.number()` (same specifier, same fixed SI prefix) when
`self.format.locale() != locale.number()`.

##### `ticks::Set` and `scale::Ticks`

```rust
/// Tick values with the step and count they were chosen for.
#[derive(Debug, Clone, PartialEq)]
pub struct Set<D, S> {
    values: Vec<D>,
    step: S,
    count: usize,
}

impl<D, S> Set<D, S> {
    pub fn new(values: Vec<D>, step: S, count: usize) -> Self;
    pub fn values(&self) -> &[D];
    pub fn step(&self) -> &S;
    /// The requested count: d3's `tickArguments[0]`, used by log labels.
    pub fn count(&self) -> usize;
    /// Replaces the values, keeping step and count: d3's `tickValues`,
    /// which still formats with `tickFormat(tickArguments)`.
    pub fn with_values(self, values: Vec<D>) -> Self;
}

/// What an axis needs from a scale: d3 duck-types it, this names it.
pub trait Ticks {
    /// The values ticks mark (`f64`, a band key, a `Timestamp`).
    type Domain: Clone;
    /// Chosen by the scale: `f64` (continuous, binned),
    /// `time::Interval` (time), `()` (band, point, ordinal).
    type Step: Clone;

    fn tick_set(&self, count: usize) -> ticks::Set<Self::Domain, Self::Step>;
    /// The tick's position along the range for an axis with `offset`
    /// (d3's `number` / `center`); `None` for a key outside the domain.
    /// A scale whose range is not numeric normalizes it to [0, 1].
    fn position(&self, value: &Self::Domain, offset: f64) -> Option<f64>;
    /// One label per value, as one batch under `locale`.
    fn labels(
        &self,
        ticks: &ticks::Set<Self::Domain, Self::Step>,
        locale: &Locale,
    ) -> Vec<String>;
    /// Minor tick values for `.minor(true)`; none by default.
    fn minor(&self, _ticks: &ticks::Set<Self::Domain, Self::Step>) -> Vec<Self::Domain> {
        Vec::new()
    }
    /// First and last range value, d3's `range[0]` and `range[n-1]`;
    /// `[0, 1]` for a range that is not numeric.
    fn extent(&self) -> [f64; 2];
}
```

The method is `tick_set`, not `ticks`, because `Continuous::ticks(count)
-> Vec<f64>` is d3's and an inherent method shadows a trait method of the
same name. `count` travels in the set beside `step` because d3's log
`tickFormat(count)` needs it: k = max(1, base·count / n), where n is the
length of the scale's default `ticks()` at count 10, not of the set.
`Ticks` declares its own `Domain` so figure bounds can read
`Ticks<Domain = <S as Kind>::Domain>` (Phase 14). The color scales of
Phase 23 and the binned scales of Phase 25 normalize `position` and
`extent` to [0, 1].

`Ticks` is final: later phases add impls (7, 23, 25, 28), never methods;
changing a method is a decision whose commit updates every call site.

##### `impl<T: scale::Transform, const N: usize> Ticks for Continuous<T, f64, N>`

- `Domain = f64`, `Step = f64`.
- `tick_set(count)`: values `self.ticks(count)` (honours `.tick_rule`),
  step `ticks::tick_step(d[0], d[N−1], count as f64)` (d3's tickFormat
  step), count.
- `position(v, _)`: `Some(self.apply(*v))`; the offset is unused
  (continuous scales have no bandwidth). Non-finite results are returned
  and the axis skips them.
- `labels(set, locale)`: `tf = self.transform().tick_format(
  locale.number(), [d[0], d[N−1]], set.count(), None)`, then
  `tf.labels(set, locale)`. With no specifier the transform's default
  always parses, so the `Result` is an invariant (`expect("invariant:
  …")`). Linear-like transforms give d3's `,.{precision_fixed(step)}f`;
  Log gives d3's log format, whose blanked labels stay `""`.
- `minor(set)`: with fewer than two majors, none. Otherwise extend the
  majors by one step on each side in transformed space (u₋₁ = 2u₀ − u₁,
  uₙ₊₁ = 2uₙ − uₙ₋₁, mapped back through `inverse`), call
  `Transform::minor` on the extended list, and keep values inside the
  domain and not equal to a major. Log-like transforms override
  `Transform::minor` (phases 4–5).
- `extent()`: `[range[0], range[N−1]]`.

##### `scale::Identity` (d3's `scaleIdentity`)

```rust
/// d3.scaleIdentity: `apply` and `invert` return their input; domain and range are one array.
#[derive(Debug, Clone, PartialEq)]
pub struct Identity { stops: [f64; 2], unknown: Option<f64> }
pub fn identity() -> Identity;                          // [0, 1]
impl Identity {
    pub fn domain(self, domain: [f64; 2]) -> Self;      // sets the range too (d3: domain = range)
    pub fn range(self, range: [f64; 2]) -> Self;        // same as domain
    pub fn unknown(self, value: f64) -> Self;
    pub fn nice(self, count: usize) -> Self;            // nice_linear on the stops
    pub fn apply(&self, x: f64) -> f64;                 // NaN → unknown (or NaN)
    pub fn invert(&self, y: f64) -> f64;                // = apply
    pub fn ticks(&self, count: usize) -> Vec<f64>;      // ticks::linear
    pub fn tick_format(&self, count: usize, specifier: Option<&str>) -> Result<TickFormat, InvalidFormat>;
    pub fn tick_format_in(&self, locale: &format::Locale, count: usize, specifier: Option<&str>)
        -> Result<TickFormat, InvalidFormat>;
    pub fn get_domain(&self) -> [f64; 2];
    pub fn get_range(&self) -> [f64; 2];
}
```

It is d3's `linearish` over a pass-through function, so it shares
`nice_linear`, `ticks::linear` and `scale::tick_format` with `Linear`;
`tick_format` uses the neutral locale, `tick_format_in` takes one, as on
`Continuous`. `scale.rs` exports `identity()`. Its `Ticks` impl mirrors
the continuous one with the stops as domain and extent: `Domain = Step =
f64`; `tick_set` from `self.ticks(count)` and `tick_step` over the stops;
`position` = `apply`; `labels` through `scale::tick_format(
locale.number(), stops, set.count(), None)`, `TickFormat::from`, then
`labels`; `minor` with the same one-step extension through
`ticks::subdivide(&scale::transform::Identity, …, 5)`. Phase 10 adds
`Rescale` and `Pad`.

Two `Identity`s exist on purpose: `scale::transform::Identity` (the
transform under `Linear`) and `scale::Identity` (this scale). Call sites
write the module path at every use and never import either name at file
scope.

##### Axes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orient { Top, Right, Bottom, Left }

pub struct Axis<'s, S: Ticks> {
    scale: &'s S,
    orient: Orient,
    count: usize,                      // 10, d3's default
    values: Option<Vec<S::Domain>>,
    format: Option<Box<dyn Fn(&S::Domain) -> String + 's>>,
    locale: Locale,                    // NEUTRAL
    inner: f64, outer: f64,            // 6, 6
    padding: f64,                      // 3
    offset: f64,                       // 0.5 (d3 at devicePixelRatio 1)
    minor: bool, minor_size: f64,      // false, 3
}

pub fn new<S: Ticks>(orient: Orient, scale: &S) -> Axis<'_, S>;
pub fn bottom<S: Ticks>(scale: &S) -> Axis<'_, S>;   // and top, left, right

impl<'s, S: Ticks> Axis<'s, S> {
    pub fn ticks(self, count: usize) -> Self;
    pub fn values(self, values: impl IntoIterator<Item = S::Domain>) -> Self;
    pub fn format(self, f: impl Fn(&S::Domain) -> String + 's) -> Self;
    pub fn locale(self, locale: Locale) -> Self;
    pub fn tick_size(self, px: f64) -> Self;        // inner and outer
    pub fn tick_size_inner(self, px: f64) -> Self;
    pub fn tick_size_outer(self, px: f64) -> Self;
    pub fn tick_size_minor(self, px: f64) -> Self;
    pub fn tick_padding(self, px: f64) -> Self;
    pub fn offset(self, px: f64) -> Self;
    pub fn minor(self, on: bool) -> Self;
    /// Domain path, then major ticks, then minor ticks, into `ctx`.
    pub fn draw(&self, ctx: &mut impl PathContext);
    /// Where the host draws each label, and what.
    pub fn labels(&self) -> Vec<Label>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor { Start, Middle, End }

/// One tick label for the host's text: position in axis coordinates,
/// text anchor and d3's `dy` in em.
#[derive(Debug, Clone, PartialEq)]
pub struct Label { x: f64, y: f64, text: String, anchor: Anchor, dy: f64 }
// accessors x(), y(), text(), anchor(), dy()
```

The format closure borrows for `'s`, never `'static` (see Accessors).
The formulas are d3-axis 3.0.0's `axis.js`, with k = −1 for Top and Left
and +1 otherwise, o = offset, r₀ = extent[0] + o, r₁ = extent[1] + o and
p(v) = `position(v, o)? + o`:

| Part | Bottom / Top | Left / Right |
| --- | --- | --- |
| Domain, outer > 0 | M r₀,k·outer L r₀,o L r₁,o L r₁,k·outer | M k·outer,r₀ L o,r₀ L o,r₁ L k·outer,r₁ |
| Domain, outer = 0 | M r₀,o L r₁,o | M o,r₀ L o,r₁ |
| Tick at v | M p,0 L p,k·inner | M 0,p L k·inner,p |
| Minor tick | M p,0 L p,k·minor_size | M 0,p L k·minor_size,p |
| Label | x = p, y = k·(max(inner, 0) + padding) | x = k·(max(inner, 0) + padding), y = p |
| Anchor, dy | Middle; 0.71 (Bottom), 0 (Top) | End (Left) / Start (Right); 0.32 |

Values come from `.values` (through `Set::with_values`) or
`tick_set(count)`; labels from `.format` if set, else `labels(set,
locale)`. Ticks whose position is `None` or non-finite are skipped in
both `draw` and `labels`: d3 keeps them off-screen, and the picture is
the same.

##### Grading

The two snapshots `dev/harness/d3/d3-axis/test/snapshots/
axisLeftScaleLinear.html` and `axisLeftScaleLinearNonNumericRange.html`
grade the axes. A test in `core/tests/main/axis.rs` reads them with a
small hand-written parser (no new dependency), extracts the domain
path's `d`, each tick's `translate(…)` and each `<text>` with its `x` and
`dy`, rewrites `H`/`V` segments to points and compares geometry. A
script recomputing `axis.js`'s formulas in Node would grade them against
themselves, so no JS fixture is written.

The harness's suite `scale` gains `d3-scale/test/identity-test.js` (90
assertion calls), ids `d3-scale/test/identity-test.js:L:C`. Translator
rows in `rules/scale.mjs`, budget 30 lines of JS: `scaleIdentity()` →
`scale::identity()`, `scaleIdentity(d)` → `scale::identity().domain(d)`;
`.domain`, `.range`, `.unknown`, `.nice`, `.ticks`, `.tickFormat` as
Phase 4's linear rows. The file's floor is set by `--bless
--bless-floors` and must reach 70 %.

Skip table:

| File | Reason |
| --- | --- |
| `d3-axis/test/axis-test.js` | file-level skip: DOM getters and defensive copies; the two snapshots grade axes |
| `d3-axis/test/snapshot-test.js` | not translated: needs jsdom; `axis_snapshots` reads its HTML directly |

##### Decisions this phase may meet

- `identity-test.js` passes fewer than 70 % of its assertions —
  recommended: bless the passing ids and list the failing groups by
  cause; record it as a decision with the per-file counts and the
  reason.
- A user `Transform` whose `tick_format(…, None)` returns `Err` makes
  `Ticks::labels` panic with its invariant message — recommended: state
  on `Transform::tick_format` that the default specifier must parse, and
  keep the panic; record it as a decision.

#### Steps

1. **`feat(core): locale and batch number labels`** — `locale.rs`,
   `Format::format_all`, `TickFormat::format_all` delegating to it.
   Tests: the Formats, time and color table rows for NEUTRAL and EN_US;
   0–8000 against 0–12000 at count 5. Verify `cargo test -p rustd3-core
   locale format`.
2. **`feat(core): ticks::Set and the Ticks trait`** — `ticks/set.rs`,
   `scale/ticks.rs`, `scale/ticks/continuous.rs`, `TickFormat::labels`.
   Tests: linear, nice, piecewise (N = 3; ticks act on the end stops),
   log labels with blanks, symlog through its decade rule,
   `.tick_rule(TickRule::custom(…))`, `with_values` keeping step and
   count, minor ticks for linear and log.
3. **`feat(core): identity scale`** — `scale/identity.rs`,
   `scale/ticks/identity.rs`, `identity()` in `scale.rs`; the translator
   rows; `--check`, read the diff, `--bless --bless-floors`. Tests: the
   identity spot rows, its `Ticks` impl against `Linear` over the same
   stops. Verify `cargo run -p rustd3-harness -- --check`.
4. **`feat(core): core axes`** — `axis.rs`, `axis/orient.rs`,
   `axis/label.rs`. Tests through a recording `PathContext` (a test
   double logging each call) for all four orients, outer = 0, inner
   negative, offset 0; the `compile_fail` doctest of exit criterion 4.
5. **`test(core): grade axes against d3`** — `core/tests/main/axis.rs`
   (registered in `core/tests/main/main.rs`) with the snapshot parser and
   `axis_snapshots`. Verify `cargo test -p rustd3-core --test main
   axis_snapshots`; `scripts/check.sh full` in the background.

#### Exit criteria

1. `cargo test -p rustd3-core` passes, including every Spot checks row
   as a named test in the `locale`, `scale::ticks`, `scale::identity` and
   `axis` tests.
2. `cargo test -p rustd3-core --test main axis_snapshots` reproduces both
   d3-axis snapshots (`axisLeftScaleLinear`,
   `axisLeftScaleLinearNonNumericRange`).
3. `cargo run -p rustd3-harness -- --check` passes: `identity-test.js` is
   translated with a floor at ≥ 70 % of its assertions (or the decision
   above), no passing id is lost, and `axis-test.js` stands in the skip
   table untranslated.
4. A `compile_fail` doctest with a passing twin shows `axis::bottom`
   rejecting a type without `Ticks` (E0277; twin: `scale::linear()`).
5. `scripts/check.sh quick` is green on every commit, `full` on the last.

#### Spot checks

Strings with spaces use U+202F under NEUTRAL; minus is U+2212. d3 values
from d3-scale 4.0.2 run in Node through the resolve hook.

| Input | Expected | Source |
| --- | --- | --- |
| linear [0, 8000], `labels(&tick_set(5), &EN_US)` | 0, 2,000, 4,000, 6,000, 8,000 | d3 |
| same, NEUTRAL | 0, 2000, 4000, 6000, 8000 | Formats, time and color |
| linear [0, 12000], count 5, EN_US | 0, 2,000, …, 10,000, 12,000 (7 labels) | d3 |
| same, NEUTRAL | 0, 2 000, 4 000, …, 12 000 | Formats, time and color |
| `,.1f` of 1234567.5, NEUTRAL / EN_US | 1 234 567.5 / 1,234,567.5 | Formats, time and color |
| linear [−1e6, 1e6], count 5, EN_US | −1,000,000, −500,000, 0, 500,000, 1,000,000 | d3 |
| same, NEUTRAL | −1 000 000, −500 000, 0, 500 000, 1 000 000 | batch rule |
| `scale::linear()`, count 10, EN_US | 0.0, 0.1, …, 1.0 (11 labels) | d3 |
| linear [0.13, 9.71] `.nice(5)` | domain [0, 10]; labels 0, 2, 4, 6, 8, 10 | d3, Scales |
| linear [0, 100] → [0, 960], count 8 | values 0, 10, …, 100; positions 0, 96, …, 960; step 10 | d3 |
| `axis::bottom` of that, defaults | domain (0.5, 6) (0.5, 0.5) (960.5, 0.5) (960.5, 6); tick at x = 96.5 from y 0 to 6; label x 96.5, y 9, Middle, dy 0.71 | formulas above |
| `axis::left(&scale::linear())`, EN_US | domain M−6,0.5 L0.5,0.5 L0.5,1.5 L−6,1.5; ticks at y 0.5, 0.6, …, 1.5; labels x −9, End, dy 0.32, "0.0" … "1.0" | snapshot `axisLeftScaleLinear` |
| left axis, linear range [0, 500] | domain M−6,0.5 L0.5,0.5 L0.5,500.5 L−6,500.5; ticks at y 0.5, 50.5, …, 500.5 | snapshot `…NonNumericRange` |
| log [1, 1e4], count 10, EN_US | 37 values (1–9 per decade); non-empty labels 1, 2, 10, 20, 100, 200, 1k, 2k, 10k, the other 28 "" | d3 |
| log [1, 1e4], count 5 | the same 37 values; non-empty labels 1, 10, 100, 1k, 10k | d3 |
| log [1, 100], count 10 | 1, 2, 3, 4, 5, "", "", "", "", 10, 20, 30, 40, 50, "", "", "", "", 100 | d3 |
| linear [0, 10], count 5, minor | majors 0, 2, …, 10; 20 minors 0.4, 0.8, 1.2, 1.6, 2.4, …, 9.6 | subdivision by 5 |
| bottom axis, `.tick_size_outer(0.0)` | domain M0.5,0.5 L960.5,0.5 | formulas above |
| top axis | ticks to y = −6; labels y −9, dy 0 | formulas above |
| `scale::identity().domain([1, 2])`: `apply(0.5)` / `invert(3)` / `get_range()` | 0.5 / 3 / [1, 2] | d3 |
| `scale::identity().apply(NaN)` / with `.unknown(-1.0)` | NaN / −1 (d3: `undefined` / −1) | d3 |

#### Not in this phase

- `Ticks` for band, point and ordinal scales (Phase 7), color scales
  (Phase 23), binned scales (Phase 25) and time (Phase 28, `Step =
  time::Interval`).
- `Locale`'s time half (Phase 28) and the named number tables such as
  `DE_DE` (Phase 26).
- `Rescale` and `Pad` for `scale::Identity` (Phase 10).
- Figure axes, tick counts from axis length, label collisions, offset
  and multiplier notation, rotated labels (phases 14, 16, 33).
- Text measurement or drawing: core returns strings and positions.
- d3-axis transitions (`context.transition`) and DOM attributes.

### Phase 7 — Categorical scales and basic color

*M1 · builds on phase 6 · about 1,300 lines, 5 commits*

#### Goal

After this phase core has d3's categorical scales — `scale::Band<K>`,
`scale::Point<K>` and `scale::Ordinal<K, V>`, with an explicit domain and
opt-in growth (`apply_or_insert`) — each implementing `scale::Ticks` with
`Step = ()`, so `axis::bottom(&band)` works. Colors are value types:
`color::Rgb` and `color::Hsl` with d3's conversions, CSS parsing
(`FromStr`, 148 named colors), formatting, and `Interpolate for Rgb`
(d3's `interpolateRgb` at gamma 1), so `scale::linear().range([white,
steel])` is a `Linear<Rgb>`. Five categorical schemes are `const`
arrays.

#### Builds on

- Phase 4: `interpolate::Interpolate` (`fn lerp(&self, to: &Self, t:
  f64) -> Self`), the type-changing `.range([R; N])` on `Continuous`,
  `invert` only for `f64` ranges.
- Phase 6: `scale::Ticks` (`Domain`, `Step`, `tick_set`, `position(v,
  offset)`, `labels(set, &Locale)`, `minor`, `extent`), `ticks::Set`,
  `locale::Locale`, `axis::*`. This phase adds impls only.
- Phase 3: `js::round` (JavaScript `Math.round`).
- Phase 2: the translator, the ratchet and floors.
- `.claude/decisions/phase-04.md` and `phase-06.md`.

#### Design

##### Modules

```text
core/src/scale/band.rs      Band<K>, scale::band, impl Ticks
core/src/scale/point.rs     Point<K>, scale::point, impl Ticks
core/src/scale/ordinal.rs   Ordinal<K, V>, scale::ordinal, impl Ticks (V = f64)
core/src/color.rs           re-exports; BRIGHTER = 1/0.7, DARKER = 0.7
core/src/color/rgb.rs       Rgb, Interpolate for Rgb
core/src/color/hsl.rs       Hsl
core/src/color/parse.rs     CSS parser (private), shared by both FromStr
core/src/color/named.rs     the 148 names (private table, sorted, binary search)
core/src/color/error.rs     ParseColorError
core/src/scheme.rs          categorical consts
```

`core/src/lib.rs` gains `pub mod color; pub mod scheme;`; `scale.rs`
declares the modules and re-exports `band`, `point`, `ordinal`.

##### Band

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Band<K> {
    keys: Vec<K>,
    range: [f64; 2],
    round: bool,
    padding_inner: f64,
    padding_outer: f64,
    align: f64,
    // cached by every builder, as d3's rescale():
    start: f64, step: f64, bandwidth: f64,
}

/// d3's `scaleBand().domain(keys)`: duplicates keep their first position.
pub fn band<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Band<K>;

impl<K: PartialEq> Band<K> {
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self;
    pub fn range(self, range: [f64; 2]) -> Self;
    pub fn range_round(self, range: [f64; 2]) -> Self;   // range + round(true)
    pub fn round(self, round: bool) -> Self;
    pub fn padding(self, p: f64) -> Self;        // inner = min(1, p), outer = p
    pub fn padding_inner(self, p: f64) -> Self;  // min(1, p)
    pub fn padding_outer(self, p: f64) -> Self;
    pub fn align(self, a: f64) -> Self;          // clamped to [0, 1]
    /// The band's start; `None` for a key outside the domain.
    pub fn apply(&self, key: &K) -> Option<f64>;
    pub fn bandwidth(&self) -> f64;
    pub fn step(&self) -> f64;
    pub fn get_domain(&self) -> &[K];
    pub fn get_range(&self) -> [f64; 2];
}
```

Layout (d3-scale 4.0.2 `band.js` `rescale`), n = keys, pi and po the
paddings, r₀ < r₁ after swapping a reversed range: step = (r₁ − r₀) /
max(1, n − pi + 2·po), floored when `round`; start = r₀ + (r₁ − r₀ −
step·(n − pi))·align; bandwidth = step·(1 − pi); when `round`, start
and bandwidth go through `js::round`; key i maps to start + step·i, or
to start + step·(n − 1 − i) for a reversed range. Lookup is a linear
`position` over `keys` (`K: PartialEq` only, as the `Bin` key in Figures
derives); a key list of hundreds is the expected size.

##### Point

```rust
/// d3's `scalePoint`: a band scale with zero bandwidth.
#[derive(Debug, Clone, PartialEq)]
pub struct Point<K> { band: Band<K> }          // padding_inner = 1

pub fn point<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Point<K>;

impl<K: PartialEq> Point<K> {
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self;
    pub fn range(self, range: [f64; 2]) -> Self;
    pub fn range_round(self, range: [f64; 2]) -> Self;   // range + round(true)
    pub fn round(self, round: bool) -> Self;
    /// The outer padding: d3's `pointish` maps `padding` to `paddingOuter`.
    pub fn padding(self, p: f64) -> Self;
    pub fn align(self, a: f64) -> Self;                  // clamped to [0, 1]
    pub fn apply(&self, key: &K) -> Option<f64>;
    pub fn bandwidth(&self) -> f64;                      // always 0
    pub fn step(&self) -> f64;
    pub fn get_domain(&self) -> &[K];
    pub fn get_range(&self) -> [f64; 2];
}
```

`Point` has no `padding_inner` or `padding_outer` builders: d3's point
scale has neither.

##### Ordinal

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Ordinal<K, V> {
    keys: Vec<K>,
    range: Vec<V>,
    unknown: Option<V>,
}

pub fn ordinal<K: PartialEq, V>(
    keys: impl IntoIterator<Item = K>,
    range: impl IntoIterator<Item = V>,
) -> Ordinal<K, V>;

impl<K: PartialEq, V> Ordinal<K, V> {
    pub fn unknown(self, value: V) -> Self;
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self;
    pub fn range(self, range: impl IntoIterator<Item = V>) -> Self;
    /// `range[i % len]` for a key at index i; else the unknown value.
    pub fn apply(&self, key: &K) -> Option<&V>;
    /// d3's implicit growth, opt in: an unseen key is appended first.
    /// `None` only when the range is empty.
    pub fn apply_or_insert(&mut self, key: K) -> Option<&V>;
    pub fn get_domain(&self) -> &[K];
    pub fn get_range(&self) -> &[V];
}
```

`apply_or_insert` returns `Option<&V>`: an empty range leaves nothing to
return, and a bare `&V` would need a panic. Holding the reference keeps
the scale borrowed mutably, so two lookups in one expression fail with
E0499 (see Scales).

##### `Ticks` impls (`Step = ()`)

| Scale | `Domain` | `tick_set(count)` | `position(v, o)` | `labels` | `extent` |
| --- | --- | --- | --- | --- | --- |
| `Band<K>` where `K: Clone + PartialEq + Display` | `K` | keys, `()`, count | `apply(v)` + max(0, bw − 2o)/2, that offset `js::round`ed when `round` (d3's `center`) | `k.to_string()` per key (d3's identity format) | range |
| `Point<K>` (same bounds) | `K` | keys | `apply(v)` (bw = 0) | same | range |
| `Ordinal<K, f64>` (same bounds) | `K` | keys | `apply(v).copied()` | same | `[range[0], range[len−1]]`, NaN when empty |

The locale is unused by these labels; `minor` keeps the default (none).
Ordinal scales over colors have no `Ticks`: legends read their keys
directly (Phase 17).

##### Colors

```rust
/// sRGB, channels in 0..=255, opacity in 0..=1, as `d3.rgb`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb { pub r: f64, pub g: f64, pub b: f64, pub opacity: f64 }

impl Rgb {
    pub const fn new(r: f64, g: f64, b: f64) -> Self;   // opacity 1
    pub const fn hex(v: u32) -> Self;                   // 0xrrggbb
    pub fn with_opacity(self, opacity: f64) -> Self;
    pub fn brighter(self, k: f64) -> Self;   // channels × (1/0.7)^k
    pub fn darker(self, k: f64) -> Self;     // channels × 0.7^k
    pub fn clamp(self) -> Self;              // clampi / clampa
    pub fn displayable(self) -> bool;        // −0.5 ≤ c < 255.5, 0 ≤ a ≤ 1
    pub fn format_hex(self) -> String;       // #rrggbb
    pub fn format_hex8(self) -> String;      // #rrggbbaa
}
impl fmt::Display for Rgb { /* formatRgb: rgb(r, g, b) or rgba(r, g, b, a) */ }
impl FromStr for Rgb { type Err = ParseColorError; /* d3.color(s).rgb() */ }
impl From<Hsl> for Rgb { /* hsl2rgb */ }
impl Interpolate for Rgb { /* below */ }

/// Hue in degrees, saturation and lightness in 0..=1, as `d3.hsl`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl { pub h: f64, pub s: f64, pub l: f64, pub opacity: f64 }

impl Hsl {
    pub fn brighter(self, k: f64) -> Self;   // l × (1/0.7)^k
    pub fn darker(self, k: f64) -> Self;
    pub fn clamp(self) -> Self;
    pub fn displayable(self) -> bool;
    pub fn format_hsl(self) -> String;
}
impl fmt::Display for Hsl { /* d3: Color.toString is formatRgb */ }
impl FromStr for Hsl { type Err = ParseColorError; /* d3.hsl(s) */ }
impl From<Rgb> for Hsl { /* hslConvert: NaN hue for grays, NaN s at l ∈ {0, 1} */ }

/// A string that is not a CSS color d3 accepts.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("not a CSS color: {0:?}")]
pub struct ParseColorError(String);   // accessor input() -> &str
```

`Rgb` and `Hsl` are plain data with public fields; `ParseColorError`
keeps its name (an allowed composite).

**Parser** (`proto/d3-src/d3-color/src/color.js`, regexes at lines
8–17): trim, then ASCII-lowercase, then in d3's order: `#rgb`, `#rgba`,
`#rrggbb`, `#rrggbbaa`; `rgb(i, i, i)` with integers only; `rgb(p%, p%,
p%)` (× 255/100); `rgba(…, n)` in both forms; `hsl(n, p%, p%)`,
`hsla(…, n)`; the 148 names; `transparent` = (NaN, NaN, NaN, 0). Hex
alpha is `aa / 255` (the short form's `a` expands to `aa`), kept
unrounded as `f64`. d3's constructor rules apply to every form that
carries an alpha: `rgba(r, g, b, a)` with a ≤ 0 makes r, g, b NaN
(`#rgba`, `#rrggbbaa`, `rgba(…)`); `hsla(h, s, l, a)` makes h, s, l NaN
when a ≤ 0, h and s NaN when l ≤ 0 or l ≥ 1, and h NaN when s ≤ 0.
Numbers follow d3's `reN` (`[+-]?(\d*\.)?\d+([eE][+-]?\d+)?`) with
optional whitespace around each argument. The parser is hand-written;
core takes no regex crate. An `Hsl` parsed from an `hsl(…)` string keeps
its components; any other input converts through `Rgb`.

**Formatting:** `clampi(v)` = clamp(`js::round(v)`, or 0 if NaN, to 0,
255); `clampa(a)` = 1 if NaN, else clamp(a, 0, 1); opacity prints with
Rust's shortest round-trip `{}`, which matches JavaScript for the values
d3 tests (`#ff000080` keeps 0.5019607843137255).

**`Interpolate for Rgb`** is d3's `interpolateRgb` at gamma 1 per channel
and opacity (`proto/d3-src/d3-interpolate/src/{rgb,color}.js`,
`nogamma`): with d = b − a, `a + t·d` when d is non-zero and not NaN,
else `a` (or `b` when `a` is NaN), so `transparent` → red at 0.5 is
`rgba(255, 0, 0, 0.5)`. It returns `Rgb`, not a string; `Display` gives
d3's string.

##### Schemes (`core/src/scheme.rs`)

```rust
pub const CATEGORY10: [Rgb; 10];   // d3, = matplotlib tab10
pub const OBSERVABLE10: [Rgb; 10]; // d3 schemeObservable10
pub const DEEP: [Rgb; 10];         // seaborn 0.13.2 "deep"
pub const COLORBLIND: [Rgb; 10];   // seaborn 0.13.2 "colorblind" (theme default)
pub const OKABE_ITO: [Rgb; 8];     // Okabe & Ito 2008, black last
```

| Scheme | Hex, in order |
| --- | --- |
| CATEGORY10 | 1f77b4 ff7f0e 2ca02c d62728 9467bd 8c564b e377c2 7f7f7f bcbd22 17becf |
| OBSERVABLE10 | 4269d0 efb118 ff725c 6cc5b0 3ca951 ff8ab7 a463f2 97bbf5 9c6b4e 9498a0 |
| DEEP | 4c72b0 dd8452 55a868 c44e52 8172b3 937860 da8bc3 8c8c8c ccb974 64b5cd |
| COLORBLIND | 0173b2 de8f05 029e73 d55e00 cc78bc ca9161 fbafe4 949494 ece133 56b4e9 |
| OKABE_ITO | e69f00 56b4e9 009e73 f0e442 0072b2 d55e00 cc79a7 000000 |

Okabe-Ito's order puts black last: that order reproduces the "first 3,
normal" distance of 31.5 CAM02-UCS units in Themes (black first gives
56.8).

##### Harness

Ids are `d3-scale/test/band-test.js:L:C`, `d3-color/test/rgb-test.js:L:C`
and so on. Translator budget: 200 lines of JS in
`rules/{band,ordinal,color}.mjs`. Mappings: `scaleBand(d, r)` and the
`.domain/.range/.padding*/.align/.round/.rangeRound` chains to builders;
`scalePoint` likewise; a call `s(x)` on an ordinal becomes
`apply_or_insert` when no `.unknown(…)` was set (d3's implicit mode) and
`apply` otherwise; `Option` to `Obs` (`None` → `Null`); `rgb(…)`,
`hsl(…)`, `color(…)` to parsing and constructors;
`interpolateRgb(a, b)(t)` to `lerp(…).to_string()`; `instanceof`,
`.copy()` identity checks and `displayable` on non-`Rgb` objects are
`js:` skips. After `--bless --bless-floors`, each of `band-test.js`,
`point-test.js`, `ordinal-test.js` (d3-scale) and `color-test.js`,
`rgb-test.js`, `hsl-test.js` (d3-color) passes ≥ 70 %.

Skip table:

| File | Reason |
| --- | --- |
| `d3-interpolate/test/rgb-test.js` | only its four gamma-1 tests are translated here; the `.gamma(…)` tests are `gap: phase 23` |
| `d3-color/test/{lab,hcl,lch,cubehelix,gray}-test.js` | not translated: Lab, HCL, Cubehelix and Gray arrive in Phase 23 |

##### Decisions this phase may meet

- One of the six graded files passes fewer than 70 % — recommended:
  bless what passes and group the rest by cause; record it as a decision
  with the per-file counts.
- `ordinal-test.js` sets `.unknown(undefined)` (explicit, non-implicit,
  no value) — recommended: translate it as no `.unknown` call with every
  `s(x)` mapped to `apply`, whose `None` grades as d3's `undefined`;
  record it as a decision.

#### Steps

1. **`feat(core): band and point scales`** — `scale/band.rs`,
   `scale/point.rs` with `Ticks`. Tests: the band and point rows of Spot
   checks (including `round`, `range_round` and `align` on `Point`); a
   bottom axis over a band (positions at band centers); the
   `compile_fail` pair for a non-`Display` key.
2. **`feat(core): ordinal scale with an explicit domain`** —
   `scale/ordinal.rs`, `Ticks for Ordinal<K, f64>`. Tests: explicit
   lookups, `unknown`, `apply_or_insert` growth order, empty range,
   duplicate keys, `range[i % len]` cycling.
3. **`feat(core): Rgb and Hsl with CSS parsing`** — `color.rs`,
   `color/{rgb,hsl,parse,named,error}.rs`. Tests: every parse form,
   whitespace and case, the alpha rules, rejects (`#zzz`, `#12345`,
   `rgb(1.5, 2, 3)`, `rgb(10%, 2, 3)`), conversions both ways, gray and
   black hue NaN, formats, clamping.
4. **`feat(core): rgb interpolation and categorical schemes`** —
   `Interpolate for Rgb`, `scheme.rs`. Tests: `Linear<Rgb>` midpoint,
   NaN-channel rules, scheme lengths and first and last entries; the
   `compile_fail` pair for `invert` on `Linear<Rgb>`.
5. **`test(harness): band, point, ordinal and color`** — the rules
   files, regenerated `dev/harness/src/generated.rs`; `--check`, read
   the diff, `--bless --bless-floors`; counts before and after in the
   commit body. `scripts/check.sh full` in the background.

Spot-check tests live in `core/tests/main/{band,point,ordinal,color,
scheme}.rs`, registered in `core/tests/main/main.rs`.

#### Exit criteria

1. `cargo test -p rustd3-core` passes, with every Spot checks row as a
   named test.
2. `cargo run -p rustd3-harness -- --check` passes with band, point,
   ordinal, color, rgb and hsl tests translated and each at its floor of
   ≥ 70 % (or the decision above), plus the four gamma-1 tests of
   d3-interpolate's `rgb-test.js`; no passing id is lost.
3. `cargo test -p rustd3-core --doc` shows two `compile_fail` doctests
   with passing twins: `axis::bottom` on a `Band<K>` whose `K` lacks
   `Display` (E0277, the `Bin` case in Figures), and `invert` on
   `Linear<Rgb>` (E0599).
4. `scripts/check.sh quick` is green on every commit, `full` on the last.

#### Spot checks

d3-scale 4.0.2, d3-color 3.1.0 and d3-interpolate 3.0.1 run in Node
through the resolve hook, or seaborn 0.13.2.

| Input | Expected | Source |
| --- | --- | --- |
| `band(["a","b","c"]).range([0,300]).padding(0.1)` | apply(&"a") Some(9.677419354838719); apply(&"z") None; bandwidth 87.09677419354838; step 96.77419354838709 | d3, Scales |
| same with `.round(true)` | a 11, b 107, bandwidth 86, step 96 | d3 |
| `band(["a","b","c"]).range([300,0])` | a 200, b 100, c 0; bandwidth 100 | d3 |
| `band(["a","b","a","c"]).range([0,120])` | domain [a, b, c]; c 80; bandwidth 40 | d3 |
| `band::<&str>([]).range([0,120])` | step 120, bandwidth 120 | d3 |
| range [0,120], padding_inner 0.2, padding_outer 0.5, align 0 | a 0, b 31.578947368421055, c 63.15789473684211; bandwidth 25.263157894736846 | d3 |
| `point(["a","b","c"]).range([0,1])` | 0, 0.5, 1; bandwidth 0; step 0.5 | d3 |
| same with `.padding(1.0)` | 0.25, 0.5, 0.75; step 0.25 | d3 |
| `point(["a","b","c"]).range([0,100]).round(true).padding(0.5)` | a 17, b 50, c 83; step 33 | d3 |
| `point(["a","b","c"]).range_round([0,100]).padding(0.5)` | a 17, b 50, c 83; step 33 | d3 |
| `point(["a","b","c"]).range([0,100]).padding(0.5).align(0.0)` | 0, 33.333333333333336, 66.66666666666667; step 33.333333333333336 | d3 |
| `axis::bottom(&band)` over the first band, offset 0.5 | tick x 53.22580645161291, 150, 246.77419354838707 (apply + max(0, bw − 1)/2, plus the axis offset 0.5) | d3 `center` |
| `ordinal(["cpu","mem"], CATEGORY10).apply(&"cpu")` | Some(rgb(31, 119, 180)) | Scales |
| … `.apply(&"disk")` | None | Scales |
| empty-domain ordinal, `apply_or_insert("disk")`, then `"net"` | Some(rgb(31, 119, 180)), Some(rgb(255, 127, 14)); domain [disk, net] | d3 |
| `linear().range([Rgb::hex(0xffffff), steelblue]).apply(0.5)` | displays rgb(163, 193, 218) | d3, Scales |
| `Rgb::from(Hsl { h: 200, s: 0.6, l: 0.5, opacity: 1 })` | rgb(51, 153, 204) | d3, Formats, time and color |
| `steelblue.brighter(1.0)` / `.darker(1.0)` | rgb(100, 186, 255) / rgb(49, 91, 126) | d3 |
| `"#zzz".parse::<Rgb>()` | Err(ParseColorError("#zzz")) | Formats, time and color |
| `Rgb::hex(0x000000).lerp(&Rgb::hex(0xffffff), 0.5)` | rgb(128, 128, 128) | d3 |
| `"#ff000080"` | r 255, g 0, b 0, opacity 0.5019607843137255 | d3 |
| `"#f008"` / `"#f000"` | opacity 0.5333333333333333 / (NaN, NaN, NaN, 0) | d3 |
| `"rgba(10,20,30,0)"`, `"hsla(120,50%,50%,0)"` as `Rgb` | (NaN, NaN, NaN, 0) each | d3 |
| `"rgb(100%, 0%, 50%)"` | (255, 0, 127.5); displays rgb(255, 0, 128) | d3 |
| `"hsl(120, 50%, 50%)"` as `Rgb` | (63.75, 191.25, 63.75); displays rgb(64, 191, 64) | d3 |
| `"transparent"` | (NaN, NaN, NaN, 0); displays rgba(0, 0, 0, 0) | d3 |
| `"  Steelblue "`, `"ReBeccaPurple"` | rgb(70, 130, 180), rgb(102, 51, 153) | d3 |
| `Hsl::from(steelblue)` | h 207.27272727272728, s 0.44, l 0.4901960784313726 | d3 |
| `Hsl::from("#808080")` | h NaN, s 0, l 0.5019607843137255 | d3 |
| `steelblue.format_hex()`; `Rgb { 255, 0, 0, 0.5 }.format_hex8()` | #4682b4; #ff000080 | d3 |
| `Rgb { r: 300, g: −5, b: 12.4, opacity: 1 }` display | rgb(255, 0, 12) | d3 |
| `"transparent".lerp("red", 0.5)` | rgba(255, 0, 0, 0.5) | d3 |
| `scheme::DEEP[0..3]`, `DEEP[3]` | 4c72b0 dd8452 55a868; c44e52 | seaborn, Themes |

#### Not in this phase

- Lab, HCL, Cubehelix, Gray; `interpolateRgb.gamma`, `rgbBasis`,
  `interpolateHsl`; sequential and diverging schemes and scales; d3's
  other categorical schemes (Accent, Dark2, Paired, Pastel1, Pastel2,
  Set1–3, Tableau10) (Phase 23).
- Quantize, quantile and threshold scales (Phase 25).
- Figure-level band handles, `.dodge()`, legends (phases 14–17, 33).
- A hashed lookup for band and ordinal keys (d3's `InternMap`).

### Phase 8 — Lines, areas and curves

*M1 · builds on phase 3 · about 1,400 lines, 5 commits*

#### Goal

After this phase `shape::line()` and `shape::area()` draw iterators of
already-scaled geometry into any `PathContext` exactly as `d3.line()` and
`d3.area()` do, with `None` breaking the shape (d3's `defined`), a
constant area baseline (`.y0(c)`), and every d3-shape curve as a variant
of the closed enum `shape::Curve`. d3-shape's curve tests grade the
output through the harness. The phase needs nothing past Phase 3 and may
run before phases 4–7 when a session has room.

#### Builds on

- Phase 3: `path::PathContext` (`move_to`, `line_to`,
  `quadratic_curve_to`, `bezier_curve_to`, `arc`, `close_path`),
  `path::SvgPath`, `path::svg(|p| …) -> String`; `SvgPath` prints
  numbers as d3-path does, and d3-shape's tests compare paths through
  `assertPathEqual`, which normalizes to 3 decimals (`Kind::PathEq`).
- Phase 2: the harness, the translator, the ratchet and floors.

#### Design

##### Modules

```text
core/src/shape.rs                    re-exports; line(), area()
core/src/shape/line.rs               Line, Vertex
core/src/shape/area.rs               Area, Span
core/src/shape/curve.rs              Curve (public enum); State, Line flag (crate-private)
core/src/shape/curve/linear.rs       Linear, LinearClosed
core/src/shape/curve/step.rs         Step(t): Step 0.5, StepBefore 0, StepAfter 1
core/src/shape/curve/basis.rs        Basis, BasisClosed, BasisOpen
core/src/shape/curve/bump.rs         BumpX, BumpY
core/src/shape/curve/bundle.rs       Bundle (wraps a Basis)
core/src/shape/curve/cardinal.rs     Cardinal, CardinalClosed, CardinalOpen
core/src/shape/curve/catmull_rom.rs  CatmullRom, …Closed, …Open (Cardinal(0) at alpha 0)
core/src/shape/curve/monotone.rs     MonotoneX, MonotoneY (reflecting context)
core/src/shape/curve/natural.rs      Natural
```

`core/src/lib.rs` gains `pub mod shape;`.

##### Public API

```rust
/// How consecutive points are joined: d3's curve factories.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    #[default]
    Linear,
    LinearClosed,
    Step,
    StepBefore,
    StepAfter,
    Basis,
    BasisClosed,
    BasisOpen,
    BumpX,
    BumpY,
    /// d3's default beta is 0.85; beta 1 is `Basis`.
    Bundle { beta: f64 },
    /// d3's default tension is 0.
    Cardinal { tension: f64 },
    CardinalClosed { tension: f64 },
    CardinalOpen { tension: f64 },
    /// d3's default alpha is 0.5; alpha 0 is `Cardinal { tension: 0 }`.
    CatmullRom { alpha: f64 },
    CatmullRomClosed { alpha: f64 },
    CatmullRomOpen { alpha: f64 },
    MonotoneX,
    MonotoneY,
    Natural,
}

/// An item a line draws: a point, or `None` for a gap.
pub trait Vertex {
    fn vertex(self) -> Option<(f64, f64)>;
}
// impls: (f64, f64), Option<(f64, f64)>

/// An item an area draws: `(x, y0, y1)`, or `(x, y1)` over the area's
/// baseline, or `None` for a gap.
pub trait Span {
    fn span(self, baseline: f64) -> Option<(f64, f64, f64)>;
}
// impls: (f64, f64, f64), Option<(f64, f64, f64)>, (f64, f64), Option<(f64, f64)>

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Line { curve: Curve }
pub fn line() -> Line;
impl Line {
    pub fn curve(self, curve: Curve) -> Self;
    pub fn draw<V: Vertex>(&self, points: impl IntoIterator<Item = V>, ctx: &mut impl PathContext);
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Area { curve: Curve, baseline: f64 }   // baseline 0, d3's y0 default
pub fn area() -> Area;
impl Area {
    pub fn curve(self, curve: Curve) -> Self;
    /// A constant baseline for `(x, y1)` items: d3's `area().y0(300)`.
    pub fn y0(self, baseline: f64) -> Self;
    pub fn draw<S: Span>(&self, items: impl IntoIterator<Item = S>, ctx: &mut impl PathContext);
}
```

Generators take iterators of geometry (option A in Accessors). Accessors
keeps option C (`impl Into<Accessor>`) for places where constants
dominate, such as an area's baseline; with iterator-first areas that
need is one `f64` builder plus 2-tuple items, with no closures and no
inference cost. A 3-tuple item ignores the baseline.
The accessor wrapper `line_of` (option B) is Phase 29's; an area accessor
wrapper is not planned (an `AreaOf` name would not be an allowed
composite). Curves are one closed enum dispatched by `match`, never a
trait over the set.

##### Drawing loops (d3-shape 3.2.0's, verbatim in structure)

`Line::draw`: create `curve::State::new(self.curve)`; `defined0 =
false`; for each item and once more past the end: when definedness
flips, call `line_start` (becoming defined) or `line_end` (becoming
undefined, and at the end); while defined, `point(x, y)`. NaN
coordinates pass through as in d3 (only `None` breaks).

`Area::draw`: for each defined run, `area_start`, `line_start`, then
`point(x, y1)` per item (buffering x and y0); at the run's end
`line_end`, `line_start`, `point(x, y0)` for the buffered items in
reverse, `line_end`, `area_end`. An area with a closed curve emits what
d3 emits: two closed rings, the top and the baseline, and no `Z` from
the area itself (`LinearClosed` over three points gives
`M0,4L1,2L2,5ZM2,0L1,0L0,0Z`).

`Curve::Bundle` in an area draws as `Curve::Basis`: d3's `Bundle` has no
`areaStart` and throws there, and a panic-free API needs some picture;
the bundle's straightening only makes sense for a single line.

##### Curve state machines

`curve::State` is a crate-private enum with one variant per family, each
holding the d3 fields (`_line`, `_point`, `_x0`, `_y0`, …), with four
methods (`area_start`, `area_end`, `line_start`, `line_end`) and
`point(x, y, ctx)`, dispatched by `match`. Port rules:

- d3's `_line` starts `undefined`, is set to 0 by `areaStart` and to
  `1 − _line` by `lineEnd` (which keeps an undefined value undefined, as
  `NaN`). It is a three-state flag in `curve.rs`, never a NaN sentinel:

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq)]
  enum Line { Unset, Zero, One }

  impl Line {
      /// d3's `_line || (_line !== 0 && _point === 1)`.
      fn closes(self, point: u8) -> bool {
          self == Line::One || (self != Line::Zero && point == 1)
      }
      /// d3's `_line = 1 - _line`.
      fn toggle(self) -> Line {
          match self { Line::Zero => Line::One, Line::One => Line::Zero, Line::Unset => Line::Unset }
      }
  }
  ```

  `area_start` sets `Zero`, `area_end` sets `Unset`, every curve starts
  at `Unset`. A lone point still closes (`M0,1Z`); an area's second
  `line_end` closes the ring.
- `Step`'s `_t` flips (`t = 1 − t`) on `line_end` when the flag is `Zero`
  or `One` (d3's `_line >= 0`), so an area's base line steps the other
  way.
- JavaScript `+x` coercions disappear; NaN comparisons keep IEEE
  semantics.
- `MonotoneY` draws `MonotoneX` through a reflecting context that swaps
  x and y in every call, as d3's `ReflectContext`.
- `CatmullRom*` with `alpha == 0` construct the `Cardinal*` state with
  tension 0; `Bundle` with `beta == 1` constructs `Basis`.
- Closed curves buffer the first points and emit them in `line_end`;
  `Natural` buffers the run and solves the tridiagonal system in
  `line_end`.

Phase 29 reuses `curve::State` for radial lines and areas; keep it
`pub(crate)` with a constructor from `Curve`.

The prototype `proto/rfc/d3/src/shape.rs` (lines 1–188: `Curve` with
four variants, `Line::draw`, `Area::draw`) supplies the iterator-first
drawing; its run-collecting `segment` function misses d3's single-point
`Z`, the area's `_line` handling and closed curves, so the curves are
d3's state machines (`proto/d3-src/d3-shape/src/curve/*.js`, 17 files,
20 factories).

##### Harness

Ids are `d3-shape/test/curve/basis-test.js:L:C`,
`d3-shape/test/line-test.js:L:C`. Translator budget: 150 lines of JS in
`rules/shape.mjs`: `line().curve(curveX)(data)` → `svg(|p|
shape::line().curve(Curve::X).draw(data, p))`, with `[[x, y], …]` →
`[(x, y), …]` and `null` items → `None`; `area()…(data)` → 2-tuple spans
(baseline 0); `curveCardinal.tension(t)` → `Curve::Cardinal { tension: t
}` (likewise `beta`, `alpha`); an empty path → `Obs::Null` (d3 returns
`null`); `assertPathEqual` → the 3-decimal normalized string compare.
JavaScript-only cases are `js:` skips (`curve.tension("0.5")` string
coercion), `.context()` getters `api:` skips. After `--bless
--bless-floors` every one of the 20 files under `d3-shape/test/curve/`
(about 280 assertions) passes ≥ 70 %.

Skip table:

| File | Reason |
| --- | --- |
| `d3-shape/test/line-test.js` | accessor-plumbing tests (`passes d, i and data`, `line(x, y) sets x and y`) are `api:` skips; its path assertions on `[[x, y], …]` data pass; may fall under 70 % |
| `d3-shape/test/area-test.js` | likewise (`lineX0()`… and accessor tests are `api:` skips); may fall under 70 % |

##### Decisions this phase may meet

- A curve file passes fewer than 70 % — recommended: bless what passes
  and group the rest by cause; record it as a decision with the
  per-file counts.

#### Steps

1. **`feat(core): line and area over linear curves`** — `shape.rs`,
   `line.rs`, `area.rs`, `curve.rs` with the `Line` flag, `Linear` and
   `LinearClosed`, `Vertex`, `Span`. Tests: the gap example, the area
   example, baseline 300, default baseline 0, single-point line
   (`M0,1Z`) and area (`M0,1L0,0Z`), empty input (no commands), the
   `LinearClosed` area row.
2. **`feat(core): step, basis, bump and bundle curves`** —
   `curve/{step,basis,bump,bundle}.rs`. Tests: the Spot checks rows, the
   6-command basis count, `Bundle { beta: 1.0 }` equal to `Basis`, the
   `BasisClosed` area row.
3. **`feat(core): cardinal and catmull-rom curves`** —
   `curve/{cardinal,catmull_rom}.rs`, all open and closed variants.
   Tests: tension 0.5, alpha 0 equal to `Cardinal { tension: 0 }`, the
   `CardinalClosed` area row.
4. **`feat(core): monotone and natural curves`** —
   `curve/{monotone,natural}.rs`. Tests: duplicate x, infinite slope,
   coincident points (from `monotoneX-test.js`), monotone Y by
   reflection.
5. **`test(harness): d3-shape curves, line and area`** — `rules/shape.mjs`,
   regenerated `generated.rs`; `--check`, read the diff, `--bless
   --bless-floors`; counts before and after in the commit body.
   `scripts/check.sh full` in the background.

Spot-check tests and the counting test double live in
`core/tests/main/shape.rs`, registered in `core/tests/main/main.rs`.

#### Exit criteria

1. `cargo test -p rustd3-core shape` passes, with every Spot checks row
   as a named test (strings compared after the 3-decimal normalization
   of d3's `assertPathEqual`).
2. `cargo run -p rustd3-harness -- --check` passes with all 20 files
   under `d3-shape/test/curve/` translated, each at its floor of ≥ 70 %
   (or the decision above); `line-test.js` and `area-test.js` translated
   with their path assertions passing.
3. A counting `PathContext` test double shows `Curve::Basis` emitting 6
   commands for 4 points (see Shapes and PathContext).
4. `scripts/check.sh quick` is green on every commit, `full` on the last.

#### Spot checks

Points `pts = [(0,0), (1,2), (2,1), (3,3)]` unless stated; outputs from
d3-shape 3.2.0 run in Node through the resolve hook, as d3 prints them
(3 digits).

| Input | Expected path |
| --- | --- |
| `Linear` | M0,0L1,2L2,1L3,3 |
| `LinearClosed` | M0,0L1,2L2,1L3,3Z |
| `Step` | M0,0L0.5,0L0.5,2L1.5,2L1.5,1L2.5,1L2.5,3L3,3 |
| `StepBefore` | M0,0L0,2L1,2L1,1L2,1L2,3L3,3 |
| `StepAfter` | M0,0L1,0L1,2L2,2L2,1L3,1L3,3 |
| `Basis` | M0,0L0.167,0.333C0.333,0.667,0.667,1.333,1,1.5C1.333,1.667,1.667,1.333,2,1.5C2.333,1.667,2.667,2.333,2.833,2.667L3,3 (6 commands) |
| `BasisClosed` | M1,1.5C1.333,1.667,1.667,1.333,2,1.5C2.333,1.667,2.667,2.333,2.333,2.167C2,2,1,1,0.667,0.833C0.333,0.667,0.667,1.333,1,1.5 |
| `BasisOpen` | M1,1.5C1.333,1.667,1.667,1.333,2,1.5 |
| `BumpX` | M0,0C0.5,0,0.5,2,1,2C1.5,2,1.5,1,2,1C2.5,1,2.5,3,3,3 |
| `BumpY` | M0,0C0,1,1,1,1,2C1,1.5,2,1.5,2,1C2,2,3,2,3,3 |
| `Bundle { beta: 0.85 }` | M0,0L0.167,0.308C0.333,0.617,0.667,1.233,1,1.425C1.333,1.617,1.667,1.383,2,1.575C2.333,1.767,2.667,2.383,2.833,2.692L3,3 |
| `Bundle { beta: 0.5 }` | M0,0L0.167,0.25C0.333,0.5,0.667,1,1,1.25C1.333,1.5,1.667,1.5,2,1.75C2.333,2,2.667,2.5,2.833,2.75L3,3 |
| `Cardinal { tension: 0 }` | M0,0C0,0,0.667,1.833,1,2C1.333,2.167,1.667,0.833,2,1C2.333,1.167,3,3,3,3 |
| `CardinalClosed { tension: 0 }` | M1,2C1.333,2.167,1.667,0.833,2,1C2.333,1.167,3.333,3.167,3,3C2.667,2.833,0.333,0.167,0,0C-0.333,-0.167,0.667,1.833,1,2 |
| `CardinalOpen { tension: 0 }` | M1,2C1.333,2.167,1.667,0.833,2,1 |
| `CatmullRom { alpha: 0.5 }` | M0,0C0,0,0.619,1.938,1,2C1.303,2.049,1.697,0.951,2,1C2.381,1.062,3,3,3,3 |
| `CatmullRomClosed { alpha: 0.5 }` | M1,2C1.303,2.049,1.697,0.951,2,1C2.381,1.062,3.112,2.919,3,3C2.845,3.111,0.155,-0.111,0,0C-0.112,0.081,0.619,1.938,1,2 |
| `CatmullRomOpen { alpha: 0.5 }` | M1,2C1.303,2.049,1.697,0.951,2,1 |
| `CatmullRom { alpha: 0 }` | same as `Cardinal { tension: 0 }` |
| `MonotoneX` | M0,0C0.333,1,0.667,2,1,2C1.333,2,1.667,1,2,1C2.333,1,2.667,2,3,3 |
| `MonotoneY` | M0,0C0.5,0.667,1,1.333,1,2C1,1.667,2,1.333,2,1C2,1.667,2.5,2.333,3,3 |
| `Natural` | M0,0C0.333,1,0.667,2,1,2C1.333,2,1.667,1,2,1C2.333,1,2.667,2,3,3 |
| `Cardinal { tension: 0 }`, points (0,1) (1,3) (2,1) (3,3) (d3 test) | M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3 |
| same with tension 0.5 (d3 test) | M0,1C0,1,0.833333,3,1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3,3,3,3 |
| line of [(0,1)] | M0,1Z |
| line of [] | no commands (d3: `null`) |
| line of [Some((0,75)), Some((100,25)), None, Some((300,50)), Some((400,0))] | M0,75L100,25M300,50L400,0 |
| area of [(0,10,4), (1,10,2), (2,10,5)] as `Some` | M0,4L1,2L2,5L2,10L1,10L0,10Z |
| area `.y0(300.0)` of [(0,4), (1,2), (2,5)] | M0,4L1,2L2,5L2,300L1,300L0,300Z |
| area of [(0,4), (1,2), (2,5)] (baseline 0) | M0,4L1,2L2,5L2,0L1,0L0,0Z |
| area `MonotoneX` of the same | M0,4C0.333,3,0.667,2,1,2C1.333,2,1.667,3.5,2,5L2,0C1.667,0,1.333,0,1,0C0.667,0,0.333,0,0,0Z |
| area `LinearClosed` of the same | M0,4L1,2L2,5ZM2,0L1,0L0,0Z |
| area `BasisClosed` of the same | M1,2.833C1.333,3,1.667,4,1.5,4.333C1.333,4.667,0.667,4.333,0.5,3.833C0.333,3.333,0.667,2.667,1,2.833M1,0C0.667,0,0.333,0,0.5,0C0.667,0,1.333,0,1.5,0C1.667,0,1.333,0,1,0 |
| area `CardinalClosed { tension: 0 }` of the same | M1,2C1.333,2.167,2.167,4.667,2,5C1.833,5.333,0.167,4.5,0,4C-0.167,3.5,0.667,1.833,1,2M1,0C0.667,0,-0.167,0,0,0C0.167,0,1.833,0,2,0C2.167,0,1.333,0,1,0 |
| area of [(0,1)] | M0,1L0,0Z |
| `Cardinal { tension: 0 }` area of (0,1) (1,3) (2,1) (d3 test) | M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1L2,0C2,0,1.333333,0,1,0C0.666667,0,0,0,0,0Z |

#### Not in this phase

- `line_of` (option B accessors), radial lines and areas, arcs, pies,
  stacks and symbols (Phase 29); `shape::link` (Later).
- Vertical areas (d3's `x0`/`x1` with a shared `y`) and
  `lineX0()`-style derived lines; a mark that needs a vertical area
  (density on y, Phase 34) draws through a coordinate-swapping
  `PathContext`.
- Thinning, guard-band clipping and linear segments for thinned data
  (figure marks, Phase 15).
- `SvgPath` digit rounding (Phase 3 owns `path`).

### Phase 9 — Recordings and data

*M1 · builds on phase 3 · about 1,700 lines, 6 commits · two sessions: cut after Step 3*

#### Goal

After this phase core owns long data. `series::Recording<X>` holds one
clock (regular or explicit x) and any number of `f64` channels in chunks
of 65,536 samples, appends atomically, drops whole chunks from the front
and carries an id and a revision; `Copy` views `Series<'a, X>` borrow
one channel. An exact M4 index (one `Vec` per level, stored extremes,
built incrementally) answers per-column first, minimum, maximum and last
within NaN-free runs, with outer neighbours inside the run and a
validity predicate, over `series::Columns` built from explicit
data-space edges. `series::Data` and `series::Points` take collected
pairs, a `Series`, or `series::slices(&x, &y)`. Everything is generic
over x: `f64` here, `Timestamp` from Phase 27.

#### Builds on

- Phase 1: `thiserror`; the process-wide `AtomicU64` counters allowed for
  recording ids; clippy's ban on ambient clocks. Read
  `.claude/decisions/phase-01.md`.
- Phase 3: `js::to_int` (guarded `f64` → integer casts) and the crate's
  conventions. Read `.claude/decisions/phase-03.md`.
- No scale is used: column edges arrive as data-space values the caller
  computed with `scale.invert` at device-pixel edges.

#### Design

##### Modules

```text
core/src/series.rs               re-exports; CHUNK = 65_536, BLOCK = 64
core/src/series/clock.rs         Clock<X>, Coordinate (+ impl for f64)
core/src/series/recording.rs     Recording<X>, Id
core/src/series/view.rs          Series<'a, X>
core/src/series/columns.rs       Columns
core/src/series/m4.rs            m4::Lines, m4::Extremes, the query
core/src/series/data.rs          Data<'a, X, Y>
core/src/series/points.rs        Points<'a, X, Y>
core/src/series/slices.rs        Slices<'a, X, Y>, slices()
core/src/series/error.rs         Error<X>
core/src/series/{chunks,channel,index,runs}.rs   crate-private storage
```

Phase 27 adds `series/clock/timestamp.rs`; keep x-specific code behind
`Coordinate`. `core/src/lib.rs` gains `pub mod series;`.

##### Clock and coordinates

```rust
/// An x type a recording can hold: `f64` here, `Timestamp` from Phase 27.
pub trait Coordinate: Copy + PartialOrd + fmt::Debug + 'static {
    /// A regular clock's step: `f64` for `f64`, `SignedDuration` for `Timestamp`.
    type Period: Copy + PartialEq + fmt::Debug;
    /// `start + index · period`, in the type's own precision.
    fn at(start: Self, period: Self::Period, index: u64) -> Self;
    /// An estimate of the first index whose `at` is ≥ `value`, saturating
    /// at 0 for `value < start`; the recording adjusts it by ±1 against `at`.
    fn index(start: Self, period: Self::Period, value: Self) -> u64;
    /// Whether a value may be stored (finite for `f64`).
    fn is_valid(self) -> bool;
    /// Whether a period is usable (finite and positive for `f64`).
    fn is_valid_period(period: Self::Period) -> bool;
}

/// Where a recording's x values come from.
#[derive(Debug, Clone, PartialEq)]
pub enum Clock<X: Coordinate> {
    /// Sample i (counted from the first sample ever appended) is at
    /// `start + i · period`; `drop_front` keeps this numbering.
    Regular { start: X, period: X::Period },
    /// One stored, non-decreasing x per sample.
    Explicit(Vec<X>),
}
```

The `f64` impl of `index` is `((value − start) / period).ceil().max(0.0)`
(NaN gives 0), cast through `js::to_int`; a value beyond `to_int`'s 2⁵³
saturates to `u64::MAX`.

##### Recording

```rust
/// Identifies a recording for caches: a fresh `AtomicU64` value per
/// recording, including clones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

/// Samples on one clock with any number of `f64` channels; NaN is a gap.
#[derive(Debug)]
pub struct Recording<X: Coordinate> { /* id, revision, offset, len, clock, channels */ }

impl<X: Coordinate> Recording<X> {
    pub fn new<C: AsRef<[f64]>>(clock: Clock<X>, channels: impl IntoIterator<Item = C>)
        -> Result<Self, Error<X>>;
    /// Appends to every channel at once, or to none. A regular clock takes `x = &[]`.
    pub fn extend<C: AsRef<[f64]>>(&mut self, x: &[X], channels: impl IntoIterator<Item = C>)
        -> Result<(), Error<X>>;
    /// Drops the whole chunks that lie before sample `before`; returns
    /// how many samples went (a multiple of 65,536). Indices shift down.
    pub fn drop_front(&mut self, before: usize) -> usize;
    pub fn series(&self, channel: usize) -> Option<Series<'_, X>>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn channels(&self) -> usize;
    pub fn id(&self) -> Id;
    /// Bumped by every successful `extend` and every `drop_front` that dropped.
    pub fn revision(&self) -> u64;
    /// Absolute number of the current first sample (samples ever dropped).
    pub fn offset(&self) -> u64;
    pub fn x(&self, i: usize) -> Option<X>;
    /// The first sample index whose x is ≥ `value` (`partition_point`),
    /// in 0..=len.
    pub fn locate(&self, value: X) -> usize;
    pub fn x_extent(&self) -> Option<[X; 2]>;
    /// Column boundaries in samples for data-space `edges` (≥ 2,
    /// non-decreasing): each edge maps to `locate(edge)`, so an edge below
    /// `x(0)` gives 0 and one above the last x gives `len`.
    pub fn columns(&self, edges: &[X]) -> Result<Columns, Error<X>>;
}

impl<X: Coordinate> Clone for Recording<X> { /* deep copy with a fresh Id */ }
```

`new` and `extend` validate everything before touching storage:
lengths; explicit x valid and non-decreasing across the append
boundary; the period; at most `u32::MAX` retained samples, the index's
offset width. NaN in channels is fine; zero channels are allowed.
`locate` on a regular clock takes `index`'s estimate minus `offset`,
clamps it to `0..=len`, then steps ±1 until `x(i − 1) < value ≤ x(i)`,
so it equals `partition_point` over the materialized x; on an explicit
clock it binary-searches the chunks.

##### Error

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error<X: Coordinate = f64>
where
    X::Period: fmt::Debug,
{
    #[error("channel {channel} has {found} samples, expected {expected}")]
    Length { channel: usize, expected: usize, found: usize },
    #[error("x is not ordered at sample {index}: {next:?} after {previous:?}")]
    Unordered { index: u64, previous: X, next: X },
    #[error("x is not valid at sample {index}: {value:?}")]
    Invalid { index: u64, value: X },
    #[error("a regular clock needs a finite positive period, got {0:?}")]
    Period(X::Period),
    #[error("a regular clock takes no x values, got {0}")]
    Regular(usize),
    #[error("column edge {index} is invalid or out of order: {value:?}")]
    Edge { index: usize, value: X },
    #[error("need at least 2 column edges, got {0}")]
    Edges(usize),
    #[error("columns were built for revision {built} of {recording:?}, which is at {current}")]
    Stale { recording: Id, built: u64, current: u64 },
    #[error("a recording holds at most {max} samples, got {len}")]
    Capacity { len: u64, max: u64 },
}
```

thiserror does not infer bounds for associated types in `{0:?}`
positions, so the `where X::Period: fmt::Debug` clause is the one the
compiler asks for; Step 1 compiles `error.rs` alone first. If
`derive(PartialEq)` needs `X::Period: PartialEq` spelled out, add it the
same way and nothing more.

##### Storage and the index

`Chunks<T>` is a `Vec<Vec<T>>` of inner vectors allocated once at
capacity 65,536, so appends never move samples; sample i is
`chunks[i >> 16][i & 0xffff]`; `drop_front` removes inner vectors.

`Index` per channel has levels `0..=L`; level 0 holds blocks of 64
samples and level k node j covers blocks `[j·2ᵏ, (j+1)·2ᵏ)`:

```rust
struct Level {
    min: Vec<f64>,    // NaN when every sample of the node is NaN
    max: Vec<f64>,
    at_min: Vec<u32>, // offset of the extreme from the node's first sample
    at_max: Vec<u32>,
}
```

Only **complete** nodes are stored: level k has ⌊complete blocks / 2ᵏ⌋
entries. Queries never need an incomplete node (their full-block part
ends at a complete block; the partial tail is scanned raw), so `extend`
only pushes: each newly completed block pushes a leaf, and each pair of
completed children pushes their parent, left to right. Combining two
nodes keeps the left extreme unless the right one is strictly smaller
(min) or larger (max); NaN-only nodes are empty. This reproduces the raw
scan's earliest-index tie rule.

`drop_front` removes the chunks and then rebuilds every level of each
channel's `Index`, and its `Runs`, from the retained samples, through
the same code path as `new`. A rebuild cannot drift from what `new`
builds and costs one pass; trimming levels in place is a Later item.

`Runs` per channel: sorted, disjoint, half-open NaN-free ranges;
`extend` grows the last run when it ended at the old length and the
first new sample is not NaN.

**Memory.** About n/32 nodes × 24 bytes = 0.75 bytes per sample, 9.4 %
of an `f64` channel. The prototype's index-only tree was 3.3 %; stored
values let queries and the validity predicate skip reads into y.
`Index::len_bytes()` (crate-private, compiled in every build) sums
`len() × size_of::<T>()` over every level's four vectors; at 1,000,000
samples the index holds 15,625 + 7,812 + … + 1 = 31,243 nodes, 749,832
bytes, 9.37 % of the 8,000,000-byte channel.

The leaf scans, the tie rule and the test trace come from
`proto/thin2/src/main.rs` (`argminmax`; `Tree`, lines 99–178, blocks of
64, power-of-two groups; `signal`, lines 12–44); per-level vectors of
stored values take the place of its heap-ordered tree of indices. The
run logic and the gap test come from `proto/thin2/src/bin/gaps.rs`
(run detection lines 50–69, per-run M4 lines 114–159, the gap test lines
192–252); shared column boundaries by `partition_point` from
`proto/cols/src/main.rs` (38 µs at 1M samples, 112 µs at 10M).

##### Extremes and M4

```rust
pub mod m4 {
    /// Indices of the minimum and maximum of a sample range, with values.
    pub struct Extremes { /* min: (usize, f64), max: (usize, f64) */ }
    impl Extremes { pub fn min(&self) -> (usize, f64); pub fn max(&self) -> (usize, f64); }

    /// Thinned polylines: sample indices, one slice per NaN-free run.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Lines { /* indices: Vec<usize>, starts: Vec<usize> */ }
    // len() polylines, points() total, iter() -> impl Iterator<Item = &[usize]>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Columns { /* recording: Id, revision: u64, edges: Vec<usize> */ }
impl Columns {
    pub fn len(&self) -> usize;          // number of columns
    pub fn edges(&self) -> &[usize];     // sample index per edge, non-decreasing
    pub fn samples(&self) -> Range<usize>;
}
```

`Series::extremes(samples, valid)`: scan the left partial block raw,
then decompose the full blocks greedily (largest aligned complete node
first), then scan the right partial block, combining left to right.
With a predicate, a node whose stored min (or max) fails `valid`
descends to its two children for that extreme; a failing leaf is
scanned raw with the predicate. NaN never qualifies. The predicate
serves transforms that reject values (`y > 0` under log); a column's
first and last samples are not filtered (the mark handles them like any
invalid point).

`Series::m4(columns, valid)` with edges e₀…e_c and view [v_a, v_b) =
[e₀, e_c):

```text
for each run [ra, rb) with ra < v_b and rb > v_a (binary search for the first):
    line = []
    if ra < v_a: push v_a − 1                      # outer neighbour, same run
    for each column k overlapping [max(ra, v_a), min(rb, v_b)):
        a = max(e_k, ra); b = min(e_{k+1}, rb); skip if a ≥ b
        push sorted, deduplicated {a, argmin, argmax, b − 1}   (extremes(a..b, valid))
    if rb > v_b: push v_b                          # outer neighbour, same run
    emit line
```

Outer neighbours are the adjacent samples inside the same NaN-free run:
a run that ends before the view edge contributes none, and a gap is
never crossed. A view between two samples of one run yields that pair; a
view inside a gap yields nothing. uPlot draws the same picture (its
`nonNullIdx` skips nulls, but its path still breaks at them). `m4`
returns `Error::Stale` when the columns came from another recording or
revision. Below four samples per column every sample is emitted, so
thinning fades out by itself.

##### Views and data

```rust
/// One channel of a recording, borrowed; `Copy`, passed by value from `view`.
pub struct Series<'a, X: Coordinate> { recording: &'a Recording<X>, channel: usize }
impl<X: Coordinate> Clone for Series<'_, X> { /* *self */ }
impl<X: Coordinate> Copy for Series<'_, X> {}

impl<'a, X: Coordinate> Series<'a, X> {
    pub fn recording(self) -> &'a Recording<X>;
    pub fn channel(self) -> usize;
    pub fn len(self) -> usize;
    pub fn is_empty(self) -> bool;
    pub fn get(self, i: usize) -> Option<(X, f64)>;
    pub fn iter(self) -> impl Iterator<Item = (X, f64)> + 'a;
    /// NaN-ignoring min and max of the whole channel, O(log n).
    pub fn extent(self) -> Option<[f64; 2]>;
    pub fn extremes(self, samples: Range<usize>, valid: impl Fn(f64) -> bool) -> Option<m4::Extremes>;
    pub fn runs(self, samples: Range<usize>) -> impl Iterator<Item = Range<usize>> + 'a;
    pub fn m4(self, columns: &Columns, valid: impl Fn(f64) -> bool) -> Result<m4::Lines, Error<X>>;
}

/// Mark data: collected pairs, a borrowed indexed view, or borrowed slices.
/// No bound on `X` or `Y`: band keys (`PartialEq` only) work.
pub trait Data<'a, X, Y> {
    fn into_points(self) -> Points<'a, X, Y>;
}
impl<'a, X, Y, I: IntoIterator<Item = (X, Y)>> Data<'a, X, Y> for I { /* Owned */ }
impl<'a, X: Coordinate> Data<'a, X, f64> for Series<'a, X> { /* Series */ }
impl<'a, X, Y> Data<'a, X, Y> for Slices<'a, X, Y> { /* Slices */ }

/// Borrowed parallel slices, unindexed: `series::slices(&x, &y)`. `Copy`.
pub struct Slices<'a, X, Y> { x: &'a [X], y: &'a [Y] }
pub fn slices<'a, X, Y>(x: &'a [X], y: &'a [Y]) -> Slices<'a, X, Y>;

/// What a mark holds. Opaque: the indexed case exists only for
/// `X: Coordinate, Y = f64`, yet `Points` itself needs no bound.
pub struct Points<'a, X, Y> { repr: Repr<'a, X, Y> }
enum Repr<'a, X, Y> {
    Owned(Vec<(X, Y)>),
    Slices(Slices<'a, X, Y>),
    /// A bound-free, object-safe view of the recording (crate-private
    /// `trait Source<X> { len, get, any }`, implemented for
    /// `Recording<X>`), the channel, and `std::convert::identity`.
    Series(&'a dyn Source<X>, usize, fn(f64) -> Y),
}
impl<'a, X: Copy, Y: Copy> Points<'a, X, Y> {
    pub fn len(&self) -> usize;              // slices: the shorter length
    pub fn is_empty(&self) -> bool;
    pub fn get(&self, i: usize) -> Option<(X, Y)>;
    pub fn iter(&self) -> impl Iterator<Item = (X, Y)> + '_;
}
impl<'a, X: Coordinate, Y> Points<'a, X, Y> {
    /// The indexed view for thinning and `Fit::Visible`: downcasts
    /// `Source::any()` to `Recording<X>` (an invariant, `expect`).
    pub fn series(&self) -> Option<Series<'a, X>>;
}
```

`Points` cannot hold a `Series<'a, X>` directly: `Recording<X>` needs `X:
Coordinate` (its clock stores `X::Period`), which would force the bound
onto every mark's data. This shape compiled clean across two crates (a
`Vec`, a `Series`, `slices`, `pairs.iter().copied()`, `Bin` x keys
without `PartialOrd`, `&str` y keys); `line(&pairs)` fails with E0271.
The impls do not overlap because `Series` and `Slices` are local types
without `IntoIterator` (see Data and identity).

##### Decisions this phase may meet

- The exactness suite agrees with the raw scan but a prototype count in
  Spot checks differs — recommended: the raw-scan reference wins; record
  the new count and its cause as a decision.

#### Steps

1. **`feat(core): series storage, clock and recording`** — first
   `error.rs` alone under `cargo check -p rustd3-core`, then `series.rs`,
   `clock.rs`, `chunks.rs`, `recording.rs`, `channel.rs` (storage only,
   `drop_front` dropping chunks). Tests: every `Error` variant with its
   value; regular `locate` equals `partition_point` over materialized x
   for 10,000 random values (start 0.1, period 1e−4, and start 1e9,
   period 1e−6), including values below `start`; clones get fresh ids;
   `revision` bumps.
2. **`feat(core): incremental M4 index`** — `index.rs`, `runs.rs`;
   `extend` pushes completed nodes; `drop_front` rebuilds the index and
   runs. Tests: levels after 1 + 63 + 64·1023 + 65,537 + random appends
   equal a one-shot build (bitwise, `f64::to_bits`);
   `index_memory_stays_under_ten_percent_of_data` at 1,000,000 samples
   (`len_bytes() / (len × 8) ≤ 0.10`); runs across appends and drops.
3. **`feat(core): extremes, columns and m4`** — `view.rs`, `columns.rs`,
   `m4.rs`. Tests: Spot checks rows A, B, C, E, F and the clamped
   columns row; ties pick the earliest index; ±∞ samples; a column of
   one sample.

   *Cut point: a session that ends here hands off with storage, the
   index and `m4` committed; the next session starts at Step 4.*
4. **`test(core): m4 exactness against full scans`** — a raw-scan
   reference `m4_scan` in the test module. The prototype's trace
   (`signal(n)`, seed 42, x = i/10,000) at n = 1,000,000: the full view
   [x₀, x_{n−1} + 1e−4] at 1000, 2000 and 4000 columns and the
   prototype's 50 other views; the gap trace (`gaps.rs`, seed 9, four
   gaps each of 1, 10, 100, 1,000, 10,000 and 50,000 samples, first
   sample `None`, last NaN) over its 41 views; predicate `y > 0`;
   columns crossing chunk boundaries.
5. **`test(core): appends, drops and clocks agree`** — 20 seeded split
   patterns (pieces of 1 to 200,000 samples) equal one-shot builds; drop
   1, 2 and 5 chunks, then compare against a fresh explicit-clock
   recording of the retained samples (index levels and M4 output);
   regular and explicit clocks of the same x values give identical
   `Columns` and `Lines`.
6. **`feat(core): Data, Points and slices`** — `data.rs`, `points.rs`,
   `slices.rs`; the coherence test and the E0271 `compile_fail` pair; the
   report-only timing test. `scripts/check.sh full` in the background.

Tests live in `core/tests/main/series.rs`, registered in
`core/tests/main/main.rs`.

#### Exit criteria

1. `cargo test -p rustd3-core series` passes, including the exactness
   suite: M4 from the index equals the raw-scan reference in every view
   (prints `51/51` and `41/41 views`, plus the new cases).
2. Split appends equal one-shot builds (index levels bitwise, M4 output)
   for 20 split patterns; a recording after `drop_front` equals a fresh
   explicit-clock recording of the retained samples; regular and
   explicit clocks agree (Step 5's tests).
3. `index_memory_stays_under_ten_percent_of_data` passes at 1,000,000
   samples.
4. `core/tests/main/series.rs` has a generic `fn mark<'a, X, Y>(d: impl
   Data<'a, X, Y>)` taking a `Vec`, a `Series` and `series::slices`;
   `cargo test -p rustd3-core --doc series` shows a `compile_fail`
   doctest where `&Vec<(f64, f64)>` fails with E0271 and its twin
   `.iter().copied()` passes.
5. `cargo test -p rustd3-core --release --test main series -- --ignored`
   prints build and full-view M4 times at 1M and 10M samples (report
   only, no budget).
6. `scripts/check.sh quick` is green on every commit, `full` on the last.

#### Spot checks

A–F: x = 0, 1, …, 9 (`Clock::Regular { start: 0.0, period: 1.0 }`); y =
[0, 5, −1, 3, NaN, NaN, 2, 8, 1, 4]; a = [0, 5, −1, 3, 1, 2, 2, 8, 1, 4].
Checked against the raw-scan reference.

| Input | Expected |
| --- | --- |
| A: y, edges [0, 5, 10] | column edges [0, 5, 10]; lines [[0, 1, 2, 3], [6, 7, 8, 9]] |
| B: y, edges [1.5, 5, 8.5] | edges [2, 5, 9]; lines [[1, 2, 3], [6, 7, 8, 9]] (1 and 9 are neighbours) |
| C: y, edges [0, 5, 10], valid `v > 0` | [[0, 1, 3], [6, 7, 8, 9]] (column 0's valid min is 3; first sample 0 kept) |
| E: a, edges [0, 5, 10] | [[0, 1, 2, 4, 5, 7, 8, 9]] |
| F: a, edges [3.2, 3.4] (view between samples) | edges [4, 4]; [[3, 4]] |
| a, edges [−1, 20] | edges [0, 10] (clamped by `locate`) |
| regular clock start 0, period 1: `locate(−5.0)`; `Coordinate::index(0.0, 1.0, −5.0)` | 0; 0 (saturated) |
| y: `extent()`; `extremes(0..10, \|_\| true)` | [−1, 8]; min (2, −1), max (7, 8) |
| prototype trace, n = 1M, full view, 1000 / 2000 / 4000 columns | one polyline of 3,949 / 7,791 / 15,429 points; all 50 impacts kept |
| same, view [x[n/2], x[n/2 + 2000]], 1000 columns | 1,981 points (the prototype's 1,979 plus two outer neighbours) |
| gap trace, n = 1M, full view, 1000 columns | 19 runs; 19 polylines, 3,124 points; no NaN emitted; no segment across a gap |
| `Recording::new(Regular { start: 0.0, period: 0.0 }, …)` | Err(Period(0.0)) |
| explicit x [0, 1, 0.5] | Err(Unordered { index: 2, previous: 1.0, next: 0.5 }) |
| channels of 3 and 2 samples, explicit x of 3 | Err(Length { channel: 1, expected: 3, found: 2 }) |
| 200,000 samples, `drop_front(150_000)` | returns 131,072; len 68,928; offset 131,072; revision +1 |
| regular clock start 0, period 0.1, after that drop: `x(0)` | 13107.2 (131,072 · 0.1) |
| index memory at 1,000,000 samples | 31,243 nodes × 24 B = 749,832 B; 9.37 % ≤ 10 % |
| timings, 2-core container, release (report only) | prototype: build 3.8–5.1 ms at 1M, 40–56 ms at 10M; full-view M4 0.54–1.16 ms at 1M |

#### Not in this phase

- `Recording<Timestamp>` (Phase 27, under `series/clock/`).
- `Series::m4_joint` over two channels of one recording, for ribbons,
  with `Error::Recordings` (Phase 34).
- Thinning inside marks, the M4 memo keyed by id, revision, channel and
  column grid, device-pixel grids, guard-band clipping (phases 15, 21);
  the `Fit::Window` and `Fit::Visible` policies (phases 17, 21) — this
  phase provides `extremes` and `x_extent`.
- Index level surgery in `drop_front` (Later).
- Non-monotonic x, `fill_between` across recordings, LTTB.

### Phase 10 — Pointer, zoom and pixel-space operations

*M1 · builds on phases 5, 6 · about 1,000 lines, 5 commits*

#### Goal

After this phase core has d3-zoom as a state machine: renderer-agnostic
input `interact::Pointer` (buttons, modifiers, line and pixel wheel
steps on both axes, `Left`), d3's `ZoomTransform` as
`interact::Transform`, and `interact::Zoom` with d3's wheel, drag-pan and
double-click behavior, scale and translate extents and d3's
`defaultWheelDelta`. Every continuous scale and `scale::Identity`
implement `interact::Rescale` (`rescale_x`, `rescale_y` and the
pixel-space gesture operations `pan_px`, `zoom_px`) and `scale::Pad`
(fitting margins, matplotlib's `nonsingular`, sticky edges, the domain's
orientation kept).

#### Builds on

- Phase 4: `scale::Transform` (`forward`, `inverse`);
  `scale::Continuous<T, R, N>` with `apply`, `invert` (clamps under
  `.clamp(true)`, as d3), `get_domain`, `get_range`, `.domain(…)` and the
  crate-private `transform()` and `forward()`; `scale::log()`.
- Phase 5: `scale::piecewise` (N > 2 stops); `AnyContinuous` (=
  `Continuous<transform::Any, R, N>`, covered by the generic impls); the
  Symlog, Logit, Asinh and Func transforms.
- Phase 6: `scale::Identity` (`get_domain`, `apply`, `invert`,
  `domain`).
- Phase 2: the translator, the ratchet and floors; Phase 5's pattern for
  checked-in matplotlib fixtures (`dev/fixtures/scales.py`).
- `.claude/decisions/phase-04.md` to `phase-06.md`.

#### Design

##### Modules

```text
core/src/interact.rs             re-exports
core/src/interact/pointer.rs     Pointer, Delta, Button
core/src/interact/modifiers.rs   Modifiers
core/src/interact/transform.rs   interact::Transform
core/src/interact/zoom.rs        Zoom (+ private Gesture)
core/src/interact/rescale.rs     Rescale, impls for Continuous<T, f64, N> and scale::Identity
core/src/scale/pad.rs            Pad, impls for Continuous<T, R, N> and scale::Identity
```

`core/src/lib.rs` and `core/src/scale.rs` declare the new modules.

**Two `Transform`s.** `scale::Transform` (the trait, Phase 4) and
`interact::Transform` (the struct, this phase) coexist. In
`interact/rescale.rs`, `scale/pad.rs` and every later file that needs
both, write the module path at each use (`impl<T: scale::Transform, const
N: usize> Rescale for Continuous<T, f64, N>`; `fn rescale_x(&self, t:
&interact::Transform)`) and never `use …::Transform` at file scope. This
section writes both with their paths throughout.

##### Input

```rust
/// Pointer input in the chart's own pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Pointer {
    Pressed { x: f64, y: f64, button: Button, modifiers: Modifiers },
    Moved { x: f64, y: f64 },
    Released { button: Button },
    Wheel { x: f64, y: f64, delta: Delta, modifiers: Modifiers },
    DoubleClicked { x: f64, y: f64 },
    /// The keyboard modifiers changed (shift-double-click zooms out;
    /// brushes accept and ignore them).
    Modifiers(Modifiers),
    /// The pointer left the area without a button held.
    Left,
}

/// A wheel step in the DOM's convention: positive `y` scrolls down
/// (d3 zooms out), positive `x` scrolls right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Delta { Lines { x: f64, y: f64 }, Pixels { x: f64, y: f64 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button { Left, Right, Middle, Back, Forward, Other(u16) }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers { shift: bool, ctrl: bool, alt: bool, logo: bool }
impl Modifiers {
    pub const NONE: Self; pub const SHIFT: Self; pub const CTRL: Self;
    pub const ALT: Self; pub const LOGO: Self;
    pub const fn union(self, other: Self) -> Self;
    pub fn shift(self) -> bool; pub fn ctrl(self) -> bool;
    pub fn alt(self) -> bool; pub fn logo(self) -> bool;
}
```

`Pointer` is plain data with public fields and `#[non_exhaustive]`, so
matches on it from other crates carry a wildcard arm. `Modifiers` keeps
private fields (it is not one of the plain data types). Phase 11 maps
iced events onto these types: wheel steps in the DOM convention (iced's
sign negated, pixels divided by the scale factor), iced's buttons onto
`Button`, iced's flags into `Modifiers` through `union`. `DoubleClicked`
carries no modifiers, so `Zoom` tracks `Pointer::Modifiers`.

##### `interact::Transform`

```rust
/// d3's `ZoomTransform`: screen = data · k + (x, y).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform { pub k: f64, pub x: f64, pub y: f64 }

impl Transform {
    pub const IDENTITY: Self = Self { k: 1.0, x: 0.0, y: 0.0 };
    pub fn scale(self, k: f64) -> Self;              // (k·self.k, x, y)
    pub fn translate(self, x: f64, y: f64) -> Self;  // (k, x + k·x', y + k·y')
    pub fn apply(self, p: (f64, f64)) -> (f64, f64);
    pub fn apply_x(self, x: f64) -> f64;
    pub fn apply_y(self, y: f64) -> f64;
    pub fn invert(self, p: (f64, f64)) -> (f64, f64);
    pub fn invert_x(self, x: f64) -> f64;
    pub fn invert_y(self, y: f64) -> f64;
}
impl fmt::Display for Transform { /* "translate(x,y) scale(k)"; −0 prints as 0, as JS */ }
```

d3's `transform.rescaleX(x)` is `x.rescale_x(&t)`. The value is lifted
from `proto/rfc/d3/src/interact.rs`; `Zoom` is rebuilt on d3's own
`scale`, `translate` and `constrain` functions
(`proto/d3-src/d3-zoom/src/zoom.js`, d3-zoom 3.0.0).

##### Zoom

```rust
/// d3.zoom(): wheel, drag-to-pan and double-click within limits.
#[derive(Debug, Clone, PartialEq)]
pub struct Zoom {
    scale_extent: [f64; 2],            // [0, ∞]
    translate_extent: [[f64; 2]; 2],   // [[−∞, −∞], [∞, ∞]]
    extent: [[f64; 2]; 2],             // viewport; [[0, 0], [0, 0]] until set
    gesture: Gesture,                  // grab, wheel anchor, modifiers
}
impl Default for Zoom;

impl Zoom {
    pub fn scale_extent(self, extent: [f64; 2]) -> Self;
    pub fn translate_extent(self, extent: [[f64; 2]; 2]) -> Self;
    pub fn extent(self, extent: [[f64; 2]; 2]) -> Self;
    /// Feeds one event; the new transform if it changed.
    pub fn handle(&mut self, input: Pointer, t: interact::Transform) -> Option<interact::Transform>;
    /// A drag is in progress (use case 1 syncs the brush only between gestures).
    pub fn is_active(&self) -> bool;
    pub fn scale_by(&self, t: interact::Transform, k: f64, at: Option<(f64, f64)>) -> interact::Transform;
    pub fn scale_to(&self, t: interact::Transform, k: f64, at: Option<(f64, f64)>) -> interact::Transform;
    pub fn translate_by(&self, t: interact::Transform, x: f64, y: f64) -> interact::Transform;
    pub fn translate_to(&self, t: interact::Transform, x: f64, y: f64, at: Option<(f64, f64)>)
        -> interact::Transform;
    /// d3's defaultConstrain against `extent` and `translate_extent`.
    pub fn constrain(&self, t: interact::Transform) -> interact::Transform;
}
```

Helpers as d3's: `scale(t, k)` clamps k to `scale_extent`;
`translate(t, p, l)` = (t.k, p.x − l.x·t.k, p.y − l.y·t.k); `at = None`
means the centroid of `extent`. `constrain` ports `defaultConstrain`
(`zoom.js` lines 42–52) with JavaScript's `a || b` (use `b` when `a` is 0
or NaN); with the default infinite translate extent it returns `t`.
`translate_to` is kept for d3 parity although no later phase uses it.

**Default extents.** `scale_extent` is d3's `[0, ∞]`; `translate_extent`
is unbounded; `extent` is `[[0, 0], [0, 0]]` until set, since d3 reads it
from the DOM element and core has none, so `at = None` anchors at (0, 0)
until then. Degenerate spans are not guarded by the scale extent but by
`zoom_px`'s `None`, and `handle` returns `None` when the constrained
transform equals the current one. The widget gives each handle's `Zoom`
`[1e-6, 1e6]` (Phase 20).

`handle`, event by event:

| Event | Behavior (d3 function) |
| --- | --- |
| `Pressed { button: Left, modifiers }` without ctrl | grab = (p, t.invert(p)); clears the wheel anchor; `None` (`mousedowned`; d3's `defaultFilter`, lines 13–15, drops other buttons and a ctrl-press) |
| `Moved { x, y }` while grabbed | t′ = constrain(translate(t, p, grab location)); `Some(t′)` if t′ ≠ t |
| `Released { button: Left }` | the grab ends; `None` |
| `Wheel { x, y, delta, modifiers }` | w = −δy · (0.05 for `Lines`, 0.002 for `Pixels`) · (10 with ctrl) (`defaultWheelDelta`, lines 34–36); k = clamp(t.k · 2^w). Anchor: reuse the stored (p, location) if p is unchanged since the last wheel event, else (p, t.invert(p)); with no anchor, k = t.k and δx = 0, `None` (d3 ignores it). t′ = constrain(translate(scale(t, k), p, location)); then a horizontal step pans: t′.x −= δx · (25 px per line, 1 per pixel), constrained, and the anchor's location becomes t′.invert(p). `Some(t′)` if changed |
| `DoubleClicked { x, y }` | k = t.k · (0.5 with shift held, else 2); t′ = constrain(translate(scale(t, k), p, t.invert(p))); `Some(t′)` if changed. No 250 ms transition: the app animates if it wants; the widget's double-click resets instead (Phase 20) |
| `Modifiers(m)` | stored; `None` |
| `Left` | clears the wheel anchor; a drag continues until `Released` (d3 listens on the window); `None` |

Differences from d3, documented on `Zoom`: the wheel anchor ends at the
next non-wheel event or a moved pointer, not after 150 ms idle (core
reads no clock; the results are equal unless `translate_extent`
constrains); 25 px per line for horizontal pans is d3's own
line-to-pixel ratio (0.05 / 0.002), since d3 ignores `deltaX`.

##### Rescale

```rust
/// A scale a zoom can rescale.
pub trait Rescale: Sized {
    /// d3's `transform.rescaleX(scale)`: domain = range.map(invertX).map(scale.invert).
    fn rescale_x(&self, t: &interact::Transform) -> Self;
    /// d3's `transform.rescaleY(scale)`.
    fn rescale_y(&self, t: &interact::Transform) -> Self;
    /// Content follows the pointer by `dx` range units (pixels): the value
    /// shown at range position r moves to r + dx. `None` if degenerate.
    fn pan_px(&self, dx: f64) -> Option<Self>;
    /// Scales by `factor` around the range position `anchor`; > 1 zooms in.
    /// `None` if degenerate.
    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self>;
}

impl<T: scale::Transform, const N: usize> Rescale for Continuous<T, f64, N>;
impl Rescale for scale::Identity;   // d3's scaleIdentity: setting the domain sets the range
```

`rescale_x` and `rescale_y` use `invert` exactly as d3, so a clamping
scale cannot zoom out, as in d3. `pan_px` and `zoom_px` work in
transformed space without clamping: every stop i gets dᵢ′ =
inverse(u(rᵢ − dx)) or inverse(u(a + (rᵢ − a)/factor)), where u is the
piecewise-linear map from range to transformed domain, extended past the
end stops, and a is the anchor. They return `None` when any dᵢ′ is not
finite or `forward(dᵢ′)` is not finite, when the orientation of
d₀′…d′_{N−1} differs from the old domain's, when `factor` is not finite
and positive, or when |d′_{N−1} − d₀′| < 2⁻⁴⁰ · |(d₀′ + d′_{N−1})/2| or
is 0. A rejected step leaves the caller's gesture unchanged. Time scales
get their own impls in Phase 27 (i128 ns, 1 ns minimum span).

##### Pad

```rust
/// Domain fitting for one scale type: margins, nonsingular, sticky edges.
pub trait Pad {
    type Domain;
    /// `extent` is the merged data extent (`None`: no valid data); `margin`
    /// is a fraction of the span per side (themes use 0.05); `sticky`
    /// values are not crossed by the margin (matplotlib's sticky edges).
    /// The result has the scale's domain orientation.
    fn pad(&self, extent: Option<[Self::Domain; 2]>, margin: f64, sticky: &[Self::Domain])
        -> [Self::Domain; 2];
}

impl<T: scale::Transform, R, const N: usize> Pad for Continuous<T, R, N> { type Domain = f64; }
impl Pad for scale::Identity { type Domain = f64; }
```

The algorithm is matplotlib 3.11.2's `_AxesBase.autoscale_view`
(`handle_single_axis`) in transformed space u, on the ordered pair:

1. `None` → [0, 1] if `forward` is finite at 0 and 1; else [1, 10] if it
   is finite at 1 and 10 (matplotlib's `LogLocator.nonsingular` start
   range, whatever the base); else [inverse(0), inverse(1)]. Then go to
   step 5: no margin is added.
2. u₀, u₁ = forward(lo), forward(hi); `nonsingular` with expander 0.05
   (`Locator.nonsingular`): non-finite → [−0.05, 0.05]; max(|u₀|, |u₁|)
   < (1e6 / 1e−15) · `f64::MIN_POSITIVE` → [−0.05, 0.05]; u₁ − u₀ ≤
   max(|u₀|, |u₁|) · 1e−15 → [−0.05, 0.05] if both are 0, else u₀ −=
   0.05|u₀|, u₁ += 0.05|u₁|. Back to x₀, x₁ through `inverse`.
3. Sticky bounds from the sorted `sticky`: tol = 1e−5 · |x₁ − x₀|;
   x₀bound = the largest sticky < x₀ + tol; x₁bound = the smallest sticky
   > x₁ − tol.
4. δ = (u₁ − u₀) · margin (0 if not finite); x₀, x₁ = inverse(u₀ − δ),
   inverse(u₁ + δ); then x₀ = max(x₀, x₀bound), x₁ = min(x₁, x₁bound).
5. The result is reversed when `self.get_domain()[0] >
   self.get_domain()[N−1]` (for N > 2 the end stops decide), as an
   inverted matplotlib axis keeps its inversion.

For identity transforms this is matplotlib exactly. matplotlib widens a
flat log extent by decades (`LogLocator.nonsingular`); `Pad` widens every
transform the same way in u-space (5 % of |u| per side), one rule for
every transform, built-in or user-written. The fixtures therefore cover
linear, sticky, non-flat log and inverted-axis cases only.

`dev/fixtures/pad.py` asserts `matplotlib.__version__ == "3.11.2"`,
records matplotlib's autoscaled limits for those cases and writes
`dev/fixtures/pad.json` (sorted keys, floats through `repr`), so reruns
are byte-identical.

##### Harness

Ids are `d3-zoom/test/transform-test.js:L:C`; the file's 11 assertions
all pass (floor 11). Translator rows in `rules/zoom.mjs`, budget 50 lines
of JS: `zoomIdentity` → `interact::Transform::IDENTITY`, `new
ZoomTransform(k, x, y)` → a struct literal, `apply([x, y])` →
`apply((x, y))`, `applyX`/`applyY`/`invertX`/`invertY` → `apply_x`, …,
`toString()` → `to_string()`.

Skip table:

| File | Reason |
| --- | --- |
| `d3-zoom/test/zoom-test.js` | DOM; its numeric sequences (`scaleBy`, `scaleTo`, `translateBy`, the transform it sets, the default `constrain` arguments) are ported by hand into `interact/zoom.rs` unit tests |
| `d3-zoom/test/zoom-events-test.js` | DOM; filter and extent callbacks |
| `d3-zoom/test/zoom-callback-test.js` | DOM; event callbacks |

##### Decisions this phase may meet

- `Pad` with no data on a Logit scale gives [0.5, 0.7310585786300049]
  (step 1's last case); matplotlib's `LogitLocator.nonsingular` starts
  at [1e−7, 1 − 1e−7] — recommended: match matplotlib by trying that
  pair before the last case; record it as a decision.

#### Steps

1. **`feat(core): pointer input and zoom transform`** — `interact.rs`,
   `interact/{pointer,modifiers,transform}.rs`. Tests: the transform
   rows, `Display`, `Modifiers` unions.
2. **`feat(core): zoom behavior`** — `interact/zoom.rs`. Tests: the wheel
   rows (pixels, lines, ctrl, zoom out, horizontal pan), drag pan with a
   constraining translate extent, double-click with and without shift,
   the hand-ported `zoom-test.js` sequences, scale-extent clamping (k
   stops at 50), `is_active`.
3. **`feat(core): rescale and pixel-space gestures`** —
   `interact/rescale.rs`. Tests: d3's `rescaleX` on linear and log, the
   pan and zoom rows, piecewise (N = 3) stops, `scale::Identity`,
   `AnyContinuous`, every degenerate-span rejection, `rescale_x` equal to
   `zoom_px` for an unclamped scale.
4. **`feat(core): pad fitted domains`** — `scale/pad.rs`,
   `dev/fixtures/pad.py`, `dev/fixtures/pad.json`. Tests: the pad rows,
   orientation, fixture match, empty and non-finite input.
5. **`test(harness): d3-zoom transform test`** — `rules/zoom.mjs`, the
   skip table, regenerated `generated.rs`; `--check`, read the diff,
   `--bless --bless-floors`. `scripts/check.sh full` in the background.

Spot-check tests live in `core/tests/main/{interact,pad}.rs`, registered
in `core/tests/main/main.rs`.

#### Exit criteria

1. `cargo test -p rustd3-core interact scale::pad` passes, including
   every Spot checks row.
2. `cargo run -p rustd3-harness -- --check` passes with
   `d3-zoom/test/transform-test.js` translated and all 11 assertions
   passing; the three DOM files stand in the skip table, and their
   numeric sequences pass as Rust tests in `interact/zoom.rs`.
3. `python3 dev/fixtures/pad.py && git diff --exit-code
   dev/fixtures/pad.json` succeeds, and `Pad` matches every fixture case
   within 1e−12 relative.
4. `scripts/check.sh quick` is green on every commit, `full` on the last.

#### Spot checks

d3 values from d3-zoom 3.0.0 and d3-scale 4.0.2 through the resolve hook;
matplotlib 3.11.2. `x` = `linear().domain([0, 100]).range([0, 960])`;
wheel events at (450, 150) from `IDENTITY`.

| Input | Expected | Source |
| --- | --- | --- |
| `IDENTITY.scale(2.5).scale(2.0)` | (5, 0, 0) | transform-test.js |
| `t = IDENTITY.translate(2, 3)`: `t.translate(−4, 4)`; `t.scale(2).translate(−4, 4)` | (1, −2, 7); (2, −6, 11) | transform-test.js |
| `IDENTITY.translate(2, 3).scale(2)`: `apply((4, 5))`, `invert((4, 5))` | (10, 13); (1, 1) | transform-test.js |
| `IDENTITY.translate(2, 3).scale(2).to_string()`; `IDENTITY.to_string()` | translate(2,3) scale(2); translate(0,0) scale(1) | d3 |
| `IDENTITY.scale(2).translate(1, −3)` | (2, 2, −6) | zoom-test.js |
| `scale_by(IDENTITY, 2, Some((0,0)))`, then `(2, Some((2,−2)))`, then `(0.25, Some((2,−2)))` | (2, 0, 0), (4, −2, 2), (1, 1, −1) | zoom-test.js |
| `scale_to` 2, 2, 1 in turn from `IDENTITY` (extent [[0,0],[0,0]]) | (2, 0, 0), (2, 0, 0), (1, 0, 0) | zoom-test.js |
| `translate_by(IDENTITY, 10, 10)`, `scale_by(·, 2, None)`, `translate_by(·, −10, −10)` (extent [[0,0],[0,0]]) | (1, 10, 10), (2, 20, 20), (2, 0, 0) | zoom-test.js |
| `Zoom::default().constrain(t)` | `t` (extent [[0,0],[0,0]], translate extent unbounded) | zoom-test.js |
| `x.rescale_x(&IDENTITY.translate(−480, 0).scale(2))` | domain [25, 75] | d3 |
| `x.zoom_px(480.0, 2.0)`; `x.pan_px(96.0)` | [25, 75]; [−10, 90] | rule above |
| `log().domain([1, 1000]).range([0, 300])`, rescale by `translate(−150, 0).scale(2)` | [5.62341325190349, 177.82794100389225]; `zoom_px(150, 2)` the same | d3 |
| Wheel `Pixels { x: 0, y: −120 }` | k 1.1809926614295305, x −81.44669764328876, y −27.148899214429576 | d3 `wheeled` |
| Wheel `Lines { x: 0, y: −1 }` | k 1.0352649238413776, x −15.869215728619906, y −5.289738576206645 | d3 |
| Wheel `Lines { x: 0, y: 3 }` (zoom out) | k 0.9012504626108302, x 44.43729182512641, y 14.812430608375479 | d3 |
| Wheel `Pixels { x: 0, y: −10 }`, ctrl | k 1.148698354997035, x −66.91425974866581, y −22.30475324955526; `x.rescale_x` → [6.067942345494186, 93.12299867510657] | d3 |
| Wheel `Lines { x: 0, y: −20 }` | k exactly 2 | Interaction and animation |
| Wheel `Pixels { x: 30, y: 0 }` / `Lines { x: 1, y: 0 }` | (1, −30, 0) / (1, −25, 0) | rule above |
| Wheel `Lines { x: 0, y: 0 }`, no anchor | `None` | d3 |
| Press Left (10, 10), Move (40, 25) | `Some((1, 30, 15))`; after `Released`, Move → `None` | d3 |
| Press Right, or Left with ctrl, then Move | `None` (filtered) | d3 |
| DoubleClicked (100, 50); with shift held | (2, −100, −50); (0.5, 50, 25) | d3 |
| `scale_extent([1, 50])`, wheel `Lines { x: 0, y: −200 }` | k 50 | d3 |
| `linear().domain([0, 1]).range([0, 960]).zoom_px(480, 1e12)` / `1e13` | `Some` (span 1e−12) / `None` (span 1e−13 < 2⁻⁴⁰ · 0.5 = 4.55e−13) | rule above |
| `zoom_px(480, −1.0)`, `zoom_px(480, 0.0)`, `pan_px(f64::NAN)` | `None` each | rule above |
| pad `Some([0, 10])`, margin 0.05 | [−0.5, 10.5] | matplotlib |
| pad `Some([3, 3])` | [2.835, 3.165] | matplotlib |
| pad `Some([0, 0])` | [−0.055, 0.055] | matplotlib |
| pad `Some([0, 4])`, sticky [0] (bars) | [0, 4.2] | matplotlib |
| `log()` pad `Some([1, 1000])` | [0.7079457843841379, 1412.5375446227545] | matplotlib |
| `linear().domain([10, 0])` pad `Some([0, 10])` | [10.5, −0.5] | matplotlib `invert_xaxis` |
| `log().domain([1000, 1])` pad `Some([1, 1000])` | [1412.5375446227545, 0.7079457843841379] | matplotlib `invert_xaxis` |
| pad `None`: linear / log | [0, 1] / [1, 10] | matplotlib |

#### Not in this phase

- Brush, drag, `join`, `anim` (Phase 30); the iced event translation
  (Phase 11); time-scale `Rescale` and `Pad` (Phase 27).
- Figure scale delegation, gesture holding, `Fit` modes, widget state
  keyed by (handle index, `TypeId`) (phases 14, 17, 20).
- Smooth zoom interpolation (`interpolateZoom`, Later); touch and pinch,
  custom `wheelDelta`/`filter`/`constrain` closures, box and axis-drag
  zoom (out of scope).

## M2 — Figure layer and widget

rustd3-iced's core level, the drawing list and its outputs, typst text, the figure model and layout, marks and legends, the `Plot` widget, and the full-frame benchmark.

### Phase 11 — rustd3-iced core level and image placement tests

*M2 · builds on phases 3, 10 · about 700 lines, 6 commits*

#### Goal

iced enters the workspace at the pinned revision. `rustd3-iced` builds
on `iced_core` and `iced_widget` alone and gives core-level apps
`IcedPath` (d3 generators drawing into iced paths, arcs included) and
`pointer::Tracker` (iced events → `interact::Pointer`). Its tests get
the headless pixel harness every later iced test uses
(`UserInterface` → `RedrawRequested` → draw → `Headless::screenshot`,
with `renderer.tick()` once per virtual frame), and placement tests
that pin the two constants the `Plot` widget is built on:
`BAND_BYTES`, the largest image wgpu uploads inside the frame, and
`NUDGE`, the bias that lands an image on exact device pixels on both
renderers. `scripts/check.sh full` runs the iced tests once per
backend.

#### Builds on

- Phase 1: the `iced/` package (`rustd3-iced`, `publish = false`) with
  no iced dependency yet; the `dev/examples` and `dev/bench` packages;
  `scripts/check.sh`; clippy's `disallowed-methods` list (dev crates
  opt out in their own `clippy.toml`).
- Phase 2: testkit `pixels::{Image, Diff}` — `Diff::between(&a, &b,
  threshold)` with `max()`, `share_over()`, `tile_score()`.
- Phase 3: `path::PathContext` (`move_to`, `line_to`,
  `quadratic_curve_to`, `bezier_curve_to`, `arc`, `close_path`) and
  `path::SvgPath`, whose arc semantics `IcedPath` reproduces (a
  negative radius draws nothing).
- Phase 10: `interact::{Pointer, Delta, Button, Modifiers}`;
  `Button::{Left, Right, Middle, Back, Forward, Other(u16)}`;
  `Modifiers::{NONE, SHIFT, CTRL, ALT, LOGO}` with `const fn union`.

#### Design

##### Manifests

This is the only phase that adds iced (Phase 1 declares every other
dependency). Root `Cargo.toml`, `[workspace.dependencies]`:

```toml
iced_core = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
iced_widget = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
iced = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
iced_test = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
iced_runtime = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
```

- `iced/Cargo.toml`: normal dependencies `rustd3-core`, `iced_core`,
  `iced_widget` with `features = ["advanced", "canvas", "image"]`;
  features `default = ["fonts"]`, `fonts = ["rustd3-figure?/fonts"]`
  (weak: it does not turn `figure` on) and `figure =
  ["dep:rustd3-figure", "rustd3-figure/raster"]` (declared here, used
  from Phase 18). Without `fonts`, the workspace entry's
  `default-features = false` would leave widget figures without the
  bundled fonts. Dev: `iced` (default features
  plus `advanced`, `canvas`), `iced_test`, `iced_runtime`,
  `rustd3-testkit`, `png`. `autotests = false` and one `[[test]] name =
  "main", path = "tests/main/main.rs"`.
- `dev/examples`: `iced` (`advanced`, `canvas`) and `rustd3-iced`
  (`features = ["figure"]`) as normal dependencies; `iced_test` and
  `png` as dev-dependencies.
- `dev/bench`: `iced` (`advanced`, `canvas`), `iced_runtime`,
  `rustd3-iced` (`features = ["figure"]`). Phase 22 drives `Plot`
  through these and changes no manifest.

Facts checked at the pinned revision. The image pipelines exist only
because `rustd3-iced` enables `iced_widget/image`, which turns on
`iced_renderer/image`, which forwards to whichever backend the app's
`iced` enables (`renderer/Cargo.toml:20`); the dev crates' `iced` has
no `image` feature, so a dev crate that draws images without depending
on `rustd3-iced` draws nothing. `iced_widget`'s `advanced` is cfg-only.
`iced`'s default features include `crisp` and `web-colors`
(`Cargo.toml:25`): every pixel number this phase records states their
state. `rustd3-iced` alone does not build in release mode
(`iced_renderer` without a backend hits `compile_error!`), so checks
build it in debug or through a dev crate that enables a backend.

##### Modules

```text
iced/src/lib.rs                crate docs; re-exports IcedPath, path, pointer
iced/src/path.rs               IcedPath, path(), arc segmentation
iced/src/pointer.rs            Tracker, button and modifier mapping
iced/tests/main/main.rs        mod declarations only
iced/tests/main/headless.rs    the headless harness
iced/tests/main/path.rs        IcedPath against SvgPath
iced/tests/main/pointer.rs     Tracker tables (events in, pointers out)
iced/tests/main/canvas.rs      a canvas::Program through the harness
iced/tests/main/placement.rs   image placement; BAND_BYTES and NUDGE
```

##### `IcedPath`

```rust
/// d3 generators draw into an iced path through this.
pub struct IcedPath<'a> {
    builder: &'a mut iced_widget::canvas::path::Builder,
    current: Option<(f64, f64)>,
    start: Option<(f64, f64)>,
}
impl<'a> IcedPath<'a> {
    pub fn new(builder: &'a mut canvas::path::Builder) -> Self;
}
impl PathContext for IcedPath<'_> { /* the six methods */ }
/// Builds an iced path from anything that draws into a PathContext.
pub fn path(draw: impl FnOnce(&mut IcedPath<'_>)) -> canvas::Path;
```

A public tuple field over the builder cannot work: iced's
`Builder::arc` calls `ellipse`, which starts with `move_to`, so a d3
pie slice (move, arc, line, arc, close) splits into subpaths and fills
wrong, and the builder exposes no current point
(`graphics/src/geometry/path/builder.rs:41,104–118`). `IcedPath`
therefore keeps private fields, tracks
`current` and the subpath `start` (for `close_path`), and segments arcs
itself with d3's semantics, as `SvgPath` implements them
(`proto/poc/core/src/path.rs:129–161`):

- `r < 0` draws nothing, as `SvgPath`'s `arc` (Phase 3).
- Start point `(x + r cos a0, y + r sin a0)`: `move_to` if there is no
  current point, `line_to` if it differs from the current point by
  more than 1e-6, nothing otherwise. `r == 0` stops there.
- `da = if ccw { a0 - a1 } else { a1 - a0 }`, wrapped as d3 does
  (`da % TAU + TAU` when negative). `da > TAU - 1e-6` draws a full
  circle ending at the start point; `da <= 1e-6` draws nothing more.
- Cubics of at most π/2 each, control distance `k = 4/3 · tan(θ/4) ·
  r`, signed by direction. Values convert to `f32` only when calling
  the builder.

`proto/rfc/glue/src/lib.rs` (69 lines) supplies the f64 → f32
conversions; its arc (which calls `Builder::arc`) and its stateless
`pointer` (no modifiers, no double clicks) are not lifted.

##### `pointer::Tracker`

```rust
pub struct Tracker { /* modifiers, pressed button, a press awaiting its
                        timestamp, the last stamped press, scale factor,
                        cursor-inside flag */ }
impl Tracker {
    pub fn new() -> Self;                     // also Default
    /// Divides ScrollDelta::Pixels, which iced passes in physical px.
    pub fn scale_factor(self, factor: f32) -> Self;
    pub fn translate(
        &mut self,
        event: &iced_core::Event,
        bounds: iced_core::Rectangle,
        cursor: iced_core::mouse::Cursor,
    ) -> Option<interact::Pointer>;
    /// A press waits for the next RedrawRequested to be timed:
    /// return `Action::request_redraw()` while this is true.
    pub fn wants_redraw(&self) -> bool;
}
```

| iced event | `Pointer` |
| --- | --- |
| `CursorMoved` inside `bounds`, or anywhere while a button is held | `Moved { x, y }` relative to `bounds` (drags continue outside, as d3's window listeners do) |
| `CursorMoved` leaving `bounds` with no button held; `CursorLeft` | `Left`, once |
| `ButtonPressed(b)` inside | `Pressed { x, y, button, modifiers }`; the press waits for a timestamp |
| `ButtonReleased(b)` of the held button, anywhere | `Released { button }` |
| `WheelScrolled { delta }` inside | `Wheel { x, y, delta, modifiers }`; `Lines` and `Pixels` both negated (iced and winit: positive y scrolls up; d3's `deltaY` is positive down); `Pixels` divided by the scale factor (`winit/src/conversion.rs:185–200` passes scroll deltas through unconverted, pixel deltas physical) |
| `Keyboard(ModifiersChanged(m))`, anywhere | `Modifiers(m)`, and remembered |
| `Window(RedrawRequested(t))` with a press waiting | stamps it with `t`; if the last stamped press used the same button, is ≤ 300 ms older and < 6 logical px away (iced's own rule, `core/src/mouse/click.rs:73–84`), returns `DoubleClicked { x, y }` |
| anything else | `None` |

Time comes only from `RedrawRequested`; the library never calls
`Instant::now`. `DoubleClicked` carries no modifiers; d3-zoom's
shift-double-click zooms out (`d3-zoom/src/zoom.js:303–314` reads
`shiftKey`), so `Zoom` reads the `Modifiers` events it receives. Apps
keep a `Tracker` in their `canvas::Program::State` and call
`tracker.translate(…)`. iced's `mouse::Button::{Left, Right, Middle,
Back, Forward, Other(u16)}` and `keyboard::Modifiers` (`shift()`,
`control()`, `alt()`, `logo()`) map onto `interact::{Button,
Modifiers}` with exhaustive matches, building modifiers as
`Modifiers::NONE.union(Modifiers::SHIFT)…`.

##### Headless pixel harness, `tests/main/headless.rs`

```rust
pub struct Harness<'a, Message> {
    ui: Option<UserInterface<'a, Message, iced::Theme, iced::Renderer>>,
    renderer: iced::Renderer,
    size: iced::Size,          // logical
    scale: f32,
    cursor: mouse::Cursor,
    bus: shell::Bus<Message>,
    wakes: Arc<AtomicUsize>,
    redraw: iced::window::RedrawRequest,
}
pub struct Shot { pub width: u32, pub height: u32, pub rgba: Vec<u8> }

impl<'a, Message> Harness<'a, Message> {
    /// None when ICED_TEST_BACKEND=wgpu has no device and
    /// RUSTD3_ALLOW_NO_GPU=1; panics otherwise.
    pub fn new(root: impl Into<iced::Element<'a, Message>>, size: iced::Size, scale: f32) -> Option<Self>;
    pub fn point_at(&mut self, at: iced::Point);
    pub fn events(&mut self, events: &[iced::Event]) -> Vec<event::Status>;
    /// `renderer.tick()`: on wgpu, drains the image upload worker.
    pub fn tick(&mut self);
    /// tick() → update(RedrawRequested(now)) → draw → screenshot.
    pub fn frame(&mut self, now: Instant) -> Shot;
    /// The redraw request of the last update.
    pub fn redraw_requested(&self) -> iced::window::RedrawRequest;
    pub fn waker(&self) -> shell::Waker;      // counts wake() calls
    pub fn wakes(&self) -> usize;
    pub fn messages(&mut self) -> Vec<Message>;
    pub fn backend(&self) -> String;          // Headless::name()
}
```

Signatures checked at the pinned revision: `UserInterface::build(root,
bounds: Size, Cache, &mut Renderer)`, `update(&mut self, &dyn Window,
&shell::Waker, &[Event], mouse::Cursor, &mut Renderer, &mut
shell::Bus<M>) -> (State, Vec<event::Status>)`, `draw(&mut self, &mut
Renderer, &Theme, &renderer::Style, mouse::Cursor)`
(`runtime/src/user_interface.rs:99,198,550`); `iced::window::Headless`
is the window (`core/src/renderer.rs:143–163`);
`Headless::new(Settings, Option<&str>)` is async (block on it as
`test/src/simulator.rs:66–77` does); `screenshot(Size<u32>, f32,
Color) -> Vec<u8>` returns RGBA at physical size `(logical ×
scale).round()`.

- **`tick`.** `renderer.tick()` is a `core::Renderer` method
  (`core/src/renderer.rs:96`). wgpu uploads large images on a worker
  and drains its results only there or in a frame that draws a raster
  image, so a harness that never ticks measures an artifact. `frame`
  ticks first.
- **`redraw_requested`** returns the `redraw_request` of the last
  update's `State::Updated` (`runtime/src/user_interface.rs:686–697`),
  and `RedrawRequest::Wait` after `State::Outdated`.
- **Waker.** The harness's waker counts wakes; it is never
  `Waker::noop()`.
- **Scale.** Before drawing, the harness calls
  `renderer.hint(renderer::Scale { window: scale, application: 1.0 })`
  so wgpu reports the factor; tiny-skia ignores hints and its
  `scale()` stays `None`.
- `matches_image`, `matches_hash` and `iced_test`'s snapshots are not
  used: they are exact-match, and the pixel bounds here are tolerant.

##### Placement tests, `tests/main/placement.rs`

A custom test widget (the `Widget` skeleton of
`proto/icedspike/src/bin/layerclip.rs`) draws RGBA
`image::Handle::from_rgba` images with `FilterMethod::Nearest` through
`image::Renderer::draw_image(image, bounds, clip_bounds)`, driven by
the harness at factors 1, 1.25, 1.5 and 2 on both backends. The file
declares the two constants the widget is built on:

```rust
/// An RGBA image of fewer bytes uploads inside the frame on wgpu.
pub const BAND_BYTES: usize = 2 * 1024 * 1024;
/// Device px added to an image's origin before dividing by the factor.
pub const NUDGE: f32 = 0.25;
```

Image bounds are `(device + NUDGE) / k` for x and y and `device_size /
k` for width and height.

- **`NUDGE`.** tiny-skia truncates the device position
  (`tiny_skia/src/raster.rs:63–64`); wgpu adds 0.001 and rounds
  (`wgpu/src/nudge.rs:43,72–74`). With a bias `b`, the device position
  a backend recomputes is `d + b ± ε`, ε being the f32 round-off of
  dividing by and multiplying back by `k`. Truncation lands on `d` for
  `0 ≤ b + ε < 1` and rounding for `b + 0.001 + ε < 0.5`. `b = 0` lands
  a pixel early on tiny-skia whenever ε < 0; `b = 0.5` lands a pixel
  late on wgpu (`d + 0.501` rounds up); `b = 0.25` sits a quarter pixel
  from every boundary on both.
- **`BAND_BYTES`.** wgpu uploads images under `MAX_SYNC_SIZE` (2 MiB)
  inside the frame (`wgpu/src/image/cache.rs:223–246`) and larger ones
  through its worker, whose results only `Cache::receive` drains
  (`cache.rs:283–330`), reached from `Renderer::tick`
  (`wgpu/src/lib.rs:709–712`) or from the next frame that draws any
  raster image (`cache.rs:200`). Phase 18 splits every cell image at or
  above `BAND_BYTES` into horizontal bands strictly under it, so every
  widget image shows in the frame that first draws it, with no
  `allocate_image`.

Tests:

- `images_land_on_device_pixels_at_every_factor`: a 2 × 2 grid of
  images, each 300 × 200 device px of known pixels (a checkerboard with
  a 1-px red border). Every image's red border lies exactly on the
  expected device rows and columns; max diff against the source pixels
  ≤ 1/255 on tiny-skia, ≤ 2/255 on wgpu; the test prints the `crisp`
  and `web-colors` state.
- `a_sub_2mib_image_shows_in_its_first_frame`: a 724 × 724 image
  (2,096,704 bytes) is visible in the frame that first draws it, on
  both backends.
- `a_larger_image_needs_a_tick`: a 1000 × 600 image (2,400,000 bytes)
  is absent from its first wgpu frame and present after
  `harness.tick()` plus one frame; on tiny-skia it is present at once.

The measured max diffs per backend and factor, with the feature state,
go into `.claude/decisions/phase-11.md` as the evidence behind the
iced pixel bounds (opaque ≤ 1/255 tiny-skia, ≤ 2/255 wgpu) that
Phases 18–21 assert against the PNG.

**Decisions this phase may meet**

- A placement row lands one device pixel off at some factor on one
  backend — do not tune `NUDGE` per backend or per factor (the widget
  places every cell and band with this one constant); record it as a
  decision with the failing rows and stop.
- The wgpu max diff exceeds 2/255 with iced's default `crisp` and
  `web-colors` — record the measured bound and feature state as a
  decision; the iced pixel tests of Phases 18–21 assert the recorded
  bound.

#### Steps

1. **build(iced): pin iced.** Root workspace dependencies and the three
   manifests above. The build adds the iced packages to `Cargo.lock`
   and nothing else changes in it. Verify exit criteria 1–2;
   `scripts/check.sh quick`.
2. **feat(iced): IcedPath.** `iced/src/path.rs`,
   `tests/main/path.rs`: lyon events through
   `canvas::Path::raw().iter()`, compared with `SvgPath` endpoints
   within 1e-3; the arc rows of Spot checks.
3. **feat(iced): pointer translation.** `iced/src/pointer.rs`,
   `tests/main/pointer.rs` (pure: events in, pointers out); the
   `Tracker` rows of Spot checks.
4. **test(iced): headless harness.** `tests/main/headless.rs` with
   `tick` and `redraw_requested`; `tests/main/canvas.rs`: a
   `canvas::Program` drawing a d3 line through `path()` and recording
   `Tracker` output, pixels checked on both backends.
5. **test(iced): image placement.** `tests/main/placement.rs` with the
   three tests above, run on both backends; record the measured diffs
   in `.claude/decisions/phase-11.md`.
6. **chore: iced tests in check.sh.** `full` runs `cargo test -p
   rustd3-iced` (and the dev crates' iced tests) once per
   `ICED_TEST_BACKEND` in `tiny-skia wgpu`. With the variable set iced
   does not fall back; a missing Vulkan device fails unless
   `RUSTD3_ALLOW_NO_GPU=1`, which skips wgpu with a printed message.
   `scripts/check.sh full`.

#### Exit criteria

1. `cargo tree -p rustd3-iced -e normal -i winit` reports that winit
   is not in the graph; `-i iced_wgpu` and `-i iced_tiny_skia` likewise.
2. `cargo check -p rustd3-iced --all-features` passes (debug).
3. `cargo test -p rustd3-iced` passes with
   `ICED_TEST_BACKEND=tiny-skia` and with `ICED_TEST_BACKEND=wgpu` (or
   `RUSTD3_ALLOW_NO_GPU=1` skips wgpu with a printed message).
4. Every Spot-checks row is a named test and passes: the arcs, the
   donut slice as one subpath, every `Tracker` row, the harness's
   canvas pixel test.
5. `cargo test -p rustd3-iced placement` passes on both backends;
   `.claude/decisions/phase-11.md` holds the measured placement diffs
   per backend and factor with the `crisp`/`web-colors` state.
6. `scripts/check.sh full` passes (the core harness count is
   unchanged: no core work).

#### Spot checks

| Input | Expected |
| --- | --- |
| `arc(0, 0, 10, 0, π/2, false)` on an empty `IcedPath` | 1 subpath, starts (10, 0), ends (0, 10) ±1e-3, one cubic |
| `move_to(0, 0); arc(0, 0, 10, 0, π/2, false)` | `Line` to (10, 0), then the arc, one subpath |
| `arc(0, 0, 10, 0, π/2, true)` | sweep −3π/2 in 3 cubics, passes (−7.071, −7.071) ±0.01 |
| `arc(0, 0, 10, 0, 2π, false)` | ends at (10, 0), 4 cubics, one subpath |
| move, outer arc, `line_to`, inner arc ccw, `close_path` (a donut slice) | exactly one `Begin` and one `End { close: true }` (iced's `Builder::arc` splits it into several) |
| `arc` with `r = 0` at (5, 5) | only a point at (5, 5) |
| `arc` with `r = −1` | nothing drawn |
| `WheelScrolled { Lines { x: 0, y: 1 } }` at (450, 150) inside | `Wheel { x: 450, y: 150, delta: Lines { x: -0, y: -1 }, .. }`; d3's `defaultWheelDelta` (`d3-zoom/src/zoom.js:34–36`) then gives +0.05 (zoom in) |
| `WheelScrolled { Pixels { y: 30 } }`, scale factor 1.5 | `Pixels { y: -20 }` |
| `ModifiersChanged(SHIFT)` then press Left at (10, 10) | `Modifiers(shift)`, then `Pressed { modifiers: shift, button: Left, .. }` |
| press at t, redraw t, press at t + 250 ms 3 px away, redraw | `DoubleClicked` at the second redraw; at t + 350 ms, none |
| press inside, move to x = −40, release outside | `Moved { x: -40, .. }`, `Released { button: Left }` |
| move out of bounds without a button; `CursorLeft` | `Left` once each time |
| harness, 320 × 240 at scale 2, canvas stroking `line()` of (0, 120) → (320, 120), width 2, black on white | screenshot 640 × 480; row 240 dark, row 200 white, on both backends; backend diff ≤ 64/255 |
| image at device x 125, factor 1.25 | bounds x = (125 + 0.25) / 1.25 = 100.2; red border in device column 125 on both backends |
| 724 × 724 RGBA = 2,096,704 B (< 2,097,152) | visible in its first frame, both backends |
| 1000 × 600 RGBA = 2,400,000 B | wgpu: absent in its first frame, present after a tick and one frame; tiny-skia: present at once |

#### Not in this phase

- The `Plot` widget and bands (18); the render worker (19); gestures
  (20); live data (21); the benchmark (22).
- Tiles, a render pool and an image cache: only the conditional
  Phase 31.
- `allocate_image` residency, atlas growth and per-image clip timing:
  the widget depends on none of them, so nothing measures them.
- `rustd3-figure` code of any kind (12 onward); `Zoom` and brush
  behaviour (10, 30); the use cases (32).

### Phase 12 — Drawing list and raster outputs

*M2 · builds on phases 1, 7 · about 2,300 lines, 7 commits · two sessions: cut after Step 4*

#### Goal

`rustd3-figure` gets the drawing list every output reads: seven
commands on kurbo and peniko types, per-point marker channels, parts, a
validating check, explicit per-command bounds and a text dump for
snapshots. On top of it: `path::Bez`, the SVG writer (glyph outlines,
deterministic ids, base64 PNG images), `Caps` and `lower` with
warnings as values, the vello_cpu rasterizer with render-an-area and
culling, PNG encoding, the tiny-skia fallback and the device-pixel
snapping helpers. Tests build lists by hand (no typst yet) and check
every backend against the others and the SVG through resvg.

#### Builds on

- Phase 1: the `figure/` package with its dependencies and features
  declared; `rustd3-fonts` with `DEJAVU_SANS`, `DEJAVU_SANS_BOLD`,
  `NEW_CM_MATH`, `ALL` (`Font::data() -> &'static [u8]`).
- Phase 2: testkit `pixels::Image::new(w, h, &rgba)` and
  `pixels::Diff::between(&a, &b, threshold)` with `max()`, `over()`,
  `share_over()`, `tile_score()` (worst 32-px tile); text snapshots
  failing on a missing reference unless `RUSTD3_BLESS=1`.
- Phase 3: `path::PathContext`, `path::SvgPath` (arc semantics).
- Phase 7: `color::Rgb` (converted into `peniko::Color`).

#### Design

##### Features

The figure crate's features are `fonts` (default), `raster`, `png`
(implies `raster`), `gif`, `pdf`, `tiny-skia` and `helpers`. This
phase uses `raster`, `png` and `tiny-skia`.

##### Modules

```text
figure/src/list.rs            List, list::stroke, re-exports, Send + Sync assertions
figure/src/list/command.rs    Command (#[non_exhaustive])
figure/src/list/clip.rs       Clip
figure/src/list/run.rs        GlyphRun, Glyph
figure/src/list/paints.rs     Paints: per-point paint table
figure/src/list/part.rs       Part, part::Id
figure/src/list/error.rs      list::Error
figure/src/list/check.rs      list validation
figure/src/list/bounds.rs     list::Bounds: per-command bounds in list px
figure/src/list/outline.rs    Outlines: glyph outline cache (ttf-parser)
figure/src/list/dump.rs       text dump for snapshots
figure/src/path.rs            Bez: PathContext into a kurbo BezPath
figure/src/svg.rs             write, Options, Text, CAPS; svg/{paint,number,base64}.rs
figure/src/lower.rs           Caps, lower, Lowered; lower/{glyphs,markers,sweep,clip,warning}.rs
figure/src/raster.rs          Image, Error (always); Renderer, Settings, Simd, render (raster)
figure/src/raster/vello.rs    the replay into vello_cpu
figure/src/raster/png.rs      encode (png)
figure/src/raster/skia.rs     tiny-skia fallback (tiny-skia)
figure/src/raster/snap.rs     device-pixel snapping helpers (always)
```

`lib.rs` re-exports `kurbo` and `peniko` (`pub use kurbo; pub use
peniko;`) so users name the same versions.

##### List types

```rust
pub struct List {
    /// Logical size in px; rasters multiply by their scale.
    pub size: kurbo::Size,
    pub fonts: Vec<peniko::FontData>,
    pub commands: Vec<Command>,
    pub parts: Vec<Part>,
}

#[non_exhaustive]
pub enum Command {
    Fill { path: BezPath, paint: Brush },
    Stroke { path: BezPath, style: kurbo::Stroke, paint: Brush },
    /// `shape` is centred on the origin at size 1.
    Markers {
        shape: BezPath,
        at: Vec<Point>,
        paint: Brush,
        /// Per-point uniform scale of `shape`; len == at.len().
        sizes: Option<Vec<f64>>,
        /// Per-point paints; when present, `paint` is unused.
        paints: Option<Paints>,
    },
    Glyphs(GlyphRun),
    Image { rect: Rect, image: peniko::ImageData, smooth: bool },
    Push { transform: Affine, clip: Option<Clip> },
    Pop,
}

pub enum Clip { Rect(Rect), Path(BezPath) }

pub struct Paints { pub table: Vec<Brush>, pub index: Vec<u32> }

pub struct GlyphRun {
    pub font: usize, pub size: f32, pub transform: Affine,
    pub fill: Brush, pub glyphs: Vec<Glyph>, pub text: Arc<str>,
}
pub struct Glyph { pub id: u32, pub x: f32, pub y: f32, pub advance: f32, pub text: Range<u32> }

pub struct Part {
    pub id: part::Id,
    /// Commands `start..end`, Push/Pop balanced inside.
    pub commands: Range<usize>,
    /// Union of the commands' bounds, in list px.
    pub bounds: Rect,
    /// FNV-1a 64 of what the part drew. The figure's list builder fills
    /// it from Phase 17; until then, and in hand-built lists, it is 0.
    pub hash: u64,
}
```

`Part` has its final shape here; no field is added later.
`part::Id` is an opaque `Copy + Eq + Hash + Ord` newtype over `u64`
with `part::Id::new(u64)`; the figure composes it (Phase 14).

`List` is `Send + Sync` (kurbo and peniko types, `Blob`, `Arc<str>`
all are), and `list.rs` asserts it at compile time, because Phase 19
moves `Arc<List>` to a worker thread:

```rust
const _: () = {
    fn assert<T: Send + Sync>() {}
    let _ = assert::<List>;
    let _ = assert::<Bounds>;
};
```

Strokes are always built with explicit caps and joins:
`list::stroke(width) -> kurbo::Stroke` returns butt caps, miter joins,
limit 4 (the SVG, tiny-skia and PDF defaults). kurbo's `Stroke::new`
defaults to round caps and joins, and a vello_cpu render with those
defaults differed from the other backends by up to 170/255 at axis
corners.

`list::check(&List) -> Result<(), list::Error>` checks: balanced
Push/Pop; glyph `font < fonts.len()`; `sizes` and `paints.index`
lengths equal `at.len()`; paint indices `< table.len()`; image data
length equals `width × height × 4`; parts ordered, non-overlapping,
balanced and in range (any `hash` is accepted); all coordinates
finite. `list::Error` variants carry the rejected value and the
command index. Every backend calls `check` first.

##### Bounds, an explicit value

```rust
/// Per-command bounds in list px, computed once per list.
pub struct Bounds(Vec<Option<Rect>>);
impl Bounds {
    pub fn of(list: &List, outlines: &mut Outlines) -> Bounds;
    pub fn get(&self, command: usize) -> Option<Rect>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Bounds are computed per command in list px through the Push transform
stack; `Push` and `Pop` have `None`. Fill: the path's bbox. Stroke:
the bbox inflated by `width / 2 × miter_limit` for miter joins,
`width / 2` otherwise. Markers: the union of the per-point shape bbox
× size. Glyphs: the union of glyph outline bboxes placed by `transform
× translate(x, y) × scale(size)`. Image: `rect`. A clip intersects the
bounds of everything inside it. `Part::bounds` is the union of
`Bounds::get` over the part's commands. `Outlines: Default`.

`List` keeps exactly its four public fields and no hidden cache: a
cache inside a struct whose fields anyone can mutate would silently
disagree with the commands after an edit. Callers that render one list
many times (Phase 18's cells and bands, Phase 19's worker, which
receives `Arc<List>` and `Arc<list::Bounds>` together) compute
`Bounds::of` once per list.

##### Dump

`list::dump(&List, dump::Options { glyphs: bool }) -> String`: one
line per command, numbers through the SVG number formatter, so dumps
are byte-stable. With `glyphs: false`, runs print font, size and text
only (for snapshots taken before typst shapes text). Parts are listed
after the commands; hashes are not dumped. Format:

```text
size 320x240 fonts 1 parts 2
0 fill M0,0L320,0L320,240L0,240Z #ffffff
1 push matrix(1 0 0 1 40 20) rect(0,0,260,180)
2 stroke M0,90L260,90 w1.5 butt miter 4 dash[6,3]@0 #4c72b0
3 markers n3 bbox(-1,-1,1,1) #dd8452 sizes[1,2,3] paints[2:0,1,0]
4 glyphs font0 11 "Wheel: RL" ids[58,75,72,72,79,29,3,53,47] matrix(1 0 0 1 10 200) #262626
5 image rect(0,0,16,16) 16x16 nearest
6 pop
part 0 1..7 bounds(40,20,300,200)
```

##### `path::Bez`

```rust
pub struct Bez { path: BezPath, current: Option<Point>, start: Option<Point> }
impl Bez { pub fn new() -> Self; pub fn finish(self) -> BezPath; }
impl PathContext for Bez { … }
pub fn bez(draw: impl FnOnce(&mut Bez)) -> BezPath;
```

The orphan rule (E0117) forbids implementing core's trait for
`BezPath`. Arcs follow d3 and `SvgPath` exactly: a negative radius
draws nothing; `line_to` the start only if it differs from the current
point by more than 1e-6, `move_to` if there is none; a full circle when
`da > τ − 1e-6`; nothing after the start point for `r == 0`; cubics
from `kurbo::Arc::new(center, (r, r), a0, sweep,
0.0).to_cubic_beziers(0.1, …)`. The prototype `Pen`
(`proto/figmodel/src/plot/draw.rs:216–283`) always emitted the
`line_to` (l. 263–267); `Bez` does not.

##### SVG writer

```rust
pub struct Options { pub text: svg::Text }          // Default: Outlines
pub enum Text { Outlines }                          // Phase 13 adds Elements
pub fn write(list: &List, options: &Options) -> Result<String, list::Error>;
pub const CAPS: Caps = Caps { glyphs: true, markers: true, clip_paths: true, sweep: false };
```

`write` takes a list that needs no lowering under `svg::CAPS`: a sweep
gradient returns `Err(list::Error::Unsupported { command, what })`.
Callers lower first; `fig.save_svg` (Phase 15) does both and returns
the warnings. Output: `<svg xmlns=… xmlns:xlink=… width height
viewBox>`; `<defs>` with one `<path id='g{font}-{glyph}'>` per glyph
used, at 1000 units/em, y down; runs as `<g transform='matrix(…)
scale(size/1000)' fill>` of `<use xlink:href='#g…' x y/>`. Ids are
assigned in command order — markers `m1, m2, …`, clips `c1, …`,
gradients `p1, …`, images `i1, …` — so output is deterministic.
Markers with `paints` group consecutive points by paint into `<g
fill>` runs; `sizes` become `<use transform='translate(x y) scale(s)'>`.
Images: `<image x y width height preserveAspectRatio='none'
[style='image-rendering:pixelated' when !smooth]
href='data:image/png;base64,…'/>`, through the `png` crate (always
on) and a hand-rolled base64 (standard alphabet, `=` padding, about
40 lines). Linear and radial gradients become `<linearGradient>` /
`<radialGradient gradientUnits='userSpaceOnUse'>`. Numbers: `{:.2}`
with trailing zeros and point trimmed, `-0` printed as `0`.

Lift from `proto/figmodel/src/plot/draw.rs`: `svg_d`, `svg_stroke`,
`svg_matrix`, `svg_paint` and the SVG skeleton (l. 330–470); drop
`Label`, `no_markers`, `census`. Outlined glyph defs at 1000 units/em
with `<use>` follow `proto/dlist/src/svg.rs`.

##### `Caps` and `lower`

```rust
pub struct Caps { pub glyphs: bool, pub markers: bool, pub clip_paths: bool, pub sweep: bool }
impl Caps { pub const ALL: Caps = Caps { glyphs: true, markers: true, clip_paths: true, sweep: true }; }
pub struct Lowered { pub list: List, pub warnings: Vec<lower::Warning> }
pub fn lower(list: &List, caps: Caps) -> Result<Lowered, list::Error>;

#[non_exhaustive]
pub enum Warning {
    /// A sweep gradient fill drawn as an embedded image (`raster` on).
    SweepRasterized { command: usize },
    /// Without `raster`: the sweep drawn in its middle stop's color.
    SweepFlattened { command: usize },
    /// A path clip on a backend without `clip_paths`: left in place
    /// and drawn unclipped by that backend.
    UnsupportedClip { command: usize },
}
```

- Glyph runs → one `Fill` per run (outline union, `transform ×
  translate(x, y) × scale(size)` per glyph; a space has no outline).
- Markers → one `Fill` per point, scaled by `sizes`, painted from
  `paints`.
- Sweep fills → `Push { clip: Path(path) }`, an `Image` of the path's
  bbox rendered by vello_cpu at 2 px per list px, `Pop`
  (`SweepRasterized`). Without `raster` the sweep becomes a solid fill
  in its middle stop's color (`SweepFlattened`): a degraded but
  readable figure beats an error in a build that left raster out.
- Path clips are never cut in geometry. For `Caps { clip_paths: false,
  .. }` the `Push { clip: Some(Clip::Path(_)) }` stays in place and
  `UnsupportedClip { command }` is reported. The clip pass runs after
  the sweep pass, so a rasterized sweep on such a backend reports both.
  No built-in backend lacks clip paths (tiny-skia clips with masks),
  so only a custom backend sees this warning.
- Parts are remapped to the new command ranges.

Lowering is list in, list out, so it is tested by pixel diff against
its input (`proto/dlist/src/lower.rs`: `Caps`, `Stats`).

##### Raster

`raster::{Image, Error, snap}` are always compiled (the tiny-skia
fallback and the snapping helpers need them); `Renderer`, `Settings`,
`Simd` and `render` need `raster`; `png` needs `png`; `skia` needs
`tiny-skia`.

```rust
pub enum Simd { Detect, Baseline }
/// Default: threads 0 (single-threaded), Simd::Detect.
pub struct Settings { pub threads: u16, pub simd: Simd }
impl Settings {
    /// vello_cpu's own pool: available parallelism − 1, clamped to 1..=8.
    pub fn parallel() -> Settings;
}
/// Straight (un-premultiplied) RGBA, row-major.
pub struct Image { pub width: u32, pub height: u32, pub rgba: Vec<u8> }
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{width} × {height} device px exceeds 65,535")] TooLarge { width: f64, height: f64 },
    #[error("empty area {0:?}")] Empty(Rect),
    #[error(transparent)] List(#[from] list::Error),
}
pub struct Renderer { /* RenderContext, Resources, image cache keyed by Blob id */ }
impl Renderer {
    pub fn new(settings: Settings) -> Self;
    /// `area` in list px; the image is exactly
    /// round(area.width × scale) × round(area.height × scale) device px.
    pub fn render(&mut self, list: &List, bounds: &list::Bounds, area: Rect, scale: f64)
        -> Result<Image, Error>;
}
/// The whole list with Settings::default(); computes Bounds::of itself.
pub fn render(list: &List, scale: f64) -> Result<Image, Error>;
```

- **Threads.** `Settings::threads` is passed verbatim as vello_cpu's
  `RenderSettings::num_threads`
  (`vello_cpu-0.3.0/src/render.rs:170–190`): `0` is single-threaded
  (vello_cpu builds no pool), `n > 0` is vello_cpu's own pool of `n`
  threads. `Settings::parallel()` is the value vello_cpu's own
  `RenderSettings::default` computes (`render.rs:183–187`); file
  outputs (Phases 15, 37) and the widget's single worker (Phase 19)
  use it. Tests pin `Settings { threads: 0, simd: Simd::Baseline }`
  for determinism.
- **Area contract.** `area` stays in list px and the output is
  `round`, not `ceil`, of `area × scale`, so a caller that passes
  `area = device_rect / scale` gets the device rect's size back (204.8
  × 1.25 → 256). Above 65,535 device px on either side: `TooLarge`;
  zero width or height: `Empty`.
- **Replay** (`proto/dlist/src/vcpu.rs`). One `Renderer` keeps one
  `RenderContext` (resized with `reset_and_resize`) and one
  `Resources`. The base transform is `scale(s) × translate(−area.x0,
  −area.y0)`. Commands whose bounds miss `area` inflated by one device
  px are skipped, markers per point (`proto/dlist/src/bin/partial.rs:15–130`);
  `render` `debug_assert_eq!(bounds.len(), list.commands.len())`.
  Transparent clear: the figure's background is a command.
  `Pixmap::take_rgba8(ImageAlphaType::Alpha)` returns straight alpha.
  Image commands: `ImageData` → premultiplied `Pixmap` cached by
  `Blob::id`, painted with `fill_rect(rect)` and a paint transform
  mapping the image onto `rect`, nearest sampling when `!smooth`.
- `raster::png::encode(&Image) -> Result<Vec<u8>, Error>` (feature
  `png`; RGBA 8-bit, default compression).
- `raster::skia::render(&List, scale: f64) -> Result<Image, Error>`
  (feature `tiny-skia`): computes `Bounds::of`, lowers with `Caps {
  glyphs: false, markers: true, clip_paths: true, sweep: true }`, then
  replays as `proto/dlist/src/skia.rs` does (path clips through
  `Mask::intersect_path`), with linear, radial and sweep gradients
  (`tiny_skia::SweepGradient::new(center, start, end, stops, mode,
  transform)` takes degrees; peniko's sweep angles are radians,
  clockwise in y-down) and images.

Crate APIs checked: kurbo 0.13.1 `Stroke { width, join, miter_limit,
start_cap, end_cap, dash_pattern, dash_offset }`; peniko 0.6.1
`Brush`, `ImageData { data: Blob<u8>, format, alpha_type, width,
height }`, `FontData { data: Blob<u8>, index: u32 }`; vello_cpu 0.3.0
`RenderContext::new_with(u16, u16, RenderSettings { level,
num_threads })`, `reset_and_resize`, `set_transform`, `set_paint`,
`set_stroke`, `fill_path`, `stroke_path`, `fill_rect`,
`push_clip_layer`, `pop_layer`, `glyph_run(&mut Resources,
&FontData).font_size(f32).hint(false).fill_glyphs(…)`, `flush`,
`render(&mut Pixmap, &mut Resources)`, `PaintType =
peniko::Brush<Image, Gradient>` with `ImageSource::Pixmap(Arc<Pixmap>)`,
`vello_cpu::Level::{try_detect, baseline}`; ttf-parser 0.25
`Face::parse(&[u8], u32)`, `outline_glyph(GlyphId(u16), &mut dyn
OutlineBuilder) -> Option<Rect>`, `units_per_em() -> u16`; resvg
0.48.1 (dev) `render(&usvg::Tree, tiny_skia::Transform, &mut
PixmapMut)`, `usvg::Tree::from_str`. The outline cache
(`proto/dlist/src/outline.rs`) is rewritten on ttf-parser, as
`proto/typstspike/src/main.rs:92–128` does.

##### Snapping helpers, `raster::snap` (always compiled)

```rust
/// A plot-area edge on the device-pixel grid.
pub fn edge(x: f64, scale: f64) -> f64;            // (x·s).round() / s
/// Device width of a hairline: at least one device px.
pub fn width(w: f64, scale: f64) -> f64;            // max(1, round(w·s)) / s
/// A line centre such that its device width covers whole pixels.
pub fn line(x: f64, w: f64, scale: f64) -> f64;     // device width odd: (floor(x·s) + 0.5) / s; even: round(x·s) / s
```

The parity in `line` is that of the snapped device width `max(1,
round(w·s))`. Vector targets never call these; Phase 17 applies them
for raster targets.

**Decisions this phase may meet**

- A kitchen-sink command that vello_cpu and tiny-skia draw more than
  64/255 apart — first establish which one departs from resvg's render
  of the SVG; if it is a backend limitation rather than a replay
  mistake, record it as a decision with the diff statistics instead of
  raising the bound.

#### Steps

1. **feat(figure): list types.** `list.rs`,
   `list/{command,clip,run,paints,part,error,check}.rs`,
   `list::stroke`, the `Send + Sync` assertion; a test for every
   `check` error.
2. **feat(figure): Bez, bounds, outlines, dump.** `path.rs`,
   `list/{bounds,outline,dump}.rs`; the kitchen-sink list in
   `tests/main/sink.rs` (glyph ids from ttf-parser's cmap, advances
   from `glyph_hor_advance`); snapshot `list_dump_of_the_kitchen_sink`.
3. **feat(figure): SVG writer.** `svg.rs`, `svg/{paint,number,base64}.rs`;
   snapshot `svg_of_the_kitchen_sink`; base64 against the standard
   test vectors (`""`, `"f"` → `Zg==`, … `"foobar"` → `Zm9vYmFy`).
4. **feat(figure): vello raster and PNG.** `raster.rs`,
   `raster/{vello,png,snap}.rs`; area, culling, shift and size tests;
   `bands_rendered_with_one_bounds_match_the_whole_render` (one
   `Bounds` value, six horizontal-band renders stitched together equal
   the whole render within 1/255). *Cut point: a second session starts
   at Step 5.*
5. **feat(figure): lower.** `lower.rs`, `lower/*`; command-count and
   pixel tests; `lower_leaves_path_clips_in_place_without_clip_paths`
   (lowers with `Caps { clip_paths: false, .. }`, asserts
   `UnsupportedClip` and the untouched `Push`).
6. **feat(figure): tiny-skia fallback.** `raster/skia.rs`; agreement
   tests.
7. **test(figure): SVG through resvg.** The resvg dev-dependency test;
   `scripts/check.sh full`.

Snapshots fail on a missing reference: create them once with
`RUSTD3_BLESS=1` after reading the output, and say in the commit body
why it is right. `figure/Cargo.toml` has `autotests = false` and
`[[test]] name = "main", path = "tests/main/main.rs"`; test files are
`figure/tests/main/{main,sink,list,svg,raster,lower,skia,resvg}.rs`
and `figure/tests/snapshots/*.txt`.

#### Exit criteria

1. `cargo test -p rustd3-figure`, `--features raster,png` and
   `--all-features` pass under `TZ=Asia/Kathmandu`.
2. Snapshot tests `list_dump_of_the_kitchen_sink` and
   `svg_of_the_kitchen_sink` pass against checked-in text references.
3. Raster agreement tests pass: vello_cpu vs tiny-skia max ≤ 64/255;
   resvg on our SVG vs vello_cpu tile score ≤ 0.02; `lower` (glyphs,
   markers, sweep) vs its input no channel off by more than 16;
   whole-device-pixel shifts ≤ 1/255; an area render equals the same
   pixels of the full render within 1/255;
   `bands_rendered_with_one_bounds_match_the_whole_render` within 1/255.
4. `Renderer::render` returns `Error::TooLarge` above 65,535 device px,
   `Error::Empty` for a zero size, and exactly `round(area × scale)`
   device px otherwise.
5. `lower_leaves_path_clips_in_place_without_clip_paths` passes;
   `List` and `list::Bounds` are `Send + Sync` (the compile-time
   assertion in `list.rs`).
6. `scripts/check.sh full` passes.

#### Spot checks

| Input | Expected |
| --- | --- |
| `bez(\|p\| { p.move_to(0,0); p.arc(0,0,10,0,π/2,false) })` | `M0,0 L10,0 C… 0,10`, one subpath |
| `bez(\|p\| p.arc(0,0,10,0,τ,false))` | starts and ends at (10, 0) |
| SVG of `Fill { Rect(0,0,10,10) → path, #4c72b0 }` | `<path d='M0,0L10,0L10,10L0,10Z' fill='#4c72b0'/>` |
| numbers 1/3, 2.5, −0.001, 10.0 | `0.33`, `2.5`, `0`, `10` |
| two `Markers`, one path clip, in that order | ids `m1`, `m2`, `c1` |
| base64 of `"foobar"`, `"fo"` | `Zm9vYmFy`, `Zm8=` |
| `lower` with `markers: false` on 10 commands holding 3 `Markers` of 10, 20, 30 points | 10 − 3 + 60 = 67 commands (the wheel figure: 468 − 12 + 5,600 = 6,056) |
| `lower` with `Caps { clip_paths: false, .. }`, path clip at command 1 | command 1 unchanged; warnings `[UnsupportedClip { command: 1 }]` |
| `raster::render` of a 70,000 × 10 list at 1.0; 40,000 × 10 at 2.0 | `TooLarge { width: 70000, .. }`; `TooLarge { width: 80000, .. }` |
| `render` of area (0, 0, 204.8, 100) at 1.25 | 256 × 125 px |
| area (0, 0, 100, 100) vs (8, 0, 108, 100) at 2.0 | overlap identical within 1/255 (a half-pixel shift differs by up to 109/255; see iced integration) |
| vello_cpu vs tiny-skia on the kitchen sink at 2.0 | max ≤ 64/255 |
| resvg(our SVG) vs vello_cpu at 2.0 | tile score ≤ 0.02 |
| `lower(Caps { glyphs: false, markers: false, sweep: false, .. })` vs input | no channel off by more than 16 (the wheel figure lowers within 7/255) |
| `snap::edge(10.3, 1.5)`, `snap::edge(10.5, 1.25)` | 10.0 (15.45 → 15), 10.4 (13.125 → 13) |
| `snap::line(10.3, 1.0, 1.0)`, `(10.3, 1.0, 2.0)`, `(10.3, 1.0, 1.25)` | 10.5 (width 1, odd), 10.5 (width 2, even: 21 / 2), 10.0 (width 1, odd: 12.5 / 1.25) |
| DejaVu Sans cmap for "Wheel: RL" (ttf-parser, upem 2048) | glyph ids 58, 75, 72, 72, 79, 29, 3, 53, 47; advances 2025, 1298, 1260, 1260, 569, 690, 651, 1423, 1141 (W at 11 px: 10.877 px) |
| `list::check` with Push, Push, Pop | `Err(Unbalanced { open: 1 })` |
| Markers `at.len() == 3`, `sizes.len() == 2` | `Err(ChannelLength { command, expected: 3, found: 2 })` |

Context numbers reached in later phases, not asserted here (see
Drawing and output): the wheel figure has 468 commands; its SVG is
344 KB with outlines and 306 KB with `<text>`; PNG at 2× takes
18–20 ms with vello_cpu against 61–66 ms with tiny-skia.

#### Not in this phase

- typst and `GlyphRun`s from text, SVG `<text>` mode (13).
- `save_svg` and `save_png` on figures (15); part hashes and cells (17).
- Bands (18); the render worker (19); tiles and a render pool only in
  the conditional Phase 31.
- PDF and GIF (37); hatching (35).
- Geometric clip cutting and an anyrender adapter are not in v0.1
  (see Later).

### Phase 13 — Text with typst

*M2 · builds on phase 12 · about 1,300 lines, 6 commits*

#### Goal

All figure text goes through typst 0.15.1. The figure crate gets a
typst `World` over `rustd3-fonts` plus user fonts; `Text` values built
from spans (`span("RL").fill(RED)`) or raw markup
(`text::typst("$m slash s^2$")`); explicit `\n` line breaks; one
batched typst document per set of new labels; a caller-owned
`text::Cache` bounded by generations, with process-wide comemo
eviction; conversion of typst frames into the list's `GlyphRun`s (font
table, positions, source text) plus fills for math rules, with
ink-sized boxes and the baseline read from the first text item;
missing-glyph counts; and the SVG writer's opt-in `<text>` mode.

#### Builds on

- Phase 12: `list::{List, Command, GlyphRun, Glyph}`,
  `list::Outlines`, `svg::{write, Options, Text}`, `raster::render`
  (pixel checks), `path::Bez`.
- Phase 7: `color::Rgb` (span fills).
- Phase 1: `rustd3_fonts::{DEJAVU_SANS, DEJAVU_SANS_BOLD, NEW_CM_MATH,
  ALL}` with `Font::data() -> &'static [u8]`.

#### Design

##### Modules

```text
figure/src/text.rs           Text, span(), typst(), re-exports
figure/src/text/span.rs      Span
figure/src/text/style.rs     Style (size px, bold, fill, align)
figure/src/text/anchor.rs    Anchor, anchor::{Horizontal, Vertical}
figure/src/text/fonts.rs     Fonts: faces, families, Library, FontBook
figure/src/text/world.rs     the private World: one value per batch
figure/src/text/key.rs       Key: the label's full typst snippet
figure/src/text/batch.rs     one document per batch, per-label fallback
figure/src/text/convert.rs   frame → runs and fills, ink box, baseline
figure/src/text/shaped.rs    Shaped, and placing it into a List
figure/src/text/cache.rs     Cache, Stats
figure/src/text/error.rs     text::Error
```

`svg.rs` gains the `<text>` mode in this phase.

##### Text values

```rust
pub struct Text { pieces: Vec<Piece> }        // Piece: Span | Markup, private
pub struct Span { text: String, fill: Option<Rgb>, bold: bool }
pub fn span(text: impl Into<String>) -> Span;
pub fn typst(markup: impl Into<String>) -> Text;   // raw markup and math
impl Span { pub fn fill(self, c: Rgb) -> Self; pub fn bold(self) -> Self; }
impl Text { pub fn push(self, piece: impl Into<Text>) -> Self; pub fn is_empty(&self) -> bool; }
impl From<&str> / From<String> / From<Span> for Text; From<&str> / From<String> for Span
```

Markup emitted per span: `#"lit"`, wrapped as `#strong("lit")` when
bold and `#text(fill: rgb("#rrggbb"), …)` when filled (alpha below 1:
`rgb(r, g, b, a)` with 0–255 integers). Escapes inside literals: `\` →
`\\`, `"` → `\"`, newline → `\n`, tab → `\t`, carriage return dropped.
So `$`, `#` and `_` from user data stay literal; only `\` and `"` need
escaping inside a typst string. Raw markup is inserted verbatim. Lift
`Text`, `Span`, `span`, `Text::typst` from
`proto/figmodel/src/plot/text.rs` and add the markup pieces and newline
escaping.

##### Fonts and the World

```rust
pub struct Fonts { /* LazyHash<Library>, LazyHash<FontBook>,
                      faces: Vec<(typst::text::Font, peniko::FontData)>, families */ }
impl Fonts {
    /// DejaVu Sans, DejaVu Sans Bold, New Computer Modern Math; built once.
    pub fn bundled() -> Arc<Fonts>;                    // OnceLock
    /// User fonts first in the fallback order, then the bundled ones.
    pub fn with(extra: impl IntoIterator<Item = Vec<u8>>) -> Result<Fonts, text::Error>;
}
```

Bundled bytes come from `rustd3_fonts::ALL` (`Font::data()`), wrapped
with `typst::foundations::Bytes::new` (a `&'static [u8]` qualifies, no
copy). Family names in documents are the ones typst reports
(`Font::info().family`: "DejaVu Sans", "New Computer Modern Math",
measured), not `rustd3_fonts::Font::family()`, which may spell the
math face without spaces. Each face's `FontData` is built once
(`Blob::new(Arc::new(bytes))`) so its blob id is stable across draws;
vello's glyph caches and part hashes depend on it. Every document's
family fallback list: user families, `"DejaVu Sans"`, `"New Computer
Modern Math"`. `Fonts::with` rejects bytes that hold no face, with an
error carrying their position in `extra`.

The World is one plain value per batch: `struct World { fonts:
Arc<Fonts>, main: Source }`, with no interior mutability and no lock.
`typst::World` is `Send + Sync`
(`typst-library-0.15.1/src/lib.rs:60`) and `compile` takes `&dyn
World` (`typst-0.15.1/src/lib.rs:74`), so building one per batch is
both possible and cheap: `Fonts` holds the hashed `Library` and
`FontBook`, and a batch costs one `Source::new`. typst's memoization
still hits inside the new document. `main` is the batch source,
`source` serves only it, `file` returns `FileError::AccessDenied`,
`today` returns `None` (no clock). The `World` impl follows
`proto/dlist/src/shape.rs:31–63`, with bundled fonts instead of system
paths and typst-assets.

##### Keys and batches

```rust
pub struct Style { pub size: f64 /* px */, pub bold: bool, pub fill: Rgb, pub align: anchor::Horizontal }
pub struct Key(Arc<str>);       // the label's typst snippet; Eq + Hash
impl Key { pub fn new(text: &Text, style: &Style) -> Key; }
```

Sizes are px: typst's `pt` is used as a plain unit, so `size:
14.6667pt` yields geometry in px (the theme converts points, 1 pt =
4/3 px, in Phase 14). The snippet is `#align(<start|center|end>,
text(size: <s>pt, weight: "<regular|bold>", fill: rgb(r, g, b,
a))[<content>])` with `s` printed to 4 decimals. A batch document:

```text
#set page(width: auto, height: auto, margin: 0pt)
#set text(font: (<families>), top-edge: "bounds", bottom-edge: "bounds")
<snippet 0>
#pagebreak()
<snippet 1>
…
```

`top-edge` and `bottom-edge` `"bounds"` size the box to the ink;
typst's default box ends at the baseline and clips descenders. If
compilation fails, or the page count differs from the label count (raw
markup may contain `#pagebreak()`), every label of the batch is
compiled alone; a label that fails or yields ≠ 1 page becomes
`Error::Markup { markup, message }` / `Error::Pages { markup, pages }`
and is cached as that error, so it is not retried every frame.

##### Conversion of one page frame

Positions in px, y down (`proto/typstspike/src/main.rs:132–220` walks
frames with group transforms and keeps shapes;
`proto/dlist/src/shape.rs` ignored group transforms):

- Walk items with the accumulated `Affine` (`translate(pos) × group
  transform`). `Text` → one run: `font` = the face index in `Fonts`,
  `size` = `TextItem.size`, origin = the item's position; glyphs with
  `x = pen + x_offset·size`, `y = −y_offset·size`, `advance =
  x_advance·size`, `text = range` (bytes of the item's text); `fill`
  from `Paint::Solid` (other paints: the style's fill). Glyph id 0
  counts as missing.
- `Shape` → a `Fill` (lines as thin rectangles of the stroke thickness,
  `Rect`, `Curve`); `Image`, `Link`, `Tag` are ignored.
- Box: width = frame width (the advance width; edges do not change
  it); `ascent` = baseline y of the first text item
  (`Frame::baseline()` is unset and returns the height); `descent` =
  height − ascent. Run origins and fills are stored relative to that
  first baseline (frame position − (0, ascent)).

typst API used (checked at 0.15.1):
`typst::compile::<PagedDocument>(&dyn World) ->
Warned<SourceResult<PagedDocument>>`;
`typst_layout::PagedDocument::pages() -> &[Page]`, `Page { frame, ..
}`; `FrameItem::{Group, Text, Shape, Image, Link, Tag}`; `TextItem {
font: FontInstance, size: Abs, fill: Paint, stroke, lang, region,
text: EcoString, glyphs: Vec<Glyph> }`; `Glyph { id: u16, x_advance,
x_offset, y_advance, y_offset: Em, range: Range<u16>, span }`;
`FontInstance::font() -> &Font`; `Font::iter(Bytes)`, `Font::data() ->
&Bytes`, `Font::index() -> u32`; `FontBook::from_fonts`;
`Library::builder()` via `typst::LibraryExt`.

```rust
pub struct Shaped { /* runs, fills, width, ascent, descent, missing */ }
impl Shaped {
    pub fn width(&self) -> f64;
    pub fn ascent(&self) -> f64;
    pub fn descent(&self) -> f64;
    /// Glyphs typst drew as .notdef (fig.check reports them).
    pub fn missing(&self) -> usize;
    /// Ink box relative to the anchor point, before rotation.
    pub fn bounds(&self, anchor: Anchor) -> kurbo::Rect;
    /// Appends runs and fills; maps faces into list.fonts by blob id.
    pub fn place(&self, list: &mut List, at: kurbo::Point, anchor: Anchor, angle: f64);
}
pub struct Anchor { pub h: anchor::Horizontal, pub v: anchor::Vertical }
// anchor::Horizontal: Start | Middle | End;  anchor::Vertical: Baseline | Top | Middle | Bottom
```

Placement transform: `translate(at) × rotate(angle) × translate(dx,
dy)` with `dx ∈ {0, −w/2, −w}` and `dy` = 0 (baseline), `+ascent`
(top), `(ascent − descent)/2` (middle), `−descent` (bottom); `angle` in
radians, clockwise on screen.

##### Cache

```rust
pub struct Cache { /* Arc<Fonts>, HashMap<Key, Entry>, generation, stats */ }
pub struct Stats { pub entries: usize, pub compiles: u64, pub hits: u64, pub misses: u64 }
impl Cache {
    pub fn new(fonts: Arc<Fonts>) -> Self;            // Default uses Fonts::bundled()
    /// Starts a figure draw: generation += 1, drops entries unused for
    /// 32 generations; every 32nd call per process runs comemo::evict(32).
    pub fn begin(&mut self);
    /// Shapes every key not cached, in one document; marks all used.
    pub fn prepare<'k>(&mut self, keys: impl IntoIterator<Item = &'k Key>);
    pub fn get(&self, key: &Key) -> Option<Result<&Shaped, &text::Error>>;
    pub fn stats(&self) -> Stats;
}
```

The process-wide cadence is a `static DRAWS: AtomicU64`, one of the
recorded process-wide exceptions. `typst::comemo::evict(32)`
(`typst-0.15.1/src/lib.rs:33` re-exports comemo;
`comemo-0.5.1/src/memoize.rs:93–105`: "removes all results whose age ≥
max_age; age grows by one per eviction and resets on a hit") is
process-global: with two `Plot`s, or a `Plot` beside file output, the
32-draw cadence counts every `Cache::begin` in the process, so eviction
runs up to N times as often for each. `begin`'s doc comment says so;
it is accepted, because eviction only ages out entries unused for 32
evictions. Entries are `Arc<Shaped>` so list building clones cheaply.
The cache is caller-owned and lives inside `figure::Cache` (Phase 14);
the widget keeps that in its tree state, `fig.draw` makes a temporary
one. This replaces the prototype's `Rc`/`RefCell` measurer
(`proto/dlist/src/measure.rs`: keyed by markup, size and weight, with
`prefetch` batching).

##### SVG `<text>` mode

`svg::Text::Elements`: each run becomes `<text
transform='matrix(…)' x='x0 x1 …' y='0' font-family='…'
font-size='…' [font-weight='bold'] fill='…'>escaped text</text>`, one
x per glyph when glyphs map one to one onto characters, else a single
x (`proto/figmodel/src/plot/draw.rs:477–500`, `text_run`). Family and
weight come from the face's `name` table (`TYPOGRAPHIC_FAMILY`, else
`FAMILY`) and `Face::is_bold()` through ttf-parser. Outlines stay the
default: they are exact, while a viewer re-shapes `<text>` (resvg's
re-shaped render differed by up to 255/255 on 0.3% of pixels).

**Decisions this phase may meet**

- typst reports warnings for a label (none does for any spot-check
  label, measured) — keep the messages on `Shaped` for `fig.check`;
  record it as a decision.
- The 4-decimal size in the key splits the cache by float noise —
  quantize sizes to 1/100 px; record it as a decision.

#### Steps

1. **feat(figure): Text and spans.** `text.rs`,
   `text/{span,style,anchor}.rs`; markup tests (escapes, fills, bold,
   raw).
2. **feat(figure): fonts and world.** `text/{fonts,world,error}.rs`;
   bundled faces load once; `Fonts::with` rejects bytes with no face.
3. **feat(figure): batch shaping.** `text/{key,batch,convert,shaped}.rs`;
   width, ascent, run and missing-glyph spot checks.
4. **feat(figure): text cache.** `text/cache.rs`; generation and
   compile-count tests.
5. **feat(figure): place shaped text.** `Shaped::place`; snapshot
   `list_dump_of_labels`; a vello render shows ink at the anchors.
6. **feat(figure): SVG text elements.** `svg.rs`; snapshot
   `svg_text_elements_of_labels`; the resvg comparison;
   `scripts/check.sh full`.

Tests live in `figure/tests/main/text.rs`; snapshots in
`figure/tests/snapshots/{list_dump_of_labels,svg_text_elements_of_labels}.txt`.

#### Exit criteria

1. `cargo test -p rustd3-figure` and `--all-features` pass; every
   Spot-checks row is a named test.
2. A second `prepare` of the same keys compiles nothing
   (`Cache::stats().compiles` unchanged); 33 `begin`s without using a
   key evict it.
3. A batch with one invalid markup label still shapes the others; the
   bad one yields `text::Error::Markup` carrying its markup.
4. Snapshots `list_dump_of_labels` (dump with glyph ids) and
   `svg_text_elements_of_labels` pass; resvg renders the `<text>` SVG
   with at most 1% of pixels more than 16/255 off the outlined SVG.
5. `scripts/check.sh full` passes.

#### Spot checks

Measured with typst 0.15.1 and the bundled fonts, size 11 px, black,
through exactly the batch template above (`#align(center, text(…))`;
plain `#box[…]` gives the same numbers):

| Label | Expected |
| --- | --- |
| `"0123456789"` | width 69.985 (0.636 em per digit), height 8.320, 1 run |
| `"x"` / `"g"` | x: width 6.510, ascent 6.016, descent 0; g: width 6.982, ascent 6.161, descent 2.288 |
| `"line one\nline two"`, centered | 2 runs at baselines 8.357 and 24.021, the second at x 0.215; width 43.817; height 24.176 |
| `Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)))` | markup `#"Wheel: "#text(fill: rgb("#c44e52"), "RL")`; 2 runs, widths 41.642 and 13.771, the second filled (196, 78, 82) |
| `"a $ # _ b"` | literal: 1 run, text `a $ # _ b`, width 49.425 |
| `"vibration RMS [m/s²]"` | width 113.137 (a 0.6 em-per-character estimate gives 132, +17%) |
| `"WWW"` / `"iii"` | 32.629 / 9.168 (the 0.6 em estimate gives 19.8 for both: −39% / +116%) |
| `text::typst("RMS [$m slash s^2$]")` | 6 runs in 2 faces (DejaVu Sans, New Computer Modern Math), superscript size 7.70, width 61.510 |
| `"漢字"` | 1 run of 2 glyphs with id 0; `missing() == 2`; no typst warning |
| `text::typst("$ x^ $")` in a batch with `"ok"`, `"fine"` | `Error::Markup` for it; the other two shaped |
| `"Wheel: RL"` glyph ids (DejaVu Sans) | 58, 75, 72, 72, 79, 29, 3, 53, 47 |
| the same 12 labels compiled twice | typst memoizes the second compile (22 µs, release); our cache makes zero compiles |
| 40 labels `"1000.0"`, `"1037.0"`, … at 11 px | one document 1.8 ms vs one each 6.0 ms (release, 2-core container; reported, not asserted) |
| `Shaped::bounds` of `"g"` with `Anchor { h: Middle, v: Top }` | `Rect(−3.491, 0, 3.491, 8.449)` (±6.982 / 2; 6.161 + 2.288) |

Context (release, 2-core container; Phase 22's benchmark reports
them): first label cold 2.3–6.7 ms; 40 new tick labels in one document
1.7–2.8 ms against 4.6–5.2 ms one document each; the same 40 again
0.1–0.2 ms; the wheel figure's 65 labels → 66 runs of 371 glyphs in
two fonts in 5.5 ms (0.8 ms on repeat).

#### Not in this phase

- Figure layout, tick labels, the theme's sizes, `figure::Cache` and
  `draw_with` (14).
- Collisions (16) and `fig.check` (17); rotated tick labels (33); PDF
  text (37).
- mitex, automatic wrapping and a plain-text fast path are out of
  scope.

### Phase 14 — Figure model and first layout

*M2 · builds on phases 6, 7, 9, 10, 13 · about 2,600 lines, 7 commits · two sessions: cut after Step 4 (14a / 14b)*

#### Goal

A `Figure<'a>` owns a table of scales and hands out typed `Copy`
handles; panels in grid cells hold marks and axes that refer to scales
only through handles. `resolve` fits each fitted handle's domain to
the data of every mark that uses it — merged in the scale's own domain
type, padded 5% through core's `scale::Pad`, with the degenerate-input
rules — honours handles a gesture holds, measures every label through
the text cache in one batch, and lays the grid out by protrusions:
each panel's reach per side, maxima per track, fixed reaches and gaps,
the rest split by weight, positions as running sums. `Fit<Span>`,
`Fittable`, `Target`, `figure::Cache` and the final `resolve` and
`draw_with` signatures are defined here. Figure axes (`bottom`,
`left`, `right`, `top`) are values placed on panels; decorations
(backgrounds, borders, axes, titles) are drawn into a `List` with
parts. The default theme is seaborn's ticks style at notebook size
with the colorblind palette.

#### Builds on

- Phase 6: `locale::Locale`, `ticks::Set`, core's `Ticks` (`Domain`,
  `Step`, `tick_set(count)`, `position(v, offset)`, `labels(&set,
  &locale)`, `extent()`); core axes' geometry conventions.
- Phase 7: `scale::{Band, Point, Ordinal}`, `color::Rgb`,
  `scheme::COLORBLIND`.
- Phase 9: `series::{Data, Points}` (used from Phase 15; the `Mark`
  trait must not preclude them).
- Phase 10: `interact::Rescale` with `pan_px(dx)` and
  `zoom_px(anchor, factor)`; `scale::Pad` for every continuous scale.
  Read their exact signatures in `core/src/interact/rescale.rs` and
  `core/src/scale/pad.rs`.
- Phase 12: `list::{List, Command, Part, part::Id, Bounds, Outlines}`,
  `list::stroke`.
- Phase 13: `text::{Text, Style, Key, Cache, Fonts, Shaped, Anchor}`.

#### Design

##### Modules

```text
figure/src/figure.rs            Figure<'a>, figure::Id
figure/src/figure/held.rs       Held: domains a gesture holds
figure/src/figure/fits.rs       Fits: typed extents and sticky values
figure/src/figure/cache.rs      figure::Cache, cache::Stats
figure/src/figure/resolve.rs    the resolve algorithm
figure/src/figure/resolved.rs   Resolved
figure/src/figure/draw.rs       draw_with
figure/src/figure/part.rs       part::Id composition for figure parts
figure/src/figure/decorate.rs   backgrounds, borders, axes, titles → List
figure/src/target.rs            Target
figure/src/panel.rs             Panel<'a>
figure/src/mark.rs              Mark trait, mark::Ctx (marks themselves: Phase 15)
figure/src/scale.rs             Scale (object-safe), re-exports
figure/src/scale/handle.rs      ScaleRef<S>
figure/src/scale/kind.rs        Kind
figure/src/scale/position.rs    Position
figure/src/scale/paint.rs       Paint
figure/src/scale/fit.rs         Fit<Span>
figure/src/scale/fittable.rs    Fittable
figure/src/scale/indexed.rs     Indexed<D>, SeriesExtent<D>: series data through generic marks
figure/src/scale/invert.rs      Invert
figure/src/scale/{continuous,band,point,ordinal}.rs   impls
figure/src/axis.rs              Axis<'a, S>, bottom/left/right/top
figure/src/axis/{draw,labels}.rs   the erased axis, tick-label boxes
figure/src/layout.rs            Grid, Length, em, px
figure/src/layout/{tracks,protrusion}.rs
figure/src/theme.rs             Theme (Default only)
```

##### Handles and the table

```rust
pub struct Figure<'a> { /* id, scales: Vec<Entry<'a>>, panels, grid, title, theme */ }
impl<'a> Figure<'a> {
    pub fn new() -> Self;                                   // also Default
    /// A scale whose domain is the one it was built with.
    pub fn scale<S: Kind>(&mut self, s: S) -> ScaleRef<S>;
    /// A scale fitted to the data of every mark that uses it: fit_with(s, Fit::All).
    pub fn fit<S: Fittable>(&mut self, s: S) -> ScaleRef<S>;
    pub fn fit_with<S: Fittable>(&mut self, s: S, fit: Fit<S::Span>) -> ScaleRef<S>;
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel<'a>;
    pub fn layout(&mut self, grid: Grid) -> &mut Self;
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self;
    pub fn theme(&mut self, t: Theme) -> &mut Self;
    pub fn resolve(&self, size: kurbo::Size, cache: &mut figure::Cache,
                   held: &Held, target: Target) -> Resolved;
    pub fn draw_with(&self, cache: &mut figure::Cache, size: kurbo::Size,
                     target: Target, held: &Held) -> List;
}
pub struct ScaleRef<S> { figure: figure::Id, index: u32, kind: PhantomData<fn() -> S> }  // Copy, Eq
impl<S> ScaleRef<S> {
    /// Position in the scale table (the widget keys state by it and the TypeId).
    pub fn index(self) -> usize;
}
```

These are the final signatures of `resolve` and `draw_with`; later
phases add methods beside them (`draw`, `resolve_held`, `draw_held`)
and never change them. `figure::Id` comes from a `static NEXT:
AtomicU64`. Every handle use goes through one `fn entry(&self, h) ->
&Entry` that `debug_assert!`s the figure id with the message "handle
from another figure"; release builds skip the check. Lifetime branding
of handles was considered and rejected (see Figures): it infects every
signature that stores a handle.

##### Fit modes and fittable kinds

```rust
/// How a fitted handle's domain follows its data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit<Span> {
    /// Every value of every mark that uses the handle.
    All,
    /// The last `span` of x (live data). Implemented in Phase 21;
    /// resolves as `All` until then.
    Window(Span),
    /// y fitted to the data inside the visible x range. Implemented in
    /// Phase 17; resolves as `All` until then.
    Visible,
}

/// A scale whose domain the figure can fit to its marks' data.
pub trait Fittable: Kind {}
```

`Fittable` bounds `fit` and `fit_with` from the start; this phase
implements it for every kind it builds (`Continuous<T, f64, N>`,
`Band<K>`, `Point<K>`, `Ordinal<K, Rgb>`), and Phase 33 implements it
for its kinds except `Quantile` without changing any signature. Only
`All` is implemented here: `Visible` and `Window` resolve exactly as
`All`, and their doc comments say so (no `todo!()`). `Window(())` on a
categorical handle is meaningless; `fit_with` documents that and
`debug_assert!`s against it.

##### Capability traits

```rust
/// Object-safe face for the table; delegates gestures to core.
pub trait Scale: Any {
    fn as_any(&self) -> &dyn Any;
    fn boxed(&self) -> Box<dyn Scale>;
    fn with_range(&self, range: [f64; 2]) -> Box<dyn Scale>;
    /// Rescale::pan_px; None for scales without it (band, ordinal).
    fn pan_px(&self, range: [f64; 2], dx: f64) -> Option<Box<dyn Scale>>;
    /// Rescale::zoom_px around `anchor` px by `factor`.
    fn zoom_px(&self, range: [f64; 2], anchor: f64, factor: f64) -> Option<Box<dyn Scale>>;
}
pub trait Kind: Scale + Clone {
    type Domain: Clone + 'static;
    type Output;
    /// What `Fit::Window` measures: f64 for numeric scales,
    /// jiff::SignedDuration for time (Phase 33), () for categorical.
    type Span: Copy + 'static;
    /// Finite and valid under the transform (log: > 0).
    fn admits(&self, v: &Self::Domain) -> bool;
    /// Widens `extent` by `v` in the domain's own order.
    fn merge(extent: &mut Option<[Self::Domain; 2]>, v: &Self::Domain);
    /// This scale with its domain fitted (see Fitting).
    fn fitted(&self, extent: Option<&[Self::Domain; 2]>, sticky: &[Self::Domain], margin: f64) -> Self;
    /// The end stops of a continuous domain, in its own type (what
    /// `Plot::on_change` publishes, Phase 20); None for band, point, ordinal.
    fn domain(&self) -> Option<[Self::Domain; 2]>;
}
pub trait Position: Kind<Output = f64> {
    fn position(&self, v: &Self::Domain) -> Option<f64>;
    fn bandwidth(&self) -> f64 { 0.0 }
    /// The bridge to core's indexed recordings (below); `None` for
    /// scales whose domain is not a `series::Coordinate`.
    fn indexed(&self) -> Option<&dyn Indexed<Self::Domain>> { None }
}
pub trait Paint: Kind<Output = Rgb> {
    fn paint(&self, v: &Self::Domain) -> Option<Rgb>;
    /// The key's slot among all keys (for `.dodge()`).
    fn slot(&self, v: &Self::Domain) -> Option<(usize, usize)> { None }
}
```

##### `Indexed` and `Invert`: series data through generic marks

Marks are generic over `X: Position`, while core's indexed recordings
exist only for `X::Domain: series::Coordinate` (Phase 9's
`Points::series()` carries that bound). Generic mark code cannot name
the bound, so the x scale supplies the bridge: `Position::indexed`
returns it for scales whose domain is a `Coordinate`, and `None` for
band, point and ordinal scales, whose marks draw owned points as given.

```rust
// figure/src/scale/indexed.rs
/// What a mark needs to fit and thin `Series` data on this x scale.
/// Object-safe; implemented for `Continuous<T, f64, N>` here and for
/// time scales in Phase 33.
pub trait Indexed<D> {
    /// The domain value at output position `px` (the scale's inverse).
    fn invert(&self, px: f64) -> D;
    /// For a `Series` view: the first and last x and the y extremes,
    /// under `valid`, of the samples whose x lies in `x` (all samples
    /// for `None`), read from the index in O(log n). `None` when
    /// `points` is not a `Series` view.
    fn series_extent<'a>(
        &self,
        points: &Points<'a, D, f64>,
        x: Option<[D; 2]>,
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<SeriesExtent<D>>;
    /// For a `Series` view: M4 over the columns between consecutive
    /// `edges` (domain values in increasing order). `None` when `points`
    /// is not a `Series` view or an edge is not a valid coordinate.
    fn thin<'a>(
        &self,
        points: &Points<'a, D, f64>,
        edges: &[D],
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<series::m4::Lines>;
}

/// What `Indexed::series_extent` reads from a recording.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesExtent<D> {
    /// First and last x of the samples considered.
    pub x: [D; 2],
    /// The y extremes; `None` when no sample is valid.
    pub y: Option<[f64; 2]>,
}

// figure/src/scale/invert.rs
/// A position scale with a typed inverse, checked at compile time:
/// brushes (Phase 36) accept only these. Continuous scales here, time
/// scales in Phase 33; never band, point or ordinal.
pub trait Invert: Position {
    /// `None` when the result is not finite.
    fn invert(&self, px: f64) -> Option<<Self as Kind>::Domain>;
}
```

The continuous impl of `Indexed<f64>` calls `points.series()`
(available because `f64` is a `Coordinate`), turns `edges` into
`series::Columns` with `recording.columns(edges)` and runs Phase 9's
`m4` and `extremes` (with `recording.locate` for the x range). Columns
built in the same call from the same view belong to its recording, so
the `Stale` error is an invariant (`expect("invariant: columns built
from this series")`). Users: Phase 15 fits and thins through
`indexed()`, Phase 17's `Fit::Visible` passes the visible x range to
`series_extent`, Phase 21's `Fit::Window` reads the last x from it,
Phase 34 adds a defaulted `thin_joint` for ribbons, and Phase 36's
brushes bound on `Invert`. Unit tests: `indexed_extent_reads_the_index`
(a 1M-sample ramp: extent equals a full scan, all samples and the range
[0.25, 0.5] of the x span), `indexed_is_none_for_band_scales`,
`invert_round_trips_position` (linear and log, 7 points each, within
1e-12 relative).

`pan_px` and `zoom_px` set the panel's range on a copy and call core's
`Rescale::pan_px(dx)` / `zoom_px(anchor, factor)` (transformed space,
`None` when degenerate), so each scale keeps its own precision.
Implementations in this phase:

- `Continuous<T, f64, N>` for every `T` and `N` that core gives
  `Rescale` and `Pad`: `Kind` (`Span = f64`), `Position`, `Fittable`;
  axes use core's `Ticks`.
- `Band<K>` and `Point<K>` (`K: PartialEq + Clone + 'static`): `Kind`
  (`Span = ()`), `Position`, `Fittable`; `fitted` returns `self`.
- `Ordinal<K, Rgb>`: `Kind` (`Span = ()`), `Paint`, `Fittable`;
  `fitted` returns `self`.

Time, sequential and binned handles are Phase 33. Axes require `S:
Position + core::scale::Ticks<Domain = <S as Kind>::Domain>`. Both
traits name an associated `Domain`, so a bare `S::Domain` is ambiguous
(E0221): figure code writes `<S as Kind>::Domain`. An axis on a
`Band<K>` whose `K` lacks `Display` fails with E0277 (see Figures).
`proto/figmodel/src/plot/scales.rs` has the trait split; its
`span`/`with_span`/`invert` are replaced by the delegation to core.

##### Fitting

`Fits` collects, per fitted handle, an `Option<[S::Domain; 2]>`
(boxed `Any`, downcast by the handle's type) and a list of sticky
values. Marks call `fits.add(h, &v)` (skips `!admits(v)`) and
`fits.stick(h, v)` (bars and areas at their base). `Kind::fitted` for
continuous scales is one call to core's `scale::Pad::pad(extent,
margin, sticky)`, which implements matplotlib's `handle_single_axis`
in the transform's space:

1. no admitted value → `[0, 1]`, or `[inverse(0), inverse(1)]` where 0
   or 1 is invalid (log₁₀: `[1, 10]`);
2. flat extents → `nonsingular` with expander 0.05 (in transformed
   space for log);
3. sticky values within a 1e-5 tolerance stop the margin;
4. margins of `margin` × span per side (the theme's 0.05).

The figure adds only the merge, the admission filter and the choice of
margin; it never re-implements padding. No `Timestamp` passes through
`f64`: merging uses the domain's own order. Held handles skip fitting.

```rust
pub struct Held { /* HashMap<u32, (TypeId, Box<dyn Scale>)> */ }   // Default
impl Held {
    pub fn hold<S: Kind>(&mut self, h: ScaleRef<S>, scale: S);
    pub fn release<S: Kind>(&mut self, h: ScaleRef<S>);
    pub fn get<S: Kind>(&self, h: ScaleRef<S>) -> Option<&S>;   // None on TypeId mismatch
    /// Drops entries whose index or type does not match the figure.
    pub fn prune(&mut self, fig: &Figure<'_>);
}
```

##### Target, cache and `Resolved`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// SVG, PDF: list px, no snapping.
    Vector,
    /// iced: device px = list px × scale.
    Screen { scale: f64 },
}

pub struct Cache { text: text::Cache }          // figure::Cache
pub struct Stats { pub text: text::Stats }       // figure::cache::Stats
impl Cache {
    pub fn new(fonts: Arc<text::Fonts>) -> Self;  // Default: Fonts::bundled()
    /// Starts a figure draw (text::Cache::begin).
    pub fn begin(&mut self);
    pub fn stats(&self) -> Stats;
}

impl Resolved {
    /// The resolved scale behind a handle, before any panel's range.
    pub fn scale<S: Kind>(&self, h: ScaleRef<S>) -> &S;
    /// Plot-area rects in list px, one per panel in panel order.
    pub fn plot_areas(&self) -> &[kurbo::Rect];
}
```

Phase 15 adds `Target::File { scale }` (the only later change to
`Target`, and an additive one) and the M4 memo to `figure::Cache`;
Phase 17 adds the `Fit::Visible` memory. In this phase `Target` is
stored in `Resolved` and otherwise ignored; Phase 16 reserves label
slack for `Screen` and Phase 17 snaps raster targets. `Resolved` also
keeps per panel the tick sets, label boxes and the target.
`Resolved::scale` lifts the prototype's `raw` (`proto/figmodel/src/plot/mod.rs`).

##### Panels, axes, marks

```rust
impl<'a> Panel<'a> {
    pub fn mark(&mut self, m: impl Mark<'a>) -> &mut Self;
    pub fn axis<S>(&mut self, a: impl Into<Axis<'a, S>>) -> &mut Self
        where S: Position + core::scale::Ticks<Domain = <S as Kind>::Domain>;
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self;
    pub fn border(&mut self, c: Rgb) -> &mut Self;
}
pub trait Mark<'a> {
    fn fit(&self, fits: &mut Fits);
    fn draw(&self, ctx: &mark::Ctx<'_>, list: &mut List);
}
// mark::Ctx: x<S: Position>(h) -> S over [0, w], y<S: Position>(h) -> S over [h, 0],
// raw<S: Kind>(h) -> S, rect() -> kurbo::Rect, theme() -> &Theme, scale_factor() -> f64.
```

`Axis<'a, S>` (lift `proto/figmodel/src/plot/axis.rs`: builder,
`origin`, `labels`, `draw`, with typst boxes): `bottom/left/right/top(h)`,
`.values(iter)`, `.format(impl Fn(&<S as Kind>::Domain) -> String +
'a)`, `.ticks(n)`, `.label(text)`, `.outward(px)`, `.color(palette:
ScaleRef<C>, key: C::Domain)` for `C: Paint`, `.labels(bool)`.
Closures are `Rc<dyn Fn + 'a>`, never `'static` boxes: the figure
stays on the UI thread (only `List` crosses threads). `From<&Axis<'a,
S>>` clones, so one axis value is placed on many panels. Tick count in
this phase: `.ticks(n)` or d3's default 10 (Phase 16 derives it from
length). Values: `.values(…)` or `tick_set(count)`; labels: `.format`
per value, else core's `Ticks::labels(&set, &Locale::default())` (the
figure locale is Phase 17). Each axis sits at its `.outward(px)` or 0;
automatic stacking, inner-axis dropping and collisions are Phase 16.

##### Layout (one protrusion pass)

Per label: `text::Key::new(text, style)`; all keys of the figure go to
the text cache's `prepare` once, before any measurement. Per panel and
side:

- axis thickness = tick (8 px) + tick pad (4.667) + label extent (max
  width for left/right, max ink height for top/bottom) +, with a title,
  label pad (5.333) + title ink height (rotated −90° on the left, +90°
  on the right);
- a panel title adds title pad (8) + its ink height on top;
- tick labels that overhang the panel's ends add to the perpendicular
  sides (every reach past the plot area counts; see Layout).

Tracks (`layout::tracks`, lifted from
`proto/figmodel/src/plot/arrange.rs:262–293`; `rects` and `tracks` at
l. 226–309 as they are): each column takes the largest left and right
reach of its panels, each row the largest top and bottom; `fixed = Σ
reaches + Σ gaps`; `free = max(0, total − fixed)` split by weight;
positions are running sums, so panels of one column share `x` and `w`
exactly. Outer padding 0.8 em on every side; the figure title reserves
its ink height + 8 px on top. Gaps: `em(x)` is `x ×` the tick-label
size (14.667 px at notebook), `px(x)` exact; a missing gap defaults to
`em(1.0)`. `Grid` holds rows, columns, weights and gaps; Phase 16 adds
spans; nested grids are not in v0.1 (see Later).

##### Theme (default only)

`Theme` has private fields and `Default`; crate-internal getters in
px: tick labels 11 pt = 14.667, axis labels and titles 12 pt = 16,
figure title 14.4 pt = 19.2, `em` = 14.667, tick 6 pt = 8, tick pad
3.5 pt = 4.667, label pad 4 pt = 5.333, title pad 6 pt = 8, axis
width 1.25 pt = 1.667, line 1.5 pt = 2, marker diameter 6 pt = 8, ink
`#262626` (seaborn's ".15"), panel and figure `#ffffff`, edge = ink, no
grid, margin 0.05, palette core's `scheme::COLORBLIND` (seaborn 0.13.2
colorblind `#0173b2 #de8f05 #029e73 #d55e00 #cc78bc #ca9161 #fbafe4
#949494 #ece133 #56b4e9`, checked), referenced, not copied. Builders
are Phase 35. `proto/figmodel/src/plot/theme.rs` has the getters.

##### Parts and drawing

`figure/part.rs` composes `part::Id::new(role << 56 | panel << 32 |
index)` with roles figure background, figure title, panel background,
border, axis, panel title (and mark and legend, used from Phases 15
and 17). Parts carry `hash: 0`; Phase 17's list builder computes the
real hash. `draw_with` calls `cache.begin()`, resolves, and emits the
decorations only — figure background, panel backgrounds, borders,
axes, titles, each wrapped in a part whose bounds are the union of
`list::Bounds` over its commands. Marks join in Phase 15.

**Decisions this phase may meet**

- A fitting row disagrees with matplotlib beyond the rows marked as
  differing — padding is `scale::Pad`'s (Phase 10), not the figure's;
  record it as a decision with the matplotlib values rather than
  patching the figure's merge.

#### Steps

1. **feat(figure): handles and scale traits.** `scale.rs`,
   `scale/{handle,kind,position,paint,fit,fittable,indexed,invert}.rs`,
   the impls; the E0277 `compile_fail` doctest for an axis on
   `Band<NoDisplay>` with its passing twin; the `Indexed` and `Invert`
   unit tests.
2. **feat(figure): figure, panels, Held.** `figure.rs`,
   `figure/{held,part}.rs`, `panel.rs`, `mark.rs`; the foreign-handle
   panic test.
3. **feat(figure): fitting.** `figure/fits.rs`; the matplotlib rows
   through an in-crate test mark, also run with `Fit::Visible` and
   `Fit::Window` (which resolve as `All`).
4. **feat(figure): theme defaults.** `theme.rs`. Test
   `default_theme_sizes` (the `em(1.6)` row: 23.467 px at notebook,
   tick-label size 14.667 px). *Cut point (14a ends here; 14b continues
   at Step 5).*
5. **feat(figure): axes.** `axis.rs`, `axis/{draw,labels}.rs`. Test
   `figure_axis_ticks_follow_core_axis_positions` (a bottom axis on the
   single-panel row's handle: tick positions equal core's
   `Ticks::position` plus the plot area's x, labels "0" … "10").
6. **feat(figure): protrusion layout.** `layout.rs`,
   `layout/{tracks,protrusion}.rs`, `target.rs`,
   `figure/{cache,resolve,resolved}.rs`; the track and single-panel
   rows.
7. **feat(figure): decorations.** `figure/{decorate,draw}.rs`; snapshot
   `decorations_of_one_panel` through `draw_with(&mut cache, size,
   Target::Vector, &Held::default())`; `scripts/check.sh full`.

Tests live in `figure/tests/main/{figure,fit,layout}.rs`; the snapshot
in `figure/tests/snapshots/decorations_of_one_panel.txt`.

#### Exit criteria

1. `cargo test -p rustd3-figure` passes, every Spot-checks row a named
   test.
2. In a debug build a handle from another figure panics with "handle
   from another figure"; `cargo test -p rustd3-figure --release fit`
   passes with the check compiled out.
3. Attached panels share pixel ranges exactly: in a 2 × 2 grid, panels
   of one column have bit-identical `x` and `w` in
   `Resolved::plot_areas()`, panels of one row `y` and `h`.
4. Snapshot `decorations_of_one_panel` (list dump without glyph ids)
   passes.
5. The E0277 doctest fails to compile and its twin compiles.
6. `scripts/check.sh full` passes.

#### Spot checks

Fitting a linear y handle with 5% margins; matplotlib 3.11.2
`get_ylim` for the same data (`ax.plot`, `ax.bar`) agrees except where
noted:

| Data | Domain |
| --- | --- |
| line 5, 5 | [4.725, 5.275] (nonsingular [4.75, 5.25], then ±0.025) |
| line 0, 0 | [−0.055, 0.055] |
| line 0.13 … 9.71 | [−0.349, 10.189] |
| line 0, 3 | [−0.15, 3.15] (lines are not sticky) |
| line −2, 8 | [−2.5, 8.5] |
| line 1e9, 1e9 + 1 | [999999999.95, 1000000001.05] |
| bars 3, 7 (sticky 0) | [0, 7.35] |
| NaN, 4 | [3.78, 4.22] |
| no data | [0, 1] (matplotlib gives ±0.055) |
| log scale, 1 … 1000 | [0.70795, 1412.54] (10^−0.15, 10^3.15) |
| log scale, −1, 10, 1000 | [7.9433, 1258.93] (−1 not admitted; 10^0.9, 10^3.1) |
| held `[2, 4]` over data 0 … 10 | [2, 4]; after `release`, [−0.5, 10.5] |
| line 0, 3 with `Fit::Visible`, and with `Fit::Window(5.0)` | [−0.15, 3.15], as `All` |
| `fit_with(band, Fit::Window(()))` | panics in a debug build |

Layout:

| Input | Expected |
| --- | --- |
| `layout::tracks` weights [1, 0.25, 1, 0.25], gaps [4, 16, 4], left reaches [94, 0, 5, 0], right [0, 0, 0, 0], start 8, total 944 (the wheel figure's columns with 10 px text) | fixed 123, free 821, 328.4 per unit; x = 102, 434.4, 537.5, 869.9; widths 328.4, 82.1, 328.4, 82.1; FL's side panel to FR's scatter 16 + 5 = 21 px (see Layout) |
| `em(1.6)` at notebook | 23.467 px |
| 640 × 480, one panel, `left(y)`, `bottom(x)`, x fitted to 0 … 10, y to 0 … 3, no titles | x domain [−0.5, 10.5], ticks 0 … 10 ("0" … "10"); y [−0.15, 3.15], ticks 0.0 … 3.0 step 0.5 ("0.0" is 23.325 px wide, digit labels 11.093 px tall); left reach 8 + 4.667 + 23.325 = 35.992; bottom reach 8 + 4.667 + 11.093 = 23.760; rect x 47.725, y 11.733, w 580.541, h 432.773 (±0.01) |
| same, 2 × 2 grid with shared handles | column panels bit-identical `x`, `w`; row panels `y`, `h` |
| `fig.title("Hub vibration and bearing temperature")` | measured 289.273 × 14.194 at 14.667 px; at 19.2 px it scales by 19.2 / 14.667 |

#### Not in this phase

- Marks, `draw`, `save_*`, `Target::File` and the M4 memo (15).
- Tick counts from length, outer-only labels, stacking, collisions,
  spans, offset notation, held layout (16).
- Legends, the figure `Locale`, part hashes, hit testing, raster
  snapping, `fig.check`, `Fit::Visible` (17).
- `zoom_x` / `zoom_y` on panels (20); `Fit::Window` and live data (21);
  tiles only in the conditional Phase 31.
- Time, sequential and binned handles, rotated labels (33); seaborn
  style and context builders, `Domains::Nice` (35).
- Nested grids are not in v0.1 (see Later).

### Phase 15 — Marks and file outputs

*M2 · builds on phases 8, 14 · about 1,500 lines, 7 commits*

#### Goal

Figures draw. Four marks — `dots`, `line`, `rule_x`, `rule_y` — take
data through core's `series::Data` (owned points, borrowed `Series`
views with their index, or borrowed slices), fit their handles, and
draw into the list as parts. Line marks thin long recordings at draw
time with M4 per column (one column per device pixel on screen, four
for files, `Thin::Off` to opt out), with column boundaries from the x
scale's `invert` at device-pixel edges and M4 output memoized per
recording, revision, channel and column grid in `figure::Cache`.
`Target` gains `File { scale }`; figures gain `draw`, `save_svg` and
`save_png`; figure tests gain `figures_equal`; `dev/examples` gains the
trace and scatter figures that later phases and the benchmark reuse.

#### Builds on

- Phase 8: `shape::line()` and `Curve` (unthinned lines).
- Phase 9: `series::{Recording, Series, Data, Points, Columns,
  slices}`; `recording.columns(&edges)`, `series.m4(&columns, valid)`
  with outer neighbours inside the same NaN-free run, the validity
  predicate, `series.extremes(range, valid)`, ids and revisions. Read
  `core/src/series/*.rs` for exact names.
- Phase 12: `list::*`, `path::Bez`, `lower`, `svg::{write, CAPS}`,
  `raster::{Renderer, Settings, Simd, png::encode}`, `list::Bounds`.
- Phase 13: `text::{Cache, Shaped::place}`.
- Phase 14: `Figure<'a>`, `ScaleRef`, `Kind`/`Position`/`Paint`,
  `Fittable`, `Fit<Span>`, `Mark<'a>`, `mark::Ctx`, `Fits`, `resolve`,
  `draw_with`, `decorate`, `Target`, `figure::Cache`, parts.

#### Design

##### Modules

```text
figure/src/mark.rs            Mark, Ctx (Phase 14), re-exports
figure/src/mark/dots.rs       Dots<'a, X, Y>, dots()
figure/src/mark/line.rs       Line<'a, X, Y>, line()
figure/src/mark/rule.rs       Rule<'a, S>, rule_x(), rule_y()
figure/src/mark/color.rs      Color<'a>: a fixed Rgb or (palette, key)
figure/src/mark/guard.rs      f64 clipping to the guard band
figure/src/thin.rs            Thin, column grids
figure/src/thin/frame.rs      Frame: per-draw column-boundary cache
figure/src/thin/memo.rs       Memo: M4 output by (recording, revision, channel, grid)
figure/src/figure/draw.rs     draw, draw_with
figure/src/figure/save.rs     save_svg, save_png, figure::Error
dev/examples/src/trace.rs     trace figure (2 panels, 3 channels)
dev/examples/src/scatter.rs   scatter figure
dev/examples/examples/{trace,scatter}.rs   thin mains
```

##### Marks

```rust
pub fn dots<'a, X: Position, Y: Position>(
    data: impl Data<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Dots<'a, X, Y>;
impl<'a, X: Position, Y: Position> Dots<'a, X, Y> {
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self;
    pub fn color(self, c: Rgb) -> Self;
    pub fn r(self, px: f64) -> Self;            // default: theme marker diameter / 2
    pub fn opacity(self, a: f64) -> Self;
}

pub fn line<'a, X: Position, Y: Position>(
    data: impl Data<'a, <X as Kind>::Domain, <Y as Kind>::Domain>, x: ScaleRef<X>, y: ScaleRef<Y>,
) -> Line<'a, X, Y>;
impl<'a, X: Position, Y: Position> Line<'a, X, Y> {
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self;
    pub fn color(self, c: Rgb) -> Self;
    pub fn width(self, px: f64) -> Self;        // default 1.5 pt = 2 px
    pub fn dash(self, pattern: impl IntoIterator<Item = f64>) -> Self;
    pub fn curve(self, c: Curve) -> Self;       // used only when not thinned
    pub fn thin(self, t: Thin) -> Self;         // Thin::Auto (default) | Thin::Off
}

pub fn rule_x<'a, X: Position>(value: <X as Kind>::Domain, x: ScaleRef<X>) -> Rule<'a, X>;
pub fn rule_y<'a, Y: Position>(value: <Y as Kind>::Domain, y: ScaleRef<Y>) -> Rule<'a, Y>;
impl<'a, S: Position> Rule<'a, S> { color, width, dash, label(impl Into<Text>) }
```

`.dash([6.0, 3.0])` works because arrays are `IntoIterator`. Strokes:
butt caps, round joins (matplotlib's lines use round joins; caps are
always explicit), dash offset 0 at the first drawn point. Default
colors: the theme palette's first color for an unkeyed mark;
`.fill`/`.stroke` look the key up through `Paint::paint` at draw time
(an unknown key draws the theme's gray `#949494`, and `fig.check`
reports it from Phase 17). Draw order is insertion order. Lift
`Dots` (`proto/figmodel/src/plot/marks.rs:59–125`), `Line`
(`:297–356`) and `RuleY` (`:358–413`), replacing owned `Vec`s with
`Points` and `Rc<dyn Fn(&Ctx)>` fills with typed `(ScaleRef<C>,
C::Domain)` resolved through `mark::Ctx`.

**Fitting.** Owned points add every value (`Fits::add` skips
non-admitted ones); a `Series` view is fitted through the x scale's
bridge, `x.indexed()?.series_extent(&points, None, &|v| y.admits(&v))`
(Phase 14), so fitting 1M samples costs an index read, not a scan, and
its x extent is the clock's first and last value; slices iterate; rules add their value. Every handle this phase
fits has `Span = f64`; `Fit::Visible` and `Fit::Window` still resolve
as `All`.

**Drawing**, in panel coordinates (`mark::Ctx`), inside the panel's
`Push { transform: translate(x, y), clip: Some(Clip::Rect(plot area))
}`:

- `dots` → one `Command::Markers { shape: circle of radius r at the
  origin, at, paint, sizes: None, paints: None }` per mark; points
  whose position is `None`, non-finite, or farther than `r` outside the
  plot area are dropped.
- `line` → one `Command::Stroke` per call: thinned (below) or core's
  `shape::line().curve(c)` through `path::Bez`; `None` positions break
  the line.
- rules → a `kurbo::Line` across the plot area; the label is placed
  with `text::Shaped::place` at the right end (`rule_y`) or top end
  (`rule_x`), anchored outside the line.

**Guard band.** Before building paths, segments are clipped in `f64`
(Liang–Barsky) to the plot area inflated by its own width and height
on every side, so no coordinate handed to a backend exceeds 3× the
panel size; a segment wholly outside is dropped and breaks the
polyline. Clipping happens before anything converts to `f32`.

##### Thinning

`Thin::Auto` thins a line whose x scale has the bridge
(`x.indexed()` is `Some`: continuous scales, and time scales from
Phase 33) and whose data is a `Series` view (the bridge's `thin`
returns `Some`). Owned
points and `series::slices` are drawn as given: core has no M4 for
unindexed data (Phase 9 keeps its raw scan as a test reference), and
rebuilding an index per frame was rejected (see Figures); `fig.check`
reports owned marks over 100,000 points (Phase 17).

1. **Columns.** `n = round(plot width × scale × k)` with `k = 1` for
   `Target::Screen`, `k = 4` for `Target::Vector` (scale 1) and
   `Target::File`. Edges are `indexed.invert(edge_i)` for device-pixel
   edges `edge_i = i / (scale × k)`, `i = 0..=n` (reversed for a
   decreasing scale so they increase), so log or time x get the right
   columns; inside the bridge `recording.columns(&edges)` turns them
   into `series::Columns` (a sample index per edge, carrying the recording
   id and revision). Column boundaries by `partition_point` follow
   `proto/thin2/src/main.rs:46–54`.
2. **Shared boundaries.** `Columns` are cached per draw in
   `thin::Frame`, keyed by (recording id, revision, the edge values'
   bits), so lines on one clock share them (`proto/cols/src/main.rs`).
3. **M4.** `series.m4(&columns, valid)` (per NaN-free run, outer
   neighbours inside the run, the y scale's `admits` as `valid`)
   returns the kept sample indices; the mark draws them as linear
   segments (curves do not apply to thinned data).
4. **Memo.** `thin::Memo` keeps the indices by `(recording id,
   revision, channel, grid)`, where `grid` hashes the boundary values'
   bits; entries unused for 32 draws are dropped, like the text cache.
   A redraw with an unchanged view queries nothing; an append bumps the
   revision and misses. Reusing single columns across pans needs
   anchored column grids and belongs to the conditional Phase 31, as
   does a dash phase held across tiles.

##### Drawing and saving

```rust
pub enum Target {
    /// SVG, PDF: list px, 4 thinning columns per px, no snapping.
    Vector,
    /// PNG, GIF: device px = list px × scale; 4 columns per device px.
    File { scale: f64 },
    /// iced: 1 column per device px.
    Screen { scale: f64 },
}
// figure::Cache gains the memo: { text: text::Cache, memo: thin::Memo }
// cache::Stats gains `pub m4_queries: u64`.

impl<'a> Figure<'a> {
    /// A fresh figure::Cache, Target::Vector, Held::default().
    pub fn draw(&self, size: kurbo::Size) -> List;
    pub fn save_svg(&self, path: impl AsRef<Path>, size: kurbo::Size)
        -> Result<Vec<lower::Warning>, figure::Error>;
    #[cfg(feature = "png")]
    pub fn save_png(&self, path: impl AsRef<Path>, size: kurbo::Size, scale: f64)
        -> Result<(), figure::Error>;
}
#[derive(Debug, thiserror::Error)]
pub enum Error {   // figure::Error
    #[error("cannot write {path}: {source}")] Io { path: PathBuf, source: std::io::Error },
    #[error(transparent)] List(#[from] list::Error),
    #[cfg(feature = "raster")] #[error(transparent)] Raster(#[from] raster::Error),
}
```

`draw_with` (signature from Phase 14): `cache.begin()` (text
generation and memo aging) → `resolve` (text batch inside) → figure
background part → per panel: background, grid (none in the default
theme), `Push` + clip, marks (one part each), `Pop`, border, axes,
title → figure title. Sizes are logical px. `save_svg` draws with
`Target::Vector`, lowers with `svg::CAPS`, writes, and returns the
warnings. `save_png` draws with `Target::File { scale }` and renders
with `raster::Settings::parallel()` (vello_cpu's own pool).

##### `figures_equal` and the examples

`figure/tests/main/support.rs`: `figures_equal(a: &Figure, b: &Figure,
size) -> Result<(), String>` compares list dumps (with glyph ids) and,
with `raster` on, renders both at 1× and 2× with `Settings { threads:
0, simd: Simd::Baseline }` and requires max ≤ 1/255. Figure tests from
Phase 16 on use it. The iced pixel tests of Phase 18 build their
reference separately, through `iced/tests/main/reference.rs::png_of`
(written in Phase 18), from the public path alone (`draw_with` with
`Target::Screen`, `list::Bounds::of`, `Renderer::render`); nothing is
added to figure or testkit for it.

- `dev/examples/src/trace.rs`: `pub fn trace(rec: &Recording<f64>) ->
  Figure<'_>` — two panels sharing x (seconds), channel 0 on top, 1
  and 2 below, `left`/`bottom` axes, no legend; `pub fn recording(n)`
  builds the 3-channel recording from `signal(n, seed)`, ported from
  `proto/thin2/src/main.rs:12–44` (the 1M-sample trace) with the seed
  as a parameter: 42 for channel 0, 43 and 44 for channels 1 and 2, so
  its numbers are comparable with Performance's.
- `dev/examples/src/scatter.rs`: `pub fn scatter(n) -> Figure<'static>`
  — `n` Gaussian points from the LCG of
  `proto/figmodel/examples/wheels_auto.rs:317–330`, 3 palette keys.
- `dev/examples/examples/{trace,scatter}.rs` write `trace.svg`,
  `trace.png`, `scatter.svg`, `scatter.png` into a directory given on
  the command line. Phase 22's benchmark builds its own workloads for
  repeatable timing; these examples are for people.

**Decisions this phase may meet**

- A thinning fidelity row misses its bound on the trace — the bounds
  come from Performance's measurements of the same generator and seed;
  check the column edges (`invert` at device-pixel edges) and the
  outer neighbours first, and if the gap is real record the measured
  shares as a decision before changing `k`.

#### Steps

1. **feat(figure): mark colors and guard band.** `mark/{color,guard}.rs`;
   Liang–Barsky tests.
2. **feat(figure): dots and rules.** `mark/{dots,rule}.rs`; fitting and
   command tests.
3. **feat(figure): line mark.** `mark/line.rs` (unthinned path).
4. **feat(figure): thinning.** `thin.rs`, `thin/{frame,memo}.rs`, the
   memo in `figure::Cache` and `m4_queries` in its stats; column, memo
   and fidelity tests.
5. **feat(figure): draw and Target::File.** `figure/draw.rs`,
   `target.rs`; snapshot `list_dump_of_scatter`.
6. **feat(figure): save_svg and save_png.** `figure/save.rs`,
   `tests/main/support.rs` (`figures_equal`); snapshot `svg_of_scatter`.
7. **feat(examples): trace and scatter.** `dev/examples/src/{lib,trace,scatter}.rs`,
   the thin mains; resvg agreement; `scripts/check.sh full`.

Tests live in `figure/tests/main/{marks,thin,draw,support}.rs`;
snapshots in `figure/tests/snapshots/{list_dump_of_scatter,svg_of_scatter}.txt`.

#### Exit criteria

1. `cargo test -p rustd3-figure --all-features` passes with every
   Spot-checks row named.
2. `cargo run -p rustd3-examples --release --example trace -- "$DIR"`
   and `--example scatter -- "$DIR"` write `trace.svg`, `trace.png`,
   `scatter.svg`, `scatter.png`; resvg of each SVG vs its PNG has tile
   score ≤ 0.02.
3. A second `draw_with` of an unchanged figure runs zero M4 queries
   and zero typst compiles (`figure::Cache::stats()`).
4. Snapshots `list_dump_of_scatter` and `svg_of_scatter` pass.
5. `scripts/check.sh full` passes.

#### Spot checks

| Input | Expected |
| --- | --- |
| `dots([(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)], x, y)` in one panel | exactly 1 `Markers` command with 3 points, inside one part |
| `dots` with a point at y = NaN | 2 points drawn; y fitted to the other values |
| `rule_y(20.0, count)` on a fitted count handle with bars up to 7 | the count domain grows to include 20 (rules fit) |
| `line` of (−1e9, 0) → (1e9, 1) on x fixed to [0, 1] | every path coordinate within ±3× the panel size; no NaN |
| trace `signal(1_000_000, 42)`, 1000 px panel, `Target::Screen { scale: 1.0 }` | 1000 columns; ≤ 4 points per column plus 2 per NaN-free run plus 2 neighbours; all 50 impacts present in the output indices (Performance: 3,949 points, 50 of 50) |
| same, `Target::Vector` | 4000 columns (Performance: 15,429 points); pixels off by more than 16/255 vs all samples stroked ≤ 5% (measured 3.8%) |
| same at 1 column per px | pixels off by more than 16/255 ≤ 12% (measured 10.3%); columns whose top or bottom is off by more than 1 px ≤ 3% (measured 2.2%) |
| two `line`s on two channels of one recording | column boundaries computed once (`thin::Frame` hit count 1) |
| `draw_with` twice, unchanged | second: 0 M4 queries, 0 typst compiles |
| `line(...).thin(Thin::Off)` on 1M samples | 1,000,000 points in the path |
| `save_png(path, 640 × 480, 2.0)` | a 1280 × 960 RGBA PNG |
| scatter of 5,600 points in 3 keys (`fill(color, key)`) | 3 `Markers` commands (the wheel figure's 5,600 dots are 12 commands across 4 panels) |

Benchmark material, not asserted (see Performance): stroking all 1M
points takes 465 ms against 4.9 ms for M4's 3,949; M4 from the index
0.54 ms for a full view, 0.22 ms at 5%; column boundaries 38 µs at 1M.

#### Not in this phase

- Tick counts from length, collisions, stacking (16).
- Legends, the figure locale, part hashes, hit testing, raster
  snapping, `fig.check`, `Fit::Visible` (17).
- `Fit::Window` and live appends (21); anchored column grids, dash
  phase across tiles and per-column memo reuse only in the conditional
  Phase 31.
- `.dodge`, `.jitter`, `.fill_by`, `.size_by`, bars, areas, histograms,
  densities (34); PDF and GIF (37).

### Phase 16 — Layout passes

*M2 · builds on phase 15 · about 1,100 lines, 6 commits*

#### Goal

Phase 14's single protrusion pass becomes the full layout. Tick counts
follow axis length (one tick per 80 px across, per 35 px up) and
layout repeats until counts settle, at most 4 passes. Shared axes are
labeled only on the outer panels; axes on one side stack outward; a
stacked axis on an inner panel is dropped. Tick labels that still
collide are dropped in two passes: a parity stride within each axis,
anchored to the absolute tick index so labels do not flicker while
panning, then a greedy pass across axes, larger panels first. Panels
may span tracks, passing their reaches to the outer tracks. Axes whose
domain is far from zero relative to its span get offset notation
("+1e9" at the axis end), tiny or huge ones a multiplier. During a
gesture, layout can be held: plot areas stay where they were and
labels grow outward, with slack reserved on interactive targets.

#### Builds on

- Phase 14: `Figure`, `Panel`, `Axis` (`ticks`, `labels`, `outward`),
  `layout::{Grid, tracks}`, protrusions, `resolve`, `Resolved`,
  `decorate`, `Theme` getters, `Target`, `figure::Cache`.
- Phase 15: marks, `draw_with`, `Target::File`, `figures_equal`.
- Phase 13: `text::Cache` batching, `Shaped::bounds`.
- Phase 6: `ticks::Set` and its `Step` (for the absolute index).

#### Design

##### Pass loop (`layout/passes.rs`)

```text
rects = place(protrusions with every axis at count 2 and no labels)
for pass in 1..=4:
    plans = plan(fig, rects)          # counts from lengths, inner/stacked
    measure all new labels (one typst batch per pass)
    rects' = place(protrusions(plans))
    if counts(plans) == previous counts: break
keep = collide(plans, rects')
```

`count = max(2, round(len / 80))` across (top, bottom) and `max(2,
round(len / 35))` up (left, right), Observable Plot 0.6.17's spacing;
`.ticks(n)` sets it. An axis whose tick rule returns no ticks has zero
label extent. The loop follows `proto/figmodel/src/plot/mod.rs:216–240`
(with rects held between passes).

```rust
impl Resolved {
    /// Layout passes until tick counts settled (1..=4).
    pub fn passes(&self) -> usize;
}
```

##### Plans (`layout/plan.rs`)

```rust
pub struct Plan { pub shown: bool, pub labeled: bool, pub offset: f64, pub thickness: f64, pub count: usize }
```

- **Inner:** an axis is inner when another panel further out on its
  side (same row for left/right, same column for top/bottom; with
  spans, overlapping rows or columns) shows the same handle on the same
  side. Inner axes keep ticks and lose labels and title.
- **Stacked:** axes on one side are placed in insertion order; each
  later one sits at `previous offset + previous thickness + 8 px`. A
  stacked axis on an inner panel is not shown. `.labels(b)` forces
  `labeled = b` and `shown = true`; `.outward(px)` sets the offset.
- **Thickness:** as in Phase 14, plus the offset label (below) when the
  axis has one, plus slack (Held layout).

Lift `inner` (`proto/figmodel/src/plot/arrange.rs:38–58`) and `plan`
(l. 60–109: stacking, shown/labeled/explicit); the stack gap is 8 px.

##### Collisions (`layout/collide.rs`)

Boxes are typst ink boxes in figure coordinates (axis-aligned; Phase 33
adds rotated boxes). Lift `collide` and `stride` (`arrange.rs:131–200`)
with the stride on the absolute index.

1. **Within an axis (parity).** Each tick gets an absolute index:
   `round(v / step)` when the set's `Step` is a positive `f64` and the
   ticks are uniform; the key's position in the domain for band and
   point scales; the position in the set otherwise (log ticks; time
   axes count whole intervals from the epoch, Phase 33). The stride is the smallest `k ≥
   1` such that labels with `index mod k == 0` pairwise do not overlap
   (touching is not overlap); those are the candidates.
2. **Across axes (greedy).** Candidates are sorted by panel plot area
   (larger first), then panel order, axis order, label order; a
   candidate is kept if its box overlaps no kept box.

Axis titles and panel titles do not take part: titles overflow
centered by design (see Text and math). `Resolved` keeps a keep-flag
per panel, axis and tick for `decorate` and for `fig.check` (Phase 17).

##### Spans (`layout/cell.rs`)

```rust
pub struct Cell { row: usize, col: usize, rows: Range<usize>, cols: Range<usize> }   // layout::Cell
impl Cell {
    pub fn new(row: usize, col: usize) -> Cell;
    pub fn span(self, rows: usize, cols: usize) -> Cell;
}
impl<'a> Figure<'a> {
    /// panel(r, c) == panel_at(Cell::new(r, c)).
    pub fn panel_at(&mut self, at: Cell) -> &mut Panel<'a>;
}
```

A spanning panel's left reach counts against its first column's left
track, its right reach against its last column's right track (top and
bottom likewise); its plot area runs from the first track's start to
the last track's end, including inner gaps and reaches. Grids do not
nest in v0.1 (see Later); fixed, content-sized and aspect tracks are
out of scope.

##### Offset and multiplier notation (`axis/offset.rs`)

For a continuous axis with domain `[a, b]` (after fitting), unless
`.offset(false)`:

- **Offset** when `|a + b| / 2 / |b − a| > 10⁴`: `p` is the smallest
  integer with `10^p ≥ |b − a|`; candidates are the multiples of `10^p`
  in `[min(a, b) − 10^p, max(a, b)]`; the offset `o` is the candidate
  with the most trailing zeros (ties: nearest to the centre). Labels
  are core's `Ticks::labels` of `t − o` (a translated linear set with
  the same step); the offset label is `+` or `−` (U+2212) followed by
  the shortest mantissa, `e`, exponent (`+1e9`, `+1.2345e4`, `−1e9`).
- **Multiplier** otherwise, when `m = floor(log10(max(|a|, |b|)))` is
  ≤ −5 or ≥ 6 (matplotlib's `axes.formatter.limits`): labels of `t /
  10^m`, label `1e−7`, `1e7`.
- The label sits at the axis's far end (the right end of a horizontal
  axis, above the top of a vertical one), in tick-label size, and adds
  to the reach on that side.

matplotlib 3.11.2's `ScalarFormatter` (offset text, order of
magnitude) is the reference for the spot rows.

##### Held layout (`layout/hold.rs`)

```rust
/// Plot areas a gesture holds, with the size and panel count they were made for.
#[derive(Debug, Clone, PartialEq)]
pub struct Areas { /* plot rects per panel, size, panel count */ }   // layout::Areas
impl Resolved {
    pub fn areas(&self) -> layout::Areas;
    /// True when this resolve used held areas.
    pub fn held(&self) -> bool;
}
impl<'a> Figure<'a> {
    pub fn resolve_held(&self, size: kurbo::Size, cache: &mut figure::Cache,
                        held: &Held, target: Target, areas: &layout::Areas) -> Resolved;
    pub fn draw_held(&self, cache: &mut figure::Cache, size: kurbo::Size,
                     target: Target, held: &Held, areas: &layout::Areas) -> List;
}
```

With `areas` given, rects are taken as they are (one pass: counts from
the kept lengths), labels and collisions are recomputed, and labels may
extend outward beyond the old reaches. If `areas` were made for
another size or panel count they are ignored: a fresh layout, and
`held()` is false. With `Target::Screen`, every labeled vertical axis
reserves two characters of slack — twice the width of `"0"` at
tick-label size (9.3315 px at notebook, so 18.663 px) — in its
thickness, in held and unheld resolves alike, so the frame laid out at
gesture end rarely moves. `resolve` and `draw_with` keep Phase 14's
signatures; `draw_held` mirrors `resolve_held`. The widget decides
when a gesture ends (Phase 20) and stores `layout::Areas` in its
gesture state. Measured on the wheel figure (see Layout): laying out
on every event moved one panel's left edge from 45 to 51 and 57 px
over eight wheel steps (matplotlib's constrained layout makes the same
jump, 22.9 to 61.2 px); holding, labels grew outward by at most 12 px.

**Decisions this phase may meet**

- The band row measures other label counts than 5, 7, 10, 20 — assert
  the measured counts, record them in `decisions/phase-16.md` as a
  decision with the label widths; Phase 33's band check reads them.
- Under the neutral SI locale's grouping (5+ integer digits grouped;
  see Formats, time and color), multipliers at `m ≥ 6` read worse than
  grouped numbers — raise the multiplier's upper limit and keep the
  offset rule; record it as a decision.

#### Steps

1. **feat(figure): tick counts from length.** `layout/passes.rs`; the
   pass loop with `Resolved::passes`. Tests: the auto-count row (4 / 2 /
   9) and `layout_settles_within_four_passes`.
2. **feat(figure): outer-only labels and stacking.** `layout/plan.rs`.
   Tests: the 1 × 2 labeled/shown rows, the stacked `temp` offset row
   and the `.labels(true)` row.
3. **feat(figure): collisions.** `layout/collide.rs`; the band and
   parity rows; the panning property test.
4. **feat(figure): spans.** `layout/cell.rs`, `Figure::panel_at`. Test:
   the spanning-panel row (its x is column 0's, its right edge column
   1's).
5. **feat(figure): offset and multiplier notation.** `axis/offset.rs`,
   `Axis::offset(bool)`. Test: the `[1e9, 1e9 + 1]` row (offset `+1e9`,
   labels 0.0 … 1.0) and `offset_false_prints_full_labels`.
6. **feat(figure): held layout.** `layout/hold.rs`,
   `Figure::{resolve_held, draw_held}`, `Resolved::{areas, held}`, the
   slack; `scripts/check.sh full`.

Files: `figure/src/layout.rs`, `figure/src/layout/{passes,plan,collide,cell,hold}.rs`,
`figure/src/axis.rs`, `figure/src/axis/{offset,labels}.rs`,
`figure/src/figure/{resolve,resolved,draw}.rs`, `figure/tests/main/layout.rs`.

#### Exit criteria

1. `cargo test -p rustd3-figure --all-features layout` passes, every
   Spot-checks row named.
2. Layout invariants hold on every figure in the tests and on the
   trace and scatter examples: no two kept tick labels' boxes overlap;
   attached panels share pixel ranges bit-exactly; `passes() ≤ 4`.
3. A property test pans a linear axis by 1–50 px steps over 200 steps,
   and the set of kept label values only gains or loses labels at the
   ends (parity is stable while panning).
4. `layout::Areas` is `Clone + Debug + PartialEq`; `resolve_held` with
   areas for another size returns `held() == false`.
5. `scripts/check.sh full` passes.

#### Spot checks

Text measured with typst at 14.667 px (notebook tick labels):

| Input | Expected |
| --- | --- |
| auto count across 328.4 px / 100 px; up 300 px | 4 / 2 / 9 |
| band of 20 keys `"0–10"` … `"190–200"`, `padding(0.1)`, bottom axis on plot widths 320, 480, 960, 1400 px (label widths 35.328, 44.659 ×8, 53.990, 63.322 ×10) | strides 4, 3, 2, 1: kept 5 (0, 4, 8, 12, 16), 7, 10, 20 labels |
| bottom axis, domain [0, 100] fixed, `.ticks(20)` (step 5), plot width 200 | stride 3: kept 0, 15, 30, 45, 60, 75, 90 |
| same panned to [2.5, 102.5] | kept 15, 30, 45, 60, 75, 90 (indices 3, 6, …, 18; local parity would give 5, 20, 35, …) |
| rows [1.0, 0.3], gap `px(4)`; top panel y fixed [0, 3], bottom panel y fixed [0, 100], both `left` | the bottom panel's "100" is dropped, the top's "0.0" kept (see Layout) |
| 1 × 2 grid, both panels `left(y)` on one handle | right panel: `shown`, not `labeled`; left: labeled |
| one panel with `left(vib)` then `left(temp)` | temp offset = vib thickness + 8 |
| same in the right panel of a 1 × 2 grid | the right panel's temp axis not shown |
| `left(y).labels(true)` on the inner panel | labeled |
| 2 × 2 grid, panel at `Cell::new(1, 0).span(1, 2)` | its x = column 0's x; its right edge = column 1's right edge |
| domain [1e9, 1e9 + 1] | offset `+1e9`, labels 0.0 … 1.0 (matplotlib: same, step 0.2 at 6 ticks) |
| domain [12345.0, 12345.3] | offset `+1.2345e4` (candidates 12344, 12345; tie → nearest the centre), labels 0.00 … 0.30 (matplotlib: same) |
| domain [−1e9 − 1, −1e9] | offset `−1e9`, labels −1.0 … 0.0 (matplotlib: same) |
| domain [1000, 1000.5] | no offset (centre / span = 2,000.5 ≤ 10⁴; matplotlib would show `+1e3`) |
| domain [0, 3e−7] | multiplier `1e−7` (m = −7), labels 0.0 … 3.0 (matplotlib: same) |
| held areas, y domain [0, 3] → [0, 30000] | plot rects bit-identical; the left reach grows by the width difference of "30 000" and "3.0" |
| `Target::Screen`, one labeled left axis | thickness + 18.663 px slack |
| 4 × 4, 8 × 8, 16 × 16 grids of shared axes | passes ≤ 4 (Layout measured 2 or 3) |

Cost context, reported by the benchmark only (see Layout): 2 × 2
0.07 ms, 4 × 4 0.14 ms, 8 × 8 0.31 ms, 16 × 16 1.04 ms; a warm layout
with typst 0.19 ms. `proto/figmodel/examples/{band_labels,layout_motion,layout_cost}.rs`
are the experiments behind these numbers.

#### Not in this phase

- Legends and their slots, the figure locale (17).
- Gesture timing and the 300 ms wheel end (20); live data (21); tiles
  and held column origins only in the conditional Phase 31.
- Rotated labels (33).
- Nested grids (see Later); fixed, content-sized and aspect tracks and
  automatic wrapping are out of scope.

### Phase 17 — Legends, parts and checks

*M2 · builds on phase 16 · about 1,500 lines, 7 commits · two sessions: cut after Step 4*

#### Goal

The figure layer is complete for the widget. Ordinal palettes get
legends in figure slots (top, bottom, left, right) or panel corners;
the figure gets its `Locale`. Every part of the list carries a hash of
what it drew, and the list is partitioned into cells by all plot-area
edges at device-pixel bounds, with the parts each cell touches, a
per-cell hash and a per-cell work count — what Phases 18–21 render and
skip by. Marks report hit candidates into a grid index for hover.
Raster targets snap plot-area edges and hairlines to device pixels.
`fig.check(size)` lays the figure out without drawing and reports
issues. Y handles can fit to the visible x range (`Fit::Visible`).

#### Builds on

- Phase 12: `list::{List, Part, part::Id, Bounds, Outlines}`,
  `raster::snap::{edge, width, line}`.
- Phase 13: `text::{Cache, Shaped::missing}`, `text::Error`.
- Phase 14: handles, `Kind`, `Paint::{paint, slot}`, `Fits`,
  `Fit<Span>` (whose `Visible` resolves as `All` until this phase),
  `figure::part` composition, `decorate`, `Resolved::plot_areas`.
- Phase 15: marks, `Target::{Vector, File, Screen}`, `figure::Cache`,
  `draw_with`, the M4 memo.
- Phase 16: the pass loop, plans, collisions, `layout::Areas`,
  `resolve_held`.
- Phase 9: the index's range-extremes query with a validity predicate,
  `recording.locate(x)`.
- Phase 6: `locale::Locale` (`Default`, `EN_US`).

#### Design

##### Modules

```text
figure/src/legend.rs          Legend<'a, K>, legend()
figure/src/legend/slot.rs     Slot
figure/src/legend/inset.rs    Inset
figure/src/legend/draw.rs     row (top/bottom), column (left/right, insets)
figure/src/list/parts.rs      part hashing (FNV-1a 64), Cells, Cell, cells()
figure/src/hit.rs             hit::Index, Hit
figure/src/hit/grid.rs        uniform bucket grid
figure/src/check.rs           check::Issue, Figure::check, size()
figure/src/scale/visible.rs   Fit::Visible memory and step rule
figure/src/layout/slots.rs    space reserved by figure slots
figure/src/layout/snap.rs     raster-target snapping of rects and hairlines
```

##### Legends

```rust
pub fn legend<'a, K>(palette: ScaleRef<Ordinal<K, Rgb>>) -> Legend<'a, K>
where K: PartialEq + Clone + Display + 'static;
impl<'a, K> Legend<'a, K> { pub fn title(self, t: impl Into<Text>) -> Self; }
pub enum Slot { Top, Bottom, Left, Right }
pub enum Inset { TopLeft, TopRight, BottomLeft, BottomRight }
impl<'a> Figure<'a> { pub fn legend<K>(&mut self, slot: Slot, l: Legend<'a, K>) -> &mut Self; }
impl<'a> Panel<'a> { pub fn legend<K>(&mut self, at: Inset, l: Legend<'a, K>) -> &mut Self; }
```

One entry per key of the ordinal domain, in domain order, label
`K::to_string()`, swatch a square of side 0.7 em in the key's color
(`.glyph(…)` is Phase 35). Sizes are in legend em (11 pt = 14.667 px
at notebook), from matplotlib 3.11.2's rc (checked): swatch to text
0.8 em (`handletextpad`); entries in a row 2.0 em apart
(`columnspacing`); rows in a column 0.5 em apart (`labelspacing`);
swatch height 0.7 em (`handleheight`); title (12 pt, seaborn) before
the entries.

- **Slots:** top and bottom are one row centered on the grid's plot
  extent (not the canvas); left and right one column centered
  vertically; each reserves its box plus 0.5 em (`borderaxespad`) from
  the grid, inside the outer padding.
- **Insets:** a column boxed with 0.4 em padding (`borderpad`), white
  at alpha 0.8 (`framealpha`), a 1 px `#cccccc` edge (`edgecolor`
  0.8), 0.5 em from the panel's corner, drawn after the marks.
- A legend wider than its slot overflows centered and `fig.check`
  reports it (text never wraps). Each legend is a part. One legend
  value may be placed in a slot and an inset at once.

Lift `Slot`, `Inset`, `Legend`, `row`, `inset` from
`proto/figmodel/src/plot/legend.rs`, replacing its 10-px constants
with the parameters above and adding left and right slots and typst
boxes; slot reservation and placement follow
`proto/figmodel/src/plot/mod.rs:144–154, 330–345`.

##### Figure locale

```rust
impl<'a> Figure<'a> { pub fn locale(&mut self, l: Locale) -> &mut Self; }  // core::locale::Locale, re-exported
```

Default tick labels and offset and multiplier labels use it through
core's `Ticks::labels(&set, &locale)`; user `.format` closures are
untouched (see Formats, time and color). This phase tests
`Locale::default()` and `Locale::EN_US`; the other number and time
tables arrive in Phases 26 and 28, and Phase 32 exercises figures with
them.

##### Part hashes

`Part::hash` (the field exists since Phase 12) is filled by the
figure's list builder as each part closes: FNV-1a 64 (hand-rolled,
about 15 lines; stable across Rust versions, unlike `DefaultHasher`)
over the transform in effect at the part's start and every command in
its range — command tag, path elements as `f64::to_bits`, paints
(color components, gradient stops, image blob ids), strokes, marker
points and channels, glyph run font, size, transform, ids and
positions. Hashing sees closure *output*, so a figure rebuilt from
equal data hashes equally. Because the transform in effect is hashed,
the same content drawn at another place hashes differently, which is
what makes a cell hash safe to skip a render by. `Part` gains no other
field (no style hash, no source metadata).

The list builder computes `list::Bounds::of(&list, &mut outlines)`
once when the list is complete; part bounds are unions over it and
`cells` buckets parts by them.

##### Cells

```rust
pub struct Cells { pub cols: Vec<u32>, pub rows: Vec<u32>, pub cells: Vec<Cell> }
pub struct Cell {
    /// Device px, integer edges.
    pub rect: kurbo::Rect,
    /// Indices into `List::parts` whose device bounds touch the cell.
    pub parts: Vec<u32>,
    /// The cell rect and its parts' hashes, in order.
    pub hash: u64,
    /// Raster work items of the listed parts.
    pub work: u64,
}
/// Partition by `xs`/`ys` (list px, snapped to device px), bucketed once.
pub fn cells(list: &List, xs: &[f64], ys: &[f64], scale: f64) -> Cells;   // list::cells
impl Cells {
    /// The cell containing a device pixel; cells are half-open.
    pub fn cell_at(&self, device: [i32; 2]) -> Option<usize>;
}
impl Resolved {
    /// Every plot-area edge plus 0 and the size, per axis.
    pub fn edges(&self) -> (Vec<f64>, Vec<f64>);
}
```

`cells` sorts and dedups the device edges `round(x · scale)` into
`cols` and `rows`, forms the grid row-major (cell index = row ×
columns + column; the wheel figure gives about 9 × 9 = 81 cells), and
lists for each cell the parts whose device bounds, inflated by one
device pixel, intersect it. `Cell::hash` combines the cell rect and
its parts' hashes in order. `cell_at` returns the cell whose `[x0, x1)
× [y0, y1)` contains the pixel, `None` outside the canvas.

`Cell::work` is the raster work of the parts the cell lists: one item
per `Fill`, `Stroke` or `Image` command plus `⌊elements / 16⌋` for its
path's elements, one per point of a `Markers` command, one per glyph
of a `Glyphs` command, zero for `Push` and `Pop`. A command whose part
touches several cells counts fully in each (conservative). Phase 19's
cost model divides it by `ITEMS_PER_MS`.

Rendering a cell is `raster::Renderer::render(&list, &bounds, cell
rect / scale, scale)` with the list's one `Bounds` value; culling
skips what misses the cell, so `parts` serve hashing and work, not
rendering. `Resolved::plot_areas()` returns the snapped rects on
raster targets; the widget tells plot-area cells from gutters by it (a
cell is a plot-area cell when its rect lies inside a plot area). Each
cell becomes one image, or a stack of bands under 2 MiB (Phase 18).
Command bounds and culling follow `proto/dlist/src/bin/partial.rs:15–48`.

##### Hit testing

```rust
pub struct Hit { pub panel: usize, pub mark: usize, pub index: usize, pub at: kurbo::Point, pub distance: f64 }
pub struct Index { /* per panel: a bucket grid of 32 px */ }   // hit::Index
impl Index { pub fn nearest(&self, at: kurbo::Point, radius: f64) -> Option<Hit>; }
impl<'a> Figure<'a> { pub fn hits(&self, resolved: &Resolved, cache: &mut figure::Cache) -> hit::Index; }
// Mark<'a> gains: fn hits(&self, ctx: &mark::Ctx<'_>, sink: &mut hit::Sink) {}   (default: none)
```

Dots report every drawn point with its datum index; lines report the
points they draw (thinned lines: the kept sample indices, from the
memo, so no second M4 query); rules report none. Coordinates are
figure px; `panel` is the flat panel index. `nearest` searches the
buckets overlapping the radius and breaks ties by insertion order.
Hover messages are Phase 36.

##### Raster targets

With `Target::File { scale }` or `Target::Screen { scale }`: after the
tracks are placed, every plot-area edge becomes `snap::edge(x, scale)`
(attached panels still share edges, because equal inputs snap
equally); axis lines, ticks, borders and rules use `snap::line` with
`snap::width`. `Target::Vector` is untouched. Cells are computed on
the snapped list.

##### `fig.check`

```rust
#[non_exhaustive]
pub enum Issue {
    /// Text longer than the space it labels (it overflows centered).
    Overflow { what: check::Element, panel: Option<usize>, needed: f64, available: f64 },
    /// No room left for plot areas after reaches and gaps.
    NoRoom { axis: check::Direction, needed: f64, available: f64 },
    /// Owned data over 100,000 points: use a Series.
    LargeOwned { panel: usize, mark: usize, points: usize },
    /// Glyphs the fonts lack.
    MissingGlyphs { text: String, count: usize },
    /// Markup that typst rejected.
    Markup { text: String, message: String },
    /// A palette key not in the palette's domain.
    UnknownKey { panel: usize, mark: usize },
}
pub enum Element { AxisTitle, PanelTitle, FigureTitle, Legend }   // check::Element
pub enum Direction { Horizontal, Vertical }                       // check::Direction
impl<'a> Figure<'a> { pub fn check(&self, size: kurbo::Size) -> Vec<check::Issue>; }
/// `size(1600.0, 1200.0)`, as in Text and math's example.
pub fn size(width: f64, height: f64) -> kurbo::Size;
```

`check` resolves with a fresh cache and `Target::Vector`, draws
nothing, and collects issues from layout, text and marks (`Mark<'a>`
gains `fn issues(&self, ctx: &mark::Ctx<'_>, issues: &mut
Vec<check::Issue>) {}`, default none).

##### `Fit::Visible`

A `Visible` handle on y fits to the data whose x lies in the panel's
resolved x domain: x handles are resolved first; owned points filter
by x; `Series` views go through the x scale's bridge,
`x.indexed()?.series_extent(&points, Some([x0, x1]), &valid)` (Phase
14), which reads the index over `recording.locate(x0)..locate(x1)` in
O(log n). Across panels sharing
the y handle, extents merge. Hysteresis keeps the domain still while
the data wobbles, through memory in `figure::Cache` keyed by handle
index and `TypeId`:

- the domain grows at once to contain the new padded extent;
- it shrinks only in steps of one eighth of its previous span, each
  end moving inward by whole steps while the padded extent still fits.

On an x handle `Visible` behaves as `All`; a y handle whose panels' x
is itself `Visible` falls back to `All` (no cycles).

**Decisions this phase may meet**

- A test shows `Fit::Visible` flickering with the one-eighth step —
  record the step that holds still, with the test's trace, as a
  decision.

#### Steps

1. **feat(figure): part hashes.** `list/parts.rs` (hash), the figure's
   list builder filling `Part::hash` and part bounds from one
   `list::Bounds`; stability tests.
2. **feat(figure): cells.** `cells`, `Cell::work`, `Cells::cell_at`,
   `Resolved::edges`; bucket and work tests.
3. **feat(figure): legends.** `legend.rs`, `legend/*`,
   `layout/slots.rs`; slot and inset tests.
4. **feat(figure): figure locale.** `Figure::locale`; EN_US vs the
   neutral default. *Cut point: a second session starts at Step 5.*
5. **feat(figure): raster targets.** `layout/snap.rs`; snapped
   `Resolved::plot_areas`; snapping tests at four scales.
6. **feat(figure): hit index and check.** `hit.rs`, `hit/grid.rs`,
   `check.rs`, `size()`; `Mark::{hits, issues}` for dots and lines.
7. **feat(figure): Fit::Visible.** `scale/visible.rs`, the memory in
   `figure::Cache`; `scripts/check.sh full`.

Files also touched: `figure/src/figure.rs`,
`figure/src/figure/{cache,resolve,resolved,draw}.rs`,
`figure/src/mark.rs`, `figure/src/mark/{dots,line}.rs`. Tests live in
`figure/tests/main/{legend,parts,hit,check,visible}.rs`.

#### Exit criteria

1. `cargo test -p rustd3-figure --all-features` passes with every
   Spot-checks row named.
2. Drawing a figure twice gives identical part and cell hashes;
   changing one mark's data changes that mark's part hash and the
   hashes of the cells it touches only; a panel pushed down by a new
   figure title changes its parts' bounds and hashes.
3. `fig.check` is empty on the trace and scatter examples at 960 × 760
   and reports the overflowing title in the check test.
4. At scales 1, 1.25, 1.5 and 2 with `Target::File`, every plot-area
   edge times the scale is an integer and every hairline covers whole
   device pixels.
5. `scripts/check.sh full` passes.

#### Spot checks

| Input | Expected |
| --- | --- |
| `legend(color)` over `["accel x", "accel z", "bearing temp"]` with colorblind | 3 squares of 10.267 px (0.7 × 14.667) in `#0173b2`, `#de8f05`, `#029e73`, domain order |
| the same value in `Slot::Bottom` and `Inset::TopLeft` | both draw; the bottom row is centered on the grid's plot extent |
| `Slot::Bottom` on a 640 × 480 one-panel figure | the panel's bottom edge moves up by the legend row height + 0.5 em |
| ticks 0 … 12000 step 2000, `Locale::default()` vs `Locale::EN_US` | "0", "2 000" … "12 000" (U+202F) vs "0", "2,000" … "12,000" |
| ticks 0 … 8000 step 2000, default locale | "2000" … "8000", ungrouped |
| a `.format(\|v\| format!("{v:.0}°"))` axis under EN_US | unchanged by the locale |
| the same figure drawn twice | equal `Part::hash` per part, equal `Cell::hash` per cell |
| one dots mark's data changed | only that part's hash and the cells it touches change |
| a part with one 33-element `Stroke` and a 5,600-point `Markers` | work 1 + ⌊33 / 16⌋ + 5,600 = 1 + 2 + 5,600 = 5,603 |
| `Cells` with `cols` [0, 100, 200], `rows` [0, 50] | `cell_at([100, 10])` = `Some(1)`; `cell_at([200, 10])` = `None` |
| wheel-like 4 × 4 grid at 960 × 760, scale 2 | 9 × 9 cells (about 81) |
| `Target::File { scale: 1.25 }` | `rect.x0 × 1.25` integral for every panel; a 1.25 pt axis line (1.667 px) → device width 2 (round(2.083)) |
| `Target::Vector` | rects identical to Phase 16's |
| 3 dots at (10, 10), (20, 10), (200, 200) (figure px), `nearest((12, 11), 5)` | index 0, distance √5 |
| `nearest((100, 100), 5)` | `None` |
| left axis title `"vibration RMS [m/s²]"` (16 px: 164.56 px wide) on a 100-px-tall panel | `Overflow { what: AxisTitle, needed: 164.56, available: 100, .. }` (at talk size on the wheel figure the same title needs 247 of 151 px; see Text and math) |
| `dots` with 150,000 owned points | `LargeOwned { points: 150000, .. }` |
| a label `"漢字"` | `MissingGlyphs { count: 2, .. }` |
| `Fit::Visible` y over a ramp y = x, x domain [0, 100] → [40, 60] | from [−5, 105] (step 110 / 8 = 13.75) to [36.25, 63.75] in one resolve, then stays (the next step, 27.5 / 8 = 3.4375, would cut into the padded [39, 61]) |
| same, x back to [0, 100] | y grows at once to [−5, 105] |

#### Not in this phase

- The widget, images and bands (18), the render worker (19); tiles,
  dependency invalidation and `Caching::Verify` only in the
  conditional Phase 31.
- Live data and `Fit::Window` (21).
- Colorbars, legend glyphs, seaborn styles (35); hover messages and
  brushes (36); `Resolved::panel_cell` (36).
- The other locale tables (26, 28; exercised in 32); time handles (33);
  PDF text (37).

### Phase 18 — Plot I: display

*M2 · builds on phases 11, 17 · about 1,000 lines (≈600 library, ≈400 tests), 5 commits*

#### Goal

`rustd3_iced::Plot` exists: a custom iced widget, behind rustd3-iced's
`figure` feature, that takes a `Figure<'a>` by value, resolves and
draws it once per `RedrawRequested`, renders every cell of the
figure's cell partition inline with vello_cpu, and shows each cell as
iced images placed exactly on device pixels at the effective scale
factor. A cell whose RGBA would reach `BAND_BYTES` (2 MiB) is rendered
and drawn as horizontal bands below that size, so wgpu uploads every
image inside the frame that first draws it. A crosshair and a
nearest-point highlight are drawn on top as quads, clipped to the
hovered panel with `with_layer`. Pixel tests through Phase 11's
headless harness show that the widget's output equals the PNG of the
same figure on both renderers. Every change re-renders every cell; no
worker, gestures or per-cell skipping yet.

#### Builds on

- Phase 11: iced at rev `3cbdacc3b0841c5145c766c3a758657d36b41ffe`;
  rustd3-iced on `iced_core` + `iced_widget` (`advanced`, `canvas`,
  `image`); the `figure` feature enabling `rustd3-figure/raster`;
  `pointer::Tracker`; the headless harness in
  `iced/tests/main/headless.rs`: `Harness::new(root, size, scale) ->
  Option<Harness>` (`None` when wgpu has no device and
  `RUSTD3_ALLOW_NO_GPU=1`), `point_at`, `events`, `tick`, `frame(now) ->
  Shot` (tick, `RedrawRequested(now)` through `update`, draw,
  screenshot), `waker`, `wakes`, `messages`, `redraw_requested()`. The
  harness hints the renderer's scale, so wgpu reports it; tiny-skia's
  `scale()` stays `None`. The placement tests in
  `iced/tests/main/placement.rs` and `decisions/phase-11.md` pin the two
  constants this phase defines: `BAND_BYTES = 2 * 1024 * 1024` and
  `NUDGE = 0.25` device px.
- Phase 12: `raster::Renderer::new(raster::Settings)`,
  `Settings::parallel()`, `list::Bounds::of(&List, &mut list::Outlines)`,
  `Renderer::render(&mut self, &List, &list::Bounds, area: Rect, scale:
  f64) -> Result<raster::Image, raster::Error>` with `area` in list px
  and output exactly `round(area × scale)` device px;
  `raster::Image { width, height, rgba }` in straight alpha
  (`take_rgba8(ImageAlphaType::Alpha)`).
- Phases 14–16: `Figure<'a>`, `ScaleRef<S>`, `Held`,
  `fig.resolve(size, &mut figure::Cache, &Held, Target::Screen { scale })
  -> Resolved`, `fig.draw_with(&mut cache, size, Target::Screen { scale },
  &held) -> List`; `figure::Cache` owns the text cache and the M4 memo;
  the theme background is a colour with alpha.
- Phase 17: `list::cells(&list, &xs, &ys, scale) -> list::Cells` with
  `(xs, ys) = resolved.edges()`, each `list::Cell { rect, parts, hash,
  work }` with integer device edges; `Resolved::plot_areas()` (snapped
  on raster targets); `Target::Screen` snaps plot-area edges and
  hairlines; `fig.hits(&resolved, &mut cache) -> hit::Index` and
  `Index::nearest(at, radius) -> Option<Hit>` with its panel.

#### Design

**Module layout** (`iced/src/`, all behind `#[cfg(feature = "figure")]`):

```text
plot.rs              Plot<'a, Message>: builders, Widget impl; BAND_BYTES, NUDGE
plot/state.rs        State: the widget-tree state, refresh
plot/cell.rs         Cell, Band: a partition cell's device rect, hash and band images
plot/handle.rs       fn from_image: raster::Image → iced image::Handle
plot/overlay.rs      Overlay: crosshair and highlight
plot/scale_factor.rs fn resolve: the scale-factor chain
```

`lib.rs` adds `#[cfg(feature = "figure")] pub mod plot;` and
`pub use plot::Plot;`.

**Constants** (`plot.rs`):

```rust
/// RGBA bytes at which a cell is split into bands: wgpu uploads images
/// under 2 MiB inside the frame (`wgpu/src/image/cache.rs:223-246`)
/// and larger ones through a worker drained only by `Renderer::tick`.
pub const BAND_BYTES: usize = 2 * 1024 * 1024;

/// Device px added to an image's origin before dividing by the factor.
pub const NUDGE: f32 = 0.25;
```

`NUDGE` lands on the intended device pixel on both renderers:
tiny-skia truncates `(bounds.x / width_scale) as i32`
(`tiny_skia/src/raster.rs:63-64`), wgpu adds 0.001 and rounds
(`wgpu/src/nudge.rs:43,72-74`). Without a nudge, `edge / k × k` can
evaluate a hair under the edge and truncate one pixel early; +0.5
rounds one pixel late on wgpu (15.5 + 0.001 → 16); +0.25 is exact on
both (15.25 → 15 either way).

**Public API**

```rust
/// A figure shown as images rendered from its drawing list.
pub struct Plot<'a, Message> {
    figure: Figure<'a>,
    width: Length,
    height: Length,
    scale_factor: Option<f32>,
    crosshair: bool,
    highlight: bool,
    _message: PhantomData<Message>,
}

impl<'a, Message> Plot<'a, Message> {
    /// Shows `figure`; the widget keeps it for relayouts.
    pub fn new(figure: Figure<'a>) -> Self;
    /// Defaults to `Length::Fill`.
    pub fn width(self, width: impl Into<Length>) -> Self;
    pub fn height(self, height: impl Into<Length>) -> Self;
    /// Device pixels per logical pixel; wins over every fallback.
    pub fn scale_factor(self, factor: f32) -> Self;
    /// Crosshair in the hovered panel (default on).
    pub fn crosshair(self, on: bool) -> Self;
    /// Ring around the nearest data point (default on).
    pub fn highlight(self, on: bool) -> Self;
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Plot<'a, Message>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{ /* size, tag, state, diff, layout, update, draw, mouse_interaction */ }

impl<Message> Meta for Plot<'_, Message> {}

impl<'a, Message, Theme, Renderer> From<Plot<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: image::Renderer<Handle = image::Handle> + 'a,
{ /* Element::new */ }
```

`image` is `iced_core::image`; `Widget`, `Meta`, `Element`, `Tree`,
`tree`, `Layout`, `layout`, `Length` come from `iced_core` (through
`iced_widget::core`). `image::Renderer` has `core::Renderer` as its
supertrait, which gives `with_layer` and `fill_quad`; no geometry
renderer is needed because the overlay is quads. Both defaults are
on, as matplotlib's and uPlot's cursors are.

**Widget methods** (iced at the pin: `core/src/widget.rs:40-150`,
`Meta` at 161; `core/src/widget/tree.rs:62-73`, `Tree::diff` calls
`Widget::diff` on every rebuild):

- `size` → `Size::new(self.width, self.height)`.
- `tag` → `tree::Tag::of::<State>()`; `state` →
  `tree::State::new(State::default())`. `State` is `'static`: it holds
  no borrow of the figure.
- `diff(&mut self, tree)` → `state.rebuilt()`: a new `view` built a new
  figure, which may or may not differ (iced rebuilds on every message).
- `layout(&mut self, tree, renderer, limits)` →
  `layout::atomic(limits, self.width, self.height)`
  (`core/src/layout.rs:117`); a size different from `state.size` is
  stored and marks the list for rebuilding.
- `update`: the event table below.
- `draw`: cells, then the overlay. Each handle is drawn once per frame
  (one cache drawn twice in a frame shows once on wgpu,
  `proto/icedspike/src/bin/xform.rs`).
- `mouse_interaction` → `Interaction::Crosshair` over a plot area,
  `None` elsewhere.

| Event | Action |
| --- | --- |
| `Window(Opened { scale_factor, .. })`, `Window(Rescaled(f))` | `state.window_factor = Some(f)`; rebuild if the effective factor changed; `request_redraw` |
| `Window(RedrawRequested(now))` | `state.refresh(&self.figure, …, renderer)` (below); store `now` |
| `Mouse(CursorMoved)`, `Mouse(CursorLeft)` | `state.overlay.hover(position, …)`; `request_redraw` only if the overlay changed |
| anything else | ignored; this phase never calls `capture_event` |

The runtime sends `RedrawRequested` through `update` before `draw` and
repeats it up to three times if it produced messages
(`winit/src/lib.rs:777-830`); `core/src/window/event.rs:10-39` defines
the events, `core/src/shell.rs:130` `request_redraw`.

**Refresh** runs only when the list needs rebuilding (figure rebuilt,
resized or rescaled):

1. Effective factor `k = scale_factor::resolve(self.scale_factor,
   renderer.scale(), state.window_factor)`.
2. Resolve and draw the figure at the widget's size as
   `Target::Screen { scale: k }` through `state.cache` (a
   `figure::Cache`).
3. Cells from `list::cells(&list, &xs, &ys, k)` with `(xs, ys) =
   resolved.edges()`. If the sequence of `(rect, hash)` equals the
   stored one, keep every handle (the rebuild changed nothing visible)
   and stop.
4. Otherwise compute `bounds = list::Bounds::of(&list, &mut
   state.outlines)` once, render every band of every cell with the
   state's `raster::Renderer` (built once with
   `Settings::parallel()`, vello_cpu's own pool) as
   `render(&list, &bounds, band_device_rect / k, k)`, turn each image
   into a handle (`handle::from_image`), replace `state.cells`, and
   rebuild the hit index with `fig.hits(&resolved, &mut state.cache)`
   and the plot areas from `Resolved::plot_areas()`.

The list is dropped after step 4; nothing borrows the figure past
`update`.

**Scale factor** (`scale_factor.rs`):

```rust
/// Explicit input, the renderer's hint, the window's factor, then 1.0.
/// Non-finite or non-positive values are skipped.
pub(crate) fn resolve(
    explicit: Option<f32>,
    renderer: Option<renderer::Scale>,
    window: Option<f32>,
) -> f32;
```

`renderer.scale()` is `None` on tiny-skia; on wgpu it is `Scale {
window, application }` and `total()` multiplies them
(`core/src/renderer.rs:75,214`). The window events carry only the
window factor (`winit/src/lib.rs:671-675`), so an app with its own
`scale_factor` on the software renderer must pass
`Plot::scale_factor`. Every pixel test sets `.scale_factor(k)` to the
factor its harness was built with.

**Cells, bands and placement** (`cell.rs`):

```rust
pub(crate) struct Cell {
    /// Device px [x0, y0, x1, y1], relative to the widget's device origin.
    rect: [i32; 4],
    hash: u64,
    bands: Vec<Band>,
}

pub(crate) struct Band {
    /// Device rows relative to the cell.
    rows: Range<i32>,
    handle: image::Handle,
}

impl Cell {
    /// Row ranges of the bands a cell of this rect is drawn as.
    fn bands(rect: [i32; 4]) -> Vec<Range<i32>>;
    /// Logical bounds that land exactly on a band's device pixels.
    fn bounds(&self, rows: &Range<i32>, origin: [i32; 2], factor: f32) -> Rectangle;
}
```

- A cell whose RGBA reaches the limit (`width × height × 4 ≥
  BAND_BYTES`) becomes bands of whole device rows: `rows_per_band =
  BAND_BYTES / (width × 4) − 1` (integer division; strictly under the
  limit even when the row size divides it), at least 1, the last band
  shorter. A smaller cell is one band `0..height`. `Cell::bands` is
  pure and unit-tested.
- The widget's device origin is `round(layout.bounds().x × k)`,
  `round(layout.bounds().y × k)`, so a widget at a fractional logical
  offset still puts every cell on whole device pixels (shifted by at
  most half a device pixel against its layout bounds).
- A band's bounds: x = `(origin.x + rect.x0 + NUDGE) / k`, y =
  `(origin.y + rect.y0 + rows.start + NUDGE) / k`, width = `(rect.x1 −
  rect.x0) / k`, height = `rows.len() / k`.
- Each band is drawn with `Image::new(handle).filter_method(
  FilterMethod::Nearest)` and `clip_bounds` equal to its bounds; cells
  partition the canvas, so nothing overlaps.

**Handles** (`handle.rs`; `core/src/image.rs:14-60,134,213,285-313`):

```rust
/// Moves a straight-alpha raster image into an iced handle (no copy).
pub(crate) fn from_image(image: raster::Image) -> image::Handle;
```

iced handles carry straight alpha (tiny-skia premultiplies on load,
`tiny_skia/src/raster.rs:104-115`), which is what Phase 12's raster
returns, so the bytes move into `Handle::from_rgba(width, height,
rgba)` via `Bytes::from(Vec<u8>)`. With an opaque theme background the
image is opaque and the two alpha forms are the same bytes; with a
translucent one the 8-bit premultiply round trip costs up to 4/255.
Every render makes a new handle and so a new texture id
(`core/src/image.rs:134`); the renderer trims the old one after the
first frame that does not draw it. Because `render` returns exactly
`round(area × k)` device px, an area of `device_rect / k` gives back
the device rect's size (204.8 × 1.25 = 256.00000000000003 → 256).

**Overlay** (`overlay.rs`):

```rust
pub(crate) struct Overlay {
    cursor: Option<Point>,
    /// Plot area (device rect) of the hovered panel.
    panel: Option<[i32; 4]>,
    /// Nearest point, device px, within 12 logical px of the cursor.
    point: Option<Point>,
}
```

`hover` finds the panel whose plot area contains the cursor (from the
plot areas kept in the state) and the nearest point through Phase 17's
grid index. `draw` wraps everything in `renderer.with_layer(panel_bounds,
…)` (`core/src/renderer.rs:28`): two `fill_quad`s one device pixel
wide (`1.0 / k` logical) for the crosshair, in the theme's ink at 40%
alpha, and one quad with `border::rounded(r)` and a 1.5 px border for
the highlight (r = 4 logical px). Quads draw the same on both
renderers and avoid tiny-skia's `Frame::with_clip` bug, which left
443,622 inked pixels outside the clip where `with_layer` leaves 0
(`proto/icedspike/src/bin/layerclip.rs`, whose `layout` signature
matches the pinned iced). `CursorLeft` clears the overlay.

**Cost.** Rendering all cells inline costs about one whole-figure
render: one culled wheel panel at 2× takes ≈ 4.4 ms against 16–25 ms
for the whole figure (`proto/dlist/src/bin/partial.rs`; list replay
into vello_cpu in `proto/dlist/src/vcpu.rs`). Phase 19 moves expensive
renders off the UI thread.

**Decisions this phase may meet**

- The crosshair and highlight defaults are disputed by a consumer —
  keep both on, as matplotlib's and uPlot's cursors are; record it as
  a decision.

#### Steps

1. **Widget skeleton and scale factor.** `plot.rs` (with `BAND_BYTES`,
   `NUDGE`), `plot/state.rs`, `plot/scale_factor.rs`; `Plot::new`,
   `width`, `height`, `scale_factor`; `size`, `tag`, `state`, `diff`,
   `layout`; `draw` draws nothing yet. Unit tests: the scale-factor
   table (spot checks). Verify: `cargo test -p rustd3-iced --features
   figure scale_factor`.
2. **Cells, bands and handles.** `plot/cell.rs`, `plot/handle.rs`;
   `refresh` resolves, draws and renders every band of every cell
   inline; `draw` places them. Unit tests:
   `cells_partition_the_canvas` (union equals the canvas rect, pairwise
   intersections empty, for a 2×2 grid figure at 1, 1.25, 1.5, 2);
   `unchanged_figure_renders_no_cell` (two refreshes after a rebuild of
   an equal figure: the second renders 0 cells, counted through a
   `State` method visible to unit tests);
   `cell_images_have_their_device_size` (factors 1.25, 1.5);
   `bands_match_the_whole_cell` (a 960 × 640 cell rendered whole and as
   bands stitched back agree within 1/255; bands `[0, 545)` and `[545,
   640)`); `bands_stay_under_the_limit` (every band `< BAND_BYTES`; a
   300 × 200 cell is one band).
3. **Pixel tests.** `iced/tests/main/reference.rs` with
   `pub fn png_of(fig: &Figure<'_>, size: kurbo::Size, scale: f64,
   background: iced::Color) -> raster::Image` (owned; tests compare it
   through `testkit::pixels::Image::new(w, h, &img.rgba)`): `fig.draw_with(
   &mut figure::Cache::default(), size, Target::Screen { scale },
   &Held::default())`, then `raster::Renderer::new(Settings { threads:
   0, simd: Simd::Baseline }).render(&list, &list::Bounds::of(&list,
   &mut list::Outlines::default()), Rect::from_origin_size((0.0, 0.0),
   size), scale)`, composited source-over onto `background`. Nothing is
   added to figure or testkit for it. `iced/tests/main/plot.rs`
   (declared in `tests/main/main.rs`): a 480 × 320 logical two-panel
   figure (a line of 10k samples and 2k dots, two axes, a title) shown
   by `Plot::new(fig).scale_factor(2.0)` through `Harness::new(plot,
   size, 2.0)` and `harness.frame(now)`, compared with `png_of(&fig,
   size, 2.0, background)` by testkit's pixel statistics (max diff and
   count over 16, printed per backend):
   `plot_matches_the_png_of_its_figure` (opaque theme background) and
   `translucent_background_matches_the_png` (background at 50% alpha).
   `plot_places_cells_on_device_pixels_at_fractional_scales` repeats
   the comparison at 1.25 and 1.5.
4. **Overlay.** `plot/overlay.rs`; hover handling in `update`; quads in
   `draw` under `with_layer`. Tests:
   `crosshair_stays_inside_the_hovered_panel` (cursor at the centre of
   panel 0, overlay ink pure red through the theme; red pixels outside
   panel 0's plot area: 0), `highlight_marks_the_nearest_point` (ring
   centre within 1 device px of the expected point), unit test
   `hover_renders_no_cell`.
5. **Docs and `check.sh full`.** Doc comments with a `no_run` example
   of `view` returning `Plot::new(figure(&self.data)).into()`.

Verify every step with `scripts/check.sh quick` and
`ICED_TEST_BACKEND=tiny-skia|wgpu cargo test -p rustd3-iced --features
figure`. `cargo build --release -p rustd3-iced` alone fails by design
(`iced_renderer` without a backend hits `compile_error!`); `check.sh`
builds it in debug.

#### Exit criteria

1. `cargo test -p rustd3-iced --features figure` passes under
   `ICED_TEST_BACKEND=tiny-skia` and `ICED_TEST_BACKEND=wgpu` (wgpu
   skipped with a message under `RUSTD3_ALLOW_NO_GPU=1`).
2. `plot_matches_the_png_of_its_figure` passes at factor 2 with max
   diff ≤ 1/255 on tiny-skia and ≤ 2/255 on wgpu;
   `translucent_background_matches_the_png` passes with max ≤ 4/255 on
   both.
3. `plot_places_cells_on_device_pixels_at_fractional_scales` passes at
   1.25 and 1.5.
4. `crosshair_stays_inside_the_hovered_panel` finds 0 overlay pixels
   outside the panel's plot area on both backends.
5. Unit tests `cells_partition_the_canvas`,
   `unchanged_figure_renders_no_cell`, `hover_renders_no_cell`,
   `cell_images_have_their_device_size`, `bands_match_the_whole_cell`,
   `bands_stay_under_the_limit` and the scale-factor table pass.
6. `scripts/check.sh quick` passes on every commit; `full` at the end.

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| `resolve(Some(1.5), Some(Scale { window: 2.0, application: 1.0 }), Some(2.0))` | 1.5 | explicit wins |
| `resolve(None, Some(Scale { window: 2.0, application: 0.75 }), Some(2.0))` | 1.5 | `total()` = 2.0 × 0.75 |
| `resolve(None, None, Some(1.25))` (tiny-skia) | 1.25 | window factor |
| `resolve(None, None, None)` | 1.0 | last fallback |
| `resolve(Some(f32::NAN), None, Some(2.0))` | 2.0 | non-finite skipped |
| 480 × 320 logical at 2.0 | 960 × 640 device px, 2,457,600 B for the whole canvas (> 2,097,152); every cell < 2,097,152 B, so each is one band | banding is covered by `bands_match_the_whole_cell` |
| `Cell::bands` of a 960 × 640 cell | 3,840 B per row; 2,097,152 / 3,840 = 546, − 1 = 545 rows per band; bands `[0, 545)`, `[545, 640)`; the first holds 2,092,800 B | band rule |
| `Cell::bands` of a 300 × 200 cell | 240,000 B → one band `[0, 200)` | band rule |
| Cell at device x 125, factor 1.25, origin 0 | logical x (125 + 0.25) / 1.25 = 100.2 | placement |
| Widget at logical x 10.3, factor 1.5 | device origin round(15.45) = 15; cell 0 drawn at logical (15 + 0.25) / 1.5 = 10.1667 | placement |
| That bound, back to device px | wgpu round(15.25 + 0.001) = 15; tiny-skia trunc(15.25) = 15; with +0.5: wgpu 16 | `wgpu/src/nudge.rs:72-74`, `tiny_skia/src/raster.rs:63-64` |
| `render` of area (0, 0, 204.8, 100) at 1.25 | 256 × 125 px | Phase 12: round, not ceil |
| Overlay outside the clip | 0 inked px with `with_layer` on both renderers; 443,622 with tiny-skia `Frame::with_clip` | `proto/icedspike/src/bin/layerclip.rs` |
| One wheel panel, culled, at 2× | ≈ 4.4 ms against 16–25 ms for the whole figure | `proto/dlist/src/bin/partial.rs` |
| `raster::Image` of 960 × 640 | `Handle::from_rgba(960, 640, …)` over 2,457,600 bytes, moved, not copied | `handle.rs` |

#### Not in this phase

- The render worker, inline-when-cheap prediction, `Render`,
  `Plot::probe`, per-cell hash skipping (Phase 19).
- Gestures, `on_change`, held layout, scaled previews, double-click
  reset (Phase 20).
- Live data, `Fit::Window`, `Plot::revision`, `Plot::caching`
  (Phase 21); the full-frame benchmark (Phase 22).
- Tiles, pan translation, a pool and `Plot::threads`,
  `Caching::Verify`, cell merging (Phase 31, conditional).
- Hover messages to the app and the brush overlay (Phase 36).
- `allocate_image` or any residency logic: every image is under
  2 MiB. No change to `figure/` or to any manifest.

### Phase 19 — Plot II: render pipeline

*M2 · builds on phase 18 · about 1,000 lines (≈650 library, ≈350 tests), 5 commits*

#### Goal

The `Plot` widget stops blocking the UI thread on expensive renders and
stops re-rendering what did not change. Every refresh compares Phase
17's per-cell hashes and plans jobs only for changed cells; it renders
them inline when a two-constant cost model predicts they fit in half
the frame budget, and otherwise hands them to one worker thread whose
`raster::Renderer` uses vello_cpu's own thread pool. Jobs are whole
cells (all bands of a cell at once); at most one job set is pending
(`replace`); results return over `std::sync::mpsc`, wake the UI
through the stored `shell.waker()` and are drained on `Event::Waken`;
the newest generation wins and the previous image stays on screen until
its replacement arrives. `Plot::render(Render)` forces either path, and
`Plot::probe(Probe)` reports per-stage durations, per-cell raster
times and what each frame showed, from an app-supplied clock. Every
image stays under 2 MiB through Phase 18's bands, so no image needs
`allocate_image`.

#### Builds on

- Phase 18: `Plot`, `plot::State` with `refresh`, `Cell { rect, hash,
  bands }` with `Cell::bands(rect)`, `Band`, `NUDGE`, `BAND_BYTES`,
  `handle::from_image`, the overlay, `scale_factor::resolve`, the pixel
  tests and `iced/tests/main/reference.rs::png_of`.
- Phase 17: `list::Cells` with per-cell `rect`, `parts`, `hash` and
  `work` (one item per `Fill`/`Stroke`/`Image` command plus one per 16
  path elements, one per marker point, one per glyph);
  `Resolved::plot_areas()`.
- Phases 14–16: `Figure::resolve`, `draw_with`, `draw_held`,
  `figure::Cache`, `Target::Screen { scale }`.
- Phase 12: `raster::Renderer::new(Settings)`, `Settings::parallel()`
  (available parallelism − 1, clamped `1..=8`), `list::Bounds::of`,
  `render(&mut self, &List, &list::Bounds, area, scale)` returning
  exactly the device size; `List: Send + Sync` and `list::Bounds: Send
  + Sync` (asserted there).
- Phase 11: the harness with `tick()`, `waker()`, `wakes()`,
  `redraw_requested()`, `events`, `frame(now)`.

#### Design

**Module layout** (`iced/src/plot/`; Phase 18's `cell.rs` folds into
`slot.rs`):

```text
render.rs   Render: Auto | Inline | Worker
cost.rs     cost::{PIXELS_PER_MS, ITEMS_PER_MS, model_ms}: the raster cost model
probe.rs    Probe; probe::{Sample, Stage, Target}
job.rs      Job; job::Done
worker.rs   Worker: the thread, its queue, the results channel, the wake flag
budget.rs   Budget: frame interval, prediction, calibration
slot.rs     Slot, Shown: one cell's shown image, generation and pending state
```

**Public API added**

```rust
/// Where the widget rasterizes changed cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Render {
    /// Inline when the predicted cost is at most half the frame budget,
    /// else on the worker thread; results arrive as `Event::Waken`.
    #[default]
    Auto,
    /// Always on the UI thread, inside `RedrawRequested`.
    Inline,
    /// Always on the worker thread (tests, the benchmark, diagnostics).
    Worker,
}

/// Stage timings from an app-supplied clock.
#[derive(Debug, Clone)]
pub struct Probe {
    clock: fn() -> Instant,
    sink: mpsc::Sender<probe::Sample>,
}

impl Probe {
    pub fn new(clock: fn() -> Instant, sink: mpsc::Sender<probe::Sample>) -> Self;
}

impl<'a, Message> Plot<'a, Message> {
    pub fn render(self, render: Render) -> Self;
    pub fn probe(self, probe: Probe) -> Self;
}
```

`plot::probe`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Sample {
    /// UI-thread work inside one `RedrawRequested`.
    Stage { frame: u64, stage: Stage, duration: Duration },
    /// One cell rasterized (worker or inline), all its bands.
    Raster {
        frame: u64,
        generation: u64,
        target: Target,
        pixels: u32,
        /// The cell's `work` (Phase 17), for the cost model.
        work: u64,
        /// Time between planning and the start of the render.
        queued: Duration,
        duration: Duration,
        inline: bool,
    },
    /// What the `draw` after this `RedrawRequested` shows.
    Frame {
        frame: u64,
        at: Instant,
        generation: u64,
        /// Slots with an image of any generation.
        images: u32,
        /// Slots with no image at all (the theme background shows).
        missing: u32,
        /// Slots whose shown content hash differs from their current
        /// hash, gutters included.
        previews: u32,
        /// Bytes of handles drawn for the first time.
        uploaded: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage { Resolve, List }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target { Cell(u32) }
```

`frame` counts `RedrawRequested` events; `at` is that event's
`Instant` (no clock call). `generation` increments whenever the state
the widget shows changes (a list whose cell hashes differ from the
stored ones; Phase 20 adds applied input). `uploaded` is accumulated in
`update` as slots are promoted and reported and cleared by the next
`Frame` sample: `draw` takes `&self` and mutates nothing
(`core/src/widget.rs:70-79`). The `Frame` sample is sent from `update`
at the end of the `RedrawRequested` handling and describes what the
following `draw` shows. There are no `Text` or `Thin` stages
(figure's `draw_with` has no clock hook); the benchmark's rule puts
resolve, text, M4 and list into one UI-thread bucket, so nothing is
lost. Cell ids are positions in Phase 17's `Cells.cells`, stable while
the partition's rect sequence is unchanged; a different partition
gets new ids and every slot is new. Without a `Probe` no sample is sent
and no clock is read: library code never calls `Instant::now`; time
comes from `RedrawRequested(Instant)` and the probe's clock.

**Slots and generations** (`slot.rs`)

```rust
pub(crate) struct Slot {
    rect: [i32; 4],
    /// Hash of the cell's current content.
    hash: u64,
    /// The image on screen, the generation it was rendered for and the
    /// content hash it shows.
    shown: Option<Shown>,
    /// The generation a render is pending for, if any.
    pending: Option<u64>,
}

pub(crate) struct Shown { bands: Vec<Band>, generation: u64, hash: u64 }
```

`draw` draws `shown` at its device rect whatever its generation: a slot
never blinks to background once it has an image. `missing` counts slots
with `shown == None`; `previews` counts slots with `shown.hash !=
hash`, an image of outdated content, plot areas and gutters alike. A
slot whose content did not change is current whatever generation its
image came from. A result for generation `g` is promoted iff `g >
shown.generation` (or the slot has no image); it carries the hash it
rendered, so a promoted result older than the latest refresh still
counts as a preview until the newest one lands.

**Refresh, once per `RedrawRequested`**

1. Resolve and build the list through `state.cache` (timed as
   `Stage::Resolve` and `Stage::List`; Phase 20 adds the held layout
   and gesture overrides).
2. Cells from `list::cells(&list, &xs, &ys, k)`. If the rect sequence
   differs from the slots', rebuild the slots (all `shown: None`).
3. Changed = slots whose `hash` differs from their cell's. If none,
   stop (`rebuild_of_an_equal_figure_renders_nothing`). Otherwise
   `generation += 1`, store the new hashes and set `pending =
   Some(generation)` on the changed slots.
4. Predict (below). `Render::Inline`, or `Auto` with a cheap
   prediction: render every changed cell at once with the state's
   `raster::Renderer` (`Settings::parallel()`), one `Raster` sample each
   with `inline: true`, promote immediately. `Render::Worker`, or
   `Auto` with an expensive prediction: `worker.replace(jobs)` with the
   list and its bounds in an `Arc<List>` and an `Arc<list::Bounds>`,
   plot-area cells first, then gutters.
5. Send the `Frame` sample.

Rendering a cell is, per band, `renderer.render(&list, &bounds,
band_rect / k, k)`; `bounds = list::Bounds::of(&list, …)` is computed
once per list, so every cell and band shares it.

**Prediction** (`budget.rs`, `cost.rs`)

```rust
pub(crate) struct Budget {
    /// Last two RedrawRequested instants.
    last: [Option<Instant>; 2],
    /// EWMA of measured / modelled raster time, with a probe.
    calibration: Option<f64>,
    /// Raster samples folded into `calibration`.
    samples: u32,
}

/// The raster cost model (public so the benchmark can record it).
pub mod cost {
    /// Device pixels rasterized per millisecond.
    pub const PIXELS_PER_MS: f64 = 5.0e5;
    /// Work items (Phase 17's `Cell::work`) per millisecond.
    pub const ITEMS_PER_MS: f64 = 400.0;
    /// Modelled single-threaded time of one cell: `pixels /
    /// PIXELS_PER_MS + work / ITEMS_PER_MS`.
    pub fn model_ms(pixels: u64, work: u64) -> f64;
}
```

- **Budget:** the interval between the last two `RedrawRequested`
  instants, clamped to `1 ms ..= 16.7 ms`; 16.7 ms until two exist. An
  idle app's long gap must not make a 100 ms render look cheap; a
  144 Hz display gives 6.9 ms (frame budgets 16.7 / 8.3 / 6.9 ms at
  60 / 120 / 144 Hz, see Performance).
- **Model:** `model_ms` summed over the changed cells. Its constants
  come from two measurements of the wheel figure at 2× (1920 × 1520 =
  2,918,400 device px) on a 2-core container (see iced integration):
  the whole figure 16–25 ms, without its 5,600 dots 6.7 ms. The dots
  cost about 14 ms, ≈ 400 points per ms. The rest (2,918,400 px and
  ≈ 760 items) costs 6.7 ms, of which ≈ 1.9 ms are items, so 4.8 ms for
  the pixels, ≈ 6.1e5 px per ms, rounded down to 5e5. Check: whole
  figure 5.84 + 15.9 ≈ 21.7 ms (measured 16–25), without dots
  5.84 + 1.9 ≈ 7.7 ms (measured 6.7). The model is conservative on the
  target machine (vello_cpu's pool, more cores).
- **With a probe:** every `Raster` sample folds `r = measured /
  model_ms` into `calibration` (starting at the first `r`, then `c += (r
  − c) / 16`, α = 1/16); from the 8th sample on, prediction =
  `model_ms × calibration`. Without a probe the factor is 1.
- **Decision (`Render::Auto`):** inline iff prediction ≤ budget / 2.
- A changed cell with `shown == None` (first frame, resize, rescale,
  new partition) counts like any other: a cheap first frame renders
  inline (`missing == 0` at once), an expensive one shows the
  background for a frame.

**Worker** (`worker.rs`, `job.rs`)

```rust
pub(crate) struct Worker {
    queue: Arc<Queue>,                 // Mutex<Option<Vec<Job>>> + Condvar + closed flag
    done: mpsc::Receiver<job::Done>,
    /// Set by the worker before `wake`, cleared by the UI on drain:
    /// many finished cells cost one `Event::Waken`.
    woken: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub(crate) struct Job {
    target: probe::Target,
    generation: u64,
    /// The cell content hash this job renders.
    hash: u64,
    list: Arc<List>,
    bounds: Arc<list::Bounds>,
    rect: [i32; 4],
    bands: Vec<Range<i32>>,
    scale: f64,
    /// The cell's work items, echoed into its `Raster` sample.
    work: u64,
    /// Probe clock at planning time.
    enqueued: Option<Instant>,
}

pub(crate) struct Done {
    target: probe::Target,
    generation: u64,
    hash: u64,
    bands: Vec<Result<raster::Image, raster::Error>>,
    work: u64,
    queued: Option<Duration>,
    duration: Option<Duration>,
}
```

- The thread starts lazily with the first job set and owns one
  `raster::Renderer::new(Settings::parallel())`: vello_cpu's pool is
  the only parallelism, the widget adds none. A `RenderContext` built
  on the worker never crosses a thread; `Arc<List>` does (vello_cpu
  0.3.0 `src/render.rs:170-190` `RenderSettings { level, num_threads
  }`, `:201` `new_with`, `:738` `reset_and_resize`, `:806` `render`).
  It loops: wait for a job set, take it, render the jobs in order, send
  a `Done` per job, and after each job check whether a newer set
  replaced the queue; if so, abandon the rest (newest wins at the
  source).
- `Worker::replace(jobs)` swaps the pending set under the mutex and
  notifies; a job set is never appended to.
- After sending, the worker does `if !woken.swap(true, AcqRel) {
  waker.wake() }`. The waker is cloned from `shell.waker()` when the
  worker starts (`update` has the shell, `layout` does not;
  `core/src/shell.rs:80`, `Waker: Clone + Send + Sync` at `:254-279`;
  the runtime's waker sends `Event::Waken`, `core/src/event.rs:36`,
  `winit/src/window.rs:69-74`).
- On `Event::Waken`: `woken.store(false)`, drain `done` with
  `try_recv` and apply results in arrival order: if `generation >
  shown.generation` (or no image), promote as `Shown { bands,
  generation, hash }` with the hash the job rendered
  (`handle::from_image` per band, `uploaded +=` its bytes); otherwise
  drop it. `pending` clears when `generation == pending`. With a probe,
  each `Done` becomes one `Raster` sample (`inline: false`). Then
  `request_redraw`. An `Err` band keeps the slot's image and
  `debug_assert!`s the error (it means the list failed `list::check`).
- Timing: with a probe the job carries `enqueued` and the worker times
  the render with the probe's `clock` (a `fn` pointer is `Send`);
  without one both are `None`.
- `Drop for Worker`: set `closed`, `notify_all`, join. A render in
  flight finishes first (one cell, bounded by the largest gutter). The
  thread is named through `std::thread::Builder::name` so tests can
  count it.
- The harness ticks the renderer every frame, so a wgpu frame that
  draws new handles completes their uploads; in the windowed runtime
  winit does the same (`winit/src/lib.rs:1789-1791`,
  `core/src/renderer.rs:96`). One `RedrawRequested` instant serves the
  up to three passes of a frame (`winit/src/lib.rs:777-830`).

**Decisions this phase may meet**

- The cost model's constants misjudge the target machine — keep them
  as the no-probe fallback; the probe's calibration calibrates them
  after 8 samples; Phase 22's benchmark records the measured factor
  and its verdict may move the constants; record it as a decision.
- A single gutter cell at 4K (3840 × 300 device px, 4,608,000 B → 3
  bands of at most 135 rows) takes longer than a frame on the worker —
  accept it: the previous image stays; Phase 31 merges and tiles if the
  benchmark verdict calls for it; record it as a decision.

#### Steps

1. **Slots, hash skipping, probe.** `slot.rs`, `probe.rs`,
   `Plot::probe`; refresh plans only changed cells and renders them
   inline; `Frame` sent from `update`. Unit tests through `State`:
   `rebuild_of_an_equal_figure_renders_nothing`,
   `unchanged_cells_are_skipped` (a two-panel figure; change panel 1's
   dots: the `Raster` targets are that panel's plot-area cell only),
   `probe_reports_every_stage` with a test clock (`fn clock() -> Instant
   { Instant::now() }` under `#[expect(clippy::disallowed_methods,
   reason = "test clock")]`).
2. **Budget and prediction.** `budget.rs`, `cost.rs`, `render.rs`,
   `Plot::render`. Unit tests: `budget_is_the_last_interval_clamped`,
   `cost_model_matches_the_spot_rows`, `cheap_cells_are_predicted_inline`
   and `expensive_cells_are_predicted_for_the_worker` (both directions
   of the rule on constructed pixel and work counts),
   `calibration_applies_after_eight_samples`.
3. **Worker.** `worker.rs`, `job.rs`; `Render::Auto` is the default.
   The harness gains `wait_for_wake(timeout)` (polls `wakes()` with
   1 ms sleeps). Harness tests: `worker_mode_matches_inline_mode` (a
   1600 × 1000 logical figure at 2.0 with 200k dots, `Render::Worker`:
   `harness.frame(t0)` reports `missing > 0`; then repeat
   `wait_for_wake(10 s)`, `harness.events(&[Event::Waken])`,
   `harness.frame(t)` until the `Frame` sample reports `missing == 0`
   and `previews == 0`; the screenshot equals the `Render::Inline` one
   within 1/255), `cheap_refresh_renders_inline` (a 320 × 200 figure at
   2.0: first frame `missing == 0`, no wake),
   `expensive_refresh_uses_the_worker` (the 200k-dot figure under
   `Render::Auto`: model ≈ 513 ms; first frame `missing > 0`;
   `redraw_requested()` is `NextFrame` after the `Waken`),
   `worker_exits_on_drop` (a count of threads with the worker's name
   before and after dropping the harness).
4. **Newest wins and previews.** Tests: `newest_result_wins` (two
   refreshes with different figures before a drain: only generation 2's
   images are shown; a generation-1 `Done` arriving after is dropped,
   injected by holding the drain),
   `previous_image_stays_until_the_new_one_arrives` (change a legend
   title under `Render::Worker`: the frame before `Waken` still shows
   the old title pixels in that gutter cell and reports `previews == 1`,
   every other slot being current; after `Waken`, `previews == 0`).
5. **Docs, `check.sh full`.** Doc comments for `Render`, `Probe` and
   `Sample` with the preview definition.

Verify with `scripts/check.sh quick` and both `ICED_TEST_BACKEND`
values; `worker_mode_matches_inline_mode` prints the wait for the first
wake and the max diff, `probe_reports_every_stage` the sample counts
and the measured rate.

#### Exit criteria

1. `worker_mode_matches_inline_mode` passes on both backends.
2. `cheap_refresh_renders_inline` and `expensive_refresh_uses_the_worker`
   pass.
3. `unchanged_cells_are_skipped` (only the changed panel's plot-area
   cell renders, counted from `Sample::Raster`) and
   `rebuild_of_an_equal_figure_renders_nothing` pass.
4. `newest_result_wins` and
   `previous_image_stays_until_the_new_one_arrives` pass on both
   backends.
5. `probe_reports_every_stage`: one refresh in each mode yields `Stage`
   samples for `Resolve` and `List`, one `Raster` sample per rendered
   cell and one `Frame` sample per `RedrawRequested`.
6. `worker_exits_on_drop` passes; `scripts/check.sh quick` per commit,
   `full` at the end on both `ICED_TEST_BACKEND` values.

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| Last two `RedrawRequested` 6.9 ms apart | budget 6.9 ms | interval |
| 2 s apart; 0.4 ms apart; one instant only | 16.7 ms; 1 ms; 16.7 ms | clamp `1 ..= 16.7` |
| 320 × 200 at 2× (256,000 px), 150 work items, no probe, 16.7 ms budget | 0.512 + 0.375 = 0.89 ms ≤ 8.35 → inline | model |
| Wheel figure at 2× (2,918,400 px, ≈ 6,360 items), no probe | 5.84 + 15.9 ≈ 21.7 ms > 8.35 → worker | measured 16–25 ms single-threaded |
| Same without its 5,600 dots (≈ 760 items) | 5.84 + 1.9 ≈ 7.7 ms ≤ 8.35 → inline at 60 Hz | measured 6.7 ms |
| 144 Hz (6.9 ms) | inline threshold 3.45 ms | budget / 2 |
| 200k dots, 1600 × 1000 logical at 2× (6.4e6 px) | 12.8 + 500 ≈ 513 ms → worker | model |
| Calibration 0.5 after 8 samples; 3 changed cells of 300 × 200 px with 100 items each | (0.36 + 0.75) × 0.5 = 0.56 ms → inline | model × calibration |
| Two refreshes before a drain, generations 1 and 2 | only generation 2 shown; a later generation-1 `Done` dropped | newest wins |
| Shown g = 3, pending g = 5, `Done` for g = 4 arrives | promoted (4 > 3); its hash ≠ the current hash, so `previews == 1` until g = 5 lands | preview definition |
| A refresh changes panel 1 only (g = 7) | panel 0's slot keeps its g = 6 image and is current; `previews` counts panel 1 alone until its render lands | content hash, not generation |
| A slot with `shown == None` while its job is pending | counted in `missing`, not `previews` | |
| One wheel panel, culled, at 2× | ≈ 4.4 ms; whole figure 16–25 ms | `proto/dlist/src/bin/partial.rs` |
| No `Probe` set on the `Plot` | no `Sample` sent, no clock read | |
| `available_parallelism` 2 / 16 | `Settings::parallel()` threads 1 / 8 | vello_cpu's own rule |

#### Not in this phase

- Gestures, `on_change`, held layout, scaled previews (Phase 20).
- Live data, `Fit::Window`, `Plot::revision`, `Plot::caching`
  (Phase 21).
- The full-frame benchmark (Phase 22).
- Tiles, pan translation, overscan, a multi-thread pool,
  `Plot::threads`, `Caching::Verify`, cell merging (Phase 31,
  conditional on the benchmark verdict).
- `allocate_image` and any residency logic (every image < 2 MiB).
- Any change to the figure crate.

### Phase 20 — Plot III: gestures

*M2 · builds on phase 19 · about 1,200 lines (≈700 library, ≈500 tests), 6 commits*

#### Goal

The `Plot` widget zooms and pans. A wheel over a plot area zooms every
handle the panel zooms (`.zoom_x`, `.zoom_y`) around the cursor; a
wheel over an axis zooms that axis's handle only; a left-button drag
pans. Each handle has its own core `Zoom`. Input accumulates between
frames and is applied once per `RedrawRequested`.
`Plot::on_change(handle, f)` turns a handle's changes into typed
messages, so the app owns that domain; otherwise the widget keeps it,
keyed by (handle index, `TypeId`). During a gesture the widget
continues from the domain it last published and drops it when the
gesture ends, so batched updates lose no distance. The layout holds
still during a gesture and settles once when it ends (a wheel gesture
ends 300 ms after its last event). While a plot-area cell's sharp
render is pending, its previous image is drawn scaled by one affine
map per axis, so a zoom or pan shows at once; gutters keep their last
image. A rejected zoom step (non-finite, reversed or sub-precision
span) leaves the gesture unchanged. A double-click resets the touched
handles to their fitted domains, and `Pointer::Left` clears the
overlay.

#### Builds on

- Phase 19: `Plot` with `Render`, `Probe`, slots with `Shown { bands,
  generation, hash }` and `pending`, one worker, generations,
  inline-when-cheap prediction, the `Frame` sample with `previews`.
- Phase 10: `interact::Pointer` (`Pressed { x, y, button, modifiers }`,
  `Moved { x, y }`, `Released { button }`, `Wheel { x, y, delta:
  Delta, modifiers }` with `Delta::{Lines, Pixels}`, `DoubleClicked`,
  `Modifiers(…)`, `Left`); `interact::Transform` (`k, x, y`); `Zoom`
  (`scale_extent`, `handle(pointer, transform) -> Option<Transform>`,
  `None` when the constrained transform equals the current one; d3's
  `defaultWheelDelta`; horizontal deltas pan); `interact::Rescale` with
  `rescale_x`/`rescale_y`, `pan_px`, `zoom_px(anchor, factor)`
  (`None` for a rejected step: non-finite, reversed, span below
  2⁻⁴⁰·|center|), `scale::Pad`. Their doc comments set the sign
  conventions; this phase states behaviour as invariants, not signs.
- Phase 11: `pointer::Tracker` (`new`, `scale_factor(f)`,
  `translate(&Event, bounds, cursor) -> Option<Pointer>`,
  `wants_redraw()`): wheel deltas negated to d3's sign, pixel deltas
  divided by the scale factor; `Moved` continues outside the bounds
  while a button is held; `Left` once; a press waits for the next
  `RedrawRequested(t)` to be stamped, and translating that
  `RedrawRequested` yields `DoubleClicked { x, y }` when the previous
  press of the same button is ≤ 300 ms older and < 6 px away (iced's
  rule, `core/src/mouse/click.rs:73-84`); no `Instant::now`
  (`mouse::Click` calls it, `core/src/mouse/click.rs:44`). The harness:
  `Harness::new`, `point_at`, `events`, `frame(now)`, `messages`.
- Phase 14: figure's object-safe `Scale` (`as_any`, `boxed`,
  `with_range`, `pan_px`, `zoom_px`) delegating to core; `Kind::domain()`
  (end stops in the domain's own type, `None` for band, point,
  ordinal); `ScaleRef::index()`; `Held::{hold, release, get, prune}`
  (`release` makes the handle refit); resolve with per-handle
  overrides, a held handle staying held until reset.
- Phase 16: `layout::Areas` (`Clone + Debug + PartialEq`, carrying the
  size and panel count it was made for), `Resolved::areas()`,
  `Figure::resolve_held` and `draw_held` (rects kept, labels grow
  outward; a mismatched `Areas` is ignored), `Resolved::held()`,
  `Resolved::passes()`, and two characters of slack per labeled
  vertical axis on `Target::Screen`.
- Phase 17: hit testing over the resolved figure (plot areas, points)
  and `Resolved::plot_areas()`.

#### Design

**Public API added**

```rust
impl<'a, Message> Plot<'a, Message> {
    /// Changes to `handle`'s domain become messages; the app owns the
    /// domain from then on. Without this, the widget keeps it.
    pub fn on_change<S>(
        self,
        handle: ScaleRef<S>,
        f: impl Fn([S::Domain; 2]) -> Message + 'a,
    ) -> Self
    where
        S: Kind + 'static;
}
```

`Kind` and `ScaleRef` are figure's. The closure gets the domain in the
scale's own type (`[Timestamp; 2]` for a time handle), which is what
makes a closure written for `f64` fail with E0631; that `compile_fail`
doctest and its twin are Phase 33's. Only continuous handles change
(figure's `Scale` pixel operations return `None` for band and point
scales).

The figure crate gains, in Step 1, `Panel::zoom_x(h)` and
`Panel::zoom_y(h)` (`&mut self -> &mut Self`, any number of handles per
axis: `.zoom_y(vib).zoom_y(temp)` zooms twins together in every panel)
in `figure/src/panel.rs`, and `Resolved::zooms(&self, panel: usize) ->
(&[usize], &[usize])` (handle indices zoomed along x, along y) in
`figure/src/figure/resolved.rs`.

**Modules** (`iced/src/plot/`):

```text
listener.rs  Listener<'a, Message>: on_change closure, type-erased
domains.rs   Domains: widget-owned and pending domains, touched set, zooms
input.rs     Input: pointer events queued until the next redraw
region.rs    Region: what the pointer is over (plot area or axis)
gesture.rs   Gesture: the active drag or wheel gesture
preview.rs   Affine1: the scaled-preview map of one axis
```

```rust
pub(crate) struct Listener<'a, Message> {
    key: domains::Key,
    publish: Box<dyn Fn(&dyn figure::Scale) -> Option<Message> + 'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Key { index: usize, scale: TypeId }

pub(crate) struct Domains {
    /// Handles nobody listens to: the widget owns them.
    owned: HashMap<Key, Box<dyn figure::Scale>>,
    /// Published during the active gesture, until it ends.
    pending: HashMap<Key, Box<dyn figure::Scale>>,
    /// Changed since the last reset.
    touched: HashSet<Key>,
    /// One core Zoom per handle, persistent.
    zooms: HashMap<Key, Zoom>,
}

pub(crate) enum Region {
    Area { panel: usize },
    Axis { panel: usize, key: Key, along: Along },
    Outside,
}

pub(crate) enum Along { X, Y }

pub(crate) struct Gesture {
    region: Region,
    kind: gesture::Kind,
    /// Per handle: its axis, the transform since the gesture began
    /// and the scale at that moment.
    tracks: Vec<Track>,
    /// Plot areas at the gesture's start (held layout).
    held: Option<layout::Areas>,
}

pub(crate) enum Kind { Drag, Wheel { last: Instant } }

pub(crate) struct Track {
    key: Key,
    along: Along,
    transform: interact::Transform,
    start: Box<dyn figure::Scale>,
}
```

Each `Listener` is built in `on_change` from `handle.index()` and
`TypeId::of::<S>()`; `publish` downcasts through `as_any` and calls
`f(domain)`. Listeners live in the `Plot` (they borrow `'a`);
everything in `State` is `'static`. `Domains` entries whose index
names a scale of another `TypeId` are dropped at the next refresh (the
app reordered its handles). Core's `Zoom` keeps d3's `[0, ∞]` scale
extent, which does not bound a gesture's `k`; the widget builds each
handle's `Zoom` as `Zoom::default().scale_extent([1e-6, 1e6])`, so `k`
stays finite across a long wheel burst, while the figure's pixel
operations still reject degenerate spans (stated in `domains.rs`'s doc
comment).

**Event handling.** `State` owns a `pointer::Tracker`, kept in sync
with the effective scale factor. `update` passes every event through
`tracker.translate(event, bounds, cursor)` (bounds = the widget's, so
coordinates are widget-local logical px), requests a redraw while
`tracker.wants_redraw()`, and:

| Input | Immediate action |
| --- | --- |
| `Pressed { button: Left }` over a region with zoomable handles | queue; `capture_event`; `request_redraw` |
| `Moved` while a drag is active or queued | queue; `request_redraw` |
| `Moved` otherwise | Phase 18's hover only |
| `Released { button: Left }` | queue; `request_redraw` |
| `Wheel` over a region with zoomable handles | queue; `capture_event`; `request_redraw` |
| `Modifiers(…)` | queue (core `Zoom` reads them) |
| `Left` | clear the overlay; an active drag continues until `Released` (a later `Pressed` ends a drag whose release was lost) |
| `DoubleClicked` (yielded while translating `RedrawRequested`) | queue as a reset request |
| `RedrawRequested(now)` | translate it first, then `apply(now)`, then Phase 19's refresh |

`Input` stores queued pointers in order; `apply(now)` stamps queued
wheel events with `now`, which times the 300 ms wheel end (input is
timed by the redraw it requested, at most one frame late; presses are
already stamped by the tracker). iced at the pin:
`core/src/shell.rs:91,113,130,135` (`publish`, `capture_event`,
`request_redraw`, `request_redraw_at`),
`core/src/window/redraw_request.rs:4-19`.

**`apply(now)`, once per `RedrawRequested`**

1. **Double-click.** A queued `DoubleClicked` is a reset (below); the
   drag its second press started ends without moving anything.
2. **Gesture boundaries.** A `Pressed` in a zoomable region starts a
   `Drag` on that region; a `Wheel` with no gesture starts a `Wheel`
   gesture on its region (a wheel during a drag feeds the drag's
   tracks). Starting a gesture snapshots each track's current scale
   (`owned`, else `pending`, else the resolved one) and the held
   `layout::Areas` (`Resolved::areas()`).
3. **Tracks.** Region `Area { panel }`: every handle in the panel's x
   list with `Along::X` and in its y list with `Along::Y`. Region
   `Axis`: that one handle, if the panel zooms it. Each pointer is fed
   to every track's `Zoom::handle(local, track.transform)`, where
   `local` is the pointer relative to the panel's held plot area; an
   `X` track reads `(k, x)` of the result, a `Y` track `(k, y)`. A drag
   on a panel with only y handles pans y only; horizontal motion is
   ignored.
4. **Domains.** For each track whose transform changed, the new scale
   is `track.start` rescaled by the transform (core's `rescale_x` /
   `rescale_y` semantics through figure's `pan_px`/`zoom_px`).
   Invariant, tested: for every output position `p` of the start
   scale, `new.apply(start.invert(p)) = p · k + offset`. Transforms
   accumulate from the gesture's start, never step to step, so floating
   error does not compound. When `zoom_px`/`pan_px` returns `None` for
   a track (a rejected step), that pointer event leaves the track's
   `transform` unchanged: nothing is published and `touched` is not
   marked; the next event works from the last accepted transform.
5. **Ownership.** For a listened key, store the scale in `pending` and
   publish **one** message per key per `apply` (the latest domain); for
   other keys, store it in `owned`. Mark keys `touched`; bump
   `generation` if anything changed.
6. **Gesture end.** A drag ends at its `Released`; a wheel gesture at
   the first `apply` with `now ≥ last + 300 ms` (d3's own wheel idle is
   150 ms, `proto/d3-src/d3-zoom/src/zoom.js:68`; 300 ms keeps the
   layout held across a burst). After every wheel event,
   `shell.request_redraw_at(last + Duration::from_millis(300))`. Ending
   clears `pending` (the app's domain rules from then), drops `held`
   and marks the list for rebuilding, so the layout settles in one pass.
7. **Skip the list when publishing.** If step 5 published, refresh does
   not build the list this pass: the runtime runs the app's `update`,
   rebuilds the view and sends `RedrawRequested` again
   (`winit/src/lib.rs:788-830`); that pass has no new input, publishes
   nothing and builds the list once, with the app's new figure plus
   `pending`. Without listeners nothing is skipped.

Refresh resolves with overrides `owned ∪ pending` (pending wins),
passed to the figure through `Held`, and, while a gesture is active,
with the held areas through `resolve_held`/`draw_held`; the figure
resolves on `Target::Screen`, so Phase 16's slack is reserved. Drags
are not snapped to device pixels: each drag frame re-renders the moved
plot-area cells and the gutters that show the panned handle (inline
when Phase 19's prediction allows, else on the worker with a preview).

**Scaled previews** (`preview.rs`)

```rust
/// One axis of a preview map, device px: p_current = scale · p_rendered + offset.
pub(crate) struct Affine1 { scale: f64, offset: f64 }
```

While a plot-area slot shows outdated content (`shown.hash != hash`)
and its sharp render is pending, `draw` shows the slot's shown bands at
mapped bounds instead of their device rect. Per axis two points determine
the map, the cell's two edges: `p_c = current.apply(rendered.
invert(p_r))`, where `rendered` is the handle's scale the shown image
was drawn with and `current` the one just resolved. Figure scales apply
pixel-space operations affinely in output space, so two points determine
the map for every continuous transform; for a handle without
`pan_px`/`zoom_px` (band, point) the axis map is the identity. Preview
bands are drawn with `FilterMethod::Linear` and `clip_bounds` = the
cell's device rect, and look soft until the sharp bands arrive; a cell
never rendered shows the theme background (`missing`). Gutter cells
keep their last image unmapped: an axis strip stretched along its axis
would show wrong labels. `Frame.previews` counts every slot whose shown
content hash differs from its current one, plot areas and gutters
alike.

**Reset.** A double-click on a panel resets every touched key the
panel zooms (others may belong to panels the user is not looking at):
`owned` entries are removed (the figure refits and the hold ends);
listened keys publish the fitted domain (the handle resolved with no
override) and leave `pending`; `touched` drops them; their `Zoom`s are
reset to their initial state. Core's own `Zoom` keeps d3's
double-click zoom-in; the widget never forwards `DoubleClicked` to it.

**Pointer feedback.** `mouse_interaction`: `Grabbing` during a drag,
`Grab` over a plot area with zoomable handles, the resize cursor of the
axis direction over a zoomable axis box, `Crosshair` over other plot
areas, `None` elsewhere (`core/src/mouse/interaction.rs`).

Evidence: `proto/figmodel/src/live.rs` (routing input to a panel's
gesture; `views` for unowned scales, `pending` for published ones,
cleared when a gesture ends), `proto/figmodel/examples/a_live.rs`
(controlled vs widget-owned, 1 vs 3 events per update: without
`pending`, a 120 px pan delivered as three events per update lost half
its distance), `proto/figmodel/examples/layout_motion.rs` (per-event
relayout moved a panel's left edge 45 → 51 → 57 px over eight wheel
steps; held, 0 px, labels growing outward by at most 12 px),
`proto/d3-src/d3-zoom/src/zoom.js:34-36` (`defaultWheelDelta`:
`−deltaY × (lines ? 0.05 : pixels ? 0.002 : 1) × (ctrl ? 10 : 1)`,
factor `2^delta`) and `:234-262` (`wheeled`).

**Decisions this phase may meet**

- Phase 17's hit test returns no axis boxes — derive each axis box from
  its panel's plot area, side and measured reach in the resolved
  layout, inside `region.rs`; record it as a decision.
- `Held` takes typed scales only while the widget's overrides are
  erased (`Box<dyn figure::Scale>` keyed by index and `TypeId`) — add an
  additive `Held::hold_erased(&mut self, index: usize, scale: Box<dyn
  Scale>)` in `figure/src/figure/held.rs` (its storage is already
  `(TypeId, Box<dyn Scale>)`); record it as a decision.
- figure's object-safe `Scale` cannot map a position for the preview's
  two points — form them from the tracks' transforms instead (`T_c ∘
  T_r⁻¹`, equal to `current.apply(rendered.invert(p))` by the invariant
  of `apply` step 4), keeping with each shown plot-area image the
  per-axis transform it was planned at; record it as a decision.

#### Steps

1. **Domains, listeners, `on_change`.** Figure first: `Panel::zoom_x`,
   `Panel::zoom_y`, `Resolved::zooms`; then `listener.rs`,
   `domains.rs`; refresh resolves with overrides; type-mismatch drop.
   Unit tests: `state_is_dropped_when_the_handle_type_changes`,
   `owned_domains_survive_rebuilds`.
2. **Input and wheel zoom.** `input.rs`, `region.rs`, `gesture.rs`
   (wheel only); publish once per apply; rejected steps. Harness tests:
   `wheel_zoom_keeps_the_point_under_the_cursor` (data value under the
   cursor before and after within 1e-12 relative),
   `wheel_over_an_axis_zooms_only_that_axis`,
   `five_wheel_events_publish_one_message`, `twin_handles_zoom_together`
   (two y handles on one panel get the same `k` and anchor; a second
   panel sharing one of them follows), `sub_precision_zoom_is_ignored`
   (x domain `[1e9, 1e9 + 1e-3]`, threshold 2⁻⁴⁰ · 1e9 ≈ 9.09e-4;
   zoom-in steps of one line each, factor 2^0.05 ≈ 1.0353, give spans
   9.66e-4 and 9.33e-4; the third, 9.01e-4, is rejected, so the
   published domain stops changing after two steps; the next zoom-out
   step works from the last accepted transform; the `[1e-6, 1e6]`
   extent is not reached, k ≈ 1.07).
3. **Drag pan and gesture sync.** Drag tracks, `capture_event`,
   `Released`, lost-release handling. The harness gains
   `rebuild(root)` and the tests a `run_frame(app)` helper mirroring
   `winit/src/lib.rs:788-830`: update `RedrawRequested`; if messages
   were published, apply them to the test app, rebuild the
   `UserInterface` from its view and repeat (at most 3 times); then
   draw. Tests: `drag_pans_every_handle_the_panel_zooms`,
   `pan_survives_batched_updates` (controlled), the same with the
   widget owning the domain (final domains equal across both).
4. **Scaled previews.** `preview.rs`; `draw` maps pending plot-area
   slots. Unit tests `affine_map_from_two_points`,
   `preview_dropped_when_the_cell_is_sharp`; harness test
   `zoom_shows_a_scaled_preview_at_once` (`Render::Worker`: the frame
   after one wheel step has `previews > 0`, `missing == 0`; after
   `Waken` and one frame it equals an inline render within 1/255).
5. **Held layout, wheel end, reset, `Left`.** Tests:
   `layout_holds_during_a_wheel_gesture` (redraw instants advanced by
   16 ms per step, then one at +300 ms; relayouts counted through
   `Resolved::held()` and the rects of `Resolved::plot_areas()`),
   `double_click_resets_touched_handles`,
   `slow_double_click_does_not_reset`, `cursor_left_clears_the_overlay`,
   `zoomed_pixels_match_a_figure_built_with_the_published_domain`
   (`Render::Inline`; ≤ 1/255 against a figure whose handle has that
   domain).
6. **Docs, `check.sh full`.** A `no_run` example with
   `.on_change(x, Message::Window)`.

Verify with `scripts/check.sh quick` and `ICED_TEST_BACKEND=tiny-skia|wgpu
cargo test -p rustd3-iced --features figure`.

#### Exit criteria

1. Every test named under Steps passes with `ICED_TEST_BACKEND=tiny-skia`
   and `wgpu`.
2. `pan_survives_batched_updates`: a 120 px drag delivered with an app
   update after every event, after every third event, and once at the
   end publishes the same final domain (relative difference ≤ 1e-12).
3. `layout_holds_during_a_wheel_gesture`: across eight wheel steps the
   plot-area rect does not move; one relayout happens at the first
   `RedrawRequested` at least 300 ms after the last wheel event.
4. `double_click_resets_touched_handles` and
   `slow_double_click_does_not_reset` pass with presses stamped 200 ms
   and 400 ms apart.
5. `zoom_shows_a_scaled_preview_at_once` and
   `sub_precision_zoom_is_ignored` pass on both backends.
6. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| iced `Lines { y: 1 }` (scroll up), translated to `Lines { y: −1 }` | d3 delta +0.05: zoom in by 2^0.05 = 1.0353 | `defaultWheelDelta` |
| 100 px of pixel delta | 2^0.2 = 1.1487; with ctrl, ×10 in the exponent | `defaultWheelDelta` |
| Presses stamped 200 ms apart, 3 px apart | tracker yields `DoubleClicked` → reset | iced's click rule |
| Presses 400 ms apart, or 8 px apart | two single presses, no reset | |
| 120 px pan as 6 moves of 20 px, app updated every 3 events | same domain as with an update per event | `pending` |
| Eight wheel steps, relayout per event | left edge 45 → 51 → 57 px; held: 0 px, labels outward ≤ 12 px | `layout_motion.rs` |
| Last wheel at t, redraws at t + 16 ms … t + 288 ms | gesture active, layout held | |
| First redraw at ≥ t + 300 ms | gesture ends; one relayout; `pending` cleared | |
| Wheel at x = 100 px over linear x [0, 100], range [0, 400] px, factor 2 | domain [12.5, 62.5]; data 25 stays at 100 px | zoom about the cursor |
| Drag of +40 px on that handle (domain [0, 100], 400 px) | domain [−10, 90] (content follows the pointer) | |
| Handle index 1 changes from `Linear` to `Log` between views | widget-owned entry for (1, `Linear`) dropped | key = (index, `TypeId`) |
| x [0, 100] over 400 device px, zoom ×2 about 200 px → [25, 75] | `Affine1 { scale: 2, offset: −200 }`; a cell over device columns [0, 256) previews at [−200, 312) | p_c = 8 (p_r / 4 − 25) |
| x `[1e9, 1e9 + 1e-3]`, three one-line zoom-in steps | spans 9.66e-4, 9.33e-4 accepted; 9.01e-4 < 9.09e-4 rejected | 2⁻⁴⁰·|center| |

#### Not in this phase

- Live data, `Fit::Window`, `Plot::revision`, `Plot::caching`
  (Phase 21); the full-frame benchmark (Phase 22).
- Snapped drags, tile translation, overscan, held column origins and
  dash phase, the pool and `Plot::threads`, newest wins per tile
  (Phase 31, conditional).
- Hover messages and brushing (Phase 36); the E0631 compile-fail
  doctest and time handles (Phase 33, which adds a time-handle wheel
  test).
- Box, pinch and axis-drag zoom, linked cursors (out of scope).

### Phase 21 — Plot IV: live data

*M2 · builds on phase 20 · about 600 lines (≈150 figure, ≈200 iced, ≈250 tests), 4 commits*

#### Goal

Figures follow live recordings. `Fit::Window(span)` keeps an x handle
on the newest `span` of data, quantized to whole device columns so
successive updates move the domain by an integer number of columns; a
gesture holds the handle until a double-click reset. An append changes
the hashes of the parts that read the recording and of nothing else,
so Phase 19's hash skipping re-renders exactly the touched cells (plus
the x gutters when the window moved). An append bumps the recording's
revision, which is part of the M4 memo key, so the memo misses and M4
runs over the visible view from the index (about 0.5 ms for a full view
of 1M samples). `Plot::revision(u64)` forces a full re-render for
changes neither hashes nor the figure can see, and
`Plot::caching(Caching::{On, Off})` is the diagnostic switch the
benchmark uses. `drop_front` rebuilds the index and re-renders the
cell like any append.

#### Builds on

- Phase 20: gestures with `Domains` (owned, pending, touched), `apply`,
  held layout, double-click reset, scaled previews, `generation`.
- Phase 19: slots with per-cell hash skipping, inline-when-cheap, the
  worker, `plot::cost::model_ms`, `probe::Sample::{Raster, Frame}`.
- Phase 17: `Fit::Visible`; part hashes (FNV-1a over emitted commands,
  so a changed `Markers` or `Stroke` command changes its part's hash);
  `Resolved::plot_areas()`.
- Phase 15: thinning with columns from `invert` at device-pixel edges
  and `thin::Memo` keyed by (recording id, revision, channel, column
  grid), inside `figure::Cache`.
- Phase 14: `pub enum Fit<Span> { All, Window(Span), Visible }`,
  `Kind::Span` (`f64` here), `Figure::fit_with<S: Fittable>`; `Window`
  resolves as `All` until this phase.
- Phase 9: `Recording<X>` with `id()`, `revision()`, `extend`
  (incremental index), `drop_front` (rebuilds the index), `locate(x)`,
  `extremes(range, valid)`.

#### Design

**`Fit::Window`** (figure: `scale/fit.rs`, `figure/resolve.rs`). For an
x handle with `Fit::Window(span)` and a resolved plot-area width of `w`
device columns (the widest panel using the handle: plot width × scale
× k, with k = 1 for `Target::Screen` and k = 4 for `Target::Vector` at
scale 1 and `Target::File`, Phase 15's column count):

```text
last  = max over the handle's marks of the last admitted x
        (owned points: their maximum; `Series` views: the bridge's
        series_extent(…).x[1], Phase 14)
col   = span / w
right = ceil(last / col) · col
left  = right − w · col
```

The domain is `[left, right]`, with no margin on x: the window is the
view. With no admitted data the handle fits as `All` (`[0, 1]`).
Appending samples moves `right` by whole multiples of `col`, so the
column boundaries `invert(edge_i)` of the new view are the old ones
shifted by an integer count and the thinned polyline's columns line up
from one update to the next (the line does not shimmer as data
streams in). `col` is computed in `f64`; correctness never depends on
exact alignment. A `Window` handle held by a gesture (Phase 20's
`owned`/`pending`) uses the held domain; a reset releases it and the
window follows again. `Window` on a y handle behaves as `All`, and
`Window(())` on a categorical handle is meaningless (`fit_with`
`debug_assert!`s against it); both are documented on `Fit::Window`.

```rust
impl<'a> Figure<'a> {
    /// `Fit::Window` is implemented by this phase.
    pub fn fit_with<S: Fittable>(&mut self, s: S, fit: Fit<S::Span>) -> ScaleRef<S>;
}
```

**Live appends** (iced: no new mechanism). An `extend` bumps the
recording's revision; the next `view` builds a figure over the grown
recording; the next `RedrawRequested` resolves, thins (the memo misses
because the revision is in its key) and builds the list; the line
part's `Stroke` command differs, so its part hash and its cell's hash
differ, and Phase 19 renders that cell (inline when cheap: a plot-area
cell of a line figure is far under the bound). With `Fit::Window` the
x gutters' tick labels move too, so those cells re-render; y gutters
are untouched unless a `Fit::Visible` y handle moved. The widget keeps
no recording state: hashes are the dependency tracker. Reusing the
memo per column across a shift, and shifting images instead of
re-rendering them, belong to Phase 31.

**`Plot::revision`, `Plot::caching`** (`plot/caching.rs`):

```rust
/// How the widget reuses rendered images (a public diagnostic option).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Caching {
    /// Re-render a cell only when its hash changed.
    #[default]
    On,
    /// Re-render every cell whenever the list changes.
    Off,
}

impl<'a, Message> Plot<'a, Message> {
    pub fn caching(self, caching: Caching) -> Self;
    /// Bump when something the hashes cannot see changed (a paint
    /// image's pixels replaced in place, a font swapped); every cell
    /// re-renders at the next refresh.
    pub fn revision(self, revision: u64) -> Self;
}
```

`State` stores the last `revision`; a different value marks every slot
changed. `Caching::Off` treats every cell as changed whenever any hash
differs (an unchanged list still renders nothing).

**`drop_front`.** `Recording::drop_front` drops whole chunks, rebuilds
the index and bumps the revision; the memo misses, M4 runs for the
view, the part hash changes and the cell renders. Nothing in the widget
distinguishes it from an append.

Evidence: M4 from the index 0.54 ms for a full view and 0.22 ms for a
5% view of 1M samples, column boundaries 38 µs (see Performance);
`proto/figmodel/src/live.rs` (the live loop), `proto/thin2/src/main.rs:46-54`
(column boundaries).

**Decisions this phase may meet**

- Two panels of different widths share a `Window` handle, so the
  narrower one's columns are not integral (`w` is the widest panel's) —
  accept it: its column edges do not line up between updates, the
  pixels are right; document it on `Fit::Window`; record it as a
  decision.
- A `Window` handle's marks read recordings on different clocks —
  `last` is the maximum over them; document it; record it as a
  decision.

#### Steps

1. **figure: `Fit::Window`.** `figure/src/scale/fit.rs`,
   `figure/src/figure/resolve.rs`; the quantization. Tests in
   `figure/tests/main/fit.rs`: `fit_window_moves_in_whole_columns`
   (spot rows), `fit_window_with_no_data_is_all`,
   `fit_window_column_edges_shift_by_whole_columns` (append 1,000
   samples: the new view's column boundaries equal the old ones shifted
   by an integer count, compared in `f64` within 1e-9 of a column).
   Verify: `cargo test -p rustd3-figure --all-features fit`.
2. **iced: `Plot::revision`, `Plot::caching`.** `iced/src/plot.rs`,
   `plot/caching.rs`, `plot/state.rs`. Tests:
   `revision_change_re_renders_everything` (`Raster` count = cell
   count), `caching_off_renders_every_cell`.
3. **iced: live tests** in `iced/tests/main/plot.rs`:
   `live_append_re_renders_only_touched_cells` (trace figure, x domain
   set wider than the data; append 1,000 samples to channel 0: the
   `Raster` targets are panel 0's plot-area cell only);
   `fit_window_follows_in_whole_device_columns` (`Fit::Window(10.0)`, 10
   appends of 1,000 samples at 10 kHz: each frame's resolved domain
   moves by a whole number of device columns, `(right' − right) / col`
   within 1e-9 of an integer, 10 for a plot area 1,000 device px wide;
   the screenshot after each append equals `png_of` the same figure
   within 1/255); `ten_hz_appends_stay_sharp` (10 appends 100 ms apart
   by redraw instants on an 800 × 500 logical two-panel trace of 1M
   samples at 2×, `Render::Auto`, no probe: first assert that the
   predicted cost of the cells an append changes is at most half the
   16.7 ms budget, about 1,400 × 760 device px of plot area → 2.1 ms
   plus at most 350 items per thinned channel (≤ 4 points × 1,400
   columns / 16) → under 1 ms each, so the test fails loudly if the
   cost model changes; then every frame after the first reports
   `previews == 0`); `drop_front_re_renders_the_cell`;
   `a_gesture_holds_a_windowed_handle` (drag, then append: the domain
   stays; double-click: the window follows again).
4. **Docs, `check.sh full`.**

Verify with `scripts/check.sh quick` and `ICED_TEST_BACKEND=tiny-skia|wgpu
cargo test -p rustd3-iced --features figure`; the live tests print the
cells rendered per frame.

#### Exit criteria

1. `cargo test -p rustd3-figure --all-features fit` passes with the
   `Fit::Window` spot rows.
2. `live_append_re_renders_only_touched_cells`,
   `fit_window_follows_in_whole_device_columns` and
   `ten_hz_appends_stay_sharp` pass on both backends.
3. `revision_change_re_renders_everything`,
   `caching_off_renders_every_cell`, `drop_front_re_renders_the_cell`
   and `a_gesture_holds_a_windowed_handle` pass.
4. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| `Fit::Window(10.0)`, plot area 1,000 device px, last = 123.456 s | col = 0.01 s, right = ceil(12,345.6) · 0.01 = 123.46, left = 113.46 | quantization |
| then 1,000 samples at 10 kHz appended (last = 123.556 s) | right = 123.56, left = 113.56: a 10-column move | |
| then 1 sample appended (last = 123.5561 s) | right = 123.56 unchanged (ceil); no x move, the tail cell re-renders | |
| `Fit::Window(10.0)` with no admitted data | [0, 1] | fits as `All` |
| `Window` handle dragged by +40 px, then an append | domain unchanged (held); after a double-click right = ceil(last / col) · col | |
| Append to channel 0 only, x domain set by the app | only panel 0's plot-area cell renders; y gutters untouched | part hashes |
| Append with `Fit::Window` | the panels' plot-area cells plus the bottom x-axis gutter cells render | |
| `Plot::revision(1)` → `(2)` | every cell renders once | |
| `drop_front` of the first chunk (65,536 samples) | revision bumps, index rebuilt, the cell renders; pixels equal a fresh figure over the shortened recording ≤ 1/255 | |
| 800 × 500 logical at 2×, plot area ≈ 1,400 × 760 device px | 1,064,000 / 5e5 ≈ 2.1 ms ≤ 8.35 → inline | Phase 19 model |
| Estimate for comparison, not asserted | 6–8 ms per update with shift and strip (see iced integration); the whole-cell re-render is measured by Phase 22 | |

#### Not in this phase

- Pan translation, shift and strip, anchored column origins, dash
  phase, tiles, per-column memo reuse, `Caching::Verify`, dependency
  classification (Phase 31, conditional on the benchmark verdict).
- The full-frame benchmark (Phase 22, next).
- `Recording<Timestamp>` (Phase 27) and `Fit::Window(SignedDuration)`
  with its tests (Phase 33).
- Thinning non-monotonic x (out of scope); envelope fills and marker
  splatting (Phase 31 measures them and builds the winners).

### Phase 22 — Full-frame benchmark

*M2 · builds on phase 21 · about 1,500 lines (dev crate `rustd3-bench`, ≈250 of them tests), 5 commits*

#### Goal

`rustd3-bench` drives the real `Plot` widget through a wrapper widget
that replays time-stamped input, in a window or headlessly, over five
workloads and ten scenarios, and writes raw records, CSV summaries and
a Markdown report that evaluates a decision rule registered before any
run. `scripts/check.sh full` runs its headless smoke plan as a report
that never fails on numbers. Agents build and smoke-test it headlessly;
the user runs the default plan on real hardware (Pop!_OS, RTX 3090)
while agents continue with Phases 23–30. The user's result becomes the
benchmark verdict in `decisions/phase-22.md`, which decides whether
Phase 31 (tiles, pan translation, the pool, envelope fills, marker
splatting) is built or skipped.

#### Builds on

- Phases 18–21: `Plot` with `scale_factor`, `render(Render)`,
  `probe(Probe)`, `on_change`, `caching(Caching::{On, Off})`,
  `revision`; `probe::Sample::{Stage, Raster, Frame}` with
  `generation`, `missing`, `previews` (slots whose shown content hash
  differs from their current one, gutters included), `uploaded`,
  `inline`; `plot::cost::{PIXELS_PER_MS, ITEMS_PER_MS, model_ms}`;
  `Fit::Window`.
- Phases 1 and 11: `dev/bench/Cargo.toml` with serde (`derive`) and
  serde_json (Phase 1, the only serde in the workspace), the
  `rustd3-examples` path dependency (Phase 1; Phase 34's wheel workload
  uses it), iced (`advanced`, `canvas`), `iced_runtime` and
  `rustd3-iced` with `figure` (Phase 11). No manifest changes here; no
  clap: arguments are parsed by hand. Dev crates may read clocks
  (`dev/clippy.toml` opts them out of `disallowed-methods`).
- Phases 9 and 15: `series::Recording<f64>`, `Clock::Regular`, marks
  and thinning; the workloads below are this phase's own (their layout
  and data are fixed here for repeatable timing; they are not the
  example figures).
- From the user, before the run: the display's refresh rate and scale
  factor (`--refresh-hz` and the plan's native factor).

#### Design

**The decision rule** (registered before any run; the report restates
it):

- **Metrics per scenario:** frame intervals (p50/p95/p99, dropped
  frames, present mode and compositor recorded); input-to-sharp
  latency; preview share (frames where any slot, gutters included,
  shows outdated content or no image); UI-thread busy time per event
  (p99); raster critical path (sum of the probe's `Raster` durations
  for the generation that became sharp); process CPU-seconds per second
  of interaction; upload bytes per second; RSS and GPU memory over a
  10-minute live-plus-pan run. Stage attribution from `Plot::probe`.
  N ≥ 5 runs per scenario with 95% confidence intervals.
- **Scenarios:** hover; pan on shared x; pan on own y; continuous zoom
  (wheel bursts, high-resolution pixel deltas); live data at 10 Hz with
  `Fit::Window`; a 4K-sized window; resize; cold start; scale factors
  1, 1.25, 1.5, 2; the workloads below.
- **Budgets:** p99 frame interval ≤ 1.5 × the display's frame period;
  input-to-sharp p95 under 100 ms for pans and 150 ms for zooms (drag
  latency becomes noticeable at about 55 ms, Deber et al. 2015; see
  Performance); preview share under 10% during pans. Each budget is
  met, missed or inconclusive by its 95% interval; the 10-minute memory
  run is one run per configuration.
- **Rule:** a missed budget is attributed to the UI thread (resolve,
  text, M4, list), raster (the worker's critical path) or upload. If
  any pan or zoom budget is missed and attributed to raster or upload,
  Phase 31 is built (tiles, translation, the pool, envelope fills,
  splatting) and this benchmark is rerun; a GPU data layer (vello in
  iced's shader widget, see iced integration) is recommended only if
  that rerun still misses. UI-thread misses route to moving list
  building off the UI thread, not to a GPU layer. If every budget is
  met, Phase 31 is skipped and the verdict says so.

**Operational readings**, set before any run (the report prints them;
the user confirms them before running; they are not changed after
seeing data):

1. "p99 frame interval ≤ 1.5 × the display's frame period": with vsync,
   intervals are multiples of the period T, so p99 ≤ 1.5 T means the
   99th-percentile frame was not a missed frame. A dropped frame is an
   interval > 1.5 T, counted as `round(interval / T) − 1`.
2. Pans are `pan-x`, `pan-y` and the pan half of `4k`; zooms are
   `zoom-lines`, `zoom-pixels` and the zoom half of `4k`.
3. A budget is **missed** when the 95% CI of the per-run statistic lies
   entirely above it, **met** when entirely below, **inconclusive**
   otherwise (rerun that cell with `--runs 10`).
4. Preview share = frames with `previews + missing > 0` / frames, from
   the first input to 300 ms after the last, per run. An outdated axis
   strip is a preview.
5. Attribution of a missed budget over its offending frames or latency
   windows: **UI** = `update_ms + draw_ms` (stage split from the probe);
   **raster** = Σ `Raster.duration` of the sharp frame's generation
   (the single worker's critical path; inline renders included and
   flagged); **upload** = Σ `Frame.uploaded` MB × the run's
   least-squares slope of frame interval on uploaded MB. The largest
   share names the bucket; the report shows all three.

**The verdict's life cycle.** This phase opens the entry in
`decisions/phase-22.md` as "pending the user's run". The user's run
adds the verdict: Phase 31 needed or not. Phase 31, if built, appends
its rerun; Phase 34 appends the label-churn rerun with real time
formats. Whichever of Phases 31 and 34 runs last writes the closing
line. When the user's results arrive, the agent writes the verdict
and, if it calls for it, Phase 31 is the next phase.

**Module layout** (`dev/bench/src/`):

```text
main.rs       commands
cli.rs        hand-parsed arguments
plan.rs       Plan: smoke | quick | default | full
scenario.rs   Scenario: generated input per scenario
replay.rs     Replay; replay::{Timed, Event, Cue}
workload.rs   Workload (+ workload/{trace,scatter,grid,label_churn}.rs)
driver.rs     Driver: replays input into the Plot, times update and draw
app.rs        the iced application
headless.rs   the headless frame loop
record.rs     Record: raw JSONL kinds
system.rs     /proc, nvidia-smi, meta.json
stats.rs      percentiles, confidence intervals
rule.rs       verdicts and attribution
report.rs     CSVs and report.md
```

**CLI** (`cli.rs`):

```text
rustd3-bench run     [--plan smoke|quick|default|full] [--scenario A,B]
                     [--workload A,B] [--factor 1,1.25] [--runs N]
                     [--refresh-hz HZ] [--backend wgpu|tiny-skia]
                     [--present-mode vsync|no_vsync|immediate|fifo|fifo_relaxed|mailbox]
                     [--headless] [--readback every|last]
                     [--render auto|inline|worker] [--caching on|off]
                     [--replay FILE] [--data-scale F] [--out DIR]
rustd3-bench child   (internal: one scenario × workload × factor × run)
rustd3-bench replay  --out DIR           write the generated replays
rustd3-bench record  --out FILE          record your own input as a replay
rustd3-bench report  DIR                 rebuild CSVs and report.md
rustd3-bench list                        scenarios, workloads, plans
```

`run` writes the replays, spawns one `child` process per cell and run
(`std::process::Command`, setting `ICED_BACKEND` and, if given,
`ICED_PRESENT_MODE`; `core/src/backend.rs:7-10,101-107`,
`wgpu/src/window/compositor.rs:434-459`), so every run is a cold
process, then builds the report. Windowed `run` requires
`--refresh-hz`; headless defaults to 60. `run` is the default command;
`--headless` without `--plan` means `--plan smoke`, so Phase 34's
`rustd3-bench --headless --workload wheels` is a headless smoke run on
one workload. `--scenario` and `--workload` restrict any plan; a
workload declares the scenarios it supports (`Workload::scenarios()`: a
figure without recordings has no `live` or `soak`, and the report marks
those cells "n/a", which is not "empty"). `--help` prints the user's
procedure. `--threads` and `--caching verify` exist only with Phase 31.

**Workloads** (`workload.rs`, `workload/*.rs`; data generated
deterministically by xorshift64* with fixed seeds):

| Name | Figure |
| --- | --- |
| `trace` | Two stacked panels sharing a linear x (seconds) with their own y; 3 channels × 1M samples on one `Clock::Regular` (10 kHz, 100 s: drift, a 7 Hz tone, noise, bursts every 10 s, 50 one-sample impacts); channels 1–2 top, 3 bottom; x zoomed by both panels, y per panel; two axes per panel and a legend |
| `scatter100k`, `scatter1m` | One panel, dots in 5 Gaussian clusters; zoom on both handles |
| `grid16` | 4 × 4 panels, x shared per column, y per row; per panel a 100k-sample line (a recording) and 2k dots |
| `label-churn` | `trace` with a churning x formatter (`format!("t+{v:.4} s")`, `.ticks(16)`): every pan step brings new label strings (real time formats in Phase 34) |

**Scenarios and replays** (`scenario.rs`, `replay.rs`):

| Scenario | Input (logical px, relative to the plot) | Duration |
| --- | --- | --- |
| `hover` | Lissajous cursor path over the plot areas, a move every 8 ms (125 Hz mouse) | 10 s |
| `pan-x` | drag on the top panel (shared x): 600 px sweeps left and right, moves of 2–8 px at 125 Hz | 12 s |
| `pan-y` | vertical drag on the bottom panel (own y), same rhythm | 12 s |
| `zoom-lines` | wheel bursts: ±1 line at 30 Hz for 1 s, 0.5 s pause (the 300 ms wheel end fires), alternating in and out | 12 s |
| `zoom-pixels` | high-resolution pixel deltas at 120 Hz, 2–12 px with a sinusoidal envelope, 1 s bursts, 0.5 s pauses | 12 s |
| `live` | append 1,000 samples per channel every 100 ms, `Fit::Window(10.0)` on x | 30 s |
| `4k` | `pan-x` then `zoom-lines` in a window of 3840 × 2160 device px (logical 3840/f × 2160/f; if the display cannot fit it, 1920 × 1080 at factor 2, recorded) | 12 s |
| `resize` | `window::resize` cycling 1200 × 800 → 1600 × 1000 → 1000 × 700 every 500 ms | 10 s |
| `cold` | none: process start (`main`'s first line) to the first `Frame` with `missing + previews == 0` | — |
| `soak` | `live` plus one `pan-x` sweep every 10 s; RSS every 1 s, GPU memory every 5 s | 600 s |

Default window: 1600 × 1000 logical. A replay is JSON Lines, one
time-stamped event per line, written to `out/replays/` before a run:

```json
{"t_ms": 0.0, "event": {"move": {"x": 120.0, "y": 80.0}}}
{"t_ms": 8.0, "event": {"press": "left"}}
{"t_ms": 16.0, "event": {"wheel": {"lines": {"x": 0.0, "y": -1.0}}}}
{"t_ms": 16.0, "event": {"wheel": {"pixels": {"x": 0.0, "y": -6.5}}}}
{"t_ms": 30.0, "event": {"modifiers": {"ctrl": true}}}
{"t_ms": 100.0, "event": {"append": 1000}}
{"t_ms": 500.0, "event": {"resize": {"width": 1600.0, "height": 1000.0}}}
```

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Replay { pub events: Vec<replay::Timed> }

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Timed { pub t_ms: f64, pub event: replay::Event }

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Move { x: f32, y: f32 },
    Press(Button),
    Release(Button),
    Wheel(Delta),
    /// Missing flags default to false.
    Modifiers { ctrl: bool, shift: bool, alt: bool },
    Append(u32),
    Resize { width: f32, height: f32 },
}

/// What the driver publishes to the app.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue { Append(u32), Resize { width: f32, height: f32 }, Done }
```

`Button` (`left`, `right`, `middle`) and `Delta` (`lines { x, y }`,
`pixels { x, y }`) are the bench's own serde types. `record` opens the
`trace` window, writes every real mouse and modifier event relative to
the first one and stops on Escape; `--replay FILE` substitutes it for a
scenario's generated input.

**The driver widget** (`driver.rs`):

```rust
/// Feeds a replay to its child as if the user produced it, and times
/// the child's update and draw calls.
pub struct Driver<'a, Message> {
    child: Element<'a, Message, Theme, Renderer>,
    replay: &'a Replay,
    records: mpsc::Sender<Record>,
    cue: fn(replay::Cue) -> Message,
}
```

Its tree state holds `start: Option<Instant>`, `next: usize`, the
synthetic cursor and the settle deadline. On `RedrawRequested(now)`:
set `start`; for every event with `t_ms ≤ now − start`, translate it to
the iced event (`CursorMoved`, `ButtonPressed(Left)`, `WheelScrolled {
delta: ScrollDelta::Lines | Pixels }`, `Keyboard(ModifiersChanged)`),
call `child.as_widget_mut().update(…)` with
`Cursor::Available(synthetic)` and record `Input { scheduled,
injected, update_ms }`; `Append` and `Resize` are published through
`cue`. Then forward the `RedrawRequested` itself (timed as the frame's
`update_ms`) and `request_redraw()`, so frames keep coming until 2 s
after the last event and every slot is sharp; then publish
`cue(Cue::Done)`. Real mouse and keyboard events are not forwarded
during a run; `Event::Waken` and window events are. `draw` calls the
child with the synthetic cursor, timed as `draw_ms`.

The app (`app.rs`) owns the recordings, builds the workload figure in
`view`, wraps `Plot::new(fig).scale_factor(f).render(r).caching(c)
.probe(Probe::new(Instant::now, tx))` in a `Driver`, applies `Append`
(extends the recording) and `Resize` (`window::resize`), and on `Done`
drains the channel, writes the raw file and returns `iced::exit()`.
Scale factors other than the display's come from
`application(…).scale_factor(|s| s.factor / s.window_factor)`, the
window factor read from `window::events()` (`src/application.rs:436`,
`runtime/src/window.rs:204,295,354`).

**Headless mode** (`headless.rs`).
`<iced::Renderer as Headless>::new(settings, Some(backend))` under
`iced::futures::executor::block_on` (as `test/src/simulator.rs:66-77`
uses it; `src/lib.rs:483`); `UserInterface::build(driver, size, cache,
&mut renderer)` (`runtime/src/user_interface.rs:99-130`); a
`shell::Waker` that sets an `AtomicBool`. Virtual frames are paced at
`--refresh-hz` with `thread::sleep` to the next deadline; per frame, in
this order: `renderer.tick()` (on wgpu it drains the image upload
worker, `wgpu/src/lib.rs:709-712`; nothing but winit's `Action::Tick`
calls it, `winit/src/lib.rs:1789-1791`, so a headless loop must), then,
if the wake flag is set, clear it and `update(&[Event::Waken])`; then
`update(&[RedrawRequested(now)])` (`user_interface.rs:198-206`), and if
messages were published, apply them to the app, rebuild and repeat (at
most three passes, one `now`, as `winit/src/lib.rs:788-830`); `draw`
(`:550`); `renderer.screenshot(physical, factor, background)`
(`core/src/renderer.rs:143-164`) per `--readback` (`every` by default:
the headless renderers rasterize images only when they render, so
readback is what exercises upload; its cost is recorded separately as
`harness_ms`). Headless frame intervals are not display intervals; the
report marks every headless number "report-only".

**Records and metrics** (`record.rs`, `stats.rs`, `system.rs`). Raw
JSONL kinds: `frame` (`Sample::Frame` joined with the driver's
`update_ms`/`draw_ms`), `input`, `stage`, `raster`, `sys`.

| Metric | Computation |
| --- | --- |
| Frame intervals p50/p95/p99, drops | differences of consecutive `RedrawRequested` instants while the driver requests frames; drops per reading 1 |
| Input-to-sharp latency | for an input applied in frame n (generation g after it), the first frame m ≥ n with generation ≥ g and `missing + previews == 0`; latency = `at(m) − scheduled` (scan-out adds up to one period, reported separately) |
| Preview share | reading 4 |
| UI-thread busy per event, p99 | driver-timed `update_ms` per injected event, and per frame `update_ms + draw_ms` |
| Raster critical path | per latency window, Σ `Raster.duration` of the sharp frame's generation (reading 5); also the `inline` share of renders |
| CPU-seconds per second | Δ(utime + stime) of `/proc/self/stat` (fields 14, 15, in `getconf CLK_TCK` ticks) over the interaction window, divided by its wall time (includes the worker and vello_cpu's pool) |
| Upload bytes per second | Σ `Frame.uploaded` / interaction seconds |
| RSS, GPU memory | `VmRSS` from `/proc/self/status`; `nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits` minus the pre-run baseline (whole-GPU, approximate; recorded as such) |
| Stage attribution | `Stage` samples (resolve, list), `Raster` durations and queue waits |
| CI | per-run statistic, mean ± t(0.975, n−1) · s/√n (n = 5 → 2.776); median also shown |

`meta.json`: CPU model and cores, RAM, kernel, GPU name and driver,
`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, present mode (env or
"AutoVsync (iced default)"), backend, refresh, factors, render mode,
caching, rustc version, iced rev, workspace git rev, plan, command
line, the widget's cost-model constants
(`rustd3_iced::plot::cost::{PIXELS_PER_MS, ITEMS_PER_MS}`) and the
calibration factor the bench computes from the `Raster` samples (median
of measured / `cost::model_ms`), which the verdict may use to move the
constants.

**Rule evaluation** (`rule.rs`). For each scenario × workload × factor:
verdicts per budget (readings 1–4). For each missed budget, attribution
per reading 5 over the offending frames or latency windows; the
largest share names the bucket. The final section prints the outcome in
one of three forms, with the supporting rows: "image path sufficient —
Phase 31 skipped", "UI-thread bound → move list building off the UI
thread; Phase 31 skipped", or "raster/upload bound → build Phase 31
(tiles, translation, pool, envelope fills, splatting) and rerun". This
run never recommends a GPU data layer; only Phase 31's rerun can.

**Plans** (`plan.rs`):

| Plan | Cells | Runs | Est. time |
| --- | --- | --- | --- |
| `smoke` | every scenario × every workload ("n/a" cells marked, not skipped); factor 2; durations ÷ 5; data ÷ 100; `soak` 20 s | 1 | < 8 min (headless) |
| `quick` | as `default` | 2 | ≈ 20 min (not valid for the verdict) |
| `default` | (A) the 9 scenarios other than `soak` × `trace` × native factor; (B) `hover`, `pan-x`, `zoom-lines` × the 4 other workloads × native; (C) `pan-x`, `zoom-lines` × `trace` × {1, 1.25, 1.5, 2} minus native; (D) `soak` once | 5 (soak 1) | ≈ 50 min |
| `full` | every scenario × workload × factor | 5 | ≈ 4 h |

**Outputs** (`--out DIR`, one subdirectory per invocation, named by UTC
time):

```text
meta.json      replays/*.jsonl      raw/<scenario>__<workload>__f<factor>__r<k>.jsonl
frames.csv     inputs.csv           summary.csv (scenario, workload, factor, metric,
memory.csv                            n, median, mean, ci_low, ci_high, budget, verdict)
report.md      operational readings, budget table, attribution, rule outcome, flags
```

**The user's procedure** (printed by `--help`):

1. Tell the agent the display's refresh rate and scale factor; read and
   confirm the operational readings.
2. `cargo build --release -p rustd3-bench`.
3. Plug in power, `system76-power profile performance`, close other
   GPU-heavy apps; leave the desktop's compositor as it is (recorded).
4. `target/release/rustd3-bench run --plan default --refresh-hz <HZ>
   --out ~/bench-out`: windows open and close for about 50 minutes; do
   not touch the mouse or keyboard.
5. Optional: the same with `--backend tiny-skia` (report-only);
   `record` then `run --scenario zoom-pixels --replay FILE` with your
   own touchpad; `--render inline` to see the pure synchronous path.
6. Send back `~/bench-out/<stamp>/` (`tar czf bench.tgz -C ~/bench-out
   <stamp>`); at least `report.md`, `summary.csv`, `meta.json`.

The agent then writes the verdict and whether Phase 31 runs; the
label-churn rows stay provisional until Phase 34.

**Decisions this phase may meet**

- An operational reading turns out ambiguous once real data exists —
  apply the reading as written (readings are never changed after
  seeing data) and note the ambiguity beside the verdict; record it as
  a decision.

#### Steps

1. **Skeleton, CLI, plans, workloads.** `main.rs`, `cli.rs`, `plan.rs`,
   `workload.rs` + `workload/{trace,scatter,grid,label_churn}.rs`.
   Tests: `cli_parses_every_documented_flag`, `rejects_unknown_flags`,
   `trace_has_three_channels_of_one_million`,
   `default_plan_has_the_pre_registered_cells`,
   `smoke_plan_covers_every_scenario_and_workload`.
2. **Replays.** `scenario.rs`, `replay.rs`, the `replay` and `record`
   commands. Tests: `replays_round_trip_through_json`,
   `zoom_lines_pauses_exceed_the_wheel_end`, `hover_moves_every_8_ms`.
3. **Driver and headless loop.** `driver.rs`, `app.rs`, `headless.rs`,
   `record.rs`, `system.rs`; `child`. Smoke: one headless `pan-x` on
   `trace` writes frames with `generation` rising and a final sharp
   frame; on wgpu, `tick()` is observed draining an upload (a frame
   whose `uploaded` > 0 follows a tick).
4. **Stats, rule, report.** `stats.rs`, `rule.rs`, `report.rs`, `run`,
   `report`. Tests on synthetic records (the spot checks), including
   `attribution_names_the_largest_share` and
   `preview_share_counts_outdated_gutters`.
5. **check.sh and the verdict entry.** `scripts/check.sh full` gains the
   headless smoke run (report-only); write the pending verdict entry in
   `.claude/decisions/phase-22.md`. Run the smoke plan on tiny-skia and,
   if a Vulkan device exists, on wgpu.

Verify with `scripts/check.sh quick`, `cargo test -p rustd3-bench` and
`cargo run --release -p rustd3-bench -- run --headless --plan smoke
--backend tiny-skia --out target/bench-smoke`. Agents never run the
windowed benchmark (see Appendix A).

#### Exit criteria

1. `cargo build --release -p rustd3-bench` succeeds; `cargo test -p
   rustd3-bench` passes (stats, rule, replay and parser tests).
2. `rustd3-bench run --headless --plan smoke --backend tiny-skia --out
   target/bench-smoke` finishes in under 8 minutes on the container and
   writes every file under Outputs; `report.md` has a row for every
   scenario × every workload of the smoke plan (values or "n/a"), a
   rule section marked "report-only: headless", no empty cell and no
   NaN. The same with `--backend wgpu` passes where a Vulkan device
   exists (lavapipe counts), else is skipped with a message.
3. `scripts/check.sh full` prints the smoke summary and fails only on
   errors, never on numbers.
4. `.claude/decisions/phase-22.md` holds the verdict entry "pending the
   user's run" with the exact command, the operational readings, the
   plan's expected duration and the three possible outcomes (image path
   sufficient, Phase 31 skipped; UI-thread bound, Phase 31 skipped;
   raster/upload bound, Phase 31 built and the benchmark rerun).

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| 144 Hz | T = 6.94 ms; p99 budget ≤ 10.4 ms; an interval of 20.8 ms = round(3.0) − 1 = 2 drops | reading 1 |
| 60 Hz | T = 16.67 ms; p99 budget ≤ 25.0 ms | reading 1 |
| Per-run p95 latencies 80, 85, 90, 95, 100 ms (pans) | mean 90, s 7.91, CI 90 ± 9.82 = [80.2, 99.8] → met | reading 3 |
| 90, 95, 100, 105, 110 ms | CI [90.2, 109.8] → inconclusive: rerun with `--runs 10` | reading 3 |
| Input scheduled at 1000 ms, applied in the frame at 1004 ms (gen 7); frames at 1011 (gen 7, previews 3), 1018 (gen 7, sharp) | latency 18 ms | latency metric |
| 2 of 40 frames during a pan with `previews + missing > 0` | preview share 5% → met | reading 4 |
| A pan frame whose x-axis gutter still shows the previous domain's labels | counts as a preview | reading 4 |
| Latency window 30 ms: UI 6 ms, raster Σ 18 ms, upload 2 MB × 1.5 ms/MB = 3 ms | bucket "raster" | reading 5 |
| `/proc/self/stat` utime + stime grows 150 ticks over 10 s, CLK_TCK 100 | 0.15 CPU-s per s | |
| `trace` | 3 × 1,000,000 samples, 100 s at 10 kHz, 50 impacts, 10 bursts | workload |
| `smoke` plan | 10 scenarios × 5 workloads = 50 cells, 46 run (`live`/`soak` n/a on the two scatter workloads, which hold no recording; `grid16`'s lines are recordings), 1 run each | plan |
| `default` plan, native factor among the four | A 9 + B 12 + C 6 + D 1 = 28 cells; 27 × 5 + 1 = 136 runs; ≈ 135 × 16 s = 36 min plus the 10-minute soak ≈ 46 min | plan |
| Estimates to compare (see iced integration) | hover overlay only; live 6–8 ms; shared pan ≈ 8 ms; own pan ≈ 5.5 ms; zoom sharp 20–30 ms after the preview; 4K 15–18 sharp frames/s | |

#### Not in this phase

- Envelope fills, marker splatting, tiles, the pool, `Plot::threads`,
  `--threads`, `--caching verify` (Phase 31, built only if the verdict
  asks; it reruns this benchmark).
- A GPU data layer (its own plan, only after Phase 31's rerun still
  misses).
- The wheel-figure workload and the label-churn rerun with time formats
  (Phase 34).
- Running the windowed benchmark (the user does).

## M3 — Core breadth

The rest of the d3 port, graded by d3's tests: color, the derive, binned scales and array statistics, number and time locales and formats, time scales, shapes, behaviors and animation.

### Phase 23 — Color spaces, interpolation and color scales

*M3 Core breadth · builds on phase 7 · about 1,700 lines of Rust plus ~1,100 generated and ≤ 250 lines of translator JS, 8 commits · two sessions: cut after Step 3*

#### Goal

Core gains d3-color's perceptual spaces (`Lab`, `Hcl`, `Cubehelix`, with d3's `gray` and `lch` constructors); d3-interpolate's color, hue, basis, piecewise and quantize interpolators, the `Interpolate` impls for the new spaces, for `Hsl` and for `jiff::Timestamp`; every d3-scale-chromatic sequential, diverging and cyclical scheme as exact `f64` d3 code behind a plain `pub fn name(t: f64) -> Rgb`, plus the categorical schemes Phase 7 left out; and the color scales `scale::Sequential<R, T>` and `scale::Diverging<R, T>` with their log, pow, sqrt and symlog variants and `Ticks` impls that place ticks in [0, 1]. d3-color (beyond Phase 7), d3-interpolate and d3-scale's sequential and diverging tests grade it through the harness; a d3 fixture at eleven samples per scheme grades every scheme exactly. No color crate enters the dependency tree.

#### Builds on

- Phase 2: the harness (`Obs`, `check`, `Kind::InDelta`, the ratchet, per-file floors in `passing.txt`), the resolve hook `dev/harness/translate/d3-resolve.mjs`.
- Phase 4: `interpolate::Interpolate: Clone` (`fn lerp(&self, to: &Self, t: f64) -> Self`) with impls for `f64` and `f32`; `interpolate::{number, round}` (closures, `Clone + Send + Sync`); `scale::Continuous<T, R, N>`; `scale::Transform` (`forward`, `inverse`, `ticks`, `minor`, `for_domain` — log reflects negative domains — `nice`, `tick_format`); `scale::transform::{Identity, Pow, Log}` with the builders `exponent` and `base`; `scale::TickFormat::format_all`.
- Phase 5: `scale::transform::Symlog` and its `constant` builder, the `ticks::` rules, `.tick_rule`.
- Phase 6: `scale::Ticks` (`type Domain`, `type Step`, `tick_set`, `position(&v, offset) -> Option<f64>`, `labels`, `minor`, `extent`), `ticks::Set`, `locale::Locale` (number half), `TickFormat::labels(&ticks::Set<f64, f64>, &Locale)`, the `Ticks` impl for `Continuous<T, f64, N>`.
- Phase 7: `color::Rgb` and `color::Hsl` (public `f64` fields), the CSS parser (`FromStr`, 148 names, `ParseColorError`), `Display` as d3's `formatRgb`, the `const fn Rgb::hex`, `brighter`/`darker`, `clamp`, `displayable`, `format_hex`, `Interpolate for Rgb` (d3's `interpolateRgb` at gamma 1 with its NaN rule), `Linear<Rgb>`, `scheme::{CATEGORY10, OBSERVABLE10, DEEP, COLORBLIND, OKABE_ITO}`. Phase 7 has no `Interpolate for Hsl`.
- Decision files `decisions/phase-04.md`, `phase-06.md`, `phase-07.md`.

#### Design

Module layout:

```text
core/src/color.rs                     re-exports Lab, Hcl, Cubehelix beside Rgb, Hsl
core/src/color/{lab,hcl,cubehelix}.rs one space each: conversions, brighter/darker, FromStr, Display
core/src/interpolate.rs               re-exports; Interpolate for Hsl, Lab, Hcl, Cubehelix, Timestamp
core/src/interpolate/gamma.rs         d3's color.js channel rules, crate-private: nogamma, gamma, hue wrap
core/src/interpolate/color.rs         rgb, rgb_gamma, rgb_basis, hsl(_long), lab, hcl(_long), cubehelix(_long);
                                      crate-private basis_at
core/src/interpolate/{hue,basis,piecewise,quantize}.rs
core/src/scheme.rs                    re-exports (Phase 7's constants, the new ones, every scheme fn)
core/src/scheme/{sequential,diverging,cyclical}.rs   scheme functions (hand-written)
core/src/scheme/stops.rs              generated: ColorBrewer stop rows, crate-private
core/src/scheme/{viridis,magma,inferno,plasma}.rs    generated: 256-entry table + its ramp fn
core/src/scheme/categorical.rs        generated: ACCENT … TABLEAU10
core/src/scale/{sequential,diverging}.rs
```

##### Color spaces

```rust
/// CIELAB under D50 with d3's constants. Plain data with public fields, like Rgb.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab { pub l: f64, pub a: f64, pub b: f64, pub opacity: f64 }
/// d3's hcl: Lab in cylindrical form (hue in degrees, chroma, luminance).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hcl { pub h: f64, pub c: f64, pub l: f64, pub opacity: f64 }
/// d3's cubehelix (Green's scheme, d3's constants).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cubehelix { pub h: f64, pub s: f64, pub l: f64, pub opacity: f64 }

impl Lab {
    pub const fn new(l: f64, a: f64, b: f64) -> Self;     // opacity 1
    /// d3.gray(l): Lab(l, 0, 0).
    pub const fn gray(l: f64) -> Self;
    pub fn brighter(self, k: f64) -> Self;                // l + 18k
    pub fn darker(self, k: f64) -> Self;                  // l − 18k
}
impl Hcl {
    pub const fn new(h: f64, c: f64, l: f64) -> Self;
    /// d3.lch(l, c, h).
    pub const fn lch(l: f64, c: f64, h: f64) -> Self;
    pub fn brighter(self, k: f64) -> Self;                // l + 18k
    pub fn darker(self, k: f64) -> Self;
}
impl Cubehelix {
    pub const fn new(h: f64, s: f64, l: f64) -> Self;
    pub fn brighter(self, k: f64) -> Self;                // l · (1/0.7)^k
    pub fn darker(self, k: f64) -> Self;                  // l · 0.7^k
}
// From in every direction through Rgb: From<Rgb> for Lab, Hcl, Cubehelix;
// From<Lab | Hcl | Cubehelix> for Rgb; From<Lab> for Hcl; From<Hcl> for Lab;
// From<Hsl> for each (via Rgb). FromStr through Rgb's parser, as d3's lab("hsl(…)"),
// with ParseColorError. Display prints the Rgb form, as d3's toString.
```

**The conversion math is d3's own**, ported from `d3-color/src/{lab,cubehelix,math}.js`: 7-digit Bradford-adapted matrices (`Xn = 0.96422`, `Zn = 0.82521`, first row `0.4360747 0.3850649 0.1430804`, …), d3's `t0`, `t1`, `t2`, `t3` constants and its sRGB transfer. A general color crate's CIELAB was measured as the alternative: palette 0.7's `Lab<D50, f64>` (XYZ under D65 adapted to D50) gives (74.968804, −3.399015, −10.696497) for `rgb(170, 187, 204)` against d3's (74.968800, −3.398999, −10.696507), 1.7·10⁻⁵ off where `lab-test.js` allows 10⁻⁶. Porting the formulas is shorter than adapting a crate, and the core keeps jiff and thiserror as its only dependencies. d3's special cases stay: an achromatic `Rgb` converts with `x = z = y` (`labConvert`), so its `a` and `b` are exactly 0; `Hcl` from a `Lab` with `a == b == 0` has `h = NaN`, and `c = 0` when `0 < l < 100`, else `c = NaN` (black and white; `hcl-test.js:15–27`); `Hcl` to `Lab` with a NaN hue gives `Lab(l, 0, 0)`.

##### Interpolators

Free functions with d3's names. Each returns a closure that is `Clone + Send + Sync`, so scales and `anim` can store it; color interpolators take the target space's type and callers convert (`Hsl::from(rgb)`), which replaces d3's string dispatch.

```rust
pub fn rgb(a: Rgb, b: Rgb) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn rgb_gamma(gamma: f64, a: Rgb, b: Rgb) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn rgb_basis(colors: &[Rgb]) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn hsl(a: Hsl, b: Hsl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;       // shortest hue
pub fn hsl_long(a: Hsl, b: Hsl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn lab(a: Lab, b: Lab) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn hcl(a: Hcl, b: Hcl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn hcl_long(a: Hcl, b: Hcl) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn cubehelix(gamma: f64, a: Cubehelix, b: Cubehelix) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn cubehelix_long(gamma: f64, a: Cubehelix, b: Cubehelix) -> impl Fn(f64) -> Rgb + Clone + Send + Sync;
pub fn hue(a: f64, b: f64) -> impl Fn(f64) -> f64 + Clone + Send + Sync;
pub fn basis(values: &[f64]) -> impl Fn(f64) -> f64 + Clone + Send + Sync;
pub fn piecewise<T, G, U>(values: &[T], interpolator: impl Fn(&T, &T) -> G)
    -> impl Fn(f64) -> U + Clone + Send + Sync
where
    T: Clone + Send + Sync + 'static,
    G: Fn(f64) -> U + Clone + Send + Sync;
pub fn quantize<T>(f: impl Fn(f64) -> T, n: usize) -> Vec<T>;
```

Write these signatures and compile a one-line use of each generic one (including `piecewise` feeding a `Sequential`) before porting the bodies, so the advertised `Send + Sync` bounds are known to hold.

Rules, from `d3-interpolate/src/*.js`:

- **Channels** (`color.js`, in `gamma.rs`): `nogamma(a, b)` is linear in `b − a`, and when that difference is NaN or 0 it is the constant `b` if `a` is NaN, else `a` — so a NaN channel takes the other side's value. `gamma(y)` with `y ≠ 1` is d3's `exponential(a, b, y)`: `(a^y + t·(b^y − a^y))^(1/y)`. The hue rule wraps a difference `d` with `|d| > 180` to `d − 360·round(d / 360)`.
- `rgb` and `rgb_gamma` apply `nogamma`/`gamma` per channel and `nogamma` to opacity, as `d3.interpolateRgb` and `.gamma(y)`. `hsl` and `hcl` use the hue rule for `h`, `*_long` use `nogamma` for `h`; `lab` is `nogamma` on every channel. `cubehelix(gamma, …)` uses the hue rule for `h`, `nogamma` for `s`, and evaluates `l` at `t^gamma` (`cubehelix.js`); `cubehelix_long` the same with `nogamma` on `h`.
- `hue(a, b)` is the hue rule followed by `x − 360·floor(x / 360)`, so the result lies in [0, 360).
- `basis(values)` is `basis.js`: with `n = len − 1`, `t ≤ 0` evaluates segment 0 at 0, `t ≥ 1` segment `n − 1` at 1, otherwise segment `i = floor(t·n)`; missing outer neighbours are reflected (`2·v1 − v2`, `2·v2 − v1`); the cubic B-spline weights are d3's. `rgb_basis` runs it per channel with opacity 1 (`rgbSpline`). The crate-private `basis_at(stops: &[Rgb], t: f64) -> Rgb` is that per-channel body without allocating, used by the ColorBrewer schemes.
- `piecewise` builds one segment interpolator per neighbouring pair, as `piecewise(interpolate, values)`: `t` is scaled by the segment count `n = len − 1`, the segment index is `floor(t·n)` clamped to `[0, n − 1]`, and that segment is evaluated at `t·n − i` (so values outside [0, 1] extrapolate the end segments, `piecewise-test.js`).
- `quantize(f, n)` samples `f(i / (n − 1))` for `i in 0..n`, as d3.

New trait impls: `Interpolate for Hsl` (hue rule on `h`, `nogamma` on the rest), `Lab` (`nogamma` per channel), `Hcl` (hue rule plus `nogamma`) and `Cubehelix` (hue rule, `nogamma`, gamma 1), each returning its own space; and `Interpolate for jiff::Timestamp`, in i128 nanoseconds: `a + round((b − a) as f64 · t)`, saturating to `Timestamp::MIN`/`MAX`. Only the difference passes through `f64`, never an epoch value, as everywhere time computes.

##### Schemes

The categorical schemes Phase 7 left out join its constants in `scheme/categorical.rs`, generated from `d3-scale-chromatic/src/categorical/*.js`: `ACCENT: [Rgb; 8]`, `DARK2: [Rgb; 8]`, `PAIRED: [Rgb; 12]`, `PASTEL1: [Rgb; 9]`, `PASTEL2: [Rgb; 8]`, `SET1: [Rgb; 9]`, `SET2: [Rgb; 8]`, `SET3: [Rgb; 12]`, `TABLEAU10: [Rgb; 10]`.

Every d3-scale-chromatic interpolator is a plain function `pub fn name(t: f64) -> Rgb`, named as d3's camel case lowered to snake case (`interpolateYlGnBu` → `yl_gn_bu`, `interpolateCubehelixDefault` → `cubehelix_default`). Each is exact `f64` d3 code; there is no `OnceLock`, no gradient object and no color crate.

| Schemes | Module | Definition (d3 source) |
| --- | --- | --- |
| ColorBrewer sequential, single hue: `blues greens greys oranges purples reds`; multi-hue: `bu_gn bu_pu gn_bu or_rd pu_bu_gn pu_bu pu_rd rd_pu yl_gn_bu yl_gn yl_or_br yl_or_rd` | `sequential.rs` | `basis_at(&stops::NAME, t)` over d3's last stop row, 9 entries (`ramp.js`: `interpolateRgbBasis(scheme[scheme.length − 1])`) |
| ColorBrewer diverging: `br_bg pr_gn pi_yg pu_or rd_bu rd_gy rd_yl_bu rd_yl_gn spectral` | `diverging.rs` | the same over the 11-entry row |
| `turbo`, `cividis` | `sequential.rs` | `sequential-multi/{turbo,cividis}.js`: `t` clamped to [0, 1], one polynomial per channel, each rounded and clamped to [0, 255] |
| `warm`, `cool` | `sequential.rs` | `cubehelix_long(1.0, Cubehelix::new(−100, 0.75, 0.35), Cubehelix::new(80, 1.5, 0.8))` and `(260, 0.75, 0.35) → (80, 1.5, 0.8)` (`rainbow.js`) |
| `cubehelix_default` | `sequential.rs` | `cubehelix_long(1.0, Cubehelix::new(300, 0.5, 0), Cubehelix::new(−240, 0.5, 1))` (`cubehelix.js`) |
| `sinebow` | `cyclical.rs` | `sinebow.js`: `u = (0.5 − t)·π`; r, g, b = `255·sin²(u)`, `255·sin²(u + π/3)`, `255·sin²(u + 2π/3)` |
| `rainbow` | `cyclical.rs` | `rainbow.js`: `t` outside [0, 1] wrapped by `t − floor(t)`; Cubehelix with `h = 360t − 100`, `s = 1.5 − 1.5·abs(t − 0.5)`, `l = 0.8 − 0.9·abs(t − 0.5)` |
| `viridis magma inferno plasma` | `viridis.rs` … | d3's 256-entry tables (`sequential-multi/viridis.js`); lookup is d3's `ramp`: `TABLE[clamp(floor(t·256), 0, 255)]` |

That is 27 ColorBrewer schemes and 11 others. The ColorBrewer stop rows are data whatever code evaluates them, so their licence travels with them: Step 5 appends the ColorBrewer notice from `d3-scale-chromatic/LICENSE` ("Apache-Style Software License for ColorBrewer software and ColorBrewer Color Schemes", Copyright 2002 Cynthia Brewer, Mark Harrower, and The Pennsylvania State University, Apache-2.0) to `core/LICENSE-d3`, where Phase 1 reserved its place.

**The generator** `dev/harness/gen/scheme_tables.mjs` (plain Node, run as `node --import dev/harness/translate/d3-resolve.mjs dev/harness/gen/scheme_tables.mjs`) imports d3-scale-chromatic 3.1.0 from the vendored sources and writes, with sorted output and every color as `Rgb::hex(0x……)`:

- `scheme/stops.rs`: `pub(crate) const BLUES: [Rgb; 9]`, …, `pub(crate) const SPECTRAL: [Rgb; 11]` — the last row of each `scheme*` array;
- `scheme/{viridis,magma,inferno,plasma}.rs`: `const TABLE: [Rgb; 256]` and the one-line `pub fn viridis(t: f64) -> Rgb` that ramps it (the ramp helper is crate-private in `scheme.rs`);
- `scheme/categorical.rs`: the nine constants above, with doc comments naming the d3 scheme.

Each generated file starts with `// generated by dev/harness/gen/scheme_tables.mjs from d3-scale-chromatic 3.1.0. Do not edit.`, prints nothing on success, and is reproduced byte for byte by a rerun. Phase 26 puts that rerun into `scripts/check.sh full`.

##### Color scales

```rust
/// d3.scaleSequential: a domain mapped to [0, 1] through a transform, then an interpolator.
pub struct Sequential<R, T = transform::Identity> {
    transform: T,
    domain: [f64; 2],
    interpolator: Arc<dyn Fn(f64) -> R + Send + Sync>,
    clamp: bool,
    unknown: Option<R>,
}
pub fn sequential<R>(domain: [f64; 2],
    interpolator: impl Fn(f64) -> R + Send + Sync + 'static) -> Sequential<R>;

impl<R: Clone, T: Transform> Sequential<R, T> {
    pub fn domain(self, domain: [f64; 2]) -> Self;
    pub fn get_domain(&self) -> [f64; 2];
    pub fn interpolator(self, f: impl Fn(f64) -> R + Send + Sync + 'static) -> Self;
    pub fn clamp(self, clamp: bool) -> Self;
    pub fn unknown(self, value: R) -> Self;
    pub fn transform<U: Transform>(self, transform: U) -> Sequential<R, U>;
    pub fn apply(&self, x: f64) -> R;
    pub fn get_range(&self) -> [R; 2];                     // [f(0), f(1)]
    pub fn ticks(&self, count: usize) -> Vec<f64>;         // the transform's rule
    pub fn nice(self, count: usize) -> Self;
    pub fn tick_format(&self, count: usize, specifier: Option<&str>)
        -> Result<scale::TickFormat, format::InvalidFormat>;
    pub fn tick_format_in(&self, locale: &format::Locale, count: usize, specifier: Option<&str>)
        -> Result<scale::TickFormat, format::InvalidFormat>;
}
impl<R: Interpolate + Send + Sync + 'static, T> Sequential<R, T> {
    pub fn range(self, range: [R; 2]) -> Self;             // interpolator = R::lerp
}
// Transform-specific builders exactly where Continuous has them, delegating to the
// transform value: Sequential<R, transform::Log>::base,
// Sequential<R, transform::Pow>::exponent, Sequential<R, transform::Symlog>::constant.

/// d3.scaleDiverging: the same with three stops.
pub struct Diverging<R, T = transform::Identity> { /* transform, domain: [f64; 3], interpolator,
                                                     clamp, unknown */ }
pub fn diverging<R>(domain: [f64; 3],
    interpolator: impl Fn(f64) -> R + Send + Sync + 'static) -> Diverging<R>;
// Every Sequential method with [f64; 3] domains; range(self, [R; 3]) builds
// interpolate::piecewise over the three values with R::lerp; get_range -> [R; 3]
// = [f(0), f(0.5), f(1)].
```

`tick_format` and `tick_format_in` have `Continuous`'s signatures (Phase 4). The log, pow, sqrt and symlog variants are `.transform(…)` plus those builders; there are no `sequential_log` functions.

**`Sequential::apply`** (`d3-scale/src/sequential.js:20–22`): with `T = transform.for_domain(domain)` as `Continuous` uses it (so a sequential log scale over a negative domain reflects as d3's does), `t0 = T(d0)`, `t1 = T(d1)`, `k10 = 1 / (t1 − t0)` or 0 when `t0 == t1`. NaN input gives `unknown` if set, else `interpolator(NaN)` (d3 returns `undefined`, which a generic `R` cannot express); `k10 == 0` gives `interpolator(0.5)`; otherwise `interpolator(clamp ? t.clamp(0, 1) : t)` with `t = (T(x) − t0)·k10`.

**`Diverging::apply`** (`diverging.js:25–27`): with `u = T(x)`, `k10 = 0.5 / (t1 − t0)` and `k21 = 0.5 / (t2 − t1)` (each 0 for a flat half), and `s = −1` when `t1 < t0`, else 1: `t = 0.5 + (u − t1)·(s·u < s·t1 ? k10 : k21)`, then clamp and interpolate as above.

Both implement `Clone` (the `Arc` is shared) and a manual `Debug` that prints the domain, transform, clamp and unknown; neither has `invert` (d3 has none). `ticks`, `nice` and `tick_format` act on the domain's end stops through the transform's `ticks`, `nice` and `tick_format`, as `Continuous` does.

**`Ticks`** is implemented for `Sequential<R, T>` and `Diverging<R, T>` with `Domain = f64` and `Step = f64`, and for `Continuous<T, Rgb, N>` (Phase 7's `Linear<Rgb>`, which Phase 6's `R = f64` impl does not cover; Phases 33 and 35 put it on colorbars). Color ranges position ticks in [0, 1]:

- `tick_set(count)`: values `self.ticks(count)`, step `ticks::tick_step` over the end stops, and `count`, as Phase 6's `Continuous` impl;
- `position(&v, _offset)`: `Some(t)`, the normalized position before clamping and before the interpolator (for `Continuous<T, Rgb, N>`, the end stops' transformed fraction); `None` for NaN; a colorbar maps it to pixels;
- `extent()`: `[0.0, 1.0]`;
- `labels(set, locale)`: the transform's `tick_format(locale.number(), [d0, d_last], set.count(), None)` followed by `TickFormat::labels(set, locale)`, the same path as Phase 6's continuous impl (factor that path into a crate-internal function if it is a method body today; that is not an API change);
- `minor(set)`: the transform's `Transform::minor`.

##### Translator rules (Step 0, budget ≤ 250 lines of JS)

- string color arguments (`"steelblue"`, `"rgb(…)"`, `"#abc"`, `"hsl(…)"`) become parsed values in the target space (`"…".parse::<Lab>()` and friends, through prelude helpers), including inside `interpolateX(a, b)(t)`;
- `lab(…)`, `hcl(…)`, `cubehelix(…)`, `gray(…)`, `lch(…)` constructors and their parse calls;
- `assertLabEqual` and `assertHclEqual` become `check(Kind::InDelta(1e-6), [l, a, b], …)` plus a strict opacity check; `assertRgbApproxEqual` compares rounded channels;
- single-parameter arrow functions with an expression body (`t => t`, `t => t * 2`) become Rust closures over `f64`;
- `interpolate` passed as `piecewise`'s interpolator over numbers, and `piecewise(values)` without one, become `interpolate::number` (string values stay skips);
- `interpolateRgb.gamma(2.2)(a, b)` becomes `rgb_gamma(2.2, a, b)`;
- `scaleSequential()` without arguments becomes `sequential([0.0, 1.0], |t| t)`; `scaleSequentialLog/Pow/Sqrt/Symlog` become `.transform(…)` on d3's default domains ([1, 10] for log, [0, 1] otherwise); `scaleDiverging*` likewise on [0, 0.5, 1] ([0.1, 1, 10] for log) (`sequential.js:76`, `diverging.js:73`);
- `.interpolator(f)` maps to the builder; `.interpolator()`, `.range()` and `.domain()` getters map to `get_*`.

Anything else is a skip with its construct named. The rules are committed before Step 1, so each later step blesses against them.

##### Decisions this phase may meet

- The figure's scale table needs `PartialEq` on `Sequential` or `Diverging` — compare interpolators with `Arc::ptr_eq` and the other fields by value; record it as a decision.
- `piecewise` over fewer than two values (d3 fails at call time) — recommended: take `&[T; N]` and reject `N < 2` at monomorphization time, as Phase 5's `scale::piecewise` does; record it as a decision.
- A scheme sampled at NaN — the graded fixture never does; document what the port yields (NaN channels for the basis and formula schemes, entry 0 for the ramps, where d3 returns `undefined`); if a test needs d3's exact NaN output, record it as a decision.
- A d3-interpolate test exposes a difference between Phase 7's `Interpolate for Rgb` and d3's `interpolateRgb` — changing that impl is a change to an existing public item: record it as a decision and update every call site in the workspace in the same commit.

#### Steps

0. **Translator rules** (`dev/harness/translate/*`), as above. Verify: the translator's own tests; budget respected.
1. **Lab, Hcl, Cubehelix, gray, lch** (`color/{lab,hcl,cubehelix}.rs`, `color.rs` re-exports). Verify: regenerate, harness counts for d3-color; bless after reading the diff.
2. **Color interpolators** (`interpolate/{gamma,color,hue}.rs`) and the `Interpolate` impls for `Hsl`, `Lab`, `Hcl`, `Cubehelix`. Verify: d3-interpolate color files; bless.
3. **Numeric interpolators** (`interpolate/{basis,piecewise,quantize}.rs`, `Interpolate for Timestamp`), after compiling the one-line uses of the generic signatures. Verify: piecewise and quantize counts; bless. *Cut point: a second session starts at Step 4.*
4. **Scheme tables**: `dev/harness/gen/scheme_tables.mjs` and its output `scheme/{stops,viridis,magma,inferno,plasma,categorical}.rs`, committed together; `scheme.rs` re-exports the new categorical constants beside Phase 7's. Verify: a rerun leaves `git diff --exit-code core/src/scheme` clean.
5. **Scheme functions** (`scheme/{sequential,diverging,cyclical}.rs`); the ColorBrewer notice in `core/LICENSE-d3`; `dev/fixtures/schemes.mjs` and its output `dev/fixtures/schemes.json` (committed together); test `schemes_match_d3_at_11_samples`.
6. **`Sequential`** with transforms and `Ticks`, and `Ticks for Continuous<T, Rgb, N>`. Verify: bless `sequential-test.js` ids.
7. **`Diverging`** with `Ticks`; test `examples_in_formats_time_and_color`; `check.sh full`. Phase 24 may follow in this session.

#### Exit criteria

1. `scripts/check.sh quick` is green (tests under `TZ=Asia/Kathmandu`).
2. `cargo run -p rustd3-harness -- --check` is green with these floors in `passing.txt`'s header (≥ 70% of a file's assertions unless the table says otherwise):

   | Module | File (assertions) | Floor |
   | --- | --- | --- |
   | d3-color | `lab` (62), `hcl` (87), `cubehelix` (2), `gray` (4), `lch` (7) | 70% |
   | d3-interpolate | `rgb` (22), `hsl` (21), `hslLong` (21), `lab` (13), `hcl` (39), `hclLong` (21), `cubehelix` (29), `cubehelixLong` (29), `hue` (21), `quantize` (2) | 70% |
   | d3-interpolate | `piecewise` (21) | 14: seven assertions interpolate strings (`"a0"` → `"a10"`), skip "types decide" |
   | d3-interpolate | `number` (13), `round` (14) | Phase 4's floors, unchanged |
   | d3-scale | `sequential` (54), `diverging` (75) | 70% |

   Listed skips, not translated: `string`, `object`, `transformCss`, `value` ("types decide": d3's value dispatch and DOM transforms); `date` ("JS `Date` with numbers"); `numberArray` ("typed-array identity"); `array`, `discrete` and `zoom` ("not in v0.1"); d3-scale's `sequentialQuantile` and `radial` ("not in v0.1").
3. Test `schemes_match_d3_at_11_samples`: every scheme function equals d3 at `t = 0, 0.1, …, 1.0`, comparing `scheme(t).to_string()` with d3's output normalized by `d3.rgb(s).formatRgb()` in `dev/fixtures/schemes.json` (written by `dev/fixtures/schemes.mjs` through the resolve hook). No tolerance.
4. Test `examples_in_formats_time_and_color` asserts the spot-check rows marked "Part I".
5. Rerunning `scheme_tables.mjs` leaves `git diff --exit-code core/src/scheme` clean; every generated file carries its header.
6. `cargo tree -p rustd3-core -e normal --depth 1` lists jiff, thiserror and the optional rustd3-derive only.
7. `cargo doc -p rustd3-core --no-deps` passes with `RUSTDOCFLAGS="-D warnings"`; `scripts/check.sh full` is green at the last commit.

#### Spot checks

d3 7.9.0 (d3-color 3.1.0, d3-interpolate 3.0.1, d3-scale 4.0.2, d3-scale-chromatic 3.1.0) in Node through the resolve hook, unless marked. `steelblue`, `brown`, `white`, `purple`, `orange`, `red`, `green`, `blue` are the parsed CSS colors.

| Input | Expected | Source |
| --- | --- | --- |
| `Lab::from(Rgb::new(170.0, 187.0, 204.0))` | l 74.96879980931759, a −3.398998724348956, b −10.696507207853333 | `lab-test.js:12` |
| `"steelblue".parse::<Lab>()` l, a, b | 51.9862, −8.3628, −32.8327 | d3 |
| `"steelblue".parse::<Hcl>()` h, c, l | 255.7101, 33.8810, 51.9862 | d3 |
| `"white".parse::<Hcl>()` | h NaN, c NaN, l 100 | `hcl-test.js` |
| `"steelblue".parse::<Cubehelix>()` h, s, l | 202.8484, 0.6273, 0.4608 | d3 |
| `Rgb::from(Lab::gray(50.0))` | rgb(119, 119, 119) | d3 |
| `Rgb::from(Hcl::new(200.0, 40.0, 60.0))` | rgb(0, 163, 167) | d3 |
| `Rgb::from("steelblue".parse::<Lab>()?.brighter(1.0))` | rgb(121, 177, 230) | d3 |
| `interpolate::rgb(steelblue, brown)(0.5)` | rgb(118, 86, 111) | d3 |
| `interpolate::rgb_gamma(2.2, steelblue, brown)(0.5)` | rgb(128, 98, 134) | d3 |
| `interpolate::lab(…)(0.5)`, steelblue → brown | rgb(139, 93, 108) | d3 |
| `interpolate::hcl(…)(0.5)`, steelblue → brown | rgb(150, 81, 154) | d3 |
| `interpolate::hcl(white, steelblue)(0.5)` (white has no hue) | rgb(138, 194, 248) | d3; Part I |
| `interpolate::hcl_long(…)(0.5)`, steelblue → brown | rgb(35, 122, 57) | d3 |
| `interpolate::hsl(…)(0.5)` / `hsl_long`, steelblue → brown | rgb(141, 55, 173) / rgb(87, 173, 55) | d3 |
| `interpolate::cubehelix(1.0, steelblue, brown)(0.5)` | rgb(143, 62, 172) | d3 |
| `interpolate::cubehelix(3.0, purple, orange)(0.5)` | rgb(207, 4, 32) | d3 |
| `interpolate::rgb_basis(&[red, green, blue])` at 0.25 / 0.5 | rgb(133, 59, 5) / rgb(43, 85, 43) (at 0.5: segment 1 at 0, `(v0 + 4v1 + v2)/6` per channel: 42.5, 85.3, 42.5) | d3 |
| `interpolate::hue(350.0, 10.0)(0.5)` | 0 (350 + 20·0.5 = 360, wrapped) | d3 |
| `interpolate::basis(&[0.0, 10.0, 0.0])(0.5)` | 6.666666666666667 (`(0 + 40 + 0)/6`) | d3 |
| `interpolate::piecewise(&[red, green, blue], \|a, b\| interpolate::rgb(*a, *b))(0.75)` | rgb(0, 64, 128) (segment 1 at 0.5) | d3 |
| `interpolate::quantize(interpolate::number(0.0, 10.0), 4)` | [0, 3.333…, 6.666…, 10] | d3 |
| `interpolate::round(20.0, 10.0)(0.5)` | 15 (Phase 4, regression) | d3 |
| `scheme::blues(0.5)` | rgb(109, 174, 213) | d3 |
| `scheme::viridis(0.0)`, `(0.5)`, `(1.0)`, `.format_hex()` | #440154, #21918c, #fde725 | d3 |
| `scheme::magma(0.25).format_hex()` | #51127c | d3 |
| `scheme::turbo(0.5)` / `rd_bu(0.25)` / `cividis(0.5)` | rgb(149, 251, 81) / rgb(228, 130, 104) / rgb(127, 124, 117) | d3 |
| `scheme::sinebow(0.3)` / `rainbow(0.3)` / `warm(0.5)` / `cubehelix_default(0.5)` | rgb(88, 252, 42) / rgb(255, 120, 71) / rgb(255, 94, 99) / rgb(160, 121, 73) | d3 |
| `scale::sequential([0.0, 50.0], scheme::blues).apply(25.0)` | rgb(109, 174, 213) | d3; Part I (Scales) |
| `scale::sequential([0.0, 80.0], scheme::viridis).apply(20.0).format_hex()` | #3b528b (entry 64 of the table) | d3 |
| `scale::sequential([1.0, 1000.0], scheme::viridis)` with a base-10 log transform, `.apply(10.0)` | #31688e | d3 |
| `sequential([0.0, 1.0], \|t\| t).apply(2.0)` / with `.clamp(true)` | 2 / 1 | d3 |
| `sequential([0.0, 50.0], \|t\| t).ticks(5)` | [0, 10, 20, 30, 40, 50] | d3 |
| `sequential([0.0, 1.0], \|t\| t).range([0.0, 10.0]).apply(0.5)` | 5 | d3 |
| `sequential([0.0, 50.0], scheme::blues)`: `position(&25.0, 0.0)` / `position(&f64::NAN, 0.0)` / `extent()` | Some(0.5) / None / [0, 1] | rule above |
| `diverging([-1.0, 0.0, 4.0], \|t\| t)` at −0.5 / 2 | 0.25 / 0.75 (`0.5 − 0.5·0.5`, `0.5 + 2·0.125`) | d3 |
| `diverging([-1.0, 0.0, 4.0], scheme::rd_bu).apply(2.0)` | rgb(107, 172, 208) | d3 |
| `diverging([-100.0, 0.0, 100.0], \|t\| t)` with a symlog transform (constant 1), `.apply(10.0)` | 0.7597868532412204 (`0.5 + ln 11 · 0.5 / ln 101`) | d3 |
| `scale::linear().range([white, steelblue]).apply(0.5)` | rgb(163, 193, 218) (Phases 4 and 7, regression) | d3; Part I (Scales) |

#### Not in this phase

- `interpolate::string`, `transformCss`/`transformSvg`, d3's value dispatch `interpolate(a, b)`, `interpolateObject`, `interpolateNumberArray`, `interpolateDate` with numbers: types decide; listed skips.
- `interpolateDiscrete`, `interpolateArray`, `interpolateBasisClosed` and `interpolateRgbBasisClosed`: nothing in the library uses them and no other d3 test needs them, so v0.1 leaves them out; `interpolateZoom` (smooth zoom) is Later.
- d3's discrete k-color arrays (`schemeBlues[k]`); `scaleSequentialQuantile`, `scaleRadial`; `rangeRound` on color scales.
- Colorbars and figure handles for color scales (Phases 33, 35).
- `#[derive(Interpolate)]` (Phase 24).

### Phase 24 — `#[derive(Interpolate)]`

*M3 Core breadth · builds on phase 4 · about 220 lines (derive 110, re-export and docs 30, tests 80), 3 commits · runs in the session of Phase 23's last step, or with Phase 26*

#### Goal

`rustd3-derive` provides `#[derive(Interpolate)]` for structs with named fields, re-exported by core behind its `derive` feature next to the `interpolate::Interpolate` trait, so `use rustd3_core::interpolate::Interpolate;` brings in both the trait and the derive, as serde does. Use case 2's hand-written `impl Interpolate for Bar` becomes one attribute. Every named field interpolates by its type or fails to compile with E0277; `#[interpolate(skip)]` fields jump at t = 1. Unsupported shapes are documented as `compile_fail` doctests with passing twins.

#### Builds on

- Phase 1: the `rustd3-derive` package (`proc-macro = true`; syn 2 with its default features, quote 1, proc-macro2 1), core's optional dependency on it wired to the `derive` feature, and `check.sh full`'s feature matrix.
- Phase 4: `pub trait Interpolate: Clone { fn lerp(&self, to: &Self, t: f64) -> Self; }` with impls for `f64` and `f32`. The `Clone` supertrait means a deriving struct also derives `Clone`; the generated bounds make that hold (a type parameter in a non-skipped field is bounded by `Interpolate`, hence `Clone`; one in a skipped field by `Clone`). `PhantomData<T>` fields need `skip`.
- Phases 7 and 23: `Rgb` and `Hsl` fields (Phase 7) and, once Phase 23 has run, `Lab`, `Hcl`, `Cubehelix` and `Timestamp` fields interpolate through their impls with no extra work; the derive's own tests use only Phase 4's number impls and Phase 7's colors.
- Decision file `decisions/phase-04.md`.

#### Design

Module layout: `derive/src/lib.rs` holds the whole derive; `core/src/interpolate.rs` gains the feature-gated re-export and its doctests; `core/tests/main/derive.rs` holds the integration tests.

##### The macro

```rust
#[proc_macro_derive(Interpolate, attributes(interpolate))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream;

/// The derive's body, on proc-macro2 types so unit tests can call it.
fn expand(input: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error>;
```

`derive` converts in and out and turns `Err(e)` into `e.to_compile_error()`. A proc-macro crate can host `#[cfg(test)]` unit tests as long as they never touch `proc_macro::TokenStream`; that is why `expand` exists.

The expansion for

```rust
#[derive(Clone, Interpolate)]
struct Mark<T> { v: T, w: f64, #[interpolate(skip)] label: String }
```

is

```rust
impl<T> Interpolate for Mark<T>
where
    T: Interpolate,
{
    fn lerp(&self, to: &Self, t: f64) -> Self {
        Self {
            v: Interpolate::lerp(&self.v, &to.v, t),
            w: Interpolate::lerp(&self.w, &to.w, t),
            label: if t < 1.0 {
                ::core::clone::Clone::clone(&self.label)
            } else {
                ::core::clone::Clone::clone(&to.label)
            },
        }
    }
}
```

Rules:

- **Path.** The trait is named unqualified, `Interpolate`, so the derive works from core's own modules, from figure and through the facade wherever the trait is in scope — and it always is, because the derive is imported under the same name. A container attribute `#[interpolate(crate = "path")]` replaces it with `path::interpolate::Interpolate` (`crate = "rustd3_core"`, `crate = "::rustd3"`), for code that imports the derive by a full path without the trait. `crate = "::rustd3"` resolves to `::rustd3::interpolate::Interpolate`, so the facade re-exports the module (`pub use rustd3_core::interpolate;`), not only the trait; Phase 40 builds it that way.
- **Bounds.** For each non-skipped field whose type mentions a type parameter, add `FieldType: Interpolate` to the where clause; for each skipped such field, `FieldType: Clone`. Fields of concrete types get no bound, so a concrete field without an impl fails with E0277 at the derive site. That is why `Mark<f64>` interpolates and `Mark<String>` simply does not implement the trait. Decide "mentions a type parameter" by walking the `syn::Type` by hand (paths and their generic arguments, references, slices, arrays, tuples, pointers): syn's default features lack `visit`, and the workspace adds no features. Never decide by stringifying tokens.
- **Skip.** A field with `#[interpolate(skip)]` keeps the start value while `t < 1.0` and takes the end value from 1.0 on (d3 jumps non-numbers at the end). Unknown field or container keys are a `syn::Error` spanned on the key ("unknown interpolate attribute `foo`; expected `skip`", or "`crate`" on the container), never ignored.
- **Rejected shapes** (`syn::Error::new_spanned`, so they surface as ordinary compile errors): enums and unions, tuple and unit structs, all with "Interpolate derives only for structs with named fields". Lifetimes and const generics pass through `split_for_impl`.
- **Errors are collected**, not unwrapped: attribute parsing combines every error with `syn::Error::combine` and emits them all.

Skipping unsupported field types automatically (autoref specialization) was built in `proto/autoskip` and is not used: a mistyped field jumped silently, and a generic `Mark<f64>` fell back to jumping, because the choice is fixed where the derive expands. The derive's shape (struct-only check, named-field check, `skip`) comes from `proto/derive/interp_derive/src/lib.rs` (43 lines), with bounds and the `crate` attribute added and its `unwrap_or(false)` on malformed attributes replaced by errors.

##### Core side

```rust
// core/src/interpolate.rs, next to `pub trait Interpolate`
#[cfg(feature = "derive")]
pub use rustd3_derive::Interpolate;
```

The doc comment on this `pub use` carries the doctests below; rustdoc collects doctests from a re-export's doc comment. The derive crate's own docs point to core's for examples (a proc-macro crate cannot run doctests against core without a dev-dependency cycle).

Each failing doctest is `compile_fail` with a passing twin right after it, and every example derives `Clone` too, so a failing one fails only for the reason it documents:

1. A `String` field without `skip` (E0277; stable rustdoc does not check the code, the comment names it). Twin: the same struct with `#[interpolate(skip)]`.
2. `#[derive(Interpolate)] enum E { A }`. Twin: the struct form.
3. A tuple struct `struct P(f64, f64);`. Twin: named fields.
4. A field of a user type without an `Interpolate` impl. Twin: the field as `f64`. (Phase 4 implements `Interpolate for f32`, so an `f32` field interpolates.)

##### Decisions this phase may meet

- A caller needs tuple structs — keep them rejected for v0.1; `self.0` indexing is a six-line extension; record it as a decision before adding it.

#### Steps

1. **Derive crate** (`derive/src/lib.rs`): parsing, bounds, `skip`, `crate = "…"`, errors. Unit test: `expand` on `quote! { struct Mark<T> { v: T, w: f64, #[interpolate(skip)] label: String } }`, comparing the output's `to_string()` with the expansion above, whitespace-normalised. Verify: `cargo clippy -p rustd3-derive -- -D warnings`; `cargo test -p rustd3-derive`.
2. **Re-export and doctests** (`core/src/interpolate.rs`). Verify: run `cargo test -p rustd3-core --features derive --doc` once with a deliberately broken twin to confirm the doctests on the re-export execute, then restore it.
3. **Integration tests**: `core/tests/main/main.rs` declares `#[cfg(feature = "derive")] mod derive;`; `core/tests/main/derive.rs` asserts the spot checks and that use case 2's `Bar` derived equals the hand-written impl at t ∈ {0, 0.25, 0.5, 1}. Verify: `scripts/check.sh quick`, then `full`.

#### Exit criteria

1. `cargo test -p rustd3-core --features derive` passes, including the spot checks.
2. `cargo test -p rustd3-core --features derive --doc` runs every `compile_fail` doctest (the four above and the `Mark<String>` one in "Spot checks") and its passing twin.
3. `cargo test -p rustd3-core` without the feature still passes (the derive tests are `#[cfg(feature = "derive")]`).
4. `cargo clippy -p rustd3-derive -- -D warnings` and `cargo doc -p rustd3-derive` with `-D warnings` are clean.
5. `cargo run -p rustd3-harness -- --check` is green with unchanged counts (no d3 tests here).
6. `scripts/check.sh full` passes at the last commit (its no-default, default and all-features matrix covers the feature split).

#### Spot checks

`proto/derive/app` prints the `Bar` rows; the others follow from the expansion.

| Input | Expected |
| --- | --- |
| `Bar { y: 0.0, width: 0.0, opacity: 0.0, value: 0.0, name: "a" }.lerp(&Bar { y: 40.0, width: 200.0, opacity: 1.0, value: 10.0, name: "b" }, 0.25)`, `name` skipped | `Bar { y: 10.0, width: 50.0, opacity: 0.25, value: 2.5, name: "a" }` |
| same at t = 1.0 | `name == "b"` |
| same at t = 0.999 | `name == "a"` |
| `Mark { v: 0.0, w: 1.0, label: "a" }.lerp(&Mark { v: 10.0, w: 3.0, label: "b" }, 0.5)` (generic `Mark<T>`) | `Mark { v: 5.0, w: 2.0, label: "a" }` |
| `Nested { m: Mark<f64>, k: f64 }` from (a, 0) to (b, 4) at 0.25 | `Nested { m: Mark { v: 2.5, w: 1.5, label: "a" }, k: 1.0 }` |
| `Mark<String>: Interpolate` | not implemented (a `compile_fail` doctest calling `lerp` on it, with the `Mark<f64>` twin) |
| `#[derive(Interpolate)] struct Bad { name: String }` | `error[E0277]: the trait bound String: Interpolate is not satisfied` |
| `#[derive(Interpolate)] #[interpolate(crate = "rustd3_core")] struct S { x: f64 }` in a module without the trait imported | compiles; `S { x: 0.0 }.lerp(&S { x: 2.0 }, 0.5).x == 1.0` |
| Use case 2's `Bar`, derived vs hand-written, t ∈ {0, 0.25, 0.5, 1} | equal |
| `#[interpolate(skp)]` | a compile error naming `skp` |

#### Not in this phase

- Enums (variant-wise interpolation), tuple structs, automatic skipping of non-interpolatable fields.
- `anim::Marks` and `Animated` (Phase 30); they need only the trait.
- A `macro_rules!` alternative (`proto/derive/app` shows one; the derive is the design).
- The facade's re-export of `interpolate` (Phase 40).

### Phase 25 — Binned scales and array statistics

*M3 Core breadth · builds on phase 6 · about 1,000 lines of Rust plus ≤ 250 lines of translator JS, 7 commits*

#### Goal

Core gains the `array` module: the d3-array statistics and binning the charts need — `count`, `min`, `max`, `extent`, `sum`, `mean`, `median`, `variance`, `deviation`, `quantile`, `quantile_sorted`, `range`, `bisect_left`, `bisect_right`, and `bin` with the Sturges, Scott and Freedman–Diaconis threshold rules — and the binned scales `scale::Quantize<R>`, `scale::Quantile<R>` and `scale::Threshold<R>` with d3's lookups, `invert_extent`, and `Ticks` impls whose ticks are the bin edges. d3-array's tests for those functions and d3-scale's quantize, quantile and threshold tests grade it. Phase 34's histogram and KDE bandwidths and Phase 38's box plots, violins and ECDFs build on this module. The phase depends only on Phase 6 and may run earlier when a session has room.

#### Builds on

- Phase 3: `ticks::{ticks, tick_increment, tick_step, nice}` (d3-array's tick functions live there, not in `array`), `format::precision_fixed`.
- Phase 6: `scale::Ticks` (`type Domain`, `type Step`, `tick_set(count) -> ticks::Set<Domain, Step>`, `position(&v, offset) -> Option<f64>`, `labels(&set, &Locale) -> Vec<String>`, `minor` with an empty default, `extent`), `ticks::Set::new(values, step, count)`, `locale::Locale::number()`, `Format::format_all` and the axis grouping rule.
- Decision files `decisions/phase-03.md`, `phase-06.md`.

#### Design

Module layout:

```text
core/src/array.rs                          module root, re-exports
core/src/array/{count,extent,min,max,sum,mean,median,variance,deviation}.rs
core/src/array/{quantile,quickselect,range,bisect}.rs
core/src/array/bin.rs                      Bin, bin()
core/src/array/bin/{thresholds,bucket}.rs  Thresholds and the three rules; Bucket
core/src/scale/{quantize,quantile,threshold}.rs   the scales and their Ticks impls
```

##### `array`

Functions take `impl IntoIterator<Item = f64>` (slices through `.iter().copied()`) and ignore NaN as d3 ignores `null`, `NaN` and `undefined`; "no defined value" is `None`.

```rust
pub fn count(values: impl IntoIterator<Item = f64>) -> usize;           // non-NaN values
pub fn min(values: impl IntoIterator<Item = f64>) -> Option<f64>;
pub fn max(values: impl IntoIterator<Item = f64>) -> Option<f64>;
pub fn extent(values: impl IntoIterator<Item = f64>) -> Option<[f64; 2]>;
pub fn sum(values: impl IntoIterator<Item = f64>) -> f64;               // 0 when empty
pub fn mean(values: impl IntoIterator<Item = f64>) -> Option<f64>;
pub fn median(values: impl IntoIterator<Item = f64>) -> Option<f64>;
pub fn variance(values: impl IntoIterator<Item = f64>) -> Option<f64>;  // n − 1, Welford
pub fn deviation(values: impl IntoIterator<Item = f64>) -> Option<f64>;
pub fn quantile(values: impl IntoIterator<Item = f64>, p: f64) -> Option<f64>;
pub fn quantile_sorted(sorted: &[f64], p: f64) -> Option<f64>;          // R-7
pub fn range(start: f64, stop: f64, step: f64) -> Vec<f64>;
pub fn bisect_left(sorted: &[f64], x: f64) -> usize;
pub fn bisect_right(sorted: &[f64], x: f64) -> usize;
pub fn bin() -> array::Bin;
```

The algorithms are d3's line by line:

- `quantile` runs quickselect on a copy (`quantile.js:10–21`): a NaN `p` gives `None`; `p ≤ 0` or fewer than two values gives the minimum, `p ≥ 1` the maximum; otherwise R-7 interpolation between the two order statistics around `(n − 1)·p`. `quantile_sorted` is the same on sorted input without the selection; `median` is `quantile(values, 0.5)`.
- `variance` is Welford's algorithm with `n − 1` and `None` below two values; `deviation` is its square root.
- `range` computes `n = ceil((stop − start)/step)` values as `start + i·step`, so `range(0, 1, 0.2)` ends with 0.6000000000000001, as d3. It returns an empty vector for a zero, NaN or infinite step and for an infinite or NaN length, without looping.
- `bisect_left` and `bisect_right` assume ascending input. A NaN `x` returns `hi` (the slice length), as `bisector.js` does (`compare(x, x) != 0` short-circuits).

##### `array::Bin` — d3's `bin()` (`bin.js:10–118`)

```rust
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bin { domain: Option<[f64; 2]>, thresholds: bin::Thresholds }
impl Bin {
    pub fn domain(self, domain: [f64; 2]) -> Self;
    pub fn thresholds(self, thresholds: bin::Thresholds) -> Self;
    pub fn layout(&self, values: &[f64]) -> Vec<bin::Bucket>;
}
// array/bin/thresholds.rs
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Thresholds {
    #[default] Sturges, Scott, FreedmanDiaconis,
    Count(usize),
    Values(Vec<f64>),
}
pub fn sturges(values: &[f64]) -> usize;
pub fn scott(values: &[f64], min: f64, max: f64) -> usize;
pub fn freedman_diaconis(values: &[f64], min: f64, max: f64) -> usize;
// array/bin/bucket.rs
/// One bin: its half-open extent and the indices of its values. Plain data.
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket { pub x0: f64, pub x1: f64, pub indices: Vec<usize> }
```

Buckets carry indices into the input, not copies, so results point back to their data (Part I, Data and identity). The count path keeps d3's subtleties: with the default domain (`extent`) the domain is niced to the tick count; the quantized fast path when the first tick ≤ x0 (`tickIncrement` step, negative steps for fractional increments, and the `j + (tz[j] <= x)` adjustment); the upper bound extended by one step when the last threshold coincides with the maximum; thresholds outside the domain dropped; values outside the domain ignored. A NaN-only or empty input gives d3's single empty bucket, with NaN where d3 has NaN or `undefined` edges; follow `bin-test.js`.

##### Binned scales

```rust
pub struct Quantize<R> { domain: [f64; 2], range: Vec<R> }
pub fn quantize<R>(domain: [f64; 2], range: impl IntoIterator<Item = R>) -> Quantize<R>;
impl<R> Quantize<R> {
    pub fn domain(self, d: [f64; 2]) -> Self;
    pub fn get_domain(&self) -> [f64; 2];
    pub fn range(self, r: impl IntoIterator<Item = R>) -> Self;
    pub fn get_range(&self) -> &[R];
    pub fn apply(&self, x: f64) -> Option<&R>;           // None: NaN, empty range
    pub fn thresholds(&self) -> Vec<f64>;                // the n − 1 inner edges
    pub fn nice(self, count: usize) -> Self;             // linearish, as d3
    pub fn ticks(&self, count: usize) -> Vec<f64>;       // d3's linear ticks
    pub fn tick_format(&self, count: usize, specifier: Option<&str>)
        -> Result<format::Format, format::InvalidFormat>;
}
impl<R: PartialEq> Quantize<R> { pub fn invert_extent(&self, y: &R) -> Option<[f64; 2]>; }

pub struct Quantile<R> { sorted: Vec<f64>, range: Vec<R>, thresholds: Vec<f64> }
pub fn quantile<R>(domain: impl IntoIterator<Item = f64>, range: impl IntoIterator<Item = R>)
    -> Quantile<R>;
// apply(&self, x) -> Option<&R>; quantiles(&self) -> &[f64];
// get_domain(&self) -> &[f64] (sorted, NaN dropped);
// invert_extent(&self, y: &R) -> Option<[f64; 2]> where R: PartialEq

pub struct Threshold<R> { domain: Vec<f64>, range: Vec<R> }
pub fn threshold<R>(domain: impl IntoIterator<Item = f64>, range: impl IntoIterator<Item = R>)
    -> Threshold<R>;
// apply(&self, x) -> Option<&R>: bisect_right over the domain; None for NaN or a missing bucket.
// invert_extent(&self, y: &R) -> Option<[Option<f64>; 2]>: open ends are None, as d3's undefined.
```

`Quantile` recomputes its thresholds as `quantile_sorted(sorted, i/n)` for `i in 1..n` (n = range length) whenever the domain or range changes (`quantile.js`). The domain of `Threshold` is `f64` (d3 also accepts strings and dates; those tests are listed skips). `Threshold` does not validate `range.len() == domain.len() + 1`: as in d3, a missing bucket is a lookup result, which the `Option` already expresses. A `Threshold` built from an unsorted domain is undefined, as in d3: no sort, no validation, and its doc comment says so.

**`Ticks`** for all three, with `Domain = f64` and `Step = f64`:

- `tick_set(count)` keeps `count` only to store it. Values: `Quantize` → `[x0, thresholds…, x1]`; `Quantile` → `[min, quantiles…, max]`; `Threshold` → its domain (the open ends have no edge). Step: the smallest gap between consecutive values, which sets the label precision as d3's `tickFormat` uses its tick step.
- `position(&v, _)`: the value's place on a colorbar in [0, 1]. `Quantize` and `Quantile`: `(v − first)/(last − first)`. `Threshold`: the i-th threshold sits at `(i + 1)/(n + 1)` for n thresholds (equal-width buckets, open ends); a value not in the domain gives `None`. NaN gives `None` everywhere. Phase 35 owns colorbar geometry and may change the `Threshold` placement, so its doc comment calls this the default placement.
- `extent()`: `[0.0, 1.0]`.
- `labels(set, locale)`: the linear labelling of Phase 6 driven by the set's step — specifier `,f` with precision `format::precision_fixed(step)`, formatted through `locale.number()` with `Format::format_all`, so the axis grouping rule applies. Reuse Phase 6's crate-internal code path; factoring it out of a method body is not an API change.
- `minor`: the default (none).

##### Translator rules (Step 0, budget ≤ 250 lines of JS)

- string arrays (`["a", "b", "c"]`) become `vec!["a", "b", "c"]`;
- `Float64Array`/`Uint8Array` literals become `vec![…]` of `f64`; `undefined` and `null` elements become `f64::NAN`;
- single-parameter arrow accessors with a member-expression body over an array of object literals (`[{x: 1}, …]`, `(d) => d.x`) become the projected `vec![…]`, computed at translation time;
- `d3.min(…)` and the other reductions over literal arrays map to the `array::` functions;
- `bin().domain(…).thresholds(n | [...] | thresholdScott)(data)` becomes the `Bin` builder plus `layout`; `bins.map(b => [b.x0, b.x1, b.length])` becomes `buckets.iter().map(|b| (b.x0, b.x1, b.indices.len() as f64))`;
- `scaleQuantize().domain(…).range([...])` chains (and the quantile and threshold forms) with string or number ranges; `s.invertExtent("b")` becomes `invert_extent(&"b")`.

Anything else is a skip with its construct named. Commit before Step 1.

##### Decisions this phase may meet

- A d3-array test's expected `undefined` cannot be told from NaN through `Obs` — keep NaN in `Bucket` and the `Option` returns as designed; make the assertion a skip with its construct named; record it as a decision if more than a handful of assertions need it.

#### Steps

0. **Translator rules**, as above. Commit before any implementation.
1. **Reductions** (`array/{count,extent,min,max,sum,mean,median,variance,deviation}.rs`). Verify: regenerate; bless after reading the diff.
2. **Quantiles, range, bisect** (`array/{quantile,quickselect,range,bisect}.rs`). Bless.
3. **Bin and thresholds** (`array/bin.rs`, `array/bin/{thresholds,bucket}.rs`). Bless.
4. **Quantize** with `Ticks`. Bless `quantize-test.js`.
5. **Quantile and Threshold** with `Ticks`. Bless.
6. **Degenerate inputs and docs**: the degenerate-input tests, doc examples, `check.sh full`.

#### Exit criteria

1. `scripts/check.sh quick` is green; `cargo test -p rustd3-core array:: scale::quantize scale::quantile scale::threshold` passes.
2. `cargo run -p rustd3-harness -- --check` is green with these floors (≥ 70% of a file's assertions unless stated):

   | Module | File (assertions) | Floor |
   | --- | --- | --- |
   | d3-array | `bin` (36), `extent` (46), `min` (46), `max` (46), `mean` (38), `median` (53), `quantile` (72), `deviation` (29), `variance` (30), `sum` (42), `range` (77), `count` (7), `threshold/sturges` (3), `threshold/scott` (4), `threshold/freedmanDiaconis` (4) | 70% |
   | d3-array | `bisect` (77) | 50%: half its tests are `bisector` with accessors and comparators, skip "bisector accessors" |
   | d3-scale | `quantize` (51), `quantile` (48), `threshold` (31) | 70% |

   Skip categories inside those files: `bin` with `Date` values ("Date values"); string-valued and accessor-function inputs ("JS coercion"); typed-array identity; iterables of objects with `valueOf`; `bisector` with accessors; `quantileIndex`; `InternMap`. Files not translated, in the harness's skip list: `fsum`, `fcumsum`, `mode`, `minIndex`, `maxIndex`, `cumsum` ("Later"), and the d3-array files of functions no chart needs ("not in v0.1").
3. The spot checks pass as tests in `core/tests/main/array.rs` and `core/tests/main/binned.rs`.
4. Degenerate inputs: `array::range(0.0, 1.0, 0.0)` and `array::range(0.0, f64::INFINITY, 1.0)` return empty without looping; `bin` on ±∞ or all-NaN data returns d3's result without hanging (each test under a 1 s timeout).
5. `scripts/check.sh full` is green at the last commit.

#### Spot checks

d3 7.9.0 (d3-array 3.2.4, d3-scale 4.0.2) in Node through the resolve hook; the `Ticks` rows follow from the rules above.

| Input | Expected |
| --- | --- |
| `quantize([0, 1], ["a", "b", "c"])` at 0.2 / 0.5 / 0.9 | a / b / c |
| `.thresholds()` | [0.3333333333333333, 0.6666666666666666] |
| `.invert_extent(&"b")` | [0.3333333333333333, 0.6666666666666666] |
| `.tick_set(5)` values / step; `position(&0.5, 0.0)` | [0, 0.3333333333333333, 0.6666666666666666, 1] / 0.3333333333333333; Some(0.5) |
| `quantize([0.13, 9.71], [0, 1, 2, 3]).nice(10)` domain | [0, 10] |
| `quantize([0, 10], …).ticks(5)` | [0, 2, 4, 6, 8, 10] |
| `quantile([3, 6, 7, 8, 8, 10, 13, 15, 16, 20], [0, 1, 2, 3]).quantiles()` | [7.25, 9, 14.5] |
| that scale at 7.9 / 8 / 15.5 | 1 / 1 / 3 |
| `.invert_extent(&2)` | [9, 14.5] |
| `quantile([6, NaN, 3, 8], …).get_domain()` | [3, 6, 8] |
| `threshold([0, 1], ["a", "b", "c"])` at −1 / 0 / 0.5 / 1 / 2 | a / b / b / c / c |
| `.invert_extent` of a / b / c | [None, Some(0)] / [Some(0), Some(1)] / [Some(1), None] |
| its `position(&0.0, 0.0)` / `(&1.0, 0.0)` / `(&0.5, 0.0)` | Some(1/3) / Some(2/3) / None |
| `array::quantile([3, 6, 7, 8, 8, 10, 13, 15, 16, 20], p)` at 0.25 / 0.5 / 0.75 | 7.25 / 9 / 14.5 |
| `quantile([1, 2, 3], NaN)`, `quantile_sorted(&[], 0.5)` | None, None |
| `median([1, NaN, 3, 2])` | 2 |
| `mean([1, 2, NaN, 4])` | 2.3333333333333335 |
| `variance([5, 1, 2, 3, 4])` / `deviation` | 2.5 / 1.5811388300841898 |
| `extent([3, NaN, -1, 7])` | [−1, 7] |
| `min([])`, `max([])` | None, None |
| `sum([0.1, 0.2, 0.3])` | 0.6000000000000001 |
| `range(0, 1, 0.2)` | [0, 0.2, 0.4, 0.6000000000000001, 0.8] |
| `bisect_left(&[1, 2, 2, 3], 2)` / `bisect_right` | 1 / 3 |
| `bisect_left(&[1, 2, 3], NaN)` / `bisect_right` | 3 / 3 |
| `bin::sturges(&range(0, 20, 1))` / `scott(…, 0, 19)` / `freedman_diaconis(…, 0, 19)` | 6 / 3 / 3 |
| `bin().layout(&[0, 1, 1, 2, 3, 5, 8, 13, 21])` as (x0, x1, len) | (0,5,5) (5,10,2) (10,15,1) (15,20,0) (20,25,1) |
| `bin().domain([0, 20]).thresholds(Count(4))`, same data | (0,5,5) (5,10,2) (10,15,1) (15,20,0) |
| `bin().thresholds(Values(vec![5, 10]))`, same data | (0,5,5) (5,10,2) (10,21,2) |

#### Not in this phase

- `fsum`, `fcumsum`, `Adder`, `mode`, `bisect_center`, `min_index`, `max_index`, `cumsum`: Later; no chart uses them.
- d3-array functions no chart needs: `group`, `rollup`, `index`, `flatGroup`, set operations, `cross`, `zip`, `pairs`, `permute`, `shuffle`, `transpose`, `merge`, `blur`, `rank`, `scan`, `least`/`greatest`, `sort` with comparators, `bisector` with accessors (Rust's `partition_point` covers it), `quantileIndex`, `InternMap`. Their files stay untranslated.
- `scaleSequentialQuantile` and `scaleRadial` (listed skips).
- Colorbar geometry for binned scales (Phase 35); histogram and box-plot marks (Phases 34, 38); KDE (Phase 34).

### Phase 26 — Number locales and `fmt!`

*M3 Core breadth · builds on phase 6 · about 350 lines of Rust plus ~300 generated, a 90-line generator and ≤ 100 lines of translator JS, 5 commits*

#### Goal

`format::Locale` gains the eleven bundled tables Part I names — en-US and the ten time-table languages (de-DE, fr-FR, es-ES, it-IT, pt-BR, nl-NL, pl-PL, ru-RU, ja-JP, zh-CN) — generated from d3-format's locale JSON; the crate gains `fmt!`, the compile-time-checked format literal, whose invalid specifiers fail with `error[E0080]: evaluation panicked: invalid d3 format specifier: …`; the harness reads every one of d3-format's 59 locale files at translation time, so the Indian and Arabic locale tests pass; and `scripts/check.sh full` gains the generator-freshness step that keeps every generated table in the workspace (Phase 23's scheme tables and this phase's locales) equal to its generator's output. The phase depends only on Phase 6 and may run earlier; it may share a session with Phase 24.

#### Builds on

- Phase 3: `format::Locale` with `Cow<'static, …>` fields and `group_from`; its constants `NEUTRAL` (also `Default`; groups from five integer digits, so a single four-digit value stays ungrouped) and `EN_US`, built by the private `const fn from_table(Table) -> Locale` over a `Copy` struct of `&'static` fields (`decimal`, `thousands`, `grouping: &'static [usize]`, `currency: [&'static str; 2]`, `numerals: Option<&'static [&'static str; 10]>`, `percent`, `minus`, `nan`, `group_from`); the runtime `Locale::new(format::Definition { … })` with public `Cow` fields; `format::Format`, `format::Specifier` (`FromStr`), `format::InvalidFormat`, `format::format(spec)`.
- Phase 6: `locale::Locale` with its number half; the combined constants for the new languages are Phase 28's, since they need the time half.
- Phase 2: the translator, the resolve hook, ratchet and floors. Phase 23, when it has run (the usual order): `dev/harness/gen/scheme_tables.mjs` and the files it generates.
- Decision files `decisions/phase-03.md`, `phase-06.md`.

#### Design

Module layout: `core/src/format.rs` declares `mod locales; pub mod literal;`; `core/src/format/locales.rs` holds only the `mod` lines and a doc table of the eleven constants; `core/src/format/locales/{en_us,de_de,fr_fr,es_es,it_it,pt_br,nl_nl,pl_pl,ru_ru,ja_jp,zh_cn}.rs` are generated; `core/src/format/literal.rs` holds `is_valid` and `fmt!`. `from_table` and `Table` become `pub(crate)` (an internal change). If Phase 3 wrote `EN_US` by hand, the generated table must equal it field for field; moving its definition into `format/locales/en_us.rs` keeps the path `format::Locale::EN_US` and is not an API change.

##### Generated tables

One file per locale, each an inherent `impl` block adding one constant:

```rust
// core/src/format/locales/de_de.rs — generated by dev/harness/gen/format_locales.mjs
// from d3-format 3.1.2 locale/de-DE.json. Do not edit.
impl crate::format::Locale {
    /// German (Germany): `1.234.567,5`, `1.234,50 €`.
    pub const DE_DE: Self = Self::from_table(Table {
        decimal: ",", thousands: ".", grouping: &[3], currency: ["", "\u{a0}€"],
        numerals: None, percent: "%", minus: "\u{2212}", nan: "NaN", group_from: 1,
    });
}
```

`group_from` is 1 for every d3 table (d3 groups from the first digit; only `NEUTRAL` uses 5). Constants: `EN_US`, `DE_DE`, `FR_FR`, `ES_ES`, `IT_IT`, `PT_BR`, `NL_NL`, `PL_PL`, `RU_RU`, `JA_JP`, `ZH_CN`. Fields absent from a JSON file take d3's defaults (`locale.js:12–21`: decimal ".", no grouping, currency ["", ""], identity numerals, percent "%", minus U+2212, nan "NaN"). fr-FR is the one bundled table with its own `percent` (U+202F "%"). Every non-ASCII character is emitted as `\u{…}`, so U+00A0 and U+202F stay visible in a diff.

##### The generator (`dev/harness/gen/format_locales.mjs`)

Plain Node, no packages: it reads the eleven JSON files under `dev/harness/d3/d3-format/locale/`, sorts keys, writes the Rust files and `locales.rs`, and prints nothing on success. It also writes `dev/fixtures/format_locales.json` by running d3's own `formatLocale` through the resolve hook on a fixed probe list per locale: `,.1f` of 1234567.5; `$,.2f` of 1234.5 and −1234.5; `.1%` of 0.123; `,d` of −12000; `.3s` of 0.000042. Run it as `node --import dev/harness/translate/d3-resolve.mjs dev/harness/gen/format_locales.mjs`.

##### Generator freshness in `check.sh full`

`scripts/check.sh full` reruns every table generator and fails on any difference, so a hand edit to a generated file cannot survive a phase:

```sh
node --import dev/harness/translate/d3-resolve.mjs dev/harness/gen/scheme_tables.mjs
node --import dev/harness/translate/d3-resolve.mjs dev/harness/gen/format_locales.mjs
git diff --exit-code \
  core/src/scheme/stops.rs core/src/scheme/viridis.rs core/src/scheme/magma.rs \
  core/src/scheme/inferno.rs core/src/scheme/plasma.rs core/src/scheme/categorical.rs \
  core/src/format/locales dev/fixtures/format_locales.json
```

Phase 28 appends its time-locale generator to the same step. If this phase runs before Phase 23, the step starts with the locale generator alone and Phase 23 adds its scheme generator to it. Inside `cargo test`, `bundled_locales_equal_their_json` is the second guard.

##### Harness: any locale file at translation time

`locale("xx-YY")` in a d3 test reads `dev/harness/d3/d3-format/locale/xx-YY.json` at translation time and emits `format::Locale::XX_YY` when that table is bundled, otherwise a runtime definition literal through Phase 3's `Locale::new(Definition { … })`, numerals and multi-size grouping included (the ar-* tables carry ten Arabic-Indic digits; en-IN carries grouping `[3, 2, 2, …]`). The JSON-iterating "locale data is valid" test becomes a generated loop over all 59 definitions that builds each without error. This replaces Phase 3's skips of category "gap: phase 26". A core test asserts each bundled constant equals the runtime definition built from its JSON, which it reads with serde_json (core's dev-dependency from Phase 1).

##### `fmt!` (`core/src/format/literal.rs`)

```rust
/// Whether `spec` is a valid d3 format specifier; usable in `const`.
pub const fn is_valid(spec: &str) -> bool;

#[macro_export]
macro_rules! fmt {
    ($spec:literal) => {{
        const _: () = if !$crate::format::literal::is_valid($spec) {
            ::core::panic!("{}", ::core::concat!("invalid d3 format specifier: ", $spec))
        };
        $crate::format::Locale::NEUTRAL
            .format($spec)
            .expect("invariant: fmt! checked the specifier at compile time")
    }};
}
```

- `is_valid` walks the bytes of `[[fill]align][sign][symbol][0][width][,][.precision][~][type]` with `while` loops only (no iterators in a `const fn`). The grammar is d3's regex (`formatSpecifier.js:2`), `^(?:(.)?([<>=^]))?([+\-( ])?([$#])?(0)?(\d+)?(,)?(\.\d+)?(~)?([a-z%])?$` with the `i` flag: the type is any ASCII letter or `%` (unknown letters parse and later format as `.12~g`, `locale.js:41`), `.` must be followed by at least one digit, and the fill is one character other than a line terminator. The fill may be any UTF-8 character, so decode its length from the first byte (`0xxxxxxx` 1, `110xxxxx` 2, `1110xxxx` 3, `11110xxx` 4) before testing for an align character. The agreement test in the exit criteria is the arbiter.
- The panic uses `panic!("{}", concat!(…))`, not `panic!(concat!(…))`, so a `{` fill cannot break the format string. Checked on Rust 1.97: `fmt!("{^10..f")` fails with `error[E0080]: evaluation panicked: invalid d3 format specifier: {^10..f`.
- The runtime call uses the neutral default. The `expect` is a true invariant once the agreement test holds, and it is the only `expect` in the macro.
- `fmt!` returns `format::Format` by value. A user who wants a locale writes `format::Locale::DE_DE.format("…")?`.
- Malformed specifiers at run time stay errors: `format::format(spec)` and `Locale::format(spec)` return `Result<Format, InvalidFormat>`; nothing in this phase returns `Option` for a malformed specifier.

##### Decisions this phase may meet

- The agreement test finds a specifier that `is_valid` and `Specifier::from_str` judge differently — the parser ports d3's regex and is the reference: change `is_valid`. If the parser itself deviates from d3's regex, changing it is a change to a Phase 3 item: record it as a decision and update every call site in the same commit.

#### Steps

1. **Generator and tables** (`dev/harness/gen/format_locales.mjs`, `core/src/format/locales.rs`, `locales/*.rs`, `dev/fixtures/format_locales.json`). Verify: `bundled_number_locales_match_d3`, `bundled_locales_equal_their_json`, `cargo doc`.
2. **Harness: locale files at translation time** (translator, budget ≤ 100 lines of JS; regenerate; bless the locale, arabicLocale and defaultLocale tests after reading the diff). Verify: `cargo run -p rustd3-harness -- --check`.
3. **`is_valid` and `fmt!`** (`format/literal.rs`): doctests, `literal_check_agrees_with_parser`.
4. **Freshness step** in `scripts/check.sh full`, covering `format_locales.mjs` and, when Phase 23 has run, `scheme_tables.mjs`. Verify: a deliberate one-character edit to a generated file (`core/src/format/locales/de_de.rs`, and `core/src/scheme/categorical.rs` when it exists) makes `full` fail; revert it.
5. **Degenerate and Part I checks**, `check.sh full`.

#### Exit criteria

1. `scripts/check.sh quick` is green.
2. Rerunning the generators that exist (this phase's, and Phase 23's when it has run) leaves `git diff --exit-code` clean on the files listed above, and `scripts/check.sh full` runs that check itself; test `bundled_locales_equal_their_json` guards between runs.
3. Test `bundled_number_locales_match_d3` passes: the eleven locale rows in "Spot checks", each against the d3 output stored in `dev/fixtures/format_locales.json`.
4. `cargo run -p rustd3-harness -- --check` is green with floors of 70% on `locale-test.js` (45), `arabicLocale-test.js` (27) and `defaultLocale-test.js` (6); the "locale data is valid" loop over the 59 files counts as its single assertion. Expected change: the en-IN block (20 assertions) and the 27 ar-* assertions move from skip to pass.
5. `fmt!` doctests: Part I's three examples pass; `fmt!("10..f")` is a `compile_fail` doctest with a passing twin.
6. Test `literal_check_agrees_with_parser`: `is_valid` accepts exactly the specifiers `format::Specifier::from_str` accepts, over every specifier in d3-format's tests plus every string of length ≤ 4 over the alphabet `0 1 , . ~ $ # + - ( space < > = ^ e f g r s % x z`.
7. `scripts/check.sh full` is green at the last commit.

#### Spot checks

d3 7.9.0 (d3-format 3.1.2) `formatLocale(json)` in Node. ⍽ marks U+00A0 and ⍿ marks U+202F; − is U+2212.

| Locale | `,.1f` of 1234567.5 | `$,.2f` of 1234.5 | `$,.2f` of −1234.5 | `.1%` of 0.123 | `,d` of −12000 |
| --- | --- | --- | --- | --- | --- |
| `EN_US` | 1,234,567.5 | $1,234.50 | −$1,234.50 | 12.3% | −12,000 |
| `DE_DE` | 1.234.567,5 | 1.234,50⍽€ | −1.234,50⍽€ | 12,3% | −12.000 |
| `FR_FR` | 1⍽234⍽567,5 | 1⍽234,50⍽€ | −1⍽234,50⍽€ | 12,3⍿% | −12⍽000 |
| `ES_ES` | 1.234.567,5 | 1.234,50⍽€ | −1.234,50⍽€ | 12,3% | −12.000 |
| `IT_IT` | 1.234.567,5 | €1.234,50 | −€1.234,50 | 12,3% | −12.000 |
| `PT_BR` | 1.234.567,5 | R$1.234,50 | −R$1.234,50 | 12,3% | −12.000 |
| `NL_NL` | 1.234.567,5 | €⍽1.234,50 | −€⍽1.234,50 | 12,3% | −12.000 |
| `PL_PL` | 1.234.567,5 | 1.234,50zł | −1.234,50zł | 12,3% | −12.000 |
| `RU_RU` | 1⍽234⍽567,5 | 1⍽234,50⍽₽ | −1⍽234,50⍽₽ | 12,3% | −12⍽000 |
| `JA_JP` | 1,234,567.5 | 1,234.50円 | −1,234.50円 | 12.3% | −12,000 |
| `ZH_CN` | 1,234,567.5 | ¥1,234.50 | −¥1,234.50 | 12.3% | −12,000 |
| en-IN (harness, runtime JSON) | 12,34,567.5 | ₹1,234.50 | −₹1,234.50 | 12.3% | −12,000 |
| ar-001 (harness, runtime JSON) | ١٬٢٣٤٬٥٦٧٫٥ | ١٬٢٣٤٫٥٠ | −١٬٢٣٤٫٥٠ | ١٢٫٣% | −١٢٬٠٠٠ |

| Input | Expected | Source |
| --- | --- | --- |
| `locale("en-IN").format(",")(10000000.4543)` | 1,00,00,000.4543 | `locale-test.js:43` |
| `fmt!("$,.2f").format(-1234.5)` (the neutral default leaves four digits ungrouped) | −1234.50 | Part I |
| `fmt!(",.1f").format(1234567.5)` | 1⍿234⍿567.5 | Part I |
| `fmt!(".3s").format(0.000_042)` | 42.0µ | Part I |
| `format::format(".1%")?.format(0.123)` | 12.3% | Part I |
| `format::format("10..f")` | `Err(InvalidFormat("10..f"))` | Part I |
| `fmt!("10..f")` | `error[E0080]: evaluation panicked: invalid d3 format specifier: 10..f` | Part I |
| `format::Locale::DE_DE.format(",.1f")?.format(1234567.5)` | 1.234.567,5 | Part I |
| Neutral `,.1f` of 1234567.5 / `$,.2f` of 1234.5 | 1⍿234⍿567.5 / 1234.50 (Phase 3, regression) | Part I |

#### Not in this phase

- `locale::Locale::{DE_DE, …}`, the combined constants with their time halves (Phase 28); `fig.locale` (Phase 17).
- The other 48 d3-format locales as bundled constants (they load at run time through `Locale::new`).
- Axis-level grouping (Phase 6) and offset notation (Phase 16).
- A `fmt!` variant taking a locale; `fmt!` for time specifiers.

### Phase 27 — Time I: intervals and time scales

*M3 Core breadth · builds on phases 9, 10 · about 2,100 lines of Rust plus ≤ 250 lines of translator JS, 11 commits · two sessions: 27a (Steps 0–5, ~1,000 lines) and 27b (Steps 6–10, ~1,100 lines), cut after Step 5*

#### Goal

Core gains time with explicit zones on jiff: `time::Interval`, d3-time's calendar intervals plus `Microsecond` and `Nanosecond`, with floor, ceil, round, offset, count, every and a lazy range; d3's tick functions as the free functions `time::{ticks, ticks_with, tick_interval, utc_ticks, utc_tick_interval}`; the `time` and `utc` scales, `time::Time`, with apply, invert, clamp, nice, ticks and `tick_interval`, computing in i128 nanoseconds from the domain start; the time impls of Phase 10's `interact::Rescale` and `scale::Pad`; and `series::Recording<Timestamp>`. d3-time (36 test files, 1,167 assertions) and d3-scale's time and utcTime tests without tick formats grade it under America/Los_Angeles; `test-time-zones` is ported by hand. Phase 27a ends with the intervals and the free tick functions; 27b adds the scale, gestures, padding and recordings.

#### Builds on

- Phase 2: `testkit::zone(name)` over the bundled tzdb; `Obs`, `check`, the ratchet and floors; the harness prelude.
- Phase 3: `ticks::tick_step` (d3-array's `tickStep`).
- Phase 9: `series::Recording<X>`, `Clock` (`Regular { start, period }` or `Explicit(Vec<X>)`), `Columns`, the M4 queries generic over the x type with `f64` impls, behind the `series::Coordinate` trait (`type Period`, `at(start, period, index)`, `index(start, period, value)`, `is_valid`, `is_valid_period`); `Recording::columns(&[X])` takes data-space edges; Phase 9's generic test suite.
- Phase 10: `interact::Transform` (`invert_x`, `invert_y`), `interact::Rescale` (`rescale_x`, `rescale_y`, `pan_px -> Option<Self>`, `zoom_px -> Option<Self>`), `scale::Pad` (`type Domain`, `pad(extent, margin, sticky)`) with their `f64` impls and the rule that a rejected zoom step returns `None`.
- Decision files `decisions/phase-09.md`, `phase-10.md`.

#### Design

Module layout:

```text
core/src/lib.rs                       pub mod time; pub use jiff;
core/src/time.rs                      module root: time(), utc(), re-exports
core/src/time/interval.rs             Interval
core/src/time/interval/{fine,clock,calendar}.rs   the unit families
core/src/time/interval/range.rs       interval::Range (lazy iterator)
core/src/time/ticks.rs                free tick functions (27a)
core/src/time/scale.rs                Time (27b)
core/src/time/scale/{ticks,rescale,pad}.rs        (27b)
core/src/series/clock/timestamp.rs    Coordinate for Timestamp (27b)
```

`pub use jiff;` lets figure, iced, the examples and users name jiff types as `rustd3_core::jiff` without a manifest entry of their own. Library code never calls `TimeZone::system`, `Zoned::now` or `Timestamp::now`; every zone is a parameter. Time computes in i128 nanoseconds and no `Timestamp` passes through an `f64` epoch value: in 2026 an `f64` of epoch nanoseconds (about 1.79·10¹⁸) has a 256 ns ulp. Zone operations cost about a microsecond, so they happen only where the calendar needs them.

##### `time::Interval`

A closed enum: a trait over intervals is not used, and custom intervals are out of scope. Every variant carries its `every` step; a step of 0 behaves as 1.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interval {
    Nanosecond(u32), Microsecond(u32), Millisecond(u32),
    Second(u32), Minute(u32), Hour(u32),
    /// Calendar days; `every(k)` keeps days of the month ≡ 1 (mod k) (d3's timeDay).
    Day(u32),
    /// Days counted from 1970-01-01 in the zone's calendar (d3's unixDay, utc ticks).
    UnixDay(u32),
    /// Weeks starting on the weekday (d3's timeSunday … timeSaturday).
    Week(jiff::civil::Weekday, u32),
    Month(u32), Year(u32),
}
impl Interval {
    pub fn floor(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp>;
    pub fn ceil(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp>;
    pub fn round(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp>;
    pub fn offset(self, t: Timestamp, steps: i64, tz: &TimeZone) -> Option<Timestamp>;
    /// Base units between floor(start) and floor(end), ignoring the step (d3's count).
    /// An extension: d3's filtered (`every`) intervals have no count.
    pub fn count(self, start: Timestamp, end: Timestamp, tz: &TimeZone) -> Option<i64>;
    /// d3's every: None for 0 (d3's null), otherwise the same unit with step k.
    pub fn every(self, k: u32) -> Option<Self>;
    /// Boundaries in [ceil(start), stop), lazily (d3's range).
    pub fn range(self, start: Timestamp, stop: Timestamp, tz: &TimeZone) -> interval::Range;
    /// d3's range(start, stop, step): every `step`-th boundary from ceil(start).
    pub fn range_step(self, start: Timestamp, stop: Timestamp, step: u32, tz: &TimeZone)
        -> interval::Range;
}
/// Iterator<Item = Timestamp>; owns a clone of the zone (jiff's TimeZone is an Arc).
pub struct Range { /* interval, next, stop, step, zone */ }
```

`None` means the result left jiff's range (years −9999 to 9999), where d3 returns an Invalid Date; a `Range` ends there.

Semantics unit by unit (d3 sources cited):

| Unit | floor | offset(n) | `every(k)` keeps / floors |
| --- | --- | --- | --- |
| Nanosecond, Microsecond, Millisecond | i128 ns floored to a multiple of `k` units since the epoch (`millisecond.js:12–23`) | `+ n·k` units, absolute | built into floor |
| Second | absolute, `t − (t mod 1 s)` (offsets are whole seconds) | absolute | UTC second-of-minute % k == 0 (`second.js:11`) |
| Minute, Hour | wall clock: subtract the local minute, second and subsecond parts using the offset **at t** (`hour.js:4–6`), one zone lookup | absolute `+ n` min or h | local minute / hour % k == 0 |
| Day | `Zoned::start_of_day` (the first instant of the day; São Paulo's skipped midnight gives 01:00) | calendar days, then floor | (day of month − 1) % k == 0 |
| UnixDay | as Day | as Day | civil days since 1970-01-01 % k == 0 |
| Week(w, _) | back to weekday `w`, start of day | calendar `7n` days | weeks from `Week(w).floor(epoch in tz)` % k == 0 (d3's `count(0, d)`) |
| Month | first of the month, start of day | calendar months | (month − 1) % k == 0 |
| Year | January 1, start of day; `every(k)`: the year floored to a multiple of k (`year.js:15–26`) | calendar years (× k for every) | built into floor |

Filtered intervals (`every` with a field test) follow `interval.js:37–68`: floor = the base floor, then, while the test fails, step back 1 ns and floor again; `offset(n)` repeats |n| times "offset by one base unit until the test passes". `ceil(t) = floor(offset(floor(t − 1 ns), 1))`. `round` picks the floor when `t − floor < ceil − t`, else the ceil (ties go to the ceil, as d3). `range` takes ceil(start), then pushes, offsets by `step` and floors while `previous < next < stop`; it is empty when `!(start < stop)`. `count`: Nanosecond to Hour divide the absolute difference of the floors; Day and Week use civil-day differences, so DST does not leak in (`day.js:6`); Month and Year use calendar differences. Zone work happens only for Minute and coarser; Second and finer are pure i128 arithmetic. On `Zoned`, jiff treats spans of hours or less as absolute and days or more as calendar units, which is the split the table uses.

The prototype `proto/rfc/d3/src/time.rs` supplies the enum, `floor` through `Zoned` and the shape of the tick table; this phase takes `Timestamp` plus `&TimeZone` instead of `Zoned` values, i128 arithmetic instead of `as_millisecond() as f64`, d3's `every` semantics (the prototype's `keep` is wrong for weeks and years) and no `expect` on jiff results.

##### Free tick functions (`time/ticks.rs`, 27a)

d3's `timeTicks`, `timeTickInterval`, `utcTicks` and `utcTickInterval` (`d3-time/src/ticks.js:13–56`):

```rust
pub fn tick_interval(start: Timestamp, stop: Timestamp, count: usize, zone: &TimeZone) -> Interval;
pub fn ticks(start: Timestamp, stop: Timestamp, count: usize, zone: &TimeZone) -> Vec<Timestamp>;
pub fn ticks_with(start: Timestamp, stop: Timestamp, interval: Interval, zone: &TimeZone)
    -> Vec<Timestamp>;
pub fn utc_tick_interval(start: Timestamp, stop: Timestamp, count: usize) -> Interval;
pub fn utc_ticks(start: Timestamp, stop: Timestamp, count: usize) -> Vec<Timestamp>;
```

`tick_interval` works on the span `s = |stop − start|` in i128 ns. With `target = s / count`, it bisects d3's 18-entry table (1, 5, 15, 30 s; 1, 5, 15, 30 min; 1, 3, 6, 12 h; 1, 2 days; 1 week; 1, 3 months; 1 year, with a month of 30 days and a year of 365) to the right and takes the nearer of the two neighbours by ratio (`target / d[i−1] < d[i] / target ? i − 1 : i`). Beyond the table: `Year(max(1, tick_step(0, years, count)))` with `years = s / 365 days`. Below it, d3 returns `Millisecond(max(tick_step, 1))`; this port extends that to nanoseconds: `step_ns = tick_step(0, s, count)`; ≥ 10⁶ gives `Millisecond(step / 10⁶)`, ≥ 10³ gives `Microsecond(step / 10³)`, else `Nanosecond(max(step, 1))`. The table's day and week entries are `Day(k)` and `Week(Sunday, 1)` for the zoned functions and `UnixDay(k)` and `Week(Sunday, 1)` for the `utc_*` pair (`ticks.js:55–56`), whose zone is UTC.

`ticks` orders the two ends, takes `interval.range(start, stop + 1 ns)` (an inclusive stop, as d3's `+stop + 1`), reverses for a descending input and returns at most `min(10·count, 10_000)` values; `ticks_with` caps at 10,000. Equal ends give one tick (the step falls to `Nanosecond(1)`), as d3 gives one.

##### `time::Time` (27b)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Time { domain: [Timestamp; 2], range: [f64; 2], zone: TimeZone,
                  clamp: bool, utc_ticks: bool }
/// d3.scaleTime with an explicit zone; domain 2000-01-01..02 in `zone`.
pub fn time(zone: TimeZone) -> Time;
/// d3.scaleUtc: zone UTC and d3's utc tick intervals (UnixDay).
pub fn utc() -> Time;
impl Time {
    pub fn domain(self, d: [Timestamp; 2]) -> Self;
    pub fn get_domain(&self) -> [Timestamp; 2];
    pub fn range(self, r: [f64; 2]) -> Self;
    pub fn get_range(&self) -> [f64; 2];
    pub fn clamp(self, clamp: bool) -> Self;
    pub fn get_zone(&self) -> &TimeZone;
    pub fn apply(&self, t: Timestamp) -> f64;
    pub fn invert(&self, y: f64) -> Timestamp;       // saturates at jiff's range
    pub fn tick_interval(&self, count: usize) -> Interval;
    pub fn ticks(&self, count: usize) -> Vec<Timestamp>;
    pub fn ticks_with(&self, interval: Interval) -> Vec<Timestamp>;
    pub fn nice(self, count: usize) -> Self;
    pub fn nice_with(self, interval: Interval) -> Self;
}
```

`time::time` and `time::utc` are functions in `core/src/time.rs`; `time::Time` is the type's path. Arithmetic: with `s = d1 − d0` and `u = t − d0` as i128 ns, `apply = r0 + (r1 − r0)·(u as f64 / s as f64)`; a flat domain gives `r0 + 0.5·(r1 − r0)`, as d3's `normalize`; when clamping, the fraction is clamped to [0, 1] first. `invert(y) = d0 + round(((y − r0)/(r1 − r0))·s as f64)` in i128, clamping the fraction when clamping. Differences, never absolute epoch values, pass through `f64`, so 2026 instants keep their nanoseconds.

`tick_interval`, `ticks` and `ticks_with` delegate to the free functions with the scale's zone, or to the `utc_*` pair when `utc_ticks` is set. `nice_with(i)` sets the domain to `[i.floor(d0), i.ceil(d1)]`, respecting a reversed domain; `nice(count)` uses `tick_interval(count)`.

##### Gestures and padding (`time/scale/{rescale,pad}.rs`, 27b)

Phase 10's traits, in time's own precision; every offset below is an i128 nanosecond count from `d0`, and only differences pass through `f64`:

```rust
impl interact::Rescale for Time {
    fn rescale_x(&self, t: &interact::Transform) -> Self;  // domain = range.map(|r| invert(t.invert_x(r)))
    fn rescale_y(&self, t: &interact::Transform) -> Self;  // with t.invert_y
    fn pan_px(&self, dx: f64) -> Option<Self>;
    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self>;
}
impl scale::Pad for Time {
    type Domain = Timestamp;
    fn pad(&self, extent: Option<[Timestamp; 2]>, margin: f64, sticky: &[Timestamp])
        -> [Timestamp; 2];
}
```

- `rescale_x` and `rescale_y` use `invert` exactly as d3 (a clamping scale cannot zoom out, as in d3 and Phase 10).
- `pan_px(dx)`: the value at range position r moves to r + dx, so both ends shift by `−round(dx / (r1 − r0) · s)` ns; `rescale_x(&Transform { k: 1, x: 80, y: 0 })` and `pan_px(80.0)` agree (the domain moves earlier). No clamping.
- `zoom_px(anchor, factor)`: `a = invert_unclamped(anchor)` as an offset; the new ends are `a + round((dᵢ − a) as f64 / factor)`.
- `None` when `dx`, `anchor` or `factor` is not finite, `factor ≤ 0`, the new span is under 1 ns, the orientation flips, or an end leaves jiff's range.
- `pad` follows Phase 10's matplotlib algorithm with time's own nonsingular rule, matplotlib 3.11.2's `DateLocator.nonsingular`: no data gives [1970-01-01T00:00Z, 1970-01-02T00:00Z]; a flat extent (equal ends) widens by ±730 days (2 × 365, matplotlib's `DAYS_PER_YEAR = 365.0`); then the margin `round(margin · span)` per side, and sticky bounds with a tolerance of `1e−5 · span` in ns. matplotlib widens anything under 10⁻⁶ days; that would flatten every recording shorter than 0.1 s, so only equal ends count as flat here. Measured in matplotlib 3.11.2: one point at 2026-01-01T12:00 gives [2023-10-21T12:00, 2028-03-14T12:00] (±2 years, then 5%), and 2026-01-01..11 gives [2025-12-31T12:00, 2026-01-11T12:00].

`proto/rfc/d3/src/interact.rs` has the shape of `Rescale for Time`.

##### `Recording<Timestamp>` (`series/clock/timestamp.rs`, 27b)

```rust
impl series::Coordinate for Timestamp {
    type Period = jiff::SignedDuration;
    /// start + index · period in i128 ns; saturates at Timestamp::MAX.
    fn at(start: Self, period: Self::Period, index: u64) -> Self;
    /// ceil((value − start) / period) in i128, clamped to [0, u64::MAX].
    fn index(start: Self, period: Self::Period, value: Self) -> u64;
    fn is_valid(self) -> bool { true }                       // every Timestamp is
    fn is_valid_period(period: Self::Period) -> bool;        // > 0 (≥ 1 ns)
}
```

No accumulation (each sample is computed from `start`) and no `f64`. `index` returns the exact first index whose `at` is ≥ `value`, which satisfies Phase 9's contract (an estimate the recording adjusts by ±1 against `at`). Column edges for M4 come from `Time::invert` at device-pixel edges and go to `Recording::columns(&[Timestamp])`, so the boundary math stays in the scale's precision. Phase 9's generic tests are instantiated for `Timestamp`: a regular 1 kHz clock over 1M samples from 2026-10-25T00:00:00Z and an explicit clock with the same instants; its `drop_front(150_000)` row (returns 131,072) holds unchanged.

##### The harness under Los Angeles

d3's tests run with `TZ=America/Los_Angeles` (`d3-time/package.json:47`). The translator emits `let la = zone("America/Los_Angeles");` per test and maps `local(y, m, d, h, mi, s, ms)` to `prelude::local(&la, …)` and `utc(…)` to `prelude::utc(…)`. Both port ECMAScript's `MakeDay`/`MakeTime`/`MakeDate`: missing trailing arguments default to `m = 0, d = 1, h = mi = s = ms = 0`; `ym = y + floor(m / 12)`, `mn = m.rem_euclid(12)`; `day = civil::date(ym, mn + 1, 1) + (d − 1)` days for any integer `d`, negative included; then `h·3_600_000 + mi·60_000 + s·1_000 + ms` milliseconds as one offset added to the civil date-time, so hours ≥ 24 and negative fields carry; years 0–99 are literal (`date.js:9–13`). The civil result resolves in the zone with jiff's `compatible` disambiguation (`tz.to_ambiguous_zoned(dt).compatible()`), which is what `Date` does. Do the arithmetic in `civil::DateTime` and convert once; do not loop over jiff `Span`s.

`new Date(ms)` in a test becomes `Timestamp::from_millisecond(ms)` (used in `hour-test.js`, `minute-test.js` and `second-test.js` around the 2016-11-06 fall-back); `new Date` without arguments and `Date.now` are skips ("ambient clock"). `timeHour` becomes `Interval::Hour(1)` with `&la`; `utcHour` the same with `&TimeZone::UTC`; `unixDay` is `UnixDay(1)` with UTC; `timeWeek` and `timeSunday` … `timeSaturday` are `Week(Weekday::…, 1)`; calling an interval as a function, `interval(d)`, is `floor`. Timestamps reach `check` through the prelude function `ts(&Option<Timestamp>) -> Obs` (and a `Vec` variant) as `Obs::Str` of `format!("{t:.9}")` (RFC 3339 with nine fraction digits) — never `Obs::Num` of epoch milliseconds, whose relative tolerance (10⁻⁹ × 1.3·10¹² ≈ 1.3 s) would hide errors. The conversion lives in the harness prelude. `+date` and `date.getTime()` become `as_millisecond()` as an exact `f64`; `count` results compare as numbers; `None` is `Obs::Null`.

##### Translator rules (Step 0, budget ≤ 250 lines of JS)

`local`/`utc` calls; `new Date(ms)`; `timeX`/`utcX` identifiers as `(Interval, &zone)` pairs; `.floor/.ceil/.round/.offset/.count/.range/.every(k)` chains; `+date` and `date.getTime()` as `as_millisecond()`; `Obs` through `ts`; `timeTicks(a, b, n)` → `time::ticks(a, b, n, &la)`, `timeTicks(a, b, timeMinute)` → `time::ticks_with(a, b, Interval::Minute(1), &la)`, `utcTicks` → `time::utc_ticks`, `timeTickInterval` → `time::tick_interval`; in 27b, `scaleTime()` and `scaleUtc()` chains. Anything else is a skip with its construct named. Commit before Step 1.

##### Decisions this phase may meet

- jiff's `compatible` disambiguation and V8 disagree on a test date — keep jiff's answer and make that assertion a skip with the date named; record it as a decision.

#### Steps

**27a: intervals and free tick functions**

0. **Translator rules**, as above.
1. **Fine intervals** (`time.rs`, `time/interval.rs`, `interval/{fine,range}.rs`): Nanosecond to Second, `every`, the lazy `Range`, `count`; prelude `local`/`utc` and `ts`. Bless the millisecond, second, utcMillisecond and utcSecond tests.
2. **Wall-clock intervals** (`interval/clock.rs`): Minute, Hour. Bless minute, hour, utcMinute, utcHour.
3. **Calendar intervals** (`interval/calendar.rs`): Day, UnixDay, Week, Month, Year and their `every`. Bless the remaining interval files.
4. **Zones by hand**: `time_zones_script_cases`, `sub_millisecond_intervals_step_exactly`, `interval_range_is_lazy`.
5. **Free tick functions** (`time/ticks.rs`): the table, the nearer-of-two rule, the sub-millisecond extension, the caps. Bless `ticks-test.js` and `utcTicks-test.js`. `check.sh full`. *Cut point: 27b starts at Step 6 and needs Steps 0–5 merged.*

**27b: scale, gestures, padding, recordings**

6. **`Time` core** (`time/scale.rs`): apply, invert, clamp, flat and reversed domains. Bless the basics of `time-test.js` and `utcTime-test.js`.
7. **Ticks and nice** (`time/scale/ticks.rs`): `tick_interval`, `ticks`, `ticks_with` delegating to the free functions, `nice`, `nice_with`. Bless.
8. **Rescale and Pad** (`time/scale/{rescale,pad}.rs`) and the precision test.
9. **`Recording<Timestamp>`** (`series/clock/timestamp.rs`), Phase 9's suite instantiated.
10. **Fuzz and finish**: tick fuzzing under a timeout, doc examples, `check.sh full`.

#### Exit criteria

**27a**

1. `scripts/check.sh quick` is green (tests run under `TZ=Asia/Kathmandu`; nothing may depend on it).
2. `cargo run -p rustd3-harness -- --check` is green, and each of the 35 translated d3-time test files passes ≥ 70% of its assertions, `ticks-test.js` (27) and `utcTicks-test.js` (26) included. Listed skips: `interval-test.js` (32 assertions: custom `timeInterval` and `.filter`; the enum is closed) is not translated ("custom intervals"); JS coercion rows (`every("1")`, `every(NaN)`, `range` with non-date arguments) are skips by construct; "ambient clock" rows as above.
3. Test `time_zones_script_cases` ports all 32 active cases of `d3-time/test/test-time-zones` (Kolkata, Kathmandu, São Paulo) with the script's millisecond values.
4. Test `sub_millisecond_intervals_step_exactly`: `Nanosecond(1)` and `Microsecond(1)` ranges over a 2026 instant are exact.
5. Test `interval_range_is_lazy`: `Nanosecond(1).range` over a day yields its first 10 items without allocating the rest.

**27b**

6. `time-test.js` (96) and `utcTime-test.js` (85) pass ≥ 60% each; the rest are their `tickFormat` tests (Phase 28, which raises both floors to 70%) and listed skips (polylinear time domains, `domain([NaN…])`, `.interpolate`, `.copy()` identity).
7. Test `berlin_ticks` (Part I's Berlin example without labels) and the rescale rows of "Spot checks".
8. Precision test `timestamps_2026_over_one_microsecond`: apply, invert, `pan_px`, `zoom_px` and `pad` on a 1 µs domain in 2026 are exact to the nanosecond. The test also reconstructs each probe instant (both domain ends and the `zoom_px`/`pan_px` results) through `((t.as_nanosecond() as f64) as i128)` and asserts that for at least one of them the round trip differs from `t.as_nanosecond()` by at least 1 ns, while the i128 path is exact.
9. `Recording<Timestamp>` tests: regular and explicit clocks agree; M4 over a `Time` scale matches a full scan; `drop_front` and split appends equal rebuilds.
10. Degenerate inputs: `zoom_px` returns `None` for factors leaving a span under 1 ns, for flipped results and for non-finite inputs; tick generation caps at `min(10 × count, 10_000)` ticks and is fuzzed under a 1 s timeout per case. `scripts/check.sh full` is green.

A one-off `TZ=UTC cargo test -p rustd3-core time::` by hand confirms zone independence.

#### Spot checks

d3 7.9.0 (d3-time 3.1.0, d3-scale 4.0.2) in Node with `TZ` as noted, and `test-time-zones`; rows marked "ours" have no d3 counterpart. "local" means the zone of the row.

**27a: intervals and free tick functions**

| Input | Expected |
| --- | --- |
| Kolkata: `Hour(1).floor(1456279843199 ms)` / `.ceil` | 1456277400000 / 1456281000000 |
| Kathmandu: `Hour(1).floor(1456279843199 ms)` / `.ceil` | 1456276500000 / 1456280100000 |
| Kathmandu: `Hour(1).range(local 2011-11-07 00:00, 03:00)` | 1320603300000, 1320606900000, 1320610500000 |
| São Paulo: `Day(1).floor(2016-10-16T03:00Z)` (midnight skipped) | 1476586800000 (01:00 local) |
| São Paulo: `Day(1).ceil(2016-10-16T04:00Z)` | 1476669600000 |
| São Paulo: `Day(1).range(local 2016-10-14, 2016-10-18)` | 1476414000000, 1476500400000, 1476586800000, 1476669600000 |
| LA: `Hour(1).floor(2011-03-13T09:30Z)` | 2011-03-13T09:00Z |
| LA: `Day(1).floor(2011-03-13T12:00Z)` | 2011-03-13T08:00Z |
| LA: `Day(1).offset(2011-03-12T08:00Z, 1)` (DST starts) | 2011-03-13T08:00Z |
| LA: `Hour(1).round(2011-01-01T08:30Z)` (tie) | 2011-01-01T09:00Z |
| LA: `Day(1).count(local 2011-01-01, local 2011-03-14)` | 72 |
| LA: `Month(1).count(local 2011-01-15, local 2012-03-02)` | 14 |
| LA: `Week(Sunday, 1).count(local 2011-01-01, local 2011-12-31)` | 52 |
| LA: `Year(5).range(local 2006, local 2026)` years | 2010, 2015, 2020, 2025 |
| UTC: `Day(2).range(2011-01-01, 2011-01-09)` / `UnixDay(2)` | 01, 03, 05, 07 / 02, 04, 06, 08 |
| Berlin: `Hour(3).range(2026-10-24T22:00Z, 2026-10-25T06:00Z)` | 22:00Z, 02:00Z, 05:00Z (01:00Z is 02:00 CET) |
| Berlin: `Week(Sunday, 2).range(local 2026-01-01, local 2026-03-01)` | 2026-01-10T23:00Z, 01-24T23:00Z, 02-07T23:00Z, 02-21T23:00Z |
| `Millisecond(50).range(1000 ms, 1200 ms)` | 1000, 1050, 1100, 1150 ms |
| `Nanosecond(1).range(2026-10-25T12:00:00Z, +3 ns)` (ours) | +0, +1, +2 ns |
| harness `local(&la, 2011, 0, 0)` | 2010-12-31 local midnight (d = 0 carries back) |
| harness `local(&la, 2011, 12, 1)` | 2012-01-01 local midnight (m = 12 carries) |
| harness `local(&la, 2011, 0, 1, 24)` | 2011-01-02 local midnight (h = 24 carries) |
| harness `local(&la, 2011, -1, 1)` | 2010-12-01 local midnight |
| LA: `time::ticks_with(local 2011-01-01 12:01, local 2011-01-01 12:04:04, Minute(1), &la)` (`ticks-test.js:5–11`) | 12:01, 12:02, 12:03, 12:04 local |

**27b: scales, gestures, padding**

| Input | Expected |
| --- | --- |
| Berlin `time(berlin).domain([2026-10-24T22:00Z, 2026-10-25T23:00Z]).range([0, 800])`: `tick_interval(8)` (Part I) | `Hour(3)` |
| that scale `.ticks(8)` | 22:00Z, then 02:00Z … 20:00Z every 3 h, then 23:00Z (9 ticks; local 00:00, 03:00 … 21:00, 00:00) |
| that scale `.apply(2026-10-25T12:00Z)` / `.invert(400)` | 448 / 2026-10-25T10:30:00Z |
| that scale `.rescale_x(&Transform { k: 1, x: 80, y: 0 })` | [2026-10-24T19:30Z, 2026-10-25T20:30Z] |
| that scale `.rescale_x(&Transform { k: 2, x: −400, y: 0 })` | [2026-10-25T04:15Z, 2026-10-25T16:45Z] |
| LA `time(la)`, domain local 2011-01-01..09, `.ticks(4)` | 01-01, 01-03, 01-05, 01-07, 01-09 at 08:00Z |
| `utc()`, same span, `.ticks(4)` | 01-02, 01-04, 01-06, 01-08 at 00:00Z |
| LA, domain local 2009-01-01 00:12..23:48, `.nice(10)` | [2009-01-01T08:00Z, 2009-01-02T08:00Z] |
| LA, domain local 2001..2138, `.nice(10)` years | 2000, 2140 |
| LA, domain local 2000..2100, `.ticks(5)` years | 2000, 2020, 2040, 2060, 2080, 2100 |
| LA, empty domain at local 00:12, `.ticks(10)` | one tick, 2009-01-01T08:12Z |
| LA, local 2009..2010, range [0, 100], `.clamp(true).invert(−50)` | 2009-01-01T08:00Z |
| that scale `.apply(local 2009-07-01)` | 49.577625570776256 |
| local 12:00:00.000..12:00:01.000, `.ticks(4)` | ms 0, 200, 400, 600, 800, then 0 of the next second |
| `[t0, t0 + 10 µs].ticks(5)`, t0 = 2026-10-25T12:00Z (ours) | `Microsecond(2)`: t0 + 0, 2, 4, 6, 8, 10 µs |
| `[t0, t0 + 100 ns].ticks(10)` (ours) | `Nanosecond(10)`: 11 ticks |
| `[t0, t0 + 1 µs]`, range [0, 1000]: `apply(t0 + 1 ns)` / `invert(500)` | 1.0 / t0 + 500 ns |
| same, `zoom_px(500.0, 2.0)` | [t0 + 250 ns, t0 + 750 ns] |
| same, `zoom_px(500.0, 2000.0)` | `None` (a span of 0.5 ns < 1 ns) |
| same, `pan_px(100.0)` | [t0 − 100 ns, t0 + 900 ns] |
| `pad(None, 0.05, &[])` | [1970-01-01T00:00Z, 1970-01-02T00:00Z] |
| `pad(Some([2026-01-01T12:00Z; 2]), 0.05, &[])` (matplotlib 3.11.2) | [2023-10-21T12:00Z, 2028-03-14T12:00Z] |
| `pad(Some([2026-01-01, 2026-01-11]), 0.05, &[])` (matplotlib 3.11.2) | [2025-12-31T12:00Z, 2026-01-11T12:00Z] |

#### Not in this phase

- Tick formats, `tick_label`, the ISO ladder, `Ticks for Time`, time locales (Phase 28).
- Figure time handles and fitting merges (Phase 33).
- Polylinear time domains, `rangeRound`, `.interpolate`, `.unknown`: listed skips.
- `TimeZone::system()` or any "local" default: never; library code reads no ambient zone or clock.
- Custom intervals (`timeInterval(floor, offset)`, `.filter`): the enum is closed; their tests are listed skips.

### Phase 28 — Time II: formats, locales and time ticks

*M3 Core breadth · builds on phases 26, 27 · about 1,200 lines of Rust plus ~400 generated, a 100-line generator and ≤ 200 lines of translator JS, 7 commits · two sessions: cut after Step 2*

#### Goal

Core gains d3-time-format's formatting over jiff — `time::Format` for every d3 directive including `%L %f %Q %s %Z`, plus `%N` for nanoseconds — with `time::Locale` (d3's `timeFormatLocale` plus the tick-label ladder, with a public `time::Locale::new(time::Definition)`), its constants `NEUTRAL`, `EN_US` and ten generated tables, `time::Ladder { Iso, D3 }`, `time::iso_format` and `time::iso_parse`; `Time::tick_label` and `Ticks for time::Time` with `Step = time::Interval`, labelling each tick by the ladder rung it needs; and the time half of `locale::Locale` with the combined constants for the ten languages. Part I's Berlin example runs end to end. d3-time-format's format, utcFormat, isoFormat, isoParse and locale tests (283 assertions) and the `tickFormat` tests of d3-scale's time and utcTime files grade it.

#### Builds on

- Phase 6: `locale::Locale` (private `number` field, `NEUTRAL`, `EN_US`, `Default`, `number()`), `scale::Ticks` (`tick_set`, `position(&v, offset)`, `labels(&set, &Locale)`, `minor`, `extent`), `ticks::Set::new(values, step, count)`.
- Phase 26: `format::Locale::{EN_US, DE_DE, FR_FR, ES_ES, IT_IT, PT_BR, NL_NL, PL_PL, RU_RU, JA_JP, ZH_CN}`, `NEUTRAL`/`Default`, and the generator-freshness step in `scripts/check.sh full`.
- Phase 27: `time::Interval` (floor, ceil, offset, count; `Week(Sunday | Monday | Thursday)`, `Year`), `time::Time` (`ticks`, `tick_interval`, `apply`, `get_zone`), the free tick functions, and the harness's Los Angeles prelude (`local`, `utc`, `ts`).
- Decision files `decisions/phase-06.md`, `phase-26.md`, `phase-27.md`.

#### Design

Module layout:

```text
core/src/time.rs                      re-exports Locale, Definition, Ladder, Format, format,
                                      iso_format, iso_parse
core/src/time/locale.rs               Locale, crate-private Table and from_table
core/src/time/locale/ladder.rs        Ladder and its crate-private rung tables
core/src/time/definition.rs           Definition
core/src/time/format.rs               Format, format()
core/src/time/format/{item,directive,pad,iso}.rs
core/src/time/locales.rs              mod lines and a doc table
core/src/time/locales/{en_us,de_de,fr_fr,es_es,it_it,pt_br,nl_nl,pl_pl,ru_ru,ja_jp,zh_cn}.rs  generated
core/src/time/scale/labels.rs         Time::tick_label, Ticks for Time
core/src/locale.rs                    the time half and the combined constants
```

##### d3's directive table over jiff's calendar

jiff's `strftime` differs from d3 beyond the four directives that obviously clash (`%L`, `%f`, `%Q`, `%Z`). Measured with jiff 0.2.38: `%y` of year −2 prints `-2` (d3 `-02`), `%Y` prints `-002` (d3 `-0002`), `%Z` is an abbreviation (`PST`; d3 `-0800`), and `%f` is a variable fraction. So the core ports d3's format table (`d3-time-format/src/locale.js:57–126`, helpers `393–697`) and jiff supplies zone conversion and civil fields, `to_zoned` once per formatted value. Delegating individual directives to jiff stays possible later and is not needed.

##### Types

```rust
/// d3's timeFormatLocale plus the tick-label ladder.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    date_time: Cow<'static, str>, date: Cow<'static, str>, time: Cow<'static, str>,
    periods: [Cow<'static, str>; 2],
    days: [Cow<'static, str>; 7], short_days: [Cow<'static, str>; 7],
    months: [Cow<'static, str>; 12], short_months: [Cow<'static, str>; 12],
    ladder: Ladder,
}
impl Locale {
    /// English names, ISO composites (`%c %x %X` as `%F %T`, `%F`, `%T`), 24-hour clock, ISO ladder.
    pub const NEUTRAL: Self;
    /// d3's default definition and d3's ladder.
    pub const EN_US: Self;
    // generated: DE_DE FR_FR ES_ES IT_IT PT_BR NL_NL PL_PL RU_RU JA_JP ZH_CN
    pub fn new(definition: Definition) -> Self;
    pub fn format(&self, spec: &str) -> Format;
}
impl Default for Locale { /* NEUTRAL */ }

/// The fields of d3's timeFormatLocale definition, plus the ladder.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub date_time: Cow<'static, str>, pub date: Cow<'static, str>, pub time: Cow<'static, str>,
    pub periods: [Cow<'static, str>; 2],
    pub days: [Cow<'static, str>; 7], pub short_days: [Cow<'static, str>; 7],
    pub months: [Cow<'static, str>; 12], pub short_months: [Cow<'static, str>; 12],
    pub ladder: Ladder,
}

/// Which tick-label ladder a locale uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ladder { #[default] Iso, D3 }

/// A compiled d3 time specifier with its locale.
#[derive(Debug, Clone, PartialEq)]
pub struct Format { items: Vec<format::Item>, locale: Locale }
impl Format { pub fn format(&self, t: Timestamp, zone: &TimeZone) -> String; }

pub fn format(spec: &str) -> Format;                            // NEUTRAL
pub fn iso_format(t: Timestamp) -> String;                      // %Y-%m-%dT%H:%M:%S.%LZ in UTC
pub fn iso_parse(s: &str) -> Result<Timestamp, jiff::Error>;    // jiff's RFC 3339 parser
```

`time::Locale` is `Clone` but not `Copy`, like `format::Locale` (Phase 3): its constants are `const` through a crate-private `const fn from_table(Table) -> Locale` over a `Copy` struct of `&'static str` fields plus the ladder, as Phase 26 did for numbers, and `Locale::new` builds one at run time. The ten generated tables and `NEUTRAL` use `Ladder::Iso`; `EN_US` uses `Ladder::D3`, so d3's `tickFormat` tests pass and d3's en-US is a named locale like any other. A ladder parameter on the scale was the alternative; it would put a second knob beside `fig.locale`, so the ladder belongs to the locale. The other 24 of d3-time-format's 35 tables are not bundled; they load through `Locale::new`.

`format::Item` (crate-private, `time/format/item.rs`) is literal text or a directive with its pad: `-` none, `_` space, `0` zero; the default is `0`, and `e` defaults to space (`newFormat`, `locale.js:182–184`). Unknown directives print the character without `%`, as d3. Malformed input is therefore not an error for formatting; `iso_parse` returns jiff's error.

**Directives** (d3's table, `locale.js:57–91` and the utc table `92–126`): `a A b B c d e f g G H I j L m M p q Q s S u U V w W x X y Y Z %`, plus `N`. `%c %x %X` expand the locale's patterns recursively. `%L` = floor(subsec / 10⁶) in 3 digits, `%f` = floor(subsec / 10³) in 6 digits, `%N` = the subsecond nanoseconds in 9 digits (jiff's meaning; d3 has no `%N`), `%Q` = epoch milliseconds floored, `%s` = epoch seconds floored, `%Z` = `±HHMM` of the offset at t, truncated to minutes. `%U %W %V %g %G %u %w %j` port d3's formulas on Phase 27's intervals (`Week(Sunday | Monday | Thursday)`, `Year`, `count`), not jiff's ISO week API, so d3's edge cases (`%V` is 53 on 2016-01-01) match by construction. `%y` = `year mod 100`, zero-padded, with its sign; `%Y` = the year zero-padded to 4 digits with its sign (d3 also reduces mod 10000; jiff's years lie in −9999..=9999, so the reduction never applies, and d3's two assertions on year 10002 are skips).

##### Ladders (`time/locale/ladder.rs`)

```rust
/// Rungs, coarsest first: year, month, week, day, hour, minute, second,
/// millisecond, microsecond, nanosecond.
const ISO: [&str; 10] = ["%Y", "%Y-%m", "%m-%d", "%m-%d", "%H:%M",
                         "%H:%M", "%H:%M:%S", "%S.%L", "%S.%f", "%S.%N"];
const D3: [&str; 10] = ["%Y", "%B", "%b %d", "%a %d", "%I %p",
                        "%I:%M", ":%S", ".%L", ".%f", ".%N"];
impl Ladder { pub(crate) fn rungs(self) -> &'static [&'static str; 10]; }
```

`Ladder::Iso` is Part I's neutral ladder (`%Y, %Y-%m, %m-%d, %H:%M, %H:%M:%S, %S.%L`, extended below the millisecond); `Ladder::D3` is d3-scale's (`time.js:15–36`).

**The rung is chosen per tick**, as d3's `multi` (`time.js:29–36`), in the scale's zone: `second.floor(t) < t` → a sub-second rung; else `minute.floor(t) < t` → second; else `hour` → minute; else `day` → hour; else `month.floor(t) < t` → (`week.floor(t) < t` ? day : week); else `year` → month; else year. The sub-second rung comes from that tick's own fraction: nanosecond if it has a non-zero part below the microsecond, else microsecond if a non-zero part below the millisecond, else millisecond. The set's step is not consulted, so a `Microsecond(2)` set labels a tick that falls on an exact millisecond on the millisecond rung. Berlin under `Locale::default()`: 22:00Z is local 00:00 on 25 October, a Sunday, so the week rung `%m-%d` gives "10-25"; 02:00Z to 20:00Z are local 03:00 to 21:00, on the hour rung `%H:%M`; 23:00Z is local 00:00 on 26 October, the day rung, "10-26".

##### Time ticks (`time/scale/labels.rs`) and `Locale`'s time half

```rust
impl Time {
    /// d3's multi-scale tick format through `locale`'s ladder, in the scale's zone.
    pub fn tick_label(&self, t: Timestamp, locale: &time::Locale) -> String;
}
impl Ticks for Time {
    type Domain = Timestamp;
    type Step = time::Interval;
    fn tick_set(&self, count: usize) -> ticks::Set<Timestamp, Interval>; // ticks(count), tick_interval(count), count
    fn position(&self, v: &Timestamp, _offset: f64) -> Option<f64>;     // Some(apply(v))
    fn labels(&self, set: &ticks::Set<Timestamp, Interval>, locale: &locale::Locale) -> Vec<String>;
    fn extent(&self) -> [f64; 2];                                         // get_range()
    // minor: the default (none)
}
```

`labels` applies `tick_label` with `locale.time()` to each tick independently. `locale::Locale` (`core/src/locale.rs`) gains the private field `time: time::Locale` and `pub fn time(&self) -> &time::Locale`; its fields are private, so this is an addition. `NEUTRAL` pairs the neutral halves, `EN_US` the d3 halves, and the new constants `DE_DE`, `FR_FR`, `ES_ES`, `IT_IT`, `PT_BR`, `NL_NL`, `PL_PL`, `RU_RU`, `JA_JP`, `ZH_CN` pair Phase 26's number tables with the generated time tables, so `fig.locale(Locale::DE_DE)` sets both.

##### Generator (`dev/harness/gen/time_locales.mjs`)

It reads the eleven JSON files under `dev/harness/d3/d3-time-format/locale/` and writes `core/src/time/locales/{en_us,de_de,…,zh_cn}.rs` (one `impl time::Locale { pub const XX_YY: Self = Self::from_table(Table { … }); }` each, non-ASCII as `\u{…}`, `Ladder::Iso` except en-US) and `dev/fixtures/time_locales.json` (d3's `timeFormatLocale(json).format` of `%c`, `%x`, `%X` and `%a %A %b %B %p` on four probe instants in the LA zone). It runs as `TZ=America/Los_Angeles node --import dev/harness/translate/d3-resolve.mjs dev/harness/gen/time_locales.mjs`, and Step 3 appends that command, with `git diff --exit-code core/src/time/locales dev/fixtures/time_locales.json`, to the freshness step in `scripts/check.sh full`. `proto/tloc/src/lib.rs` (128 lines) shows the table shape and the rewrite of `%a %A %b %B %p %c %x %X`; its test values are spot checks.

##### Translator rules (Step 0, budget ≤ 200 lines of JS)

`timeFormat(spec)` → `time::Locale::EN_US.format(spec)` plus `.format(t, &la)`; `utcFormat` the same with `&TimeZone::UTC`; `isoFormat` → `time::iso_format`, `isoParse` → `time::iso_parse`; `scale.tickFormat()(d)` → `scale.tick_label(d, &time::Locale::EN_US)`; `tickFormat(count, spec)` → `time::Locale::EN_US.format(spec)`; `timeFormatLocale(json)` with a literal or vendored-file argument → `time::Locale::new(Definition { … })`, the file read at translation time. Anything else is a skip with its construct named. Commit before Step 1.

##### Decisions this phase may meet

- A d3 format test whose expected output depends on how JS `Date` resolves a gap or fold differently from jiff — keep jiff's answer and make the assertion a skip with the date named; record it as a decision.

#### Steps

0. **Translator rules**, as above.
1. **`time::Locale`, `Definition`, `Ladder` and `Format`** (`time/locale.rs`, `time/locale/ladder.rs`, `time/definition.rs`, `time/format.rs`, `time/format/{item,directive,pad}.rs`), with `NEUTRAL` and `EN_US` hand-written from d3's default definition. Bless the format and utcFormat ids after reading the diff.
2. **ISO helpers and `%N`** (`time/format/iso.rs`). Bless isoFormat and isoParse. *Cut point: a second session starts at Step 3.*
3. **Generator and ten tables** (`dev/harness/gen/time_locales.mjs`, `time/locales.rs`, `time/locales/*.rs`, `dev/fixtures/time_locales.json`), test `time_locales_match_d3`, the `locale-test.js` loop over the 35 vendored tables, and the freshness command in `scripts/check.sh full`. Move `EN_US` into its generated file if it equals the hand-written one.
4. **Ladders and `Time::tick_label`** (`time/scale/labels.rs`). Bless the `tickFormat` tests of `time-test.js` and `utcTime-test.js` and raise those files' floors to 70%.
5. **`Ticks for Time`, `locale::Locale`'s time half and the combined constants**; tests `berlin_example` and `sub_second_labels`.
6. **Docs and `check.sh full`.**

#### Exit criteria

1. `scripts/check.sh quick` is green.
2. `cargo run -p rustd3-harness -- --check` is green with floors of 70% on `format-test.js` (137), `utcFormat-test.js` (140), `isoFormat-test.js` (2), `isoParse-test.js` (3) and `locale-test.js` (1, translated as a loop over the 35 vendored tables building `time::Locale::new`), all under `time::Locale::EN_US` and America/Los_Angeles; and of 70% on d3-scale's `time-test.js` and `utcTime-test.js`. Listed skips: `defaultLocale-test.js` (13; it mutates d3's global default); `parse-test.js` (139) and `utcParse-test.js` (97), "parsing (Later)"; `format-test.js:270` and `utcFormat-test.js:285` (`local(10002, 0, 1)`), "jiff range"; the seven `multi()` assertions at `format-test.js:285–291`, whose file-level `formatMillisecond` constants and `function multi(d)` (`format-test.js:6–20`) the translator does not handle, "file-level helpers" (`tick_label` covers the same behaviour).
3. Rerunning `time_locales.mjs` reproduces `core/src/time/locales/*.rs` and the fixture byte for byte, and `scripts/check.sh full` checks it.
4. Test `berlin_example` (Part I): `tick_interval(8) == Hour(3)` and the labels `10-25 03:00 06:00 09:00 12:00 15:00 18:00 21:00 10-26` under `Locale::default()`.
5. Test `time_locales_match_d3`: every row of "Spot checks" marked (d3) from `dev/fixtures/time_locales.json`.
6. Test `sub_second_labels`: in a `Microsecond(k)` set, ticks with a non-zero part below the millisecond label with `%S.%f`; in a `Nanosecond(k)` set, ticks with a non-zero part below the microsecond label with `%S.%N`.
7. `scripts/check.sh full` is green at the last commit.

#### Spot checks

d3 7.9.0 (d3-time-format 4.1.0) in Node with `TZ=America/Los_Angeles` unless noted; `t` = local 2026-03-07 23:59:30.123 in Los Angeles (a Saturday), zone LA.

| Input | Expected |
| --- | --- |
| `EN_US.format("%c")` (d3) | 3/7/2026, 11:59:30 PM |
| `EN_US.format("%x %X")` (d3) | 3/7/2026 11:59:30 PM |
| `DE_DE.format("%c")` (d3; tloc) | Samstag, der  7. März 2026, 23:59:30 |
| `DE_DE.format("%a %d %b")` (d3; tloc) | Sa 07 Mrz |
| `FR_FR.format("%c")` (d3) | samedi  7 mars 2026 à 23:59:30 |
| `JA_JP.format("%c")` (d3) | 2026/03/07 土 23:59:30 |
| `RU_RU.format("%B %b")` / `PL_PL` (d3) | марта мар / Marzec Marz. |
| `ZH_CN.format("%x")` (d3) | 2026年3月7日 |
| `NEUTRAL.format("%c")` / `"%a %d %b"` (tloc; Part I) | 2026-03-07 23:59:30 / Sat 07 Mar |
| `%Y-%m-%d %H:%M:%S.%L` | 2026-03-07 23:59:30.123 |
| `%f` / `%Q` / `%s` / `%Z` | 123000 / 1772956770123 / 1772956770 / `-0800` (ASCII hyphen) |
| `%j` / `%U` / `%W` / `%V` / `%u` / `%w` | 066 / 09 / 09 / 10 / 6 / 6 |
| `%G-%g` / `%q` / `%e` / `%-d` / `%_H` / `%y` / `%I %p` | 2026-26 / 1 / ` 7` / 7 / 23 / 26 / 11 PM |
| `%N` of t + 456,789 ns (ours) | 123456789 |
| `utc` `%Y-%m-%dT%H:%M:%S.%LZ` / `iso_format(t)` | 2026-03-08T07:59:30.123Z (both) |
| `format("%y")` of local year −2 / `%Y` of local years 123 and −2 (`format-test.js:258–271`) | -02 / 0123, -0002 |
| `format("%V")` of local 2015-12-31 and 2016-01-01 (`format-test.js:242–243`) | 53, 53 |
| `iso_parse("2026-03-07T23:59:30.123Z")` | `Ok` of that instant |
| Berlin scale (Part I), `labels(&tick_set(8), &Locale::default())` | 10-25, 03:00, 06:00, 09:00, 12:00, 15:00, 18:00, 21:00, 10-26 |
| same, `&Locale::EN_US` (d3 in `TZ=Europe/Berlin`) | Oct 25, 03 AM, 06 AM, 09 AM, 12 PM, 03 PM, 06 PM, 09 PM, Mon 26 |
| `tick_label(local 1990-01-01 00:00:00.012, EN_US)` (d3 `multi`) | .012 |
| `tick_label(local 1990-01-02, EN_US)` / `(1990-02-01)` / `(1990-01-01)` | Tue 02 / February / 1990 |
| `utc()` over `[t0, t0 + 10 µs]`, t0 = 2026-10-25T12:00Z: `labels(&tick_set(5), &Locale::default())` (ours) | 12:00 (t0 is on the hour: hour rung), then 00.000002, 00.000004, 00.000006, 00.000008, 00.000010 (`%S.%f`; each has a non-zero part below the millisecond) |

#### Not in this phase

- Parsing d3 format strings (`timeParse`, `utcParse`, a `time::Parser`): Later; users parse with jiff's `strtime` and `time::iso_parse`.
- Minor ticks for time axes (`Ticks::minor` keeps Phase 6's empty default; if Phase 33 wants them, it records the rung rule).
- `timeFormatDefaultLocale` (global state, never); the other 24 d3 time tables as constants.
- jiff's `strftime`-only directives (`%F %T %k %l %P %:z` …): unknown directives print as d3 prints them.
- Figure time handles and rotated date labels (Phase 33); ICU4X.

### Phase 29 — Shapes breadth

*M3 Core breadth · builds on phase 8 · about 1,150 lines of Rust plus ≤ 250 lines of translator JS, 8 commits*

#### Goal

Core gains the rest of d3-shape that charts use: `shape::pie` with `shape::Slice`; `shape::arc` (inner and outer radius, corner radius, pad angle and pad radius, centroid); `shape::stack` and `shape::Stack` with d3's five offsets and six orders over columns; `shape::Symbol` with d3's seven fill symbols; radial lines and areas (`shape::line::Radial`, `shape::area::Radial`); the accessor wrapper `shape::LineOf<'a, D>`; and a provided `PathContext::rect`. d3's arc, pie, stack, offset, order, symbol, symbols, lineRadial and areaRadial tests grade it. The phase depends only on Phase 8 and may run earlier.

#### Builds on

- Phase 3: `path::PathContext` with `move_to`, `line_to`, `quadratic_curve_to`, `bezier_curve_to`, `arc` (a negative radius draws nothing) and `close_path`; `path::SvgPath`, which prints what `d3.path()` prints, with an inherent `SvgPath::rect` (`M{x},{y}h{w}v{h}h{−w}Z`) and `with_digits`.
- Phase 8: `shape::Curve` (every d3 curve, `BumpX` and `BumpY` included), `shape::Vertex` ((x, y) or `None`), `shape::Span` ((x, y0, y1), (x, y1) over a baseline, or `None`), `shape::line()`, `shape::area()`, and the crate-private `curve::State` (`area_start`, `area_end`, `line_start`, `line_end`, `point`) with a constructor from `Curve`; a `Bundle` curve in an area draws as `Basis`.
- Decision files `decisions/phase-03.md`, `phase-08.md`.

#### Design

Module layout, one type per module:

```text
core/src/shape.rs                     module declarations; pie(), arc(), stack()
core/src/shape/slice.rs               Slice
core/src/shape/pie.rs                 Pie, pie::Sort
core/src/shape/arc.rs                 Arc
core/src/shape/stack.rs               Stack, stack()
core/src/shape/stack/{order,offset}.rs   stack::Order, stack::Offset
core/src/shape/symbol.rs              Symbol
core/src/shape/line/radial.rs         line::Radial, line::radial()
core/src/shape/area/radial.rs         area::Radial, area::radial()
core/src/shape/area.rs                + the crate-internal area loop
core/src/shape/line_of.rs             LineOf
core/src/path.rs, core/src/path/svg.rs   the provided rect and SvgPath's own
```

`shape::line` and `shape::area` stay functions (Phase 8) and are also public modules (`pub mod line; pub mod area;`): functions and modules live in different namespaces. If Phase 8 declared the modules private, making them public is an addition.

##### Pie and Slice

```rust
/// One wedge: d3's arc datum with indices in place of `data`. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    /// Position of the datum in the input (d3's `data`, as an index).
    pub datum: usize,
    /// Position in drawing order (d3's `index`).
    pub index: usize,
    pub value: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub pad_angle: f64,
}
pub fn pie() -> Pie;
impl Pie {
    pub fn sort(self, sort: pie::Sort) -> Self;     // Descending (d3's default), Ascending, Input
    pub fn start_angle(self, a: f64) -> Self;       // 0
    pub fn end_angle(self, a: f64) -> Self;         // τ
    pub fn pad_angle(self, a: f64) -> Self;         // 0
    pub fn layout(&self, values: impl IntoIterator<Item = f64>) -> Vec<Slice>;
}
```

Slices carry both `datum` (which input value) and `index` (drawing order), because d3's `index` is the rank after sorting and results must still point back to their data. `layout` returns slices in input order (`slices[i].datum == i`), following `pie.js:15–52`: `da = end − start` clamped to ±τ, `p = min(|da|/n, pad)`, `pa = p·sign(da)`, `k = (da − n·pa)/sum` (0 when the sum is 0), and each slice in drawing order spans `v·k + pa`, where a non-positive value contributes no `v·k`; every slice's `pad_angle` is `p`. `Descending` and `Ascending` sort stably with `f64::total_cmp` (reversed for descending) and put NaN values last: d3's comparator calls NaN pairs equal, which leaves their place to the JS engine, and a non-total comparator can panic in Rust's sort since 1.81. The prototype `proto/rfc/d3/src/shape.rs:210–319` stores the data index in `index` and ignores `padAngle`, and its arc has no corners or padding; port d3 instead.

##### Arc

```rust
pub fn arc() -> Arc;
impl Arc {
    pub fn inner_radius(self, r: f64) -> Self;        // 0
    pub fn outer_radius(self, r: f64) -> Self;        // 0
    pub fn corner_radius(self, r: f64) -> Self;       // 0
    pub fn pad_radius(self, r: Option<f64>) -> Self;  // None: √(r0² + r1²)
    pub fn pad_angle(self, a: Option<f64>) -> Self;   // None: the slice's pad_angle
    pub fn draw(&self, slice: &Slice, ctx: &mut impl PathContext);
    pub fn centroid(&self, slice: &Slice) -> (f64, f64);
}
```

A line-by-line port of `arc.js:88–227` (`intersect`, `cornerTangents`, the `cw` flag, the four `epsilon` guards with d3's `1e-12` from `math.js`, r0 > r1 swapped). Angles run clockwise from 12 o'clock; `ctx.arc(…, anticlockwise = !cw)`. A single annulus is drawn from a `Slice` literal.

##### Stack

```rust
/// d3.stack() over columns, one layer per column (Part I, Shapes and PathContext).
pub fn stack(columns: &[&[f64]]) -> Vec<Vec<[f64; 2]>>;
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stack { order: stack::Order, offset: stack::Offset }
impl Stack {
    pub fn order(self, order: stack::Order) -> Self;
    pub fn offset(self, offset: stack::Offset) -> Self;
    /// Layers indexed like `columns`, each `[y0, y1]` per row.
    pub fn layout(&self, columns: &[&[f64]]) -> Vec<Vec<[f64; 2]>>;
    /// The stacking order `layout` used, bottom first: d3's `series[i].index` inverted.
    pub fn stacking_order(&self, columns: &[&[f64]]) -> Vec<usize>;
}
// shape/stack/{order,offset}.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order { #[default] None, Ascending, Descending, InsideOut, Appearance, Reverse }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Offset { #[default] None, Expand, Diverging, Silhouette, Wiggle }
impl Order { pub fn apply(self, layers: &[Vec<[f64; 2]>]) -> Vec<usize>; }
impl Offset { pub fn apply(self, layers: &mut [Vec<[f64; 2]>], order: &[usize]); }
```

Rows are the shortest column's length; longer columns are truncated, and the doc comment says so. NaN values follow d3's offsets (`offset/none.js`: a NaN top inherits the bottom). `Order::apply` and `Offset::apply` are public because d3's order and offset tests call them directly. The figure's helpers stack with `shape::stack` and emit explicit-baseline marks (Phase 34).

##### Symbols and the provided `rect`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol { Circle, Cross, Diamond, Square, Star, Triangle, Wye }
impl Symbol {
    /// d3.symbolsFill, in d3's order: circle cross diamond square star triangle wye.
    pub const FILL: [Symbol; 7];
    /// Draws the symbol centred at the origin with area `size` (d3's default 64).
    pub fn draw(self, size: f64, ctx: &mut impl PathContext);
}
```

Constants come from `d3-shape/src/symbol/*.js` (for `star`: `ka = 0.89081309152928522810`, `kr = sin(π/10)/sin(7π/10)`). `Square` calls `ctx.rect(x, x, w, w)` (`square.js:7`), so `PathContext` gains a provided method:

```rust
fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
    self.move_to(x, y);
    self.line_to(x + w, y);
    self.line_to(x + w, y + h);
    self.line_to(x, y + h);
    self.close_path();
}
```

`impl PathContext for SvgPath` supplies its own `rect`, delegating to the inherent `SvgPath::rect`, so `Symbol::Square.draw(64.0, …)` prints d3-path's `M-4,-4h8v8h-8Z`. `IcedPath` and figure's `path::Bez` take the provided method. A provided method is an addition to the trait; no implementor changes.

##### Radial lines and areas

```rust
// shape/line/radial.rs, reached as shape::line::radial() and shape::line::Radial
pub fn radial() -> Radial;
impl Radial {
    pub fn curve(self, curve: Curve) -> Self;
    /// Items are (angle, radius), the angle clockwise from 12 o'clock.
    pub fn draw<V: Vertex>(&self, points: impl IntoIterator<Item = V>, ctx: &mut impl PathContext);
}
// shape/area/radial.rs, reached as shape::area::radial() and shape::area::Radial
pub fn radial() -> Radial;
impl Radial {
    pub fn curve(self, curve: Curve) -> Self;
    pub fn inner_radius(self, r: f64) -> Self;   // the baseline for (angle, outer) items
    /// Items are (angle, inner, outer), (angle, outer), or None.
    pub fn draw<S: Span>(&self, items: impl IntoIterator<Item = S>, ctx: &mut impl PathContext);
}
```

d3's `curveRadial` maps each point `(a, r)` to `(r·sin a, −r·cos a)` before the wrapped curve sees it; do exactly that, feeding Phase 8's `curve::State`. The radial area's top line uses the outer radius and its base line the inner radius, so the two lines have different x, while Phase 8's `Area::draw` shares x. Factor Phase 8's area loop into a crate-internal function over `(x0, y0, x1, y1)` runs and call it from both `Area` and `area::Radial`; Phase 8's tests stay green.

##### `LineOf<'a, D>`

Part I's accessor option B, lifted from `proto/rfc/d3/src/shape.rs:210–247`: boxed accessors borrowing for `'a` (`Box<dyn Fn(&D) -> f64 + 'a>`, never `'static`) for `x`, `y` and `defined`, plus `curve`, and `draw(&self, data: &[D], ctx)` that maps the slice and calls `shape::line().curve(…).draw(…)` — no second code path. Default accessors return 0 and `defined` returns true. There is no `AreaOf`.

##### Translator rules (Step 0, budget ≤ 250 lines of JS)

- arc datum object literals (`{startAngle: 0, endAngle: Math.PI, innerRadius: 0, outerRadius: 100, padAngle: 0}`) become `Slice { datum: 0, index: 0, value: 0.0, start_angle, end_angle, pad_angle }` plus `arc().inner_radius(…)`-style builder calls for the radius keys (d3 reads them from the datum through its default accessors); `arc().innerRadius(() => 42)` constant-returning arrows become the builder;
- `pie()(data)` with `.value(d => …)` over literal arrays, `.sort(null)`, `.sortValues` with `ascending`/`descending`; results compared field-wise against object-literal arrays (`{data, value, index, startAngle, endAngle, padAngle}` → `Slice`, with `datum` the position of `data` in the input);
- `stack().keys([k…])(rows)` over a literal array of arrays or of objects becomes columns built at translation time (`rows[i][k]` per key, transposed); the test helper `series([[…]], data, key, index)` becomes the expected `Vec<[f64; 2]>`, with `stacking_order` for `index`;
- `symbol().type(symbolStar).size(64)()` becomes `Symbol::Star.draw(64.0, &mut SvgPath)`;
- `pathRound` and `assertPathEqual` as in Phase 3.

Anything else is a skip with its construct named. Commit before Step 1.

##### Decisions this phase may meet

- A pie test depends on where NaN values land in sorted order — keep NaN last, as designed, make the assertion a skip; record it as a decision.

#### Steps

0. **Translator rules**, as above.
1. **Pie and Slice** (`shape/{slice,pie}.rs`). Bless pie ids after reading the diff.
2. **Arc** (`shape/arc.rs`): a straight port. Bless arc ids.
3. **Stack, orders, offsets** (`shape/stack.rs`, `stack/{order,offset}.rs`). Bless stack, offset and order ids.
4. **Symbols and `rect`** (`shape/symbol.rs`, the provided method in `path.rs`, `SvgPath`'s own in `path/svg.rs`). Bless symbol and symbols ids.
5. **Radial line and area** (`shape/{line,area}/radial.rs`, the area-loop extraction). Bless lineRadial and areaRadial ids.
6. **`LineOf`** (`shape/line_of.rs`).
7. **Tests from Part I** (`pie_and_donut_examples`, `stack_columns_example`, the option-B line) and `check.sh full`.

#### Exit criteria

1. `scripts/check.sh quick` is green.
2. `cargo run -p rustd3-harness -- --check` is green with these floors (≥ 70% of a file's assertions unless stated): `arc-test.js` (122), `pie-test.js` (38), `stack-test.js` (23), `lineRadial-test.js` (6), `areaRadial-test.js` (28), `offset/*.js` and `order/*.js` (35 together); `symbol-test.js` (44) ≥ 70% of the 31 assertions on fill symbols; `symbols-test.js` (3): its `symbolsFill` row. Listed skips: accessor functions reading `this` or `arguments`; `context(null)` return values; object identity (`arcs[i].data === data[i]`, `symbols === symbolsFill`); `stack.keys(function)` and `stack.value(function)` getters (function identity); the 13 assertions on stroke symbols in `symbol-test.js` and the `symbolsStroke` row ("stroke symbols: Later"); `link-test.js` (42), not translated ("`shape::link`: Later").
3. Tests `pie_and_donut_examples` and `stack_columns_example` assert the rows marked "Part I", and Part I's option-B line drawn through `LineOf` equals the option-A output.
4. A command-counting `PathContext` test double that counts `rect` itself: `Symbol::Circle` emits one `move_to` and two `arc`s, `Symbol::Square` one `rect`; a double that keeps the provided `rect` sees `move_to`, three `line_to` and `close_path` for the square.
5. `scripts/check.sh full` is green at the last commit.

#### Spot checks

d3 7.9.0 (d3-shape 3.2.0) in Node; paths printed by d3-shape's default 3-digit path.

| Input | Expected |
| --- | --- |
| `pie().layout([3, 1, 2])` as (index, start, end) | (0, 0, π), (2, 5.235988, τ), (1, π, 5.235988) |
| `pie().sort(Input).layout([3, 1, 2])` | (0, 0, π), (1, π, 4.18879), (2, 4.18879, τ) |
| `pie().pad_angle(0.1).layout([1, 1])` | (0, π, pad 0.1), (π, τ, pad 0.1) |
| `arc().inner_radius(40).outer_radius(80).centroid(&slices[0])` of `pie().layout([3, 1, 2])` (Part I) | (60, 0) |
| that arc drawn for `slices[0]` (Part I) | M0,-80A80,80,0,1,1,0,80L0,40A40,40,0,1,0,0,-40Z |
| `arc().outer_radius(100)`, 0..π/2 | M0,-100A100,100,0,0,1,100,0L0,0Z |
| `arc().inner_radius(50).outer_radius(100).corner_radius(10)`, 0..π/2 | M0,-89.443A10,10,0,0,1,11.111,-99.381A100,100,0,0,1,99.381,-11.111A10,10,0,0,1,89.443,0L59.161,0A10,10,0,0,1,49.301,-8.333A50,50,0,0,0,8.333,-49.301A10,10,0,0,1,0,-59.161Z |
| same radii, `pad_angle(Some(0.1))`, no corners | M5.588,-99.844A100,100,0,0,1,99.844,-5.588L49.687,-5.588A50,50,0,0,0,5.588,-49.687Z |
| `stack(&[&[3, 2, 1], &[1, 1, 4]])[1]` (Part I) | [[3, 4], [2, 3], [1, 5]] |
| `Stack::default().offset(Expand)`, same columns, layer 0 | [[0, 0.75], [0, 0.6666666666666666], [0, 0.2]] |
| `offset(Diverging)` on columns [3, −2, 1] and [−1, 1, 4] | [[0, 3], [−2, 0], [0, 1]] and [[−1, 0], [0, 1], [1, 5]] |
| `offset(Silhouette)` on [1, 2, 4], [2, 3, 1], layer 0 | [[−1.5, −0.5], [−2.5, −0.5], [−2.5, 1.5]] |
| `offset(Wiggle)` on [1, 2, 4], [2, 3, 1] | [[0, 1], [−1.1, 0.9], [−2.1, 1.9]] and [[1, 3], [0.9, 3.9], [1.9, 2.9]] |
| `order(InsideOut)` on columns [1, 2, 3], [5, 6, 7], [2, 1, 3]: `stacking_order` (d3's `order(series)`, bottom first) | [2, 0, 1] (d3's per-series `.index`: 1, 2, 0) |
| `Symbol::Circle.draw(64.0)` | M4.514,0A4.514,4.514,0,1,1,-4.514,0A4.514,4.514,0,1,1,4.514,0 |
| `Symbol::Square` | M-4,-4h8v8h-8Z |
| `Symbol::Diamond` / `Triangle` | M0,-7.445L4.298,0L0,7.445L-4.298,0Z / M0,-7.019L6.079,3.51L-6.079,3.51Z |
| `Symbol::Star` | M0,-7.551L1.695,-2.333L7.181,-2.333L2.743,0.891L4.438,6.109L0,2.884L-4.438,6.109L-2.743,0.891L-7.181,-2.333L-1.695,-2.333Z |
| `line::radial().draw([(0, 1), (π/2, 2)])` | M0,-1L2,0 |
| `area::radial().draw([(0, 1, 2), (π/2, 1, 3)])` | M0,-2L3,0L1,0L0,-1Z |
| Part I's option-B line: `line_of().x(…).y(…).defined(…)` over its readings | M0,75L100,25M300,50L400,0 |

#### Not in this phase

- `shape::link` (horizontal, vertical, radial links) and the stroke symbols (`Asterisk`, `Diamond2`, `Plus`, `Square2`, `Times`, `Triangle2`, `symbolsStroke`): Later.
- d3's index-passing accessors (`(d, i, data)`): indices come from `enumerate()`.
- `arc.context`, `pie.value` with non-literal functions, `AreaOf`, a `Symbol` builder type, custom symbol types (the enum is closed).
- `stack.keys` with string keys over row objects (columns instead).
- Figure marks over these shapes: bars, pies, wheels (Phases 34, 39).

### Phase 30 — Behaviors and animation

*M3 Core breadth · builds on phase 10 · about 900 lines of Rust, 5 commits*

#### Goal

Core gains the remaining d3 behaviors as renderer-agnostic state machines over Phase 10's `interact::Pointer`: `interact::BrushX` and `interact::BrushY`, d3-brush's one-dimensional brushes (a press on the overlay starts a selection, on the selection drags it, on a handle resizes it with flips), reporting `interact::BrushEvent<S = [f64; 2]>`; `interact::Drag` (d3-drag with a subject and a click distance); the keyed `join::join`; and `anim` — `Animated<T>`, `Timing`, `Marks<K, T>` with enter, update, exit and stagger — sampled at caller-supplied `Instant`s. Modifier events are accepted and ignored: v0.1 has no keyboard modes. d3-brush's only test is an export list, so hand-written traces derived from `d3-brush/src/brush.js` grade the brushes; use case 2's lifecycle trace grades `anim`. No d3 test file is translated in this phase.

#### Builds on

- Phase 3: `ease::Ease` (a closed enum: `Linear`, `Cubic(Mode)`, …; `apply(&self, t)`; `Default` = `Cubic(InOut)`, d3's transition default).
- Phase 4: `interpolate::Interpolate`.
- Phase 10: `interact::Pointer` (`#[non_exhaustive]`: `Pressed { x, y, button, modifiers }`, `Moved { x, y }`, `Released { button }`, `Wheel { x, y, delta, modifiers }`, `DoubleClicked { x, y }`, `Modifiers(Modifiers)`, `Left`), `interact::Button`, `interact::Modifiers` (`shift()`, `ctrl()`, `alt()`, `logo()`, constants `NONE`, `SHIFT`, …, `union`), `interact::Zoom::is_active`.
- Phase 24: with the `derive` feature the anim tests may use `#[derive(Interpolate)]`; without it they write the impl.
- Decision files `decisions/phase-03.md`, `phase-10.md`.

#### Design

Module layout:

```text
core/src/interact.rs                  re-exports BrushX, BrushY, BrushEvent, Drag
core/src/interact/brush.rs            module root
core/src/interact/brush/engine.rs     Engine (crate-private): d3's brush(dim), dim ∈ {X, Y}
core/src/interact/brush/event.rs      BrushEvent
core/src/interact/brush/{x,y}.rs      BrushX, BrushY
core/src/interact/drag.rs             Drag
core/src/interact/drag/event.rs       drag::Event
core/src/join.rs                      Join, join()
core/src/anim.rs                      re-exports
core/src/anim/{animated,timing,marks}.rs   Animated; Timing; Marks and anim::Phase
core/src/lib.rs                       pub mod join; pub mod anim;
```

Matches on `Pointer` inside core need no wildcard arm (same crate) and list every variant explicitly; no `_ =>` arm on workspace enums. The behaviors never read a clock; `anim` takes `Instant`s from the caller.

##### Brushes

```rust
/// What a brush reports: d3's start, brush and end events. `S` is the selection;
/// the default is an interval, and a 2-D form would use `[[f64; 2]; 2]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushEvent<S = [f64; 2]> { Start(Option<S>), Brush(Option<S>), End(Option<S>) }

/// d3.brushX(): selects an x interval inside `extent` = [[x0, y0], [x1, y1]].
#[derive(Debug, Clone, PartialEq)]
pub struct BrushX { engine: brush::Engine }
impl BrushX {
    pub fn new(extent: [[f64; 2]; 2]) -> Self;
    pub fn handle_size(self, px: f64) -> Self;                     // 6, d3's default
    pub fn handle(&mut self, input: Pointer) -> Option<BrushEvent>;
    pub fn selection(&self) -> Option<[f64; 2]>;
    pub fn set_selection(&mut self, selection: Option<[f64; 2]>);  // brush.move, no events
    pub fn is_active(&self) -> bool;
}
/// d3.brushY(): the same over y.
#[derive(Debug, Clone, PartialEq)]
pub struct BrushY { engine: brush::Engine }   // the same methods
```

`brush::Engine` is d3's `brush(dim)` for `dim ∈ {X, Y}`. It holds the `extent`, `selection: Option<[[f64; 2]; 2]>` (an X brush stores the extent's y, a Y brush the extent's x, as `dim.input` does), the last modifiers, and an optional gesture with d3's locals for one pointer: `type` (overlay, selection, or a handle `w`, `e`, `n`, `s`), `mode` (drag or handle), `signX`, `signY`, `w0 n0 e0 s0`, `w1 n1 e1 s1`, `dx dy`, `point0`, `point`.

| Input | d3 (`brush.js`) | Engine |
| --- | --- | --- |
| `Pressed { button: Left, modifiers }` without ctrl, hitting a handle, the selection or the overlay | `started` (334–411) | the hit test gives `type`; `mode` is drag on the selection, else handle; a press on the overlay starts a zero-size selection at the point (`w0 = e0 = x` for X; the other axis spans the extent); emit `Start(selection)` |
| `Pressed` with another button or with ctrl, or hitting nothing | the filter (9–11), or no element | `None` |
| `Moved { x, y }` during a gesture | `moved` → `move` (413–491) | `dx`, `dy` from `point0`. Drag mode: `dx` clamped to `[W − w0, E − e0]` and both edges move. Handle mode: for `signX < 0`, `dx` clamped to `[W − w0, E − w0]` and `w1 = w0 + dx`; for `signX > 0`, to `[W − e0, E − e0]` and `e1 = e0 + dx` (likewise n, s with `signY`). `e1 < w1` flips: `signX` negates, `w0 ↔ e0` and `w1 ↔ e1` swap, and the handle type flips `w ↔ e` (`n ↔ s` on Y). Emit `Brush(selection)` only if the selection changed |
| `Released { button: Left }` during a gesture | `ended` (493–508) | an empty selection (zero width; zero height for Y) becomes `None`; emit `End(selection)` |
| `Released` of another button, or with no gesture | — | `None` |
| `Modifiers(m)`, during a gesture or not | keyboard modes, Later | stored; `None`; no mode changes |
| `Wheel`, `DoubleClicked`, `Left` | not brush events | `None`; a drag continues after `Left` until `Released`, as d3 listens on the window |

Modifiers on a press other than ctrl are accepted and ignored: alt does not center, logo (d3's `metaKey`) does not force the overlay, shift does not lock an axis.

**Hit test**, in d3's DOM stacking order — handles above the selection above the overlay, later handles above earlier ones: for X test `e`, then `w`, then the selection, then the overlay (the extent); for Y `s`, then `n`. A handle's rect is `redraw`'s (269–273): x at `sel.e − hs/2` for types ending in `e`, else `sel.w − hs/2`; y at `sel.s − hs/2` for types starting with `s`, else `sel.n − hs/2`; width `sel.e − sel.w + hs` for `n`/`s`, else `hs`; height `sel.s − sel.n + hs` for `e`/`w`, else `hs`; edges inclusive. When a selection is narrower than the handle size (`[50, 50]` right after a press, or `[10, 12]`), both handle rects cover the point and the later handle wins: `e` over `w` (`brush.js:27`), `s` over `n` (`brush.js:34`). Trace T11 asserts it.

`set_selection` during a gesture replaces the selection the gesture compares against, as `brush.move` does mid-gesture (`brush.js:479, 505`); use case 1 still syncs the brush only between gestures (`if !brush.is_active()`). Programmatic moves emit no events: the caller already knows.

The prototype `proto/rfc/d3/src/interact.rs:112–188` has the `BrushX` shape (`is_active`, `set_selection`); its press hit test and events differ from d3, so port d3.

##### Drag

```rust
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Drag { click_distance: f64, gesture: Option<Gesture> }   // Gesture: crate-private
// interact/drag/event.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Start { x: f64, y: f64 },
    Drag { x: f64, y: f64, dx: f64, dy: f64 },
    /// `moved`: the pointer travelled beyond the click distance (d3 then suppresses the click).
    End { x: f64, y: f64, moved: bool },
}
impl Drag {
    pub fn click_distance(self, px: f64) -> Self;   // 0
    /// `subject` is asked on a press: the dragged thing's position, or None to ignore the press.
    pub fn handle(&mut self, input: Pointer, subject: impl FnOnce(f64, f64) -> Option<(f64, f64)>)
        -> Option<drag::Event>;
    pub fn is_active(&self) -> bool;
}
```

d3-drag's semantics (`d3-drag/src/drag.js`): the filter (9–11) drops non-left buttons and ctrl; the subject's offset `s − p0` is added to every reported position (`x = p.x + s.x − p0.x`, `beforestart` 121–140); `dx` and `dy` are the pointer's movement since the previous event; `moved` latches once `dx² + dy² > click_distance²` measured from the press point (`mousemoved` 65–70); `End` reports the last position, since `Released` has no coordinates. `Modifiers`, `Wheel`, `DoubleClicked` and `Left` give `None`, and a drag continues after `Left`.

##### Join

```rust
/// Plain result with public fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join<K> { pub enter: Vec<K>, pub update: Vec<K>, pub exit: Vec<K> }
pub fn join<K: Eq + Hash + Clone>(old: impl IntoIterator<Item = K>, new: impl IntoIterator<Item = K>)
    -> Join<K>;
```

Each side keeps its order: `enter` and `update` in `new`'s order, `exit` in `old`'s. Duplicates follow d3-selection's keyed data join: the first occurrence of a key in `new` pairs with the first unmatched occurrence in `old`; further copies in `new` enter and further copies in `old` exit. `proto/rfc/d3/src/join.rs` (24 lines) is the starting point, with the duplicate handling added.

##### Animation

Lift `proto/rfc/d3/src/anim.rs` (150 lines) with these changes:

- `Timing { duration: Duration, delay: Duration, stagger: Duration, ease: ease::Ease }`, plain data with public fields; `Default` is d3's 250 ms, no delay, no stagger and `Ease::default()` (cubic in-out). The prototype's `fn(f64) -> f64` field becomes Phase 3's enum, which is `Copy` and comparable.
- `Animated<T: Interpolate>`: `new(value, now)`, `go(target, now, timing, extra_delay)` (retarget from the current value, as an interrupted d3 transition does), `value(now)`, `target()`, `is_animating(now)`. The start is `now.checked_add(delay)`, keeping `now` if that overflows; library code does not panic.
- `Marks<K, T>` with `anim::Phase { Present, Exiting }`: `update(targets, now, timing, enter, exit)` staggers by the target's position, enters from `enter(&target)`, animates exiting marks to `exit(&current)`, and retargets a key that returns while exiting, which becomes `Present` again; `values(now) -> impl Iterator<Item = (&K, T, Phase)>` in insertion order; `prune(now)`; `is_animating(now)`. It plans with `join::join`, as the prototype does.

**Decisions this phase may meet**

- None expected: the brush and drag behaviors are fixed by d3's source
  and the traces above, and animation by use case 2's lifecycle table.
  A trace that d3's source does not settle (a tie this section does not
  name) is recorded as a decision with the d3 line it depends on.

#### Steps

1. **Join** (`join.rs`) with duplicate handling; test `join_matches_d3_data_join` on the rows below.
2. **Brush engine and `BrushX`** (`interact/brush.rs`, `brush/{engine,event,x}.rs`); traces T1–T4, T8 and T11 in `core/tests/main/brush.rs` as Rust data, each annotated with the `brush.js` lines it exercises (`started` 334–411, `moved` 413–430, `move` 432–491, `ended` 493–508).
3. **`BrushY`** (`brush/y.rs`); the Y mirrors.
4. **Drag** (`interact/drag.rs`, `drag/event.rs`); traces G1–G4 in `core/tests/main/drag.rs`.
5. **Animation** (`anim.rs`, `anim/{animated,timing,marks}.rs`), `use_case_2_lifecycle` in `core/tests/main/anim.rs`; docs, each behavior's doc showing a trace as its example; `check.sh full`.

#### Exit criteria

1. `scripts/check.sh quick` is green.
2. Test `brush_traces_match_d3` replays T1–T4, T8, T11 and the Y mirrors T1Y–T4Y and asserts every emitted event.
3. Tests `drag_traces` (G1–G4) and `join_matches_d3_data_join` pass.
4. Test `use_case_2_lifecycle` reproduces Part I's lifecycle trace exactly (values to one decimal, phases, `is_animating`).
5. The behaviors never read a clock; `anim` takes `Instant`s. Test code that needs a base `Instant` calls `Instant::now()` under `#[allow(clippy::disallowed_methods)]` with the reason "a base instant for offsets; values do not depend on it".
6. `cargo run -p rustd3-harness -- --check` is green with unchanged counts.
7. `scripts/check.sh full` is green at the last commit.

#### Spot checks

Brush traces transcribed from d3-brush 3.0.0's `brush.js` for one pointer; handle size 6. X traces: extent [[0, 0], [100, 50]], pointer y = 10. Y mirrors: `BrushY` with the same extent, pointer x = 10, handles `n`/`s`, so the selection is clamped to [0, 50]. `Start(s)` and the others show the selection.

| Trace | Inputs → events |
| --- | --- |
| T1 create and flip | press 20 (overlay, handle mode) → Start [20, 20]; move 60 → Brush [20, 60]; move 10 → Brush [10, 20] (e passes w: flip); release → End [10, 20] |
| T2 click clears | sel [10, 20]; press 50 (overlay) → Start [50, 50]; release → End None |
| T3 drag, clamped | sel [10, 20]; press 15 (selection, drag) → Start [10, 20]; move 95 → Brush [90, 100] (dx 80 within [−10, 80]); release → End [90, 100] |
| T4 handle | sel [10, 20]; press 21 (handle e, rect [17, 23]) → Start [10, 20]; move 41 → Brush [10, 40]; release → End [10, 40] |
| T8 ctrl, right button | press 50 + ctrl → None; `Pressed { button: Right, … }` → None |
| T11 overlapping handles | sel [10, 12]; press 11 (w rect [7, 13] and e rect [9, 15] both cover it; e wins, `w0 = 10, e0 = 12`) → Start [10, 12]; move 30 (dx 19, e1 = 31) → Brush [10, 31]; release → End [10, 31]. Had w won: [12, 29] after the flip |
| T1Y | press 20 → Start [20, 20]; move 60 → Brush [20, 50] (dy 40 clamped to S − s0 = 30); move 10 → Brush [10, 20] (flip); release → End [10, 20] |
| T2Y | sel [10, 20]; press 40 (overlay) → Start [40, 40]; release → End None |
| T3Y | sel [10, 20]; press 15 (selection, drag) → Start [10, 20]; move 95 → Brush [40, 50] (dy 80 within [−10, 30]: 30); release → End [40, 50] |
| T4Y | sel [10, 20]; press 21 (handle s, rect [17, 23]) → Start [10, 20]; move 41 → Brush [10, 40]; release → End [10, 40] |
| G1 drag, subject offset | press (12, 13), subject (10, 10) → Start (10, 10); move (20, 25) → Drag x 18, y 22, dx 8, dy 12; release → End (18, 22), moved true |
| G2 drag, click | press (5, 5), subject Some → Start; release → End (5, 5), moved false |
| G3 drag, click distance 5 | press (0, 0); move (3, 4) → Drag (moved stays false: 25 ≯ 25); move (3, 5) → Drag; release → End, moved true |
| G4 drag, no subject | press with subject `None` → None; move → None |

| Input | Expected |
| --- | --- |
| `join(["a", "b", "c"], ["b", "c", "d"])` (Part I, Data and identity) | enter ["d"], update ["b", "c"], exit ["a"] |
| `join(["a", "a", "b"], ["a", "b", "b"])` | enter ["b"], update ["a", "b"], exit ["a"] |
| Use case 2: 100 ms, stagger 20 ms, cubic in-out; `update({a: 10, b: 20})` at 0 | t=0: a=0.0 b=0.0; t=50: a=5.0 b=2.2 |
| `update({b: 40, c: 30})` at 60 ms | t=60: a=7.4 (exit) b=5.1 c=0.0; t=110: a=3.7 (exit) b=22.6 c=3.2; t=180: a=0.0 (exit) b=40.0 c=30.0 |
| `prune(180 ms)` | b=40.0 c=30.0; `is_animating(180 ms)` false |
| The arithmetic behind those rows | cubic in-out(0.3) = 0.108, so b at 50 = 20 · 0.108 = 2.16; (0.6) = 0.744, so a at 60 = 7.44; b at 110 = 5.12 + (40 − 5.12) · 0.5 = 22.56; c at 110 = 30 · 0.108 = 3.24 |

#### Not in this phase

- The 2-D brush, d3-brush's keyboard modes (shift locks an axis, alt centers, meta starts a new selection, space moves it) and `Plot::on_region`: Later. Their traces, from the same transcription, for the writer who builds them (2-D extent [[0, 0], [100, 100]]):
  - T5 alt at press: sel [40, 60]; press 61 + alt (e, center) → Start [40, 60]; move 71 → Brush [30, 70]; release → End [30, 70].
  - T6 alt mid-drag: sel [40, 60]; press 61 → Start; move 71 → Brush [40, 70]; `Modifiers(ALT)` → none (unchanged); move 81 → Brush [30, 80]; `Modifiers(NONE)` → none; move 91 → Brush [30, 90]; release → End [30, 90]. It depends on keyupped-ALT (`brush.js:556–562`: `w0 = w1` for `signX > 0`, `e0 = e1` for `signX < 0`) not being the mirror of keydowned-ALT (`517–523`: `e0 = e1 − dx·signX`, `w0 = w1 + dx·signX`).
  - T7 logo inside the selection: sel [40, 60]; press 50 + logo (overlay) → Start [50, 50]; move 70 → Brush [50, 70]; release → End [50, 70].
  - T9 2-D shift lock: press (10, 10) + shift → Start [[10, 10], [10, 10]]; move (50, 20) → Brush [[10, 10], [50, 10]] (y locked); move (60, 70) → Brush [[10, 10], [60, 10]]; `Modifiers(NONE)` → Brush [[10, 10], [60, 70]]; release → End [[10, 10], [60, 70]].
  - T10 2-D corner: sel [[10, 10], [50, 50]]; press (51, 51) (handle se) → Start; move (71, 61) → Brush [[10, 10], [70, 60]]; release → End, the same.
- Touch and multi-pointer gestures, `brush.move` transitions, cursors and the DOM overlay.
- `Pointer` translation from iced (Phase 11); the brush overlay in `Plot`, on `BrushX` and `BrushY` (Phase 36); use cases 1 and 2 as apps (Phase 32).
- d3-transition's chained and named transitions, `attrTween`, `on("end")`; smooth zoom through `interpolateZoom` (Later).

## Conditional — tiles and pan translation

Built only when the benchmark verdict in `decisions/phase-22.md` asks for it.

### Phase 31 — Plot V: tiles and pan translation

*Conditional · builds on phase 22 and the user's benchmark run · about 2,600 lines (≈450 figure, ≈1,300 iced, ≈250 bench, ≈600 tests), 10 commits · two or three sessions: cuts after Steps 2 and 5*

**Condition.** This phase is built only if the benchmark verdict in
`decisions/phase-22.md` attributes a missed pan or zoom budget to
raster or upload. Otherwise it is skipped: the verdict records "Phase
31 not built" with its reason (budgets met, or UI-thread bound), the
handoff marks the phase skipped, and tiles and pan translation
(≈ 2,600 lines) become a 0.2 item. Phase 40 then has no Phase 31 work
to check.

#### Goal

Pans and zooms stop re-rendering what is still valid, and the raster
path gains two cheaper primitives where they measurably win. Plot
areas, not partition cells, carry tiles of 256 device px on a grid
anchored in data space to the thinning column grid (a spanning panel's
plot area is cut into several cells by other panels' edges, but its
tile grid must be one). A drag snaps to whole device pixels, resident
tiles translate and only entering tiles render; gutter cells that show
the panned handle re-render whole. Thinning columns keep a data-space
origin during a gesture and dashed lines keep their phase on the tile
grid, so translated pixels stay exact. A zoom draws the previous tiles
scaled (Phase 20's previews, per tile) and renders the new ones nearest
the cursor first on a pool of threads (`Plot::threads`), newest wins
per tile. Plot areas keep one ring of overscan tiles. Invalidation is by
dependency (which handle moved), checked against part hashes; adjacent
gutter cells merge; `Caching::Verify` re-renders kept tiles in the
background and reports mismatches. Steps 1–2 measure envelope fills and
marker splatting on the verdict's workloads and build the winners. The
phase ends with a rerun of the benchmark and the GPU-layer
recommendation.

#### Builds on

- Phase 22: `rustd3-bench` (CLI, workloads, driver, headless loop,
  report) and the benchmark verdict in `decisions/phase-22.md` naming
  the missed budgets, their scenarios, workloads and buckets, and the
  measured cost-model calibration factor.
- Phase 21: `Fit::Window` quantization, `Caching::{On, Off}`,
  `Plot::revision`.
- Phase 20: gestures with per-handle tracks and transforms, `Domains`,
  held layout (`layout::Areas`), scaled previews (`Affine1`).
- Phase 19: the worker, `Worker::replace`, `Job`/`Done`, generations,
  slots with `Shown`, `Frame` accounting, `budget.rs`, `probe::Target`.
- Phase 17: part hashes, `Resolved::plot_areas()`, `list::cells`.
- Phase 15: `thin::Memo`, thinning columns from `invert` at
  device-pixel edges; M4 returns one sample beyond each side of the
  view.
- Phase 14: `Held` (private fields) and `Target` (both keep their
  shapes; this phase only adds methods to `Held`).
- Phase 12: `raster::Renderer`, `raster::Settings { threads, simd }`.
- Phase 9: M4 queries and `extremes`.

#### Design

##### Raster variants (`dev/bench/src/raster.rs`, CPU only)

For each workload the verdict named, at factors 1 and 2 and at 4K,
render each plot-area cell with figure's raster path (median of 7):

- **stroke** (as drawn: the thinned M4 polyline) vs **envelope** (per
  device column the M4 min/max from core's query on the same columns,
  filled as one band polygon grown by half the line width);
- **markers** (the `Markers` command) vs **splat** (the marker
  rasterized once into 16 coverage stamps at quarter-pixel phases, 4 ×
  4, composited source-over per point).

`rustd3-bench raster --plan default --out DIR` writes `raster.csv` with
columns workload, factor, variant, ms, speedup, max diff, % channels >
16/255. A variant is built into the figure crate only if its speedup is
≥ 1.5× with max diff ≤ 32/255 on the workloads the verdict named:
envelope as `Thin::Envelope` on `mark::line`, splatting as a `Markers`
path behind an additive switch:

```rust
impl raster::Renderer {
    /// Composite markers from pre-rasterized coverage stamps (default off).
    pub fn splat(&mut self, on: bool) -> &mut Renderer;
}
```

`raster::Settings` keeps its fields, so existing struct literals still
compile. Stamping dots from pre-rendered images measured 17.8 against
22.3 ms warm (37.7 ms cold) and differed by up to 54/255 (see iced
integration; `proto/dlist/src/bin/{stamps,sprites}.rs`), which is why
the rule asks for both a clear speedup and a bounded difference. The
table and the choice are recorded as a decision in
`decisions/phase-31.md`.

##### Tiles per plot area (`iced/src/plot/tile.rs`)

```rust
pub(crate) const SIZE: i32 = 256;

/// The tile grid of one plot area: device position of its data anchor
/// and the scale density it was rendered at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Grid {
    origin: [i32; 2],
    /// Transformed-domain span per device px, per axis, as f64 bits.
    density: [u64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Key { area: u32, i: i32, j: i32 }

pub(crate) struct Tile { key: Key, rect: [i32; 4], generation: u64, image: Option<image::Handle> }
```

Tile `(i, j)` is the square `[origin.x + 256 i, origin.x + 256 (i + 1))
× [origin.y + 256 j, origin.y + 256 (j + 1))` intersected with the plot
area's device rect grown by one tile (256 px) on each side: the
overscan ring (without the grown rect the ring tiles would be empty).
The plot area's rect is the snapped one from `Resolved::plot_areas()`;
a plot area is one slot even when the partition cuts it. Gutter cells
stay Phase 19's slots, with bands. A tile is 256 × 256 × 4 = 262,144 B,
far under `BAND_BYTES`, and wgpu's 2048² atlas layers hold 64 tiles
each (`wgpu/src/image/atlas.rs:13-14`). `probe::Target` gains `Tile {
area: u32, i: i32, j: i32 }`.

A density change (zoom) makes every key new; the old tiles become the
area's preview layer (Phase 20's `Affine1` per axis,
`FilterMethod::Linear` (`core/src/image.rs:213-220`), clipped to the
area) until every visible key is sharp; at most one preview layer per
area is kept (a second zoom re-maps it). The renderers trim undrawn
images each frame (`wgpu/src/image/raster.rs:78-118`,
`tiny_skia/src/raster.rs:141-144`); this phase discards tiles that
leave the grid itself. Translated and preview images are placed at
mapped bounds; `with_transformation` (`core/src/renderer.rs:43`) also
moves cached content on both renderers
(`proto/icedspike/src/bin/xform.rs`).

##### Snapped drags and the held column grid

While a drag is active, each track's offset is snapped to whole device
pixels before it is applied: `x' = round(x · f) / f` (`k` untouched);
published and owned domains during the drag are the snapped ones. At
the drag's end the unsnapped transform applies once, the residual
(< ½ device px) re-renders the moved areas fully, and the exact domain
is published. Whole-pixel shifts render identically within 1/255; a
half-pixel shift differs by up to 109/255
(`proto/dlist/src/bin/partial.rs:199-220`), hence the snap.

Figure side (`figure/src/thin.rs`, `figure/src/figure/held.rs`):

```rust
/// A thinning column grid anchored in output space (plain data).
pub struct Grid {
    /// Output px of the data anchor.
    pub origin: f64,
    /// Columns per output px.
    pub density: f64,
}

impl Held {
    /// Column grids to thin with, per handle index (additive).
    pub fn grids(&mut self, grids: Vec<(usize, thin::Grid)>) -> &mut Held;
    pub fn grids_for(&self, handle: usize) -> Option<thin::Grid>;
    /// Extends each plot area's data range and clip by `px` list px,
    /// for plot-area tile jobs (additive).
    pub fn overscan(&mut self, px: f64) -> &mut Held;
}
```

Grids and overscan travel in `Held`, so `Target::Screen { scale }` and
`raster::Settings` keep their shapes. With a grid, column boundaries
are `scale.invert(origin + c / density)` for the integer columns `c`
that intersect the view, so a whole-pixel pan reuses the same data
boundaries and `thin::Memo` hits for every resident column (shared
column boundaries: `proto/cols`; NaN runs: `proto/thin2/src/bin/gaps.rs`).
Outside gestures the anchor is the plot area's left edge (Phase 15's
behaviour). **Dash phase:** for a dashed thinned line the dash offset is
set so the pattern's phase is 0 at the first sample of the column
containing `origin`, measured along the emitted polyline in device
pixels; unthinned lines keep their phase from their first point.

##### Translation and overscan

A whole-pixel change of an x handle by `d` device px moves `Grid.origin`
of every plot area using it along x (likewise for y). Keys stay; squares
move. A resident tile stays valid when its rendered rect, shifted by
`d`, covers its new rect; otherwise it renders. New keys that intersect
the grown area render; keys that leave are discarded. Overscan tiles
render after the visible ones and need data beyond the view:
`Held::overscan` extends each plot area's data range and clip for
plot-area tile jobs only (gutters keep the normal list), and thinning
returns one sample beyond the grown view. A zoom-out by ≤ 1.2 is covered
by the ring.

##### Invalidation by dependency and merging

Each refresh classifies every slot:

1. **Handle changes.** Compare the scale a slot was rendered with to
   the new one at two domain points: a constant integral device-pixel
   offset is a *translation*, anything else a *change*.
2. **Part hashes.** A plot area whose parts' ids and hashes are
   unchanged apart from positions is checked with a second hash that
   strips positions: `style::hash(list: &List, part: &list::Part) ->
   u64` in `iced/src/plot/style.rs`, computed from the part's commands
   (paints, stroke styles, dashes, marker shapes, glyph ids and fonts);
   `Part` keeps its fields.

Rules: plot-area tile — hashes equal → keep; style hash equal and every
handle change a translation → translate and render entering tiles; else
render every tile. Gutter cell — hash equal → keep, else render whole.
`Plot::revision` and `Caching::Off` act as in Phase 21. A `Fit::Visible`
y handle that refits during an x pan is a change. After classification,
adjacent gutter cells in one row band with the same dependency set
merge greedily into one slot while the merged image stays under
`BAND_BYTES` (else it is drawn as bands).

##### Pool, order, threads, Verify

`worker.rs` becomes a pool: `n` threads (default available parallelism
− 1, clamped `1..=8`), each owning a `raster::Renderer::new(Settings {
threads: 0, .. })`, so the pool and vello_cpu's own threads never run
together and oversubscribe the cores. The queue (`queue.rs`) is
`Mutex<VecDeque<Job>> + Condvar` with `replace`; jobs are sorted:
visible tiles by distance from the zoom anchor or cursor, then gutters,
then overscan. Newest wins per tile as Phase 19 does per slot; a result
newer than what a tile shows but older than its pending generation is
shown as a preview of its own grid. With a pool, Phase 19's prediction
divides the modelled time of the work sent to the pool by `n`.

```rust
impl<'a, Message> Plot<'a, Message> {
    /// Render pool size, clamped to 1..=64; changing it restarts the pool.
    pub fn threads(self, threads: usize) -> Self;
}

pub enum Caching {
    On,
    Off,
    /// As `On`, then re-render every kept or translated slot after the
    /// real jobs, replace it if it differs by more than 1/255, and
    /// report `probe::Sample::Mismatch`.
    Verify,
}

// probe::Sample gains:
//     Mismatch { frame: u64, target: Target, max: u8 }
```

The benchmark gains `--threads N` and `--caching verify`.

**Decisions this phase may meet**

- A pan on a non-monotonic `Func` transform fails the two-point
  translation test — treat it as a change; record it as a decision if
  it occurs.
- The overscan margin exposes thinning past the recording's ends (no
  samples) — the grown view clips to the data; record it as a decision
  if a test shows a seam.

#### Steps

1. **bench: raster variants.** `dev/bench/src/raster.rs`, the `raster`
   command; run `--plan default` (headless, CPU only); test
   `envelope_covers_every_column_extreme`; record the table and the
   choice in `.claude/decisions/phase-31.md`. Verify: `cargo run
   --release -p rustd3-bench -- raster --plan default --out
   target/bench-raster`.
2. **figure: the winning variants.** `Thin::Envelope` in
   `figure/src/mark/line.rs` and/or `Renderer::splat` in
   `figure/src/raster.rs`, with agreement tests in
   `figure/tests/main/raster.rs` (≤ 32/255 against stroke and markers).
   If neither wins, the commit records that in the decision file only.
   *Cut point.*
3. **figure: held column grid, dash phase, overscan.** `thin.rs`,
   `mark/line.rs`, `figure/held.rs`. Tests in
   `figure/tests/main/thin.rs`:
   `anchored_columns_repeat_after_a_whole_pixel_pan`,
   `dash_phase_is_zero_at_the_anchor`, `overscan_extends_the_view`.
   Verify: `cargo test -p rustd3-figure --all-features -- thin raster`.
4. **iced: tiles and previews per area.** `tile.rs`, `preview.rs` keyed
   per area, `probe::Target::Tile`. Tests `tiles_match_the_whole_area`
   (≤ 1/255), `tiles_cover_the_area_once`, and Phase 20's preview tests
   pointed at tiles.
5. **iced: snapped drags, translation, overscan ring.** `gesture.rs`,
   `tile.rs`, `state.rs`. Tests `pan_renders_only_entering_tiles`,
   `thinned_lines_match_a_fresh_render_after_a_pan`,
   `dashed_lines_keep_their_phase_across_a_pan`,
   `drag_settles_on_the_exact_domain`,
   `zoom_out_preview_uses_the_overscan_ring`. *Cut point.*
6. **iced: dependency classification, style hash, merging.**
   `style.rs`, `merge.rs`, `state.rs`. Tests `unchanged_panels_are_kept`,
   `style_change_during_a_pan_re_renders`,
   `gutters_merge_into_fewer_images`.
7. **iced: pool, order, `Plot::threads`.** `worker.rs` becomes the
   pool, `queue.rs`, `plot.rs`. Tests `tiles_near_the_cursor_render_first`,
   `newest_result_wins_per_tile`, `threads_sets_the_pool_size`.
8. **iced: `Caching::Verify`.** `caching.rs`, `probe.rs`. Test
   `verify_replaces_a_differing_image` (corrupt a resident tile in
   `State`).
9. **bench: `--threads`, `--caching verify`, rerun.**
   `dev/bench/src/{cli,app}.rs`; headless smoke (`cargo run --release -p
   rustd3-bench -- run --headless --plan smoke --backend tiny-skia --out
   target/bench-smoke`); the user reruns `--plan default`; append the
   outcome to the verdict in `decisions/phase-22.md` (image path
   sufficient, or GPU layer recommended) and, if Phase 34 has already
   appended its label-churn rerun, write the closing line.
10. **Docs, `check.sh full`.**

Every step passes `scripts/check.sh quick`; iced steps also run
`ICED_TEST_BACKEND=tiny-skia|wgpu cargo test -p rustd3-iced --features
figure`.

#### Exit criteria

1. `raster.csv` is written by `rustd3-bench raster --plan default`;
   `decisions/phase-31.md` records which variants are built.
2. `pan_renders_only_entering_tiles`: a +37 device-px drag on a 900 ×
   600 device-px plot area with its overscan ring resident renders no
   visible tile; the entering ring column i = −2 (5 tiles) renders after
   the frame, plus the gutter cells that show the x handle, counted from
   `Sample::Raster`; the screenshot equals a fresh `Render::Inline`
   render of the snapped domain within 1/255.
3. `thinned_lines_match_a_fresh_render_after_a_pan`,
   `dashed_lines_keep_their_phase_across_a_pan` and
   `drag_settles_on_the_exact_domain` pass (≤ 1/255).
4. `zoom_out_preview_uses_the_overscan_ring` reports `missing == 0` on
   every frame of a ×1.2 zoom-out; `tiles_near_the_cursor_render_first`
   and `newest_result_wins_per_tile` pass.
5. `unchanged_panels_are_kept`, `style_change_during_a_pan_re_renders`,
   `gutters_merge_into_fewer_images`, `verify_replaces_a_differing_image`
   and `threads_sets_the_pool_size` pass; every iced test passes on both
   backends.
6. The user's `--plan default` rerun is appended to the verdict in
   `decisions/phase-22.md` with the final rule outcome;
   `scripts/check.sh full` passes.

#### Spot checks

| Input | Expected | Rule or source |
| --- | --- | --- |
| Plot area 900 × 600 device px, anchor at its corner, ring resident (grown rect [−256, 1156) × [−256, 856)), drag +37 px | origin x 37; every resident tile's rendered rect shifted by 37 covers its new rect (column −1: [−256, 0) → [−219, 37) covers [0, 37)); 0 visible tiles render; new ring key i = −2 (square [−475, −219) ∩ grown = [−256, −219)), 5 rows → 5 tiles render after the frame | translation |
| Same, drag −37 px | visible tiles kept (column 3: [768, 1024) → [731, 987) covers [731, 900)); no new key (i = −2 is [−549, −293), outside); ring column i = 4 (new rect [987, 1156), rendered [1024, 1156) → [987, 1119)) leaves [1119, 1156) uncovered → its 5 tiles re-render | translation |
| Drag at factor 1.25, raw offset 10.3 logical px | round(12.875) / 1.25 = 10.4 logical px (13 device px) | snap |
| Whole-pixel shift vs fresh render (vello_cpu) | ≤ 1/255; half pixel up to 109/255 | `partial.rs:199-220` |
| Plot area 900 × 600 with overscan | 4 × 3 visible keys + a ring = 6 × 5 = 30 keys | grid |
| One tile | 256 × 256 × 4 = 262,144 B < `BAND_BYTES` | |
| Zoom out ×1.2 about the centre of a 1,000 px area | previous view maps to [83.3, 916.7); the 83 px margins come from the ring | 500 ± 500 / 1.2 |
| x [0, 100] over 400 px, zoom ×2 about 200 px | `Affine1 { scale: 2, offset: −200 }`; a tile at [0, 256) previews at [−200, 312) | Phase 20 |
| Results for generations 3, 5, 4 arriving in that order, current 5 | shows 3 (preview of its own grid), then 5 (sharp); 4 dropped | newest wins per tile |
| 4K plot area 3600 × 1900 | 15 × 8 = 120 visible tiles; ≤ 64 per 2048² atlas layer → 2 layers | `atlas.rs:13-14` |
| 4 × 4 grid partition | 81 cells, 16 plot areas, 65 gutter cells before merging | 9 × 9 cells |
| `available_parallelism` 2 / 16 | 1 / 8 pool threads, each `threads: 0` | |
| Envelope vs stroke on `trace` at 2× | reported; built only if ≥ 1.5× and ≤ 32/255 | variant rule |
| Estimates (see iced integration) | pan strip 0.33 / 0.38 ms at 2×; shared-axis pan ≈ 8 ms; live 6–8 ms; 4K 15–18 sharp frames/s | |

#### Not in this phase

- A GPU data layer (its own plan, only if this phase's rerun still
  misses).
- Box, pinch and axis-drag zoom; thinning non-monotonic x; animated
  transitions.
- Everything above when the verdict says the budgets were met or the
  misses are UI-thread bound: the phase is skipped (see Condition).

## M4 — Complete figure

The use cases, the remaining handle kinds and marks, the wheel figure, themes, colorbars and hatching, hover and brushing, PDF and GIF, and the helpers.

### Phase 32 — The use cases

*M4 Complete figure · builds on phases 11, 17, 24, 28, 30 · about 850 lines (≈600 app code, ≈250 tests), 5 commits*

#### Goal

Part I's two core-level use cases run as iced apps in `rustd3-examples`,
built on the real crates rather than the prototypes: use case 1 (focus
and context: one `visible` window driven by a `Zoom` on the focus and a
`BrushX` on the overview) and use case 2 (the general update pattern:
keyed, staggered enter, update and exit through `anim::Marks` and
`#[derive(Interpolate)]`). Both are driven headlessly by `iced_test`'s
simulator, including the brush-sync regression that Use case 1
describes. A figure-level test file checks `fig.locale(Locale::DE_DE)`
against d3's de-DE output. The use cases stay at the core level by
design (see Architecture: two levels): no `Plot`, no figure and no
typst text inside them.

#### Builds on

- Phase 6: core `axis`, `ticks::Set`, `Ticks::labels(&set, &Locale)`.
- Phase 7: `scale::band`, `scale::ordinal`, `scheme::CATEGORY10`.
- Phase 8: `shape::line().curve(Curve::MonotoneX)`.
- Phase 10: `interact::Zoom` (`scale_extent`), `interact::Transform`,
  `interact::Rescale::rescale_x`.
- Phase 11: `rustd3_iced::IcedPath`, the `rustd3_iced::path(|p| …)`
  helper and the stateful `rustd3_iced::pointer::Tracker` (one per
  `canvas::Program::State`; `tracker.translate(&event, bounds, cursor)
  -> Option<Pointer>`, `wants_redraw()`), which produces
  `interact::Pointer` (`Pressed { x, y, button, modifiers }`,
  `Released { button }`, `Wheel { x, y, delta, modifiers }`,
  `Modifiers`, `Left`), with d3's wheel sign (positive `y` scrolls
  down and zooms out, Phase 10's `Delta`).
- Phase 15: the `trace` and `scatter` modules already in
  `dev/examples/src/lib.rs`.
- Phase 17: `Figure::locale`, the figure `Locale` applied to the tick
  formats a figure creates.
- Phase 24: `#[derive(Interpolate)]` behind core's `derive` feature
  (`dev/examples` depends on `rustd3-core` with `features =
  ["derive"]`).
- Phase 27: `time::time(zone)` and `time::utc()` scales over
  `jiff::Timestamp`, reached as `rustd3_core::jiff` (`dev/examples` has
  no jiff dependency of its own), and their `Rescale` impl.
- Phase 28: the ISO tick ladder; `locale::Locale::{EN_US, DE_DE}` with
  both halves.
- Phase 30: `interact::{BrushX, BrushEvent}`, `BrushX::is_active`,
  `BrushX::set_selection` (d3's `brush.move`); `anim::{Animated, Marks,
  Phase, Timing}`; `join`. `BrushX` has no keyboard modes:
  `Pointer::Modifiers` events reach it and are ignored, and nothing here
  uses them.
- Phase 1: `autotests = false` on every crate; the phase that writes a
  crate's first integration test adds its `[[test]] name = "main",
  path = "tests/main/main.rs"` stanza;
  `dev/clippy.toml` exempts the dev crates from the ambient-clock lint.
- Decision files: `phase-11.md`, `phase-17.md`, `phase-24.md`,
  `phase-28.md`, `phase-30.md`.

#### Design

##### Crate layout

Integration tests cannot import an `examples/*.rs` binary, so the app
code lives in the library and the examples are thin `main`s:

```text
dev/examples/src/lib.rs                  pub mod focus_context; pub mod update_pattern; (+ Phase 15's modules)
dev/examples/src/focus_context.rs        App, Message, Trace, Focus, Overview, data
dev/examples/src/update_pattern.rs       App, Message, Bar, Chart
dev/examples/examples/focus_context.rs   fn main() -> iced::Result
dev/examples/examples/update_pattern.rs  fn main() -> iced::Result
dev/examples/tests/main/main.rs          mod focus_context; mod update_pattern;
dev/examples/tests/main/{focus_context,update_pattern}.rs
figure/tests/main/locale.rs              registered in figure/tests/main/main.rs
```

`update_pattern::App::new` takes its first `Instant` as an argument, so
tests are deterministic; only `examples/update_pattern.rs` calls
`Instant::now()`.

##### Use case 1 (`focus_context.rs`)

```rust
pub const MARGIN: f32 = 40.0;
pub const FOCUS_HEIGHT: f32 = 340.0;
pub const OVERVIEW_HEIGHT: f32 = 90.0;

pub struct Trace {
    pub name: &'static str,
    pub points: Vec<(Timestamp, f64)>,
}

pub struct App {
    traces: Vec<Trace>,
    tz: TimeZone,
    full: [Timestamp; 2],
    /// The one source of truth: the brush and the zoom derive from it.
    visible: [Timestamp; 2],
}

#[derive(Debug, Clone)]
pub enum Message {
    Show([Timestamp; 2]),
}

impl App {
    pub fn new() -> Self;                         // 3 traces, 720 hourly points
    pub fn update(&mut self, message: Message);   // clamps to `full`
    pub fn view(&self) -> impl iced::Widget<Message> + '_;
    pub fn visible(&self) -> [Timestamp; 2];      // for tests
    pub fn full(&self) -> [Timestamp; 2];
}
```

**Data**, exactly as the prototype: start `2026-09-01T00:00:00Z`;
traces "sensor a", "sensor b", "sensor c" (s = 0, 1, 2); hour `h` in
`0..720`; value `10 + 3s + 2·sin(h/24) + noise`, the noise from the LCG
`seed = seed·1664525 + 1013904223` (u32, wrapping, seed 7) mapped to
`seed/u32::MAX − 0.5`. `full = [start, start + 719 h]`. The zone is
`TimeZone::UTC`, explicit; never the system zone.

**Scales** are rebuilt from state in `update` and `draw`:
`time::time(tz).domain(d).range([0, width])` with `width = bounds.width
− MARGIN`, and a linear y over all values with `.nice(5)`.

**Pointer origin.** The plot rectangle is `Rectangle { x: bounds.x +
MARGIN, width: bounds.width − MARGIN, .. }`, and the Phase 11 tracker
receives that rectangle, so a pointer's `x` is in range coordinates.
The prototype passes raw canvas coordinates to `Zoom` and `BrushX`,
whose ranges start at the plot area, which puts every anchor 40 px off.

**Focus.** State `Zoom::default().scale_extent([1.0, 50.0])`. In
`update`: from `visible` on the full scale, with `[v0, v1]` the visible
window's pixels over the range `[r0, r1]`, compute `k = (r1 − r0)/(v1 −
v0)` and `current = interact::Transform { k, x: −v0·k, y: 0.0 }`; feed the
pointer; rescale with `full.rescale_x(&next).get_domain()`; publish
`Show` with `and_capture()`.

**Overview.** State `Option<BrushX>`, created as
`BrushX::new([[r0, 0.0], [r1, OVERVIEW_HEIGHT as f64]])` with `[r0, r1]
= x.get_range()` (Phase 30's extent is `[[x0, y0], [x1, y1]]`):

```rust
// Between gestures follow the app's window; during one, the view may
// not have caught up with our own messages yet (use case 1's bug).
if !brush.is_active() {
    let shown = self.app.visible.map(|t| x.apply(t));
    brush.set_selection((self.app.visible != self.app.full).then_some(shown));
}
match brush.handle(input)? {
    BrushEvent::Brush(Some([a, b])) | BrushEvent::End(Some([a, b])) =>
        Some(Action::publish(Message::Show([x.invert(a), x.invert(b)])).and_capture()),
    BrushEvent::End(None) => Some(Action::publish(Message::Show(self.app.full))),
    _ => Some(Action::capture()),
}
```

**Drawing.** Lines through `rustd3_iced::path` with `Curve::MonotoneX`;
colors from `scale::ordinal(names, scheme::CATEGORY10)`; a selection
band `rgba(0.3, 0.5, 0.8, 0.2)` in the overview; axes from core's
`axis` with labels from `Ticks::labels(&set, &Locale::default())`,
drawn with iced's `fill_text` at 11 px (at the core level the host
draws text). `Frame::with_clip` clips nothing on the software renderer
(see iced integration), so the focus masks its left margin with a white
rectangle after drawing the lines.

**What to lift.** From `proto/rfc/glue/examples/focus_context.rs` (read
it, do not build it; `proto/README.md` says why): data generation
(lines 36–58), `update` (60–64), both `canvas::Program`s (107–204),
`draw_axis` (206–219), the two tests (243–277). Different here: the
pointer origin above, the `Pointer` variants of Phase 11,
`Ticks::labels`, no `unwrap`, no `matches_image`, no hard-coded `/tmp`
paths. iced at 3cbdacc: `test/src/simulator.rs:55`
`Simulator::with_size(settings, size, element)`, `:126` `point_at`,
`:208` `scroll`, `:217` `simulate`, `:283` `into_messages`;
`widget/src/action.rs:28-68` `Action::{capture, publish,
and_capture}`.

##### Use case 2 (`update_pattern.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Interpolate)]
pub struct Bar { pub y: f64, pub width: f64, pub opacity: f64, pub value: f64 }

pub struct App { seed: u32, marks: anim::Marks<&'static str, Bar>, now: Instant }

#[derive(Debug, Clone)]
pub enum Message { NewData(Instant), Frame(Instant) }

impl App {
    pub fn new(now: Instant) -> Self;              // seed 1, first NewData(now)
    pub fn update(&mut self, message: Message);
    pub fn subscription(&self) -> Subscription<Message>; // frames only while animating
    pub fn view(&self) -> impl iced::Widget<Message> + '_;
    pub fn bars(&self) -> Vec<(&'static str, Bar, anim::Phase)>; // tests: marks.values(now)
}
```

**Data.** Eight names `ant bee cat dog eel fox gnu hen`, each given the
value `(seed >> 8)/2²⁴ · 100` from the same LCG (seed 1, advanced once
per name before reading), sorted descending, top five. **Scales:** x
linear `[0, 100] → [0, 500]`; y `scale::band(names).range([0,
250]).padding(0.1)` (step 49.0196, bandwidth 44.1176). **Timing**
`{ duration: 750 ms, stagger: 20 ms, ..Timing::default() }`; enter
`|t| Bar { width: 0.0, ..*t }`; exit `|b| Bar { width: 0.0, opacity:
0.0, ..*b }`. `Frame(now)` stores `now` and prunes. Draw rectangles at
`x = 60` with height `y.bandwidth()`, labels only for
`Phase::Present`.

**What to lift.** `proto/rfc/glue/examples/update_pattern.rs` whole,
with the derive in place of its 12-line hand-written `impl Interpolate
for Bar` and `y.bandwidth()` in place of its hard-coded bar height 40.
The lifecycle trace of `proto/rfc/d3/examples/marks_trace.rs` is Phase
30's core test; its instants are reused here.

##### Figure-level locale tests

`figure/tests/main/locale.rs` uses the public figure API only. Labels
are read back from the drawing list: every `GlyphRun` keeps its source
text (`run.text`), so a helper collects the texts of one axis's runs in
order. Domains are set with `fig.scale(…)` and tick counts with
`.ticks(n)`, so layout cannot change the tick set. A figure applies its
locale to the formats it creates; a user's own `.format(|v| …)` stays
the user's (see Formats, time and color), and the default time ladder
is ISO numeric in every locale.

**Decisions this phase may meet**

- `Ticks::labels` for time scales needs a zone the core axis does not
  pass — pass the app's `tz`; record it as a decision.
- A use-case test fails because of core behavior (brush, zoom,
  animation) — change core in this phase; when the change touches a
  public item built in Phases 1–10, record it as a decision and update
  every call site in figure, iced and the dev crates in the same
  commit, with `check.sh quick` green.

#### Steps

1. **Use case 1 app** (`src/focus_context.rs`,
   `examples/focus_context.rs`, `src/lib.rs`): port as described.
   Verify: `cargo build -p rustd3-examples --examples`.
2. **Use case 1 tests** (`tests/main/main.rs`,
   `tests/main/focus_context.rs`). Add the `[[test]] name = "main"`
   stanza to `dev/examples/Cargo.toml` unless an earlier phase already
   wrote an integration test there and added it (a target, not a
   dependency change).
   Window size `(900.0, 430.0)`; the overview starts at y = 340. Each
   test builds `Simulator::with_size(Settings::default(), SIZE,
   app.view())`, sends events, applies `into_messages()` in order to
   `app`, then asserts on `app.visible()`:
   - `wheel_over_the_focus_halves_the_window_around_the_cursor`:
     `point_at((450, 150))`, `scroll(Lines { x: 0, y: 20 })` → k =
     2^(20·0.05) = 2 about range x = 410.
   - `brushing_the_overview_sets_the_window`: at y = 380, press at x =
     300, move to 520, release at 520 (one `simulate` each, no `update`
     in between: iced may deliver several events per view). The last
     message wins; without the guard it is `Show(full)`.
   - `after_a_zoom_dragging_the_band_moves_the_window`: zoom as in the
     first test, apply the messages, rebuild the simulator from the new
     view, press inside the band (x = 40 + 420), move by +86 px,
     release: the window moves by 86/860 of 719 h and keeps its span,
     which proves the brush picked its selection up from `visible`.
   Verify on both backends; revert the guard once by hand, see the
   second test fail, and state it in the commit body.
3. **Use case 2 app and tests** (`src/update_pattern.rs`,
   `examples/update_pattern.rs`, `tests/main/update_pattern.rs`), with
   `t0` an arbitrary `Instant` and `ms(n) = t0 + n ms`:
   - `first_update_grows_five_bars_from_zero_width`: `App::new(t0)`; at
     `t0` every width is 0; at `ms(830)` the widths and ys of the spot
     checks and `marks.is_animating(ms(830)) == false`.
   - `second_update_exits_two_and_enters_two`: `NewData(ms(1500))`; at
     `ms(1500)` gnu and cat are `Phase::Exiting`, bee and eel enter
     with width 0, hen retargets from y 102.941 towards 4.902 without a
     jump (its value at `ms(1500)` equals its value just before);
     `Frame(ms(2330))` prunes the exits and five `Present` marks
     remain.
   - `chart_renders_on_each_backend`: renders `Chart` once through the
     simulator; `ui.snapshot(&Theme::Light)` is `Ok`. No image is
     compared.
4. **de-DE figure tests** (`figure/tests/main/locale.rs`,
   `figure/tests/main/main.rs`):
   - `default_tick_labels_follow_the_figure_locale`: a panel with
     `left(y).ticks(6)` on `fig.scale(scale::linear().domain([0.0,
     12000.0]))`; the neutral default gives `0, 2 000, …, 12 000`
     (U+202F), `fig.locale(Locale::EN_US)` gives `2,000 …`,
     `Locale::DE_DE` gives `2.000 … 12.000`.
   - `fractions_use_the_locale_decimal`: domain `[0, 1]`, `.ticks(5)`:
     DE `0,0 0,2 0,4 0,6 0,8 1,0`.
   - `negative_labels_use_the_unicode_minus`: domain `[-1, 1]`: DE
     `−1,0 … 1,0` with U+2212.
   - `a_user_format_is_not_localized`: `.format(|v| format!("{v:.1}"))`
     keeps `0.5` under `DE_DE`.
   - `time_axes_keep_the_iso_ladder_under_de_de`: domain
     `2026-10-24T22:00Z … 2026-10-25T23:00Z`, zone from
     `testkit::zone("Europe/Berlin")`, labels `10-25 03:00 … 21:00
     10-26` under both `Locale::default()` and `Locale::DE_DE`.
5. **Measure and record** (`.claude/decisions/phase-32.md`): count the
   lines of both apps without tests, imports, data generation and
   `main` (rustfmt defaults). Part I quotes 247 lines for use case 1
   (tests excluded) and 133 for use case 2's core; record the measured
   counts and what differs (the derive removes `Bar`'s 12-line impl).

#### Exit criteria

1. `cargo build -p rustd3-examples --examples` builds `focus_context`
   and `update_pattern` (`cargo run` them only where a display exists;
   the tests are the gate).
2. `ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-examples` and
   `ICED_TEST_BACKEND=wgpu cargo test -p rustd3-examples` each pass the
   five tests `wheel_over_the_focus_halves_the_window_around_the_cursor`,
   `brushing_the_overview_sets_the_window`,
   `after_a_zoom_dragging_the_band_moves_the_window`,
   `first_update_grows_five_bars_from_zero_width`,
   `second_update_exits_two_and_enters_two`, plus
   `chart_renders_on_each_backend` (`RUSTD3_ALLOW_NO_GPU=1` skips wgpu
   with a message).
3. Reverting the `if !brush.is_active()` guard makes
   `brushing_the_overview_sets_the_window` fail (checked by hand once;
   Step 2's commit body states it).
4. `cargo test -p rustd3-figure --test main -- locale` passes the five
   de-DE tests.
5. `cargo run -p rustd3-harness -- --check` reports unchanged counts
   (this phase touches no core code unless a decision above did, which
   its commit body then explains).
6. `decisions/phase-32.md` holds the measured line counts.
7. `scripts/check.sh quick` green on every commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| Wheel `Lines { y: 20 }` at canvas (450, 150) | k 1 → 2; visible `[start + 171.390 h, start + 530.890 h]`, span 359.5 h (± 1 s) | d3 `defaultWheelDelta`; 205 and 635 of 860 px |
| Overview drag 300 → 520 px | window `[start + 217.372 h, start + 401.302 h]`, span 183.930 h = 7.664 days (± 1 min) | Use case 1's "about 7.7 days"; 260 and 480 of 860 px |
| Same, guard removed | final message `Show(full)`, span 719 h | Use case 1: snapped back to all 30 days |
| Band dragged +86 px after the zoom | window shifts by 71.9 h, span stays 359.5 h | 86/860 × 719 h |
| Use case 2, first data (seed 1) | gnu 387.381, dog 352.442, hen 278.094, cat 252.121, fox 184.759 px wide | LCG replayed in Python |
| Same, band ys | 4.902, 53.922, 102.941, 151.961, 200.980; bandwidth 44.118 | d3 band: step 250/5.1 |
| Second data | hen 490.423, fox 418.467, bee 319.623, eel 295.345, dog 211.189; gnu and cat exit | LCG |
| Last entering bar settles | `ms(750 + 4·20) = ms(830)` | `Timing` stagger |
| Core trace (100 ms, 20 ms stagger, new data at 60 ms) | `t=110 a=3.7(exit) b=22.6 c=3.2`; after the prune at 180 ms `b=40.0 c=30.0` | Use case 2's lifecycle trace |
| DE `,.0f` ticks of `[0, 12000]` | `0 1.000 2.000 … 12.000` (step 1000 at d3's default count; `.ticks(6)` gives step 2000) | d3-format de-DE in Node |
| DE `,.1f` of 1234567.5 / `$,.2f` of 1234.5 | `1.234.567,5` / `1.234,50 €` (U+00A0 before €) | Formats, time and color; d3 in Node |
| DE `,.1f` of −0.5 | `−0,5` (U+2212) | d3 in Node |
| Neutral ticks 0–12000 | `2 000 … 12 000` (U+202F) | Formats, time and color |
| DE `%c` of 2026-10-25T09:05Z (UTC) | `Sonntag, der 25. Oktober 2026, 09:05:00` | d3-time-format de-DE in Node |
| Berlin day, 8 ticks, any locale | `10-25 03:00 06:00 … 21:00 10-26`, interval `Hour(3)` | Formats, time and color |

#### Not in this phase

- The figure-level focus and context (a `Plot` with a brush panel):
  Phase 36.
- `Plot`, figures or typst text inside the use cases: they stay core
  level by design.
- Image comparisons of the apps: pixel tests live in rustd3-iced's
  headless harness, and `iced_test`'s exact-match snapshots are never
  used.
- Chained or named transitions (Use case 2: what strains).

### Phase 33 — Figure handles breadth

*M4 Complete figure · builds on phases 17, 20, 21, 23, 25, 28 · about 1,300–1,800 lines, 5 commits · two sessions: cut after Step 3*

#### Goal

Every core scale kind the figure layer needs can be a figure handle:
band and point scales (Phase 14's, plus the out-of-domain check and
Part I's band rows), time scales (`time::Time`, zoned or UTC: positions
over `Timestamp`, fitted and padded in i128 ns), the sequential and
diverging color scales and `Linear<Rgb>`, and the binned color scales
quantize, quantile and threshold. Each new kind implements Phase 14's
`Fittable` except `Quantile`, which a `compile_fail` doctest pins. Figure
axes take `.rotate(deg)`, and layout and both collision passes measure
rotated boxes. The remaining rows of "Types catch the common mistakes"
(see Figures) become `compile_fail` doctests with passing twins:
timestamps on a linear handle (E0271), an `on_change` message of the
wrong domain type (E0631), a fitted quantile handle (E0277), and any
row no earlier phase wrote.

#### Builds on

- Phase 13: label ink boxes; `GlyphRun::transform`.
- Phase 14: `Figure`, `Figure::panel(row, col)`, `ScaleRef<S>`, the
  object-safe `figure::scale::Scale` delegating gestures to core
  (`pan_px`, `zoom_px` return `None` for scales without them); `Kind`
  (`Domain`, `Output`, `Span`, `admits`, `merge`, `fitted`, `domain()`),
  `Position` (`position`, `bandwidth`), `Paint` (`paint`, `slot`);
  `Fittable: Kind` bounding `fit` and `fit_with`; `Fit<Span>`; `Fits`
  (an extent plus sticky values per handle); 5% margins; the `Band<K>`,
  `Point<K>` and `Ordinal<K, Rgb>` impls; `Held` (`hold`, `release`,
  `get`, `prune`). Read `figure/src/scale.rs` and `figure/src/scale/*`
  first: this phase adds impls, not traits.
- Phase 16: tick counts from length (80/35 px per tick, ≤ 4 passes),
  the protrusion layout, both collision passes (parity anchored to the
  absolute tick index), and its band-label spot row.
- Phase 17: `fig.check`, the figure `Locale`, part hashes,
  `Fit::Visible`.
- Phase 20: `Plot::on_change(handle, f)` with `f: impl Fn([S::Domain;
  2]) -> Message`, widget-owned domains, reset through `Held::release`.
- Phase 21: `Fit::Window(span)` quantized to whole device columns.
- Phase 23: `scale::{Sequential, Diverging}` (`Arc<dyn Fn + Send +
  Sync>` interpolators, a transform parameter), `Ticks` for them and
  for `Linear<Rgb>`, `scheme::*`.
- Phase 25: `scale::{Quantize, Quantile, Threshold}` and their `Ticks`
  (`Domain = f64`, `Step = f64`).
- Phase 27: `time::Time` from `time::time(zone)` and `time::utc()`; its
  `Rescale`, `pan_px`/`zoom_px` and `scale::Pad` in i128 ns, with the
  empty-data and flat-extent rules; `Recording<Timestamp>`.
- Phase 28: `Ticks for time::Time` with `Step = time::Interval`, the
  ISO ladder, the time half of `Locale`.
- Decision files: `phase-14.md`, `phase-16.md`, `phase-17.md`,
  `phase-20.md`, `phase-21.md`, `phase-27.md`, `phase-28.md`.

#### Design

##### Modules

```text
figure/src/scale/{band,point}.rs   Phase 14's impls + out-of-domain keys
figure/src/scale/time.rs           time::Time
figure/src/scale/color.rs          Sequential<Rgb, T>, Diverging<Rgb, T>, Linear<Rgb>
figure/src/scale/binned.rs         Quantize<Rgb>, Quantile<Rgb>, Threshold<Rgb>; the Quantile doctest
figure/src/scale/fittable.rs       Fittable impls for the kinds above (the trait is Phase 14's)
figure/src/layout/obb.rs           layout::Obb, the oriented-box test
figure/src/check.rs                out-of-domain band and point keys
```

##### The handles

| Scale | `Kind::Domain` | `Output` | `Span` | Capability | `Fittable` | Fitting |
| --- | --- | --- | --- | --- | --- | --- |
| `Band<K>`, `Point<K>` | `K` | `f64` | `()` | `Position` (`bandwidth` > 0 for band, 0 for point) | yes (Phase 14) | none: the keys are the domain |
| `time::Time` | `Timestamp` | `f64` | `jiff::SignedDuration` | `Position`; pixel operations through core `Rescale` | yes | min/max in i128 ns, `Pad` 5% |
| `Sequential<Rgb, T>`, `Linear<Rgb>` | `f64` | `Rgb` | `()` | `Paint` | yes | data min/max, no margin |
| `Diverging<Rgb, T>` | `f64` | `Rgb` | `()` | `Paint` | yes | ends to data min/max, midpoint kept |
| `Quantize<Rgb>` | `f64` | `Rgb` | `()` | `Paint` | yes | data min/max, no margin |
| `Quantile<Rgb>` | `f64` | `Rgb` | `()` | `Paint` | **no** | the user's sample, through `fig.scale` |
| `Threshold<Rgb>` | `f64` | `Rgb` | `()` | `Paint` | yes | none: the thresholds are the domain |

Bounds follow `proto/figmodel/src/plot/scales.rs` (lift the impl shape
of `Band<K>` and `Ordinal<K, Rgb>`, not its signatures): `K: PartialEq
+ Clone + 'static` for positions and paint; axes additionally need core
`Ticks`, which band implements only for `K: Display` — the E0277 row of
the types table. Every handle's object-safe `Scale` impl delegates
gestures to core: continuous and time scales implement `pan_px` and
`zoom_px(anchor, factor)` through core `Rescale`; band, point and color
scales return `None` from both and ignore gestures. Reset is
`Held::release`; widget state stays keyed by (handle index, `TypeId`).
`Kind::domain()` is `Some` for time and for the continuous color
scales' end stops, `None` for band, point and binned scales.

##### Fitting

- **Time.** Merge `[min, max]` as `Timestamp`s in their own order; pad
  with core's `scale::Pad` for time (i128 ns from the domain start;
  no `Timestamp` passes through `f64`), sticky values as for numbers.
  Empty data gives `[UNIX_EPOCH, UNIX_EPOCH + 24 h]` (matplotlib's
  empty date axis, Phase 27's rule). A single timestamp follows Phase
  27's flat rule (see Decisions). The figure never pads on its own; it
  delegates to `Pad`. `Fit::Window(span)` takes a `SignedDuration` and
  quantizes it to the device-column grid in i128 ns — Phase 21's rule
  applied to time (no margin on x, span over the widest panel sharing
  the handle, `All` on y). `Fit::Visible` reads the M4 index of a
  `Recording<Timestamp>`.
- **The series bridge and `Invert` for time.** `time::Time` implements
  Phase 14's `Indexed<Timestamp>` (its `Position::indexed` returns
  `Some`) and `Invert`, so lines over `Recording<Timestamp>` views thin
  (edges are `Timestamp`s from `invert`, in i128 ns), fit, and follow
  `Fit::Visible` and `Fit::Window` exactly as numeric ones do, and
  brushes (Phase 36) accept time handles.
- **Label parity for time ticks.** Phase 16's absolute tick index for a
  time axis is the number of whole tick intervals from the Unix epoch
  floored in the axis's zone (`interval.count(interval.floor(epoch),
  tick)`, Phase 27's `Interval::count`), so the kept labels stay the
  same instants while a time axis pans.
- **Color scales** do not pad: matplotlib's `Normalize.autoscale_None`
  uses the data extent, and a padded color domain wastes ramp. A flat
  extent widens by Phase 14's `nonsingular` (0.05).
- **Quantile.** `Quantile<Rgb>` implements `Kind` (`Span = ()`,
  `domain() -> None`) and `Paint`, but not `Fittable`, so `fig.fit` and
  `fig.fit_with` reject it at compile time (E0277). d3's quantile domain
  is the sample itself; fitting it would need marks to report every
  value, where `Fits` keeps an extent and sticky values per handle and
  never collects samples. Users build the domain from their data:
  `fig.scale(scale::quantile(sample, colors))`.
- **Band and point.** `fig.fit(band)` behaves as `fig.scale(band)`.
  Data keys outside the domain draw nothing (`apply` returns `None`),
  and `fig.check` reports them once per handle with a count. Growing the
  domain from data in first-seen order was weighed: it would make key
  order depend on data order.
- **Threshold.** `fig.fit(threshold)` behaves as `fig.scale(threshold)`.

##### Ticks at the figure level

Every handle labels through core `Ticks` with the figure `Locale`: time
with `Step = time::Interval` and the ISO ladder; binned scales tick at
their tick sets (colorbars consume them in Phase 35); band and point
ticks are their keys, thinned by Phase 16's parity pass.

##### Rotated tick labels

```rust
impl<'a, S> figure::axis::Axis<'a, S> {
    /// Rotates tick labels counter-clockwise by `deg` (matplotlib's sign).
    /// Labels anchor at their end for positive angles on bottom axes
    /// (matplotlib's `autofmt_xdate`: 30°, `ha="right"`), at their start
    /// for negative ones; top axes mirror; left and right axes rotate
    /// about the label's center.
    pub fn rotate(self, deg: f64) -> Self;
}
```

Geometry, with `w × h` the label's ink box (Phase 13) and θ the angle:

- **Reach.** Perpendicular to a bottom or top axis (what the protrusion
  layout reads): `w·|sin θ| + h·|cos θ|`; along the axis: `w·|cos θ| +
  h·|sin θ|`. Left and right axes swap the two.
- **Same-axis collisions.** Two labels of one axis at the same angle,
  centers `d` apart along the axis, with `u = (cos θ, −sin θ)` and `n =
  (sin θ, cos θ)` in y-down coordinates: two congruent boxes are
  disjoint iff `d·|cos θ| ≥ w` or `d·|sin θ| ≥ h` (the separating-axis
  test on their own two axes). Phase 16's parity pass keeps choosing
  "the smallest k with no overlap", with this predicate on adjacent kept
  labels (max `w` and `h` over the pair).
- **Cross-axis collisions** (larger panels first) test oriented boxes
  with the general four-axis SAT, implemented once in `layout::Obb`
  (`figure/src/layout/obb.rs`).
- Tick counts from length keep their 80/35 px spacing; rotation changes
  only label boxes.

Rotation does not change typst shaping: a run is shaped once, and
`GlyphRun::transform` carries the rotation about the anchor point.

##### Compile-fail doctests

The types table of Figures has six rows. Each becomes a
`compile_fail` doctest with a passing twin; stable rustdoc does not
check the error code, so the twin is what shows the failure is the
intended one.

E0271, in `figure/src/mark/dots.rs`:

````rust
/// ```compile_fail
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::scale;
/// let mut fig = Figure::new();
/// let x = fig.fit(scale::linear());
/// let y = fig.fit(scale::linear());
/// let t = rustd3_core::jiff::Timestamp::UNIX_EPOCH;
/// fig.panel(0, 0).mark(dots([(t, 1.0)], x, y)); // E0271
/// ```
/// ```
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::{scale, time};
/// let mut fig = Figure::new();
/// let x = fig.fit(time::utc());
/// let y = fig.fit(scale::linear());
/// let t = rustd3_core::jiff::Timestamp::UNIX_EPOCH;
/// fig.panel(0, 0).mark(dots([(t, 1.0)], x, y));
/// ```
````

E0631, in `iced/src/plot.rs` (feature `figure`): `Plot::new(fig)
.on_change(x_time, Msg::Range)` with `enum Msg { Range([f64; 2]) }`
fails; the twin declares `Range([rustd3_core::jiff::Timestamp; 2])`
(figure and rustd3-iced reach jiff through core's re-export).

E0277 (not `Fittable`), in `figure/src/scale/binned.rs`, written with
Phase 25's `quantile(domain, range)` builder:

````rust
/// ```compile_fail
/// # use rustd3_figure::Figure;
/// # use rustd3_core::{scale, scheme};
/// let mut fig = Figure::new();
/// let c = fig.fit(scale::quantile([1.0, 2.0, 3.0, 10.0], scheme::CATEGORY10)); // E0277
/// ```
/// ```
/// # use rustd3_figure::Figure;
/// # use rustd3_core::{scale, scheme};
/// let mut fig = Figure::new();
/// let c = fig.scale(scale::quantile([1.0, 2.0, 3.0, 10.0], scheme::CATEGORY10));
/// ```
````

The other three rows: E0277 for an axis on a band whose key lacks
`Display` (Phase 14 wrote it); E0308 for `.fill(color, Wheel::FL)` with
a palette over `Source`; E0277 for an ordinal handle used as a position.
Step 5 greps for each and adds what is missing. Part I writes the
ordinal row as `histogram(bins, speed, color)`; `histogram` arrives in
Phase 34, which adds that form, so here the row uses `dots(…, x,
ordinal_handle)`, which fails with the same `Ordinal<Source, Rgb>:
Position` error.

**Decisions this phase may meet**

- A flat time domain. Phase 27's `Pad for Time` widens a single
  timestamp by about two years on each side, as matplotlib's
  `DateLocator.nonsingular` does (`2026-10-25T12:00` with 5% margins
  spans roughly `2024-08 … 2029-01`), which is useless for sensor data —
  recommended: ±1 hour, so the ladder shows `%H:%M`, changed in core's
  `Pad for Time` (`core/src/time/scale/pad.rs`) rather than special-cased
  in the figure, which keeps delegating padding; record it as a
  decision. `Pad for Time` is a public item of Phase 27, so the rule for
  items of Phases 1–10 does not apply.
- Diverging data entirely on one side of the midpoint (`[2, 5]` over
  midpoint 0) — recommended: keep the midpoint and never let an end
  cross it, domain `[min(lo, mid), mid, max(hi, mid)]` (here `[0, 0,
  5]`), so the ramp never reverses; record it as a decision.
- Phase 14's `Kind` cannot express "no fitting" for band and threshold
  — add a defaulted method rather than a new trait; record it as a
  decision.

#### Steps

1. **Band and point handles** (`scale/{band,point}.rs`, `check.rs`,
   tests in `tests/main/handles.rs`): Phase 14's impls plus the
   out-of-domain report; dodging through the ordinal palette's `slot`
   is unchanged. Verify the band spot checks, including Phase 16's
   label-count row with rotation off.
2. **Time handles** (`scale/time.rs`, `scale/fittable.rs`): `Kind` with
   `Span = SignedDuration`, `Position`, pixel operations, fitting with
   `All`, `Window` and `Visible`, ticks with the figure locale, the
   `Indexed` and `Invert` impls, time label parity. Tests:
   `fitting_two_timestamps_a_microsecond_apart_pads_by_fifty_nanoseconds`,
   the Berlin labels through a figure axis,
   `time_lines_thin_through_the_bridge` (a 1M-sample
   `Recording<Timestamp>` at 10 kHz thins to at most 4 points per
   device column, equal to the `f64` recording of the same samples
   shifted to epoch seconds),
   `time_ticks_keep_parity_while_panning` (an hourly axis over a Berlin
   day panned by 37 minutes keeps labels on the same hours), and, in
   rustd3-iced's tests, `wheel_zoom_on_a_time_handle_publishes_timestamps`
   (one wheel line over the cursor: `on_change` receives a
   `[Timestamp; 2]` whose span shrank by 2^0.05 about the cursor's
   instant, to 1 ns).
3. **Color and binned handles** (`scale/color.rs`, `scale/binned.rs`,
   `scale/fittable.rs`): impls and fitting; `Quantile` without
   `Fittable` and its doctest pair. Test `Paint::paint` through resolve
   (continuous color reaches marks through `fill_by`, Phase 34).
   **Cut here** if the phase takes two sessions; the handoff records
   "Step 3 done".
4. **Rotation** (`axis.rs`, `axis/*`, `layout/*`, `layout/obb.rs`,
   tests in `tests/main/rotate.rs`): `.rotate`, rotated reaches, the
   SAT predicate in both collision passes.
5. **Compile-fail rows** (`mark/dots.rs`, `iced/src/plot.rs`, any file
   that hosts a missing row): the E0271 and E0631 doctests and twins;
   grep for the E0308, ordinal-as-position and band-`Display` rows and
   add any that is missing.

#### Exit criteria

1. `cargo test -p rustd3-figure --test main -- handles` passes: one test
   per scale kind (fit, resolve, position or paint, ticks), including
   the time-precision test.
2. `cargo test -p rustd3-figure --test main -- rotate` passes: rotated
   extents, the spacing rule, end anchoring, no overlap after the
   collision passes.
3. `cargo test -p rustd3-figure --doc` and `cargo test -p rustd3-iced
   --features figure --doc` run the E0271, E0631 and quantile
   `compile_fail` doctests and their twins.
4. `grep -rn --after-context=1 'compile_fail' figure/src iced/src` shows a
   `compile_fail` doctest with a passing twin for each of the six rows
   of the types table.
5. `TZ=Asia/Kathmandu cargo test -p rustd3-figure` passes (zone
   independence); `fig.check` on the trace and scatter examples stays
   empty.
6. `cargo run -p rustd3-harness -- --check` unchanged, unless the
   flat-time decision changed core.
7. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| `band(["a","b","c"]).range([0, 300]).padding(0.1)` via a handle resolved to a 300-px panel | `a` at 9.677…, bandwidth 87.10; `"z"` draws nothing and `fig.check` reports 1 unknown key | Scales; d3 band: step 300/3.1 |
| Band of 20 keys `"0–10"` … `"190–200"`, `padding(0.1)`, bottom axis at plot widths 320 / 480 / 960 px, rotation off | strides 4, 3, 2: keeps 5, 7, 10 labels (typst-measured at notebook tick size) | Phase 16's spot row (use `decisions/phase-16.md`'s counts if it recorded others) |
| Time handle fitted to `2026-10-24T22:00Z` and `2026-10-25T23:00Z` | `[2026-10-24T20:45Z, 2026-10-26T00:15Z]` exactly | 5% of 25 h = 75 min; matplotlib gives the same |
| Time handle fitted to `t` and `t + 1 µs` (t in 2026) | `[t − 50 ns, t + 1 µs + 50 ns]` exactly | i128 ns (an f64 epoch loses ~240 ns) |
| Time handle, no data | `[1970-01-01T00:00Z, 1970-01-02T00:00Z]` | matplotlib's empty date axis |
| Berlin day on an 800-px time axis, 8 ticks | `10-25 03:00 06:00 09:00 12:00 15:00 18:00 21:00 10-26`, `Hour(3)` | Formats, time and color |
| `sequential([0, 80], viridis)` | 0 → #440154, 20 → #3b528b, 40 → #21918c, 80 → #fde725, 100 → #fde725 | d3 in Node |
| Sequential handle fitted to values `[3, 7]` | domain `[3, 7]`; 5 → #21918c | no margin on color |
| `diverging(RdBu)` over `[−1, 0, 1]` | −1 → rgb(103, 0, 31), 0 → rgb(242, 239, 238), 0.5 → rgb(107, 172, 208) | d3 in Node |
| `quantize([0, 100], blues5)`, `blues5` = d3's `schemeBlues[5]` as a literal array (core has no k-color schemes) | thresholds 20, 40, 60, 80; 10 → #eff3ff, 30 → #bdd7e7, 99 → #08519c | d3 in Node |
| `threshold([0, 1], [red, white, blue])` | −1 red, 0 white, 0.5 white, 1 blue | d3 in Node |
| `fig.fit(scale::quantile(sample, colors))` | does not compile (E0277, `Quantile<Rgb>: Fittable`); `fig.scale(…)` twin compiles | types table |
| Label box 60 × 15 px at 0° / 30° / 45° / 90° | min spacing 60 / 30 / 21.21 / 15 px; reach below the axis 15 / 42.99 / 53.03 / 60 px | SAT formula above |
| Rotation +30° on a bottom axis | label end at the tick, text rising left to right | matplotlib `autofmt_xdate` |
| `dots([(Timestamp, f64)], linear_x, y)` | does not compile (E0271); twin with a time handle compiles | types table |
| `on_change(time_x, Msg::Range)` with `Range([f64; 2])` | does not compile (E0631); twin with `[Timestamp; 2]` compiles | types table |

#### Not in this phase

- Colorbars (Phase 35) and the continuous-color channels `fill_by` and
  `size_by` (Phase 34): this phase makes the handles only.
- Brushes across handles and their E0271 case (`brush::x(full).sets(focus)`
  with different domain types): Phase 36.
- Polar handles (Later); log or symlog color handles beyond what
  `Sequential<Rgb, T>` with a transform already gives.
- Axis-drag zoom and box zoom (out of scope).

### Phase 34 — Figure marks breadth

*M4 Complete figure · builds on phases 22 (its code), 29, 33 · about 2,500–3,200 lines, 7 commits · two sessions: cut after Step 4*

#### Goal

The figure layer gets the rest of Part I's marks: areas; ribbons with
explicit lower and upper edges, from owned rows or from two channels of
a recording thinned together; bars from a base and bars over explicit
spans; a general `rect`; histograms over keys and over numbers;
`density_y`/`density_x` with a Gaussian KDE (Scott's rule); `.dodge()`
and `.jitter()` on dots; per-point color and size channels (`.fill_by`,
`.size_by`); an image mark for raster heatmaps; and text notes. Marks
stay per-mark: stacking belongs to the helpers (Phases 38–39), which
compute baselines with core `shape::stack` and emit these
explicit-baseline marks (see Figures: stacking is computed by helpers).
Core gains `series::m4_joint` for ribbons. With the marks, the worked
example of Figures (four wheels) is built verbatim in `rustd3-examples`,
checked and added to the benchmark as a workload, and the label-churn
scenario reruns with real time formats, appended to the benchmark
verdict (and closing it when Phase 31 is skipped or done).

#### Builds on

- Phase 9: `Series<'a, X>` views, `series::Columns`, `m4::Lines`,
  `Series::m4`, `Series::extent`, `series::Error<X>`, NaN-free runs per
  channel and the M4 index; its test channel `a` (the joint rows add a
  second channel `b`, defined in the spot checks below).
- Phase 12: `Command::Fill`, `Command::Markers` with optional per-point
  sizes and paint indices into a paint table, `Command::Image`.
- Phase 13: typst `Text`.
- Phase 14: `Mark<'a>` (`fit`, `draw`), `Fits::add`/`Fits::stick`
  (sticky edges), `Paint::slot`, `mark::Ctx`.
- Phase 15: `dots`, `line`, `rule_x`, `rule_y`, `.fill(palette, key)`,
  `.stroke(…)`, `.dash`, `Thin::{Auto, Off}`, thinning at draw time with
  the M4 memo in `figure::Cache`; `draw`, `save_svg`, `save_png`,
  `figures_equal`.
- Phase 17: `Mark::hits`, `fig.check` (owned marks over 100,000
  points), legends.
- Phase 22: `rustd3-bench` (`--headless`, plans, the workloads `trace`,
  `scatter`, `grid16` and `label-churn`), and the benchmark verdict in
  `decisions/phase-22.md`.
- Phase 25: core `array` (`extent`, `quantile`, `variance`,
  `deviation`, `bin` with its threshold rules); `ticks` (Phase 3).
- Phase 29: `shape::stack` (columns in, `[y0, y1]` per point out).
- Phase 33: band, time and color handles (`Paint` for sequential,
  diverging and binned scales).
- Phase 1: `rustd3-bench` depends on `rustd3-examples` (path), so the
  bench reuses the wheel figure.
- Decision files: `phase-12.md`, `phase-15.md`, `phase-22.md`,
  `phase-25.md`, `phase-29.md`, `phase-33.md`.

#### Design

##### Modules

```text
figure/src/mark/area.rs        Area (baseline), Ribbon (y0, y1); area, ribbon, ribbon_series
figure/src/mark/bar.rs         BarY, BarX; bar_y, bar_x, bar_y_spans, bar_x_spans
figure/src/mark/rect.rs        Rect: one rectangle per row in data units
figure/src/mark/histogram.rs   Histogram, mark::Bins, the mark::Binned trait
figure/src/mark/density.rs     DensityY, DensityX
figure/src/mark/image.rs       Image, mark::Grid
figure/src/mark/text.rs        Note
figure/src/mark/channel.rs     per-point fill_by / size_by channels
figure/src/mark/dots.rs        Phase 15's Dots + .dodge, .jitter, .fill_by, .size_by
figure/src/mark/line.rs        Phase 15's Line + .dodge
figure/src/stats.rs, figure/src/stats/kde.rs   stats::Kde
core/src/series/m4.rs, core/src/series/error.rs   m4_joint, Error::Recordings
```

##### Core: `series::m4_joint`

```rust
impl<'a, X: Coordinate> Series<'a, X> {
    /// M4 over two channels of one recording, e.g. a ribbon's lower and
    /// upper edge: both edges keep their extremes in every column.
    pub fn m4_joint(self, upper: Series<'a, X>, columns: &Columns, valid: impl Fn(f64) -> bool)
        -> Result<m4::Lines, Error<X>>;
}

// series::Error<X> gains
#[error("joint query over two recordings: {left:?} and {right:?}")]
Recordings { left: series::Id, right: series::Id },
```

Both views must belong to one recording (else `Error::Recordings`).
The query walks the intersection of both channels' NaN-free runs; per
column `[a, b)` inside a joint run it pushes the sorted, deduplicated
union of `{a, b − 1}` and both channels' extremes (at most 6 indices),
with the outer neighbours inside the same joint run, exactly as `m4`
does for one channel. Columns built for another recording or revision
are the same error as for `m4`. Below four samples per column every
sample is emitted. Adding the variant changes Phase 9's public
`series::Error<X>`, so it is recorded as a decision in
`decisions/phase-34.md` (Conventions, item 7) and the same commit
updates every match on it in figure, iced, examples and bench, with
`check.sh quick` green. The test
`joint_m4_splits_where_either_channel_has_a_gap` goes into
`core/tests/main/series.rs`.

##### Signatures

```rust
pub fn area<'a, X, Y>(data: impl Data<'a, X::Domain, f64>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Area<'a, X, Y>
where X: Position, Y: Position<Domain = f64>;
impl<'a, X, Y> Area<'a, X, Y> {
    pub fn base(self, y0: f64) -> Self;           // default 0.0, held in fitting
    pub fn fill<C: Paint>(self, c: ScaleRef<C>, key: C::Domain) -> Self;
}

/// Rows (x, y0, y1): `shape::stack`'s `[y0, y1]` plugs in directly. Never thinned.
pub fn ribbon<'a, X, Y>(rows: impl IntoIterator<Item = (X::Domain, f64, f64)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Ribbon<'a, X, Y>
where X: Position, Y: Position<Domain = f64>;
/// Two channels of one recording, thinned together through `m4_joint` (`Thin::Auto`).
pub fn ribbon_series<'a, C, X, Y>(lower: Series<'a, C>, upper: Series<'a, C>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Ribbon<'a, X, Y>
where C: series::Coordinate, X: Position<Domain = C>, Y: Position<Domain = f64>;
// Ribbon: .fill(c, key), .thin(Thin)

pub fn bar_y<X, Y>(data: impl IntoIterator<Item = (X::Domain, f64)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> BarY<X, Y>
where X: Position, Y: Position<Domain = f64>;    // vertical bars from .base (default 0, held)
pub fn bar_x<X, Y>(data: impl IntoIterator<Item = (f64, Y::Domain)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> BarX<X, Y>
where X: Position<Domain = f64>, Y: Position;
/// One bar from y0 to y1 per row; both ends enter fitting, neither is held.
pub fn bar_y_spans<X, Y>(rows: impl IntoIterator<Item = (X::Domain, [f64; 2])>, x: ScaleRef<X>, y: ScaleRef<Y>) -> BarY<X, Y>
where X: Position, Y: Position<Domain = f64>;
pub fn bar_x_spans<X, Y>(rows: impl IntoIterator<Item = ([f64; 2], Y::Domain)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> BarX<X, Y>
where X: Position<Domain = f64>, Y: Position;
// BarY, BarX: .fill(c, key), .dodge(), .width(px) for non-band axes, .base(v) (rows from bar_y/bar_x only)

/// Observable Plot's `rect`: one filled rectangle per row, x0..x1 × y0..y1 in data units.
pub fn rect<X, Y>(rows: impl IntoIterator<Item = ([f64; 2], [f64; 2])>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Rect<X, Y>
where X: Position<Domain = f64>, Y: Position<Domain = f64>;
// Rect: .fill(c, key), .stroke(c, key)

pub fn histogram<X, Y>(values: impl IntoIterator<Item = X::Domain>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Histogram<X, Y>
where X: Position + mark::Binned, Y: Position<Domain = f64>;
impl<X, Y> Histogram<X, Y> { pub fn bins(self, b: mark::Bins) -> Self; /* .fill, .dodge */ }

pub fn density_y<Y, W>(values: impl IntoIterator<Item = f64>, y: ScaleRef<Y>, w: ScaleRef<W>) -> DensityY<Y, W>
where Y: Position<Domain = f64>, W: Position<Domain = f64>;
pub fn density_x<X, H>(values: impl IntoIterator<Item = f64>, x: ScaleRef<X>, h: ScaleRef<H>) -> DensityX<X, H>
where X: Position<Domain = f64>, H: Position<Domain = f64>;
// both: .fill(c, key), .bandwidth(h) (explicit h), .adjust(f) (seaborn's bw_adjust)

impl<'a, X: Position, Y: Position> Line<'a, X, Y> {
    /// Vertices at the centre of the stroke key's sub-band (band x only).
    pub fn dodge(self) -> Self;
}

impl<'a, X: Position, Y: Position> Dots<'a, X, Y> {
    pub fn dodge(self) -> Self;
    pub fn jitter(self, share: f64) -> Self;      // of the band; deterministic
    pub fn fill_by<C>(self, c: ScaleRef<C>, values: impl IntoIterator<Item = f64>) -> Self
    where C: Paint<Domain = f64>;
    pub fn size_by<S>(self, s: ScaleRef<S>, values: impl IntoIterator<Item = f64>) -> Self
    where S: Kind<Domain = f64, Output = f64>;    // output = radius in px
}

pub fn image<X, Y, C>(grid: mark::Grid, x: ScaleRef<X>, y: ScaleRef<Y>, color: ScaleRef<C>) -> Image<X, Y, C>
where X: Position<Domain = f64>, Y: Position<Domain = f64>, C: Paint<Domain = f64>;
impl mark::Grid {
    /// Row-major values, row 0 at `y[0]`; errors carry the lengths.
    pub fn new(cols: usize, rows: usize, values: Vec<f64>) -> Result<Self, mark::Error>;
    pub fn extent(self, x: [f64; 2], y: [f64; 2]) -> Self;   // default [0, cols] × [0, rows]
}

pub fn note<X, Y>(at: (X::Domain, Y::Domain), text: impl Into<Text>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Note<X, Y>;
impl<X, Y> Note<X, Y> {
    pub fn anchor(self, a: Anchor) -> Self;
    pub fn offset(self, dx: f64, dy: f64) -> Self;
    pub fn fill(self, c: Rgb) -> Self;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {   // mark::Error
    #[error("a grid of {cols} × {rows} cells got {len} values")]
    GridLength { cols: usize, rows: usize, len: usize },
    #[error("channel has {values} values for {points} points")]
    ChannelLength { points: usize, values: usize },
}
```

##### Rules

- **Held at zero.** Bars from a base, histograms and areas report their
  base (0 or `.base`) as a sticky value, so the 5% margin stops there
  (matplotlib's sticky edges). Dots, lines, ribbons, span bars and
  `rect` are not held: both of their ends enter fitting.
- **Dodge.** `.dodge()` on dots, lines, bars and histograms: the
  mark's fill (or stroke) key's slot among the palette's keys, from
  `Paint::slot`, splits the band into equal sub-bands in legend order
  (see Figures, settled details). A dodged line places each vertex at
  the centre of its key's sub-band instead of the band's centre, which
  is what the wheel figure's dashed regression lines use, so they sit
  on their dots.
- **`mark::Binned`** decides what `histogram` counts: implemented for
  `Band<K>` and `Point<K>` (count per key, the wheel figure's case) and
  for continuous `f64` scales (bin the values); a capability trait in
  `mark/histogram.rs`, implemented by the figure crate for those types
  only. One function keeps Part I's call `histogram(bins, speed,
  count)` and the matplotlib-like numeric call identical, where a second
  function (`histogram_keys`) would split them.
- **`mark::Bins`** for numeric values: `Nice(n)` (default
  `Nice(sturges)`, d3.bin: Sturges' count `⌈log₂ n⌉ + 1`, thresholds
  from `ticks(min, max, count)`, domain niced to the outer ticks, last
  bin closed) and `Count(n)` (n equal-width bins over `[min, max]`,
  numpy's and matplotlib's rule). Non-finite values are dropped.
  Weights, density, cumulative counts, `Auto` and `Edges` are Phase
  38's. The continuous branch draws its bars through `rect`'s private
  rectangle code (one implementation).
- **`rect`.** One `Fill` command per row and one part per mark; all four
  edges enter fitting.
- **Span bars.** `bar_y_spans` rows carry both ends; `.base` applies to
  `bar_y`/`bar_x` rows only, which its doc comment states.
- **Ribbons.** `ribbon_series` stores both views as `Points` and works
  through Phase 14's bridge: it reports both channels' extents to
  fitting with `x.indexed()?.series_extent(…)`, as `line` does for one,
  and thins with `Thin::Auto` through a method this phase adds to
  `Indexed<D>` with a default, so the trait stays compatible:
  `fn thin_joint<'a>(&self, lower: &Points<'a, D, f64>, upper:
  &Points<'a, D, f64>, edges: &[D], valid: &dyn Fn(f64) -> bool) ->
  Option<series::m4::Lines> { None }`. The continuous impl (Phase 14)
  and the time impl (Phase 33) override it with `m4_joint`; the result
  is memoized in `figure::Cache` like a line's M4 with both channels in
  the key. The owned-rows `ribbon` is never thinned.
- **KDE** (`stats::Kde`). `Kde::new(values) -> Option<Kde>` drops
  non-finite values and is `None` when fewer than two remain or their
  variance is zero (the mark then draws nothing and `fig.check` reports
  it). Bandwidth, Scott's rule as scipy and seaborn use it: `h = σ̂ ·
  n^(−1/5) · adjust`, with `σ̂` the sample standard deviation (n − 1).
  `eval(x) = Σ φ((x − xᵢ)/h) / (n·h)`, φ the standard normal density.
  `grid(points, cut)`: `points` evenly spaced over `[min − cut·h, max +
  cut·h]`; `density_y` uses 200 points and `cut = 3` (seaborn's
  `kdeplot`). The curve is over the data, not the view, so zooming does
  not reshape it. `density_y` peak-normalizes to 1 on its width scale
  (the wheel figure's side panels share `width = [0, 1]`), fills at
  0.35 opacity and strokes at 0.75 pt. `proto/figmodel/src/plot/marks.rs`
  (`DensityY`, lines 197–295) uses `1.06·σ_pop·n^−0.2`, which is
  Silverman's rule; do not lift it.
- **Jitter.** Position inside the (dodged) slot: `start + w·(0.5 +
  share·(j(i) − 0.5))`, with `j` the splitmix64 hash of the point index
  mapped to `[0, 1)` (`proto/figmodel/src/plot/marks.rs:48-55`), so the
  output is the same on every run and platform.
- **`fill_by` / `size_by`.** Colors resolve at draw time through the
  handle; the `Markers` command gets a paint table of the distinct
  `Rgb` values (8-bit, first-seen order) and one index per point; sizes
  become per-point radii in px. A point whose value maps to `None` or to
  a non-finite size is skipped together with its position. A channel
  whose length differs from the point count is never truncated: the
  length check runs when `fill_by`/`size_by` is called, the mark keeps
  `mark::Error::ChannelLength { points, values }`, draws nothing, and
  `fig.check` reports the error.
- **Image.** One pixel per cell: an RGBA8 image of `cols × rows` placed
  at the device rectangle of the grid extent, `smooth: false` (nearest,
  as colorbars need), opaque cells unless the color has alpha. Rows are
  flipped so row 0 sits at `y[0]` (data space, y up).
- **Notes** are typst `Text` placed in data coordinates with an anchor
  and a pixel offset; they take part in no collision pass (users place
  them).

##### The wheel figure

`dev/examples/src/wheels.rs` holds Figures' worked example `figure(samples,
inset) -> Figure<'static>`, `trend`, `simulate` (600 drives per wheel,
seed 11, temperature every third drive, RL worn above 100 km/h) and
`figure_with(samples, inset, palette)`, lifted from
`proto/figmodel/examples/wheels_auto.rs` (the figure function, `trend`,
`simulate`, `Display` impls; without `unwrap` and the trace dump) and
`proto/dlist/src/wheels.rs` (`figure_with`). `examples/wheels.rs` writes
SVG and PNG at 2×. The function body stays the text of Figures, modulo
names an earlier phase chose differently (see Decisions).

`dev/fixtures/wheels.py` (committed with `wheels.json`) replays
`simulate()` (seed 11, the LCG of `wheels_auto.rs`) and `trend`, and
writes per-handle extents, the padded domains (5%; dots and lines not
held; `count` held at 0) and the sample, mark and bar counts.
`dev/fixtures/kde.py` (committed with `kde.json`) writes scipy's
`gaussian_kde` and seaborn 0.13.2's `KDE` (`bw_method="scott"`,
gridsize 200, cut 3) values for the spot-check data. Tests read both
JSON files through testkit's tolerant `Obs` compare (±1e-4 where a row
says so).

##### Benchmark

`dev/bench/src/workload/wheels.rs` calls
`rustd3_examples::wheels::{simulate, figure}` and runs the wheel figure
at 960 × 760 under every scenario that applies to a figure without
recordings (hover, pan and zoom on `vib`, resize, scale factors).
`dev/bench/src/workload/label_churn.rs` replaces its stand-in formatter
(`format!("t+{v:.4} s")` with `.ticks(16)`) by a time axis with real
`Ticks for Time` labels (ISO ladder, figure locale) panned across a day
boundary. Rerun it headlessly, compare with Phase 22's report, and
append to the benchmark verdict in `decisions/phase-22.md` whether its
conclusion holds. Then, if the user's run has given the verdict and
Phase 31 is either skipped or done with its rerun appended, write the
verdict's closing line; otherwise leave it open for Phase 31 (Order of
work: the verdict may arrive after this phase). The file is
append-only.

**Decisions this phase may meet**

- The wheel function cannot match the text of Figures because an
  earlier phase named something differently — keep the earlier phase's
  name; record the difference as a decision, naming the lines of the
  worked example it touches.

#### Steps

1. **Areas, ribbons, bars, spans, `rect` and the joint M4**
   (`core/src/series/{m4,error}.rs`, `core/tests/main/series.rs`,
   `mark/{area,bar,rect}.rs`, `tests/main/marks.rs`): `m4_joint` and
   `Error::Recordings` with every match on `series::Error` updated;
   held edges, spans, `rect`, `ribbon_series` thinning. Verify:
   `cargo test -p rustd3-core --test main -- series` and the bar, span
   and `rect` spot checks.
2. **Histogram** (`mark/histogram.rs`): `Binned`, `Bins`, `.dodge()`,
   the continuous branch through the rectangle code; the
   ordinal-as-position row in Part I's form (`histogram(bins, speed,
   color)`, E0277) as a `compile_fail` doctest with its twin.
3. **Density and KDE** (`mark/density.rs`, `stats.rs`, `stats/kde.rs`,
   `dev/fixtures/kde.{py,json}`): run `python3 dev/fixtures/kde.py`
   once and commit its JSON; fixture test through the tolerant format.
4. **Dodge and jitter on dots, dodge on lines** (`mark/dots.rs`,
   `mark/line.rs`; spot row: a line over band keys 0..4 with 3 palette
   keys, stroke key in slot 2, `padding(0.1)` on 300 px → its vertices
   sit at the centres of the third sub-bands). **Cut here** if the
   phase takes two sessions; the handoff records "Step 4 done".
5. **Channels, image, notes** (`mark/{channel,image,text}.rs`,
   `mark/dots.rs`): `fill_by`/`size_by` end to end into
   `Command::Markers`; the length check; an image placement test (a 2 ×
   2 grid becomes a 2 × 2 RGBA image at the device rect).
6. **The wheel figure** (`dev/examples/src/wheels.rs`,
   `dev/examples/examples/wheels.rs`, `dev/examples/tests/main/wheels.rs`
   declared in `tests/main/main.rs`, `dev/fixtures/wheels.{py,json}`):
   counts, domains, checks; SVG and list-dump snapshots (text
   references only).
7. **Benchmark** (`dev/bench/src/workload/{wheels,label_churn}.rs`,
   `.claude/decisions/phase-22.md`): add the workload, rerun label
   churn, append the result and the closing line.

#### Exit criteria

1. `cargo test -p rustd3-figure --test main -- marks` passes the mark
   tests of Steps 1–5 and the KDE fixture test; `cargo test -p
   rustd3-core --test main -- series` passes the joint test.
2. `cargo run -p rustd3-examples --example wheels` writes
   `target/wheels/{wheels,wheels_inset}.{svg,png}`; `cargo test -p
   rustd3-examples -- wheels` passes (counts and domains against
   `wheels.json`, `fig.check` empty at 960 × 760 and 1600 × 1200 under
   the default theme).
3. `cargo test -p rustd3-figure --doc` runs the histogram E0277 doctest
   and its twin.
4. `cargo run -p rustd3-bench -- --headless --workload wheels` and
   `--workload label-churn` run and print their reports (report-only).
5. `decisions/phase-22.md` has the label-churn rerun appended, earlier
   entries untouched, and the closing line when its condition above
   holds.
6. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

Data `X = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2,
7.9, 9.5, 14.0]` (n = 15) for the statistical rows. M4 rows use Phase
9's data: x = 0, 1, …, 9 on `Clock::Regular { start: 0.0, period: 1.0
}`, `a = [0, 5, −1, 3, 1, 2, 2, 8, 1, 4]`, `b = [2, 2, 2, NaN, 2, 2, 2,
2, 2, 2]`.

| Input | Expected | Source |
| --- | --- | --- |
| `Kde::new(X).bandwidth()` | σ̂ = 3.229654, n^(−1/5) = 0.581811, h = 1.879048 | scipy `gaussian_kde` |
| `eval` at 0 / 2 / 4 / 4.4 / 6 / 10 / 14 | 0.034339 / 0.097612 / 0.138075 / 0.136582 / 0.103621 / 0.026567 / 0.015035 | scipy |
| `grid(200, 3)` | `[−4.437143, 19.637143]`, step 0.120976; peak at 4.031199, 0.138086 | seaborn `KDE` |
| Prototype bandwidth on X | 1.924 (Silverman); must not be used | `1.06·σ_pop·n^−0.2` |
| `histogram(X)` on a linear x, default bins | `[0,2) 1, [2,4) 5, [4,6) 5, [6,8) 2, [8,10) 1, [10,12) 0, [12,14) 0, [14,16] 1` | d3.bin in Node |
| `.bins(Bins::Count(10))` | edges 1.2, 2.48, …, 14.0; counts 2 3 4 3 0 1 1 0 0 1 | numpy |
| Bars `[(a, 3)]`, y fitted | domain `[0, 3.15]` (held at 0) | 5% margin, sticky edge |
| Dots `[(a, 3)], [(b, −1)]` | domain `[−1.2, 3.2]` (not held) | 5% margin |
| `bar_y_spans([(a, [1, 3])])`, y fitted | domain `[0.9, 3.1]` (neither end held) | 5% margin |
| `rect([([0, 1], [0, 2]), ([1, 3], [1, 4])], x, y)`, x and y fitted | x `[−0.15, 3.15]`, y `[−0.2, 4.2]`; 2 `Fill` commands, 1 part | 5% margins, nothing held |
| `a.m4_joint(b, edges [0, 5, 10], \|_\| true)` | `[[0, 1, 2], [4, 5, 7, 8, 9]]` (b's NaN at 3 splits the run) | Phase 9's raw-scan reference |
| `m4_joint` over channels of two recordings | `Err(Recordings { left, right })` | rule above |
| Wheel `simulate()` | 5,600 samples; 4 × (600 + 600 + 200) | `dev/fixtures/wheels.py`; Figures: "5,600 dots" |
| Wheel figure marks | 52 (per wheel 3 dots, 3 lines, 3 densities, 3 histograms, 1 rule) | `wheels.py`; Figures: "52 marks" |
| Wheel list | 12 `Markers` commands holding 5,600 points; 232 histogram bars (nonzero key counts) | `wheels.py`; Drawing and output |
| Wheel domains | vib `[−0.08002, 3.05344]`, temp `[31.70122, 65.65070]` (a trend endpoint 33.2444 sits below the data min 33.5096), count `[0, 90.30]` (±1e-4) | `wheels.py` |
| Wheel figure at 960 × 760 and 1600 × 1200, default theme | `fig.check(size)` empty; ≤ 4 layout passes (Themes: 3) | Layout; Text and math |
| Wheel labels | about 65 labels → 66 runs, 371 glyphs, two fonts (record the actual numbers) | Drawing and output |
| Lowering the wheel list's markers | commands grow by 5,600 − 12 (468 → 6,056) | Drawing and output |

#### Not in this phase

- Hatching (`.hatch`), colorbars, legend glyphs, theme presets and the
  talk-context check of the wheel figure: Phase 35.
- Histogram weights, density and cumulative counts, `Bins::Auto` and
  `Edges`, stacked histograms, violins, box plots, ECDF and the other
  statistical helpers: Phase 38. Stacked bars and areas: Phase 39.
- Bar and pie value labels, size and color legends: Phase 39.
- `fill_between` across two recordings, thinning non-monotonic x (out
  of scope).
- Running the benchmark on real hardware (the user does that).

### Phase 35 — Themes, colorbars and hatching

*M4 Complete figure · builds on phases 23, 34 · about 1,900–2,500 lines, 5 commits · two sessions: cut after Step 2*

#### Goal

`Theme` becomes seaborn's split as a typed struct with builders only
(see Themes): five styles, four contexts, a font scale, palettes and
`Domains::Nice`, and legend swatches get glyphs. Continuous and binned
color scales get their legend, the colorbar: an image strip sampled
with nearest filtering, labelled by the scale's own ticks, with
matplotlib's end triangles on request; threshold colorbars get their
geometry here. Area-like marks take hatching built as clipped line
geometry anchored to the figure, and legend swatches repeat it.
Hatching has the lowest priority and comes last: if a session runs
short, hatching is the part that carries over, never colorbars.

#### Builds on

- Phase 7: core's `scheme::{CATEGORY10, OBSERVABLE10, DEEP, COLORBLIND,
  OKABE_ITO}`; the theme palettes re-export these and never redefine
  them.
- Phase 12: `Command::{Image, Push, Stroke, Pop}`, `Clip::{Rect, Path}`.
- Phase 13: typst text for colorbar labels.
- Phase 14: `Theme` (default only so far), sizes in points (1 pt = 4/3
  px), `em`, and `Kind::fitted`, beside which this phase adds `Kind::niced` for `Domains::Nice`.
- Phase 16: tick counts from track length (80/35 px per tick), the
  protrusion layout and its ≤ 4 passes.
- Phase 17: legends in slots and insets (`legend(palette).title(…)`,
  `Slot`, `Inset`; `Figure::legend(slot, Legend<'a, K>)` and
  `Panel::legend(inset, Legend<'a, K>)` taking the concrete type).
- Phase 23: `scale::{Sequential, Diverging}` with `.transform(…)`,
  `scheme::*` with d3's 256-entry viridis-family tables, `Ticks` for
  color scales positioned in [0, 1].
- Phase 25: the binned scales and their `Ticks`; `Ticks for
  Threshold::position` returns `(i + 1)/(n + 1)` for threshold `i`.
- Phase 33: their figure handles (`Paint`).
- Phase 34: area-like marks (`area`, `ribbon`, `bar_y`, `bar_x`, span
  bars, `histogram`, `density_y`, `density_x`), the wheel figure and
  `figure_with(samples, inset, palette)`.
- Decision files: `phase-14.md`, `phase-16.md`, `phase-17.md`,
  `phase-23.md`, `phase-25.md`, `phase-33.md`, `phase-34.md`.

#### Design

##### Themes (`figure/src/theme.rs`, `figure/src/theme/{style,context,palette,domains}.rs`)

```rust
pub struct Theme { /* private: style, context, font_scale, palette: Vec<Rgb>, domains */ }
impl Theme {
    pub fn seaborn(style: theme::Style, context: theme::Context) -> Self;
    /// Copies the colors; an empty slice leaves the theme unchanged.
    pub fn palette(self, colors: impl AsRef<[Rgb]>) -> Self;
    /// Scales text only; a non-finite or non-positive `k` leaves the theme unchanged.
    pub fn font_scale(self, k: f64) -> Self;
    /// `Domains::Margin(m)` with `m` outside [0, 0.5) leaves the theme unchanged.
    pub fn domains(self, d: theme::Domains) -> Self;
    // getters used by layout and marks: tick_text(), label_text(), …
}
impl Default for Theme { /* seaborn(Ticks, Notebook), palette COLORBLIND, Margin(0.05) */ }

pub enum Style { DarkGrid, WhiteGrid, Dark, White, Ticks }
pub enum Context { Paper, Notebook, Talk, Poster }      // 0.8, 1, 1.5, 2
pub enum Domains { Margin(f64), Nice }
pub mod palette {
    /// Core's schemes, re-exported: one definition each.
    pub use rustd3_core::scheme::{CATEGORY10, COLORBLIND, DEEP, OBSERVABLE10, OKABE_ITO};
    /// seaborn palettes core does not carry.
    pub const MUTED: [Rgb; 10]; pub const PASTEL: [Rgb; 10];
    pub const BRIGHT: [Rgb; 10]; pub const DARK: [Rgb; 10];
}
```

`impl AsRef<[Rgb]>` lets `.palette(palette::DEEP)` (an `[Rgb; 10]` by
value) and `.palette(&my_vec)` both compile. Invalid builder input
leaves the theme as it was, documented on each builder: no panic, no
theme error type. `Theme` has no serde derives (theme files are Later).
Lift `Style`, `Context::scale` (0.8/1/1.5/2), `Domains::{Margin, Nice}`
and the size getters from `proto/figmodel/src/plot/theme.rs`; it has
three styles and public fields, so add `White` and `Dark` and make the
fields private. `proto/dlist/src/bin/themes.rs` renders seven theme
cases on the wheel figure (plot-area share, passes, dropped labels).

Values: seaborn 0.13.2 `plotting_context` at notebook, in points; every
size is multiplied by the context factor, text sizes also by
`font_scale`:

| Setting | Value | Setting | Value |
| --- | --- | --- | --- |
| text, axis label, title | 12 | tick label, legend | 11 |
| legend title | 12 | figure title | 14.4 (matplotlib's `figure.titlesize = "large"`, 1.2 × 12; seaborn sets none) |
| axis line width | 1.25 | grid width | 1.0 |
| line width | 1.5 | patch edge width | 1.0 |
| major tick size / width | 6 / 1.25 | minor tick size / width | 4 / 1.0 |
| marker size | 6 | ink (text, ticks, labels) | `.15` = #262626 |

Styles (seaborn `axes_style`):

| Style | Panel | Axis line | Grid | Tick marks |
| --- | --- | --- | --- | --- |
| DarkGrid | #EAEAF2 | none (white) | white | off |
| WhiteGrid | white | `.8` = #CCCCCC | #CCCCCC | off |
| Dark | #EAEAF2 | none (white) | none | off |
| White | white | #262626 | none | off |
| Ticks | white | #262626 | none | on (outward) |

Palettes (first ten colors of seaborn's `color_palette`; DEEP and
COLORBLIND are core's constants, listed for the tests):

```text
DEEP       4c72b0 dd8452 55a868 c44e52 8172b3 937860 da8bc3 8c8c8c ccb974 64b5cd
MUTED      4878d0 ee854a 6acc64 d65f5f 956cb4 8c613c dc7ec0 797979 d5bb67 82c6e2
PASTEL     a1c9f4 ffb482 8de5a1 ff9f9b d0bbff debb9b fab0e4 cfcfcf fffea3 b9f2f0
BRIGHT     023eff ff7c00 1ac938 e8000b 8b2be2 9f4800 f14cc1 a3a3a3 ffc400 00d7ff
DARK       001c7f b1400d 12711c 8c0800 591e71 592f0d a23582 3c3c3c b8850a 006374
COLORBLIND 0173b2 de8f05 029e73 d55e00 cc78bc ca9161 fbafe4 949494 ece133 56b4e9
```

The theme palette colors marks without a fill or stroke key (its first
color) and the color scales helpers build (Phases 38–39); an explicit
`scale::ordinal(keys, colors)` keeps its own colors.

**`Domains::Nice`** rounds fitted continuous domains outward with
core's `nice(count)` instead of adding margins. `count` is the axis's
requested count (`.ticks(n)`) or, without one, the default count Phase
16 derives from the panel's track length at the first layout pass
(80/35 px per tick). Later passes do not nice again: the domain is
settled before the ≤ 4 passes iterate. Time handles floor and ceil the
extent to the ladder interval chosen for that count
(`interval.floor`/`interval.ceil`).

The hook: this phase adds to Phase 14's `Kind` a method with a default,
so no existing implementor changes:

```rust
/// This scale fitted with its ends rounded outward to `count` nice
/// ticks instead of margins. Default: `self.fitted(extent, sticky, 0.0)`.
fn niced(&self, extent: Option<&[Self::Domain; 2]>, sticky: &[Self::Domain], count: usize) -> Self {
    self.fitted(extent, sticky, 0.0)
}
```

Continuous scales override it with core's `nice(count)` over the merged
extent (sticky values still hold), time handles with the interval
floor and ceil above; band, point, ordinal and binned scales keep the
default. Resolve fits every handle as usual, runs layout pass 1, and
when the theme's domains are `Domains::Nice` refits each fitted
continuous or time handle with `niced(count)`, `count` being the
axis's requested count or pass 1's count for its track; the remaining
passes run on those domains. A handle held by a gesture is not
re-niced.

##### Legend glyphs (`figure/src/legend/swatch.rs`)

`legend(palette).glyph(legend::Swatch::Dot)`; `pub enum Swatch {
Square, Dot, Line { dash: Option<[f64; 2]> } }`; the default is
`Square` at 0.7 em, matplotlib's `handleheight` and Phase 17's size (see Figures: swatches are squares, `.glyph(…)`
shows a dot or a dashed line). A legend whose marks are hatched draws
the hatch in each square (Step 5). Row and inset swatches lift from
`proto/figmodel/src/plot/legend.rs`.

##### Colorbars (`figure/src/colorbar.rs`, `figure/src/colorbar/{ramp,extend}.rs`)

```rust
pub fn colorbar<'a, S>(handle: ScaleRef<S>) -> Colorbar<'a, S>
where S: Paint<Domain = f64> + colorbar::Ramp;

impl<'a, S> Colorbar<'a, S> {
    pub fn title(self, t: impl Into<Text>) -> Self;
    pub fn extend(self, e: colorbar::Extend) -> Self;            // default Neither
    pub fn ticks(self, n: usize) -> Self;                        // default from length, as axes
    pub fn format(self, f: impl Fn(f64) -> String + 'a) -> Self; // borrows for 'a, like axes
}

pub enum Extend { Neither, Min, Max, Both }

/// How a color scale becomes a strip. Implemented for Sequential<Rgb, T>
/// and Diverging<Rgb, T> (any transform), Linear<Rgb>, Quantize<Rgb>,
/// Quantile<Rgb> and Threshold<Rgb>. Not for Ordinal: `colorbar(ordinal)`
/// is E0277 (categories get `legend`).
pub trait Ramp {
    fn samples(&self) -> colorbar::Samples;
}
pub enum Samples {
    /// Equal rows in transformed space: 256 for interpolators,
    /// one per entry for d3's 256-entry tables.
    Even(Vec<Rgb>),
    /// One row per bin, with the bin edges in domain units.
    Bins { colors: Vec<Rgb>, edges: Vec<f64> },
}

// figure/src/legend.rs: anything a slot or an inset holds
pub trait Guide<'a> { /* measure and draw; implemented by Legend and Colorbar */ }
impl<'a> Figure<'a> { pub fn legend(&mut self, slot: Slot, g: impl legend::Guide<'a>) -> &mut Self; }
impl<'a> Panel<'a> { pub fn legend(&mut self, at: Inset, g: impl legend::Guide<'a>) -> &mut Self; }
```

A colorbar is a legend value: `fig.legend(Slot::Right,
colorbar(heat).title("temp [°C]"))` or `panel.legend(Inset::TopRight,
…)`. Phase 17's entry points took the concrete `Legend<'a, K>`; they
take `impl legend::Guide<'a>`, implemented by `Legend<'a, K>` and
`Colorbar<'a, S>`, so existing calls compile unchanged.

Geometry:

- **Size.** Thickness `em(1)` (the tick-label size). Length: a side slot
  spans the plot areas of the panels beside it (first to last row), a
  top or bottom slot the columns; an inset is 40% of the panel's long
  side.
- **Strip.** One `Command::Image`, `smooth: false`, `N × 1` (or `1 ×
  N`) pixels stretched to the bar. With `Samples::Even`, row `i` is
  `interp((i + 0.5)/N)` in transformed space, so for a table scheme `N =
  256` reproduces the table exactly: d3's ramp is `table[⌊t·256⌋]`, and
  d3's viridis and magma tables equal matplotlib's 256-entry colormaps
  byte for byte. Binned scales draw one row per bin at the bins'
  transformed widths; unequal bins become groups of image rows, still
  one image. Part I's measurements (see Figures): the strip equals the
  scale at every pixel of a 300 px bar for viridis, magma, turbo and
  RdBu, a 256-stop linear gradient is off by 1.2–3.5/255, and
  rectangles per bin leave seams in some PDF viewers.
- **Bin widths.** Quantize rows are equal in domain units; quantile
  rows follow the sample quantiles (unequal). A threshold scale with `n`
  thresholds draws `n + 1` rows of equal width, threshold `i` at `(i +
  1)/(n + 1)` of the bar: its open ends have no edge in domain units,
  and equal rows invent none. That is what Phase 25's `Ticks for
  Threshold::position` returns; this phase makes it the colorbar
  geometry and states so in that function's doc comment.
  `Samples::Bins { edges }` carries the thresholds for the labels.
- **Ticks.** The scale's own `Ticks` (tick rule, transform, figure
  `Locale`), so a symlog color scale gets decade ticks; binned scales
  label their tick sets (Phase 25: quantize `[x0, thresholds…, x1]`,
  quantile `[min, quantiles…, max]`, threshold its domain). Tick marks
  point out of the bar on the label side; the bar's outline is its axis
  line (theme edge color, axis line width).
- **`Extend`.** Triangles of 5% of the bar's interior length
  (matplotlib's default `extendfrac`), filled with the scale's clamped
  end colors, outlined with the bar.
- **Layout.** A colorbar's reach (bar, ticks, labels, title) enters the
  protrusion layout like a legend in that slot.

##### Hatching (`figure/src/hatch.rs`)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Hatch { /* angles, spacing_pt, width_pt, color: Option<Rgb> */ }
impl Hatch {
    pub fn lines(deg: f64) -> Self;           // one family
    pub fn cross(deg: f64) -> Self;           // deg and deg + 90
    pub fn spacing(self, pt: f64) -> Self;    // default 6 pt
    pub fn width(self, pt: f64) -> Self;      // default 0.8 pt
    pub fn color(self, c: Rgb) -> Self;       // default theme ink
}
// on Area, Ribbon, BarY, BarX, Histogram, DensityY, DensityX:
pub fn hatch(self, h: Hatch) -> Self;
```

Angles are counter-clockwise on screen, matplotlib's sense (45° looks
like `/`). In y-down device coordinates a family at angle θ is the set
of lines `n·p = k·s` with normal `n = (sin θ, cos θ)` and spacing `s =
spacing · 4/3 · context` px, for the integers `k` from `⌈min(n·c)/s⌉`
to `⌊max(n·c)/s⌋` over the shape's device bounding-box corners `c`.
Because `k·s` is measured from the figure origin, lines continue across
neighbouring bars. Each hatched shape emits, after its fill:

```text
Push { transform: IDENTITY, clip: Some(Clip::Rect(r)) }   // bars
Push { transform: IDENTITY, clip: Some(Clip::Path(p)) }   // areas, ribbons, densities
Stroke { path: one BezPath of all lines, style: butt caps, miter joins, paint: ink }
Pop
```

Lines are generated in device space and mapped back through the
inverse of the current transform, so one push carries the
pre-transformed line path. Rectangles use `Clip::Rect` and everything
else `Clip::Path` (cheap where it can be; see Drawing and output).
Legend swatches of a hatched mark draw the same hatch inside the
swatch. `proto/dlist/src/bin/hatch.rs` has the line generation and a
double push with the inverse transform; its angle runs opposite to
matplotlib's `/` and it detects bars from path data, whereas here the
marks emit their own hatching.

**Decisions this phase may meet**

- seaborn's styles set `patch.edgecolor = "w"` with
  `patch.force_edgecolor`, so seaborn's bars have white 1-pt edges. If
  bars without edges look wrong under the grid styles — recommended: no
  edges by default, a `Theme` builder only if asked; record it as a
  decision.
- The colorbar inset length (40% of the panel's long side) is this
  document's choice; if Phase 17's inset sizing suggests another — use
  that one; record it as a decision.

#### Steps

1. **Theme struct and presets** (`theme.rs`, `theme/{style,context,
   palette,domains}.rs`): builders with validation, getters, defaults,
   `Domains::Nice`; tests compare every setting with the tables above,
   and the builder example of Themes is a test in
   `figure/tests/main/theme.rs`.
2. **Legend glyphs and the wheel presets** (`legend/swatch.rs`,
   `dev/examples/examples/wheels.rs --theme <style>:<context>`): render
   every style × context preset at 960 × 760 and check those at paper
   and notebook; the talk test of Text and math (`wheels_fit_at_talk`:
   `Theme::seaborn(Style::Ticks, Context::Talk)` at 1600 × 1200, no
   issues) in `dev/examples/tests/main/wheels.rs`; record the measured
   plot-area shares in `decisions/phase-35.md`. **Cut here** if the
   phase takes two sessions; the handoff records "Step 2 done".
3. **Colorbar strips** (`legend.rs`, `colorbar.rs`,
   `colorbar/ramp.rs`, `figure.rs`, `panel.rs`): `legend::Guide`, the
   generalized entry points, `colorbar`, `Ramp` for the six kinds,
   `Samples`, the image strip, length and placement; the per-pixel strip
   test renders a 300-px bar at 1× with vello_cpu (feature `raster`).
4. **Colorbar ticks and ends** (`colorbar.rs`, `colorbar/extend.rs`):
   ticks with the figure locale, the threshold geometry, tick marks and
   outline, `Extend` triangles, the layout reach; the `compile_fail`
   doctest `colorbar(ordinal_handle)` (E0277) with its twin on a
   sequential handle.
5. **Hatching** (`hatch.rs`, `mark/*` builders, `legend/swatch.rs`,
   `wheels.rs --hatch`): geometry, clip groups, hatched swatches; tests
   on the line-count rule and on neighbouring bars sharing lines (equal
   `k·s` offsets).

#### Exit criteria

1. `cargo test -p rustd3-figure --features raster --test main --
   colorbar hatch theme` passes.
2. The strip test: on a 300-px bar, every pixel of the rendered strip
   equals the scale's color at that pixel's center value, exactly for
   viridis and magma (table schemes), and for interpolators equals the
   interpolator at the row center.
3. `cargo test -p rustd3-examples -- wheels` passes with every style ×
   context preset free of `fig.check` issues at paper and notebook on
   960 × 760 and `wheels_fit_at_talk`; `cargo run -p rustd3-examples
   --example wheels -- --hatch` writes the hatched figure.
4. `cargo test -p rustd3-figure --doc` runs the colorbar E0277 doctest
   and its twin.
5. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| `sequential([0, 80], viridis)` colorbar, 300-px bar, pixel 150 from the low end | value 40.13, t = 0.50167 → row 128 → #21918c, equal to `apply(40.13)` | d3 table `⌊t·256⌋` |
| Same bar, pixels 0 and 299 | #440154 and #fde725 | d3 in Node |
| d3 viridis and magma tables vs matplotlib's 256-entry colormaps | 0 of 256 rows differ | Node vs matplotlib |
| `sequential([0, 1e6], viridis)` with `.transform(…)` to symlog, constant 1, at 1000 | #21918c (log1p ratio 0.50007); ticks 0 1 10 … 1000000 (8) | d3 in Node; Scales |
| `quantize([0, 100], blues5)` colorbar, `blues5` = d3's `schemeBlues[5]` as a literal array (#eff3ff #bdd7e7 #6baed6 #3182bd #08519c) | 5 equal rows; thresholds 20 40 60 80; tick labels at 0 20 40 60 80 100 (Phase 25's tick set) | d3 in Node |
| `threshold([0, 1], [red, white, blue])` colorbar | 3 equal rows; ticks 0 and 1 at 1/3 and 2/3 of the bar | `(i + 1)/(n + 1)` |
| `colorbar(ordinal_handle)` | does not compile (E0277); twin with a sequential handle compiles | Figures |
| `.extend(Extend::Both)` on a 300-px bar | two triangles of 15 px each, end colors | matplotlib `extendfrac` 5% |
| Bar `[100, 200] × [300, 400]` px, `Hatch::lines(45.0)`, notebook | s = 8 px; k = 36 … 53: 18 lines, one `Stroke`, width 1.067 px | n·c over the corners: 282.84 … 424.26 |
| Two adjacent bars, same hatch | line offsets `k·8` px coincide across the shared edge | anchored to the figure |
| Wheel histograms hatched | 232 bars (the prototype's size filter kept 190: 0.39 ms, +91 KB SVG; record the measured numbers) | `dev/fixtures/wheels.py`; Figures |
| `Theme::default()` | Ticks, Notebook, COLORBLIND, `Margin(0.05)` | Themes |
| `Theme::seaborn(Ticks, Talk).tick_text()` | 11 × 1.5 pt = 16.5 pt = 22 px | seaborn talk `xtick.labelsize` 16.5 |
| Paper: line width, major tick size | 1.2 pt, 4.8 pt | seaborn paper context |
| Poster: font size, axis line width | 24 pt, 2.5 pt | seaborn poster context |
| `font_scale(1.1)` at notebook | tick labels 12.1 pt; line widths unchanged | font scale scales text only |
| `Theme::default().font_scale(-1.0)`, `.palette(Vec::<Rgb>::new())`, `.domains(Domains::Margin(0.5))` | each equals `Theme::default()` | builder validation |
| `Domains::Nice`, a linear handle fitted to `[0.3, 9.7]`, axis `.ticks(5)` | domain `[0, 10]` | d3 `nice(5)`: step 2 |
| Wheel plot-area share at 960 × 760 | about 58.6% paper, 50.7% notebook, 34.7% talk (record measured) | Themes |
| Minimum CAM02-UCS distance, first 3 colors, protanopia | DEEP 5.6, COLORBLIND 18.6 | Themes table |

#### Not in this phase

- Dot and star hatches, path effects (out of scope).
- Theme files in TOML (Later); a string-keyed rcParams map (rejected,
  see Themes).
- Size and color legends for scatter channels (Phase 39); colorbars for
  `fill_by` scatters come for free through the handle.
- Hover and brushes (Phase 36), PDF and GIF outputs (Phase 37).

### Phase 36 — Plot interactions II: hover and 1-D brush

*M4 Complete figure · builds on phases 20, 21, 30, 33 · about 1,000–1,400 lines, 4 commits*

#### Goal

The `Plot` widget reports what the pointer is over and gets
one-dimensional brushes. Hover produces app messages carrying hit-test
results, deduplicated, so moving within one point sends nothing, from
a hit index cached per resolve. A panel can declare a brush over x or
y (core's `BrushX` or `BrushY`) whose live rectangle and persistent
selection band the widget draws as overlay geometry; the selection is
published with `on_brush` and can set the domain of another handle,
which makes Part I's focus and context a figure-level chart. Brushing
alone redraws only the overlay.

#### Builds on

- Phase 11: `Pointer` events from `pointer::Tracker`, including
  `ModifiersChanged` → `Pointer::Modifiers`; the headless harness in
  `iced/tests/` (`iced_runtime::UserInterface` → `RedrawRequested` →
  draw → `Headless::screenshot`, `renderer.tick()` per virtual frame).
- Phase 14: `Kind`, `Position`, `ScaleRef`, `Panel`, `Resolved`,
  `Figure::resolve(size, &mut cache, &held, target)`.
- Phase 17: hit testing with a grid index: `hit::Index::nearest(at,
  radius) -> Option<Hit>`, `Hit { panel, mark, index, at, distance }`
  with `panel` the flat panel index, `Figure::hits(&resolved, &mut
  cache)`, `Mark::hits`.
- Phase 18: `Plot::new(fig)`, the overlay layer clipped per panel with
  `with_layer` (crosshair, highlight).
- Phase 19: worker renders (tests inject `Event::Waken`), `Plot::probe`
  with the `Resolve` and `List` stages, per-cell hashes.
- Phase 20: input accumulated per `RedrawRequested`, held layout during
  gestures, `Plot::on_change(handle, f)`, widget-owned domains keyed by
  (handle index, `TypeId`), double-click resetting touched handles,
  `Held`. Read `iced/src/plot.rs` and `iced/src/plot/*` first.
- Phase 21: `Fit::Window`, `Plot::revision`, `Plot::caching`.
- Phase 30: `interact::{BrushX, BrushY, BrushEvent}`, `is_active`,
  `set_selection` (d3's `brush.move`); `Pointer::Modifiers` events are
  accepted and ignored.
- Phase 33: time handles (the focus-and-context test and the
  brush-kinds twin use them).
- Phase 31, if built: tiles and previews; nothing here needs them.
- Decision files: `phase-11.md`, `phase-17.md`, `phase-18.md` …
  `phase-21.md` (`phase-31.md` if it exists), `phase-30.md`,
  `phase-33.md`.

#### Design

##### The API

A brush is a value placed on a panel, like axes and legends, and the
widget publishes what it does, like `on_change`; `.zoom_y(h)` already
sets the pattern of one place per panel for what input does. Two other
shapes were weighed: everything on `Plot` (`Plot::brush_x(cell, over,
sets)`) splits a panel's configuration across two places, and a brush
as a mark (`panel.mark(brush_x(over))`) puts input among things that
draw data, which every file backend would then have to skip. File
outputs ignore a placed brush (they have no input).

```rust
// Invert is Phase 14's (figure/src/scale/invert.rs), implemented for
// Continuous<T, f64, N> there and for time handles in Phase 33;
// not for Band or Point.

// figure/src/brush.rs: values, like axes
pub fn x<A: Invert>(over: ScaleRef<A>) -> brush::Brush<A, A>;   // brush::x
pub fn y<A: Invert>(over: ScaleRef<A>) -> brush::Brush<A, A>;   // brush::y
impl<A: Invert> brush::Brush<A, A> {
    /// The selection becomes the domain of `target` (d3's focus and context).
    pub fn sets<B>(self, target: ScaleRef<B>) -> brush::Brush<A, B>
    where B: Position<Domain = <A as Kind>::Domain>;
}

// figure/src/panel.rs
impl Panel<'_> {
    /// Drag in this panel brushes; one brush per panel (a second replaces it).
    pub fn brush(&mut self, b: impl Into<brush::Placed>) -> &mut Self;
}

// figure/src/figure/resolved.rs
impl Resolved {
    /// (row, col) of a flat panel index, as `hit::Hit::panel` reports it.
    pub fn panel_cell(&self, panel: usize) -> (usize, usize);
}

// iced/src/plot.rs
impl<'a, Message> Plot<'a, Message> {
    pub fn on_hover(self, f: impl Fn(Option<plot::Hover>) -> Message + 'a) -> Self;
    pub fn on_brush<A: Invert>(
        self,
        over: ScaleRef<A>,
        f: impl Fn(Option<[<A as Kind>::Domain; 2]>) -> Message + 'a,
    ) -> Self;
}

// iced/src/plot/hover.rs, re-exported as plot::Hover
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hover {
    /// Flat panel index (`hit::Hit::panel`); `Resolved::panel_cell` maps it back.
    pub panel: usize,
    /// Index of the mark in the panel's insertion order.
    pub mark: usize,
    /// Index of the item in the mark's data (thinned marks: the sample index).
    pub index: usize,
    /// Pointer position in widget coordinates.
    pub at: iced_core::Point,
}
```

`Hover` is plain data with public fields, like `Pointer`. Its point is
`iced_core::Point`, because rustd3-iced depends on `iced_core` and
`iced_widget` only.

Publishing a selection needs a typed inverse from pixels to the
handle's domain. Continuous and time handles have one; band and point
handles do not, and Phase 20's `on_change` is bounded only by `S: Kind`
(it publishes domains that gestures computed and never inverts pixels).
Bounding `brush::x`, `brush::y` and `on_brush` on `Invert` makes a band
handle in a brush a compile error instead of a runtime `None`. `.sets`
keeps the prototype's bound (`proto/figmodel/src/fig.rs:420-436`,
`B: Kind<Domain = A::Domain>`), so `brush::x(full_linear).sets(focus_time)`
fails with E0271 (`proto/figmodel/fail/brush_kinds.rs`).

##### Behavior

- **Hover.** On `CursorMoved` (accumulated per frame), hit-test the
  panel under the cursor with the cached index; emit
  `on_hover(Some(hover))` when the hit's `(panel, mark, index)` changes,
  and `on_hover(None)` once when the hit is lost or on `Pointer::Left`.
  The highlight overlay (Phase 18) shows the same hit. No hit testing
  while a gesture is active.
- **The hit index is cached per resolve.** The widget keeps
  `hit::Index` in its tree state next to the `Resolved` it was built
  from, and rebuilds it only when the list is rebuilt (a
  `RedrawRequested` that resolved again); `CursorMoved` events between
  redraws query the cached index. `iced/src/plot/hover.rs` holds this
  state.
- **Brush gesture.** A press inside a panel with a brush starts the
  core state machine (`BrushX` for `brush::x`, `BrushY` for `brush::y`)
  with its extent set to the panel's plot rectangle in widget pixels.
  `Pointer::Modifiers` events are routed to the core brush, which
  ignores them. `Brush` and `End` events publish through `on_brush`, with
  domain values from the resolved scale's `invert` at the selection's
  pixel ends, and with `.sets(target)` they set the target's domain
  through the same path as `on_change` (published if registered, else
  widget-owned). A target under `Fit::Window` or `Fit::Visible` is held
  by the brush exactly as a zoom holds it, and released by double-click.
- **Sync rule** (use case 1's bug, generalized). Between gestures the
  widget sets the brush's selection from the target's current domain
  (`set_selection`, d3's `brush.move`); during a gesture it never does,
  and it continues from the domain it last published (see Figures:
  gestures write scales). A zoom on the target moves the band at the
  next redraw.
- **Precedence.** In a panel with both a zoom and a brush on x, a drag
  brushes and the wheel zooms. Double-click resets touched handles
  (Phase 20) and clears the brush selection.
- **Overlay drawing.** The live rectangle and the persistent band are
  filled with the theme's ink at 10% alpha with a 1-px ink outline at
  40%, drawn in the overlay layer of the brushed panel only; a band
  spans the panel's full height (x brush) or width (y brush). A brush
  or hover changes no cell image (part hashes unchanged), so only the
  overlay redraws, unless `.sets(target)` changes a domain.
- **State.** Brush state is keyed like gesture state, by (panel cell,
  handle index, `TypeId`), and dropped on mismatch, so rebuilding the
  figure in `view` keeps a gesture alive.

Tests drive the real widget headlessly through Phase 11's harness,
inject `Event::Waken` for worker renders, and read positions from the
panel rectangles of `fig.resolve(size, &mut cache, &held, target)`
rather than hard-coding layout pixels. The d3 references are
`d3-brush/src/brush.js` (selection events) and Phase 30's brush traces.

**Decisions this phase may meet**

- Phases 18–21 chose names that differ from the signatures above (a
  module, a builder) — follow theirs; record the mapping as a decision.
- A `Pointer::Pressed` lands on a panel with a brush but outside its
  plot rectangle (on an axis): d3 ignores it, and so does the widget —
  record a decision only if Phase 20 already routes axis presses
  elsewhere.

#### Steps

1. **Figure side** (`figure/src/brush.rs`, `figure/src/panel.rs`,
   `figure/src/figure/resolved.rs`; `Invert` already exists for
   continuous handles from Phase 14 and time handles from Phase 33):
   `Resolved::panel_cell`, `brush::{x,
   y}`, `.sets`, `brush::Placed`, `Panel::brush` (stored on the panel,
   ignored by file outputs); the brush-kinds `compile_fail` doctest
   (`brush::x(full_linear).sets(focus_time)`, E0271) and its twin with
   two time handles. Verify: `cargo test -p rustd3-figure --doc brush`.
2. **Hover** (`iced/src/plot/hover.rs`, `iced/src/plot.rs`): `Hover`,
   `on_hover`, the cached index, dedup, `Left`. Tests
   `hovering_a_dot_reports_its_index`,
   `moving_within_one_dot_sends_nothing`,
   `leaving_the_widget_reports_none_once`,
   `hover_does_not_rebuild_the_hit_index` (hovers between two redraws
   add no `Resolve` or `List` probe samples).
3. **Brush gestures** (`iced/src/plot/brush.rs`, `iced/src/plot.rs`,
   state in `iced/src/plot/*`): routing, `on_brush`, `.sets` through the
   `on_change` path, the sync rule, precedence, double-click. Tests
   `brushing_the_context_sets_the_focus_window`,
   `events_before_update_keep_the_last_window`,
   `zooming_the_focus_shrinks_the_band`,
   `an_empty_release_reports_none`,
   `modifiers_do_not_change_a_selection` (a modifier change mid-drag
   leaves the selection as it was).
4. **Overlay** (`iced/src/plot/*` overlay drawing): live rectangle and
   band per panel. Tests `brush_overlay_stays_inside_its_panel` (both
   backends) and `brushing_alone_renders_no_cell`.

Tests live in `iced/tests/main/{hover,brush}.rs`, declared in
`iced/tests/main/main.rs`.

#### Exit criteria

1. `ICED_TEST_BACKEND=tiny-skia cargo test -p rustd3-iced --features
   figure --test main -- hover brush` and the same with
   `ICED_TEST_BACKEND=wgpu` pass the tests of Steps 2–4, including the
   figure-level focus-and-context test.
2. `cargo test -p rustd3-figure --doc brush` runs the brush-kinds
   `compile_fail` doctest and its twin with two time handles.
3. `brush_overlay_stays_inside_its_panel`: after a brush, overlay
   pixels exist inside the brushed panel's plot rectangle only, on both
   backends.
4. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| Focus/context figure: `full = fig.scale(time::utc().domain([2026-09-01T00:00Z, +719 h]))` on the context panel, `panel.brush(brush::x(full).sets(focus))`; drag from 25% to 50% of its plot width | `focus` domain `[start + 179.75 h, start + 359.5 h]` ± one pixel's time (719 h / width) | linear in px (use case 1's 7.7-day check is the 220/860 case) |
| Same drag with the press and both moves delivered before any `update` | the last message still carries the 25–50% window | use case 1's bug, generalized |
| `.sets(focus)` drag, the app applies the domain through `on_change(focus)`, then a wheel zoom ×2 at the focus center | the context band shrinks to half its width about its center at the next redraw | sync rule |
| Release with an empty selection (press and release at one point) | `on_brush(None)`; band removed | d3: empty selection → null |
| A modifier change (alt, shift) in the middle of a drag | selection unchanged | modifiers are ignored |
| Hover a dot at its exact pixel, then move 1 px within the hit radius | one `Some(Hover { mark, index, .. })`, then no message | dedup |
| Ten `CursorMoved` events between two redraws | no new `Resolve` or `List` probe sample | index cached per resolve |
| Pointer leaves the widget | one `on_hover(None)` | `Pointer::Left` |
| Hover over a dot in panel (1, 0) of a 2 × 2 figure | `resolved.panel_cell(hover.panel) == (1, 0)` | flat index mapped back |
| Brush in panel (0, 0) of a 2 × 2 figure | overlay pixels only inside panel (0, 0)'s plot rect, both backends | per-panel `with_layer` clip (see iced integration) |
| `brush::x(full_linear).sets(focus_time)` | does not compile (E0271 on `B::Domain == A::Domain`); twin with two time handles compiles | `proto/figmodel/fail/brush_kinds.rs` |
| Brushing only (no `.sets`) | no cell re-renders (part hashes unchanged); overlay redraw only | iced integration: hover and brushing redraw only the overlay |

#### Not in this phase

- The 2-D brush, keyboard modes and `Plot::on_region` (Later).
- Box zoom, pinch zoom, axis-drag zoom, linked cursors (out of scope).
- Tooltips as iced widgets: apps build them from `on_hover` messages;
  the widget draws only the highlight.
- Brushes in core-level apps (use case 1, Phase 32) and core brush
  semantics (Phase 30).
- Output formats (Phase 37); helpers (Phases 38–39).

### Phase 37 — Outputs II: PDF and GIF

*M4 Complete figure · builds on phases 12, 17, 34 · about 800 lines (≈500 library, ≈300 tests), 3 commits*

#### Goal

The drawing list reaches its two remaining file backends: PDF through
krilla with real, selectable text and subset fonts (feature `pdf`), and
animated GIF from raster frames (feature `gif`). PDFs are checked by
rendering them back with poppler (`pdftoppm`, testkit's tile score
against a pinned vello_cpu reference) and extracting their text
(`pdftotext`, `pdffonts`). The PDF pixel bound is measured first,
recorded, then asserted. v0.1 has no anyrender adapter and no
geometric clip cutting (see Later): no built-in backend needs either,
and `lower` keeps Phase 12's signature, returning warnings only.

#### Builds on

- Phase 2: testkit's pixel statistics and 32-px tile score.
- Phase 12: `list::{List, Command, Clip, GlyphRun, Glyph, Caps}` with
  per-point marker sizes and paint indices and `parts`; `lower(&List,
  Caps)` returning warnings as values; `raster::Renderer::new(Settings)`
  and `render(&list, &bounds, area, scale)`, `raster::Settings {
  threads, simd }`, `Settings::parallel()`, `Simd::{Detect, Baseline}`,
  `list::Bounds::of`; PNG; `list::dump`.
- Phase 13: glyph runs indexing `List::fonts` (peniko `FontData`), the
  source text kept per run.
- Phase 15: `draw`, `draw_with(&mut figure::Cache, size, Target, &Held)
  -> List`, `Target::File { scale }`, `save_svg`, `save_png` returning
  `figure::Error` (`figure/src/figure/save.rs`), `figures_equal`.
- Phase 17: `fig.check`.
- Phase 34: the wheel figure (`rustd3_examples::wheels`), a test input;
  tests that need it live in `dev/examples`.
- Decision files: `phase-12.md`, `phase-13.md`, `phase-15.md`,
  `phase-17.md`.

#### Design

Modules are named after their format. Inside `figure/src/gif.rs` the
dependency is reached by absolute path (`::gif::Encoder`), because the
module name shadows the crate name and imports are never aliased.

##### PDF (`figure/src/pdf.rs`, `figure/src/pdf/image.rs`, feature `pdf`)

```rust
pub const CAPS: Caps = Caps { glyphs: true, markers: true, clip_paths: true, sweep: true };

#[derive(Debug, Clone, Default)]
pub struct Options { /* private: outlined: bool, title: Option<String> */ }
impl Options {
    /// Glyphs as filled outlines instead of text (no fonts embedded).
    pub fn outlined(self, yes: bool) -> Self;
    pub fn title(self, t: impl Into<String>) -> Self;
}

pub fn write(list: &List, options: &Options) -> Result<Vec<u8>, pdf::Error>;

impl Figure<'_> {
    pub fn save_pdf(&self, path: impl AsRef<Path>, size: Size) -> Result<(), figure::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("page size {width} × {height} px is not positive and finite")]
    Size { width: f64, height: f64 },
    #[error("font {index} of the list could not be loaded by krilla")]
    Font { index: usize },
    #[error("command {index}: {what} cannot be written to PDF")]
    Unsupported { index: usize, what: &'static str },
    #[error("krilla: {0}")]
    Krilla(String),
}

// figure/src/figure/save.rs: figure::Error gains
#[cfg(feature = "pdf")]
#[error(transparent)]
Pdf(#[from] pdf::Error),
```

Mapping rules (krilla 0.8.2: `Document::{new, start_page_with,
set_metadata, finish}`, `PageSettings::from_wh`, `Surface::{set_fill,
set_stroke, draw_path, draw_glyphs, push_transform, push_clip_path,
draw_image, pop}`, `Image::from_rgba8`, `Image::from_custom`,
`paint::{LinearGradient, RadialGradient, SweepGradient, Stop,
SpreadMethod}`, `Metadata` with an optional creation date):

- **Units.** The page is `width × 0.75` by `height × 0.75` points (1 pt
  = 4/3 px) and the root content transform is `scale(0.75)`, so list
  coordinates stay in px. A 960 × 760 px figure is a 720 × 570 pt page;
  `pdftoppm -r 96` gives 960 × 760 px back, `-r 192` the 2× raster.
- **Paints.** `Brush::Solid` → a krilla rgb color plus opacity (alpha);
  linear, radial and sweep gradients → krilla's gradient structs with
  `Stop { offset, color, opacity }` and `SpreadMethod` from peniko's
  extend; `Brush::Image` → `Error::Unsupported` (the figure layer emits
  `Command::Image`, never image brushes).
- **Fills** use `FillRule::NonZero`; **strokes** map width, start cap
  (butt, round, square), join, miter limit, dash array and offset. Caps
  and joins are always explicit in the list, since kurbo's defaults
  differ from PDF's.
- **Markers** stamp the shape with `push_transform(translate)` per
  point; per-point sizes add a uniform scale, per-point paint indices
  switch the fill. Points are grouped by paint index to limit fill
  switches.
- **Glyphs.** One krilla `Font` per list font, built once per document
  from `Data::from(Arc::new(font.data.clone()) as Arc<dyn AsRef<[u8]> +
  Send + Sync>)` (no byte copy). Per run: `push_transform(run.transform)`,
  then `KrillaGlyph::new(GlyphId::new(g.id), g.advance/size, (g.x −
  pen)/size, −g.y/size, 0.0, g.text range, None)` with `pen` the sum of
  previous advances, and `draw_glyphs(origin, &glyphs, font, &run.text,
  size, outlined)`. Selectable text follows from `run.text` and the
  byte ranges.
- **Images.** Un-premultiply if the image is premultiplied. `smooth:
  false` → `Image::from_rgba8` (krilla never interpolates these);
  `smooth: true` → `Image::from_custom(rgba, true)`. `from_custom<T:
  CustomImage>` needs an impl (separate color and alpha channels, `Hash
  + Clone + Send + Sync + 'static`), so `figure/src/pdf/image.rs` holds
  `pub(crate) struct Rgba { width: u32, height: u32, pixels: Arc<[u8]> }`
  implementing `CustomImage`: straight RGB as the color channel, A as
  the alpha channel, `Hash` over the bytes (about 40 lines). An image is
  drawn as `push_transform(translate(rect.origin) · scale)` then
  `draw_image(image, Size::from_wh(w, h))`.
- **Push/Pop.** `push_transform`, then for `Clip::Rect` or `Clip::Path`
  a `push_clip_path(…, NonZero)`; a stack records how many surface pops
  each `Pop` owes (1 or 2).
- **Determinism.** No creation date (no ambient clock in library code);
  metadata only when `title` is set. Writing the same list twice yields
  identical bytes.

Lift from `proto/dlist/src/pdf.rs`: path and transform conversion,
stroke mapping, glyph normalization (`advance/size`, `(x − pen)/size`,
`−y/size`), the push/pop count stack. Different here: the page in
points, every paint (the prototype greys out gradients), images, marker
channels, no `unwrap`, no font copy.

**The PDF pixel bound.** The Testing section asks for a poppler round trip and
extractable text, not a pixel bound, and poppler's glyph anti-aliasing
differs from vello_cpu's, so the bound is measured before it is
asserted. Render the test figure and the wheel figure with `pdftoppm
-r 192 -png`, decode with the `png` crate, and compute testkit's 32-px
tile score against a reference rendered through
`raster::Renderer::new(Settings { threads: 0, simd: Simd::Baseline })`
at 2× (pinned for determinism; not `save_png`, which uses
`Settings::parallel()`). Record the measured scores in
`decisions/phase-37.md`. The test then asserts a tile score ≤ 0.02 if
the measurement is ≤ 0.02, otherwise ≤ 1.25 × the measured value, and
never above 0.05: a measurement above 0.05 is a bug in the writer (a
wrong transform, a dropped paint, a missing clip) to repair in this
phase, not a bound to record.

The poppler tools run through `std::process::Command`. When `pdftoppm`
is missing, the PDF tests fail with a message naming poppler-utils (the
container has poppler 24.02).

##### GIF (`figure/src/gif.rs`, feature `gif` = `raster` + the gif crate)

```rust
pub enum Repeat { Forever, Times(u16) }

pub struct Writer<W: std::io::Write> { /* ::gif::Encoder<W>, size, scale, figure::Cache */ }
impl<W: std::io::Write> Writer<W> {
    pub fn new(out: W, size: Size, scale: f64, repeat: Repeat) -> Result<Self, gif::Error>;
    /// Draws, rasterizes (vello_cpu, `Settings::parallel()`) and encodes one frame.
    pub fn frame(&mut self, fig: &Figure<'_>, delay: Duration) -> Result<(), gif::Error>;
    pub fn finish(self) -> Result<W, gif::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{width} × {height} device px exceeds GIF's 65,535 limit")]
    Size { width: u64, height: u64 },
    #[error("figure has a translucent background; GIF has 1-bit alpha")]
    Translucent,
    #[error("gif: {0}")]
    Encoding(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
```

Each frame: `fig.draw_with(&mut self.cache, size, Target::File { scale
}, &Held::default())`, with one `figure::Cache` for the whole sequence
so labels repeated between frames are shaped once; raster at `scale`;
convert to straight RGBA; `Frame::from_rgba_speed(w, h, &mut rgba, 10)`
with `delay = max(2, round(ms / 10))` centiseconds (browsers treat
delays under 2 cs as 10 cs). The repeat extension is written in `new`,
before any frame. GIF's alpha is one bit, so a translucent theme
background is `Error::Translucent`, not a silent change. gif 0.14.2:
`Encoder::new(w, width: u16, height: u16, &[])`,
`set_repeat(Repeat::{Infinite, Finite(n)})`, `Frame::from_rgba_speed(w,
h, &mut [u8], speed 1..=30)`, `delay` in centiseconds, and a
`DecodeOptions` reader for the tests. The 30-frame zoom sequence of
`proto/dlist/src/bin/extra.rs` (lines 88–138) is the sequence test's
model.

**The GIF agreement bound** has no number in Part I: measure a decoded
frame against the PNG of the test figure (palette quantization), record
the max and tile score in `decisions/phase-37.md`, and assert 1.5 × the
measured values.

**Decisions this phase may meet**

- `pdftoppm` is unavailable where the tests must run — keep the failing
  message; add a skip switch only as a recorded decision.

#### Steps

1. **PDF writer** (`figure/src/pdf.rs`, `figure/src/pdf/image.rs`,
   `figure/src/figure/save.rs`, tests in `figure/tests/main/pdf.rs`):
   mapping, units, determinism, the `CustomImage` impl, `save_pdf` and
   the `figure::Error::Pdf` variant. Tests include
   `writing_a_list_twice_gives_identical_bytes` and
   `smooth_and_nearest_images_both_round_trip` (both render back
   through poppler; nearest keeps hard edges: a 2 × 2 checker scaled to
   200 px has ≤ 2 distinct grey levels per tile).
2. **PDF round trip** (`figure/tests/main/pdf.rs`,
   `dev/examples/tests/main/wheels.rs`, `.claude/decisions/phase-37.md`):
   measure and record the tile scores, then assert the bound; `pdftotext`
   returns every label; `pdffonts` lists embedded subset fonts;
   `wheels_pdf` runs the wheel figure through PDF, `pdftoppm`,
   `pdftotext` and `pdffonts`.
3. **GIF** (`figure/src/gif.rs`, `figure/tests/main/gif.rs`): writer,
   delays, repeat, size and translucency errors; decode with
   `::gif::DecodeOptions` to check frame count, delays and repeat;
   measure, record and assert the agreement bound.

#### Exit criteria

1. `cargo test -p rustd3-figure --features pdf,gif,raster --test main --
   pdf gif` passes.
2. The PDF of the test figure and of the wheel figure, rendered with
   `pdftoppm -r 192`, meets the asserted tile-score bound (≤ 0.02, or
   1.25 × the measured value recorded in `decisions/phase-37.md`, never
   above 0.05); `pdftotext` returns every label; `pdffonts` lists
   embedded subset fonts (`cargo test -p rustd3-examples -- wheels_pdf`).
3. Each feature builds alone: `for f in pdf gif; do cargo check -p
   rustd3-figure --no-default-features --features $f; done`.
4. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| 960 × 760 px list → PDF | page 720 × 570 pt; `pdftoppm -r 192` gives 1920 × 1520 px | 1 pt = 4/3 px |
| Wheel figure → PDF | about 114 KB (record), text extractable: "Hub vibration and bearing temperature", "Wheel: RL", "speed [km/h]", "min. count" | Drawing and output |
| `pdffonts` on it | DejaVuSans and DejaVuSans-Bold, `emb yes`, `sub yes` | subset fonts |
| Same with `Options::outlined(true)` | `pdftotext` returns no label text; `pdffonts` lists none | krilla `outlined` |
| PDF vs the pinned PNG at 2× | tile score ≤ 0.02 if measured ≤ 0.02, else ≤ 1.25 × measured, never > 0.05 | measured bound in `decisions/phase-37.md` |
| Writing the same list twice | byte-identical PDFs | no creation date |
| 2 × 2 checker image, `smooth: false`, scaled to 200 px, round trip | ≤ 2 distinct grey levels per tile | `Image::from_rgba8` |
| GIF frame delay 40 ms / 16.7 ms / 5 ms | 4 / 2 / 2 cs | rounding, 2-cs floor |
| GIF at 2× of a 40,000-px-wide figure | `Error::Size { width: 80000, .. }` | `u16` frame size |
| 30-frame sequence, `Repeat::Forever` | decoder: 30 frames, repeat infinite | gif 0.14 decoder |
| GIF per-frame cost at 1× (wheel) | report only: 12.5 ms raster + 4.8 ms list + 2.0 ms text | Drawing and output |

#### Not in this phase

- An anyrender adapter and geometric clip cutting in `lower` (Later).
- PDF tagging, PDF/A, multi-page documents, embedded hyperlinks.
- iced output: the `Plot` widget shows vello_cpu images and needs no
  lowering (see iced integration).
- Video formats, APNG, WebP.
- SVG's lowering of sweep gradients (Phase 12's).

### Phase 38 — Helpers I: statistical charts

*M4 Complete figure · builds on phases 25, 33, 34, 35 · about 1,000 lines (≈550 library, ≈450 tests and fixtures), 4 commits*

#### Goal

The `helpers` feature of rustd3-figure ships matplotlib's statistical
conveniences as ordinary code over the public figure API: `subplots`,
box plots (1.5 × IQR whiskers, outliers, notches), violins from Phase
34's KDE, histogram options (weights, density, cumulative counts, more
bin rules, stacking over shared edges), ECDF and error bars. Their
statistics are pure functions graded against matplotlib, numpy and
scipy reference values from a checked-in Python script, compared
tolerantly, never byte for byte. Hexbin, stem and event plots are Later.

#### Builds on

- Phase 2: testkit's tolerant `Obs` compare and `TryFrom<&serde_json::Value>
  for Obs` (serde_json is a dev-dependency of rustd3-figure).
- Phases 14 and 16: `Figure`, `Panel`, `Grid`, spans, outer-only labels.
- Phase 25: core `array`: `quantile` (R-7, d3's and numpy's default
  "linear"), `extent`, `bin`; `ticks` (Phase 3).
- Phase 29: `shape::stack` (columns in, `[y0, y1]` out).
- Phase 33: band handles for box and violin groups.
- Phase 34: `bar_y`, `bar_x`, `histogram` with `mark::Bins::{Nice,
  Count}` and `mark::Binned`, `rect`, `stats::Kde` (Scott), `dots`,
  `rule_x`, `rule_y`, `ribbon`, `note`, `.fill`, `.dodge`; marks that
  draw nothing report to `fig.check`.
- Phase 35: the theme palette (helpers color by it) and legend
  swatches.
- Decision files: `phase-25.md`, `phase-33.md`, `phase-34.md`,
  `phase-35.md`.

#### Design

##### Modules and feature

```text
figure/src/helpers.rs                 the module, declared #[cfg(feature = "helpers")] pub mod helpers; in lib.rs
figure/src/helpers/subplots.rs        Subplots, subplots
figure/src/helpers/stats.rs           pub use of the chart functions below
figure/src/helpers/stats/boxplot.rs   boxplot::Summary, boxplot::summary, boxplot
figure/src/helpers/stats/violin.rs    violin
figure/src/helpers/stats/hist.rs      hist (the options layer over mark::histogram), hist::table, hist::stacked
figure/src/helpers/stats/ecdf.rs      ecdf
figure/src/helpers/stats/errorbar.rs  errorbar
```

Every chart function returns a value implementing the figure's `Mark`
trait, or a small struct of marks with `fn add_to(self, panel: &mut
Panel<'_>)`, so it is added with `panel.mark(…)` like any mark. The
statistics are separate pure functions returning plain data, so tests
grade them without drawing. Each helper is code users can copy and
change (see Helpers).

##### `subplots`

```rust
pub struct Subplots<'a> { /* fig: Figure<'a>, x: Vec<ScaleRef<Linear>>, y: Vec<ScaleRef<Linear>> */ }

/// One panel per cell; x fitted and shared per column, y per row, with
/// bottom and left axes on every panel (outer-only labels by layout).
pub fn subplots<'a>(rows: usize, cols: usize) -> Subplots<'a>;

impl<'a> Subplots<'a> {
    pub fn x(&self, col: usize) -> ScaleRef<Linear>;   // panics in debug on a bad index
    pub fn y(&self, row: usize) -> ScaleRef<Linear>;
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel<'a>;
    pub fn figure(&mut self) -> &mut Figure<'a>;
    pub fn into_figure(self) -> Figure<'a>;
}
```

Other scale kinds come from copying the helper; v0.1 has no generic
variant.

##### Box plots

```rust
pub mod boxplot {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Summary {
        pub q1: f64, pub median: f64, pub q3: f64, pub mean: f64,
        pub whisker_low: f64, pub whisker_high: f64,
        pub notch_low: f64, pub notch_high: f64,
        pub fliers: Vec<f64>,
    }
    /// matplotlib's `cbook.boxplot_stats` with `whis`: quantiles R-7;
    /// whiskers at the most extreme data within
    /// [q1 − whis·IQR, q3 + whis·IQR]; notch = median ± 1.57·IQR/√n.
    /// `None` for no finite values.
    pub fn summary(values: &[f64], whis: Whis) -> Option<Summary>;
    pub enum Whis { Iqr(f64), Percentiles(f64, f64) }   // default Iqr(1.5)
}
pub fn boxplot<X, Y>(groups: impl IntoIterator<Item = (X::Domain, Vec<f64>)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> BoxPlot<X, Y>
where X: Position, Y: Position<Domain = f64>;
// .notch(bool), .whis(Whis), .fill(c, key), .dodge(), .means(bool)
```

With `Percentiles(lo, hi)` the whiskers sit at the most extreme data
inside the `lo`th and `hi`th percentiles, and everything beyond them is
a flier. Drawing per group, in its (dodged) band slot at 50% of the
slot width: the box `q1…q3` (fill plus a 1 pt edge in ink), a median
line, whiskers and caps (caps 50% of the box width), fliers as dot
markers. Notched boxes narrow to 50% width at the median, with the
notch from `notch_low` to `notch_high` (matplotlib's shape). Whisker
ends and fliers enter fitting; the box does not hold zero.

##### Violins

```rust
pub fn violin<X, Y>(groups: impl IntoIterator<Item = (X::Domain, Vec<f64>)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Violin<X, Y>
where X: Position, Y: Position<Domain = f64>;
// .points(n) default 100, .cut(k) default 0 (matplotlib: the data range),
// .inner(Inner::{None, Box, Quartiles}) default Box, .fill(c, key), .dodge()
```

Each group: `stats::Kde::new(values)` (Scott), evaluated at `points`
evenly spaced over `[min − cut·h, max + cut·h]`; the outline is the
density mirrored about the slot center. Widths are shared within one
mark: the largest density among all of the mark's groups spans the
slot width, so areas compare between groups (matplotlib scales each
violin on its own). Degenerate groups (n < 2 or zero variance) draw a
horizontal line at the value. The outline is drawn as `DensityY` draws
(fill plus stroke, `proto/figmodel/src/plot/marks.rs`).

##### Histogram options

```rust
pub fn hist<X, Y>(values: impl IntoIterator<Item = f64>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Hist<X, Y>
where X: Position<Domain = f64>, Y: Position<Domain = f64>;
impl<X, Y> Hist<X, Y> {
    pub fn bins(self, b: hist::Bins) -> Self;
    pub fn weights(self, w: impl IntoIterator<Item = f64>) -> Self;   // length checked
    pub fn density(self, yes: bool) -> Self;      // count / (total weight · width)
    pub fn cumulative(self, yes: bool) -> Self;
    pub fn step(self, yes: bool) -> Self;         // outline only (histtype="step")
}

pub mod hist {
    /// `mark::Bins`' two rules plus caller edges and numpy's "auto".
    pub enum Bins { Nice(usize), Count(usize), Edges(Vec<f64>), Auto }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Table { pub edges: Vec<f64>, pub counts: Vec<f64> }
    /// The statistic behind `hist` and `stacked`.
    pub fn table(values: &[f64], weights: Option<&[f64]>, bins: &Bins, density: bool, cumulative: bool)
        -> Result<Table, hist::Error>;
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("edge {index} is not finite or not above the one before: {value}")]
        Edge { index: usize, value: f64 },
        #[error("{weights} weights for {values} values")]
        Weights { values: usize, weights: usize },
    }

    /// Groups stacked over one set of edges, one `rect` mark per group.
    pub fn stacked<K, X, Y, C>(groups: impl IntoIterator<Item = (K, Vec<f64>)>, x: ScaleRef<X>, y: ScaleRef<Y>, color: ScaleRef<C>)
        -> hist::Stacked<K, X, Y, C>
    where C: Paint<Domain = K>, X: Position<Domain = f64>, Y: Position<Domain = f64>;
    // Stacked: .bins(hist::Bins), .density(bool), fn add_to(self, panel: &mut Panel<'_>)
}
```

- **Bin rules.** `Nice(n)` and `Count(n)` are Phase 34's (default
  `Nice` with Sturges' count). `Edges` uses the caller's edges, which
  must be finite and strictly increasing; the error carries the first
  bad index. `Auto` is numpy's `"auto"`: the larger count of Sturges and
  Freedman–Diaconis (bin width `2·IQR·n^(−1/3)`), falling back to
  Sturges when the IQR is 0. Bins are left-closed, the last one
  right-closed; values outside `Edges` are dropped, as numpy and
  matplotlib's `hist` drop them.
- **Density and cumulative.** With `density`, the bars' area sums to 1;
  with `cumulative` and `density`, the last bar is 1.
- **Errors in the mark.** `.bins` and `.weights` return `Self`; a `Hist`
  whose `table` fails draws nothing and `fig.check` reports the error,
  as Phase 34's marks report theirs.
- **Stacked.** `hist::stacked` computes one set of edges from the union
  of all groups' finite values, counts per group per bin, stacks the
  count columns with core `shape::stack` (order: the color handle's key
  order; offset none) and adds one `mark::rect` per group with rows
  `([edge_i, edge_{i+1}], [y0, y1])` and `.fill(color, key)`. With
  `density`, counts divide by the total weight of all groups
  (matplotlib's `stacked=True, density=True`). Marks have no `.stack()`
  (see Figures: stacking is computed by helpers).

##### ECDF and error bars

```rust
pub fn ecdf<X, Y>(values: impl IntoIterator<Item = f64>, x: ScaleRef<X>, y: ScaleRef<Y>) -> Ecdf<X, Y>
where X: Position<Domain = f64>, Y: Position<Domain = f64>;
// a step after each sorted value, y = i/n; .complementary(bool); y reports [0, 1]

pub fn errorbar<X, Y>(rows: impl IntoIterator<Item = (X::Domain, f64, errorbar::Err)>, x: ScaleRef<X>, y: ScaleRef<Y>) -> ErrorBar<X, Y>
where X: Position, Y: Position<Domain = f64>;
// errorbar::Err::{Sym(f64), Asym(f64, f64)}; .caps(pt)
```

Error bars draw no caps by default: matplotlib's default `capsize` is
0 and seaborn draws none. `.caps(pt)` draws caps reaching `pt` to each
side of the bar (matplotlib's `capsize`). Bar ends enter fitting.

**Decisions this phase may meet**

- `python3 dev/fixtures/stats.py --check` differs beyond 1e-9 on another
  machine's matplotlib, numpy or scipy — keep the committed JSON, read
  the differing keys, and regenerate only when the new values are the
  reference's own change; record it as a decision.

#### Steps

1. **Fixture script** (`dev/fixtures/stats.py`, `dev/fixtures/stats.json`):
   emit every row of the spot checks for `X` and the other datasets
   below, floats written with `repr()` and keys sorted; `--check`
   recomputes every value and compares it with the committed JSON at
   relative 1e-9 (exit status 1 and the differing keys otherwise).
   Commit both. The Rust tests read the JSON with serde_json into
   `Obs`.
2. **`subplots` and box plots** (`helpers.rs`, `helpers/subplots.rs`,
   `helpers/stats.rs`, `helpers/stats/boxplot.rs`, tests, a doc example
   for each).
3. **Violins and histogram options** (`helpers/stats/violin.rs`,
   `helpers/stats/hist.rs` with `hist::table` and `hist::stacked`,
   tests, doc examples).
4. **ECDF and error bars** (`helpers/stats/ecdf.rs`,
   `helpers/stats/errorbar.rs`, tests, doc examples).

Tests live in `figure/tests/main/helpers.rs` and
`figure/tests/main/helpers/*.rs`, declared in `figure/tests/main/main.rs`
as `#[cfg(feature = "helpers")] mod helpers;`.

#### Exit criteria

1. `python3 dev/fixtures/stats.py --check` exits 0 (matplotlib 3.11.2,
   numpy and scipy as installed). It is not part of `check.sh`, which
   cannot assume those packages.
2. `cargo test -p rustd3-figure --features helpers --test main --
   helpers` passes: every statistic below against the fixture through
   the tolerant `Obs` compare, plus figure-level tests (marks produced,
   `fig.check` empty).
3. `cargo test -p rustd3-figure --features helpers --doc` runs an
   example for every helper.
4. `cargo check -p rustd3-figure --no-default-features --features
   helpers` builds.
5. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

Dataset `X = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6,
6.2, 7.9, 9.5, 14.0]` (n = 15) unless stated; values from matplotlib
3.11.2, numpy and scipy.

| Input | Expected |
| --- | --- |
| `boxplot::summary(X, Iqr(1.5))` | q1 3.25, median 4.4, q3 5.9, IQR 2.65, mean 5.206667, whiskers 1.2 and 9.5, fliers [14.0], notch 3.325764 … 5.474236 |
| `summary(X, Percentiles(5, 95))` | whiskers 2.3 and 9.5; fliers [1.2, 14.0] |
| core `array::quantile(X, 0.25 / 0.5 / 0.75)` | 3.25 / 4.4 / 5.9 (d3 and numpy agree) |
| `hist::table(X, Count(10))` | edges 1.2, 2.48, 3.76, 5.04, 6.32, 7.6, 8.88, 10.16, 11.44, 12.72, 14.0; counts 2 3 4 3 0 1 1 0 0 1 |
| same, density | 0.104167 0.15625 0.208333 0.15625 0 0.052083 0.052083 0 0 0.052083 |
| `Auto` | 6 bins: 1.2, 3.333333, 5.466667, 7.6, 9.733333, 11.866667, 14.0 (Freedman–Diaconis wins over Sturges' 5) |
| `Count(5)` | edges 1.2, 3.76, 6.32, 8.88, 11.44, 14.0; counts 5 7 1 1 1 |
| same, cumulative | 5 12 13 14 15 |
| same, weights 1..=15 | 15 63 13 14 15 |
| default `mark::Bins::Nice` (d3.bin) | `[0,2) 1, [2,4) 5, [4,6) 5, [6,8) 2, [8,10) 1, [10,12) 0, [12,14) 0, [14,16] 1` |
| `hist::table` with `Edges(vec![0.0, 1.0, 1.0, 2.0])` | `Err(Edge { index: 2, value: 1.0 })` (not strictly increasing) |
| `hist::stacked`, groups a = [1, 2, 2, 3], b = [2, 3, 3], `Edges(vec![0.5, 1.5, 2.5, 3.5])` | a: [0, 1], [0, 2], [0, 1]; b: [1, 1], [2, 3], [1, 3] (counts a 1 2 1, b 0 1 2); two `rect` marks |
| violin of X, 100 points over [1.2, 14] | coords 1.2 … 14.0; max density 0.138084 at 4.044444; mean 5.206667, median 4.4 |
| KDE bandwidth on X | 1.879048 (Scott: 3.229654 × 15^(−1/5)) |
| `ecdf(X)` first steps | (1.2, 0.066667), (2.3, 0.133333), (2.9, 0.2), (3.1, 0.266667); last (14.0, 1.0) |
| `subplots(2, 3)` | 6 panels; `x(0..3)` and `y(0..2)` distinct handles; only the bottom row shows x labels, only the left column y labels |
| `summary(&[])`, `summary(&[f64::NAN])` | `None` |
| `summary(&[3.0])` | every quartile 3.0, whiskers 3.0, no fliers, notch 3.0 … 3.0 |

#### Not in this phase

- Chart presets, bar and pie labels, size and color legends, axis
  defaults: Phase 39.
- Hexbin, stem and event plots, polar axes, inset zoom (Later).
- New scale types (see Helpers: no scale is a helper).
- seaborn's `hue` semantics or a grammar of graphics (non-goals).
- 2-D KDE, regression plots, swarm plots.

### Phase 39 — Helpers II: presets, labels and axis defaults

*M4 Complete figure · builds on phases 29, 35, 38 · about 1,400–1,800 lines, 4 commits · two sessions: cut after Step 2*

#### Goal

The rest of the `helpers` feature: one-call chart presets (line,
scatter, bar grouped, stacked or diverging, stacked area, pie and
donut, heatmap) with axes, legend and theme defaults; value labels on
bars and pies; size and color legends for scatter channels; and axis
defaults (minor ticks on, with offset and multiplier notation kept on), for which
figure axes gain `Axis::minor(bool)`. Everything except `Axis::minor`
is ordinary code over public APIs. Stacked charts compute baselines
with `shape::stack` and emit explicit-baseline marks. Polar axes, inset
zoom, the streamgraph preset and the concise-date offset label are
Later; with the ISO ladder, time axes already label concisely.

#### Builds on

- Phase 5: `AnyContinuous`, `Transform::minor`.
- Phase 6: core `Ticks::minor` (core axes' `.minor(true)`).
- Phase 9: `Data` for `Series<'a, X>` by value; `Recording::series(i)
  -> Option<Series>`.
- Phase 16: offset and multiplier notation on figure axes. Figure axes
  have no minor ticks yet (Phases 14–17 built none).
- Phase 28: the ISO time ladder (`%Y`, `%Y-%m`, `%m-%d`, `%H:%M`,
  `%H:%M:%S`, `%S.%L`).
- Phase 29: `shape::{pie, arc, stack}`, `stack::{Order, Offset}` with
  `Offset::Diverging`, `Arc::centroid`, pie slices carrying `datum` and
  `index`.
- Phases 33–34: band, time and color handles; `bar_y`, `bar_x`,
  `bar_y_spans`, `ribbon`, `image`, `note`, `dots` with
  `fill_by`/`size_by`, `.dodge()`.
- Phase 35: `Theme` (palette, sizes, minor tick size and width),
  colorbars, legend swatches, `legend::Guide`.
- Phase 38: the `helpers` module, `subplots`, `hist`.
- Decision files: `phase-16.md`, `phase-28.md`, `phase-29.md`,
  `phase-35.md`, `phase-38.md`.

#### Design

##### Modules

```text
figure/src/helpers/chart.rs                                       chart:: presets (re-exports)
figure/src/helpers/chart/{line,scatter,bar,area,pie,heatmap}.rs
figure/src/helpers/labels.rs                                      labels::bars, labels::pie
figure/src/helpers/legend.rs                                      legend::sizes, legend::colors
figure/src/helpers/axes.rs                                        axes::defaults
figure/src/axis.rs                                                Axis::minor(bool)
```

##### Presets (`helpers::chart`)

Every preset is a consuming builder that ends in a figure:

```rust
pub fn line<'a>(data: impl Data<'a, f64, f64>) -> chart::Line<'a>;   // more series: .series(data, key)
pub fn scatter<'a>(points: impl IntoIterator<Item = (f64, f64)>) -> chart::Scatter<'a>;
pub fn bar<K>(rows: impl IntoIterator<Item = (K, Vec<f64>)>, series: Vec<String>) -> chart::Bar<K>;
pub fn area<'a>(columns: Vec<Vec<f64>>, x: Vec<f64>) -> chart::Area<'a>;
pub fn pie<K>(slices: impl IntoIterator<Item = (K, f64)>) -> chart::Pie<K>;
pub fn heatmap(grid: mark::Grid) -> chart::Heatmap;

// common builder methods (each preset):
//   .title(t) .x_label(t) .y_label(t) .size(w, h) (default 640 × 480 px,
//   matplotlib's 6.4 × 4.8 in at 100 dpi) .theme(Theme) .minor(bool)
//   .figure() -> Figure<'a>
//   .save_svg(path) -> Result<(), figure::Error>
//   #[cfg(feature = "png")] .save_png(path, scale) -> Result<(), figure::Error>
impl chart::Line<'_> { pub fn y_scale(self, s: impl Into<AnyContinuous>) -> Self; pub fn x_scale(self, s: impl Into<AnyContinuous>) -> Self; }
impl<K> chart::Bar<K> { pub fn mode(self, m: chart::bar::Mode) -> Self; }  // Grouped | Stacked | Diverging
impl<K> chart::Pie<K> { pub fn inner(self, share: f64) -> Self; }         // donut: inner radius / outer
impl chart::Heatmap { pub fn colors(self, scheme: fn(f64) -> Rgb) -> Self; } // default viridis, colorbar right
```

`chart::line` takes a recording channel by value, as every mark does:
`chart::line(rec.series(0).expect("channel 0"))` (see Helpers for the
full sketch). Behavior:

- **Colors and legend.** Colors come from the theme palette through an
  ordinal scale over the series keys; a legend goes into `Slot::Right`
  when there is more than one series; axes get `axes::defaults`.
- **Bars.** `Grouped` emits one `bar_y` per series with `.fill(color,
  key).dodge()`. `Stacked` computes `[y0, y1]` with `shape::stack`
  (offset none) and `Diverging` with `Offset::Diverging` (d3's
  `stackOffsetDiverging`: positives up from 0, negatives down); both
  emit one `bar_y_spans` mark per series with `.fill(color, key)`.
- **Area.** `shape::stack` (offset none), one `ribbon` per series.
- **Pie.** `shape::pie` angles (d3: 0 at 12 o'clock, clockwise, sorted
  descending by value, slices in data order carrying their index),
  `shape::arc` paths centered in the plot area. Labels sit at
  `arc.centroid` inside the slices, so the pie reserves no room for
  them: outer radius = `0.5 × min(plot width, plot height) − em(0.5)`.
- **Heatmap.** An `image` mark with a sequential color handle fitted to
  the values, a colorbar in `Slot::Right`.

##### Labels (`helpers::labels`)

```rust
/// matplotlib's `bar_label`: the formatted value at the bar's end,
/// offset outward by `padding` (default 3 pt): above positive bars,
/// below negative ones; `.center()` puts it inside, centered.
pub fn bars<X, Y>(data: &[(X::Domain, f64)], x: ScaleRef<X>, y: ScaleRef<Y>) -> labels::Bars<X, Y>;
// .format(spec) (a core format specifier through the figure locale) .padding(pt) .center()
/// One label per slice at `arc.centroid` of the slice between `inner` and `outer`.
pub fn pie<K: Display>(slices: &[(K, f64)], inner: f64, outer: f64) -> labels::Pie<K>;
// .percent(bool) (`.0%` of the total) .radius(r) to place them on another ring
```

`.radius(r)` places pie labels on another ring inside the outer radius;
labels outside the pie are not in v0.1. Labels use the figure's
`Locale` (neutral default: U+2212 minus), so `−1.0` differs from
matplotlib's ASCII `-1.0` by design.

##### Size and color legends (`helpers::legend`)

```rust
/// Entries are the size scale's ticks(count) inside its domain, each a
/// dot of that radius: d3's tick choice, not matplotlib's MaxNLocator.
pub fn sizes<S: Kind<Domain = f64, Output = f64> + Ticks<Domain = f64>>(handle: ScaleRef<S>) -> legend::Sizes<S>;   // .count(n) default 4, .values(v)
/// Entries are ticks(count) of a continuous color handle, as dots.
pub fn colors<C: Paint<Domain = f64> + Ticks<Domain = f64>>(handle: ScaleRef<C>) -> legend::Colors<C>;            // .count(n) default 5
```

Both implement `legend::Guide`, so they go into a slot or an inset
like any legend.

##### Axis defaults (`helpers::axes`) and `Axis::minor`

```rust
/// Minor ticks on; offset and multiplier notation, already the
/// figure axis's default (Phase 16), stays on.
pub fn defaults<'a, S: Ticks>(axis: Axis<'a, S>) -> Axis<'a, S>;

// figure/src/axis.rs
impl<'a, S> Axis<'a, S> {
    /// Draws core's `Ticks::minor` values at the theme's minor tick size
    /// and width (4 pt / 1.0 pt at notebook), without labels.
    pub fn minor(self, on: bool) -> Self;
}
```

Offset and multiplier notation are on by default since Phase 16
(`.offset(false)` turns them off); `defaults` leaves them on and turns
minor ticks on. `Axis::minor` is the one figure-layer addition of this
phase.

**Decisions this phase may meet**

- `mark::Grid` carries no row or column keys, so the heatmap cannot
  know when to draw band axes — recommended: `chart::Heatmap::keys(rows:
  Vec<String>, cols: Vec<String>)`, band axes when set and linear axes
  over the grid extent otherwise; record it as a decision.

#### Steps

1. **Labels and legends** (`helpers/labels.rs`, `helpers/legend.rs`,
   tests in `figure/tests/main/helpers/labels.rs`).
2. **Axis defaults and minor ticks** (`helpers/axes.rs`,
   `figure/src/axis.rs`, tests in `figure/tests/main/helpers/axes.rs`):
   minor ticks drawn at the theme's size and width without labels, the
   symlog decade minors. **Cut here** if the phase takes two sessions;
   the handoff records "Step 2 done".
3. **Presets: line, scatter, bar** (`helpers/chart.rs`,
   `helpers/chart/{line,scatter,bar}.rs`, tests in
   `figure/tests/main/helpers/presets.rs`): one test per preset that
   counts marks and checks `fig.check`; the stack-offset spot checks;
   the line sketch of Helpers as a doctest.
4. **Presets: area, pie and donut, heatmap**
   (`helpers/chart/{area,pie,heatmap}.rs`, tests,
   `dev/examples/examples/presets.rs`): reference renders of every
   preset (SVG, and PNG with `png`).

#### Exit criteria

1. `cargo test -p rustd3-figure --features helpers --test main --
   presets labels axes` passes, including every spot check.
2. `cargo run -p rustd3-examples --example presets` renders each preset
   (SVG, and PNG with `png`), and the preset tests assert `fig.check` is
   empty for each at its default size.
3. `cargo test -p rustd3-figure --features helpers,png --doc` passes:
   a doc example for every public helper, among them the line sketch of
   Helpers.
4. `scripts/check.sh quick` per commit; `full` at the end.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| `shape::pie([3, 1, 2])` | slice 0: index 0, 0 … π; slice 1: index 2, 5.235988 … 2π; slice 2: index 1, π … 5.235988 | d3 in Node |
| Donut 40–80, centroids of slices 0, 1, 2 | (60, 0), (−30, −51.961524), (−51.961524, 30) | d3; Shapes and PathContext `(60.0, 0.0)` |
| Pie 0–100, label centroids | (50, 0), (−25, −43.301270), (−43.301270, 25) | d3 in Node |
| Pie preset, plot area 600 × 400 px at notebook | outer radius 200 − em(0.5) = 200 − 7.333 = 192.667 px (em = 11 pt = 14.667 px) | radius rule |
| `bar(…).mode(Diverging)` with series a, b, c over two keys, a = [3, −2], b = [−1, 4], c = [2, 1] (rows `(k0, [3, −1, 2])`, `(k1, [−2, 4, 1])`) | a: [0, 3], [−2, 0]; b: [−1, 0], [0, 4]; c: [3, 5], [4, 5] | d3 `stackOffsetDiverging` |
| `bar(…).mode(Stacked)`, series apples [3, 2, 1], bananas [1, 1, 4] | bananas spans [[3, 4], [2, 3], [1, 5]]; two `bar_y_spans` marks | Shapes and PathContext |
| `labels::bars` of [3, −1, 2.5], format `.1f`, padding 3 pt | "3.0" 4 px above 3; "−1.0" 4 px below −1; "2.5" above | matplotlib `bar_label` (xyann ±3 pt) |
| `legend::sizes` on sizes 1 … 100, count 4 | 20, 40, 60, 80, 100 (matplotlib `num=4`: 25, 50, 75, 100) | d3 `ticks(1, 100, 4)` |
| `legend::colors` on 1 … 100, count 5 | 20, 40, 60, 80, 100 (matplotlib agrees) | d3; matplotlib `legend_elements` |
| `chart::line(rec.series(0).expect("channel 0")).y_scale(scale::symlog().constant(1.0)).minor(true)` | compiles; symlog ticks with decade minors; `save_png(path, 2.0)` writes a 1280 × 960 PNG | Helpers; default size 640 × 480 |

#### Not in this phase

- Polar axes, inset zoom, the streamgraph preset and the concise-date
  offset label (Later).
- Pie labels outside the pie; equal-aspect tracks (out of scope).
- Box zoom, linked cursors.
- Facet grammars and `hue` semantics (non-goals).
- Release work: facade, README, publishing (Phase 40).

## M5 — Release

The facade, documentation and release preparation.

### Phase 40 — Facade, documentation and release preparation

*M5 Release · builds on every v0.1 phase (31 if built) · about 650 lines, 5 commits*

#### Goal

The workspace is ready for the user to publish v0.1.0 of five crates:
the `rustd3` facade re-exports the library by feature (no `iced`
feature until iced 0.15 is on crates.io), every published crate has
documentation, a README, metadata including `documentation`, an
explicit `include` list and its licence files, a root `CHANGELOG.md`
lists what ships and what is Later, and `cargo publish --dry-run
--workspace --exclude rustd3-iced` succeeds with no lockfile pointing
at a git source. MSRV is stated and justified. A release checklist
tells the user exactly what to run; the agent never runs `cargo
publish` without `--dry-run` (Appendix A).

#### Builds on

- Every earlier phase done and `scripts/check.sh full` green.
- Phase 1: the workspace layout, `[workspace.package]`, workspace
  dependencies (the `rustd3-figure` entry sets `default-features =
  false`), `LICENSE-MIT`, `LICENSE-APACHE`, `core/LICENSE-d3`,
  `fonts/LICENSE` (a summary plus the Bitstream Vera and GUST texts)
  with `license-file = "LICENSE"` and the separate font licence files,
  `rust-version = "1.97"`, `publish = false` on `rustd3-iced` and every
  `dev/` crate, root `exclude = [".claude"]`, `autotests = false` on
  every crate (each crate's `main` integration-test target comes with
  its first integration test).
- Phase 24: the derive re-exported beside the trait as
  `rustd3_core::interpolate::Interpolate`.
- Phase 26: the `fmt!` macro. Phase 27: `rustd3_core::jiff`.
- `core/src/lib.rs`'s public modules and `figure/Cargo.toml`'s features
  as the earlier phases left them.
- `.claude/DECISIONS.md` and every `decisions/phase-*.md` entry that
  mentions publishing, features or licences.
- Before starting: `cargo search rustd3` (see Spot checks).

#### Design

##### What is published

| Package | Publish | Why |
| --- | --- | --- |
| `rustd3-fonts` | yes | figure depends on it |
| `rustd3-derive` | yes | core's `derive` feature |
| `rustd3-core` | yes | |
| `rustd3-figure` | yes | |
| `rustd3` (facade, repository root) | yes | |
| `rustd3-iced` | no (`publish = false`) | git dependency on iced; also passed as `--exclude` |
| `rustd3-testkit`, `-harness`, `-examples`, `-bench` | no (`publish = false`) | dev crates |

All five publish as `0.1.0`. Workspace dependencies on workspace crates
carry both `path` and `version = "0.1.0"`; dev-dependencies on
`rustd3-testkit` stay path-only (cargo strips a path-only dev-dependency
on an unpublished crate from the published manifest; with a `version`
publishing would fail).

##### The facade (`src/lib.rs`, root `Cargo.toml`)

```toml
[package]
name = "rustd3"
include = ["src/**/*.rs", "Cargo.toml", "README.md", "CHANGELOG.md", "LICENSE-MIT", "LICENSE-APACHE"]
readme = "README.md"
documentation = "https://docs.rs/rustd3"

[dependencies]
rustd3-core.workspace = true
rustd3-figure = { workspace = true, optional = true }

[features]
default = ["fonts"]                      # bundled fonts on by default
fonts = ["rustd3-figure?/fonts"]         # weak: does not turn `figure` on
derive = ["rustd3-core/derive"]
figure = ["dep:rustd3-figure"]
# one facade feature per rustd3-figure feature, same name, each enabling `figure`:
raster = ["figure", "rustd3-figure/raster"]
png = ["figure", "rustd3-figure/png"]
gif = ["figure", "rustd3-figure/gif"]
pdf = ["figure", "rustd3-figure/pdf"]
tiny-skia = ["figure", "rustd3-figure/tiny-skia"]
helpers = ["figure", "rustd3-figure/helpers"]

[package.metadata.docs.rs]
all-features = true
```

The figure features forwarded are exactly `fonts`, `raster`, `png`,
`gif`, `pdf`, `tiny-skia` and `helpers`: read `figure/Cargo.toml` and
check the list one to one. The weak `fonts` feature works only because
Phase 1's workspace entry for `rustd3-figure` sets `default-features =
false`; the facade's own dependency line must not add
`default-features` (edition 2024 rejects it over an inheriting entry).

```rust
//! (crate docs: the README, so its examples are doctests)
#![doc = include_str!("../README.md")]

pub use rustd3_core::{anim, array, axis, color, ease, format, interact, interpolate,
    join, locale, path, scale, scheme, series, shape, ticks, time};
pub use rustd3_core::jiff;                     // core's re-export
pub use rustd3_core::fmt;                      // the `fmt!` macro
#[cfg(feature = "figure")]
pub use rustd3_figure as figure;
```

The module list mirrors exactly what `core/src/lib.rs` declares
public. The derive needs no line: it is re-exported beside the trait,
so `rustd3::interpolate::Interpolate` brings both when `derive` is on.
`rustd3::figure` keeps figure's own paths (`rustd3::figure::Figure`,
`rustd3::figure::mark::dots`), while `rustd3::axis::bottom` stays
core's axis (see Architecture). `pub use rustd3_figure as figure`
renames a crate to a module, the facade pattern: the style guide's
no-alias rule targets name clashes at call sites, and the alternative,
`pub mod figure { pub use rustd3_figure::*; }`, is a glob import, which
the guide also forbids.

##### Include lists and licences per package

| Package | `include` | Licence field |
| --- | --- | --- |
| `rustd3` | `src/**/*.rs`, `Cargo.toml`, `README.md`, `CHANGELOG.md`, `LICENSE-MIT`, `LICENSE-APACHE` | `license = "MIT OR Apache-2.0"` |
| `rustd3-core` | `src/**/*.rs`, `Cargo.toml`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-d3` | `license = "MIT OR Apache-2.0"`; `LICENSE-d3` carries d3's ISC notice for the ported code and generated tables |
| `rustd3-derive` | `src/**/*.rs`, `Cargo.toml`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE` | `license = "MIT OR Apache-2.0"` |
| `rustd3-figure` | `src/**/*.rs`, `Cargo.toml`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE` (+ `LICENSE-d3` if any figure code is ported from d3) | `license = "MIT OR Apache-2.0"` |
| `rustd3-fonts` | `src/**/*.rs`, `Cargo.toml`, `README.md`, `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-DejaVu`, `LICENSE-NewCMMath.txt`, `DejaVuSans.ttf`, `DejaVuSans-Bold.ttf`, `NewCMMath-Regular.otf` | `license-file = "LICENSE"` (Phase 1) |

- `LICENSE-MIT` and `LICENSE-APACHE` inside each package are symlinks
  to the root files (tracked by git, packaged as regular files).
- The fonts licence is settled by Phase 1; this phase checks only the
  include list. crates.io shows a `license-file` package as
  "non-standard"; an SPDX expression is Later.
- Font `include_bytes!` paths stay inside `fonts/`, and any non-Rust
  file another crate pulls in with `include_str!` or `include_bytes!`
  joins its `include` list (the dry run's verification build fails on a
  file outside the package or missing from it).
- A root package without `include` would ship everything under the
  root that is not a nested package, `.claude/` included; `exclude =
  [".claude"]` under `[workspace]` does not affect packaging (measured
  with cargo 1.97.0 in a scratch workspace).

##### Metadata (`[workspace.package]` and per package)

`version = "0.1.0"`, `edition = "2024"`, `rust-version = "1.97"`,
`license` as above, `repository` (from the user, checklist item 2),
`documentation = "https://docs.rs/<package>"` per published package,
`description` (one sentence each), `keywords` (≤ 5): facade and figure
`["d3", "plot", "chart", "visualization", "dataviz"]`, core `["d3",
"scale", "chart", "visualization", "time"]`; `categories`: facade and
figure `["visualization", "graphics", "rendering"]`, core
`["visualization", "science", "date-and-time"]`, fonts
`["visualization"]`, derive
`["development-tools::procedural-macro-helpers"]`; `readme =
"README.md"` per package; `[package.metadata.docs.rs] all-features =
true` on core, figure and the facade.

##### Documentation

- **Root `README.md`** (the facade's crate doc): what rustd3 is (from
  Summary, two paragraphs); the two levels (core and figure); the
  feature table (from Architecture, with `iced` marked "rustd3-iced
  from git until iced 0.15 is on crates.io"); the usage snippets; a
  scale example (`x.apply(25.0) // 240`, `x.invert(480.0) // 50`); a
  figure example writing an SVG (PNG behind `png`); outputs; MSRV;
  licence and credits (d3, ISC, Mike Bostock; seaborn's palette and
  context values, BSD-3-Clause; typst; the fonts).
- **Doctests under every feature set.** `check.sh full` tests the
  facade with `--no-default-features` and `--all-features`, and
  doctests see the crate's features as `cfg`. Every README block that
  uses `rustd3::figure` wraps its body: `# #[cfg(feature = "figure")]
  fn main() { … }` plus `# #[cfg(not(feature = "figure"))] fn main()
  {}`, and the PNG line the same way with `png`. File writes use
  `no_run`; `ignore` is not allowed.
- **The iced snippet** cannot compile against the facade, which has no
  rustd3-iced dependency: it is a ```` ```toml ```` block (the
  `Cargo.toml` lines with the git dependency) followed by a
  ```` ```text ```` block for the Rust, never `ignore`.
- **Per-package `README.md`**: three to ten lines each, pointing to the
  facade README and docs.rs.
- **Crate docs** (`//!`) in every published `lib.rs`: purpose, a
  runnable example, the feature list with what each enables (figure).
- **`CHANGELOG.md`** (root, in the facade's `include`): a `## 0.1.0`
  entry listing what ships (the five crates and rustd3-iced from git,
  the figure features, the outputs, the widget) and what is Later, from
  the Later section.

##### MSRV

Stay at `rust-version = "1.97"` unless the user supplies an older
toolchain. Floors from dependency metadata (`~/.cargo/registry`): typst
0.15.1, typst-layout 0.15.1 and krilla 0.8.2 need 1.92; vello_cpu 0.3.0
1.89; kurbo 0.13.1 and peniko 0.6.1 1.85; thiserror 2.0.21 1.77; png
0.18.1 1.73; syn 2.0.119 1.71; jiff 0.2.38 1.70; edition 2024 itself
1.85. So figure and the facade with `figure` cannot go below 1.92, and
core alone cannot go below 1.85 (edition 2024). The lockfile may hold
transitive crates with higher floors; list them with:

```sh
cargo metadata --format-version 1 | jq -r '.packages[] | select(.rust_version != null) | "\(.rust_version) \(.name) \(.version)"' | sort -V | tail
```

Lowering is then: the user installs the toolchain; `cargo +1.92.0
check -p rustd3-figure --all-features`, `cargo +1.92.0 test -p
rustd3-core -p rustd3-figure --all-features`; set `rust-version` per
package to the checked value.

##### `scripts/check.sh release`

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p rustd3 -p rustd3-core -p rustd3-figure -p rustd3-fonts -p rustd3-derive --all-features
cargo test -p rustd3 --all-features --doc
cargo test -p rustd3 --no-default-features --doc
for p in rustd3-fonts rustd3-derive rustd3-core rustd3-figure rustd3; do
  cargo package --list -p "$p" > "target/package-$p.txt"
  if grep -E '^(\.claude/|dev/|target/)' "target/package-$p.txt"; then
    echo "release: $p packages files it must not" >&2; exit 1
  fi
done
git diff --quiet && git diff --cached --quiet   # clean tree required
cargo publish --dry-run --workspace --exclude rustd3-iced
for p in rustd3-fonts rustd3-derive rustd3-core rustd3-figure rustd3; do
  n=$(tar -xzOf "target/package/$p-0.1.0.crate" "$p-0.1.0/Cargo.lock" | grep -c 'git+' || true)
  if [ "$n" != 0 ]; then
    echo "release: $p's packaged Cargo.lock has a git source" >&2; exit 1
  fi
done
```

`cargo publish --dry-run --workspace` skips `publish = false` members
without error, packages the rest in dependency order and verifies each
against a temporary registry of its unpublished workspace dependencies;
`--exclude` composes with it. It fails on a dirty git tree, including
untracked files under `target/package/` when `target/` is not ignored
(measured with cargo 1.97.0). Network access goes to crates.io only.

The dry run's tail lists `Uploading rustd3-fonts`, `rustd3-derive`,
`rustd3-core`, `rustd3-figure`, `rustd3` (fonts and derive may swap),
each followed by `aborting upload due to dry run`.

##### The release checklist (the content of `RELEASING.md`)

1. `cargo search rustd3` and `cargo info rustd3`: no crate named
   `rustd3*` may exist (on 2026-10-17 the search listed none and
   `cargo info rustd3` reported "could not find").
2. Give the repository URL; set `repository` (and `homepage` if any) in
   `[workspace.package]`; make the repository public.
3. A crates.io account with a verified email; `cargo login` with a token
   that may publish new crates.
4. On a clean clone of the release commit: `scripts/check.sh full`, then
   `scripts/check.sh release`. Read the five `target/package-*.txt`
   lists once.
5. Optional MSRV lowering (install 1.92.0, run the checks of the MSRV
   section).
6. `git tag v0.1.0 && git push origin v0.1.0`.
7. `cargo publish --workspace --exclude rustd3-iced`: publishes
   rustd3-fonts and rustd3-derive, then rustd3-core, rustd3-figure and
   rustd3, waiting for each in the index. Publishing is permanent; a bad
   version can only be yanked.
8. Check docs.rs for all five crates (all features), and in a scratch
   project `cargo add rustd3 --features figure,png` and build the
   README's figure example.
9. Leave `rustd3-iced` on git. When iced 0.15 is on crates.io: switch
   its iced dependencies from the git revision to the version, rerun the
   iced tests on both backends, set `publish = true`, add the facade
   feature `iced`, and release 0.1.1.
10. Optional: `cargo owner --add <user>` for co-maintainers.

**Decisions this phase may meet**

- The user supplies an older toolchain — lower `rust-version` per
  package to the checked value; record it as a decision.
- A packaged `Cargo.lock` contains a `git+` source — stop; record it as
  a decision before anything else changes.

#### Steps

1. **Manifests and changelog** (`Cargo.toml`,
   `{core,derive,figure,fonts}/Cargo.toml`, licence symlinks,
   `CHANGELOG.md`): metadata, `include` lists, version requirements on
   workspace path dependencies, path-only testkit dev-dependencies.
   Verify: `cargo package --list -p <crate>` for each crate.
2. **Facade** (`src/lib.rs`, root features). Verify: `cargo check -p
   rustd3` with no features, each feature alone, and all features; the
   two `cargo tree` checks of the weak `fonts` feature.
3. **Docs** (`README.md`, `{core,derive,figure,fonts}/README.md`, crate
   docs in every published `lib.rs`). Verify: exit criterion 1.
4. **Release script and dry run** (`scripts/check.sh` gains `release`):
   commit, then run it on the clean tree. Verify: exit criteria 3 and 4.
5. **Checklist** (`RELEASING.md`, repository root, not packaged) with
   the MSRV note; final `scripts/check.sh full`.

#### Exit criteria

1. `RUSTDOCFLAGS="-D warnings" cargo doc -p rustd3 --all-features`
   builds; `cargo test -p rustd3 --all-features --doc` and `cargo test
   -p rustd3 --no-default-features --doc` pass (the README is the
   facade's crate doc, so its Rust examples run).
2. For each of `rustd3-fonts`, `rustd3-derive`, `rustd3-core`,
   `rustd3-figure`, `rustd3`, `cargo package --list -p <crate>` prints
   exactly the files of the include table (plus cargo's own
   `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`,
   `.cargo_vcs_info.json`) and no path under `.claude/`, `dev/` or
   `target/`.
3. On a clean tree, `cargo publish --dry-run --workspace --exclude
   rustd3-iced` packages, verifies and "uploads" (aborted) exactly those
   five crates, in dependency order. The only warnings allowed are a
   missing `repository` (until the user gives one) and cargo's
   per-crate note that the `main` integration-test target is not
   included in the package (`tests/` lies outside `include`).
4. For each packaged crate, `tar -xzOf target/package/<crate>-0.1.0.crate
   <crate>-0.1.0/Cargo.lock | grep -c 'git+'` prints 0 (no git source,
   iced in particular).
5. `scripts/check.sh release` runs criteria 1–4 in one command.
6. `RELEASING.md` holds the release checklist of the Design section, with the MSRV note.
7. `scripts/check.sh full` green.

#### Spot checks

| Input | Expected | Source |
| --- | --- | --- |
| `cargo search rustd3` | no line naming a `rustd3` crate | run 2026-10-17 |
| `cargo info rustd3` | `error: could not find rustd3 in registry` | run 2026-10-17 |
| `cargo package --list -p rustd3` | `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `LICENSE-APACHE`, `LICENSE-MIT`, `README.md`, `src/lib.rs` | include list |
| Same without `include` | lists `.claude/…` files | measured in a scratch workspace |
| `cargo package -p rustd3-fonts` size | about 1.5 MiB compressed (< 10 MB): DejaVuSans.ttf 759,720 B, DejaVuSans-Bold.ttf 708,920 B, NewCMMath-Regular.otf 1,306,268 B, with licences 1,603,734 B tarred | `proto/fonts` |
| `cargo publish --dry-run --workspace` with a `publish = false` member | member skipped, no error | cargo 1.97.0 |
| Published manifest of a crate with a path-only testkit dev-dependency | no `rustd3-testkit` entry | cargo 1.97.0 |
| `cargo tree -p rustd3 --no-default-features --features figure -e features -i rustd3-fonts` | `rustd3-fonts` is not in the tree; with `--features figure,fonts` it is | weak `fonts` feature |
| `tar -xzOf target/package/rustd3-figure-0.1.0.crate rustd3-figure-0.1.0/Cargo.lock \| grep -c 'git+'` | 0 | no git source in packaged lockfiles |
| `rustd3::scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]).apply(25.0)` (README doctest) | 240 | Scales |
| `rustd3::axis::bottom` | core's axis; `rustd3::figure::axis::bottom` is the figure axis | Architecture |
| MSRV floors | figure ≥ 1.92 (typst, typst-layout, krilla); core ≥ 1.85 (edition 2024) | dependency `rust-version` |

#### Not in this phase

- `cargo publish` itself (the user runs it), tags pushed by the agent,
  `cargo owner` changes.
- Publishing `rustd3-iced` or a facade `iced` feature (until iced 0.15
  is on crates.io).
- New features or API changes beyond the facade; wasm (a non-goal).
- Changing licences beyond Phase 1's without the user's choice; an SPDX
  expression for the fonts crate (Later).

## Later (0.2)

These have a home in the design and are built after v0.1 ships. Each
is listed with its estimated cost, so the order can be set by value.
No v0.1 phase builds any of them. (Phase 31 is not on this list: it is
a conditional v0.1 phase, built when the benchmark verdict asks.)

| Item | Cost | Why it waits |
| --- | --- | --- |
| anyrender adapter (vello on the GPU, vello_hybrid, Skia) | ~100 lines, +2 0.x crates | no v0.1 consumer |
| Geometric clip cutting in `lower` (i_overlay) | ~350 lines, +i_overlay | no built-in backend lacks path clips |
| TOML themes | ~150 lines, +serde, +toml | builders cover v0.1 |
| Two-dimensional brush, d3-brush's keyboard modes, `Plot::on_region` | ~300 lines and both-backend tests | the one-dimensional brushes cover both use cases |
| Parsing d3-time-format strings (`time::Parser`) | ~400 lines, 236 d3 assertions | jiff parses ISO 8601 and its own formats |
| `shape::link` and d3's stroke symbols | ~200 lines | links serve hierarchies (out of scope); markers use the fill symbols |
| Smooth zoom interpolation (`interpolateZoom`) | ~80 lines | no consumer |
| d3-interpolate's `discrete`, closed-basis and array interpolators | ~120 lines | no consumer beyond d3's own tests |
| d3-array extras: `fsum`, `fcumsum`, `mode`, `bisect_center`, `min_index`, `max_index`, `cumsum` | ~250 lines | no chart uses them |
| matplotlib's minor-tick rules for logit and asinh | ~150 lines | the default subdivision serves v0.1 |
| Nested grids (spans are in v0.1) | ~200 lines | no v0.1 figure nests |
| Level-preserving index surgery in `drop_front` | ~150 lines | rebuilding the index is correct and fast enough |
| Polar axes | ~300 lines and a clip | low value per cost |
| Inset zoom (`Figure::inset`, indicator and connectors) | ~300 lines and a layout change | low value per cost |
| Streamgraph preset | ~100 lines | low value per cost |
| The concise-date offset label | ~200 lines with its layout reach | the ISO ladder already labels time axes concisely |
| Hexbin | ~200 lines and a fixture | dots with thinning cover large scatters |
| Stem and event plots | ~120 lines | low value per cost |
| An SPDX licence expression for rustd3-fonts | — | `license-file` is accepted by crates.io |
| LaTeX math input through mitex | +1 crate and its typst package | typst markup covers math; mitex's output needs its own typst package |
| fast_image_resize for raster heatmaps | +1 crate | nearest sampling of the image mark covers v0.1's heatmaps |

## Appendix A — Dispatch preamble

Paste as the prompt for a phase agent, with `{N}`, `{NN}` (zero-padded)
and `{title}` filled in.

> You are implementing **Phase {N} — {title}** of rustd3. The
> repository is the current directory; the specification is
> `.claude/map/rustd3/rustd3.md`.
>
> **Read first, in this order:**
> 1. `CLAUDE.md`
> 2. `.claude/RUST_STYLE.md` and `.claude/OPAQUE.md`
> 3. `.claude/DECISIONS.md` and the `.claude/decisions/phase-*.md`
>    files that exist for the phases yours builds on (rustd3.md, "Order
>    of work")
> 4. rustd3.md's Implementation front matter, from "How to use this
>    part" to "Order of work"
> 5. rustd3.md's section "Phase {N}", then the Part I sections it names
> 6. The `proto/` files it cites
>
> If the newest `SESSION_HANDOFF-NN.md` says this phase is half done,
> continue from the step it names.
>
> **You may run:** `scripts/check.sh quick` (and `full` at the end, in
> the background); `cargo fmt|clippy|test|doc|build|run` on workspace
> packages; `cargo run -p rustd3-harness -- --check` (and `--bless` /
> `--bless-floors` under the rule below); `node` on scripts under
> `dev/`, including d3 itself through `dev/harness/translate/d3-resolve.mjs`
> to compute reference values; `python3 dev/fixtures/*.py` when your
> phase says so; read-only inspection of `~/.cargo/registry` and
> `~/.cargo/git`.
>
> **Do not run:** `cargo publish` (only `--dry-run`, Phase 40); `cargo
> update`, or dependency changes to any manifest, outside Phases 1 and
> 11; `cargo install`, `rustup`, `npm install`, `pip install` or any
> other installation (ask instead); `git push`; network access other
> than crates.io and the pinned iced revision with its git
> dependencies; `--bless`, `--bless-floors` or `RUSTD3_BLESS=1` without
> reading the diff and stating in the commit body why the new output is
> right; the windowed benchmark (use `--headless`; the user runs the
> windowed one). Never edit vendored d3 files, generated files by hand,
> or rustd3.md.
>
> **Self-review before each commit:**
> - No `mod.rs`; one semantic type per module; module paths, not
>   composite names (the Conventions list excepted).
> - No `unwrap()` in library code; `expect("invariant: …")` only for
>   true invariants; `thiserror` errors carrying the rejected value.
> - No wildcard `_ =>` arm on enums this workspace defines, except the
>   one the compiler requires on `#[non_exhaustive]` enums from another
>   crate.
> - Doc comments on every public item; no comments restating code;
>   nothing copied from `proto/` without bringing it to this style.
> - No `todo!()`, `unimplemented!()`, placeholders or dead code; no
>   `#[allow(…)]` without a reason.
> - Tests named as sentences; the phase's spot checks asserted;
>   compile-fail examples have passing twins.
> - `f64` geometry; `f32` only where Conventions allows; no ambient
>   clock or zone.
> - Changing or removing a public item of Phases 1–10: a decision, and
>   every call site updated in the same commit.
>
> **Forbidden:** trait abstractions over closed sets (curves, commands,
> intervals, backends); shims or deprecated aliases; `Box<dyn Fn +
> 'static>` accessors in view-time generators; `Rc<RefCell<…>>` object
> graphs in the figure model; global mutable state beyond Conventions
> item 5; anything listed under Later (0.2); AI attribution.
>
> **Commits:** conventional commits, `type(scope): summary`, scope =
> crate; one logical change per commit; each commit passes `check.sh
> quick`; the phase's last commit passes `check.sh full`.
>
> **If you hit a design decision rustd3.md does not make, or two of
> its statements cannot both hold, STOP.** Write `D{N}.{k}` in
> `.claude/decisions/phase-{NN}.md` (issue, options table,
> recommendation) and report instead of choosing silently.
>
> **Report at the end:** commits; each exit criterion with the command
> that shows it; harness counts and floors before and after; decisions
> written; anything left for the next session or phase.

## Appendix B — Resume

A fresh session picks the project up like this:

1. Read `goon.yaml`, then the newest `SESSION_HANDOFF-NN.md`.
2. The next phase is the lowest-numbered one that is not done; a
   half-done phase continues at the step its handoff names. Phase 31 is
   next only when the verdict in `decisions/phase-22.md` asks for it
   (as soon as it does, even if Phases 32–39 have started); a verdict
   that says the image path suffices marks it skipped; with no verdict
   yet, it is passed over until one arrives (Order of work).
3. Run `scripts/check.sh quick`. A red check is fixed first.
4. Work through "Before a phase starts" (see Order of work).
5. Dispatch with Appendix A, or implement in the session under its
   rules.
6. After the phase: record harness counts and floors in the handoff,
   then write the next `SESSION_HANDOFF-NN.md`.
