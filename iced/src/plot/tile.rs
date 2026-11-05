//! Tiles: a plot area cut into squares of [`SIZE`] device px on a [`Grid`]
//! anchored at the device position of a data point. Each square is a
//! [`Tile`] that shows one image, found by its [`Key`].
//!
//! Plot areas, not partition cells, carry tiles, so a plot area that other
//! panels' edges cut into several cells still has one grid. Tile `(i, j)`
//! is the square `[origin.x + SIZE·i, origin.x + SIZE·(i + 1)) ×
//! [origin.y + SIZE·j, origin.y + SIZE·(j + 1))`. An area has the keys
//! whose squares meet its device rect grown by one tile on each side, and
//! the keys outside the area are the overscan ring. Every tile renders its
//! whole square, so a ring tile reaches up to two tiles beyond the area,
//! which the list's overscan covers. A tile's image is 256 × 256 × 4 =
//! 262,144 B, far under `BAND_BYTES`, so it is one band, and wgpu's 2048²
//! atlas layers hold 64 of them (`wgpu/src/image/atlas.rs:13-14`).
//!
//! A whole-pixel pan moves the grid's origin by the same device px. The
//! keys stay and their squares move. A tile's image lies where it was
//! rendered, moved by how far the origin moved since, so it covers its
//! square until the key leaves the grown rect. A steady pan therefore
//! renders one new column of keys per 256 px.

use crate::plot::slot::Slot;

/// The side of a tile, in device px.
pub(crate) const SIZE: i32 = 256;

/// Device px the list reaches past the farthest ring tile, which lies two
/// tiles beyond the area. A line drawn to the edge of what a list holds
/// ends one sample beyond it, so within half its width of that edge it is
/// not the line a wider list draws. The margin keeps that edge out of the
/// ring's tiles for lines up to 16 device px wide.
pub(crate) const MARGIN: i32 = 8;

/// Device px the plot areas' layers are drawn beyond each area: two tiles,
/// as far as a ring tile's square reaches, plus the margin.
pub(crate) const OVERSCAN: i32 = 2 * SIZE + MARGIN;

/// The tile grid of one plot area: the device position of its data anchor
/// and the scale density it was rendered at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Grid {
    origin: [i32; 2],
    /// The transformed-domain span per device px, per axis, as f64 bits.
    /// It is the scale of the view each axis's lead handle shows (range
    /// shares per share at its epoch's start), which a zoom changes and a
    /// pan keeps.
    density: [u64; 2],
}

/// A tile: its plot area's position among the plot areas, and its column
/// and row on the area's grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    pub(crate) area: u32,
    pub(crate) i: i32,
    pub(crate) j: i32,
}

/// One tile of a plot area's grid: its key, its square where the grid
/// lies, and the slot that holds the image it shows, the generation that
/// image was rendered for and the render it waits for. The slot's rect is
/// where the image was rendered.
pub(crate) struct Tile {
    key: Key,
    /// Its square in device px, where the grid lies.
    rect: [i32; 4],
    /// The grid's origin when the image shown was planned.
    at: [i32; 2],
    slot: Slot,
}

impl Grid {
    /// The grid whose tile (0, 0) starts at device `origin`, rendered at
    /// view scales `density`.
    pub(crate) fn new(origin: [i32; 2], density: [f64; 2]) -> Grid {
        Grid {
            origin,
            density: density.map(f64::to_bits),
        }
    }

    /// The device position of the data anchor.
    pub(crate) fn origin(self) -> [i32; 2] {
        self.origin
    }

    /// The grid moved by `[dx, dy]` device px, as by a whole-pixel pan.
    pub(crate) fn moved(self, [dx, dy]: [i32; 2]) -> Grid {
        Grid {
            origin: [self.origin[0] + dx, self.origin[1] + dy],
            ..self
        }
    }

    /// The square of tile `(i, j)`, in device px.
    pub(crate) fn square(self, i: i32, j: i32) -> [i32; 4] {
        let [x, y] = self.origin;
        [
            x + SIZE * i,
            y + SIZE * j,
            x + SIZE * (i + 1),
            y + SIZE * (j + 1),
        ]
    }

    /// Every `(i, j)` whose square meets `bounds` (device px), row by row,
    /// with its whole square.
    pub(crate) fn cover(self, bounds: [i32; 4]) -> Vec<(i32, i32, [i32; 4])> {
        let [x0, y0, x1, y1] = bounds;
        if x1 <= x0 || y1 <= y0 {
            return Vec::new();
        }
        let first = |from: i32, at: i32| (from - at).div_euclid(SIZE);
        let last = |to: i32, at: i32| (to - 1 - at).div_euclid(SIZE);
        let (i0, i1) = (first(x0, self.origin[0]), last(x1, self.origin[0]));
        let (j0, j1) = (first(y0, self.origin[1]), last(y1, self.origin[1]));
        (j0..=j1)
            .flat_map(|j| (i0..=i1).map(move |i| (i, j)))
            .map(|(i, j)| (i, j, self.square(i, j)))
            .collect()
    }
}

impl Tile {
    /// Tile `key` over device rect `rect` of the grid at `origin`, holding
    /// content `hash`, with no image yet.
    pub(crate) fn new(key: Key, rect: [i32; 4], origin: [i32; 2], hash: u64) -> Tile {
        Tile {
            key,
            rect,
            at: origin,
            slot: Slot::new(rect, hash),
        }
    }

    /// The tile's key.
    pub(crate) fn key(&self) -> Key {
        self.key
    }

    /// The tile's device rect, where the grid lies.
    pub(crate) fn rect(&self) -> [i32; 4] {
        self.rect
    }

    /// Records that the grid moved the tile's square to `rect`.
    pub(crate) fn place(&mut self, rect: [i32; 4]) {
        self.rect = rect;
    }

    /// How far the image shown has moved with the grid, now at `origin`,
    /// since it was planned.
    pub(crate) fn shift(&self, origin: [i32; 2]) -> [i32; 2] {
        [origin[0] - self.at[0], origin[1] - self.at[1]]
    }

    /// Whether the image shown, moved with the grid now at `origin`,
    /// covers the tile's rect.
    pub(crate) fn covers(&self, origin: [i32; 2]) -> bool {
        if self.slot.is_missing() {
            return false;
        }
        let [dx, dy] = self.shift(origin);
        let [x0, y0, x1, y1] = self.slot.rect();
        let [a0, b0, a1, b1] = self.rect;
        x0 + dx <= a0 && y0 + dy <= b0 && a1 <= x1 + dx && b1 <= y1 + dy
    }

    /// Whether the image shown, moved with the grid now at `origin`,
    /// covers the part of the tile's rect inside `within` (true when they
    /// do not meet, false without an image).
    pub(crate) fn covers_in(&self, origin: [i32; 2], within: [i32; 4]) -> bool {
        if self.slot.is_missing() {
            return false;
        }
        let [a0, b0, a1, b1] = intersect(self.rect, within);
        if a0 >= a1 || b0 >= b1 {
            return true;
        }
        let [dx, dy] = self.shift(origin);
        let [x0, y0, x1, y1] = self.slot.rect();
        x0 + dx <= a0 && y0 + dy <= b0 && a1 <= x1 + dx && b1 <= y1 + dy
    }

    /// Records that an image rendered over `rect` of the grid at `origin`
    /// landed and is the one shown.
    pub(crate) fn shown_at(&mut self, rect: [i32; 4], origin: [i32; 2]) {
        self.slot.reshape(rect);
        self.at = origin;
    }

    /// The slot holding its image.
    pub(crate) fn slot(&self) -> &Slot {
        &self.slot
    }

    /// The slot holding its image, to change.
    pub(crate) fn slot_mut(&mut self) -> &mut Slot {
        &mut self.slot
    }
}

/// The part of device rect `a` inside `b`, empty (zero area) when they do
/// not meet.
pub(crate) fn intersect(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    let [x0, y0] = [a[0].max(b[0]), a[1].max(b[1])];
    let [x1, y1] = [a[2].min(b[2]).max(x0), a[3].min(b[3]).max(y0)];
    [x0, y0, x1, y1]
}

/// Whether device rects `a` and `b` share a pixel.
pub(crate) fn meets(a: [i32; 4], b: [i32; 4]) -> bool {
    let [x0, y0, x1, y1] = intersect(a, b);
    x0 < x1 && y0 < y1
}

#[cfg(test)]
mod tests {
    use super::{Grid, SIZE, intersect};
    use crate::plot::BAND_BYTES;

    #[test]
    fn a_900_by_600_area_has_4_by_3_tiles() {
        let grid = Grid::new([0, 0], [1.0; 2]);
        let cover = grid.cover([0, 0, 900, 600]);
        assert_eq!(cover.len(), 12);
        assert_eq!(cover[0], (0, 0, [0, 0, 256, 256]));
        assert_eq!(cover[3], (3, 0, [768, 0, 1024, 256]));
        assert_eq!(cover[11], (3, 2, [768, 512, 1024, 768]));
        // With a ring of one tile there are 6 × 5 = 30 keys, each with its
        // whole square.
        let ring = grid.cover([-256, -256, 1156, 856]);
        assert_eq!(ring.len(), 30);
        assert_eq!(ring[0], (-1, -1, [-256, -256, 0, 0]));
        assert_eq!(ring[29], (4, 3, [1024, 768, 1280, 1024]));
    }

    #[test]
    fn tiles_follow_their_origin() {
        let grid = Grid::new([37, 0], [1.0; 2]);
        assert_eq!(grid.square(-2, 0), [-475, 0, -219, 256]);
        let cover = grid.cover([-256, 0, 1156, 256]);
        assert_eq!(cover[0], (-2, 0, [-475, 0, -219, 256]));
        assert_eq!(cover.last().copied(), Some((4, 0, [1061, 0, 1317, 256])));
        // The farthest squares stay inside a 900 px area's overscan.
        let (first, last) = (cover[0].2, cover[cover.len() - 1].2);
        assert!(first[0] >= -super::OVERSCAN && last[2] <= 900 + super::OVERSCAN);
    }

    #[test]
    fn one_tile_is_far_under_band_bytes() {
        let bytes = usize::try_from(SIZE * SIZE * 4).unwrap();
        assert_eq!(bytes, 262_144);
        assert!(bytes < BAND_BYTES);
        // 2048² atlas layers hold 64 tiles, and a 3600 × 1900 area needs
        // 120.
        assert_eq!((2048 / SIZE) * (2048 / SIZE), 64);
        let tiles = Grid::new([0, 0], [1.0; 2]).cover([0, 0, 3600, 1900]).len();
        assert_eq!(tiles, 15 * 8);
        assert_eq!(tiles.div_ceil(64), 2);
    }

    #[test]
    fn rects_that_do_not_meet_intersect_empty() {
        assert_eq!(intersect([0, 0, 10, 10], [5, 5, 20, 20]), [5, 5, 10, 10]);
        let [x0, _, x1, _] = intersect([0, 0, 10, 10], [20, 0, 30, 10]);
        assert_eq!(x0, x1);
    }
}
