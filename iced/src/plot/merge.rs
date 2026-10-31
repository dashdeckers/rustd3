//! [`Run`]: gutter cells merged into one slot. The partition cuts the
//! canvas at every plot-area edge, so an axis strip under a row of panels,
//! with the corners its labels reach into, is several cells that a pan
//! re-renders together; each would be its own render and its own image.
//! After the cells are classified, the adjacent gutter cells of one row
//! band that depend on the same handles ([`Dep`]) merge, from the left,
//! while the merged image stays under `BAND_BYTES` (a single cell at or
//! over it stays alone and is drawn as bands). Merging happens when the
//! partition is built: during a gesture the layout holds, so the runs hold
//! too, and a run whose cells change apart from each other only renders
//! more pixels than it had to.

use std::ops::Range;

use rustd3_figure::Resolved;
use rustd3_figure::list::{self, List, part};

use crate::plot::BAND_BYTES;
use crate::plot::layer::{self, FNV, fnv};
use crate::plot::slot::{self, Slot};

/// Adjacent gutter cells of one row band shown as one slot.
pub(crate) struct Run {
    /// The partition cells it holds, consecutive in one row.
    cells: Range<usize>,
    slot: Slot,
}

/// What a gutter cell's content changes with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Dep {
    /// A handle, by its table position: the ticks and labels of an axis
    /// that shows it.
    Handle(usize),
    /// A part that changes on its own: a legend.
    Part(part::Id),
}

impl Run {
    /// The cells `cells` of a partition whose device rects are `rects`,
    /// holding content `hash`, with no image yet.
    pub(crate) fn new(cells: Range<usize>, rects: &[[i32; 4]], hash: u64) -> Run {
        let rect = cells
            .clone()
            .map(|c| rects[c])
            .reduce(|a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            })
            .expect("invariant: a run holds a cell");
        Run {
            cells,
            slot: Slot::new(rect, hash),
        }
    }

    /// The partition cells it holds.
    pub(crate) fn cells(&self) -> Range<usize> {
        self.cells.clone()
    }

    /// The slot that shows it.
    pub(crate) fn slot(&self) -> &Slot {
        &self.slot
    }

    /// The slot that shows it, to change.
    pub(crate) fn slot_mut(&mut self) -> &mut Slot {
        &mut self.slot
    }

    /// The slot that shows it, as the run leaves the partition.
    pub(crate) fn into_slot(self) -> Slot {
        self.slot
    }

    /// The content hash of its cells, whose hashes are `hashes` (one per
    /// partition cell): FNV-1a 64 of theirs in order.
    pub(crate) fn hash(&self, hashes: &[u64]) -> u64 {
        self.cells.clone().fold(FNV, |h, c| fnv(h, hashes[c]))
    }
}

/// The runs of a partition `across` cells wide whose cells' device rects
/// are `rects`, plot-area `owner`s and dependency sets `deps`: in each
/// row, the cells no plot area holds, adjacent ones with the same set
/// merged from the left while the merged RGBA stays under `BAND_BYTES`.
pub(crate) fn runs(
    rects: &[[i32; 4]],
    across: usize,
    owner: &[Option<usize>],
    deps: &[Vec<Dep>],
) -> Vec<Range<usize>> {
    let mut runs: Vec<Range<usize>> = Vec::new();
    for c in 0..rects.len() {
        if owner[c].is_some() {
            continue;
        }
        let joins = runs.last().is_some_and(|run| {
            let first = run.start;
            let adjacent = run.end == c && c % across != 0;
            let [x0, y0, ..] = rects[first];
            let [.., x1, y1] = rects[c];
            adjacent
                && deps[first] == deps[c]
                && slot::pixels([x0, y0, x1, y1]) * 4 < BAND_BYTES as u64
        });
        match runs.last_mut() {
            Some(run) if joins => run.end = c + 1,
            Some(_) | None => runs.push(c..c + 1),
        }
    }
    runs
}

/// The dependency set of each cell of `cells`, cut from the chrome `list`
/// of `resolved`: the handles of the axes whose ticks and labels reach
/// into it, and the legends that do, sorted, each once. Spines, borders,
/// titles and the background change with the layout alone.
pub(crate) fn deps(cells: &list::Cells, list: &List, resolved: &Resolved) -> Vec<Vec<Dep>> {
    cells
        .cells
        .iter()
        .map(|cell| {
            let mut deps: Vec<Dep> = cell
                .parts
                .iter()
                .filter_map(|&index| {
                    let part = list.parts.get(usize::try_from(index).ok()?)?;
                    let (role, panel, axis) = layer::decode(part.id);
                    match role {
                        layer::AXIS => Some(Dep::Handle(resolved.get_axis(panel, axis)?.1)),
                        layer::LEGEND => Some(Dep::Part(part.id)),
                        _ => None,
                    }
                })
                .collect();
            deps.sort_unstable();
            deps.dedup();
            deps
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Dep, runs};
    use crate::plot::BAND_BYTES;

    /// A row of `widths` px cells 40 px tall, side by side from x = 0.
    fn row(widths: &[i32]) -> Vec<[i32; 4]> {
        let mut x = 0;
        widths
            .iter()
            .map(|&w| {
                x += w;
                [x - w, 0, x, 40]
            })
            .collect()
    }

    #[test]
    fn cells_with_the_same_handles_merge_along_a_row() {
        // Corner, axis strip, corner, the next panel's strip, corner; the
        // second panel's cell is a plot area's.
        let rects = row(&[50, 400, 30, 400, 20]);
        let owner = [None, None, None, Some(1), None];
        let x = vec![Dep::Handle(0)];
        let deps = [x.clone(), x.clone(), x.clone(), x.clone(), Vec::new()];
        assert_eq!(runs(&rects, 5, &owner, &deps), [0..3, 4..5]);
        // Another handle breaks the run.
        let deps = [x.clone(), vec![Dep::Handle(2)], x.clone(), x, Vec::new()];
        assert_eq!(runs(&rects, 5, &owner, &deps), [0..1, 1..2, 2..3, 4..5]);
    }

    #[test]
    fn runs_stay_in_their_row_and_under_band_bytes() {
        // Two rows of two cells: the second row starts a new run.
        let mut rects = row(&[100, 100]);
        rects.extend(
            row(&[100, 100])
                .into_iter()
                .map(|[x0, _, x1, _]| [x0, 40, x1, 80]),
        );
        let none = vec![Vec::new(); 4];
        assert_eq!(runs(&rects, 2, &[None; 4], &none), [0..2, 2..4]);
        // Cells 4,000 px wide and 40 tall hold 640,000 bytes: three merge
        // (1,920,000 < 2 MiB = 2,097,152), the fourth starts a new run.
        let wide = row(&[4_000; 5]);
        assert_eq!(BAND_BYTES, 2_097_152);
        assert_eq!(
            runs(&wide, 5, &[None; 5], &vec![Vec::new(); 5]),
            [0..3, 3..5]
        );
    }
}
