//! Option B: sharing follows the layout tree.

use figmodel::tree::{self, Resolve::*, hconcat, layer};
use figmodel::tree::{vconcat, view};

fn main() {
    let v = || view().line([(0.0, 1.0), (1.0, 2.0)]);

    // Twin axes: one layer, y independent.
    let twin = layer([v(), v()]).resolve_y(Independent);
    println!("layer, y independent: {:?}", tree::count(&twin));

    // Rows share y, the column shares x.
    let row = || hconcat([v(), v()]).resolve_y(Shared);
    let t = vconcat([row(), row()]).resolve_x(Shared);
    println!("rows share y, outer x: {:?}", tree::count(&t));

    // Rows share x and y, the column shares x.
    let row = || {
        hconcat([v(), v()])
            .resolve_x(Shared)
            .resolve_y(Shared)
    };
    let t = vconcat([row(), row()]).resolve_x(Shared);
    println!("rows share x+y, outer x: {:?}", tree::count(&t));

    // Columns share x, the row shares y.
    let col = || vconcat([v(), v()]).resolve_x(Shared);
    let t = hconcat([col(), col()]).resolve_y(Shared);
    println!("cols share x, outer y: {:?}", tree::count(&t));

    let fig = tree::build(&t);
    println!("compiles to option A: {} panels", fig_panels(&fig));
}

fn fig_panels(fig: &figmodel::fig::Figure) -> usize {
    let r = fig.resolve([1.0, 1.0], &Default::default());
    r.rects.len()
}
