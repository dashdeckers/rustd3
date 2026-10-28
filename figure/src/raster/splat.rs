//! Marker splatting (D31.2): a markers command whose points share one
//! solid paint table and one size, drawn through groups that only scale
//! and translate and clip to rectangles, is composited from coverage
//! stamps of its shape instead of filled once per point. The shape is
//! rasterized by vello_cpu itself at 8 × 8 eighth-pixel phases under the
//! device transform's scale; each point lays the stamp of the phase
//! nearest its centre source-over onto the premultiplied pixmap. A centre
//! moves by at most 1/16 px along each axis, which bounds the difference
//! from filling each point (within 32/255 on the benchmark's scatters);
//! file outputs never splat.

use std::collections::HashMap;

use kurbo::{Affine, BezPath, PathEl, Point, Shape};
use peniko::Brush;
use vello_cpu::{Level, PaintType, Pixmap, RenderContext, RenderSettings};

use crate::list::Paints;

/// Sub-pixel phases per axis.
pub(super) const PHASES: usize = 8;

/// Stamp sets kept before the cache starts over.
const KEPT: usize = 64;

/// The stamps of each shape and scale a renderer splatted, kept between
/// renders.
#[derive(Debug, Default)]
pub(super) struct Stamps {
    sets: HashMap<u64, Vec<Stamp>>,
}

/// A shape's coverage at one phase: 8-bit, row-major; its top-left pixel
/// lies `offset` px from the pixel holding the centre.
#[derive(Debug)]
pub(super) struct Stamp {
    width: usize,
    height: usize,
    offset: [i64; 2],
    coverage: Vec<u8>,
}

/// What one splatted command needs: its shape, centres and paints, the
/// device transform of its group and the device clip, `[x0, y0, x1, y1]`
/// in whole px.
pub(super) struct Splat<'a> {
    pub(super) shape: &'a BezPath,
    pub(super) at: &'a [Point],
    pub(super) paint: &'a Brush,
    pub(super) paints: Option<&'a Paints>,
    pub(super) device: Affine,
    pub(super) clip: [i64; 4],
}

impl Stamps {
    /// Lays `splat`'s points onto `pixmap` (premultiplied), its stamps
    /// made with vello_cpu at `level` on first use. Points whose stamp
    /// misses the clip are skipped.
    pub(super) fn splat(&mut self, pixmap: &mut Pixmap, splat: &Splat<'_>, level: Level) {
        let [a, _, _, d, _, _] = splat.device.as_coeffs();
        let key = key(splat.shape, [a, d]);
        if !self.sets.contains_key(&key) && self.sets.len() >= KEPT {
            self.sets.clear();
        }
        let set = self
            .sets
            .entry(key)
            .or_insert_with(|| stamps(splat.shape, Affine::scale_non_uniform(a, d), level));
        let colors: Vec<[u32; 4]> = match splat.paints {
            Some(paints) => paints.table.iter().map(premultiplied).collect(),
            None => vec![premultiplied(splat.paint)],
        };
        let size = [i64::from(pixmap.width()), i64::from(pixmap.height())];
        let clip = [
            splat.clip[0].max(0),
            splat.clip[1].max(0),
            splat.clip[2].min(size[0]),
            splat.clip[3].min(size[1]),
        ];
        if clip[0] >= clip[2] || clip[1] >= clip[3] {
            return;
        }
        let width = pixmap.width() as usize;
        let data = pixmap.data_as_u8_slice_mut();
        for (n, p) in splat.at.iter().enumerate() {
            let index = splat.paints.map_or(0, |p| p.index[n] as usize);
            let Some(&color) = colors.get(index) else {
                continue;
            };
            lay(data, width, set, (splat.device * *p, color), clip);
        }
        pixmap.set_may_have_transparency(true);
    }
}

/// The 64 stamps of `shape` under `linear`, phase (x, y) at index
/// `8 y + x`: opaque black filled by vello_cpu, coverage read from alpha.
fn stamps(shape: &BezPath, linear: Affine, level: Level) -> Vec<Stamp> {
    let placed = linear * shape.clone();
    let b = placed.bounding_box();
    let offset = [b.x0.floor() as i64 - 1, b.y0.floor() as i64 - 1];
    let width = (b.x1.ceil() as i64 - offset[0] + 2).clamp(1, i64::from(u16::MAX));
    let height = (b.y1.ceil() as i64 - offset[1] + 2).clamp(1, i64::from(u16::MAX));
    let (w, h) = (
        u16::try_from(width).expect("invariant: clamped to u16"),
        u16::try_from(height).expect("invariant: clamped to u16"),
    );
    let mut context = RenderContext::new_with(
        w,
        h,
        RenderSettings {
            level,
            num_threads: 0,
        },
    );
    let mut resources = vello_cpu::Resources::new();
    let q = PHASES as f64;
    let mut set = Vec::with_capacity(PHASES * PHASES);
    for py in 0..PHASES {
        for px in 0..PHASES {
            let at = Affine::translate((
                px as f64 / q - offset[0] as f64,
                py as f64 / q - offset[1] as f64,
            ));
            context.reset();
            context.set_paint(PaintType::Solid(peniko::Color::from_rgba8(0, 0, 0, 255)));
            context.set_transform(at);
            context.fill_path(&placed);
            context.flush();
            let mut pixmap = Pixmap::new(w, h);
            context.render(&mut pixmap, &mut resources);
            set.push(Stamp {
                width: usize::from(w),
                height: usize::from(h),
                offset,
                coverage: pixmap.data().iter().map(|p| p.a).collect(),
            });
        }
    }
    set
}

/// Lays the stamp of the phase nearest `at` (device px) in `color`
/// (premultiplied, 0–255 per channel) source-over onto the premultiplied
/// RGBA `data`, `width` px a row, inside `clip`.
fn lay(
    data: &mut [u8],
    width: usize,
    set: &[Stamp],
    (at, color): (Point, [u32; 4]),
    clip: [i64; 4],
) {
    if !at.is_finite() {
        return;
    }
    let q = PHASES as f64;
    let (mut ix, mut iy) = (at.x.floor() as i64, at.y.floor() as i64);
    let mut fx = ((at.x - ix as f64) * q).round() as usize;
    let mut fy = ((at.y - iy as f64) * q).round() as usize;
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
    let xs = x0.max(clip[0])..(x0 + stamp.width as i64).min(clip[2]);
    let ys = y0.max(clip[1])..(y0 + stamp.height as i64).min(clip[3]);
    for y in ys {
        let row = (y - y0) as usize * stamp.width;
        for x in xs.clone() {
            let coverage = u32::from(stamp.coverage[row + (x - x0) as usize]);
            if coverage == 0 {
                continue;
            }
            let i = (y as usize * width + x as usize) * 4;
            let pixel = &mut data[i..i + 4];
            let alpha = div255(color[3] * coverage);
            for (channel, source) in pixel.iter_mut().zip(color) {
                let source = div255(source * coverage);
                let under = div255(u32::from(*channel) * (255 - alpha));
                *channel = (source + under).min(255) as u8;
            }
        }
    }
}

/// `v / 255` rounded, for `v` up to 255².
fn div255(v: u32) -> u32 {
    (v + 128 + ((v + 128) >> 8)) >> 8
}

/// A solid brush premultiplied, 0–255 per channel; other brushes (never
/// splatted) as transparent.
fn premultiplied(brush: &Brush) -> [u32; 4] {
    match brush {
        Brush::Solid(color) => {
            let c = color.to_rgba8();
            let a = u32::from(c.a);
            [
                div255(u32::from(c.r) * a),
                div255(u32::from(c.g) * a),
                div255(u32::from(c.b) * a),
                a,
            ]
        }
        Brush::Gradient(_) | Brush::Image(_) => [0; 4],
    }
}

/// A stamp set's key: FNV-1a 64 of the shape's points and the scale.
fn key(shape: &BezPath, scale: [f64; 2]) -> u64 {
    let points = shape.elements().iter().flat_map(|e| match *e {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => vec![p],
        PathEl::QuadTo(p, q) => vec![p, q],
        PathEl::CurveTo(p, q, r) => vec![p, q, r],
        PathEl::ClosePath => Vec::new(),
    });
    points
        .flat_map(|p| [p.x, p.y])
        .chain(scale)
        .flat_map(f64::to_le_bytes)
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::div255;

    #[test]
    fn div255_rounds_to_nearest() {
        for v in [0, 1, 127, 128, 254, 255, 256, 32_767, 65_025] {
            assert_eq!(div255(v), (v + 127) / 255, "{v}");
        }
        assert_eq!(div255(255 * 255), 255);
    }
}
