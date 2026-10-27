//! A figure's decorations into a drawing list: backgrounds, borders, axes
//! and titles, each a part.

use kurbo::{Point, Rect, Shape};

use crate::figure::part::{self, Role};
use crate::figure::{Figure, Resolved};
use crate::list::{self, Command, List, Part};
use crate::text::{self, Anchor, Key, Shaped, anchor};

/// Tolerance of a rectangle's path: exact, rectangles have no curves.
const TOLERANCE: f64 = 0.1;

impl Figure<'_> {
    /// Appends the decorations of `resolved` to `list`, each wrapped in a
    /// part with bounds `Rect::ZERO` and hash 0: the figure background,
    /// then per panel its background, border, axes and title, then the
    /// figure title, centred on the plot areas' extent.
    pub(super) fn decorate(&self, resolved: &Resolved, text: &text::Cache, list: &mut List) {
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
            if let Some(color) = panel.frame() {
                part(list, part::id(Role::Border, i, 0), |list| {
                    list.commands.push(Command::Stroke {
                        path: area.to_path(TOLERANCE),
                        style: list::stroke(theme.axis_width()),
                        paint: list::brush(color),
                    });
                });
            }
            for (j, (labels, positions)) in laid.axes.iter().enumerate() {
                part(list, part::id(Role::Axis, i, j), |list| {
                    labels.draw(positions, *area, text, theme, list);
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
