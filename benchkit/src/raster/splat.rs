//! Marker splatting, which composites each markers command of a plot-area
//! cell from coverage stamps instead of filling the shape once per point.
//! The shape is rasterized once at 16 quarter-pixel phases (4 × 4) and
//! laid source-over at every point. Everything else renders as drawn, in
//! order.
//!
//! The figure's renderer renders the commands between markers commands,
//! over their bounds only. The stamps are its own renders of the shape
//! alone, so only the phase (rounded to the nearest quarter pixel) and the
//! compositing's rounding differ from filling each point.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use rustd3_figure::kurbo::{Affine, BezPath, PathEl, Point, Rect, Shape, Size};
use rustd3_figure::list::{Bounds, Clip, Command, List, Outlines};
use rustd3_figure::peniko::{Brush, Color};
use rustd3_figure::raster::{self, Image, Renderer};

/// Sub-pixel phases per axis.
const PHASES: usize = 4;

/// One cell's splatting render, prepared once per list: the runs of
/// commands rendered as drawn and the markers commands splatted, in order.
#[derive(Debug)]
pub struct Plan<'a> {
    list: &'a List,
    /// The cell, list px.
    area: Rect,
    /// Device px per list px.
    k: f64,
    /// The runs of commands rendered as drawn, with their bounds.
    runs: Vec<(List, Bounds)>,
    steps: Vec<Step>,
}

/// One step of a [`Plan`].
#[derive(Debug)]
enum Step {
    /// Run `run` of [`Plan::runs`] rendered over `area`, list px, and laid
    /// over the cell.
    Drawn { run: usize, area: Rect },
    /// A markers command splatted through `device` (its group's space into
    /// the cell's device px), clipped to `clip` (device px).
    Splat {
        command: usize,
        device: Affine,
        clip: [i64; 4],
    },
}

/// The stamps of one shape under one linear transform, kept between
/// renders.
#[derive(Debug, Default)]
pub struct Stamps {
    sets: HashMap<u64, Vec<Stamp>>,
}

/// A shape's coverage at one phase: 8-bit, row-major, the shape's center
/// at `offset` + the phase.
#[derive(Debug, Clone)]
struct Stamp {
    width: usize,
    height: usize,
    /// The stamp's top-left pixel relative to the pixel the center is in.
    offset: [i64; 2],
    coverage: Vec<u8>,
}

impl<'a> Plan<'a> {
    /// The plan for `list` (with its `bounds`) over the cell `area` (list
    /// px) at `k`. Unsized markers commands of solid paints, in groups that
    /// only scale, translate and clip to rectangles, are splatted.
    /// Everything else is rendered as drawn. Returns `None` when no markers
    /// command in the cell can be splatted.
    pub fn new(list: &'a List, bounds: &Bounds, area: Rect, k: f64) -> Option<Plan<'a>> {
        let cell = Affine::scale(k) * Affine::translate((-area.x0, -area.y0));
        let size = [
            (area.width() * k).round() as i64,
            (area.height() * k).round() as i64,
        ];
        let margin = 1.0 / k;
        let visible = |i: usize| {
            bounds
                .get(i)
                .map(|b| b.intersect(area.inflate(margin, margin)))
                .filter(|b| b.area() > 0.0)
        };
        // Open groups: their Push's index, transform into list px and the
        // clip so far in device px.
        let mut groups: Vec<(usize, Affine, Option<[f64; 4]>, bool)> =
            vec![(usize::MAX, Affine::IDENTITY, None, true)];
        let mut steps = Vec::new();
        let mut runs = Vec::new();
        let mut run = Run::default();
        let mut splatted = 0;
        for (i, command) in list.commands.iter().enumerate() {
            let &(_, to_list, clip, rectangular) = groups
                .last()
                .expect("invariant: the root group is never popped");
            match command {
                Command::Push {
                    transform,
                    clip: own,
                } => {
                    let next = to_list * *transform;
                    let (clip, rectangular) = match own {
                        Some(Clip::Rect(r)) if axis_aligned(cell * next) => {
                            let d = (cell * next).transform_rect_bbox(*r);
                            let mine = [d.x0, d.y0, d.x1, d.y1];
                            (Some(clip.map_or(mine, |c| intersect(c, mine))), rectangular)
                        }
                        Some(_) => (clip, false),
                        None => (clip, rectangular),
                    };
                    groups.push((i, next, clip, rectangular));
                    run.commands.push(i);
                }
                Command::Pop => {
                    groups.pop();
                    run.commands.push(i);
                }
                Command::Markers {
                    sizes: None,
                    paint,
                    paints,
                    ..
                } if visible(i).is_some()
                    && rectangular
                    && axis_aligned(cell * to_list)
                    && solid(paint, paints.as_ref()) =>
                {
                    let open: Vec<usize> = groups.iter().skip(1).map(|g| g.0).collect();
                    run.finish(list, (&mut runs, &mut steps), (area, k));
                    run = Run::after(open);
                    let clip = clip.unwrap_or([0.0, 0.0, size[0] as f64, size[1] as f64]);
                    steps.push(Step::Splat {
                        command: i,
                        device: cell * to_list,
                        clip: [
                            clip[0].round().max(0.0) as i64,
                            clip[1].round().max(0.0) as i64,
                            (clip[2].round() as i64).min(size[0]),
                            (clip[3].round() as i64).min(size[1]),
                        ],
                    });
                    splatted += 1;
                }
                Command::Fill { .. }
                | Command::Stroke { .. }
                | Command::Markers { .. }
                | Command::Glyphs(_)
                | Command::Image { .. }
                | _ => {
                    run.commands.push(i);
                    if let Some(b) = visible(i) {
                        let d = cell.transform_rect_bbox(b).expand();
                        run.reach.push([d.x0, d.y0, d.x1, d.y1].map(|v| v as i64));
                    }
                }
            }
        }
        run.finish(list, (&mut runs, &mut steps), (area, k));
        (splatted > 0).then_some(Plan {
            list,
            area,
            k,
            runs,
            steps,
        })
    }

    /// Renders the cell: the drawn runs with `renderer`, the markers from
    /// `stamps` (made on first use).
    pub fn render(
        &self,
        renderer: &mut Renderer,
        stamps: &mut Stamps,
    ) -> Result<Image, raster::Error> {
        let (width, height) = (
            (self.area.width() * self.k).round() as u32,
            (self.area.height() * self.k).round() as u32,
        );
        let mut canvas = Image {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        let mut blank = true;
        for step in &self.steps {
            match step {
                Step::Drawn { run, area } => {
                    let (list, bounds) = &self.runs[*run];
                    let image = renderer.render(list, bounds, *area, self.k)?;
                    let at = [
                        ((area.x0 - self.area.x0) * self.k).round() as i64,
                        ((area.y0 - self.area.y0) * self.k).round() as i64,
                    ];
                    if blank && at == [0, 0] && (image.width, image.height) == (width, height) {
                        canvas = image;
                    } else {
                        over(&mut canvas, &image, at);
                    }
                    blank = false;
                }
                Step::Splat {
                    command,
                    device,
                    clip,
                } => {
                    let Command::Markers {
                        shape,
                        at,
                        paint,
                        paints,
                        ..
                    } = &self.list.commands[*command]
                    else {
                        continue;
                    };
                    let set = stamps.of(shape, *device)?;
                    let colors: Vec<[u8; 4]> = match paints {
                        Some(p) => p.table.iter().map(rgba).collect(),
                        None => vec![rgba(paint)],
                    };
                    for (n, p) in at.iter().enumerate() {
                        let color = colors[paints.as_ref().map_or(0, |p| p.index[n] as usize)];
                        splat(&mut canvas, set, *device * *p, color, *clip);
                    }
                    blank = false;
                }
            }
        }
        Ok(canvas)
    }
}

/// A run of commands between splatted markers: their indices (with the
/// groups' ends inside), the groups open before it and where its visible
/// commands reach, device px of the cell (`[x0, y0, x1, y1]`).
#[derive(Debug, Default)]
struct Run {
    open: Vec<usize>,
    commands: Vec<usize>,
    reach: Vec<[i64; 4]>,
}

impl Run {
    /// A run after a splatted command, inside the groups `open` opened.
    fn after(open: Vec<usize>) -> Run {
        Run {
            open,
            ..Run::default()
        }
    }

    /// Pushes the run when anything in it is visible in the cell `area` at
    /// `k`. It pushes its commands inside the groups open around it, with
    /// one step per rectangle of a disjoint cover of where they reach, so a
    /// strip along each edge renders without the cell between them.
    fn finish(
        &mut self,
        list: &List,
        (runs, steps): (&mut Vec<(List, Bounds)>, &mut Vec<Step>),
        (area, k): (Rect, f64),
    ) {
        let size = [
            (area.width() * k).round() as i64,
            (area.height() * k).round() as i64,
        ];
        let reach: Vec<[i64; 4]> = self
            .reach
            .iter()
            .map(|r| {
                [
                    r[0].max(0),
                    r[1].max(0),
                    r[2].min(size[0]),
                    r[3].min(size[1]),
                ]
            })
            .filter(|r| r[0] < r[2] && r[1] < r[3])
            .collect();
        if reach.is_empty() {
            return;
        }
        let mut commands: Vec<Command> = self
            .open
            .iter()
            .map(|&j| list.commands[j].clone())
            .collect();
        commands.extend(self.commands.iter().map(|&j| list.commands[j].clone()));
        let depth = commands.iter().fold(0_i64, |d, c| match c {
            Command::Push { .. } => d + 1,
            Command::Pop => d - 1,
            Command::Fill { .. }
            | Command::Stroke { .. }
            | Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. }
            | _ => d,
        });
        commands.extend((0..depth).map(|_| Command::Pop));
        let sub = List {
            size: list.size,
            fonts: list.fonts.clone(),
            commands,
            parts: Vec::new(),
        };
        let bounds = Bounds::of(&sub, &mut Outlines::default());
        runs.push((sub, bounds));
        for r in cover(&reach) {
            let [x0, y0, x1, y1] = r.map(|v| v as f64 / k);
            steps.push(Step::Drawn {
                run: runs.len() - 1,
                area: Rect::new(area.x0 + x0, area.y0 + y0, area.x0 + x1, area.y0 + y1),
            });
        }
    }
}

/// Disjoint rectangles covering exactly the union of `rects`. Each band
/// between their y edges holds the merged x intervals, and bands with the
/// same intervals are joined.
fn cover(rects: &[[i64; 4]]) -> Vec<[i64; 4]> {
    let mut ys: Vec<i64> = rects.iter().flat_map(|r| [r[1], r[3]]).collect();
    ys.sort_unstable();
    ys.dedup();
    let mut bands: Vec<(i64, i64, Vec<[i64; 2]>)> = Vec::new();
    for pair in ys.windows(2) {
        let (y0, y1) = (pair[0], pair[1]);
        let mut xs: Vec<[i64; 2]> = rects
            .iter()
            .filter(|r| r[1] <= y0 && r[3] >= y1)
            .map(|r| [r[0], r[2]])
            .collect();
        xs.sort_unstable();
        let mut merged: Vec<[i64; 2]> = Vec::new();
        for x in xs {
            match merged.last_mut() {
                Some(last) if x[0] <= last[1] => last[1] = last[1].max(x[1]),
                Some(_) | None => merged.push(x),
            }
        }
        match bands.last_mut() {
            Some(last) if last.1 == y0 && last.2 == merged => last.1 = y1,
            Some(_) | None if merged.is_empty() => {}
            Some(_) | None => bands.push((y0, y1, merged)),
        }
    }
    bands
        .into_iter()
        .flat_map(|(y0, y1, xs)| xs.into_iter().map(move |[x0, x1]| [x0, y0, x1, y1]))
        .collect()
}

impl Stamps {
    /// The 16 stamps of `shape` under `device`'s linear part, made with
    /// the figure's renderer on first use.
    fn of(&mut self, shape: &BezPath, device: Affine) -> Result<&[Stamp], raster::Error> {
        let [a, b, c, d, _, _] = device.as_coeffs();
        let linear = Affine::new([a, b, c, d, 0.0, 0.0]);
        let points = shape.elements().iter().flat_map(|e| match *e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => vec![p],
            PathEl::QuadTo(p, q) => vec![p, q],
            PathEl::CurveTo(p, q, r) => vec![p, q, r],
            PathEl::ClosePath => Vec::new(),
        });
        let key = fnv(points
            .flat_map(|p| [p.x, p.y])
            .chain([a, b, c, d])
            .flat_map(f64::to_le_bytes));
        let set = match self.sets.entry(key) {
            Entry::Occupied(set) => set.into_mut(),
            Entry::Vacant(slot) => slot.insert(stamps(shape, linear)?),
        };
        Ok(set)
    }
}

/// The 16 stamps of `shape` under `linear`, phase (x, y) at index
/// `4 y + x`. Each is opaque black filled by the figure's renderer, with
/// coverage read from alpha.
fn stamps(shape: &BezPath, linear: Affine) -> Result<Vec<Stamp>, raster::Error> {
    let placed = linear * shape.clone();
    let b = placed.bounding_box();
    let offset = [b.x0.floor() as i64 - 1, b.y0.floor() as i64 - 1];
    let (width, height) = (
        (b.x1.ceil() as i64 - offset[0] + 2) as usize,
        (b.y1.ceil() as i64 - offset[1] + 2) as usize,
    );
    let mut set = Vec::with_capacity(PHASES * PHASES);
    for py in 0..PHASES {
        for px in 0..PHASES {
            let phase = [px as f64 / PHASES as f64, py as f64 / PHASES as f64];
            let at = Affine::translate((phase[0] - offset[0] as f64, phase[1] - offset[1] as f64));
            let list = List {
                size: Size::new(width as f64, height as f64),
                fonts: Vec::new(),
                commands: vec![Command::Fill {
                    path: at * placed.clone(),
                    paint: Brush::Solid(Color::from_rgba8(0, 0, 0, 255)),
                }],
                parts: Vec::new(),
            };
            let image = raster::render(&list, 1.0)?;
            set.push(Stamp {
                width,
                height,
                offset,
                coverage: image.rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect(),
            });
        }
    }
    Ok(set)
}

/// Lays the stamp of the phase nearest `at` (device px) in `color`
/// (straight RGBA) source-over onto `canvas`, inside `clip`.
fn splat(canvas: &mut Image, set: &[Stamp], at: Point, color: [u8; 4], clip: [i64; 4]) {
    if !at.is_finite() {
        return;
    }
    let q = PHASES as f64;
    let (mut ix, mut iy) = (at.x.floor() as i64, at.y.floor() as i64);
    let (mut fx, mut fy) = (
        ((at.x - ix as f64) * q).round() as usize,
        ((at.y - iy as f64) * q).round() as usize,
    );
    if fx == PHASES {
        ix += 1;
        fx = 0;
    }
    if fy == PHASES {
        iy += 1;
        fy = 0;
    }
    let stamp = &set[fy * PHASES + fx];
    let (x0, y0) = (ix + stamp.offset[0], iy + stamp.offset[1]);
    let width = canvas.width as i64;
    let xs = x0.max(clip[0])..(x0 + stamp.width as i64).min(clip[2]);
    let ys = y0.max(clip[1])..(y0 + stamp.height as i64).min(clip[3]);
    let alpha = u32::from(color[3]);
    for y in ys {
        let row = ((y - y0) as usize) * stamp.width;
        for x in xs.clone() {
            let coverage = u32::from(stamp.coverage[row + (x - x0) as usize]);
            if coverage == 0 {
                continue;
            }
            let i = ((y * width + x) * 4) as usize;
            let a = (coverage * alpha + 127) / 255;
            blend(
                &mut canvas.rgba[i..i + 4],
                [color[0], color[1], color[2]],
                a,
            );
        }
    }
}

/// `image` laid source-over onto `canvas` with its top-left pixel at `at`.
fn over(canvas: &mut Image, image: &Image, at: [i64; 2]) {
    for y in 0..image.height as i64 {
        let cy = y + at[1];
        if cy < 0 || cy >= canvas.height as i64 {
            continue;
        }
        for x in 0..image.width as i64 {
            let cx = x + at[0];
            if cx < 0 || cx >= canvas.width as i64 {
                continue;
            }
            let s = ((y * image.width as i64 + x) * 4) as usize;
            let a = u32::from(image.rgba[s + 3]);
            if a == 0 {
                continue;
            }
            let d = ((cy * canvas.width as i64 + cx) * 4) as usize;
            let [r, g, b] = [image.rgba[s], image.rgba[s + 1], image.rgba[s + 2]];
            blend(&mut canvas.rgba[d..d + 4], [r, g, b], a);
        }
    }
}

/// Source-over of straight `color` at alpha `a` (0–255) onto the straight
/// RGBA pixel `dst`.
fn blend(dst: &mut [u8], color: [u8; 3], a: u32) {
    let da = u32::from(dst[3]);
    if a == 255 || da == 0 {
        dst[..3].copy_from_slice(&color);
        dst[3] = a as u8;
        return;
    }
    if da == 255 {
        for (d, s) in dst.iter_mut().zip(color) {
            *d = ((u32::from(s) * a + u32::from(*d) * (255 - a) + 127) / 255) as u8;
        }
        return;
    }
    // Both translucent: in premultiplied terms, then back.
    let out = a * 255 + da * (255 - a);
    for (d, s) in dst.iter_mut().zip(color) {
        let premultiplied = u32::from(s) * a * 255 + u32::from(*d) * da * (255 - a);
        *d = ((premultiplied + out / 2) / out) as u8;
    }
    dst[3] = ((out + 127) / 255) as u8;
}

/// A solid brush as straight 8-bit RGBA; other brushes as transparent.
fn rgba(brush: &Brush) -> [u8; 4] {
    match brush {
        Brush::Solid(c) => {
            let c = c.to_rgba8();
            [c.r, c.g, c.b, c.a]
        }
        Brush::Gradient(_) | Brush::Image(_) => [0; 4],
    }
}

/// Whether every paint of a markers command is solid.
fn solid(paint: &Brush, paints: Option<&rustd3_figure::list::Paints>) -> bool {
    match paints {
        Some(p) => p.table.iter().all(|b| matches!(b, Brush::Solid(_))),
        None => matches!(paint, Brush::Solid(_)),
    }
}

/// Whether `t` only scales and translates.
fn axis_aligned(t: Affine) -> bool {
    let [_, b, c, _, _, _] = t.as_coeffs();
    b == 0.0 && c == 0.0
}

/// The intersection of two `[x0, y0, x1, y1]` rects.
fn intersect(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ]
}

/// FNV-1a 64 of `bytes`.
fn fnv(bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;
    use rustd3_figure::cache::Cache;
    use rustd3_figure::kurbo::{Rect, Size};
    use rustd3_figure::list::{Bounds, Outlines};
    use rustd3_figure::mark::dots;
    use rustd3_figure::raster::{Renderer, Settings};
    use rustd3_figure::{Figure, Held, Target};

    use super::{Plan, Stamps, blend, cover};

    #[test]
    fn source_over_matches_premultiplied_compositing() {
        let mut opaque = [10, 20, 30, 255];
        blend(&mut opaque, [250, 0, 100], 128);
        assert_eq!(opaque, [130, 10, 65, 255]);
        let mut clear = [0, 0, 0, 0];
        blend(&mut clear, [250, 0, 100], 128);
        assert_eq!(clear, [250, 0, 100, 128]);
        // Half over half: alpha 0.75, red (1·0.5 + 0.5·0.25) / 0.75.
        let mut half = [0, 0, 255, 128];
        blend(&mut half, [255, 0, 0], 128);
        assert_eq!(half[3], 192);
        assert!(half[0].abs_diff(170) <= 1, "{half:?}");
        assert!(half[2].abs_diff(85) <= 1, "{half:?}");
    }

    /// The largest channel difference between 400 dots filled and
    /// splatted at 1×, on a plot area 76 px square over a domain of 76, so
    /// one data unit is one device px. `phase` rounds the dots' centers to
    /// quarter pixels.
    fn worst(phase: bool) -> u8 {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 76.0]));
        let y = fig.scale(scale::linear().domain([0.0, 76.0]));
        let points: Vec<(f64, f64)> = (0..400)
            .map(|i| {
                let t = f64::from(i);
                let (a, b) = (
                    38.0 + (t * 0.37).sin() * 30.0,
                    38.0 + (t * 0.23).cos() * 30.0,
                );
                if phase {
                    ((a * 4.0).round() / 4.0, (b * 4.0).round() / 4.0)
                } else {
                    (a, b)
                }
            })
            .collect();
        fig.panel(0, 0).mark(dots(points, x, y));
        let (k, size) = (1.0, Size::new(100.0, 100.0));
        let target = Target::Screen { scale: k };
        let mut cache = Cache::default();
        let list = fig.draw_with(&mut cache, size, target, &Held::default());
        let area: Rect = fig
            .resolve(size, &mut cache, &Held::default(), target)
            .get_plot_areas()[0];
        assert_eq!(area.width(), 76.0);
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let mut renderer = Renderer::new(Settings::default());
        let drawn = renderer.render(&list, &bounds, area, k).expect("a render");
        let plan = Plan::new(&list, &bounds, area, k).expect("markers to splat");
        let splatted = plan
            .render(&mut renderer, &mut Stamps::default())
            .expect("a render");
        assert_eq!(
            (drawn.width, drawn.height),
            (splatted.width, splatted.height)
        );
        drawn
            .rgba
            .iter()
            .zip(&splatted.rgba)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn a_cover_of_two_edge_strips_is_disjoint_and_leaves_the_middle_out() {
        // A left strip and a bottom strip that share a corner.
        let rects = [[0, 0, 4, 100], [0, 96, 200, 100]];
        assert_eq!(cover(&rects), [[0, 0, 4, 96], [0, 96, 200, 100]]);
        // Overlapping and touching intervals merge.
        assert_eq!(
            cover(&[[0, 0, 10, 10], [5, 0, 20, 10], [20, 0, 30, 10]]),
            [[0, 0, 30, 10]]
        );
        assert!(cover(&[]).is_empty());
    }

    #[test]
    fn splatted_dots_on_quarter_pixels_match_filled_ones() {
        // Only the compositing's rounding differs.
        let on = worst(true);
        assert!(on <= 4, "max {on}/255");
        // Elsewhere a center moves up to an eighth of a pixel along each
        // axis to its stamp's phase. That changes up to a quarter of an
        // edge pixel's coverage, 64/255 for a dark dot on white.
        let off = worst(false);
        assert!((on..=68).contains(&off), "max {off}/255");
    }
}
