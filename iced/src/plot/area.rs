//! [`Area`]: one plot area's slot, its tiles on one grid however the
//! partition cuts it, the tiles of an earlier grid shown as a preview, and
//! the overlays of what stays put over them.
//!
//! A refresh moves each area to the views its axes show now (D20.3: each
//! lead handle's view in shares of its range). Measured from the views
//! the grid began at, a move by whole device pixels on both axes moves the
//! grid's origin by as much: keys stay, squares move, and every tile's
//! image, rendered over its whole square, moves along and stays valid, so
//! a pan renders only the keys entering the area's grown rect, a column
//! per 256 px (D31.6). Any other move begins the grid anew
//! at the area's corner: every key is new, and the old tiles become the
//! area's preview layer, drawn mapped from the views each was planned at
//! to the views now (one affine map per axis, linear filtering, clipped
//! to the area) under the new tiles as they land. At most one preview
//! layer is kept: a second change before the first is sharp keeps the
//! first and maps it on.
//!
//! The keys cover the area grown by one tile on every side: the overscan
//! ring, rendered after the visible tiles, which a pan brings into view
//! and a zoom-out maps into the preview. What stays put inside the area
//! (the inner halves of axis lines, a border, an inset legend) is not in
//! the tiles but in overlays drawn over them, rendered with nothing under
//! them.

use iced_core::Color;
use iced_core::image;
use iced_core::renderer::Quad;
use rustd3_figure::kurbo::Rect;

use crate::plot::preview::{Affine1, Basis};
use crate::plot::region::Along;
use crate::plot::slot::{self, Band, Slot};
use crate::plot::tile::{self, Grid, Key, SIZE, Tile};

/// One plot area: where it lies, the grid its tiles lie on and the views
/// that grid began at, its content, its tiles, its preview layer and its
/// overlays.
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
    /// Hash of the area's current layer.
    hash: u64,
    /// The style hash of its current layer: what it looks like apart from
    /// where it draws ([`style::hash`](crate::plot::style::hash)).
    style: u64,
    /// The grid's keys that meet the grown rect, row by row.
    tiles: Vec<Tile>,
    /// Tiles of an earlier grid, drawn mapped while the grid's visible
    /// ones are not all sharp.
    preview: Vec<Tile>,
    /// What stays put over the tiles, each over the device rect it reaches
    /// into the area.
    overlays: Vec<Slot>,
}

/// How a refresh moved an area's grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moved {
    /// Not at all.
    Still,
    /// By whole device pixels: its tiles' images move along.
    Translated,
    /// Otherwise: the grid began anew.
    Anew,
}

/// What an area shows in the following draw, counted over its visible
/// tiles and its overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Shown {
    /// Those that show an image: their own, or the preview layer's.
    pub(crate) images: u32,
    /// Those that show neither: the theme background.
    pub(crate) missing: u32,
    /// Those that show other content than the current: an outdated image
    /// of their own, or the preview layer.
    pub(crate) previews: u32,
}

impl Area {
    /// Plot area `index` over device rect `rect` (list px `list`), holding
    /// a layer of hash `hash` and style hash `style`, its grid anchored at
    /// its corner at the views `now` and numbered `serial`; no tile has an
    /// image yet.
    pub(crate) fn new(
        index: u32,
        (rect, list): ([i32; 4], Rect),
        (hash, style): (u64, u64),
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
            style,
            tiles: Vec::new(),
            preview: Vec::new(),
            overlays: Vec::new(),
        };
        area.tiles = area.cover(hash);
        area
    }

    /// The device rect.
    pub(crate) fn rect(&self) -> [i32; 4] {
        self.rect
    }

    /// The rect the keys meet: the area grown by one tile on each side.
    pub(crate) fn grown(&self) -> [i32; 4] {
        let [x0, y0, x1, y1] = self.rect;
        [x0 - SIZE, y0 - SIZE, x1 + SIZE, y1 + SIZE]
    }

    /// The grid's number.
    pub(crate) fn serial(&self) -> u64 {
        self.serial
    }

    /// The device position of the grid's data anchor.
    pub(crate) fn origin(&self) -> [i32; 2] {
        self.grid.origin()
    }

    /// The tiles, row by row.
    pub(crate) fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// The tiles, to change.
    pub(crate) fn tiles_mut(&mut self) -> &mut [Tile] {
        &mut self.tiles
    }

    /// The overlays.
    pub(crate) fn overlays(&self) -> &[Slot] {
        &self.overlays
    }

    /// The overlays, to change.
    pub(crate) fn overlays_mut(&mut self) -> &mut [Slot] {
        &mut self.overlays
    }

    /// Whether `tile` meets the area itself, not only its ring.
    pub(crate) fn shows(&self, tile: &Tile) -> bool {
        let [x0, y0, x1, y1] = tile::intersect(tile.rect(), self.rect);
        x0 < x1 && y0 < y1
    }

    /// Whether `tile` shows its part inside the area exact: an image of
    /// the current layer, moved with the grid, covering that part. A
    /// pending render of such a tile (one every slot re-renders for, or a
    /// ring tile) goes after the rest.
    pub(crate) fn is_exact(&self, tile: &Tile) -> bool {
        tile.slot().is_current() && tile.covers_in(self.grid.origin(), self.rect)
    }

    /// Follows the views `now` at `factor`: measured from the views the
    /// grid began at, a move by whole device pixels on both axes moves the
    /// grid's origin by as much (keys entering the grown rect are added
    /// without an image, those leaving it dropped); any other move begins
    /// the grid anew at the area's corner, numbered `serial`, every key
    /// new, and the old tiles become the preview layer (all of them when
    /// each visible one had an image, else the earlier preview stays if
    /// there is one).
    pub(crate) fn follow(&mut self, now: [Option<Basis>; 2], factor: f64, serial: u64) -> Moved {
        let Some(offset) = self.translation(now, factor) else {
            let old = std::mem::take(&mut self.tiles);
            let complete = old
                .iter()
                .filter(|tile| self.shows(tile))
                .all(|tile| !tile.slot().is_missing());
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
            return Moved::Anew;
        };
        let origin = self.grid.origin();
        let corner = [self.rect[0] + offset[0], self.rect[1] + offset[1]];
        if corner == origin {
            return Moved::Still;
        }
        let grid = self
            .grid
            .moved([corner[0] - origin[0], corner[1] - origin[1]]);
        self.grid = grid;
        let mut old = std::mem::take(&mut self.tiles);
        let (hash, index) = (self.hash, self.index);
        self.tiles = grid
            .cover(self.grown())
            .into_iter()
            .map(|(i, j, rect)| {
                let found = old.iter().position(|t| (t.key().i, t.key().j) == (i, j));
                match found {
                    Some(found) => {
                        let mut tile = old.swap_remove(found);
                        tile.place(rect);
                        tile
                    }
                    None => Tile::new(Key { area: index, i, j }, rect, grid.origin(), hash),
                }
            })
            .collect();
        Moved::Translated
    }

    /// Whether layer `hash` needs a render: another hash, or a tile
    /// without an image and without a pending render.
    pub(crate) fn is_stale(&self, hash: u64) -> bool {
        hash != self.hash
            || self
                .tiles
                .iter()
                .any(|tile| tile.slot().is_missing() && tile.slot().pending().is_none())
    }

    /// The area's layer is `hash`, of style hash `style`, at
    /// `generation`, its grid having `moved`: after a translation that
    /// kept the layer's style, an image that showed the layer before and
    /// covers its tile shows it now (the rules of Phase 31's dependency
    /// classification: a translation renders only entering tiles, anything
    /// else every tile); every other tile that does not show it, or that
    /// has no image and no pending render, is marked pending (every tile
    /// with `every`). True when one was.
    pub(crate) fn change(
        &mut self,
        (hash, style): (u64, u64),
        generation: u64,
        (moved, every): (Moved, bool),
    ) -> bool {
        let translated = moved == Moved::Translated && style == self.style && !every;
        self.hash = hash;
        self.style = style;
        let origin = self.grid.origin();
        let mut changed = false;
        for tile in &mut self.tiles {
            let covers = tile.covers(origin);
            let slot = tile.slot_mut();
            if translated && slot.is_current() && covers {
                slot.relabel(hash);
                continue;
            }
            let idle = slot.pending().is_none();
            let stale = slot.hash() != hash || (!covers && idle);
            if every || stale {
                slot.change(hash, generation);
                changed = true;
            }
        }
        changed
    }

    /// Replaces the overlays by one per rect of `rects` (device px inside
    /// the area) with the content hash beside it; an overlay whose rect
    /// stays keeps its image. Marks those whose content changed (all of
    /// them with `every`) pending at `generation` and returns their
    /// indices.
    pub(crate) fn overlay(
        &mut self,
        rects: &[([i32; 4], u64)],
        generation: u64,
        every: bool,
    ) -> Vec<usize> {
        let mut old = std::mem::take(&mut self.overlays);
        self.overlays = rects
            .iter()
            .map(
                |&(rect, hash)| match old.iter().position(|s| s.rect() == rect) {
                    Some(found) => old.swap_remove(found),
                    // A new overlay holds no content yet: any hash differs.
                    None => Slot::new(rect, !hash),
                },
            )
            .collect();
        let mut changed = Vec::new();
        for (index, (slot, &(_, hash))) in self.overlays.iter_mut().zip(rects).enumerate() {
            if every || slot.hash() != hash {
                slot.change(hash, generation);
                changed.push(index);
            }
        }
        changed
    }

    /// Whether the overlays over `rects` with those content hashes would
    /// change: another rect, or another hash.
    pub(crate) fn overlays_differ(&self, rects: &[([i32; 4], u64)]) -> bool {
        self.overlays.len() != rects.len()
            || self
                .overlays
                .iter()
                .zip(rects)
                .any(|(slot, &(rect, hash))| slot.rect() != rect || slot.hash() != hash)
    }

    /// Applies a tile's rendered image: shown when it was rendered on the
    /// current grid for tile `(i, j)` over `rect`, the grid's origin then
    /// `at`, and is newer than the tile's image. The preview layer goes once
    /// every visible tile is current. True when shown.
    pub(crate) fn land(
        &mut self,
        (serial, i, j, at): (u64, i32, i32, [i32; 2]),
        (generation, hash): (u64, u64),
        (rect, bands): ([i32; 4], Option<Vec<Band>>),
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
        if shown {
            tile.shown_at(rect, at);
        }
        let sharp = self
            .tiles
            .iter()
            .filter(|tile| self.shows(tile))
            .all(|tile| self.is_exact(tile));
        if sharp {
            self.preview.clear();
        }
        shown
    }

    /// Checks a fresh render of tile `(i, j)` of grid `serial`, over
    /// `rect` with the grid's origin at `at`, against the image it shows of
    /// generation and content `shown` (see [`Slot::check`]); a replacing
    /// render lies where it was rendered. `None` when the tile shows
    /// another image now.
    pub(crate) fn check(
        &mut self,
        (serial, i, j, at): (u64, i32, i32, [i32; 2]),
        shown: (u64, u64),
        (rect, bands): ([i32; 4], Vec<Band>),
    ) -> Option<u8> {
        if serial != self.serial {
            return None;
        }
        let tile = self
            .tiles
            .iter_mut()
            .find(|tile| (tile.key().i, tile.key().j) == (i, j))?;
        let max = tile.slot_mut().check(shown, bands)?;
        if max > 1 {
            tile.shown_at(rect, at);
        }
        Some(max)
    }

    /// Checks a fresh render of the overlay over `rect` against the image
    /// it shows (see [`Slot::check`]).
    pub(crate) fn check_overlay(
        &mut self,
        rect: [i32; 4],
        shown: (u64, u64),
        bands: Vec<Band>,
    ) -> Option<u8> {
        self.overlays
            .iter_mut()
            .find(|slot| slot.rect() == rect)?
            .check(shown, bands)
    }

    /// Applies an overlay's rendered image, if the area still has an
    /// overlay over `rect`. True when shown.
    pub(crate) fn land_overlay(
        &mut self,
        rect: [i32; 4],
        (generation, hash): (u64, u64),
        bands: Option<Vec<Band>>,
    ) -> bool {
        self.overlays
            .iter_mut()
            .find(|slot| slot.rect() == rect)
            .is_some_and(|slot| slot.land(generation, hash, bands, [None; 2]))
    }

    /// What the following draw shows at factor `factor` and the views
    /// `now`, counted over the visible tiles and the overlays: a tile whose
    /// image falls short of its part inside the area counts as a preview,
    /// and so does one without an image that the preview layer covers.
    pub(crate) fn shown(&self, factor: f32, now: [Option<Basis>; 2]) -> Shown {
        let cover = self.coverage(factor, now);
        let mut shown = Shown::default();
        for tile in self.tiles.iter().filter(|tile| self.shows(tile)) {
            let slot = tile.slot();
            let visible = tile::intersect(tile.rect(), self.rect);
            let covered = cover.is_some_and(|c| contains(c, visible));
            if slot.is_missing() && !covered {
                shown.missing += 1;
                continue;
            }
            shown.images += 1;
            if !self.is_exact(tile) {
                shown.previews += 1;
            }
        }
        for slot in &self.overlays {
            if slot.is_missing() {
                shown.missing += 1;
                continue;
            }
            shown.images += 1;
            if slot.is_preview() {
                shown.previews += 1;
            }
        }
        shown
    }

    /// Bytes of images shown since the last call.
    pub(crate) fn take_fresh(&mut self) -> u64 {
        let tiles: u64 = self
            .tiles
            .iter_mut()
            .map(|tile| tile.slot_mut().take_fresh())
            .sum();
        tiles + self.overlays.iter_mut().map(Slot::take_fresh).sum::<u64>()
    }

    /// Draws the area for a widget whose device origin is `origin` at
    /// `factor`, the views now `now`: the theme `background` when a
    /// visible tile's part inside the area is not covered by its image,
    /// the preview layer mapped
    /// to `now`, each visible tile's image moved with the grid and clipped
    /// to its square, then the overlays, all inside the area.
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        (origin, factor): ([i32; 2], f32),
        now: [Option<Basis>; 2],
        background: Color,
    ) where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let at = self.grid.origin();
        let visible = || self.tiles.iter().filter(|tile| self.shows(tile));
        if visible().any(|tile| !tile.covers_in(at, self.rect)) {
            renderer.fill_quad(
                Quad {
                    bounds: slot::logical(self.rect, origin, factor),
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
        for tile in visible() {
            let clip = tile::intersect(tile.rect(), self.rect);
            tile.slot()
                .draw_at(renderer, (origin, factor), tile.shift(at), clip);
        }
        for slot in &self.overlays {
            slot.draw_at(renderer, (origin, factor), [0, 0], slot.rect());
        }
    }

    /// The whole device pixels both axes moved by from the views the grid
    /// began at to `now` at `factor`; `None` for any other move (a zoom, a
    /// fraction of a pixel, another lead handle or epoch).
    fn translation(&self, now: [Option<Basis>; 2], factor: f64) -> Option<[i32; 2]> {
        let mut offset = [0; 2];
        for along in [Along::X, Along::Y] {
            let i = along.index();
            let edges = match along {
                Along::X => [self.rect[0], self.rect[2]],
                Along::Y => [self.rect[1], self.rect[3]],
            };
            let map = match (self.anchor[i], now[i]) {
                (Some(was), Some(is)) => {
                    let shares = was.to(is)?;
                    Affine1::device(shares, along, self.list, factor, edges)?
                }
                (None, None) => Affine1::IDENTITY,
                (Some(_), None) | (None, Some(_)) => return None,
            };
            let d = map.apply(0.0);
            let whole = d.round();
            if (map.scale() - 1.0).abs() > 1e-12 || (d - whole).abs() > 1e-6 {
                return None;
            }
            offset[i] = i32::try_from(whole as i64).ok()?;
        }
        Some(offset)
    }

    /// The grid's keys that meet the grown rect, holding layer `hash`, without
    /// images.
    fn cover(&self, hash: u64) -> Vec<Tile> {
        let origin = self.grid.origin();
        self.grid
            .cover(self.grown())
            .into_iter()
            .map(|(i, j, rect)| {
                let key = Key {
                    area: self.index,
                    i,
                    j,
                };
                Tile::new(key, rect, origin, hash)
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

#[cfg(test)]
mod tests {
    use iced_core::image::Handle;
    use rustd3_figure::kurbo::Rect;

    use super::{Area, Moved};
    use crate::plot::domains::Key;
    use crate::plot::preview::{Affine1, Basis, View};
    use crate::plot::slot::Band;

    /// The view of handle 0 panned by `offset` range shares.
    fn panned(offset: f64) -> [Option<Basis>; 2] {
        let key = Key::of::<rustd3_core::scale::Linear>(0);
        let view = View {
            epoch: 1,
            map: Affine1::new(1.0, offset),
        };
        [Some(Basis { key, view }), None]
    }

    /// A 900 × 600 px plot area at the origin at factor 1, every tile of
    /// its grid (the ring included) rendered for content 1.
    fn rendered() -> Area {
        let rect = [0, 0, 900, 600];
        let list = Rect::new(0.0, 0.0, 900.0, 600.0);
        let mut area = Area::new(0, (rect, list), (1, 0), 1, panned(0.0));
        assert!(area.change((1, 0), 1, (Moved::Anew, false)));
        let jobs: Vec<_> = area
            .tiles()
            .iter()
            .map(|tile| (tile.key(), tile.rect()))
            .collect();
        for (key, rect) in jobs {
            let [x0, y0, x1, y1] = rect;
            let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
            let band = Band::new(
                0..y1 - y0,
                Handle::from_rgba(w, h, vec![0; (w * h * 4) as usize]),
            );
            let at = (1, key.i, key.j, [0, 0]);
            assert!(area.land(at, (1, 1), (rect, Some(vec![band])), panned(0.0)));
        }
        area
    }

    /// Lands an image for every tile pending a render, of content `hash`
    /// on the grid as it lies now.
    fn land_pending(area: &mut Area, hash: u64) {
        let jobs: Vec<_> = area
            .tiles()
            .iter()
            .filter_map(|tile| Some((tile.key(), tile.rect(), tile.slot().pending()?)))
            .collect();
        let (serial, at) = (area.serial(), area.origin());
        for (key, rect, generation) in jobs {
            let [x0, y0, x1, y1] = rect;
            let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
            let band = Band::new(
                0..y1 - y0,
                Handle::from_rgba(w, h, vec![0; (w * h * 4) as usize]),
            );
            let tile = (serial, key.i, key.j, at);
            area.land(
                tile,
                (generation, hash),
                (rect, Some(vec![band])),
                [None; 2],
            );
        }
    }

    /// The keys of the tiles pending a render.
    fn pending(area: &Area) -> Vec<(i32, i32)> {
        area.tiles()
            .iter()
            .filter(|tile| tile.slot().pending().is_some())
            .map(|tile| (tile.key().i, tile.key().j))
            .collect()
    }

    #[test]
    fn a_900_by_600_area_has_30_keys_with_its_ring() {
        let area = rendered();
        assert_eq!(area.tiles().len(), 30);
        assert_eq!(area.grown(), [-256, -256, 1156, 856]);
        let shown = area.tiles().iter().filter(|tile| area.shows(tile)).count();
        assert_eq!(shown, 12);
        assert!(pending(&area).is_empty());
    }

    #[test]
    fn a_37_px_drag_renders_only_the_entering_ring_column() {
        let mut area = rendered();
        assert_eq!(area.follow(panned(37.0 / 900.0), 1.0, 2), Moved::Translated);
        assert_eq!(area.origin(), [37, 0]);
        // Column −1, [−256, 0) rendered, now [−219, 37): its square.
        let left = area
            .tiles()
            .iter()
            .find(|t| (t.key().i, t.key().j) == (-1, 0));
        assert_eq!(left.map(|t| t.rect()), Some([-219, 0, 37, 256]));
        assert!(left.is_some_and(|t| t.covers([37, 0])));
        assert!(area.change((2, 0), 2, (Moved::Translated, false)));
        // The new key i = −2, its whole square [−475, −219), in 5 rows;
        // nothing visible.
        let entering: Vec<(i32, i32)> = (-1..=3).map(|j| (-2, j)).collect();
        assert_eq!(pending(&area), entering);
        let new = area
            .tiles()
            .iter()
            .find(|t| (t.key().i, t.key().j) == (-2, 0));
        assert_eq!(new.map(|t| t.rect()), Some([-475, 0, -219, 256]));
    }

    #[test]
    fn a_37_px_drag_left_renders_no_tile() {
        let mut area = rendered();
        assert_eq!(
            area.follow(panned(-37.0 / 900.0), 1.0, 2),
            Moved::Translated
        );
        assert_eq!(area.origin(), [-37, 0]);
        // Column 4, rendered over its square [1024, 1280), is at [987,
        // 1243): still its square; no new key (i = −2 would be [−549,
        // −293)).
        assert!(!area.change((2, 0), 2, (Moved::Translated, false)));
        assert!(pending(&area).is_empty());
        assert_eq!(area.tiles().len(), 30);
    }

    #[test]
    fn a_translation_with_another_style_renders_every_tile() {
        let mut area = rendered();
        assert_eq!(
            area.follow(panned(-37.0 / 900.0), 1.0, 2),
            Moved::Translated
        );
        // The positions moved and the style hash changed: no image shows
        // the new layer, so all 30 tiles render.
        assert!(area.change((2, 1), 2, (Moved::Translated, false)));
        assert_eq!(pending(&area).len(), 30);
        // Every image moved with the grid and still shows old content.
        let shown = area.shown(1.0, panned(-37.0 / 900.0));
        assert_eq!((shown.missing, shown.previews), (0, 12));
    }

    /// A render of tile (0, 0) for `generation`, of content `hash`,
    /// planned on the grid at `origin` over its square there.
    fn render_00(area: &mut Area, (generation, hash): (u64, u64), origin: [i32; 2]) -> bool {
        let rect = [origin[0], origin[1], origin[0] + 256, origin[1] + 256];
        let band = Band::new(0..256, Handle::from_rgba(256, 256, vec![0; 256 * 256 * 4]));
        let tile = (area.serial(), 0, 0, origin);
        area.land(
            tile,
            (generation, hash),
            (rect, Some(vec![band])),
            [None; 2],
        )
    }

    #[test]
    fn newest_result_wins_per_tile() {
        // Generation 3 changes the content, 4 pans it by 37 px, 5 changes
        // it again; their results for tile (0, 0) land as 3, 5, 4.
        let mut area = rendered();
        assert!(area.change((3, 0), 3, (Moved::Still, false)));
        assert_eq!(area.follow(panned(37.0 / 900.0), 1.0, 2), Moved::Translated);
        assert!(area.change((4, 0), 4, (Moved::Translated, false)));
        assert!(area.change((5, 0), 5, (Moved::Still, false)));
        let tile = |area: &Area| {
            let found = area
                .tiles()
                .iter()
                .find(|t| (t.key().i, t.key().j) == (0, 0));
            let tile = found.expect("tile (0, 0)");
            (
                tile.slot().generation(),
                area.is_exact(tile),
                tile.shift(area.origin()),
            )
        };
        // 3 is newer than what shows: shown, a preview of the grid it was
        // planned on, moved with it since.
        assert!(render_00(&mut area, (3, 3), [0, 0]));
        assert_eq!(tile(&area), (Some(3), false, [37, 0]));
        // 5 is the pending one: sharp.
        assert!(render_00(&mut area, (5, 5), [37, 0]));
        assert_eq!(tile(&area), (Some(5), true, [0, 0]));
        // 4 is older than what shows: dropped.
        assert!(!render_00(&mut area, (4, 4), [37, 0]));
        assert_eq!(tile(&area), (Some(5), true, [0, 0]));
    }

    #[test]
    fn a_steady_pan_renders_a_column_per_tile() {
        let mut area = rendered();
        // Left by 4 px a frame: the column i = 5 enters at the frame whose
        // origin brings its square [1280 + o, …) inside the grown rect,
        // o < −124, and nothing else renders on the way.
        let mut entered = Vec::new();
        for frame in 1..=40_u64 {
            let offset = -4 * frame as i32;
            let now = panned(f64::from(offset) / 900.0);
            let hash = 1 + frame;
            assert_eq!(area.follow(now, 1.0, 2), Moved::Translated);
            if area.change((hash, 0), 1 + frame, (Moved::Translated, false)) {
                entered.push((offset, pending(&area)));
                land_pending(&mut area, hash);
            }
        }
        let column: Vec<(i32, i32)> = (-1..=3).map(|j| (5, j)).collect();
        assert_eq!(entered, [(-128, column)]);
    }

    #[test]
    fn a_zoom_begins_the_grid_anew_and_keeps_the_old_tiles_as_its_preview() {
        let mut area = rendered();
        let key = Key::of::<rustd3_core::scale::Linear>(0);
        // Zoom out ×1.2 about the area's centre: shares p ↦ p / 1.2 + 0.0833.
        let zoom = Affine1::new(1.0 / 1.2, 0.5 - 0.5 / 1.2);
        let view = View {
            epoch: 1,
            map: zoom,
        };
        let now = [Some(Basis { key, view }), None];
        assert_eq!(area.follow(now, 1.0, 2), Moved::Anew);
        assert_eq!(area.serial(), 2);
        // The old view [0, 900) maps to [75, 825); its ring fills the rest,
        // so every new tile is covered: previews, none missing.
        let shown = area.shown(1.0, now);
        assert_eq!((shown.missing, shown.previews), (0, 12));
    }
}
