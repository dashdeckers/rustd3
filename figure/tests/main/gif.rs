//! GIF output: delays, repeat, the size and translucency errors, read
//! back by the gif crate's own decoder, and a frame against the PNG of
//! the same figure.

use std::time::Duration;

use rustd3_core::color::Rgb;
use rustd3_core::scale;
use rustd3_figure::gif::{Error, Repeat, Writer};
use rustd3_figure::kurbo::{Point, Rect, Size};
use rustd3_figure::list::{Bounds, Outlines};
use rustd3_figure::mark::line;
use rustd3_figure::raster::{self, Image};
use rustd3_figure::{Figure, Held, Target, Theme, axis};

use crate::raster::vello::diff;
use crate::scatter;

/// The frames' size, logical px.
const SIZE: Size = Size::new(160.0, 120.0);

/// A sine over [0, 10] seen through x in [lo, hi].
fn window(lo: f64, hi: f64) -> Figure<'static> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([lo, hi]));
    let y = fig.scale(scale::linear().domain([-1.2, 1.2]));
    let points: Vec<(f64, f64)> = (0..=200)
        .map(|i| {
            let t = f64::from(i) / 20.0;
            (t, t.sin())
        })
        .collect();
    fig.panel(0, 0)
        .axis(axis::bottom(x))
        .axis(axis::left(y))
        .mark(line(points, x, y));
    fig
}

/// A writer of `SIZE` at 1× into memory.
fn writer(repeat: Repeat) -> Writer<Vec<u8>> {
    match Writer::new(Vec::new(), SIZE, 1.0, repeat) {
        Ok(writer) => writer,
        Err(e) => panic!("a writer: {e}"),
    }
}

/// Each frame's delay and pixels (RGBA), and the loop count, as the gif
/// crate's decoder reads them.
fn decode(bytes: &[u8]) -> (Vec<(u16, Image)>, ::gif::Repeat) {
    let mut options = ::gif::DecodeOptions::new();
    options.set_color_output(::gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(bytes).expect("a GIF");
    let mut frames = Vec::new();
    while let Some(frame) = decoder.read_next_frame().expect("a frame") {
        let image = Image {
            width: u32::from(frame.width),
            height: u32::from(frame.height),
            rgba: frame.buffer.to_vec(),
        };
        frames.push((frame.delay, image));
    }
    (frames, decoder.repeat())
}

#[test]
fn delays_round_to_centiseconds_and_never_below_two() {
    let mut gif = writer(Repeat::Forever);
    let fig = window(0.0, 10.0);
    for ms in [40.0, 16.7, 5.0] {
        gif.frame(&fig, Duration::from_secs_f64(ms / 1000.0))
            .expect("a frame");
    }
    let (frames, _) = decode(&gif.finish().expect("the trailer"));
    let delays: Vec<u16> = frames.iter().map(|(delay, _)| *delay).collect();
    assert_eq!(delays, [4, 2, 2]);
}

#[test]
fn a_thirty_frame_zoom_repeats_forever() {
    let mut gif = writer(Repeat::Forever);
    for i in 0..30 {
        let k = 0.93_f64.powi(i);
        gif.frame(
            &window(5.0 - 5.0 * k, 5.0 + 5.0 * k),
            Duration::from_millis(40),
        )
        .expect("a frame");
    }
    let (frames, repeat) = decode(&gif.finish().expect("the trailer"));
    assert_eq!(frames.len(), 30);
    assert_eq!(repeat, ::gif::Repeat::Infinite);
    assert!(
        frames
            .iter()
            .all(|(_, f)| (f.width, f.height) == (160, 120))
    );
    assert_ne!(frames[0].1, frames[29].1, "the frames move");
}

#[test]
fn a_loop_count_is_written_before_the_frames() {
    let mut gif = writer(Repeat::Times(3));
    gif.frame(&window(0.0, 10.0), Duration::from_millis(100))
        .expect("a frame");
    let (frames, repeat) = decode(&gif.finish().expect("the trailer"));
    assert_eq!((frames.len(), repeat), (1, ::gif::Repeat::Finite(3)));
}

#[test]
fn a_40000_px_wide_figure_at_2x_exceeds_the_limit() {
    let Err(e) = Writer::new(Vec::new(), Size::new(40_000.0, 10.0), 2.0, Repeat::Forever) else {
        panic!("80,000 device px fit no GIF");
    };
    assert!(
        matches!(
            e,
            Error::Size {
                width: 80_000,
                height: 20
            }
        ),
        "{e}"
    );
}

#[test]
fn a_translucent_background_is_an_error() {
    let mut fig = window(0.0, 10.0);
    let half = Rgb {
        r: 255.0,
        g: 255.0,
        b: 255.0,
        opacity: 0.5,
    };
    fig.theme(Theme::default().background(half));
    let mut gif = writer(Repeat::Forever);
    let e = gif
        .frame(&fig, Duration::from_millis(40))
        .expect_err("GIF has 1-bit alpha");
    assert!(matches!(e, Error::Translucent), "{e}");
}

#[test]
fn a_frame_matches_the_png_of_its_figure_within_the_bound() {
    let fig = scatter::scatter(scatter::POINTS);
    let size = scatter::SIZE;
    // The pixels `save_png` encodes: the figure drawn for a file at 1× and
    // rendered on vello_cpu's pool.
    let target = Target::File { scale: 1.0 };
    let list = fig.draw_with(&mut crate::figure::cache(), size, target, &Held::default());
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let png = raster::Renderer::new(raster::Settings::parallel())
        .render(
            &list,
            &bounds,
            Rect::from_origin_size(Point::ORIGIN, size),
            1.0,
        )
        .expect("the figure renders");
    let mut gif = match Writer::new(Vec::new(), size, 1.0, Repeat::Forever) {
        Ok(writer) => writer,
        Err(e) => panic!("a writer: {e}"),
    };
    gif.frame(&fig, Duration::from_millis(100))
        .expect("a frame");
    let (frames, _) = decode(&gif.finish().expect("the trailer"));
    let d = diff(&frames[0].1, &png);
    eprintln!(
        "gif frame against its png: max {}, tile score {:.4} at {:?}, {:.3}% over 16",
        d.max(),
        d.tile_score(),
        d.worst_tile(),
        100.0 * d.share_over()
    );
    // Measured: max 60/255, tile score 0.0095 (decisions/phase-37.md).
    assert!(d.max() <= 90, "max {}", d.max());
    assert!(
        d.tile_score() <= 1.5 * 0.0095,
        "tile score {:.4}",
        d.tile_score()
    );
}
