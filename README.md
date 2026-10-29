# rustd3

rustd3 is a Rust plotting library that speaks d3's vocabulary (scales,
ticks, shapes, number and time formats, color, zoom and brush) and adds
the pieces only matplotlib has: typst text and math, legends and
colorbars, layout from measured text, statistical charts. It is
standalone first: one drawing list renders SVG, PNG and PDF, and GIF for
sequences, with time handled by jiff. iced integration comes second, as
one backend among several: a canvas path for apps that draw themselves
and a `Plot` widget for figures, with zoom, pan, hover and brushing.

Scales and generators are plain values: build them where you need them,
call them, drop them; calling one never changes it. Data reaches
generators as iterators of geometry, generators draw into `PathContext`
(d3's canvas path interface), and the figure layer turns that geometry,
text and images into a styled drawing list that every backend renders.
Zoom, brush and drag are small state machines over pointer input, and the
time zone, the locale and the clock are always parameters. The core is
graded by d3's own test suites, translated to Rust.

## Two levels

- **Core level** (`rustd3` without `figure`, the `rustd3-core` crate):
  your code draws, as with d3 in a browser. Scales, ticks, axes, shapes
  and curves draw into any `PathContext`: an SVG path string, an iced
  canvas path, or your own.
- **Figure level** (the `figure` feature, the `rustd3-figure` crate): your
  code describes a `Figure` of panels, marks, axes, legends and
  colorbars; layout measures its text with typst, and a backend draws
  it. `rustd3::axis::bottom` is core's axis for a scale; at the figure
  level `rustd3::figure::axis::bottom` places one on a panel.

Depend on `rustd3` alone. The API documentation of the component crates
(rustd3-core, rustd3-figure, rustd3-iced) shows examples under their own
crate names; with the facade, `rustd3_core::x` is `rustd3::x` and
`rustd3_figure::x` is `rustd3::figure::x`. `rustd3_iced` is not
re-exported: an iced app depends on `rustd3-iced` and uses it directly
(see [In an iced canvas](#in-an-iced-canvas)).

## A figure to PNG

```toml
[dependencies]
rustd3 = { version = "0.1", features = ["png"] }
```

```rust no_run
# #[cfg(all(feature = "figure", feature = "fonts"))]
# fn main() -> Result<(), rustd3::figure::Error> {
use rustd3::figure::kurbo::Size;
use rustd3::figure::mark::{dots, line};
use rustd3::figure::text::typst;
use rustd3::figure::{Figure, axis};
use rustd3::scale;

// A damped wave sampled every 50 ms; every tenth sample is also a dot.
let wave: Vec<(f64, f64)> = (0..=400)
    .map(|i| {
        let t = f64::from(i) / 20.0;
        (t, (-t / 8.0).exp() * t.sin())
    })
    .collect();

let mut fig = Figure::new();
let t = fig.fit(scale::linear());
let y = fig.fit(scale::linear());
fig.title("A damped wave");
fig.panel(0, 0)
    .mark(line(wave.iter().copied(), t, y))
    .mark(dots(wave.iter().copied().step_by(10), t, y))
    .axis(axis::bottom(t).label("time [s]"))
    .axis(axis::left(y).label(typst("$e^(-t slash 8) sin t$")));

let size = Size::new(640.0, 400.0); // logical px
fig.save_svg("wave.svg", size)?;
# #[cfg(feature = "png")]
fig.save_png("wave.png", size, 2.0)?; // 1280 × 800 device px
# Ok(())
# }
# #[cfg(not(all(feature = "figure", feature = "fonts")))]
# fn main() {}
```

`fig.fit` spans the data of every mark that uses the scale; `fig.scale`
keeps the domain you give it. Two panels share a scale by using the same
handle. With the `helpers` feature, `rustd3::figure::helpers::chart` has
one-call presets (line, scatter, bar, area, pie, heatmap) and
`helpers::stats` the statistical charts (histogram, box plot, violin,
ECDF, error bars).

## d3 at the core level

```rust
use rustd3::{axis, path, scale, shape};

// Scales are values: build, call, drop.
let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
assert_eq!(x.apply(25.0), 240.0);
assert_eq!(x.invert(480.0), 50.0);
assert_eq!(x.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);

// Generators draw into any PathContext; here an SVG path string.
let y = scale::linear().domain([0.0, 1.0]).range([100.0, 0.0]);
let data = [(0.0, 0.5), (50.0, 1.0), (100.0, 0.0)];
let d = path::svg(|p| shape::line().draw(data.map(|(a, b)| (x.apply(a), y.apply(b))), p));
assert_eq!(d, "M0,50L480,0L960,100");

// Axes are d3's: tick positions, labels, and the path of their lines.
let bottom = axis::bottom(&x).ticks(5);
let labels: Vec<String> = bottom.labels().iter().map(|l| l.text().to_owned()).collect();
assert_eq!(labels, ["0", "20", "40", "60", "80", "100"]);

// Number formats are d3-format's, in the locale you pass; `fmt!` checks
// its specifier at compile time and writes the neutral SI style.
let dollars = rustd3::format::Locale::EN_US.format("$,.2f")?;
assert_eq!(dollars.format(1234.5), "$1,234.50");
assert_eq!(rustd3::fmt!(".1%").format(0.123), "12.3%");
# Ok::<(), rustd3::format::InvalidFormat>(())
```

## In an iced canvas

`rustd3-iced` depends on iced at a pinned git revision, and crates.io
does not accept git dependencies, so until iced 0.15 is on crates.io it
comes from git; take `rustd3` from the same repository, so that cargo
builds one `rustd3-core` and the two crates' types match.

```toml
[dependencies]
rustd3 = { git = "https://…/rustd3" }
rustd3-iced = { git = "https://…/rustd3" }
iced = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe", features = ["canvas"] }
```

`rustd3_iced::path` turns d3's drawing into an iced `canvas::Path`; the
app writes the axis labels with iced's text. A whole `main.rs`:

```text
use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Stroke, Text};
use iced::{Fill, Point, Rectangle, Renderer, Theme, Vector, Widget, mouse};
use rustd3::{axis, scale, shape};

fn main() -> iced::Result {
    iced::run(Wave::update, Wave::view)
}

/// Ten seconds of a sine wave over a time axis.
#[derive(Default)]
struct Wave;

impl Wave {
    fn update(&mut self, _message: ()) {}

    fn view(&self) -> impl Widget<()> {
        canvas(Wave).width(Fill).height(Fill)
    }
}

impl canvas::Program<()> for Wave {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (width, height) = (f64::from(bounds.width), f64::from(bounds.height));
        // Scales are values: build them for this size, use them, drop them.
        let x = scale::linear()
            .domain([0.0, 10.0])
            .range([40.0, width - 40.0]);
        let y = scale::linear()
            .domain([-1.0, 1.0])
            .range([height - 60.0, 20.0]);
        let palette = theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let wave = rustd3_iced::path(|p| {
            let points = (0..=200).map(|i| {
                let t = f64::from(i) / 20.0;
                (x.apply(t), y.apply(t.sin()))
            });
            shape::line().draw(points, p);
        });
        let line = Stroke::default().with_width(2.0);
        frame.stroke(&wave, line.with_color(palette.primary.base.color));
        // The axis draws its line and ticks; iced writes its labels.
        let ink = palette.background.base.text;
        let bottom = axis::bottom(&x);
        frame.translate(Vector::new(0.0, (height - 40.0) as f32));
        let ticks = rustd3_iced::path(|p| bottom.draw(p));
        frame.stroke(&ticks, Stroke::default().with_color(ink));
        for label in bottom.labels() {
            frame.fill_text(Text {
                content: label.text().to_owned(),
                position: Point::new(label.x() as f32, label.y() as f32),
                color: ink,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Top,
                ..Text::default()
            });
        }
        vec![frame.into_geometry()]
    }
}
```

`rustd3_iced::pointer::Tracker` turns iced's events into the core's
`Pointer` input, which `rustd3::interact::Zoom`, `BrushX`, `BrushY` and
`Drag` take.

## The `Plot` widget

The widget shows a `Figure` as the PNG's own pixels, with a crosshair
and a nearest-point highlight drawn over them; the wheel zooms and a drag
pans. It needs iced from git as above, and the `figure` features:

```toml
[dependencies]
rustd3 = { git = "https://…/rustd3", features = ["figure"] }
rustd3-iced = { git = "https://…/rustd3", features = ["figure"] }
iced = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
```

The app builds the figure in `view` and owns the x window here; y stays
the widget's:

```text
use iced::Element;
use rustd3::scale;
use rustd3::figure::mark::line;
use rustd3::figure::{Figure, axis};
use rustd3_iced::Plot;

fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

/// The x window the user chose by zooming or panning, once there is one.
#[derive(Default)]
struct App {
    window: Option<[f64; 2]>,
}

#[derive(Debug, Clone)]
enum Message {
    Window([f64; 2]),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Window(window) => self.window = Some(window),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let wave = (0..=2000).map(|i| {
            let t = f64::from(i) / 100.0;
            (t, (-t / 8.0).exp() * t.sin())
        });
        let mut fig = Figure::new();
        let x = match self.window {
            Some(window) => fig.scale(scale::linear().domain(window)),
            None => fig.fit(scale::linear()),
        };
        let y = fig.fit(scale::linear());
        fig.panel(0, 0)
            .mark(line(wave, x, y))
            .axis(axis::bottom(x).label("time [s]"))
            .axis(axis::left(y))
            .zoom_x(x);
        // Wheel to zoom, drag to pan: the widget sends the new x window.
        Plot::new(fig).on_change(x, Message::Window).into()
    }
}
```

The widget renders for the window's scale factor, which it learns from
the window. `iced_test`'s `Simulator` has none: it snapshots at a factor
of 2 and tells widgets no factor, so a test that snapshots a `Plot` sets
it, `Plot::new(fig).scale_factor(2.0)`; without it the widget draws for
1 and the snapshot shows each of its pixels doubled.

`rustd3-iced` is not on docs.rs while it comes from git;
`cargo doc -p rustd3-iced --open` in your project shows its
documentation. When iced 0.15 is on crates.io, `rustd3-iced` is
published and the facade gains an `iced` feature:
`rustd3 = { version = "0.1", features = ["figure", "iced"] }`.

## Features

| Feature | Adds | Crates it builds |
| --- | --- | --- |
| none | the core: scales, ticks, axes, shapes, formats, time, locales, color and schemes, interpolation and easing, array statistics, recordings, zoom, brush, drag, animation | 8 beyond rustd3's own: jiff and thiserror with their dependencies |
| `derive` | `#[derive(Interpolate)]` in `rustd3::interpolate` | +2: rustd3-derive and syn 2 |
| `figure` | figures, layout, typst text and math with fonts the app supplies, the SVG writer | about 270, most of them typst's |
| `fonts` (default) | DejaVu Sans in two weights and New Computer Modern Math, 2.8 MB, bundled; it does not turn `figure` on | +1 |
| `raster` | vello_cpu rendering the drawing list into pixels | +10 |
| `png`, `gif` | `raster` and an encoder | none beyond `raster` |
| `pdf` | PDF through krilla, with selectable text | +5 |
| `tiny-skia` | the fallback rasterizer | +1 |
| `helpers` | ready-made charts (`helpers::chart`, `helpers::stats`) and `helpers::subplots` | none |
| `iced` | not yet: `rustd3-iced` from git until iced 0.15 is on crates.io | |

Every feature but `fonts`, `derive` and `figure` itself turns `figure`
on. `gif::Writer` writes its frames' text with the bundled fonts, so it
needs `fonts`.

## Outputs

- **SVG**, always: `Figure::save_svg`, text as glyph outlines.
- **PNG** (`png`): `Figure::save_png` at any device scale, rendered by
  vello_cpu; `tiny-skia` renders the same list as a fallback.
- **PDF** (`pdf`): `Figure::save_pdf`, text as selectable text in subset
  fonts.
- **GIF** (`gif`): `rustd3::figure::gif`, sequences of frames.
- **iced**: the `Plot` widget shows the pixels the PNG has.

Every output is drawn from the same drawing list
(`rustd3::figure::list::List`), so they cannot drift apart, and a list
can be rendered by your own backend.

## Performance

- **Text** is laid out by typst once per label and style: laid-out labels
  are cached in the figure's `Cache`, and each frame's new labels go to
  typst together, so a pan lays out only the labels that entered.
- **Long series** in a `series::Recording` are thinned at draw time to at
  most four points per device-pixel column (first, last, minimum and
  maximum, M4) from an index that grows with the recording: drawing costs
  what the screen or the file shows, not what the recording holds, and
  every extreme survives.
- **The `Plot` widget** re-renders only what changed: hover draws an
  overlay and renders nothing, a live append renders the cells it
  touched, and a pan moves cached 256-px tiles by whole device pixels
  and renders only the tiles that enter. Renders run inline when the
  widget's cost model, fitted to its own renders on the machine it runs
  on, says they fit half the frame budget, and otherwise on a pool of
  `Plot::threads` threads (the available parallelism minus one, 1 to 8).
  On screen, plot areas with 10,000 markers or more stamp them from
  precomputed coverage stamps; files always render every marker exactly.
- **Measured, not assumed.** `dev/bench` drives the widget on real
  hardware against frame budgets registered before each run. Its first
  run, on a desktop with a GPU, found pans bound by rasterizing and
  uploading whole plot areas, which the tiles, the pool and the stamps
  answer; the rerun after them, with profiles for 4 cores, 2 cores and no
  GPU, decides whether a GPU data layer follows.
- **Known limits.** The figure is resolved and its drawing list built on
  the UI thread, so panning a scatter of a million points is bound by
  list building; moving it off the UI thread is planned for 0.2. A
  continuous zoom shows the previous image scaled, soft until the sharp
  one lands.

### Reporting a performance problem

If a plot is slow on your machine, run the benchmark from a checkout of
rustd3 and attach the file it writes to an issue:

```bash
cargo rustd3-bench --quick
```

The quick level takes about ten minutes on a desktop, plus the first
release build. It measures the display, times three fixed renders on one
core, then drives the `Plot` widget through hovers, pans, zooms and a
window drag in windows that open and close by themselves (leave the mouse
and keyboard alone), on your GPU and on the software renderer. It runs on
Linux, Windows and macOS, with or without a GPU or a display. It ends by
printing the path of one file,
`bench-results/rustd3-bench-quick-<UTC stamp>.tar.gz`: attach that to a
"Performance problem" issue. It holds the measurements and a description
of the machine, and no host name, user name or home path; its
`PRIVACY.md` lists what it holds. The levels, the archive and the rest
are in [dev/bench/README.md](dev/bench/README.md).

## What ships and what waits

0.1.0 ships the facade and four library crates on crates.io (`rustd3`,
`rustd3-core`, `rustd3-figure`, `rustd3-fonts`, `rustd3-derive`) and
`rustd3-iced` from git. What it contains, and what is planned for 0.2
(an anyrender adapter, TOML themes, a two-dimensional brush, parsing d3
time-format strings, polar axes, hexbin and more), is in the
[changelog](https://docs.rs/crate/rustd3/latest/source/CHANGELOG.md).

## Minimum Rust version

Rust 1.97 (`rust-version = "1.97"`), the toolchain this release is built
and tested with. The dependencies allow less: typst 0.15.1 and krilla
0.8.2 need Rust 1.92, so the figure layer cannot go below 1.92, and the
core alone needs edition 2024, Rust 1.85. A lower `rust-version` comes
only with a run of the tests on that toolchain.

## Licence and credits

rustd3 is licensed under MIT OR Apache-2.0, at your option. The bundled
fonts keep their own licences: DejaVu Sans under the Bitstream Vera
licence (with the Arev terms), New Computer Modern Math under the GUST
Font License (`rustd3-fonts`' `LICENSE`).

- **d3** by Mike Bostock, ISC: the core ports d3's scales, ticks, axes,
  shapes, formats, time, color, interpolation, arrays, zoom, brush and
  drag, and its color schemes (ColorBrewer's under Apache-2.0); d3-ease,
  with Robert Penner's easing equations, is BSD-3-Clause. The notices are
  in `rustd3-core`'s `LICENSE-d3`.
- **seaborn** (BSD-3-Clause): the themes' styles, contexts and palettes
  take seaborn 0.13.2's values.
- **typst** sets all figure text and math.
