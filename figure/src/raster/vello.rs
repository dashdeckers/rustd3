//! The replay of a list into vello_cpu: kurbo paths and peniko paints pass
//! through unconverted; images become premultiplied pixmaps, cached by
//! their blob between renders. With splatting on, the commands between
//! splatted markers render into the pixmap in turn, source-over, and the
//! markers are composited there from stamps ([`splat`](super::splat)).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kurbo::{Affine, BezPath, Point, Rect, Shape};
use peniko::{Brush, ImageBrush, ImageData, ImageFormat, ImageQuality, ImageSampler};
use vello_cpu::{
    ImageSource, Level, PaintType, Pixmap, RasterizerSettings, RenderContext, Resources, TargetInit,
};

use crate::list::{self, Bounds, Clip, Command, GlyphRun, List, Paints};
use crate::raster::splat::{Splat, Stamps};
use crate::raster::{Error, MAX_IMAGE};

/// An image's identity: its blob, and the layout read from it.
type Key = (u64, u32, u32, u8);

/// The pixmaps of the images the last render drew, keyed by blob.
#[derive(Debug, Default)]
pub(super) struct Images {
    cache: HashMap<Key, Arc<Pixmap>>,
    used: HashSet<Key>,
}

/// One render of a list into a render context.
pub(super) struct Replay<'a> {
    pub(super) context: &'a mut RenderContext,
    pub(super) resources: &'a mut Resources,
    pub(super) images: &'a mut Images,
    pub(super) list: &'a List,
    pub(super) bounds: &'a Bounds,
    /// The area rendered, list px.
    pub(super) area: Rect,
    /// Device px per list px.
    pub(super) scale: f64,
    /// The stamps markers are splatted from, when splatting is on, and
    /// the SIMD level they are made at.
    pub(super) splat: Option<(&'a mut Stamps, Level)>,
}

/// An open group: its transform into list px, the clip layer it pushed
/// (the path and its device transform), its clip in device px (when every
/// clip so far is a rectangle in an axis-aligned space), and whether its
/// space and every enclosing one only scale and translate.
#[derive(Clone)]
struct Group {
    to_list: Affine,
    layer: Option<(Affine, BezPath)>,
    rect: Option<[f64; 4]>,
    aligned: bool,
}

impl Replay<'_> {
    /// Draws every command that may touch the area into `pixmap` (new,
    /// transparent); images not drawn are dropped from the cache.
    pub(super) fn run(mut self, pixmap: &mut Pixmap) -> Result<(), Error> {
        let device = Affine::scale(self.scale) * Affine::translate((-self.area.x0, -self.area.y0));
        let margin = 1.0 / self.scale;
        let visible = self.area.inflate(margin, margin);
        let mut groups = vec![Group {
            to_list: Affine::IDENTITY,
            layer: None,
            rect: None,
            aligned: true,
        }];
        // Whether a run of commands was rendered into the pixmap already.
        let mut started = false;
        self.images.used.clear();
        for (i, command) in self.list.commands.iter().enumerate() {
            let group = groups
                .last()
                .expect("invariant: the root group is never popped");
            let to_list = group.to_list;
            let to_device = device * to_list;
            let shown = self
                .bounds
                .get(i)
                .is_some_and(|b| b.intersect(visible).area() > 0.0);
            match command {
                Command::Markers {
                    shape,
                    at,
                    paint,
                    sizes: None,
                    paints,
                } if shown
                    && self.splat.is_some()
                    && group.aligned
                    && axis_aligned(to_device)
                    && solid(paint, paints.as_ref()) =>
                {
                    let size = [pixmap.width(), pixmap.height()].map(f64::from);
                    let [x0, y0, x1, y1] = group.rect.unwrap_or([0.0, 0.0, size[0], size[1]]);
                    let splat = Splat {
                        shape,
                        at,
                        paint,
                        paints: paints.as_ref(),
                        device: to_device,
                        clip: [x0, y0, x1, y1].map(|v| v.round() as i64),
                    };
                    self.render(pixmap, &groups, started);
                    started = true;
                    if let Some((stamps, level)) = &mut self.splat {
                        stamps.splat(pixmap, &splat, *level);
                    }
                }
                Command::Fill { path, paint } if shown => {
                    self.paint(i, paint, Affine::IDENTITY)?;
                    self.context.set_transform(to_device);
                    self.context.fill_path(path);
                }
                Command::Stroke { path, style, paint } if shown => {
                    self.paint(i, paint, Affine::IDENTITY)?;
                    self.context.set_transform(to_device);
                    self.context.set_stroke(style.clone());
                    self.context.stroke_path(path);
                }
                Command::Markers {
                    shape,
                    at,
                    paint,
                    sizes,
                    paints,
                } if shown => {
                    let marks = Marks {
                        shape,
                        at,
                        paint,
                        sizes: sizes.as_deref(),
                        paints: paints.as_ref(),
                    };
                    self.markers(i, &marks, to_list, to_device, visible)?;
                }
                Command::Glyphs(run) if shown => self.glyphs(i, run, to_device)?,
                Command::Image {
                    rect,
                    image,
                    smooth,
                } if shown => self.image(i, *rect, image, *smooth, to_device)?,
                Command::Push { transform, clip } => {
                    let next = to_list * *transform;
                    let aligned = group.aligned && axis_aligned(device * next);
                    let mut rect = group.rect;
                    let layer = clip.as_ref().map(|clip| {
                        let path = match clip {
                            Clip::Rect(r) => {
                                let d = (device * next).transform_rect_bbox(*r);
                                let own = [d.x0, d.y0, d.x1, d.y1];
                                rect = Some(rect.map_or(own, |outer| intersect(outer, own)));
                                r.to_path(0.1)
                            }
                            Clip::Path(p) => p.clone(),
                        };
                        self.context.set_transform(device * next);
                        self.context.push_clip_layer(&path);
                        (device * next, path)
                    });
                    let aligned = aligned && !matches!(clip, Some(Clip::Path(_)));
                    groups.push(Group {
                        to_list: next,
                        layer,
                        rect,
                        aligned,
                    });
                }
                Command::Pop => {
                    if groups.pop().is_some_and(|group| group.layer.is_some()) {
                        self.context.pop_layer();
                    }
                }
                Command::Fill { .. }
                | Command::Stroke { .. }
                | Command::Markers { .. }
                | Command::Glyphs(_)
                | Command::Image { .. } => {}
            }
        }
        let used = &self.images.used;
        self.images.cache.retain(|key, _| used.contains(key));
        self.render(pixmap, &[], started);
        Ok(())
    }

    /// Renders what the context holds into `pixmap`, over what it shows
    /// when `over` (earlier runs), and starts the context afresh inside
    /// the clip layers of the open `groups`: a group whose commands are
    /// split around splatted markers composites each part source-over,
    /// which is how its layer composites them whole.
    fn render(&mut self, pixmap: &mut Pixmap, groups: &[Group], over: bool) {
        let open: Vec<&(Affine, BezPath)> =
            groups.iter().filter_map(|g| g.layer.as_ref()).collect();
        for _ in &open {
            self.context.pop_layer();
        }
        self.context.flush();
        let settings = RasterizerSettings {
            target_init: if over {
                TargetInit::SrcOver
            } else {
                RasterizerSettings::default().target_init
            },
            ..RasterizerSettings::default()
        };
        self.context
            .render_with(&mut *pixmap, self.resources, settings);
        self.context.reset();
        for (transform, path) in open {
            self.context.set_transform(*transform);
            self.context.push_clip_layer(path);
        }
    }

    /// Sets `brush` as the paint, positioned in the group's space by
    /// `inverse` (the inverse of the command's own placement).
    fn paint(&mut self, command: usize, brush: &Brush, inverse: Affine) -> Result<(), Error> {
        let paint = match brush {
            Brush::Solid(color) => PaintType::Solid(*color),
            Brush::Gradient(gradient) => PaintType::Gradient(gradient.clone()),
            Brush::Image(image) => PaintType::Image(ImageBrush {
                image: ImageSource::Pixmap(self.pixmap(command, &image.image)?),
                sampler: image.sampler,
            }),
        };
        self.context.set_paint(paint);
        self.context.set_paint_transform(match brush {
            Brush::Solid(_) => Affine::IDENTITY,
            Brush::Gradient(_) | Brush::Image(_) => inverse,
        });
        Ok(())
    }

    /// Each point that may touch the area, its shape placed by its size.
    fn markers(
        &mut self,
        command: usize,
        marks: &Marks<'_>,
        to_list: Affine,
        to_device: Affine,
        visible: Rect,
    ) -> Result<(), Error> {
        let unit = marks.shape.bounding_box();
        let mut current = None;
        for (k, p) in marks.at.iter().enumerate() {
            let place =
                Affine::translate(p.to_vec2()) * Affine::scale(marks.sizes.map_or(1.0, |s| s[k]));
            let bounds = to_list.transform_rect_bbox(place.transform_rect_bbox(unit));
            if bounds.intersect(visible).area() <= 0.0 {
                continue;
            }
            let index = marks.paints.map_or(0, |p| p.index[k]);
            let brush = marks
                .paints
                .map_or(marks.paint, |p| &p.table[index as usize]);
            if current != Some(index) || !matches!(brush, Brush::Solid(_)) {
                self.paint(command, brush, place.inverse())?;
                current = Some(index);
            }
            self.context.set_transform(to_device * place);
            self.context.fill_path(marks.shape);
        }
        Ok(())
    }

    /// A run through vello_cpu's glyph renderer, unhinted.
    fn glyphs(&mut self, command: usize, run: &GlyphRun, to_device: Affine) -> Result<(), Error> {
        self.paint(command, &run.fill, run.transform.inverse())?;
        self.context.set_transform(to_device * run.transform);
        let glyphs = run.glyphs.iter().map(|g| vello_cpu::Glyph {
            id: g.id,
            x: g.x,
            y: g.y,
        });
        // A glyph that cannot be drawn is left blank, as in every backend.
        let _ = self
            .context
            .glyph_run(self.resources, &self.list.fonts[run.font])
            .font_size(run.size)
            .hint(false)
            .fill_glyphs(glyphs);
        Ok(())
    }

    /// The image stretched over `rect` by the paint transform, nearest or
    /// bilinear.
    fn image(
        &mut self,
        command: usize,
        rect: Rect,
        image: &ImageData,
        smooth: bool,
        to_device: Affine,
    ) -> Result<(), Error> {
        if image.width == 0 || image.height == 0 {
            return Ok(());
        }
        let rect = rect.abs();
        let pixmap = self.pixmap(command, image)?;
        let quality = if smooth {
            ImageQuality::Medium
        } else {
            ImageQuality::Low
        };
        self.context.set_paint(PaintType::Image(ImageBrush {
            image: ImageSource::Pixmap(pixmap),
            sampler: ImageSampler::new().with_quality(quality),
        }));
        self.context.set_paint_transform(
            Affine::translate(rect.origin().to_vec2())
                * Affine::scale_non_uniform(
                    rect.width() / f64::from(image.width),
                    rect.height() / f64::from(image.height),
                ),
        );
        self.context.set_transform(to_device);
        self.context.fill_rect(&rect);
        Ok(())
    }

    /// The premultiplied pixmap of `image`, from the cache when its blob
    /// was drawn last time.
    fn pixmap(&mut self, command: usize, image: &ImageData) -> Result<Arc<Pixmap>, Error> {
        let format = match image.format {
            ImageFormat::Rgba8 => 0,
            ImageFormat::Bgra8 => 1,
            // `ImageFormat` is #[non_exhaustive] in peniko.
            _ => {
                return Err(list::Error::Unsupported {
                    command,
                    what: "an image format other than RGBA8 and BGRA8",
                }
                .into());
            }
        };
        let (width, height) = (f64::from(image.width), f64::from(image.height));
        if width > MAX_IMAGE || height > MAX_IMAGE {
            return Err(Error::TooLarge { width, height });
        }
        let key = (image.data.id(), image.width, image.height, format);
        self.images.used.insert(key);
        let pixmap = self.images.cache.entry(key).or_insert_with(|| {
            let ImageSource::Pixmap(pixmap) = ImageSource::from_peniko_image_data(image) else {
                unreachable!("invariant: from_peniko_image_data returns pixels");
            };
            pixmap
        });
        Ok(Arc::clone(pixmap))
    }
}

/// Whether `t` only scales and translates.
fn axis_aligned(t: Affine) -> bool {
    let [_, b, c, _, _, _] = t.as_coeffs();
    b == 0.0 && c == 0.0
}

/// Whether every paint of a markers command is solid.
fn solid(paint: &Brush, paints: Option<&Paints>) -> bool {
    match paints {
        Some(p) => p.table.iter().all(|b| matches!(b, Brush::Solid(_))),
        None => matches!(paint, Brush::Solid(_)),
    }
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

/// A markers command's fields.
struct Marks<'a> {
    shape: &'a BezPath,
    at: &'a [Point],
    paint: &'a Brush,
    sizes: Option<&'a [f64]>,
    paints: Option<&'a Paints>,
}
