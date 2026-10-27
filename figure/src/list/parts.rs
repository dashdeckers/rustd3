//! Part hashes: FNV-1a 64 over what each part of a list drew, so that a
//! figure drawn again from equal data hashes equally and a part drawn
//! elsewhere or differently does not.

use std::fmt;

use kurbo::{Affine, BezPath, PathEl, Point, Rect};
use peniko::{Brush, FontData, ImageData};

use crate::list::{Clip, Command, GlyphRun, List, Paints};

/// FNV-1a 64's offset basis.
const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64's prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

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
        for i in &paints.index {
            self.bytes(&i.to_le_bytes());
        }
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
                for p in at {
                    self.point(*p);
                }
                self.brush(paint);
                match sizes {
                    Some(sizes) => {
                        self.len(sizes.len());
                        for s in sizes {
                            self.f64(*s);
                        }
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
/// images by their blob ids). A part drawn again from equal data hashes
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
