//! tiny-skia raster time for the wheel figure, as
//! iced's software renderer pays it on every redraw.
use dlist::shape::Shaper;
use dlist::{skia, wheels};
use std::time::Instant;
fn main() {
    let samples = wheels::simulate();
    let mut l = wheels::figure(&samples, false).draw([960.0, 760.0]);
    Shaper::new().shape(&mut l);
    let mut o = dlist::outline::Outlines::default();
    for s in [1.0, 2.0] {
        let best = (0..3)
            .map(|_| {
                let t = Instant::now();
                std::hint::black_box(skia::render(&l, s, &mut o));
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::MAX, f64::min);
        println!("tiny-skia whole figure at {s}x: {best:.1} ms");
    }
}
