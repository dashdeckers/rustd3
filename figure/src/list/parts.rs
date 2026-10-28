//! Part hashes: FNV-1a 64 over what each part of a list drew, so that a
//! figure drawn again from equal data hashes equally and a part drawn
//! elsewhere or differently does not; and [`Cells`], the canvas cut at
//! every plot-area edge, with the parts each cell touches, a hash of them
//! and their raster work, which a widget renders and skips by.

use std::fmt;
use std::ops::Range;

use kurbo::{Affine, BezPath, PathEl, Point, Rect, Shape};
use peniko::{Brush, FontData, ImageData};

use crate::list::{Clip, Command, GlyphRun, List, Paints};

/// FNV-1a 64's offset basis.
const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64's prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Path elements per raster work item of a fill or stroke.
const ELEMENTS: u64 = 16;

/// The canvas cut into cells at device-pixel edges: the column and row
/// edges, and the cells row-major (cell `row × (cols.len() − 1) + col`).
/// Plain data.
///
/// ```
/// use rustd3_figure::list::{self, List};
///
/// let list = List::default();
/// let cells = list::cells(&list, &[0.0, 100.0, 200.0], &[0.0, 50.0], 1.0);
/// assert_eq!(cells.cell_at([100, 10]), Some(1));
/// assert_eq!(cells.cell_at([200, 10]), None);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cells {
    /// Column edges in device px, increasing.
    pub cols: Vec<u32>,
    /// Row edges in device px, increasing.
    pub rows: Vec<u32>,
    /// The cells, row-major.
    pub cells: Vec<Cell>,
}

/// One cell of [`Cells`]. Plain data.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Device px, integer edges.
    pub rect: Rect,
    /// Indices into `List::parts` whose device bounds touch the cell.
    pub parts: Vec<u32>,
    /// The cell rect and its parts' hashes, in order.
    pub hash: u64,
    /// Raster work items of the listed parts.
    pub work: u64,
}

impl Cells {
    /// The cell containing a device pixel; cells are half-open, so a pixel
    /// on the last edge, or outside the canvas, is in none.
    pub fn cell_at(&self, device: [i32; 2]) -> Option<usize> {
        let col = at(&self.cols, device[0])?;
        let row = at(&self.rows, device[1])?;
        Some(row * (self.cols.len() - 1) + col)
    }
}

/// The canvas of `list` cut at `xs` and `ys` (list px, each snapped to
/// device px at `scale`: `round(x · scale)`, sorted and deduplicated; edges
/// that are not finite are ignored, negative ones taken as 0), with each
/// cell's parts bucketed once: the parts whose bounds in device px,
/// inflated by one device px for antialiasing but not past the clip the
/// part is drawn in (taken out to whole device px), share interior with
/// the cell, so marks clipped to a plot area whose edges lie on device
/// pixels touch only its cells. A cell's hash is FNV-1a 64 of its rect and its parts' hashes in
/// order; its work counts one item per `Fill`, `Stroke` or `Image` command
/// of its parts plus ⌊elements / 16⌋ of a fill's or stroke's path, one per
/// point of a `Markers` command and one per glyph of a `Glyphs` command;
/// a part that touches several cells counts fully in each. Parts that
/// draw nothing (bounds of no area) touch no cell.
pub fn cells(list: &List, xs: &[f64], ys: &[f64], scale: f64) -> Cells {
    let cols = edges(xs, scale);
    let rows = edges(ys, scale);
    let (across, down) = (cols.len().saturating_sub(1), rows.len().saturating_sub(1));
    let mut parts: Vec<Vec<u32>> = vec![Vec::new(); across * down];
    let clips = clips(list);
    for (index, (part, clip)) in list.parts.iter().zip(clips).enumerate() {
        if part.bounds.area() <= 0.0 {
            continue;
        }
        let inflated = part.bounds.scale_from_origin(scale).inflate(1.0, 1.0);
        let device = match clip {
            Some(clip) => inflated.intersect(outward(clip.scale_from_origin(scale))),
            None => inflated,
        };
        if device.area() <= 0.0 {
            continue;
        }
        let index = u32::try_from(index).expect("invariant: a list holds fewer than 2^32 parts");
        for row in span(&rows, device.y0, device.y1) {
            for col in span(&cols, device.x0, device.x1) {
                parts[row * across + col].push(index);
            }
        }
    }
    let work: Vec<u64> = list
        .parts
        .iter()
        .map(|part| {
            list.commands
                .get(part.commands.clone())
                .unwrap_or(&[])
                .iter()
                .map(work)
                .sum()
        })
        .collect();
    let cells = parts
        .into_iter()
        .enumerate()
        .map(|(i, parts)| {
            let (row, col) = (i / across, i % across);
            let rect = Rect::new(
                f64::from(cols[col]),
                f64::from(rows[row]),
                f64::from(cols[col + 1]),
                f64::from(rows[row + 1]),
            );
            let mut fnv = Fnv::default();
            fnv.rect(rect);
            for &p in &parts {
                fnv.u64(list.parts[p as usize].hash);
            }
            Cell {
                rect,
                hash: fnv.finish(),
                work: parts.iter().map(|&p| work[p as usize]).sum(),
                parts,
            }
        })
        .collect();
    Cells { cols, rows, cells }
}

/// The clip each part of `list` is drawn in, in list px: the intersection
/// of the clips of the groups open at its first command, through their
/// transforms; `None` outside every clip.
fn clips(list: &List) -> Vec<Option<Rect>> {
    let mut starts = list.parts.iter().map(|part| part.commands.start).peekable();
    let mut groups = vec![(Affine::IDENTITY, None::<Rect>)];
    let mut found = Vec::with_capacity(list.parts.len());
    for (i, command) in list.commands.iter().enumerate() {
        let (transform, clip) = *groups
            .last()
            .expect("invariant: the root group is never popped");
        while starts.next_if(|&start| start == i).is_some() {
            found.push(clip);
        }
        match command {
            Command::Push {
                transform: inner,
                clip: inner_clip,
            } => {
                let next = transform * *inner;
                let own = inner_clip.as_ref().map(|c| {
                    next.transform_rect_bbox(match c {
                        Clip::Rect(r) => r.abs(),
                        Clip::Path(p) => p.bounding_box(),
                    })
                });
                let clip = match (clip, own) {
                    (Some(a), Some(b)) => Some(a.intersect(b)),
                    (a, b) => a.or(b),
                };
                groups.push((next, clip));
            }
            Command::Pop => {
                if groups.len() > 1 {
                    groups.pop();
                }
            }
            Command::Fill { .. }
            | Command::Stroke { .. }
            | Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. } => {}
        }
    }
    found.resize(list.parts.len(), None);
    found
}

/// `rect` grown to whole device px: each edge within 1e-6 of a whole
/// pixel lands on it, the others move outward.
fn outward(rect: Rect) -> Rect {
    let edge = |v: f64, out: fn(f64) -> f64| {
        let near = v.round();
        if (v - near).abs() <= 1e-6 {
            near
        } else {
            out(v)
        }
    };
    Rect::new(
        edge(rect.x0, f64::floor),
        edge(rect.y0, f64::floor),
        edge(rect.x1, f64::ceil),
        edge(rect.y1, f64::ceil),
    )
}

/// `values` in device px at `scale`, rounded, sorted and deduplicated.
fn edges(values: &[f64], scale: f64) -> Vec<u32> {
    let mut edges: Vec<u32> = values
        .iter()
        .map(|v| v * scale)
        .filter(|v| v.is_finite())
        // Saturates past u32::MAX; negative edges are 0.
        .map(|v| v.round().max(0.0) as u32)
        .collect();
    edges.sort_unstable();
    edges.dedup();
    edges
}

/// The cells along one axis between `edges` that share interior with
/// `(lo, hi)`.
fn span(edges: &[u32], lo: f64, hi: f64) -> Range<usize> {
    let cells = edges.len().saturating_sub(1);
    let start = edges
        .partition_point(|&e| f64::from(e) <= lo)
        .saturating_sub(1);
    let end = edges.partition_point(|&e| f64::from(e) < hi).min(cells);
    start..end.max(start)
}

/// The cell along one axis between `edges` holding pixel `p`.
fn at(edges: &[u32], p: i32) -> Option<usize> {
    let after = edges.partition_point(|&e| i64::from(e) <= i64::from(p));
    (after > 0 && after < edges.len()).then(|| after - 1)
}

/// The raster work of one command.
fn work(command: &Command) -> u64 {
    let path = |path: &BezPath| 1 + path.elements().len() as u64 / ELEMENTS;
    match command {
        Command::Fill { path: p, .. } | Command::Stroke { path: p, .. } => path(p),
        Command::Image { .. } => 1,
        Command::Markers { at, .. } => at.len() as u64,
        Command::Glyphs(run) => run.glyphs.len() as u64,
        Command::Push { .. } | Command::Pop => 0,
    }
}

/// An FNV-1a 64 hash being written: byte-wise, the same on every platform
/// and Rust version, unlike `std`'s `DefaultHasher`. Numbers go in as their
/// little-endian bytes (floats by their bits); enumerated settings of
/// gradients and images as their names, through [`fmt::Write`].
pub(crate) struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Fnv {
        Fnv(BASIS)
    }
}

impl fmt::Write for Fnv {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.bytes(s.as_bytes());
        Ok(())
    }
}

impl Fnv {
    /// Hashes `bytes`.
    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(PRIME);
        }
    }

    /// Hashes `v`.
    pub(crate) fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }

    /// Hashes `words` a whole 64-bit word at a time (FNV-1a's step on a
    /// word instead of a byte), for the long arrays of marker commands:
    /// an eighth of the steps, and a sequence that differs from another in
    /// one word still hashes differently, as each step is a bijection.
    fn words(&mut self, words: impl IntoIterator<Item = u64>) {
        for w in words {
            self.0 = (self.0 ^ w).wrapping_mul(PRIME);
        }
    }

    /// Hashes `v`'s bits.
    pub(crate) fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }

    /// Hashes `v`'s bits.
    fn f32(&mut self, v: f32) {
        self.bytes(&v.to_bits().to_le_bytes());
    }

    /// Hashes a length or an index.
    fn len(&mut self, n: usize) {
        self.u64(n as u64);
    }

    /// Hashes a rectangle's edges.
    pub(crate) fn rect(&mut self, r: Rect) {
        for v in [r.x0, r.y0, r.x1, r.y1] {
            self.f64(v);
        }
    }

    /// The hash.
    pub(crate) fn finish(&self) -> u64 {
        self.0
    }

    fn point(&mut self, p: Point) {
        self.f64(p.x);
        self.f64(p.y);
    }

    fn affine(&mut self, a: Affine) {
        for v in a.as_coeffs() {
            self.f64(v);
        }
    }

    fn path(&mut self, path: &BezPath) {
        let elements = path.elements();
        self.len(elements.len());
        for e in elements {
            match e {
                PathEl::MoveTo(p) => {
                    self.bytes(b"M");
                    self.point(*p);
                }
                PathEl::LineTo(p) => {
                    self.bytes(b"L");
                    self.point(*p);
                }
                PathEl::QuadTo(a, b) => {
                    self.bytes(b"Q");
                    self.point(*a);
                    self.point(*b);
                }
                PathEl::CurveTo(a, b, c) => {
                    self.bytes(b"C");
                    self.point(*a);
                    self.point(*b);
                    self.point(*c);
                }
                PathEl::ClosePath => self.bytes(b"Z"),
            }
        }
    }

    fn brush(&mut self, brush: &Brush) {
        match brush {
            Brush::Solid(color) => {
                self.bytes(b"s");
                for c in color.components {
                    self.f32(c);
                }
            }
            Brush::Gradient(gradient) => {
                // Gradients are rare and nested several types deep: their
                // debug form, whose floats round-trip, holds all of it.
                self.bytes(b"g");
                debug(self, gradient);
            }
            Brush::Image(brush) => {
                self.bytes(b"i");
                self.image(&brush.image);
                debug(self, &brush.sampler);
            }
        }
    }

    fn image(&mut self, image: &ImageData) {
        self.u64(image.data.id());
        self.u64(u64::from(image.width));
        self.u64(u64::from(image.height));
        debug(self, &image.format);
        debug(self, &image.alpha_type);
    }

    fn stroke(&mut self, style: &kurbo::Stroke) {
        self.f64(style.width);
        debug(self, &style.join);
        self.f64(style.miter_limit);
        debug(self, &style.start_cap);
        debug(self, &style.end_cap);
        self.len(style.dash_pattern.len());
        for v in &style.dash_pattern {
            self.f64(*v);
        }
        self.f64(style.dash_offset);
    }

    fn clip(&mut self, clip: Option<&Clip>) {
        match clip {
            None => self.bytes(b"-"),
            Some(Clip::Rect(r)) => {
                self.bytes(b"r");
                self.rect(*r);
            }
            Some(Clip::Path(p)) => {
                self.bytes(b"p");
                self.path(p);
            }
        }
    }

    fn paints(&mut self, paints: &Paints) {
        self.len(paints.table.len());
        for brush in &paints.table {
            self.brush(brush);
        }
        self.len(paints.index.len());
        self.words(
            paints
                .index
                .chunks(2)
                .map(|pair| pair.iter().fold(0, |w, &i| w << 32 | u64::from(i))),
        );
    }

    /// A run's face (its font data's identity, not its place in the
    /// table), size, transform, paint and glyph ids and positions.
    fn run(&mut self, run: &GlyphRun, fonts: &[FontData]) {
        match fonts.get(run.font) {
            Some(font) => {
                self.u64(font.data.id());
                self.u64(u64::from(font.index));
            }
            None => self.len(run.font),
        }
        self.f32(run.size);
        self.affine(run.transform);
        self.brush(&run.fill);
        self.len(run.glyphs.len());
        for g in &run.glyphs {
            self.bytes(&g.id.to_le_bytes());
            self.f32(g.x);
            self.f32(g.y);
        }
    }

    fn command(&mut self, command: &Command, fonts: &[FontData]) {
        match command {
            Command::Fill { path, paint } => {
                self.bytes(b"fill");
                self.path(path);
                self.brush(paint);
            }
            Command::Stroke { path, style, paint } => {
                self.bytes(b"stroke");
                self.path(path);
                self.stroke(style);
                self.brush(paint);
            }
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } => {
                self.bytes(b"markers");
                self.path(shape);
                self.len(at.len());
                self.words(at.iter().flat_map(|p| [p.x.to_bits(), p.y.to_bits()]));
                self.brush(paint);
                match sizes {
                    Some(sizes) => {
                        self.len(sizes.len());
                        self.words(sizes.iter().map(|s| s.to_bits()));
                    }
                    None => self.bytes(b"-"),
                }
                match paints {
                    Some(paints) => self.paints(paints),
                    None => self.bytes(b"-"),
                }
            }
            Command::Glyphs(run) => {
                self.bytes(b"glyphs");
                self.run(run, fonts);
            }
            Command::Image {
                rect,
                image,
                smooth,
            } => {
                self.bytes(b"image");
                self.rect(*rect);
                self.image(image);
                self.bytes(&[u8::from(*smooth)]);
            }
            Command::Push { transform, clip } => {
                self.bytes(b"push");
                self.affine(*transform);
                self.clip(clip.as_ref());
            }
            Command::Pop => self.bytes(b"pop"),
        }
    }
}

/// Fills every part's hash: FNV-1a 64 over the groups open at the part's
/// start (each one's transform and clip, so the same content drawn at
/// another place or under another clip hashes differently) and over every
/// command in its range, as drawn (paths, paints, strokes, marker points
/// and channels, glyph runs' faces, sizes, transforms, ids and positions,
/// images by their blob ids); a markers command's points, sizes and paint
/// indices a 64-bit word per step. A part drawn again from equal data hashes
/// equally. Parts are in order and inside the commands, as a figure
/// builds them.
pub(crate) fn hash(list: &mut List) {
    let mut open: Vec<&Command> = Vec::new();
    let mut followed = 0;
    let hashes: Vec<u64> = list
        .parts
        .iter()
        .map(|part| {
            let range = part.commands.clone();
            for command in &list.commands[followed..range.start] {
                follow(&mut open, command);
            }
            let mut fnv = Fnv::default();
            fnv.len(open.len());
            for group in &open {
                fnv.command(group, &list.fonts);
            }
            for command in &list.commands[range.clone()] {
                fnv.command(command, &list.fonts);
                follow(&mut open, command);
            }
            followed = range.end;
            fnv.finish()
        })
        .collect();
    for (part, hash) in list.parts.iter_mut().zip(hashes) {
        part.hash = hash;
    }
}

/// Opens or closes a group on `open` for `command`.
fn follow<'l>(open: &mut Vec<&'l Command>, command: &'l Command) {
    match command {
        Command::Push { .. } => open.push(command),
        Command::Pop => {
            open.pop();
        }
        Command::Fill { .. }
        | Command::Stroke { .. }
        | Command::Markers { .. }
        | Command::Glyphs(_)
        | Command::Image { .. } => {}
    }
}

/// Hashes `value`'s debug form.
fn debug(fnv: &mut Fnv, value: &impl fmt::Debug) {
    // Writing into the hasher cannot fail.
    let _ = fmt::Write::write_fmt(fnv, format_args!("{value:?}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_take_the_cells_whose_interior_a_range_shares() {
        let edges = [0, 100, 200];
        assert_eq!(span(&edges, -1.0, 50.0), 0..1);
        assert_eq!(span(&edges, 99.0, 101.0), 0..2);
        assert_eq!(span(&edges, 100.0, 150.0), 1..2);
        assert_eq!(span(&edges, 250.0, 300.0), 2..2);
        assert_eq!(span(&edges, -50.0, -10.0), 0..0);
        assert_eq!(span(&[], 0.0, 10.0), 0..0);
    }

    #[test]
    fn edges_snap_sort_and_drop_duplicates_and_non_finite_values() {
        let xs = [200.0, 0.0, 99.7, 100.2, f64::NAN, -3.0];
        assert_eq!(edges(&xs, 1.0), [0, 100, 200]);
        assert_eq!(edges(&[10.3, 10.5], 1.5), [15, 16]);
    }

    #[test]
    fn markers_hash_every_point_size_and_paint_index_a_word_at_a_time() {
        let markers = |x: f64, size: f64, index: u32| {
            let red = Brush::Solid(peniko::Color::from_rgba8(255, 0, 0, 255));
            let mut list = List {
                commands: vec![Command::Markers {
                    shape: Rect::new(-1.0, -1.0, 1.0, 1.0).to_path(0.1),
                    at: vec![
                        Point::new(1.0, 2.0),
                        Point::new(x, 4.0),
                        Point::new(5.0, 6.0),
                    ],
                    paint: red.clone(),
                    sizes: Some(vec![1.0, size, 1.0]),
                    paints: Some(Paints {
                        table: vec![red.clone(), red],
                        index: vec![0, 1, index],
                    }),
                }],
                parts: vec![crate::list::Part {
                    id: crate::list::part::Id::new(0),
                    commands: 0..1,
                    bounds: Rect::ZERO,
                    hash: 0,
                }],
                ..List::default()
            };
            hash(&mut list);
            list.parts[0].hash
        };
        let base = markers(3.0, 2.0, 1);
        assert_eq!(base, markers(3.0, 2.0, 1));
        // One ulp in a point, a size, the last (odd) paint index.
        assert_ne!(base, markers(f64::from_bits(3.0_f64.to_bits() + 1), 2.0, 1));
        assert_ne!(base, markers(3.0, 2.5, 1));
        assert_ne!(base, markers(3.0, 2.0, 0));
    }

    #[test]
    fn fnv_1a_64_matches_its_reference_values() {
        let of = |bytes: &[u8]| {
            let mut fnv = Fnv::default();
            fnv.bytes(bytes);
            fnv.finish()
        };
        assert_eq!(of(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(of(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(of(b"foobar"), 0x8594_4171_f739_67e8);
    }
}
