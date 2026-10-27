//! Space reserved by figure slots: legends outside the grid, stacked
//! outward from it per slot, and where they land once the plot areas are
//! known.

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

    /// Each legend and its top-left corner, in the order they were added:
    /// a slot's first legend 0.5 em outside `grid` (the box the tracks
    /// fill), each further one 0.5 em past the one before; top and bottom
    /// legends centred across `extent` (the plot areas' union), left and
    /// right ones centred on it vertically. A legend larger than the
    /// extent overflows it equally on both ends.
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
                let at = match slot {
                    Slot::Top => {
                        Point::new(center.x - size.width / 2.0, grid.y0 - near - size.height)
                    }
                    Slot::Bottom => Point::new(center.x - size.width / 2.0, grid.y1 + near),
                    Slot::Left => {
                        Point::new(grid.x0 - near - size.width, center.y - size.height / 2.0)
                    }
                    Slot::Right => Point::new(grid.x1 + near, center.y - size.height / 2.0),
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
