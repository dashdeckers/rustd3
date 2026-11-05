//! Space reserved by figure slots: legends and colorbars outside the grid,
//! stacked outward from it per slot, and where they land once the plot
//! areas are known.

use kurbo::{Point, Rect};

use crate::legend::{Block, Slot};

/// From the grid to the nearest legend of a slot, and between the legends
/// of one slot, em: matplotlib's `legend.borderaxespad`.
const GAP: f64 = 0.5;

/// The legends in figure slots, laid out, in the order they were added.
pub(crate) struct Slots {
    legends: Vec<(Slot, Block)>,
}

impl Slots {
    /// The slots holding `legends`.
    pub(crate) fn new(legends: Vec<(Slot, Block)>) -> Slots {
        Slots { legends }
    }

    /// The px each side's legends take from the grid, as [top, right,
    /// bottom, left]: per legend, 0.5 em and its extent across the side.
    pub(crate) fn reserve(&self, em: f64) -> [f64; 4] {
        let mut reserve = [0.0; 4];
        for (slot, block) in &self.legends {
            let (side, extent) = across(*slot, block);
            reserve[side] += GAP * em + extent;
        }
        reserve
    }

    /// Each guide and its top-left corner, in the order they were added. A
    /// slot's first guide sits 0.5 em outside `grid` (the box the tracks
    /// fill), and each further one 0.5 em past the one before. Top and
    /// bottom legends are centered across `extent` (the plot areas' union),
    /// and left and right ones are centered on it vertically. A legend
    /// larger than the extent overflows it equally on both ends. A
    /// colorbar's bar runs along the extent, from its start.
    pub(crate) fn place(self, grid: Rect, extent: Rect, em: f64) -> Vec<(Block, Point)> {
        let gap = GAP * em;
        // How far each side's legends reach past the grid so far.
        let mut out = [0.0; 4];
        let center = extent.center();
        self.legends
            .into_iter()
            .map(|(slot, block)| {
                let (side, thickness) = across(slot, &block);
                let size = block.size();
                let near = out[side] + gap;
                out[side] = near + thickness;
                // Where the block starts along the side: centered, or with
                // a colorbar's bar at the extent's start.
                let (x, y) = match block.bar() {
                    Some(bar) => (extent.x0 - bar.x0, extent.y0 - bar.y0),
                    None => (center.x - size.width / 2.0, center.y - size.height / 2.0),
                };
                let at = match slot {
                    Slot::Top => Point::new(x, grid.y0 - near - size.height),
                    Slot::Bottom => Point::new(x, grid.y1 + near),
                    Slot::Left => Point::new(grid.x0 - near - size.width, y),
                    Slot::Right => Point::new(grid.x1 + near, y),
                };
                (block, at)
            })
            .collect()
    }
}

/// The side a legend in `slot` reserves (top, right, bottom, left as 0 to
/// 3) and its extent across it.
fn across(slot: Slot, block: &Block) -> (usize, f64) {
    let size = block.size();
    match slot {
        Slot::Top => (0, size.height),
        Slot::Right => (1, size.width),
        Slot::Bottom => (2, size.height),
        Slot::Left => (3, size.width),
    }
}
