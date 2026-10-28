//! [`Area`]: one plot area's slot, its tiles on one grid however the
//! partition cuts it, and the tiles of an earlier grid shown as a preview
//! until every tile of the new one is sharp.
//!
//! A refresh moves each area to the views its axes show now. While they
//! show what the grid began at, the grid stays and a change of content
//! re-renders its tiles, each keeping its previous image until the new one
//! lands. When a gesture moved them, every key is new: the old tiles
//! become the area's preview layer, drawn mapped from the views each was
//! planned at to the views now (one affine map per axis, linear filtering,
//! clipped to the area) under the new tiles as they land. At most one
//! preview layer is kept: a second change before the first is sharp keeps
//! the first and maps it on.

use iced_core::image;
use iced_core::renderer::Quad;
use iced_core::{Color, Rectangle};
use rustd3_figure::kurbo::Rect;

use crate::plot::preview::{Affine1, Basis};
use crate::plot::region::Along;
use crate::plot::slot::{self, Band};
use crate::plot::tile::{Grid, Key, Tile};

/// One plot area: where it lies, the grid its tiles lie on and the views
/// that grid began at, its content, its tiles and its preview layer.
pub(crate) struct Area {
    /// Its position among the plot areas.
    index: u32,
    /// Device px, relative to the widget's device origin.
    rect: [i32; 4],
    /// The plot area in list px, which preview maps go through.
    list: Rect,
    grid: Grid,
    /// Distinct for every grid a widget's areas begin: results rendered on
    /// another grid are dropped.
    serial: u64,
    /// The views of the handles leading each axis when the grid began.
    anchor: [Option<Basis>; 2],
    /// Hash of the area's current content.
    hash: u64,
    /// The grid's tiles, row by row.
    tiles: Vec<Tile>,
    /// Tiles of an earlier grid, drawn mapped while the grid's are not all
    /// sharp.
    preview: Vec<Tile>,
}

/// What an area shows in the following draw, counted over its tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Shown {
    /// Tiles that show an image: their own, or the preview layer's.
    pub(crate) images: u32,
    /// Tiles that show neither: the theme background.
    pub(crate) missing: u32,
    /// Tiles that show other content than the current: an outdated image
    /// of their own, or the preview layer.
    pub(crate) previews: u32,
}

impl Area {
    /// Plot area `index` over device rect `rect` (list px `list`), holding
    /// content `hash`, its grid anchored at its corner at the views `now`
    /// and numbered `serial`; no tile has an image yet.
    pub(crate) fn new(
        index: u32,
        (rect, list): ([i32; 4], Rect),
        hash: u64,
        serial: u64,
        now: [Option<Basis>; 2],
    ) -> Area {
        let mut area = Area {
            index,
            rect,
            list,
            grid: Grid::new([rect[0], rect[1]], density(now)),
            serial,
            anchor: now,
            hash,
            tiles: Vec::new(),
            preview: Vec::new(),
        };
        area.tiles = area.cover(hash);
        area
    }

    /// The device rect.
    pub(crate) fn rect(&self) -> [i32; 4] {
        self.rect
    }

    /// The grid's number.
    pub(crate) fn serial(&self) -> u64 {
        self.serial
    }

    /// The tiles, row by row.
    pub(crate) fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// The tiles, to change.
    pub(crate) fn tiles_mut(&mut self) -> &mut [Tile] {
        &mut self.tiles
    }

    /// Follows the views `now`: when they show what the grid began at
    /// nothing moves; otherwise the grid begins anew there, numbered
    /// `serial`, every key new and without an image, and the old tiles
    /// become the preview layer (all of them when each had an image, else
    /// the earlier preview stays if there is one). True when the grid
    /// began anew.
    pub(crate) fn follow(&mut self, now: [Option<Basis>; 2], serial: u64) -> bool {
        let still = self.anchor.iter().zip(&now).all(|pair| match pair {
            (Some(was), Some(is)) => was.to(*is) == Some(Affine1::IDENTITY),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        });
        if still {
            return false;
        }
        let old = std::mem::take(&mut self.tiles);
        let complete = old.iter().all(|tile| !tile.slot().is_missing());
        if complete || self.preview.is_empty() {
            self.preview = old
                .into_iter()
                .filter(|tile| !tile.slot().is_missing())
                .collect();
        }
        self.grid = Grid::new([self.rect[0], self.rect[1]], density(now));
        self.serial = serial;
        self.anchor = now;
        self.tiles = self.cover(self.hash);
        true
    }

    /// Whether content `hash` needs a render: another hash, or a tile
    /// without an image and without a pending render.
    pub(crate) fn is_stale(&self, hash: u64) -> bool {
        hash != self.hash
            || self
                .tiles
                .iter()
                .any(|tile| tile.slot().is_missing() && tile.slot().pending().is_none())
    }

    /// The area's content is `hash` at `generation`: every tile that does
    /// not hold it (all of them with `every`), or holds it without an
    /// image and without a pending render, is marked pending. True when
    /// one was.
    pub(crate) fn change(&mut self, hash: u64, generation: u64, every: bool) -> bool {
        self.hash = hash;
        let mut changed = false;
        for tile in &mut self.tiles {
            let slot = tile.slot_mut();
            let stale = slot.hash() != hash || (slot.is_missing() && slot.pending().is_none());
            if every || stale {
                slot.change(hash, generation);
                changed = true;
            }
        }
        changed
    }

    /// Applies a tile's rendered image: shown when it was rendered on the
    /// current grid for tile `(i, j)` and is newer than the tile's image.
    /// The preview layer goes once every tile is current. True when
    /// shown.
    pub(crate) fn land(
        &mut self,
        (serial, i, j): (u64, i32, i32),
        (generation, hash): (u64, u64),
        bands: Option<Vec<Band>>,
        view: [Option<Basis>; 2],
    ) -> bool {
        if serial != self.serial {
            return false;
        }
        let Some(tile) = self
            .tiles
            .iter_mut()
            .find(|tile| (tile.key().i, tile.key().j) == (i, j))
        else {
            return false;
        };
        let shown = tile.slot_mut().land(generation, hash, bands, view);
        if self.tiles.iter().all(|tile| tile.slot().is_current()) {
            self.preview.clear();
        }
        shown
    }

    /// What the following draw shows at factor `factor` and the views
    /// `now`, counted over the tiles: a tile without an image that the
    /// preview layer covers counts as a preview.
    pub(crate) fn shown(&self, factor: f32, now: [Option<Basis>; 2]) -> Shown {
        let cover = self.coverage(factor, now);
        let mut shown = Shown::default();
        for tile in &self.tiles {
            let slot = tile.slot();
            let covered = cover.is_some_and(|c| contains(c, slot.rect()));
            if slot.is_missing() && !covered {
                shown.missing += 1;
                continue;
            }
            shown.images += 1;
            if !slot.is_current() {
                shown.previews += 1;
            }
        }
        shown
    }

    /// Bytes of tile images shown since the last call.
    pub(crate) fn take_fresh(&mut self) -> u64 {
        self.tiles
            .iter_mut()
            .map(|tile| tile.slot_mut().take_fresh())
            .sum()
    }

    /// Draws the area for a widget whose device origin is `origin` at
    /// `factor`, the views now `now`: the theme `background` when a tile
    /// has no image, the preview layer mapped to `now`, then each tile's
    /// image on its device pixels, all clipped to the area.
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        (origin, factor): ([i32; 2], f32),
        now: [Option<Basis>; 2],
        background: Color,
    ) where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        if self.tiles.iter().any(|tile| tile.slot().is_missing()) {
            let bounds: Rectangle = slot::logical(self.rect, origin, factor);
            renderer.fill_quad(
                Quad {
                    bounds,
                    ..Quad::default()
                },
                background,
            );
        }
        for tile in &self.preview {
            let maps = self.maps(tile, factor, now);
            tile.slot()
                .draw_mapped(renderer, (origin, factor), maps, self.rect);
        }
        for tile in &self.tiles {
            let slot = tile.slot();
            slot.draw_at(renderer, (origin, factor), [0, 0], slot.rect());
        }
    }

    /// The grid's tiles over the area, holding content `hash`, without
    /// images.
    fn cover(&self, hash: u64) -> Vec<Tile> {
        self.grid
            .cover(self.rect)
            .into_iter()
            .map(|(i, j, rect)| {
                let key = Key {
                    area: self.index,
                    i,
                    j,
                };
                Tile::new(key, rect, hash)
            })
            .collect()
    }

    /// The device maps (x, then y) of a preview tile from the views it was
    /// planned at to `now`: the identity along an axis whose lead handle
    /// or epoch changed.
    fn maps(&self, tile: &Tile, factor: f32, now: [Option<Basis>; 2]) -> [Affine1; 2] {
        let planned = tile.slot().planned().unwrap_or([None; 2]);
        let rect = tile.slot().rect();
        [Along::X, Along::Y].map(|along| {
            let edges = match along {
                Along::X => [rect[0], rect[2]],
                Along::Y => [rect[1], rect[3]],
            };
            let i = along.index();
            planned[i]
                .zip(now[i])
                .and_then(|(planned, now)| planned.to(now))
                .and_then(|shares| {
                    Affine1::device(shares, along, self.list, f64::from(factor), edges)
                })
                .unwrap_or(Affine1::IDENTITY)
        })
    }

    /// The device rect the preview layer covers, mapped to `now`: the
    /// bounds of its tiles' mapped rects; `None` without a preview.
    fn coverage(&self, factor: f32, now: [Option<Basis>; 2]) -> Option<[f64; 4]> {
        self.preview
            .iter()
            .map(|tile| {
                let [mx, my] = self.maps(tile, factor, now);
                let [x0, y0, x1, y1] = tile.slot().rect().map(f64::from);
                let (a, b) = (mx.apply(x0), mx.apply(x1));
                let (c, d) = (my.apply(y0), my.apply(y1));
                [a.min(b), c.min(d), a.max(b), c.max(d)]
            })
            .reduce(|a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            })
    }
}

/// Each axis's view scale in `now` (1 for an axis without a lead handle):
/// the grid's density.
fn density(now: [Option<Basis>; 2]) -> [f64; 2] {
    now.map(|basis| basis.map_or(1.0, |basis| basis.view.map.scale()))
}

/// Whether `outer` (device px, f64) contains device rect `inner`.
fn contains(outer: [f64; 4], inner: [i32; 4]) -> bool {
    let [x0, y0, x1, y1] = inner.map(f64::from);
    outer[0] <= x0 && outer[1] <= y0 && x1 <= outer[2] && y1 <= outer[3]
}
