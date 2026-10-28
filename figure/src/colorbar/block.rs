//! A colorbar over one resolve ([`Content`]) and laid out as a [`Block`]:
//! its bar along a side of the grid or a panel with its end triangles,
//! ticks, labels and title, and drawing it.

use kurbo::{Point, Rect, Size, Vec2};
use rustd3_core::axis::Orient;

use crate::axis::{self, Labels, Placed};
use crate::colorbar::{Draw, Side, Strip, extend, side};
use crate::figure::Resolved;
use crate::layout::{Plan, plan};
use crate::legend::keys;
use crate::list::{self, Command, List};
use crate::mark::Snap;
use crate::text::{self, Key};
use crate::theme::Theme;

/// A colorbar over one resolve: its strip, and, once layout measured it
/// for a pass, the side its ticks are on, their count, their labels and
/// each tick's share of the bar from its low end.
pub(crate) struct Content<'g, 'a> {
    bar: &'g (dyn Draw<'a> + 'a),
    strip: Strip,
    side: Option<Measured>,
}

/// A colorbar's long axis as one layout pass measured it.
struct Measured {
    orient: Orient,
    count: usize,
    labels: Labels,
    shares: Vec<Option<f64>>,
}

/// A colorbar laid out, relative to its top-left corner: its size, whether
/// it is framed, its bar's interior, its end triangles' lengths ([low,
/// high]), whether the bar is vertical, the strip drawn over it, and its
/// ticks, labels and title placed as an axis beside the interior (`None`
/// before layout measured them).
pub(crate) struct Block {
    size: Size,
    framed: bool,
    bar: Rect,
    ends: [f64; 2],
    vertical: bool,
    strip: Strip,
    placed: Option<Placed>,
}

impl<'g, 'a> Content<'g, 'a> {
    /// `bar` over the resolved scales, not yet measured.
    pub(crate) fn new(bar: &'g (dyn Draw<'a> + 'a), resolved: &Resolved) -> Content<'g, 'a> {
        Content {
            bar,
            strip: bar.strip(resolved),
            side: None,
        }
    }

    /// Measures the colorbar for a pass with its ticks on side `orient` of
    /// a bar `length` px long with its triangles: about `.ticks(n)` ticks,
    /// or as many as an axis as long as its interior takes (one per 80 px
    /// across, per 35 px up), and their labels, through `text`'s line box;
    /// the count.
    pub(crate) fn measure(
        &mut self,
        orient: Orient,
        length: f64,
        resolved: &Resolved,
        theme: &Theme,
        text: &text::Cache,
    ) -> usize {
        let (interior, _) = self.bar.extend().split(length);
        let count = self
            .bar
            .count()
            .unwrap_or_else(|| plan::count(orient, interior));
        let side = Side {
            bar: self.bar,
            orient,
            binned: self.strip.binned(),
        };
        let labels = Labels::new(&side, resolved, theme, text, true, count);
        self.side = Some(Measured {
            orient,
            count,
            labels,
            shares: self.bar.shares(resolved, count),
        });
        count
    }

    /// The side the last measure put the ticks on.
    pub(crate) fn orient(&self) -> Option<Orient> {
        self.side.as_ref().map(|m| m.orient)
    }

    /// The keys of the measured tick labels and title, for the text
    /// cache's batch.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &Key> {
        self.side.iter().flat_map(|m| m.labels.keys())
    }

    /// The colorbar with its ticks on side `orient` (a vertical bar for
    /// left and right, a horizontal one for top and bottom), its bar with
    /// its triangles `length` px long and one em of `theme` thick, its
    /// ticks, labels and title as measured when measured on that side: the
    /// block holds the bar, its triangles, the labels' boxes and the title
    /// centred along the interior.
    pub(crate) fn block(
        &self,
        orient: Orient,
        length: f64,
        text: &text::Cache,
        theme: &Theme,
    ) -> Block {
        let em = theme.em();
        let vertical = !axis::across(orient);
        let (interior, [low, high]) = self.bar.extend().split(length);
        let bar = if vertical {
            Rect::new(0.0, 0.0, em, interior)
        } else {
            Rect::new(0.0, 0.0, interior, em)
        };
        let mut bounds = if vertical {
            Rect::new(0.0, -high, em, interior + low)
        } else {
            Rect::new(-low, 0.0, interior + high, em)
        };
        let measured = self.side.as_ref().filter(|m| m.orient == orient);
        let placed = measured.map(|m| {
            let positions = m
                .shares
                .iter()
                .map(|s| s.map(|s| side::along(orient, interior, s)))
                .collect();
            let plan = Plan {
                shown: true,
                labeled: true,
                offset: 0.0,
                thickness: m.labels.thickness(text, theme),
                count: m.count,
            };
            Placed::new(m.labels.clone(), plan, positions, bar, text, theme)
        });
        if let Some(placed) = &placed {
            let reach = placed.plan.thickness;
            match orient {
                Orient::Right => bounds.x1 = bounds.x1.max(em + reach),
                Orient::Left => bounds.x0 = bounds.x0.min(-reach),
                Orient::Bottom => bounds.y1 = bounds.y1.max(em + reach),
                Orient::Top => bounds.y0 = bounds.y0.min(-reach),
            }
            for b in placed.boxes.iter().flatten() {
                bounds = bounds.union(b.aabb());
            }
            let title = placed.labels.title().and_then(|key| text.get(key)?.ok());
            if let Some(title) = title {
                let half = title.width() / 2.0;
                let middle = interior / 2.0;
                if vertical {
                    bounds.y0 = bounds.y0.min(middle - half);
                    bounds.y1 = bounds.y1.max(middle + half);
                } else {
                    bounds.x0 = bounds.x0.min(middle - half);
                    bounds.x1 = bounds.x1.max(middle + half);
                }
            }
        }
        let by = Vec2::new(-bounds.x0, -bounds.y0);
        Block {
            size: bounds.size(),
            framed: false,
            bar: bar + by,
            ends: [low, high],
            vertical,
            strip: self.strip.clone(),
            placed,
        }
    }
}

impl Block {
    /// Its size, px.
    pub(crate) fn size(&self) -> Size {
        self.size
    }

    /// Its bar with its triangles, relative to its top-left corner.
    pub(crate) fn bar(&self) -> Rect {
        let [low, high] = self.ends;
        let b = self.bar;
        if self.vertical {
            Rect::new(b.x0, b.y0 - high, b.x1, b.y1 + low)
        } else {
            Rect::new(b.x0 - low, b.y0, b.x1 + high, b.y1)
        }
    }

    /// The block in a frame 0.4 em larger on every side, as an inset draws
    /// it.
    pub(crate) fn framed(self, em: f64) -> Block {
        let pad = keys::PAD * em;
        Block {
            size: Size::new(self.size.width + 2.0 * pad, self.size.height + 2.0 * pad),
            framed: true,
            bar: self.bar + Vec2::new(pad, pad),
            ..self
        }
    }

    /// Appends the block with its top-left corner at `at`: the frame (as a
    /// legend's), the end triangles in the scale's clamped end colors, the
    /// strip's image ([`Strip::image`]) stretched over the interior with
    /// nearest sampling, the outline of the bar and its triangles as the
    /// theme's axis line (none in the styles without one), then the tick
    /// marks in every style (matplotlib's colorbar turns its long axis's
    /// ticks on whatever the style), labels and title as an axis's. On
    /// raster targets (`snap`) the interior's edges lie on device pixels
    /// and the outline and ticks on whole device pixels.
    pub(crate) fn draw(
        &self,
        at: Point,
        text: &text::Cache,
        theme: &Theme,
        snap: Snap,
        list: &mut List,
    ) {
        let by = at.to_vec2();
        if self.framed {
            keys::frame(self.size.to_rect() + by, theme, list);
        }
        let interior = snap.rect(self.bar + by);
        let ends = [false, true]
            .into_iter()
            .zip(self.ends)
            .zip(self.strip.ends());
        for ((high, length), color) in ends.filter(|((_, length), _)| *length > 0.0) {
            list.commands.push(Command::Fill {
                path: extend::triangle(interior, self.vertical, high, length),
                paint: list::brush(color),
            });
        }
        let length = if self.vertical {
            interior.height()
        } else {
            interior.width()
        };
        if let Some(image) = self.strip.image(length, snap.scale(), self.vertical) {
            list.commands.push(Command::Image {
                rect: interior,
                image,
                smooth: false,
            });
        }
        if let Some(edge) = theme.edge() {
            let width = theme.axis_width();
            let corners = [interior.origin(), Point::new(interior.x1, interior.y1)];
            let [a, b] = corners.map(|p| snap.point(p, width));
            list.commands.push(Command::Stroke {
                path: extend::outline(Rect::from_points(a, b), self.vertical, self.ends),
                style: list::stroke(snap.width(width)),
                paint: list::brush(edge),
            });
        }
        if let Some(placed) = &self.placed {
            placed.marks(interior, theme, snap, list);
            placed.texts(interior, text, theme, list);
        }
    }
}
