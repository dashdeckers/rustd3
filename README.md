# rustd3

rustd3 is a Rust plotting library that speaks d3's vocabulary: scales,
ticks, axes, shapes and curves, number and time formats, locales, color
and color schemes, interpolation, zoom, brush and drag. On top of it sits
a figure layer with what d3 leaves to the browser and matplotlib
provides: text and math set by typst, legends and colorbars, layout from
measured text, statistical charts and themes. One drawing list renders
SVG, PNG, PDF and GIF, and an iced integration draws d3's geometry into a
canvas or shows a whole figure as a zoomable, pannable widget.

It works at two levels. At the **core level** (`rustd3` without
features, the `rustd3-core` crate) your code draws, as with d3 in a
browser: scales and generators are plain values, built where you need
them, called and dropped, and generators draw into any `PathContext`: an
SVG path string, an iced canvas path, or your own. At the **figure
level** (the `figure` feature, the `rustd3-figure` crate) your code
describes a `Figure` of panels, marks, axes, legends and colorbars, and
layout and a backend do the rest.

The time zone, the locale and the clock are always parameters: nothing
in the library reads the system's. The core follows d3's semantics and
is checked by tests derived from d3's own test suites (5,308 assertions,
each naming d3's file and line), and the figure layer against matplotlib
and seaborn reference values.

[`GUIDE.md`](GUIDE.md) is the glossary and handbook: every concept, its
type, and how the pieces fit.

## Installation

The crates are not on crates.io yet; depend on the repository:

```toml
[dependencies]
rustd3 = { git = "https://github.com/dashdeckers/rustd3", features = ["png"] }
```

`rustd3` is the one dependency for the library: it re-exports the core
at its root and the figure layer as `rustd3::figure`. The component
crates' documentation (`cargo doc --open`) shows examples under their own
names: `rustd3_core::x` is `rustd3::x` and `rustd3_figure::x` is
`rustd3::figure::x`.

## A figure to PNG

```rust
use rustd3::figure::kurbo::Size;
use rustd3::figure::mark::{dots, line};
use rustd3::figure::text::typst;
use rustd3::figure::{Error, Figure, axis};
use rustd3::scale;

fn main() -> Result<(), Error> {
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
    fig.save_png("wave.png", size, 2.0)?; // 1280 × 800 device px
    Ok(())
}
```

`fig.fit` spans the data of every mark that uses the scale; `fig.scale`
keeps the domain you give it. Two panels share a scale by using the same
handle. With the `helpers` feature, `rustd3::figure::helpers::chart` has
one-call presets (line, scatter, bar, area, pie, heatmap; `.key(name)`
and `.keys(names)` name the series a legend shows) and
`helpers::stats` the statistical charts (histogram, box plot, violin,
ECDF, error bars).

## d3 at the core level

No features needed. Scales, a line and an axis, written as an SVG:

```rust
use rustd3::format::{InvalidFormat, Locale};
use rustd3::{axis, path, scale, shape};

fn main() -> Result<(), InvalidFormat> {
    // Scales are values: build them, call them, drop them.
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    let y = scale::linear().domain([0.0, 1.0]).range([100.0, 0.0]);
    assert_eq!(x.apply(25.0), 240.0);
    assert_eq!(x.invert(480.0), 50.0);
    assert_eq!(x.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);

    // Generators draw scaled geometry into any PathContext: here an SVG path.
    let data = [(0.0, 0.5), (50.0, 1.0), (100.0, 0.0)];
    let points = data.map(|(a, b)| (x.apply(a), y.apply(b)));
    let line = path::svg(|p| shape::line().draw(points, p));
    assert_eq!(line, "M0,50L480,0L960,100");

    // An axis is d3's: its domain line and ticks as a path, and its labels.
    let bottom = axis::bottom(&x).ticks(5);
    let ticks = path::svg(|p| bottom.draw(p));
    let labels: Vec<String> = bottom
        .labels()
        .iter()
        .map(|l| l.get_text().to_owned())
        .collect();
    assert_eq!(labels, ["0", "20", "40", "60", "80", "100"]);

    // Number formats are d3-format's, in the locale you pass; `fmt!` checks
    // its specifier at compile time.
    assert_eq!(Locale::EN_US.format("$,.2f")?.format(1234.5), "$1,234.50");
    assert_eq!(rustd3::fmt!(".1%").format(0.123), "12.3%");

    println!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"980\" height=\"140\">\
         <g transform=\"translate(10,10)\">\
         <path d=\"{line}\" fill=\"none\" stroke=\"steelblue\"/>\
         <path d=\"{ticks}\" transform=\"translate(0,105)\" fill=\"none\" stroke=\"black\"/>\
         </g></svg>"
    );
    Ok(())
}
```

## The `Plot` widget

`rustd3-iced` shows a `Figure` in an iced app as the pixels its PNG has,
with a crosshair and a nearest-point highlight drawn over them; the wheel
zooms and a drag pans. It depends on iced at a pinned git revision; use
the same revision in your app:

```toml
[dependencies]
rustd3 = { git = "https://github.com/dashdeckers/rustd3", features = ["figure"] }
rustd3-iced = { git = "https://github.com/dashdeckers/rustd3", features = ["figure"] }
iced = { git = "https://github.com/iced-rs/iced", rev = "3cbdacc3b0841c5145c766c3a758657d36b41ffe" }
```

The app builds the figure in `view` and owns the x window here; y stays
the widget's:

```rust
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

`rustd3_iced::path` draws the core level's geometry into an iced canvas
path, and `rustd3_iced::pointer::Tracker` turns iced's events into the
pointer input of `rustd3::interact`'s zoom, brushes and drag; the guide
shows both.

## Features

| Feature | Adds |
| --- | --- |
| none | the core: scales, ticks, axes, shapes, formats, time, locales, color and schemes, interpolation and easing, array statistics, recordings, zoom, brush, drag, animation |
| `derive` | `#[derive(Interpolate)]` in `rustd3::interpolate` |
| `figure` | figures, layout, typst text and math, the SVG writer |
| `fonts` (default) | DejaVu Sans in two weights and New Computer Modern Math, bundled (2.8 MB); it does not turn `figure` on |
| `raster` | vello_cpu rendering the drawing list into pixels |
| `png`, `gif` | `raster` and an encoder |
| `pdf` | PDF through krilla, with selectable text |
| `tiny-skia` | the fallback rasterizer |
| `helpers` | ready-made charts (`helpers::chart`, `helpers::stats`) and `helpers::subplots` |

Every feature but `fonts`, `derive` and `figure` itself turns `figure`
on. `gif::Writer` writes its frames' text with the bundled fonts, so it
needs `fonts`. The `Plot` widget is `rustd3-iced`'s `figure` feature.

## Outputs

- **SVG**, always: `Figure::save_svg`, text as glyph outlines.
- **PNG** (`png`): `Figure::save_png` at any device scale, rendered by
  vello_cpu; `tiny-skia` renders the same list as a fallback.
- **PDF** (`pdf`): `Figure::save_pdf`, text selectable in subset fonts.
- **GIF** (`gif`): `rustd3::figure::gif`, sequences of frames.
- **iced**: the `Plot` widget shows the pixels the PNG has.

Every output is drawn from the same drawing list
(`rustd3::figure::list::List`), so they cannot drift apart, and a list
can be rendered by a backend of your own.

## Performance

Text is laid out once per label and style and cached; long recordings
are thinned at draw time to at most four points per pixel column (M4), so
drawing costs what the output shows, not what the data holds; the `Plot`
widget renders only what changed, moves cached tiles on a pan, and
renders on a thread pool when a render would not fit the frame. The guide
has the details.

To measure a machine, run the benchmark from a checkout:

```bash
cargo run --release --example bench_all -- --quick                       # the library and the widget
cargo run --release -p rustd3-figure --example bench_figure -- --quick   # the library alone, headless
```

`bench_all` takes about a quarter of an hour, drives the widget and times
figures from data to SVG, PDF and PNG, and ends by printing the path of
one archive; if a plot is slow, open an issue and attach the archive.
`bench_figure` times the figures alone, in about a minute, without iced.
`bench_gestures` and `bench_software` run the widget's profiles one at a
time, and `bench_compare` and `bench_report` read archives.
[benchkit/README.md](benchkit/README.md) describes the levels, the flags
and what the archive holds.

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
