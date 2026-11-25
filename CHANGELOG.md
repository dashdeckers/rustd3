# Changelog

## 0.1.0

The first release.

### Crates

- `rustd3`: the facade. It re-exports the core's modules and, behind the
  `figure` feature, `rustd3-figure` as `rustd3::figure`, so an application
  adds one dependency.
- `rustd3-core`: d3's vocabulary, renderer-agnostic. It follows d3
  7.9.0's semantics and is checked by tests derived from d3's own test
  suites: 5,321 assertions in 1,593 tests, each naming d3's file and
  line.
- `rustd3-figure`: the figure layer and its outputs.
- `rustd3-fonts`: DejaVu Sans (regular and bold) and New Computer Modern
  Math, bundled.
- `rustd3-derive`: `#[derive(Interpolate)]`, behind the core's `derive`
  feature.
- `rustd3-iced`, from git, not on crates.io: it depends on iced at
  revision 98bd48f, and crates.io does not accept git dependencies. It is
  published, and the facade gains an `iced` feature, once iced 0.15 is on
  crates.io.

### Core

- Scales: linear, pow, sqrt, log, symlog, logit, asinh, user transforms
  and `AnyContinuous` for a transform chosen at runtime; time and UTC
  scales on jiff timestamps; band, point, ordinal and identity; sequential
  and diverging color scales; quantize, quantile and threshold. Ticks,
  nice domains, tick formats and tick rules; d3's axes, drawn into any
  `PathContext`.
- Shapes: lines and areas with d3's curves, arcs and pies, stacks and
  symbols; `PathContext`, d3's canvas path interface, and `SvgPath`.
- Number formats with d3's locales, and the `fmt!` macro that checks a
  specifier at compile time. Time intervals, time formats and locales,
  ISO 8601 formatting and parsing; the zone is always a parameter.
- Color: RGB, HSL, Lab, HCL and Cubehelix with d3's rules; interpolators,
  easing, and every d3-scale-chromatic scheme, exact in `f64`.
- Arrays: extent, sums, means, medians, quantiles, variance, deviation,
  bisection and binning.
- `series::Recording`: channels of sampled data on one clock, with an M4
  index that grows with each append, for thinning long series.
- Keyed joins; zoom, one-dimensional brushes and drag as state machines
  over pointer input; animation sampled from explicit instants,
  `anim::Instant` (`web_time`'s: `std::time::Instant` on native targets,
  a clock that works on wasm).
- `array::max_index` and `array::min_index` are d3's `maxIndex` and
  `minIndex`: the position of the greatest and of the least value.
- `locale::Locale::ladder(l)` puts a locale's dates on another ladder in
  one call.
- `shape::Symbol` has d3's seven stroke symbols (`Symbol::STROKE`: plus,
  times, asterisk and stroke-sized circle, triangle, square and diamond)
  beside its fill symbols, and Observable Plot's hexagon.
  `Symbol::is_open` tells the three that are lines without an inside.

### Figure

- A `Figure` of panels on a grid with spans, sharing scales through typed
  handles; fitting to all data, a live window or the visible data.
- Marks: dots, line, area, ribbon, bars and spans, rect, histogram,
  densities, rules, image (raster heatmaps) and notes; per-point color and
  size channels; hatching. A density over more than 10,000 values is
  estimated from linearly binned values convolved with the kernel
  (`stats::Kde::binned`), within a relative 1e-3 of the exact sum and
  about a hundred times faster on a million values; below, it is exact.
- Axes, legends and colorbars; layout from text measured with typst;
  typst text and math in every label; themes with seaborn's styles,
  contexts and palettes; `fig.check` for figures that will not read
  well.
- Outputs: SVG always; PNG (`png`), GIF (`gif`) and PDF (`pdf`); the
  vello_cpu rasterizer (`raster`) and the tiny-skia fallback
  (`tiny-skia`); the drawing list for backends of your own.
- `helpers`: one-call charts (line, scatter, bar, area, pie, heatmap),
  statistical charts (histogram, KDE and violin, box plot, ECDF, error
  bars), value labels, legend helpers and `subplots`.
- Legend squares repeat the opacity their key's areas, ribbons and
  densities fill at, as matplotlib's handles do; a key several marks
  fill takes the most opaque, so squares of opaque marks stay opaque.
- `helpers::chart::Line::key` and `Area::keys` name the series their
  legends show; a name given twice is `check::Issue::DuplicateKey`.
- `Theme::panel(c)` sets the color behind the plot areas, so a figure can
  be dark throughout; `Theme::get_panel` reads it.
- `rect` takes edges of any position scale's domain: a span of time is a
  pair of timestamps.
- `Axis::group(f)` adds a second level of labels: each run of neighboring
  ticks with one group name is labeled once, past the tick labels and
  under a line that spans the run. `axis::Nested(outer, inner)` is a key
  of two levels for band and point scales.
- `Note::avoid(Avoid::Step)` moves a note up or down by its height when
  its box would overlap a note drawn before it, and `Avoid::Drop` leaves
  it out. Notes that do not avoid are drawn where they are put.
- `Dots::symbol` draws dots as any of d3's symbols or Observable Plot's
  hexagon at the size of the circle they replace, and `Dots::hollow`
  draws their outlines one patch edge wide; d3's line symbols are always
  outlines. Legend swatches repeat both (`legend::Swatch::Symbol`).
- `Rect::stroke_width` sets the width of a rectangle's outline, 1 pt by
  default: a heatmap can box the cells it highlights.
- `Rule::label_at(mark::Place::Start | Middle | End)` chooses where along
  a rule its label sits, and `Rule::label_offset(dx, dy)` moves it, so a
  threshold can be named clear of the marks.
- Labels keep typst's clips (`box(clip: true)`): what typst clips is
  drawn in a group clipped to its path, in every output.
- `text::draw(markup, &fonts)` draws a typst snippet as a list of its own
  size, without a label's style: an icon in one call. `typst` is
  re-exported, so a crate that compiles typst itself uses the same
  version.
- `Note::size(px)` and `Note::bold(true)` set a note's text size and
  weight without typst markup.

### iced (`rustd3-iced`, from git)

- `IcedPath` and `pointer::Tracker` for apps that draw with iced's canvas.
- The `Plot` widget (`figure` feature): a figure shown as the pixels its
  PNG has; wheel zoom and drag pan per scale handle, live data, hover
  with a crosshair and highlight, one-dimensional brushing; cached tiles
  that pans move by whole device pixels, a render pool and a cost model
  fitted on the machine it runs on.

### Robustness

Property tests feed every scale, format, generator, interpolator,
figure and the widget NaN, infinities, −0.0, subnormals, 1e±300, empty
and single values, reversed and zero-width domains and counts up to
`usize::MAX`, and none of it panics. Where an input has no meaning, the
rule is:

- An asinh scale ticks a zero-width domain once; log minor ticks of a
  base beyond 2⁵³ end where its multiples stop moving.
- `interpolate::quantize` and `stats::Kde`'s grids give nothing for
  2³¹ samples or more, as `array::range` does.
- A grid weight that is not a finite number of zero or more weighs 1,
  and a gap that is not a finite length is `em(1.0)`, as missing ones
  are; weights split the room by their ratios however small or large.
- A figure size side that is not a finite number of zero or more is 0:
  the figure has no room, and `fig.check` reports `NoRoom`.
- A spanning panel's plot area is empty, never inverted, when negative
  gaps pull its tracks back over each other.
- `axis.outward(px)` ignores a `px` that is not finite.
- Dot radii are drawn at most three times the plot area's larger side,
  the guard band's reach.
- The grid has at most 4,096 tracks a side: a panel in a cell past them
  is not laid out or drawn, and `fig.check` reports
  `check::Issue::OutsideGrid`.
- `raster::Renderer::render` with bounds of another list is
  `raster::Error::Bounds`.
- `Resolved::get_panel_cell` returns `None` for a panel the figure lacks.
- A scale handle of another figure finds no scale: `Resolved::get_scale` and
  `mark::Ctx::x`, `y` and `get_raw` return `Option`, the marks, axes, guides,
  zooms, brushes and widget listeners given one draw or do nothing, and
  `fig.check` reports `check::Issue::ForeignHandle`; `Figure::owns` tells
  whether a handle is the figure's. `draw_resolved` resolves afresh for a
  `Resolved` of another figure. `Fit::Window` on a categorical scale fits
  as `Fit::All` and is reported (`check::Issue::CategoricalWindow`).
  `helpers::Subplots::get_x`, `get_y` and `panel` return `Option` outside the
  grid.
- The `Plot` widget takes a scale factor above 8 as 8, and renders a
  widget wider or taller than 65,408 device px at the factor that fits
  it. `Plot::magnification(z)` multiplies the factor for a host that
  draws the widget under a scaling transformation.

### API review

Names and visibility settled before the first release:

- rustd3-iced: `plot::BAND_BYTES` and `plot::NUDGE` are crate-private;
  `plot::cost`, the cost model a probe's raster samples are set against,
  is public.
- rustd3-core: `format::literal`, `fmt!`'s compile-time check, is hidden
  from the docs; `format::format(spec).is_ok()` is the run-time check.
- `helpers::stats::errorbar::Err` is `errorbar::Reach`: it no longer
  shadows the prelude's `Err`.
- `mark::Note::fill(color)` is `Note::color(color)`, as every mark names a
  literal color; `fill` and `stroke` take a palette and a key.
- `check::size(w, h)` is gone: `kurbo::Size::new(w, h)`.
- `helpers::axes::defaults(axis)` is gone: `axis.minor(true)`.
- Counts are `usize`: `path::SvgPath::with_digits(digits)` and
  `raster::Settings::threads`.
- A method that reads a stored or configured value is `get_` and its
  name, as the scales' `get_domain` and `get_range` are; predicates
  (`is_`, `has_`, `wants_`), std's names (`len`, `iter`, `as_*`) and
  computed quantities (`bandwidth`, `step`, `ticks`) keep theirs. Renamed
  in rustd3-core: `format::Specifier`'s `fill`, `align`, `sign`,
  `symbol`, `zero`, `width`, `comma`, `precision`, `trim`, `kind`;
  `format::InvalidFormat::input`, `color::ParseColorError::input`,
  `format::Format::locale`, `locale::Locale::{number, time}`,
  `ticks::Set::{values, step, count}`, `axis::Label::{x, y, text,
  anchor, dy}`, `series::Recording::{id, revision, offset}`,
  `series::Series::{recording, channel}`, `series::Columns::edges`,
  `series::m4::Extremes::{min, max}`, `anim::Animated::target`,
  `interact::BrushX::selection` and `BrushY::selection`,
  `interact::Modifiers::{shift, ctrl, alt, logo}`. In rustd3-figure:
  `mark::Ctx::{rect, theme, panel, mark, scale_factor, locale, raw,
  label}`, `Fits::{theme, locale}`, `Resolved::{scale, scale_at,
  plot_areas, zooms, axis, panel_cell, brush, plan, passes, areas,
  held}`, `text::Shaped::{width, ascent, descent, last, missing}`,
  `text::Cache::stats`, `cache::Cache::stats`, `scale::ScaleRef::index`,
  `theme::Context::scale`, `brush::Placed::{dim, over, target}`,
  `helpers::Subplots::{x, y}`. Each is `get_…`, and `Held::grids_for` is
  `Held::get_grid`. `time::Time::get_zone` and the theme's `get_color`,
  `get_gray`, `get_line_width`, `get_patch_width`, `get_density_width`
  and `get_marker` keep their names.
- rustd3-iced: `Plot` is `rustd3_iced::Plot` and `Probe` is
  `rustd3_iced::plot::Probe` only (`plot::Plot` and `plot::probe::Probe`
  are gone).
- rustd3-figure: `cache` is the module of `Cache` and its `Stats`:
  `rustd3_figure::Cache` is `cache::Cache`.
- Output options are built, not written as literals: `svg::Options`,
  `list::dump::Options` and `raster::Settings` take builders
  (`Options::default().text(svg::Text::Elements)`, `.glyphs(true)`,
  `Settings::default().threads(n).simd(s)`), as `pdf::Options` does.
- The settings enums `plot::Render`, `plot::Caching`, `raster::Simd`,
  `svg::Text`, `thin::Thin` and `Target` are `#[non_exhaustive]`.
- Sealed traits: core's `series::Data`, `series::Coordinate`,
  `scale::Ticks`, `scale::Pad` and `interact::Rescale`, and figure's
  `scale::Scale` (with it `Kind`, `Position`, `Invert`, `Paint`,
  `Fittable`, `mark::Binned`, and through `Ticks` `colorbar::Ramp`) and
  `scale::Indexed`. `PathContext`, `scale::Transform`, `Interpolate`,
  `mark::Mark`, `shape::Vertex` and `shape::Span` stay open.
- `scale::SeriesExtent` is `scale::indexed::Extent`.
- The two ends a scale maps are `ends`: core's `scale::Ticks::extent` is
  `Ticks::ends` (the range's first and last value) and figure's
  `scale::Kind::domain` is `Kind::ends` (the domain's first and last
  stop, `None` for categorical and binned scales), distinct from the
  readers `get_range` and `get_domain`.

### Benchmarks (examples, in the repository)

- rustd3's benchmark runs as examples, on any machine, from a checkout:
  `cargo run --release --example bench_all -- [--quick | --default |
  --full]` runs a level: it measures the display, runs headless without
  one, falls back to the software renderer without a GPU, skips what the
  machine cannot run, and records failed runs without stopping. It builds
  and runs the other examples as child processes.
- `cargo run --release --example bench_figure -- --quick`
  times the plotting library alone, without iced, winit or wgpu: figures
  timed from data to SVG, PDF and PNG (and GIF at `--full`) stage by stage
  through the library's own calls (build, resolve, list, each output), each
  figure in a process of its own with its peak memory, from six figures at
  `--quick` to ladders of lines and scatters, the chart presets and the
  wheel figure; `bench_all --figure-only` runs the same.
- `bench_software` (rustd3-iced) drives the `Plot` widget on the software
  renderer offscreen, every frame timed by part; `bench_gestures` runs a
  plan of the widget's cells in windows, on the native backend or the
  software renderer (`--scenario hover` for hover).
- One file to attach to an issue,
  `bench-results/bench-[figure-]<level>-<UTC stamp>.tar.gz`, with
  `report.md`, a versioned `results.json`, a CPU reference and
  `PRIVACY.md`; no host name, user name or home path goes in.
- `bench_compare A B` and `bench_report A` read a directory or an archive.
- Their machinery is `benchkit`, an unpublished workspace crate.

### Later (0.2)

Each has its place in the design and waits for a consumer or for the
value it brings:

- an anyrender adapter (vello on the GPU, vello_hybrid, Skia);
- cutting non-rectangular clips in geometry in `lower`;
- TOML themes;
- a two-dimensional brush, d3-brush's keyboard modes and
  `Plot::on_region`;
- parsing d3-time-format strings;
- `shape::link` and d3's stroke symbols;
- smooth zoom interpolation;
- d3-interpolate's `discrete`, closed-basis and array interpolators;
- d3-array's `fsum`, `fcumsum`, `mode`, `bisect_center`, `min_index`,
  `max_index` and `cumsum`;
- matplotlib's minor-tick rules for logit and asinh scales;
- nested grids;
- level-preserving index surgery in `drop_front`;
- polar axes;
- inset zoom;
- a streamgraph preset;
- the concise-date offset label;
- hexbin;
- stem and event plots;
- an SPDX licence expression for `rustd3-fonts`;
- LaTeX math input through mitex;
- fast_image_resize for raster heatmaps;
- building the `Plot` widget's drawing list off the UI thread.
