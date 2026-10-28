//! Value labels as marks: [`bars()`], matplotlib's `bar_label`, the value
//! at each bar's end; and [`pie()`], one label per slice at its arc's
//! centroid, inside the slice, so a pie reserves no room for them.
//!
//! Labels are typst text at the axis-label size (12 pt at notebook) in the
//! theme's ink, numbers written in the figure's
//! [`Locale`](rustd3_core::locale::Locale): the neutral default writes
//! U+2212 for minus, where matplotlib writes an ASCII `-`. They sit by the
//! line box their size gives (the text face's ascender above the first
//! baseline, its descender below the last), as tick labels do, and take
//! part in no collision pass. Like every mark they are clipped to the plot
//! area.
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::Figure;
//! use rustd3_figure::helpers::labels;
//! use rustd3_figure::mark::bar_y;
//!
//! let rows = [("a", 3.0), ("b", -1.0), ("c", 2.5)];
//! let mut fig = Figure::new();
//! let x = fig.scale(scale::band(["a", "b", "c"]));
//! let y = fig.fit(scale::linear());
//! fig.panel(0, 0)
//!     .mark(bar_y(rows, x, y))
//!     .mark(labels::bars(&rows, x, y).format(".1f".parse()?));
//! # Ok::<(), rustd3_core::format::InvalidFormat>(())
//! ```

mod bars;
mod pie;

use kurbo::Point;

use crate::list::List;
use crate::mark::Ctx;
use crate::text::{Anchor, Key, Text, anchor};
use crate::theme::Theme;

pub use bars::{Bars, bars};
pub use pie::{Pie, pie};

/// px per pt: the CSS ratio.
const PX_PER_PT: f64 = 4.0 / 3.0;

/// Which edge of a label's line box sits at the point it is drawn at.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Edge {
    /// Its bottom: the label stands above the point.
    Bottom,
    /// Its top: the label hangs below the point.
    Top,
    /// Its middle: the label is centred on the point.
    Middle,
}

/// A label's key: `text` at the axis-label size, its lines centred, in
/// `theme`'s ink.
fn key(text: String, theme: &Theme) -> Key {
    let style = theme.note(anchor::Horizontal::Middle, theme.get_ink());
    Key::new(&Text::from(text), &style)
}

/// Appends the label of `key` centred across `at.x`, its line box's `edge`
/// at `at.y`; nothing when typst rejected it.
fn place(key: &Key, at: Point, edge: Edge, ctx: &Ctx<'_>, list: &mut List) {
    let Some(shaped) = ctx.label(key) else {
        return;
    };
    let [ascender, descender] = ctx.line(ctx.theme().label_text());
    let last = shaped.last();
    let baseline = match edge {
        Edge::Bottom => at.y - descender - last,
        Edge::Top => at.y + ascender,
        Edge::Middle => at.y + (ascender - descender - last) / 2.0,
    };
    let anchor = Anchor {
        h: anchor::Horizontal::Middle,
        v: anchor::Vertical::Baseline,
    };
    shaped.place(list, Point::new(at.x, baseline), anchor, 0.0);
}
