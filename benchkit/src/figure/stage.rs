//! [`Stage`]: what a user of the plotting library pays for one figure, on
//! its way from data to files, each stage timed on its own through the
//! library's public API with its default settings: building the
//! [`Figure`], resolving it, drawing its list, then each output. SVG and
//! PDF are written from that one list, as `save_svg` and `save_pdf` write
//! theirs. A PNG (and a GIF) is drawn for a file target, which snaps to
//! device pixels and thins at its own scale, so it needs a resolve and a
//! list of its own: its stage is `save_png`'s whole work, end to end. Every
//! output goes to memory, not to a file, so no disk is timed.

use std::error::Error;
use std::time::{Duration, Instant};

use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::{Point, Rect, Size};
use rustd3_figure::list::{Bounds, Outlines};
use rustd3_figure::{Figure, Held, Target, gif, lower, pdf, raster, svg};

/// One stage of a figure's way to its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Building the [`Figure`] from data in memory: marks collect what
    /// they own (a histogram its values, a density its estimate).
    Build,
    /// [`Figure::resolve`] for vector output with a fresh [`Cache`], as
    /// `Figure::draw` starts: fitting, layout and typst text.
    Resolve,
    /// [`Figure::draw_resolved`] of that resolve: the drawing list.
    List,
    /// SVG of that list: lowered for SVG and written with glyph outlines,
    /// what `save_svg` does after drawing.
    Svg,
    /// PDF of that list, what `save_pdf` does after drawing.
    Pdf,
    /// `save_png`'s work at this many device px per logical px, end to
    /// end: drawn for a file target with a fresh cache, rendered by
    /// vello_cpu on its own pool, encoded.
    Png(u8),
    /// A GIF of [`GIF_FRAMES`] frames of the figure at 1× through
    /// `gif::Writer`, end to end.
    Gif,
}

/// One stage's time in one repetition, and what it wrote.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured {
    /// The stage.
    pub stage: Stage,
    /// Its wall time, ms.
    pub ms: f64,
    /// The output's bytes, for the outputs.
    pub bytes: Option<u64>,
}

/// The frames of the GIF stage.
pub const GIF_FRAMES: u32 = 3;

impl Stage {
    /// Its name in the reports and `results.json`.
    pub fn name(self) -> &'static str {
        match self {
            Stage::Build => "build",
            Stage::Resolve => "resolve",
            Stage::List => "list",
            Stage::Svg => "svg",
            Stage::Pdf => "pdf",
            Stage::Png(1) => "png-1x",
            Stage::Png(2) => "png-2x",
            Stage::Png(_) => "png",
            Stage::Gif => "gif",
        }
    }
}

/// One repetition: builds the figure with `build`, resolves it for
/// vector output at `size` with a fresh cache, draws its list, then writes
/// each of `outputs`; every stage timed apart. The first error ends it.
pub fn repetition<'a>(
    build: &dyn Fn() -> Figure<'a>,
    size: Size,
    outputs: &[Stage],
) -> Result<Vec<Measured>, Box<dyn Error>> {
    let mut out = Vec::with_capacity(3 + outputs.len());
    let mut time = |stage: Stage, begin: Instant, bytes: Option<usize>| {
        out.push(Measured {
            stage,
            ms: begin.elapsed().as_secs_f64() * 1e3,
            bytes: bytes.map(|n| n as u64),
        });
    };
    let begin = Instant::now();
    let fig = build();
    time(Stage::Build, begin, None);
    let begin = Instant::now();
    let mut cache = Cache::default();
    cache.begin();
    let resolved = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
    time(Stage::Resolve, begin, None);
    let begin = Instant::now();
    let list = fig.draw_resolved(&mut cache, &resolved);
    time(Stage::List, begin, None);
    for &stage in outputs {
        let begin = Instant::now();
        let bytes = match stage {
            Stage::Svg => {
                let lowered = lower::lower(&list, svg::CAPS)?;
                svg::write(&lowered.list, &svg::Options::default())?.len()
            }
            Stage::Pdf => pdf::write(&list, &pdf::Options::default())?.len(),
            Stage::Png(scale) => png(&fig, size, f64::from(scale))?.len(),
            Stage::Gif => gif(&fig, size)?.len(),
            Stage::Build | Stage::Resolve | Stage::List => continue,
        };
        time(stage, begin, Some(bytes));
    }
    Ok(out)
}

/// The PNG `Figure::save_png` writes of `fig` at `size` × `scale`, in
/// memory: its own draw for the file target with a fresh cache, vello_cpu
/// on its pool (`Settings::parallel`), the encoder.
pub fn png(fig: &Figure<'_>, size: Size, scale: f64) -> Result<Vec<u8>, Box<dyn Error>> {
    let target = Target::File { scale };
    let list = fig.draw_with(&mut Cache::default(), size, target, &Held::default());
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let area = Rect::from_origin_size(Point::ORIGIN, size);
    let mut renderer = raster::Renderer::new(raster::Settings::parallel());
    let image = renderer.render(&list, &bounds, area, scale)?;
    Ok(raster::png::encode(&image)?)
}

/// A GIF of [`GIF_FRAMES`] frames of `fig` at `size`, 1×, 100 ms each.
fn gif(fig: &Figure<'_>, size: Size) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut writer = gif::Writer::new(Vec::new(), size, 1.0, gif::Repeat::Forever)?;
    for _ in 0..GIF_FRAMES {
        writer.frame(fig, Duration::from_millis(100))?;
    }
    Ok(writer.finish()?)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rustd3_core::scale;
    use rustd3_figure::kurbo::Size;
    use rustd3_figure::mark::{dots, line};
    use rustd3_figure::text::typst;
    use rustd3_figure::{Figure, axis};

    use super::{Stage, png, repetition};

    /// A small figure with a line, dots and math.
    fn small<'a>() -> Figure<'a> {
        let wave: Vec<(f64, f64)> = (0..=200)
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
            .mark(line(wave.clone(), t, y))
            .mark(dots(wave.into_iter().step_by(10), t, y))
            .axis(axis::bottom(t).label("time [s]"))
            .axis(axis::left(y).label(typst("$e^(-t slash 8) sin t$")));
        fig
    }

    #[test]
    fn every_stage_is_timed_in_order_and_outputs_have_bytes() {
        let outputs = [Stage::Svg, Stage::Pdf, Stage::Png(1), Stage::Gif];
        let measured = repetition(&small, Size::new(320.0, 200.0), &outputs).unwrap();
        let stages: Vec<&str> = measured.iter().map(|m| m.stage.name()).collect();
        assert_eq!(
            stages,
            ["build", "resolve", "list", "svg", "pdf", "png-1x", "gif"]
        );
        for m in &measured {
            assert!(m.ms >= 0.0);
            let output = outputs.contains(&m.stage);
            assert_eq!(m.bytes.is_some_and(|b| b > 0), output, "{m:?}");
        }
    }

    #[test]
    fn the_outputs_are_the_bytes_the_library_saves() {
        // The stages time the library's own calls: what they write in
        // memory is what save_svg, save_pdf and save_png write to files.
        let dir = std::env::temp_dir().join(format!("rustd3-figure-stage-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let fig = small();
        let size = Size::new(320.0, 200.0);
        fig.save_svg(dir.join("a.svg"), size).unwrap();
        fig.save_pdf(dir.join("a.pdf"), size).unwrap();
        fig.save_png(dir.join("a.png"), size, 2.0).unwrap();
        let measured = repetition(&small, size, &[Stage::Svg, Stage::Pdf]).unwrap();
        let bytes = |stage: Stage| measured.iter().find(|m| m.stage == stage).unwrap().bytes;
        let len = |name: &str| fs::metadata(dir.join(name)).unwrap().len();
        assert_eq!(bytes(Stage::Svg), Some(len("a.svg")));
        assert_eq!(bytes(Stage::Pdf), Some(len("a.pdf")));
        assert_eq!(
            png(&fig, size, 2.0).unwrap(),
            fs::read(dir.join("a.png")).unwrap()
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
