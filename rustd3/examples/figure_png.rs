//! A damped wave as an SVG and a PNG.

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
