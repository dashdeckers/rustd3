//! [`Layout`]: a figure laid out in passes. Tick counts follow axis
//! lengths, labels follow the counts and lengths follow the labels, so
//! layout repeats until the counts settle, at most four times.

use kurbo::{Point, Rect, Size};
use rustd3_core::axis::Orient;

use crate::axis::{self, Labels, Placed};
use crate::figure::{Figure, Resolved};
use crate::layout::slots::Slots;
use crate::layout::{self, INSET, PAD, Plan, Reach, Snap, collide, plan};
use crate::legend::{Block, Content, Inset, Slot};
use crate::panel::Panel;
use crate::text::{self, Key, Shaped, Text, anchor};
use crate::theme::Theme;

/// Passes at most; counts that have not settled by then are kept.
const PASSES: usize = 4;

/// A figure laid out for one size: every panel's plot area and, per panel,
/// its axes as placed, its title and its inset legends; and the legends in
/// figure slots.
pub(crate) struct Layout {
    /// Plot areas in list px, in panel order.
    pub(crate) areas: Vec<Rect>,
    /// The panels as laid out, in panel order.
    pub(crate) panels: Vec<Laid>,
    /// The legends in figure slots and their top-left corners, in the
    /// order they were added.
    pub(crate) legends: Vec<(Block, Point)>,
    /// Passes run: 1 to 4.
    pub(crate) passes: usize,
    /// What the last placement's tracks needed (reaches and gaps) and had
    /// (the length they fill), px, across the columns and down the rows;
    /// 0 and 0 for held areas.
    pub(crate) room: [[f64; 2]; 2],
}

/// One panel as laid out.
pub(crate) struct Laid {
    /// Its axes, in insertion order.
    pub(crate) axes: Vec<Placed>,
    /// The title's key.
    pub(crate) title: Option<Key>,
    /// How far the decorations on top reach, the title aside.
    pub(crate) top: f64,
    /// Its inset legends, framed, and their top-left corners, in the order
    /// they were added.
    pub(crate) insets: Vec<(Block, Point)>,
}

/// One pass's axes: per panel, per axis, its labels and plan.
type Axes = Vec<Vec<(Labels, Plan)>>;

/// A placement: the plot areas, and what the tracks needed and had across
/// the columns and down the rows ([`Layout::room`]).
type Placement = (Vec<Rect>, [[f64; 2]; 2]);

impl Layout {
    /// `figure` laid out for `size` over the scales of `resolved`, its
    /// labels measured through `text` with `extra` (the marks' labels) in
    /// each pass's batch; with `held` plot areas, laid out in those.
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
    /// below the figure title's ink height and 8 px, inside what the
    /// legends in figure slots reserve. Passes stop when the
    /// counts equal the last pass's. Held areas are kept as they are: one
    /// pass takes the counts from their lengths and measures the labels,
    /// which may then reach past the old reaches. Every labelled vertical
    /// axis of a figure drawn for the screen reserves twice the width of
    /// "0" at tick-label size past its decorations, so labels can grow
    /// during a gesture.
    pub(crate) fn new(
        figure: &Figure<'_>,
        resolved: &Resolved,
        text: &mut text::Cache,
        extra: &[Key],
        size: Size,
        held: Option<&[Rect]>,
    ) -> Layout {
        let theme = figure.get_theme();
        let panels = figure.panels();
        let titles: Vec<Option<Key>> = panels
            .iter()
            .map(|p| p.heading().map(|t| Key::new(t, &theme.title())))
            .collect();
        let title = figure.heading().map(|t| Key::new(t, &theme.figure_title()));
        let zero = resolved.target().interactive().then(|| {
            let style = theme.tick_label(anchor::Horizontal::Middle, theme.get_ink());
            Key::new(&Text::from("0"), &style)
        });
        let roles = plan::roles(panels);
        let slotted: Vec<(Slot, Content)> = figure
            .legends()
            .iter()
            .map(|(slot, l)| (*slot, Content::new(&**l, resolved, theme)))
            .collect();
        let insets: Vec<Vec<(Inset, Content)>> = panels
            .iter()
            .map(|p| {
                p.insets()
                    .iter()
                    .map(|(at, l)| (*at, Content::new(&**l, resolved, theme)))
                    .collect()
            })
            .collect();
        let legend_keys: Vec<Key> = slotted
            .iter()
            .map(|(_, c)| c)
            .chain(insets.iter().flatten().map(|(_, c)| c))
            .flat_map(Content::keys)
            .cloned()
            .collect();
        let snap = Snap::of(resolved.target());
        let (mut areas, mut room) = match held {
            Some(rects) => (rects.to_vec(), [[0.0; 2]; 2]),
            None => {
                let inner = inner(size, theme.em(), [0.0; 4]);
                place(figure, inner, &bare(figure, &roles), snap)
            }
        };
        let mut last: Option<Vec<Vec<usize>>> = None;
        let mut axes: Axes = Vec::new();
        let mut reaches: Vec<(Reach, f64)> = Vec::new();
        let mut passes = 0;
        for pass in 1..=PASSES {
            passes = pass;
            let counts = counts(figure, &areas);
            let labels: Vec<Vec<Labels>> = panels
                .iter()
                .zip(&roles)
                .zip(&counts)
                .map(|((p, roles), counts)| {
                    p.axes()
                        .iter()
                        .zip(roles)
                        .zip(counts)
                        .map(|((a, role), &count)| {
                            let labeled = role.shown && role.labeled;
                            Labels::new(&**a, resolved, theme, text, labeled, count)
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
                    .chain(&zero)
                    .chain(extra)
                    .chain(&legend_keys),
            );
            let text = &*text;
            let slack = zero
                .as_ref()
                .and_then(|key| get(text, key))
                .map_or(0.0, |s| 2.0 * s.width());
            axes = panels
                .iter()
                .zip(labels)
                .zip(roles.iter().zip(&counts))
                .map(|((p, labels), (roles, counts))| {
                    let thicknesses: Vec<f64> = p
                        .axes()
                        .iter()
                        .zip(&labels)
                        .zip(roles)
                        .map(|((a, l), role)| {
                            let vertical = !axis::across(a.orient());
                            let roomy = role.shown && role.labeled && vertical;
                            l.thickness(text, theme) + if roomy { slack } else { 0.0 }
                        })
                        .collect();
                    let plans = plan::stack(p.axes(), roles, counts, &thicknesses);
                    labels.into_iter().zip(plans).collect()
                })
                .collect();
            reaches = panels
                .iter()
                .zip(&axes)
                .zip(&areas)
                .map(|((p, axes), area)| reach(p, axes, *area, resolved, text, theme))
                .collect();
            if held.is_some() {
                break;
            }
            let titled: Vec<Reach> = reaches
                .iter()
                .zip(&titles)
                .map(|((reach, _), title)| {
                    let mut reach = *reach;
                    if let Some(s) = title.as_ref().and_then(|key| get(text, key)) {
                        reach.top += theme.title_pad() + ink(s);
                    }
                    reach
                })
                .collect();
            let reserve = reserve(&slots(&slotted, text, theme), title.as_ref(), text, theme);
            (areas, room) = place(figure, inner(size, theme.em(), reserve), &titled, snap);
            if last.as_ref() == Some(&counts) {
                break;
            }
            last = Some(counts);
        }
        let text = &*text;
        let em = theme.em();
        let slots = slots(&slotted, text, theme);
        let grid = inner(size, em, reserve(&slots, title.as_ref(), text, theme));
        let extent = areas.iter().copied().reduce(|a, b| a.union(b));
        let legends = slots.place(grid, extent.unwrap_or(grid), em);
        let mut panels: Vec<Laid> = panels
            .iter()
            .zip(axes)
            .zip(titles)
            .zip(reaches.iter().zip(&areas))
            .zip(insets)
            .map(|((((p, axes), title), ((reach, above), area)), insets)| {
                let axes = p
                    .axes()
                    .iter()
                    .zip(axes)
                    .map(|(a, (labels, plan))| {
                        let length = length(a.orient(), *area);
                        let positions = a.positions(resolved, length, plan.count);
                        let mut placed = Placed::new(labels, plan, positions, *area, text, theme);
                        placed.above = *above;
                        placed
                    })
                    .collect();
                let insets = insets
                    .into_iter()
                    .map(|(at, content)| {
                        let block = content.column(text, theme).framed(em);
                        let corner = at.place(*area, block.size(), INSET * em);
                        (block, corner)
                    })
                    .collect();
                Laid {
                    axes,
                    title,
                    top: reach.top,
                    insets,
                }
            })
            .collect();
        keep(&mut panels, &areas);
        Layout {
            areas,
            panels,
            legends,
            passes,
            room,
        }
    }
}

/// How far each panel of `figure` reaches with every drawn axis at its
/// offset and tick alone, nothing measured: the first placement's reaches.
fn bare(figure: &Figure<'_>, roles: &[Vec<plan::Role>]) -> Vec<Reach> {
    let tick = figure.get_theme().tick();
    figure
        .panels()
        .iter()
        .zip(roles)
        .map(|(p, roles)| {
            let counts = vec![2; roles.len()];
            let thicknesses = vec![tick; roles.len()];
            let plans = plan::stack(p.axes(), roles, &counts, &thicknesses);
            let mut reach = Reach::default();
            for (a, plan) in p.axes().iter().zip(plans).filter(|(_, plan)| plan.shown) {
                reach.widen(a.orient(), plan.offset + plan.thickness);
            }
            reach
        })
        .collect()
}

/// Runs the collision passes over the tick labels of `panels`, laid out in
/// plot areas `areas`, and keeps their verdicts in each axis.
fn keep(panels: &mut [Laid], areas: &[Rect]) {
    let candidates: Vec<collide::Axis<'_>> = panels
        .iter()
        .zip(areas)
        .flat_map(|(laid, area)| {
            laid.axes.iter().map(move |a| collide::Axis {
                area: area.area(),
                boxes: &a.boxes,
                indices: a.labels.indices(),
            })
        })
        .collect();
    let verdicts = collide::collide(&candidates);
    let axes = panels.iter_mut().flat_map(|laid| laid.axes.iter_mut());
    for (axis, keep) in axes.zip(verdicts) {
        axis.keep = keep;
    }
}

/// Each panel's axes' tick counts for plot areas `areas`: `.ticks(n)`, or
/// from the axis's length.
fn counts(figure: &Figure<'_>, areas: &[Rect]) -> Vec<Vec<usize>> {
    figure
        .panels()
        .iter()
        .zip(areas)
        .map(|(p, area)| {
            p.axes()
                .iter()
                .map(|a| {
                    a.count()
                        .unwrap_or_else(|| plan::count(a.orient(), length(a.orient(), *area)))
                })
                .collect()
        })
        .collect()
}

/// How far `panel`'s drawn `axes` reach past its plot area `area`: their
/// offsets and thicknesses, their tick labels that overhang its ends, and
/// above all that the offset or multiplier labels of its vertical axes;
/// and how far the top reaches without those labels.
fn reach(
    panel: &Panel<'_>,
    axes: &[(Labels, Plan)],
    area: Rect,
    resolved: &Resolved,
    text: &text::Cache,
    theme: &Theme,
) -> (Reach, f64) {
    let mut reach = Reach::default();
    let shown = || {
        panel
            .axes()
            .iter()
            .zip(axes)
            .filter(|(_, (_, plan))| plan.shown)
    };
    for (a, (labels, plan)) in shown() {
        let orient = a.orient();
        reach.widen(orient, plan.offset + plan.thickness);
        let length = length(orient, area);
        let positions = a.positions(resolved, length, plan.count);
        reach.overhang(orient, labels.overhang(&positions, length, text));
    }
    let top = reach.top;
    for (_, (labels, _)) in shown().filter(|(a, _)| !axis::across(a.orient())) {
        reach.widen(Orient::Top, top + labels.raise(text, theme));
    }
    (reach, top)
}

/// The legends of `slotted` laid out for their slots: rows on top and
/// bottom, columns on the sides.
fn slots(slotted: &[(Slot, Content)], text: &text::Cache, theme: &Theme) -> Slots {
    Slots::new(
        slotted
            .iter()
            .map(|(slot, content)| {
                let block = match slot {
                    Slot::Top | Slot::Bottom => content.row(text, theme),
                    Slot::Left | Slot::Right => content.column(text, theme),
                };
                (*slot, block)
            })
            .collect(),
    )
}

/// What the figure title (its ink height and 8 px, on top) and the legends
/// in `slots` reserve outside the grid, px, as [top, right, bottom, left].
fn reserve(slots: &Slots, title: Option<&Key>, text: &text::Cache, theme: &Theme) -> [f64; 4] {
    let mut reserve = slots.reserve(theme.em());
    reserve[0] += title
        .and_then(|key| get(text, key))
        .map_or(0.0, |s| ink(s) + theme.title_pad());
    reserve
}

/// The box the grid's tracks fill on a canvas of `size`: inside 0.8 em of
/// padding and `reserve` px ([top, right, bottom, left]) beyond it.
fn inner(size: Size, em: f64, [top, right, bottom, left]: [f64; 4]) -> Rect {
    let pad = PAD * em;
    Rect::new(
        pad + left,
        pad + top,
        size.width - pad - right,
        size.height - pad - bottom,
    )
}

/// The plot areas of `figure`'s panels with the tracks filling `inner` and
/// each panel reaching `reaches` past its plot area, their edges snapped by
/// `snap`. A panel's left reach counts against its first column, its right
/// reach against its last, and likewise for rows; its plot area runs from
/// its first track's start to its last track's end. Edges are snapped one
/// by one, so panels that share an edge still share it.
fn place(figure: &Figure<'_>, inner: Rect, reaches: &[Reach], snap: Snap) -> Placement {
    let theme = figure.get_theme();
    let grid = figure.grid();
    let panels = figure.panels();
    let em = theme.em();
    let rows = tracks(
        grid.row_weights().len(),
        panels.iter().map(|p| p.cell().rows().end),
    );
    let cols = tracks(
        grid.col_weights().len(),
        panels.iter().map(|p| p.cell().cols().end),
    );
    let mut row_reach = vec![[0.0_f64; 2]; rows];
    let mut col_reach = vec![[0.0_f64; 2]; cols];
    for (panel, reach) in panels.iter().zip(reaches) {
        let (r, c) = (panel.cell().rows(), panel.cell().cols());
        row_reach[r.start][0] = row_reach[r.start][0].max(reach.top);
        row_reach[r.end - 1][1] = row_reach[r.end - 1][1].max(reach.bottom);
        col_reach[c.start][0] = col_reach[c.start][0].max(reach.left);
        col_reach[c.end - 1][1] = col_reach[c.end - 1][1].max(reach.right);
    }
    let (row_gaps, col_gaps) = (grid.row_gaps_px(rows, em), grid.col_gaps_px(cols, em));
    let ys = layout::tracks(
        grid.row_weights(),
        &row_gaps,
        &row_reach,
        [inner.y0, inner.height()],
    );
    let xs = layout::tracks(
        grid.col_weights(),
        &col_gaps,
        &col_reach,
        [inner.x0, inner.width()],
    );
    let fixed = |reach: &[[f64; 2]], gaps: &[f64]| {
        reach.iter().map(|[low, high]| low + high).sum::<f64>() + gaps.iter().sum::<f64>()
    };
    let room = [
        [fixed(&col_reach, &col_gaps), inner.width()],
        [fixed(&row_reach, &row_gaps), inner.height()],
    ];
    let span = |tracks: &[[f64; 2]], range: std::ops::Range<usize>| {
        let ([start, _], [last, length]) = (tracks[range.start], tracks[range.end - 1]);
        [start, last + length]
    };
    let areas = panels
        .iter()
        .map(|p| {
            let [y0, y1] = span(&ys, p.cell().rows());
            let [x0, x1] = span(&xs, p.cell().cols());
            snap.rect(Rect::new(x0, y0, x1, y1))
        })
        .collect();
    (areas, room)
}

/// The number of tracks along one axis: as many as there are weights, or
/// as the panels' last tracks need.
fn tracks(weights: usize, ends: impl Iterator<Item = usize>) -> usize {
    ends.fold(weights, usize::max)
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
