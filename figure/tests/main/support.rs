//! What figure tests share: [`figures_equal`].

use rustd3_figure::kurbo::Size;
use rustd3_figure::list;
use rustd3_figure::{Figure, Held, Target};

use crate::figure::cache;
#[cfg(feature = "raster")]
use crate::raster::vello::{diff, whole};

/// Whether `a` and `b` draw the same at `size` for vector output: equal
/// list dumps (glyph ids included) and, with `raster`, renders of the two
/// lists at 1× and 2× (single-threaded at the baseline SIMD level) no
/// channel more than 1/255 apart; the first difference otherwise.
pub fn figures_equal(a: &Figure<'_>, b: &Figure<'_>, size: Size) -> Result<(), String> {
    let draw =
        |fig: &Figure<'_>| fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
    let (a, b) = (draw(a), draw(b));
    let options = list::dump::Options::default().glyphs(true);
    let (dump_a, dump_b) = (list::dump(&a, options), list::dump(&b, options));
    if let Some((x, y)) = dump_a.lines().zip(dump_b.lines()).find(|(x, y)| x != y) {
        return Err(format!("dumps differ:\n{x}\n{y}"));
    }
    if dump_a.lines().count() != dump_b.lines().count() {
        return Err("dumps differ in length".to_string());
    }
    #[cfg(feature = "raster")]
    for scale in [1.0, 2.0] {
        let d = diff(&whole(&a, scale), &whole(&b, scale));
        if d.max() > 1 {
            return Err(format!(
                "renders at {scale}× differ by {}/255 at {:?}",
                d.max(),
                d.worst_tile()
            ));
        }
    }
    Ok(())
}
