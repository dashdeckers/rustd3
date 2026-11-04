//! Legends of a scatter's numeric channels: [`sizes()`], dots of the radii
//! a size handle gives its ticks, and [`colors()`], dots of the colors a
//! continuous color handle gives its ticks (matplotlib's
//! `legend_elements`). Both are [`Guide`](crate::legend::Guide)s, so they
//! go into a figure slot or a panel corner like any legend; their entries
//! are d3's ticks of the resolved scale, labelled as an axis labels them in
//! the figure's locale.
//!
//! ```
//! use rustd3_core::{scale, scheme};
//! use rustd3_figure::Figure;
//! use rustd3_figure::helpers::legend;
//! use rustd3_figure::legend::Slot;
//! use rustd3_figure::mark::dots;
//!
//! let mut fig = Figure::new();
//! let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
//! let heat = fig.scale(scale::sequential([1.0, 100.0], scheme::viridis));
//! let radius = fig.scale(scale::sqrt().domain([1.0, 100.0]).range([2.0, 12.0]));
//! fig.panel(0, 0).mark(
//!     dots([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y)
//!         .fill_by(heat, [20.0, 45.0, 80.0])
//!         .size_by(radius, [10.0, 50.0, 100.0]),
//! );
//! fig.legend(Slot::Right, legend::sizes(radius))
//!     .legend(Slot::Right, legend::colors(heat));
//! ```

mod colors;
mod sizes;

use rustd3_core::scale::Ticks;
use rustd3_core::ticks;

use crate::figure::Resolved;
use crate::scale::{Kind, ScaleRef};

pub use colors::{Colors, colors};
pub use sizes::{Sizes, sizes};

/// The values and labels of a legend's entries over `handle`'s resolved
/// scale: its about `count` ticks, or `values` with the ticks' step, kept
/// inside its domain, labelled by the scale's tick format in the figure's
/// locale.
fn entries<S>(
    handle: ScaleRef<S>,
    count: usize,
    values: Option<&[f64]>,
    resolved: &Resolved,
) -> Vec<(f64, String)>
where
    S: Kind<Domain = f64> + Ticks<Domain = f64>,
{
    let Some(scale) = resolved.get_scale(handle) else {
        return Vec::new();
    };
    let set = Ticks::tick_set(scale, count);
    let chosen = values.map_or_else(|| set.get_values().to_vec(), <[f64]>::to_vec);
    let inside: Vec<f64> = match Kind::ends(scale) {
        Some([a, b]) => {
            let (low, high) = (a.min(b), a.max(b));
            chosen
                .into_iter()
                .filter(|v| (low..=high).contains(v))
                .collect()
        }
        None => chosen,
    };
    let set: ticks::Set<f64, <S as Ticks>::Step> = set.with_values(inside);
    let labels = Ticks::labels(scale, &set, resolved.locale());
    set.get_values().iter().copied().zip(labels).collect()
}
