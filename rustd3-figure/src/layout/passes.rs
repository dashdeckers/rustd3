//! [`Layout`]: a figure laid out in passes. Tick counts follow axis
//! lengths, labels follow the counts and lengths follow the labels, so
//! layout repeats until the counts settle, at most four times.

use kurbo::{Point, Rect, Size};
use rustd3_core::axis::Orient;

use crate::axis::{self, Labels, Placed};
use crate::figure::{Figure, Resolved};
use crate::layout::slots::Slots;
use crate::layout::{self, INSET, PAD, Plan, Reach, collide, plan};
use crate::legend::{Block, Content, Inset, Slot};
use crate::mark::Snap;
use crate::panel::Panel;
use crate::target::Target;
use crate::text::{self, Key, Shaped, Text, anchor};
use crate::theme::Theme;

/// The most layout passes. Counts that have not settled by then are kept.
const PASSES: usize = 4;

/// A figure laid out for one size.
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
    /// (the length they fill), in px, across the columns and then down the
    /// rows. Both are 0 for held areas.
    pub(crate) room: [[f64; 2]; 2],
}

/// One panel as laid out.
pub(crate) struct Laid {
    /// Its axes, in insertion order.
    pub(crate) axes: Vec<Placed>,
    /// The title's key.
    pub(crate) title: Option<Key>,
    /// How far above the plot area the title's last baseline sits: the
    /// same for every panel whose cell starts in its grid row.
    pub(crate) heading: f64,
    /// Its inset legends, framed, and their top-left corners, in the order
    /// they were added.
    pub(crate) insets: Vec<(Block, Point)>,
}

/// One pass's axes: per panel, per axis, its labels and plan.
type Axes = Vec<Vec<(Labels, Plan)>>;

/// One pass's tick counts: per panel and axis, then per guide in a figure
/// slot or a panel corner (`None` for legends).
type Counts = (Vec<Vec<usize>>, Vec<Option<usize>>);

/// A placement: the plot areas, and what the tracks needed and had across
/// the columns and down the rows ([`Layout::room`]).
type Placement = (Vec<Rect>, [[f64; 2]; 2]);

impl Layout {
    /// `figure` laid out for `size` over the scales of `resolved`. Labels
    /// are measured through `text`, with `extra` (the marks' labels) in
    /// each pass's batch. With `held` plot areas, it is laid out in those.
    ///
    /// The first placement has every axis at its offset and tick alone.
    /// Each pass then takes each axis's tick count from its length in the
    /// last placement (or `.ticks(n)`), measures the new labels in one
    /// batch, and places the plot areas again. Each panel reaches past its
    /// plot area by:
    /// - its axes' offsets and thicknesses;
    /// - tick labels that overhang its ends, at their positions in the last
    ///   placement;
    /// - its title ([`headings`]). Its last baseline sits the title pad and
    ///   the text face's descender above the top-side axes of its grid row,
    ///   and at least the title pad and its ink height above the panel's
    ///   own top reach.
    ///
    /// Each column takes the largest reach of its panels per side, and each
    /// row likewise. The rest is split by weight inside 0.8 em of padding,
    /// below the figure title's ink height and 8 px, and inside what the
    /// legends in figure slots reserve. Passes stop when the counts equal
    /// the last pass's.
    ///
    /// Held areas are kept as they are. One pass takes the counts from
    /// their lengths and measures the labels, which may then reach past the
    /// old reaches.
    ///
    /// Every labeled vertical axis reserves the figure's axis room, in
    /// widths of "0" at tick-label size, past its decorations, so labels
    /// can grow during a gesture (`Figure::axis_room`).
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
        let title = figure
            .get_title()
            .map(|t| Key::new(t, &theme.figure_title()));
        let spare = figure.room(resolved.target());
        let zero = (spare > 0.0).then(|| {
            let style = theme.tick_label(anchor::Horizontal::Middle, theme.get_ink());
            Key::new(&Text::from("0"), &style)
        });
        let roles = plan::roles(panels);
        let mut slotted: Vec<(Slot, Content)> = figure
            .legends()
            .iter()
            .map(|(slot, g)| (*slot, Content::new(g, resolved, theme)))
            .collect();
        let mut insets: Vec<Vec<(Inset, Content)>> = panels
            .iter()
            .map(|p| {
                p.insets()
                    .iter()
                    .map(|(at, g)| (*at, Content::new(g, resolved, theme)))
                    .collect()
            })
            .collect();
        let snap = Snap::of(resolved.target());
        let (mut areas, mut room) = match held {
            Some(rects) => (rects.to_vec(), [[0.0; 2]; 2]),
            None => first(figure, size, snap),
        };
        let mut last: Option<Counts> = None;
        let mut axes: Axes = Vec::new();
        let mut reaches: Vec<(Reach, f64)> = Vec::new();
        let mut passes = 0;
        for pass in 1..=PASSES {
            passes = pass;
            let counts = counts(figure, &areas);
            let extent = union(&areas).unwrap_or_else(|| inner(size, theme.em(), [0.0; 4]));
            // The colorbars' tick counts and labels for this pass's lengths.
            let bars: Vec<Option<usize>> = {
                let text = &*text;
                let slots = slotted
                    .iter_mut()
                    .map(|(slot, c)| c.measure_slot(*slot, extent, resolved, theme, text));
                let corners = insets.iter_mut().zip(&areas).flat_map(|(insets, area)| {
                    insets
                        .iter_mut()
                        .map(move |(_, c)| c.measure_inset(*area, resolved, theme, text))
                });
                slots.chain(corners).collect()
            };
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
                            let shown = [role.labeled, role.titled].map(|on| role.shown && on);
                            Labels::new(&**a, resolved, theme, text, shown, count)
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
                    .chain(slotted.iter().flat_map(|(_, c)| c.keys()))
                    .chain(insets.iter().flatten().flat_map(|(_, c)| c.keys())),
            );
            let text = &*text;
            let slack = zero
                .as_ref()
                .and_then(|key| get(text, key))
                .map_or(0.0, |s| spare * s.get_width());
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
                .zip(headings(panels, &axes, text, theme))
                .map(|(((reach, _), title), heading)| {
                    let mut reach = *reach;
                    if let Some(s) = title.as_ref().and_then(|key| get(text, key)) {
                        let above = reach.top + theme.title_pad() + ink(s);
                        reach.top = above.max(heading + s.get_last() + s.get_ascent());
                    }
                    reach
                })
                .collect();
            let slots = slots(&slotted, extent, text, theme);
            let reserve = reserve(&slots, title.as_ref(), text, theme);
            (areas, room) = place(figure, inner(size, theme.em(), reserve), &titled, snap);
            let settled = (counts, bars);
            if last.as_ref() == Some(&settled) {
                break;
            }
            last = Some(settled);
        }
        let text = &*text;
        let em = theme.em();
        let extent = union(&areas).unwrap_or_else(|| inner(size, em, [0.0; 4]));
        let slots = slots(&slotted, extent, text, theme);
        let grid = inner(size, em, reserve(&slots, title.as_ref(), text, theme));
        let legends = slots.place(grid, union(&areas).unwrap_or(grid), em);
        let headings = headings(panels, &axes, text, theme);
        let mut panels: Vec<Laid> = panels
            .iter()
            .zip(axes)
            .zip(titles.into_iter().zip(headings))
            .zip(reaches.iter().zip(&areas))
            .zip(insets)
            .map(
                |((((p, axes), (title, heading)), ((_, above), area)), insets)| {
                    let axes = p
                        .axes()
                        .iter()
                        .zip(axes)
                        .map(|(a, (labels, plan))| {
                            let length = length(a.orient(), *area);
                            let positions = a.positions(resolved, length, plan.count);
                            let minors = a.minors(resolved, length, plan.count);
                            let mut placed =
                                Placed::new(labels, plan, positions, *area, text, theme);
                            placed.minors = minors;
                            placed.above = *above;
                            placed
                        })
                        .collect();
                    let mut stacked: Vec<(Inset, f64)> = Vec::new();
                    let insets = insets
                        .into_iter()
                        .map(|(at, content)| {
                            let block = content.inset(*area, text, theme);
                            let inward = stacked
                                .iter()
                                .filter(|(c, _)| *c == at)
                                .map(|(_, h)| h + INSET * em)
                                .sum();
                            let gap = INSET * content.em(theme);
                            let corner = at.place(*area, block.size(), gap, inward);
                            stacked.push((at, block.size().height));
                            (block, corner)
                        })
                        .collect();
                    Laid {
                        axes,
                        title,
                        heading,
                        insets,
                    }
                },
            )
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

/// Each panel's axes' tick counts in layout's first pass for `size` and
/// `target`: `.ticks(n)`, or from the axis's length in the first placement
/// (every axis at its offset and tick alone, nothing measured), or in
/// `held` plot areas.
pub(crate) fn first_counts(
    figure: &Figure<'_>,
    size: Size,
    target: Target,
    held: Option<&[Rect]>,
) -> Vec<Vec<usize>> {
    match held {
        Some(rects) => counts(figure, rects),
        None => counts(figure, &first(figure, size, Snap::of(target)).0),
    }
}

/// The first placement of `figure` on a canvas of `size`: every drawn axis
/// at its offset and tick alone, nothing measured, no title or legend.
fn first(figure: &Figure<'_>, size: Size, snap: Snap) -> Placement {
    let roles = plan::roles(figure.panels());
    let inner = inner(size, figure.get_theme().em(), [0.0; 4]);
    place(figure, inner, &bare(figure, &roles), snap)
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

/// How far above each of `panels`' plot areas its title's last baseline
/// sits, with `axes` placed. It is the title pad and the text face's
/// descender at the title size, above the largest reach of the shown
/// top-side axes among the panels whose cells start in its grid row. Only
/// top-side axes count, as matplotlib moves a title up for those alone.
/// The titles of a row thus share one baseline, whatever their descenders
/// and the overhangs of side axes.
fn headings(panels: &[Panel<'_>], axes: &Axes, text: &text::Cache, theme: &Theme) -> Vec<f64> {
    let row = |p: &Panel<'_>| p.cell().rows().start;
    let mut tops = vec![0.0_f64; panels.iter().map(|p| row(p) + 1).max().unwrap_or(0)];
    for (p, axes) in panels.iter().zip(axes) {
        let top = p
            .axes()
            .iter()
            .zip(axes)
            .filter(|(a, (_, plan))| plan.shown && a.orient() == Orient::Top)
            .map(|(_, (_, plan))| plan.offset + plan.thickness)
            .fold(0.0, f64::max);
        tops[row(p)] = tops[row(p)].max(top);
    }
    let [_, descender] = text.fonts().line(theme.label_text());
    panels
        .iter()
        .map(|p| tops[row(p)] + theme.title_pad() + descender)
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
                restarts: a.labels.restarts(),
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

/// How far `panel`'s drawn `axes` reach past its plot area `area`, and how
/// far the top reaches without the offset or multiplier labels. The reach
/// covers the axes' offsets and thicknesses and their tick labels that
/// overhang its ends. On a horizontal axis of a `Fit::Window` handle, the
/// overhang is the most any label would overhang at either end. Above all
/// that sit the offset or multiplier labels of its vertical axes.
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
        let overhang = if axis::across(orient) && resolved.windowed(a.handle()) {
            labels.reserve(text)
        } else {
            let length = length(orient, area);
            let positions = a.positions(resolved, length, plan.count);
            labels.overhang(&positions, length, text)
        };
        reach.overhang(orient, overhang);
    }
    let top = reach.top;
    for (_, (labels, _)) in shown().filter(|(a, _)| !axis::across(a.orient())) {
        reach.widen(Orient::Top, top + labels.raise(text, theme));
    }
    (reach, top)
}

/// The guides of `slotted` laid out for their slots beside plot areas
/// whose union is `extent` ([`Content::slotted`]).
fn slots(slotted: &[(Slot, Content)], extent: Rect, text: &text::Cache, theme: &Theme) -> Slots {
    Slots::new(
        slotted
            .iter()
            .map(|(slot, content)| (*slot, content.slotted(*slot, extent, text, theme)))
            .collect(),
    )
}

/// The union of plot areas `areas`; `None` without panels.
fn union(areas: &[Rect]) -> Option<Rect> {
    areas.iter().copied().reduce(|a, b| a.union(b))
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
    // A span whose tracks negative gaps or reaches pull back over each
    // other is empty, never inverted.
    let span = |tracks: &[[f64; 2]], range: std::ops::Range<usize>| {
        let ([start, _], [last, length]) = (tracks[range.start], tracks[range.end - 1]);
        [start, (last + length).max(start)]
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
    shaped.get_ascent() + shaped.get_descent()
}

/// The length of `area` along an axis on side `orient`.
fn length(orient: Orient, area: Rect) -> f64 {
    if axis::across(orient) {
        area.width()
    } else {
        area.height()
    }
}
