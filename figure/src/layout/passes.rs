//! [`Layout`]: a figure laid out in passes. Tick counts follow axis
//! lengths, labels follow the counts and lengths follow the labels, so
//! layout repeats until the counts settle, at most four times.

use kurbo::{Rect, Size};
use rustd3_core::axis::Orient;

use crate::axis::{self, Labels};
use crate::figure::{Figure, Resolved};
use crate::layout::{self, Plan, Reach, plan};
use crate::text::{self, Key, Shaped};

/// Passes at most; counts that have not settled by then are kept.
const PASSES: usize = 4;

/// A figure laid out for one size: every panel's plot area and, per panel,
/// its axes as placed and its title.
pub(crate) struct Layout {
    /// Plot areas in list px, in panel order.
    pub(crate) areas: Vec<Rect>,
    /// The panels as laid out, in panel order.
    pub(crate) panels: Vec<Laid>,
    /// Passes run: 1 to 4.
    pub(crate) passes: usize,
}

/// One panel as laid out.
pub(crate) struct Laid {
    /// Its axes, in insertion order.
    pub(crate) axes: Vec<Placed>,
    /// The title's key.
    pub(crate) title: Option<Key>,
    /// How far the decorations on top reach, the title aside.
    pub(crate) top: f64,
}

/// One axis as laid out.
pub(crate) struct Placed {
    /// Its labels.
    pub(crate) labels: Labels,
    /// Its plan.
    pub(crate) plan: Plan,
    /// Its ticks' positions along the plot area, aligned with the labels.
    pub(crate) positions: Vec<Option<f64>>,
}

/// One pass's axes: per panel, per axis, its labels and plan.
type Axes = Vec<Vec<(Labels, Plan)>>;

impl Layout {
    /// `figure` laid out for `size` over the scales of `resolved`, its
    /// labels measured through `text` with `extra` (the marks' labels) in
    /// each pass's batch.
    ///
    /// The first placement has every axis at its offset and tick alone.
    /// Each pass then takes each axis's tick count from its length in the
    /// last placement (or `.ticks(n)`), measures the new labels in one
    /// batch, and places the plot areas again: each panel reaches past its
    /// plot area by its axes (offset and thickness), by tick labels that
    /// overhang its ends at their positions in the last placement, and by
    /// its title (title pad and ink height, above everything on top); each
    /// column takes the largest reach of its panels per side and each row
    /// likewise, and the rest is split by weight inside 0.8 em of padding,
    /// below the figure title's ink height and 8 px. Passes stop when the
    /// counts equal the last pass's.
    pub(crate) fn new(
        figure: &Figure<'_>,
        resolved: &Resolved,
        text: &mut text::Cache,
        extra: &[Key],
        size: Size,
    ) -> Layout {
        let theme = figure.style();
        let panels = figure.panels();
        let titles: Vec<Option<Key>> = panels
            .iter()
            .map(|p| p.heading().map(|t| Key::new(t, &theme.title())))
            .collect();
        let title = figure.heading().map(|t| Key::new(t, &theme.figure_title()));
        let bare: Vec<Reach> = panels
            .iter()
            .map(|p| {
                let mut reach = Reach::default();
                for a in p.axes() {
                    reach.widen(a.orient(), a.outward().unwrap_or(0.0) + theme.tick());
                }
                reach
            })
            .collect();
        let mut areas = place(figure, size, 0.0, &bare);
        let mut last: Option<Vec<Vec<usize>>> = None;
        let mut axes: Axes = Vec::new();
        let mut reaches: Vec<Reach> = Vec::new();
        let mut passes = 0;
        for pass in 1..=PASSES {
            passes = pass;
            let counts: Vec<Vec<usize>> = panels
                .iter()
                .zip(&areas)
                .map(|(p, area)| {
                    p.axes()
                        .iter()
                        .map(|a| {
                            a.count().unwrap_or_else(|| {
                                plan::count(a.orient(), length(a.orient(), *area))
                            })
                        })
                        .collect()
                })
                .collect();
            let labels: Vec<Vec<Labels>> = panels
                .iter()
                .zip(&counts)
                .map(|(p, counts)| {
                    p.axes()
                        .iter()
                        .zip(counts)
                        .map(|(a, &count)| {
                            let labeled = a.labels().unwrap_or(true);
                            Labels::new(&**a, resolved, theme, labeled, count)
                        })
                        .collect()
                })
                .collect();
            text.prepare(
                labels
                    .iter()
                    .flatten()
                    .flat_map(Labels::keys)
                    .chain(titles.iter().flatten())
                    .chain(&title)
                    .chain(extra),
            );
            let text = &*text;
            axes = panels
                .iter()
                .zip(labels)
                .zip(&counts)
                .map(|((p, labels), counts)| {
                    p.axes()
                        .iter()
                        .zip(labels)
                        .zip(counts)
                        .map(|((a, labels), &count)| {
                            let plan = Plan {
                                shown: true,
                                labeled: a.labels().unwrap_or(true),
                                offset: a.outward().unwrap_or(0.0),
                                thickness: labels.thickness(text, theme),
                                count,
                            };
                            (labels, plan)
                        })
                        .collect()
                })
                .collect();
            reaches = panels
                .iter()
                .zip(&axes)
                .zip(&areas)
                .map(|((p, axes), area)| {
                    let mut reach = Reach::default();
                    for (a, (labels, plan)) in p.axes().iter().zip(axes) {
                        let orient = a.orient();
                        reach.widen(orient, plan.offset + plan.thickness);
                        let length = length(orient, *area);
                        let positions = a.positions(resolved, length, plan.count);
                        reach.overhang(orient, labels.overhang(&positions, length, text));
                    }
                    reach
                })
                .collect();
            let reserve = title
                .as_ref()
                .and_then(|key| get(text, key))
                .map_or(0.0, |s| ink(s) + theme.title_pad());
            let titled: Vec<Reach> = reaches
                .iter()
                .zip(&titles)
                .map(|(reach, title)| {
                    let mut reach = *reach;
                    if let Some(s) = title.as_ref().and_then(|key| get(text, key)) {
                        reach.top += theme.title_pad() + ink(s);
                    }
                    reach
                })
                .collect();
            areas = place(figure, size, reserve, &titled);
            if last.as_ref() == Some(&counts) {
                break;
            }
            last = Some(counts);
        }
        let panels = panels
            .iter()
            .zip(axes)
            .zip(titles)
            .zip(reaches.iter().zip(&areas))
            .map(|(((p, axes), title), (reach, area))| {
                let axes = p
                    .axes()
                    .iter()
                    .zip(axes)
                    .map(|(a, (labels, plan))| {
                        let length = length(a.orient(), *area);
                        let positions = a.positions(resolved, length, plan.count);
                        Placed {
                            labels,
                            plan,
                            positions,
                        }
                    })
                    .collect();
                Laid {
                    axes,
                    title,
                    top: reach.top,
                }
            })
            .collect();
        Layout {
            areas,
            panels,
            passes,
        }
    }
}

/// The plot areas of `figure`'s panels for `size`, with `reserve` px above
/// the grid for the figure title and each panel reaching `reaches` past
/// its plot area.
fn place(figure: &Figure<'_>, size: Size, reserve: f64, reaches: &[Reach]) -> Vec<Rect> {
    let theme = figure.style();
    let grid = figure.grid();
    let panels = figure.panels();
    let em = theme.em();
    let pad = 0.8 * em;
    let rows = tracks(grid.row_weights().len(), panels.iter().map(|p| p.cell().0));
    let cols = tracks(grid.col_weights().len(), panels.iter().map(|p| p.cell().1));
    let mut row_reach = vec![[0.0_f64; 2]; rows];
    let mut col_reach = vec![[0.0_f64; 2]; cols];
    for (panel, reach) in panels.iter().zip(reaches) {
        let (r, c) = panel.cell();
        let row = &mut row_reach[r];
        *row = [row[0].max(reach.top), row[1].max(reach.bottom)];
        let col = &mut col_reach[c];
        *col = [col[0].max(reach.left), col[1].max(reach.right)];
    }
    let ys = layout::tracks(
        grid.row_weights(),
        &grid.row_gaps_px(rows, em),
        &row_reach,
        [pad + reserve, size.height - 2.0 * pad - reserve],
    );
    let xs = layout::tracks(
        grid.col_weights(),
        &grid.col_gaps_px(cols, em),
        &col_reach,
        [pad, size.width - 2.0 * pad],
    );
    panels
        .iter()
        .map(|p| {
            let ([y, h], [x, w]) = (ys[p.cell().0], xs[p.cell().1]);
            Rect::new(x, y, x + w, y + h)
        })
        .collect()
}

/// The number of tracks along one axis: as many as there are weights, or
/// as the panels' cells need.
fn tracks(weights: usize, cells: impl Iterator<Item = usize>) -> usize {
    cells.map(|c| c + 1).fold(weights, usize::max)
}

/// `key`'s shaped label; `None` when typst rejected it.
fn get<'c>(text: &'c text::Cache, key: &Key) -> Option<&'c Shaped> {
    text.get(key)?.ok()
}

/// The ink height of a shaped label.
fn ink(shaped: &Shaped) -> f64 {
    shaped.ascent() + shaped.descent()
}

/// The length of `area` along an axis on side `orient`.
fn length(orient: Orient, area: Rect) -> f64 {
    if axis::across(orient) {
        area.width()
    } else {
        area.height()
    }
}
