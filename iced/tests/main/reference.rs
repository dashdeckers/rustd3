//! The reference the widget's pixels are compared with: the PNG of the
//! same figure, rendered by vello_cpu for the screen at the widget's
//! factor and composited onto the screenshot's background, its plot areas
//! as the widget draws them.
//!
//! The widget draws its plot areas with an overscan of two tiles (512
//! device px), so that a pan can bring in what lies beyond the view: a
//! thinned line crossing a plot area's left or right edge continues past
//! it there, where the file output ends it one sample beyond the edge.
//! Within half a line width of the edge the two differ (up to
//! 32/255 measured), so the reference takes each plot area's pixels
//! from the figure drawn with the same overscan without the other panels'
//! plot-area layers (which the overscan grows over it); everything else is
//! the PNG's.

use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::list::{Bounds, Command, List, Outlines, part};
use rustd3_figure::raster::{self, Settings, Simd};
use rustd3_figure::{Figure, Held, Target, layout, thin};

/// The widget's overscan: two tiles and a margin, device px.
const OVERSCAN: f64 = 520.0;

/// `fig` at `size` logical px and `scale` device px per px, as the PNG
/// output renders it (single-threaded, baseline SIMD), its plot areas
/// drawn with the widget's overscan, composited source-over onto
/// `background`.
pub fn png_of(
    fig: &Figure<'_>,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
) -> raster::Image {
    let draw = |held: &Held| {
        let mut cache = Cache::default();
        cache.begin();
        let resolved = fig.resolve(size, &mut cache, held, Target::Screen { scale });
        (
            fig.draw_resolved(&mut cache, &resolved),
            resolved.get_plot_areas().to_vec(),
        )
    };
    let (list, areas) = draw(&Held::default());
    let mut held = Held::default();
    held.overscan(OVERSCAN / scale);
    let (grown, _) = draw(&held);
    composed((&list, &grown), &areas, size, scale, background)
}

/// As [`png_of`], laid out in the plot areas `areas` a gesture holds.
pub fn png_held(
    fig: &Figure<'_>,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
    areas: &layout::Areas,
) -> raster::Image {
    png_held_on(fig, size, scale, background, (areas, Vec::new()))
}

/// As [`png_held`], its lines thinned on the column `grids` a drag holds
/// for their x handles (which also anchor dashes).
pub fn png_held_on(
    fig: &Figure<'_>,
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
    (areas, grids): (&layout::Areas, Vec<(usize, thin::Grid)>),
) -> raster::Image {
    let draw = |held: &Held| {
        let mut cache = Cache::default();
        cache.begin();
        let target = Target::Screen { scale };
        let resolved = fig.resolve_held(size, &mut cache, held, target, areas);
        (
            fig.draw_resolved(&mut cache, &resolved),
            resolved.get_plot_areas().to_vec(),
        )
    };
    let (list, rects) = draw(&Held::default());
    let mut held = Held::default();
    held.overscan(OVERSCAN / scale).grids(grids);
    let (grown, _) = draw(&held);
    composed((&list, &grown), &rects, size, scale, background)
}

/// `list` rendered, with the pixels of `areas` (list px) taken from
/// `grown`'s render, composited onto `background`.
fn composed(
    (list, grown): (&List, &List),
    areas: &[Rect],
    size: kurbo::Size,
    scale: f64,
    background: iced::Color,
) -> raster::Image {
    let mut image = render(list, size, scale);
    let width = image.width as usize;
    for (panel, area) in areas.iter().enumerate() {
        let inside = render(&only(grown, panel, areas.len()), size, scale);
        let [x0, y0, x1, y1] =
            [area.x0, area.y0, area.x1, area.y1].map(|e| (e * scale).round() as usize);
        for y in y0..y1 {
            let row = (y * width + x0) * 4..(y * width + x1) * 4;
            image.rgba[row.clone()].copy_from_slice(&inside.rgba[row]);
        }
    }
    onto(&mut image, background);
    image
}

/// `list` without the plot-area layers of the panels other than `panel`
/// (of `panels`): each panel's background (part role 2), grid lines (9)
/// and marks (6), with the group the marks are drawn in.
fn only(list: &List, panel: usize, panels: usize) -> List {
    let id = |role: u64, panel: usize| part::Id::new(role << 56 | (panel as u64) << 32);
    let moves = |p: &part::Id, q: usize| {
        [2, 6, 9]
            .into_iter()
            .any(|role| id(role, q) <= *p && *p < id(role, q + 1))
    };
    let mut drop = vec![false; list.commands.len()];
    for q in (0..panels).filter(|&q| q != panel) {
        let parts: Vec<_> = list.parts.iter().filter(|p| moves(&p.id, q)).collect();
        if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
            let mut end = last.commands.end;
            if matches!(list.commands.get(end), Some(Command::Pop)) {
                end += 1;
            }
            drop[first.commands.start..end].fill(true);
        }
    }
    List {
        commands: list
            .commands
            .iter()
            .zip(&drop)
            .filter(|(_, d)| !**d)
            .map(|(c, _)| c.clone())
            .collect(),
        parts: Vec::new(),
        ..list.clone()
    }
}

/// `list` as the PNG output renders it.
fn render(list: &List, size: kurbo::Size, scale: f64) -> raster::Image {
    raster::Renderer::new(Settings::default().simd(Simd::Baseline))
        .render(
            list,
            &Bounds::of(list, &mut Outlines::default()),
            Rect::from_origin_size((0.0, 0.0), size),
            scale,
        )
        .expect("the figure renders")
}

/// Composites `image` source-over onto `background`.
fn onto(image: &mut raster::Image, background: iced::Color) {
    let [r, g, b, a] = background.into_rgba8().map(f64::from);
    for pixel in image.rgba.as_chunks_mut::<4>().0.iter_mut() {
        let alpha = f64::from(pixel[3]) / 255.0;
        let under = a / 255.0 * (1.0 - alpha);
        let out = alpha + under;
        for (channel, back) in pixel.iter_mut().zip([r, g, b]) {
            let value = (f64::from(*channel) * alpha + back * under) / out;
            *channel = value.round() as u8;
        }
        pixel[3] = (out * 255.0).round() as u8;
    }
}
