//! A figure into a drawing list: backgrounds, marks, borders, axes and
//! titles, each a part.

use std::cell::RefCell;

use kurbo::{Affine, Point, Rect, Shape};

use crate::figure::part::{self, Role};
use crate::figure::{Figure, Resolved};
use crate::list::{self, Clip, Command, List, Part};
use crate::mark::Ctx;
use crate::panel::Panel;
use crate::text::{self, Anchor, Key, Shaped, anchor};
use crate::thin;

/// Tolerance of a rectangle's path: exact, rectangles have no curves.
const TOLERANCE: f64 = 0.1;

impl Figure<'_> {
    /// Appends the figure as `resolved` lays it out to `list`, each element
    /// wrapped in a part with bounds `Rect::ZERO` and hash 0: the figure
    /// background, then per panel its background, its marks in insertion
    /// order inside one group translated to the plot area and clipped to
    /// it, its inset legends, its border, axes and title, then the figure
    /// title, centred on the plot areas' extent, and the legends in figure
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
        part(list, part::id(Role::Background, 0, 0), |list| {
            list.commands.push(Command::Fill {
                path: size.to_rect().to_path(TOLERANCE),
                paint: list::brush(theme.background()),
            });
        });
        let areas = resolved.plot_areas();
        for (i, ((panel, laid), area)) in self
            .panels
            .iter()
            .zip(resolved.panels())
            .zip(areas)
            .enumerate()
        {
            part(list, part::id(Role::Panel, i, 0), |list| {
                list.commands.push(Command::Fill {
                    path: area.to_path(TOLERANCE),
                    paint: list::brush(theme.panel()),
                });
            });
            let ctx = Ctx::new(resolved, theme, *area, text, frame);
            marks(list, i, panel, &ctx);
            for (j, (block, at)) in laid.insets.iter().enumerate() {
                part(list, part::id(Role::Legend, i, j), |list| {
                    block.draw(*at, text, theme, list);
                });
            }
            if let Some(color) = panel.frame() {
                part(list, part::id(Role::Border, i, 0), |list| {
                    list.commands.push(Command::Stroke {
                        path: area.to_path(TOLERANCE),
                        style: list::stroke(theme.axis_width()),
                        paint: list::brush(color),
                    });
                });
            }
            let shown = laid.axes.iter().enumerate().filter(|(_, a)| a.plan.shown);
            for (j, placed) in shown {
                part(list, part::id(Role::Axis, i, j), |list| {
                    placed.draw(*area, text, theme, list);
                });
            }
            if let Some(title) = shaped(text, laid.title.as_ref()) {
                let at = Point::new(area.center().x, area.y0 - laid.top - theme.title_pad());
                part(list, part::id(Role::Heading, i, 0), |list| {
                    title.place(list, at, bottom(), 0.0);
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
                block.draw(*at, text, theme, list);
            });
        }
    }
}

/// Appends the marks of `panel`, the `index`th, each a part, inside a group
/// translated to its plot area and clipped to it; nothing when no mark
/// draws.
fn marks(list: &mut List, index: usize, panel: &Panel<'_>, ctx: &Ctx<'_>) {
    let area = ctx.rect();
    let start = list.commands.len();
    list.commands.push(Command::Push {
        transform: Affine::translate(area.origin().to_vec2()),
        clip: Some(Clip::Rect(area.with_origin(Point::ORIGIN))),
    });
    for (j, mark) in panel.marks().iter().enumerate() {
        part(list, part::id(Role::Mark, index, j), |list| {
            mark.draw(ctx, list)
        });
    }
    if list.commands.len() == start + 1 {
        list.commands.truncate(start);
    } else {
        list.commands.push(Command::Pop);
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

/// The anchor at the middle of a label's bottom edge.
fn bottom() -> Anchor {
    Anchor {
        h: anchor::Horizontal::Middle,
        v: anchor::Vertical::Bottom,
    }
}
