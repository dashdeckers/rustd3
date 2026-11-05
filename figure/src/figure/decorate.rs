//! Turning a resolved figure into a drawing list, one part per element.

use std::cell::RefCell;

use kurbo::{Affine, BezPath, Point, Rect, Shape};
use rustd3_core::color::Rgb;

use crate::figure::part::{self, Role};
use crate::figure::{Figure, Resolved};
use crate::layout::Laid;
use crate::list::{self, Clip, Command, List, Part};
use crate::mark::{Ctx, Snap};
use crate::panel::Panel;
use crate::text::{self, Anchor, Key, Shaped, anchor};
use crate::thin;

/// The tolerance of a rectangle's path. Any value is exact, since
/// rectangles have no curves.
const TOLERANCE: f64 = 0.1;

impl Figure<'_> {
    /// Appends the figure, as `resolved` lays it out, to `list`. Each
    /// element is wrapped in a part with bounds `Rect::ZERO` and hash 0.
    /// The figure background comes first. Then each panel adds its
    /// background, its grid lines (in styles with a grid), its marks in
    /// insertion order inside one group translated to the plot area and
    /// clipped to it, its inset legends, its border, its axes (each its
    /// line, then its ticks and labels) and its title. Last come the figure
    /// title, centered on the plot areas' extent, and the legends in figure
    /// slots. Marks thin through `frame`.
    pub(super) fn decorate(
        &self,
        resolved: &Resolved,
        text: &text::Cache,
        frame: &RefCell<thin::Frame>,
        list: &mut List,
    ) {
        let theme = &self.theme;
        let size = list.size;
        let snap = Snap::of(resolved.target());
        part(list, part::id(Role::Background, 0, 0), |list| {
            list.commands.push(Command::Fill {
                path: size.to_rect().to_path(TOLERANCE),
                paint: list::brush(theme.get_background()),
            });
        });
        let areas = resolved.get_plot_areas();
        for (i, ((panel, laid), area)) in self
            .panels
            .iter()
            .zip(resolved.panels())
            .zip(areas)
            .enumerate()
        {
            let overscan = resolved.overscan();
            part(list, part::id(Role::Panel, i, 0), |list| {
                list.commands.push(Command::Fill {
                    path: area.inflate(overscan, overscan).to_path(TOLERANCE),
                    paint: list::brush(theme.panel()),
                });
            });
            if let Some(color) = theme.grid() {
                part(list, part::id(Role::Grid, i, 0), |list| {
                    let style = (color, theme.grid_width(), snap);
                    grid(list, laid, (*area, overscan), style);
                });
            }
            let ctx = Ctx::new(resolved, theme, *area, text, frame);
            marks(list, i, panel, &ctx);
            for (j, (block, at)) in laid.insets.iter().enumerate() {
                part(list, part::id(Role::Legend, i, j), |list| {
                    block.draw(*at, text, theme, snap, list);
                });
            }
            if let Some(color) = panel.frame() {
                let width = theme.axis_width();
                let corners = [area.origin(), Point::new(area.x1, area.y1)];
                let [a, b] = corners.map(|p| snap.point(p, width));
                part(list, part::id(Role::Border, i, 0), |list| {
                    list.commands.push(Command::Stroke {
                        path: Rect::from_points(a, b).to_path(TOLERANCE),
                        style: list::stroke(snap.width(width)),
                        paint: list::brush(color),
                    });
                });
            }
            let shown = laid.axes.iter().enumerate().filter(|(_, a)| a.plan.shown);
            for (j, placed) in shown {
                part(list, part::id(Role::Spine, i, j), |list| {
                    placed.line(*area, theme, snap, list);
                });
                part(list, part::id(Role::Axis, i, j), |list| {
                    placed.draw(*area, text, theme, snap, list);
                });
            }
            if let Some(title) = shaped(text, laid.title.as_ref()) {
                let baseline = area.y0 - laid.heading - title.get_last();
                let at = Point::new(area.center().x, baseline);
                let anchor = Anchor {
                    h: anchor::Horizontal::Middle,
                    v: anchor::Vertical::Baseline,
                };
                part(list, part::id(Role::Heading, i, 0), |list| {
                    title.place(list, at, anchor, 0.0);
                });
            }
        }
        if let Some(title) = shaped(text, resolved.title()) {
            let extent = areas.iter().copied().reduce(|a, b| a.union(b));
            let x = extent.map_or(size.width / 2.0, |e| e.center().x);
            let at = Point::new(x, 0.8 * theme.em());
            let top = Anchor {
                h: anchor::Horizontal::Middle,
                v: anchor::Vertical::Top,
            };
            part(list, part::id(Role::Title, 0, 0), |list| {
                title.place(list, at, top, 0.0);
            });
        }
        for (k, (block, at)) in resolved.legends().iter().enumerate() {
            part(list, part::id(Role::Legend, part::FIGURE, k), |list| {
                block.draw(*at, text, theme, snap, list);
            });
        }
    }
}

/// Appends the marks of `panel`, the `index`th, each a part, inside a group
/// translated to its plot area and clipped to it (grown by a widget's
/// overscan); nothing when no mark draws.
fn marks(list: &mut List, index: usize, panel: &Panel<'_>, ctx: &Ctx<'_>) {
    let area = ctx.get_rect();
    let start = list.commands.len();
    list.commands.push(Command::Push {
        transform: Affine::translate(area.origin().to_vec2()),
        clip: Some(Clip::Rect(ctx.view())),
    });
    for (j, mark) in panel.marks().iter().enumerate() {
        part(list, part::id(Role::Mark, index, j), |list| {
            mark.draw(&ctx.at(index, j), list)
        });
    }
    if list.commands.len() == start + 1 {
        list.commands.truncate(start);
    } else {
        list.commands.push(Command::Pop);
    }
}

/// Appends the grid lines of the panel laid out as `laid`, across its plot
/// area `area` and `overscan` px past its ends, in `color` and `width` px
/// wide (snapped by `snap`). There is one line per major tick of the
/// panel's first shown horizontal axis and of its first shown vertical
/// axis, as matplotlib draws one x and one y grid per axes. Nothing is
/// appended without such ticks.
fn grid(
    list: &mut List,
    laid: &Laid,
    (area, overscan): (Rect, f64),
    (color, width, snap): (Rgb, f64, Snap),
) {
    let mut path = BezPath::new();
    for horizontal in [true, false] {
        let first = laid
            .axes
            .iter()
            .find(|a| a.plan.shown && a.horizontal() == horizontal);
        if let Some(axis) = first {
            axis.grid(area, overscan, (width, snap), &mut path);
        }
    }
    if !path.elements().is_empty() {
        list.commands.push(Command::Stroke {
            path,
            style: list::stroke(snap.width(width)),
            paint: list::brush(color),
        });
    }
}

/// Runs `draw` and wraps the commands it appends in a part with id `id`;
/// no part when it appends none.
fn part(list: &mut List, id: list::part::Id, draw: impl FnOnce(&mut List)) {
    let start = list.commands.len();
    draw(list);
    let end = list.commands.len();
    if end > start {
        list.parts.push(Part {
            id,
            commands: start..end,
            bounds: Rect::ZERO,
            hash: 0,
        });
    }
}

/// The label of `key`, when there is one and typst shaped it.
fn shaped<'c>(text: &'c text::Cache, key: Option<&Key>) -> Option<&'c Shaped> {
    text.get(key?)?.ok()
}
