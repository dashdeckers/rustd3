//! [`Figure::check`]: layout, text and marks searched for issues.

use std::cell::RefCell;

use crate::axis;
use crate::check::{Direction, Element, Issue};
use crate::figure::{Figure, Resolved};
use crate::layout::{INSET, PAD};
use crate::legend::Block;
use crate::mark::Ctx;
use crate::text::{self, Key};

impl Figure<'_> {
    /// The figure resolved for `size` with a fresh cache over the bundled
    /// fonts for vector output, nothing drawn, and what it finds: grid axes
    /// with no room left for plot areas, text longer than what it labels
    /// (axis and panel titles, the figure title, legends), labels with
    /// glyphs the fonts lack or markup typst rejects, and what each mark
    /// reports ([`Mark::issues`](crate::mark::Mark::issues): collected data
    /// over 100,000 points, palette keys outside their palette). Layout's
    /// issues come first, in panel order, then text's, by label, then the
    /// marks', in panel and mark order.
    ///
    /// ```
    /// use rustd3_figure::check::{Issue, size};
    /// use rustd3_figure::Figure;
    ///
    /// let mut fig = Figure::new();
    /// fig.title("漢字").panel(0, 0);
    /// let issues = fig.check(size(640.0, 480.0));
    /// assert!(matches!(issues[..], [Issue::MissingGlyphs { count: 2, .. }]));
    /// ```
    pub fn check(&self, size: kurbo::Size) -> Vec<Issue> {
        let mut cache = crate::Cache::default();
        let held = crate::Held::default();
        let resolved = self.resolve(size, &mut cache, &held, crate::Target::Vector);
        let mut issues = room(&resolved);
        issues.extend(overflow(self, &resolved, cache.text()));
        issues.extend(labels(cache.text()));
        let (text, frame) = cache.start();
        let frame = RefCell::new(frame);
        for (i, (panel, area)) in self.panels().iter().zip(resolved.plot_areas()).enumerate() {
            let ctx = Ctx::new(&resolved, self.get_theme(), *area, text, &frame);
            for (j, mark) in panel.marks().iter().enumerate() {
                mark.issues(&ctx.at(i, j), &mut issues);
            }
        }
        cache.end(frame.into_inner());
        issues
    }
}

/// Grid axes whose reaches and gaps leave no room for plot areas.
fn room(resolved: &Resolved) -> Vec<Issue> {
    let directions = [Direction::Horizontal, Direction::Vertical];
    directions
        .into_iter()
        .zip(resolved.room())
        .filter(|(_, [needed, available])| needed >= available)
        .map(|(axis, [needed, available])| Issue::NoRoom {
            axis,
            needed,
            available,
        })
        .collect()
}

/// Text larger than its room: the figure title, then per panel its title,
/// its labelled axes' titles and its inset legends, then the legends in
/// figure slots.
fn overflow(figure: &Figure<'_>, resolved: &Resolved, text: &text::Cache) -> Vec<Issue> {
    let em = figure.get_theme().em();
    let canvas = resolved.size();
    let inside = kurbo::Size::new(
        canvas.width - 2.0 * PAD * em,
        canvas.height - 2.0 * PAD * em,
    );
    let mut issues = Vec::new();
    let mut over = |what, panel, needed: f64, available: f64| {
        if needed > available {
            issues.push(Issue::Overflow {
                what,
                panel,
                needed,
                available,
            });
        }
    };
    if let Some(width) = width(text, resolved.title()) {
        over(Element::FigureTitle, None, width, inside.width);
    }
    for (i, (laid, area)) in resolved
        .panels()
        .iter()
        .zip(resolved.plot_areas())
        .enumerate()
    {
        if let Some(width) = width(text, laid.title.as_ref()) {
            over(Element::PanelTitle, Some(i), width, area.width());
        }
        for placed in laid.axes.iter().filter(|a| a.plan.shown && a.plan.labeled) {
            if let Some(width) = width(text, placed.labels.title()) {
                let length = if axis::across(placed.labels.orient()) {
                    area.width()
                } else {
                    area.height()
                };
                over(Element::AxisTitle, Some(i), width, length);
            }
        }
        let room = kurbo::Size::new(
            area.width() - 2.0 * INSET * em,
            area.height() - 2.0 * INSET * em,
        );
        for (block, _) in &laid.insets {
            legend(&mut over, block, Some(i), room);
        }
    }
    for (block, _) in resolved.legends() {
        legend(&mut over, block, None, inside);
    }
    issues
}

/// A legend `block` overflowing `room`: by its width, else by its height.
fn legend(
    over: &mut impl FnMut(Element, Option<usize>, f64, f64),
    block: &Block,
    panel: Option<usize>,
    room: kurbo::Size,
) {
    let size = block.size();
    if size.width > room.width {
        over(Element::Legend, panel, size.width, room.width);
    } else {
        over(Element::Legend, panel, size.height, room.height);
    }
}

/// The advance width of `key`'s label, when there is one and typst shaped
/// it.
fn width(text: &text::Cache, key: Option<&Key>) -> Option<f64> {
    Some(text.get(key?)?.ok()?.width())
}

/// Labels of `cache` (a fresh one, holding the figure's labels alone) with
/// glyphs the fonts lack or markup typst rejects, by label.
fn labels(cache: &text::Cache) -> Vec<Issue> {
    let mut found: Vec<(String, Issue)> = cache
        .entries()
        .filter_map(|(key, shaped)| {
            let markup = key.markup();
            let issue = match shaped {
                Ok(shaped) if shaped.missing() > 0 => Issue::MissingGlyphs {
                    text: plain(markup),
                    count: shaped.missing(),
                },
                Ok(_) | Err(text::Error::Font { .. }) => return None,
                Err(text::Error::Markup { message, .. }) => Issue::Markup {
                    text: markup.to_string(),
                    message: message.clone(),
                },
                Err(error @ text::Error::Pages { .. }) => Issue::Markup {
                    text: markup.to_string(),
                    message: error.to_string(),
                },
            };
            Some((markup.to_string(), issue))
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found.into_iter().map(|(_, issue)| issue).collect()
}

/// A label's text: one literal span's characters, any other markup as it
/// is.
fn plain(markup: &str) -> String {
    let Some(inner) = markup.strip_prefix("#\"").and_then(|m| m.strip_suffix('"')) else {
        return markup.to_string();
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(c) => out.push(c),
                None => return markup.to_string(),
            },
            // A quote inside: more than one literal.
            '"' => return markup.to_string(),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_labels_read_as_their_text_and_markup_as_it_is() {
        assert_eq!(plain(r#"#"漢字""#), "漢字");
        assert_eq!(plain(r#"#"a \"b\" \\ c\n""#), "a \"b\" \\ c\n");
        let spans = r##"#"Wheel: "#text(fill: rgb("#c44e52"), "RL")"##;
        assert_eq!(plain(spans), spans);
        assert_eq!(plain("$x^2$"), "$x^2$");
    }
}
