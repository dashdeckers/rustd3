//! A colorbar over one resolve ([`Content`]) and laid out as a [`Block`]:
//! its bar along a side of the grid or a panel, and drawing it.

use kurbo::{Point, Rect, Size, Vec2};
use rustd3_core::axis::Orient;

use crate::axis;
use crate::colorbar::{Draw, Strip};
use crate::figure::Resolved;
use crate::layout::Snap;
use crate::legend::keys;
use crate::list::{Command, List};
use crate::theme::Theme;

/// A colorbar over one resolve, before layout gives it a side and a
/// length: its strip.
pub(crate) struct Content {
    strip: Strip,
}

/// A colorbar laid out, relative to its top-left corner: its size, whether
/// it is framed, its bar, whether the bar is vertical, and the strip drawn
/// over it.
pub(crate) struct Block {
    size: Size,
    framed: bool,
    bar: Rect,
    vertical: bool,
    strip: Strip,
}

impl Content {
    /// `bar` over the resolved scales.
    pub(crate) fn new<'a>(bar: &(dyn Draw<'a> + 'a), resolved: &Resolved) -> Content {
        Content {
            strip: bar.strip(resolved),
        }
    }

    /// The colorbar with its ticks on side `orient` (a vertical bar for
    /// left and right, a horizontal one for top and bottom), its bar
    /// `length` px long and one em of `theme` thick.
    pub(crate) fn block(&self, orient: Orient, length: f64, theme: &Theme) -> Block {
        let em = theme.em();
        let vertical = !axis::across(orient);
        let size = if vertical {
            Size::new(em, length)
        } else {
            Size::new(length, em)
        };
        Block {
            size,
            framed: false,
            bar: size.to_rect(),
            vertical,
            strip: self.strip.clone(),
        }
    }
}

impl Block {
    /// Its size, px.
    pub(crate) fn size(&self) -> Size {
        self.size
    }

    /// Its bar, relative to its top-left corner.
    pub(crate) fn bar(&self) -> Rect {
        self.bar
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
    /// legend's), then the strip's image ([`Strip::image`]) stretched over
    /// the bar with nearest sampling, its edges on device pixels on raster
    /// targets (`snap`).
    pub(crate) fn draw(&self, at: Point, theme: &Theme, snap: Snap, list: &mut List) {
        let by = at.to_vec2();
        if self.framed {
            keys::frame(self.size.to_rect() + by, theme, list);
        }
        let rect = snap.rect(self.bar + by);
        let length = if self.vertical {
            rect.height()
        } else {
            rect.width()
        };
        if let Some(image) = self.strip.image(length, snap.scale(), self.vertical) {
            list.commands.push(Command::Image {
                rect,
                image,
                smooth: false,
            });
        }
    }
}
