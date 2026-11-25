# The rustd3 guide

A glossary and handbook: what each concept is, which type holds it, and
how the pieces fit. The API documentation (`cargo doc --open`) has every
item's details; this guide is the map. Paths are written with the facade,
`rustd3::x` for `rustd3_core::x` and `rustd3::figure::x` for
`rustd3_figure::x`; the iced crate is `rustd3_iced`.

## 1. Two levels, six crates

rustd3 works at two levels.

- **The core level** is d3 in Rust: scales, ticks, axes, shapes, formats,
  time, color, interpolation, arrays, recordings, zoom, brush and drag.
  Everything is a plain value or a small state machine; nothing draws by
  itself. Generators draw into a `PathContext` you choose, and your code
  owns the canvas, as with d3 in a browser.
- **The figure level** describes a whole figure (panels, marks, axes,
  legends, colorbars, a title, a theme) and turns it into a drawing list
  that every output renders: SVG, PNG, PDF, GIF, or the iced `Plot`
  widget.

| Crate | Level | What |
| --- | --- | --- |
| `rustd3` | both | the facade: the core's modules at its root, `rustd3-figure` as `rustd3::figure` (feature `figure`) |
| `rustd3-core` | core | d3's scales, shapes, formats and interaction with d3's semantics; renderer-agnostic |
| `rustd3-derive` | core | `#[derive(Interpolate)]` (feature `derive`) |
| `rustd3-figure` | figure | the figure model, layout, typst text, the drawing list and its outputs |
| `rustd3-fonts` | figure | DejaVu Sans (regular, bold) and New Computer Modern Math (feature `fonts`) |
| `rustd3-iced` | both | `IcedPath` and `pointer::Tracker` for the core level; the `Plot` widget (feature `figure`) |

Two rules hold everywhere. Time zones, locales and clocks are
parameters: no library code reads the system's clock or zone. And every
public item has one path: submodules are private and re-exported by
their parent.

## 2. Glossary

**Scale.** A function from a domain to a range, as a value: built by
consuming builders, read by `apply`, `invert`, `ticks` and
`tick_format`, never changed by a call. Continuous scales are
`rustd3::scale::Continuous<T>` over a `scale::Transform` (`Linear`,
`Pow`, `Log`, `Symlog`, `Logit`, `Asinh`, or your own); `AnyContinuous`
picks the transform at run time. Time scales are `rustd3::time::Time`.
Band, point, ordinal, identity, sequential, diverging, quantize,
quantile and threshold scales live beside them in `rustd3::scale`.

**Domain, range.** The input and output intervals of a scale
(`.domain([a, b])`, `.range([a, b])`, read back with `get_domain` and
`get_range`). For band and ordinal scales the domain is a list of keys.

**Builders and readers.** A builder takes a setting's name and returns
the value it was called on with the setting changed (`.domain(d)`, and
`with_precision(p)` on the types d3 reads as fields); a method that
returns a stored or configured value is `get_` and the name
(`get_domain()`, `Theme::get_line_width()`,
`Specifier::get_precision()`), with or without a builder beside it.
Predicates are `is_`, `has_` or `wants_`, std's names (`len`, `iter`,
`into_*`) mean what they mean in std, and a method that computes keeps
the name of what it gives (`bandwidth`, `ticks`, `invert`).

**Tick.** A nicely rounded domain value a scale offers for labelling:
`scale.ticks(count)`. The free functions in `rustd3::ticks` are d3-array's
(`ticks`, `tick_step`, `nice`); `rustd3::ticks::Set` carries a scale's
chosen ticks with their step.

**Tick rule.** How a scale places its ticks: `rustd3::scale::TickRule`,
d3's for linear and log scales and matplotlib's locators for symlog,
logit and asinh. A transform states its own rule; the building blocks are
in `rustd3::ticks` (`linear`, `log`, `decades`, `subdivide`,
`transformed`).

**Format.** A number format compiled from d3-format's specifier
mini-language: `rustd3::format::Format`, from `format::format(spec)`,
`Locale::format(spec)`, or the `rustd3::fmt!` macro, which checks a
literal specifier at compile time. Time formats are
`rustd3::time::Format`, from `time::format("%Y-%m-%d")` or a time
locale. `scale::TickFormat` is the format a scale chooses for its ticks.

**Locale.** What labels are written in. `rustd3::locale::Locale` has a
number half (`format::Locale`, d3's formatLocale) and a time half
(`time::Locale`, d3's timeFormatLocale plus the `time::Ladder` that tick
labels climb). `Locale::NEUTRAL` writes SI style (a point, narrow
no-break space grouping, ISO dates) and is the default; `EN_US`, `DE_DE`,
`FR_FR` and eight more are constants.

**Generator.** A d3-shape value that turns geometry into path commands:
`rustd3::shape::Line`, `Area`, `Arc`, `Pie`, `Stack`, `Symbol`, built by
`shape::line()`, `shape::area()` and so on. Generators take iterators of
already-scaled points; `shape::LineOf` adds d3's accessors over a slice
of data.

**`PathContext`.** `rustd3::path::PathContext`, d3's canvas path
interface (`move_to`, `line_to`, `arc`, `bezier_curve_to`, …): the one
seam between generators and backends. Implementations: `path::SvgPath`
(SVG path data, `path::svg(|p| …)`), `rustd3::figure::path::Bez` (a kurbo
path for the drawing list) and `rustd3_iced::IcedPath` (an iced canvas
path).

**Curve.** How a line or area joins its points: `rustd3::shape::Curve`
(`Linear`, `Step`, `StepAfter`, `Basis`, `BumpX`, `Cardinal { .. }`,
`CatmullRom { .. }`, `MonotoneX`, `Natural`, and their closed and open
forms), d3's curve factories as an enum.

**Figure.** `rustd3::figure::Figure`: a table of scales behind typed
handles, panels in grid cells, a layout, a title, legends, a locale and a
theme. It is a description; resolving it fits the domains and lays the
grid out, and drawing it makes a drawing list.

**Panel.** `rustd3::figure::Panel`, one cell (or span of cells) of the
figure's grid: its marks, axes, title, legends in its corners, border,
zoom handles and brush. `fig.panel(row, col)` makes or returns it;
`fig.panel_at(Cell::new(r, c).span(rows, cols))` spans tracks.

**Mark.** Something a panel draws from data:
`rustd3::figure::mark::Mark`, implemented by `dots`, `line`, `area`,
`ribbon`, `bar_y`, `bar_x`, `rect`, `histogram`, `density_y`,
`density_x`, `image`, `note`, `rule_x` and `rule_y`. A mark of your own
implements the trait and draws through `mark::Ctx`.

**Axis.** At the core level `rustd3::axis::Axis`, d3-axis: it draws its
domain line and ticks into a `PathContext` and returns its labels as
positions and strings. At the figure level `rustd3::figure::axis::Axis`,
built from a handle (`axis::bottom(x).label("time [s]")`) and placed on
panels; the figure measures and draws its labels.

**Legend.** `rustd3::figure::legend::Legend`, built from an ordinal
color handle: one swatch and key per entry. It goes into a figure
`legend::Slot` beside the grid or a panel corner (`legend::Inset`).

**Colorbar.** `rustd3::figure::colorbar::Colorbar`, what a legend is to
a continuous or binned color scale: a ramp labelled by the scale's own
ticks, with optional end triangles (`colorbar::Extend`). Legends and
colorbars are both `legend::Guide`s.

**Layout.** How the figure's grid becomes rectangles:
`rustd3::figure::layout::Grid` (track weights and gaps in
`layout::Length`, `px` or `em`). Each column takes the widest reach of
its panels' decorations (tick labels, axis labels, titles) past their
plot areas; text is measured, not estimated, and layout repeats until
the labels fit.

**Theme.** `rustd3::figure::Theme`: colors, sizes and which lines are
drawn, built as seaborn builds one from a `theme::Style`, a
`theme::Context`, a font scale and a `theme::palette`, plus how fitted
domains end (`theme::Domains`).

**Drawing list.** `rustd3::figure::list::List`: seven kinds of
`list::Command` (fills, strokes, glyph runs, markers, images, groups and
clips) on kurbo geometry and peniko paints, a font table, and the parts
that say which panel and role drew each command. Every output renders
this list.

**Output.** A backend that renders a drawing list: SVG
(`rustd3::figure::svg`), raster (`rustd3::figure::raster`, vello_cpu; PNG
in `raster::png`; `raster::skia` the tiny-skia fallback), PDF
(`rustd3::figure::pdf`, krilla) and GIF (`rustd3::figure::gif`). Before
writing, a list is lowered (`rustd3::figure::lower`) to the commands the
backend draws natively (`lower::Caps`).

**The `Plot` widget.** `rustd3_iced::Plot`, an iced widget that shows a
`Figure` as the pixels its PNG would have, with a crosshair and a
nearest-point highlight drawn over them, wheel zoom, drag pan, brushing
and live data.

**Handle.** `rustd3::figure::scale::ScaleRef<S>`, a typed, `Copy`
reference to a scale in the figure's table, returned by `fig.scale(s)`
(domain as given) and `fig.fit(s)` (domain fitted to the data). Marks,
axes, legends, brushes and the widget's listeners all name scales by
handle; two panels share a scale by sharing a handle. A handle's type
decides what it can do: an axis needs `scale::Position`, a brush
`scale::Invert`, a colorbar `colorbar::Ramp`. A handle carries its
figure: given to another figure it finds no scale (`Resolved::get_scale` is
`None`), what uses it draws nothing, and `fig.check` reports
`check::Issue::ForeignHandle`.

**Listener.** A closure the `Plot` widget calls when a handle's domain
changes: `Plot::on_change(handle, f)` (the app then owns that domain),
`Plot::on_brush(handle, f)` and `Plot::on_hover(f)`.

**Gesture.** A wheel burst, a drag or a brush on the `Plot` widget:
pointer input queued per frame, applied through the core's `Zoom`,
`BrushX` or `BrushY`, with the layout held still until the gesture ends.

**Tile.** A 256 × 256 device-px square of a plot area, the unit the
`Plot` widget renders, caches and moves. A plot area has one grid of
tiles, anchored at a data point so that a pan by whole device pixels
moves every tile unchanged, plus a ring of tiles around it rendered
ahead of need.

**Preview.** An image the `Plot` widget shows while its sharp
replacement renders: the previous tiles mapped onto the new view by one
affine map per axis, soft until the new tiles land.

**Cost model.** The `Plot` widget's prediction of a render's time: a
fixed overhead per render plus a time per pixel and per drawing item,
seeded by two constants measured on a reference figure and fitted to the
widget's own renders on the machine it runs on, tiles apart from axis
strips. It decides which renders run inline on the UI thread.

**Render mode.** `rustd3_iced::plot::Render`: where changed cells
render. `Auto` (inline while the cost model says they fit half the frame
budget, the rest on the pool), `Inline` or `Worker`.

**Caching.** `rustd3_iced::plot::Caching`: how the widget reuses
rendered images. `On` (re-render only what changed), `Off` (every cell
whenever any changed) or `Verify` (re-render kept images and compare, a
diagnostic).

**Probe.** `rustd3_iced::plot::Probe`: a channel of
`plot::probe::Sample`s describing what each frame of a `Plot` cost and
showed (UI stages, renders, previews), timed by a clock the app supplies.
The benchmark is built on it.

## 3. Handbook

### Building a figure

A figure is built in four moves: make scales into handles, put marks
and axes on panels, set what the figure shares (title, theme, locale,
legends), then draw or save it.

```rust
use rustd3::figure::mark::{dots, line};
use rustd3::figure::{Figure, axis};
use rustd3::scale;

let data = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)];
let mut fig = Figure::new();
let x = fig.fit(scale::linear()); // fitted to every mark that uses x
let y = fig.scale(scale::linear().domain([0.0, 4.0])); // kept as given
fig.title("Readings");
fig.panel(0, 0)
    .mark(line(data, x, y))
    .mark(dots(data, x, y))
    .axis(axis::bottom(x).label("t [s]"))
    .axis(axis::left(y));
```

`fig.fit_with(scale, fit)` chooses how a handle follows its data
(`rustd3::figure::scale::Fit`): `All` (every value, the default),
`Window(span)` (the newest `span` of x, for live data) or `Visible` (y
fitted to what the x handles show). A mark keeps a value at the edge of
its fitted domain with `Fits::stick`, as matplotlib's sticky edges do
(bars start at 0).

`fig.check(size)` lays the figure out without drawing and returns the
`check::Issue`s that will not read well (labels that do not fit, marks
with nothing to draw), so a test can assert that a figure fits.

### Scales and axes

Every scale kind has a constructor in `rustd3::scale` (`linear`, `log`,
`pow`, `sqrt`, `symlog`, `logit`, `asinh`, `band`, `point`, `ordinal`,
`sequential`, `diverging`, `quantize`, `quantile`, `threshold`,
`identity`) or `rustd3::time` (`time(zone)`, `utc()`). Continuous scales
take `.nice(count)`, `.clamp(on)`, `.interpolate(…)` and `.unknown(…)`
as d3's do. A scale used as a figure handle gets its domain from the
figure when fitted, and its range from the panel it is drawn on. A
handle's type is its scale's, so a transform chosen at run time goes
through `scale::AnyContinuous::from(scale::log())`, which is one type
for every transform.

At the core level an axis is a drawing:

```rust
use rustd3::{axis, path, scale};

let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
let bottom = axis::bottom(&x).ticks(2);
let d = path::svg(|p| bottom.draw(p)); // domain line and tick marks
for label in bottom.labels() {
    // label.get_x(), label.get_y(), label.get_text(), label.get_anchor(): your text
}
```

At the figure level an axis is a value placed on panels:
`axis::bottom(x)`, `left`, `top`, `right`, with `.label(text)`,
`.ticks(count)`, `.values(…)`, `.minor(on)`, `.format(spec)` and
`.rotate(…)`; grid lines are the theme's. `.group(f)` names each tick's
group and labels each run of neighbors once, past the tick labels, and
`axis::Nested(outer, inner)` is a key of two levels for a band scale. Band and point axes label
their keys through `Display`.

### Marks

| Mark | What it draws |
| --- | --- |
| `dots(points, x, y)` | markers, with per-point size and color channels (`size_by`, `fill_by`), dodge and jitter; `.symbol(Symbol::Diamond)` for d3's symbols and `.hollow(true)` for outlines |
| `line(points, x, y)` | a polyline with a `shape::Curve`; recordings are thinned (below) |
| `area(rows, x, y)`, `ribbon`, `ribbon_series` | filled areas between a baseline and a line, or two lines |
| `bar_y`, `bar_x`, `bar_y_spans`, `bar_x_spans` | bars from a base to each value, or spans, on a band scale; `.dodge()` groups them |
| `rect(rows, x, y)` | rectangles `([x0, x1], [y0, y1])` in data coordinates, on any position scales; `.stroke(palette, key)` outlines them, `.stroke_width(px)` sets how wide |
| `histogram(values, x, y)` | counts of values per key or per bin (`mark::Bins`: d3's nice bins or a count of equal ones) |
| `density_y(values, y, w)`, `density_x` | a Gaussian kernel density estimate (`rustd3::figure::stats::Kde`, Scott's bandwidth) |
| `image(grid, x, y, color)` | a grid of values as a raster heatmap through a color handle |
| `note(at, text, x, y)` | a label at a data position; `.avoid(Avoid::Step)` steps aside for the notes before it, `Avoid::Drop` leaves it out |
| `rule_x(value, x)`, `rule_y` | a reference line across the plot area |

Areas and bars take a `hatch::Hatch`. A density over more than 10,000
values is estimated from linearly binned values convolved with the
kernel, within a relative 1e-3 of the exact sum and about a hundred times
faster on a million values; below that it is exact.

With the `helpers` feature, `rustd3::figure::helpers` composes these
into one-call charts: `helpers::chart` (`line`, `scatter`, `bar`, `area`,
`pie`, `heatmap`, each ending in `.figure()`, `.save_svg(path)` or
`.save_png(path, scale)`; `line(..).key(name)` and `area(..).keys(names)`
name the series the legend shows), `helpers::stats` (box plots, violins, ECDF,
error bars, histogram options, with the statistics as pure functions),
`helpers::labels` (value labels), `helpers::legend` (size and color
legends) and `helpers::subplots`. Each helper is ordinary code over the
public API, written to be copied and changed.

### Text, math and labels

Every label is a `rustd3::figure::text::Text`: literal spans and typst
markup, with explicit `\n` line breaks.

```rust
use rustd3::color::Rgb;
use rustd3::figure::text::{self, Text, span};

let title = Text::from("Wheel: ").push(span("RL").fill(Rgb::hex(0xc44e52)));
let unit = Text::from("RMS [").push(text::typst("$m slash s^2$")).push("]");
```

A `&str` or `String` converts into a literal `Text`. typst shapes each
batch of new labels in one document; shaped labels live in the figure's
`Cache` (`rustd3::figure::cache::Cache`), keyed by text and style, so a redraw
shapes only labels it has not seen. Without the `fonts` feature the app
supplies fonts through `text::Fonts::with`. Tick labels are written
through the figure's locale; axes drop labels that would collide, keeping
a regular stride.

### Legends and colorbars

```rust
use rustd3::figure::Figure;
use rustd3::figure::colorbar::{Extend, colorbar};
use rustd3::figure::legend::{Inset, Slot, legend};
use rustd3::{scale, scheme};

let mut fig = Figure::new();
let source = fig.scale(scale::ordinal(["accel x", "accel z"], scheme::COLORBLIND));
let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
fig.legend(Slot::Bottom, legend(source).title("source"));
fig.legend(Slot::Right, colorbar(heat).title("temp [°C]").extend(Extend::Both));
fig.panel(0, 0).legend(Inset::TopLeft, legend(source));
```

A figure slot (`Slot::Top`, `Bottom`, `Left`, `Right`) sits outside the
grid, centred on the plot areas beside it; a panel inset sits in a
corner. Guides in one slot stack outward in the order they are added. A
colorbar of an ordinal palette, or a legend of a continuous scale, does
not compile. Each key's swatch is what its marks draw: a line, a dot, or
a square that repeats a hatch and the opacity a translucent area, ribbon
or density fills at (an error band's square is as pale as the band; a
key several marks fill takes the most opaque).

### Layout and sizes

Sizes are logical px; a raster output multiplies them by its device
scale (`save_png(path, size, 2.0)` writes twice as many pixels each way).
The grid splits the figure by weights after fixed reaches, gaps and
padding:

```rust
use rustd3::figure::Figure;
use rustd3::figure::layout::{Cell, Grid, em};

let mut fig = Figure::new();
fig.layout(Grid::new().cols([1.0, 0.25]).col_gaps([em(0.4)]));
fig.panel_at(Cell::new(1, 0).span(1, 2)); // a panel across both columns
```

`em` is the tick-label size, so gaps scale with the theme. Panels of one
column share their plot area's `x` and width exactly, and panels of one
row their `y` and height.

### Themes

```rust
use rustd3::figure::theme::{Context, Domains, Style, palette};
use rustd3::figure::{Figure, Theme};

let mut fig = Figure::new();
fig.theme(
    Theme::seaborn(Style::WhiteGrid, Context::Paper)
        .palette(palette::DEEP)
        .font_scale(1.1)
        .domains(Domains::Nice),
);
```

Styles are seaborn's (`DarkGrid`, `WhiteGrid`, `Dark`, `White`,
`Ticks`), contexts scale every size at once (`Paper`, `Notebook`,
`Talk`, `Poster`), palettes are seaborn's and d3's. `Theme::default()`
is rustd3's own. `.background(c)`, `.panel(c)` and `.ink(c)` recolor the
figure, its plot areas and its text, which makes a dark figure. Custom
marks read the theme through `Theme::get_color`,
`get_line_width` and the other readers.

### Time, zones and locales

Instants are `jiff::Timestamp` (re-exported as `rustd3::jiff`), and every
operation that needs a calendar takes a `jiff::tz::TimeZone`: intervals
(`rustd3::time::Interval`, d3-time's plus microseconds and nanoseconds),
ticks, time scales and formats. Arithmetic on instants is in `i128`
nanoseconds, never through an `f64` epoch.

```rust
use rustd3::jiff::Timestamp;
use rustd3::jiff::tz::TimeZone;
use rustd3::time;

let day: [Timestamp; 2] = ["2026-10-28T00:00:00Z".parse()?, "2026-10-29T00:00:00Z".parse()?];
let t = time::time(TimeZone::get("Europe/Berlin")?).domain(day).range([0.0, 960.0]);
let label = time::format("%H:%M").format(t.invert(480.0), &TimeZone::UTC);
```

A time axis labels its ticks by climbing the locale's ladder
(`time::Ladder`): the neutral `Iso` ladder writes `2026-03`, `03-08`,
`12:00`; d3's ladder writes `March`, `Mar 08`. Set the figure's locale
with `fig.locale(rustd3::locale::Locale::EN_US)`. Every constant but
`EN_US` keeps the ISO ladder, and `Locale::DE_DE.ladder(time::Ladder::D3)`
writes d3's.

### Outputs and their settings

| Output | Feature | Call | Notes |
| --- | --- | --- | --- |
| SVG | always (`fonts` for `save_svg`) | `fig.save_svg(path, size)`, `svg::write(&list, &options)` | text as glyph outlines defined once and reused, or `<text>` (`svg::Text`) |
| PNG | `png` | `fig.save_png(path, size, scale)`, `raster::png` | vello_cpu at any device scale; `raster::Settings` picks threads and SIMD |
| PDF | `pdf` | `fig.save_pdf(path, size)`, `pdf::write(&list, &options)` | one page in points; text selectable in subset fonts |
| GIF | `gif` | `gif::Writer::new(file, size, scale, repeat)`, `.frame(&fig, delay)` | one cache for the sequence, so repeated labels are shaped once |
| your own | always | `fig.draw(size)` → `list::List` | lower it with `lower::lower(&list, caps)` for what your backend draws |

A figure is drawn for a `rustd3::figure::Target`: `Vector` (SVG, PDF),
`File { scale }` (PNG, GIF) or `Screen { scale }` (the widget). The
target sets how finely lines are thinned and whether geometry snaps to
device pixels. `fig.draw_with(&mut cache, size, target, &held)` draws
with a cache you keep between frames.

### The core level in an iced canvas

`rustd3_iced::path` turns d3 drawing into an iced `canvas::Path`; the
app writes the axis labels with iced's own text. The `canvas` example
of rustd3-iced is a whole app; its draw function, in part:

```rust
let x = scale::linear().domain([0.0, 10.0]).range([40.0, width - 40.0]);
let y = scale::linear().domain([-1.0, 1.0]).range([height - 60.0, 20.0]);
let wave = rustd3_iced::path(|p| {
    let points = (0..=200).map(|i| {
        let t = f64::from(i) / 20.0;
        (x.apply(t), y.apply(t.sin()))
    });
    shape::line().draw(points, p);
});
frame.stroke(&wave, Stroke::default().with_width(2.0));
let bottom = axis::bottom(&x);
let ticks = rustd3_iced::path(|p| bottom.draw(p));
for label in bottom.labels() {
    // frame.fill_text(Text { content: label.get_text().to_owned(), … })
}
```

For interaction, keep a `rustd3_iced::pointer::Tracker` in the
program's state and pass it every event with the bounds and cursor:
`tracker.translate(event, bounds, cursor)` returns
`rustd3::interact::Pointer` input in the area's own coordinates, which
`interact::Zoom::handle`, `BrushX`, `BrushY` and `interact::Drag` take.
They return the new transform or selection; `interact::Rescale` applies a
zoom transform to a scale (`scale.rescale_x(&transform)`). Time comes
only from `RedrawRequested`, so a program returns
`Action::request_redraw()` while `tracker.wants_redraw()`. A program
captures the presses it handles, since iced selects text from a press no
widget captured and redraws every frame until its release.
`anim::Animated` and `anim::Marks` are sampled at the instants the app
passes in, `anim::Instant` (`web_time`'s: std's on native targets, iced's
`time::Instant`, and a working clock on wasm). The `focus_context` and
`update_pattern` examples are complete apps.

### The `Plot` widget

`Plot::new(fig)` in `view`, rebuilt every time the app's state changes:
the widget keeps its own state (the figure cache, rendered tiles, the
domains of handles nobody listens to) in iced's widget tree.

- **Zoom and pan.** `panel.zoom_x(handle)` and `zoom_y` say which
  handles a panel's gestures move. A wheel over a plot area zooms them
  around the cursor, a wheel over an axis zooms that axis alone, a
  left-button drag pans, a double click returns them to their fitted
  domains.
- **Owning a domain.** `Plot::on_change(handle, |domain| message)` sends
  the new domain once per frame; the app then builds the figure with
  that domain (`fig.scale(scale.domain(window))`). Handles without a
  listener keep the widget's domain across rebuilds.
- **Brush.** `panel.brush(brush::x(handle))` makes a drag over the plot
  area brush instead of pan; `Plot::on_brush` publishes the selection,
  and `brush::x(over).sets(target)` makes the selection the target's
  domain (focus and context).
- **Hover.** The crosshair and highlight are drawn over the images and
  render nothing; `Plot::on_hover` reports the nearest drawn point
  (`plot::Hover`) from a hit index of the last drawing list
  (`rustd3::figure::hit`). `Plot::crosshair(false)` and
  `highlight(false)` turn them off.
- **Live data.** Append to a `series::Recording` and rebuild the figure;
  the hashes of the drawing list's parts are the dependency tracker, so
  only the cells the new data touched render. `Fit::Window(span)` on x
  follows the newest data in whole device columns.
- **Render modes and threads.** Changed tiles render inline while the
  cost model says they fit half the frame budget, the rest on a pool of
  `Plot::threads(n)` threads (default: the available parallelism minus
  one, 1 to 8); `Plot::render(Render::Inline | Worker)` forces a path.
  Results land newest first, and an older result never replaces a newer
  image.
- **Scale factor.** The widget renders for the window's scale factor;
  `Plot::scale_factor(k)` sets it where the host reports none (iced's
  test simulator snapshots at 2 and reports none). A host that draws the
  widget under a scaling transformation, such as a pan-and-zoom surface,
  passes the scale to `Plot::magnification(z)`, and the widget renders at
  the factor times `z`.
- **Diagnostics.** `Plot::probe(probe)` reports every frame,
  `Plot::caching(Caching::Verify)` checks the tile cache, and
  `Plot::revision(n)` re-renders everything once for a change the hashes
  cannot see (an image's pixels replaced in place).

### Performance characteristics

- **Text** is shaped once per label and style and cached; a redraw sends
  only new labels to typst, in one batch.
- **Thinning.** A `line` over a `series::Series` view of a recording on
  an x scale with an index bridge is thinned at draw time with M4: per
  column, the first, last, least and greatest sample, from an index that
  grows with each append. One column per device pixel on screen, four per
  device pixel for PNG and GIF, four per list pixel for SVG and PDF.
  Drawing costs what the output shows, not what the recording holds, and
  every extreme survives. Owned points and slices are drawn as given.
- **Tiles.** On a pan by whole device pixels a plot area's tiles move
  unchanged, and only the column entering the ring renders (one per 256
  px). A zoom begins a new grid and shows previews until the sharp tiles
  land; a burst of wheel steps keeps only the newest step's renders
  queued.
- **The pool.** Each pool thread owns a single-threaded vello_cpu
  renderer, so renders never oversubscribe the cores; the pool is the
  widget's only parallelism.
- **Stamping.** On screen, a markers command of 10,000 points or more is
  composited from precomputed coverage stamps at eighth-pixel phases
  (`raster::Renderer::splat`), within 32/255 of the filled markers and
  several times faster; files always fill every marker.
- **Software renderer.** iced's vello_cpu renderer draws the widget's
  tiles and previews as wgpu does, each image on its device pixels. A
  software frame at 1600 × 1000 costs 10 to 30 ms at factor 1 and 25 to
  115 ms at factor 2; a zoom's latency there is bounded by the wheel
  burst, since no frame of a burst is sharp until the burst pauses.
- **Known limit.** The figure is resolved and its drawing list built on
  the UI thread, so panning a scatter of a million points is bound by
  list building.

`cargo run --release --example bench_all -- --quick` measures all of
this on a machine; `benchkit/README.md` describes the levels and the
archive.

### Error handling

Every fallible call returns a typed error carrying what was rejected;
nothing returns `Result<_, String>`.

| Error | From |
| --- | --- |
| `rustd3::format::InvalidFormat` | a d3-format specifier that does not parse |
| `rustd3::color::ParseColorError` | a CSS color that does not parse |
| `rustd3::series::Error` | an explicit x out of order, channels of the wrong length or count, an invalid period |
| `rustd3::path::Error` | an arc with a radius below zero |
| `rustd3::figure::Error` | `save_svg`, `save_png`, `save_pdf`: the file, the list, the render or the PDF |
| `rustd3::figure::list::Error` | a list that fails `list::check` (unbalanced groups, a glyph run outside the font table, a non-finite coordinate) |
| `rustd3::figure::mark::Error` | data a mark cannot draw, such as a density of fewer than two values (reported by `fig.check`) |
| `rustd3::figure::text::Error` | typst markup that does not compile |
| `rustd3::figure::raster::Error`, `pdf::Error`, `gif::Error` | an output that cannot render or write |

Marks never panic on bad data: non-finite values are gaps, values a
scale does not take are skipped, and `fig.check(size)` lists what will
not draw. Scale calls return `Option` where d3 returns `undefined`
(`ordinal.apply(&key)`, `band.apply(&key)`), and `invert` returns NaN
where d3 does.
